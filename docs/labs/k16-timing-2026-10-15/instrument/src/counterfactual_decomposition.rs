//! Separate fail-closed command/pin for the FIT'd decomposition design.
//! Host checks parse/hash records only; model work exists only in cuda::run.
use super::{json, Value};
use serde::{Deserialize, Serialize};
#[cfg(any(feature = "cuda", test))]
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uor_r4_discrete::decomposition::{CudaImages, DENOMINATOR, LIMITATION};

const SCHEMA: &str = "m1-counterfactual-decomposition-pin/2";
const ARTIFACT: &str = super::checkpoint_bridge::ORIGINAL_ARTIFACT;
const SOURCE: &str = super::checkpoint_bridge::ORIGINAL_SOURCE;
const KERNEL: &str = super::checkpoint_bridge::KERNEL;
const GEOMETRY: &str = super::checkpoint_bridge::GEOMETRY;
const CONFIG: &str = super::checkpoint_bridge::CONFIG;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub schema: String,
    pub snapshot_path: PathBuf,
    pub original_snapshot_sha256: String,
    pub original_source_sha256: String,
    pub original_kernel_sha256: String,
    pub target_source_sha256: String,
    pub ordinary_kernel_sha256: String,
    pub decomposition_kernel_sha256: String,
    pub cuda_images: CudaImages,
    pub geometry_sha256: String,
    pub config_path: PathBuf,
    pub config_file_sha256: String,
    pub evaluation_only: bool,
    pub training_resume: bool,
    pub out_dir: PathBuf,
}
#[cfg(any(feature = "cuda", test))]
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_hash(h: &str) -> bool {
    h.len() == 64
        && h.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && h != "0".repeat(64)
}
fn validate(pin: &Pin) -> Result<(), String> {
    pin.cuda_images.validate()?;
    if pin.schema != SCHEMA
        || pin.original_snapshot_sha256 != ARTIFACT
        || pin.original_source_sha256 != SOURCE
        || pin.original_kernel_sha256 != KERNEL
        || pin.ordinary_kernel_sha256 != KERNEL
        || pin.geometry_sha256 != GEOMETRY
        || pin.config_file_sha256 != CONFIG
        || !pin.evaluation_only
        || pin.training_resume
        || !valid_hash(&pin.target_source_sha256)
        || !valid_hash(&pin.decomposition_kernel_sha256)
        || pin.target_source_sha256 == SOURCE
        || pin.decomposition_kernel_sha256 == KERNEL
        || pin.snapshot_path.as_os_str().is_empty()
        || pin.config_path.as_os_str().is_empty()
        || pin.out_dir.as_os_str().is_empty()
    {
        return Err("decomposition pin refused: exact checkpoint0 source/PTX mapping, hashes or evaluation-only scope missing; Q4 NOT_EXECUTABLE".into());
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn compiled_matches(pin: &Pin, compiled: &Value) -> Result<(), String> {
    pin.cuda_images.validate()?;
    if compiled["cuda_images"] != json!(pin.cuda_images) {
        return Err(
            "decomposition loaded image/report/target binding differs before CUDA context".into(),
        );
    }
    for (key, value) in [
        ("source_sha256", &pin.target_source_sha256),
        ("kernel_sha256", &pin.ordinary_kernel_sha256),
        (
            "decomposition_kernel_sha256",
            &pin.decomposition_kernel_sha256,
        ),
        ("geometry_sha256", &pin.geometry_sha256),
    ] {
        if compiled[key].as_str() != Some(value.as_str()) {
            return Err(format!(
                "decomposition compiled {key} differs from reviewed exact pair"
            ));
        }
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn config_matches(pin: &Pin, bytes: &[u8]) -> Result<(), String> {
    if digest(bytes) != pin.config_file_sha256 {
        return Err("decomposition config checksum differs".into());
    }
    let cfg: uor_r4_discrete::training::DriverConfig =
        serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    cfg.validate().map_err(|e| e.to_string())?;
    if cfg.spec != super::K16
        || cfg.seed != 1
        || cfg.batch_rows != 64
        || cfg.read_spec != uor_r4_discrete::ReadSpec::Route
        || cfg.objective_policy != uor_r4_discrete::ObjectivePolicy::NearestAddressMarginV1
    {
        return Err("decomposition requires exact seed1 K16-A initialization configuration".into());
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
fn map_bytes(
    pin: &Pin,
    bytes: &[u8],
    compiled: &Value,
    expected_artifact: &str,
) -> Result<(Vec<u8>, Value), String> {
    compiled_matches(pin, compiled)?;
    if digest(bytes) != expected_artifact {
        return Err("decomposition bridge original artifact checksum differs".into());
    }
    let mut v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if v["schema"] != "uor-r4.discrete-spiralcore-fbs/2"
        || v["source_sha256"] != SOURCE
        || v["kernel_sha256"] != KERNEL
        || v["geometry_sha256"] != GEOMETRY
        || v["seed"] != 1
        || v["epoch"] != 0
        || v["read_spec"] != json!({"kind":"route"})
        || v["spec"] != json!(super::K16)
    {
        return Err(
            "decomposition bridge requires original epoch0/seed1 K16 Route envelope".into(),
        );
    }
    let original = v.clone();
    v["source_sha256"] = json!(pin.target_source_sha256);
    let mapped = serde_json::to_vec(&v).map_err(|e| e.to_string())?;
    v["source_sha256"] = json!(SOURCE);
    if v != original {
        return Err("decomposition bridge changed payload".into());
    }
    let object = v.as_object_mut().ok_or("snapshot must be an object")?;
    object.remove("source_sha256");
    let payload = serde_json::to_vec(&v).map_err(|e| e.to_string())?;
    let receipt = json!({"schema":"uor-r4.checkpoint0-decomposition-bridge/2","record_kind":"checkpoint0_source_ptx_image_bridge","original_snapshot_sha256":digest(bytes),"in_memory_snapshot_sha256":digest(&mapped),"pre_source_sha256":SOURCE,"post_source_sha256":pin.target_source_sha256,"pre_kernel_sha256":KERNEL,"ordinary_kernel_sha256":pin.ordinary_kernel_sha256,"decomposition_kernel_sha256":pin.decomposition_kernel_sha256,"cuda_images":pin.cuda_images,"geometry_sha256":GEOMETRY,"config_file_sha256":CONFIG,"unchanged_non_source_payload_sha256":digest(&payload),"changed_fields":["source_sha256"],"evaluation_only":true,"training_resume":false,"rebound_checkpoint_written":false,"device_admission":"separately required through strict production snapshot and compiled-image admission"});
    Ok((mapped, receipt))
}
#[cfg(any(feature = "cuda", test))]
fn prepare(pin: &Pin, bytes: &[u8], compiled: &Value) -> Result<(Vec<u8>, Value), String> {
    validate(pin)?;
    map_bytes(pin, bytes, compiled, ARTIFACT)
}

pub fn mode_order(batch: usize) -> [&'static str; 2] {
    if batch == 2 {
        ["D", "U"]
    } else {
        ["U", "D"]
    }
}
fn plan(pin: &Pin) -> Value {
    json!({"schema":"m1-counterfactual-decomposition-plan/1","pin":pin,"status":"NOT_RUN","model_compute_executed":false,"apply_calls":0,"rows":64,"shape":super::K16,"policies":["nearest_address_margin_v1","occupied_cell_rank_hinge_v1"],"operations":["Route(0)","VectorBlock(0)"],"warmup_batches_per_mode_per_policy":1,"timed_batches_per_mode_per_policy":3,"total_proposals":32,"timed_proposals":24,"pair_order":[mode_order(1),mode_order(2),mode_order(3)],"share_denominator":DENOMINATOR,"retained_lanes":"all sixteen completed nonfaulted lanes per sample; none discarded","stable":"every retained lane in all three samples lies on same side of 20/80 threshold","resource_parity":"zero U/D spill stores/loads; runtime registers == AOT registers and runtime local == AOT stack (zero deltas); D runtime local and AOT stack <= U+1024; identical shared bytes; max_threads_per_block >=16 is launchability disclosure only; fixed host launch and E/S bound retained; otherwise UNRESOLVED","limitations":LIMITATION,"scope":super::DIAGNOSTIC_SCOPE,"h1_magnitude":"NOT_APPROVED; launch-only comparison separately preregistered","training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"})
}

pub fn entry(args: &[String]) -> Result<(), String> {
    let command = args.first().map(String::as_str).unwrap_or_default();
    if command == "decompose-identity" {
        #[cfg(feature = "cuda")]
        {
            println!("{}", uor_r4_discrete::device::compiled_identity());
            return Ok(());
        }
        #[cfg(not(feature = "cuda"))]
        return Err(
            "identity requires the CUDA build; it creates no context and executes no model".into(),
        );
    }
    let path = args
        .iter()
        .position(|a| a == "--pin")
        .and_then(|i| args.get(i + 1))
        .ok_or("decompose-check|decompose-plan|decompose --pin FILE")?;
    let pin: Pin = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate(&pin)?;
    match command {
        "decompose-check" | "decompose-plan" => {
            println!(
                "{}",
                serde_json::to_string_pretty(&plan(&pin)).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        "decompose" => {
            #[cfg(feature = "cuda")]
            {
                cuda::run(&pin)
            }
            #[cfg(not(feature = "cuda"))]
            Err("decompose requires CUDA; no CPU model fallback".into())
        }
        _ => Err("unknown decomposition command".into()),
    }
}

#[cfg(feature = "cuda")]
mod cuda {
    use super::super::{batches, targets, timed, Ledger};
    use super::*;
    use std::time::Instant;
    use uor_r4_discrete::{
        decomposition as meta,
        device::{compiled_identity, Device, Snapshot},
        ObjectivePolicy, Parameter,
    };
    type Sample = (u128, u128, Vec<meta::LaneRecord>);
    pub fn run(pin: &Pin) -> Result<(), String> {
        let mut execution_header = plan(pin);
        // The plan's false/NOT_RUN labels describe a host plan, never an
        // executed record. Actual model work is evidenced by cell/call records.
        if let Some(object) = execution_header.as_object_mut() {
            object.remove("model_compute_executed");
            object.remove("status");
        }
        execution_header["trace_word_names"] = json!(meta::WORD_NAMES.as_slice());
        let mut ledger = Ledger::new(
            pin.out_dir.join("counterfactual-decomposition-record.json"),
            json!({"decomposition":execution_header}),
        );
        match execute(pin, &mut ledger) {
            Ok(()) => ledger.flush("complete"),
            Err(error) => Err(ledger.fault("decomposition", &error)),
        }
    }
    fn execute(pin: &Pin, ledger: &mut Ledger) -> Result<(), String> {
        let bytes = std::fs::read(&pin.snapshot_path).map_err(|e| e.to_string())?;
        config_matches(
            pin,
            &std::fs::read(&pin.config_path).map_err(|e| e.to_string())?,
        )?;
        let (mapped, receipt) = prepare(pin, &bytes, &compiled_identity())?;
        ledger.push(receipt);
        let windows = batches(4);
        let targets = targets();
        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let mut snapshot: Snapshot =
                serde_json::from_slice(&mapped).map_err(|e| e.to_string())?;
            snapshot.objective_policy = policy;
            let device = Device::from_snapshot(snapshot, 0).map_err(|e| e.to_string())?;
            let before = digest(
                &serde_json::to_vec(&device.snapshot().map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?,
            );
            let epoch_before = device.epoch().map_err(|e| e.to_string())?;
            ledger.push(json!({"record_kind":"cell_start","policy":policy,"snapshot_before":before,"epoch":epoch_before,"original_artifact_sha256":ARTIFACT}));
            if epoch_before != 0 {
                return Err("nonzero before epoch".into());
            }
            let mut profiler = device.proposal_profiler().map_err(|e| e.to_string())?;
            let mut probe = device.decomposition_profiler().map_err(|e| e.to_string())?;
            let u_resources = device
                .ordinary_counterfactual_resources()
                .map_err(|e| e.to_string())?;
            let d_resources = probe.resources().map_err(|e| e.to_string())?;
            let resource_match = meta::resource_parity(&u_resources, &d_resources);
            ledger.push(json!({"record_kind":"resource_parity","policy":policy,"U":u_resources,"D":d_resources,"cuda_images":pin.cuda_images,"within_allowance":resource_match,"share_denominator":DENOMINATOR}));
            if !resource_match {
                return Err("UNRESOLVED: resource parity outside prospective allowance; no diagnostic dose executed".into());
            }
            let mut samples: [Vec<Sample>; 2] = Default::default();
            for (batch, window) in windows.iter().enumerate() {
                let inputs =
                    digest(&serde_json::to_vec(&(window, &targets)).map_err(|e| e.to_string())?);
                type ModeReceipt = (meta::ProposalReceipt, u128, Vec<meta::LaneRecord>);
                let mut by_mode: [Vec<ModeReceipt>; 2] = Default::default();
                for mode in mode_order(batch) {
                    let slot = usize::from(mode == "D");
                    device.synchronize().map_err(|e| e.to_string())?;
                    let mode_started = Instant::now();
                    let (carried, carry_ns) = timed(|| {
                        let batch = device.carry_windows(window).map_err(|e| e.to_string())?;
                        device.synchronize().map_err(|e| e.to_string())?;
                        Ok(batch)
                    });
                    let carried = carried?;
                    let mut propose_total = 0;
                    let mut observation_total = 0;
                    for parameter in [Parameter::Route(0), Parameter::VectorBlock(0)] {
                        device.synchronize().map_err(|e| e.to_string())?;
                        let mut sync_ns = 0;
                        let (result, wall) = timed(|| {
                            let result = if mode == "D" {
                                device.propose_decomposed(
                                    &carried,
                                    &targets,
                                    parameter,
                                    &mut profiler,
                                    &mut probe,
                                )
                            } else {
                                device.propose_profiled(
                                    &carried,
                                    &targets,
                                    parameter,
                                    &mut profiler,
                                )
                            }
                            .map_err(|e| e.to_string())?;
                            let started = Instant::now();
                            device.synchronize().map_err(|e| e.to_string())?;
                            sync_ns = started.elapsed().as_nanos();
                            Ok(result)
                        });
                        let proposal = match result {
                            Ok(value) => value,
                            Err(error) => {
                                let trace = if mode == "D" {
                                    probe.raw_trace().ok()
                                } else {
                                    None
                                };
                                ledger.push(json!({"record_kind":"partial_decomposition_proposal","batch":batch,"mode":mode,"parameter":format!("{parameter:?}"),"status":profiler.counterfactual_status(),"fault_losses":profiler.fault_losses(),"raw_trace":trace,"host":profiler.host_timings(),"wall_ns":wall}));
                                return Err(error);
                            }
                        };
                        let timings = profiler.timings().map_err(|e| e.to_string())?;
                        let event = timings.counterfactual.nanoseconds_floor;
                        let tolerance = 1_000_000u128.max(wall / 100);
                        let clock_valid = !event
                            .checked_add(timings.chooser.nanoseconds_floor)
                            .is_none_or(|v| v > wall.saturating_add(tolerance));
                        let (observed, observation_ns) = timed(|| {
                            let observation = proposal.observation().map_err(|e| e.to_string())?;
                            device.synchronize().map_err(|e| e.to_string())?;
                            Ok(observation)
                        });
                        let observed = observed?;
                        propose_total += wall;
                        observation_total += observation_ns;
                        let readback_started = Instant::now();
                        let losses = proposal.candidate_losses().map_err(|e| e.to_string())?;
                        let record = meta::ProposalReceipt {
                            losses,
                            status: profiler
                                .counterfactual_status()
                                .ok_or("missing counterfactual status")?,
                            proposal: [
                                observed.candidate,
                                observed.before,
                                observed.after,
                                observed.epoch,
                                u64::from(observed.any_candidate_cost_differs_from_identity),
                            ],
                        };
                        let raw = if mode == "D" {
                            Some(probe.raw_trace().map_err(|e| e.to_string())?)
                        } else {
                            None
                        };
                        ledger.push(json!({"record_kind":if batch==0{"excluded_warmup_proposal"}else{"timed_decomposition_proposal"},"policy":policy,"batch":batch,"mode":mode,"parameter":format!("{parameter:?}"),"input_sha256":inputs,"propose_ns":wall,"extra_context_sync_ns":sync_ns,"subphases":timings,"observation":record,"observation_ns":observation_ns,"raw_trace":raw,"loss_trace_readback_ns":readback_started.elapsed().as_nanos(),"applied":false,"rates":null}));
                        if !clock_valid {
                            return Err("GPU event/wall contradiction".into());
                        }
                        let lanes = raw
                            .as_ref()
                            .map(|words| meta::decode(words))
                            .transpose()?
                            .unwrap_or_default();
                        record.validate()?;
                        by_mode[slot].push((record, event, lanes));
                    }
                    let full_unit_ns = carry_ns + propose_total + observation_total;
                    let diagnostic_batch_wall_ns = mode_started.elapsed().as_nanos();
                    if full_unit_ns > diagnostic_batch_wall_ns {
                        return Err("full unit exceeds diagnostic batch wall".into());
                    }
                    ledger.push(json!({"record_kind":"mode_batch_accounting","policy":policy,"batch":batch,"warmup":batch==0,"mode":mode,"carry_ns":carry_ns,"propose_ns":propose_total,"observation_ns":observation_total,"full_unit_ns":full_unit_ns,"diagnostic_batch_wall_ns":diagnostic_batch_wall_ns,"extra_readback_gaps_ns":diagnostic_batch_wall_ns-full_unit_ns,"rate":null}));
                }
                for (operation, output) in samples.iter_mut().enumerate() {
                    let u = by_mode[0].get(operation).ok_or("missing U proposal")?;
                    let d = by_mode[1].get(operation).ok_or("missing D proposal")?;
                    meta::parity(&u.0, &d.0)?;
                    if batch != 0 {
                        output.push((u.1, d.1, d.2.clone()));
                    }
                }
            }
            let after = digest(
                &serde_json::to_vec(&device.snapshot().map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?,
            );
            let epoch = device.epoch().map_err(|e| e.to_string())?;
            ledger.push(json!({"record_kind":"cell_summary","policy":policy,"snapshot_before":before,"snapshot_after":after,"epoch":epoch,"apply_calls":0,"timed_proposals":12,"warmup_proposals":4,"route":meta::decide(&samples[0],resource_match)?,"vector_block":meta::decide(&samples[1],resource_match)?,"limitations":LIMITATION,"h1_magnitude":"NOT_APPROVED"}));
            if before != after || epoch != 0 {
                return Err("snapshot or epoch changed".into());
            }
        }
        if digest(&std::fs::read(&pin.snapshot_path).map_err(|e| e.to_string())?) != ARTIFACT {
            return Err("original artifact bytes changed".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn images() -> CudaImages {
        CudaImages {
            schema: "uor-r4.cuda-images/1".into(),
            format: "cubin".into(),
            target: "sm_120".into(),
            ordinary_sha256: "1".repeat(64),
            decomposition_sha256: "2".repeat(64),
            ordinary_report_sha256: "3".repeat(64),
            decomposition_report_sha256: "4".repeat(64),
        }
    }
    fn pin() -> Pin {
        Pin {
            schema: SCHEMA.into(),
            snapshot_path: "checkpoint0-timing-snapshot.json".into(),
            original_snapshot_sha256: ARTIFACT.into(),
            original_source_sha256: SOURCE.into(),
            original_kernel_sha256: KERNEL.into(),
            target_source_sha256: "a".repeat(64),
            ordinary_kernel_sha256: KERNEL.into(),
            decomposition_kernel_sha256: "b".repeat(64),
            cuda_images: images(),
            geometry_sha256: GEOMETRY.into(),
            config_path: "config.json".into(),
            config_file_sha256: CONFIG.into(),
            evaluation_only: true,
            training_resume: false,
            out_dir: "records".into(),
        }
    }
    fn compiled(p: &Pin) -> Value {
        json!({"source_sha256":p.target_source_sha256,"kernel_sha256":p.ordinary_kernel_sha256,"decomposition_kernel_sha256":p.decomposition_kernel_sha256,"geometry_sha256":p.geometry_sha256,"cuda_images":p.cuda_images})
    }
    fn envelope() -> Value {
        json!({"schema":"uor-r4.discrete-spiralcore-fbs/2","spec":super::super::K16,"read_spec":{"kind":"route"},"seed":1,"epoch":0,"table":[2,2,0,0,0,0,0,0],"routes":[0],"gates":[0],"operator_kappa":"fixture-metadata","composition_kappa":"fixture-metadata","source_sha256":SOURCE,"kernel_sha256":KERNEL,"geometry_sha256":GEOMETRY})
    }
    #[test]
    fn exact_pair_scope_refuses_training_new_artifact_and_compiled_mismatch() {
        let p = pin();
        assert!(validate(&p).is_ok());
        let original = serde_json::to_value(&p).unwrap();
        for field in [
            "original_snapshot_sha256",
            "original_source_sha256",
            "original_kernel_sha256",
            "ordinary_kernel_sha256",
            "geometry_sha256",
            "config_file_sha256",
        ] {
            let mut v = original.clone();
            v[field] = json!("0".repeat(64));
            assert!(validate(&serde_json::from_value(v).unwrap()).is_err());
        }
        let mut p = pin();
        p.training_resume = true;
        assert!(validate(&p).is_err());
        let mut c = compiled(&pin());
        c["decomposition_kernel_sha256"] = json!(KERNEL);
        assert!(compiled_matches(&pin(), &c).is_err());
        assert!(prepare(&pin(), b"{}", &compiled(&pin())).is_err());
    }
    #[test]
    fn producer_shaped_snapshot_mapping_changes_only_source_and_preserves_both_ptx_identities() {
        let p = pin();
        let original = envelope();
        let bytes = serde_json::to_vec(&original).unwrap();
        let (mapped, receipt) = map_bytes(&p, &bytes, &compiled(&p), &digest(&bytes)).unwrap();
        let mut after: Value = serde_json::from_slice(&mapped).unwrap();
        after["source_sha256"] = json!(SOURCE);
        assert_eq!(after, original);
        assert_eq!(receipt["pre_kernel_sha256"], KERNEL);
        assert_eq!(
            receipt["decomposition_kernel_sha256"],
            p.decomposition_kernel_sha256
        );
        assert_eq!(receipt["training_resume"], false);
        assert_eq!(receipt["cuda_images"], json!(p.cuda_images));
        assert_eq!(receipt["changed_fields"], json!(["source_sha256"]));
        let mut stale = original;
        stale["epoch"] = json!(1);
        let stale = serde_json::to_vec(&stale).unwrap();
        assert!(map_bytes(&p, &stale, &compiled(&p), &digest(&stale)).is_err());
    }
    #[test]
    fn pair_order_and_plan_cannot_count_99_or_confuse_with_qualification() {
        assert_eq!(mode_order(1), ["U", "D"]);
        assert_eq!(mode_order(2), ["D", "U"]);
        assert_eq!(mode_order(3), ["U", "D"]);
        let plan = plan(&pin());
        assert_eq!(plan["total_proposals"], 32);
        assert_eq!(plan["timed_proposals"], 24);
        assert_eq!(plan["apply_calls"], 0);
        assert_eq!(plan["status"], "NOT_RUN");
        assert_eq!(plan["limitations"], LIMITATION);
        assert_ne!(plan["schema"], super::super::PIN_SCHEMA);
    }
    #[test]
    fn config_pin_is_checked_against_real_merged_initialization_metadata() {
        let p = pin();
        let bytes = include_bytes!(
            "../../../admission-redirect-packet-2026-10-15/configs/k16-A-seed1.json"
        );
        assert!(config_matches(&p, bytes).is_ok());
        assert!(config_matches(&p, b"{}").is_err());
    }
    #[test]
    fn image_binding_refuses_omission_old_pins_and_each_compiled_drift() {
        let p = pin();
        assert!(compiled_matches(&p, &compiled(&p)).is_ok());
        let mut omitted = serde_json::to_value(&p).unwrap();
        omitted.as_object_mut().unwrap().remove("cuda_images");
        assert!(serde_json::from_value::<Pin>(omitted).is_err());
        let mut old = p.clone();
        old.schema = "m1-counterfactual-decomposition-pin/1".into();
        assert!(validate(&old).is_err());
        let c = compiled(&p);
        for (field, value) in [
            ("schema", "uor-r4.cuda-images/2".into()),
            ("format", "ptx".into()),
            ("target", "sm_100".into()),
            ("ordinary_sha256", "2".repeat(64)),
            ("decomposition_sha256", "1".repeat(64)),
            ("ordinary_report_sha256", "4".repeat(64)),
            ("decomposition_report_sha256", "3".repeat(64)),
        ] {
            let mut altered = c.clone();
            altered["cuda_images"][field] = json!(value);
            assert!(compiled_matches(&p, &altered).is_err(), "{field}");
        }
        let mut missing = c;
        missing.as_object_mut().unwrap().remove("cuda_images");
        assert!(compiled_matches(&p, &missing).is_err());
    }
}
