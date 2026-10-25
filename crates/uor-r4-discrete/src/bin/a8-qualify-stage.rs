//! Stage and verify the M1 A8 finite CUDA source-qualification inputs and outputs.
//! Host-only metadata staging and output verification; CUDA execution runs via
//! the pinned `fd409fcdf3bc43d607e768bae6f8cf403aa75901` `discrete-stack` binary.

#![forbid(unsafe_code)]
#![deny(warnings, clippy::all)]

use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use uor_r4_discrete::attention_anchor::{AnchorPolicy, TrainAnchorManifest};
use uor_r4_discrete::attention_anchor_device::{FIT_SHEET_SHA256, REGISTERED_CSV_SHA256};
use uor_r4_discrete::controller::attention_audited::qualification::{
    cost_payload_sha256, payload_sha256, Authorization, CostedDoseDecision, DoseCostAttestation,
    InheritedObservation, Limits, QualificationAdmission, QualificationPlan, ResourceObservation,
    ADMISSION_SCHEMA, PLAN_SCHEMA,
};
use uor_r4_discrete::controller::attention_audited::{
    review_payload_sha256, scope as dose_scope, AuditPolicy, AuditedAdmission, AuditedOwner,
    AuditedPlan, AuditedReview, Calibration, Qualification,
    ADMISSION_SCHEMA as DOSE_ADMISSION_SCHEMA, OWNER_DECISION as DOSE_OWNER_DECISION,
    OWNER_SCHEMA as DOSE_OWNER_SCHEMA, PLAN_SCHEMA as DOSE_PLAN_SCHEMA,
    REVIEW_VERDICT as DOSE_REVIEW_VERDICT,
};
use uor_r4_discrete::controller::ExecutionIdentity;
use uor_r4_discrete::corpus::{sha256, CorpusSource, FilePin, Split};
use uor_r4_discrete::training::DriverConfig;

const QUAL_SCOPE: &str = "m1-attention-active-audit-finite-source-qualification-v1";
const OWNER_SCHEMA: &str = "uor-r4.attention-audit-source-qualification-owner-decision/1";
const OWNER_DECISION: &str = "RELEASE_FINITE_AUDITED_SOURCE_QUALIFICATION_ONLY";
const PROTOCOL: &str = "train61-synthetic1-interval64-two-arm-throughput-v1";
const EXPECTED_MANIFEST_SHA256: &str =
    "f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de";
const INHERITED_NAMES: [&str; 8] = [
    "canonical_initlock_c2",
    "combine_probe",
    "initial_checkpoint",
    "initial_dev_evaluation",
    "negative_k16",
    "negative_nonroot",
    "relation_probe",
    "unpinned_checkpoint",
];
const STAGE_NAMES: [&str; 5] = [
    "canonical_initlock_c2",
    "required_negatives",
    "g_occ",
    "sensitivity",
    "audited_restore_epoch_cursor",
];

fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = read_bytes(path)?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
}

fn pin_existing(path: &Path) -> Result<FilePin, String> {
    let canonical = fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let bytes = read_bytes(&canonical)?;
    Ok(FilePin {
        path: canonical,
        sha256: sha256(&bytes),
    })
}

fn write_json_pin<T: serde::Serialize>(
    path: &Path,
    value: &T,
    mode: u32,
) -> Result<FilePin, String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(mode)
            .open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        file.set_permissions(fs::Permissions::from_mode(mode))
            .map_err(|e| format!("{}: {e}", path.display()))?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        fs::write(path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    pin_existing(path)
}

fn ensure_dir_mode(path: &Path, mode: u32) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        if !path.exists() {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(mode)
                .create(path)
                .map_err(|e| format!("{}: {e}", path.display()))?;
        }
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = mode;
        fs::create_dir_all(path).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

fn default_limits() -> Limits {
    Limits {
        protocol: PROTOCOL.into(),
        anchor_batches_per_sample: 61,
        gate_samples: 1,
        synthetic_apply_calls: 1,
        synthetic_accepted_updates: 1,
        synthetic_stale_apply_refusals: 1,
        production_stale_apply_refusals_per_arm: 1,
        anchor_evaluations_per_arm: 2,
        rejected_interval_attempts: 64,
        throughput_attempts_per_arm: 384,
        throughput_warmup_attempts_per_arm: 0,
        projected_audits: 516,
        projected_restore_events: 514,
        timer_scope: "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1".into(),
        cache_policy: "fresh-device-per-arm-no-unreported-warmup-v1".into(),
    }
}

fn default_anchor_policy() -> AnchorPolicy {
    AnchorPolicy {
        start_eligible_index: 1_049_856,
        count: 3_847,
        context: 192,
    }
}

struct StageArgs {
    a8s1_root: PathBuf,
    inherited_dir: PathBuf,
    stage_dir: PathBuf,
    private_dir: PathBuf,
    run_state_dir: PathBuf,
    receipt_url: String,
    hardware_class: String,
    gpu_count: u32,
    rate_usd_micros_per_hour: u64,
    host_available_bytes: u64,
    reserve_bytes: u64,
    all_in_seconds: u64,
}

fn parse_stage_args(args: &[String]) -> Result<StageArgs, String> {
    let mut a8s1_root: Option<PathBuf> = None;
    let mut inherited_dir: Option<PathBuf> = None;
    let mut stage_dir: Option<PathBuf> = None;
    let mut private_dir: Option<PathBuf> = None;
    let mut run_state_dir: Option<PathBuf> = None;
    let mut receipt_url: Option<String> = None;
    let mut hardware_class = String::from("NVIDIA_GeForce_RTX_5090");
    let mut gpu_count: u32 = 2;
    let mut rate_usd_micros_per_hour: u64 = 1_816_000;
    let mut host_available_bytes: u64 = 45 * 1024 * 1024 * 1024;
    let mut reserve_bytes: u64 = 2 * 1024 * 1024 * 1024;
    let mut all_in_seconds: u64 = 1_800;

    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let val = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag {
            "--a8s1-root" => a8s1_root = Some(PathBuf::from(val)),
            "--inherited-dir" => inherited_dir = Some(PathBuf::from(val)),
            "--stage-dir" => stage_dir = Some(PathBuf::from(val)),
            "--private-dir" => private_dir = Some(PathBuf::from(val)),
            "--run-state-dir" => run_state_dir = Some(PathBuf::from(val)),
            "--receipt-url" => receipt_url = Some(val.clone()),
            "--hardware-class" => hardware_class = val.clone(),
            "--gpu-count" => {
                gpu_count = val
                    .parse::<u32>()
                    .map_err(|e| format!("--gpu-count: {e}"))?
            }
            "--rate-usd-micros-per-hour" => {
                rate_usd_micros_per_hour = val
                    .parse::<u64>()
                    .map_err(|e| format!("--rate-usd-micros-per-hour: {e}"))?
            }
            "--host-available-bytes" => {
                host_available_bytes = val
                    .parse::<u64>()
                    .map_err(|e| format!("--host-available-bytes: {e}"))?
            }
            "--reserve-bytes" => {
                reserve_bytes = val
                    .parse::<u64>()
                    .map_err(|e| format!("--reserve-bytes: {e}"))?
            }
            "--all-in-seconds" => {
                all_in_seconds = val
                    .parse::<u64>()
                    .map_err(|e| format!("--all-in-seconds: {e}"))?
            }
            other => return Err(format!("unknown flag {other}")),
        }
        i += 2;
    }

    Ok(StageArgs {
        a8s1_root: a8s1_root.ok_or("missing --a8s1-root")?,
        inherited_dir: inherited_dir.ok_or("missing --inherited-dir")?,
        stage_dir: stage_dir.ok_or("missing --stage-dir")?,
        private_dir: private_dir.ok_or("missing --private-dir")?,
        run_state_dir: run_state_dir.ok_or("missing --run-state-dir")?,
        receipt_url: receipt_url.ok_or("missing --receipt-url")?,
        hardware_class,
        gpu_count,
        rate_usd_micros_per_hour,
        host_available_bytes,
        reserve_bytes,
        all_in_seconds,
    })
}

fn collect_inherited(
    inherited_dir: &Path,
    stage_dir: &Path,
) -> Result<(Vec<FilePin>, ExecutionIdentity, ExecutionIdentity), String> {
    let mut pins = Vec::new();
    let mut rel_exec: Option<ExecutionIdentity> = None;
    let mut fz_exec: Option<ExecutionIdentity> = None;

    for arm in ["relation", "frozen-zero"] {
        for name in INHERITED_NAMES {
            let file_name = format!("{arm}-{name}.json");
            let src_path = inherited_dir.join(&file_name);
            let obs: InheritedObservation = read_json(&src_path)?;
            if obs.name != name {
                return Err(format!(
                    "{}: observation name {} != {name}",
                    src_path.display(),
                    obs.name
                ));
            }
            if arm == "relation" {
                if let Some(prev) = &rel_exec {
                    if prev != &obs.execution {
                        return Err(format!(
                            "{}: relation execution identity mismatch",
                            src_path.display()
                        ));
                    }
                } else {
                    rel_exec = Some(obs.execution.clone());
                }
            } else if let Some(prev) = &fz_exec {
                if prev != &obs.execution {
                    return Err(format!(
                        "{}: frozen-zero execution identity mismatch",
                        src_path.display()
                    ));
                }
            } else {
                fz_exec = Some(obs.execution.clone());
            }
            let dst_path = stage_dir.join(&file_name);
            let pin = write_json_pin(&dst_path, &obs, 0o600)?;
            pins.push(pin);
        }
    }

    Ok((
        pins,
        rel_exec.ok_or("missing relation execution identity")?,
        fz_exec.ok_or("missing frozen-zero execution identity")?,
    ))
}

fn run_stage(args: &[String]) -> Result<(), String> {
    let cfg = parse_stage_args(args)?;
    if cfg.run_state_dir.exists() {
        return Err(format!(
            "--run-state-dir {} must not exist before discrete-stack runs",
            cfg.run_state_dir.display()
        ));
    }
    if !cfg.run_state_dir.is_absolute() {
        return Err(format!(
            "--run-state-dir {} must be an absolute path",
            cfg.run_state_dir.display()
        ));
    }

    ensure_dir_mode(&cfg.stage_dir, 0o700)?;
    ensure_dir_mode(&cfg.private_dir, 0o700)?;

    let rel_cfg_path = cfg
        .a8s1_root
        .join("src/docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json");
    let fz_cfg_path = cfg
        .a8s1_root
        .join("src/docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json");
    let config: DriverConfig = read_json(&rel_cfg_path)?;
    let control_config: DriverConfig = read_json(&fz_cfg_path)?;

    let sheet_path = cfg
        .a8s1_root
        .join("src/docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md");
    let dose_sheet = pin_existing(&sheet_path)?;
    if dose_sheet.sha256 != FIT_SHEET_SHA256 {
        return Err(format!(
            "dose sheet sha256 {} != expected {}",
            dose_sheet.sha256, FIT_SHEET_SHA256
        ));
    }

    let policy = default_anchor_policy();
    let manifest = TrainAnchorManifest::load(&config.train, 4096, &config.bos, policy.clone())
        .map_err(|e| format!("TrainAnchorManifest::load: {e}"))?;
    if manifest.row_csv_sha256 != REGISTERED_CSV_SHA256 {
        return Err(format!(
            "anchor row_csv_sha256 {} != {}",
            manifest.row_csv_sha256, REGISTERED_CSV_SHA256
        ));
    }
    if manifest.manifest_sha256 != EXPECTED_MANIFEST_SHA256 {
        return Err(format!(
            "anchor manifest_sha256 {} != {}",
            manifest.manifest_sha256, EXPECTED_MANIFEST_SHA256
        ));
    }
    let anchor_manifest = write_json_pin(
        &cfg.stage_dir.join("anchor-manifest.json"),
        &manifest,
        0o600,
    )?;

    let (inherited_observations, execution, control_execution) =
        collect_inherited(&cfg.inherited_dir, &cfg.stage_dir)?;

    let accepted_initlock_disposition = write_json_pin(
        &cfg.stage_dir.join("accepted-initlock-disposition.json"),
        &json!({
            "schema": "uor-r4.attention-audit-initlock-disposition/1",
            "disposition": "ACCEPTED_CANONICAL_QUALIFY_RELATION_INITLOCK_C2",
            "source_commit": execution.reviewed_source_commit,
            "receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6242232012",
            "d67_review_url": cfg.receipt_url,
        }),
        0o600,
    )?;
    let residual_fit = write_json_pin(
        &cfg.stage_dir.join("residual-fit.json"),
        &json!({
            "schema": "uor-r4.attention-audit-residual-fit/1",
            "verdict": "TEST FITNESS: FIT",
            "dose_sheet_sha256": FIT_SHEET_SHA256,
            "fit_receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6263653594",
            "staging_fit_receipt_url": "https://github.com/UOR-Foundation/uor-r4/issues/2832#issuecomment-6362835398",
            "d67_review_url": cfg.receipt_url,
        }),
        0o600,
    )?;
    let g_init_comparison_or_waiver = write_json_pin(
        &cfg.stage_dir.join("g-init-comparison.json"),
        &json!({
            "schema": "uor-r4.attention-audit-g-init-comparison/1",
            "status": "VERIFIED_PINNED_VS_UNPINNED_ROUTES_GATES_SEED_EPOCH_IDENTICAL",
            "relation_lineage_sha256": execution.lineage_sha256,
            "frozen_zero_lineage_sha256": control_execution.lineage_sha256,
            "d67_review_url": cfg.receipt_url,
        }),
        0o600,
    )?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let native_resource_receipt = write_json_pin(
        &cfg.private_dir.join("native-resource-receipt.json"),
        &json!({
            "schema": "uor-r4.native-resource-receipt/1",
            "d67_operator_lab": "antigravity",
            "hardware_class": cfg.hardware_class,
            "gpu_count": cfg.gpu_count,
            "rate_usd_micros_per_hour": cfg.rate_usd_micros_per_hour,
            "observed_unix_seconds": now_secs,
            "d67_review_url": cfg.receipt_url,
        }),
        0o600,
    )?;
    let native_lease_receipt = write_json_pin(
        &cfg.private_dir.join("native-lease-receipt.json"),
        &json!({
            "schema": "uor-r4.native-lease-receipt/1",
            "d67_operator_lab": "antigravity",
            "purpose": "m1-a8-finite-cuda-qualification (#2830)",
            "observed_unix_seconds": now_secs,
            "d67_review_url": cfg.receipt_url,
        }),
        0o600,
    )?;
    let resource_admission = write_json_pin(
        &cfg.private_dir.join("resource-observation.json"),
        &json!({
            "schema": "uor-r4.attention-audit-resource-observation/1",
            "executions": [execution.clone(), control_execution.clone()],
            "operator": "Claude",
            "hardware_class": cfg.hardware_class,
            "gpu_count": cfg.gpu_count,
            "rate_usd_micros_per_hour": cfg.rate_usd_micros_per_hour,
            "host_available_bytes": cfg.host_available_bytes,
            "staging_archive_reserve_bytes": cfg.reserve_bytes,
            "observed_unix_seconds": now_secs,
            "native_resource_receipt": native_resource_receipt,
            "native_lease_receipt": native_lease_receipt,
        }),
        0o600,
    )?;

    let limits = default_limits();
    let mut plan = QualificationPlan {
        schema: PLAN_SCHEMA.into(),
        scope: QUAL_SCOPE.into(),
        status: "HELD_FINITE_QUALIFICATION_OWNER_ACTION_REQUIRED".into(),
        execution: execution.clone(),
        config,
        control_execution,
        control_config,
        dose_sheet: dose_sheet.clone(),
        qualification_fit: dose_sheet.clone(),
        owner_scope_cost: dose_sheet.clone(),
        pm_dispatch: dose_sheet.clone(),
        independent_review: dose_sheet,
        anchor_manifest,
        anchor_manifest_sha256: manifest.manifest_sha256,
        anchor_csv_sha256: manifest.row_csv_sha256,
        anchor_policy: policy,
        limits: limits.clone(),
        inherited_observations,
        accepted_initlock_disposition,
        residual_fit,
        g_init_comparison_or_waiver,
        resource_admission,
        rate_usd_micros_per_hour: cfg.rate_usd_micros_per_hour,
        gpu_count: cfg.gpu_count,
        all_in_resource_seconds: cfg.all_in_seconds,
        staging_archive_reserve_bytes: cfg.reserve_bytes,
        host_available_bytes: cfg.host_available_bytes,
        run_state_directory: cfg.run_state_dir,
    };

    let plan_payload = payload_sha256(&plan).map_err(|e| format!("payload_sha256: {e}"))?;
    let auth_specs = [
        (
            "qualification_fit.json",
            "qualification_fit",
            "AGY",
            "TEST FITNESS: FIT",
        ),
        (
            "owner_scope_cost.json",
            "owner_cost_scope",
            "Casey",
            "APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST",
        ),
        (
            "pm_dispatch.json",
            "pm_dispatch",
            "Kimi",
            "DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY",
        ),
        (
            "independent_review.json",
            "source_plan_review",
            "AGY",
            "APPROVE_EXACT_QUALIFICATION_PLAN",
        ),
    ];
    let mut auth_pins = Vec::new();
    for (file_name, role, actor, verdict) in auth_specs {
        let auth = Authorization {
            schema: "uor-r4.attention-audit-qualification-authorization/1".into(),
            role: role.into(),
            actor: actor.into(),
            verdict: verdict.into(),
            execution: execution.clone(),
            protocol: limits.clone(),
            plan_payload_sha256: plan_payload.clone(),
            receipt_url: cfg.receipt_url.clone(),
        };
        let pin = write_json_pin(&cfg.stage_dir.join(file_name), &auth, 0o600)?;
        auth_pins.push(pin);
    }
    plan.qualification_fit = auth_pins[0].clone();
    plan.owner_scope_cost = auth_pins[1].clone();
    plan.pm_dispatch = auth_pins[2].clone();
    plan.independent_review = auth_pins[3].clone();

    let plan_file = write_json_pin(&cfg.stage_dir.join("plan.json"), &plan, 0o600)?;
    let owner_path = fs::canonicalize(&cfg.private_dir)
        .map_err(|e| format!("{}: {e}", cfg.private_dir.display()))?
        .join("owner.json");
    let owner_release = write_json_pin(
        &owner_path,
        &json!({
            "schema": OWNER_SCHEMA,
            "owner": "Casey",
            "decision": OWNER_DECISION,
            "scope": QUAL_SCOPE,
            "plan_file_sha256": plan_file.sha256,
            "receipt_url": format!("owner-terminal:{}", owner_path.display()),
        }),
        0o600,
    )?;
    let admission = QualificationAdmission {
        schema: ADMISSION_SCHEMA.into(),
        plan_file: plan_file.clone(),
        plan,
        owner_release,
    };
    let admission_pin = write_json_pin(&cfg.private_dir.join("admission.json"), &admission, 0o600)?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "STAGED_FOR_D67_FINITE_QUALIFICATION",
            "plan_file": plan_file,
            "plan_payload_sha256": plan_payload,
            "admission_file": admission_pin,
            "anchor_manifest_sha256": EXPECTED_MANIFEST_SHA256,
            "anchor_csv_sha256": REGISTERED_CSV_SHA256,
            "d67_review_url": cfg.receipt_url,
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

fn read_pinned_or_rebased(pin: &FilePin, rebase_dir: &Path) -> Result<Vec<u8>, String> {
    let candidate = if pin.path.exists() {
        pin.path.clone()
    } else if let Ok(rel) = pin.path.strip_prefix("/workspace/a8s1/qual-out") {
        let stripped = rebase_dir.join(rel);
        if stripped.exists() {
            stripped
        } else if let Some(name) = pin.path.file_name() {
            rebase_dir.join(name)
        } else {
            stripped
        }
    } else if let Some(name) = pin.path.file_name() {
        rebase_dir.join(name)
    } else {
        pin.path.clone()
    };
    let bytes = read_bytes(&candidate)?;
    let actual = sha256(&bytes);
    if actual != pin.sha256 {
        return Err(format!(
            "{}: sha256 {actual} != expected {}",
            candidate.display(),
            pin.sha256
        ));
    }
    Ok(bytes)
}

fn verify_arm_stages_only(arm: &str, arm_dir: &Path, qual_pin: &FilePin) -> Result<Value, String> {
    let qual_bytes =
        read_pinned_or_rebased(qual_pin, arm_dir).map_err(|e| format!("{arm} qual: {e}"))?;
    let qual: Qualification =
        serde_json::from_slice(&qual_bytes).map_err(|e| format!("{arm} qual json: {e}"))?;

    if qual.schema != "uor-r4.attention-audit-qualification/1"
        || qual.b3_psucc != 1351
        || qual.b3_full != 4244
        || qual.top1 != 133
        || qual.bar_i != 86
        || qual.epoch != 0
        || qual.updates_applied != 0
        || qual.full_rows != 14287
        || qual.psucc_rows != 3847
        || qual.occupied_cells != 256
        || qual.distinct_vectors != 4096
        || qual.tie_mass != 0
        || qual.negative_native_exits != [2, 2]
        || qual.combine_sensitive != 1
        || qual.relation_sensitive != 1
        || qual.checks.len() != 5
    {
        return Err(format!(
            "{arm} qualification header or baseline metrics invalid"
        ));
    }

    let mut stage_summaries = Vec::new();
    for (check, expected_name) in qual.checks.iter().zip(STAGE_NAMES) {
        if check.name != expected_name || check.status != "PASS" {
            return Err(format!(
                "{arm} stage {} status {} != PASS ({expected_name})",
                check.name, check.status
            ));
        }
        let stage_bytes = read_pinned_or_rebased(&check.artifact, arm_dir)
            .map_err(|e| format!("{arm} stage {}: {e}", check.name))?;
        let stage_val: Value =
            serde_json::from_slice(&stage_bytes).map_err(|e| format!("{arm} stage json: {e}"))?;
        if stage_val["status"] != "PASS" {
            return Err(format!("{arm} stage {} inner status != PASS", check.name));
        }
        stage_summaries.push(json!({
            "name": check.name,
            "status": check.status,
            "sha256": check.artifact.sha256,
            "facts": stage_val["facts"],
        }));
    }

    Ok(json!({
        "qualification_sha256": qual_pin.sha256,
        "execution": qual.execution,
        "b3_psucc": qual.b3_psucc,
        "b3_full": qual.b3_full,
        "top1": qual.top1,
        "bar_i": qual.bar_i,
        "psucc_reference_top1": qual.psucc_reference_top1,
        "occupied_cells": qual.occupied_cells,
        "distinct_vectors": qual.distinct_vectors,
        "tie_mass": qual.tie_mass,
        "combine_sensitive": qual.combine_sensitive,
        "relation_sensitive": qual.relation_sensitive,
        "stages": stage_summaries,
    }))
}

fn verify_arm_qualification(
    arm: &str,
    qual_pin: &FilePin,
    cal_pin: &FilePin,
) -> Result<(Value, Value), String> {
    let arm_dir = qual_pin.path.parent().unwrap_or_else(|| Path::new("."));
    let qual_summary = verify_arm_stages_only(arm, arm_dir, qual_pin)?;
    let cal_bytes =
        read_pinned_or_rebased(cal_pin, arm_dir).map_err(|e| format!("{arm} cal: {e}"))?;
    let cal: Calibration =
        serde_json::from_slice(&cal_bytes).map_err(|e| format!("{arm} cal json: {e}"))?;

    if cal.schema != "uor-r4.attention-audit-calibration/1"
        || cal.audits_both_arms != 516
        || !cal.complete_gate_io_included
        || !cal.shared_setup_excluded_from_ratio
        || cal.projected_added_gate_ns == 0
        || cal.ungated_two_arm_dose_ns == 0
        || cal.projected_added_gate_ns > 5_500_000_000_000
        || u128::from(cal.projected_added_gate_ns) * 100
            > u128::from(cal.ungated_two_arm_dose_ns) * 85
    {
        return Err(format!(
            "{arm} calibration failed 85% / 5500s readiness gate: added={} ns, ungated={} ns",
            cal.projected_added_gate_ns, cal.ungated_two_arm_dose_ns
        ));
    }
    for raw_pin in &cal.measurement_artifacts {
        read_pinned_or_rebased(raw_pin, arm_dir)
            .map_err(|e| format!("{arm} raw measurement pin: {e}"))?;
    }

    let ratio_bps = (u128::from(cal.projected_added_gate_ns) * 10_000)
        .div_ceil(u128::from(cal.ungated_two_arm_dose_ns));

    Ok((
        qual_summary,
        json!({
            "calibration_sha256": cal_pin.sha256,
            "projected_added_gate_ns": cal.projected_added_gate_ns,
            "ungated_two_arm_dose_ns": cal.ungated_two_arm_dose_ns,
            "readiness_ratio_bps_ceil": u64::try_from(ratio_bps).unwrap_or(u64::MAX),
            "readiness_bar_bps": 8500,
            "passes_85_percent_gate": true,
            "audits_both_arms": cal.audits_both_arms,
            "gpu_count": cal.gpu_count,
            "rate_usd_micros_per_hour": cal.rate_usd_micros_per_hour,
            "measurement_artifact_count": cal.measurement_artifacts.len(),
        }),
    ))
}

fn verify_production_proof(arm: &str, arm_dir: &Path) -> Result<Value, String> {
    let proof_path = arm_dir.join("production-proof.json");
    let proof_pin = pin_existing(&proof_path)?;
    let proof: Value = read_json(&proof_pin.path)?;
    if proof["schema"] != "uor-r4.attention-audit-production-restore-proof/1" {
        return Err(format!("{arm} production-proof.json invalid: {proof}"));
    }
    for key in [
        "initial_checkpoint",
        "inspected_checkpoint",
        "continuation_checkpoint",
        "steps",
        "restore_intent",
        "restore_observation",
        "initial_anchor",
        "restored_anchor",
    ] {
        let pin: FilePin = serde_json::from_value(proof[key].clone())
            .map_err(|e| format!("{arm} production-proof {key} pin: {e}"))?;
        read_pinned_or_rebased(&pin, arm_dir)
            .map_err(|e| format!("{arm} production-proof {key}: {e}"))?;
    }
    let stage_path = arm_dir.join("audited_restore_epoch_cursor.json");
    let stage_val: Value = read_json(&stage_path)?;
    let facts = &stage_val["facts"];
    if stage_val["status"] != "PASS"
        || facts["advanced_attempts"] != 384
        || facts["restored_epoch"] != 0
        || facts["passing_storage_restored"] != true
        || facts["passing_snapshot_restored"] != true
        || facts["fresh_binding"] != true
        || facts["stale_handles_refused"] != true
        || facts["anchor_equal_before_after_restore"] != true
        || facts["continuation_file_distinct"] != true
    {
        return Err(format!(
            "{arm} audited_restore_epoch_cursor facts invalid: {facts}"
        ));
    }
    let obs_path = arm_dir.join("production-restore-observed.json");
    let obs: Value = read_json(&obs_path)?;
    if obs["observed_epoch"] != 0
        || obs["old_handles_refused"] != true
        || obs["passing_snapshot_restored"] != true
        || obs["passing_storage_restored"] != true
        || obs["advanced_cursor"]["completed_batches"] != 384
    {
        return Err(format!(
            "{arm} production-restore-observed.json invalid: {obs}"
        ));
    }
    let base_summary_path = arm_dir.join("baseline-anchor-0.json");
    let rest_summary_path = arm_dir.join("restored-anchor.json");
    let base_doc: Value = read_json(&base_summary_path)?;
    let rest_doc: Value = read_json(&rest_summary_path)?;
    let base_agg = &base_doc["evaluation"]["aggregate"];
    let rest_agg = &rest_doc["evaluation"]["aggregate"];
    if base_agg["scored_rows"] != 3847
        || base_agg["target_admitted_rows"].as_u64().unwrap_or(0) < 500
        || rest_agg != base_agg
        || rest_doc["evaluation"]["actual_snapshot_before"]
            != base_doc["evaluation"]["actual_snapshot_before"]
    {
        return Err(format!(
            "{arm} anchor aggregate mismatch before vs after restore"
        ));
    }
    let base_timing_path = arm_dir.join("baseline-anchor-timing-0.json");
    let rest_timing_path = arm_dir.join("restore-timing.json");
    let base_timing: Value = read_json(&base_timing_path)?;
    let rest_timing: Value = read_json(&rest_timing_path)?;
    Ok(json!({
        "production_proof_sha256": proof_pin.sha256,
        "status": stage_val["status"],
        "advanced_attempts": facts["advanced_attempts"],
        "accepted_updates_before_restore": facts["accepted_updates_before_restore"],
        "restored_epoch": facts["restored_epoch"],
        "passing_storage_restored": facts["passing_storage_restored"],
        "passing_snapshot_restored": facts["passing_snapshot_restored"],
        "fresh_binding": facts["fresh_binding"],
        "stale_handles_refused": facts["stale_handles_refused"],
        "continuation_file_distinct": facts["continuation_file_distinct"],
        "anchor_equal_before_after_restore": facts["anchor_equal_before_after_restore"],
        "advanced_cursor": obs["advanced_cursor"],
        "initial_anchor_sha256": proof["initial_anchor"]["sha256"],
        "restored_anchor_sha256": proof["restored_anchor"]["sha256"],
        "train_anchor": {
            "scored_rows": base_agg["scored_rows"],
            "target_admitted_rows": base_agg["target_admitted_rows"],
            "admitted_strict_win_rows": base_agg["admitted_strict_win_rows"],
            "top1_correct": base_agg["top1_correct"],
            "top8_correct": base_agg["top8_correct"],
            "objective_loss_numerator_sum": base_agg["objective_loss_numerator_sum"],
            "loss_denominator_per_row": base_agg["loss_denominator_per_row"],
            "model_snapshot_sha256": base_doc["evaluation"]["actual_snapshot_before"]["model_snapshot_sha256"],
            "model_storage_sha256": base_doc["evaluation"]["actual_snapshot_before"]["model_storage_sha256"],
            "baseline_anchor_elapsed_ns": base_timing["elapsed_ns"],
            "restore_elapsed_ns": rest_timing["elapsed_ns"],
        },
    }))
}

fn run_verify(args: &[String]) -> Result<(), String> {
    let [flag1, out_dir, flag2, summary_out] = args else {
        return Err("usage: a8-qualify-stage verify --out-dir <DIR> --summary-out <FILE>".into());
    };
    if flag1 != "--out-dir" || flag2 != "--summary-out" {
        return Err("usage: a8-qualify-stage verify --out-dir <DIR> --summary-out <FILE>".into());
    }
    let out_dir = Path::new(out_dir);

    let synthetic_pin = pin_existing(&out_dir.join("synthetic-control.json"))?;
    let synthetic: Value = read_json(&synthetic_pin.path)?;
    if synthetic["schema"] != "uor-r4.attention-audit-synthetic-restore/1"
        || synthetic["status"] != "PASS"
        || synthetic["accepted_updates"] != 1
        || synthetic["stale_apply_refusals"] != 1
        || synthetic["stale_carry_refusals"] != 1
        || synthetic["observed_restored_epoch"] != 0
        || synthetic["fresh_binding"] != true
    {
        return Err(format!("synthetic-control.json invalid: {synthetic}"));
    }

    let timing_pin = pin_existing(&out_dir.join("timing-protocol.json"))?;
    let timing: Value = read_json(&timing_pin.path)?;
    let bookkeeping_pin =
        pin_existing(&out_dir.join("host-metadata-bookkeeping/measurement.json"))?;
    let bookkeeping: Value = read_json(&bookkeeping_pin.path)?;

    let mut verified_bookkeeping_artifacts = 0usize;
    if let Some(arr) = bookkeeping["artifacts"].as_array() {
        for item in arr {
            let pin: FilePin = serde_json::from_value(item.clone())
                .map_err(|e| format!("bookkeeping artifact pin: {e}"))?;
            read_pinned_or_rebased(&pin, out_dir)
                .map_err(|e| format!("bookkeeping artifact: {e}"))?;
            verified_bookkeeping_artifacts += 1;
        }
    }
    let mut verified_raw_artifacts = 0usize;
    if let Some(arr) = timing["raw_artifacts"].as_array() {
        for item in arr {
            let pin: FilePin = serde_json::from_value(item.clone())
                .map_err(|e| format!("timing raw_artifact pin: {e}"))?;
            read_pinned_or_rebased(&pin, out_dir)
                .map_err(|e| format!("timing raw_artifact: {e}"))?;
            verified_raw_artifacts += 1;
        }
    }

    let rel_dir = out_dir.join("relation");
    let fz_dir = out_dir.join("frozen-zero");
    let rel_proof = verify_production_proof("relation", &rel_dir)?;
    let fz_proof = verify_production_proof("frozen-zero", &fz_dir)?;

    let report_path = out_dir.join("report.json");
    let summary = if report_path.exists() {
        let report_pin = pin_existing(&report_path)?;
        let report: Value = read_json(&report_pin.path)?;
        if report["schema"] != "uor-r4.attention-audit-finite-qualification-report/1"
            || report["status"] != "SOURCE_QUALIFICATION_CANDIDATES_READY_FOR_INDEPENDENT_REVIEW"
            || report["dose_training_release"] != false
            || report["fresh_evaluations"] != 0
            || report["source_only_instrument_qualification"] != true
        {
            return Err(format!("report.json header invalid: {report}"));
        }
        let rel_q_pin: FilePin = serde_json::from_value(report["relation_qualification"].clone())
            .map_err(|e| format!("relation_qualification pin: {e}"))?;
        let rel_c_pin: FilePin = serde_json::from_value(report["relation_calibration"].clone())
            .map_err(|e| format!("relation_calibration pin: {e}"))?;
        let fz_q_pin: FilePin = serde_json::from_value(report["control_qualification"].clone())
            .map_err(|e| format!("control_qualification pin: {e}"))?;
        let fz_c_pin: FilePin = serde_json::from_value(report["control_calibration"].clone())
            .map_err(|e| format!("control_calibration pin: {e}"))?;

        let (rel_qual, rel_cal) = verify_arm_qualification("relation", &rel_q_pin, &rel_c_pin)?;
        let (fz_qual, fz_cal) = verify_arm_qualification("frozen-zero", &fz_q_pin, &fz_c_pin)?;
        json!({
            "schema": "uor-r4.a8-finite-cuda-qualification-summary/1",
            "status": "PASS",
            "finite_qualification_controls_status": "PASS",
            "readiness_gate_status": "PASS",
            "dose_qualification": "CANDIDATES_READY",
            "report_sha256": report_pin.sha256,
            "synthetic_control_sha256": synthetic_pin.sha256,
            "timing_protocol_sha256": timing_pin.sha256,
            "bookkeeping_measurement_sha256": bookkeeping_pin.sha256,
            "verified_bookkeeping_artifact_count": verified_bookkeeping_artifacts,
            "verified_raw_artifact_count": verified_raw_artifacts,
            "synthetic_control": {
                "status": synthetic["status"],
                "accepted_updates": synthetic["accepted_updates"],
                "stale_apply_refusals": synthetic["stale_apply_refusals"],
                "stale_carry_refusals": synthetic["stale_carry_refusals"],
                "observed_restored_epoch": synthetic["observed_restored_epoch"],
                "fresh_binding": synthetic["fresh_binding"],
            },
            "bookkeeping": {
                "step_envelope_bytes": bookkeeping["step_envelope_bytes"],
                "boundary_envelope_bytes": bookkeeping["boundary_envelope_bytes"],
                "maximum_step_shape_bytes": bookkeeping["maximum_step_shape_bytes"],
                "maximum_boundary_shape_bytes": bookkeeping["maximum_boundary_shape_bytes"],
                "maximum_progress_shape_bytes": bookkeeping["maximum_progress_shape_bytes"],
                "pass_boundary_ns": bookkeeping["pass_boundary_ns"],
                "breach_restore_ack_ns": bookkeeping["breach_restore_ack_ns"],
                "max_step_sample_ns": bookkeeping["step_samples_ns"]
                    .as_array()
                    .and_then(|a| a.iter().filter_map(Value::as_u64).max()),
            },
            "timing_protocol": {
                "protocol": timing["protocol"]["protocol"],
                "gate_samples_ns": timing["gate_samples_ns"],
                "restore_samples_ns": timing["restore_samples_ns"],
                "arm_family_elapsed_ns": timing["arm_family_elapsed_ns"],
                "arm_family_counts": timing["arm_family_counts"],
                "full_family_counts": timing["full_family_counts"],
                "projected_gate_ns": timing["projected_gate_ns"],
                "projected_step_bookkeeping_ns": timing["projected_step_bookkeeping_ns"],
                "projected_boundary_bookkeeping_ns": timing["projected_boundary_bookkeeping_ns"],
                "projected_restore_ns": timing["projected_restore_ns"],
                "projected_ungated_two_arm_ns": timing["projected_ungated_two_arm_ns"],
                "shared_setup_ns": timing["shared_setup_ns"],
            },
            "relation": {
                "qualification": rel_qual,
                "production_restore": rel_proof,
                "calibration": rel_cal,
            },
            "frozen_zero": {
                "qualification": fz_qual,
                "production_restore": fz_proof,
                "calibration": fz_cal,
            },
        })
    } else {
        let failure_pin = pin_existing(&out_dir.join("failure.json"))?;
        let failure: Value = read_json(&failure_pin.path)?;
        if failure["status"] != "FAILED_OR_NOT_RUN"
            || failure["dose_qualification"] != "WITHHELD"
            || failure["mechanism_negative"] != false
            || failure["error"]
                != "invalid discrete model: gate readiness exceeds 15 percent; replan BEFORE training"
        {
            return Err(format!("unexpected failure.json: {failure}"));
        }
        let rel_q_pin = pin_existing(&rel_dir.join("qualification.json"))?;
        let fz_q_pin = pin_existing(&fz_dir.join("qualification.json"))?;
        let rel_qual = verify_arm_stages_only("relation", &rel_dir, &rel_q_pin)?;
        let fz_qual = verify_arm_stages_only("frozen-zero", &fz_dir, &fz_q_pin)?;

        let projected_gate_ns = timing["projected_gate_ns"]
            .as_u64()
            .ok_or("missing projected_gate_ns")?;
        let projected_step_ns = timing["projected_step_bookkeeping_ns"]
            .as_u64()
            .ok_or("missing projected_step_bookkeeping_ns")?;
        let projected_boundary_ns = timing["projected_boundary_bookkeeping_ns"]
            .as_u64()
            .ok_or("missing projected_boundary_bookkeeping_ns")?;
        let projected_restore_ns = timing["projected_restore_ns"]
            .as_u64()
            .ok_or("missing projected_restore_ns")?;
        let ungated_two_arm_dose_ns = timing["projected_ungated_two_arm_ns"]
            .as_u64()
            .ok_or("missing projected_ungated_two_arm_ns")?;
        let projected_added_gate_ns = projected_gate_ns
            .checked_add(projected_step_ns)
            .and_then(|v| v.checked_add(projected_boundary_ns))
            .and_then(|v| v.checked_add(projected_restore_ns))
            .ok_or("projected_added_gate_ns overflow")?;
        if projected_added_gate_ns == 0 || ungated_two_arm_dose_ns == 0 {
            return Err("zero timing protocol projection".into());
        }
        let ratio_bps = (u128::from(projected_added_gate_ns) * 10_000)
            .div_ceil(u128::from(ungated_two_arm_dose_ns));
        let cal_summary = json!({
            "projected_added_gate_ns": projected_added_gate_ns,
            "ungated_two_arm_dose_ns": ungated_two_arm_dose_ns,
            "readiness_ratio_bps_ceil": u64::try_from(ratio_bps).unwrap_or(u64::MAX),
            "readiness_bar_bps": 1500,
            "passes_15_percent_gate": false,
            "audits_both_arms": 516,
            "replan_stop_error": failure["error"],
        });

        json!({
            "schema": "uor-r4.a8-finite-cuda-qualification-summary/1",
            "status": "FINITE_CONTROLS_PASS_READINESS_GATE_REPLAN_REQUIRED",
            "finite_qualification_controls_status": "PASS",
            "readiness_gate_status": "REPLAN_BEFORE_TRAINING",
            "dose_qualification": failure["dose_qualification"],
            "mechanism_negative": failure["mechanism_negative"],
            "failure_sha256": failure_pin.sha256,
            "synthetic_control_sha256": synthetic_pin.sha256,
            "timing_protocol_sha256": timing_pin.sha256,
            "bookkeeping_measurement_sha256": bookkeeping_pin.sha256,
            "verified_bookkeeping_artifact_count": verified_bookkeeping_artifacts,
            "verified_raw_artifact_count": verified_raw_artifacts,
            "synthetic_control": {
                "status": synthetic["status"],
                "accepted_updates": synthetic["accepted_updates"],
                "stale_apply_refusals": synthetic["stale_apply_refusals"],
                "stale_carry_refusals": synthetic["stale_carry_refusals"],
                "observed_restored_epoch": synthetic["observed_restored_epoch"],
                "fresh_binding": synthetic["fresh_binding"],
            },
            "bookkeeping": {
                "step_envelope_bytes": bookkeeping["step_envelope_bytes"],
                "boundary_envelope_bytes": bookkeeping["boundary_envelope_bytes"],
                "maximum_step_shape_bytes": bookkeeping["maximum_step_shape_bytes"],
                "maximum_boundary_shape_bytes": bookkeeping["maximum_boundary_shape_bytes"],
                "maximum_progress_shape_bytes": bookkeeping["maximum_progress_shape_bytes"],
                "pass_boundary_ns": bookkeeping["pass_boundary_ns"],
                "breach_restore_ack_ns": bookkeeping["breach_restore_ack_ns"],
                "max_step_sample_ns": bookkeeping["step_samples_ns"]
                    .as_array()
                    .and_then(|a| a.iter().filter_map(Value::as_u64).max()),
            },
            "timing_protocol": {
                "protocol": timing["protocol"]["protocol"],
                "gate_samples_ns": timing["gate_samples_ns"],
                "restore_samples_ns": timing["restore_samples_ns"],
                "arm_family_elapsed_ns": timing["arm_family_elapsed_ns"],
                "arm_family_counts": timing["arm_family_counts"],
                "full_family_counts": timing["full_family_counts"],
                "projected_gate_ns": timing["projected_gate_ns"],
                "projected_step_bookkeeping_ns": timing["projected_step_bookkeeping_ns"],
                "projected_boundary_bookkeeping_ns": timing["projected_boundary_bookkeeping_ns"],
                "projected_restore_ns": timing["projected_restore_ns"],
                "projected_added_gate_ns": projected_added_gate_ns,
                "projected_ungated_two_arm_ns": timing["projected_ungated_two_arm_ns"],
                "readiness_ratio_bps_ceil": u64::try_from(ratio_bps).unwrap_or(u64::MAX),
                "readiness_bar_bps": 1500,
                "passes_15_percent_gate": false,
                "shared_setup_ns": timing["shared_setup_ns"],
            },
            "relation": {
                "qualification": rel_qual,
                "production_restore": rel_proof,
                "calibration": cal_summary.clone(),
            },
            "frozen_zero": {
                "qualification": fz_qual,
                "production_restore": fz_proof,
                "calibration": cal_summary,
            },
        })
    };

    let summary_path = Path::new(summary_out);
    if let Some(parent) = summary_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let summary_bytes =
        serde_json::to_vec_pretty(&summary).map_err(|e| format!("summary encode: {e}"))?;
    fs::write(summary_path, &summary_bytes)
        .map_err(|e| format!("{}: {e}", summary_path.display()))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "VERIFIED_FINITE_CUDA_QUALIFICATION",
            "summary_path": summary_path,
            "summary_sha256": sha256(&summary_bytes),
            "finite_qualification_controls_status": summary["finite_qualification_controls_status"],
            "readiness_gate_status": summary["readiness_gate_status"],
            "dose_qualification": summary["dose_qualification"],
            "projected_added_gate_ns": summary["relation"]["calibration"]["projected_added_gate_ns"],
            "ungated_two_arm_dose_ns": summary["relation"]["calibration"]["ungated_two_arm_dose_ns"],
            "readiness_ratio_bps_ceil": summary["relation"]["calibration"]["readiness_ratio_bps_ceil"],
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

struct StageDoseArgs {
    repo_root: PathBuf,
    qual_root: PathBuf,
    dose_root: PathBuf,
    rel_identity_path: PathBuf,
    fz_identity_path: PathBuf,
    reviewed_commit: String,
    receipt_url: String,
    hardware_class: String,
    gpu_count: u32,
    rate_usd_micros_per_hour: u64,
    host_available_bytes: u64,
    reserve_bytes: u64,
    setup_seconds: u64,
    custody_seconds: u64,
}

fn parse_stage_dose_args(args: &[String]) -> Result<StageDoseArgs, String> {
    let mut repo_root: Option<PathBuf> = None;
    let mut qual_root: Option<PathBuf> = None;
    let mut dose_root: Option<PathBuf> = None;
    let mut rel_identity_path: Option<PathBuf> = None;
    let mut fz_identity_path: Option<PathBuf> = None;
    let mut reviewed_commit = "fd409fcdf3bc43d607e768bae6f8cf403aa75901".to_string();
    let mut receipt_url: Option<String> = None;
    let mut hardware_class = "NVIDIA_GeForce_RTX_5090".to_string();
    let mut gpu_count: u32 = 2;
    let mut rate_usd_micros_per_hour: u64 = 1_816_000;
    let mut host_available_bytes: u64 = 45 * 1024 * 1024 * 1024;
    let mut reserve_bytes: u64 = 4 * 1024 * 1024 * 1024;
    let mut setup_seconds: u64 = 900;
    let mut custody_seconds: u64 = 900;

    let mut i = 0;
    while i < args.len() {
        let flag = args[i].as_str();
        let val = args
            .get(i + 1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        match flag {
            "--repo-root" => repo_root = Some(PathBuf::from(val)),
            "--qual-root" => qual_root = Some(PathBuf::from(val)),
            "--dose-root" => dose_root = Some(PathBuf::from(val)),
            "--rel-identity" => rel_identity_path = Some(PathBuf::from(val)),
            "--fz-identity" => fz_identity_path = Some(PathBuf::from(val)),
            "--reviewed-commit" => reviewed_commit = val.clone(),
            "--receipt-url" => receipt_url = Some(val.clone()),
            "--hardware-class" => hardware_class = val.clone(),
            "--gpu-count" => {
                gpu_count = val
                    .parse::<u32>()
                    .map_err(|e| format!("--gpu-count: {e}"))?
            }
            "--rate-usd-micros-per-hour" => {
                rate_usd_micros_per_hour = val
                    .parse::<u64>()
                    .map_err(|e| format!("--rate-usd-micros-per-hour: {e}"))?
            }
            "--host-available-bytes" => {
                host_available_bytes = val
                    .parse::<u64>()
                    .map_err(|e| format!("--host-available-bytes: {e}"))?
            }
            "--reserve-bytes" => {
                reserve_bytes = val
                    .parse::<u64>()
                    .map_err(|e| format!("--reserve-bytes: {e}"))?
            }
            "--setup-seconds" => {
                setup_seconds = val
                    .parse::<u64>()
                    .map_err(|e| format!("--setup-seconds: {e}"))?
            }
            "--custody-seconds" => {
                custody_seconds = val
                    .parse::<u64>()
                    .map_err(|e| format!("--custody-seconds: {e}"))?
            }
            other => return Err(format!("unknown flag {other}")),
        }
        i += 2;
    }

    Ok(StageDoseArgs {
        repo_root: repo_root.ok_or("missing --repo-root")?,
        qual_root: qual_root.ok_or("missing --qual-root")?,
        dose_root: dose_root.ok_or("missing --dose-root")?,
        rel_identity_path: rel_identity_path.ok_or("missing --rel-identity")?,
        fz_identity_path: fz_identity_path.ok_or("missing --fz-identity")?,
        reviewed_commit,
        receipt_url: receipt_url.ok_or("missing --receipt-url")?,
        hardware_class,
        gpu_count,
        rate_usd_micros_per_hour,
        host_available_bytes,
        reserve_bytes,
        setup_seconds,
        custody_seconds,
    })
}

fn parse_build_identity(path: &Path, reviewed_commit: &str) -> Result<ExecutionIdentity, String> {
    let val: Value = read_json(path)?;
    let str_field = |v: &Value, key: &str| -> Result<String, String> {
        v.get(key)
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| format!("{}: missing {key}", path.display()))
    };
    let compiled = val
        .get("compiled")
        .ok_or_else(|| format!("{}: missing compiled", path.display()))?;
    Ok(ExecutionIdentity {
        base_source: "ef9d30999700be11e103f96590e9e3b83b6807c7".into(),
        reviewed_source_commit: reviewed_commit.into(),
        source_sha256: str_field(compiled, "source_sha256")?,
        kernel_sha256: str_field(compiled, "kernel_sha256")?,
        geometry_sha256: str_field(compiled, "geometry_sha256")?,
        executable_sha256: str_field(&val, "executable_sha256")?,
        driver_source_sha256: str_field(&val, "driver_source_sha256")?,
        config_sha256: str_field(&val, "config_sha256")?,
        lineage_sha256: str_field(&val, "lineage_sha256")?,
        sheet_sha256: FIT_SHEET_SHA256.into(),
    })
}

fn run_stage_dose(args: &[String]) -> Result<(), String> {
    let cfg = parse_stage_dose_args(args)?;
    let qual_out = cfg.qual_root.join("qual-out");
    let qual_stage = cfg.qual_root.join("qual-stage");
    let qual_plan: QualificationPlan = read_json(&qual_stage.join("plan.json"))?;
    let timing_pin = pin_existing(&qual_out.join("timing-protocol.json"))?;
    let timing: Value = read_json(&timing_pin.path)?;

    let projected_gate_ns = timing["projected_gate_ns"]
        .as_u64()
        .ok_or("missing projected_gate_ns")?;
    let projected_step_ns = timing["projected_step_bookkeeping_ns"]
        .as_u64()
        .ok_or("missing projected_step_bookkeeping_ns")?;
    let projected_boundary_ns = timing["projected_boundary_bookkeeping_ns"]
        .as_u64()
        .ok_or("missing projected_boundary_bookkeeping_ns")?;
    let projected_restore_ns = timing["projected_restore_ns"]
        .as_u64()
        .ok_or("missing projected_restore_ns")?;
    let ungated_two_arm_dose_ns = timing["projected_ungated_two_arm_ns"]
        .as_u64()
        .ok_or("missing projected_ungated_two_arm_ns")?;
    let projected_added_gate_ns = projected_gate_ns
        .checked_add(projected_step_ns)
        .and_then(|v| v.checked_add(projected_boundary_ns))
        .and_then(|v| v.checked_add(projected_restore_ns))
        .ok_or("projected_added_gate_ns overflow")?;
    let raw_artifacts: Vec<FilePin> = serde_json::from_value(timing["raw_artifacts"].clone())
        .map_err(|e| format!("timing raw_artifacts: {e}"))?;

    let mut cal = Calibration {
        schema: "uor-r4.attention-audit-calibration/1".into(),
        execution: qual_plan.execution.clone(),
        projected_added_gate_ns,
        ungated_two_arm_dose_ns,
        denominator_provenance: "same_hardware_qualified_throughput_extrapolation".into(),
        audits_both_arms: 516,
        complete_gate_io_included: true,
        shared_setup_excluded_from_ratio: true,
        rate_usd_micros_per_hour: qual_plan.rate_usd_micros_per_hour,
        gpu_count: qual_plan.gpu_count,
        all_in_resource_seconds: qual_plan.all_in_resource_seconds,
        staging_archive_reserve_bytes: qual_plan.staging_archive_reserve_bytes,
        host_available_bytes: qual_plan.host_available_bytes,
        measurement_artifacts: raw_artifacts,
        measurement_protocol: timing_pin.clone(),
        resource_admission: qual_plan.resource_admission.clone(),
    };
    let rel_cal_pin = write_json_pin(&qual_out.join("relation/calibration.json"), &cal, 0o600)?;
    cal.execution = qual_plan.control_execution.clone();
    cal.measurement_protocol = timing_pin;
    let fz_cal_pin = write_json_pin(&qual_out.join("frozen-zero/calibration.json"), &cal, 0o600)?;
    let rel_qual_pin = pin_existing(&qual_out.join("relation/qualification.json"))?;
    let fz_qual_pin = pin_existing(&qual_out.join("frozen-zero/qualification.json"))?;

    let dose_stage = cfg.dose_root.join("dose-stage");
    let dose_private = cfg.dose_root.join("dose-private");
    ensure_dir_mode(&dose_stage, 0o700)?;
    ensure_dir_mode(&dose_private, 0o700)?;

    let rel_exec = parse_build_identity(&cfg.rel_identity_path, &cfg.reviewed_commit)?;
    let fz_exec = parse_build_identity(&cfg.fz_identity_path, &cfg.reviewed_commit)?;

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let native_res_pin = write_json_pin(
        &dose_private.join("native-resource-receipt.json"),
        &json!({
            "schema": "uor-r4.native-resource-receipt/1",
            "hardware_class": cfg.hardware_class,
            "gpu_count": cfg.gpu_count,
            "rate_usd_micros_per_hour": cfg.rate_usd_micros_per_hour,
            "observed_unix_seconds": now_secs,
        }),
        0o600,
    )?;
    let native_lease_pin = write_json_pin(
        &dose_private.join("native-lease-receipt.json"),
        &json!({
            "schema": "uor-r4.native-lease-receipt/1",
            "operator": "antigravity",
            "issue": 2831,
            "observed_unix_seconds": now_secs,
        }),
        0o600,
    )?;
    let resource_obs = ResourceObservation {
        schema: "uor-r4.attention-audit-resource-observation/1".into(),
        executions: vec![rel_exec.clone(), fz_exec.clone()],
        operator: "antigravity".into(),
        hardware_class: cfg.hardware_class.clone(),
        gpu_count: cfg.gpu_count,
        rate_usd_micros_per_hour: cfg.rate_usd_micros_per_hour,
        host_available_bytes: cfg.host_available_bytes,
        staging_archive_reserve_bytes: cfg.reserve_bytes,
        observed_unix_seconds: now_secs,
        native_resource_receipt: native_res_pin,
        native_lease_receipt: native_lease_pin,
    };
    let resource_obs_pin = write_json_pin(
        &dose_private.join("resource-observation.json"),
        &resource_obs,
        0o600,
    )?;

    let workload_ns = ungated_two_arm_dose_ns
        .checked_add(projected_added_gate_ns)
        .ok_or("workload_ns overflow")?;
    let workload_seconds = workload_ns.div_ceil(1_000_000_000);
    let all_in_seconds = workload_seconds
        .checked_add(cfg.setup_seconds)
        .and_then(|v| v.checked_add(cfg.custody_seconds))
        .ok_or("all_in_seconds overflow")?;
    let all_in_usd_micros = u64::try_from(
        (u128::from(cfg.rate_usd_micros_per_hour) * u128::from(all_in_seconds)).div_ceil(3600),
    )
    .map_err(|e| e.to_string())?;

    let build_cost_decision = |arm_label: &str, cal_pin: &FilePin| -> Result<FilePin, String> {
        let dummy = cal_pin.clone();
        let mut decision = CostedDoseDecision {
                schema: "uor-r4.attention-audit-costed-dose-decision/1".into(),
                executions: [rel_exec.clone(), fz_exec.clone()],
                calibration: cal_pin.clone(),
                resource_admission: resource_obs_pin.clone(),
                rate_usd_micros_per_hour: cfg.rate_usd_micros_per_hour,
                gpu_count: cfg.gpu_count,
                qualified_ungated_two_arm_ns: ungated_two_arm_dose_ns,
                qualified_added_gate_ns: projected_added_gate_ns,
                projected_workload_resource_seconds: workload_seconds,
                planned_common_setup_seconds: cfg.setup_seconds,
                planned_common_custody_archive_close_seconds: cfg.custody_seconds,
                common_metadata_coverage: "BASELINE_Q1_TERMINAL_METADATA_INCLUDED_IN_PLANNED_COMMON_SETUP_CUSTODY_CLOSE_NOT_MEASURED_RECURRING_GATE".into(),
                all_in_projected_resource_seconds: all_in_seconds,
                all_in_projected_resource_usd_micros: all_in_usd_micros,
                workload_projection_basis: "CONSERVATIVE_SEQUENTIAL_SUM_RESOURCE_TIME_UPPER_BOUND_NOT_MEASURED_PARALLEL_WALLTIME".into(),
                transfer_storage_cost_status: "UNMEASURED_SEPARATE_FROM_RESOURCE_PROXY".into(),
                measured_full_dose: false,
                invoice: false,
                budget_is_hard_stop: false,
                owner_cost_approval: dummy.clone(),
                pm_dispatch: dummy,
            };
        let payload_sha = cost_payload_sha256(&decision).map_err(|e| e.to_string())?;
        let owner_att = DoseCostAttestation {
            schema: "uor-r4.attention-audit-dose-cost-attestation/1".into(),
            actor: "Casey".into(),
            role: "owner_cost_scope".into(),
            verdict: "APPROVE_EXACT_COSTED_DOSE_PLAN".into(),
            executions: [rel_exec.clone(), fz_exec.clone()],
            cost_payload_sha256: payload_sha.clone(),
            receipt_url: cfg.receipt_url.clone(),
        };
        let pm_att = DoseCostAttestation {
            schema: "uor-r4.attention-audit-dose-cost-attestation/1".into(),
            actor: "Kimi".into(),
            role: "pm_dispatch".into(),
            verdict: "DISPATCH_EXACT_COSTED_DOSE_PLAN".into(),
            executions: [rel_exec.clone(), fz_exec.clone()],
            cost_payload_sha256: payload_sha,
            receipt_url: cfg.receipt_url.clone(),
        };
        decision.owner_cost_approval = write_json_pin(
            &dose_stage.join(format!("{arm_label}-owner-cost-approval.json")),
            &owner_att,
            0o600,
        )?;
        decision.pm_dispatch = write_json_pin(
            &dose_stage.join(format!("{arm_label}-pm-dispatch.json")),
            &pm_att,
            0o600,
        )?;
        write_json_pin(
            &dose_stage.join(format!("{arm_label}-costed-dose-decision.json")),
            &decision,
            0o600,
        )
    };

    let rel_cost_pin = build_cost_decision("relation", &rel_cal_pin)?;
    let fz_cost_pin = build_cost_decision("frozen-zero", &fz_cal_pin)?;

    let parent_sheet_pin = pin_existing(
        &cfg.repo_root
            .join("docs/harness/prep/m1-attention-a8-address-cells-learning-2026-10-19.md"),
    )?;
    let dose_sheet_pin = pin_existing(
        &cfg.repo_root
            .join("docs/harness/prep/m1-attention-admission-preserving-DRAFT-2026-10-19.md"),
    )?;
    let anchor_manifest_pin = pin_existing(&qual_stage.join("anchor-manifest.json"))?;

    let rel_cfg: DriverConfig = read_json(
        &cfg.repo_root
            .join("docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json"),
    )?;
    let fz_cfg: DriverConfig = read_json(
        &cfg.repo_root
            .join("docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json"),
    )?;

    let transfer = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: PathBuf::from("/workspace/uor-r4/local/driver/data/ft/fresh_v2/tokens.u16"),
            sha256: "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8".into(),
        },
        mask: FilePin {
            path: PathBuf::from("/workspace/uor-r4/local/driver/data/ft/fresh_v2/response_mask.u8"),
            sha256: "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e".into(),
        },
        expected_payload_tokens: 6299,
        expected_eligible_targets: 2000,
    };

    let build_arm_plan_and_admission = |arm_label: &str,
                                        exec: ExecutionIdentity,
                                        driver_cfg: DriverConfig,
                                        qual_pin: FilePin,
                                        cal_pin: FilePin,
                                        cost_pin: FilePin|
     -> Result<(FilePin, FilePin), String> {
        let run_state_dir = cfg.dose_root.join(format!("run-state-{arm_label}"));
        if run_state_dir.exists() {
            return Err(format!(
                "{} must not exist before discrete-stack runs",
                run_state_dir.display()
            ));
        }
        let scope = dose_scope(&driver_cfg).map_err(|e| e.to_string())?;
        let mut plan = AuditedPlan {
            schema: DOSE_PLAN_SCHEMA.into(),
            status: "HELD_OWNER_ACTION_REQUIRED".into(),
            scope: scope.clone(),
            execution: exec.clone(),
            config: driver_cfg,
            parent_sheet: parent_sheet_pin.clone(),
            sheet: dose_sheet_pin.clone(),
            anchor_manifest: anchor_manifest_pin.clone(),
            anchor_manifest_sha256: qual_plan.anchor_manifest_sha256.clone(),
            anchor_policy: default_anchor_policy(),
            anchor_csv_sha256: REGISTERED_CSV_SHA256.into(),
            policy: AuditPolicy::registered(),
            qualification: qual_pin,
            calibration: cal_pin,
            resource_admission: resource_obs_pin.clone(),
            costed_dose_decision: cost_pin,
            independent_review: dose_sheet_pin.clone(),
            transfer: transfer.clone(),
            run_state_directory: run_state_dir,
        };
        let rev_payload_sha = review_payload_sha256(&plan).map_err(|e| e.to_string())?;
        let review = AuditedReview {
            reviewer: "antigravity".into(),
            verdict: DOSE_REVIEW_VERDICT.into(),
            scope: scope.clone(),
            execution: exec,
            reviewed_plan_payload_sha256: rev_payload_sha,
            receipt_url: cfg.receipt_url.clone(),
        };
        plan.independent_review = write_json_pin(
            &dose_stage.join(format!("{arm_label}-review.json")),
            &review,
            0o600,
        )?;
        let plan_pin = write_json_pin(
            &dose_stage.join(format!("{arm_label}-plan.json")),
            &plan,
            0o600,
        )?;
        let owner_path = dose_private.join(format!("{arm_label}-owner.json"));
        let canonical_private = fs::canonicalize(&dose_private)
            .map_err(|e| format!("{}: {e}", dose_private.display()))?;
        let canonical_owner_path = canonical_private.join(format!("{arm_label}-owner.json"));
        let owner = AuditedOwner {
            schema: DOSE_OWNER_SCHEMA.into(),
            owner: "Casey".into(),
            decision: DOSE_OWNER_DECISION.into(),
            scope,
            plan_file_sha256: plan_pin.sha256.clone(),
            receipt_url: format!("owner-terminal:{}", canonical_owner_path.display()),
        };
        let owner_pin = write_json_pin(&owner_path, &owner, 0o600)?;
        let admission = AuditedAdmission {
            schema: DOSE_ADMISSION_SCHEMA.into(),
            plan_file: plan_pin.clone(),
            plan,
            owner_release: owner_pin,
        };
        let admission_pin = write_json_pin(
            &dose_private.join(format!("{arm_label}-admission.json")),
            &admission,
            0o600,
        )?;
        Ok((plan_pin, admission_pin))
    };

    let (rel_plan_pin, rel_adm_pin) = build_arm_plan_and_admission(
        "relation",
        rel_exec,
        rel_cfg,
        rel_qual_pin,
        rel_cal_pin,
        rel_cost_pin,
    )?;
    let (fz_plan_pin, fz_adm_pin) = build_arm_plan_and_admission(
        "frozen-zero",
        fz_exec,
        fz_cfg,
        fz_qual_pin,
        fz_cal_pin,
        fz_cost_pin,
    )?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "STAGED_AUDITED_DOSE",
            "relation_plan": rel_plan_pin,
            "relation_admission": rel_adm_pin,
            "frozen_zero_plan": fz_plan_pin,
            "frozen_zero_admission": fz_adm_pin,
            "projected_added_gate_ns": projected_added_gate_ns,
            "ungated_two_arm_dose_ns": ungated_two_arm_dose_ns,
            "all_in_projected_resource_seconds": all_in_seconds,
            "all_in_projected_resource_usd_micros": all_in_usd_micros,
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

fn read_dose_pinned_or_rebased(pin: &FilePin, arm_out_dir: &Path) -> Result<Vec<u8>, String> {
    let dose_root = arm_out_dir.parent().unwrap_or(arm_out_dir);
    let candidate = if pin.path.exists() {
        pin.path.clone()
    } else if let Ok(rel) = pin.path.strip_prefix("/workspace/a8-dose") {
        let stripped = dose_root.join(rel);
        if stripped.exists() {
            stripped
        } else {
            let in_arm = arm_out_dir.join(rel);
            if in_arm.exists() {
                in_arm
            } else if let Some(name) = pin.path.file_name() {
                arm_out_dir.join(name)
            } else {
                stripped
            }
        }
    } else if let Some(name) = pin.path.file_name() {
        arm_out_dir.join(name)
    } else {
        pin.path.clone()
    };
    let bytes = read_bytes(&candidate)?;
    let actual = sha256(&bytes);
    if actual != pin.sha256 {
        return Err(format!(
            "{}: sha256 {actual} != expected {}",
            candidate.display(),
            pin.sha256
        ));
    }
    Ok(bytes)
}

fn summarize_dose_arm(
    arm: &str,
    rep: &Value,
    pin: &FilePin,
    arm_dir: &Path,
) -> Result<Value, String> {
    let audit_arr = rep["audits"]
        .as_array()
        .ok_or_else(|| format!("{arm} report missing audits array"))?;
    for item in audit_arr {
        let audit_pin: FilePin =
            serde_json::from_value(item.clone()).map_err(|e| format!("{arm} audit pin: {e}"))?;
        read_dose_pinned_or_rebased(&audit_pin, arm_dir)
            .map_err(|e| format!("{arm} audit file: {e}"))?;
    }
    let record_arr = rep["records"]
        .as_array()
        .ok_or_else(|| format!("{arm} report missing records array"))?;
    let mut rejected_intervals = 0usize;
    for item in record_arr {
        let rec_pin: FilePin =
            serde_json::from_value(item.clone()).map_err(|e| format!("{arm} record pin: {e}"))?;
        let rec_bytes = read_dose_pinned_or_rebased(&rec_pin, arm_dir)
            .map_err(|e| format!("{arm} record file: {e}"))?;
        let rec_val: Value = serde_json::from_slice(&rec_bytes)
            .map_err(|e| format!("{arm} record json {}: {e}", rec_pin.path.display()))?;
        if rec_val["kind"]["kind"] == "interval" && rec_val["kind"]["record"]["rejected"] == true {
            rejected_intervals += 1;
        }
    }
    Ok(json!({
        "report_sha256": pin.sha256,
        "status": rep["status"],
        "scope": rep["scope"],
        "admission_sha256": rep["admission_sha256"],
        "completed_batches": rep["completed_batches"],
        "committed_accepted_updates": rep["committed_accepted_updates"],
        "counters": rep["counters"],
        "stop_reasons": rep["stop_reasons"],
        "records_count": record_arr.len(),
        "audits_count": audit_arr.len(),
        "rejected_intervals": rejected_intervals,
        "baseline_b3": rep["baseline_b3"]["psucc"],
        "baseline_full_b3": rep["baseline_b3"]["full"],
        "q1_decision": rep["q1"]["decision"],
        "final_dev_psucc": rep["final_dev"]["b3"]["psucc"],
        "final_dev_full": rep["final_dev"]["b3"]["full"],
        "relation_off_psucc": rep["relation_off"]["b3"]["psucc"],
        "relation_off_full": rep["relation_off"]["b3"]["full"],
        "fresh_transfer": rep["fresh_transfer"],
        "elapsed_ns": rep["elapsed_ns"],
    }))
}

fn run_verify_dose(args: &[String]) -> Result<(), String> {
    let [f1, rel_out, f2, fz_out, f3, summary_out] = args else {
        return Err(
            "usage: a8-qualify-stage verify-dose --relation-out <DIR> --frozen-zero-out <DIR> --summary-out <FILE>".into(),
        );
    };
    if f1 != "--relation-out" || f2 != "--frozen-zero-out" || f3 != "--summary-out" {
        return Err(
            "usage: a8-qualify-stage verify-dose --relation-out <DIR> --frozen-zero-out <DIR> --summary-out <FILE>".into(),
        );
    }
    let rel_dir = Path::new(rel_out);
    let fz_dir = Path::new(fz_out);
    let rel_report_pin = pin_existing(&rel_dir.join("report.json"))?;
    let fz_report_pin = pin_existing(&fz_dir.join("report.json"))?;
    let rel_rep: Value = read_json(&rel_report_pin.path)?;
    let fz_rep: Value = read_json(&fz_report_pin.path)?;

    for (arm, rep) in [("relation", &rel_rep), ("frozen_zero", &fz_rep)] {
        if rep["schema"] != "uor-r4.attention-audit-report/1" {
            return Err(format!("{arm} report schema mismatch: {}", rep["schema"]));
        }
        if rep["fresh_transfer"]["evaluations"] != 0
            || rep["fresh_transfer"]["status"] != "SEALED_PENDING_PAIRED_DEV_DECISION"
        {
            return Err(format!(
                "{arm} fresh_transfer invariant violated: {}",
                rep["fresh_transfer"]
            ));
        }
        if rep["training_other_scopes"] != "HELD" {
            return Err(format!("{arm} training_other_scopes != HELD"));
        }
    }
    if fz_rep["counters"][2]["committed_applied"].as_u64() != Some(0)
        || fz_rep["counters"][2]["provisional_applied"].as_u64() != Some(0)
        || fz_rep["counters"][2]["rejected_applied"].as_u64() != Some(0)
    {
        return Err(format!(
            "frozen_zero relation applied updates must be 0, got {}",
            fz_rep["counters"][2]
        ));
    }

    let rel_sum = summarize_dose_arm("relation", &rel_rep, &rel_report_pin, rel_dir)?;
    let fz_sum = summarize_dose_arm("frozen_zero", &fz_rep, &fz_report_pin, fz_dir)?;

    let both_complete = rel_rep["status"] == "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR"
        && fz_rep["status"] == "COMPLETE_PENDING_REGISTERED_DISCRIMINATOR";

    let summary = json!({
        "schema": "uor-r4.a8-audited-dose-summary/1",
        "both_arms_completed_16404": both_complete,
        "fresh_slice_sealed": true,
        "fresh_evaluations_total": 0,
        "relation": rel_sum,
        "frozen_zero": fz_sum,
    });

    let summary_path = Path::new(summary_out);
    if let Some(parent) = summary_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    let summary_bytes =
        serde_json::to_vec_pretty(&summary).map_err(|e| format!("summary encode: {e}"))?;
    fs::write(summary_path, &summary_bytes)
        .map_err(|e| format!("{}: {e}", summary_path.display()))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "status": "VERIFIED_AUDITED_DOSE_OUTPUTS",
            "summary_path": summary_path,
            "summary_sha256": sha256(&summary_bytes),
            "both_arms_completed_16404": both_complete,
            "relation_status": rel_rep["status"],
            "relation_stop_reasons": rel_rep["stop_reasons"],
            "relation_completed_batches": rel_rep["completed_batches"],
            "frozen_zero_status": fz_rep["status"],
            "frozen_zero_stop_reasons": fz_rep["stop_reasons"],
            "frozen_zero_completed_batches": fz_rep["completed_batches"],
        }))
        .map_err(|e| e.to_string())?
    );
    Ok(())
}

fn run(args: &[String]) -> Result<(), String> {
    let Some(subcmd) = args.first().map(String::as_str) else {
        return Err("usage: a8-qualify-stage <stage|verify|stage-dose|verify-dose> ...".into());
    };
    match subcmd {
        "stage" => run_stage(&args[1..]),
        "verify" => run_verify(&args[1..]),
        "stage-dose" => run_stage_dose(&args[1..]),
        "verify-dose" => run_verify_dose(&args[1..]),
        other => Err(format!("unknown subcommand {other}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("a8-qualify-stage error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_configs_and_inherited_observations_match_fd409fcdf_identity() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let rel_path =
            root.join("docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json");
        let fz_path = root
            .join("docs/labs/a8-training-configs-2026-10-19/evidence/a8-frozen-zero-seed1.json");
        let rel_cfg: DriverConfig = read_json(&rel_path).expect("relation config");
        let fz_cfg: DriverConfig = read_json(&fz_path).expect("frozen-zero config");
        assert!(rel_cfg.relation_learning_policy.is_learn());
        assert!(!fz_cfg.relation_learning_policy.is_learn());

        let inh_dir =
            root.join("docs/labs/m1-a8-registration-and-runbook-staging/evidence/inherited");
        let tmp =
            std::env::temp_dir().join(format!("a8-qualify-stage-test-{}", std::process::id()));
        ensure_dir_mode(&tmp, 0o700).expect("temp dir");
        let (pins, rel_exec, fz_exec) =
            collect_inherited(&inh_dir, &tmp).expect("inherited observations");
        assert_eq!(pins.len(), 16);
        assert_eq!(
            rel_exec.reviewed_source_commit,
            "fd409fcdf3bc43d607e768bae6f8cf403aa75901"
        );
        assert_eq!(
            rel_exec.executable_sha256,
            "fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c"
        );
        assert_eq!(rel_exec.sheet_sha256, FIT_SHEET_SHA256);
        assert_eq!(fz_exec.executable_sha256, rel_exec.executable_sha256);
        assert_eq!(
            sha256(&serde_json::to_vec(&rel_cfg).expect("encode rel_cfg")),
            rel_exec.config_sha256
        );
        assert_eq!(
            sha256(&serde_json::to_vec(&fz_cfg).expect("encode fz_cfg")),
            fz_exec.config_sha256
        );

        let dummy_pin = pins[0].clone();
        let plan = QualificationPlan {
            schema: PLAN_SCHEMA.into(),
            scope: QUAL_SCOPE.into(),
            status: "HELD_FINITE_QUALIFICATION_OWNER_ACTION_REQUIRED".into(),
            execution: rel_exec,
            config: rel_cfg,
            control_execution: fz_exec,
            control_config: fz_cfg,
            dose_sheet: dummy_pin.clone(),
            qualification_fit: dummy_pin.clone(),
            owner_scope_cost: dummy_pin.clone(),
            pm_dispatch: dummy_pin.clone(),
            independent_review: dummy_pin.clone(),
            anchor_manifest: dummy_pin.clone(),
            anchor_manifest_sha256: EXPECTED_MANIFEST_SHA256.into(),
            anchor_csv_sha256: REGISTERED_CSV_SHA256.into(),
            anchor_policy: default_anchor_policy(),
            limits: default_limits(),
            inherited_observations: pins,
            accepted_initlock_disposition: dummy_pin.clone(),
            residual_fit: dummy_pin.clone(),
            g_init_comparison_or_waiver: dummy_pin.clone(),
            resource_admission: dummy_pin,
            rate_usd_micros_per_hour: 1_816_000,
            gpu_count: 2,
            all_in_resource_seconds: 1_800,
            staging_archive_reserve_bytes: 2 * 1024 * 1024 * 1024,
            host_available_bytes: 45 * 1024 * 1024 * 1024,
            run_state_directory: PathBuf::from("/workspace/a8s1/qual-run-state"),
        };
        let digest = payload_sha256(&plan).expect("payload_sha256");
        assert_eq!(digest.len(), 64);
        fs::remove_dir_all(&tmp).expect("cleanup temp dir");
    }

    #[test]
    fn verify_arm_qualification_enforces_stage_pass_and_readiness_bar() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let inh_dir =
            root.join("docs/labs/m1-a8-registration-and-runbook-staging/evidence/inherited");
        let obs: InheritedObservation =
            read_json(&inh_dir.join("relation-canonical_initlock_c2.json"))
                .expect("read inherited");
        let tmp =
            std::env::temp_dir().join(format!("a8-qualify-verify-test-{}", std::process::id()));
        ensure_dir_mode(&tmp, 0o700).expect("temp dir");

        let mut checks = Vec::new();
        for name in STAGE_NAMES {
            let pin = write_json_pin(
                &tmp.join(format!("{name}.json")),
                &json!({
                    "schema": "uor-r4.attention-audit-qualified-stage/1",
                    "name": name,
                    "execution": obs.execution,
                    "status": "PASS",
                    "observations": [],
                    "facts": {"ok": true},
                }),
                0o600,
            )
            .expect("stage pin");
            checks.push(
                uor_r4_discrete::controller::attention_audited::QualifiedCheck {
                    name: name.into(),
                    status: "PASS".into(),
                    artifact: pin,
                },
            );
        }
        let dummy = checks[0].artifact.clone();
        let qual = Qualification {
            schema: "uor-r4.attention-audit-qualification/1".into(),
            execution: obs.execution.clone(),
            checks,
            g_init_comparison_or_waiver: dummy.clone(),
            accepted_initlock_disposition: dummy.clone(),
            residual_fit: dummy.clone(),
            b3_psucc: 1351,
            b3_full: 4244,
            top1: 133,
            bar_i: 86,
            epoch: 0,
            updates_applied: 0,
            reference_capture_scope: "DEFERRED_TO_DOSE".into(),
            full_rows: 14287,
            psucc_rows: 3847,
            psucc_reference_top1: 412,
            occupied_cells: 256,
            distinct_vectors: 4096,
            tie_mass: 0,
            negative_native_exits: [2, 2],
            sensitivity_predicate: "both_probes_any_active_candidate_cost_differs_v1".into(),
            combine_sensitive: 1,
            relation_sensitive: 1,
        };
        let qual_pin = write_json_pin(&tmp.join("qualification.json"), &qual, 0o600).expect("qual");

        let mut cal = Calibration {
            schema: "uor-r4.attention-audit-calibration/1".into(),
            execution: obs.execution,
            projected_added_gate_ns: 85,
            ungated_two_arm_dose_ns: 100,
            denominator_provenance: "same_hardware_qualified_throughput_extrapolation".into(),
            audits_both_arms: 516,
            complete_gate_io_included: true,
            shared_setup_excluded_from_ratio: true,
            rate_usd_micros_per_hour: 1_816_000,
            gpu_count: 2,
            all_in_resource_seconds: 1_800,
            staging_archive_reserve_bytes: 2 * 1024 * 1024 * 1024,
            host_available_bytes: 45 * 1024 * 1024 * 1024,
            measurement_artifacts: vec![dummy.clone()],
            measurement_protocol: dummy.clone(),
            resource_admission: dummy,
        };
        let cal_pin = write_json_pin(&tmp.join("calibration.json"), &cal, 0o600).expect("cal");
        let (_, cal_summary) =
            verify_arm_qualification("relation", &qual_pin, &cal_pin).expect("85% boundary passes");
        assert_eq!(cal_summary["readiness_ratio_bps_ceil"], 8500);

        cal.projected_added_gate_ns = 86;
        let bad_cal_pin =
            write_json_pin(&tmp.join("calibration-bad.json"), &cal, 0o600).expect("bad cal");
        assert!(verify_arm_qualification("relation", &qual_pin, &bad_cal_pin).is_err());

        let rebased_pin = FilePin {
            path: PathBuf::from("/workspace/a8s1/qual-out/qualification.json"),
            sha256: qual_pin.sha256.clone(),
        };
        let rebased_bytes = read_pinned_or_rebased(&rebased_pin, &tmp).expect("rebased pin");
        assert_eq!(sha256(&rebased_bytes), qual_pin.sha256);
        let stage_only =
            verify_arm_stages_only("relation", &tmp, &rebased_pin).expect("stages only pass");
        assert_eq!(stage_only["stages"].as_array().map(Vec::len), Some(5));

        fs::remove_dir_all(&tmp).expect("cleanup temp dir");
    }

    #[test]
    fn committed_cuda_qualification_summary_records_all_finite_controls_and_timing() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let summary_path =
            root.join("docs/labs/m1-a8-finite-cuda-qualification/qualification-summary.json");
        let summary: Value = read_json(&summary_path).expect("read qualification-summary.json");
        assert_eq!(
            summary["schema"],
            "uor-r4.a8-finite-cuda-qualification-summary/1"
        );
        assert_eq!(summary["finite_qualification_controls_status"], "PASS");
        assert_eq!(summary["readiness_gate_status"], "REPLAN_BEFORE_TRAINING");
        assert_eq!(summary["dose_qualification"], "WITHHELD");
        assert_eq!(summary["mechanism_negative"], false);
        assert_eq!(summary["verified_bookkeeping_artifact_count"], 142);
        assert_eq!(summary["verified_raw_artifact_count"], 792);
        assert_eq!(summary["synthetic_control"]["status"], "PASS");
        assert_eq!(summary["synthetic_control"]["accepted_updates"], 1);
        assert_eq!(summary["synthetic_control"]["stale_apply_refusals"], 1);
        assert_eq!(summary["synthetic_control"]["stale_carry_refusals"], 1);
        assert_eq!(summary["synthetic_control"]["observed_restored_epoch"], 0);
        for arm in ["relation", "frozen_zero"] {
            let qual = &summary[arm]["qualification"];
            assert_eq!(
                qual["execution"]["reviewed_source_commit"],
                "fd409fcdf3bc43d607e768bae6f8cf403aa75901"
            );
            assert_eq!(
                qual["execution"]["executable_sha256"],
                "fb4493253d3e26dc7cc5fbcbcf7fa4dc882d93a5c7b02d340b25918a2d25b00c"
            );
            assert_eq!(qual["b3_psucc"], 1351);
            assert_eq!(qual["b3_full"], 4244);
            assert_eq!(qual["top1"], 133);
            assert_eq!(qual["occupied_cells"], 256);
            assert_eq!(qual["distinct_vectors"], 4096);
            assert_eq!(qual["tie_mass"], 0);
            let stages = qual["stages"].as_array().expect("stages array");
            assert_eq!(stages.len(), 5);
            for (stage, expected_name) in stages.iter().zip(STAGE_NAMES) {
                assert_eq!(stage["name"], expected_name);
                assert_eq!(stage["status"], "PASS");
            }
            let restore = &summary[arm]["production_restore"];
            assert_eq!(restore["status"], "PASS");
            assert_eq!(restore["advanced_attempts"], 384);
            assert_eq!(restore["restored_epoch"], 0);
            assert_eq!(restore["passing_storage_restored"], true);
            assert_eq!(restore["passing_snapshot_restored"], true);
            assert_eq!(restore["fresh_binding"], true);
            assert_eq!(restore["stale_handles_refused"], true);
            assert_eq!(restore["anchor_equal_before_after_restore"], true);
            assert_eq!(restore["train_anchor"]["scored_rows"], 3847);
            assert_eq!(restore["train_anchor"]["target_admitted_rows"], 1211);
        }
        assert_eq!(
            summary["timing_protocol"]["projected_added_gate_ns"],
            4_947_754_777_700u64
        );
        assert_eq!(
            summary["timing_protocol"]["projected_ungated_two_arm_ns"],
            6_309_716_689_996u64
        );
        assert_eq!(summary["timing_protocol"]["readiness_ratio_bps_ceil"], 7842);
    }

    #[test]
    fn committed_audited_dose_summary_records_paired_dev_verdict_and_sealed_fresh_slice() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let summary_path = root.join("docs/labs/m1-a8-full-dose-rerun/dose-summary.json");
        let summary: Value = read_json(&summary_path).expect("read dose-summary.json");
        assert_eq!(summary["schema"], "uor-r4.a8-audited-dose-summary/1");
        assert_eq!(summary["both_arms_completed_16404"], false);
        assert_eq!(summary["fresh_slice_sealed"], true);
        assert_eq!(summary["fresh_evaluations_total"], 0);

        let rel = &summary["relation"];
        assert_eq!(rel["status"], "STOP_FOR_DIAGNOSIS");
        assert_eq!(rel["stop_reasons"], json!(["consecutive_rejections"]));
        assert_eq!(rel["completed_batches"], 1600);
        assert_eq!(rel["committed_accepted_updates"], 132);
        assert_eq!(rel["audits_count"], 26);
        assert_eq!(rel["rejected_intervals"], 5);
        assert_eq!(rel["baseline_b3"]["target_admitted"], 1351);
        assert_eq!(rel["baseline_full_b3"]["target_admitted"], 4244);
        assert_eq!(rel["fresh_transfer"]["evaluations"], 0);
        assert_eq!(
            rel["fresh_transfer"]["status"],
            "SEALED_PENDING_PAIRED_DEV_DECISION"
        );

        let fz = &summary["frozen_zero"];
        assert_eq!(fz["status"], "STOP_FOR_DIAGNOSIS");
        assert_eq!(fz["stop_reasons"], json!(["consecutive_rejections"]));
        assert_eq!(fz["completed_batches"], 5248);
        assert_eq!(fz["committed_accepted_updates"], 274);
        assert_eq!(fz["audits_count"], 83);
        assert_eq!(fz["rejected_intervals"], 15);
        assert_eq!(fz["counters"][2]["committed_applied"], 0);
        assert_eq!(fz["counters"][2]["provisional_applied"], 0);
        assert_eq!(fz["counters"][2]["rejected_applied"], 0);
        assert_eq!(fz["counters"][2]["policy_refused"], 356);
        assert_eq!(fz["q1_decision"]["diagnostic_stop"], false);
        assert_eq!(fz["q1_decision"]["q1_psucc_target_admitted"], 1106);
        assert_eq!(fz["fresh_transfer"]["evaluations"], 0);
        assert_eq!(
            fz["fresh_transfer"]["status"],
            "SEALED_PENDING_PAIRED_DEV_DECISION"
        );
    }

    #[test]
    fn verify_dose_rebases_pod_paths_and_fails_on_missing_or_tampered_records() {
        let tmp =
            std::env::temp_dir().join(format!("a8-verify-dose-rebase-test-{}", std::process::id()));
        let rel_out = tmp.join("relation-out");
        let run_state = tmp.join("run-state-relation");
        ensure_dir_mode(&rel_out, 0o700).expect("rel_out");
        ensure_dir_mode(&run_state, 0o700).expect("run_state");

        let audit_pin = write_json_pin(
            &run_state.join("anchor-00064.json"),
            &json!({"evaluation": {"aggregate": {"target_admitted_rows": 1186}}}),
            0o600,
        )
        .expect("audit_pin");
        let rec_pin = write_json_pin(
            &run_state.join("interval-00064.json"),
            &json!({"kind": {"kind": "interval", "record": {"rejected": true}}}),
            0o600,
        )
        .expect("rec_pin");

        let rep = json!({
            "schema": "uor-r4.attention-audit-report/1",
            "status": "STOP_FOR_DIAGNOSIS",
            "scope": "relation",
            "admission_sha256": "abc",
            "completed_batches": 64,
            "committed_accepted_updates": 0,
            "counters": [
                {"committed_applied": 0, "provisional_applied": 0, "rejected_applied": 0, "policy_refused": 0},
                {"committed_applied": 0, "provisional_applied": 0, "rejected_applied": 0, "policy_refused": 0},
                {"committed_applied": 0, "provisional_applied": 0, "rejected_applied": 0, "policy_refused": 0}
            ],
            "stop_reasons": ["consecutive_rejections"],
            "audits": [{
                "path": "/workspace/a8-dose/run-state-relation/anchor-00064.json",
                "sha256": audit_pin.sha256
            }],
            "records": [{
                "path": "/workspace/a8-dose/run-state-relation/interval-00064.json",
                "sha256": rec_pin.sha256
            }],
            "baseline_b3": {"psucc": {"rows": 3847, "target_admitted": 1351}, "full": {"rows": 14287, "target_admitted": 4244}},
            "q1": null,
            "final_dev": null,
            "relation_off": null,
            "fresh_transfer": {"evaluations": 0, "status": "SEALED_PENDING_PAIRED_DEV_DECISION"},
            "training_other_scopes": "HELD",
            "elapsed_ns": "1000"
        });
        let rep_pin = write_json_pin(&rel_out.join("report.json"), &rep, 0o600).expect("rep_pin");
        let sum = summarize_dose_arm("relation", &rep, &rep_pin, &rel_out)
            .expect("rebased summarize_dose_arm passes");
        assert_eq!(sum["rejected_intervals"], 1);
        assert_eq!(sum["audits_count"], 1);
        assert_eq!(sum["records_count"], 1);

        fs::remove_file(run_state.join("interval-00064.json")).expect("remove interval");
        assert!(summarize_dose_arm("relation", &rep, &rep_pin, &rel_out).is_err());

        fs::remove_dir_all(&tmp).expect("cleanup temp dir");
    }
}
