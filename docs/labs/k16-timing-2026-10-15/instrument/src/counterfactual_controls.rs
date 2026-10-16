//! Prospective finite controls, NOT_RUN at source delivery. No execution release
//! follows from this command's existence, a host check, or a source-only FIT.
//! Literal geometry/endpoint recipes are copied from the source-bound C-R3/C-R4
//! runner at c461350058e6ea1b188a60944da964982f272b60:
//! docs/labs/admission-rank-fixture-runner-2026-10-15/instrument/src/fixtures.rs.
//! Host code constructs storage and compares CUDA observations; it never scores,
//! routes, chooses a model candidate, applies a move, or runs a reference model.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;

const SCHEMA: &str = "m1-counterfactual-controls-pin/1";
const PENDING: &str = "PENDING";
const PROVENANCE: &str =
    "c461350058e6ea1b188a60944da964982f272b60:C-R3/C-R4 literal fixture recipes";
const GEOMETRY: &str = "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327";
const ROOTS: [[i32; 8]; 16] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
];
// Storage recipes, not a host action implementation. The source runner uses
// these exact literals for independent materialized CUDA endpoint admission.
const ROOT0_IMAGES: [usize; 16] = [0, 14, 12, 4, 10, 8, 10, 2, 5, 6, 8, 14, 13, 6, 12, 2];
const REFUSALS: [&str; 8] = [
    "route_index_U",
    "vector_index_U",
    "gate_in_route_U",
    "wrong_batch_U",
    "wrong_batch_D",
    "wrong_profiler_U",
    "wrong_profiler_D",
    "wrong_probe_D",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub schema: String,
    /// source_template allows only PENDING identity fields, never execution.
    pub stage: String,
    pub source_commit: String,
    pub standalone_source_sha256: String,
    pub binary_sha256: String,
    pub source_sha256: String,
    pub ordinary_kernel_sha256: String,
    pub decomposition_kernel_sha256: String,
    pub geometry_sha256: String,
    pub fixture_inventory_sha256: String,
    pub seed: u32,
    pub evaluation_only: bool,
    pub training_resume: bool,
    /// Reviewed external authority references, not authority minted by a CLI.
    pub test_fitness_reference: Option<String>,
    pub execution_release_reference: Option<String>,
    pub out_dir: PathBuf,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_hex(value: &str, n: usize) -> bool {
    value.len() == n
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && value.bytes().any(|b| b != b'0')
}

/// Framed ordered embedded source inventory. This is independent of the crate
/// composite source identity, which excludes this standalone executable.
fn standalone_source_hash() -> String {
    let mut hash = Sha256::new();
    hash.update(b"m1-counterfactual-controls-standalone-source/1");
    for (name, bytes) in [
        ("Cargo.toml", include_bytes!("../Cargo.toml").as_slice()),
        ("Cargo.lock", include_bytes!("../Cargo.lock").as_slice()),
        ("src/main.rs", include_bytes!("main.rs").as_slice()),
        (
            "src/checkpoint_bridge.rs",
            include_bytes!("checkpoint_bridge.rs").as_slice(),
        ),
        (
            "src/counterfactual_decomposition.rs",
            include_bytes!("counterfactual_decomposition.rs").as_slice(),
        ),
        (
            "src/counterfactual_controls.rs",
            include_bytes!("counterfactual_controls.rs").as_slice(),
        ),
    ] {
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}

fn inventory() -> Value {
    json!({"schema":"m1-counterfactual-controls-fixtures/1","provenance":PROVENANCE,
        "roots":ROOTS,"root0_candidate_images":ROOT0_IMAGES,
        "policies":["nearest_address_margin_v1","occupied_cell_rank_hinge_v1"],
        "cases":[
            {"id":"ineligible","shape":[2,2,1,1],"tokens":[[0,0],[15,15]],"windows":[[0]],"targets":[0],"parameters":["Route(0)","VectorBlock(0)"],"expected_status":8},
            {"id":"eligible","shape":[2,2,1,1],"tokens":[[0,0],[15,15]],"windows":[[0]],"targets":[1],"parameters":["Route(0)","VectorBlock(0)"],"expected_status":0},
            {"id":"mixed","shape":[2,2,1,1],"tokens":[[0,0],[15,15]],"windows":[[0],[0]],"targets":[0,1],"parameters":["Route(0)","VectorBlock(0)"],"expected_status":0},
            {"id":"tie","shape":[2,2,1,2],"tokens":[[0,0],[3,0]],"windows":[[1]],"targets":[0],"parameters":["Route(0)","VectorBlock(0)"],"expected_status":0},
            {"id":"virtual_ineligible","shape":[2,2,1,1],"tokens":[[0,0],[0,0]],"windows":[[0],[0]],"targets":[0,0],"parameters":["VectorBlock(0)"],"expected_status":0},
            {"id":"route_endpoint","shape":[2,2,2,2],"tokens":[[3,0],[2,0]],"routes":[0,0],"windows":[[0,1]],"targets":[0],"parameters":["Route(0)"],"expected_status":0,"frontier_entries":2}],
        "identities":{"occurrence":"one-based position within window","version":1,"record":1},
        "host_refusals_per_policy":REFUSALS,"proposal_attempts":40,"expected_cuda_status8":8,
        "successful_proposals":32,"endpoint_evaluations":128,"host_refusal_attempts":16,
        "endpoints":[["eligible","VectorBlock(0)"],["tie","VectorBlock(0)"],["virtual_ineligible","VectorBlock(0)"],["route_endpoint","Route(0)"]],
        "endpoint_candidates":16,"seed":1,"apply_calls":0,"seed2":false,
        "manual_predictions":{"not_measured":true,"tie_vector_winner":3,"tie_vector_equal_best":[3,12],"tie_route_winner":0,
            "virtual_ineligible_identity_two_row_loss":256,"virtual_ineligible_identity_one_row_loss":128,
            "virtual_ineligible_nonidentity_loss":"MAX; virtual token_rivals0; not an arithmetic fault"}})
}
fn inventory_hash() -> Result<String, String> {
    serde_json::to_vec(&inventory())
        .map(|bytes| digest(&bytes))
        .map_err(|e| e.to_string())
}
fn validate(pin: &Pin, execution: bool) -> Result<(), String> {
    if pin.schema != SCHEMA
        || !matches!(pin.stage.as_str(), "source_template" | "execution_pinned")
        || pin.seed != 1
        || !pin.evaluation_only
        || pin.training_resume
        || pin.geometry_sha256 != GEOMETRY
        || pin.fixture_inventory_sha256 != inventory_hash()?
        || pin.out_dir.as_os_str().is_empty()
    {
        return Err(
            "controls pin scope/inventory refused; Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE"
                .into(),
        );
    }
    let template = pin.stage == "source_template";
    for (name, value, size) in [
        ("source_commit", &pin.source_commit, 40),
        (
            "standalone_source_sha256",
            &pin.standalone_source_sha256,
            64,
        ),
        ("binary_sha256", &pin.binary_sha256, 64),
        ("source_sha256", &pin.source_sha256, 64),
        ("ordinary_kernel_sha256", &pin.ordinary_kernel_sha256, 64),
        (
            "decomposition_kernel_sha256",
            &pin.decomposition_kernel_sha256,
            64,
        ),
    ] {
        if !(valid_hex(value, size) || template && value == PENDING) {
            return Err(format!("controls pin invalid {name}"));
        }
    }
    if execution {
        let authority = |r: &Option<String>| {
            r.as_deref().is_some_and(|s| {
                s.starts_with("https://github.com/UOR-Foundation/uor-r4/") && s.contains('#')
            })
        };
        if template
            || !authority(&pin.test_fitness_reference)
            || !authority(&pin.execution_release_reference)
        {
            return Err("controls execution NOT_RELEASED: exact pin, independent FIT and separate release references required; references remain externally reviewed authority".into());
        }
        if pin.standalone_source_sha256 != standalone_source_hash() {
            return Err("controls standalone source identity differs".into());
        }
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        if digest(&std::fs::read(exe).map_err(|e| e.to_string())?) != pin.binary_sha256 {
            return Err("controls running binary SHA256 differs".into());
        }
    }
    Ok(())
}
fn plan(pin: &Pin) -> Value {
    json!({"schema":"m1-counterfactual-controls-plan/1","status":"NOT_RUN","execution":"NOT_RELEASED_BY_THIS_PLAN",
        "pin":pin,"standalone_source_sha256":standalone_source_hash(),"fixture_inventory_sha256":pin.fixture_inventory_sha256,
        "inventory":inventory(),"model_compute_executed":false,"cuda_context_created":false,"apply_calls":0,
        "diagnostic_dose":"unchanged separate32-proposal diagnostic; not executed by controls",
        "unreachable":{"cuda2":"opaque admitted BinderPlan and device-validated routes; not injected",
            "cuda4":"counterfactual Gate index branch excluded by admitted Route/VectorBlock parameter API",
            "cuda9":"mode/parameter mismatch refused before counterfactual submission",
            "cuda12":"admitted exact root/dimension bounds: lossTR<=37760,row sum<=2416640; unrelated aggregate overflow is not U/D parity",
            "stale_epoch":"opaque same-model batch epoch cannot become stale with zeroapply; NOT_EXECUTED",
            "stale_proposal_cuda3":"apply-only branch; NOT_EXECUTED"},
        "claims":"finite exact-pair qualification only; no H1/H2/H3, rate, language, training or mechanism result",
        "training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"})
}

fn template(out_dir: PathBuf) -> Result<Pin, String> {
    Ok(Pin {
        schema: SCHEMA.into(),
        stage: "source_template".into(),
        source_commit: PENDING.into(),
        standalone_source_sha256: standalone_source_hash(),
        binary_sha256: PENDING.into(),
        source_sha256: PENDING.into(),
        ordinary_kernel_sha256: PENDING.into(),
        decomposition_kernel_sha256: PENDING.into(),
        geometry_sha256: GEOMETRY.into(),
        fixture_inventory_sha256: inventory_hash()?,
        seed: 1,
        evaluation_only: true,
        training_resume: false,
        test_fitness_reference: None,
        execution_release_reference: None,
        out_dir,
    })
}

pub fn entry(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or_default();
    if command == "decompose-controls-template" && args.len() == 3 && args[1] == "--out-dir" {
        let pin = template(PathBuf::from(&args[2]))?;
        validate(&pin, false)?;
        println!(
            "{}",
            serde_json::to_string_pretty(&pin).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if command == "decompose-controls-identity" && args.len() == 1 {
        let binary = digest(
            &std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?,
        );
        #[cfg(feature = "cuda")]
        let compiled = uor_r4_discrete::device::compiled_identity();
        #[cfg(not(feature = "cuda"))]
        let compiled = json!({"status":"UNAVAILABLE_WITHOUT_CUDA_BUILD"});
        println!(
            "{}",
            json!({"standalone_source_sha256":standalone_source_hash(),"binary_sha256":binary,
            "fixture_inventory_sha256":inventory_hash()?,"compiled":compiled,"cuda_context_created":false,"model_compute_executed":false})
        );
        return Ok(());
    }
    if !matches!(
        command,
        "decompose-controls" | "decompose-controls-check" | "decompose-controls-plan"
    ) || args.len() != 3
        || args[1] != "--pin"
    {
        return Err("decompose-controls-check|decompose-controls-plan|decompose-controls --pin FILE; decompose-controls-identity; decompose-controls-template --out-dir NEW_DIR".into());
    }
    let pin: Pin = serde_json::from_slice(&std::fs::read(&args[2]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate(&pin, false)?;
    if command != "decompose-controls" {
        println!(
            "{}",
            serde_json::to_string_pretty(&plan(&pin)).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    #[cfg(feature = "cuda")]
    {
        validate(&pin, true)?;
        cuda::run(&pin)
    }
    #[cfg(not(feature = "cuda"))]
    {
        Err(
            "controls execution requires CUDA build; no CPU model fallback; source remains NOT_RUN"
                .into(),
        )
    }
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Receipt {
    status: Option<u32>,
    losses: Option<Vec<u64>>,
    proposal: Option<[u64; 5]>,
    error: Option<ErrorDetail>,
    readback_errors: Vec<ErrorDetail>,
    submission_evidence: String,
    raw_trace: Option<Vec<u64>>,
    trace_error: Option<ErrorDetail>,
}
#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct ErrorDetail {
    class: String,
    message: String,
    site: String,
}
#[cfg(any(feature = "cuda", test))]
fn error_detail(error: &uor_r4_discrete::Error, site: &str) -> ErrorDetail {
    use uor_r4_discrete::Error;
    ErrorDetail {
        class: match error {
            Error::Invalid(_) => "invalid",
            Error::Cuda(_) => "cuda",
            Error::TrainingHeld => "training_held",
            Error::UnimplementedParameter { .. } => "unimplemented_parameter",
        }
        .into(),
        message: error.to_string(),
        site: site.into(),
    }
}
#[cfg(any(feature = "cuda", test))]
fn host_refusal_check(receipt: &Receipt, id: &str) -> Result<(), String> {
    let (class,message)=match id {
        "route_index_U"|"vector_index_U"=>("invalid","invalid discrete model: proposal parameter index exceeds its declared inventory"),
        "gate_in_route_U"=>("unimplemented_parameter","parameter kind 3 index 0 is reserved but unimplemented"),
        "wrong_batch_U"|"wrong_batch_D"=>("invalid","invalid discrete model: carried batch belongs to another model/source/geometry"),
        "wrong_profiler_U"=>("invalid","invalid discrete model: proposal profiler belongs to another model/stream"),
        "wrong_profiler_D"|"wrong_probe_D"=>("invalid","invalid discrete model: decomposition requires same model/stream and Route/VectorBlock parameter"),
        _=>return Err("unknown expected host refusal".into()),
    };
    let expected = ErrorDetail {
        class: class.into(),
        message: message.into(),
        site: "proposal".into(),
    };
    let expected_trace = ErrorDetail {
        class: "invalid".into(),
        message: "invalid discrete model: diagnostic kernel was not submitted".into(),
        site: "raw_trace".into(),
    };
    if receipt.error.as_ref() != Some(&expected)
        || !receipt.readback_errors.is_empty()
        || receipt.status.is_some()
        || receipt.losses.is_some()
        || receipt.proposal.is_some()
        || receipt.raw_trace.is_some()
        || (id.ends_with("_D") && receipt.trace_error.as_ref() != Some(&expected_trace))
        || (!id.ends_with("_D") && receipt.trace_error.is_some())
    {
        return Err(
            "expected exact typed admission refusal/unsubmitted trace guard was not observed"
                .into(),
        );
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn receipt_parity(u: &Receipt, d: &Receipt, status: u32) -> Result<(), String> {
    if u.status != Some(status)
        || d.status != Some(status)
        || u.losses != d.losses
        || u.losses.as_ref().is_none_or(|v| v.len() != 16)
        || u.proposal != d.proposal
    {
        return Err("U/D status/loss/proposal parity failed".into());
    }
    if status == 0 {
        if u.error.is_some()
            || d.error.is_some()
            || !u.readback_errors.is_empty()
            || !d.readback_errors.is_empty()
        {
            return Err("successful proposal returned error".into());
        }
        let p = u.proposal.ok_or("missing successful proposal")?;
        let record = uor_r4_discrete::decomposition::ProposalReceipt {
            losses: u.losses.clone().ok_or("missing losses")?,
            status,
            proposal: p,
        };
        record.validate().map_err(|e| e.to_string())?;
    } else if status != 8
        || u.proposal.is_some()
        || u.error.is_none()
        || d.error.is_none()
        || !u.readback_errors.is_empty()
        || !d.readback_errors.is_empty()
        || u.losses
            .as_ref()
            .is_none_or(|v| v.iter().any(|&x| x != u64::MAX))
    {
        return Err("expected status8 refusal receipt incomplete".into());
    }
    if status == 8 {
        let expected = error_detail(
            &uor_r4_discrete::Error::Invalid(
                "discrete CUDA counterfactual refused with status 8".into(),
            ),
            "proposal",
        );
        if u.error.as_ref() != Some(&expected) || d.error.as_ref() != Some(&expected) {
            return Err("status8 call returned an unexpected error class/site/message".into());
        }
    }
    if u.raw_trace.is_some()
        || u.trace_error.is_some()
        || d.raw_trace.is_none()
        || d.trace_error.is_some()
    {
        return Err("U/D trace availability differs from submitted modes".into());
    }
    Ok(())
}

#[cfg(feature = "cuda")]
mod cuda {
    use super::*;
    use uor_r4_discrete::{
        decomposition as meta,
        device::{
            compiled_identity, CarriedBatch, DecompositionProfiler, Device, ProposeProfiler,
            Snapshot,
        },
        Identity, ObjectivePolicy, Parameter, ReadSpec, Spec, SCHEMA as SNAPSHOT_SCHEMA,
    };

    const OPERATOR: &str =
        "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b";
    const COMPOSITION: &str =
        "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0";
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
                .ok_or("record inventory missing")?
                .push(value);
            self.save()
        }
    }
    fn field(v: &Value, key: &str) -> Result<String, String> {
        v[key]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("missing compiled {key}"))
    }
    fn snapshot(id: &str, policy: ObjectivePolicy) -> Result<Snapshot, String> {
        let (blocks, context, k, roots) = if id == "route_endpoint" {
            (2, 2, 2, vec![vec![3, 0], vec![2, 0]])
        } else if id == "tie" {
            (2, 1, 2, vec![vec![0, 0], vec![3, 0]])
        } else if id == "virtual_ineligible" {
            (2, 1, 1, vec![vec![0, 0], vec![0, 0]])
        } else {
            (2, 1, 1, vec![vec![0, 0], vec![15, 15]])
        };
        let table = roots
            .iter()
            .flat_map(|t| t.iter().flat_map(|&r| ROOTS[r]))
            .collect();
        let stages = 1;
        let compiled = compiled_identity();
        Ok(Snapshot {
            schema: SNAPSHOT_SCHEMA.into(),
            spec: Spec {
                vocabulary: roots.len() as u32,
                blocks,
                context,
                kept_cells: k,
            },
            read_spec: ReadSpec::Route,
            objective_policy: policy,
            seed: 1,
            epoch: 0,
            table,
            routes: vec![0; (blocks * stages) as usize],
            gates: vec![0; (blocks * stages) as usize],
            operator_kappa: OPERATOR.into(),
            composition_kappa: COMPOSITION.into(),
            kernel_sha256: field(&compiled, "kernel_sha256")?,
            source_sha256: field(&compiled, "source_sha256")?,
            geometry_sha256: field(&compiled, "geometry_sha256")?,
        })
    }
    fn inputs(id: &str) -> (Vec<Vec<Identity>>, Vec<u32>) {
        let tokens = match id {
            "mixed" | "virtual_ineligible" => vec![vec![0], vec![0]],
            "tie" => vec![vec![1]],
            "route_endpoint" => vec![vec![0, 1]],
            _ => vec![vec![0]],
        };
        let targets = match id {
            "ineligible" => vec![0],
            "mixed" => vec![0, 1],
            "virtual_ineligible" => vec![0, 0],
            "tie" | "route_endpoint" => vec![0],
            _ => vec![1],
        };
        (
            tokens
                .into_iter()
                .map(|w| {
                    w.into_iter()
                        .enumerate()
                        .map(|(i, token)| Identity {
                            token,
                            occurrence: i as u64 + 1,
                            version: 1,
                            record: 1,
                        })
                        .collect()
                })
                .collect(),
            targets,
        )
    }
    fn state(device: &Device) -> Result<Value, String> {
        let bytes = serde_json::to_vec(&device.snapshot().map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        Ok(
            json!({"snapshot_sha256":digest(&bytes),"epoch":device.epoch().map_err(|e|e.to_string())?}),
        )
    }
    fn resources(
        device: &Device,
        probe: &DecompositionProfiler,
        log: &mut Log,
        id: &str,
    ) -> Result<(), String> {
        let u = device
            .ordinary_counterfactual_resources()
            .map_err(|e| e.to_string())?;
        let d = probe.resources().map_err(|e| e.to_string())?;
        let pass = meta::resource_parity(&u, &d);
        log.push(json!({"kind":"resources","fixture":id,"U":u,"D":d,"within_allowance":pass}))?;
        if !pass {
            return Err("BLOCKED: runtime/AOT resource mismatch before control proposals".into());
        }
        Ok(())
    }
    // Capture all storage before interpreting it. A refusal never inherits a
    // previous D trace: propose_decomposed resets submitted before admission.
    fn call(
        device: &Device,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profile: &mut ProposeProfiler,
        probe: &mut DecompositionProfiler,
        diagnostic: bool,
    ) -> Receipt {
        let result = if diagnostic {
            device.propose_decomposed(batch, targets, parameter, profile, probe)
        } else {
            device.propose_profiled(batch, targets, parameter, profile)
        };
        let mut receipt = Receipt {
            status: profile.counterfactual_status(),
            losses: profile.fault_losses().map(|x| x.to_vec()),
            proposal: None,
            error: None,
            readback_errors: Vec::new(),
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
        if diagnostic {
            match probe.raw_trace() {
                Ok(v) => receipt.raw_trace = Some(v),
                Err(e) => receipt.trace_error = Some(error_detail(&e, "raw_trace")),
            }
        }
        receipt
    }
    fn trace(receipt: &Receipt, id: &str) -> Result<Vec<meta::LaneRecord>, String> {
        let lanes = meta::decode(receipt.raw_trace.as_deref().ok_or("missing raw D words")?)
            .map_err(|e| e.to_string())?;
        let visited = if id == "mixed" { 3 } else { 1 };
        let eligible = match id {
            "ineligible" => 0,
            "mixed" => 2,
            _ => 1,
        };
        let code = if id == "ineligible" { 8 } else { 0 };
        if lanes.iter().any(|l| {
            let (visited, eligible) = if id == "virtual_ineligible" {
                if l.candidate == 0 {
                    (3, 3)
                } else {
                    (1, 1)
                }
            } else {
                (visited, eligible)
            };
            l.fault_code != code
                || l.visited_mask != visited
                || l.eligible_mask != eligible
                || (id == "virtual_ineligible"
                    && (l.words[13] != l.visited_rows || l.words[14] != l.visited_rows))
        }) {
            return Err(
                "D terminal fault/visited/eligible masks differ from prospective fixture".into(),
            );
        }
        Ok(lanes)
    }
    fn materialized(
        base: &Snapshot,
        parameter: Parameter,
        candidate: usize,
    ) -> Result<Snapshot, String> {
        let mut endpoint = base.clone();
        match parameter {
            Parameter::Route(0) => endpoint.routes[0] = candidate as u32,
            Parameter::VectorBlock(0) => {
                endpoint.table[..8].copy_from_slice(&ROOTS[ROOT0_IMAGES[candidate]])
            }
            _ => return Err("endpoint inventory contains unreviewed parameter".into()),
        }
        Ok(endpoint)
    }
    fn endpoints(
        base: &Snapshot,
        id: &str,
        parameter: Parameter,
        receipt: &Receipt,
        log: &mut Log,
    ) -> Result<(), String> {
        let losses = receipt
            .losses
            .as_ref()
            .ok_or("endpoint reference losses missing")?;
        let (mut windows, mut targets) = inputs(id);
        // Materialized endpoints are one row even when the proposal fixture
        // has two rows. No host reduction of model scores is performed.
        windows.truncate(1);
        targets.truncate(1);
        let endpoint_input_sha256 =
            digest(&serde_json::to_vec(&(&windows, &targets)).map_err(|e| e.to_string())?);
        for (candidate, &expected) in losses.iter().enumerate() {
            let endpoint = materialized(base, parameter, candidate)?;
            let endpoint_hash = digest(&serde_json::to_vec(&endpoint).map_err(|e| e.to_string())?);
            let device = Device::from_snapshot(endpoint, 0).map_err(|e| e.to_string())?;
            let before = state(&device)?;
            let result = (|| {
                let carried = device.carry_windows(&windows).map_err(|e| e.to_string())?;
                device
                    .evaluate(&carried, &targets)
                    .map_err(|e| e.to_string())
            })();
            let after = state(&device);
            log.push(json!({"kind":"materialized_endpoint","fixture":id,"policy":base.objective_policy,
                "parameter":format!("{parameter:?}"),"candidate":candidate,"literal_snapshot_sha256":endpoint_hash,
                "input_sha256":endpoint_input_sha256,
                "before":before,"after":after.as_ref().ok(),"state_readback_error":after.as_ref().err(),
                "expected_virtual_loss":expected,"evaluation":result.as_ref().ok(),"error":result.as_ref().err(),"apply_calls":0}))?;
            if after? != before {
                return Err("endpoint snapshot or epoch changed".into());
            }
            let evaluation = result?;
            if id == "virtual_ineligible" {
                if evaluation.rows.len() != 1
                    || (candidate == 0
                        && (evaluation.rows[0].token_rivals == 0
                            || evaluation.aggregate.objective_loss_numerator_sum != 128
                            || expected != 256))
                    || (candidate != 0
                        && (evaluation.rows[0].token_rivals != 0 || expected != u64::MAX))
                {
                    return Err("virtual-ineligible endpoint failed prospective eligibility/MAX prediction; finite absent-rival score is not compared toMAX".into());
                }
            } else if evaluation.rows.len() != 1
                || evaluation.rows[0].token_rivals == 0
                || evaluation.aggregate.objective_loss_numerator_sum != expected
            {
                return Err("BLOCKED: materialized GPU endpoint is ineligible or disagrees with virtual loss".into());
            }
        }
        Ok(())
    }
    fn predicted_ties(
        receipt: &Receipt,
        parameter: Parameter,
        policy: ObjectivePolicy,
    ) -> Result<(), String> {
        let losses = receipt.losses.as_ref().ok_or("tie losses missing")?;
        let proposal = receipt.proposal.ok_or("tie proposal missing")?;
        match parameter {
            Parameter::Route(0) => {
                if losses.iter().any(|v| v != &losses[0]) || proposal[0] != 0 || proposal[4] != 0 {
                    return Err("identity tie prediction not observed on CUDA".into());
                }
            }
            Parameter::VectorBlock(0) => {
                let (identity, best) = if policy == ObjectivePolicy::NearestAddressMarginV1 {
                    (448, 352)
                } else {
                    (192, 160)
                };
                if losses[0] != identity
                    || losses[3] != best
                    || losses[12] != best
                    || proposal[0] != 3
                    || proposal[2] != best
                    || proposal[4] != 1
                {
                    return Err("manual nonidentity tie prediction not observed on CUDA; fixture blocker, not mechanism negative".into());
                }
            }
            _ => return Err("unexpected tie parameter".into()),
        }
        Ok(())
    }
    fn host_refusals(
        a: &Device,
        batch_a: &CarriedBatch,
        pa: &mut ProposeProfiler,
        da: &mut DecompositionProfiler,
        base: &Snapshot,
        log: &mut Log,
    ) -> Result<(), String> {
        let policy = base.objective_policy;
        let b = Device::from_snapshot(base.clone(), 0).map_err(|e| e.to_string())?;
        let (windows, targets) = inputs("tie");
        let batch_b = b.carry_windows(&windows).map_err(|e| e.to_string())?;
        let mut pb = b.proposal_profiler().map_err(|e| e.to_string())?;
        let mut db = b.decomposition_profiler().map_err(|e| e.to_string())?;
        // A's probe contains the preceding successful D tie trace. Host
        // refusals must not expose that old terminal record as a new result.
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
                    da,
                    false,
                ),
                "vector_index_U" => call(
                    a,
                    batch_a,
                    &targets,
                    Parameter::VectorBlock(u32::MAX),
                    pa,
                    da,
                    false,
                ),
                "gate_in_route_U" => call(a, batch_a, &targets, Parameter::Gate(0), pa, da, false),
                "wrong_batch_U" => call(
                    &b,
                    batch_a,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    &mut db,
                    false,
                ),
                "wrong_batch_D" => call(
                    &b,
                    batch_a,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    &mut db,
                    true,
                ),
                "wrong_profiler_U" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    pa,
                    &mut db,
                    false,
                ),
                "wrong_profiler_D" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    pa,
                    &mut db,
                    true,
                ),
                "wrong_probe_D" => call(
                    &b,
                    &batch_b,
                    &targets,
                    Parameter::Route(0),
                    &mut pb,
                    da,
                    true,
                ),
                _ => return Err("unknown host refusal inventory".into()),
            };
            let after_a = state(a);
            let after_b = state(&b);
            let admission_check = host_refusal_check(&receipt, id);
            log.push(json!({"kind":"host_admission_refusal","id":id,"policy":policy,"receipt":receipt,
                "before_A":before_a,"before_B":before_b,"after_A":after_a.as_ref().ok(),"after_B":after_b.as_ref().ok(),
                "state_readback_error_A":after_a.as_ref().err(),"state_readback_error_B":after_b.as_ref().err(),
                "admission_check_error":admission_check.as_ref().err(),
                "submission_evidence":if admission_check.is_ok(){"exact_typed_API_admission_guard; source_bound_pre_submission"}else{"UNKNOWN"},
                "cuda_fault_parity_claim":false}))?;
            admission_check?;
            if after_a? != before_a || after_b? != before_b {
                return Err(
                    "host refusal submitted work, leaked a stale trace, or changed state".into(),
                );
            }
        }
        Ok(())
    }
    fn execute(pin: &Pin, log: &mut Log) -> Result<(), String> {
        let compiled = compiled_identity();
        for (key, expected) in [
            ("source_sha256", &pin.source_sha256),
            ("kernel_sha256", &pin.ordinary_kernel_sha256),
            (
                "decomposition_kernel_sha256",
                &pin.decomposition_kernel_sha256,
            ),
            ("geometry_sha256", &pin.geometry_sha256),
        ] {
            if field(&compiled, key)? != *expected {
                return Err(format!("compiled {key} differs before CUDA context"));
            }
        }
        log.push(json!({"kind":"compiled_identity","compiled":compiled,"source_commit":pin.source_commit,
            "commit_binding":"external protected source/build receipt; embedded standalone and PTX identities checked here"}))?;
        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let mut eligible: Vec<(Parameter, Receipt)> = Vec::new();
            for id in [
                "ineligible",
                "eligible",
                "mixed",
                "tie",
                "virtual_ineligible",
                "route_endpoint",
            ] {
                let base = snapshot(id, policy)?;
                let fixture_hash = digest(&serde_json::to_vec(&base).map_err(|e| e.to_string())?);
                let device = Device::from_snapshot(base.clone(), 0).map_err(|e| e.to_string())?;
                let mut profile = device.proposal_profiler().map_err(|e| e.to_string())?;
                let mut probe = device.decomposition_profiler().map_err(|e| e.to_string())?;
                resources(&device, &probe, log, id)?;
                let before = state(&device)?;
                let (windows, targets) = inputs(id);
                let input_hash =
                    digest(&serde_json::to_vec(&(&windows, &targets)).map_err(|e| e.to_string())?);
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
                    for diagnostic in [false, true] {
                        let receipt = call(
                            &device,
                            &batch,
                            &targets,
                            parameter,
                            &mut profile,
                            &mut probe,
                            diagnostic,
                        );
                        let after = state(&device);
                        log.push(json!({"kind":"proposal_attempt","fixture":id,"fixture_snapshot_sha256":fixture_hash,
                            "input_sha256":input_hash,"policy":policy,"parameter":format!("{parameter:?}"),"mode":if diagnostic {"D"} else {"U"},
                            "before":before,"after":after.as_ref().ok(),"state_readback_error":after.as_ref().err(),"receipt":receipt,
                            "prediction_status":"manual prospective expectation; GPU outputs above are observations","apply_calls":0}))?;
                        if after? != before || before["epoch"] != 0 {
                            return Err("control snapshot/epoch changed".into());
                        }
                        receipts.push(receipt);
                    }
                    let u = &receipts[0];
                    let d = &receipts[1];
                    receipt_parity(u, d, if id == "ineligible" { 8 } else { 0 })?;
                    let lanes = trace(d, id)?;
                    log.push(json!({"kind":"decoded_control_lanes","fixture":id,
                        "policy":policy,"parameter":format!("{parameter:?}"),
                        "trace_word_names":meta::WORD_NAMES.as_slice(),"lanes":lanes,
                        "timing_support":false}))?;
                    if id == "eligible" {
                        eligible.push((parameter, u.clone()));
                    }
                    if id == "mixed" {
                        let reference = eligible
                            .iter()
                            .find(|(p, _)| *p == parameter)
                            .ok_or("eligible reference missing")?;
                        if u.losses != reference.1.losses || u.proposal != reference.1.proposal {
                            return Err("ineligible row changed proposal".into());
                        }
                    }
                    if id == "tie" {
                        predicted_ties(u, parameter, policy)?;
                    }
                    if id == "route_endpoint" && u.proposal.ok_or("route proposal missing")?[4] != 1
                    {
                        return Err(
                            "nontrivial Route0 fixture failed sensitivity prerequisite".into()
                        );
                    }
                    if (matches!(id, "eligible" | "tie" | "virtual_ineligible")
                        && parameter == Parameter::VectorBlock(0))
                        || id == "route_endpoint"
                    {
                        endpoints(&base, id, parameter, u, log)?;
                    }
                    log.push(json!({"kind":"control_check","fixture":id,"policy":policy,"parameter":format!("{parameter:?}"),"result":"PASS"}))?;
                }
                if id == "tie" {
                    host_refusals(&device, &batch, &mut profile, &mut probe, &base, log)?;
                }
            }
        }
        let records = log.value["records"].as_array().ok_or("records missing")?;
        let count = |kind: &str| records.iter().filter(|r| r["kind"] == kind).count();
        let actual = json!({"proposal_attempts":count("proposal_attempt"),"endpoint_evaluations":count("materialized_endpoint"),
            "host_refusal_attempts":count("host_admission_refusal")});
        if actual
            != json!({"proposal_attempts":40,"endpoint_evaluations":128,"host_refusal_attempts":16})
        {
            return Err("finite executed inventory differs from reviewed dose".into());
        }
        log.push(json!({"kind":"executed_inventory","actual":actual}))?;
        Ok(())
    }
    pub(super) fn run(pin: &Pin) -> Result<(), String> {
        // New output directory prevents accidental replacement of sealed bytes.
        std::fs::create_dir(&pin.out_dir)
            .map_err(|e| format!("new controls output directory required: {e}"))?;
        let mut runtime_plan = plan(pin);
        // Host-plan false/NOT_RUN fields never label an executed record.
        if let Some(object) = runtime_plan.as_object_mut() {
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
            path: pin.out_dir.join("counterfactual-controls-record.json"),
            value: json!({
            "schema":"m1-counterfactual-controls-record/1","status":"STARTED","execution_contract":runtime_plan,"records":[],
            "apply_calls":0,"training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}),
        };
        log.save()?;
        match execute(pin, &mut log) {
            Ok(()) => {
                log.value["status"] = json!("FINITE_CONTROL_PASS");
                log.value["diagnostic_execution_released"] = json!(false);
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
    use super::*;
    fn template() -> Pin {
        Pin {
            schema: SCHEMA.into(),
            stage: "source_template".into(),
            source_commit: PENDING.into(),
            standalone_source_sha256: PENDING.into(),
            binary_sha256: PENDING.into(),
            source_sha256: PENDING.into(),
            ordinary_kernel_sha256: PENDING.into(),
            decomposition_kernel_sha256: PENDING.into(),
            geometry_sha256: GEOMETRY.into(),
            fixture_inventory_sha256: inventory_hash().expect("metadata"),
            seed: 1,
            evaluation_only: true,
            training_resume: false,
            test_fitness_reference: None,
            execution_release_reference: None,
            out_dir: "new-output".into(),
        }
    }
    #[test]
    fn explicit_template_can_plan_but_never_execute() {
        let p = template();
        assert!(validate(&p, false).is_ok());
        assert!(validate(&p, true).is_err());
        assert_eq!(plan(&p)["status"], "NOT_RUN");
        assert_eq!(plan(&p)["inventory"]["proposal_attempts"], 40);
        assert_eq!(plan(&p)["inventory"]["endpoint_evaluations"], 128);
    }
    #[test]
    fn scope_unknown_fields_and_inventory_drift_refuse() {
        let mut p = template();
        p.seed = 2;
        assert!(validate(&p, false).is_err());
        p.seed = 1;
        p.fixture_inventory_sha256 = "1".repeat(64);
        assert!(validate(&p, false).is_err());
        let mut v = serde_json::to_value(template()).expect("pin");
        v["extra"] = json!(true);
        assert!(serde_json::from_value::<Pin>(v).is_err());
    }
    #[test]
    fn fault_receipt_requires_all_losses_and_current_d_trace() {
        let u = Receipt {
            status: Some(8),
            losses: Some(vec![u64::MAX; 16]),
            proposal: None,
            error: Some(error_detail(
                &uor_r4_discrete::Error::Invalid(
                    "discrete CUDA counterfactual refused with status 8".into(),
                ),
                "proposal",
            )),
            readback_errors: Vec::new(),
            submission_evidence: "counterfactual_status_read".into(),
            raw_trace: None,
            trace_error: None,
        };
        let mut d = u.clone();
        d.raw_trace = Some(vec![0; 640]);
        assert!(receipt_parity(&u, &d, 8).is_ok());
        d.trace_error = Some(error_detail(
            &uor_r4_discrete::Error::Invalid("diagnostic kernel was not submitted".into()),
            "raw_trace",
        ));
        assert!(receipt_parity(&u, &d, 8).is_err());
        d.trace_error = None;
        d.losses.as_mut().expect("losses")[15] = 0;
        assert!(receipt_parity(&u, &d, 8).is_err());
    }
    #[test]
    fn cuda_failure_cannot_pass_as_host_admission_refusal() {
        let mut receipt = Receipt {
            status: None,
            losses: None,
            proposal: None,
            error: Some(error_detail(
                &uor_r4_discrete::Error::Invalid(
                    "carried batch belongs to another model/source/geometry".into(),
                ),
                "proposal",
            )),
            readback_errors: Vec::new(),
            submission_evidence: "NOT_ESTABLISHED_BY_STATUS".into(),
            raw_trace: None,
            trace_error: Some(error_detail(
                &uor_r4_discrete::Error::Invalid("diagnostic kernel was not submitted".into()),
                "raw_trace",
            )),
        };
        assert!(host_refusal_check(&receipt, "wrong_batch_D").is_ok());
        receipt.error = Some(error_detail(
            &uor_r4_discrete::Error::Cuda("driver failed".into()),
            "proposal",
        ));
        assert!(host_refusal_check(&receipt, "wrong_batch_D").is_err());
        receipt.error = Some(error_detail(
            &uor_r4_discrete::Error::Invalid(
                "carried batch belongs to another model/source/geometry".into(),
            ),
            "proposal",
        ));
        receipt.trace_error = Some(error_detail(
            &uor_r4_discrete::Error::Cuda("readback failed".into()),
            "raw_trace",
        ));
        assert!(host_refusal_check(&receipt, "wrong_batch_D").is_err());
    }
    #[test]
    fn unknown_cli_command_refuses_before_reading_pin_or_device() {
        assert!(entry(&[
            "decompose-controls-typo".into(),
            "--pin".into(),
            "missing".into()
        ])
        .unwrap_err()
        .starts_with("decompose-controls-check"));
    }
    #[test]
    #[cfg(not(feature = "cuda"))]
    fn run_without_cuda_has_no_cpu_fallback() {
        let dir = tempfile::tempdir().expect("metadata tempdir");
        let path = dir.path().join("pin.json");
        std::fs::write(&path, serde_json::to_vec(&template()).expect("pin")).expect("pin file");
        let error = entry(&[
            "decompose-controls".into(),
            "--pin".into(),
            path.to_string_lossy().into_owned(),
        ])
        .unwrap_err();
        assert!(error.contains("no CPU model fallback"));
        assert!(!dir.path().join("new-output").exists());
    }
}
