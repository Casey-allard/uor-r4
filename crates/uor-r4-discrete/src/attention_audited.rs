//! Separately released active whole-interval admission preservation.
//! Model arithmetic is exclusively the existing CUDA driver.

use super::*;
#[path = "attention_audit_qualification.rs"]
pub mod qualification;
use crate::attention_anchor::{AnchorPolicy, TrainAnchorManifest};
use crate::attention_anchor_device::{
    RegisteredAnchorBinding, FIT_SHEET_SHA256, REGISTERED_CSV_SHA256,
};
#[cfg(any(feature = "cuda", test))]
use crate::attention_audit::{AuditMachine, DurableRecord};

pub const PLAN_SCHEMA: &str = "uor-r4.attention-audit-owner-plan/1";
pub const ADMISSION_SCHEMA: &str = "uor-r4.attention-audit-owner-admission/1";
pub const OWNER_SCHEMA: &str = "uor-r4.attention-audit-owner-decision/1";
pub const SCOPE: &str = "m1-attention-active-whole-interval-rejection-v1";
pub const OWNER_DECISION: &str = "RELEASE_D40_AUDITED_WHOLE_INTERVAL_ARM_ONLY";
pub const REVIEW_VERDICT: &str = "APPROVE_ACTIVE_WHOLE_INTERVAL_ADMISSION_PLAN";
pub(crate) const QUALIFIED_STAGE1_COMMIT: &str = "fd409fcdf3bc43d607e768bae6f8cf403aa75901";
pub(crate) const QUALIFIED_STAGE1_SOURCE_SHA256: &str =
    "81651081145fd4712046a8158a26c3bd6940ff5a60e5cdddf666057cb9fc0084";
pub(crate) const QUALIFIED_STAGE1_KERNEL_SHA256: &str =
    "5cd0e73241fd895f2dd0004c83af3d1eb9f89e00ec2e933c1acc44fc7127bfd6";
pub(crate) const QUALIFIED_STAGE1_DRIVER_SOURCE_SHA256: &str =
    "0f8ce4061996acdcce3ad4ae01d9e267f280703a77536eb6ecc24bfdc6e6df1d";

pub(crate) fn matches_qualified_execution(
    plan_exec: &ExecutionIdentity,
    qual_exec: &ExecutionIdentity,
) -> bool {
    qual_exec == plan_exec
        || (qual_exec.reviewed_source_commit == QUALIFIED_STAGE1_COMMIT
            && qual_exec.source_sha256 == QUALIFIED_STAGE1_SOURCE_SHA256
            && qual_exec.kernel_sha256 == QUALIFIED_STAGE1_KERNEL_SHA256
            && qual_exec.driver_source_sha256 == QUALIFIED_STAGE1_DRIVER_SOURCE_SHA256
            && qual_exec.geometry_sha256 == plan_exec.geometry_sha256
            && qual_exec.config_sha256 == plan_exec.config_sha256
            && qual_exec.sheet_sha256 == plan_exec.sheet_sha256)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditPolicy {
    pub cadence: u64,
    pub max_batches: u64,
    pub final_tail_batches: u64,
    pub margin: u64,
    pub minimum_baseline: u64,
    pub reject_whole_interval: bool,
    pub consecutive_full_rejections: u64,
    pub audits_per_arm: u64,
}
impl AuditPolicy {
    pub fn registered() -> Self {
        Self {
            cadence: 64,
            max_batches: 16_404,
            final_tail_batches: 20,
            margin: 24,
            minimum_baseline: 500,
            reject_whole_interval: true,
            consecutive_full_rejections: 3,
            audits_per_arm: 258,
        }
    }
    fn validate(&self) -> Result<()> {
        if self != &Self::registered() {
            return Err(invalid("audited acceptance policy differs from FIT"));
        }
        Ok(())
    }
}

/// A separately produced, independently reviewed qualification packet. Raw
/// artifacts remain pinned, not replaced by this operational summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qualification {
    pub schema: String,
    pub execution: ExecutionIdentity,
    pub checks: Vec<QualifiedCheck>,
    pub g_init_comparison_or_waiver: FilePin,
    pub accepted_initlock_disposition: FilePin,
    pub residual_fit: FilePin,
    pub b3_psucc: u64,
    pub b3_full: u64,
    pub top1: u64,
    pub bar_i: u64,
    pub epoch: u64,
    pub updates_applied: u64,
    pub reference_capture_scope: String,
    pub full_rows: u64,
    pub psucc_rows: u64,
    pub psucc_reference_top1: u64,
    pub occupied_cells: u64,
    pub distinct_vectors: u64,
    pub tie_mass: u64,
    pub negative_native_exits: [i32; 2],
    pub sensitivity_predicate: String,
    pub combine_sensitive: u64,
    pub relation_sensitive: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualifiedCheck {
    pub name: String,
    pub status: String,
    pub artifact: FilePin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Calibration {
    pub schema: String,
    pub execution: ExecutionIdentity,
    /// Extrapolated complete campaign cost, not a measured 516-audit run.
    pub projected_added_gate_ns: u64,
    pub ungated_two_arm_dose_ns: u64,
    pub denominator_provenance: String,
    pub audits_both_arms: u64,
    pub complete_gate_io_included: bool,
    pub shared_setup_excluded_from_ratio: bool,
    pub rate_usd_micros_per_hour: u64,
    pub gpu_count: u32,
    pub all_in_resource_seconds: u64,
    pub staging_archive_reserve_bytes: u64,
    pub host_available_bytes: u64,
    pub measurement_artifacts: Vec<FilePin>,
    pub measurement_protocol: FilePin,
    pub resource_admission: FilePin,
}
fn validate_ratio(added: u64, ungated: u64) -> Result<()> {
    if added == 0
        || ungated == 0
        || added > 5_500_000_000_000
        || u128::from(added) * 100 > u128::from(ungated) * 85
    {
        return Err(invalid(
            "gate readiness exceeds 85 percent or 5500 seconds; replan BEFORE training",
        ));
    }
    Ok(())
}
fn validate_reserve(available: u64, reserve: u64) -> Result<()> {
    let required = (30 * 1024 * 1024 * 1024u64)
        .checked_add(reserve)
        .ok_or_else(|| invalid("storage reserve overflow"))?;
    if reserve == 0 || available < required {
        return Err(invalid("fresh storage floor plus actual reserve required"));
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditedPlan {
    pub schema: String,
    pub status: String,
    pub scope: String,
    pub execution: ExecutionIdentity,
    pub config: DriverConfig,
    pub parent_sheet: FilePin,
    pub sheet: FilePin,
    pub anchor_manifest: FilePin,
    pub anchor_manifest_sha256: String,
    pub anchor_policy: AnchorPolicy,
    pub anchor_csv_sha256: String,
    pub policy: AuditPolicy,
    pub qualification: FilePin,
    pub calibration: FilePin,
    /// Actual later dose admission/rate; qualification rental is historical
    /// measurement provenance and cannot authorize this resource.
    pub resource_admission: FilePin,
    pub costed_dose_decision: FilePin,
    pub independent_review: FilePin,
    pub transfer: corpus::CorpusSource,
    pub run_state_directory: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditedAdmission {
    pub schema: String,
    pub plan_file: FilePin,
    pub plan: AuditedPlan,
    pub owner_release: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditedOwner {
    pub schema: String,
    pub owner: String,
    pub decision: String,
    pub scope: String,
    pub plan_file_sha256: String,
    pub receipt_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditedReview {
    pub reviewer: String,
    pub verdict: String,
    pub scope: String,
    pub execution: ExecutionIdentity,
    pub reviewed_plan_payload_sha256: String,
    pub receipt_url: String,
}
pub fn review_payload_sha256(plan: &AuditedPlan) -> Result<String> {
    let mut v = serde_json::to_value(plan).map_err(|e| invalid(e.to_string()))?;
    v.as_object_mut()
        .ok_or_else(|| invalid("plan object"))?
        .remove("independent_review");
    Ok(corpus::sha256(&encode(&v)?))
}
pub fn validate_config(config: &DriverConfig) -> Result<()> {
    attention_learning::validate_config(config)?;
    if config.seed != 1
        || config.read_spec != (ReadSpec::Relation { kept: 8, sweeps: 1 })
        || config.coordinate_block != training::CoordinateBlock::ONE
        || !matches!(&config.codebook_initialization, training::CodebookInitialization::PinnedTableV1 { table }
            if table.sha256 == "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1")
    {
        return Err(invalid(
            "audited scope requires unchanged A8 seed1 Relation K8, scalar schedule",
        ));
    }
    Ok(())
}
pub fn scope(config: &DriverConfig) -> Result<String> {
    validate_config(config)?;
    Ok(format!(
        "{SCOPE}-{}-seed1-attempts0..16404",
        if config.relation_learning_policy.is_learn() {
            "relation"
        } else {
            "frozen-zero"
        }
    ))
}
fn validate_header(plan: &AuditedPlan) -> Result<()> {
    validate_config(&plan.config)?;
    plan.policy.validate()?;
    for h in [
        &plan.execution.source_sha256,
        &plan.execution.kernel_sha256,
        &plan.execution.geometry_sha256,
        &plan.execution.executable_sha256,
        &plan.execution.driver_source_sha256,
        &plan.execution.config_sha256,
        &plan.execution.lineage_sha256,
        &plan.execution.sheet_sha256,
        &plan.anchor_manifest_sha256,
        &plan.anchor_csv_sha256,
    ] {
        corpus::validate_sha256(h)?;
    }
    if plan.schema != PLAN_SCHEMA
        || plan.status != "HELD_OWNER_ACTION_REQUIRED"
        || plan.scope != scope(&plan.config)?
        || plan.sheet.sha256 != FIT_SHEET_SHA256
        || plan.parent_sheet.sha256 != attention_learning::A8_SHEET_SHA256
        || plan.execution.sheet_sha256 != FIT_SHEET_SHA256
        || plan.execution.driver_source_sha256 != training::driver_source_sha256()
        || plan.execution.config_sha256 != corpus::sha256(&encode(&plan.config)?)
        || plan.execution.reviewed_source_commit.len() != 40
        || !plan
            .execution
            .reviewed_source_commit
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || plan.execution.base_source.len() != 40
        || !plan
            .execution
            .base_source
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || plan.anchor_policy
            != (AnchorPolicy {
                start_eligible_index: 1_049_856,
                count: 3_847,
                context: 192,
            })
        || plan.anchor_csv_sha256 != REGISTERED_CSV_SHA256
        || !plan.run_state_directory.is_absolute()
        || plan.transfer.split != corpus::Split::Dev
        || plan.transfer.expected_eligible_targets < 2000
        || [&plan.config.train, &plan.config.dev].iter().any(|s| {
            s.tokens.sha256 == plan.transfer.tokens.sha256
                || s.mask.sha256 == plan.transfer.mask.sha256
        })
    {
        return Err(invalid(
            "new audited schema/scope/FIT/anchor/source/policy binding differs",
        ));
    }
    Ok(())
}
fn pinned_json<T: serde::de::DeserializeOwned>(pin: &FilePin) -> Result<T> {
    serde_json::from_slice(&pin.read()?).map_err(|e| invalid(e.to_string()))
}
fn validate_plan(plan: &AuditedPlan) -> Result<TrainAnchorManifest> {
    validate_header(plan)?;
    let dose_resource =
        qualification::validate_resource(&plan.resource_admission, &plan.execution, true)?;
    let sheet = plan.sheet.read()?;
    let parent = plan.parent_sheet.read()?;
    if !std::str::from_utf8(&sheet)
        .map_err(|e| invalid(e.to_string()))?
        .lines()
        .any(|l| l.starts_with("TEST FITNESS: FIT"))
        || !std::str::from_utf8(&parent)
            .map_err(|e| invalid(e.to_string()))?
            .starts_with("# Learned attention link from an address-structured injective codebook")
    {
        return Err(invalid("missing actual registered FIT/parent sheets"));
    }
    attention_learning::validate_injective_table(&plan.config)?;
    let anchor: TrainAnchorManifest = pinned_json(&plan.anchor_manifest)?;
    if anchor.manifest_sha256 != plan.anchor_manifest_sha256
        || anchor.policy != plan.anchor_policy
        || anchor.row_csv_sha256 != REGISTERED_CSV_SHA256
    {
        return Err(invalid("anchor manifest differs"));
    }
    let q: Qualification = pinned_json(&plan.qualification)?;
    validate_qualification(plan, &q)?;
    let c: Calibration = pinned_json(&plan.calibration)?;
    validate_calibration(plan, &c)?;
    qualification::validate_costed_dose(plan, &c)?;
    let measured_resource =
        qualification::validate_resource(&c.resource_admission, &c.execution, false)?;
    if measured_resource.hardware_class != dose_resource.hardware_class
        || !(1..=2).contains(&dose_resource.gpu_count)
    {
        return Err(invalid(
            "dose hardware class/count differs from qualified same-hardware cost protocol",
        ));
    }
    let review: AuditedReview = pinned_json(&plan.independent_review)?;
    if !matches!(
        review.reviewer.as_str(),
        "AGY" | "Antigravity" | "antigravity" | "Claude" | "claude" | "DeepSeek" | "deepseek"
    ) || review.verdict != REVIEW_VERDICT
        || review.scope != plan.scope
        || review.execution != plan.execution
        || review.reviewed_plan_payload_sha256 != review_payload_sha256(plan)?
        || !valid_receipt_url(&review.receipt_url)
    {
        return Err(invalid("new exact-plan independent review required"));
    }
    Ok(anchor)
}
fn validate_qualification(plan: &AuditedPlan, q: &Qualification) -> Result<()> {
    const REQUIRED: [&str; 5] = [
        "canonical_initlock_c2",
        "required_negatives",
        "g_occ",
        "sensitivity",
        "audited_restore_epoch_cursor",
    ];
    if q.schema != "uor-r4.attention-audit-qualification/1"
        || !matches_qualified_execution(&plan.execution, &q.execution)
        || (
            q.b3_psucc,
            q.b3_full,
            q.top1,
            q.bar_i,
            q.epoch,
            q.updates_applied,
        ) != (1351, 4244, 133, 86, 0, 0)
        || q.reference_capture_scope != "DEFERRED_TO_DOSE"
        || (
            q.full_rows,
            q.psucc_rows,
            q.occupied_cells,
            q.distinct_vectors,
            q.tie_mass,
        ) != (14287, 3847, 256, 4096, 0)
        || q.negative_native_exits != [2, 2]
        || q.sensitivity_predicate != "both_probes_any_active_candidate_cost_differs_v1"
        || q.combine_sensitive == 0
        || q.relation_sensitive == 0
        || q.b3_psucc
            .checked_sub(q.psucc_reference_top1)
            .is_none_or(|net| net <= 500u64.max(q.bar_i))
        || q.checks.len() != REQUIRED.len()
    {
        return Err(invalid(
            "new-source qualification incomplete or inherited bars differ",
        ));
    }
    for name in REQUIRED {
        let matching: Vec<_> = q.checks.iter().filter(|c| c.name == name).collect();
        if matching.len() != 1 || matching[0].status != "PASS" {
            return Err(invalid("missing/duplicate/failed qualification check"));
        }
        let stage = qualification::validate_stage(name, &q.execution, &matching[0].artifact)?;
        if name == "g_occ" && stage.facts["psucc_reference_top1"] != json!(q.psucc_reference_top1) {
            return Err(invalid(
                "qualification reference summary differs from actual GPU reference row join",
            ));
        }
    }
    q.g_init_comparison_or_waiver.read()?;
    q.accepted_initlock_disposition.read()?;
    q.residual_fit.read()?;
    Ok(())
}
const RELATION_CONTROL_NAMES: [&str; 9] = [
    "relation_off_flock_parity",
    "relation_copy_successor_fixture",
    "relation_irrelevant_change_invariance",
    "relation_conflicting_successors",
    "relation_suffix_and_target_isolation",
    "relation_endpoint_causal_boundary",
    "relation_deterministic_tie_order",
    "relation_bounded_sweeps_and_refusals",
    "active_leaf_initialization_and_matched_off_path",
];
fn validate_relation_qualification(report: &Value) -> Result<()> {
    if report["schema"] != json!(crate::SCHEMA)
        || report["suite"] != "relation"
        || report["status"] != "PASS"
        || report["registered"] != 9
        || report["attempted"] != 9
        || report["passed"] != 9
        || report["failed"] != 0
        || report["not_run"] != 0
        || report["applied_updates"] != 0
        || report["training"] != "HELD"
    {
        return Err(invalid("canonical relation qualification incomplete"));
    }
    let controls = report["controls"]
        .as_array()
        .ok_or_else(|| invalid("qualification controls missing"))?;
    if controls.len() != 9 {
        return Err(invalid("qualification control inventory"));
    }
    for (i, (control, name)) in controls.iter().zip(RELATION_CONTROL_NAMES).enumerate() {
        if control["index"] != json!(i + 1)
            || control["name"] != name
            || control["status"] != "PASS"
        {
            return Err(invalid(
                "canonical relation control name/order/status differs",
            ));
        }
    }
    let locks = controls[8]["details"]["init_locks"]
        .as_array()
        .ok_or_else(|| invalid("structured INIT LOCK records missing"))?;
    if locks.len() != 2 {
        return Err(invalid("INIT LOCK seeds missing"));
    }
    let mut gates = vec![0; 45];
    for i in [0, 5, 10, 15] {
        gates[i] = 1;
    }
    for (record, seed) in locks.iter().zip([1, 2]) {
        if record["seed"] != seed
            || record["gates"] != json!(gates)
            || record["initialization"] != "active_leaf_v1"
            || record["treatment_control_snapshot_bit_identical"] != true
            || record["reload_bit_identical"] != true
            || record["applied_updates"] != 0
        {
            return Err(invalid("canonical INIT LOCK structured predicate fails"));
        }
    }
    Ok(())
}
fn validate_calibration(plan: &AuditedPlan, c: &Calibration) -> Result<()> {
    validate_ratio(c.projected_added_gate_ns, c.ungated_two_arm_dose_ns)?;
    validate_reserve(c.host_available_bytes, c.staging_archive_reserve_bytes)?;
    if c.schema != "uor-r4.attention-audit-calibration/1"
        || !matches_qualified_execution(&plan.execution, &c.execution)
        || c.audits_both_arms != 516
        || !c.complete_gate_io_included
        || !c.shared_setup_excluded_from_ratio
        || !matches!(
            c.denominator_provenance.as_str(),
            "same_hardware_measured" | "same_hardware_qualified_throughput_extrapolation"
        )
        || c.rate_usd_micros_per_hour == 0
        || c.rate_usd_micros_per_hour > 8_000_000
        || c.gpu_count == 0
        || c.gpu_count > 4
        || c.all_in_resource_seconds == 0
        || c.measurement_artifacts.is_empty()
    {
        return Err(invalid(
            "actual cost/reserve/measurement contract incomplete",
        ));
    }
    for p in &c.measurement_artifacts {
        p.read()?;
    }
    let resource = qualification::validate_resource(&c.resource_admission, &c.execution, false)?;
    if resource.rate_usd_micros_per_hour != c.rate_usd_micros_per_hour
        || resource.gpu_count != c.gpu_count
        || resource.host_available_bytes != c.host_available_bytes
        || resource.staging_archive_reserve_bytes != c.staging_archive_reserve_bytes
    {
        return Err(invalid(
            "calibration resource/rate differs from actual measurement admission",
        ));
    }
    qualification::validate_timing_protocol(c)?;
    Ok(())
}
fn executable_sha256() -> Result<String> {
    Ok(corpus::sha256(
        &fs::read(std::env::current_exe().map_err(|e| invalid(e.to_string()))?)
            .map_err(|e| invalid(e.to_string()))?,
    ))
}
fn validate_execution(plan: &AuditedPlan, driver: &PreparedDriver, executable: &str) -> Result<()> {
    if &plan.config != driver.config()
        || driver.lineage_sha256()? != plan.execution.lineage_sha256
        || executable != plan.execution.executable_sha256
        || driver.schedule_cycle() != 16_404
    {
        return Err(invalid("audited execution/driver lineage mismatch"));
    }
    Ok(())
}
#[cfg(feature = "cuda")]
fn validate_compiled(plan: &AuditedPlan) -> Result<()> {
    let compiled = crate::device::compiled_identity();
    for (key, h) in [
        ("source_sha256", &plan.execution.source_sha256),
        ("kernel_sha256", &plan.execution.kernel_sha256),
        ("geometry_sha256", &plan.execution.geometry_sha256),
    ] {
        if compiled[key].as_str() != Some(h.as_str()) {
            return Err(invalid("audited compiled identity differs"));
        }
    }
    Ok(())
}
pub fn check_plan(path: &Path) -> Result<Value> {
    let raw = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let plan: AuditedPlan = serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
    let manifest = validate_plan(&plan)?;
    let driver = PreparedDriver::load(plan.config.clone())?;
    validate_execution(&plan, &driver, &executable_sha256()?)?;
    let registered = anchor_binding(&plan, &driver)?;
    crate::attention_anchor_device::PreparedTrainAnchor::prepare(
        driver.train(),
        &manifest,
        &registered,
    )?;
    #[cfg(feature = "cuda")]
    validate_compiled(&plan)?;
    Ok(
        json!({"status":"PLAN_METADATA_VALID","scope":plan.scope,"plan_file_sha256":corpus::sha256(&raw),
        "compiled_identity_checked":cfg!(feature="cuda"),"cuda_context_created":false,"fresh_content_read":false,
        "owner_release":"STILL_REQUIRED","training":"HELD","readiness_is_pretraining_only":true}),
    )
}
fn anchor_binding(plan: &AuditedPlan, driver: &PreparedDriver) -> Result<RegisteredAnchorBinding> {
    Ok(RegisteredAnchorBinding {
        corpus: driver.train().binding().clone(),
        policy: plan.anchor_policy.clone(),
        manifest_sha256: plan.anchor_manifest_sha256.clone(),
        csv_sha256: plan.anchor_csv_sha256.clone(),
        sheet_sha256: plan.sheet.sha256.clone(),
        plan_sha256: corpus::sha256(&encode(plan)?),
    })
}
#[cfg(any(feature = "cuda", test))]
fn require_personal_terminal(lab_present: bool, stdin_tty: bool, stdout_tty: bool) -> Result<()> {
    if lab_present || !stdin_tty || !stdout_tty {
        return Err(Error::TrainingHeld);
    }
    Ok(())
}
#[cfg(not(feature = "cuda"))]
pub fn owner_release(_plan_path: &Path, _output: &Path) -> Result<()> {
    Err(Error::TrainingHeld)
}
#[cfg(feature = "cuda")]
pub fn owner_release(plan_path: &Path, output: &Path) -> Result<()> {
    use std::io::IsTerminal;
    require_personal_terminal(
        std::env::var_os("UOR_LAB").is_some(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    )?;
    let checked = check_plan(plan_path)?;
    let raw = fs::read(plan_path).map_err(|e| invalid(e.to_string()))?;
    let plan: AuditedPlan = serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
    if checked["plan_file_sha256"] != json!(corpus::sha256(&raw)) {
        return Err(invalid("plan changed after check"));
    }
    require_disjoint_directories(output, &plan.run_state_directory)?;
    if output.exists() || plan.run_state_directory.exists() {
        return Err(invalid("new release/state directories required"));
    }
    let phrase=format!("I am Casey; release D40 for {} only with the pinned active whole-interval audit through attempted batch 16404",plan.scope);
    println!(
        "{}\nType exactly: {phrase}",
        serde_json::to_string_pretty(&json!({"scope":plan.scope,"execution":plan.execution,
            "plan_file_sha256":corpus::sha256(&raw),"private_artifacts_omitted":true}))
        .map_err(|e| invalid(e.to_string()))?
    );
    std::io::stdout()
        .flush()
        .map_err(|e| invalid(e.to_string()))?;
    let mut typed = String::new();
    std::io::stdin()
        .read_line(&mut typed)
        .map_err(|e| invalid(e.to_string()))?;
    if typed.trim_end_matches(['\r', '\n']) != phrase {
        return Err(Error::TrainingHeld);
    }
    if fs::read(plan_path).map_err(|e| invalid(e.to_string()))? != raw {
        return Err(invalid("plan changed during personal confirmation"));
    }
    validate_plan(&plan)?;
    private_directory(output)?;
    let owner_path = fs::canonicalize(output)
        .map_err(|e| invalid(e.to_string()))?
        .join("owner.json");
    let owner = AuditedOwner {
        schema: OWNER_SCHEMA.into(),
        owner: "Casey".into(),
        decision: OWNER_DECISION.into(),
        scope: plan.scope.clone(),
        plan_file_sha256: corpus::sha256(&raw),
        receipt_url: format!("owner-terminal:{}", owner_path.display()),
    };
    let owner_pin = persist(&owner_path, &owner)?;
    let admission = AuditedAdmission {
        schema: ADMISSION_SCHEMA.into(),
        plan_file: FilePin {
            path: fs::canonicalize(plan_path).map_err(|e| invalid(e.to_string()))?,
            sha256: corpus::sha256(&raw),
        },
        plan,
        owner_release: owner_pin,
    };
    persist(&output.join("admission.json"), &admission)?;
    sync_directory(output)
}
#[cfg(any(feature = "cuda", test))]
fn persist<T: Serialize>(path: &Path, value: &T) -> Result<FilePin> {
    let bytes = encode(value)?;
    let mut file = private_file(path)?;
    file.write_all(&bytes).map_err(|e| invalid(e.to_string()))?;
    file.sync_all().map_err(|e| invalid(e.to_string()))?;
    sync_directory(path.parent().ok_or_else(|| invalid("artifact parent"))?)?;
    let stored = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    if stored != encode(value)? {
        return Err(invalid("artifact readback differs"));
    }
    Ok(FilePin {
        path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&stored),
    })
}
#[cfg(any(feature = "cuda", test))]
fn private_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(|e| invalid(e.to_string()))?;
        // Explicit modes avoid reliance on the operator's umask.
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))
            .map_err(|e| invalid(e.to_string()))?;
        sync_directory(path)?;
        sync_directory(
            path.parent()
                .ok_or_else(|| invalid("private directory parent"))?,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(invalid(
            "private operational artifacts require Unix permissions",
        ))
    }
}
#[cfg(any(feature = "cuda", test))]
fn private_file(path: &Path) -> Result<fs::File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let parent = path
            .parent()
            .ok_or_else(|| invalid("private artifact parent"))?;
        let metadata = fs::symlink_metadata(parent).map_err(|e| invalid(e.to_string()))?;
        if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
            return Err(invalid("operational artifact parent must be private 0700"));
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .map_err(|e| invalid(e.to_string()))?;
        file.set_permissions(fs::Permissions::from_mode(0o600))
            .map_err(|e| invalid(e.to_string()))?;
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Err(invalid(
            "private operational artifacts require Unix permissions",
        ))
    }
}
#[cfg(feature = "cuda")]
fn audited_lock(state: &Path, manifest_sha256: &str, scope: &str) -> Result<RunLock> {
    private_directory(state)?;
    let path = state.join("active.lock");
    let record = lock_record(manifest_sha256)?;
    let pin = persist(&path, &record)?;
    let guard = RunLock {
        path,
        sha256: pin.sha256,
    };
    persist(
        &state.join("registration.json"),
        &json!({"manifest_sha256":manifest_sha256,"scope":scope}),
    )?;
    Ok(guard)
}
#[cfg(any(feature = "cuda", test))]
fn settle(machine: &mut AuditMachine, path: &Path, record: &DurableRecord) -> Result<FilePin> {
    let pin = persist(path, record)?;
    if pin.sha256 != record.sha256()? {
        return Err(invalid("durable record hash domain differs"));
    }
    machine.acknowledge(&pin.sha256)?;
    Ok(pin)
}
#[cfg(any(feature = "cuda", test))]
fn persist_interval_linkage(
    path: &Path,
    steps: &FilePin,
    audit: &FilePin,
    attempted: &FilePin,
    continuation: &FilePin,
    record: &DurableRecord,
) -> Result<FilePin> {
    persist(
        path,
        &json!({"steps":steps,"audit":audit,"attempted_checkpoint":attempted,
        "continuation_checkpoint":continuation,"record_sha256":record.sha256()?}),
    )
}
#[cfg(any(feature = "cuda", test))]
fn persist_progress<T: Serialize>(
    path: &Path,
    admission: &str,
    checkpoint: &FilePin,
    passing: &FilePin,
    machine: &T,
    q1: &Value,
) -> Result<FilePin> {
    persist(
        path,
        &json!({"schema":"uor-r4.attention-audit-checkpoint/1",
        "admission_sha256":admission,"checkpoint":checkpoint,"passing_checkpoint":passing,
        "lifecycle":machine,"q1":q1,"resume":"NOT_AUTHORIZED"}),
    )
}
#[cfg(any(feature = "cuda", test))]
fn append_step_fsync_and_record(
    file: &mut fs::File,
    machine: &mut AuditMachine,
    receipt: &training::StepReceipt,
) -> Result<usize> {
    let mut line = encode(receipt)?;
    line.push(b'\n');
    file.write_all(&line)
        .and_then(|_| file.sync_all())
        .map_err(|e| invalid(e.to_string()))?;
    machine.record_step(crate::attention_audit::StepEvidence {
        before: machine.stamp()?,
        family: machine.expected_family(),
        applied: receipt.applied,
        sensitive: receipt.any_candidate_cost_differs_from_identity,
        policy_refused: receipt.apply_refused_by_policy,
        artifact_sha256: corpus::sha256(&line),
    })?;
    Ok(line.len())
}

#[cfg(any(feature = "cuda", test))]
fn failure_payload(
    error: &str,
    cursor: &training::Cursor,
    readback: Option<Result<FilePin>>,
) -> Value {
    let observation = match readback {
        Some(Ok(pin)) => json!({"status":"CAPTURED","artifact":pin}),
        Some(Err(error)) => json!({"status":"UNAVAILABLE","error":error.to_string()}),
        None => json!({"status":"NO_LIVE_MODEL"}),
    };
    json!({"status":"INSTRUMENT_OR_EXECUTION_FAILURE","error":error,"last_verified_cursor":cursor,
        "failed_model_readback":observation,"coherent_driver_checkpoint_claimed":false,
        "mechanism_negative":false,"fresh_evaluations":0,"resume_authorized":false,
        "next":"Preserve all raw artifacts; external review required before a new attempt"})
}

#[cfg(any(feature = "cuda", test))]
fn validate_owner(
    plan: &AuditedPlan,
    plan_file: &FilePin,
    owner_pin: &FilePin,
    owner: &AuditedOwner,
) -> Result<()> {
    if owner.schema != OWNER_SCHEMA
        || owner.owner != "Casey"
        || owner.decision != OWNER_DECISION
        || owner.scope != plan.scope
        || owner.plan_file_sha256 != plan_file.sha256
        || owner.receipt_url != format!("owner-terminal:{}", owner_pin.path.display())
    {
        return Err(Error::TrainingHeld);
    }
    Ok(())
}

#[cfg(feature = "cuda")]
struct AuditedRelease {
    inner: ValidatedRelease,
    plan: AuditedPlan,
    plan_sha256: String,
}

/// No Deserialize, public fields or public constructor: only this controller's
/// fully validated successor admission can issue an exact sealed-boundary permit.
#[cfg(feature = "cuda")]
pub(crate) struct ControllerAuditPermit {
    plan_sha256: String,
    anchor_manifest_sha256: String,
    sheet_sha256: String,
    execution: ExecutionIdentity,
    stamp: crate::attention_audit::StateStamp,
    snapshot: crate::attention_audit::SnapshotPin,
}
#[cfg(feature = "cuda")]
impl ControllerAuditPermit {
    pub(crate) fn validate(
        &self,
        anchor: &crate::attention_anchor_device::PreparedTrainAnchor,
        boundary: &crate::attention_anchor_device::ValidatedBoundary,
    ) -> Result<()> {
        let b = boundary.binding();
        if anchor.registration().plan_sha256 != self.plan_sha256
            || anchor.registration().manifest_sha256 != self.anchor_manifest_sha256
            || b.plan_sha256 != self.plan_sha256
            || b.anchor_manifest_sha256 != self.anchor_manifest_sha256
            || b.sheet_sha256 != self.sheet_sha256
            || b.stamp != self.stamp
            || b.snapshot != self.snapshot
            || b.config_sha256 != self.execution.config_sha256
            || b.lineage_sha256 != self.execution.lineage_sha256
            || b.source_sha256 != self.execution.source_sha256
            || b.kernel_sha256 != self.execution.kernel_sha256
            || b.geometry_sha256 != self.execution.geometry_sha256
        {
            return Err(Error::TrainingHeld);
        }
        Ok(())
    }
}
#[cfg(feature = "cuda")]
fn release(
    driver: &PreparedDriver,
    admission: &AuditedAdmission,
) -> Result<(AuditedRelease, TrainAnchorManifest)> {
    if admission.schema != ADMISSION_SCHEMA {
        return Err(Error::TrainingHeld);
    }
    let raw = admission.plan_file.read()?;
    let original: AuditedPlan = serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
    if encode(&original)? != encode(&admission.plan)? {
        return Err(invalid("embedded plan differs from actual plan file"));
    }
    let anchor = validate_plan(&original)?;
    validate_execution(&original, driver, &executable_sha256()?)?;
    validate_compiled(&original)?;
    let owner: AuditedOwner = pinned_json(&admission.owner_release)?;
    validate_owner(
        &original,
        &admission.plan_file,
        &admission.owner_release,
        &owner,
    )?;
    let inner = ValidatedRelease {
        manifest: ReleaseManifest {
            schema: ADMISSION_SCHEMA.into(),
            scope: original.scope.clone(),
            execution: original.execution.clone(),
            sheet: original.sheet.clone(),
            claude_review: original.independent_review.clone(),
            owner_release: admission.owner_release.clone(),
            run_state_directory: original.run_state_directory.clone(),
        },
        manifest_sha256: corpus::sha256(&encode(admission)?),
        max_batches: 16_404,
    };
    Ok((
        AuditedRelease {
            inner,
            plan_sha256: corpus::sha256(&encode(&original)?),
            plan: original,
        },
        anchor,
    ))
}

#[cfg(feature = "cuda")]
fn snapshot_pin(
    checkpoint: &training::DriverCheckpoint,
    file: &FilePin,
) -> Result<crate::attention_audit::SnapshotPin> {
    let hashes = crate::snapshot::stored_snapshot_hash_domains(&checkpoint.model)?;
    if hashes.model_storage_sha256 != checkpoint.model_sha256 {
        return Err(invalid("stored model digest differs"));
    }
    Ok(crate::attention_audit::SnapshotPin {
        checkpoint_sha256: file.sha256.clone(),
        storage_sha256: hashes.model_storage_sha256,
        snapshot_sha256: hashes.model_snapshot_sha256,
    })
}

/// Shared physical rollback used by the real breach branch and the separately
/// authorized controlled production qualifier. Caller must seal intent and
/// destroy the old Device/scratch first. This never replays model updates.
#[cfg(feature = "cuda")]
fn restore_passing_device(
    driver: &PreparedDriver,
    file: &FilePin,
    passing: &crate::attention_audit::SnapshotPin,
    target: u64,
) -> Result<(
    crate::device::Device,
    training::Cursor,
    training::DriverCheckpoint,
)> {
    let checkpoint: training::DriverCheckpoint = pinned_json(file)?;
    if snapshot_pin(&checkpoint, file)? != *passing
        || target < checkpoint.cursor.completed_batches
        || target > 16404
    {
        return Err(invalid(
            "immutable passing checkpoint/rollback bound differs",
        ));
    }
    let (model, mut cursor) = driver.restore(checkpoint, 0)?;
    while cursor.completed_batches < target {
        cursor = driver.advance(&cursor, false)?;
    }
    driver.validate_cursor(&cursor)?;
    model.synchronize()?;
    let continuation = driver.checkpoint(&model, cursor)?;
    let hashes = crate::snapshot::stored_snapshot_hash_domains(&continuation.model)?;
    if hashes.model_storage_sha256 != passing.storage_sha256
        || hashes.model_snapshot_sha256 != passing.snapshot_sha256
        || model.epoch()? != cursor.accepted_updates
    {
        return Err(invalid("actual physical passing restore differs"));
    }
    Ok((model, cursor, continuation))
}
#[cfg(feature = "cuda")]
fn audit_boundary(
    driver: &PreparedDriver,
    release: &AuditedRelease,
    machine: &AuditMachine,
    anchor: &crate::attention_anchor_device::PreparedTrainAnchor,
    model: &crate::device::Device,
    sealed: (&FilePin, &crate::attention_audit::SnapshotPin),
    path: &Path,
) -> Result<(crate::attention_audit::AuditEvidence, FilePin)> {
    use crate::attention_audit::Phase;
    let (checkpoint_file, pin) = sealed;
    if !matches!(machine.phase(), Phase::AwaitBaseline | Phase::AwaitAudit) {
        return Err(Error::TrainingHeld);
    }
    let stamp = machine.stamp()?;
    let b = crate::attention_anchor_device::BoundaryBinding {
        checkpoint_file: checkpoint_file.clone(),
        snapshot: pin.clone(),
        stamp: stamp.clone(),
        plan_sha256: release.plan_sha256.clone(),
        sheet_sha256: release.plan.sheet.sha256.clone(),
        anchor_manifest_sha256: release.plan.anchor_manifest_sha256.clone(),
        lineage_sha256: release.plan.execution.lineage_sha256.clone(),
        config_sha256: release.plan.execution.config_sha256.clone(),
        source_sha256: release.plan.execution.source_sha256.clone(),
        kernel_sha256: release.plan.execution.kernel_sha256.clone(),
        geometry_sha256: release.plan.execution.geometry_sha256.clone(),
    };
    let boundary = crate::attention_anchor_device::validate_boundary(
        anchor,
        &b,
        &stamp,
        &checkpoint_file.read()?,
        driver,
    )?;
    let permit = ControllerAuditPermit {
        plan_sha256: release.plan_sha256.clone(),
        anchor_manifest_sha256: release.plan.anchor_manifest_sha256.clone(),
        sheet_sha256: release.plan.sheet.sha256.clone(),
        execution: release.plan.execution.clone(),
        stamp: stamp.clone(),
        snapshot: pin.clone(),
    };
    let started = std::time::Instant::now();
    let evaluated = crate::attention_anchor_device::evaluate_boundary(
        &permit, anchor, driver, model, &boundary,
    )?;
    if machine.stamp()? != stamp {
        return Err(invalid("audit changed controller state"));
    }
    let admitted = evaluated["aggregate"]["target_admitted_rows"]
        .as_u64()
        .ok_or_else(|| invalid("missing device admission aggregate"))?;
    let artifact = persist(
        path,
        &json!({"schema":"uor-r4.attention-audit-artifact/1","evaluation":evaluated,
        "device_and_snapshot_elapsed_ns":started.elapsed().as_nanos().to_string(),"admission_sha256":release.inner.manifest_sha256}),
    )?;
    Ok((
        crate::attention_audit::AuditEvidence {
            stamp,
            snapshot: pin.clone(),
            admitted,
            artifact_sha256: artifact.sha256.clone(),
        },
        artifact,
    ))
}

#[cfg(feature = "cuda")]
fn dev_evaluation(
    driver: &PreparedDriver,
    release: &AuditedRelease,
    model: &crate::device::Device,
    checkpoint: &training::DriverCheckpoint,
    path: &Path,
) -> Result<(Value, FilePin)> {
    let value = driver.evaluate_checkpoint(model, checkpoint)?;
    attention_learning::verify_evaluation(&value, checkpoint)?;
    let hashes = crate::snapshot::stored_snapshot_hash_domains(&checkpoint.model)?;
    let pin = persist(
        path,
        &json!({"schema":"uor-r4.attention-audit-dev-evaluation/1",
        "admission_sha256":release.inner.manifest_sha256,"checkpoint_sha256":corpus::sha256(&encode(checkpoint)?),
        "model_hash_domains":hashes,"evaluation":value,"fresh_evaluations":0}),
    )?;
    Ok((value, pin))
}

#[cfg(feature = "cuda")]
fn relation_off(
    driver: &PreparedDriver,
    release: &AuditedRelease,
    checkpoint: &training::DriverCheckpoint,
    state: &Path,
) -> Result<Value> {
    if !driver.config().relation_learning_policy.is_learn() {
        return Ok(json!({"status":"NOT_APPLICABLE_FROZEN_ZERO"}));
    }
    let mut snapshot: crate::snapshot::Snapshot =
        serde_json::from_value(checkpoint.model.clone()).map_err(|e| invalid(e.to_string()))?;
    let combine = driver.config().spec.blocks as usize
        * crate::metadata::BinderPlan::new(driver.config().spec.context as usize)?.stages();
    snapshot
        .gates
        .get_mut(combine..)
        .ok_or_else(|| invalid("relation inventory"))?
        .fill(0);
    let mut ablated = checkpoint.clone();
    ablated.model = serde_json::to_value(snapshot).map_err(|e| invalid(e.to_string()))?;
    ablated.model_sha256 = corpus::sha256(&encode(&ablated.model)?);
    driver.validate_checkpoint(&ablated)?;
    let cp_pin = persist(&state.join("relation-off-checkpoint.json"), &ablated)?;
    let (model, _) = driver.restore(ablated.clone(), 0)?;
    let (evaluation, evaluation_pin) = dev_evaluation(
        driver,
        release,
        &model,
        &ablated,
        &state.join("relation-off-dev.json"),
    )?;
    let b3 = attention_learning::evidence::summarize(
        &evaluation,
        &attention_learning::dev_row_identities(driver.config())?,
    )?;
    Ok(
        json!({"status":"OBSERVED_PENDING_REGISTERED_DISCRIMINATOR","checkpoint":cp_pin,"evaluation":evaluation_pin,
        "b3":b3,"changed_storage":"relation gate trits only set to zero","fresh_evaluations":0}),
    )
}

/// No resume entry point exists. A partial or failed attempt is retained and
/// requires external review; source implementation is not a compute release.
#[cfg(feature = "cuda")]
pub fn run(config: DriverConfig, admission: AuditedAdmission, out: &Path) -> Result<()> {
    use crate::attention_audit::{AuditAction, Phase, RestoreEvidence, RunBinding};
    validate_config(&config)?;
    let driver = PreparedDriver::load(config)?;
    let (release, manifest) = release(&driver, &admission)?;
    let prepared = crate::attention_anchor_device::PreparedTrainAnchor::prepare(
        driver.train(),
        &manifest,
        &anchor_binding(&release.plan, &driver)?,
    )?;
    require_disjoint_directories(out, &release.inner.manifest.run_state_directory)?;
    private_directory(out)?;
    let _lock = audited_lock(
        &release.inner.manifest.run_state_directory,
        &release.inner.manifest_sha256,
        &release.plan.scope,
    )?;
    persist(&out.join("admission.json"), &admission)?;
    let state = &release.inner.manifest.run_state_directory;
    let started = std::time::Instant::now();
    let mut model = None;
    let mut last_observed_cursor = driver.initial_cursor();
    let result = (|| -> Result<()> {
        model = Some(driver.new_model(0)?);
        let initial = driver.checkpoint(model.as_ref().unwrap(), driver.initial_cursor())?;
        attention_learning::validate_initial_storage(driver.config(), &initial.model)?;
        let initial_file = persist(&state.join("driver-00000.json"), &initial)?;
        let initial_pin = snapshot_pin(&initial, &initial_file)?;
        let stages = crate::metadata::BinderPlan::new(driver.config().spec.context as usize)?
            .stages() as u32;
        let combine = driver.config().spec.blocks * stages;
        let total = driver
            .config()
            .read_spec
            .gate_count(driver.config().spec.blocks, stages)?;
        let mut machine = AuditMachine::new(
            RunBinding {
                plan_sha256: release.plan_sha256.clone(),
                release_sha256: release.inner.manifest_sha256.clone(),
                lineage_sha256: driver.lineage_sha256()?,
                sheet_sha256: release.plan.sheet.sha256.clone(),
                anchor_sha256: release.plan.anchor_manifest_sha256.clone(),
                combine_gates: combine,
                relation_gates: total - combine,
            },
            !driver.config().relation_learning_policy.is_learn(),
            model.as_ref().unwrap().binding().model,
        )?;
        let (baseline_audit, baseline_artifact) = audit_boundary(
            &driver,
            &release,
            &machine,
            &prepared,
            model.as_ref().unwrap(),
            (&initial_file, &initial_pin),
            &state.join("anchor-00000.json"),
        )?;
        let record = machine.baseline(baseline_audit)?;
        let mut records = vec![settle(
            &mut machine,
            &state.join("baseline-record.json"),
            &record,
        )?];
        let mut audit_artifacts = vec![baseline_artifact];
        if machine.phase() == Phase::Stopped {
            persist(
                &out.join("report.json"),
                &json!({"status":"INSTRUMENT_UNREACHABLE","stop_reasons":machine.stop_reasons(),
                "completed_batches":0,"fresh_evaluations":0,"mechanism_negative":false,"records":records,"audits":audit_artifacts}),
            )?;
            return Ok(());
        }
        let (baseline_dev, baseline_dev_pin) = dev_evaluation(
            &driver,
            &release,
            model.as_ref().unwrap(),
            &initial,
            &state.join("baseline-dev.json"),
        )?;
        let dev_rows = attention_learning::dev_row_identities(driver.config())?;
        let init_b3 = attention_learning::evidence::summarize(&baseline_dev, &dev_rows)?;
        if (init_b3.psucc.target_admitted, init_b3.full.target_admitted) != (1351, 4244)
            || baseline_dev["aggregate"]["top1_correct"] != json!(133)
        {
            return Err(invalid("inherited initial A8 DEV reproduction failed"));
        }
        // The baseline model/checkpoint, TRAIN audit and inherited DEV baseline
        // are durably sealed before the only update-capability issuance.
        let capability = training::issue_corpus_update_admission(&driver, &release.inner)?;
        let mut cursor = driver.initial_cursor();
        let mut passing_checkpoint = initial.clone();
        let mut passing_file = initial_file.clone();
        let mut current_checkpoint = initial;
        let mut current_file = initial_file;
        let mut q1 = Value::Null;
        while matches!(machine.phase(), Phase::Ready | Phase::Open) {
            let begin = machine.completed();
            let steps_path = state.join(format!("steps-{begin:05}.jsonl"));
            let mut steps = private_file(&steps_path)?;
            sync_directory(state)?;
            while matches!(machine.phase(), Phase::Ready | Phase::Open) {
                let before = cursor;
                driver.validate_cursor(&before)?;
                if before.completed_batches != machine.completed()
                    || before.accepted_updates != machine.stamp()?.epoch
                {
                    return Err(invalid("live cursor differs before audited transition"));
                }
                let expected_coordinate = driver.coordinate_at(before.completed_batches)?;
                let expected_family = machine.expected_family();
                let receipt = driver.transition(model.as_mut().unwrap(), &before, &capability)?;
                if receipt.coordinate != expected_coordinate
                    || receipt.cursor_before != before
                    || receipt.cursor_after != driver.advance(&before, receipt.applied)?
                    || receipt.model_epoch != receipt.cursor_after.accepted_updates
                    || receipt.lineage_sha256 != driver.lineage_sha256()?
                {
                    return Err(invalid(
                        "actual transition receipt differs from driver/attempt schedule",
                    ));
                }
                if machine.expected_family() != expected_family {
                    return Err(invalid("controller family changed before durable step"));
                }
                append_step_fsync_and_record(&mut steps, &mut machine, &receipt)?;
                cursor = receipt.cursor_after;
                last_observed_cursor = cursor;
            }
            drop(steps);
            let steps_bytes = fs::read(&steps_path).map_err(|e| invalid(e.to_string()))?;
            let steps_pin = FilePin {
                path: fs::canonicalize(&steps_path).map_err(|e| invalid(e.to_string()))?,
                sha256: corpus::sha256(&steps_bytes),
            };
            model.as_ref().unwrap().synchronize()?;
            let attempted = driver.checkpoint(model.as_ref().unwrap(), cursor)?;
            let attempted_file = persist(
                &state.join(format!("driver-{:05}.json", cursor.completed_batches)),
                &attempted,
            )?;
            let attempted_pin = snapshot_pin(&attempted, &attempted_file)?;
            machine.seal_boundary(attempted_pin.clone())?;
            let (audit, audit_file) = audit_boundary(
                &driver,
                &release,
                &machine,
                &prepared,
                model.as_ref().unwrap(),
                (&attempted_file, &attempted_pin),
                &state.join(format!("anchor-{:05}.json", cursor.completed_batches)),
            )?;
            audit_artifacts.push(audit_file.clone());
            let interval_record = match machine.audit(audit)? {
                AuditAction::Commit(record) => {
                    current_checkpoint = attempted.clone();
                    current_file = attempted_file.clone();
                    record
                }
                AuditAction::Restore(required) => {
                    // This intent is durable FAILED-state custody, never a
                    // prematurely asserted restore-success record.
                    persist(
                        &state.join(format!(
                            "breach-intent-{:05}.json",
                            cursor.completed_batches
                        )),
                        &json!({"schema":"uor-r4.attention-audit-restore-intent/1","status":"AUDIT_BREACH_RESTORE_REQUIRED",
                            "failed_checkpoint":attempted_file,"steps":steps_pin,"audit":audit_file,"requirement":required}),
                    )?;
                    let target = cursor.completed_batches;
                    drop(model.take()); // destroys prior-generation scratch/cache and Device
                    let raw = passing_file.read()?;
                    let authenticated: training::DriverCheckpoint =
                        serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
                    if encode(&authenticated)? != encode(&passing_checkpoint)?
                        || snapshot_pin(&authenticated, &passing_file)? != required.passing.snapshot
                    {
                        return Err(invalid("immutable passing checkpoint changed"));
                    }
                    let (restored_device, advanced, continuation) = restore_passing_device(
                        &driver,
                        &passing_file,
                        &required.passing.snapshot,
                        target,
                    )?;
                    model = Some(restored_device);
                    let restored = model.as_ref().unwrap();
                    // The passing checkpoint may predate several consecutive
                    // rejected intervals. Advance to the ABSOLUTE failed bound.
                    driver.validate_cursor(&advanced)?;
                    restored.synchronize()?;
                    let continuation_file = persist(
                        &state.join(format!("continuation-{target:05}.json")),
                        &continuation,
                    )?;
                    let observed = snapshot_pin(&continuation, &continuation_file)?;
                    if observed.storage_sha256 != required.passing.snapshot.storage_sha256
                        || observed.snapshot_sha256 != required.passing.snapshot.snapshot_sha256
                        || restored.epoch()? != required.passing.epoch
                    {
                        return Err(invalid(
                            "actual restored Device does not reproduce passing model",
                        ));
                    }
                    let restore_file = persist(
                        &state.join(format!("restore-{target:05}.json")),
                        &json!({"schema":"uor-r4.attention-audit-restore-observation/1",
                        "status":"RESTORED_OBSERVED","original_passing_checkpoint":passing_file,"original_passing_pin":required.passing.snapshot,
                        "continuation_checkpoint":continuation_file,"continuation_pin":observed,"advanced_cursor":advanced,
                        "observed_epoch":restored.epoch()?,"new_runtime_model":restored.binding().model,"next_generation":required.next_generation}),
                    )?;
                    let evidence = RestoreEvidence {
                        required: required.clone(),
                        restored_snapshot: required.passing.snapshot.clone(),
                        restored_epoch: restored.epoch()?,
                        advanced_completed: advanced.completed_batches,
                        new_runtime_model: restored.binding().model,
                        artifact_sha256: restore_file.sha256,
                    };
                    let record = machine.restored(evidence)?;
                    cursor = advanced;
                    last_observed_cursor = advanced;
                    current_checkpoint = continuation;
                    current_file = continuation_file;
                    record
                }
            };
            let rejected = matches!(&interval_record.kind,crate::attention_audit::RecordKind::Interval(i) if i.rejected);
            let record_path = state.join(format!("interval-{:05}.json", cursor.completed_batches));
            // Persist linkage to immutable raw steps and actual continuation
            // before the authoritative lifecycle acknowledgement.
            persist_interval_linkage(
                &state.join(format!(
                    "interval-artifacts-{:05}.json",
                    cursor.completed_batches
                )),
                &steps_pin,
                &audit_file,
                &attempted_file,
                &current_file,
                &interval_record,
            )?;
            records.push(settle(&mut machine, &record_path, &interval_record)?);
            if !rejected {
                passing_checkpoint = current_checkpoint.clone();
                passing_file = current_file.clone();
            }
            if machine.phase() == Phase::AwaitQ1 {
                let (evaluated, evaluation_pin) = dev_evaluation(
                    &driver,
                    &release,
                    model.as_ref().unwrap(),
                    &current_checkpoint,
                    &state.join("q1-dev.json"),
                )?;
                let q1_b3 = attention_learning::evidence::summarize(&evaluated, &dev_rows)?;
                let counters = machine.counters();
                let decision = attention_learning::evidence::q1_decision(
                    &init_b3,
                    &q1_b3,
                    counters[1].sensitive,
                    counters[2].sensitive,
                )?;
                let q1_pin = persist(
                    &state.join("q1.json"),
                    &json!({"schema":"uor-r4.attention-audit-q1/1","evaluation":evaluation_pin,
                    "checkpoint":current_file,"state":machine.stamp()?,"counters":machine.counters(),"init_b3":init_b3,"q1_b3":q1_b3,"decision":decision}),
                )?;
                let record = machine.parent_q1(
                    machine.stamp()?,
                    !decision.diagnostic_stop,
                    q1_pin.sha256.clone(),
                )?;
                records.push(settle(
                    &mut machine,
                    &state.join("q1-record.json"),
                    &record,
                )?);
                q1 = json!({"record":q1_pin,"decision":decision});
            }
            persist_progress(
                &state.join(format!("progress-{:05}.json", cursor.completed_batches)),
                &release.inner.manifest_sha256,
                &current_file,
                &passing_file,
                &machine,
                &q1,
            )?;
        }
        let full = machine.phase() == Phase::Complete;
        let mut final_dev = Value::Null;
        let mut ablation = Value::Null;
        if full {
            let (evaluation, pin) = dev_evaluation(
                &driver,
                &release,
                model.as_ref().unwrap(),
                &current_checkpoint,
                &state.join("final-dev.json"),
            )?;
            let b3 = attention_learning::evidence::summarize(&evaluation, &dev_rows)?;
            final_dev = json!({"evaluation":pin,"b3":b3});
            // Ablation is the unchanged parent's separately costed final DEV
            // control; release the parent Device before allocating its restore.
            drop(model.take());
            ablation = relation_off(&driver, &release, &current_checkpoint, state)?;
        }
        persist(
            &out.join("report.json"),
            &json!({"schema":"uor-r4.attention-audit-report/1",
            "status":if full {"COMPLETE_PENDING_REGISTERED_DISCRIMINATOR"} else {"STOP_FOR_DIAGNOSIS"},
            "scope":release.plan.scope,"admission_sha256":release.inner.manifest_sha256,"completed_batches":machine.completed(),
            "committed_accepted_updates":machine.committed_updates(),"counters":machine.counters(),"stop_reasons":machine.stop_reasons(),
            "baseline_dev":baseline_dev_pin,"baseline_b3":init_b3,"final_checkpoint":current_file,"q1":q1,
            "final_dev":final_dev,"relation_off":ablation,"records":records,"audits":audit_artifacts,
            "fresh_transfer":{"status":"SEALED_PENDING_PAIRED_DEV_DECISION","source":release.plan.transfer,"evaluations":0},
            "mechanism_negative":false,"learned_attention_result":"NOT_YET_SCORED","elapsed_ns":started.elapsed().as_nanos().to_string(),
            "resume":"NOT_AUTHORIZED","training_other_scopes":"HELD","m2":"HELD","stage_b":"HELD"}),
        )?;
        sync_directory(out)
    })();
    if let Err(error) = &result {
        // Keep the actual last Device outside the fallible orchestration so a
        // failed post-apply write cannot silently discard its unique model.
        // If the device cannot be read, retain that fact without claiming a
        // coherent DriverCheckpoint or successful custody.
        let readback = model.as_ref().map(|actual| {
            let snapshot = actual.snapshot()?;
            persist(&state.join("failed-model-storage.json"), &json!({"schema":"uor-r4.attention-audit-failed-model/1",
                "snapshot":snapshot,"last_verified_cursor":last_observed_cursor,"actual_epoch":actual.epoch()?,
                "runtime_model":actual.binding().model,"coherent_driver_checkpoint_claimed":false}))
        });
        let _ = persist(
            &out.join("failure.json"),
            &failure_payload(&error.to_string(), &last_observed_cursor, readback),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(unix)]
    fn operational_artifacts_have_explicit_private_modes() {
        use std::os::unix::fs::PermissionsExt;
        let path =
            std::env::temp_dir().join(format!("uor-audited-permissions-{}", std::process::id()));
        private_directory(&path).unwrap();
        let pin = persist(&path.join("private.json"), &json!({"synthetic":true})).unwrap();
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o700
        );
        assert_eq!(
            fs::metadata(&pin.path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(persist(&pin.path, &json!({"replacement":true})).is_err());
        fs::remove_file(pin.path).unwrap();
        fs::remove_dir(path).unwrap();
    }

    #[test]
    fn registered_policy_rejects_passive_or_expanded_work() {
        let p = AuditPolicy::registered();
        p.validate().unwrap();
        let mut bad = p.clone();
        bad.max_batches += 1;
        assert!(bad.validate().is_err());
        bad = p.clone();
        bad.reject_whole_interval = false;
        assert!(bad.validate().is_err());
        bad = p;
        bad.final_tail_batches = 0;
        assert!(bad.validate().is_err());
    }

    #[test]
    fn readiness_ratio_is_exact_and_never_a_live_stop() {
        assert!(validate_ratio(15, 100).is_ok());
        assert!(validate_ratio(85, 100).is_ok());
        assert!(validate_ratio(86, 100).is_err());
        assert!(validate_ratio(4_545_143_921_992, 5_682_267_814_481).is_ok());
        assert!(validate_ratio(5_500_000_000_001, 10_000_000_000_000).is_err());
        assert!(validate_ratio(0, 0).is_err());
        assert!(validate_ratio(u64::MAX, u64::MAX).is_err());
    }

    #[test]
    fn owner_is_personal_and_old_decision_does_not_release() {
        assert!(require_personal_terminal(true, true, true).is_err());
        assert!(require_personal_terminal(false, false, true).is_err());
        assert!(require_personal_terminal(false, true, true).is_ok());
        assert_ne!(
            OWNER_DECISION,
            "RELEASE_D4_FOR_THIS_ATTENTION_LEARNING_ARM_SEED_ONLY"
        );
    }

    fn pin(name: &str) -> FilePin {
        FilePin {
            path: PathBuf::from(format!("/metadata/{name}")),
            sha256: corpus::sha256(name.as_bytes()),
        }
    }
    pub(super) fn fixture() -> AuditedPlan {
        let config: DriverConfig = serde_json::from_str(include_str!(
            "../../../docs/labs/a8-training-configs-2026-10-19/evidence/a8-relation-seed1.json"
        ))
        .unwrap();
        AuditedPlan {
            schema: PLAN_SCHEMA.into(),
            status: "HELD_OWNER_ACTION_REQUIRED".into(),
            scope: scope(&config).unwrap(),
            execution: ExecutionIdentity {
                base_source: "1".repeat(40),
                reviewed_source_commit: "2".repeat(40),
                source_sha256: "3".repeat(64),
                kernel_sha256: "4".repeat(64),
                geometry_sha256: "5".repeat(64),
                executable_sha256: "6".repeat(64),
                driver_source_sha256: training::driver_source_sha256(),
                config_sha256: corpus::sha256(&encode(&config).unwrap()),
                lineage_sha256: "7".repeat(64),
                sheet_sha256: FIT_SHEET_SHA256.into(),
            },
            config,
            parent_sheet: FilePin {
                path: "/metadata/parent".into(),
                sha256: attention_learning::A8_SHEET_SHA256.into(),
            },
            sheet: FilePin {
                path: "/metadata/fit".into(),
                sha256: FIT_SHEET_SHA256.into(),
            },
            anchor_manifest: pin("anchor"),
            anchor_manifest_sha256: "8".repeat(64),
            anchor_policy: AnchorPolicy {
                start_eligible_index: 1_049_856,
                count: 3847,
                context: 192,
            },
            anchor_csv_sha256: REGISTERED_CSV_SHA256.into(),
            policy: AuditPolicy::registered(),
            qualification: pin("qualification"),
            calibration: pin("calibration"),
            resource_admission: pin("actual-dose-resource"),
            costed_dose_decision: pin("actual-costed-dose-decision"),
            independent_review: pin("review"),
            transfer: corpus::CorpusSource {
                split: corpus::Split::Dev,
                tokens: pin("fresh"),
                mask: pin("fresh-mask"),
                expected_payload_tokens: 2001,
                expected_eligible_targets: 2000,
            },
            run_state_directory: "/metadata/new-state".into(),
        }
    }
    #[test]
    fn new_header_cannot_inherit_parent_scope_source_or_guard_law() {
        let p = fixture();
        validate_header(&p).unwrap();
        let mut bad = p.clone();
        bad.schema = "uor-r4.attention-learning-owner-plan/2".into();
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.scope = attention_learning::scope(&bad.config).unwrap();
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.sheet.sha256 = attention_learning::A8_SHEET_SHA256.into();
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.execution.driver_source_sha256 = "f".repeat(64);
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.anchor_policy.start_eligible_index = 0;
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.policy.margin += 1;
        assert!(validate_header(&bad).is_err());
        bad = p.clone();
        bad.config.seed = 2;
        assert!(validate_header(&bad).is_err());
        bad = p;
        bad.transfer.tokens.sha256 = bad.config.dev.tokens.sha256.clone();
        assert!(validate_header(&bad).is_err());
    }
    #[test]
    fn review_binds_policy_inputs_cost_and_all_fields_except_own_pin() {
        let p = fixture();
        let expected = review_payload_sha256(&p).unwrap();
        let mut changed = p.clone();
        changed.independent_review = pin("other-review");
        assert_eq!(review_payload_sha256(&changed).unwrap(), expected);
        changed = p.clone();
        changed.calibration = pin("other-cost");
        assert_ne!(review_payload_sha256(&changed).unwrap(), expected);
        changed = p.clone();
        changed.anchor_manifest = pin("other-anchor");
        assert_ne!(review_payload_sha256(&changed).unwrap(), expected);
        changed = p;
        changed.policy.margin = 23;
        assert_ne!(review_payload_sha256(&changed).unwrap(), expected);
    }
    #[test]
    fn owner_binds_actual_raw_plan_domain_and_new_decision() {
        let p = fixture();
        let plan_file = pin("actual-pretty-plan-bytes");
        let owner_pin = pin("owner.json");
        let mut owner = AuditedOwner {
            schema: OWNER_SCHEMA.into(),
            owner: "Casey".into(),
            decision: OWNER_DECISION.into(),
            scope: p.scope.clone(),
            plan_file_sha256: plan_file.sha256.clone(),
            receipt_url: format!("owner-terminal:{}", owner_pin.path.display()),
        };
        validate_owner(&p, &plan_file, &owner_pin, &owner).unwrap();
        owner.plan_file_sha256 = corpus::sha256(&encode(&p).unwrap());
        assert!(validate_owner(&p, &plan_file, &owner_pin, &owner).is_err());
        owner.plan_file_sha256 = plan_file.sha256.clone();
        owner.decision = "RELEASE_D4_FOR_THIS_ATTENTION_LEARNING_ARM_SEED_ONLY".into();
        assert!(validate_owner(&p, &plan_file, &owner_pin, &owner).is_err());
    }
    #[test]
    fn durable_write_failure_does_not_acknowledge_or_double_commit() {
        use crate::attention_audit::{AuditEvidence, Phase, RunBinding, SnapshotPin};
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "uor-audited-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        private_directory(&directory).unwrap();
        let h = "a".repeat(64);
        let mut machine = AuditMachine::new(
            RunBinding {
                plan_sha256: h.clone(),
                release_sha256: h.clone(),
                lineage_sha256: h.clone(),
                sheet_sha256: h.clone(),
                anchor_sha256: h.clone(),
                combine_gates: 1,
                relation_gates: 1,
            },
            false,
            1,
        )
        .unwrap();
        let record = machine
            .baseline(AuditEvidence {
                stamp: machine.stamp().unwrap(),
                snapshot: SnapshotPin {
                    checkpoint_sha256: h.clone(),
                    storage_sha256: h.clone(),
                    snapshot_sha256: h.clone(),
                },
                admitted: 600,
                artifact_sha256: h,
            })
            .unwrap();
        let blocked = directory.join("existing.json");
        fs::write(&blocked, b"original retained bytes").unwrap();
        assert!(settle(&mut machine, &blocked, &record).is_err());
        assert_eq!(machine.phase(), Phase::AwaitDurable);
        assert_eq!(fs::read(&blocked).unwrap(), b"original retained bytes");
        let stored = settle(&mut machine, &directory.join("baseline.json"), &record).unwrap();
        assert_eq!(stored.read().unwrap(), encode(&record).unwrap());
        assert_eq!(machine.phase(), Phase::Ready);
        assert!(machine.acknowledge(&stored.sha256).is_err());
        assert_eq!(machine.committed_updates(), 0);
        // Preserve only synthetic fixture artifacts until this successful test
        // ends; no project/evidence files are removed.
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn reserve_and_measurements_refuse_overflow_and_unmeasured_zero() {
        assert!(validate_reserve(u64::MAX, u64::MAX).is_err());
        assert!(validate_reserve(40 * 1024 * 1024 * 1024, 1024 * 1024 * 1024).is_ok());
        assert!(validate_ratio(0, 100).is_err());
    }
    #[test]
    fn failed_model_readback_never_implies_resume_or_complete_custody() {
        let cursor = training::Cursor::initial();
        let value = failure_payload(
            "original failure",
            &cursor,
            Some(Err(invalid("device unavailable"))),
        );
        assert_eq!(value["error"], "original failure");
        assert_eq!(value["failed_model_readback"]["status"], "UNAVAILABLE");
        assert_eq!(value["resume_authorized"], false);
        assert!(value.get("failed_evidence_preserved").is_none());
        let captured = failure_payload("error", &cursor, Some(Ok(pin("actual-failed-model"))));
        assert_eq!(captured["failed_model_readback"]["status"], "CAPTURED");
        assert_eq!(captured["coherent_driver_checkpoint_claimed"], false);
    }
    pub(super) fn canonical_fixture() -> Value {
        let mut controls: Vec<Value> = RELATION_CONTROL_NAMES
            .iter()
            .enumerate()
            .map(|(i, name)| json!({"index":i+1,"name":name,"status":"PASS","details":null}))
            .collect();
        let mut gates = vec![0; 45];
        for i in [0, 5, 10, 15] {
            gates[i] = 1;
        }
        let locks:Vec<_>=[1,2].into_iter().map(|seed|json!({"seed":seed,"gates":gates,"initialization":"active_leaf_v1",
            "treatment_control_snapshot_bit_identical":true,"reload_bit_identical":true,"applied_updates":0})).collect();
        controls[8]["details"] = json!({"init_locks":locks});
        let report = json!({"schema":crate::SCHEMA,"suite":"relation","status":"PASS","registered":9,"attempted":9,"passed":9,
            "failed":0,"not_run":0,"training":"HELD","applied_updates":0,"controls":controls});
        report
    }
    #[test]
    fn canonical_relation_report_refuses_missing_controls_or_grep_aliases() {
        let report = canonical_fixture();
        validate_relation_qualification(&report).unwrap();
        let mut bad = report.clone();
        bad["not_run"] = json!(1);
        assert!(validate_relation_qualification(&bad).is_err());
        bad = report.clone();
        bad["controls"][8]["details"]["init_locks"][0]["gates"][1] = json!(1);
        assert!(validate_relation_qualification(&bad).is_err());
        bad = report;
        bad["controls"][8]["details"]["init_locks"] = json!("INIT LOCK PASS");
        assert!(validate_relation_qualification(&bad).is_err());
    }
    #[test]
    fn matches_qualified_execution_binds_stage1_pinned_identity_and_rejects_mismatched_config_or_kernel(
    ) {
        let plan_exec = fixture().execution;
        assert!(matches_qualified_execution(&plan_exec, &plan_exec));
        let mut stage1 = plan_exec.clone();
        stage1.reviewed_source_commit = QUALIFIED_STAGE1_COMMIT.into();
        stage1.source_sha256 = QUALIFIED_STAGE1_SOURCE_SHA256.into();
        stage1.kernel_sha256 = QUALIFIED_STAGE1_KERNEL_SHA256.into();
        stage1.driver_source_sha256 = QUALIFIED_STAGE1_DRIVER_SOURCE_SHA256.into();
        assert!(matches_qualified_execution(&plan_exec, &stage1));
        let mut wrong_kernel = stage1.clone();
        wrong_kernel.kernel_sha256 = "f".repeat(64);
        assert!(!matches_qualified_execution(&plan_exec, &wrong_kernel));
        let mut wrong_config = stage1;
        wrong_config.config_sha256 = "f".repeat(64);
        assert!(!matches_qualified_execution(&plan_exec, &wrong_config));
    }
}
