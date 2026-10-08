//! Frozen parent-reached supervision. Actual accepted alternatives are rejected
//! explicitly in this pilot if their token sequence differs from the canonical one.
use super::*;

pub(super) const OBJECTIVE: &str = "mean all noncomplete-episode parent-reached first-divergence CE + 1.0 * full accepted-trajectory CE (equal successful episode/nonempty phase/position); no B8 sampling; parent reachability only";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Trajectory {
    index: usize,
    id: String,
    generated_ids: Vec<u32>,
    decoded: String,
    eos: bool,
    accepted_complete: bool,
    first_divergence: Option<usize>,
    saved_generation_sha256: String,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Term {
    pub(super) index: usize,
    pub(super) position: usize,
    pub(super) component: usize,
    pub(super) target: u32,
    phase: usize,
    weight_denominator: usize,
    pub(super) weight: f64,
    pub(super) parent_actual_prefix_ids: Vec<u32>,
}
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    schema: String,
    objective: String,
    pub(super) canonical_reference: ReferencePlan,
    trajectories: Vec<Trajectory>,
    pub(super) terms: Vec<Term>,
}
fn classify(
    targets: &[u32],
    generated: &[u32],
    prefixes: &[Vec<u32>],
    chosen: &[u32],
    eos: u32,
    reported_eos: bool,
    accepted_member: bool,
) -> Result<(bool, Option<usize>)> {
    replay_require(
        !targets.is_empty()
            && targets.last() == Some(&eos)
            && !generated.is_empty()
            && generated.len() <= 32
            && generated.len() == prefixes.len()
            && generated.len() == chosen.len(),
        "frontier trajectory lengths/EOS target differ",
    )?;
    for (t, ((prefix, &chosen), &token)) in prefixes.iter().zip(chosen).zip(generated).enumerate() {
        replay_require(
            prefix.as_slice() == &generated[..t] && chosen == token,
            "saved emitted prefix/prediction chain differs",
        )?;
    }
    let has_eos = generated.last() == Some(&eos);
    replay_require(
        has_eos == reported_eos
            && !generated[..generated.len() - usize::from(has_eos)].contains(&eos),
        "saved trajectory EOS placement/flag differs",
    )?;
    let complete = has_eos && accepted_member;
    if complete {
        replay_require(generated==targets,"accepted alternative token sequence unsupported by this canonical-equality frontier pilot; never relabel an accepted trajectory")?;
        return Ok((true, None));
    }
    let first=generated.iter().zip(targets).position(|(a,b)|a!=b).ok_or_else(||bad("noncomplete trajectory has no observed canonical divergence; a censored matching prefix is not a training decision"))?;
    replay_require(
        generated[..first] == targets[..first],
        "frontier preceding prefix differs",
    )?;
    Ok((false, Some(first)))
}
fn trajectory(
    index: usize,
    row: &ReferenceRow,
    saved: &Value,
    answers: &FrozenAnswers,
    tok: &ByteBpeTokenizer,
    eos: u32,
) -> Result<Trajectory> {
    let generated: Vec<u32> = serde_json::from_value(saved["generated_ids"].clone())?;
    let generation = saved["generation"]
        .as_array()
        .ok_or_else(|| bad("saved own-prefix trajectory absent"))?;
    let reported_eos = saved["eos"]
        .as_bool()
        .ok_or_else(|| bad("saved EOS flag absent"))?;
    let canonical = saved["canonical"]
        .as_array()
        .ok_or_else(|| bad("canonical trace absent"))?;
    let prefixes = generation
        .iter()
        .map(|s| {
            serde_json::from_value::<Vec<u32>>(s["actual_prefix_ids"].clone()).map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;
    let chosen = generation
        .iter()
        .map(|s| {
            let value = s["pool"]["summary"]["chosen_token_id"]
                .as_u64()
                .ok_or_else(|| bad("saved winner absent"))?;
            u32::try_from(value).map_err(Into::into)
        })
        .collect::<Result<Vec<_>>>()?;
    replay_require(!generated.is_empty(), "empty own-prefix trajectory")?;
    let actual_eos = generated.last() == Some(&eos);
    let decoded = tok.decode(&generated[..generated.len() - usize::from(actual_eos)]);
    replay_require(
        saved["decoded"] == decoded,
        "saved decoded trajectory differs from actual emitted tokens",
    )?;
    let (complete, first) = classify(
        &row.targets,
        &generated,
        &prefixes,
        &chosen,
        eos,
        reported_eos,
        answers.accepts(&decoded),
    )?;
    replay_require(
        saved["complete"] == complete,
        "saved complete flag differs from EOS and accepted membership",
    )?;
    for (t, step) in generation.iter().enumerate() {
        let pool = &step["pool"]["summary"];
        replay_require(
            step["continuation"]["actual_prefix_tokens"] == t
                && pool["legal_generate_actions"] == 4096
                && pool["copy_actions"] == row.copy_ids.len(),
            "own-prefix step support/prefix count differs",
        )?;
        if complete || first.is_some_and(|f| t <= f) {
            replay_require(
                canonical
                    .get(t)
                    .is_some_and(|c| c["native"]["pool"]["summary"] == *pool),
                "reached own-prefix pool differs from same canonical-prefix pool",
            )?;
        }
    }
    Ok(Trajectory {
        index,
        id: row.id.clone(),
        generated_ids: generated,
        decoded,
        eos: reported_eos,
        accepted_complete: complete,
        first_divergence: first,
        saved_generation_sha256: sha256_bytes(&serde_json::to_vec(&saved["generation"])?),
    })
}
fn derive_terms(reference: &ReferencePlan, trajectories: &[Trajectory]) -> Result<Vec<Term>> {
    replay_require(
        reference.rows.len() == trajectories.len(),
        "trajectory population differs",
    )?;
    let failures = trajectories.iter().filter(|t| !t.accepted_complete).count();
    let mut success = reference.clone();
    for (row, t) in success.rows.iter_mut().zip(trajectories) {
        replay_require(
            row.index == t.index && row.id == t.id,
            "trajectory row identity differs",
        )?;
        row.eligible.fill(t.accepted_complete);
    }
    reference_weights(&mut success.rows)?;
    replay_require(failures > 0, "no noncomplete frontier population")?;
    let mut terms = Vec::new();
    // Canonical row order within each component is also the gradient/scorer order.
    for t in trajectories {
        if t.accepted_complete {
            continue;
        }
        let position = t
            .first_divergence
            .ok_or_else(|| bad("noncomplete frontier absent"))?;
        let row = &reference.rows[t.index];
        let target = *row
            .targets
            .get(position)
            .ok_or_else(|| bad("frontier target absent"))?;
        replay_require(
            position < t.generated_ids.len()
                && t.generated_ids[..position] == row.targets[..position]
                && t.generated_ids[position] != target,
            "frontier is not the first reached divergence",
        )?;
        terms.push(Term {
            index: t.index,
            position,
            component: 0,
            target,
            phase: row.phases[position],
            weight_denominator: failures,
            weight: 1. / failures as f64,
            parent_actual_prefix_ids: t.generated_ids[..position].to_vec(),
        });
    }
    for t in trajectories {
        if !t.accepted_complete {
            continue;
        }
        let row = &success.rows[t.index];
        replay_require(
            t.first_divergence.is_none() && t.generated_ids == row.targets,
            "successful actual sequence cannot be relabeled",
        )?;
        for (position, &target) in t.generated_ids.iter().enumerate() {
            terms.push(Term {
                index: t.index,
                position,
                component: 1,
                target,
                phase: row.phases[position],
                weight_denominator: row.weight_denominators[position],
                weight: row.weights[position],
                parent_actual_prefix_ids: t.generated_ids[..position].to_vec(),
            });
        }
    }
    for component in 0..2 {
        replay_require(
            (terms
                .iter()
                .filter(|t| t.component == component)
                .map(|t| t.weight)
                .sum::<f64>()
                - 1.)
                .abs()
                < 1e-12,
            "frontier component mass differs from one",
        )?;
    }
    Ok(terms)
}
pub(super) fn build(c: &ReferencePrepareConfig) -> Result<(Plan, Value)> {
    let (reference, legacy_counts) = build_reference_plan(c)?;
    let labels: Labels = serde_json::from_slice(&fs::read(&c.training_labels)?)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&fs::read(
        c.reference_root
            .join("checkpoint-0000/native/tokenizer.json"),
    )?)
    .ok_or_else(|| bad("frontier tokenizer absent"))?;
    let summary = read(&c.reference_root.join("development-0000.json"))?;
    let refs = summary["rows"]
        .as_array()
        .ok_or_else(|| bad("frontier initial row references absent"))?;
    let mut trajectories = Vec::new();
    for (index, (row, label)) in reference.rows.iter().zip(&labels.cases).enumerate() {
        let saved = reference_saved_row(&c.reference_root, &refs[index])?;
        let observed = trajectory(
            index,
            row,
            &saved,
            &label.answers,
            &tok,
            reference.eos_token_id,
        )?;
        replay_require(
            refs[index]["generated_ids"] == json!(observed.generated_ids)
                && refs[index]["complete"] == observed.accepted_complete
                && refs[index]["eos"] == observed.eos,
            "saved trajectory summary differs",
        )?;
        trajectories.push(observed);
    }
    let terms = derive_terms(&reference, &trajectories)?;
    let frontier = terms.iter().filter(|t| t.component == 0).count();
    let successes = trajectories.iter().filter(|t| t.accepted_complete).count();
    let reference_positions = terms.iter().filter(|t| t.component == 1).count();
    replay_require(
        frontier == 504 && successes == 8 && reference_positions == 84,
        "pinned parent reached population differs from declared504/8/84",
    )?;
    let mut phase_counts = [[0usize; 3]; 2];
    let mut phase_weights = [[0f64; 3]; 2];
    let mut eos_counts = [0usize; 2];
    for t in &terms {
        phase_counts[t.component][t.phase] += 1;
        phase_weights[t.component][t.phase] += t.weight;
        eos_counts[t.component] += usize::from(t.target == reference.eos_token_id);
    }
    let counts = json!({"episodes":reference.rows.len(),"frontier_positions":frontier,"successful_episodes":successes,"success_positions":reference_positions,
        "total_terms":terms.len(),"component_weight_sums":[1.,1.],"phase_counts":phase_counts,"phase_weights":phase_weights,"eos_counts":eos_counts,
        "canonical_reference_diagnostic":legacy_counts,"all_prefixes_reached_by_parent":true,"candidate_reachability":"not asserted",
        "alternative_answer_scope":"explicit rejection of accepted noncanonical token sequence; no silent replacement"});
    Ok((
        Plan {
            schema: "uor-r4.reached-frontier-plan/1".into(),
            objective: OBJECTIVE.into(),
            canonical_reference: reference,
            trajectories,
            terms,
        },
        counts,
    ))
}
pub(super) fn components(plan: &Plan) -> Result<(ReferencePlan, ReferencePlan)> {
    let rebuilt = derive_terms(&plan.canonical_reference, &plan.trajectories)?;
    replay_require(
        serde_json::to_vec(&rebuilt)? == serde_json::to_vec(&plan.terms)?,
        "frontier eligibility/weight/prefix reconstruction differs",
    )?;
    let mut masks = [
        plan.canonical_reference.clone(),
        plan.canonical_reference.clone(),
    ];
    for mask in &mut masks {
        for row in &mut mask.rows {
            row.eligible.fill(false);
            row.weights.fill(0.);
            row.weight_denominators.fill(0);
        }
    }
    let mut seen = BTreeSet::new();
    for t in &plan.terms {
        replay_require(
            t.component < 2 && seen.insert((t.index, t.position)),
            "frontier components overlap or contain duplicate positions",
        )?;
        let row = masks[t.component]
            .rows
            .get_mut(t.index)
            .ok_or_else(|| bad("frontier term row absent"))?;
        replay_require(
            row.targets.get(t.position) == Some(&t.target)
                && row.phases.get(t.position) == Some(&t.phase)
                && row.targets.get(..t.position) == Some(t.parent_actual_prefix_ids.as_slice()),
            "frontier term canonical-equality admission differs",
        )?;
        row.eligible[t.position] = true;
        row.weights[t.position] = t.weight;
        row.weight_denominators[t.position] = t.weight_denominator;
    }
    let [task, success] = masks;
    Ok((task, success))
}
pub(super) fn load(a: &Args, parent: &ContinuationParent, eps: &[Episode]) -> Result<Option<Plan>> {
    if !a.reached_frontier_objective {
        return Ok(None);
    }
    reference_replay_settings(a)?;
    let c = a
        .reference_replay
        .as_ref()
        .ok_or_else(|| bad("frontier configuration absent"))?;
    report_output::verify(&c.plan_root)?;
    replay_require(
        sha256_file(&c.plan_root.join("report.json"))? == c.expected_plan_report_sha256
            && sha256_file(&c.plan_root.join("manifest.json"))? == c.expected_plan_manifest_sha256,
        "frontier preparation seal differs",
    )?;
    let report = read(&c.plan_root.join("report.json"))?;
    let bytes = fs::read(c.plan_root.join("plan.json"))?;
    replay_require(
        report["schema"] == "uor-r4.reached-frontier-preparation/1"
            && report["status"] == "COMPLETED"
            && report["plan_sha256"] == sha256_bytes(&bytes),
        "frontier preparation identity differs",
    )?;
    let plan: Plan = serde_json::from_slice(&bytes)?;
    let (mut rebuilt, _) = build(&ReferencePrepareConfig {
        reference_root: c.reference_root.clone(),
        training_inputs: a.training_inputs.clone(),
        training_labels: a.training_labels.clone(),
        out: a.out.clone(),
        maximum_report_bytes: 64 << 20,
    })?;
    rebuilt.canonical_reference.reference_root = plan.canonical_reference.reference_root.clone();
    replay_require(
        serde_json::to_vec(&rebuilt)? == serde_json::to_vec(&plan)?,
        "frontier plan authenticated reconstruction differs",
    )?;
    replay_require(
        plan.schema == "uor-r4.reached-frontier-plan/1"
            && plan.objective == OBJECTIVE
            && plan.canonical_reference.source_binding == parent.binding
            && plan.canonical_reference.generate_sha256 == parent.generate_sha256
            && plan.canonical_reference.rows.len() == eps.len(),
        "frontier parent/model binding differs",
    )?;
    for (row, e) in plan.canonical_reference.rows.iter().zip(eps) {
        let copy = e
            .views
            .iter()
            .flatten()
            .flat_map(|v| v.emitted_token_ids().iter().copied())
            .collect::<BTreeSet<_>>();
        replay_require(
            row.id == e.packet.id
                && row.targets == e.target
                && row.packet_sha256 == sha256_bytes(&serde_json::to_vec(&e.packet)?)
                && row.copy_ids.iter().copied().collect::<BTreeSet<_>>() == copy,
            "frontier live packet/target/Copy binding differs",
        )?;
    }
    components(&plan)?;
    write(a, "frontier-plan.json", &serde_json::to_value(&plan)?)?;
    write(
        a,
        "reference-plan.json",
        &serde_json::to_value(&plan.canonical_reference)?,
    )?;
    Ok(Some(plan))
}
pub(super) fn successful_indices(plan: &Plan) -> Vec<usize> {
    plan.trajectories
        .iter()
        .filter(|t| t.accepted_complete)
        .map(|t| t.index)
        .collect()
}
pub(super) fn read_plan(a: &Args) -> Result<Plan> {
    Ok(serde_json::from_value(read(
        &a.out.join("frontier-plan.json"),
    )?)?)
}
pub(super) fn outcomes(
    a: &Args,
    plan: &Plan,
    before: &Value,
    after: &Value,
    verify_parent: bool,
) -> Result<Value> {
    let old = before["rows"]
        .as_array()
        .ok_or_else(|| bad("frontier initial rows absent"))?;
    let new = after["rows"]
        .as_array()
        .ok_or_else(|| bad("frontier final rows absent"))?;
    replay_require(
        old.len() == plan.trajectories.len() && new.len() == old.len(),
        "frontier endpoint population differs",
    )?;
    let mut losses = [[0f64; 2]; 2];
    let mut rows = Vec::new();
    let mut cached = BTreeMap::new();
    for term in &plan.terms {
        if let std::collections::btree_map::Entry::Vacant(entry) = cached.entry(term.index) {
            let initial = reference_saved_row(&a.out, &old[term.index])?;
            let final_row = reference_saved_row(&a.out, &new[term.index])?;
            if verify_parent {
                let t = &plan.trajectories[term.index];
                replay_require(
                    initial["generated_ids"] == json!(t.generated_ids)
                        && initial["decoded"] == t.decoded
                        && initial["complete"] == t.accepted_complete
                        && initial["eos"] == t.eos
                        && sha256_bytes(&serde_json::to_vec(&initial["generation"])?)
                            == t.saved_generation_sha256,
                    "fresh initial actual trajectory differs from frozen donor",
                )?;
            }
            entry.insert((initial, final_row));
        }
        let (left, right) = &cached[&term.index];
        let ce = [
            reference_native_loss(&left["canonical"][term.position], term.target)?,
            reference_native_loss(&right["canonical"][term.position], term.target)?,
        ];
        for endpoint in 0..2 {
            losses[term.component][endpoint] += term.weight * ce[endpoint];
        }
        rows.push(json!({"term":term,"native_losses_before_after":ce}));
    }
    Ok(
        json!({"objective":OBJECTIVE,"frontier_losses_before_after":losses[0],"success_losses_before_after":losses[1],
        "combined_losses_before_after":[losses[0][0]+losses[1][0],losses[0][1]+losses[1][1]],"terms":rows,
        "all_terms_parent_reached":true,"candidate_reachability":"not implied by fixed-objective evaluation","legacy398":"diagnostic only"}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    fn trace(ids: &[u32]) -> (Vec<Vec<u32>>, Vec<u32>) {
        (
            (0..ids.len()).map(|t| ids[..t].to_vec()).collect(),
            ids.to_vec(),
        )
    }
    #[test]
    fn frontier_requires_actual_first_divergence_and_rejects_censoring() -> Result<()> {
        let ids = vec![3, 7, 8];
        let (p, c) = trace(&ids);
        assert_eq!(
            classify(&[3, 4, 9], &ids, &p, &c, 9, false, false)?,
            (false, Some(1))
        );
        let (p, c) = trace(&[3]);
        assert!(classify(&[3, 4, 9], &[3], &p, &c, 9, false, false).is_err());
        let mut broken = trace(&ids).0;
        broken[1] = vec![4];
        assert!(classify(&[3, 4, 9], &ids, &broken, &ids, 9, false, false).is_err());
        Ok(())
    }
    #[test]
    fn frontier_eos_and_accepted_alternative_are_explicit_boundaries() -> Result<()> {
        let ids = vec![3, 4, 9];
        let (p, c) = trace(&ids);
        assert_eq!(classify(&ids, &ids, &p, &c, 9, true, true)?, (true, None));
        assert!(classify(&ids, &ids, &p, &c, 9, false, true).is_err());
        let alternate = vec![7, 9];
        let (p, c) = trace(&alternate);
        assert!(classify(&ids, &alternate, &p, &c, 9, true, true).is_err());
        let early = vec![9, 3];
        let (p, c) = trace(&early);
        assert!(classify(&ids, &early, &p, &c, 9, false, false).is_err());
        Ok(())
    }
    fn weighted_fixture() -> Result<Plan> {
        let specs = [
            (vec![1, 2, 9], vec![0, 1, 2], false),
            (vec![3, 4, 5, 9], vec![0, 1, 1, 2], true),
            (vec![6, 9], vec![0, 2], true),
        ];
        let mut rows = Vec::new();
        let mut trajectories = Vec::new();
        for (index, (targets, phases, complete)) in specs.into_iter().enumerate() {
            let id = format!("row-{index}");
            trajectories.push(Trajectory {
                index,
                id: id.clone(),
                generated_ids: if complete { targets.clone() } else { vec![7] },
                decoded: "fixture".into(),
                eos: complete,
                accepted_complete: complete,
                first_divergence: if complete { None } else { Some(0) },
                saved_generation_sha256: "a".repeat(64),
            });
            rows.push(ReferenceRow {
                index,
                id,
                packet_sha256: "a".repeat(64),
                saved_row_sha256: "b".repeat(64),
                eligible: vec![true; targets.len()],
                weights: vec![0.; targets.len()],
                weight_denominators: vec![0; targets.len()],
                targets,
                copy_ids: vec![2, 4, 5],
                phases,
            });
        }
        let binding = serde_json::from_value(
            json!({"metadata_sha256":"a".repeat(64),"identity":{"tokenizer_sha256":"b".repeat(64),"parent_checkpoint_manifest_sha256":"c".repeat(64),"parent_model_sha256":"d".repeat(64),"parent_config_sha256":"e".repeat(64)}}),
        )?;
        let reference = ReferencePlan {
            schema: "uor-r4.supervised-reference-plan/1".into(),
            reference_root: "/fixture".into(),
            source_binding: binding,
            generate_sha256: "a".repeat(64),
            initial_receipt_sha256: "b".repeat(64),
            reference_report_sha256: REPLAY_REPORT_SHA.into(),
            reference_manifest_sha256: REPLAY_MANIFEST_SHA.into(),
            input_sha256: INPUT_SHA.into(),
            labels_sha256: LABEL_SHA.into(),
            eos_token_id: 9,
            rows,
        };
        let terms = derive_terms(&reference, &trajectories)?;
        Ok(Plan {
            schema: "uor-r4.reached-frontier-plan/1".into(),
            objective: OBJECTIVE.into(),
            canonical_reference: reference,
            trajectories,
            terms,
        })
    }
    #[test]
    fn frontier_weights_exclude_unreached_legacy_terms_and_balance_success_phases() -> Result<()> {
        let plan = weighted_fixture()?;
        let (task, success) = components(&plan)?;
        assert_eq!(plan.terms.len(), 7);
        assert_eq!(task.rows[0].weights, vec![1., 0., 0.]);
        assert!(task.rows[1..]
            .iter()
            .all(|r| r.weights.iter().all(|&w| w == 0.)));
        assert!(success.rows[0].weights.iter().all(|&w| w == 0.));
        assert_eq!(
            success.rows[1].weights,
            vec![1. / 6., 1. / 12., 1. / 12., 1. / 6.]
        );
        assert_eq!(success.rows[2].weights, vec![0.25, 0.25]);
        assert_eq!(
            plan.terms
                .last()
                .ok_or_else(|| bad("fixture term"))?
                .parent_actual_prefix_ids,
            vec![6]
        );
        for component in 0..2 {
            let mass = plan
                .terms
                .iter()
                .filter(|t| t.component == component)
                .map(|t| t.weight)
                .sum::<f64>();
            assert!((mass - 1.).abs() < 1e-12);
        }
        Ok(())
    }
    #[test]
    fn frontier_serialized_plan_reconstructs_and_rejects_term_tampering() -> Result<()> {
        let plan = weighted_fixture()?;
        let encoded = serde_json::to_vec(&plan)?;
        let mut restored: Plan = serde_json::from_slice(&encoded)?;
        components(&restored)?;
        restored.terms[0].weight = 0.5;
        assert!(components(&restored).is_err());
        let mut restored: Plan = serde_json::from_slice(&encoded)?;
        restored.terms[1].target += 1;
        assert!(components(&restored).is_err());
        let mut restored: Plan = serde_json::from_slice(&encoded)?;
        restored
            .terms
            .last_mut()
            .ok_or_else(|| bad("fixture term"))?
            .parent_actual_prefix_ids
            .clear();
        assert!(components(&restored).is_err());
        Ok(())
    }
}
