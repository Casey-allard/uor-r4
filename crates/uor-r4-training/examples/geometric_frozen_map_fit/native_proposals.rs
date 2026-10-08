//! One frozen parent-rooted discrete native-code optimization round. No Adam update.
use super::*;

pub(super) type Shadows = BTreeMap<String, Vec<f32>>;
pub(super) const BASIS: [&str; 6] = [
    "self_transition",
    "neighbor_transition",
    "self_root",
    "neighbor_root",
    "self_category",
    "neighbor_category",
];
#[derive(Clone, Debug, Deserialize, serde::Serialize, PartialEq)]
pub(super) struct Edit {
    pub(super) name: String,
    pub(super) index: usize,
    pub(super) before: i8,
    pub(super) after: i8,
}
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub(super) struct Proposal {
    pub(super) family: String,
    pub(super) name: String,
    pub(super) row: usize,
    rank_l1: f64,
    pub(super) selected_gradient: Vec<f32>,
    pub(super) direction: i8,
    pub(super) edits: Vec<Edit>,
    pub(super) blocked_coordinates: Vec<usize>,
    pub(super) status: String,
}
#[derive(Clone, Debug, Deserialize, serde::Serialize)]
pub(super) struct Term {
    pub(super) index: usize,
    pub(super) position: usize,
    pub(super) target: u32,
    pub(super) component: usize,
    pub(super) weight: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) parent_actual_prefix_ids: Option<Vec<u32>>,
}
pub(super) fn policy(reached: bool) -> Value {
    let mut result = json!({"schema":"uor-r4.native-code-proposals/1", "rounds":1,"maximum_candidates":18,
        "context":"one highest L1 combined-gradient [H,L,class,4] row per self/neighbor transition/root/category tensor; both signs",
        "other":"one highest absolute combined-gradient scalar each Potential, Generate unary/pair only, U; both signs",
        "rank_ties":"lexical parameter name then flattened row/index; candidate order frozen before any proposal loss",
        "quantum":"q=round(4*master); clamp(q +/- sign(gradient),-7,7); only changed coordinates become q_new/4; saturated coordinates retain exact master bits",
        "noops":"record and skip empty or duplicate native edits; never refill",
        "acceptance":"at most one minimum native combined CE, strict decrease > 1e-10*(1+abs(parent)); first candidate on equal losses; parent preferred otherwise",
        "objective":"same 119 task terms + 398 frozen reference terms, including two overlaps twice; lambda1; no half average",
        "optimizer_updates":0,"global_clipping_applied":false,"gradients":"parent ranking only",
        "scope":"finite native shared-operator neighborhood; scalar-only descent is not Context repair; native CE descent is not useful chat qualification"});
    if reached {
        result["objective"] = json!(frontier::OBJECTIVE);
        result["objective_terms"] = json!({"frontier":504,"success":84,"total":588});
    }
    result
}
fn code(v: f32) -> Result<i8> {
    replay_require(
        v.is_finite() && (-1.75..=1.75).contains(&v),
        "native proposal master outside legal Q4 range",
    )?;
    Ok((v * 4.).round() as i8)
}
pub(super) fn snapshot(params: &BTreeMap<String, Var>) -> Result<Shadows> {
    params
        .iter()
        .map(|(name, v)| {
            Ok((
                name.clone(),
                v.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            ))
        })
        .collect()
}
pub(super) fn same_bits(a: &Shadows, b: &Shadows) -> bool {
    a.len() == b.len()
        && a.iter().all(|(name, x)| {
            b.get(name).is_some_and(|y| {
                x.len() == y.len() && x.iter().zip(y).all(|(x, y)| x.to_bits() == y.to_bits())
            })
        })
}
pub(super) fn restore(params: &BTreeMap<String, Var>, parent: &Shadows) -> Result<()> {
    for (name, values) in parent {
        let v = params
            .get(name)
            .ok_or_else(|| bad("proposal restore parameter absent"))?;
        v.set(&Tensor::from_vec(values.clone(), v.shape(), v.device())?)?;
    }
    replay_require(
        same_bits(parent, &snapshot(params)?),
        "proposal parent bits not restored",
    )
}
pub(super) fn edited(parent: &Shadows, edits: &[Edit]) -> Result<Shadows> {
    let mut expected = parent.clone();
    let mut seen = BTreeSet::new();
    for edit in edits {
        replay_require(
            seen.insert((&edit.name, edit.index))
                && edit.before != edit.after
                && (-7..=7).contains(&edit.after),
            "invalid/duplicate native edit",
        )?;
        let value = expected
            .get_mut(&edit.name)
            .and_then(|v| v.get_mut(edit.index))
            .ok_or_else(|| bad("native edit coordinate absent"))?;
        replay_require(
            code(*value)? == edit.before && (edit.after - edit.before).abs() == 1,
            "native edit is not one parent-rooted quantum",
        )?;
        *value = f32::from(edit.after) * 0.25;
    }
    Ok(expected)
}
pub(super) fn apply_edits(
    params: &BTreeMap<String, Var>,
    parent: &Shadows,
    edits: &[Edit],
) -> Result<()> {
    replay_require(
        same_bits(parent, &snapshot(params)?),
        "proposal did not start from parent masters",
    )?;
    let expected = edited(parent, edits)?;
    for name in edits.iter().map(|e| &e.name).collect::<BTreeSet<_>>() {
        let v = params
            .get(name)
            .ok_or_else(|| bad("native edit Var absent"))?;
        v.set(&Tensor::from_vec(
            expected[name].clone(),
            v.shape(),
            v.device(),
        )?)?;
    }
    replay_require(
        same_bits(&expected, &snapshot(params)?),
        "unselected master bits changed",
    )
}
pub(super) fn attempt_restored<T>(
    params: &BTreeMap<String, Var>,
    parent: &Shadows,
    f: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let outcome = f();
    // Restore even after export/scoring/write failure. Restoration errors are fatal.
    match restore(params, parent) {
        Ok(()) => outcome,
        Err(error) => Err(bad(&format!(
            "native proposal restoration failed: {error}; attempted operation: {}",
            outcome
                .as_ref()
                .err()
                .map(ToString::to_string)
                .unwrap_or_else(|| "completed".into())
        ))),
    }
}
fn make_pair(
    family: &str,
    name: &str,
    row: usize,
    width: usize,
    rank: f64,
    parent: &[f32],
    gradient: &[f32],
) -> Result<Vec<Proposal>> {
    let mut out = Vec::new();
    for direction in [-1i8, 1] {
        let mut p = Proposal {
            family: family.into(),
            name: name.into(),
            row,
            rank_l1: rank,
            selected_gradient: gradient[row * width..(row + 1) * width].to_vec(),
            direction,
            edits: Vec::new(),
            blocked_coordinates: Vec::new(),
            status: "evaluate".into(),
        };
        for index in row * width..(row + 1) * width {
            let before = code(
                *parent
                    .get(index)
                    .ok_or_else(|| bad("proposal row outside tensor"))?,
            )?;
            let g = *gradient
                .get(index)
                .ok_or_else(|| bad("proposal gradient outside tensor"))?;
            replay_require(g.is_finite(), "nonfinite proposal gradient")?;
            let sign = if g > 0. {
                1
            } else if g < 0. {
                -1
            } else {
                0
            };
            let raw = before + direction * sign;
            let after = raw.clamp(-7, 7);
            if raw != after {
                p.blocked_coordinates.push(index);
            }
            if after != before {
                p.edits.push(Edit {
                    name: name.into(),
                    index,
                    before,
                    after,
                });
            }
        }
        if p.edits.is_empty() {
            p.status = "skip_native_noop".into();
        }
        out.push(p);
    }
    Ok(out)
}
pub(super) fn ranked_pair(
    family: &str,
    names: &[String],
    width: usize,
    parent: &Shadows,
    gradients: &Shadows,
) -> Result<Vec<Proposal>> {
    let mut best: Option<(String, usize, f64)> = None;
    for name in names {
        let values = parent
            .get(name)
            .ok_or_else(|| bad("proposal family absent"))?;
        let g = gradients
            .get(name)
            .ok_or_else(|| bad("proposal gradient family absent"))?;
        replay_require(
            values.len() == g.len() && !values.is_empty() && values.len() % width == 0,
            "proposal family shape differs",
        )?;
        for (row, chunk) in g.chunks_exact(width).enumerate() {
            replay_require(
                chunk.iter().all(|v| v.is_finite()),
                "nonfinite rank gradient",
            )?;
            let rank = chunk.iter().map(|v| f64::from(v.abs())).sum::<f64>();
            if best.as_ref().is_none_or(|(old, r, score)| {
                rank > *score || (rank == *score && (name, row) < (old, *r))
            }) {
                best = Some((name.clone(), row, rank));
            }
        }
    }
    let (name, row, rank) = best.ok_or_else(|| bad("empty native proposal group"))?;
    make_pair(
        family,
        &name,
        row,
        width,
        rank,
        &parent[&name],
        &gradients[&name],
    )
}
fn bank(
    params: &BTreeMap<String, Var>,
    parent: &Shadows,
    gradients: &BTreeMap<String, Tensor>,
) -> Result<Vec<Proposal>> {
    let mut g = Shadows::new();
    for (name, values) in parent {
        let values = match gradients.get(name) {
            Some(t) => t.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            None => vec![0.; values.len()],
        };
        g.insert(name.clone(), values);
    }
    let mut proposals = Vec::new();
    for family in BASIS {
        let name = format!("consumer.context.{family}");
        let dims = params
            .get(&name)
            .ok_or_else(|| bad("shared Context basis absent"))?
            .dims();
        replay_require(
            dims.len() == 4 && dims[3] == 4,
            "Context proposal needs [H,L,class,4]",
        )?;
        proposals.extend(ranked_pair(
            &format!("context.{family}"),
            &[name],
            4,
            parent,
            &g,
        )?);
    }
    for (family, names) in [
        (
            "potential",
            parent
                .keys()
                .filter(|s| s.starts_with("consumer.potential."))
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            "generate",
            vec!["generate.pair".into(), "generate.unary".into()],
        ),
        ("continuation", vec!["continuation.unary".into()]),
    ] {
        proposals.extend(ranked_pair(family, &names, 1, parent, &g)?);
    }
    let mut seen = BTreeSet::new();
    for p in &mut proposals {
        if !p.edits.is_empty() && !seen.insert(serde_json::to_vec(&p.edits)?) {
            p.status = "skip_duplicate_native_edits".into();
        }
    }
    replay_require(proposals.len() == 18, "native proposal inventory differs")?;
    Ok(proposals)
}
pub(super) fn save_ranking_gradients(
    a: &Args,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
) -> Result<Value> {
    let root = a.out.join("native-code-ranking-gradients");
    fs::create_dir(&root)?;
    let mut inventory = BTreeMap::new();
    for (name, var) in params {
        let eligible = BASIS
            .iter()
            .any(|family| name == &format!("consumer.context.{family}"))
            || name.starts_with("consumer.potential.")
            || [
                "generate.unary",
                "generate.pair",
                "continuation.unary",
                "cue.coefficients",
                "prefix.coefficients",
            ]
            .contains(&name.as_str());
        if !eligible {
            continue;
        }
        let values = match gradients.get(name) {
            Some(g) => g.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            None => vec![0.; var.elem_count()],
        };
        replay_require(
            values.len() == var.elem_count() && values.iter().all(|v| v.is_finite()),
            "rank gradient shape/finite mismatch",
        )?;
        let bytes = values
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect::<Vec<_>>();
        let file = format!("{name}.f32le");
        fs::write(root.join(&file), &bytes)?;
        inventory.insert(name.clone(),json!({"file":format!("native-code-ranking-gradients/{file}"),"shape":var.dims(),
            "bytes":bytes.len(),"sha256":sha256_bytes(&bytes),"missing_gradient_filled_zero":!gradients.contains_key(name)}));
    }
    write(a, "native-code-ranking-gradients.json", &json!(inventory))?;
    Ok(json!(inventory))
}
fn terms(eps: &[Episode], indices: &[usize], plan: &ReferencePlan) -> Result<Vec<Term>> {
    let mut out = Vec::new();
    for &index in indices {
        let e = eps
            .get(index)
            .ok_or_else(|| bad("proposal task episode absent"))?;
        let row = plan
            .rows
            .get(index)
            .ok_or_else(|| bad("proposal reference row absent"))?;
        let copy = row.copy_ids.iter().copied().collect();
        let weights = episode_loss_weights(&e.target, &copy, indices.len(), true, LossScope::All)?;
        for (position, (&target, &weight)) in e.target.iter().zip(&weights.weights).enumerate() {
            out.push(Term {
                index,
                position,
                target,
                component: 0,
                weight,
                parent_actual_prefix_ids: None,
            });
        }
    }
    for row in &plan.rows {
        for (position, &eligible) in row.eligible.iter().enumerate() {
            if eligible {
                out.push(Term {
                    index: row.index,
                    position,
                    target: row.targets[position],
                    component: 1,
                    weight: row.weights[position],
                    parent_actual_prefix_ids: None,
                });
            }
        }
    }
    replay_require(
        out.iter().filter(|t| t.component == 0).count() == 119
            && out.iter().filter(|t| t.component == 1).count() == 398,
        "native objective is not declared 119+398 terms",
    )?;
    for component in 0..2 {
        replay_require(
            (out.iter()
                .filter(|t| t.component == component)
                .map(|t| t.weight)
                .sum::<f64>()
                - 1.)
                .abs()
                < 1e-12,
            "native component weights not normalized",
        )?;
    }
    Ok(out)
}
pub(super) fn score(
    a: &Args,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    plan: &ReferencePlan,
    terms: &[Term],
    start: Instant,
) -> Result<Value> {
    score_with_capture(a, p, field, eps, plan, terms, start, None)
}
/// Capture only actual successful-trajectory executions, including source-bank
/// admission before caching. Other endpoints use the same numerical scorer.
pub(super) fn score_with_capture(
    a: &Args,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    plan: &ReferencePlan,
    terms: &[Term],
    start: Instant,
    capture_file: Option<&str>,
) -> Result<Value> {
    let clock = Instant::now();
    let mut captured = Vec::new();
    let mut calls = Vec::new();
    let protected = terms
        .iter()
        .filter(|t| t.component == 1)
        .map(|t| t.index)
        .collect::<BTreeSet<_>>();
    let bytes = field.to_bytes()?;
    let sha = sha256_bytes(&bytes);
    let mut generator = p.generator()?.with_continuation_field(BoundNativeBytes {
        bytes: &bytes,
        sha256: &sha,
    })?;
    // Admission binds Source, so this cache lives only for this candidate.
    let mut banks = BTreeMap::new();
    for term in terms {
        if let std::collections::btree_map::Entry::Vacant(entry) = banks.entry(term.index) {
            let capture = if capture_file.is_some() && protected.contains(&term.index) {
                Some(
                    generator.capture_context_decisions(
                        constrained_context::MAX_CAPTURE_EVENTS
                            .checked_sub(captured.len())
                            .ok_or_else(|| bad("Context capture budget exhausted"))?,
                    )?,
                )
            } else {
                None
            };
            entry.insert(generator.admit_bank(continuation_snapshot(&eps[term.index].packet)?)?);
            if let Some(capture) = capture {
                let first = captured.len();
                captured.extend(capture.finish()?);
                calls.push(json!({"kind":"bank_admission","index":term.index,"first":first,"count":captured.len()-first}));
            }
        }
    }
    let mut losses = [0.; 2];
    let mut rows = Vec::new();
    for term in terms {
        deadline(a, start)?;
        let e = &eps[term.index];
        replay_require(
            e.target.get(term.position) == Some(&term.target),
            "objective target/prefix mismatch",
        )?;
        let prefix = term
            .parent_actual_prefix_ids
            .as_deref()
            .unwrap_or(&e.target[..term.position]);
        replay_require(
            prefix == &e.target[..term.position],
            "native objective actual-prefix admission differs",
        )?;
        let capture = if capture_file.is_some() && term.component == 1 {
            Some(
                generator.capture_context_decisions(
                    constrained_context::MAX_CAPTURE_EVENTS
                        .checked_sub(captured.len())
                        .ok_or_else(|| bad("Context capture budget exhausted"))?,
                )?,
            )
        } else {
            None
        };
        let step = generator.step(&banks[&term.index], prefix)?;
        if let Some(capture) = capture {
            let first = captured.len();
            captured.extend(capture.finish()?);
            calls.push(json!({"kind":"native_step","index":term.index,"position":term.position,"prefix":prefix,"first":first,"count":captured.len()-first}));
        }
        let total = step.actions.summary.total_weight_q31;
        let mass = step
            .actions
            .token_masses
            .iter()
            .find(|m| m.token_id == term.target)
            .ok_or_else(|| bad("native objective full support missing"))?
            .weight_q31;
        replay_require(
            total > 0
                && mass > 0
                && mass <= total
                && step.actions.summary.legal_generate_actions == 4096
                && step.copy_token_ids == plan.rows[term.index].copy_ids,
            "native objective support/physical alias identity differs",
        )?;
        let ce = -(mass as f64 / total as f64).ln();
        losses[term.component] += term.weight * ce;
        rows.push(json!({"term":term,"target_mass":mass,"denominator":total,"ce":ce,
            "chosen":step.actions.summary.chosen_token_id,"pool":step.actions.summary,
            "factual_state":step.post_state.iter().map(|c|c.index()).collect::<Vec<_>>(),
            "generate_raw_scores_sha256":sha256_bytes(&serde_json::to_vec(&step.generate_raw_scores_q24)?),
            "continuation":continuation_witness(&step)?}));
    }
    let mut receipt = json!({"task":losses[0],"reference":losses[1],"combined":losses[0]+losses[1],"terms":rows,
        "source_binding":p.binding,"generate_sha256":p.generate_sha256,"continuation_sha256":sha,
        "native_calls":terms.len(),"elapsed_seconds":clock.elapsed().as_secs_f64()});
    if let Some(file) = capture_file {
        replay_require(
            !captured.is_empty()
                && protected.len() == 8
                && calls.iter().filter(|v| v["kind"] == "native_step").count() == 84,
            "protected Context capture population differs",
        )?;
        let path = a.out.join(file);
        let stream = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let remaining = a
            .maximum_report_bytes
            .checked_sub(size(&a.out)?.saturating_add(1 << 20))
            .ok_or_else(|| bad("Context capture report budget exhausted"))?;
        disk_floor(a)?;
        let mut writer = constrained_context::BudgetWriter {
            inner: io::BufWriter::new(stream),
            remaining,
        };
        serde_json::to_writer(
            &mut writer,
            &constrained_context::CapturedDecisions {
                calls,
                events: captured,
            },
        )?;
        std::io::Write::flush(&mut writer)?;
        disk_floor(a)?;
    }
    if a.categorical_action_learning {
        receipt["categorical_sha256"] = json!(p.bridge_sha256);
    }
    Ok(receipt)
}
pub(super) fn objective(value: &Value) -> Result<f64> {
    let v = value["combined"]
        .as_f64()
        .ok_or_else(|| bad("native objective absent"))?;
    replay_require(v.is_finite(), "nonfinite native objective")?;
    Ok(v)
}
fn select(parent: f64, values: &[Option<f64>]) -> Option<usize> {
    let threshold = 1e-10 * (1. + parent.abs());
    let mut best = None;
    let mut loss = parent - threshold;
    for (i, v) in values.iter().enumerate() {
        if let Some(v) = v {
            if v.is_finite() && *v < loss {
                loss = *v;
                best = Some(i);
            }
        }
    }
    best
}
fn native_codes(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    parent: &Shadows,
) -> Result<BTreeMap<String, Vec<i8>>> {
    let mut result = BTreeMap::new();
    for (prefix, file, families) in [
        (
            "consumer.context.",
            "consumer/context-q4.bin",
            uor_r4_integer::geometric_context_q4::FAMILY_NAMES.as_slice(),
        ),
        (
            "consumer.potential.",
            "consumer/potential-q4.bin",
            uor_r4_integer::geometric_potential_q4::FAMILY_NAMES.as_slice(),
        ),
    ] {
        let bytes = fs::read(p.native_directory.join(file))?;
        let names = families
            .iter()
            .map(|n| format!("{prefix}{n}"))
            .filter(|n| parent.contains_key(n))
            .collect::<Vec<_>>();
        let count = names.iter().map(|n| parent[n].len()).sum();
        let codes = uor_r4_integer::geometric_context_q4::unpack_coefficients(count, &bytes)?;
        let mut offset = 0;
        for name in names {
            let length = parent[&name].len();
            result.insert(name, codes[offset..offset + length].to_vec());
            offset += length;
        }
    }
    let generator = p.generator()?;
    let g = generator.generate_model();
    let mut unary = Vec::new();
    let mut pair = Vec::new();
    for lane in 0..g.energy().lanes() {
        for r in 0..120 {
            unary.push(g.energy().get_unary(lane, r)?);
        }
    }
    for edge in 0..g.energy().edges().len() {
        for left in 0..120 {
            for right in 0..120 {
                pair.push(g.energy().get_pair(edge, left, right)?);
            }
        }
    }
    result.insert("generate.unary".into(), unary);
    result.insert("generate.pair".into(), pair);
    result.insert(
        "generate.bias".into(),
        (0..g.vocab_size())
            .map(|i| g.token_bias(i))
            .collect::<std::result::Result<Vec<_>, _>>()?,
    );
    result.insert(
        "continuation.unary".into(),
        uor_r4_integer::geometric_context_q4::unpack_coefficients(
            parent["continuation.unary"].len(),
            field.packed_unary(),
        )?,
    );
    Ok(result)
}
pub(super) fn verify_codes(
    p: &ContinuationParent,
    field: &NativeContinuationField,
    expected: &Shadows,
) -> Result<()> {
    let wanted = expected
        .iter()
        .map(|(name, v)| {
            Ok((
                name.clone(),
                v.iter().map(|&v| code(v)).collect::<Result<Vec<_>>>()?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    replay_require(
        native_codes(p, field, expected)? == wanted,
        "independent native codes do not equal exact declared master edits",
    )
}
pub(super) fn run(
    a: &Args,
    l: &Loaded,
    weights: &ContinuationLearningWeights,
    params: &BTreeMap<String, Var>,
    gradients: &BTreeMap<String, Tensor>,
    initial: &ContinuationParent,
    initial_field: &NativeContinuationField,
    eps: &[Episode],
    indices: &[usize],
    plan: &ReferencePlan,
    start: Instant,
) -> Result<Value> {
    let parent = snapshot(params)?;
    let proposals = bank(params, &parent, gradients)?;
    let ranking_gradients = save_ranking_gradients(a, params, gradients)?;
    let terms = if a.reached_frontier_objective {
        let reached = frontier::read_plan(a)?;
        frontier::components(&reached)?;
        reached
            .terms
            .iter()
            .map(|t| Term {
                index: t.index,
                position: t.position,
                target: t.target,
                component: t.component,
                weight: t.weight,
                parent_actual_prefix_ids: Some(t.parent_actual_prefix_ids.clone()),
            })
            .collect::<Vec<_>>()
    } else {
        terms(eps, indices, plan)?
    };
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    write(
        a,
        "native-code-proposal-bank.json",
        &json!({"policy":policy(a.reached_frontier_objective),"proposals":proposals,
        "parent_master_identities":identities(params)?,"parent_receipt":initial.receipt,
        "ranking_gradients":ranking_gradients,"ranking_gradients_inventory_sha256":sha256_file(&a.out.join("native-code-ranking-gradients.json"))?,
        "objective_terms_sha256":sha256_file(&a.out.join("native-code-objective-terms.json"))?,
        "reference_plan_sha256":sha256_file(&a.out.join(if a.reached_frontier_objective {"frontier-plan.json"}else{"reference-plan.json"}))?,
        "training_inputs_sha256":INPUT_SHA,"training_labels_sha256":LABEL_SHA}),
    )?;
    verify_codes(initial, initial_field, &parent)?;
    let baseline = score(a, initial, initial_field, eps, plan, &terms, start)?;
    let expected = read(&a.out.join(if a.reached_frontier_objective {
        "frontier-initial-validation.json"
    } else {
        "reference-initial-validation.json"
    }))?;
    let baseline_loss = objective(&baseline)?;
    for (key, expected_key) in [
        (
            "task",
            if a.reached_frontier_objective {
                "frontier_losses_before_after"
            } else {
                "current_batch_losses_before_after"
            },
        ),
        (
            "reference",
            if a.reached_frontier_objective {
                "success_losses_before_after"
            } else {
                "reference_losses_before_after"
            },
        ),
        ("combined", "combined_losses_before_after"),
    ] {
        let x = baseline[key]
            .as_f64()
            .ok_or_else(|| bad("baseline component absent"))?;
        let y = expected[expected_key][0]
            .as_f64()
            .ok_or_else(|| bad("saved baseline component absent"))?;
        replay_require(
            (x - y).abs() <= 1e-10 * (1. + y.abs()),
            "proposal baseline differs from full native evaluation",
        )?;
    }
    write(a, "native-code-parent-objective.json", &baseline)?;
    let mut results = Vec::new();
    let mut losses = Vec::new();
    for (index, proposal) in proposals.iter().enumerate() {
        deadline(a, start)?;
        disk_floor(a)?;
        if proposal.status != "evaluate" {
            results.push(json!({"index":index,"status":proposal.status}));
            losses.push(None);
            continue;
        }
        let mut candidate = a.clone();
        candidate.out = a.out.join(format!("native-candidate-{index:02}"));
        let remaining = a
            .maximum_report_bytes
            .checked_sub(size(&a.out)?)
            .ok_or_else(|| bad("global proposal report cap"))?;
        replay_require(remaining > 2_097_152, "proposal report reserve exhausted")?;
        candidate.maximum_report_bytes = remaining;
        report_output::claim(&candidate.out)?;
        let outcome = attempt_restored(params, &parent, || {
            fs::copy(
                a.out.join("reference-plan.json"),
                candidate.out.join("reference-plan.json"),
            )?;
            if a.reached_frontier_objective {
                fs::copy(
                    a.out.join("frontier-plan.json"),
                    candidate.out.join("frontier-plan.json"),
                )?;
            }
            write(&candidate, "proposal.json", &json!(proposal))?;
            apply_edits(params, &parent, &proposal.edits)?;
            let checkpoint_clock = Instant::now();
            let (current, field, receipt) = joint_checkpoint(&candidate, 0, l, weights)?;
            let checkpoint_seconds = checkpoint_clock.elapsed().as_secs_f64();
            verify_codes(&current, &field, &edited(&parent, &proposal.edits)?)?;
            let measured = score(&candidate, &current, &field, eps, plan, &terms, start)?;
            write(&candidate, "objective.json", &measured)?;
            Ok(
                json!({"status":"COMPLETED","proposal":proposal,"receipt":receipt,
                "combined":objective(&measured)?,"task":measured["task"],"reference":measured["reference"],
                "native_calls":terms.len(),"checkpoint_seconds":checkpoint_seconds,"native_evaluation_seconds":measured["elapsed_seconds"],"objective_sha256":sha256_file(&candidate.out.join("objective.json"))?,
                "all_parent_bits_restored":true,"exact_native_code_edits_verified":true,"optimizer_updates":0}),
            )
        });
        let restored = snapshot(params)
            .map(|current| same_bits(&parent, &current))
            .unwrap_or(false);
        let receipt = match &outcome {
            Ok(v) => v.clone(),
            Err(e) => {
                json!({"status":"FAILED","error":e.to_string(),"all_parent_bits_restored":restored,"model_verdict":"execution failure, not candidate quality"})
            }
        };
        fs::write(
            candidate.out.join("report.json"),
            serde_json::to_vec(&receipt)?,
        )?;
        report_output::seal(&candidate.out)?;
        report_output::verify(&candidate.out)?;
        let result = outcome?;
        losses.push(Some(objective(&result)?));
        results.push(json!({"index":index,"report":result,"report_sha256":sha256_file(&candidate.out.join("report.json"))?,"manifest_sha256":sha256_file(&candidate.out.join("manifest.json"))?}));
        write(
            a,
            "native-code-progress.json",
            &json!({"completed":results,"parent_restored":true}),
        )?;
    }
    let winner = select(baseline_loss, &losses);
    let committed = (|| -> Result<Value> {
        if let Some(index) = winner {
            apply_edits(params, &parent, &proposals[index].edits)?;
        }
        let result = json!({"policy":policy(a.reached_frontier_objective),"status":if winner.is_some(){"NATIVE_OBJECTIVE_DESCENT_SELECTED"}else{"NO_IMPROVING_DECLARED_CANDIDATE"},
            "proposal_bank_sha256":sha256_file(&a.out.join("native-code-proposal-bank.json"))?,"parent_combined":baseline_loss,
            "winner":winner,"winner_family":winner.map(|i|proposals[i].family.clone()),
            "selected_combined":winner.and_then(|i|losses[i]).unwrap_or(baseline_loss),
            "accepted_code_proposals":usize::from(winner.is_some()),"optimizer_updates":0,
            "candidate_checkpoint_seconds":results.iter().filter_map(|r|r["report"]["checkpoint_seconds"].as_f64()).sum::<f64>(),
            "native_objective_seconds":baseline["elapsed_seconds"].as_f64().unwrap_or(0.) + results.iter().filter_map(|r|r["report"]["native_evaluation_seconds"].as_f64()).sum::<f64>(),
            "native_objective_calls_before_final":terms.len() * (1 + losses.iter().filter(|v|v.is_some()).count()),"results":results,
            "selected_master_identities":identities(params)?,"context_candidates_evaluated":proposals.iter().take(12).filter(|p|p.status=="evaluate").count()});
        write(a, "native-code-proposals.json", &result)?;
        Ok(result)
    })();
    if committed.is_err() {
        // The in-memory selected state is committed only once its selection receipt is durable.
        return attempt_restored(params, &parent, || committed);
    }
    committed
}

pub(super) fn verify_final(
    a: &Args,
    p: &ContinuationParent,
    field: &NativeContinuationField,
    eps: &[Episode],
    plan: &ReferencePlan,
    start: Instant,
) -> Result<()> {
    let summary = read(&a.out.join("native-code-proposals.json"))?;
    let terms: Vec<Term> =
        serde_json::from_value(read(&a.out.join("native-code-objective-terms.json"))?)?;
    let actual = score(a, p, field, eps, plan, &terms, start)?;
    let expected = match summary["winner"].as_u64() {
        Some(index) => read(
            &a.out
                .join(format!("native-candidate-{index:02}/objective.json")),
        )?,
        None => read(&a.out.join("native-code-parent-objective.json"))?,
    };
    // Exclude elapsed time only. Every pool, score hash, state, weight and binding must match.
    for key in [
        "task",
        "reference",
        "combined",
        "terms",
        "source_binding",
        "generate_sha256",
        "continuation_sha256",
        "categorical_sha256",
        "native_calls",
    ] {
        replay_require(
            actual[key] == expected[key],
            "final re-export differs from selected native candidate",
        )?;
    }
    write(a, "native-code-final-objective.json", &actual)?;
    write(
        a,
        "native-code-final-parity.json",
        &json!({"selected_native_objective_exact":true,
        "final_objective_sha256":sha256_file(&a.out.join("native-code-final-objective.json"))?,"winner":summary["winner"]}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_code_block_clamps_and_keeps_unselected_off_grid_bits() -> Result<()> {
        let values = vec![1.73, -1.73, 0.031, -0.0];
        let p = make_pair("context", "x", 0, 4, 3., &values, &[1., -1., 0., 1.])?;
        assert_eq!(p[1].blocked_coordinates, vec![0, 1]);
        let parent = BTreeMap::from([("x".into(), values)]);
        let changed = edited(&parent, &p[1].edits)?;
        assert_eq!(changed["x"][0].to_bits(), parent["x"][0].to_bits());
        assert_eq!(changed["x"][1].to_bits(), parent["x"][1].to_bits());
        assert_eq!(changed["x"][2].to_bits(), parent["x"][2].to_bits());
        assert_eq!(changed["x"][3], 0.25);
        assert_eq!(p[0].edits.len(), 3);
        Ok(())
    }
    #[test]
    fn native_code_rank_uses_whole_rows_and_deterministic_names() -> Result<()> {
        let parent = BTreeMap::from([("a".into(), vec![0.; 8]), ("b".into(), vec![0.; 8])]);
        let gradient = BTreeMap::from([
            ("a".into(), vec![1., 1., 1., 1., 3., 0., 0., 0.]),
            ("b".into(), vec![1.; 8]),
        ]);
        let p = ranked_pair("context", &["b".into(), "a".into()], 4, &parent, &gradient)?;
        assert_eq!(p[0].name, "a");
        assert_eq!(p[0].row, 0);
        assert_eq!(p[0].edits.len(), 4);
        assert_eq!(
            select(
                2.,
                &[Some(2. - 1e-12), Some(1.9), Some(1.9), Some(f64::NAN)]
            ),
            Some(1)
        );
        assert_eq!(select(2., &[Some(2.), None]), None);
        Ok(())
    }
    #[test]
    fn native_code_bank_has_six_basis_pairs_and_excludes_token_bias_and_prototypes() -> Result<()> {
        let mut params = BTreeMap::new();
        for family in BASIS {
            params.insert(
                format!("consumer.context.{family}"),
                Var::from_vec(vec![0f32; 8], (1, 1, 2, 4), &Device::Cpu)?,
            );
        }
        for name in [
            "consumer.context.token_transition",
            "consumer.potential.pair",
            "generate.unary",
            "generate.pair",
            "generate.bias",
            "continuation.unary",
        ] {
            params.insert(name.into(), Var::from_vec(vec![0f32; 2], 2, &Device::Cpu)?);
        }
        let mut gradients = params
            .iter()
            .map(|(n, v)| {
                Ok((
                    n.clone(),
                    Tensor::ones(v.shape(), candle_core::DType::F32, &Device::Cpu)?,
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?;
        gradients.insert(
            "generate.bias".into(),
            Tensor::from_vec(vec![1e6f32; 2], 2, &Device::Cpu)?,
        );
        let proposals = bank(&params, &snapshot(&params)?, &gradients)?;
        assert_eq!(proposals.len(), 18);
        assert_eq!(
            proposals
                .iter()
                .take(12)
                .filter(|p| p.edits.len() == 4)
                .count(),
            12
        );
        assert!(proposals.iter().all(|p| p.name != "generate.bias"
            && !p.name.contains("token_")
            && !p.name.contains("prototype")));
        assert_eq!(proposals[14].name, "generate.pair");
        assert_eq!(proposals[14].direction, -1);
        assert_eq!(proposals[15].direction, 1);
        Ok(())
    }
    #[test]
    fn native_code_failed_attempt_restores_every_parent_bit() -> Result<()> {
        let params = BTreeMap::from([(
            "x".into(),
            Var::from_vec(vec![-0f32, 0.03, 1.71], 3, &Device::Cpu)?,
        )]);
        let parent = snapshot(&params)?;
        let outcome: Result<()> = attempt_restored(&params, &parent, || {
            apply_edits(
                &params,
                &parent,
                &[Edit {
                    name: "x".into(),
                    index: 1,
                    before: 0,
                    after: 1,
                }],
            )?;
            Err(bad("synthetic checkpoint/scoring failure"))
        });
        assert!(outcome.is_err());
        assert!(same_bits(&parent, &snapshot(&params)?));
        Ok(())
    }
}
