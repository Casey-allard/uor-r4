//! D5 operational dose completion. No numerical implementation lives in this module.
//! Separate arm/seed scopes require separate owner-personal releases. D33 pilot delegation
//! does not apply. Imported model bytes and baseline are retained; only driver lineage
//! is migrated, from the single archived pilot artifact, with an explicit receipt.
use super::*;

#[path = "completion_baseline.rs"]
mod baseline_authentication;

#[path = "redirect_admission.rs"]
mod redirect;
pub use redirect::RedirectEvidence;

pub const COMPLETION_BATCHES: u64 = 16_404;
pub const CHECKPOINTS: [u64; 4] = [4_101, 8_202, 12_303, 16_404];
pub const PARENT_SHA256: &str = "46e8ed5ea0a282bcd8373922d6fa2c8e773b52efce9fe8c1ba2c4b5698ee0440";
const OLD_DRIVER: &str = "236e7fb6e5269e3bd4fed21d391d1ce82f7d9bd31275f1a0d97dd0c4ce9d9806";
const OLD_LINEAGE: &str = "6db5ecf5d6e1b334b6f788a28b741b7ff2e012e7bf451e5f89ed29f89ef82a12";
fn scope(seed: u32) -> Result<&'static str> {
    match seed {
        1 => Ok("m1-a2-completion-seed1-checkpoint1640..16404"),
        2 => Ok("m1-a2-completion-seed2-checkpoint0..16404"),
        _ => Err(invalid(
            "completion admits only separately released seed 1 or seed 2",
        )),
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionPlan {
    pub schema: String,
    pub status: String,
    pub scope: String,
    pub execution: ExecutionIdentity,
    pub config: DriverConfig,
    pub sheet: FilePin,
    pub claude_review: FilePin,
    pub run_state_directory: PathBuf,
    /// K8 seed 1 requires the archived pilot. K8 seed 2 and both K16 seeds are fresh.
    pub pilot_parent: Option<FilePin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub redirect: Option<RedirectEvidence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletionAdmission {
    pub schema: String,
    pub plan: CompletionPlan,
    pub owner_release: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionOwner {
    owner: String,
    decision: String,
    scope: String,
    plan_sha256: String,
    receipt_url: String,
}
fn pilot_config() -> Result<DriverConfig> {
    let plan: OwnerReleasePlan = serde_json::from_str(include_str!(
        "../../../docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
    ))
    .map_err(|e| invalid(e.to_string()))?;
    Ok(plan.config)
}
pub fn validate_config(config: &DriverConfig) -> Result<()> {
    if config.spec.kept_cells == 16 {
        return validate_redirect_config(config);
    }
    scope(config.seed)?;
    let mut expected = pilot_config()?;
    expected.seed = config.seed;
    if config != &expected {
        return Err(invalid("completion configuration must preserve exact pilot paths/data/spec/schedule, changing only the seed for separately released seed 2"));
    }
    config.validate()
}
fn parent(plan: &CompletionPlan) -> Result<Option<PilotCheckpoint>> {
    if plan.config.spec.kept_cells == 16 {
        if plan.pilot_parent.is_some() {
            return Err(invalid(
                "K16 redirect requires fresh initialization for both seeds",
            ));
        }
        return Ok(None);
    }
    match (plan.config.seed, &plan.pilot_parent) {
        (1, Some(pin)) if pin.sha256 == PARENT_SHA256 => {
            let p: PilotCheckpoint = serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
            if p.schema != "uor-r4.pilot-checkpoint/1"
                || p.checkpoint.cursor != (training::Cursor { completed_batches:1640, accepted_updates:236, corpus_pass:0, next_target:104960, next_coordinate:1640, block: None })
                || p.vector != (KindCounters { attempted:1640, any_candidate_cost_differs_from_identity:1179, accepted_updates:236 })
                || p.checkpoint.lineage.driver_source_sha256 != OLD_DRIVER
                || corpus::sha256(&encode(&p.checkpoint.lineage)?) != OLD_LINEAGE
                || p.checkpoint.lineage.config != plan.config
                || p.baseline_evaluation["evaluation_lineage_sha256"] != json!(OLD_LINEAGE)
                || p.baseline_evaluation["model_epoch"] != json!(0)
                || corpus::sha256(&encode(&p.checkpoint.model)?) != p.checkpoint.model_sha256
            { return Err(invalid("archived pilot parent provenance/cursor/baseline/counters mismatch")); }
            for (key, expected) in [("source_sha256", &plan.execution.source_sha256), ("kernel_sha256", &plan.execution.kernel_sha256), ("geometry_sha256", &plan.execution.geometry_sha256)] {
                if p.checkpoint.model[key].as_str() != Some(expected.as_str()) {
                    return Err(invalid(format!("completion must preserve pilot model {key}")));
                }
            }
            Ok(Some(p))
        }
        (2, None) => Ok(None),
        _ => Err(invalid("seed 1 requires exact archived checkpoint1640; seed 2 requires fresh initialization and its own release")),
    }
}
fn validate_plan(plan: &CompletionPlan) -> Result<()> {
    validate_config(&plan.config)?;
    if plan.config.spec.kept_cells == 16 {
        return redirect::validate_plan(plan);
    }
    if plan.redirect.is_some() {
        return Err(invalid("K8 completion cannot import redirect evidence"));
    }
    let e = &plan.execution;
    let original: OwnerReleasePlan = serde_json::from_str(include_str!(
        "../../../docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
    ))
    .map_err(|e| invalid(e.to_string()))?;
    if e.source_sha256 != original.execution.source_sha256
        || e.kernel_sha256 != original.execution.kernel_sha256
        || e.geometry_sha256 != original.execution.geometry_sha256
    {
        return Err(invalid(
            "completion must retain the exact qualified numerical source/PTX/geometry",
        ));
    }
    for h in [
        &e.source_sha256,
        &e.kernel_sha256,
        &e.geometry_sha256,
        &e.executable_sha256,
        &e.driver_source_sha256,
        &e.config_sha256,
        &e.lineage_sha256,
        &e.sheet_sha256,
    ] {
        corpus::validate_sha256(h)?;
    }
    if plan.schema != "uor-r4.completion-owner-release-plan/1"
        || plan.status != "HELD_OWNER_ACTION_REQUIRED"
        || plan.scope != scope(plan.config.seed)?
        || e.base_source != BASE_SOURCE
        || e.reviewed_source_commit.len() != 40
        || !e
            .reviewed_source_commit
            .bytes()
            .all(|c| c.is_ascii_hexdigit())
        || e.driver_source_sha256 != training::driver_source_sha256()
        || e.config_sha256 != corpus::sha256(&encode(&plan.config)?)
        || e.sheet_sha256 != plan.sheet.sha256
        || !plan.run_state_directory.is_absolute()
    {
        return Err(invalid(
            "completion plan schema/scope/source/config/sheet pins mismatch",
        ));
    }
    let bytes = plan.sheet.read()?;
    let sheet = std::str::from_utf8(&bytes).map_err(|e| invalid(e.to_string()))?;
    if !sheet.lines().any(|l| l.starts_with("TEST FITNESS: FIT"))
        || !sheet.contains("discrete-rung0-completion")
    {
        return Err(invalid("completion sheet lacks prospective FIT"));
    }
    let review: ReviewRecord =
        serde_json::from_slice(&plan.claude_review.read()?).map_err(|e| invalid(e.to_string()))?;
    if review.reviewer != "Claude"
        || review.verdict != "APPROVE_A2_COMPLETION_ADMISSION_DIFF"
        || review.scope != plan.scope
        || review.execution != *e
        || !valid_receipt_url(&review.receipt_url)
    {
        return Err(invalid(
            "completion requires Claude exact-source scoped review",
        ));
    }
    parent(plan)?;
    if let Some(pin) = &plan.pilot_parent {
        let original_root = pin
            .path
            .parent()
            .ok_or_else(|| invalid("pilot checkpoint requires a parent directory"))?;
        require_disjoint_directories(&plan.run_state_directory, original_root)?;
    }
    Ok(())
}
/// Read-only metadata check. No CUDA context, owner record, or apply capability is created.
pub fn check_plan(plan: &CompletionPlan) -> Result<Value> {
    validate_plan(plan)?;
    Ok(
        json!({"status":"PLAN_METADATA_VALID","scope":plan.scope,"seed":plan.config.seed,"plan_sha256":corpus::sha256(&encode(plan)?),"driver_source_sha256":training::driver_source_sha256(),"max_batches":COMPLETION_BATCHES,"scored_checkpoints":CHECKPOINTS,"owner_release":"STILL_REQUIRED","training":"HELD","cuda_context_created":false}),
    )
}
fn validate_owner(plan: &CompletionPlan, pin: &FilePin, owner: &CompletionOwner) -> Result<()> {
    if owner.owner != "Casey"
        || owner.decision != owner_decision(plan)
        || owner.scope != plan.scope
        || owner.plan_sha256 != corpus::sha256(&encode(plan)?)
        || owner.receipt_url != format!("owner-terminal:{}", pin.path.display())
    {
        return Err(Error::TrainingHeld);
    }
    Ok(())
}
fn owner_decision(plan: &CompletionPlan) -> &'static str {
    if plan.config.spec.kept_cells == 16 {
        "RELEASE_D4_FOR_THIS_K16_REDIRECT_ARM_SEED_ONLY"
    } else {
        "RELEASE_D4_FOR_THIS_COMPLETION_SEED_ONLY"
    }
}
fn admission_schema(plan: &CompletionPlan) -> &'static str {
    if plan.config.spec.kept_cells == 16 {
        "uor-r4.redirect-owner-admission/1"
    } else {
        "uor-r4.completion-owner-admission/1"
    }
}
fn imports_pilot(config: &DriverConfig) -> bool {
    config.spec.kept_cells == 8 && config.seed == 1
}
/// Casey personally at a live terminal only. No pilot delegation is inherited.
pub fn owner_release(plan_path: &Path, output: &Path) -> Result<()> {
    use std::io::IsTerminal;
    if std::env::var_os("UOR_LAB").is_some()
        || !std::io::stdin().is_terminal()
        || !std::io::stdout().is_terminal()
    {
        return Err(invalid("completion owner release is Casey-personal only; no lab or D33 pilot delegation may invoke it"));
    }
    let plan: CompletionPlan = read_json(plan_path)?;
    validate_plan(&plan)?;
    require_disjoint_directories(output, &plan.run_state_directory)?;
    if let Some(pin) = &plan.pilot_parent {
        require_disjoint_directories(
            output,
            pin.path
                .parent()
                .ok_or_else(|| invalid("pilot checkpoint directory missing"))?,
        )?;
    }
    if output.exists() || plan.run_state_directory.exists() {
        return Err(invalid(
            "completion release and run-state directories must be fresh",
        ));
    }
    let phrase = if plan.config.spec.kept_cells == 16 {
        format!(
            "I am Casey; release D4 for {} only through batch 16404",
            plan.scope
        )
    } else {
        format!(
            "I am Casey; release D4 for completion seed {} only through batch 16404",
            plan.config.seed
        )
    };
    println!(
        "{}\nType exactly: {phrase}\nEvery other training scope and M2 remain HELD.",
        serde_json::to_string_pretty(&plan).map_err(|e| invalid(e.to_string()))?
    );
    std::io::stdout()
        .flush()
        .map_err(|e| invalid(e.to_string()))?;
    let mut typed = String::new();
    std::io::stdin()
        .read_line(&mut typed)
        .map_err(|e| invalid(e.to_string()))?;
    if typed.trim_end() != phrase {
        return Err(Error::TrainingHeld);
    }
    fs::create_dir(output).map_err(|e| invalid(e.to_string()))?;
    let output = fs::canonicalize(output).map_err(|e| invalid(e.to_string()))?;
    let path = output.join("owner.json");
    let owner = CompletionOwner {
        owner: "Casey".into(),
        decision: owner_decision(&plan).into(),
        scope: plan.scope.clone(),
        plan_sha256: corpus::sha256(&encode(&plan)?),
        receipt_url: format!("owner-terminal:{}", path.display()),
    };
    write_new(&path, &owner)?;
    let admission = CompletionAdmission {
        schema: admission_schema(&plan).into(),
        plan,
        owner_release: FilePin {
            path,
            sha256: corpus::sha256(&encode(&owner)?),
        },
    };
    write_new(&output.join("admission.json"), &admission)?;
    sync_directory(&output)
}
fn release(
    driver: &PreparedDriver,
    a: &CompletionAdmission,
    compiled: &Value,
    executable: &str,
) -> Result<ValidatedRelease> {
    validate_plan(&a.plan)?;
    if a.schema != admission_schema(&a.plan)
        || a.plan.config != *driver.config()
        || a.plan.execution.lineage_sha256 != driver.lineage_sha256()?
        || a.plan.execution.executable_sha256 != executable
    {
        return Err(invalid("completion executable/lineage/admission mismatch"));
    }
    for (key, want) in [
        ("source_sha256", &a.plan.execution.source_sha256),
        ("kernel_sha256", &a.plan.execution.kernel_sha256),
        ("geometry_sha256", &a.plan.execution.geometry_sha256),
    ] {
        if compiled[key].as_str() != Some(want.as_str()) {
            return Err(invalid(format!("completion compiled {key} mismatch")));
        }
    }
    let owner: CompletionOwner =
        serde_json::from_slice(&a.owner_release.read()?).map_err(|e| invalid(e.to_string()))?;
    validate_owner(&a.plan, &a.owner_release, &owner)?;
    Ok(ValidatedRelease {
        manifest: ReleaseManifest {
            schema: a.schema.clone(),
            scope: a.plan.scope.clone(),
            execution: a.plan.execution.clone(),
            sheet: a.plan.sheet.clone(),
            claude_review: a.plan.claude_review.clone(),
            owner_release: a.owner_release.clone(),
            run_state_directory: a.plan.run_state_directory.clone(),
        },
        manifest_sha256: corpus::sha256(&encode(a)?),
        max_batches: COMPLETION_BATCHES,
    })
}
/// The only permitted lineage delta is the operational driver-source hash. Model bytes,
/// model hash, cursor, config, corpora and role/window bindings are unchanged.
fn migrate_parent(
    p: &PilotCheckpoint,
    lineage: &training::Lineage,
) -> Result<training::DriverCheckpoint> {
    if corpus::sha256(&encode(&p.checkpoint.model)?) != p.checkpoint.model_sha256
        || p.checkpoint.lineage.driver_source_sha256 != OLD_DRIVER
        || corpus::sha256(&encode(&p.checkpoint.lineage)?) != OLD_LINEAGE
    {
        return Err(invalid("migration source is not the pinned pilot lineage"));
    }
    let mut expected = p.checkpoint.lineage.clone();
    expected.driver_source_sha256 = training::driver_source_sha256();
    if &expected != lineage {
        return Err(invalid(
            "completion migration may change only operational driver hash",
        ));
    }
    let mut result = p.checkpoint.clone();
    result.lineage = expected;
    Ok(result)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompletionCheckpoint {
    schema: String,
    admission_sha256: String,
    pilot_parent: Option<FilePin>,
    checkpoint: training::DriverCheckpoint,
    baseline_evaluation: Value,
    baseline_snapshot_metadata: Value,
    vector: KindCounters,
    route: KindCounters,
}
fn validate_progress(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    p: &CompletionCheckpoint,
) -> Result<()> {
    driver.validate_checkpoint(&p.checkpoint)?;
    let n = p.checkpoint.cursor.completed_batches;
    let imports_pilot = imports_pilot(driver.config());
    if p.schema != "uor-r4.completion-checkpoint/1"
        || p.admission_sha256 != release.manifest_sha256
        || n > COMPLETION_BATCHES
        || (imports_pilot && n < 1640)
        || p.vector.attempted != n.min(16384)
        || p.route.attempted != n.saturating_sub(16384)
        || p.vector.accepted_updates + p.route.accepted_updates
            != p.checkpoint.cursor.accepted_updates
        || p.baseline_evaluation["model_epoch"] != json!(0)
        || p.baseline_evaluation["evaluation_lineage_sha256"]
            != json!(if imports_pilot {
                OLD_LINEAGE.to_owned()
            } else {
                driver.lineage_sha256()?
            })
    {
        return Err(invalid(
            "completion progress cursor/baseline/provenance/counters mismatch",
        ));
    }
    for c in [&p.vector, &p.route] {
        if c.accepted_updates > c.any_candidate_cost_differs_from_identity
            || c.any_candidate_cost_differs_from_identity > c.attempted
        {
            return Err(invalid("completion sensitivity counter ordering invalid"));
        }
    }
    if imports_pilot {
        if p.pilot_parent.as_ref().map(|p| p.sha256.as_str()) != Some(PARENT_SHA256) {
            return Err(invalid("completion lost exact pilot parent"));
        }
    } else if p.pilot_parent.is_some() {
        return Err(invalid("fresh completion cannot import a pilot parent"));
    }
    Ok(())
}
/// Publish only fully written, synced JSON under the run lock. Pending files are
/// never reclaimed here: they may be evidence from an interrupted attempt. Only
/// latest.json is replaceable; immutable initial/evaluation names refuse reuse.
fn publish_state_json<T: Serialize>(path: &Path, value: &T, replace_latest: bool) -> Result<()> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let directory = path
        .parent()
        .ok_or_else(|| invalid("state publication requires a parent directory"))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| invalid("state publication requires a UTF8 basename"))?;
    if replace_latest && name != "latest.json" {
        return Err(invalid("only latest.json may be replaced"));
    }
    let require_absent = || -> Result<()> {
        match fs::symlink_metadata(path) {
            Ok(_) => Err(invalid(format!(
                "immutable state already exists: {}",
                path.display()
            ))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(invalid(e.to_string())),
        }
    };
    if !replace_latest {
        require_absent()?;
    }
    let bytes = encode(value)?;
    let (pending, mut file) = loop {
        let pending = directory.join(format!(
            ".{name}.pending-{}-{}-{}",
            std::process::id(),
            unix_ns()?,
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)
        {
            Ok(file) => break (pending, file),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(invalid(e.to_string())),
        }
    };
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| invalid(e.to_string()))?;
    drop(file);
    // The run lock serializes publishers; check again immediately before rename.
    // On any error preserve both the original destination and our pending file.
    if !replace_latest {
        require_absent()?;
    }
    fs::rename(&pending, path).map_err(|e| invalid(e.to_string()))?;
    sync_directory(directory)
}
#[cfg(feature = "cuda")]
fn save(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    out: &Path,
    p: &CompletionCheckpoint,
) -> Result<()> {
    validate_progress(driver, release, p)?;
    let path = out.join(format!(
        "checkpoint-{:05}.json",
        p.checkpoint.cursor.completed_batches
    ));
    write_new(&path, p)?;
    sync_directory(out)?;
    let pin = FilePin {
        path: fs::canonicalize(&path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&encode(p)?),
    };
    let state = &release.manifest.run_state_directory;
    if !state.join("initial.json").exists() {
        publish_state_json(&state.join("initial.json"), &pin, false)?;
    }
    publish_state_json(&state.join("latest.json"), &pin, true)
}
fn validate_evaluation(
    report: &Value,
    release: &ValidatedRelease,
    p: &CompletionCheckpoint,
) -> Result<()> {
    let n = p.checkpoint.cursor.completed_batches;
    if !CHECKPOINTS.contains(&n)
        || report["schema"] != json!("uor-r4.completion-evaluation/1")
        || report["admission_sha256"] != json!(release.manifest_sha256)
        || report["completed_batches"] != json!(n)
        || report["driver_checkpoint_sha256"] != json!(corpus::sha256(&encode(&p.checkpoint)?))
        || report["cursor"]
            != serde_json::to_value(p.checkpoint.cursor).map_err(|e| invalid(e.to_string()))?
        || report["evaluation"]["evaluation_lineage_sha256"] != json!(release.lineage_sha256())
        || report["evaluation"]["model_epoch"] != json!(p.checkpoint.cursor.accepted_updates)
        || report["evaluation"]["count_reference"] != p.baseline_evaluation["count_reference"]
        || report["sensitivity"] != json!({"vector_block":p.vector,"route":p.route})
    {
        return Err(invalid(
            "persisted evaluation cursor/model/admission/reference mismatch",
        ));
    }
    Ok(())
}
fn read_evaluation(
    path: &Path,
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    baseline: &Value,
    expected_cursor: u64,
) -> Result<FilePin> {
    let evaluation_pin: FilePin = read_json(&path.with_extension("pin.json"))?;
    if evaluation_pin.path != fs::canonicalize(path).map_err(|e| invalid(e.to_string()))? {
        return Err(invalid(
            "evaluation pin path differs from registered cursor",
        ));
    }
    let bytes = evaluation_pin.read()?;
    let report: Value = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    let checkpoint_pin: FilePin =
        serde_json::from_value(report["checkpoint"].clone()).map_err(|e| invalid(e.to_string()))?;
    let checkpoint: CompletionCheckpoint =
        serde_json::from_slice(&checkpoint_pin.read()?).map_err(|e| invalid(e.to_string()))?;
    validate_progress(driver, release, &checkpoint)?;
    if checkpoint.checkpoint.cursor.completed_batches != expected_cursor {
        return Err(invalid("wrong quarter checkpoint in evaluation"));
    }
    if &checkpoint.baseline_evaluation != baseline {
        return Err(invalid("evaluation checkpoint replaced original baseline"));
    }
    validate_evaluation(&report, release, &checkpoint)?;
    Ok(FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&bytes),
    })
}
#[cfg(feature = "cuda")]
fn score(
    driver: &PreparedDriver,
    release: &ValidatedRelease,
    model: &crate::device::Device,
    p: &CompletionCheckpoint,
) -> Result<FilePin> {
    let state = &release.manifest.run_state_directory;
    let n = p.checkpoint.cursor.completed_batches;
    if !CHECKPOINTS.contains(&n) {
        return Err(invalid("unregistered completion evaluation cursor"));
    }
    let path = state.join(format!("evaluation-{n:05}.json"));
    if path.exists() {
        let report: Value = read_json(&path)?;
        validate_evaluation(&report, release, p)?;
    } else {
        let checkpoint: FilePin = read_json(&state.join("latest.json"))?;
        if checkpoint.read()? != encode(p)? {
            return Err(invalid(
                "evaluation requires exact persisted current checkpoint",
            ));
        }
        let evaluation = driver.evaluate(model)?;
        let value = json!({"schema":"uor-r4.completion-evaluation/1","admission_sha256":release.manifest_sha256,"completed_batches":n,"cursor":p.checkpoint.cursor,"checkpoint":checkpoint,"driver_checkpoint_sha256":corpus::sha256(&encode(&p.checkpoint)?),"evaluation":evaluation,"sensitivity":{"vector_block":p.vector,"route":p.route},"snapshot_metadata":training::snapshot_metadata(&p.checkpoint.model)?});
        validate_evaluation(&value, release, p)?;
        publish_state_json(&path, &value, false)?;
    }
    // Seal the successful durable result separately. A crash before this pin is written
    // can be recovered at the same quarter only after all metadata checks above pass.
    let pin_path = path.with_extension("pin.json");
    if !pin_path.exists() {
        let bytes = fs::read(&path).map_err(|e| invalid(e.to_string()))?;
        publish_state_json(
            &pin_path,
            &FilePin {
                path: fs::canonicalize(&path).map_err(|e| invalid(e.to_string()))?,
                sha256: corpus::sha256(&bytes),
            },
            false,
        )?;
    }
    read_evaluation(&path, driver, release, &p.baseline_evaluation, n)
}
/// Each invocation operates on precisely one owner-released seed. Never chains to seed 2.
#[cfg(feature = "cuda")]
pub fn run(config: DriverConfig, a: CompletionAdmission, out: &Path, resume: bool) -> Result<()> {
    let started = std::time::Instant::now();
    validate_config(&config)?;
    let driver = PreparedDriver::load(config)?;
    let executable = std::env::current_exe().map_err(|e| invalid(e.to_string()))?;
    let release = release(
        &driver,
        &a,
        &crate::device::compiled_identity(),
        &corpus::sha256(&fs::read(executable).map_err(|e| invalid(e.to_string()))?),
    )?;
    require_disjoint_directories(out, &release.manifest.run_state_directory)?;
    if let Some(pin) = &a.plan.pilot_parent {
        require_disjoint_directories(
            out,
            pin.path
                .parent()
                .ok_or_else(|| invalid("pilot checkpoint directory missing"))?,
        )?;
    }
    fs::create_dir(out).map_err(|e| invalid(format!("exclusive completion output: {e}")))?;
    let _lock = lock_run(&release, resume)?;
    write_new(&out.join("admission.json"), &a)?;
    let result = (|| -> Result<()> {
        let (mut model, mut p) = if resume {
            let pin: FilePin =
                read_json(&release.manifest.run_state_directory.join("latest.json"))?;
            let p: CompletionCheckpoint =
                serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))?;
            validate_progress(&driver, &release, &p)?;
            if let Some(original) = parent(&a.plan)? {
                if p.baseline_evaluation != original.baseline_evaluation
                    || p.baseline_snapshot_metadata != original.baseline_snapshot_metadata
                    || p.pilot_parent != a.plan.pilot_parent
                {
                    return Err(invalid(
                        "seed 1 resume must retain original archived baseline and parent verbatim",
                    ));
                }
            }
            let initial_pin: FilePin =
                read_json(&release.manifest.run_state_directory.join("initial.json"))?;
            let initial: CompletionCheckpoint =
                serde_json::from_slice(&initial_pin.read()?).map_err(|e| invalid(e.to_string()))?;
            validate_progress(&driver, &release, &initial)?;
            if initial.baseline_evaluation != p.baseline_evaluation
                || initial.baseline_snapshot_metadata != p.baseline_snapshot_metadata
                || initial.pilot_parent != p.pilot_parent
                || initial.checkpoint.cursor.completed_batches
                    != if imports_pilot(driver.config()) {
                        1640
                    } else {
                        0
                    }
            {
                return Err(invalid(
                    "completion resume baseline/parent differs from immutable initial",
                ));
            }
            write_new(&out.join("resume-parent.json"), &pin)?;
            let (model, _) = driver.restore(p.checkpoint.clone(), 0)?;
            (model, p)
        } else if let Some(parent) = parent(&a.plan)? {
            let checkpoint = migrate_parent(&parent, driver.lineage())?;
            driver.validate_checkpoint(&checkpoint)?;
            write_new(
                &out.join("lineage-migration.json"),
                &json!({"pilot_parent":a.plan.pilot_parent,"old_lineage":parent.checkpoint.lineage,"new_lineage":checkpoint.lineage,"unchanged_model_sha256":checkpoint.model_sha256,"unchanged_cursor":checkpoint.cursor,"baseline_provenance":"original pilot evaluation retained verbatim; completion evaluations use new operational lineage"}),
            )?;
            let (model, _) = driver.restore(checkpoint.clone(), 0)?;
            (
                model,
                CompletionCheckpoint {
                    schema: "uor-r4.completion-checkpoint/1".into(),
                    admission_sha256: release.manifest_sha256.clone(),
                    pilot_parent: a.plan.pilot_parent.clone(),
                    checkpoint,
                    baseline_evaluation: parent.baseline_evaluation,
                    baseline_snapshot_metadata: parent.baseline_snapshot_metadata,
                    vector: parent.vector,
                    route: KindCounters::default(),
                },
            )
        } else {
            let model = driver.new_model(0)?;
            let checkpoint = driver.checkpoint(&model, driver.initial_cursor())?;
            let baseline_snapshot_metadata = training::snapshot_metadata(&checkpoint.model)?;
            let baseline_evaluation = driver.evaluate(&model)?;
            (
                model,
                CompletionCheckpoint {
                    schema: "uor-r4.completion-checkpoint/1".into(),
                    admission_sha256: release.manifest_sha256.clone(),
                    pilot_parent: None,
                    checkpoint,
                    baseline_evaluation,
                    baseline_snapshot_metadata,
                    vector: KindCounters::default(),
                    route: KindCounters::default(),
                },
            )
        };
        save(&driver, &release, out, &p)?;
        // A saved quarter checkpoint without an evaluation resumes by scoring it before apply.
        if CHECKPOINTS.contains(&p.checkpoint.cursor.completed_batches) {
            score(&driver, &release, &model, &p)?;
        }
        // Never proceed with a missing or changed previously due evaluation. An unsealed
        // current quarter is recovered above; older quarters must already be immutable.
        for n in CHECKPOINTS
            .into_iter()
            .filter(|n| *n <= p.checkpoint.cursor.completed_batches)
        {
            read_evaluation(
                &release
                    .manifest
                    .run_state_directory
                    .join(format!("evaluation-{n:05}.json")),
                &driver,
                &release,
                &p.baseline_evaluation,
                n,
            )?;
        }
        baseline_authentication::require_before_apply(&driver, &release, &a.plan, &p)?;
        if a.plan.redirect.is_some() {
            redirect::require_initial(&driver, &release, &a.plan, &p)?;
        }
        let admission = training::issue_corpus_update_admission(&driver, &release)?;
        let mut receipts = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(out.join("steps.jsonl"))
            .map_err(|e| invalid(e.to_string()))?;
        while p.checkpoint.cursor.completed_batches < COMPLETION_BATCHES {
            let receipt = driver.transition(&mut model, &p.checkpoint.cursor, &admission)?;
            receipts
                .write_all(&encode(&receipt)?)
                .and_then(|_| receipts.write_all(b"\n"))
                .and_then(|_| receipts.sync_all())
                .map_err(|e| invalid(e.to_string()))?;
            let counter = match receipt.coordinate.kind {
                training::CoordinateKind::VectorBlock => &mut p.vector,
                training::CoordinateKind::Route => &mut p.route,
                _ => return Err(invalid("completion never admits flock gates")),
            };
            counter.attempted += 1;
            counter.any_candidate_cost_differs_from_identity +=
                u64::from(receipt.any_candidate_cost_differs_from_identity);
            counter.accepted_updates += u64::from(receipt.applied);
            p.checkpoint.cursor = receipt.cursor_after;
            let n = receipt.cursor_after.completed_batches;
            if n % 64 == 0 || CHECKPOINTS.contains(&n) {
                p.checkpoint = driver.checkpoint(&model, receipt.cursor_after)?;
                save(&driver, &release, out, &p)?;
            }
            if CHECKPOINTS.contains(&n) {
                score(&driver, &release, &model, &p)?;
            }
        }
        let evaluations = CHECKPOINTS
            .iter()
            .map(|n| {
                let path = release
                    .manifest
                    .run_state_directory
                    .join(format!("evaluation-{n:05}.json"));
                read_evaluation(&path, &driver, &release, &p.baseline_evaluation, *n)
            })
            .collect::<Result<Vec<_>>>()?;
        write_new(
            &out.join("report.json"),
            &json!({"schema":"uor-r4.completion-report/1","status":"COMPLETE_PENDING_REGISTERED_TWO_SEED_INTERPRETATION","seed":driver.config().seed,"scope":a.plan.scope,"admission_sha256":release.manifest_sha256,"completed_batches":COMPLETION_BATCHES,"eligible_train_target_exposures":1049856,"baseline_evaluation":p.baseline_evaluation,"baseline_snapshot_metadata":p.baseline_snapshot_metadata,"pilot_parent":p.pilot_parent,"sensitivity":{"vector_block":p.vector,"route":p.route},"evaluations":evaluations,"attempt_elapsed_ns":started.elapsed().as_nanos().to_string(),"fresh_transfer":"UNAVAILABLE_NOT_RUN","next_seed":"SEPARATE_OWNER_RELEASE_REQUIRED","m2":"HELD"}),
        )?;
        sync_directory(out)
    })();
    if let Err(e) = &result {
        write_new(
            &out.join("failure.json"),
            &json!({"status":"INSTRUMENT_OR_EXECUTION_FAILURE","error":e.to_string(),"mechanism_negative":false,"attempt_elapsed_ns":started.elapsed().as_nanos().to_string(),"resume":"latest durable checkpoint; persisted evaluation reused at exact cursor","training":"no further scope authorized"}),
        )?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn old_parent() -> Result<PilotCheckpoint> {
        let config = pilot_config()?;
        let binding = |source: corpus::CorpusSource, documents| corpus::CorpusBinding {
            vocabulary: 4096,
            bos: config.bos.clone(),
            payload_tokens: source.expected_payload_tokens,
            mask_bytes: source.expected_payload_tokens,
            mask_selected_positions: source.expected_eligible_targets,
            eligible_targets: source.expected_eligible_targets,
            documents,
            window_policy: corpus::WINDOW_POLICY.into(),
            identity_policy: corpus::IDENTITY_POLICY.into(),
            source,
        };
        let lineage = training::Lineage {
            schema: training::DRIVER_SCHEMA.into(),
            driver_source_sha256: OLD_DRIVER.into(),
            config: config.clone(),
            train: binding(config.train.clone(), 60000),
            dev: binding(config.dev.clone(), 400),
        };
        assert_eq!(corpus::sha256(&encode(&lineage)?), OLD_LINEAGE);
        // A serialized metadata sentinel, not initialized model state or a host model.
        let model = json!({"metadata_fixture":"no numerical computation"});
        Ok(PilotCheckpoint {
            schema: "uor-r4.pilot-checkpoint/1".into(),
            manifest_sha256: "a".repeat(64),
            checkpoint: training::DriverCheckpoint {
                schema: training::CHECKPOINT_SCHEMA.into(),
                lineage,
                cursor: training::Cursor {
                    completed_batches: 1640,
                    accepted_updates: 236,
                    corpus_pass: 0,
                    next_target: 104960,
                    next_coordinate: 1640,
                    block: None,
                },
                model_sha256: corpus::sha256(&encode(&model)?),
                model,
            },
            baseline_evaluation: json!({"model_epoch":0,"evaluation_lineage_sha256":OLD_LINEAGE}),
            baseline_snapshot_metadata: json!({"distinct_full_vector_patterns":16}),
            vector: KindCounters {
                attempted: 1640,
                accepted_updates: 236,
                any_candidate_cost_differs_from_identity: 1179,
            },
        })
    }
    #[test]
    fn completion_migration_changes_only_operational_lineage() -> Result<()> {
        let p = old_parent()?;
        let before = encode(&p)?;
        let mut new = p.checkpoint.lineage.clone();
        new.driver_source_sha256 = training::driver_source_sha256();
        let migrated = migrate_parent(&p, &new)?;
        assert_eq!(migrated.model, p.checkpoint.model);
        assert_eq!(migrated.model_sha256, p.checkpoint.model_sha256);
        assert_eq!(migrated.cursor, p.checkpoint.cursor);
        assert_eq!(encode(&p)?, before);
        for case in 0..6 {
            let mut wrong = new.clone();
            match case {
                0 => wrong.config.seed = 2,
                1 => wrong.config.batch_rows = 32,
                2 => wrong.train.window_policy = "changed".into(),
                3 => wrong.dev.source.mask.sha256 = "a".repeat(64),
                4 => wrong.driver_source_sha256 = OLD_DRIVER.into(),
                _ => wrong.train.identity_policy = "changed".into(),
            }
            assert!(migrate_parent(&p, &wrong).is_err(), "{case}");
        }
        let mut damaged = p.clone();
        damaged.checkpoint.model = json!({"changed":true});
        assert!(migrate_parent(&damaged, &new).is_err());
        Ok(())
    }
    #[test]
    fn completion_seed_scopes_never_expand_pilot_or_chain_seeds() -> Result<()> {
        let mut config = pilot_config()?;
        validate_config(&config)?;
        validate_pilot_config(&config)?;
        config.seed = 2;
        validate_config(&config)?;
        assert!(validate_pilot_config(&config).is_err());
        assert_ne!(scope(1)?, scope(2)?);
        assert_ne!(scope(1)?, PILOT_SCOPE);
        config.seed = 3;
        assert!(validate_config(&config).is_err());
        config = pilot_config()?;
        config.train.tokens.path = "other/path".into();
        assert!(validate_config(&config).is_err());
        assert_eq!(CHECKPOINTS, [4101, 8202, 12303, 16404]);
        assert_eq!(PILOT_BATCHES, 1640);
        Ok(())
    }
    #[test]
    fn completion_owner_fixture_is_memory_only_and_binds_entire_plan() -> Result<()> {
        let original: OwnerReleasePlan = serde_json::from_str(include_str!(
            "../../../docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let mut config = original.config;
        config.seed = 2;
        let plan = CompletionPlan {
            schema: "uor-r4.completion-owner-release-plan/1".into(),
            status: "HELD_OWNER_ACTION_REQUIRED".into(),
            scope: scope(2)?.into(),
            execution: original.execution,
            config,
            sheet: original.sheet,
            claude_review: original.claude_review,
            run_state_directory: PathBuf::from("/no-owner-record-is-written"),
            pilot_parent: None,
            redirect: None,
        };
        let pin = FilePin {
            path: PathBuf::from("/never-created-owner-record"),
            sha256: "0".repeat(64),
        };
        let owner = CompletionOwner {
            owner: "Casey".into(),
            decision: "RELEASE_D4_FOR_THIS_COMPLETION_SEED_ONLY".into(),
            scope: plan.scope.clone(),
            plan_sha256: corpus::sha256(&encode(&plan)?),
            receipt_url: format!("owner-terminal:{}", pin.path.display()),
        };
        validate_owner(&plan, &pin, &owner)?;
        let mut wrong = plan.clone();
        wrong.config.seed = 1;
        wrong.scope = scope(1)?.into();
        assert!(validate_owner(&wrong, &pin, &owner).is_err());
        let mut wrong = plan.clone();
        wrong.run_state_directory = "/different-run".into();
        assert!(validate_owner(&wrong, &pin, &owner).is_err());
        let mut wrong = owner.clone();
        wrong.decision = "RELEASE_D4_FOR_A2_PILOT_ONLY".into();
        assert!(validate_owner(&plan, &pin, &wrong).is_err());
        let mut wrong = owner.clone();
        wrong.owner = "Claude".into();
        assert!(validate_owner(&plan, &pin, &wrong).is_err());
        assert!(!pin.path.exists());
        Ok(())
    }
    #[test]
    fn pilot_owner_record_is_not_a_completion_release() -> Result<()> {
        let old: OwnerReleasePlan = serde_json::from_str(include_str!(
            "../../../docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let pilot_owner = OwnerRecord {
            owner: "Casey".into(),
            decision: "RELEASE_D4_FOR_A2_PILOT_ONLY".into(),
            scope: PILOT_SCOPE.into(),
            execution: old.execution,
            claude_review_sha256: old.claude_review.sha256,
            receipt_url: "owner-terminal:/not-created".into(),
        };
        assert!(serde_json::from_slice::<CompletionOwner>(&encode(&pilot_owner)?).is_err());
        assert!(serde_json::from_slice::<CompletionPlan>(&encode(&old.config)?).is_err());
        Ok(())
    }
    #[test]
    fn quarter_payload_requires_exact_cursor_epoch_reference_and_lineage() -> Result<()> {
        let old = old_parent()?;
        let plan: OwnerReleasePlan = serde_json::from_str(include_str!(
            "../../../docs/labs/rung0-pilot-release-2026-10-14/PILOT_PLAN.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        let mut checkpoint = old.checkpoint;
        checkpoint.lineage.driver_source_sha256 = training::driver_source_sha256();
        checkpoint.cursor = training::Cursor {
            completed_batches: 4101,
            accepted_updates: 300,
            corpus_pass: 0,
            next_target: 262464,
            next_coordinate: 4101,
            block: None,
        };
        let lineage = corpus::sha256(&encode(&checkpoint.lineage)?);
        let mut execution = plan.execution;
        execution.lineage_sha256 = lineage.clone();
        let release = ValidatedRelease {
            manifest: ReleaseManifest {
                schema: "uor-r4.completion-owner-admission/1".into(),
                scope: scope(1)?.into(),
                execution,
                sheet: plan.sheet,
                claude_review: plan.claude_review,
                owner_release: FilePin {
                    path: "/never-written".into(),
                    sha256: "0".repeat(64),
                },
                run_state_directory: "/never-created".into(),
            },
            manifest_sha256: "a".repeat(64),
            max_batches: COMPLETION_BATCHES,
        };
        let p = CompletionCheckpoint {
            schema: "uor-r4.completion-checkpoint/1".into(),
            admission_sha256: release.manifest_sha256.clone(),
            pilot_parent: None,
            checkpoint,
            baseline_evaluation: json!({"count_reference":{"metadata_fixture":1}}),
            baseline_snapshot_metadata: json!({}),
            vector: KindCounters {
                attempted: 4101,
                accepted_updates: 300,
                any_candidate_cost_differs_from_identity: 3200,
            },
            route: KindCounters::default(),
        };
        let report = json!({"schema":"uor-r4.completion-evaluation/1","admission_sha256":release.manifest_sha256,"completed_batches":4101,"driver_checkpoint_sha256":corpus::sha256(&encode(&p.checkpoint)?),"cursor":p.checkpoint.cursor,"evaluation":{"evaluation_lineage_sha256":lineage,"model_epoch":300,"count_reference":{"metadata_fixture":1}},"sensitivity":{"vector_block":p.vector,"route":p.route}});
        validate_evaluation(&report, &release, &p)?;
        for case in 0..8 {
            let mut wrong = report.clone();
            match case {
                0 => wrong["completed_batches"] = json!(8202),
                1 => wrong["cursor"]["next_target"] = json!(0),
                2 => wrong["driver_checkpoint_sha256"] = json!("0".repeat(64)),
                3 => wrong["evaluation"]["model_epoch"] = json!(299),
                4 => wrong["evaluation"]["count_reference"] = json!({}),
                5 => wrong["evaluation"]["evaluation_lineage_sha256"] = json!(OLD_LINEAGE),
                6 => wrong["admission_sha256"] = json!("b".repeat(64)),
                _ => wrong["sensitivity"]["route"]["attempted"] = json!(1),
            }
            assert!(validate_evaluation(&wrong, &release, &p).is_err(), "{case}");
        }
        Ok(())
    }
    #[test]
    fn atomic_state_publication_preserves_stray_pending_and_existing_final() -> Result<()> {
        let root =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local/controller-metadata-tests");
        fs::create_dir_all(&root).map_err(|e| invalid(e.to_string()))?;
        let root = root.join(format!(
            "atomic-state-{}-{}",
            std::process::id(),
            unix_ns()?
        ));
        fs::create_dir(&root).map_err(|e| invalid(e.to_string()))?;
        for name in [
            "initial.json",
            "evaluation-04101.json",
            "evaluation-04101.pin.json",
        ] {
            let stray = root.join(format!(".{name}.pending-{}-0-0", std::process::id()));
            fs::write(&stray, b"interrupted partial metadata")
                .map_err(|e| invalid(e.to_string()))?;
            let final_path = root.join(name);
            publish_state_json(&final_path, &json!({"generation":1}), false)?;
            assert_eq!(read_json::<Value>(&final_path)?, json!({"generation":1}));
            assert_eq!(
                fs::read(&stray).map_err(|e| invalid(e.to_string()))?,
                b"interrupted partial metadata"
            );
            let original = fs::read(&final_path).map_err(|e| invalid(e.to_string()))?;
            assert!(publish_state_json(&final_path, &json!({"generation":2}), false).is_err());
            assert_eq!(
                fs::read(&final_path).map_err(|e| invalid(e.to_string()))?,
                original
            );
            assert!(publish_state_json(&final_path, &json!({"generation":2}), true).is_err());
        }
        // Pointer replacement is the sole intentional mutation; stale PID-only pending
        // names from the earlier controller cannot prevent a new durable publication.
        let old_pending = root.join(format!("latest-{}.pending", std::process::id()));
        fs::write(&old_pending, b"old interrupted pointer").map_err(|e| invalid(e.to_string()))?;
        let latest = root.join("latest.json");
        publish_state_json(&latest, &json!({"generation":1}), true)?;
        publish_state_json(&latest, &json!({"generation":2}), true)?;
        assert_eq!(read_json::<Value>(&latest)?, json!({"generation":2}));
        assert_eq!(
            fs::read(&old_pending).map_err(|e| invalid(e.to_string()))?,
            b"old interrupted pointer"
        );
        fs::remove_dir_all(root).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}
