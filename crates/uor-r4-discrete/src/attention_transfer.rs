//! Deferred paired fresh evaluation after a sealed DEV H_geom. PM records bind reviewed bytes;
//! they do not cryptographically authenticate scientific analysis or issue a
//! training capability. An uncertain transfer intent is never replayed.
use super::*;

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArmPins {
    admission: FilePin,
    final_checkpoint: FilePin,
    final_dev: FilePin,
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransferDecision {
    schema: String,
    verdict: String,
    dev_verdict: String,
    pm_receipt_url: String,
    treatment: ArmPins,
    control: ArmPins,
    treatment_ablation_checkpoint: FilePin,
    treatment_ablation_dev: FilePin,
    /// Exact paired DEV analysis reviewed by PM. This module validates its byte
    /// pin, not the scientific discriminator or the analysis's authorship.
    paired_discriminator: FilePin,
}

#[cfg(any(feature = "cuda", test))]
fn validate_decision_header(decision: &TransferDecision) -> Result<()> {
    let receipt_id = decision
        .pm_receipt_url
        .strip_prefix("https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-");
    if decision.schema != "uor-r4.attention-transfer-decision/2"
        || decision.verdict != "RELEASE_PAIRED_FRESH_AFTER_DEV_H_GEOM"
        || decision.dev_verdict != "H_geom"
        || !receipt_id.is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
        || !valid_receipt_url(&decision.pm_receipt_url)
    {
        return Err(invalid(
            "transfer needs a distinct explicit paired-DEV PM decision",
        ));
    }
    for pin in [
        &decision.treatment.admission,
        &decision.treatment.final_checkpoint,
        &decision.treatment.final_dev,
        &decision.control.admission,
        &decision.control.final_checkpoint,
        &decision.control.final_dev,
        &decision.treatment_ablation_checkpoint,
        &decision.treatment_ablation_dev,
        &decision.paired_discriminator,
    ] {
        corpus::validate_sha256(&pin.sha256)?;
        if !pin.path.is_absolute() {
            return Err(invalid("transfer decision pins require absolute paths"));
        }
    }
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn private_new(path: &Path, value: &Value) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| invalid(format!("private exclusive {}: {e}", path.display())))?;
        file.write_all(&encode(value)?)
            .and_then(|_| file.sync_all())
            .map_err(|e| invalid(e.to_string()))
    }
    #[cfg(not(unix))]
    {
        let _ = (path, value);
        Err(invalid(
            "private paired transfer records require Unix file permissions",
        ))
    }
}

#[cfg(any(feature = "cuda", test))]
fn reserve_joint_intent(treatment: &Path, control: &Path, binding: &Value) -> Result<()> {
    for state in [treatment, control] {
        if state.join("fresh-transfer-intent.json").exists()
            || state.join("fresh-transfer-partial.json").exists()
            || state.join("fresh-transfer-joint.json").exists()
        {
            return Err(invalid(
                "paired fresh attempt exists or is uncertain; no automatic replay",
            ));
        }
    }
    // No rollback: a crash or second reservation failure leaves the first
    // durable marker and refuses every automatic replay without reading fresh.
    for state in [treatment, control] {
        private_new(&state.join("fresh-transfer-intent.json"), binding)?;
        sync_directory(state)?;
    }
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn require_fresh_inventory(rows: &[(u64, u32, bool)]) -> Result<()> {
    if rows.len() < 2000
        || rows.iter().filter(|row| row.2).count() < 500
        || rows.windows(2).any(|pair| pair[0].0 >= pair[1].0)
    {
        return Err(invalid(
            "fresh transfer needs >=2000 ordered unique rows and >=500 actual copy-successor rows",
        ));
    }
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn paired_floor(discordant: u64) -> Result<u64> {
    let square = discordant
        .checked_mul(4)
        .ok_or_else(|| invalid("fresh paired floor radicand overflow"))?;
    let root = square.isqrt();
    let exact = root
        .checked_mul(root)
        .ok_or_else(|| invalid("fresh paired floor square overflow"))?;
    let ceiling = root
        .checked_add(u64::from(exact < square))
        .ok_or_else(|| invalid("fresh paired floor ceiling overflow"))?;
    Ok(20.max(ceiling))
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PairedFresh {
    psucc_rows: u64,
    b3_r: u64,
    b3_c: u64,
    b3_i: u64,
    /// Control correct and treatment wrong, on P_succ B3_I only.
    b_fresh: u64,
    /// Treatment correct and control wrong, on P_succ B3_I only.
    c_fresh: u64,
    both_correct: u64,
    both_wrong: u64,
    floor: u64,
    treatment_direction_and_floor_met: bool,
}

#[cfg(any(feature = "cuda", test))]
fn hit_words(evaluation: &Value) -> Result<Vec<u64>> {
    let mut hits = Vec::new();
    for batch in evaluation["batches"]
        .as_array()
        .ok_or_else(|| invalid("fresh batches missing"))?
    {
        for row in batch["rows"]
            .as_array()
            .ok_or_else(|| invalid("fresh rows missing"))?
        {
            hits.push(
                row["top1_correct"]
                    .as_u64()
                    .filter(|hit| *hit <= 1)
                    .ok_or_else(|| invalid("fresh GPU top1_correct must be0 or1"))?,
            );
        }
    }
    Ok(hits)
}

#[cfg(any(feature = "cuda", test))]
fn increment(count: &mut u64) -> Result<()> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| invalid("fresh paired count overflow"))?;
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn paired_observations(
    treatment: &Value,
    control: &Value,
    expected: &[(u64, u32, bool)],
) -> Result<PairedFresh> {
    let r = evidence::summarize(treatment, expected)?;
    let c = evidence::summarize(control, expected)?;
    let rh = hit_words(treatment)?;
    let ch = hit_words(control)?;
    if rh.len() != expected.len() || ch.len() != expected.len() {
        return Err(invalid("fresh paired hit inventory differs"));
    }
    let mut paired = PairedFresh {
        psucc_rows: r.psucc.rows,
        b3_r: r.psucc.target_admitted,
        b3_c: c.psucc.target_admitted,
        b3_i: 0,
        b_fresh: 0,
        c_fresh: 0,
        both_correct: 0,
        both_wrong: 0,
        floor: 20,
        treatment_direction_and_floor_met: false,
    };
    for (((rr, cr), &rhit), &chit) in r.rows.iter().zip(&c.rows).zip(&rh).zip(&ch) {
        if rhit > u64::from(rr.target_admitted) || chit > u64::from(cr.target_admitted) {
            return Err(invalid(
                "fresh correct hit cannot exceed CUDA target admission",
            ));
        }
        if rr.is_psucc && rr.target_admitted && cr.target_admitted {
            increment(&mut paired.b3_i)?;
            match (rhit, chit) {
                (0, 1) => increment(&mut paired.b_fresh)?,
                (1, 0) => increment(&mut paired.c_fresh)?,
                (1, 1) => increment(&mut paired.both_correct)?,
                (0, 0) => increment(&mut paired.both_wrong)?,
                _ => return Err(invalid("invalid fresh paired hit")),
            }
        }
    }
    paired.floor = paired_floor(
        paired
            .b_fresh
            .checked_add(paired.c_fresh)
            .ok_or_else(|| invalid("fresh discordance overflow"))?,
    )?;
    paired.treatment_direction_and_floor_met = paired
        .c_fresh
        .checked_sub(paired.b_fresh)
        .is_some_and(|delta| delta >= paired.floor);
    Ok(paired)
}

#[cfg(feature = "cuda")]
fn fresh_rows(
    source: &corpus::PreparedCorpus,
    config: &DriverConfig,
) -> Result<Vec<(u64, u32, bool)>> {
    let mut rows = Vec::with_capacity(source.eligible_len());
    let mut first = 0;
    while first < source.eligible_len() as u64 {
        let batch = source.batch(first, config.spec.context, config.batch_rows)?;
        for ((window, &position), &target) in batch
            .windows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
        {
            rows.push((
                position,
                target,
                evidence::is_copy_successor(window, target),
            ));
        }
        first = batch.next_target;
    }
    require_fresh_inventory(&rows)?;
    Ok(rows)
}

#[cfg(feature = "cuda")]
fn pinned_json<T: serde::de::DeserializeOwned>(pin: &FilePin) -> Result<T> {
    serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))
}

#[cfg(feature = "cuda")]
fn same_path(actual: &Path, expected: &Path) -> Result<()> {
    if fs::canonicalize(actual).map_err(|e| invalid(e.to_string()))?
        != fs::canonicalize(expected).map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "transfer pin is not the canonical registered state file",
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
struct Arm {
    admission: AttentionAdmission,
    driver: PreparedDriver,
    release: ValidatedRelease,
    progress: AttentionCheckpoint,
}

#[cfg(feature = "cuda")]
fn admit_arm(pins: &ArmPins, compiled: &Value, executable: &str) -> Result<Arm> {
    let admission: AttentionAdmission = pinned_json(&pins.admission)?;
    let driver = PreparedDriver::load(admission.plan.config.clone())?;
    let release = release(&driver, &admission, compiled, executable)?;
    let progress = read_progress(&pins.final_checkpoint)?;
    validate_progress(&driver, &release, &progress)?;
    if progress.checkpoint.cursor.completed_batches != ATTENTION_BATCHES
        || pins.final_checkpoint.path.file_name()
            != Some(std::ffi::OsStr::new("checkpoint-16404.json"))
    {
        return Err(invalid("transfer refuses nonfinal or selected checkpoints"));
    }
    let state = &release.manifest.run_state_directory;
    let latest: FilePin = read_json(&state.join("latest.json"))?;
    if latest.sha256 != pins.final_checkpoint.sha256 {
        return Err(invalid(
            "transfer checkpoint differs from the final state pointer",
        ));
    }
    same_path(&latest.path, &pins.final_checkpoint.path)?;
    // Preserve the exact immutable initialization baseline, not only an epoch0
    // label supplied alongside a selected final checkpoint.
    let initial_pin: FilePin = read_json(&state.join("initial.json"))?;
    let initial = read_progress(&initial_pin)?;
    validate_progress(&driver, &release, &initial)?;
    if initial.checkpoint.cursor != training::Cursor::initial()
        || initial.counters != Counters::default()
        || initial.baseline_snapshot_metadata
            != training::snapshot_metadata(&initial.checkpoint.model)?
        || progress.baseline_evaluation != initial.baseline_evaluation
        || progress.baseline_snapshot_metadata != initial.baseline_snapshot_metadata
    {
        return Err(invalid("transfer initial baseline identity differs"));
    }
    verify_evaluation(&initial.baseline_evaluation, &initial.checkpoint)?;
    Ok(Arm {
        admission,
        driver,
        release,
        progress,
    })
}

#[cfg(feature = "cuda")]
fn dev_record(
    pin: &FilePin,
    arm: &Arm,
    checkpoint: &training::DriverCheckpoint,
    filename: &str,
) -> Result<Value> {
    same_path(
        &pin.path,
        &arm.release.manifest.run_state_directory.join(filename),
    )?;
    let value: Value = pinned_json(pin)?;
    if value["schema"] != json!("uor-r4.attention-learning-evaluation/1")
        || value["admission_sha256"] != json!(arm.release.manifest_sha256)
        || value["driver_checkpoint_sha256"] != json!(corpus::sha256(&encode(checkpoint)?))
        || value["transfer"] != Value::Null
        || value["evaluation"].get("transfer_corpus").is_some()
    {
        return Err(invalid(
            "transfer DEV evidence has wrong arm/checkpoint/panel",
        ));
    }
    verify_evaluation(&value["evaluation"], checkpoint)?;
    Ok(value["evaluation"].clone())
}

#[cfg(feature = "cuda")]
fn match_arms(treatment: &Arm, control: &Arm) -> Result<()> {
    let t = &treatment.admission.plan;
    let c = &control.admission.plan;
    if !matches!(t.config.read_spec, ReadSpec::Relation { .. })
        || t.config.relation_learning_policy != training::RelationLearningPolicy::LearnV1
        || c.config.relation_learning_policy != training::RelationLearningPolicy::FrozenZeroV1
    {
        return Err(invalid(
            "transfer needs Relation treatment and matched45-coordinate off control",
        ));
    }
    let mut matched = c.config.clone();
    matched.relation_learning_policy = training::RelationLearningPolicy::LearnV1;
    let mut execution = c.execution.clone();
    execution.config_sha256 = t.execution.config_sha256.clone();
    execution.lineage_sha256 = t.execution.lineage_sha256.clone();
    if matched != t.config
        || execution != t.execution
        || c.sheet != t.sheet
        || c.transfer != t.transfer
        || treatment.progress.baseline_snapshot_metadata
            != control.progress.baseline_snapshot_metadata
        || treatment.progress.baseline_evaluation["model_snapshot_sha256"]
            != control.progress.baseline_evaluation["model_snapshot_sha256"]
    {
        return Err(invalid(
            "transfer arms differ beyond registered relation policy/identities",
        ));
    }
    require_disjoint_directories(&t.run_state_directory, &c.run_state_directory)
}

#[cfg(not(feature = "cuda"))]
pub fn run(_decision_path: &Path, _out: &Path) -> Result<()> {
    Err(invalid(
        "attention transfer requires CUDA; no fresh content read or capability issued",
    ))
}

#[cfg(feature = "cuda")]
fn partial_pin(arm: &Arm) -> Result<FilePin> {
    let path = arm
        .release
        .manifest
        .run_state_directory
        .join("fresh-transfer-partial.json");
    let bytes = fs::read(&path).map_err(|e| invalid(e.to_string()))?;
    Ok(FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&bytes),
    })
}

#[cfg(feature = "cuda")]
fn validate_partial(
    value: &Value,
    arm: &Arm,
    label: &str,
    binding: &Value,
    fresh: &corpus::PreparedCorpus,
    rows: &[(u64, u32, bool)],
) -> Result<()> {
    if value["schema"] != json!("uor-r4.attention-transfer-partial/2")
        || value["arm"] != json!(label)
        || value["binding"] != *binding
        || value["admission_sha256"] != json!(arm.release.manifest_sha256)
        || value["driver_checkpoint_sha256"]
            != json!(corpus::sha256(&encode(&arm.progress.checkpoint)?))
        || value["snapshot_unchanged"] != json!(true)
        || value["updates_applied"] != json!(0)
        || value["evaluation"]["transfer_corpus"] != json!(fresh.binding())
        || value["evaluation"]["transfer_is_separate_evaluation_content"] != json!(true)
    {
        return Err(invalid("paired fresh partial binding differs"));
    }
    verify_evaluation(&value["evaluation"], &arm.progress.checkpoint)?;
    if value["b3"] != json!(evidence::summarize(&value["evaluation"], rows)?) {
        return Err(invalid("paired fresh partial B3 differs"));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
fn observe_arm(
    arm: &Arm,
    label: &str,
    binding: &Value,
    fresh: &corpus::PreparedCorpus,
    rows: &[(u64, u32, bool)],
) -> Result<Value> {
    let (model, _) = arm.driver.restore(arm.progress.checkpoint.clone(), 0)?;
    if serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
        != arm.progress.checkpoint.model
    {
        return Err(invalid("fresh restore changed the admitted snapshot"));
    }
    let evaluation = arm.driver.evaluate_transfer(&model, fresh)?;
    verify_evaluation(&evaluation, &arm.progress.checkpoint)?;
    let b3 = evidence::summarize(&evaluation, rows)?;
    if serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
        != arm.progress.checkpoint.model
    {
        return Err(invalid("fresh evaluation changed snapshot storage"));
    }
    let value = json!({"schema":"uor-r4.attention-transfer-partial/2","arm":label,
        "binding":binding,"admission_sha256":arm.release.manifest_sha256,
        "driver_checkpoint_sha256":corpus::sha256(&encode(&arm.progress.checkpoint)?),
        "evaluation":evaluation,"b3":b3,"snapshot_unchanged":true,"updates_applied":0});
    validate_partial(&value, arm, label, binding, fresh, rows)?;
    let state = &arm.release.manifest.run_state_directory;
    private_new(&state.join("fresh-transfer-partial.json"), &value)?;
    sync_directory(state)?;
    Ok(value)
}

/// One joint paired transfer attempt after sealed DEV H_geom. A FilePin
/// authenticates retained bytes, not PM authorship or discriminator correctness.
#[cfg(feature = "cuda")]
pub fn run(decision_path: &Path, out: &Path) -> Result<()> {
    let decision_bytes = fs::read(decision_path).map_err(|e| invalid(e.to_string()))?;
    let decision: TransferDecision =
        serde_json::from_slice(&decision_bytes).map_err(|e| invalid(e.to_string()))?;
    validate_decision_header(&decision)?;
    let decision_pin = FilePin {
        path: fs::canonicalize(decision_path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&decision_bytes),
    };
    decision.paired_discriminator.read()?;
    let compiled = crate::device::compiled_identity();
    let executable = executable_sha256()?;
    let treatment = admit_arm(&decision.treatment, &compiled, &executable)?;
    let control = admit_arm(&decision.control, &compiled, &executable)?;
    match_arms(&treatment, &control)?;
    // Both existing state locks prevent a concurrent training/resume invocation.
    // Failure to acquire either refuses; this command never waits on a peer.
    let _treatment_lock = lock_run(&treatment.release, true)?;
    let _control_lock = lock_run(&control.release, true)?;
    let treatment_dev = dev_record(
        &decision.treatment.final_dev,
        &treatment,
        &treatment.progress.checkpoint,
        "final-evaluation.json",
    )?;
    let control_dev = dev_record(
        &decision.control.final_dev,
        &control,
        &control.progress.checkpoint,
        "final-evaluation.json",
    )?;
    let state = &treatment.release.manifest.run_state_directory;
    same_path(
        &decision.treatment_ablation_checkpoint.path,
        &state.join("relation-off-checkpoint.json"),
    )?;
    let ablation: Value = pinned_json(&decision.treatment_ablation_checkpoint)?;
    let mut expected = treatment.progress.checkpoint.clone();
    let (combine, total) = gate_inventory(treatment.driver.config())?;
    let gates = expected.model["gates"]
        .as_array_mut()
        .ok_or_else(|| invalid("ablation parent gates missing"))?;
    if gates.len() != total as usize {
        return Err(invalid("ablation gate shape differs"));
    }
    for word in &mut gates[combine as usize..] {
        *word = json!(0);
    }
    expected.model_sha256 = corpus::sha256(&encode(&expected.model)?);
    if ablation
        != json!({"schema":"uor-r4.attention-relation-off/1",
        "admission_sha256":treatment.release.manifest_sha256,
        "parent_driver_checkpoint_sha256":corpus::sha256(&encode(&treatment.progress.checkpoint)?),
        "parent_model_sha256":treatment.progress.checkpoint.model_sha256,
        "modified_storage":"only relation gate trits set to zero", "checkpoint":expected})
    {
        return Err(invalid(
            "transfer ablation differs beyond relation-zero storage",
        ));
    }
    treatment.driver.validate_checkpoint(&expected)?;
    let ablation_dev = dev_record(
        &decision.treatment_ablation_dev,
        &treatment,
        &expected,
        "relation-off-evaluation.json",
    )?;
    // Reuse the exact pinned DEV row/partition and EvaluationRow count contract;
    // no Forward slot IDs or host model score are substituted.
    let rows = dev_row_identities(treatment.driver.config())?;
    for evaluation in [&treatment_dev, &control_dev, &ablation_dev] {
        evidence::summarize(evaluation, &rows)?;
    }
    if evidence::summarize(&treatment.progress.baseline_evaluation, &rows)?
        != evidence::summarize(&control.progress.baseline_evaluation, &rows)?
    {
        return Err(invalid("matched initial admission rows differ"));
    }
    require_disjoint_directories(out, state)?;
    require_disjoint_directories(out, &control.release.manifest.run_state_directory)?;
    let binding = json!({"decision":decision_pin,
        "decision_payload":decision,
        "treatment_admission_sha256":treatment.release.manifest_sha256,
        "control_admission_sha256":control.release.manifest_sha256,
        "treatment_final_checkpoint":decision.treatment.final_checkpoint,
        "control_final_checkpoint":decision.control.final_checkpoint,
        "transfer_source":treatment.admission.plan.transfer});
    let control_state = &control.release.manifest.run_state_directory;
    let result_path = state.join("fresh-transfer-joint.json");
    if result_path.exists() {
        for arm_state in [state, control_state] {
            let intent: Value = read_json(&arm_state.join("fresh-transfer-intent.json"))?;
            if intent != binding {
                return Err(invalid("joint fresh intent binding differs"));
            }
        }
        let receipt: Value = read_json(&result_path)?;
        if receipt["binding"] != binding
            || receipt["schema"] != json!("uor-r4.attention-transfer-result/2")
            || receipt["status"] != json!("PAIRED_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR")
            || receipt["snapshot_unchanged"] != json!(true)
            || receipt["updates_applied"] != json!(0)
            || receipt["treatment_fresh_evaluations"] != json!(1)
            || receipt["control_fresh_evaluations"] != json!(1)
            || receipt["gpu_fresh_evaluations"] != json!(2)
            || receipt["ablation_fresh_evaluations"] != json!(0)
        {
            return Err(invalid("completed transfer binding differs"));
        }
        // Both exact partials and both durable intents must exist. Revalidation
        // reads metadata only; no partial/uncertain attempt can reach this path.
        let treatment_pin = partial_pin(&treatment)?;
        let control_pin = partial_pin(&control)?;
        if receipt["treatment_partial"] != json!(treatment_pin)
            || receipt["control_partial"] != json!(control_pin)
        {
            return Err(invalid("completed joint transfer partial pins differ"));
        }
        let r: Value = pinned_json(&treatment_pin)?;
        let c: Value = pinned_json(&control_pin)?;
        let fresh = corpus::PreparedCorpus::load(
            &treatment.admission.plan.transfer,
            treatment.driver.config().spec.vocabulary,
            &treatment.driver.config().bos,
        )?;
        let rows = fresh_rows(&fresh, treatment.driver.config())?;
        validate_partial(&r, &treatment, "treatment", &binding, &fresh, &rows)?;
        validate_partial(&c, &control, "control", &binding, &fresh, &rows)?;
        if receipt["paired_fresh"]
            != json!(paired_observations(
                &r["evaluation"],
                &c["evaluation"],
                &rows
            )?)
        {
            return Err(invalid("completed fresh paired observations differ"));
        }
        fs::create_dir(out).map_err(|e| invalid(e.to_string()))?;
        private_new(&out.join("report.json"), &receipt)?;
        return sync_directory(out);
    }
    // Both exclusive, identical durable markers precede fresh loading/CUDA.
    // Any partial reservation is retained and never automatically repaired.
    fs::create_dir(out).map_err(|e| invalid(e.to_string()))?;
    reserve_joint_intent(state, control_state, &binding)?;
    let result = (|| -> Result<()> {
        let fresh = corpus::PreparedCorpus::load(
            &treatment.admission.plan.transfer,
            treatment.driver.config().spec.vocabulary,
            &treatment.driver.config().bos,
        )?;
        let rows = fresh_rows(&fresh, treatment.driver.config())?;
        // observe_arm syncs a private immutable partial before returning. Thus
        // treatment evidence is durable before control creates any CUDA context.
        let r = observe_arm(&treatment, "treatment", &binding, &fresh, &rows)?;
        let treatment_partial = partial_pin(&treatment)?;
        let c = observe_arm(&control, "control", &binding, &fresh, &rows)?;
        let control_partial = partial_pin(&control)?;
        let paired = paired_observations(&r["evaluation"], &c["evaluation"], &rows)?;
        let receipt = json!({"schema":"uor-r4.attention-transfer-result/2",
            "status":"PAIRED_FRESH_OBSERVED_PENDING_PM_DISCRIMINATOR",
            "binding":binding,"treatment_partial":treatment_partial,"control_partial":control_partial,
            "paired_fresh":paired,"snapshot_unchanged":true,
            "updates_applied":0,"treatment_fresh_evaluations":1,"control_fresh_evaluations":1,
            "gpu_fresh_evaluations":2,"ablation_fresh_evaluations":0,"training_capability_issued":false,
            "paired_floor_contract":"max(20, ceil(2*sqrt(b_fresh+c_fresh))) using checked integer arithmetic on P_succ B3_I",
            "paired_direction":"c_fresh treatment-only correct minus b_fresh control-only correct",
            "scientific_discriminator":"PENDING_PM; paired observations and floor are not an automatic KEEP",
            "training_other_scopes":"HELD","q4":"NOT_EXECUTABLE"});
        private_new(&result_path, &receipt)?;
        sync_directory(state)?;
        private_new(&out.join("report.json"), &receipt)?;
        sync_directory(out)
    })();
    if let Err(error) = &result {
        let _ = private_new(
            &out.join("failure.json"),
            &json!({
            "status":"TRANSFER_ATTEMPT_UNCERTAIN_OR_FAILED_NO_AUTOMATIC_REPLAY",
            "error":error.to_string(),"binding":binding,"mechanism_negative":false}),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decision() -> TransferDecision {
        let pin = FilePin {
            path: PathBuf::from("/sealed/metadata.json"),
            sha256: "a".repeat(64),
        };
        let arm = ArmPins {
            admission: pin.clone(),
            final_checkpoint: pin.clone(),
            final_dev: pin.clone(),
        };
        TransferDecision {
            schema: "uor-r4.attention-transfer-decision/2".into(),
            verdict: "RELEASE_PAIRED_FRESH_AFTER_DEV_H_GEOM".into(),
            dev_verdict: "H_geom".into(),
            pm_receipt_url: "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-1"
                .into(),
            treatment: arm.clone(),
            control: arm,
            treatment_ablation_checkpoint: pin.clone(),
            treatment_ablation_dev: pin.clone(),
            paired_discriminator: pin,
        }
    }
    #[test]
    fn transfer_metadata_requires_explicit_prospective_decision_and_all_pins() -> Result<()> {
        let good = decision();
        validate_decision_header(&good)?;
        for verdict in ["H_GEOM", "APPROVE", "KEEP"] {
            let mut bad = good.clone();
            bad.verdict = verdict.into();
            assert!(validate_decision_header(&bad).is_err());
        }
        for verdict in ["H_mixed", "H_null", "H_control", "UNDERPOWERED", "PENDING"] {
            let mut bad = good.clone();
            bad.dev_verdict = verdict.into();
            assert!(validate_decision_header(&bad).is_err());
        }
        let mut bad = good.clone();
        bad.schema = "uor-r4.attention-transfer-decision/1".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good.clone();
        bad.pm_receipt_url = "owner-terminal:inherited".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good.clone();
        bad.pm_receipt_url =
            "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good.clone();
        bad.control.final_checkpoint.sha256 = "bad".into();
        assert!(validate_decision_header(&bad).is_err());
        let mut bad = good;
        bad.paired_discriminator.path = PathBuf::from("relative.json");
        assert!(validate_decision_header(&bad).is_err());
        Ok(())
    }

    #[test]
    fn joint_intent_is_exclusive_and_preserves_partial_reservation() -> Result<()> {
        let state = std::env::temp_dir().join(format!(
            "uor-attention-transfer-intent-{}-{}",
            std::process::id(),
            unix_ns()?
        ));
        fs::create_dir(&state).map_err(|e| invalid(e.to_string()))?;
        let treatment = state.join("treatment");
        let control = state.join("control");
        fs::create_dir(&treatment).map_err(|e| invalid(e.to_string()))?;
        fs::create_dir(&control).map_err(|e| invalid(e.to_string()))?;
        let binding = json!({"decision_sha256":"a".repeat(64),"checkpoint":16404});
        // Simulate a crash after the first canonical reservation; it must never
        // fill the second marker and continue to fresh loading on replay.
        private_new(&treatment.join("fresh-transfer-intent.json"), &binding)?;
        sync_directory(&treatment)?;
        assert!(reserve_joint_intent(&treatment, &control, &binding).is_err());
        assert!(!control.join("fresh-transfer-intent.json").exists());
        let retained: Value = read_json(&treatment.join("fresh-transfer-intent.json"))?;
        assert_eq!(retained, binding);
        // A clean pair reserves identical bytes exactly once.
        let clean_t = state.join("clean-treatment");
        let clean_c = state.join("clean-control");
        fs::create_dir(&clean_t).map_err(|e| invalid(e.to_string()))?;
        fs::create_dir(&clean_c).map_err(|e| invalid(e.to_string()))?;
        reserve_joint_intent(&clean_t, &clean_c, &binding)?;
        assert_eq!(
            fs::read(clean_t.join("fresh-transfer-intent.json"))
                .map_err(|e| invalid(e.to_string()))?,
            fs::read(clean_c.join("fresh-transfer-intent.json"))
                .map_err(|e| invalid(e.to_string()))?
        );
        assert!(reserve_joint_intent(&clean_t, &clean_c, &binding).is_err());
        assert!(reserve_joint_intent(&clean_t, &clean_c, &json!({"decision":"changed"})).is_err());
        assert!(!clean_t.join("fresh-transfer-joint.json").exists());
        fs::remove_dir_all(&state).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }

    #[test]
    fn actual_fresh_inventory_requires_both_floors_and_unique_order() -> Result<()> {
        let rows = (0..2000).map(|i| (i, 0, i < 500)).collect::<Vec<_>>();
        require_fresh_inventory(&rows)?;
        assert!(require_fresh_inventory(&rows[..1999]).is_err());
        let mut fewer_successors = rows.clone();
        fewer_successors[499].2 = false;
        assert!(require_fresh_inventory(&fewer_successors).is_err());
        let mut duplicate = rows;
        duplicate[1].0 = duplicate[0].0;
        assert!(require_fresh_inventory(&duplicate).is_err());
        Ok(())
    }

    #[test]
    fn fresh_floor_is_exact_integer_ceiling_and_refuses_overflow() -> Result<()> {
        for (n, floor) in [
            (0, 20),
            (99, 20),
            (100, 20),
            (101, 21),
            (110, 21),
            (111, 22),
            (400, 40),
            (401, 41),
        ] {
            assert_eq!(paired_floor(n)?, floor);
        }
        assert!(paired_floor(u64::MAX / 4 + 1).is_err());
        Ok(())
    }

    #[test]
    fn fresh_discordance_uses_psucc_intersection_and_actual_hit_bits() -> Result<()> {
        let expected = (1..=6).map(|p| (p, 1, p <= 5)).collect::<Vec<_>>();
        let evaluation = |hits: &[u64], admitted: &[bool]| {
            json!({"batches":[{"rows":
            hits.iter().zip(admitted).enumerate().map(|(i,(&hit,&admitted))|
                json!({"target_position":i+1,"target_token":1,"admitted_cells":1,
                    "address_rivals":u64::from(!admitted),"top1_correct":hit})).collect::<Vec<_>>() }]})
        };
        let r = evaluation(&[1, 0, 1, 1, 0, 1], &[true, true, true, true, false, true]);
        let c = evaluation(&[0, 1, 1, 0, 0, 0], &[true, true, true, false, false, true]);
        let observed = paired_observations(&r, &c, &expected)?;
        assert_eq!(
            observed,
            PairedFresh {
                psucc_rows: 5,
                b3_r: 4,
                b3_c: 3,
                b3_i: 3,
                b_fresh: 1,
                c_fresh: 1,
                both_correct: 1,
                both_wrong: 0,
                floor: 20,
                treatment_direction_and_floor_met: false
            }
        );
        let mut invalid = r.clone();
        invalid["batches"][0]["rows"][0]["top1_correct"] = json!(2);
        assert!(paired_observations(&invalid, &c, &expected).is_err());
        let mut invalid = r.clone();
        invalid["batches"][0]["rows"][4]["top1_correct"] = json!(1);
        assert!(paired_observations(&invalid, &c, &expected).is_err());
        let mut reordered = c;
        reordered["batches"][0]["rows"][0]["target_position"] = json!(2);
        assert!(paired_observations(&r, &reordered, &expected).is_err());
        let expected = (1..=20).map(|p| (p, 1, true)).collect::<Vec<_>>();
        let winning = evaluation(&[1; 20], &[true; 20]);
        let losing = evaluation(&[0; 20], &[true; 20]);
        assert!(
            paired_observations(&winning, &losing, &expected)?.treatment_direction_and_floor_met
        );
        assert!(
            !paired_observations(&losing, &winning, &expected)?.treatment_direction_and_floor_met
        );
        Ok(())
    }
}
