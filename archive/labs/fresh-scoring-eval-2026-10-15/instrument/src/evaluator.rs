use super::receipts::{new_output, seal, write_new};
use super::*;
use std::{fs::OpenOptions, io::Write, time::Instant};
use uor_r4_discrete::device::{self, Evaluation};

fn evaluate(prepared: &Prepared, output: &Path) -> Result<Value> {
    let started = Instant::now();
    let checkpoint = &prepared.input.checkpoint;
    let (model, cursor) = prepared.driver.restore(checkpoint.clone(), 0)?;
    model.synchronize()?;
    let before = model.snapshot()?;
    require(
        serde_json::to_value(&before)? == checkpoint.model,
        "restored snapshot differs from pinned checkpoint",
    )?;
    let before_sha = digest(&before)?;
    let before_epoch = model.epoch()?;
    require(
        before_epoch == cursor.accepted_updates,
        "restored epoch differs from checkpoint cursor",
    )?;
    write_new(
        &output.join("before.json"),
        &encode(&json!({"typed_snapshot_sha256":before_sha,
        "model_storage_sha256":checkpoint.model_sha256,"epoch":before_epoch,"compiled":device::compiled_identity()}))?,
    )?;
    let mut partial = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output.join("batches.jsonl"))?;
    let mut evaluations: Vec<Evaluation> = Vec::new();
    let mut observed = Vec::with_capacity(ROWS);
    for first in (0..ROWS).step_by(BATCH_ROWS as usize) {
        let batch = prepared
            .fresh
            .corpus
            .batch(first as u64, CONTEXT, BATCH_ROWS)?;
        let correspondence = row_correspondence(&batch, first, &prepared.fresh.rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        let mut evaluation = model.evaluate(&carried, &batch.targets)?;
        require(
            evaluation.rows.len() == batch.targets.len()
                && evaluation.aggregate.scored_rows == batch.targets.len() as u64
                && evaluation.objective_policy == prepared.driver.config().objective_policy,
            "INSTRUMENT_FAULT: GPU receipt cardinality/objective mismatch",
        )?;
        for (row, identity) in evaluation.rows.iter_mut().zip(&correspondence) {
            require(
                row.target_position.is_none() && row.target_token.is_none(),
                "unexpected preexisting target annotation",
            )?;
            row.target_position = Some(identity.target_position);
            row.target_token = Some(identity.target_token);
        }
        model.synchronize()?;
        require(
            model.epoch()? == before_epoch,
            "INSTRUMENT_FAULT: epoch changed during batch",
        )?;
        partial.write_all(&encode(
            &json!({"first_row":first,"correspondence":correspondence,"evaluation":evaluation}),
        )?)?;
        partial.write_all(b"\n")?;
        partial.sync_all()?;
        observed.extend(correspondence);
        evaluations.push(evaluation);
    }
    require(
        observed == prepared.fresh.correspondence,
        "INSTRUMENT_FAULT: missing/duplicated/reordered fresh rows",
    )?;
    let aggregate = model.aggregate_evaluations(&evaluations)?;
    require(
        aggregate.scored_rows == ROWS as u64,
        "INSTRUMENT_FAULT: GPU aggregate row count mismatch",
    )?;
    model.synchronize()?;
    let after = model.snapshot()?;
    let after_sha = digest(&after)?;
    let after_epoch = model.epoch()?;
    write_new(
        &output.join("after.json"),
        &encode(&json!({"typed_snapshot_sha256":after_sha,"epoch":after_epoch}))?,
    )?;
    verify_unchanged(
        &before_sha,
        &after_sha,
        before_epoch,
        after_epoch,
        cursor.accepted_updates,
    )?;
    let mut report = prepared.plan()?;
    report["status"] = json!("FRESH_SCORED_EVAL_ONLY");
    report["model_execution"] = json!("CUDA_EVALUATION_ONLY");
    report["instrument_executable_sha256"] =
        json!(corpus::sha256(&fs::read(std::env::current_exe()?)?));
    report["compiled"] = device::compiled_identity();
    report["model_bindings"] = json!({"schema":before.schema,"spec":before.spec,"read_spec":before.read_spec,
        "objective_policy":before.objective_policy,"seed":before.seed,"source_sha256":before.source_sha256,
        "kernel_sha256":before.kernel_sha256,"geometry_sha256":before.geometry_sha256,
        "operator_kappa":before.operator_kappa,"composition_kappa":before.composition_kappa});
    report["before_typed_snapshot_sha256"] = json!(before_sha);
    report["after_typed_snapshot_sha256"] = json!(after_sha);
    report["before_epoch"] = json!(before_epoch);
    report["after_epoch"] = json!(after_epoch);
    report["evaluation_mutates_model"] = json!(false);
    report["aggregate"] = serde_json::to_value(aggregate)?;
    report["batches"] = serde_json::to_value(evaluations)?;
    report["elapsed_ns"] = json!(started.elapsed().as_nanos().to_string());
    report["interpretation"] =
        json!("raw fresh-transfer evidence; no automatic KEEP/capability promotion");
    Ok(report)
}

pub(super) fn run(request_path: &Path, output: &Path) -> Result<()> {
    new_output(output)?;
    let result = (|| {
        let request: Request = read_json(request_path)?;
        write_new(&output.join("request.json"), &encode(&request)?)?;
        let prepared = Prepared::load(request)?;
        write_new(&output.join("plan.json"), &encode(&prepared.plan()?)?)?;
        write_new(
            &output.join("checkpoint-input.json"),
            &prepared.request.checkpoint.read()?,
        )?;
        write_new(
            &output.join("admission-input.json"),
            &prepared.request.admission.read()?,
        )?;
        let report = evaluate(&prepared, output)?;
        write_new(&output.join("report.json"), &encode(&report)?)?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })();
    if let Err(error) = &result {
        write_new(
            &output.join("failure.json"),
            &encode(&json!({"status":"INSTRUMENT_OR_INPUT_FAILURE",
            "error":error.to_string(),"mechanism_negative":false,"partial_receipts":"preserved, never resumed or silently retried",
            "updates_applied_by_evaluation":0}))?,
        )?;
    }
    seal(output)?;
    result
}
