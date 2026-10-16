//! Fixed prospective U/R controls. Source delivery is NOT_RUN; neither this
//! entry point nor host metadata tests release CUDA execution. Literal fixtures
//! and typed receipts are shared with the sealed U/D control implementation.
//! Host code prepares storage and checks CUDA observations, never model losses.
use serde_json::{json, Value};

const REFUSALS: [&str; 8] = [
    "route_index_U",
    "vector_index_U",
    "gate_in_route_U",
    "wrong_batch_U",
    "wrong_batch_R",
    "wrong_profiler_U",
    "wrong_profiler_R",
    "wrong_handle_R",
];
const CASES: [&str; 9] = [
    "ineligible",
    "eligible",
    "mixed",
    "tie",
    "virtual_ineligible",
    "route_endpoint",
    "boundary16",
    "boundary17",
    "boundary64",
];

pub fn inventory() -> Value {
    let mut prior = super::counterfactual_controls::inventory();
    if let Some(object) = prior.as_object_mut() {
        for key in [
            "endpoints",
            "endpoint_candidates",
            "endpoint_evaluations",
            "proposal_attempts",
            "successful_proposals",
            "expected_cuda_status8",
            "host_refusal_attempts",
            "host_refusals_per_policy",
        ] {
            object.remove(key);
        }
    }
    json!({"schema":"m1-counterfactual-reuse-controls-fixtures/1",
        "literal_base_fixtures":prior,"modes":["U","R"],
        "case_order":CASES,"boundary_fixtures":[
            {"id":"boundary16","repeat_fixture":"eligible","rows":16},
            {"id":"boundary17","repeat_fixture":"eligible","rows":17},
            {"id":"boundary64","repeat_fixture":"eligible","rows":64}],
        "proposal_attempts":64,"successful_proposals":56,"expected_cuda_status8":8,
        "host_refusals_per_policy":REFUSALS,"host_refusal_attempts":16,
        "endpoint_evaluations":0,"prior_endpoints":"sealed #2557 evidence, not rerun",
        "audit_words":160,"audit_bytes":1280,"shared_bytes":512,
        "producer_rule":"row=lane+16*k; every admitted row exactly once",
        "boundary_identity_rule":"repeat literal row and target without changing within-window identities",
        "seed":1,"seed2":false,"apply_calls":0,"prediction_status":"prospective, not measurements"})
}

#[cfg(any(feature = "cuda", test))]
fn rows_for(id: &str) -> usize {
    match id {
        "boundary16" => 16,
        "boundary17" => 17,
        "boundary64" => 64,
        "mixed" | "virtual_ineligible" => 2,
        _ => 1,
    }
}

#[cfg(any(feature = "cuda", test))]
fn full_mask(rows: usize) -> Result<u64, String> {
    match rows {
        1..=63 => Ok((1u64 << rows) - 1),
        64 => Ok(u64::MAX),
        _ => Err("finite audit rows outside 1..64".into()),
    }
}

#[cfg(any(feature = "cuda", test))]
fn check_audit(raw: &[u64], id: &str) -> Result<Value, String> {
    use uor_r4_discrete::baseline_reuse;
    if !CASES.contains(&id) {
        return Err("unknown finite fixture".into());
    }
    let rows = rows_for(id);
    let audit = baseline_reuse::decode(raw, rows).map_err(|e| e.to_string())?;
    let all = full_mask(rows)?;
    for (row, record) in audit.rows.iter().enumerate() {
        let eligible = id != "ineligible" && !(id == "mixed" && row == 0);
        if record.status != 0 || record.eligible != eligible {
            return Err(
                "baseline producer status/eligibility differs from fixed fixture prediction".into(),
            );
        }
    }
    for lane in &audit.lanes {
        let mut producer = 0u64;
        for row in (lane.candidate..rows).step_by(16) {
            producer |= 1u64 << row;
        }
        let (visited, eligible) = match id {
            "ineligible" => (1, 0),
            "mixed" => (3, 2),
            "virtual_ineligible" if lane.candidate != 0 => (1, 1),
            _ => (all, all),
        };
        if lane.producer_mask != producer
            || lane.producer_call_count != u64::from(producer.count_ones())
            || lane.consumer_visited_mask != visited
            || lane.consumer_eligible_mask != eligible
            || lane.virtual_visited_mask != eligible
            || lane.terminal_status != if id == "ineligible" { 8 } else { 0 }
        {
            return Err("finite producer/consumer audit differs from fixed masks/status".into());
        }
    }
    serde_json::to_value(&audit).map_err(|e| e.to_string())
}

#[cfg(any(feature = "cuda", test))]
fn refusal_check(
    receipt: &super::counterfactual_controls::Receipt,
    id: &str,
) -> Result<(), String> {
    use super::counterfactual_controls::ErrorDetail;
    let (class, message) = match id {
        "route_index_U" | "vector_index_U" => (
            "invalid",
            "invalid discrete model: proposal parameter index exceeds its declared inventory",
        ),
        "gate_in_route_U" => (
            "unimplemented_parameter",
            "parameter kind 3 index 0 is reserved but unimplemented",
        ),
        "wrong_batch_U" | "wrong_batch_R" => (
            "invalid",
            "invalid discrete model: carried batch belongs to another model/source/geometry",
        ),
        "wrong_profiler_U" | "wrong_profiler_R" => (
            "invalid",
            "invalid discrete model: proposal profiler belongs to another model/stream",
        ),
        "wrong_handle_R" => (
            "invalid",
            "invalid discrete model: baseline reuse handle belongs to another model/stream",
        ),
        _ => return Err("unknown finite admission refusal".into()),
    };
    let expected = ErrorDetail {
        class: class.into(),
        message: message.into(),
        site: "proposal".into(),
    };
    let expected_trace = ErrorDetail {
        class: "invalid".into(),
        message: "invalid discrete model: baseline reuse kernel was not submitted".into(),
        site: "raw_trace".into(),
    };
    if receipt.error.as_ref() != Some(&expected)
        || receipt.status.is_some()
        || receipt.losses.is_some()
        || receipt.proposal.is_some()
        || !receipt.readback_errors.is_empty()
        || receipt.raw_trace.is_some()
        || (id.ends_with("_R") && receipt.trace_error.as_ref() != Some(&expected_trace))
        || (!id.ends_with("_R") && receipt.trace_error.is_some())
    {
        return Err("expected exact typed admission refusal and fresh unsubmitted audit guard were not observed".into());
    }
    Ok(())
}

#[cfg(feature = "cuda")]
pub fn run(pin: &super::counterfactual_reuse::Pin) -> Result<(), String> {
    cuda::run(pin)
}

#[cfg(feature = "cuda")]
mod cuda {
    use super::super::{
        counterfactual_controls::{
            self as shared,
            cuda::{inputs, predicted_ties, snapshot, state},
            error_detail, Receipt,
        },
        counterfactual_reuse::{self as command, Pin},
    };
    use super::*;
    use std::path::PathBuf;
    use uor_r4_discrete::{
        baseline_reuse as meta,
        device::{
            compiled_identity, BaselineReuseProfiler, CarriedBatch, Device, ProposeProfiler,
            Snapshot,
        },
        ObjectivePolicy, Parameter,
    };

    struct Log {
        path: PathBuf,
        value: Value,
    }
    impl Log {
        fn save(&self) -> Result<(), String> {
            super::super::write_atomic(
                &self.path,
                &(serde_json::to_string_pretty(&self.value).map_err(|e| e.to_string())? + "\n"),
            )
            .map_err(|e| e.to_string())
        }
        fn push(&mut self, value: Value) -> Result<(), String> {
            self.value["records"]
                .as_array_mut()
                .ok_or("finite records missing")?
                .push(value);
            self.save()
        }
    }
    fn call(
        device: &Device,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profile: &mut ProposeProfiler,
        handle: &mut BaselineReuseProfiler,
        reuse: bool,
    ) -> Receipt {
        let result = if reuse {
            device.propose_reused(batch, targets, parameter, profile, handle)
        } else {
            device.propose_profiled(batch, targets, parameter, profile)
        };
        let mut receipt = Receipt {
            status: profile.counterfactual_status(),
            losses: profile.fault_losses().map(|v| v.to_vec()),
            proposal: None,
            error: None,
            readback_errors: vec![],
            submission_evidence: if profile.counterfactual_status().is_some() {
                "counterfactual_status_read"
            } else {
                "NOT_ESTABLISHED_BY_STATUS"
            }
            .into(),
            raw_trace: None,
            trace_error: None,
        };
        match result {
            Ok(p) => {
                match p.candidate_losses() {
                    Ok(v) => receipt.losses = Some(v),
                    Err(e) => receipt
                        .readback_errors
                        .push(error_detail(&e, "candidate_losses")),
                }
                match p.observation() {
                    Ok(o) => {
                        receipt.proposal = Some([
                            o.candidate,
                            o.before,
                            o.after,
                            o.epoch,
                            u64::from(o.any_candidate_cost_differs_from_identity),
                        ])
                    }
                    Err(e) => receipt
                        .readback_errors
                        .push(error_detail(&e, "observation")),
                }
            }
            Err(e) => receipt.error = Some(error_detail(&e, "proposal")),
        }
        if reuse {
            match handle.raw_trace() {
                Ok(raw) => receipt.raw_trace = Some(raw),
                Err(e) => receipt.trace_error = Some(error_detail(&e, "raw_trace")),
            }
        }
        receipt
    }
    fn resources(
        device: &Device,
        handle: &BaselineReuseProfiler,
        log: &mut Log,
        id: &str,
    ) -> Result<(), String> {
        let u = device
            .ordinary_counterfactual_resources()
            .map_err(|e| e.to_string())?;
        let r = handle.resources().map_err(|e| e.to_string())?;
        let pass = meta::resource_parity(&u, &r);
        log.push(json!({"kind":"resources","fixture":id,"U":u,"R":r,"within_allowance":pass}))?;
        if !pass {
            return Err(
                "BLOCKED: U/R runtime/AOT resource mismatch before finite proposals".into(),
            );
        }
        Ok(())
    }
    fn refusals(
        a: &Device,
        batch_a: &CarriedBatch,
        pa: &mut ProposeProfiler,
        ra: &mut BaselineReuseProfiler,
        base: &Snapshot,
        log: &mut Log,
    ) -> Result<(), String> {
        let b = Device::from_snapshot(base.clone(), 0).map_err(|e| e.to_string())?;
        let (windows, targets) = inputs("tie");
        let batch_b = b.carry_windows(&windows).map_err(|e| e.to_string())?;
        let mut pb = b.proposal_profiler().map_err(|e| e.to_string())?;
        let mut rb = b.baseline_reuse_profiler().map_err(|e| e.to_string())?;
        let before_a = state(a)?;
        let before_b = state(&b)?;
        for id in REFUSALS {
            let receipt = match id {
                "route_index_U" => call(
                    a,
                    batch_a,
                    &targets,
                    Parameter::Route(u32::MAX),
                    pa,
                    ra,
                    false,
                ),
                "vector_index_U" => call(
                    a,
                    batch_a,
                    &targets,
                    Parameter::VectorBlock(u32::MAX),
                    pa,
                    ra,
                    false,
                ),
                "gate_in_route_U" => call(a, batch_a, &targets, Parameter::Gate(0), pa, ra, false),
                "wrong_batch_U" => call(
                    &b,
                    batch_a,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    &mut rb,
                    false,
                ),
                "wrong_batch_R" => call(
                    &b,
                    batch_a,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    &mut rb,
                    true,
                ),
                "wrong_profiler_U" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    pa,
                    &mut rb,
                    false,
                ),
                "wrong_profiler_R" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    pa,
                    &mut rb,
                    true,
                ),
                "wrong_handle_R" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    ra,
                    true,
                ),
                _ => return Err("unreviewed refusal".into()),
            };
            let after_a = state(a);
            let after_b = state(&b);
            let check = refusal_check(&receipt, id);
            log.push(json!({"kind":"host_admission_refusal","id":id,"policy":base.objective_policy,"receipt":receipt,
                "before_A":before_a,"before_B":before_b,"after_A":after_a.as_ref().ok(),"after_B":after_b.as_ref().ok(),
                "state_readback_error_A":after_a.as_ref().err(),"state_readback_error_B":after_b.as_ref().err(),
                "admission_check_error":check.as_ref().err(),"submission_evidence":if check.is_ok(){"exact_typed_API_admission_guard; source_bound_pre_submission"}else{"UNKNOWN"},"cuda_fault_parity_claim":false}))?;
            check?;
            if after_a? != before_a || after_b? != before_b {
                return Err("finite admission refusal changed snapshot/epoch".into());
            }
        }
        Ok(())
    }
    fn execute(pin: &Pin, log: &mut Log) -> Result<(), String> {
        let compiled = compiled_identity();
        command::compiled_matches(pin, &compiled)?;
        log.push(json!({"kind":"compiled_identity","compiled":compiled,"source_commit":pin.source_commit}))?;
        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let mut eligible = Vec::<(Parameter, Receipt)>::new();
            for id in CASES {
                let literal_id = if id.starts_with("boundary") {
                    "eligible"
                } else {
                    id
                };
                let base = snapshot(literal_id, policy)?;
                let fixture_hash =
                    command::digest(&serde_json::to_vec(&base).map_err(|e| e.to_string())?);
                let (mut windows, mut targets) = inputs(literal_id);
                if id.starts_with("boundary") {
                    windows = vec![windows[0].clone(); rows_for(id)];
                    targets = vec![targets[0]; rows_for(id)];
                }
                let input_hash = command::digest(
                    &serde_json::to_vec(&(&windows, &targets)).map_err(|e| e.to_string())?,
                );
                log.push(json!({"kind":"fixture_start","fixture":id,"policy":policy,"fixture_snapshot_sha256":fixture_hash,"input_sha256":input_hash,"rows":windows.len()}))?;
                let device = Device::from_snapshot(base.clone(), 0).map_err(|e| e.to_string())?;
                let mut profile = device.proposal_profiler().map_err(|e| e.to_string())?;
                let mut handle = device
                    .baseline_reuse_profiler()
                    .map_err(|e| e.to_string())?;
                resources(&device, &handle, log, id)?;
                let before = state(&device)?;
                let batch = device.carry_windows(&windows).map_err(|e| e.to_string())?;
                let parameters = if id == "route_endpoint" {
                    vec![Parameter::Route(0)]
                } else if id == "virtual_ineligible" {
                    vec![Parameter::VectorBlock(0)]
                } else {
                    vec![Parameter::Route(0), Parameter::VectorBlock(0)]
                };
                for parameter in parameters {
                    let mut receipts = Vec::new();
                    for reuse in [false, true] {
                        let receipt = call(
                            &device,
                            &batch,
                            &targets,
                            parameter,
                            &mut profile,
                            &mut handle,
                            reuse,
                        );
                        let after = state(&device);
                        // Persist fault-aware raw values and actual state before any interpretation.
                        log.push(json!({"kind":"proposal_attempt","fixture":id,"fixture_snapshot_sha256":fixture_hash,
                            "input_sha256":input_hash,"policy":policy,"parameter":format!("{parameter:?}"),"mode":if reuse{"R"}else{"U"},
                            "before":before,"after":after.as_ref().ok(),"state_readback_error":after.as_ref().err(),"receipt":receipt,"apply_calls":0}))?;
                        if after? != before || before["epoch"] != 0 {
                            return Err("finite snapshot/epoch changed".into());
                        }
                        receipts.push(receipt);
                    }
                    let u = &receipts[0];
                    let r = &receipts[1];
                    shared::receipt_parity(u, r, if id == "ineligible" { 8 } else { 0 })?;
                    let audit =
                        check_audit(r.raw_trace.as_deref().ok_or("missing raw R audit")?, id)?;
                    log.push(json!({"kind":"decoded_control_audit","fixture":id,"policy":policy,"parameter":format!("{parameter:?}"),"audit":audit,"timing_support":false}))?;
                    if id == "eligible" {
                        eligible.push((parameter, u.clone()));
                    }
                    if id == "mixed" {
                        let reference = &eligible
                            .iter()
                            .find(|(p, _)| *p == parameter)
                            .ok_or("eligible reference missing")?
                            .1;
                        if u.losses != reference.losses || u.proposal != reference.proposal {
                            return Err("ineligible row changed CUDA proposal".into());
                        }
                    }
                    if id == "tie" {
                        predicted_ties(u, parameter, policy)?;
                    }
                    if id == "route_endpoint" && u.proposal.ok_or("missing Route proposal")?[4] != 1
                    {
                        return Err("Route sensitivity prerequisite failed".into());
                    }
                    log.push(json!({"kind":"control_check","fixture":id,"policy":policy,"parameter":format!("{parameter:?}"),"result":"PASS"}))?;
                }
                if id == "tie" {
                    refusals(&device, &batch, &mut profile, &mut handle, &base, log)?;
                }
            }
        }
        let records = log.value["records"].as_array().ok_or("missing records")?;
        let count = |kind: &str| records.iter().filter(|r| r["kind"] == kind).count();
        let success = records
            .iter()
            .filter(|r| r["kind"] == "proposal_attempt" && r["receipt"]["status"] == 0)
            .count();
        let status8 = records
            .iter()
            .filter(|r| r["kind"] == "proposal_attempt" && r["receipt"]["status"] == 8)
            .count();
        let actual = json!({"proposal_attempts":count("proposal_attempt"),"successful_proposals":success,"expected_cuda_status8":status8,"host_refusal_attempts":count("host_admission_refusal"),"endpoint_evaluations":0});
        log.push(json!({"kind":"executed_inventory","actual":actual}))?;
        if actual
            != json!({"proposal_attempts":64,"successful_proposals":56,"expected_cuda_status8":8,"host_refusal_attempts":16,"endpoint_evaluations":0})
        {
            return Err("finite executed dose differs from FIT inventory".into());
        }
        Ok(())
    }
    pub(super) fn run(pin: &Pin) -> Result<(), String> {
        command::validate(pin, true)?;
        if pin.phase != "finite" {
            return Err("finite command refuses nonfinite phase".into());
        }
        command::compiled_matches(pin, &compiled_identity())?;
        std::fs::create_dir(&pin.out_dir)
            .map_err(|e| format!("new finite output directory required: {e}"))?;
        let mut contract = command::plan(pin);
        if let Some(object) = contract.as_object_mut() {
            for field in [
                "status",
                "model_compute_executed",
                "cuda_context_created",
                "execution",
            ] {
                object.remove(field);
            }
        }
        let mut log = Log {
            path: pin
                .out_dir
                .join("counterfactual-reuse-controls-record.json"),
            value: json!({"schema":"m1-counterfactual-reuse-controls-record/1","status":"STARTED","execution_contract":contract,"inventory":inventory(),"records":[],"apply_calls":0,"training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}),
        };
        log.save()?;
        match execute(pin, &mut log) {
            Ok(()) => {
                log.value["status"] = json!("FINITE_CONTROL_PASS");
                log.value["timing_execution_released"] = json!(false);
                log.save()
            }
            Err(error) => {
                log.value["status"] = json!("BLOCKED_INSTRUMENT_OR_FIXTURE");
                log.value["error"] = json!(error);
                log.save()
                    .map_err(|save| format!("{error}; partial receipt write failed: {save}"))?;
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::counterfactual_controls::{ErrorDetail, Receipt};
    use super::*;
    #[test]
    fn fixed_inventory_excludes_endpoint_rerun_and_seed2() {
        let value = inventory();
        assert_eq!(value["proposal_attempts"], 64);
        assert_eq!(value["successful_proposals"], 56);
        assert_eq!(value["expected_cuda_status8"], 8);
        assert_eq!(value["host_refusal_attempts"], 16);
        assert_eq!(value["endpoint_evaluations"], 0);
        assert_eq!(value["seed2"], false);
        assert_eq!(rows_for("boundary16"), 16);
        assert_eq!(rows_for("boundary17"), 17);
        assert_eq!(full_mask(64).unwrap(), u64::MAX);
        assert!(full_mask(0).is_err());
        assert!(full_mask(65).is_err());
    }
    #[test]
    fn refusal_must_be_typed_and_reject_stale_audit() {
        let mut receipt = Receipt {
            status: None,
            losses: None,
            proposal: None,
            error: Some(ErrorDetail {
                class: "invalid".into(),
                message:
                    "invalid discrete model: baseline reuse handle belongs to another model/stream"
                        .into(),
                site: "proposal".into(),
            }),
            readback_errors: vec![],
            submission_evidence: "NOT_ESTABLISHED_BY_STATUS".into(),
            raw_trace: None,
            trace_error: Some(ErrorDetail {
                class: "invalid".into(),
                message: "invalid discrete model: baseline reuse kernel was not submitted".into(),
                site: "raw_trace".into(),
            }),
        };
        assert!(refusal_check(&receipt, "wrong_handle_R").is_ok());
        receipt.raw_trace = Some(vec![0; 160]);
        assert!(refusal_check(&receipt, "wrong_handle_R").is_err());
        receipt.raw_trace = None;
        receipt.error.as_mut().unwrap().class = "cuda".into();
        assert!(refusal_check(&receipt, "wrong_handle_R").is_err());
    }
    #[test]
    fn virtual_early_exit_retains_complete_producers_and_short_consumers() {
        // Literal audit storage, no writer/reference evaluation. Producers
        // publish both rows even when a virtual consumer stops after row zero.
        let mut raw = vec![0u64; 160];
        raw[0] = 1u64 << 32;
        raw[1] = 1u64 << 32;
        for lane in 0..16 {
            let start = 64 + 6 * lane;
            raw[start] = match lane {
                0 => 1,
                1 => 2,
                _ => 0,
            };
            raw[start + 1] = u64::from(lane < 2);
            let consumed = if lane == 0 { 3 } else { 1 };
            raw[start + 2] = consumed;
            raw[start + 3] = consumed;
            raw[start + 4] = consumed;
        }
        assert!(check_audit(&raw, "virtual_ineligible").is_ok());
        let mut stale = raw.clone();
        stale[2] = 1u64 << 32;
        assert!(check_audit(&stale, "virtual_ineligible").is_err());
        let mut missing = raw.clone();
        missing[64 + 6] = 0;
        assert!(check_audit(&missing, "virtual_ineligible").is_err());
        raw[64 + 6 + 2] = 3;
        raw[64 + 6 + 3] = 3;
        raw[64 + 6 + 4] = 3;
        assert!(check_audit(&raw, "virtual_ineligible").is_err());
    }
    #[test]
    fn full_mask_boundary_observation_never_shifts_by64() {
        let mut raw = vec![0u64; 160];
        for slot in &mut raw[..64] {
            *slot = 1u64 << 32;
        }
        for lane in 0..16 {
            let start = 64 + 6 * lane;
            raw[start] = (1u64 << lane)
                | (1u64 << (lane + 16))
                | (1u64 << (lane + 32))
                | (1u64 << (lane + 48));
            raw[start + 1] = 4;
            raw[start + 2] = u64::MAX;
            raw[start + 3] = u64::MAX;
            raw[start + 4] = u64::MAX;
        }
        assert!(check_audit(&raw, "boundary64").is_ok());
        raw[64 + 2] = u64::MAX - 1;
        assert!(check_audit(&raw, "boundary64").is_err());
    }
    #[test]
    fn receipt_parity_rejects_loss_proposal_and_readback_drift() {
        use super::super::counterfactual_controls::receipt_parity;
        let u = Receipt {
            status: Some(0),
            losses: Some(vec![7; 16]),
            proposal: Some([0, 7, 7, 0, 0]),
            error: None,
            readback_errors: vec![],
            submission_evidence: "counterfactual_status_read".into(),
            raw_trace: None,
            trace_error: None,
        };
        let mut r = u.clone();
        r.raw_trace = Some(vec![0; 160]);
        assert!(receipt_parity(&u, &r, 0).is_ok());
        r.losses.as_mut().unwrap()[15] = 8;
        assert!(receipt_parity(&u, &r, 0).is_err());
        r.losses = u.losses.clone();
        r.proposal.as_mut().unwrap()[3] = 1;
        assert!(receipt_parity(&u, &r, 0).is_err());
        r.proposal = u.proposal;
        r.readback_errors.push(ErrorDetail {
            class: "cuda".into(),
            message: "readback failure".into(),
            site: "observation".into(),
        });
        assert!(receipt_parity(&u, &r, 0).is_err());
    }
}
