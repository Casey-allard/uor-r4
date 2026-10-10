//! One pre-registered cross-state forward mechanism; no constructor or selector.
use super::*;
pub(super) const ORIGINAL8: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
pub(super) fn settings(a: &Args) -> Result<()> {
    if a.mode != Mode::CrossStateContinuation {
        return Ok(());
    }
    if a.updates != 64
        || a.seed != 1001
        || a.credit != Credit::RawIdentity
        || a.read_state_pullback != ReadStatePullback::Legacy
        || a.loss_scope != LossScope::All
        || a.ceiling_scorer
        || a.ceiling_float
        || a.query_conditioned_read
        || a.panel_inputs.is_some()
        || a.panel_labels.is_some()
        || a.baseline.is_some()
        || a.prediction_control_updates.is_some()
        || a.prediction_control_rates.is_some()
        || a.prediction_control_resume.is_some()
        || a.prediction_control_resume_source_step.is_some()
        || a.prediction_control_resume_generate_step.is_some()
        || a.prediction_control_trainable != ControlTrainable::Joint
        || a.continuation
            .as_ref()
            .is_none_or(|c| c.learning_rate != 0.03)
        || a.joint_continuation.is_some()
        || a.reference_replay.is_some()
        || a.native_code_proposals
        || a.reached_frontier_objective
        || a.categorical_action_learning
        || a.categorical_action_only
        || a.constrained_context_learning
        || a.constrained_emission_learning
        || a.retained_context_root.is_some()
        || a.prototype_compensation.is_some()
        || a.reached_u.is_some()
        || a.readout_coadaptation.is_some()
        || a.prefix_context_credit.is_some()
        || a.context_path_credit.is_some()
        || a.context_cue_coadapt.is_some()
        || a.prefix_fragment_learning.is_some()
        || a.generate_episode_learning.is_some()
        || a.coupled_episode_learning.is_some()
        || a.generate_episode_completion.is_some()
        || a.prefix_artifact_check.is_some()
    {
        return Err(bad("cross-state continuation requires fixed64/seed1001/lr.03/raw_identity/all-answer fitting with frozen upstream and no proposal/diagnostic options"));
    }
    Ok(())
}

pub(super) fn baseline(initial: &Value) -> Result<()> {
    let rows = initial["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("cross-state baseline coverage"))?;
    let complete = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r["complete"] == true)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if complete != ORIGINAL8 || initial["complete"] != 8 {
        return Err(bad(
            "cross-state zero field did not reproduce exact original eight",
        ));
    }
    Ok(())
}

pub(super) fn outcomes(initial: &Value, final_eval: &Value) -> Result<Value> {
    baseline(initial)?;
    let before = initial["rows"]
        .as_array()
        .ok_or_else(|| bad("initial rows absent"))?;
    let after = final_eval["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("final rows coverage"))?;
    let mut seen = BTreeSet::new();
    let mut gained = Vec::new();
    let mut lost = Vec::new();
    let mut retained = Vec::new();
    let mut changed = Vec::new();
    let mut strata = BTreeMap::<String, (usize, usize, usize)>::new();
    for (old, new) in before.iter().zip(after) {
        let id = old["id"]
            .as_str()
            .ok_or_else(|| bad("row identity absent"))?;
        if new["id"] != id || !seen.insert(id) {
            return Err(bad("row identity/uniqueness differs"));
        }
        let a = old["complete"]
            .as_bool()
            .ok_or_else(|| bad("initial completion absent"))?;
        let b = new["complete"]
            .as_bool()
            .ok_or_else(|| bad("final completion absent"))?;
        match (a, b) {
            (false, true) => gained.push(id),
            (true, false) => lost.push(id),
            (true, true) => retained.push(id),
            _ => {}
        }
        if old["generated_ids"] != new["generated_ids"] {
            changed.push(id);
        }
        let stratum = id
            .split('-')
            .nth(2)
            .ok_or_else(|| bad("stratum absent"))?
            .to_string();
        let counts = strata.entry(stratum).or_default();
        counts.0 += 1;
        counts.1 += usize::from(a);
        counts.2 += usize::from(b);
    }
    let final_count = gained.len() + retained.len();
    if final_eval["complete"] != final_count {
        return Err(bad("final complete summary differs"));
    }
    Ok(json!({"initial_complete":8,"final_complete":final_count,
        "gained_complete_ids":gained,"lost_complete_ids":lost,"retained_complete_ids":retained,
        "changed_output_ids":changed,"by_stratum_cases_initial_final":strata,
        "keep":final_count>8,"decision":if final_count>8 {"KEEP"} else {"REJECT"},
        "bar":"prospective developmental netcomplete>8 on unchanged512 own-prefix; original-eight endpoint retention reported, not veto; M2 remains256 thenfresh40%",
        "prior_runs":"prior all-original8 local KEEP decisions unchanged; no retrospective reclassification"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn endpoint(indices: &[usize]) -> Value {
        json!({"complete":indices.len(),"rows":(0..512).map(|i|json!({
   "id":format!("development-diverse-length2-{i}"),"complete":indices.contains(&i),
   "generated_ids":[i]})).collect::<Vec<_>>()})
    }
    #[test]
    fn cross_state_net_gain_records_original_loss_without_old_local_veto() -> Result<()> {
        let initial = endpoint(&ORIGINAL8);
        let next = endpoint(&[1, 4, 5, 8, 9, 12, 13, 100, 101]);
        let result = outcomes(&initial, &next)?;
        assert_eq!(result["keep"], true);
        assert_eq!(result["final_complete"], 9);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(outcomes(&initial, &initial)?["keep"], false);
        Ok(())
    }
    #[test]
    fn cross_state_rejects_wrong_zero_baseline_or_duplicate_rows() -> Result<()> {
        assert!(baseline(&endpoint(&[0, 1, 4, 5, 8, 9, 12])).is_err());
        let initial = endpoint(&ORIGINAL8);
        let mut next = initial.clone();
        next["rows"][1]["id"] = next["rows"][0]["id"].clone();
        assert!(outcomes(&initial, &next).is_err());
        Ok(())
    }
    #[test]
    fn cross_state_admission_rejects_recipe_or_credit_variations() -> Result<()> {
        let base = json!({"mode":"cross_state_continuation","credit":"raw_identity","seed":1001,
   "updates":64,"checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"p",
   "training_inputs":"i","training_labels":"l","development_inputs":"i","development_labels":"l",
   "maximum_seconds":7200,"maximum_report_bytes":1073741824,"out":"new",
   "continuation":{"expected_model_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
   "expected_model_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
   "learning_rate":0.03,"maximum_cache_tensor_bytes":536870912}});
        let parsed: Args = serde_json::from_value(base.clone())?;
        settings(&parsed)?;
        continuation_settings(&parsed)?;
        for (key, value) in [
            ("updates", json!(96)),
            ("seed", json!(1002)),
            ("native_code_proposals", json!(true)),
            ("read_state_pullback", json!("categorical")),
            ("loss_scope", json!("entry_only")),
        ] {
            let mut bad = base.clone();
            bad[key] = value;
            assert!(settings(&serde_json::from_value::<Args>(bad)?).is_err());
        }
        Ok(())
    }
}
