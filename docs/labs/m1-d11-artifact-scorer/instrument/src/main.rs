//! `d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON`
//! `d11-scorer export-trunk CHECKPOINT_JSON CONFIG_JSON OUT_LUT REPORT_JSON`
//! `d11-scorer rows CONFIG_JSON OUT_JSON`
//! `d11-scorer score ARTIFACT_LUT CONFIG_JSON REPORT_JSON ROWS_JSONL [--reference] [--train-eval TRAIN_EVAL_JSON GAP_REPORT_JSON]`
//! `d11-scorer fresh-score SEED1_LUT SEED2_LUT FRESH_CONFIG_JSON DECISION_RECEIPT_JSON REPORT_JSON`
//! `d11-scorer m1-close-grade OUT_GRADE_JSON OUT_FRESH_GATE_JSON`
//! `d11-scorer toks ARTIFACT_LUT REPORT_JSON [--prompt TOKENS_JSON]`
//!
//! `toks` times `IntegerDiscreteModel` serving on this machine by the step-6
//! protocol (1 warmup, 20 runs, 192-token prefill, 64-token decode).
//!
//! `m1-d11-artifact-scorer` (#3248), `m1-export-gap-and-toks` (#2839), and
//! `m1-fresh-panel-transfer` (#2841). `score` runs on the GPU (`cuda` feature)
//! over the config's DEV rows (and refuses the sealed fresh panel); `fresh-score`
//! runs on the GPU over the sealed Fresh V2 panel (`0b7e754c…` / `a1cda529…`)
//! only after `validate_fresh_transfer_gate` verifies a sealed two-seed DEV
//! bar-(a)–(d) `PASS_BARS_A_TO_D_TWO_SEEDS` decision receipt. Every output is
//! created new: an existing file is never overwritten.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use m1_d11_artifact_scorer_instrument as inst;
use serde_json::Value;
use uor_r4_discrete::training::DriverConfig;
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

fn read_json(path: &str) -> Result<Value, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))
}

fn write_new(path: &str, bytes: &[u8]) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(path))
        .map_err(|e| format!("{path}: {e} (outputs are never overwritten)"))?;
    file.write_all(bytes).map_err(|e| format!("{path}: {e}"))
}

/// Refuses before any work when an output exists, so a command that writes two
/// files never leaves one without its partner.
fn refuse_existing(paths: &[&str]) -> Result<(), String> {
    match paths.iter().find(|path| Path::new(path).exists()) {
        Some(path) => Err(format!("{path} exists (outputs are never overwritten)")),
        None => Ok(()),
    }
}

fn json_bytes(value: &Value) -> Result<Vec<u8>, String> {
    let mut text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    text.push('\n');
    Ok(text.into_bytes())
}

fn run(args: &[String]) -> Result<(), String> {
    match args {
        [command, checkpoint, out, report] if command == "export" => {
            refuse_existing(&[out, report])?;
            let exported = inst::export_checkpoint(&read_json(checkpoint)?)?;
            write_new(out, &exported.bytes)?;
            write_new(report, &json_bytes(&exported.report)?)?;
            println!(
                "exported {} bytes, file sha256 {}",
                exported.bytes.len(),
                exported.report["file_sha256"]
            );
            Ok(())
        }
        [command, checkpoint, config, out, report] if command == "export-trunk" => {
            export_trunk(checkpoint, config, out, report)
        }
        [command, config, out] if command == "rows" => {
            let config: DriverConfig = serde_json::from_value(read_json(config)?)
                .map_err(|e| format!("{config}: {e}"))?;
            let rows = inst::dev_rows(&config)?;
            let report = inst::rows_report(&rows);
            write_new(out, &json_bytes(&report)?)?;
            println!("{} rows, sha256 {}", rows.len(), report["rows_sha256"]);
            Ok(())
        }
        [command, artifact, config, report, row_records, flags @ ..] if command == "score" => {
            let mut reference = false;
            let mut train_eval: Option<(&str, &str)> = None;
            let mut idx = 0;
            while idx < flags.len() {
                if flags[idx] == "--reference" {
                    reference = true;
                    idx += 1;
                } else if flags[idx] == "--train-eval" && idx + 2 < flags.len() {
                    train_eval = Some((&flags[idx + 1], &flags[idx + 2]));
                    idx += 3;
                } else {
                    return Err(
                        "score takes only [--reference] [--train-eval TRAIN_EVAL_JSON GAP_REPORT_JSON] after its paths"
                            .into(),
                    );
                }
            }
            score(artifact, config, report, row_records, reference, train_eval)
        }
        [command, seed1_lut, seed2_lut, fresh_config, decision_receipt, report]
            if command == "fresh-score" =>
        {
            fresh_score(seed1_lut, seed2_lut, fresh_config, decision_receipt, report)
        }
        [command, out_grade, out_fresh_gate] if command == "m1-close-grade" => {
            refuse_existing(&[out_grade, out_fresh_gate])?;
            let (audit, fz_gate_receipt) = inst::m1_close::canonical_m1_saved_model_audit()?;
            write_new(out_grade, &json_bytes(&audit)?)?;
            write_new(out_fresh_gate, &json_bytes(&fz_gate_receipt)?)?;
            println!(
                "m1-close-grade: m1_close_pass {}, status {}, fz dev_decision {}",
                audit["m1_close_pass"],
                audit["m1_milestone_status"],
                fz_gate_receipt["dev_decision"]
            );
            Ok(())
        }
        [command, artifact, report, flags @ ..] if command == "toks" => {
            let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
            let model = IntegerDiscreteModel::from_bytes(&bytes)
                .map_err(|e| format!("the integer reader refused the artifact: {e}"))?;
            let protocol = inst::toks::STEP6;
            let (prompt, source) = match flags {
                [] => (
                    inst::toks::default_prompt(model.shape().vocabulary, protocol.prompt_tokens),
                    "default: token i = (7919 i + 13) mod vocabulary".to_owned(),
                ),
                [flag, path] if flag == "--prompt" => {
                    let tokens: Vec<u32> = serde_json::from_value(read_json(path)?)
                        .map_err(|e| format!("{path}: a JSON array of token ids: {e}"))?;
                    (tokens, format!("--prompt {path}"))
                }
                _ => return Err("toks takes only --prompt TOKENS_JSON after its paths".into()),
            };
            let mut timing = inst::toks::time_serving(&model, &prompt, protocol)?;
            timing["prompt_source"] = serde_json::json!(source);
            timing["hardware"] = inst::toks::hardware_profile();
            write_new(report, &json_bytes(&timing)?)?;
            println!(
                "prefill median {} tok/s (p95 {}), decode median {} tok/s (p95 {})",
                timing["prefill"]["median_tokens_per_s"],
                timing["prefill"]["p95_tokens_per_s"],
                timing["decode"]["median_tokens_per_s"],
                timing["decode"]["p95_tokens_per_s"]
            );
            Ok(())
        }
        _ => Err("usage: d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON \
                  | export-trunk CHECKPOINT_JSON CONFIG_JSON OUT_LUT REPORT_JSON | rows CONFIG_JSON OUT_JSON \
                  | score ARTIFACT_LUT CONFIG_JSON REPORT_JSON ROWS_JSONL [--reference] [--train-eval TRAIN_EVAL_JSON GAP_REPORT_JSON] \
                  | fresh-score SEED1_LUT SEED2_LUT FRESH_CONFIG_JSON DECISION_RECEIPT_JSON REPORT_JSON \
                  | m1-close-grade OUT_GRADE_JSON OUT_FRESH_GATE_JSON \
                  | toks ARTIFACT_LUT REPORT_JSON [--prompt TOKENS_JSON]"
            .into()),
    }
}

#[cfg(not(feature = "cuda"))]
fn export_trunk(_: &str, _: &str, _: &str, _: &str) -> Result<(), String> {
    Err("export-trunk runs on the GPU: build with --features cuda (and UOR_NVCC_ARCH)".into())
}

#[cfg(feature = "cuda")]
fn export_trunk(checkpoint: &str, config: &str, out: &str, report: &str) -> Result<(), String> {
    use inst::device::DeviceScorer;
    use inst::fresh_transfer::MATCHED_TRAIN_BATCHES;
    refuse_existing(&[out, report])?;
    let ckpt_val = read_json(checkpoint)?;
    let cfg: DriverConfig =
        serde_json::from_value(read_json(config)?).map_err(|e| format!("{config}: {e}"))?;
    let exported = inst::export_trunk_with_scorer(
        &ckpt_val,
        &cfg,
        MATCHED_TRAIN_BATCHES,
        |tables, packed| DeviceScorer::new(0, tables)?.score(packed),
    )?;
    write_new(out, &exported.bytes)?;
    write_new(report, &json_bytes(&exported.report)?)?;
    println!(
        "exported trunk {} bytes, file sha256 {}, train_rows {}, distinct_read_codes {}, distinct_rec_codes {}",
        exported.bytes.len(),
        exported.report["file_sha256"],
        exported.report["train_rows_captured"],
        exported.report["distinct_train_read_codes"],
        exported.report["distinct_train_recurrence_codes"]
    );
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn fresh_score(
    seed1_lut: &str,
    seed2_lut: &str,
    fresh_config: &str,
    decision_receipt: &str,
    report: &str,
) -> Result<(), String> {
    use inst::artifact::ArtifactTables;
    refuse_existing(&[report])?;
    let receipt = read_json(decision_receipt)?;
    let config: DriverConfig = serde_json::from_value(read_json(fresh_config)?)
        .map_err(|e| format!("{fresh_config}: {e}"))?;
    let bytes1 = std::fs::read(seed1_lut).map_err(|e| format!("{seed1_lut}: {e}"))?;
    let (tables1, _) = ArtifactTables::from_bytes(&bytes1)?;
    let bytes2 = std::fs::read(seed2_lut).map_err(|e| format!("{seed2_lut}: {e}"))?;
    let (tables2, _) = ArtifactTables::from_bytes(&bytes2)?;
    inst::fresh_transfer::validate_fresh_transfer_gate(
        &receipt,
        &tables1.artifact_sha256,
        &config.dev,
    )?;
    inst::fresh_transfer::validate_fresh_transfer_gate(
        &receipt,
        &tables2.artifact_sha256,
        &config.dev,
    )?;
    Err("fresh-score runs on the GPU: build with --features cuda (and UOR_NVCC_ARCH)".into())
}

#[cfg(feature = "cuda")]
fn fresh_score(
    seed1_lut: &str,
    seed2_lut: &str,
    fresh_config: &str,
    decision_receipt: &str,
    report: &str,
) -> Result<(), String> {
    use inst::artifact::ArtifactTables;
    use inst::device::DeviceScorer;
    use inst::fresh_transfer::{
        build_train_count_tables, evaluate_fresh_transfer_two_seeds,
        score_fresh_against_count_tables, validate_fresh_transfer_gate, MATCHED_TRAIN_BATCHES,
    };
    use uor_r4_discrete::corpus::PreparedCorpus;

    refuse_existing(&[report])?;
    let receipt = read_json(decision_receipt)?;
    let config: DriverConfig = serde_json::from_value(read_json(fresh_config)?)
        .map_err(|e| format!("{fresh_config}: {e}"))?;
    let bytes1 = std::fs::read(seed1_lut).map_err(|e| format!("{seed1_lut}: {e}"))?;
    let (tables1, model1) = ArtifactTables::from_bytes(&bytes1)?;
    let bytes2 = std::fs::read(seed2_lut).map_err(|e| format!("{seed2_lut}: {e}"))?;
    let (tables2, model2) = ArtifactTables::from_bytes(&bytes2)?;

    validate_fresh_transfer_gate(&receipt, &tables1.artifact_sha256, &config.dev)?;
    validate_fresh_transfer_gate(&receipt, &tables2.artifact_sha256, &config.dev)?;

    let rows = inst::fresh_rows(&config, &receipt, &tables1.artifact_sha256)?;
    let train_corpus = PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    let count_tables = build_train_count_tables(
        &train_corpus,
        config.spec.context,
        config.batch_rows,
        MATCHED_TRAIN_BATCHES,
    )?;

    let views: Vec<&[u32]> = rows.iter().map(|r| r.prefix.as_slice()).collect();
    let targets: Vec<u32> = rows.iter().map(|r| r.target_token).collect();
    let packed = inst::kernel::pack_rows_with_targets(&views, &targets)?;

    let scorer1 = DeviceScorer::new(0, &tables1)?;
    let scores1 = scorer1.score(&packed)?;
    let ref1 = inst::reference_check(&model1, &rows, &scores1);
    let ref1_mismatches = ref1["checked"]
        .as_u64()
        .unwrap_or(0)
        .saturating_sub(ref1["agreed"].as_u64().unwrap_or(0));
    let b4_1 = inst::planted_fault_control(&bytes1, &tables1, &rows, &scores1, |ft| {
        DeviceScorer::new(0, ft)?.score(&packed)
    })?;
    let b4_1_ok = b4_1["b4_pass"].as_bool() == Some(true);

    let scorer2 = DeviceScorer::new(0, &tables2)?;
    let scores2 = scorer2.score(&packed)?;
    let ref2 = inst::reference_check(&model2, &rows, &scores2);
    let ref2_mismatches = ref2["checked"]
        .as_u64()
        .unwrap_or(0)
        .saturating_sub(ref2["agreed"].as_u64().unwrap_or(0));
    let b4_2 = inst::planted_fault_control(&bytes2, &tables2, &rows, &scores2, |ft| {
        DeviceScorer::new(0, ft)?.score(&packed)
    })?;
    let b4_2_ok = b4_2["b4_pass"].as_bool() == Some(true);

    let seed1_score = score_fresh_against_count_tables(
        1,
        &tables1.artifact_sha256,
        &count_tables,
        &rows,
        &scores1,
        ref1_mismatches,
        b4_1_ok,
    )?;
    let seed2_score = score_fresh_against_count_tables(
        2,
        &tables2.artifact_sha256,
        &count_tables,
        &rows,
        &scores2,
        ref2_mismatches,
        b4_2_ok,
    )?;

    let out = evaluate_fresh_transfer_two_seeds(&seed1_score, &seed2_score)?;
    write_new(report, &json_bytes(&out)?)?;
    println!(
        "fresh-score: outcome {}, seed1 top1 {}, seed2 top1 {}, unigram {}, t2 {}",
        out["outcome"],
        seed1_score.model_top1,
        seed2_score.model_top1,
        seed1_score.unigram_top1,
        seed1_score.t2_top1
    );
    Ok(())
}

#[cfg(not(feature = "cuda"))]
fn score(
    _: &str,
    _: &str,
    _: &str,
    _: &str,
    _: bool,
    _: Option<(&str, &str)>,
) -> Result<(), String> {
    Err("score runs on the GPU: build with --features cuda (and UOR_NVCC_ARCH)".into())
}

#[cfg(feature = "cuda")]
fn score(
    artifact: &str,
    config: &str,
    report: &str,
    row_records: &str,
    reference: bool,
    train_eval: Option<(&str, &str)>,
) -> Result<(), String> {
    use inst::artifact::ArtifactTables;
    use inst::device::{self, DeviceScorer};
    if let Some((_, gap_report)) = train_eval {
        refuse_existing(&[report, row_records, gap_report])?;
    } else {
        refuse_existing(&[report, row_records])?;
    }
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let (tables, model) = ArtifactTables::from_bytes(&bytes)?;
    let config: DriverConfig =
        serde_json::from_value(read_json(config)?).map_err(|e| format!("{config}: {e}"))?;
    let shape = model.shape();
    if config.spec.vocabulary != shape.vocabulary || config.spec.context != shape.context {
        return Err("the config's vocabulary or context differs from the artifact's".into());
    }
    let rows = inst::dev_rows(&config)?;
    let views: Vec<&[u32]> = rows.iter().map(|row| row.prefix.as_slice()).collect();
    let targets: Vec<u32> = rows.iter().map(|row| row.target_token).collect();
    let packed = inst::kernel::pack_rows_with_targets(&views, &targets)?;
    let scorer = DeviceScorer::new(0, &tables)?;
    let started = std::time::Instant::now();
    let scores = scorer.score(&packed)?;
    let elapsed_us = u64::try_from(started.elapsed().as_micros()).map_err(|e| e.to_string())?;
    let (mut summary, lines) = inst::score_summary(&tables, &rows, &scores)?;
    summary["device"] = serde_json::json!({
        "ordinal": 0,
        "compute_capability": [scorer.capability.0, scorer.capability.1],
        "image_target": device::IMAGE_TARGET,
        "cubin_sha256": device::cubin_sha256(),
        "ptx_sha256": device::ptx_sha256(),
        "resources": device::RESOURCES,
    });
    summary["score_elapsed_us"] = serde_json::json!(elapsed_us);
    let b4 = inst::planted_fault_control(&bytes, &tables, &rows, &scores, |fault_tables| {
        DeviceScorer::new(0, fault_tables)?.score(&packed)
    })?;
    summary["planted_fault_b4"] = b4.clone();
    let mut mismatch = false;
    if reference {
        let check = inst::reference_check(&model, &rows, &scores);
        mismatch = check["agreed"] != check["checked"];
        summary["reference"] = check;
    }
    if let Some((train_eval_path, gap_report_path)) = train_eval {
        let train_eval_json = read_json(train_eval_path)?;
        let gap = inst::compare_with_train_eval(&summary, &rows, &scores, &train_eval_json, &b4)?;
        let exact_zero = gap["exact_zero_gap"].as_bool() == Some(true);
        let b4_ok = gap["b4_pass"].as_bool() == Some(true);
        write_new(gap_report_path, &json_bytes(&gap)?)?;
        if !exact_zero || !b4_ok {
            write_new(row_records, &lines)?;
            write_new(report, &json_bytes(&summary)?)?;
            return Err(format!(
                "export gap or planted-fault B4 failed: exact_zero_gap={exact_zero}, b4_pass={b4_ok}"
            ));
        }
    }
    write_new(row_records, &lines)?;
    write_new(report, &json_bytes(&summary)?)?;
    println!(
        "{} rows: top1 {}, top8 {}, admitted {}, loss_ta {}, loss_tr {}, refused {}",
        summary["rows"],
        summary["top1"],
        summary["top8"],
        summary["admitted"],
        summary["loss_ta_numerator_sum"],
        summary["loss_tr_numerator_sum"],
        summary["refused"]
    );
    if mismatch {
        return Err(format!(
            "the GPU scores differ from observe_target: {}",
            summary["reference"]["first_mismatch"]
        ));
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("refused: {e}");
            ExitCode::from(1)
        }
    }
}
