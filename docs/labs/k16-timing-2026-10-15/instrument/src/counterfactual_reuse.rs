//! Exact FIT2560 opt-in baseline-reuse qualification. Host work hashes, parses,
//! and compares observed metadata only; no host model or loss reference.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use uor_r4_discrete::{baseline_reuse::BaselineReuseImage, decomposition::CudaImages};

const SCHEMA: &str = "m1-counterfactual-reuse-pin/1";
const ARTIFACT: &str = super::checkpoint_bridge::ORIGINAL_ARTIFACT;
const ORIGINAL_SOURCE: &str = super::checkpoint_bridge::ORIGINAL_SOURCE;
const KERNEL: &str = super::checkpoint_bridge::KERNEL;
const GEOMETRY: &str = super::checkpoint_bridge::GEOMETRY;
const CONFIG: &str = super::checkpoint_bridge::CONFIG;
const ORDINARY_CUBIN: &str = "1648564346740520c125fcfb5bc407cbbca40e66e95e2e3ad89b526a321ffcee";
const ORDINARY_SOURCE: &str = "f679f8736500925014ff721ab035f3c284446baa6af0763f27b09bc388763008";
const PENDING: &str = "PENDING";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub schema: String,
    pub phase: String,
    pub stage: String,
    pub source_commit: String,
    pub standalone_source_sha256: String,
    pub binary_sha256: String,
    pub target_source_sha256: String,
    pub ordinary_source_sha256: String,
    pub ordinary_kernel_sha256: String,
    pub cuda_images: Option<CudaImages>,
    pub baseline_reuse_image: Option<BaselineReuseImage>,
    pub geometry_sha256: String,
    pub fixture_inventory_sha256: String,
    pub snapshot_path: PathBuf,
    pub original_snapshot_sha256: String,
    pub original_source_sha256: String,
    pub config_path: PathBuf,
    pub config_file_sha256: String,
    pub out_dir: PathBuf,
    pub test_fitness_reference: Option<String>,
    pub execution_release_reference: Option<String>,
    pub finite_acceptance_reference: Option<String>,
    pub finite_record_sha256: Option<String>,
    pub evaluation_only: bool,
    pub training_resume: bool,
}

pub(super) fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_hash(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        && value.bytes().any(|b| b != b'0')
}
fn authority(value: &Option<String>) -> bool {
    value.as_deref().is_some_and(|s| {
        s.starts_with("https://github.com/UOR-Foundation/uor-r4/")
            && s.contains('#')
            && !s.chars().any(char::is_whitespace)
    })
}
fn standalone_source_hash() -> String {
    let mut hash = Sha256::new();
    hash.update(b"m1-counterfactual-reuse-standalone-source/1");
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
        (
            "src/counterfactual_reuse.rs",
            include_bytes!("counterfactual_reuse.rs").as_slice(),
        ),
        (
            "src/counterfactual_reuse_controls.rs",
            include_bytes!("counterfactual_reuse_controls.rs").as_slice(),
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
    json!({"schema":"m1-counterfactual-reuse-inventory/1",
        "finite":super::counterfactual_reuse_controls::inventory(),
        "timing":{"seed":1,"epoch":0,"shape":super::K16,"rows":64,
          "policies":["nearest_address_margin_v1","occupied_cell_rank_hinge_v1"],
          "operations":["Route(0)","VectorBlock(0)"],"batches":4,
          "mode_order":[["U","R"],["U","R"],["R","U"],["U","R"]],
          "nesting":"policy,batch,mode,operation; recarry identical windows before each operation",
          "proposal_attempts":32,"excluded_warmups":8,"timed":24,
          "generator":"existing main.rs batches(4) and targets()",
          "whole_unit":"per operation: carry + propose + full loss/proposal/audit readback + remainder; includes synchronization; excludes subsequent metadata validation and record persistence"},
        "apply_calls":0,"read_mode":"route","shared_bytes_R":512,"audit_bytes_R":1280,
        "no_endpoint_rerun":true,"no_clock64":true})
}
fn inventory_hash() -> Result<String, String> {
    serde_json::to_vec(&inventory())
        .map(|bytes| digest(&bytes))
        .map_err(|e| e.to_string())
}
pub(super) fn validate(pin: &Pin, execution: bool) -> Result<(), String> {
    let template = pin.stage == "source_template";
    if pin.schema != SCHEMA
        || !matches!(pin.stage.as_str(), "source_template" | "execution_pinned")
        || !matches!(pin.phase.as_str(), "finite" | "timing")
        || !pin.evaluation_only
        || pin.training_resume
        || pin.ordinary_source_sha256 != ORDINARY_SOURCE
        || pin.ordinary_kernel_sha256 != KERNEL
        || pin.geometry_sha256 != GEOMETRY
        || pin.original_snapshot_sha256 != ARTIFACT
        || pin.original_source_sha256 != ORIGINAL_SOURCE
        || pin.config_file_sha256 != CONFIG
        || pin.fixture_inventory_sha256 != inventory_hash()?
        || pin.out_dir.as_os_str().is_empty()
        || pin.snapshot_path.as_os_str().is_empty()
        || pin.config_path.as_os_str().is_empty()
    {
        return Err("reuse pin scope/identity/inventory refused; Training/M2/Stage B HELD; Q4 NOT_EXECUTABLE".into());
    }
    for (value, n) in [
        (&pin.source_commit, 40),
        (&pin.standalone_source_sha256, 64),
        (&pin.binary_sha256, 64),
        (&pin.target_source_sha256, 64),
    ] {
        if !(valid_hash(value, n) || template && value == PENDING) {
            return Err("reuse pin requires exact source/executable SHA identities".into());
        }
    }
    match (&pin.cuda_images, &pin.baseline_reuse_image) {
        (Some(images), Some(reuse)) => {
            images.validate()?;
            reuse.validate()?;
            if images.ordinary_sha256 != ORDINARY_CUBIN || images.target != reuse.target {
                return Err("ordinary comparator cubin differs from sealed2559 bytes".into());
            }
        }
        (None, None) if template => (),
        _ => {
            return Err(
                "reuse pin needs both exact U/D image inventory and separate R binding".into(),
            )
        }
    }
    if !template {
        if !authority(&pin.test_fitness_reference)
            || !authority(&pin.execution_release_reference)
            || pin.test_fitness_reference == pin.execution_release_reference
        {
            return Err("reuse execution NOT_RELEASED: independent FIT and distinct externally reviewed release references required".into());
        }
        if pin.phase == "timing"
            && (!authority(&pin.finite_acceptance_reference)
                || pin.finite_acceptance_reference == pin.execution_release_reference
                || !pin
                    .finite_record_sha256
                    .as_deref()
                    .is_some_and(|h| valid_hash(h, 64)))
        {
            return Err("timing needs accepted finite record hash and separate acceptance/release references".into());
        }
        if pin.phase == "finite"
            && (pin.finite_acceptance_reference.is_some() || pin.finite_record_sha256.is_some())
        {
            return Err("finite pin cannot masquerade as prior finite acceptance".into());
        }
    }
    if execution {
        if template {
            return Err("source template is NOT_EXECUTABLE".into());
        }
        if pin.standalone_source_sha256 != standalone_source_hash() {
            return Err("reuse standalone source identity differs".into());
        }
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        if digest(&std::fs::read(exe).map_err(|e| e.to_string())?) != pin.binary_sha256 {
            return Err("reuse executable identity differs".into());
        }
    }
    Ok(())
}
#[cfg(any(feature = "cuda", test))]
pub(super) fn compiled_matches(pin: &Pin, compiled: &Value) -> Result<(), String> {
    let images = pin.cuda_images.as_ref().ok_or("missing U/D binding")?;
    let reuse = pin
        .baseline_reuse_image
        .as_ref()
        .ok_or("missing R binding")?;
    images.validate()?;
    reuse.validate()?;
    if compiled["cuda_images"] != json!(images)
        || compiled["baseline_reuse_image"] != json!(reuse)
        || compiled["source_sha256"] != pin.target_source_sha256
        || compiled["kernel_sha256"] != KERNEL
        || compiled["geometry_sha256"] != GEOMETRY
        || images.ordinary_sha256 != ORDINARY_CUBIN
        || digest(include_bytes!(
            "../../../../../crates/uor-r4-discrete/src/discrete.cu"
        )) != ORDINARY_SOURCE
    {
        return Err(
            "reuse compiled source/PTX/cubin/report binding differs before CUDA context".into(),
        );
    }
    Ok(())
}
pub(super) fn plan(pin: &Pin) -> Value {
    json!({"schema":"m1-counterfactual-reuse-plan/1","status":"NOT_RUN","pin":pin,
        "execution":"NOT_RELEASED_BY_THIS_PLAN","inventory":inventory(),"model_compute_executed":false,
        "cuda_context_created":false,"apply_calls":0,
        "decision":"all four cells: min(U)-max(R) > max(range(U),range(R)) for event and propose wall; whole max(R)-min(U) <= spread",
        "resources":"U static shared0 / R512, dynamic0, each runtime matches own AOT, zero spills, launch16",
        "limitations":"same enclosing profiler in U/R; no profiling-off claim, p95, training rate, language result or energy claim; no 47% or16x speedup presumed",
        "training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"})
}
fn template(phase: String, out_dir: PathBuf) -> Result<Pin, String> {
    let pin = Pin {
        schema: SCHEMA.into(),
        phase,
        stage: "source_template".into(),
        source_commit: PENDING.into(),
        standalone_source_sha256: standalone_source_hash(),
        binary_sha256: PENDING.into(),
        target_source_sha256: PENDING.into(),
        ordinary_source_sha256: ORDINARY_SOURCE.into(),
        ordinary_kernel_sha256: KERNEL.into(),
        cuda_images: None,
        baseline_reuse_image: None,
        geometry_sha256: GEOMETRY.into(),
        fixture_inventory_sha256: inventory_hash()?,
        snapshot_path: "checkpoint0-timing-snapshot.json".into(),
        original_snapshot_sha256: ARTIFACT.into(),
        original_source_sha256: ORIGINAL_SOURCE.into(),
        config_path: "config.json".into(),
        config_file_sha256: CONFIG.into(),
        out_dir,
        test_fitness_reference: None,
        execution_release_reference: None,
        finite_acceptance_reference: None,
        finite_record_sha256: None,
        evaluation_only: true,
        training_resume: false,
    };
    validate(&pin, false)?;
    Ok(pin)
}
pub fn entry(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("reuse-identity") if args.len() == 1 => {
            #[cfg(feature = "cuda")]
            let compiled = uor_r4_discrete::device::compiled_identity();
            #[cfg(not(feature = "cuda"))]
            let compiled = json!({"status":"UNAVAILABLE_WITHOUT_CUDA_BUILD"});
            let binary = digest(
                &std::fs::read(std::env::current_exe().map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?,
            );
            println!(
                "{}",
                json!({"standalone_source_sha256":standalone_source_hash(),"binary_sha256":binary,
                "fixture_inventory_sha256":inventory_hash()?,"compiled":compiled,"cuda_context_created":false,"model_compute_executed":false})
            );
            return Ok(());
        }
        Some("reuse-template")
            if args.len() == 5 && args[1] == "--phase" && args[3] == "--out-dir" =>
        {
            println!(
                "{}",
                serde_json::to_string_pretty(&template(args[2].clone(), args[4].clone().into())?)
                    .map_err(|e| e.to_string())?
            );
            return Ok(());
        }
        _ => (),
    }
    if args.len() != 3
        || args[1] != "--pin"
        || !matches!(
            args[0].as_str(),
            "reuse-check" | "reuse-plan" | "reuse-controls" | "reuse-timing"
        )
    {
        return Err("reuse-check|reuse-plan|reuse-controls|reuse-timing --pin FILE; reuse-identity; reuse-template --phase finite|timing --out-dir DIR".into());
    }
    let pin: Pin = serde_json::from_slice(&std::fs::read(&args[2]).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    validate(&pin, false)?;
    if matches!(args[0].as_str(), "reuse-check" | "reuse-plan") {
        println!(
            "{}",
            serde_json::to_string_pretty(&plan(&pin)).map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    if (args[0] == "reuse-controls") != (pin.phase == "finite") {
        return Err("command and pinned phase differ".into());
    }
    validate(&pin, true)?;
    #[cfg(feature = "cuda")]
    {
        if pin.phase == "finite" {
            super::counterfactual_reuse_controls::run(&pin)
        } else {
            cuda::run(&pin)
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        Err("reuse requires CUDA; no CPU model fallback".into())
    }
}

/// Time statistics operate only on measured durations, never model values.
#[cfg(any(feature = "cuda", test))]
fn metric(pairs: &[(u128, u128)]) -> Result<Value, String> {
    if pairs.len() != 3 || pairs.iter().any(|&(u, r)| u == 0 || r == 0) {
        return Err("exactly three nonzero timed pairs required".into());
    }
    let mut u: Vec<_> = pairs.iter().map(|p| p.0).collect();
    let mut r: Vec<_> = pairs.iter().map(|p| p.1).collect();
    u.sort_unstable();
    r.sort_unstable();
    let signed = |n: u128| {
        i128::try_from(n).map_err(|_| "duration exceeds signed accounting range".to_owned())
    };
    let gap = signed(u[0])? - signed(r[2])?;
    let spread = (u[2] - u[0]).max(r[2] - r[0]);
    let regression = signed(r[2])? - signed(u[0])?;
    let differences = pairs.iter().map(|&(u,r)|Ok(json!({"U_ns":u,"R_ns":r,"U_minus_R_ns":signed(u)?-signed(r)?,"R_over_U":{"numerator":r,"denominator":u}}))).collect::<Result<Vec<Value>,String>>()?;
    Ok(
        json!({"U":{"min":u[0],"median":u[1],"max":u[2]},"R":{"min":r[0],"median":r[1],"max":r[2]},
        "gap_ns":gap,"spread_ns":spread,"improved":gap>signed(spread)?,"nonregression":regression<=signed(spread)?,"pairs":differences}),
    )
}

#[cfg(feature = "cuda")]
mod cuda {
    use super::super::{batches, targets, timed, Ledger};
    use super::*;
    use std::time::Instant;
    use uor_r4_discrete::{
        baseline_reuse as reuse, decomposition as meta,
        device::{compiled_identity, Device, Snapshot},
        ObjectivePolicy, Parameter,
    };

    fn prepare(pin: &Pin) -> Result<(Vec<u8>, Value), String> {
        compiled_matches(pin, &compiled_identity())?;
        let config = std::fs::read(&pin.config_path).map_err(|e| e.to_string())?;
        if digest(&config) != CONFIG {
            return Err("reuse config checksum differs".into());
        }
        let cfg: uor_r4_discrete::training::DriverConfig =
            serde_json::from_slice(&config).map_err(|e| e.to_string())?;
        cfg.validate().map_err(|e| e.to_string())?;
        if cfg.spec != super::super::K16
            || cfg.seed != 1
            || cfg.batch_rows != 64
            || cfg.read_spec != uor_r4_discrete::ReadSpec::Route
            || cfg.objective_policy != ObjectivePolicy::NearestAddressMarginV1
        {
            return Err("reuse config is not original seed1 K16 Route A".into());
        }
        let bytes = std::fs::read(&pin.snapshot_path).map_err(|e| e.to_string())?;
        map_checkpoint(pin, &bytes)
    }
    pub fn run(pin: &Pin) -> Result<(), String> {
        validate(pin, true)?;
        if pin.phase != "timing" {
            return Err("timing command refuses non-timing phase".into());
        }
        compiled_matches(pin, &compiled_identity())?;
        std::fs::create_dir(&pin.out_dir)
            .map_err(|e| format!("new timing output directory required: {e}"))?;
        let mut header = plan(pin);
        header
            .as_object_mut()
            .ok_or("header object")?
            .remove("model_compute_executed");
        header
            .as_object_mut()
            .ok_or("header object")?
            .remove("status");
        header
            .as_object_mut()
            .ok_or("header object")?
            .remove("cuda_context_created");
        header
            .as_object_mut()
            .ok_or("header object")?
            .remove("execution");
        let mut ledger = Ledger::new(
            pin.out_dir.join("counterfactual-reuse-timing-record.json"),
            json!({"baseline_reuse":header}),
        );
        match execute(pin, &mut ledger) {
            Ok(()) => ledger.flush("complete"),
            Err(e) => Err(ledger.fault("baseline_reuse_timing", &e)),
        }
    }
    #[derive(Clone)]
    struct Observed {
        receipt: meta::ProposalReceipt,
        event: u128,
        wall: u128,
        whole: u128,
    }
    fn execute(pin: &Pin, ledger: &mut Ledger) -> Result<(), String> {
        let (mapped, bridge) = prepare(pin)?;
        ledger.push(bridge);
        ledger.flush("partial")?;
        let windows = batches(4);
        let targets = targets();
        let mut all_improved = true;
        let mut any_regression = false;
        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let mut snapshot: Snapshot =
                serde_json::from_slice(&mapped).map_err(|e| e.to_string())?;
            snapshot.objective_policy = policy;
            let device = Device::from_snapshot(snapshot, 0).map_err(|e| e.to_string())?;
            let state = || -> Result<String, String> {
                Ok(digest(
                    &serde_json::to_vec(&device.snapshot().map_err(|e| e.to_string())?)
                        .map_err(|e| e.to_string())?,
                ))
            };
            let before = state()?;
            if device.epoch().map_err(|e| e.to_string())? != 0 {
                return Err("nonzero initial epoch".into());
            }
            let mut profiler = device.proposal_profiler().map_err(|e| e.to_string())?;
            let mut audit = device
                .baseline_reuse_profiler()
                .map_err(|e| e.to_string())?;
            let u_resources = device
                .ordinary_counterfactual_resources()
                .map_err(|e| e.to_string())?;
            let r_resources = audit.resources().map_err(|e| e.to_string())?;
            let admitted = reuse::resource_parity(&u_resources, &r_resources);
            ledger.push(json!({"record_kind":"resource_admission","policy":policy,"U":u_resources,"R":r_resources,"admitted":admitted,"snapshot_before":before,"epoch":0,"cuda_images":pin.cuda_images,"baseline_reuse_image":pin.baseline_reuse_image}));
            ledger.flush("partial")?;
            if !admitted {
                return Err("BLOCKED: U/R resource admission failed before dose".into());
            }
            let mut samples: [Vec<(Observed, Observed)>; 2] = Default::default();
            for (batch, window) in windows.iter().enumerate() {
                let input_hash =
                    digest(&serde_json::to_vec(&(window, &targets)).map_err(|e| e.to_string())?);
                let mut by_mode: [Vec<Observed>; 2] = Default::default();
                for mode in if batch == 2 { ["R", "U"] } else { ["U", "R"] } {
                    for parameter in [Parameter::Route(0), Parameter::VectorBlock(0)] {
                        device.synchronize().map_err(|e| e.to_string())?;
                        let started = Instant::now();
                        let (carried, carry_ns) = timed(|| {
                            let b = device.carry_windows(window).map_err(|e| e.to_string())?;
                            device.synchronize().map_err(|e| e.to_string())?;
                            Ok(b)
                        });
                        let carried = carried?;
                        let (proposal, wall) = timed(|| {
                            let p = if mode == "R" {
                                device.propose_reused(
                                    &carried,
                                    &targets,
                                    parameter,
                                    &mut profiler,
                                    &mut audit,
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
                            device.synchronize().map_err(|e| e.to_string())?;
                            Ok(p)
                        });
                        let proposal = match proposal {
                            Ok(p) => p,
                            Err(e) => {
                                ledger.push(json!({"record_kind":"fault_proposal","policy":policy,"batch":batch,"mode":mode,"parameter":format!("{parameter:?}"),"input_sha256":input_hash,"carry_ns":carry_ns,"propose_ns":wall,"counterfactual_status":profiler.counterfactual_status(),"fault_losses":profiler.fault_losses(),"raw_audit":if mode=="R"{audit.raw_trace().ok()}else{None},"host":profiler.host_timings()}));
                                return Err(e);
                            }
                        };
                        let (readback, observation_ns) = timed(|| {
                            let observed = proposal.observation().map_err(|e| e.to_string())?;
                            let losses = proposal.candidate_losses().map_err(|e| e.to_string())?;
                            let raw = if mode == "R" {
                                Some(audit.raw_trace().map_err(|e| e.to_string())?)
                            } else {
                                None
                            };
                            device.synchronize().map_err(|e| e.to_string())?;
                            Ok((observed, losses, raw))
                        });
                        let whole = started.elapsed().as_nanos();
                        let (observed, losses, raw) = readback?;
                        let timings = profiler.timings().map_err(|e| e.to_string())?;
                        let receipt = meta::ProposalReceipt {
                            losses,
                            status: profiler.counterfactual_status().ok_or("missing status")?,
                            proposal: [
                                observed.candidate,
                                observed.before,
                                observed.after,
                                observed.epoch,
                                u64::from(observed.any_candidate_cost_differs_from_identity),
                            ],
                        };
                        let measured = carry_ns
                            .checked_add(wall)
                            .and_then(|n| n.checked_add(observation_ns))
                            .ok_or("duration overflow")?;
                        let remainder = whole
                            .checked_sub(measured)
                            .ok_or("whole-unit accounting contradiction")?;
                        ledger.push(json!({"record_kind":if batch==0{"excluded_warmup_proposal"}else{"timed_reuse_proposal"},"policy":policy,"batch":batch,"mode":mode,"parameter":format!("{parameter:?}"),"input_sha256":input_hash,"carry_ns":carry_ns,"propose_ns":wall,"full_observation_ns":observation_ns,"whole_unit_ns":whole,"remainder_ns":remainder,"subphases":timings,"observation":receipt,"raw_audit":raw,"apply_calls":0}));
                        ledger.flush("partial")?;
                        if timings
                            .counterfactual
                            .nanoseconds_floor
                            .checked_add(timings.chooser.nanoseconds_floor)
                            .is_none_or(|e| e > wall.saturating_add(1_000_000u128.max(wall / 100)))
                        {
                            return Err("GPU event/wall contradiction".into());
                        }
                        receipt.validate()?;
                        if let Some(raw) = &raw {
                            reuse::decode(raw, 64)?;
                        }
                        by_mode[usize::from(mode == "R")].push(Observed {
                            receipt,
                            event: timings.counterfactual.nanoseconds_floor,
                            wall,
                            whole,
                        });
                    }
                }
                for (op, destination) in samples.iter_mut().enumerate() {
                    let u = by_mode[0].get(op).ok_or("missing U")?;
                    let r = by_mode[1].get(op).ok_or("missing R")?;
                    meta::parity(&u.receipt, &r.receipt)?;
                    if batch != 0 {
                        destination.push((u.clone(), r.clone()));
                    }
                }
            }
            for (op, pairs) in samples.iter().enumerate() {
                let event = metric(
                    &pairs
                        .iter()
                        .map(|(u, r)| (u.event, r.event))
                        .collect::<Vec<_>>(),
                )?;
                let wall = metric(
                    &pairs
                        .iter()
                        .map(|(u, r)| (u.wall, r.wall))
                        .collect::<Vec<_>>(),
                )?;
                let whole = metric(
                    &pairs
                        .iter()
                        .map(|(u, r)| (u.whole, r.whole))
                        .collect::<Vec<_>>(),
                )?;
                let improved = event["improved"] == true
                    && wall["improved"] == true
                    && whole["nonregression"] == true;
                let regressed = event["nonregression"] == false
                    || wall["nonregression"] == false
                    || whole["nonregression"] == false;
                all_improved &= improved;
                any_regression |= regressed;
                ledger.push(json!({"record_kind":"timing_cell_summary","policy":policy,"operation":if op==0{"Route(0)"}else{"VectorBlock(0)"},"event":event,"propose_wall":wall,"whole_unit":whole,"engineering_bar_pass":improved,"configuration_regression":regressed,"no_automatic_extra_samples":true}));
            }
            let after = state()?;
            let epoch = device.epoch().map_err(|e| e.to_string())?;
            ledger.push(json!({"record_kind":"state_invariance","policy":policy,"before":before,"after":after,"epoch":epoch,"apply_calls":0}));
            if before != after || epoch != 0 {
                return Err("state/epoch changed".into());
            }
        }
        if digest(&std::fs::read(&pin.snapshot_path).map_err(|e| e.to_string())?) != ARTIFACT {
            return Err("original snapshot bytes changed".into());
        }
        ledger.push(json!({"record_kind":"fixed_dose_decision","proposal_attempts":32,"warmups":8,"timed":24,"pairs":12,"decision":if all_improved{"ENGINEERING_BENEFIT_FIXED_SCOPE"}else if any_regression{"CONFIGURATION_REGRESSION"}else{"INCONCLUSIVE"},"activate_reuse":false,"next":"separate prospective disposition; no automatic qualification or training","training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}));
        Ok(())
    }
}

#[cfg(any(feature = "cuda", test))]
fn map_checkpoint(pin: &Pin, bytes: &[u8]) -> Result<(Vec<u8>, Value), String> {
    if digest(bytes) != ARTIFACT {
        return Err("reuse bridge original artifact checksum differs".into());
    }
    map_envelope(pin, bytes)
}
#[cfg(any(feature = "cuda", test))]
fn map_envelope(pin: &Pin, bytes: &[u8]) -> Result<(Vec<u8>, Value), String> {
    let mut v: Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    if v["schema"] != "uor-r4.discrete-spiralcore-fbs/2"
        || v["source_sha256"] != ORIGINAL_SOURCE
        || v["kernel_sha256"] != KERNEL
        || v["geometry_sha256"] != GEOMETRY
        || v["seed"] != 1
        || v["epoch"] != 0
        || v["spec"] != json!(super::K16)
        || v["read_spec"] != json!({"kind":"route"})
    {
        return Err("reuse bridge original envelope differs".into());
    }
    let before = v.clone();
    v["source_sha256"] = json!(pin.target_source_sha256);
    let mapped = serde_json::to_vec(&v).map_err(|e| e.to_string())?;
    v["source_sha256"] = json!(ORIGINAL_SOURCE);
    if v != before {
        return Err("reuse bridge changed non-source payload".into());
    }
    v.as_object_mut()
        .ok_or("snapshot object")?
        .remove("source_sha256");
    let payload = digest(&serde_json::to_vec(&v).map_err(|e| e.to_string())?);
    Ok((
        mapped.clone(),
        json!({"record_kind":"checkpoint0_reuse_identity_bridge","original_snapshot_sha256":digest(bytes),"in_memory_snapshot_sha256":digest(&mapped),"pre_source_sha256":ORIGINAL_SOURCE,"post_source_sha256":pin.target_source_sha256,"ordinary_kernel_sha256":KERNEL,"cuda_images":pin.cuda_images,"baseline_reuse_image":pin.baseline_reuse_image,"unchanged_non_source_payload_sha256":payload,"changed_fields":["source_sha256"],"evaluation_only":true,"training_resume":false,"rebound_checkpoint_written":false}),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pinned() -> Pin {
        let mut p = template("finite".into(), "records".into()).expect("template");
        p.stage = "execution_pinned".into();
        p.source_commit = "1".repeat(40);
        p.target_source_sha256 = "2".repeat(64);
        p.binary_sha256 = "3".repeat(64);
        p.cuda_images = Some(CudaImages {
            schema: "uor-r4.cuda-images/1".into(),
            format: "cubin".into(),
            target: "sm_120".into(),
            ordinary_sha256: ORDINARY_CUBIN.into(),
            decomposition_sha256: "4".repeat(64),
            ordinary_report_sha256: "5".repeat(64),
            decomposition_report_sha256: "6".repeat(64),
        });
        p.baseline_reuse_image = Some(BaselineReuseImage {
            schema: "uor-r4.baseline-reuse-image/1".into(),
            format: "cubin".into(),
            target: "sm_120".into(),
            source_sha256: "7".repeat(64),
            ptx_sha256: "8".repeat(64),
            cubin_sha256: "9".repeat(64),
            report_sha256: "a".repeat(64),
        });
        p.test_fitness_reference = Some(
            "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-test-fit".into(),
        );
        p.execution_release_reference = Some(
            "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-finite-release"
                .into(),
        );
        p
    }
    #[test]
    fn exact_loaded_images_and_distinct_stage_authority_are_required() {
        let p = pinned();
        assert!(validate(&p, false).is_ok());
        let mut same_release = p.clone();
        same_release.execution_release_reference = same_release.test_fitness_reference.clone();
        assert!(validate(&same_release, false).is_err());
        let compiled = json!({"source_sha256":p.target_source_sha256,"kernel_sha256":KERNEL,"geometry_sha256":GEOMETRY,"cuda_images":p.cuda_images,"baseline_reuse_image":p.baseline_reuse_image});
        assert!(compiled_matches(&p, &compiled).is_ok());
        for path in [
            "/source_sha256",
            "/kernel_sha256",
            "/geometry_sha256",
            "/cuda_images/ordinary_sha256",
            "/baseline_reuse_image/cubin_sha256",
            "/baseline_reuse_image/report_sha256",
            "/baseline_reuse_image/ptx_sha256",
            "/baseline_reuse_image/source_sha256",
        ] {
            let mut wrong = compiled.clone();
            *wrong.pointer_mut(path).expect("field") = json!("b".repeat(64));
            assert!(compiled_matches(&p, &wrong).is_err(), "{path}");
        }
        let mut timing = p.clone();
        timing.phase = "timing".into();
        assert!(validate(&timing, false).is_err());
        timing.finite_record_sha256 = Some("c".repeat(64));
        timing.finite_acceptance_reference = timing.execution_release_reference.clone();
        assert!(validate(&timing, false).is_err());
        timing.finite_acceptance_reference = Some(
            "https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-finite-acceptance"
                .into(),
        );
        assert!(validate(&timing, false).is_ok());
        let mut wrong = p;
        wrong.cuda_images.as_mut().expect("images").ordinary_sha256 = "d".repeat(64);
        assert!(validate(&wrong, false).is_err());
    }
    #[test]
    fn template_cannot_execute_or_change_fixed_scope() {
        let p = template("finite".into(), "records".into()).expect("template");
        assert!(validate(&p, false).is_ok());
        assert!(validate(&p, true).is_err());
        let value = serde_json::to_value(p).expect("pin");
        for (key, wrong) in [
            ("phase", json!("training")),
            ("ordinary_kernel_sha256", json!("a".repeat(64))),
            ("ordinary_source_sha256", json!("a".repeat(64))),
            ("training_resume", json!(true)),
            ("evaluation_only", json!(false)),
            ("fixture_inventory_sha256", json!("b".repeat(64))),
        ] {
            let mut changed = value.clone();
            changed[key] = wrong;
            let changed: Pin = serde_json::from_value(changed).expect("typed");
            assert!(validate(&changed, false).is_err(), "{key}");
        }
    }
    #[test]
    fn duration_bar_retains_all_pairs_and_requires_strict_separation() {
        let good = metric(&[(110, 70), (120, 80), (115, 75)]).expect("durations");
        assert_eq!(good["improved"], true);
        assert_eq!(good["pairs"].as_array().expect("pairs").len(), 3);
        let boundary = metric(&[(100, 80), (110, 90), (105, 85)]).expect("durations");
        assert_eq!(boundary["improved"], false);
        let slower = metric(&[(70, 110), (80, 120), (75, 115)]).expect("durations");
        assert_eq!(slower["nonregression"], false);
        assert!(metric(&[(1, 1), (2, 2)]).is_err());
        assert!(metric(&[(0, 1), (2, 2), (3, 3)]).is_err());
    }
    #[test]
    fn bridge_preserves_every_non_source_field_and_refuses_wrong_bytes() {
        let mut p = template("timing".into(), "records".into()).expect("template");
        p.target_source_sha256 = "a".repeat(64);
        let value = json!({"schema":"uor-r4.discrete-spiralcore-fbs/2","source_sha256":ORIGINAL_SOURCE,"kernel_sha256":KERNEL,"geometry_sha256":GEOMETRY,"seed":1,"epoch":0,"spec":super::super::K16,"read_spec":{"kind":"route"},"opaque_payload":[1,2,3]});
        let bytes = serde_json::to_vec(&value).expect("metadata");
        assert!(map_checkpoint(&p, &bytes).is_err());
        let (mapped, receipt) = map_envelope(&p, &bytes).expect("envelope-only metadata control");
        let mut changed: Value = serde_json::from_slice(&mapped).expect("mapped");
        changed["source_sha256"] = json!(ORIGINAL_SOURCE);
        assert_eq!(changed, value);
        assert_eq!(receipt["rebound_checkpoint_written"], false);
        let mut wrong = value;
        wrong["epoch"] = json!(1);
        assert!(map_envelope(&p, &serde_json::to_vec(&wrong).expect("metadata")).is_err());
    }
}
