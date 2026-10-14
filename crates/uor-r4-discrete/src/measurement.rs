//! Integer wall-clock observations of the synchronized native GPU API.
//! No corpus, optimizer, apply, floating timing conversion or CPU model.
use crate::{
    device::{Device, Snapshot},
    metadata::BinderPlan,
    Error, Identity, Parameter, ReadSpec, Result, Spec,
};
use serde_json::{json, Value};
use std::{path::Path, time::Instant};

pub const REPEATS: usize = 3;
pub const SHAPE: Spec = Spec {
    vocabulary: 4096,
    blocks: 4,
    context: 192,
    kept_cells: 8,
};
pub const ROWS: usize = 64;

/// Times an operation even when it fails. Callers include synchronization in
/// their closure; this observes host/API elapsed nanoseconds, not GPU cycles.
pub fn timed<T>(operation: impl FnOnce() -> Result<T>) -> (Result<T>, u128) {
    let start = Instant::now();
    let result = operation();
    (result, start.elapsed().as_nanos())
}

/// Records the completed timing, including failures, before propagation.
pub fn observe<T>(
    records: &mut Vec<Value>,
    name: &str,
    repetition: usize,
    operation: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let (result, elapsed_ns) = timed(operation);
    records.push(json!({
        "operation":name,"repetition":repetition,"elapsed_ns":elapsed_ns,
        "status":if result.is_ok(){"PASS"}else{"FAILED"},
        "error":result.as_ref().err().map(ToString::to_string)
    }));
    result
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

fn setup_snapshot(records: &mut Vec<Value>, model: &Device, name: &str) -> Result<Snapshot> {
    let (outcome, setup_elapsed_ns) = timed(|| model.snapshot());
    records.push(json!({"record_kind":"untimed_setup_event","operation":name,
        "setup_elapsed_ns":setup_elapsed_ns,"status":if outcome.is_ok(){"PASS"}else{"FAILED"},
        "error":outcome.as_ref().err().map(ToString::to_string),
        "timing_scope":"setup/storage outside measured GPU API samples; no snapshot parameters in report"}));
    outcome
}

fn windows() -> Vec<Vec<Identity>> {
    // Fixed input identities only. This constructs fixture bytes, not a host
    // model, scorer, learned schedule, or real corpus.
    (0..ROWS)
        .map(|row| {
            (0..SHAPE.context)
                .map(|position| Identity {
                    token: (row as u32 + position) % SHAPE.vocabulary,
                    occurrence: (row as u64) * u64::from(SHAPE.context) + u64::from(position),
                    record: row as u64,
                    version: 1,
                })
                .collect()
        })
        .collect()
}

fn summarize(records: &[Value], name: &str, context_tokens: u128, targets: u128) -> Result<Value> {
    let mut raw = records
        .iter()
        .filter(|r| r["operation"] == name && r["repetition"].as_u64().is_some_and(|n| n > 0))
        .map(|r| {
            r["elapsed_ns"]
                .as_u64()
                .ok_or_else(|| invalid("timing observation exceeds u64 nanoseconds"))
        })
        .collect::<Result<Vec<_>>>()?;
    if raw.len() != REPEATS || raw.contains(&0) {
        return Err(invalid(
            "cost summary requires three positive integer durations",
        ));
    }
    let observed = raw.clone();
    raw.sort_unstable();
    let median = raw[1];
    Ok(json!({"raw_ns":observed,"min_ns":raw[0],"median_ns":median,
        "context_tokens_per_second_floor":context_tokens*1_000_000_000/u128::from(median),
        "target_rows_per_second_floor":targets*1_000_000_000/u128::from(median),
        "rate_formula":"integer floor(work_units * 1000000000 / median_ns)"}))
}

fn mode_run(records: &mut Vec<Value>, name: &str, read_spec: ReadSpec) -> Result<Value> {
    let memory_before = observe(records, &format!("{name}_context_memory_setup"), 0, || {
        Device::memory_info(0)
    })?;
    let model = observe(records, &format!("{name}_load_jit_init"), 0, || {
        let model = Device::new_with_read(SHAPE, read_spec, 7, 0)?;
        if matches!(read_spec, ReadSpec::Flock { .. }) {
            // Authored ACTIVE read fixture, not an applied or learned update.
            // All-zero gates would omit ranking and cannot price attention.
            let mut fixture = model.snapshot()?;
            fixture.gates.fill(1);
            let model = Device::from_snapshot(fixture, 0)?;
            model.synchronize()?;
            Ok(model)
        } else {
            model.synchronize()?;
            Ok(model)
        }
    })?;
    if matches!(read_spec, ReadSpec::Flock { .. }) {
        records.push(json!({"record_kind":"untimed_setup_event","operation":"flock_authored_gate_fixture_snapshot",
            "status":"PASS","timing_scope":"included in combined flock_load_jit_init setup, not operation samples"}));
    }
    let before = setup_snapshot(records, &model, &format!("{name}_snapshot_before"))?;
    let windows = windows();
    let targets: Vec<u32> = (0..ROWS)
        .map(|row| (row as u32 + SHAPE.context) % SHAPE.vocabulary)
        .collect();
    let mut observations = Vec::new();
    let proposals = if read_spec == ReadSpec::Route {
        [
            ("route_proposal", Parameter::Route(0)),
            ("vector_proposal", Parameter::VectorBlock(0)),
        ]
    } else {
        [
            ("gate_proposal", Parameter::Gate(0)),
            ("vector_proposal", Parameter::VectorBlock(0)),
        ]
    };
    let resources_after_model = model.resource_metadata()?;
    records.push(
        json!({"record_kind":"resource_sample","mode":name,"stage":"after_model",
        "memory_before_model":memory_before,"resources":resources_after_model}),
    );
    // Every entry used in timed samples is exercised once before sampling.
    // This setup phase includes first-use JIT effects, with no claim to isolate
    // only the driver's internal JIT time from allocation/init/launch.
    observe(records, &format!("{name}_warmup"), 0, || {
        let batch = model.carry_windows(&windows)?;
        for (_, parameter) in proposals {
            let _proposal = model.propose(&batch, &targets, parameter)?;
        }
        let _forward = model.forward(&batch, &targets)?;
        let _evaluation = model.evaluate(&batch, &targets)?;
        model.synchronize()
    })?;
    let memory_after_warmup = model.resource_metadata()?;
    records.push(json!({"record_kind":"resource_sample","mode":name,"stage":"after_warmup","resources":memory_after_warmup}));
    for repetition in 1..=REPEATS {
        model.synchronize()?;
        let batch = observe(records, &format!("{name}_carry"), repetition, || {
            let batch = model.carry_windows(&windows)?;
            model.synchronize()?;
            Ok(batch)
        })?;
        observations.push(json!({"operation":"memory_after_carry","repetition":repetition,"resources":model.resource_metadata()?}));
        for (operation, parameter) in proposals {
            model.synchronize()?;
            let proposal = observe(records, &format!("{name}_{operation}"), repetition, || {
                let proposal = model.propose(&batch, &targets, parameter)?;
                model.synchronize()?;
                Ok(proposal)
            })?;
            let selected = proposal.observation()?;
            observations.push(json!({
                "operation":operation,"repetition":repetition,"candidate":selected.candidate,
                "before":selected.before,"after":selected.after,"epoch":selected.epoch,
                "active_candidates":if matches!(parameter,Parameter::Gate(_)){3}else{16},"batch_rows":ROWS,"applied":false
            }));
        }
        model.synchronize()?;
        let forward = observe(records, &format!("{name}_forward"), repetition, || {
            let forward = model.forward(&batch, &targets)?;
            model.synchronize()?;
            Ok(forward)
        })?;
        observations.push(json!({"operation":"forward","repetition":repetition,
            "predictions":forward.predictions,"loss_numerators":forward.losses,
            "token_rivals":forward.token_rivals,"address_rivals":forward.address_rivals}));
        model.synchronize()?;
        let evaluation = observe(records, &format!("{name}_evaluate"), repetition, || {
            let evaluation = model.evaluate(&batch, &targets)?;
            model.synchronize()?;
            Ok(evaluation)
        })?;
        observations.push(json!({
            "operation":"evaluate","repetition":repetition,
            "aggregate":evaluation.aggregate,
            "resources_after_evaluation":model.resource_metadata()?
        }));
    }
    let after = setup_snapshot(records, &model, &format!("{name}_snapshot_after"))?;
    if before.table != after.table
        || before.routes != after.routes
        || before.gates != after.gates
        || before.epoch != after.epoch
    {
        return Err(invalid(
            "production-shape measurement mutated learned state",
        ));
    }
    let stages = BinderPlan::new(SHAPE.context as usize)?.stages() as u64;
    let vector_parameters = u64::from(SHAPE.vocabulary) * u64::from(SHAPE.blocks);
    let read_parameters = u64::from(SHAPE.blocks) * stages;
    if vector_parameters != 16384 || read_parameters != 20 {
        return Err(invalid(
            "cost-shape parameter inventory differs from pinned VB16384/BS20",
        ));
    }
    let mut summaries = serde_json::Map::new();
    let physical_tokens = ROWS as u128 * u128::from(SHAPE.context);
    summaries.insert(
        "carry".into(),
        summarize(
            records,
            &format!("{name}_carry"),
            physical_tokens,
            ROWS as u128,
        )?,
    );
    summaries.insert(
        "forward".into(),
        summarize(
            records,
            &format!("{name}_forward"),
            physical_tokens,
            ROWS as u128,
        )?,
    );
    summaries.insert(
        "evaluate".into(),
        summarize(
            records,
            &format!("{name}_evaluate"),
            physical_tokens,
            ROWS as u128,
        )?,
    );
    for (operation, _) in proposals {
        summaries.insert(
            operation.into(),
            summarize(
                records,
                &format!("{name}_{operation}"),
                physical_tokens,
                ROWS as u128,
            )?,
        );
    }
    let read_name = if read_spec == ReadSpec::Route {
        "route_proposal"
    } else {
        "gate_proposal"
    };
    let vector_ns = summaries["vector_proposal"]["median_ns"]
        .as_u64()
        .ok_or_else(|| invalid("missing vector timing"))?;
    let read_ns = summaries[read_name]["median_ns"]
        .as_u64()
        .ok_or_else(|| invalid("missing read timing"))?;
    let vector_pass_ns = u128::from(vector_parameters)
        .checked_mul(u128::from(vector_ns))
        .ok_or_else(|| invalid("vector parameter-pass nanoseconds overflow"))?;
    let read_pass_ns = u128::from(read_parameters)
        .checked_mul(u128::from(read_ns))
        .ok_or_else(|| invalid("read parameter-pass nanoseconds overflow"))?;
    let projected_ns = vector_pass_ns
        .checked_add(read_pass_ns)
        .ok_or_else(|| invalid("total parameter-pass nanoseconds overflow"))?;
    Ok(
        json!({"spec":SHAPE,"read_spec":model.read_spec(),"active_flock_gates":matches!(read_spec,ReadSpec::Flock {..}),"binding":{
        "seed":model.binding().seed,"geometry_sha256":model.binding().geometry_sha256,
        "source_sha256":model.binding().source_sha256
    },"memory_before_model":memory_before,"resources_after_model":resources_after_model,
      "resources_after_warmup":memory_after_warmup,"memory_scope":"point samples, not peak",
      "summaries":summaries,"observations":observations,
      "parameter_pass_projection":{"vector_parameters":vector_parameters,"read_parameters":read_parameters,
        "total_parameters":vector_parameters+read_parameters,"estimated_ns":projected_ns,
        "formula":"vector_parameters*median_vector_ns + read_parameters*median_read_ns",
        "scope":"projection on one fixed 64-row batch; not measured sweep, corpus pass or training dose",
        "status":"UNVERIFIED"}}),
    )
}

fn run_inner(records: &mut Vec<Value>) -> Result<Value> {
    let route = mode_run(records, "route", ReadSpec::Route)?;
    records.push(json!({"record_kind":"completed_mode","mode":"route","result":route}));
    let flock = mode_run(records, "flock", ReadSpec::Flock { kept: 8 })?;
    records.push(json!({"record_kind":"completed_mode","mode":"active_flock","result":flock}));
    Ok(json!({"route":route,"active_flock":flock}))
}

/// Exclusive output precedes any CUDA work. Any device error propagates and
/// ends this bounded measurement; partial timing records remain preserved.
pub fn run(report: &Path) -> Result<()> {
    std::fs::create_dir(report).map_err(|e| invalid(format!("exclusive measurement: {e}")))?;
    let mut records = Vec::new();
    let outcome = run_inner(&mut records);
    let timed_operations = records
        .iter()
        .filter(|record| {
            record["elapsed_ns"].is_number() && record["repetition"].as_u64().is_some_and(|n| n > 0)
        })
        .collect::<Vec<_>>();
    let attempted = timed_operations.len();
    let completed = timed_operations
        .iter()
        .filter(|record| record["status"] == "PASS")
        .count();
    let attempted_proposals = timed_operations
        .iter()
        .filter(|record| {
            record["operation"]
                .as_str()
                .is_some_and(|n| n.ends_with("_proposal"))
        })
        .count();
    let completed_proposals = timed_operations
        .iter()
        .filter(|record| {
            record["status"] == "PASS"
                && record["operation"]
                    .as_str()
                    .is_some_and(|n| n.ends_with("_proposal"))
        })
        .count();
    let payload = json!({
        "schema":"uor-r4.discrete-cost-observation/1",
        "status":if outcome.is_ok(){"PASS"}else if matches!(&outcome,Err(Error::Cuda(_))){"ABORTED"}else{"FAILED"},
        "spec":SHAPE,"rows":ROWS,"repeats":REPEATS,"read_modes":["route","active_flock_kept8"],
        "timing_units":"integer host wall nanoseconds, synchronized native API",
        "initialization_and_snapshot_outside_timed_intervals":true,
        "applied_updates":0,"training":"HELD","corpus":false,
        "timings":records,"result":outcome.as_ref().ok(),
        "error":outcome.as_ref().err().map(ToString::to_string),
        "projection_status":"UNVERIFIED fixed-batch parameter-pass estimate; dollars need pinned price/dose",
        "observed_api_operations":{"attempted_timed":attempted,"completed_timed":completed,
          "attempted_timed_proposals":attempted_proposals,"completed_timed_proposals":completed_proposals,
          "scope":"API calls, not a claim that all candidate/row work completed after a fault"},
        "declared_timed_capacity":{"proposal_attempts":REPEATS*4,"active_candidate_evaluations":REPEATS*(16+16+3+16),
          "candidate_row_capacity":REPEATS*(16+16+3+16)*ROWS,"evaluation_rows":REPEATS*2*ROWS,
          "forward_rows":REPEATS*2*ROWS,
          "carried_physical_tokens":REPEATS*2*ROWS*SHAPE.context as usize},
        "declared_warmup_capacity":{"proposal_attempts":4,"active_candidate_evaluations":51,"candidate_row_capacity":51*ROWS,
          "evaluation_rows":2*ROWS,"carried_physical_tokens":2*ROWS*SHAPE.context as usize},
        "warmup_forward_rows":2*ROWS,
        "capacity_scope":"planned bounds only; raw completed/failed records state what actually ran",
        "setup_scope":"combined module load/JIT/initialization and separate first-use warmup; not isolated JIT-only time"
    });
    std::fs::write(
        report.join("measurement.json"),
        serde_json::to_vec_pretty(&payload).map_err(|e| invalid(e.to_string()))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    outcome.map(|_| ())
}
