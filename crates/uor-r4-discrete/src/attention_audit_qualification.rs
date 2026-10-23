//! Separate, finite source-qualification admission. Never consumes a dose release.
use super::*;

pub const PLAN_SCHEMA: &str = "uor-r4.attention-audit-source-qualification-plan/1";
pub const ADMISSION_SCHEMA: &str = "uor-r4.attention-audit-source-qualification-admission/1";
#[cfg(feature = "cuda")]
const OWNER_SCHEMA: &str = "uor-r4.attention-audit-source-qualification-owner-decision/1";
#[cfg(feature = "cuda")]
const DECISION: &str = "RELEASE_FINITE_AUDITED_SOURCE_QUALIFICATION_ONLY";
const QUAL_SCOPE: &str = "m1-attention-active-audit-finite-source-qualification-v1";
const PROTOCOL: &str = "train61-synthetic1-interval64-two-arm-throughput-v1";
struct AddedProjection {
    gate: u64,
    step: u64,
    boundary: u64,
    restore: u64,
    total: u64,
}
fn project_added_components(
    gate: u64,
    step: u64,
    boundary: u64,
    physical_restore: u64,
    restore_ack: u64,
) -> Result<AddedProjection> {
    if [gate, step, boundary, physical_restore, restore_ack].contains(&0) {
        return Err(invalid("all five measured cost components required"));
    }
    let gate = gate
        .checked_mul(516)
        .ok_or_else(|| invalid("gate projection overflow"))?;
    let step = step
        .checked_mul(32808)
        .ok_or_else(|| invalid("step projection overflow"))?;
    let boundary = boundary
        .checked_mul(516)
        .ok_or_else(|| invalid("boundary projection overflow"))?;
    let restore = physical_restore
        .checked_add(restore_ack)
        .and_then(|n| n.checked_mul(514))
        .ok_or_else(|| invalid("restore projection overflow"))?;
    let total = gate
        .checked_add(step)
        .and_then(|n| n.checked_add(boundary))
        .and_then(|n| n.checked_add(restore))
        .ok_or_else(|| invalid("complete conservative cost projection overflow"))?;
    Ok(AddedProjection {
        gate,
        step,
        boundary,
        restore,
        total,
    })
}
#[cfg(any(feature = "cuda", test))]
fn exact_refusal<T>(result: Result<T>, expected: &str) -> Result<()> {
    match result {
        Err(Error::Invalid(message)) if message == expected => Ok(()),
        Err(error @ Error::Cuda(_)) => Err(error),
        Err(error) => Err(invalid(format!("unexpected refusal: {error}"))),
        Ok(_) => Err(invalid(
            "expected stale/foreign handle refusal did not occur",
        )),
    }
}
#[cfg(any(feature = "cuda", test))]
fn stable_anchor_equal(left: &Value, right: &Value) -> bool {
    let pointer = "/boundary/checkpoint_file/path";
    if left.pointer(pointer).and_then(Value::as_str).is_none()
        || right.pointer(pointer).and_then(Value::as_str).is_none()
    {
        return false;
    }
    let mut left = left.clone();
    let mut right = right.clone();
    *left.pointer_mut(pointer).unwrap() = json!("AUTHENTICATED_PATH_NORMALIZED");
    *right.pointer_mut(pointer).unwrap() = json!("AUTHENTICATED_PATH_NORMALIZED");
    left == right
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResourceObservation {
    pub schema: String,
    pub executions: Vec<ExecutionIdentity>,
    pub operator: String,
    pub hardware_class: String,
    pub gpu_count: u32,
    pub rate_usd_micros_per_hour: u64,
    pub host_available_bytes: u64,
    pub staging_archive_reserve_bytes: u64,
    pub observed_unix_seconds: u64,
    pub native_resource_receipt: FilePin,
    pub native_lease_receipt: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostedDoseDecision {
    pub schema: String,
    pub executions: [ExecutionIdentity; 2],
    pub calibration: FilePin,
    pub resource_admission: FilePin,
    pub rate_usd_micros_per_hour: u64,
    pub gpu_count: u32,
    pub qualified_ungated_two_arm_ns: u64,
    pub qualified_added_gate_ns: u64,
    pub projected_workload_resource_seconds: u64,
    pub planned_common_setup_seconds: u64,
    pub planned_common_custody_archive_close_seconds: u64,
    pub common_metadata_coverage: String,
    pub all_in_projected_resource_seconds: u64,
    pub all_in_projected_resource_usd_micros: u64,
    pub workload_projection_basis: String,
    pub transfer_storage_cost_status: String,
    pub measured_full_dose: bool,
    pub invoice: bool,
    pub budget_is_hard_stop: bool,
    pub owner_cost_approval: FilePin,
    pub pm_dispatch: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DoseCostAttestation {
    schema: String,
    actor: String,
    role: String,
    verdict: String,
    executions: [ExecutionIdentity; 2],
    cost_payload_sha256: String,
    receipt_url: String,
}
pub fn cost_payload_sha256(record: &CostedDoseDecision) -> Result<String> {
    let mut value = serde_json::to_value(record).map_err(|e| invalid(e.to_string()))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| invalid("costed decision object"))?;
    object.remove("owner_cost_approval");
    object.remove("pm_dispatch");
    Ok(corpus::sha256(&encode(&value)?))
}
fn resource_proxy_micros(rate: u64, seconds: u64) -> Result<u64> {
    if rate == 0 || seconds == 0 {
        return Err(invalid(
            "positive actual rate and planned resource seconds required",
        ));
    }
    u64::try_from((u128::from(rate) * u128::from(seconds)).div_ceil(3600))
        .map_err(|_| invalid("projected resource cost overflow"))
}
pub(super) fn validate_costed_dose(plan: &AuditedPlan, calibration: &Calibration) -> Result<()> {
    let decision: CostedDoseDecision = pinned_json(&plan.costed_dose_decision)?;
    let resource = validate_resource(&plan.resource_admission, &plan.execution, true)?;
    let timing: TimingProtocol = pinned_json(&calibration.measurement_protocol)?;
    let workload_ns = calibration
        .ungated_two_arm_dose_ns
        .checked_add(calibration.projected_added_gate_ns)
        .ok_or_else(|| invalid("qualified workload forecast overflow"))?;
    let workload_seconds = workload_ns.div_ceil(1_000_000_000);
    let all_in_minimum = workload_seconds
        .checked_add(decision.planned_common_setup_seconds)
        .and_then(|n| n.checked_add(decision.planned_common_custody_archive_close_seconds))
        .ok_or_else(|| invalid("all-in forecast overflow"))?;
    if decision.schema!="uor-r4.attention-audit-costed-dose-decision/1"
        || decision.executions!=[timing.execution,timing.control_execution]
        || !decision.executions.contains(&plan.execution) || decision.calibration!=plan.calibration || decision.resource_admission!=plan.resource_admission
        || decision.rate_usd_micros_per_hour!=resource.rate_usd_micros_per_hour || decision.gpu_count!=resource.gpu_count
        || decision.qualified_ungated_two_arm_ns!=calibration.ungated_two_arm_dose_ns || decision.qualified_added_gate_ns!=calibration.projected_added_gate_ns
        || decision.projected_workload_resource_seconds!=workload_seconds || decision.planned_common_setup_seconds==0
        || decision.planned_common_custody_archive_close_seconds==0 || decision.all_in_projected_resource_seconds<all_in_minimum
        || decision.common_metadata_coverage!="BASELINE_Q1_TERMINAL_METADATA_INCLUDED_IN_PLANNED_COMMON_SETUP_CUSTODY_CLOSE_NOT_MEASURED_RECURRING_GATE"
        || decision.all_in_projected_resource_usd_micros!=resource_proxy_micros(resource.rate_usd_micros_per_hour,decision.all_in_projected_resource_seconds)?
        || decision.workload_projection_basis!="CONSERVATIVE_SEQUENTIAL_SUM_RESOURCE_TIME_UPPER_BOUND_NOT_MEASURED_PARALLEL_WALLTIME"
        || decision.transfer_storage_cost_status!="UNMEASURED_SEPARATE_FROM_RESOURCE_PROXY"
        || decision.measured_full_dose || decision.invoice || decision.budget_is_hard_stop {
        return Err(invalid("exact costed dose source/resource/rate/workload/setup/custody forecast incomplete"));
    }
    let payload = cost_payload_sha256(&decision)?;
    for (pin, actor, role, verdict) in [
        (
            &decision.owner_cost_approval,
            "Casey",
            "owner_cost_scope",
            "APPROVE_EXACT_COSTED_DOSE_PLAN",
        ),
        (
            &decision.pm_dispatch,
            "Kimi",
            "pm_dispatch",
            "DISPATCH_EXACT_COSTED_DOSE_PLAN",
        ),
    ] {
        let actual: DoseCostAttestation = pinned_json(pin)?;
        if actual.schema != "uor-r4.attention-audit-dose-cost-attestation/1"
            || actual.actor != actor
            || actual.role != role
            || actual.verdict != verdict
            || actual.executions != decision.executions
            || actual.cost_payload_sha256 != payload
            || !valid_receipt_url(&actual.receipt_url)
        {
            return Err(invalid("actual owner/PM costed plan record binding required; this is not personal D40 release"));
        }
    }
    Ok(())
}
fn private_pin(pin: &FilePin) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file = fs::symlink_metadata(&pin.path).map_err(|e| invalid(e.to_string()))?;
        let parent = fs::symlink_metadata(
            pin.path
                .parent()
                .ok_or_else(|| invalid("private input parent"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
        if !file.is_file()
            || !parent.is_dir()
            || file.permissions().mode() & 0o777 != 0o600
            || parent.permissions().mode() & 0o777 != 0o700
        {
            return Err(invalid(
                "private operational input must be0600 under0700 directory",
            ));
        }
        pin.read()?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        let _ = pin;
        Err(invalid(
            "private operational input requires Unix permissions",
        ))
    }
}
pub(super) fn validate_resource(
    pin: &FilePin,
    execution: &ExecutionIdentity,
    fresh: bool,
) -> Result<ResourceObservation> {
    private_pin(pin)?;
    let resource: ResourceObservation = pinned_json(pin)?;
    if resource.schema != "uor-r4.attention-audit-resource-observation/1"
        || !resource.executions.contains(execution)
        || resource.executions.is_empty()
        || resource.executions.len() > 2
        || resource.executions.iter().any(|other| {
            other.source_sha256 != execution.source_sha256
                || other.kernel_sha256 != execution.kernel_sha256
                || other.geometry_sha256 != execution.geometry_sha256
                || other.executable_sha256 != execution.executable_sha256
                || other.driver_source_sha256 != execution.driver_source_sha256
        })
        || !["Claude", "Codex"].contains(&resource.operator.as_str())
        || resource.hardware_class.is_empty()
        || !(1..=4).contains(&resource.gpu_count)
        || !(1..=8_000_000).contains(&resource.rate_usd_micros_per_hour)
        || resource.observed_unix_seconds == 0
    {
        return Err(invalid("actual source/resource/rate observation differs"));
    }
    validate_reserve(
        resource.host_available_bytes,
        resource.staging_archive_reserve_bytes,
    )?;
    private_pin(&resource.native_resource_receipt)?;
    private_pin(&resource.native_lease_receipt)?;
    if fresh {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| invalid(e.to_string()))?
            .as_secs();
        if now
            .checked_sub(resource.observed_unix_seconds)
            .is_none_or(|age| age > 300)
        {
            return Err(invalid(
                "fresh actual resource/lease/rate observation required",
            ));
        }
    }
    Ok(resource)
}

fn projected_ns(
    samples_ns: &[u64; 3],
    sample_counts: &[u64; 3],
    full_counts: &[u64; 3],
) -> Result<u64> {
    let total = (0..3).try_fold(0u128, |sum, i| {
        if samples_ns[i] == 0 || sample_counts[i] == 0 || full_counts[i] == 0 {
            return Err(invalid("missing measured production family"));
        }
        let numerator = u128::from(samples_ns[i]) * u128::from(full_counts[i]);
        let denominator = u128::from(sample_counts[i]);
        sum.checked_add(numerator.div_ceil(denominator))
            .ok_or_else(|| invalid("timing sum overflow"))
    })?;
    u64::try_from(total).map_err(|_| invalid("projected timing overflow"))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimingProtocol {
    pub schema: String,
    pub execution: ExecutionIdentity,
    pub control_execution: ExecutionIdentity,
    pub configs: [DriverConfig; 2],
    pub protocol: Limits,
    pub gate_samples_ns: Vec<u64>,
    pub restore_samples_ns: Vec<u64>,
    pub arm_family_elapsed_ns: [[u64; 3]; 2],
    pub arm_family_counts: [[u64; 3]; 2],
    pub full_family_counts: [[u64; 3]; 2],
    pub projected_gate_ns: u64,
    pub projected_step_bookkeeping_ns: u64,
    pub projected_boundary_bookkeeping_ns: u64,
    pub projected_restore_ns: u64,
    pub bookkeeping: FilePin,
    pub projected_ungated_two_arm_ns: u64,
    pub raw_artifacts: Vec<FilePin>,
    pub shared_setup_ns: u64,
    pub timer_scope: String,
    pub complete_gate_io_included: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransitionTiming {
    schema: String,
    execution: ExecutionIdentity,
    family: usize,
    elapsed_ns: u64,
    receipt: training::StepReceipt,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationTiming {
    schema: String,
    execution: ExecutionIdentity,
    operation: String,
    elapsed_ns: u64,
    artifact: FilePin,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BookkeepingMeasurement {
    schema: String,
    execution: ExecutionIdentity,
    scope: String,
    driver_source_sha256: String,
    step_envelope_bytes: u64,
    boundary_envelope_bytes: u64,
    maximum_step_shape_bytes: u64,
    maximum_boundary_shape_bytes: u64,
    progress_history_entries: u64,
    maximum_progress_shape_bytes: u64,
    step_samples_ns: Vec<u64>,
    pass_boundary_ns: u64,
    breach_restore_ack_ns: u64,
    conservative_overlap: String,
    artifacts: Vec<FilePin>,
}
const STEP_ENVELOPE_BYTES: usize = 16 * 1024;
const BOUNDARY_ENVELOPE_BYTES: usize = 64 * 1024;
// Full attempted dose, conservatively two audit/restore artifacts at every
// boundary including baseline/tail, plus baseline and parent-Q1 records.
const MAX_PROGRESS_HISTORY: usize = 16404 + 2 * 258 + 2;
const PROGRESS_ENVELOPE_BYTES: usize = 2 * 1024 * 1024;
fn maximum_numeric_width(value: &mut Value) {
    match value {
        Value::Number(_) => *value = json!(u64::MAX),
        Value::Array(values) => {
            for v in values {
                maximum_numeric_width(v);
            }
        }
        Value::Object(values) => {
            for v in values.values_mut() {
                maximum_numeric_width(v);
            }
        }
        _ => (),
    }
}
fn validate_bookkeeping(
    record: &BookkeepingMeasurement,
    execution: &ExecutionIdentity,
) -> Result<()> {
    if record.schema != "uor-r4.attention-audit-host-bookkeeping-measurement/1"
        || record.execution != *execution
        || record.driver_source_sha256 != training::driver_source_sha256()
        || record.scope != "METADATA_FIXTURE_ONLY_NO_MODEL_OR_NATURAL_BREACH_CLAIM"
        || record.conservative_overlap
            != "CHARGE_ALL_PARENT_STEP_IO_AGAIN_AND_PASS_BOUNDARY_PLUS_RESTORE_ACK"
        || record.step_envelope_bytes != STEP_ENVELOPE_BYTES as u64
        || record.boundary_envelope_bytes != BOUNDARY_ENVELOPE_BYTES as u64
        || record.step_samples_ns.len() != 128
        || record.step_samples_ns.contains(&0)
        || record.pass_boundary_ns == 0
        || record.breach_restore_ack_ns == 0
        || record.progress_history_entries != MAX_PROGRESS_HISTORY as u64
    {
        return Err(invalid(
            "host bookkeeping scope/timing/source coverage differs",
        ));
    }
    let mut paths = std::collections::BTreeSet::new();
    let mut steps = 0;
    let mut boundaries = 0;
    let mut padded_steps = 0;
    let mut padded_boundaries = 0;
    let mut max_step = 0;
    let mut max_boundary = 0;
    let mut progress_records = 0;
    let mut linkage_records = 0;
    let mut max_progress = 0;
    let mut rejection_flags = std::collections::BTreeSet::new();
    for pin in &record.artifacts {
        if !paths.insert(pin.path.clone()) {
            return Err(invalid("duplicate host metadata artifact"));
        }
        let raw = pin.read()?;
        if pin.path.extension().and_then(|v| v.to_str()) == Some("jsonl") {
            for line in raw.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
                let receipt: training::StepReceipt =
                    serde_json::from_slice(line).map_err(|e| invalid(e.to_string()))?;
                if receipt.target_positions.len() != 64 {
                    return Err(invalid(
                        "metadata step fixture does not size full64row payload",
                    ));
                }
                let mut shape =
                    serde_json::to_value(receipt).map_err(|e| invalid(e.to_string()))?;
                maximum_numeric_width(&mut shape);
                max_step = max_step.max(encode(&shape)?.len() + 1);
                steps += 1;
            }
        } else if pin.path.file_name().and_then(|v| v.to_str()) == Some("progress.json") {
            let value: Value = serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
            if value["schema"] != "uor-r4.attention-audit-checkpoint/1"
                || value["resume"] != "NOT_AUTHORIZED"
                || value["lifecycle"]["used_artifacts"]
                    .as_array()
                    .map(Vec::len)
                    != Some(MAX_PROGRESS_HISTORY)
            {
                return Err(invalid(
                    "exact shared progress maximum-history fixture absent",
                ));
            }
            let mut shape = value;
            maximum_numeric_width(&mut shape);
            max_progress = max_progress.max(encode(&shape)?.len());
            progress_records += 1;
        } else if pin.path.file_name().and_then(|v| v.to_str()) == Some("interval-artifacts.json") {
            let value: Value = serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
            for field in [
                "steps",
                "audit",
                "attempted_checkpoint",
                "continuation_checkpoint",
            ] {
                let pin: FilePin = serde_json::from_value(value[field].clone())
                    .map_err(|e| invalid(e.to_string()))?;
                pin.read()?;
            }
            corpus::validate_sha256(
                value["record_sha256"]
                    .as_str()
                    .ok_or_else(|| invalid("linkage record hash"))?,
            )?;
            linkage_records += 1;
        } else if pin.path.file_name().and_then(|v| v.to_str()) == Some("record.json") {
            let value: crate::attention_audit::DurableRecord =
                serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
            let crate::attention_audit::RecordKind::Interval(ref interval) = value.kind else {
                return Err(invalid("metadata boundary fixture is not full interval"));
            };
            if interval.steps.len() != 64 {
                return Err(invalid("metadata boundary fixture does not size64steps"));
            }
            rejection_flags.insert(interval.rejected);
            let mut shape = serde_json::to_value(value).map_err(|e| invalid(e.to_string()))?;
            maximum_numeric_width(&mut shape);
            max_boundary = max_boundary.max(encode(&shape)?.len());
            boundaries += 1;
        } else if pin.path.file_name().and_then(|v| v.to_str()) == Some("boundary-envelope.json") {
            if raw.len() < BOUNDARY_ENVELOPE_BYTES {
                return Err(invalid("boundary I/O byte envelope truncated"));
            }
            padded_boundaries += 1;
        } else if pin
            .path
            .file_stem()
            .and_then(|v| v.to_str())
            .is_some_and(|v| v.starts_with("step-"))
        {
            if raw.len() < STEP_ENVELOPE_BYTES {
                return Err(invalid("step I/O byte envelope truncated"));
            }
            padded_steps += 1;
        }
    }
    if steps != 128
        || boundaries != 2
        || padded_steps != 128
        || padded_boundaries != 2
        || rejection_flags.len() != 2
        || progress_records != 2
        || linkage_records != 2
        || max_progress == 0
        || max_progress > PROGRESS_ENVELOPE_BYTES
        || max_progress as u64 != record.maximum_progress_shape_bytes
        || max_step == 0
        || max_step > STEP_ENVELOPE_BYTES
        || max_step as u64 != record.maximum_step_shape_bytes
        || max_boundary == 0
        || max_boundary > BOUNDARY_ENVELOPE_BYTES
        || max_boundary as u64 != record.maximum_boundary_shape_bytes
    {
        return Err(invalid(
            "raw metadata record shape/byte inventory does not support conservative cost envelope",
        ));
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn elapsed_ns(started: std::time::Instant) -> Result<u64> {
    u64::try_from(started.elapsed().as_nanos()).map_err(|_| invalid("elapsed nanosecond overflow"))
}
/// Measured host metadata work only: no Device, model arithmetic or numerical
/// admission. Actual source lifecycle/persist/ack, oversized I/O envelopes.
#[cfg(any(feature = "cuda", test))]
fn measure_bookkeeping(execution: &ExecutionIdentity, out: &Path) -> Result<FilePin> {
    use crate::attention_audit::{
        AuditAction, AuditEvidence, Family, RestoreEvidence, RunBinding, SnapshotPin,
    };
    private_directory(out)?;
    let mut samples = Vec::new();
    let mut artifacts = Vec::new();
    let mut boundaries = [0u64; 2];
    let mut maximum_step_shape = 0usize;
    let mut maximum_boundary_shape = 0usize;
    let mut maximum_progress_shape = 0usize;
    for (case, admitted) in [("pass", 576u64), ("synthetic-breach", 575u64)] {
        let directory = out.join(case);
        private_directory(&directory)?;
        let h = |label: &str| corpus::sha256(format!("METADATA_FIXTURE/{case}/{label}").as_bytes());
        let pins = |label: &str| SnapshotPin {
            checkpoint_sha256: h(&format!("{label}/file")),
            storage_sha256: h(&format!("{label}/storage")),
            snapshot_sha256: h(&format!("{label}/typed")),
        };
        let mut machine = AuditMachine::new(
            RunBinding {
                plan_sha256: h("plan"),
                release_sha256: h("release"),
                lineage_sha256: h("lineage"),
                sheet_sha256: h("sheet"),
                anchor_sha256: h("anchor"),
                combine_gates: 20,
                relation_gates: 25,
            },
            false,
            1,
        )?;
        let baseline = machine.baseline(AuditEvidence {
            stamp: machine.stamp()?,
            snapshot: pins("baseline"),
            admitted: 600,
            artifact_sha256: h("baseline-audit"),
        })?;
        let baseline_pin = settle(&mut machine, &directory.join("baseline.json"), &baseline)?;
        artifacts.push(baseline_pin.clone());
        let journal_path = directory.join("steps.jsonl");
        let mut journal = private_file(&journal_path)?;
        for i in 0..64 {
            let start = std::time::Instant::now();
            let before = training::Cursor {
                completed_batches: i,
                accepted_updates: i,
                corpus_pass: u64::MAX,
                next_target: u64::MAX,
                next_coordinate: u32::MAX,
                block: None,
            };
            let after = training::Cursor {
                completed_batches: i + 1,
                accepted_updates: i + 1,
                ..before
            };
            let kind = if machine.expected_family() == Family::Vector {
                training::CoordinateKind::VectorBlock
            } else {
                training::CoordinateKind::Gate
            };
            let receipt = training::StepReceipt {
                lineage_sha256: h("lineage"),
                cursor_before: before,
                cursor_after: after,
                coordinate: training::Coordinate {
                    kind,
                    index: u32::MAX,
                },
                target_positions: vec![u64::MAX; 64],
                candidate: u64::MAX,
                loss_before: u64::MAX,
                loss_after: u64::MAX,
                model_epoch: i + 1,
                applied: true,
                apply_refused_by_policy: false,
                any_candidate_cost_differs_from_identity: true,
            };
            let mut shape = serde_json::to_value(&receipt).map_err(|e| invalid(e.to_string()))?;
            maximum_numeric_width(&mut shape);
            maximum_step_shape = maximum_step_shape.max(encode(&shape)?.len() + 1);
            if maximum_step_shape > STEP_ENVELOPE_BYTES {
                return Err(invalid(
                    "production StepReceipt shape exceeds conservative step I/O envelope",
                ));
            }
            append_step_fsync_and_record(&mut journal, &mut machine, &receipt)?;
            let payload = json!({"scope":"METADATA_FIXTURE_NOT_NUMERICAL","ordinal":i,"padding":"x".repeat(STEP_ENVELOPE_BYTES)});
            let pin = persist(&directory.join(format!("step-{i:02}.json")), &payload)?;
            samples.push(elapsed_ns(start)?);
            artifacts.push(pin);
        }
        drop(journal);
        let bytes = fs::read(&journal_path).map_err(|e| invalid(e.to_string()))?;
        artifacts.push(FilePin {
            path: fs::canonicalize(&journal_path).map_err(|e| invalid(e.to_string()))?,
            sha256: corpus::sha256(&bytes),
        });
        let start = std::time::Instant::now();
        let snapshot = pins("boundary");
        let stamp = machine.seal_boundary(snapshot.clone())?;
        let audit = AuditEvidence {
            stamp,
            snapshot,
            admitted,
            artifact_sha256: h("boundary-audit"),
        };
        let padded = persist(
            &directory.join("boundary-envelope.json"),
            &json!({"scope":"METADATA_FIXTURE_NOT_NUMERICAL",
            "padding":"x".repeat(BOUNDARY_ENVELOPE_BYTES),"audit":audit}),
        )?;
        artifacts.push(padded);
        let record = match machine.audit(audit)? {
            AuditAction::Commit(record) => record,
            AuditAction::Restore(required) => {
                artifacts.push(persist(&directory.join("restore-intent.json"),&json!({"scope":"METADATA_FIXTURE_NOT_NUMERICAL",
                    "status":"SYNTHETIC_METADATA_BREACH_RESTORE_INTENT","required":required,"physical_restore_claimed":false}))?);
                let observed = persist(
                    &directory.join("restore-observation.json"),
                    &json!({"scope":"METADATA_FIXTURE_NOT_NUMERICAL",
                    "required":required,"physical_restore_claimed":false}),
                )?;
                let record = machine.restored(RestoreEvidence {
                    restored_snapshot: required.passing.snapshot.clone(),
                    restored_epoch: required.passing.epoch,
                    advanced_completed: required.failed.completed,
                    new_runtime_model: 2,
                    artifact_sha256: observed.sha256.clone(),
                    required,
                })?;
                artifacts.push(observed);
                record
            }
        };
        // Exact production DurableRecord serialization, persistence and ack.
        let mut shape = serde_json::to_value(&record).map_err(|e| invalid(e.to_string()))?;
        maximum_numeric_width(&mut shape);
        maximum_boundary_shape = maximum_boundary_shape.max(encode(&shape)?.len());
        if maximum_boundary_shape > BOUNDARY_ENVELOPE_BYTES {
            return Err(invalid(
                "metadata boundary exceeds conservative byte envelope",
            ));
        }
        artifacts.push(settle(
            &mut machine,
            &directory.join("record.json"),
            &record,
        )?);
        artifacts.push(persist_interval_linkage(
            &directory.join("interval-artifacts.json"),
            &baseline_pin,
            &baseline_pin,
            &baseline_pin,
            &baseline_pin,
            &record,
        )?);
        let mut maximum_history =
            serde_json::to_value(&machine).map_err(|e| invalid(e.to_string()))?;
        maximum_history["used_artifacts"] = json!((0..MAX_PROGRESS_HISTORY)
            .map(|i| format!("{i:064x}"))
            .collect::<Vec<_>>());
        let progress = persist_progress(
            &directory.join("progress.json"),
            &h("admission"),
            &baseline_pin,
            &baseline_pin,
            &maximum_history,
            &json!({"scope":"METADATA_Q1_SIZE_ENVELOPE","padding":"x".repeat(BOUNDARY_ENVELOPE_BYTES)}),
        )?;
        let mut shape: Value = pinned_json(&progress)?;
        maximum_numeric_width(&mut shape);
        maximum_progress_shape = maximum_progress_shape.max(encode(&shape)?.len());
        if maximum_progress_shape > PROGRESS_ENVELOPE_BYTES {
            return Err(invalid(
                "maximum-history progress exceeds declared envelope",
            ));
        }
        artifacts.push(progress);
        boundaries[usize::from(case != "pass")] = elapsed_ns(start)?;
    }
    persist(
        &out.join("measurement.json"),
        &BookkeepingMeasurement {
            schema: "uor-r4.attention-audit-host-bookkeeping-measurement/1".into(),
            execution: execution.clone(),
            scope: "METADATA_FIXTURE_ONLY_NO_MODEL_OR_NATURAL_BREACH_CLAIM".into(),
            driver_source_sha256: training::driver_source_sha256(),
            step_envelope_bytes: STEP_ENVELOPE_BYTES as u64,
            boundary_envelope_bytes: BOUNDARY_ENVELOPE_BYTES as u64,
            step_samples_ns: samples,
            maximum_step_shape_bytes: maximum_step_shape as u64,
            maximum_boundary_shape_bytes: maximum_boundary_shape as u64,
            progress_history_entries: MAX_PROGRESS_HISTORY as u64,
            maximum_progress_shape_bytes: maximum_progress_shape as u64,
            pass_boundary_ns: boundaries[0],
            breach_restore_ack_ns: boundaries[1],
            conservative_overlap:
                "CHARGE_ALL_PARENT_STEP_IO_AGAIN_AND_PASS_BOUNDARY_PLUS_RESTORE_ACK".into(),
            artifacts,
        },
    )
}
pub(super) fn validate_timing_protocol(calibration: &Calibration) -> Result<()> {
    let timing: TimingProtocol = pinned_json(&calibration.measurement_protocol)?;
    let bookkeeping: BookkeepingMeasurement = pinned_json(&timing.bookkeeping)?;
    validate_bookkeeping(&bookkeeping, &timing.execution)?;
    if bookkeeping.schema != "uor-r4.attention-audit-host-bookkeeping-measurement/1"
        || bookkeeping.execution != timing.execution
        || bookkeeping.driver_source_sha256 != training::driver_source_sha256()
        || bookkeeping.scope != "METADATA_FIXTURE_ONLY_NO_MODEL_OR_NATURAL_BREACH_CLAIM"
        || bookkeeping.conservative_overlap
            != "CHARGE_ALL_PARENT_STEP_IO_AGAIN_AND_PASS_BOUNDARY_PLUS_RESTORE_ACK"
        || bookkeeping.step_envelope_bytes != STEP_ENVELOPE_BYTES as u64
        || bookkeeping.boundary_envelope_bytes != BOUNDARY_ENVELOPE_BYTES as u64
        || bookkeeping.maximum_step_shape_bytes == 0
        || bookkeeping.maximum_step_shape_bytes > bookkeeping.step_envelope_bytes
        || bookkeeping.maximum_boundary_shape_bytes == 0
        || bookkeeping.maximum_boundary_shape_bytes > bookkeeping.boundary_envelope_bytes
        || bookkeeping.step_samples_ns.len() != 128
        || bookkeeping.step_samples_ns.contains(&0)
        || bookkeeping.pass_boundary_ns == 0
        || bookkeeping.breach_restore_ack_ns == 0
        || bookkeeping.artifacts.is_empty()
    {
        return Err(invalid(
            "actual host lifecycle/bookkeeping coverage incomplete",
        ));
    }
    for pin in &bookkeeping.artifacts {
        pin.read()?;
    }
    timing.protocol.validate()?;
    if timing.schema != "uor-r4.attention-audit-measurement-protocol/1"
        || ![&timing.execution, &timing.control_execution].contains(&&calibration.execution)
        || timing.gate_samples_ns.len() != timing.protocol.gate_samples as usize
        || timing.restore_samples_ns.len() != 2
        || timing.gate_samples_ns.contains(&0)
        || timing.restore_samples_ns.contains(&0)
        || timing.timer_scope != timing.protocol.timer_scope
        || !timing.complete_gate_io_included
        || timing.raw_artifacts.is_empty()
        || timing.raw_artifacts != calibration.measurement_artifacts
    {
        return Err(invalid(
            "actual measured/extrapolated timing protocol incomplete",
        ));
    }
    for (i, execution) in [&timing.execution, &timing.control_execution]
        .iter()
        .enumerate()
    {
        validate_identity(&timing.configs[i], execution)?;
        if timing.arm_family_counts[i] != schedule_counts(&timing.configs[i], 384)?
            || timing.full_family_counts[i] != schedule_counts(&timing.configs[i], 16404)?
        {
            return Err(invalid(
                "timing family inventory differs from actual registered schedule",
            ));
        }
    }
    let gate = *timing
        .gate_samples_ns
        .iter()
        .max()
        .ok_or_else(|| invalid("gate samples missing"))?;
    let restore = *timing
        .restore_samples_ns
        .iter()
        .max()
        .ok_or_else(|| invalid("restore samples missing"))?;
    let projected = project_added_components(
        gate,
        *bookkeeping
            .step_samples_ns
            .iter()
            .max()
            .ok_or_else(|| invalid("step metadata samples missing"))?,
        bookkeeping.pass_boundary_ns,
        restore,
        bookkeeping.breach_restore_ack_ns,
    )?;
    let denominator = projected_ns(
        &timing.arm_family_elapsed_ns[0],
        &timing.arm_family_counts[0],
        &timing.full_family_counts[0],
    )?
    .checked_add(projected_ns(
        &timing.arm_family_elapsed_ns[1],
        &timing.arm_family_counts[1],
        &timing.full_family_counts[1],
    )?)
    .ok_or_else(|| invalid("dose projection overflow"))?;
    if timing.projected_gate_ns != projected.gate
        || timing.projected_step_bookkeeping_ns != projected.step
        || timing.projected_boundary_bookkeeping_ns != projected.boundary
        || timing.projected_restore_ns != projected.restore
        || timing.projected_ungated_two_arm_ns != denominator
        || calibration.projected_added_gate_ns != projected.total
        || calibration.ungated_two_arm_dose_ns != denominator
        || calibration.denominator_provenance != "same_hardware_qualified_throughput_extrapolation"
    {
        return Err(invalid(
            "timing projection formula differs from actual samples",
        ));
    }
    let mut raw_counts = [[0u64; 3]; 2];
    let mut raw_elapsed = [[0u64; 3]; 2];
    let drivers = [
        PreparedDriver::load(timing.configs[0].clone())?,
        PreparedDriver::load(timing.configs[1].clone())?,
    ];
    let mut seen: [std::collections::BTreeSet<u64>; 2] = [
        std::collections::BTreeSet::new(),
        std::collections::BTreeSet::new(),
    ];
    let mut gate_samples: [Vec<u64>; 2] = [Vec::new(), Vec::new()];
    let mut restores = [0u64; 2];
    for raw in &timing.raw_artifacts {
        let value: Value = pinned_json(raw)?;
        if value["schema"] == "uor-r4.attention-audit-transition-timing/1" {
            let sample: TransitionTiming =
                serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
            let i = if sample.execution == timing.execution {
                0
            } else if sample.execution == timing.control_execution {
                1
            } else {
                return Err(invalid("raw timing source/arm differs"));
            };
            if sample.family > 2 || sample.elapsed_ns == 0 {
                return Err(invalid("raw family timing missing"));
            }
            if encode(&sample.receipt)?.len() + 1 > STEP_ENVELOPE_BYTES {
                return Err(invalid(
                    "actual production receipt exceeds measured conservative I/O byte coverage",
                ));
            }
            let index = sample.receipt.cursor_before.completed_batches;
            let coordinate = drivers[i].coordinate_at(index)?;
            let stages =
                crate::metadata::BinderPlan::new(drivers[i].config().spec.context as usize)?
                    .stages() as u32;
            let family = match coordinate.kind {
                training::CoordinateKind::VectorBlock => 0,
                training::CoordinateKind::Gate => {
                    if coordinate.index < drivers[i].config().spec.blocks * stages {
                        1
                    } else {
                        2
                    }
                }
                _ => return Err(invalid("unexpected timed coordinate family")),
            };
            if index >= 384
                || !seen[i].insert(index)
                || sample.receipt.coordinate != coordinate
                || sample.family != family
                || sample.receipt.lineage_sha256 != drivers[i].lineage_sha256()?
                || sample.receipt.cursor_after
                    != drivers[i].advance(&sample.receipt.cursor_before, sample.receipt.applied)?
            {
                return Err(invalid(
                    "raw timing receipt schedule/uniqueness/identity differs",
                ));
            }
            raw_counts[i][sample.family] += 1;
            raw_elapsed[i][sample.family] = raw_elapsed[i][sample.family]
                .checked_add(sample.elapsed_ns)
                .ok_or_else(|| invalid("raw timer overflow"))?;
        } else if value["schema"] == "uor-r4.attention-audit-operation-timing/1" {
            let sample: OperationTiming =
                serde_json::from_value(value).map_err(|e| invalid(e.to_string()))?;
            let i = if sample.execution == timing.execution {
                0
            } else if sample.execution == timing.control_execution {
                1
            } else {
                return Err(invalid("raw operation timing source/arm differs"));
            };
            if sample.elapsed_ns == 0 {
                return Err(invalid("raw operation duration missing"));
            }
            sample.artifact.read()?;
            match sample.operation.as_str() {
                "complete_anchor_sample" => gate_samples[i].push(sample.elapsed_ns),
                "complete_restore_sample" => {
                    if restores[i] != 0 {
                        return Err(invalid("duplicate restore timing"));
                    }
                    restores[i] = sample.elapsed_ns;
                }
                _ => return Err(invalid("unknown timing operation")),
            }
        }
    }
    let expected_gate: Vec<_> = gate_samples[0]
        .iter()
        .zip(&gate_samples[1])
        .map(|(a, b)| (*a).max(*b))
        .collect();
    if raw_counts != timing.arm_family_counts
        || raw_elapsed != timing.arm_family_elapsed_ns
        || restores.as_slice() != timing.restore_samples_ns
        || gate_samples
            .iter()
            .any(|samples| samples.len() != timing.protocol.gate_samples as usize)
        || expected_gate != timing.gate_samples_ns
        || seen.iter().any(|indices| indices.len() != 384)
    {
        return Err(invalid(
            "timing summary differs from actual raw measured samples",
        ));
    }
    Ok(())
}
fn schedule_counts(config: &DriverConfig, attempts: u64) -> Result<[u64; 3]> {
    let stages = crate::metadata::BinderPlan::new(config.spec.context as usize)?.stages() as u32;
    let combine = config.spec.blocks * stages;
    let total = config.read_spec.gate_count(config.spec.blocks, stages)?;
    let mut counts = [0u64; 3];
    if total == 0 || attempts > 16404 {
        return Err(invalid("timing schedule bound/inventory"));
    }
    for i in 0..attempts {
        let family = if i % 8 != 7 {
            0
        } else if (i / 8) % u64::from(total) < u64::from(combine) {
            1
        } else {
            2
        };
        counts[family] += 1;
    }
    Ok(counts)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub protocol: String,
    pub anchor_batches_per_sample: u64,
    pub gate_samples: u64,
    pub synthetic_apply_calls: u64,
    pub synthetic_accepted_updates: u64,
    pub synthetic_stale_apply_refusals: u64,
    pub production_stale_apply_refusals_per_arm: u64,
    pub anchor_evaluations_per_arm: u64,
    pub rejected_interval_attempts: u64,
    pub throughput_attempts_per_arm: u64,
    pub throughput_warmup_attempts_per_arm: u64,
    pub projected_audits: u64,
    pub projected_restore_events: u64,
    pub timer_scope: String,
    pub cache_policy: String,
}
impl Limits {
    fn validate(&self) -> Result<()> {
        if self.protocol != PROTOCOL || self.anchor_batches_per_sample != 61
            || !(1..=3).contains(&self.gate_samples)
            || self.synthetic_apply_calls != 1 || self.synthetic_accepted_updates != 1
            || self.synthetic_stale_apply_refusals != 1 || self.production_stale_apply_refusals_per_arm != 1
            || self.anchor_evaluations_per_arm != self.gate_samples + 1
            || self.rejected_interval_attempts != 64
            || self.throughput_attempts_per_arm != 384
            || self.throughput_warmup_attempts_per_arm != 0
            || self.projected_audits != 516
            // Conservative full-rejection allowance; no guessed zero restores.
            || self.projected_restore_events != 514
            || self.timer_scope != "synchronized-device-snapshot-seal-fsync-readback-lifecycle-v1"
            || self.cache_policy != "fresh-device-per-arm-no-unreported-warmup-v1"
        {
            return Err(invalid("finite qualification limits/protocol differ"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authorization {
    pub schema: String,
    pub role: String,
    pub actor: String,
    pub verdict: String,
    pub execution: ExecutionIdentity,
    pub protocol: Limits,
    pub plan_payload_sha256: String,
    pub receipt_url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationPlan {
    pub schema: String,
    pub scope: String,
    pub status: String,
    pub execution: ExecutionIdentity,
    pub config: DriverConfig,
    pub control_execution: ExecutionIdentity,
    pub control_config: DriverConfig,
    pub dose_sheet: FilePin,
    pub qualification_fit: FilePin,
    pub owner_scope_cost: FilePin,
    pub pm_dispatch: FilePin,
    pub independent_review: FilePin,
    pub anchor_manifest: FilePin,
    pub anchor_manifest_sha256: String,
    pub anchor_csv_sha256: String,
    pub anchor_policy: AnchorPolicy,
    pub limits: Limits,
    pub inherited_observations: Vec<FilePin>,
    pub accepted_initlock_disposition: FilePin,
    pub residual_fit: FilePin,
    pub g_init_comparison_or_waiver: FilePin,
    pub resource_admission: FilePin,
    pub rate_usd_micros_per_hour: u64,
    pub gpu_count: u32,
    pub all_in_resource_seconds: u64,
    pub staging_archive_reserve_bytes: u64,
    pub host_available_bytes: u64,
    pub run_state_directory: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualificationAdmission {
    pub schema: String,
    pub plan_file: FilePin,
    pub plan: QualificationPlan,
    pub owner_release: FilePin,
}

/// Raw current-executable command observations, never a replacement for their
/// original report/stdout/stderr bytes. Created by the authorized S1 operator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InheritedObservation {
    pub schema: String,
    pub name: String,
    pub execution: ExecutionIdentity,
    pub executable: FilePin,
    pub context_config: FilePin,
    pub producer_command: String,
    pub argv: Vec<String>,
    pub inputs: Vec<FilePin>,
    pub native_exit: i32,
    pub stdout: FilePin,
    pub stderr: FilePin,
    pub report: Option<FilePin>,
}
fn observation<'a>(
    all: &'a [InheritedObservation],
    execution: &ExecutionIdentity,
    name: &str,
) -> Result<&'a InheritedObservation> {
    let found: Vec<_> = all
        .iter()
        .filter(|v| v.execution == *execution && v.name == name)
        .collect();
    if found.len() != 1 {
        return Err(invalid("missing/duplicate exact-arm inherited observation"));
    }
    Ok(found[0])
}
fn observation_report(observed: &InheritedObservation) -> Result<Value> {
    if observed.native_exit != 0 {
        return Err(invalid("inherited positive gate did not exit zero"));
    }
    pinned_json(
        observed
            .report
            .as_ref()
            .ok_or_else(|| invalid("inherited native report missing"))?,
    )
}
fn validate_producer_identity(report: &Value, execution: &ExecutionIdentity) -> Result<()> {
    for (key, expected) in [
        ("source_sha256", &execution.source_sha256),
        ("kernel_sha256", &execution.kernel_sha256),
        ("geometry_sha256", &execution.geometry_sha256),
    ] {
        if report["producer_compiled_identity"][key].as_str() != Some(expected.as_str()) {
            return Err(invalid("raw producer numerical identity differs"));
        }
    }
    if report["producer_driver_source_sha256"] != json!(execution.driver_source_sha256)
        || report["producer_executable_sha256"] != json!(execution.executable_sha256)
    {
        return Err(invalid("raw producer executing source/ELF differs"));
    }
    Ok(())
}
fn validate_current_relation(report: &Value, execution: &ExecutionIdentity) -> Result<()> {
    validate_relation_qualification(report)?;
    validate_producer_identity(report, execution)?;
    if report["producer_identity_scope"]
        != "actual-executing-canonical-relation-fixture-suite-not-A8-config"
    {
        return Err(invalid("canonical fixture identity scope missing"));
    }
    Ok(())
}
fn validate_probe_costs(report: &Value, execution: &ExecutionIdentity, index: u32) -> Result<()> {
    validate_producer_identity(report, execution)?;
    if report["parameter_kind"] != "gate"
        || report["parameter_index"] != index
        || report["epoch"] != 0
        || report["applied"] != false
    {
        return Err(invalid("probe parameter/source/epoch differs"));
    }
    let losses = report["candidate_losses"]
        .as_array()
        .ok_or_else(|| invalid("raw candidate costs missing"))?;
    let before = report["before"]
        .as_u64()
        .ok_or_else(|| invalid("raw current cost missing"))?;
    let candidate = report["candidate"]
        .as_u64()
        .ok_or_else(|| invalid("raw chosen ordinal missing"))?;
    if losses.len() != 16
        || losses[..3]
            .iter()
            .any(|v| v.as_u64().is_none_or(|n| n == u64::MAX))
        || losses[3..].iter().any(|v| v.as_u64() != Some(u64::MAX))
        || losses[0].as_u64() != Some(before)
        || !losses[..3].iter().any(|v| v.as_u64() != Some(before))
        || report["any_candidate_cost_differs_from_identity"] != true
        || candidate > 2
        || report["after"] != losses[candidate as usize]
    {
        return Err(invalid(
            "accepted raw all-active-candidate sensitivity predicate fails",
        ));
    }
    Ok(())
}
fn validate_probe(
    o: &InheritedObservation,
    execution: &ExecutionIdentity,
    index: u32,
    initial: &training::DriverCheckpoint,
) -> Result<()> {
    if o.execution != *execution
        || o.producer_command != "propose"
        || o.argv.len() != 5
        || o.argv.first().map(String::as_str) != Some("propose")
        || o.argv[3] != "gate"
        || o.argv.last() != Some(&index.to_string())
    {
        return Err(invalid(
            "actual named Gate0/Gate22 producer identity differs",
        ));
    }
    let report = observation_report(o)?;
    validate_probe_costs(&report, execution, index)?;
    let config: DriverConfig = pinned_json(&o.context_config)?;
    if corpus::sha256(&encode(&config)?) != execution.config_sha256 {
        return Err(invalid("probe arm context differs"));
    }
    let request_pin = o
        .inputs
        .iter()
        .find(|pin| Some(pin.sha256.as_str()) == report["request_file_sha256"].as_str())
        .ok_or_else(|| invalid("actual probe request FilePin missing"))?;
    let snapshot_pin = o
        .inputs
        .iter()
        .find(|pin| Some(pin.sha256.as_str()) == report["snapshot_file_sha256"].as_str())
        .ok_or_else(|| invalid("actual probe snapshot FilePin missing"))?;
    let request: Value = pinned_json(request_pin)?;
    let snapshot: Value = pinned_json(snapshot_pin)?;
    validate_probe_initial_snapshot(&snapshot, initial)?;
    let canonical = attention_learning::preflight::panel(&o.context_config.path)?;
    if request != canonical["probe_request"]
        || snapshot["epoch"] != 0
        || snapshot["source_sha256"] != json!(execution.source_sha256)
        || report["parameter_kind"] != "gate"
        || report["parameter_index"] != index
        || report["epoch"] != 0
        || report["applied"] != false
    {
        return Err(invalid(
            "probe source/request/parameter/zero-update scope differs",
        ));
    }
    Ok(())
}
fn same_initial_probe_model(snapshot: &Value, initial_model: &Value) -> Result<()> {
    if snapshot != initial_model {
        return Err(invalid(
            "probe snapshot differs from independently authenticated initial checkpoint model",
        ));
    }
    Ok(())
}
fn validate_probe_initial_snapshot(
    snapshot: &Value,
    initial: &training::DriverCheckpoint,
) -> Result<()> {
    let driver = PreparedDriver::load(initial.lineage.config.clone())?;
    driver.validate_checkpoint(initial)?;
    if initial.cursor != driver.initial_cursor() {
        return Err(invalid(
            "probe snapshot differs from independently authenticated initial checkpoint model",
        ));
    }
    same_initial_probe_model(snapshot, &initial.model)
}
fn validate_probe_pair_header(
    observations: &[InheritedObservation],
    execution: &ExecutionIdentity,
) -> Result<()> {
    if observations.len() != 2 {
        return Err(invalid(
            "exact Gate0/Gate22 sensitivity observation pair required",
        ));
    }
    let mut names = std::collections::BTreeSet::new();
    for o in observations {
        if o.execution != *execution
            || !["combine_probe", "relation_probe"].contains(&o.name.as_str())
            || !names.insert(o.name.as_str())
        {
            return Err(invalid(
                "sensitivity probe role/source duplicate or mismatch",
            ));
        }
    }
    Ok(())
}
fn validate_negative(o: &InheritedObservation) -> Result<()> {
    let (command,expected)=match o.name.as_str() {
        "negative_k16"=>("attention-build-identity","invalid discrete model: attention requires K8, nearest-address objective, active-leaf initialization and interleaved read-every-8 schedule"),
        "negative_nonroot"=>("init-corpus","invalid discrete model: discrete CUDA snapshot validation refused with status 6"),
        _=>return Err(invalid("unknown required negative")),
    };
    let stderr = o.stderr.read()?;
    if o.native_exit != 2
        || o.producer_command != command
        || std::str::from_utf8(&stderr)
            .map_err(|e| invalid(e.to_string()))?
            .trim()
            != expected
    {
        return Err(invalid(
            "required negative native exit/refusal reason differs",
        ));
    }
    if o.name == "negative_nonroot" {
        let actual: Value = pinned_json(
            o.report
                .as_ref()
                .ok_or_else(|| invalid("failed native initializer report missing"))?,
        )?;
        if actual["status"] != "FAILED" || actual["error"] != expected {
            return Err(invalid("nonroot native failed report differs"));
        }
    } else if o.report.is_some() {
        return Err(invalid("metadata negative unexpectedly supplied a report"));
    }
    Ok(())
}
fn validate_observations(plan: &QualificationPlan) -> Result<Vec<InheritedObservation>> {
    let mut all = Vec::new();
    for pin in &plan.inherited_observations {
        let observed: InheritedObservation = pinned_json(pin)?;
        if observed.schema != "uor-r4.attention-audit-inherited-command/1"
            || ![&plan.execution, &plan.control_execution].contains(&&observed.execution)
            || observed.argv.first() != Some(&observed.producer_command)
            || observed.inputs.is_empty()
            || observed.executable.sha256 != observed.execution.executable_sha256
        {
            return Err(invalid(
                "current-runtime inherited command identity/argv incomplete",
            ));
        }
        observed.executable.read()?;
        let context: DriverConfig = pinned_json(&observed.context_config)?;
        if corpus::sha256(&encode(&context)?) != observed.execution.config_sha256 {
            return Err(invalid("inherited arm context configuration differs"));
        }
        observed.stdout.read()?;
        observed.stderr.read()?;
        for p in &observed.inputs {
            p.read()?;
        }
        all.push(observed);
    }
    for execution in [&plan.execution, &plan.control_execution] {
        let c2 = observation(&all, execution, "canonical_initlock_c2")?;
        if c2.producer_command != "qualify-relation" {
            return Err(invalid("canonical producer command differs"));
        }
        validate_current_relation(&observation_report(c2)?, execution)?;
        for name in ["negative_k16", "negative_nonroot"] {
            validate_negative(observation(&all, execution, name)?)?;
        }
        let initial: training::DriverCheckpoint = serde_json::from_value(observation_report(
            observation(&all, execution, "initial_checkpoint")?,
        )?)
        .map_err(|e| invalid(e.to_string()))?;
        for (name, index) in [("combine_probe", 0), ("relation_probe", 22)] {
            let o = observation(&all, execution, name)?;
            validate_probe(o, execution, index, &initial)?;
        }
        let cp = observation_report(observation(&all, execution, "initial_checkpoint")?)?;
        let unpinned = observation_report(observation(&all, execution, "unpinned_checkpoint")?)?;
        let actual_config = if execution == &plan.execution {
            &plan.config
        } else {
            &plan.control_config
        };
        let checkpoint: training::DriverCheckpoint =
            serde_json::from_value(cp.clone()).map_err(|e| invalid(e.to_string()))?;
        let unpinned_checkpoint: training::DriverCheckpoint =
            serde_json::from_value(unpinned.clone()).map_err(|e| invalid(e.to_string()))?;
        let mut expected_unpinned = actual_config.clone();
        expected_unpinned.codebook_initialization =
            training::CodebookInitialization::SeededDeviceV1;
        if checkpoint.lineage.config != *actual_config
            || unpinned_checkpoint.lineage.config != expected_unpinned
        {
            return Err(invalid(
                "GInit comparison configs differ beyond pinned initialization",
            ));
        }
        PreparedDriver::load(expected_unpinned)?.validate_checkpoint(&unpinned_checkpoint)?;
        for key in ["routes", "gates", "seed", "epoch"] {
            if cp["model"][key].is_null() || cp["model"][key] != unpinned["model"][key] {
                return Err(invalid("actual pinned/unpinned GInit identity differs"));
            }
        }
        if cp["cursor"]["completed_batches"] != 0
            || cp["cursor"]["accepted_updates"] != 0
            || unpinned["cursor"]["completed_batches"] != 0
            || unpinned["cursor"]["accepted_updates"] != 0
        {
            return Err(invalid("GInit observations contain updates"));
        }
        let eval = observation_report(observation(&all, execution, "initial_dev_evaluation")?)?;
        let facts = baseline_facts(actual_config, execution, &checkpoint, &eval)?;
        if facts["full_rows"] != 14287
            || facts["psucc_rows"] != 3847
            || facts["b3_psucc"] != 1351
            || facts["b3_full"] != 4244
            || facts["top1"] != 133
            || facts["occupied_cells"] != 256
            || facts["distinct_vectors"] != 4096
            || facts["tie_mass"] != 0
            || facts["psucc_reference_top1"]
                .as_u64()
                .is_none_or(|reference| 1351u64.checked_sub(reference).is_none_or(|net| net <= 500))
        {
            return Err(invalid(
                "inherited initial reproduction/occupancy/net GCap gate fails",
            ));
        }
        if eval["model_epoch"] != 0
            || eval["updates_applied_by_evaluation"] != 0
            || eval["evaluation_mutates_model"] != false
            || eval["evaluation_lineage_sha256"] != json!(execution.lineage_sha256)
        {
            return Err(invalid(
                "actual baseline DEV source/zero-update identity differs",
            ));
        }
        for (key, expected) in [
            ("source_sha256", &execution.source_sha256),
            ("kernel_sha256", &execution.kernel_sha256),
            ("geometry_sha256", &execution.geometry_sha256),
        ] {
            if cp["model"][key].as_str() != Some(expected.as_str()) {
                return Err(invalid("inherited checkpoint numerical source differs"));
            }
        }
    }
    if all.len() != 16 {
        return Err(invalid(
            "inherited observation inventory must be eight per actual arm",
        ));
    }
    Ok(all)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StageRecord {
    pub schema: String,
    pub name: String,
    pub execution: ExecutionIdentity,
    pub status: String,
    pub observations: Vec<FilePin>,
    pub facts: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductionRestoreProof {
    schema: String,
    execution: ExecutionIdentity,
    initial_checkpoint: FilePin,
    inspected_checkpoint: FilePin,
    continuation_checkpoint: FilePin,
    steps: FilePin,
    restore_intent: FilePin,
    restore_observation: FilePin,
    initial_anchor: FilePin,
    restored_anchor: FilePin,
}
fn validate_receipt_policy(config: &DriverConfig, receipt: &training::StepReceipt) -> Result<()> {
    let stages = crate::metadata::BinderPlan::new(config.spec.context as usize)?.stages() as u32;
    let relation = receipt.coordinate.kind == training::CoordinateKind::Gate
        && receipt.coordinate.index >= config.spec.blocks * stages;
    let refused = !config.relation_learning_policy.is_learn() && relation;
    if receipt.apply_refused_by_policy != refused
        || (receipt.applied && (!receipt.any_candidate_cost_differs_from_identity || refused))
    {
        return Err(invalid(
            "production receipt violates sensitivity/FrozenZero policy ordering",
        ));
    }
    Ok(())
}
fn validate_production_proof(
    execution: &ExecutionIdentity,
    pin: &FilePin,
    facts: &Value,
) -> Result<()> {
    let proof: ProductionRestoreProof = pinned_json(pin)?;
    if proof.schema != "uor-r4.attention-audit-production-restore-proof/1"
        || proof.execution != *execution
    {
        return Err(invalid("production proof identity differs"));
    }
    let initial: training::DriverCheckpoint = pinned_json(&proof.initial_checkpoint)?;
    let inspected: training::DriverCheckpoint = pinned_json(&proof.inspected_checkpoint)?;
    let continuation: training::DriverCheckpoint = pinned_json(&proof.continuation_checkpoint)?;
    let driver = PreparedDriver::load(initial.lineage.config.clone())?;
    for cp in [&initial, &inspected, &continuation] {
        driver.validate_checkpoint(cp)?;
    }
    if driver.lineage_sha256()? != execution.lineage_sha256
        || corpus::sha256(&encode(driver.config())?) != execution.config_sha256
        || initial.cursor != driver.initial_cursor()
        || inspected.cursor.completed_batches != 384
        || inspected.cursor.accepted_updates == 0
        || continuation.cursor.completed_batches != 384
        || continuation.cursor.accepted_updates != 0
        || continuation.model_sha256 != initial.model_sha256
        || proof.continuation_checkpoint.sha256 == proof.initial_checkpoint.sha256
        || facts["accepted_updates_before_restore"] != json!(inspected.cursor.accepted_updates)
    {
        return Err(invalid(
            "production proof checkpoints/cursor/update evidence differs",
        ));
    }
    let receipts: Vec<training::StepReceipt> = pinned_json(&proof.steps)?;
    if receipts.len() != 384 {
        return Err(invalid("production proof finite receipt inventory"));
    }
    let mut cursor = driver.initial_cursor();
    for receipt in &receipts {
        validate_receipt_policy(driver.config(), receipt)?;
        if receipt.lineage_sha256 != execution.lineage_sha256
            || receipt.cursor_before != cursor
            || receipt.coordinate != driver.coordinate_at(cursor.completed_batches)?
            || receipt.cursor_after != driver.advance(&cursor, receipt.applied)?
            || receipt.model_epoch != receipt.cursor_after.accepted_updates
            || (receipt.apply_refused_by_policy && receipt.applied)
        {
            return Err(invalid("production proof receipt sequence/epoch differs"));
        }
        cursor = receipt.cursor_after;
    }
    if cursor != inspected.cursor {
        return Err(invalid(
            "production proof checkpoint differs from actual steps",
        ));
    }
    let intent: Value = pinned_json(&proof.restore_intent)?;
    let observation: Value = pinned_json(&proof.restore_observation)?;
    if intent["status"] != "CONTROLLED_PRODUCTION_RESTORE_EXERCISE_INTENT"
        || intent["natural_audit_breach_claimed"] != false
        || intent["passing"] != json!(proof.initial_checkpoint)
        || intent["inspected"] != json!(proof.inspected_checkpoint)
        || intent["steps_sha256"] != json!(proof.steps.sha256)
        || intent["attempted_cursor"] != json!(cursor)
        || observation["intent"] != json!(proof.restore_intent)
        || observation["passing"] != json!(proof.initial_checkpoint)
        || observation["continuation"] != json!(proof.continuation_checkpoint)
        || observation["observed_epoch"] != 0
        || observation["passing_storage_restored"] != true
        || observation["passing_snapshot_restored"] != true
        || observation["old_handles_refused"] != true
    {
        return Err(invalid(
            "actual controlled restore intent/observation differs",
        ));
    }
    let initial_anchor: Value = pinned_json(&proof.initial_anchor)?;
    let restored_anchor: Value = pinned_json(&proof.restored_anchor)?;
    if initial_anchor["execution"] != json!(execution)
        || restored_anchor["execution"] != json!(execution)
        || initial_anchor["evaluation"]["aggregate"] != restored_anchor["evaluation"]["aggregate"]
        || initial_anchor["evaluation"]["batches"] != restored_anchor["evaluation"]["batches"]
        || initial_anchor["evaluation"]["actual_snapshot_before"]
            != initial_anchor["evaluation"]["actual_snapshot_after"]
        || restored_anchor["evaluation"]["actual_snapshot_before"]
            != restored_anchor["evaluation"]["actual_snapshot_after"]
        || initial_anchor["evaluation"]["aggregate"]["scored_rows"] != 3847
        || initial_anchor["evaluation"]["boundary"]["stamp"]["runtime_model"]
            == restored_anchor["evaluation"]["boundary"]["stamp"]["runtime_model"]
    {
        return Err(invalid(
            "actual restored TRAIN anchor row/state invariance differs",
        ));
    }
    Ok(())
}
fn baseline_facts(
    config: &DriverConfig,
    execution: &ExecutionIdentity,
    checkpoint: &training::DriverCheckpoint,
    evaluation: &Value,
) -> Result<Value> {
    let driver = PreparedDriver::load(config.clone())?;
    driver.validate_checkpoint(checkpoint)?;
    let hashes = crate::snapshot::stored_snapshot_hash_domains(&checkpoint.model)?;
    let stages = crate::metadata::BinderPlan::new(config.spec.context as usize)?.stages();
    let combine = config.spec.blocks as usize * stages;
    let gates = checkpoint.model["gates"]
        .as_array()
        .ok_or_else(|| invalid("actual initial gates missing"))?;
    if checkpoint.cursor != driver.initial_cursor()
        || checkpoint.model["epoch"] != 0
        || checkpoint.model["read_initialization"] != json!(crate::ReadInitialization::ActiveLeafV1)
        || !crate::normalization_mechanism::stored_mode_is_default(&checkpoint.model)
        || gates.len()
            != config
                .read_spec
                .gate_count(config.spec.blocks, stages as u32)? as usize
        || gates
            .iter()
            .enumerate()
            .any(|(i, v)| v.as_i64() != Some(i64::from(i < combine && i.is_multiple_of(stages))))
        || execution.config_sha256 != corpus::sha256(&encode(config)?)
        || driver.lineage_sha256()? != execution.lineage_sha256
        || evaluation["evaluation_lineage_sha256"].as_str()
            != Some(execution.lineage_sha256.as_str())
        || evaluation["model_storage_sha256"] != json!(hashes.model_storage_sha256)
        || evaluation["model_snapshot_sha256"] != json!(hashes.model_snapshot_sha256)
        || evaluation["model_epoch"] != 0
        || evaluation["updates_applied_by_evaluation"] != 0
        || evaluation["evaluation_mutates_model"] != false
    {
        return Err(invalid(
            "actual baseline checkpoint/evaluation/INIT LOCK mismatch",
        ));
    }
    let b3 = attention_learning::evidence::summarize(
        evaluation,
        &attention_learning::dev_row_identities(config)?,
    )?;
    let occupancy = training::snapshot_metadata(&checkpoint.model)?;
    let reference_top1 = evaluation["count_reference"]["train_masked_reference"]["top8"][0]
        .as_u64()
        .ok_or_else(|| invalid("actual GPU count reference missing"))?;
    let reference_psucc = b3
        .rows
        .iter()
        .filter(|r| r.is_psucc && u64::from(r.target_token) == reference_top1)
        .count() as u64;
    Ok(
        json!({"full_rows":b3.full.rows,"psucc_rows":b3.psucc.rows,"b3_full":b3.full.target_admitted,"b3_psucc":b3.psucc.target_admitted,
        "top1":evaluation["aggregate"]["top1_correct"],"occupied_cells":occupancy["occupied_writer_cells"],
        "distinct_vectors":occupancy["distinct_full_vector_patterns"],"tie_mass":4096u64.checked_sub(occupancy["distinct_full_vector_patterns"].as_u64().ok_or_else(||invalid("occupancy count missing"))?).ok_or_else(||invalid("occupancy exceeds vocabulary"))?,
        "psucc_reference_top1":reference_psucc,"updates_applied":0}),
    )
}
pub(super) fn validate_stage(
    name: &str,
    execution: &ExecutionIdentity,
    pin: &FilePin,
) -> Result<StageRecord> {
    let stage: StageRecord = pinned_json(pin)?;
    if stage.schema != "uor-r4.attention-audit-qualified-stage/1"
        || stage.name != name
        || stage.execution != *execution
        || stage.status != "PASS"
        || stage.observations.is_empty()
    {
        return Err(invalid(
            "typed qualification stage/execution/result missing",
        ));
    }
    for raw in &stage.observations {
        raw.read()?;
    }
    match name {
        "canonical_initlock_c2" => {
            let raw: Value = pinned_json(&stage.observations[0])?;
            validate_current_relation(&raw, execution)?;
        }
        "required_negatives" => {
            if stage.facts["native_exits"] != json!([2, 2])
                || stage.facts["required_reasons_verified"] != true
            {
                return Err(invalid("negative stage facts differ"));
            }
            if stage.observations.len() != 2 {
                return Err(invalid("negative observation inventory differs"));
            }
            let mut names = std::collections::BTreeSet::new();
            for pin in &stage.observations {
                let o: InheritedObservation = pinned_json(pin)?;
                if o.native_exit != 2 || o.execution != *execution {
                    return Err(invalid("negative observation exit/execution differs"));
                }
                validate_negative(&o)?;
                names.insert(o.name);
            }
            if names.len() != 2 {
                return Err(invalid("duplicate required negative"));
            }
        }
        "g_occ" => {
            if stage.facts["full_rows"] != 14287
                || stage.facts["psucc_rows"] != 3847
                || stage.facts["b3_full"] != 4244
                || stage.facts["b3_psucc"] != 1351
                || stage.facts["top1"] != 133
                || stage.facts["occupied_cells"] != 256
                || stage.facts["distinct_vectors"] != 4096
                || stage.facts["tie_mass"] != 0
                || stage.facts["updates_applied"] != 0
                || stage.facts["psucc_reference_top1"]
                    .as_u64()
                    .is_none_or(|reference| {
                        1351u64.checked_sub(reference).is_none_or(|net| net <= 500)
                    })
            {
                return Err(invalid("actual source baseline/occupancy stage fails"));
            }
            if stage.observations.len() != 2 {
                return Err(invalid("baseline checkpoint/evaluation inventory differs"));
            }
            let cp: training::DriverCheckpoint = pinned_json(&stage.observations[0])?;
            let eval: Value = pinned_json(&stage.observations[1])?;
            if baseline_facts(&cp.lineage.config, execution, &cp, &eval)? != stage.facts {
                return Err(invalid(
                    "baseline facts differ from actual joined raw observations",
                ));
            }
        }
        "sensitivity" => {
            if stage.facts["predicate"] != "both_probes_any_active_candidate_cost_differs_v1"
                || stage.facts["combine_sensitive"] != 1
                || stage.facts["relation_sensitive"] != 1
            {
                return Err(invalid("sensitivity stage differs"));
            }
            if stage.observations.len() != 4 {
                return Err(invalid("sensitivity requires two probes plus authenticated initial checkpoint/evaluation"));
            }
            let initial: training::DriverCheckpoint = pinned_json(&stage.observations[2])?;
            let evaluation: Value = pinned_json(&stage.observations[3])?;
            baseline_facts(&initial.lineage.config, execution, &initial, &evaluation)?;
            let observations: Vec<InheritedObservation> = stage.observations[..2]
                .iter()
                .map(pinned_json)
                .collect::<Result<_>>()?;
            validate_probe_pair_header(&observations, execution)?;
            for o in &observations {
                let index = match o.name.as_str() {
                    "combine_probe" => 0,
                    "relation_probe" => 22,
                    _ => return Err(invalid("unknown sensitivity observation")),
                };
                validate_probe(o, execution, index, &initial)?;
            }
        }
        "audited_restore_epoch_cursor" => {
            if stage.facts["scope"] != "actual-a8-post-update-restore-and-train-anchor-v1"
                || stage.facts["accepted_updates_before_restore"]
                    .as_u64()
                    .unwrap_or(0)
                    == 0
                || stage.facts["advanced_attempts"] != 384
                || stage.facts["restored_epoch"] != 0
                || stage.facts["fresh_binding"] != true
                || stage.facts["passing_storage_restored"] != true
                || stage.facts["passing_snapshot_restored"] != true
                || stage.facts["anchor_equal_before_after_restore"] != true
                || stage.facts["stale_handles_refused"] != true
                || stage.facts["failure_intent_durable_before_restore"] != true
                || stage.facts["continuation_file_distinct"] != true
            {
                return Err(invalid("actual post-update A8 restore proof incomplete; synthetic controls insufficient"));
            }
            let proof: FilePin = serde_json::from_value(stage.facts["production_proof"].clone())
                .map_err(|e| invalid(e.to_string()))?;
            validate_production_proof(execution, &proof, &stage.facts)?;
        }
        _ => return Err(invalid("unknown qualification stage")),
    }
    Ok(stage)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg(feature = "cuda")]
struct QualificationOwner {
    schema: String,
    owner: String,
    decision: String,
    scope: String,
    plan_file_sha256: String,
    receipt_url: String,
}

pub fn payload_sha256(plan: &QualificationPlan) -> Result<String> {
    let mut payload = serde_json::to_value(plan).map_err(|e| invalid(e.to_string()))?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| invalid("qualification plan object"))?;
    // Authorization records independently bind the complete execution/protocol
    // payload without circular file hashes. Their contents are then validated.
    for name in [
        "qualification_fit",
        "owner_scope_cost",
        "pm_dispatch",
        "independent_review",
    ] {
        object.remove(name);
    }
    Ok(corpus::sha256(&encode(&payload)?))
}
fn validate_identity(config: &DriverConfig, execution: &ExecutionIdentity) -> Result<()> {
    super::validate_config(config)?;
    if execution.config_sha256 != corpus::sha256(&encode(config)?)
        || execution.driver_source_sha256 != training::driver_source_sha256()
        || execution.sheet_sha256 != FIT_SHEET_SHA256
        || execution.reviewed_source_commit.len() != 40
        || execution.base_source.len() != 40
        || !execution
            .reviewed_source_commit
            .bytes()
            .chain(execution.base_source.bytes())
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("qualification config/source identity differs"));
    }
    for hash in [
        &execution.source_sha256,
        &execution.kernel_sha256,
        &execution.geometry_sha256,
        &execution.executable_sha256,
        &execution.lineage_sha256,
    ] {
        corpus::validate_sha256(hash)?;
    }
    Ok(())
}
fn validate_header(plan: &QualificationPlan) -> Result<()> {
    plan.limits.validate()?;
    validate_identity(&plan.config, &plan.execution)?;
    validate_identity(&plan.control_config, &plan.control_execution)?;
    let mut paired = plan.config.clone();
    paired.relation_learning_policy = plan.control_config.relation_learning_policy;
    if plan.schema != PLAN_SCHEMA
        || plan.scope != QUAL_SCOPE
        || plan.status != "HELD_FINITE_QUALIFICATION_OWNER_ACTION_REQUIRED"
        || !plan.config.relation_learning_policy.is_learn()
        || plan.control_config.relation_learning_policy.is_learn()
        || paired != plan.control_config
        || plan.execution.source_sha256 != plan.control_execution.source_sha256
        || plan.execution.kernel_sha256 != plan.control_execution.kernel_sha256
        || plan.execution.geometry_sha256 != plan.control_execution.geometry_sha256
        || plan.execution.executable_sha256 != plan.control_execution.executable_sha256
        || plan.dose_sheet.sha256 != FIT_SHEET_SHA256
        || plan.anchor_csv_sha256 != REGISTERED_CSV_SHA256
        || plan.anchor_policy
            != (AnchorPolicy {
                start_eligible_index: 1_049_856,
                count: 3847,
                context: 192,
            })
        || !plan.run_state_directory.is_absolute()
        || plan.rate_usd_micros_per_hour == 0
        || plan.rate_usd_micros_per_hour > 8_000_000
        || !(1..=4).contains(&plan.gpu_count)
        || plan.all_in_resource_seconds == 0
    {
        return Err(invalid(
            "distinct finite qualification header/paired scope differs",
        ));
    }
    validate_reserve(
        plan.host_available_bytes,
        plan.staging_archive_reserve_bytes,
    )
}
fn validate_authorization(
    plan: &QualificationPlan,
    pin: &FilePin,
    role: &str,
    actor: &str,
    verdict: &str,
) -> Result<()> {
    let record: Authorization = pinned_json(pin)?;
    if record.schema != "uor-r4.attention-audit-qualification-authorization/1"
        || record.role != role
        || (if actor == "independent_reviewer" {
            !["Claude", "DeepSeek", "AGY"].contains(&record.actor.as_str())
        } else {
            record.actor != actor
        })
        || record.verdict != verdict
        || record.execution != plan.execution
        || record.protocol != plan.limits
        || record.plan_payload_sha256 != payload_sha256(plan)?
        || !valid_receipt_url(&record.receipt_url)
    {
        return Err(invalid(
            "actual finite qualification FIT/owner/PM/review binding required",
        ));
    }
    Ok(())
}
fn validate_plan(plan: &QualificationPlan) -> Result<TrainAnchorManifest> {
    validate_header(plan)?;
    plan.dose_sheet.read()?;
    for (pin, role, actor, verdict) in [
        (
            &plan.qualification_fit,
            "qualification_fit",
            "independent_reviewer",
            "TEST FITNESS: FIT",
        ),
        (
            &plan.owner_scope_cost,
            "owner_cost_scope",
            "Casey",
            "APPROVE_EXACT_FINITE_QUALIFICATION_SCOPE_AND_COST",
        ),
        (
            &plan.pm_dispatch,
            "pm_dispatch",
            "Kimi",
            "DISPATCH_FINITE_SOURCE_QUALIFICATION_ONLY",
        ),
        (
            &plan.independent_review,
            "source_plan_review",
            "independent_reviewer",
            "APPROVE_EXACT_QUALIFICATION_PLAN",
        ),
    ] {
        validate_authorization(plan, pin, role, actor, verdict)?;
    }
    attention_learning::validate_injective_table(&plan.config)?;
    let anchor: TrainAnchorManifest = pinned_json(&plan.anchor_manifest)?;
    if anchor.manifest_sha256 != plan.anchor_manifest_sha256
        || anchor.policy != plan.anchor_policy
        || anchor.row_csv_sha256 != plan.anchor_csv_sha256
    {
        return Err(invalid("qualification anchor differs"));
    }
    for pin in [
        &plan.accepted_initlock_disposition,
        &plan.residual_fit,
        &plan.g_init_comparison_or_waiver,
    ] {
        pin.read()?;
    }
    let resource = validate_resource(&plan.resource_admission, &plan.execution, true)?;
    if resource.gpu_count != plan.gpu_count
        || resource.rate_usd_micros_per_hour != plan.rate_usd_micros_per_hour
        || resource.host_available_bytes != plan.host_available_bytes
        || resource.staging_archive_reserve_bytes != plan.staging_archive_reserve_bytes
    {
        return Err(invalid(
            "qualification plan resource/rate differs from actual native observation",
        ));
    }
    validate_observations(plan)?;
    Ok(anchor)
}
pub fn check_plan(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let plan: QualificationPlan =
        serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    let anchor = validate_plan(&plan)?;
    let driver = PreparedDriver::load(plan.config.clone())?;
    validate_runtime(&plan, &driver)?;
    crate::attention_anchor_device::PreparedTrainAnchor::prepare(
        driver.train(),
        &anchor,
        &registration(&plan, &driver)?,
    )?;
    Ok(
        json!({"status":"QUALIFICATION_PLAN_METADATA_VALID","plan_file_sha256":corpus::sha256(&bytes),
        "cuda_context_created":false,"dose_release":false,"qualification_executed":false}),
    )
}
fn registration(
    plan: &QualificationPlan,
    driver: &PreparedDriver,
) -> Result<RegisteredAnchorBinding> {
    Ok(RegisteredAnchorBinding {
        corpus: driver.train().binding().clone(),
        policy: plan.anchor_policy.clone(),
        manifest_sha256: plan.anchor_manifest_sha256.clone(),
        csv_sha256: plan.anchor_csv_sha256.clone(),
        sheet_sha256: plan.dose_sheet.sha256.clone(),
        plan_sha256: corpus::sha256(&encode(plan)?),
    })
}
fn validate_runtime(plan: &QualificationPlan, driver: &PreparedDriver) -> Result<()> {
    if driver.config() != &plan.config
        || driver.lineage_sha256()? != plan.execution.lineage_sha256
        || executable_sha256()? != plan.execution.executable_sha256
    {
        return Err(invalid("qualification actual runtime differs"));
    }
    let control = PreparedDriver::load(plan.control_config.clone())?;
    if control.lineage_sha256()? != plan.control_execution.lineage_sha256 {
        return Err(invalid("qualification control lineage differs"));
    }
    #[cfg(feature = "cuda")]
    for (name, expected) in [
        ("source_sha256", &plan.execution.source_sha256),
        ("kernel_sha256", &plan.execution.kernel_sha256),
        ("geometry_sha256", &plan.execution.geometry_sha256),
    ] {
        if crate::device::compiled_identity()[name].as_str() != Some(expected.as_str()) {
            return Err(invalid("qualification compiled identity differs"));
        }
    }
    Ok(())
}
#[cfg(not(feature = "cuda"))]
pub fn owner_release(_path: &Path, _output: &Path) -> Result<()> {
    Err(Error::TrainingHeld)
}
#[cfg(feature = "cuda")]
pub fn owner_release(path: &Path, output: &Path) -> Result<()> {
    use std::io::IsTerminal;
    require_personal_terminal(
        std::env::var_os("UOR_LAB").is_some(),
        std::io::stdin().is_terminal(),
        std::io::stdout().is_terminal(),
    )?;
    check_plan(path)?;
    let raw = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let plan: QualificationPlan =
        serde_json::from_slice(&raw).map_err(|e| invalid(e.to_string()))?;
    require_disjoint_directories(output, &plan.run_state_directory)?;
    if output.exists() || plan.run_state_directory.exists() {
        return Err(invalid("fresh qualification release/state required"));
    }
    let phrase = format!(
        "I am Casey; release finite audited source qualification only for plan {}",
        corpus::sha256(&raw)
    );
    println!("Finite source qualification; dose training remains held.\nType exactly: {phrase}");
    std::io::stdout()
        .flush()
        .map_err(|e| invalid(e.to_string()))?;
    let mut entered = String::new();
    std::io::stdin()
        .read_line(&mut entered)
        .map_err(|e| invalid(e.to_string()))?;
    if entered.trim_end_matches(['\r', '\n']) != phrase {
        return Err(Error::TrainingHeld);
    }
    if fs::read(path).map_err(|e| invalid(e.to_string()))? != raw {
        return Err(invalid(
            "qualification plan changed during personal confirmation",
        ));
    }
    validate_plan(&plan)?;
    private_directory(output)?;
    let owner_path = fs::canonicalize(output)
        .map_err(|e| invalid(e.to_string()))?
        .join("owner.json");
    let owner = QualificationOwner {
        schema: OWNER_SCHEMA.into(),
        owner: "Casey".into(),
        decision: DECISION.into(),
        scope: QUAL_SCOPE.into(),
        plan_file_sha256: corpus::sha256(&raw),
        receipt_url: format!("owner-terminal:{}", owner_path.display()),
    };
    let owner_release = persist(&owner_path, &owner)?;
    persist(
        &output.join("admission.json"),
        &QualificationAdmission {
            schema: ADMISSION_SCHEMA.into(),
            plan_file: FilePin {
                path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
                sha256: corpus::sha256(&raw),
            },
            plan,
            owner_release,
        },
    )?;
    Ok(())
}

/// Only `validated_release` constructs this capability after authenticating
/// the distinct finite authorization chain. It carries no dose release.
#[cfg(feature = "cuda")]
pub(crate) struct ValidatedQualificationRelease {
    plan: QualificationPlan,
    admission_sha256: String,
    plan_sha256: String,
}
#[cfg(feature = "cuda")]
impl ValidatedQualificationRelease {
    pub(crate) fn finite_bound(&self, driver: &PreparedDriver) -> Result<u64> {
        let execution = if driver.config() == &self.plan.config {
            &self.plan.execution
        } else if driver.config() == &self.plan.control_config {
            &self.plan.control_execution
        } else {
            return Err(Error::TrainingHeld);
        };
        if driver.lineage_sha256()? != execution.lineage_sha256
            || self.plan.limits.throughput_attempts_per_arm != 384
            || self.plan.limits.protocol != PROTOCOL
        {
            return Err(Error::TrainingHeld);
        }
        Ok(self.plan.limits.throughput_attempts_per_arm)
    }
    fn execution(&self, driver: &PreparedDriver) -> Result<&ExecutionIdentity> {
        self.finite_bound(driver)?;
        Ok(if driver.config() == &self.plan.config {
            &self.plan.execution
        } else {
            &self.plan.control_execution
        })
    }
}
#[cfg(feature = "cuda")]
fn validated_release(
    admission: &QualificationAdmission,
    driver: &PreparedDriver,
) -> Result<(ValidatedQualificationRelease, TrainAnchorManifest)> {
    if admission.schema != ADMISSION_SCHEMA {
        return Err(Error::TrainingHeld);
    }
    let plan: QualificationPlan = pinned_json(&admission.plan_file)?;
    if encode(&plan)? != encode(&admission.plan)? {
        return Err(invalid("qualification embedded/raw plan differ"));
    }
    let anchor = validate_plan(&plan)?;
    validate_runtime(&plan, driver)?;
    let owner: QualificationOwner = pinned_json(&admission.owner_release)?;
    if owner.schema != OWNER_SCHEMA
        || owner.owner != "Casey"
        || owner.decision != DECISION
        || owner.scope != QUAL_SCOPE
        || owner.plan_file_sha256 != admission.plan_file.sha256
        || owner.receipt_url != format!("owner-terminal:{}", admission.owner_release.path.display())
    {
        return Err(Error::TrainingHeld);
    }
    Ok((
        ValidatedQualificationRelease {
            plan_sha256: corpus::sha256(&encode(&plan)?),
            plan,
            admission_sha256: corpus::sha256(&encode(admission)?),
        },
        anchor,
    ))
}

#[cfg(feature = "cuda")]
fn nanos(started: std::time::Instant) -> Result<u64> {
    u64::try_from(started.elapsed().as_nanos()).map_err(|_| invalid("elapsed nanosecond overflow"))
}
#[cfg(feature = "cuda")]
fn finite_anchor(
    release: &ValidatedQualificationRelease,
    driver: &PreparedDriver,
    anchor: &crate::attention_anchor_device::PreparedTrainAnchor,
    model: &crate::device::Device,
    state: (training::Cursor, u64, &str),
    path: &Path,
) -> Result<(Value, FilePin, u64)> {
    let (cursor, generation, counters_sha256) = state;
    let execution = release.execution(driver)?;
    model.synchronize()?;
    let started = std::time::Instant::now();
    let checkpoint = driver.checkpoint(model, cursor)?;
    let cp = persist(&path.with_extension("checkpoint.json"), &checkpoint)?;
    let snapshot = snapshot_pin(&checkpoint, &cp)?;
    let stamp = crate::attention_audit::StateStamp {
        run_sha256: release.admission_sha256.clone(),
        counters_sha256: counters_sha256.into(),
        completed: cursor.completed_batches,
        epoch: cursor.accepted_updates,
        generation,
        runtime_model: model.binding().model,
    };
    let binding = crate::attention_anchor_device::BoundaryBinding {
        checkpoint_file: cp.clone(),
        snapshot: snapshot.clone(),
        stamp: stamp.clone(),
        plan_sha256: release.plan_sha256.clone(),
        sheet_sha256: FIT_SHEET_SHA256.into(),
        anchor_manifest_sha256: release.plan.anchor_manifest_sha256.clone(),
        lineage_sha256: execution.lineage_sha256.clone(),
        config_sha256: execution.config_sha256.clone(),
        source_sha256: execution.source_sha256.clone(),
        kernel_sha256: execution.kernel_sha256.clone(),
        geometry_sha256: execution.geometry_sha256.clone(),
    };
    let boundary = crate::attention_anchor_device::validate_boundary(
        anchor,
        &binding,
        &stamp,
        &cp.read()?,
        driver,
    )?;
    // Narrow same-module issuance: qualified stage, actual driver lineage,
    // immutable anchor registration and freshly authenticated full checkpoint.
    if anchor.registration().plan_sha256 != release.plan_sha256 {
        return Err(Error::TrainingHeld);
    }
    let permit = ControllerAuditPermit {
        plan_sha256: release.plan_sha256.clone(),
        anchor_manifest_sha256: release.plan.anchor_manifest_sha256.clone(),
        sheet_sha256: FIT_SHEET_SHA256.into(),
        execution: execution.clone(),
        stamp,
        snapshot,
    };
    let evaluation = crate::attention_anchor_device::evaluate_boundary(
        &permit, anchor, driver, model, &boundary,
    )?;
    let pin = persist(
        path,
        &json!({"schema":"uor-r4.attention-audit-finite-anchor/1","execution":execution,
        "scope":"finite-qualification-only","evaluation":evaluation,"checkpoint":cp,"updates_applied":0}),
    )?;
    // End after device sync AND full payload fsync/readback, not kernel time.
    let elapsed = nanos(started)?;
    Ok((evaluation, pin, elapsed))
}

#[cfg(feature = "cuda")]
fn family_index(driver: &PreparedDriver, index: u64) -> Result<usize> {
    let coordinate = driver.coordinate_at(index)?;
    let combine = driver.config().spec.blocks
        * crate::metadata::BinderPlan::new(driver.config().spec.context as usize)?.stages() as u32;
    Ok(match coordinate.kind {
        training::CoordinateKind::VectorBlock => 0,
        training::CoordinateKind::Gate => {
            if coordinate.index < combine {
                1
            } else {
                2
            }
        }
        _ => return Err(invalid("qualification unexpected coordinate family")),
    })
}
#[cfg(feature = "cuda")]
struct ProductionObserved {
    stage: StageRecord,
    family_ns: [u64; 3],
    family_counts: [u64; 3],
    full_counts: [u64; 3],
    gate_ns: Vec<u64>,
    restore_ns: u64,
    raw: Vec<FilePin>,
}
#[cfg(feature = "cuda")]
fn production_stage(
    release: &ValidatedQualificationRelease,
    driver: &PreparedDriver,
    manifest: &TrainAnchorManifest,
    out: &Path,
) -> Result<ProductionObserved> {
    private_directory(out)?;
    let execution = release.execution(driver)?.clone();
    let registration = RegisteredAnchorBinding {
        corpus: driver.train().binding().clone(),
        policy: release.plan.anchor_policy.clone(),
        manifest_sha256: release.plan.anchor_manifest_sha256.clone(),
        csv_sha256: REGISTERED_CSV_SHA256.into(),
        sheet_sha256: FIT_SHEET_SHA256.into(),
        plan_sha256: release.plan_sha256.clone(),
    };
    let anchor = crate::attention_anchor_device::PreparedTrainAnchor::prepare(
        driver.train(),
        manifest,
        &registration,
    )?;
    let mut model = Some(driver.new_model(0)?);
    let mut cursor = driver.initial_cursor();
    let result = (|| -> Result<ProductionObserved> {
        let baseline = driver.checkpoint(model.as_ref().unwrap(), cursor)?;
        attention_learning::validate_initial_storage(driver.config(), &baseline.model)?;
        let baseline_file = persist(&out.join("passing-initial.json"), &baseline)?;
        let mut gate_ns = Vec::new();
        let mut raw = vec![baseline_file.clone()];
        let empty_counters = corpus::sha256(&encode(&Vec::<training::StepReceipt>::new())?);
        let mut baseline_eval = None;
        let mut baseline_anchor_pin = None;
        for sample in 0..release.plan.limits.gate_samples {
            let (evaluation, pin, elapsed) = finite_anchor(
                release,
                driver,
                &anchor,
                model.as_ref().unwrap(),
                (cursor, 0, &empty_counters),
                &out.join(format!("baseline-anchor-{sample}.json")),
            )?;
            if baseline_eval
                .as_ref()
                .is_some_and(|v| !stable_anchor_equal(v, &evaluation))
            {
                return Err(invalid("repeated baseline anchor differs"));
            }
            let measurement = persist(
                &out.join(format!("baseline-anchor-timing-{sample}.json")),
                &OperationTiming {
                    schema: "uor-r4.attention-audit-operation-timing/1".into(),
                    execution: execution.clone(),
                    operation: "complete_anchor_sample".into(),
                    elapsed_ns: elapsed,
                    artifact: pin.clone(),
                },
            )?;
            baseline_eval = Some(evaluation);
            baseline_anchor_pin = Some(pin.clone());
            gate_ns.push(elapsed);
            raw.push(pin);
            raw.push(measurement);
        }
        let initial_anchor = baseline_eval.ok_or_else(|| invalid("baseline anchor missing"))?;
        let capability = training::issue_audit_qualification_admission(driver, release)?;
        let mut family_ns = [0u64; 3];
        let mut family_counts = [0u64; 3];
        let mut receipts = Vec::new();
        for index in 0..release.finite_bound(driver)? {
            let actual = model.as_mut().unwrap();
            actual.synchronize()?;
            let started = std::time::Instant::now();
            let receipt = driver.transition(actual, &cursor, &capability)?;
            actual.synchronize()?;
            let elapsed = nanos(started)?;
            let family = family_index(driver, index)?;
            family_ns[family] = family_ns[family]
                .checked_add(elapsed)
                .ok_or_else(|| invalid("family timer overflow"))?;
            family_counts[family] += 1;
            let pin = persist(
                &out.join(format!("step-{index:03}.json")),
                &TransitionTiming {
                    schema: "uor-r4.attention-audit-transition-timing/1".into(),
                    execution: execution.clone(),
                    family,
                    elapsed_ns: elapsed,
                    receipt: receipt.clone(),
                },
            )?;
            cursor = receipt.cursor_after;
            receipts.push(receipt);
            raw.push(pin);
        }
        if cursor.accepted_updates == 0 {
            return Err(invalid("NOT_RUN: production restore after update unavailable in declared finite sample; no search/retry permitted"));
        }
        let restore_started = std::time::Instant::now();
        let steps_file = persist(&out.join("steps.json"), &receipts)?;
        raw.push(steps_file.clone());
        let failed = driver.checkpoint(model.as_ref().unwrap(), cursor)?;
        let failed_file = persist(&out.join("inspected-finite-checkpoint.json"), &failed)?;
        raw.push(failed_file.clone());
        let carried = model
            .as_ref()
            .unwrap()
            .carry_windows(anchor.batches()[0].windows())?;
        let proposal = model.as_ref().unwrap().propose(
            &carried,
            anchor.batches()[0].targets(),
            crate::Parameter::Gate(0),
        )?;
        let old_binding = model.as_ref().unwrap().binding().model;
        let intent = persist(
            &out.join("controlled-restore-intent.json"),
            &json!({"schema":"uor-r4.attention-audit-controlled-production-restore/1",
            "status":"CONTROLLED_PRODUCTION_RESTORE_EXERCISE_INTENT","natural_audit_breach_claimed":false,
            "execution":execution,"passing":baseline_file,"inspected":failed_file,"attempted_cursor":cursor,
            "steps_sha256":corpus::sha256(&encode(&receipts)?),"resume":false}),
        )?;
        raw.push(intent.clone());
        drop(model.take()); // old scratch and model buffers gone before restore
        let (restored, advanced, continuation) = restore_passing_device(
            driver,
            &baseline_file,
            &snapshot_pin(&baseline, &baseline_file)?,
            cursor.completed_batches,
        )?;
        model = Some(restored);
        let actual = model.as_mut().unwrap();
        actual.synchronize()?;
        exact_refusal(
            actual.forward(&carried, anchor.batches()[0].targets()),
            "carried batch belongs to another model/source/geometry",
        )?;
        exact_refusal(
            actual.apply(&proposal),
            "proposal belongs to another model/source/geometry",
        )?;
        let continued = persist(&out.join("advanced-continuation.json"), &continuation)?;
        let same_storage = continuation.model_sha256 == baseline.model_sha256;
        let same_snapshot = crate::snapshot::stored_snapshot_hash_domains(&continuation.model)?
            == crate::snapshot::stored_snapshot_hash_domains(&baseline.model)?;
        if !same_storage
            || !same_snapshot
            || actual.epoch()? != 0
            || actual.binding().model == old_binding
            || continued.sha256 == baseline_file.sha256
            || advanced.completed_batches != 384
            || advanced.accepted_updates != 0
        {
            return Err(invalid(
                "actual production restore/cursor/file identity differs",
            ));
        }
        let restore_record = persist(
            &out.join("production-restore-observed.json"),
            &json!({"execution":execution,"intent":intent,"passing":baseline_file,
            "continuation":continued,"advanced_cursor":advanced,"observed_epoch":actual.epoch()?,"new_runtime_model":actual.binding().model,
            "passing_storage_restored":same_storage,"passing_snapshot_restored":same_snapshot,"old_handles_refused":true}),
        )?;
        raw.push(continued.clone());
        raw.push(restore_record.clone());
        let restore_ns = nanos(restore_started)?;
        raw.push(persist(
            &out.join("restore-timing.json"),
            &OperationTiming {
                schema: "uor-r4.attention-audit-operation-timing/1".into(),
                execution: execution.clone(),
                operation: "complete_restore_sample".into(),
                elapsed_ns: restore_ns,
                artifact: restore_record.clone(),
            },
        )?);
        let counters_sha256 = corpus::sha256(&encode(&receipts)?);
        let (after, pin, _) = finite_anchor(
            release,
            driver,
            &anchor,
            actual,
            (advanced, 1, &counters_sha256),
            &out.join("restored-anchor.json"),
        )?;
        raw.push(pin.clone());
        // Different generation/cursor/binding are expected. Compare the actual
        // ordered row outputs and GPU aggregate, never recompute admissions.
        if initial_anchor["aggregate"] != after["aggregate"]
            || initial_anchor["batches"] != after["batches"]
        {
            return Err(invalid(
                "production TRAIN anchor changed after passing-model restore",
            ));
        }
        let mut full_counts = [0u64; 3];
        for i in 0..16404 {
            full_counts[family_index(driver, i)?] += 1;
        }
        if family_counts.contains(&0) {
            return Err(invalid("finite sample omitted production family"));
        }
        let proof = persist(
            &out.join("production-proof.json"),
            &ProductionRestoreProof {
                schema: "uor-r4.attention-audit-production-restore-proof/1".into(),
                execution: execution.clone(),
                initial_checkpoint: baseline_file.clone(),
                inspected_checkpoint: failed_file.clone(),
                continuation_checkpoint: continued.clone(),
                steps: steps_file,
                restore_intent: intent,
                restore_observation: restore_record,
                initial_anchor: baseline_anchor_pin
                    .ok_or_else(|| invalid("baseline anchor pin"))?,
                restored_anchor: pin,
            },
        )?;
        raw.push(proof.clone());
        let stage = StageRecord {
            schema: "uor-r4.attention-audit-qualified-stage/1".into(),
            name: "audited_restore_epoch_cursor".into(),
            execution,
            status: "PASS".into(),
            observations: raw.clone(),
            facts: json!({"scope":"actual-a8-post-update-restore-and-train-anchor-v1",
            "exercise":"CONTROLLED_PRODUCTION_RESTORE_EXERCISE","natural_audit_breach_claimed":false,
            "accepted_updates_before_restore":cursor.accepted_updates,"advanced_attempts":384,"restored_epoch":0,
            "fresh_binding":true,"passing_storage_restored":same_storage,"passing_snapshot_restored":same_snapshot,
            "anchor_equal_before_after_restore":true,"stale_handles_refused":true,"failure_intent_durable_before_restore":true,
            "continuation_file_distinct":continued.sha256!=baseline_file.sha256,"production_proof":proof}),
        };
        Ok(ProductionObserved {
            stage,
            family_ns,
            family_counts,
            full_counts,
            gate_ns,
            restore_ns,
            raw,
        })
    })();
    if let Err(error) = &result {
        let readback = model.as_ref().map(|m| {
            m.snapshot()
                .and_then(|v| persist(&out.join("failed-model.json"), &v))
        });
        let _ = persist(
            &out.join("failure.json"),
            &failure_payload(&error.to_string(), &cursor, readback),
        );
    }
    result
}

#[cfg(feature = "cuda")]
fn synthetic_stage(release: &ValidatedQualificationRelease, out: &Path) -> Result<FilePin> {
    release.plan.limits.validate()?;
    let passing = crate::driver_qualification::literal_fixture(2)?;
    let mut model = crate::device::Device::from_snapshot(passing.clone(), 0)?;
    let identity = crate::Identity {
        token: 1,
        occurrence: 0,
        record: 17,
        version: 9,
    };
    let carried = model.carry(&[identity])?;
    let proposal = model.propose(&carried, &[0], crate::Parameter::VectorBlock(0))?;
    let observed = proposal.observation()?;
    if observed.candidate != 7
        || observed.before != 640
        || observed.after != 544
        || observed.epoch != 0
    {
        return Err(invalid(
            "predeclared synthetic fixture differs; no replacement search",
        ));
    }
    let passing_pin = persist(&out.join("synthetic-passing.json"), &passing)?;
    let old_model = model.binding().model;
    if !model.apply(&proposal)? || model.epoch()? != 1 {
        return Err(invalid("synthetic improving update absent"));
    }
    exact_refusal(
        model.forward(&carried, &[0]),
        "carried vectors are stale at the current model epoch",
    )?;
    let failed = persist(&out.join("synthetic-updated.json"), &model.snapshot()?)?;
    let intent = persist(
        &out.join("synthetic-restore-intent.json"),
        &json!({"status":"SYNTHETIC_RESTORE_INTENT",
        "passing":passing_pin,"failed":failed,"accepted_updates":1,"production_rollback_claimed":false}),
    )?;
    drop(model);
    let mut restored = crate::device::Device::from_snapshot(pinned_json(&passing_pin)?, 0)?;
    exact_refusal(
        restored.apply(&proposal),
        "proposal belongs to another model/source/geometry",
    )?;
    exact_refusal(
        restored.forward(&carried, &[0]),
        "carried batch belongs to another model/source/geometry",
    )?;
    if restored.epoch()? != 0
        || restored.binding().model == old_model
        || crate::snapshot::snapshot_hash_domains(&restored.snapshot()?)?
            != crate::snapshot::snapshot_hash_domains(&passing)?
    {
        return Err(invalid(
            "synthetic fresh-binding restore/stale-handle control failed",
        ));
    }
    persist(
        &out.join("synthetic-control.json"),
        &json!({"schema":"uor-r4.attention-audit-synthetic-restore/1","status":"PASS",
        "scope":"synthetic-only-not-production-rollback","protocol":release.plan.limits,"intent":intent,"passing":passing_pin,
        "failed":failed,"apply_calls":1,"accepted_updates":1,"stale_apply_refusals":1,"stale_carry_refusals":1,
        "observed_restored_epoch":restored.epoch()?,"fresh_binding":true,"production_qualification_claimed":false}),
    )
}

#[cfg(feature = "cuda")]
fn emit_qualification(
    release: &ValidatedQualificationRelease,
    all: &[InheritedObservation],
    production: &ProductionObserved,
    out: &Path,
) -> Result<FilePin> {
    let execution = &production.stage.execution;
    let c2 = observation(all, execution, "canonical_initlock_c2")?
        .report
        .clone()
        .ok_or_else(|| invalid("canonical report"))?;
    let initial = observation(all, execution, "initial_checkpoint")?
        .report
        .clone()
        .ok_or_else(|| invalid("initial checkpoint"))?;
    let evaluation = observation(all, execution, "initial_dev_evaluation")?
        .report
        .clone()
        .ok_or_else(|| invalid("initial evaluation"))?;
    let checkpoint: training::DriverCheckpoint = pinned_json(&initial)?;
    let facts = baseline_facts(
        &checkpoint.lineage.config,
        execution,
        &checkpoint,
        &pinned_json(&evaluation)?,
    )?;
    let mut checks = Vec::new();
    let canonical = StageRecord {
        schema: "uor-r4.attention-audit-qualified-stage/1".into(),
        name: "canonical_initlock_c2".into(),
        execution: execution.clone(),
        status: "PASS".into(),
        observations: vec![c2],
        facts: json!({"controls":9,"applied_updates":0}),
    };
    let negatives = StageRecord {
        schema: canonical.schema.clone(),
        name: "required_negatives".into(),
        execution: execution.clone(),
        status: "PASS".into(),
        observations: release
            .plan
            .inherited_observations
            .iter()
            .filter_map(|p| {
                pinned_json::<InheritedObservation>(p)
                    .ok()
                    .filter(|o| {
                        o.execution == *execution
                            && ["negative_k16", "negative_nonroot"].contains(&o.name.as_str())
                    })
                    .map(|_| p.clone())
            })
            .collect(),
        facts: json!({"native_exits":[2,2],"required_reasons_verified":true}),
    };
    let occupancy = StageRecord {
        schema: canonical.schema.clone(),
        name: "g_occ".into(),
        execution: execution.clone(),
        status: "PASS".into(),
        observations: vec![initial.clone(), evaluation.clone()],
        facts: facts.clone(),
    };
    let sensitive = StageRecord {
        schema: canonical.schema.clone(),
        name: "sensitivity".into(),
        execution: execution.clone(),
        status: "PASS".into(),
        observations: release
            .plan
            .inherited_observations
            .iter()
            .filter_map(|pin| {
                pinned_json::<InheritedObservation>(pin)
                    .ok()
                    .filter(|o| {
                        o.execution == *execution
                            && ["combine_probe", "relation_probe"].contains(&o.name.as_str())
                    })
                    .map(|_| pin.clone())
            })
            .chain([initial, evaluation])
            .collect(),
        facts: json!({"predicate":"both_probes_any_active_candidate_cost_differs_v1","combine_sensitive":1,"relation_sensitive":1}),
    };
    for stage in [
        &canonical,
        &negatives,
        &occupancy,
        &sensitive,
        &production.stage,
    ] {
        let pin = persist(&out.join(format!("{}.json", stage.name)), stage)?;
        validate_stage(&stage.name, execution, &pin)?;
        checks.push(QualifiedCheck {
            name: stage.name.clone(),
            status: "PASS".into(),
            artifact: pin,
        });
    }
    let qualification = Qualification {
        schema: "uor-r4.attention-audit-qualification/1".into(),
        execution: execution.clone(),
        checks,
        g_init_comparison_or_waiver: release.plan.g_init_comparison_or_waiver.clone(),
        accepted_initlock_disposition: release.plan.accepted_initlock_disposition.clone(),
        residual_fit: release.plan.residual_fit.clone(),
        b3_psucc: facts["b3_psucc"].as_u64().ok_or_else(|| invalid("B3"))?,
        b3_full: facts["b3_full"]
            .as_u64()
            .ok_or_else(|| invalid("B3 full"))?,
        top1: facts["top1"].as_u64().ok_or_else(|| invalid("top1"))?,
        bar_i: 86,
        epoch: 0,
        updates_applied: 0,
        reference_capture_scope: "DEFERRED_TO_DOSE".into(),
        full_rows: 14287,
        psucc_rows: 3847,
        psucc_reference_top1: facts["psucc_reference_top1"]
            .as_u64()
            .ok_or_else(|| invalid("reference"))?,
        occupied_cells: 256,
        distinct_vectors: 4096,
        tie_mass: 0,
        negative_native_exits: [2, 2],
        sensitivity_predicate: "both_probes_any_active_candidate_cost_differs_v1".into(),
        combine_sensitive: 1,
        relation_sensitive: 1,
    };
    persist(&out.join("qualification.json"), &qualification)
}

/// Requires separately approved finite qualification inputs. This never
/// creates a resource, changes a lease or runs a complete scientific dose.
#[cfg(feature = "cuda")]
pub fn run(config: DriverConfig, admission: QualificationAdmission, out: &Path) -> Result<()> {
    let driver = PreparedDriver::load(config)?;
    let (release, manifest) = validated_release(&admission, &driver)?;
    let all = validate_observations(&release.plan)?;
    require_disjoint_directories(out, &release.plan.run_state_directory)?;
    private_directory(out)?;
    let _lock = audited_lock(
        &release.plan.run_state_directory,
        &release.admission_sha256,
        QUAL_SCOPE,
    )?;
    persist(&out.join("admission.json"), &admission)?;
    let setup = std::time::Instant::now();
    let result = (|| -> Result<()> {
        let synthetic = synthetic_stage(&release, out)?;
        let bookkeeping_pin = measure_bookkeeping(
            &release.plan.execution,
            &out.join("host-metadata-bookkeeping"),
        )?;
        let bookkeeping: BookkeepingMeasurement = pinned_json(&bookkeeping_pin)?;
        let setup_ns = nanos(setup)?;
        let relation = production_stage(&release, &driver, &manifest, &out.join("relation"))?;
        let control_driver = PreparedDriver::load(release.plan.control_config.clone())?;
        let control = production_stage(
            &release,
            &control_driver,
            &manifest,
            &out.join("frozen-zero"),
        )?;
        let relation_q = emit_qualification(&release, &all, &relation, &out.join("relation"))?;
        let control_q = emit_qualification(&release, &all, &control, &out.join("frozen-zero"))?;
        let gate = relation
            .gate_ns
            .iter()
            .chain(&control.gate_ns)
            .copied()
            .max()
            .ok_or_else(|| invalid("gate timing absent"))?;
        let restore = relation.restore_ns.max(control.restore_ns);
        let projected = project_added_components(
            gate,
            *bookkeeping
                .step_samples_ns
                .iter()
                .max()
                .ok_or_else(|| invalid("step metadata samples missing"))?,
            bookkeeping.pass_boundary_ns,
            restore,
            bookkeeping.breach_restore_ack_ns,
        )?;
        let denominator = projected_ns(
            &relation.family_ns,
            &relation.family_counts,
            &relation.full_counts,
        )?
        .checked_add(projected_ns(
            &control.family_ns,
            &control.family_counts,
            &control.full_counts,
        )?)
        .ok_or_else(|| invalid("denominator overflow"))?;
        let mut raw = vec![synthetic, bookkeeping_pin.clone()];
        raw.extend(relation.raw.clone());
        raw.extend(control.raw.clone());
        // Both arms' observed complete gate cost: keep the conservative maximum
        // sample for each registered sample index; the raw arm samples survive.
        let gate_samples = relation
            .gate_ns
            .iter()
            .zip(&control.gate_ns)
            .map(|(a, b)| (*a).max(*b))
            .collect();
        let timing = TimingProtocol {
            schema: "uor-r4.attention-audit-measurement-protocol/1".into(),
            execution: release.plan.execution.clone(),
            control_execution: release.plan.control_execution.clone(),
            configs: [
                release.plan.config.clone(),
                release.plan.control_config.clone(),
            ],
            protocol: release.plan.limits.clone(),
            gate_samples_ns: gate_samples,
            restore_samples_ns: vec![relation.restore_ns, control.restore_ns],
            arm_family_elapsed_ns: [relation.family_ns, control.family_ns],
            arm_family_counts: [relation.family_counts, control.family_counts],
            full_family_counts: [relation.full_counts, control.full_counts],
            projected_gate_ns: projected.gate,
            projected_step_bookkeeping_ns: projected.step,
            projected_boundary_bookkeeping_ns: projected.boundary,
            projected_restore_ns: projected.restore,
            bookkeeping: bookkeeping_pin,
            projected_ungated_two_arm_ns: denominator,
            raw_artifacts: raw.clone(),
            shared_setup_ns: setup_ns,
            timer_scope: release.plan.limits.timer_scope.clone(),
            complete_gate_io_included: true,
        };
        let timing_pin = persist(&out.join("timing-protocol.json"), &timing)?;
        let mut calibration = Calibration {
            schema: "uor-r4.attention-audit-calibration/1".into(),
            execution: release.plan.execution.clone(),
            projected_added_gate_ns: projected.total,
            ungated_two_arm_dose_ns: denominator,
            denominator_provenance: "same_hardware_qualified_throughput_extrapolation".into(),
            audits_both_arms: 516,
            complete_gate_io_included: true,
            shared_setup_excluded_from_ratio: true,
            rate_usd_micros_per_hour: release.plan.rate_usd_micros_per_hour,
            gpu_count: release.plan.gpu_count,
            all_in_resource_seconds: release.plan.all_in_resource_seconds,
            staging_archive_reserve_bytes: release.plan.staging_archive_reserve_bytes,
            host_available_bytes: release.plan.host_available_bytes,
            measurement_artifacts: raw,
            measurement_protocol: timing_pin.clone(),
            resource_admission: release.plan.resource_admission.clone(),
        };
        validate_timing_protocol(&calibration)?;
        validate_ratio(
            calibration.projected_added_gate_ns,
            calibration.ungated_two_arm_dose_ns,
        )?;
        let relation_c = persist(&out.join("relation/calibration.json"), &calibration)?;
        calibration.execution = release.plan.control_execution.clone();
        // One protocol binds both exact arm identities, not a renamed result.
        calibration.measurement_protocol = timing_pin;
        validate_timing_protocol(&calibration)?;
        let control_c = persist(&out.join("frozen-zero/calibration.json"), &calibration)?;
        persist(
            &out.join("report.json"),
            &json!({"schema":"uor-r4.attention-audit-finite-qualification-report/1","status":"SOURCE_QUALIFICATION_CANDIDATES_READY_FOR_INDEPENDENT_REVIEW",
            "relation_qualification":relation_q,"control_qualification":control_q,"relation_calibration":relation_c,"control_calibration":control_c,
            "dose_training_release":false,"fresh_evaluations":0,"source_only_instrument_qualification":true}),
        )?;
        Ok(())
    })();
    if let Err(error) = &result {
        let _ = persist(
            &out.join("failure.json"),
            &json!({"status":"FAILED_OR_NOT_RUN","error":error.to_string(),
        "remaining_stages":"NOT_RUN","automatic_retry":false,"dose_qualification":"WITHHELD","mechanism_negative":false}),
        );
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recurring_boundary_writes_are_shared_with_the_measured_fixture() {
        let source = include_str!("attention_audited.rs");
        let production = source.split("fn run(").nth(1).unwrap();
        assert!(production.contains("persist_interval_linkage("));
        assert!(production.contains("persist_progress("));
        assert_eq!(
            source.matches("\"interval-artifacts-{:05}.json\"").count(),
            1
        );
        assert_eq!(source.matches("\"progress-{:05}.json\"").count(), 1);
        let fixture = include_str!("attention_audit_qualification.rs")
            .split("fn measure_bookkeeping(")
            .nth(1)
            .unwrap()
            .split("fn validate_timing")
            .next()
            .unwrap();
        for helper in ["persist_interval_linkage(", "persist_progress(", "settle("] {
            assert!(fixture.contains(helper));
        }
    }
    #[test]
    fn same_source_epoch_zero_probe_cannot_substitute_another_initial_model() {
        let initial = json!({"source_sha256":"a".repeat(64),"epoch":0,"table":[1,2],"routes":[0,1],"gates":[1,0]});
        same_initial_probe_model(&initial, &initial).unwrap();
        for field in ["table", "routes", "gates"] {
            let mut forged = initial.clone();
            forged[field][0] = json!(9);
            assert_eq!(forged["source_sha256"], initial["source_sha256"]);
            assert_eq!(forged["epoch"], initial["epoch"]);
            assert!(same_initial_probe_model(&forged, &initial).is_err());
        }
    }
    #[test]
    fn costed_dose_proxy_rounds_up_and_refuses_zero_or_overflow() {
        assert_eq!(resource_proxy_micros(1_816_334, 3600).unwrap(), 1_816_334);
        assert_eq!(resource_proxy_micros(1, 1).unwrap(), 1);
        assert!(resource_proxy_micros(0, 1).is_err());
        assert!(resource_proxy_micros(1, 0).is_err());
        assert!(resource_proxy_micros(u64::MAX, u64::MAX).is_err());
    }
    #[test]
    fn cost_payload_binds_workload_rate_and_common_stages_without_self_pin_cycle() {
        let p = fixture();
        let mut decision = CostedDoseDecision {
            schema: "uor-r4.attention-audit-costed-dose-decision/1".into(),
            executions: [p.execution, p.control_execution],
            calibration: p.dose_sheet.clone(), resource_admission: p.resource_admission.clone(),
            rate_usd_micros_per_hour: 1_816_334, gpu_count: 2,
            qualified_ungated_two_arm_ns: 100, qualified_added_gate_ns: 10,
            projected_workload_resource_seconds: 1, planned_common_setup_seconds: 60,
            planned_common_custody_archive_close_seconds: 60,
            common_metadata_coverage: "BASELINE_Q1_TERMINAL_METADATA_INCLUDED_IN_PLANNED_COMMON_SETUP_CUSTODY_CLOSE_NOT_MEASURED_RECURRING_GATE".into(),
            all_in_projected_resource_seconds: 121,
            all_in_projected_resource_usd_micros: resource_proxy_micros(1_816_334,121).unwrap(),
            workload_projection_basis: "CONSERVATIVE_SEQUENTIAL_SUM_RESOURCE_TIME_UPPER_BOUND_NOT_MEASURED_PARALLEL_WALLTIME".into(),
            transfer_storage_cost_status: "UNMEASURED_SEPARATE_FROM_RESOURCE_PROXY".into(),
            measured_full_dose: false, invoice: false, budget_is_hard_stop: false,
            owner_cost_approval: p.owner_scope_cost, pm_dispatch: p.pm_dispatch,
        };
        let original = cost_payload_sha256(&decision).unwrap();
        decision.owner_cost_approval.sha256 = "a".repeat(64);
        decision.pm_dispatch.sha256 = "b".repeat(64);
        assert_eq!(cost_payload_sha256(&decision).unwrap(), original);
        for field in [
            "rate_usd_micros_per_hour",
            "qualified_added_gate_ns",
            "planned_common_setup_seconds",
            "planned_common_custody_archive_close_seconds",
        ] {
            let mut changed = serde_json::to_value(&decision).unwrap();
            let value = changed[field].as_u64().unwrap();
            changed[field] = json!(value + 1);
            let changed: CostedDoseDecision = serde_json::from_value(changed).unwrap();
            assert_ne!(cost_payload_sha256(&changed).unwrap(), original);
        }
    }
    #[test]
    fn cost_projection_requires_all_components_and_charges_steps_boundaries_and_tail_restores() {
        let p = project_added_components(1, 2, 3, 4, 5).unwrap();
        assert_eq!(p.total, 516 + 32808 * 2 + 516 * 3 + 514 * (4 + 5));
        for missing in 0..5 {
            let mut inputs = [1; 5];
            inputs[missing] = 0;
            assert!(project_added_components(
                inputs[0], inputs[1], inputs[2], inputs[3], inputs[4]
            )
            .is_err());
        }
        assert!(project_added_components(u64::MAX, 1, 1, 1, 1).is_err());
    }
    #[test]
    fn sensitivity_pair_refuses_empty_duplicate_wrong_role_and_wrong_source() {
        let p = fixture();
        let pin = p.dose_sheet;
        let one = InheritedObservation {
            schema: "uor-r4.attention-audit-inherited-command/1".into(),
            name: "combine_probe".into(),
            execution: p.execution.clone(),
            executable: pin.clone(),
            context_config: pin.clone(),
            producer_command: "propose".into(),
            argv: vec![
                "propose".into(),
                "snapshot".into(),
                "request".into(),
                "gate".into(),
                "0".into(),
            ],
            inputs: vec![pin.clone()],
            native_exit: 0,
            stdout: pin.clone(),
            stderr: pin,
            report: None,
        };
        let mut two = one.clone();
        two.name = "relation_probe".into();
        two.argv[4] = "22".into();
        validate_probe_pair_header(&[one.clone(), two.clone()], &p.execution).unwrap();
        assert!(validate_probe_pair_header(&[], &p.execution).is_err());
        assert!(validate_probe_pair_header(&[one.clone(), one.clone()], &p.execution).is_err());
        let mut wrong = two.clone();
        wrong.name = "unregistered_probe".into();
        assert!(validate_probe_pair_header(&[one.clone(), wrong], &p.execution).is_err());
        wrong = two;
        wrong.execution.executable_sha256 = "f".repeat(64);
        assert!(validate_probe_pair_header(&[one, wrong], &p.execution).is_err());
    }
    #[test]
    fn sensitivity_is_both_named_raw_probes_not_best_candidate_improvement() {
        let execution = fixture().execution;
        let mut losses = vec![u64::MAX; 16];
        losses[..3].copy_from_slice(&[10, 11, 10]);
        let report = json!({"parameter_kind":"gate","parameter_index":0,"epoch":0,"applied":false,"before":10,"after":10,"candidate":0,
            "candidate_losses":losses,"any_candidate_cost_differs_from_identity":true,
            "producer_compiled_identity":{"source_sha256":execution.source_sha256,"kernel_sha256":execution.kernel_sha256,"geometry_sha256":execution.geometry_sha256},
            "producer_driver_source_sha256":execution.driver_source_sha256,"producer_executable_sha256":execution.executable_sha256});
        validate_probe_costs(&report, &execution, 0).unwrap(); // worse-only cost differs; CURRENT remains best
        assert!(validate_probe_costs(&report, &execution, 22).is_err());
        let mut wrong = report.clone();
        wrong["candidate_losses"][1] = json!(10);
        assert!(validate_probe_costs(&wrong, &execution, 0).is_err());
        wrong = report.clone();
        wrong["producer_executable_sha256"] = json!("f".repeat(64));
        assert!(validate_probe_costs(&wrong, &execution, 0).is_err());
        wrong = report;
        wrong["applied"] = json!(true);
        assert!(validate_probe_costs(&wrong, &execution, 0).is_err());
    }
    #[test]
    fn host_bookkeeping_uses_full_receipt_journal_and_both_exact_lifecycle_branches() {
        let directory =
            std::env::temp_dir().join(format!("uor-host-bookkeeping-{}", std::process::id()));
        let execution = fixture().execution;
        let pin = measure_bookkeeping(&execution, &directory).unwrap();
        let record: BookkeepingMeasurement = pinned_json(&pin).unwrap();
        validate_bookkeeping(&record, &execution).unwrap();
        assert_eq!(record.step_samples_ns.len(), 128);
        assert_eq!(record.progress_history_entries, MAX_PROGRESS_HISTORY as u64);
        assert!(record.maximum_progress_shape_bytes > 1_000_000);
        assert!(
            record.maximum_step_shape_bytes > 1000 && record.maximum_boundary_shape_bytes > 16000
        );
        let mut false_record = record.clone();
        false_record.step_samples_ns.pop();
        assert!(validate_bookkeeping(&false_record, &execution).is_err());
        let mut missing_progress = record.clone();
        missing_progress
            .artifacts
            .retain(|p| p.path.file_name().and_then(|v| v.to_str()) != Some("progress.json"));
        assert!(validate_bookkeeping(&missing_progress, &execution).is_err());
        let mut missing_linkage = record.clone();
        missing_linkage.artifacts.retain(|p| {
            p.path.file_name().and_then(|v| v.to_str()) != Some("interval-artifacts.json")
        });
        assert!(validate_bookkeeping(&missing_linkage, &execution).is_err());
        false_record = record;
        false_record.maximum_step_shape_bytes = 1;
        assert!(validate_bookkeeping(&false_record, &execution).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn self_consistent_frozen_relation_update_or_wrong_refusal_is_rejected() {
        let config = fixture().control_config;
        let mut receipt = training::StepReceipt {
            lineage_sha256: "a".repeat(64),
            cursor_before: training::Cursor::initial(),
            cursor_after: training::Cursor {
                completed_batches: 1,
                accepted_updates: 1,
                ..training::Cursor::initial()
            },
            coordinate: training::Coordinate {
                kind: training::CoordinateKind::Gate,
                index: 22,
            },
            target_positions: vec![1],
            candidate: 1,
            loss_before: 2,
            loss_after: 1,
            model_epoch: 1,
            applied: true,
            apply_refused_by_policy: false,
            any_candidate_cost_differs_from_identity: true,
        };
        assert!(validate_receipt_policy(&config, &receipt).is_err());
        receipt.applied = false;
        receipt.apply_refused_by_policy = false;
        assert!(validate_receipt_policy(&config, &receipt).is_err());
        receipt.apply_refused_by_policy = true;
        validate_receipt_policy(&config, &receipt).unwrap();
        let mut learn = config;
        learn.relation_learning_policy = training::RelationLearningPolicy::LearnV1;
        assert!(validate_receipt_policy(&learn, &receipt).is_err());
        receipt.applied = true;
        receipt.apply_refused_by_policy = false;
        receipt.any_candidate_cost_differs_from_identity = false;
        assert!(validate_receipt_policy(&learn, &receipt).is_err());
    }
    #[test]
    fn old_c2_bytes_cannot_be_relabelled_as_current_source_qualification() {
        let execution = fixture().execution;
        let mut report = super::super::tests::canonical_fixture();
        validate_relation_qualification(&report).unwrap();
        assert!(validate_current_relation(&report, &execution).is_err());
        report["producer_compiled_identity"] = json!({"source_sha256":execution.source_sha256,"kernel_sha256":execution.kernel_sha256,"geometry_sha256":execution.geometry_sha256});
        report["producer_driver_source_sha256"] = json!(execution.driver_source_sha256);
        report["producer_executable_sha256"] = json!(execution.executable_sha256);
        report["producer_identity_scope"] =
            json!("actual-executing-canonical-relation-fixture-suite-not-A8-config");
        validate_current_relation(&report, &execution).unwrap();
        report["producer_executable_sha256"] = json!("f".repeat(64));
        assert!(validate_current_relation(&report, &execution).is_err());
    }
    #[test]
    fn repeated_anchor_outputs_ignore_new_file_provenance_but_refuse_numeric_drift() {
        let first = json!({"aggregate":{"scored_rows":3847},"batches":[{"rows":[1,2]}],"actual_snapshot_before":{"hash":"same"},
            "actual_snapshot_after":{"hash":"same"},"evaluated_state":{"epoch":0},"boundary":{"checkpoint_file":{"path":"one.json"}}});
        let mut repeated = first.clone();
        repeated["boundary"]["checkpoint_file"]["path"] = json!("two.json");
        assert!(stable_anchor_equal(&first, &repeated));
        repeated["aggregate"]["scored_rows"] = json!(3846);
        assert!(!stable_anchor_equal(&first, &repeated));
        assert!(!stable_anchor_equal(&json!({}), &json!({})));
    }
    #[test]
    fn device_fault_or_wrong_refusal_never_counts_as_a_guard_pass() {
        assert!(exact_refusal::<()>(Err(Error::Invalid("expected".into())), "expected").is_ok());
        assert!(exact_refusal::<()>(Err(Error::Cuda("fixture fault".into())), "expected").is_err());
        assert!(
            exact_refusal::<()>(Err(Error::Invalid("different failure".into())), "expected")
                .is_err()
        );
        assert!(exact_refusal(Ok(()), "expected").is_err());
    }
    #[test]
    fn pass_label_cannot_turn_arbitrary_raw_bytes_into_qualification() {
        let dir =
            std::env::temp_dir().join(format!("uor-audit-raw-refusal-{}", std::process::id()));
        private_directory(&dir).unwrap();
        let raw = persist(&dir.join("arbitrary.json"), &json!({"status":"PASS"})).unwrap();
        let execution = fixture().execution;
        let stage = StageRecord {
            schema: "uor-r4.attention-audit-qualified-stage/1".into(),
            name: "canonical_initlock_c2".into(),
            execution: execution.clone(),
            status: "PASS".into(),
            observations: vec![raw],
            facts: json!({"controls":9}),
        };
        let pin = persist(&dir.join("stage.json"), &stage).unwrap();
        assert!(validate_stage("canonical_initlock_c2", &execution, &pin).is_err());
        assert!(validate_stage("required_negatives", &execution, &pin).is_err());
        let mut wrong = execution;
        wrong.executable_sha256 = "f".repeat(64);
        assert!(validate_stage("canonical_initlock_c2", &wrong, &pin).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn timing_extrapolation_requires_every_production_family_and_actual_samples() {
        assert!(projected_ns(&[10, 20, 30], &[2, 2, 2], &[100, 200, 300]).is_ok());
        assert!(projected_ns(&[10, 20, 30], &[2, 2, 0], &[100, 200, 300]).is_err());
        assert!(projected_ns(&[10, 0, 30], &[2, 2, 2], &[100, 200, 300]).is_err());
        assert!(projected_ns(&[u64::MAX, 20, 30], &[1, 1, 1], &[u64::MAX, 200, 300]).is_err());
    }
    fn limits() -> Limits {
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
    #[test]
    fn finite_limits_do_not_admit_full_dose_or_zero_restore_estimate() {
        let p = limits();
        p.validate().unwrap();
        let mut bad = p.clone();
        bad.throughput_attempts_per_arm = 16404;
        assert!(bad.validate().is_err());
        bad = p.clone();
        bad.projected_restore_events = 0;
        assert!(bad.validate().is_err());
        bad = p;
        bad.synthetic_accepted_updates = 0;
        assert!(bad.validate().is_err());
    }
    fn fixture() -> QualificationPlan {
        let p = super::super::tests::fixture();
        let mut control = p.config.clone();
        control.relation_learning_policy = training::RelationLearningPolicy::FrozenZeroV1;
        let mut control_execution = p.execution.clone();
        control_execution.config_sha256 = corpus::sha256(&encode(&control).unwrap());
        QualificationPlan {
            schema: PLAN_SCHEMA.into(),
            scope: QUAL_SCOPE.into(),
            status: "HELD_FINITE_QUALIFICATION_OWNER_ACTION_REQUIRED".into(),
            execution: p.execution,
            config: p.config,
            control_execution,
            control_config: control,
            dose_sheet: p.sheet.clone(),
            qualification_fit: p.sheet.clone(),
            owner_scope_cost: p.sheet.clone(),
            pm_dispatch: p.sheet.clone(),
            independent_review: p.sheet.clone(),
            anchor_manifest: p.anchor_manifest,
            anchor_manifest_sha256: p.anchor_manifest_sha256,
            anchor_csv_sha256: p.anchor_csv_sha256,
            anchor_policy: p.anchor_policy,
            limits: limits(),
            inherited_observations: Vec::new(),
            accepted_initlock_disposition: p.sheet.clone(),
            residual_fit: p.sheet.clone(),
            g_init_comparison_or_waiver: p.sheet.clone(),
            resource_admission: p.sheet,
            rate_usd_micros_per_hour: 1_816_334,
            gpu_count: 2,
            all_in_resource_seconds: 3600,
            staging_archive_reserve_bytes: 1024,
            host_available_bytes: 40 * 1024 * 1024 * 1024,
            run_state_directory: p.run_state_directory,
        }
    }
    #[test]
    fn qualifier_has_no_dose_result_prerequisite_and_rejects_dose_scope_or_unreviewed_inputs() {
        let plan = fixture();
        validate_header(&plan).unwrap();
        // No Qualification/Calibration output exists in this valid header.
        let raw = encode(&plan).unwrap();
        let parsed: QualificationPlan = serde_json::from_slice(&raw).unwrap();
        validate_header(&parsed).unwrap();
        let mut old = plan.clone();
        old.schema = super::super::PLAN_SCHEMA.into();
        assert!(validate_header(&old).is_err());
        let mut bad = plan.clone();
        bad.scope = super::super::SCOPE.into();
        assert!(validate_header(&bad).is_err());
        bad = plan.clone();
        bad.control_config.seed = 2;
        assert!(validate_header(&bad).is_err());
        bad = plan.clone();
        bad.execution.driver_source_sha256 = "f".repeat(64);
        assert!(validate_header(&bad).is_err());
        assert!(validate_plan(&plan).is_err()); // missing actual independent FIT/owner/PM/raw artifacts
        let mut extra = serde_json::to_value(&plan).unwrap();
        extra["qualification"] = json!({"status":"PASS"});
        assert!(serde_json::from_value::<QualificationPlan>(extra).is_err());
    }
    #[test]
    fn authorization_payload_binds_cost_limits_inputs_and_both_arms() {
        let p = fixture();
        let expected = payload_sha256(&p).unwrap();
        let mut changed = p.clone();
        changed.rate_usd_micros_per_hour += 1;
        assert_ne!(payload_sha256(&changed).unwrap(), expected);
        changed = p.clone();
        changed.limits.throughput_attempts_per_arm += 1;
        assert_ne!(payload_sha256(&changed).unwrap(), expected);
        changed = p.clone();
        changed.control_config.seed = 2;
        assert_ne!(payload_sha256(&changed).unwrap(), expected);
        changed = p;
        changed
            .inherited_observations
            .push(changed.dose_sheet.clone());
        assert_ne!(payload_sha256(&changed).unwrap(), expected);
    }
    #[test]
    fn finite_timing_schedule_covers_all_registered_read_coordinates() {
        let p = fixture();
        let sixty_four = schedule_counts(&p.config, 64).unwrap();
        assert_eq!(sixty_four[2], 0);
        let complete = schedule_counts(&p.config, 384).unwrap();
        assert_eq!(complete[0], 336);
        assert!(complete[1] > 0 && complete[2] > 0);
        assert_eq!(complete.iter().sum::<u64>(), 384);
        assert_eq!(
            schedule_counts(&p.config, 16404)
                .unwrap()
                .iter()
                .sum::<u64>(),
            16404
        );
    }
}
