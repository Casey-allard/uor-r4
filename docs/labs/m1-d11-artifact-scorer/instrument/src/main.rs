//! `d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON`
//! `d11-scorer rows CONFIG_JSON OUT_JSON`
//! `d11-scorer score ARTIFACT_LUT CONFIG_JSON REPORT_JSON ROWS_JSONL [--reference]`
//! `d11-scorer toks ARTIFACT_LUT REPORT_JSON [--prompt TOKENS_JSON]`
//!
//! `toks` times `IntegerDiscreteModel` serving on this machine by the step-6
//! protocol (1 warmup, 20 runs, 192-token prefill, 64-token decode).
//!
//! `m1-d11-artifact-scorer` (#3248). `score` runs on the GPU (`cuda` feature)
//! over the config's DEV rows; `--reference` also checks every row against
//! `IntegerDiscreteModel::evaluate_prefix` and refuses on any mismatch. Every
//! output is created new: an existing file is never overwritten.

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
            println!("exported {} bytes, file sha256 {}", exported.bytes.len(), exported.report["file_sha256"]);
            Ok(())
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
            let reference = match flags {
                [] => false,
                [flag] if flag == "--reference" => true,
                _ => return Err("score takes only --reference after its paths".into()),
            };
            score(artifact, config, report, row_records, reference)
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
        _ => Err("usage: d11-scorer export CHECKPOINT_JSON OUT_LUT REPORT_JSON | rows CONFIG_JSON OUT_JSON \
                  | score ARTIFACT_LUT CONFIG_JSON REPORT_JSON ROWS_JSONL [--reference] \
                  | toks ARTIFACT_LUT REPORT_JSON [--prompt TOKENS_JSON]"
            .into()),
    }
}

#[cfg(not(feature = "cuda"))]
fn score(_: &str, _: &str, _: &str, _: &str, _: bool) -> Result<(), String> {
    Err("score runs on the GPU: build with --features cuda (and UOR_NVCC_ARCH)".into())
}

#[cfg(feature = "cuda")]
fn score(
    artifact: &str,
    config: &str,
    report: &str,
    row_records: &str,
    reference: bool,
) -> Result<(), String> {
    use inst::artifact::ArtifactTables;
    use inst::device::{self, DeviceScorer};
    refuse_existing(&[report, row_records])?;
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
    let packed = inst::kernel::pack_rows(&views)?;
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
    let mut mismatch = false;
    if reference {
        let check = inst::reference_check(&model, &rows, &scores);
        mismatch = check["agreed"] != check["checked"];
        summary["reference"] = check;
    }
    write_new(row_records, &lines)?;
    write_new(report, &json_bytes(&summary)?)?;
    println!(
        "{} rows: top1 {}, top8 {}, admitted {}, refused {}",
        summary["rows"], summary["top1"], summary["top8"], summary["admitted"], summary["refused"]
    );
    if mismatch {
        return Err(format!(
            "the GPU scores differ from evaluate_prefix: {}",
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
