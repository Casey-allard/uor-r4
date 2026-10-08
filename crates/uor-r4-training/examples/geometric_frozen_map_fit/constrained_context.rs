//! One shared Context update constructed against exact native successful paths.
use super::native_proposals as np;
use super::*;
use sha2::{Digest, Sha256};
use uor_r4_integer::geometric_context::ContextDecisionEvent;

pub(super) const MAX_CAPTURE_EVENTS: usize = 500_000;
/// Bound streamed diagnostic output without materializing another full JSON copy.
pub(super) struct BudgetWriter<W> {
    pub inner: W,
    pub remaining: u64,
}
impl<W: io::Write> io::Write for BudgetWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() as u64 > self.remaining {
            return Err(io::Error::other(
                "Context capture exceeds remaining report budget",
            ));
        }
        let written = self.inner.write(bytes)?;
        self.remaining -= written as u64;
        Ok(written)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CapturedDecisions {
    pub calls: Vec<Value>,
    pub events: Vec<ContextDecisionEvent>,
}
pub(super) fn policy() -> Value {
    json!({"schema":"uor-r4.native-constrained-context/1","rounds":1,"maximum_candidates":1,
        "construction":"one frozen ascending g*actual-master-delta/name/index pass over all six shared Context basis scalars; one negative-gradient adjacent legal Q4 code per coordinate; accumulate only exact native protected-winner-feasible changes",
        "objective":frontier::OBJECTIVE,"optimizer_updates":0,"global_clipping_applied":false,
        "protected":"complete actual native bank admission and all84 original-success steps; transition plus root/category, all physical source/query/prefix uses; exact per-factor rounding and earliest-index ties",
        "maximum_capture_events_per_endpoint":MAX_CAPTURE_EVENTS,
        "acceptance":"one composite candidate only; failed-frontier consumed-state/Generate-score change and strict native combined CE descent >1e-10*(1+abs(parent)); otherwise exact parent restore",
        "frozen":"Context token coefficients, Potential, Generate, U, categorical map, prototypes, tokenizer and geometry",
        "scope":"greedy feasible-prefix construction, not global feasibility/optimality or exact recurrent gradient; successful latent-decision preservation is an experimental constraint, not permanent architecture; no refill/order/radius/round sweep"})
}
fn capture(a: &Args, name: &str) -> Result<CapturedDecisions> {
    Ok(serde_json::from_reader(io::BufReader::new(
        fs::File::open(a.out.join(name))?,
    ))?)
}
fn scores_sha256(events: &[ContextDecisionEvent]) -> String {
    let mut h = Sha256::new();
    for event in events {
        for score in &event.scores_q24 {
            h.update(score.to_le_bytes());
        }
    }
    format!("{:x}", h.finalize())
}
fn same_decisions(a: &CapturedDecisions, b: &CapturedDecisions) -> bool {
    a.calls == b.calls
        && a.events.len() == b.events.len()
        && a.events.iter().zip(&b.events).all(|(x, y)| {
            x.call_index == y.call_index
                && x.invocation == y.invocation
                && x.family == y.family
                && x.token_id == y.token_id
                && x.head == y.head
                && x.lane == y.lane
                && x.flat_lane == y.flat_lane
                && x.own == y.own
                && x.neighbor == y.neighbor
                && x.winner == y.winner
        })
}
fn failed_effect(before: &Value, after: &Value) -> Result<usize> {
    let a = before["terms"]
        .as_array()
        .ok_or_else(|| bad("parent terms absent"))?;
    let b = after["terms"]
        .as_array()
        .ok_or_else(|| bad("candidate terms absent"))?;
    replay_require(a.len() == b.len(), "Context effect term population differs")?;
    let mut changed = 0;
    for (a, b) in a.iter().zip(b) {
        replay_require(
            a["term"] == b["term"],
            "Context effect term identity differs",
        )?;
        if a["term"]["component"] == 0
            && (a["factual_state"] != b["factual_state"]
                || a["generate_raw_scores_sha256"] != b["generate_raw_scores_sha256"])
        {
            changed += 1;
        }
    }
    Ok(changed)
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
    plan: &ReferencePlan,
    start: Instant,
) -> Result<Value> {
    let parent = np::snapshot(params)?;
    let reached = frontier::read_plan(a)?;
    frontier::components(&reached)?;
    let terms = reached
        .terms
        .iter()
        .map(|t| np::Term {
            index: t.index,
            position: t.position,
            target: t.target,
            component: t.component,
            weight: t.weight,
            parent_actual_prefix_ids: Some(t.parent_actual_prefix_ids.clone()),
        })
        .collect::<Vec<_>>();
    write(a, "native-code-objective-terms.json", &json!(terms))?;
    let inventory = np::save_ranking_gradients(a, params, gradients)?;
    let mut g = np::Shadows::new();
    for family in np::BASIS {
        let name = format!("consumer.context.{family}");
        let count = parent
            .get(&name)
            .ok_or_else(|| bad("Context parent tensor absent"))?
            .len();
        let values = match gradients.get(&name) {
            Some(v) => v.flatten_all()?.to_device(&Device::Cpu)?.to_vec1::<f32>()?,
            None => vec![0.; count],
        };
        g.insert(name, values);
    }
    np::verify_codes(initial, initial_field, &parent)?;
    let baseline = np::score_with_capture(
        a,
        initial,
        initial_field,
        eps,
        plan,
        &terms,
        start,
        Some("context-parent-decisions.json"),
    )?;
    let expected = read(&a.out.join("frontier-initial-validation.json"))?;
    for (key, saved) in [
        ("task", "frontier_losses_before_after"),
        ("reference", "success_losses_before_after"),
        ("combined", "combined_losses_before_after"),
    ] {
        let x = baseline[key]
            .as_f64()
            .ok_or_else(|| bad("parent native objective absent"))?;
        let y = expected[saved][0]
            .as_f64()
            .ok_or_else(|| bad("initial native objective absent"))?;
        replay_require(
            (x - y).abs() <= 1e-10 * (1. + y.abs()),
            "constrained Context parent differs from initial native evaluation",
        )?;
    }
    write(a, "native-code-parent-objective.json", &baseline)?;
    let protected = capture(a, "context-parent-decisions.json")?;
    let construction_clock = Instant::now();
    let construction = context_constraints::construct(&parent, &g, &protected.events)?;
    let construction_seconds = construction_clock.elapsed().as_secs_f64();
    write(
        a,
        "context-construction.json",
        &json!({"policy":policy(),"construction":construction,
        "construction_seconds":construction_seconds,"parent_master_identities":identities(params)?,
        "parent_capture_sha256":sha256_file(&a.out.join("context-parent-decisions.json"))?,
        "parent_scores_sha256":scores_sha256(&protected.events),"parent_gradient_inventory":inventory,
        "objective_terms_sha256":sha256_file(&a.out.join("native-code-objective-terms.json"))?,
        "frontier_plan_sha256":sha256_file(&a.out.join("frontier-plan.json"))?}),
    )?;
    let baseline_loss = np::objective(&baseline)?;
    let mut selected = false;
    let mut status = "EMPTY_FEASIBLE_PREFIX_CONSTRUCTION";
    let mut measured_loss = None;
    let mut results = Vec::new();
    let mut checkpoint_seconds = 0.;
    let mut native_seconds = baseline["elapsed_seconds"]
        .as_f64()
        .ok_or_else(|| bad("parent score timing absent"))?;
    if !construction.edits.is_empty() {
        let mut candidate = a.clone();
        candidate.out = a.out.join("native-candidate-00");
        candidate.maximum_report_bytes = a
            .maximum_report_bytes
            .checked_sub(size(&a.out)?)
            .ok_or_else(|| bad("Context report cap"))?;
        replay_require(
            candidate.maximum_report_bytes > 2_097_152,
            "Context candidate reserve exhausted",
        )?;
        report_output::claim(&candidate.out)?;
        let outcome = np::attempt_restored(params, &parent, || {
            for name in ["reference-plan.json", "frontier-plan.json"] {
                fs::copy(a.out.join(name), candidate.out.join(name))?;
            }
            write(&candidate, "proposal.json", &json!(construction))?;
            np::apply_edits(params, &parent, &construction.edits)?;
            let clock = Instant::now();
            let (current, field, receipt) = joint_checkpoint(&candidate, 0, l, weights)?;
            let checkpoint_seconds = clock.elapsed().as_secs_f64();
            np::verify_codes(&current, &field, &np::edited(&parent, &construction.edits)?)?;
            replay_require(
                current.bridge == initial.bridge,
                "constrained Context changed categorical map",
            )?;
            let measured = np::score_with_capture(
                &candidate,
                &current,
                &field,
                eps,
                plan,
                &terms,
                start,
                Some("context-candidate-decisions.json"),
            )?;
            let actual = capture(&candidate, "context-candidate-decisions.json")?;
            replay_require(
                same_decisions(&protected, &actual),
                "independently compiled Context violates protected causal decisions",
            )?;
            let actual_hash = scores_sha256(&actual.events);
            replay_require(
                actual_hash == construction.final_scores_sha256,
                "incremental Context scores differ from independent native compilation",
            )?;
            let changed = failed_effect(&baseline, &measured)?;
            write(&candidate, "objective.json", &measured)?;
            Ok(
                json!({"status":"COMPLETED","combined":np::objective(&measured)?,"task":measured["task"],"reference":measured["reference"],
                "failed_frontier_consumed_state_or_generate_changed":changed,
                "exact_protected_decisions_preserved":true,"predicted_native_scores_verified":true,"actual_scores_sha256":actual_hash,
                "capture_sha256":sha256_file(&candidate.out.join("context-candidate-decisions.json"))?,
                "checkpoint_seconds":checkpoint_seconds,"native_evaluation_seconds":measured["elapsed_seconds"],
                "receipt":receipt,"optimizer_updates":0,"all_parent_bits_restored":true}),
            )
        });
        let restored = np::same_bits(&parent, &np::snapshot(params)?);
        let result = match &outcome {
            Ok(v) => v.clone(),
            Err(e) => {
                json!({"status":"FAILED","error":e.to_string(),"all_parent_bits_restored":restored,"model_verdict":"execution failure, not model-quality evidence"})
            }
        };
        write(&candidate, "report.json", &result)?;
        report_output::seal(&candidate.out)?;
        report_output::verify(&candidate.out)?;
        let result = outcome?;
        let loss = result["combined"]
            .as_f64()
            .ok_or_else(|| bad("candidate objective absent"))?;
        let effect = result["failed_frontier_consumed_state_or_generate_changed"]
            .as_u64()
            .ok_or_else(|| bad("candidate effect absent"))?;
        selected = effect > 0 && loss < baseline_loss - 1e-10 * (1. + baseline_loss.abs());
        status = if selected {
            "NATIVE_CONSTRAINED_CONTEXT_DESCENT_SELECTED"
        } else if effect == 0 {
            "NO_FAILED_FRONTIER_FACTUAL_STATE_OR_GENERATE_EFFECT"
        } else {
            "CONSTRAINED_CONTEXT_NATIVE_OBJECTIVE_NOT_IMPROVED"
        };
        measured_loss = Some(loss);
        checkpoint_seconds = result["checkpoint_seconds"]
            .as_f64()
            .ok_or_else(|| bad("candidate checkpoint timing absent"))?;
        native_seconds += result["native_evaluation_seconds"]
            .as_f64()
            .ok_or_else(|| bad("candidate evaluation timing absent"))?;
        results.push(json!({"index":0,"report":result,"report_sha256":sha256_file(&candidate.out.join("report.json"))?,"manifest_sha256":sha256_file(&candidate.out.join("manifest.json"))?}));
    }
    let committed = (|| -> Result<Value> {
        if selected {
            np::apply_edits(params, &parent, &construction.edits)?;
        }
        let winner = if selected { Some(0usize) } else { None };
        let result = json!({"policy":policy(),"status":status,"winner":winner,"winner_family":winner.map(|_|"constrained_context"),
            "parent_combined":baseline_loss,"candidate_combined":measured_loss,"selected_combined":if selected { measured_loss.ok_or_else(|| bad("selected loss absent"))? } else { baseline_loss },
            "accepted_code_proposals":usize::from(selected),"optimizer_updates":0,"construction_seconds":construction_seconds,
            "construction_sha256":sha256_file(&a.out.join("context-construction.json"))?,"candidate_checkpoint_seconds":checkpoint_seconds,
            "native_objective_seconds":native_seconds,"native_objective_calls_before_final":terms.len()*(1+results.len()),
            "results":results,"selected_master_identities":identities(params)?,"context_candidates_evaluated":usize::from(measured_loss.is_some())});
        write(a, "native-code-proposals.json", &result)?;
        Ok(result)
    })();
    if committed.is_err() {
        return np::attempt_restored(params, &parent, || committed);
    }
    committed
}
