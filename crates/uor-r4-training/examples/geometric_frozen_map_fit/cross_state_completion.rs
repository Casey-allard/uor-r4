//! One pre-registered cross-state forward mechanism; no constructor or selector.
use super::*;
pub(super) const ORIGINAL8: [usize; 8] = [0, 1, 4, 5, 8, 9, 12, 13];
pub(super) const RESUME_REPORT_SHA: &str =
    "738649541df34641ffe0eb6e45a151d612d485ff8902b2d9d8d1e812844a2cde";
pub(super) const RESUME_MANIFEST_SHA: &str =
    "e5623d996b60308d8ca1080f748c062388ff74c6b14b16f9956b4b792a9ca8c2";
pub(super) const RESUME_FIELD_SHA: &str =
    "f61249ae4032a92d452689a52add6c9b5a67099f55b1a9291ba6f6f66a0d9acf";
#[derive(Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResumeConfig {
    pub root: PathBuf,
    pub expected_report_sha256: String,
    pub expected_manifest_sha256: String,
    pub expected_field_sha256: String,
}

pub(super) struct ResumeState {
    root: PathBuf,
    field_bytes: Vec<u8>,
    receipt: Value,
    pub prior_final: Value,
    pub provenance: Value,
}

pub(super) fn load_resume(a: &Args, parent: &ContinuationParent) -> Result<Option<ResumeState>> {
    let Some(config) = &a.cross_state_resume else {
        return Ok(None);
    };
    settings(a)?;
    report_output::verify(&config.root)?;
    if sha256_file(&config.root.join("report.json"))? != RESUME_REPORT_SHA
        || sha256_file(&config.root.join("manifest.json"))? != RESUME_MANIFEST_SHA
    {
        return Err(bad("cross-state resume sealed report identity differs"));
    }
    let report = read(&config.root.join("report.json"))?;
    let checkpoint = config.root.join("checkpoint-0064");
    let receipt = read(&checkpoint.join("receipt.json"))?;
    let field_bytes = fs::read(checkpoint.join("continuation-field.bin"))?;
    if report["status"] != "COMPLETED"
        || report["mode"] != "cross_state_continuation"
        || report["updates"] != 64
        || report["final_receipt"] != receipt
        || receipt["step"] != 64
        || receipt["parent"] != serde_json::to_value(&parent.binding)?
        || receipt["generate_sha256"] != parent.generate_sha256
        || receipt["frozen_model_report_sha256"] != CONTINUATION_PARENT_REPORT_SHA
        || receipt["frozen_model_manifest_sha256"] != CONTINUATION_PARENT_MANIFEST_SHA
        || receipt["frozen_parent_receipt"] != parent.receipt
        || receipt["continuation_sha256"] != RESUME_FIELD_SHA
        || report["final"]["continuation_sha256"] != RESUME_FIELD_SHA
        || sha256_bytes(&field_bytes) != RESUME_FIELD_SHA
        || receipt["shared_coefficients"] != 115200
        || receipt["active_parameter_names"] != json!(["continuation.cross_state"])
        || read(&checkpoint.join("continuation-source/metadata.json"))? != receipt
    {
        return Err(bad(
            "cross-state resume checkpoint/upstream/field identity differs",
        ));
    }
    validate_rows(&report["final"], 22)?;
    let provenance = json!({"root":fs::canonicalize(&config.root)?,
        "report_sha256":RESUME_REPORT_SHA,"manifest_sha256":RESUME_MANIFEST_SHA,
        "checkpoint_receipt_sha256":sha256_file(&checkpoint.join("receipt.json"))?,
        "field_sha256":RESUME_FIELD_SHA,"source_master_inventory":receipt["parameters"],
        "prior_step":64,"prior_complete":22,"optimizer":"reset fresh AdamW moments; parameter continuation only"});
    Ok(Some(ResumeState {
        root: checkpoint,
        field_bytes,
        receipt,
        prior_final: report["final"].clone(),
        provenance,
    }))
}

impl ResumeState {
    pub fn restore_weights(
        &self,
        parent: &ContinuationParent,
        device: &Device,
    ) -> Result<ContinuationLearningWeights> {
        let generator = parent.generator()?;
        let field = NativeContinuationField::from_bytes(
            &self.field_bytes,
            generator.source_binding(),
            generator.generate_model(),
        )?;
        if !field.is_cross_state() || field.lanes() != 8 || field.vocab_size() != 4096 {
            return Err(bad("cross-state resume native schema/dimensions differ"));
        }
        let weights = ContinuationLearningWeights::from_native(
            &field,
            generator.generate_model(),
            parent.integer.binding(),
            device,
        )?;
        restore(
            &self.root.join("continuation-source"),
            &self.receipt["parameters"],
            &weights.parameters(),
            device,
        )?;
        if weights
            .export_native(generator.source_binding(), generator.generate_model())?
            .to_bytes()?
            != self.field_bytes
        {
            return Err(bad(
                "cross-state resumed fractional masters do not export exact saved field",
            ));
        }
        Ok(weights)
    }
}

pub(super) fn schedule_indices(schedule: &[usize], update: usize) -> Vec<usize> {
    (0..BATCH)
        .map(|i| schedule[(update * BATCH + i) % schedule.len()])
        .collect()
}

fn validate_rows(endpoint: &Value, expected: usize) -> Result<()> {
    let rows = endpoint["rows"]
        .as_array()
        .filter(|r| r.len() == 512)
        .ok_or_else(|| bad("cross-state endpoint coverage"))?;
    let mut seen = BTreeSet::new();
    let mut count = 0;
    for row in rows {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| bad("cross-state row ID absent"))?;
        if !seen.insert(id)
            || row["generated_ids"]
                .as_array()
                .is_none_or(|v| v.iter().any(|id| id.as_u64().is_none()))
        {
            return Err(bad("cross-state row identity/generated IDs invalid"));
        }
        count += usize::from(
            row["complete"]
                .as_bool()
                .ok_or_else(|| bad("cross-state completion absent"))?,
        );
    }
    if count != expected || endpoint["complete"] != expected {
        return Err(bad("cross-state complete count differs"));
    }
    Ok(())
}

pub(super) fn settings(a: &Args) -> Result<()> {
    if a.mode != Mode::CrossStateContinuation {
        return if a.cross_state_resume.is_some() {
            Err(bad("cross-state resume requires cross-state mode"))
        } else {
            Ok(())
        };
    }
    if let Some(c) = &a.cross_state_resume {
        if c.expected_report_sha256 != RESUME_REPORT_SHA
            || c.expected_manifest_sha256 != RESUME_MANIFEST_SHA
            || c.expected_field_sha256 != RESUME_FIELD_SHA
        {
            return Err(bad("cross-state resume configuration pins differ"));
        }
    }
    if a.updates
        != if a.cross_state_resume.is_some() {
            256
        } else {
            64
        }
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
        return Err(bad("cross-state continuation requires fresh64 or pinned-resume256/seed1001/lr.03/raw_identity/all-answer fitting with frozen upstream and no proposal/diagnostic options"));
    }
    Ok(())
}

pub(super) fn baseline(initial: &Value, prior: Option<&Value>) -> Result<()> {
    if let Some(prior) = prior {
        if prior["continuation_sha256"] != RESUME_FIELD_SHA
            || initial["continuation_sha256"] != RESUME_FIELD_SHA
        {
            return Err(bad("cross-state resume baseline field identity differs"));
        }
        validate_rows(prior, 22)?;
        validate_rows(initial, 22)?;
        let old = prior["rows"]
            .as_array()
            .ok_or_else(|| bad("resume prior rows absent"))?;
        let current = initial["rows"]
            .as_array()
            .ok_or_else(|| bad("resume baseline rows absent"))?;
        for (a, b) in old.iter().zip(current) {
            for key in ["id", "generated_ids", "complete"] {
                if a[key] != b[key] {
                    return Err(bad(
                        "cross-state resumed baseline differs from saved512 rows",
                    ));
                }
            }
        }
        return Ok(());
    }
    validate_rows(initial, 8)?;
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

pub(super) fn outcomes(
    initial: &Value,
    final_eval: &Value,
    prior: Option<&Value>,
) -> Result<Value> {
    baseline(initial, prior)?;
    let initial_count = if prior.is_some() { 22 } else { 8 };
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
    Ok(
        json!({"initial_complete":initial_count,"final_complete":final_count,
        "gained_complete_ids":gained,"lost_complete_ids":lost,"retained_complete_ids":retained,
        "changed_output_ids":changed,"by_stratum_cases_initial_final":strata,
        "keep":final_count>initial_count,"decision":if final_count>initial_count {"KEEP"} else {"REJECT"},
        "bar":format!("prospective developmental netcomplete>{initial_count} on unchanged512 own-prefix; exact loss/gain IDs reported, not endpoint retention veto; M2 remains256 thenfresh40%"),
        "prior_runs":"prior all-original8 local KEEP decisions unchanged; no retrospective reclassification"}),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn endpoint(indices: &[usize]) -> Value {
        json!({"complete":indices.len(),"continuation_sha256":RESUME_FIELD_SHA,"rows":(0..512).map(|i|json!({
   "id":format!("development-diverse-length2-{i}"),"complete":indices.contains(&i),
   "generated_ids":[i]})).collect::<Vec<_>>()})
    }
    #[test]
    fn cross_state_net_gain_records_original_loss_without_old_local_veto() -> Result<()> {
        let initial = endpoint(&ORIGINAL8);
        let next = endpoint(&[1, 4, 5, 8, 9, 12, 13, 100, 101]);
        let result = outcomes(&initial, &next, None)?;
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
        assert_eq!(outcomes(&initial, &initial, None)?["keep"], false);
        Ok(())
    }
    #[test]
    fn cross_state_rejects_wrong_zero_baseline_or_duplicate_rows() -> Result<()> {
        assert!(baseline(&endpoint(&[0, 1, 4, 5, 8, 9, 12]), None).is_err());
        let initial = endpoint(&ORIGINAL8);
        let mut next = initial.clone();
        next["rows"][1]["id"] = next["rows"][0]["id"].clone();
        assert!(outcomes(&initial, &next, None).is_err());
        Ok(())
    }
    fn config() -> Value {
        json!({"mode":"cross_state_continuation","credit":"raw_identity","seed":1001,
   "updates":64,"checkpoint":"cp","saved_fit":"fit","categorical":"cat","parent_config":"p",
   "training_inputs":"i","training_labels":"l","development_inputs":"i","development_labels":"l",
   "maximum_seconds":7200,"maximum_report_bytes":1073741824,"out":"new",
   "continuation":{"expected_model_report_sha256":CONTINUATION_PARENT_REPORT_SHA,
   "expected_model_manifest_sha256":CONTINUATION_PARENT_MANIFEST_SHA,
   "learning_rate":0.03,"maximum_cache_tensor_bytes":536870912}})
    }
    #[test]
    fn cross_state_admission_rejects_recipe_or_credit_variations() -> Result<()> {
        let base = config();
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
    #[test]
    fn cross_state_resume_requires_exact_all512_outputs_not_only_complete22() -> Result<()> {
        let prior = endpoint(&(0..22).collect::<Vec<_>>());
        baseline(&prior, Some(&prior))?;
        for (row, key, value) in [
            (300, "generated_ids", json!([999])),
            (300, "id", json!("development-diverse-length2-changed")),
            (300, "complete", json!(true)),
        ] {
            let mut changed = prior.clone();
            changed["rows"][row][key] = value;
            assert!(baseline(&changed, Some(&prior)).is_err(), "{key}");
        }
        let mut swapped = prior.clone();
        swapped["rows"]
            .as_array_mut()
            .ok_or_else(|| bad("fixture rows"))?
            .swap(300, 301);
        assert!(baseline(&swapped, Some(&prior)).is_err());
        let mut duplicate = prior.clone();
        duplicate["rows"][300]["id"] = duplicate["rows"][301]["id"].clone();
        assert!(baseline(&duplicate, Some(&duplicate)).is_err());
        let mut wrong_field = prior.clone();
        wrong_field["continuation_sha256"] = json!("wrong");
        assert!(baseline(&wrong_field, Some(&prior)).is_err());
        Ok(())
    }

    #[test]
    fn cross_state_resume_net_gain_uses_saved22_and_reports_losses() -> Result<()> {
        let prior = endpoint(&(0..22).collect::<Vec<_>>());
        let next = endpoint(&(1..24).collect::<Vec<_>>());
        let result = outcomes(&prior, &next, Some(&prior))?;
        assert_eq!(result["initial_complete"], 22);
        assert_eq!(result["final_complete"], 23);
        assert_eq!(result["keep"], true);
        assert_eq!(
            result["lost_complete_ids"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            result["gained_complete_ids"].as_array().map(Vec::len),
            Some(2)
        );
        assert_eq!(outcomes(&prior, &prior, Some(&prior))?["keep"], false);
        assert_eq!(
            outcomes(
                &prior,
                &endpoint(&(0..20).collect::<Vec<_>>()),
                Some(&prior)
            )?["keep"],
            false
        );
        Ok(())
    }

    #[test]
    fn cross_state_resume_schedule_is_four_identical_full_passes() {
        let schedule = order(1001, 512);
        let fresh = (0..64)
            .flat_map(|u| schedule_indices(&schedule, u))
            .collect::<Vec<_>>();
        let resumed = (0..256)
            .flat_map(|u| schedule_indices(&schedule, u))
            .collect::<Vec<_>>();
        assert_eq!(fresh, schedule);
        assert_eq!(
            fresh.iter().copied().collect::<BTreeSet<_>>(),
            (0..512).collect()
        );
        assert_eq!(resumed.len(), 2048);
        for pass in resumed.chunks_exact(512) {
            assert_eq!(pass, fresh.as_slice());
        }
        for index in 0..512 {
            assert_eq!(resumed.iter().filter(|&&i| i == index).count(), 4);
        }
    }

    #[test]
    fn cross_state_resume_admission_pins_parent_and_exact_local256() -> Result<()> {
        let mut valid = config();
        valid["updates"] = json!(256);
        valid["cross_state_resume"] = json!({"root":"saved22",
            "expected_report_sha256":RESUME_REPORT_SHA,
            "expected_manifest_sha256":RESUME_MANIFEST_SHA,
            "expected_field_sha256":RESUME_FIELD_SHA});
        let parsed: Args = serde_json::from_value(valid.clone())?;
        settings(&parsed)?;
        continuation_settings(&parsed)?;
        for key in [
            "expected_report_sha256",
            "expected_manifest_sha256",
            "expected_field_sha256",
        ] {
            let mut changed = valid.clone();
            changed["cross_state_resume"][key] = json!("wrong");
            assert!(
                settings(&serde_json::from_value(changed)?).is_err(),
                "{key}"
            );
        }
        for updates in [0, 64, 128, 320] {
            let mut changed = valid.clone();
            changed["updates"] = json!(updates);
            assert!(settings(&serde_json::from_value(changed)?).is_err());
        }
        let mut wrong_mode = valid.clone();
        wrong_mode["mode"] = json!("continuation_only");
        assert!(settings(&serde_json::from_value(wrong_mode)?).is_err());
        let mut fresh = config();
        fresh["updates"] = json!(256);
        assert!(settings(&serde_json::from_value(fresh)?).is_err());
        valid["cross_state_resume"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<Args>(valid).is_err());
        Ok(())
    }

    #[test]
    fn cross_state_resume_restores_fractional_master_bits_and_rejects_tampering() -> Result<()> {
        let owned = owned_fixture()?;
        let values = vec![0.031f32, -0.124, 0.376, 1.749];
        let name = "continuation.cross_state".to_string();
        let source = BTreeMap::from([(
            name.clone(),
            Var::from_vec(values.clone(), (1, 4), &Device::Cpu)?,
        )]);
        let inventory = save_masters(&owned.0.join("masters"), &source)?;
        let restored = BTreeMap::from([(
            name.clone(),
            Var::zeros((1, 4), candle_core::DType::F32, &Device::Cpu)?,
        )]);
        restore(
            &owned.0.join("masters"),
            &inventory,
            &restored,
            &Device::Cpu,
        )?;
        assert_eq!(identities(&source)?, identities(&restored)?);
        let observed = restored[&name].flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(
            observed.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            values.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        );
        assert!(observed.iter().any(|x| *x != (x * 4.).round() * 0.25));
        let path = owned.0.join("masters").join(format!("{name}.f32le"));
        let original = fs::read(&path)?;
        let mut altered = original.clone();
        altered[0] ^= 1;
        fs::write(&path, &altered)?;
        assert!(restore(
            &owned.0.join("masters"),
            &inventory,
            &restored,
            &Device::Cpu
        )
        .is_err());
        fs::write(&path, &original)?;
        let mut wrong_shape = inventory.clone();
        wrong_shape[&name]["shape"] = json!([2, 2]);
        assert!(restore(
            &owned.0.join("masters"),
            &wrong_shape,
            &restored,
            &Device::Cpu
        )
        .is_err());
        let mut missing = inventory;
        missing
            .as_object_mut()
            .ok_or_else(|| bad("fixture inventory"))?
            .remove(&name);
        assert!(restore(&owned.0.join("masters"), &missing, &restored, &Device::Cpu).is_err());
        Ok(())
    }
    struct OwnedFixture(PathBuf);
    impl Drop for OwnedFixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn owned_fixture() -> Result<OwnedFixture> {
        // Keep test artifacts in the owned worktree, never system temp or a
        // shared cache. Exclusive creation prevents replacing another fixture.
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = PathBuf::from("local").join(format!(
            "cross-state-resume-test-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all("local")?;
        fs::create_dir(&root)?;
        Ok(OwnedFixture(root))
    }
    #[test]
    fn cross_state_resume_rejects_output_overlap_before_claim() -> Result<()> {
        let owned = owned_fixture()?;
        let safe_input = owned.0.join("inputs");
        let retained = owned.0.join("saved22");
        fs::create_dir(&safe_input)?;
        fs::create_dir(&retained)?;
        let mut config = config();
        for key in [
            "checkpoint",
            "saved_fit",
            "categorical",
            "parent_config",
            "training_inputs",
            "training_labels",
            "development_inputs",
            "development_labels",
        ] {
            config[key] = json!(safe_input);
        }
        config["cross_state_resume"] = json!({"root":retained,
            "expected_report_sha256":RESUME_REPORT_SHA,
            "expected_manifest_sha256":RESUME_MANIFEST_SHA,
            "expected_field_sha256":RESUME_FIELD_SHA});
        for output in [retained.clone(), retained.join("new-run"), owned.0.clone()] {
            config["out"] = json!(output);
            assert!(validate_input_output_paths(&serde_json::from_value(config.clone())?).is_err());
        }
        assert!(!retained.join("new-run").exists());
        config["out"] = json!(owned.0.join("new-run"));
        validate_input_output_paths(&serde_json::from_value(config)?)?;
        assert!(!owned.0.join("new-run").exists());
        Ok(())
    }
}
