//! Evaluate a `UORLUT01` SpiralCore/FBS artifact on the GPU (claude-128).
//!
//! ```text
//! d11-fbs-eval qualify OUT_JSON
//! d11-fbs-eval run ARTIFACT SEQUENCES_JSON OUT_JSON [--verify]
//! d11-fbs-eval windows READOUT_JSON OUT_JSON
//! d11-fbs-eval singles ARTIFACT OUT_JSON
//! ```
//!
//! `windows` writes the token windows of a float readout
//! (`sequences[].tokens`, all of one length) as a sequences file; `singles`
//! writes every token of the artifact's vocabulary as one-token sequences.
//!
//! `qualify` evaluates the synthetic FBS controls on the GPU and holds every
//! position against the host twin and the Rust integer session (bit for bit).
//! `run` evaluates every sequence of `SEQUENCES_JSON` (`{"sequences": [[..]]}`
//! or a bare array, all of one length) from a fresh session, and writes the
//! per-position tokens, kept cells and Q12 queries with the artifact's,
//! PTX's and cubin's sha256; with `--verify` it also holds them against the
//! host twin and the Rust session. The output is never overwritten.

use std::io::Write;
use std::process::ExitCode;
use std::time::Instant;

use d11_fbs_evaluator::device::{self, DeviceEvaluator};
use d11_fbs_evaluator::{first_difference, host_sequences, session_sequences, sha256_hex, Outputs};
use serde_json::{json, Value};
use uor_r4_integer::stack::synthetic::{
    fbs_synthetic_non_square, fbs_synthetic_tiny_gain, synthetic_fbs_trunk,
    synthetic_fbs_trunk_with, FbsSynth, Lcg, FBS_SYNTH_NON_SQUARE,
};
use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, IntegerStackModel};

const USAGE: &str = "usage: d11-fbs-eval qualify OUT_JSON | run ARTIFACT SEQUENCES_JSON OUT_JSON [--verify] | windows READOUT_JSON OUT_JSON | singles ARTIFACT OUT_JSON";

fn write_new(path: &str, value: &Value) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| format!("{path}: {e}"))?;
    let text = serde_json::to_string(value).map_err(|e| e.to_string())?;
    file.write_all(text.as_bytes())
        .map_err(|e| format!("{path}: {e}"))
}

fn compare(name: &str, a: &Outputs, b: &Outputs) -> Value {
    let difference = first_difference(a, b);
    json!({
        "against": name,
        "positions": a.sequences * a.len,
        "equal": difference.is_none() && a == b,
        "first_difference": difference.map(|(s, t)| json!({"sequence": s, "position": t})),
    })
}

fn verify(
    model: &IntegerStackModel,
    sequences: &[Vec<u32>],
    gpu: &Outputs,
) -> Result<Value, String> {
    let host = host_sequences(model, sequences)?;
    let session = session_sequences(model, sequences)?;
    Ok(json!([
        compare("host twin", gpu, &host),
        compare("Rust session", gpu, &session)
    ]))
}

fn all_equal(checks: &Value) -> bool {
    checks
        .as_array()
        .is_some_and(|c| c.iter().all(|v| v["equal"] == json!(true)))
}

fn hashes(evaluator: &DeviceEvaluator) -> Value {
    json!({
        "ptx_sha256": device::ptx_sha256(),
        "ptxas": device::RESOURCES,
        "cubin_sha256": device::cubin_sha256(),
        "image_target": device::IMAGE_TARGET,
        "capability": [evaluator.capability.0, evaluator.capability.1],
    })
}

fn draw(vocab: usize, count: usize, len: usize, seed: u64) -> Vec<Vec<u32>> {
    let mut rng = Lcg(seed);
    (0..count)
        .map(|_| {
            (0..len)
                .map(|_| (rng.next() % vocab as u64) as u32)
                .collect()
        })
        .collect()
}

fn qualify(out: &str) -> Result<bool, String> {
    let long = synthetic_fbs_trunk_with(
        20261027,
        FbsSynth {
            context: 64,
            ..FBS_SYNTH_NON_SQUARE
        },
    );
    let controls = [
        ("small", synthetic_fbs_trunk(20261025)),
        ("non-square", fbs_synthetic_non_square(20261026)),
        ("tiny-gain", fbs_synthetic_tiny_gain(20261026)),
        ("non-square, 64 positions", long),
    ];
    let mut records = Vec::new();
    let mut pass = true;
    let mut shared = Value::Null;
    for (name, snap) in &controls {
        let bytes = export_fbs_trunk_uorlut01(snap).map_err(|e| e.to_string())?;
        let model = IntegerStackModel::parse(&bytes).map_err(|e| e.to_string())?;
        let evaluator = DeviceEvaluator::new(0, &model)?;
        shared = hashes(&evaluator);
        let singles: Vec<Vec<u32>> = (0..snap.vocab as u32).map(|t| vec![t]).collect();
        let runs = [
            ("single tokens", singles),
            ("sequences", draw(snap.vocab, 16, snap.context, 7)),
        ];
        for (kind, sequences) in runs {
            let start = Instant::now();
            let gpu = evaluator.evaluate(&sequences)?;
            let gpu_ns = start.elapsed().as_nanos() as u64;
            let checks = verify(&model, &sequences, &gpu)?;
            let equal = all_equal(&checks);
            pass &= equal;
            eprintln!(
                "{name}, {kind}: {} positions, equal {equal}",
                gpu.sequences * gpu.len
            );
            records.push(json!({
                "control": name,
                "kind": kind,
                "artifact_sha256": sha256_hex(&bytes),
                "gpu_ns": gpu_ns,
                "checks": checks,
            }));
        }
    }
    write_new(
        out,
        &json!({"schema": "uor-r4.d11-fbs-eval-qualify/1", "pass": pass, "build": shared, "controls": records}),
    )?;
    Ok(pass)
}

/// The sequences of `path` and the sha256 of the bytes they were parsed from.
fn read_sequences(path: &str) -> Result<(Vec<Vec<u32>>, String), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|e| format!("{path}: {e}"))?;
    let list = value.get("sequences").unwrap_or(&value);
    let sequences =
        serde_json::from_value(list.clone()).map_err(|e| format!("{path}: sequences: {e}"))?;
    Ok((sequences, sha256_hex(&bytes)))
}

fn windows(readout: &str, out: &str) -> Result<bool, String> {
    let text = std::fs::read_to_string(readout).map_err(|e| format!("{readout}: {e}"))?;
    let value: Value = serde_json::from_str(&text).map_err(|e| format!("{readout}: {e}"))?;
    let windows: Vec<Vec<u32>> = value["sequences"]
        .as_array()
        .ok_or(format!("{readout}: no sequences"))?
        .iter()
        .map(|s| {
            serde_json::from_value(s["tokens"].clone())
                .map_err(|e| format!("{readout}: tokens: {e}"))
        })
        .collect::<Result<_, _>>()?;
    let len = windows.first().map_or(0, Vec::len);
    if windows.is_empty() || len == 0 || windows.iter().any(|w| w.len() != len) {
        return Err(format!(
            "{readout}: the windows are empty or of unequal lengths"
        ));
    }
    eprintln!("{} windows of {len} tokens", windows.len());
    write_new(out, &json!({ "sequences": windows }))?;
    Ok(true)
}

fn singles(artifact: &str, out: &str) -> Result<bool, String> {
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| e.to_string())?;
    let vocab = model.shape().vocab as u32;
    let sequences: Vec<Vec<u32>> = (0..vocab).map(|t| vec![t]).collect();
    eprintln!("{vocab} single tokens");
    write_new(out, &json!({ "sequences": sequences }))?;
    Ok(true)
}

fn run(artifact: &str, sequences_path: &str, out: &str, check: bool) -> Result<bool, String> {
    let bytes = std::fs::read(artifact).map_err(|e| format!("{artifact}: {e}"))?;
    let model = IntegerStackModel::parse(&bytes).map_err(|e| e.to_string())?;
    let (sequences, sequences_sha256) = read_sequences(sequences_path)?;
    if sequences.is_empty() || sequences[0].is_empty() {
        return Err(format!("{sequences_path}: no sequences, or empty ones"));
    }
    let evaluator = DeviceEvaluator::new(0, &model)?;
    let start = Instant::now();
    let gpu = evaluator.evaluate(&sequences)?;
    let gpu_ns = start.elapsed().as_nanos() as u64;
    let checks = if check {
        verify(&model, &sequences, &gpu)?
    } else {
        Value::Null
    };
    let pass = !check || all_equal(&checks);
    eprintln!(
        "{} sequences of {} positions in {gpu_ns} ns{}",
        gpu.sequences,
        gpu.len,
        if check {
            format!(", verify equal {pass}")
        } else {
            String::new()
        }
    );
    write_new(
        out,
        &json!({
            "schema": "uor-r4.d11-fbs-eval/1",
            "artifact_sha256": sha256_hex(&bytes),
            "sequences_sha256": sequences_sha256,
            "build": hashes(&evaluator),
            "context": model.shape().context,
            "gpu_ns": gpu_ns,
            "sequences": gpu.sequences,
            "len": gpu.len,
            "kept": gpu.kept,
            "width": gpu.width,
            "tokens": gpu.tokens,
            "kept_cells": gpu.kept_cells,
            "queries": gpu.queries,
            "verify": checks,
        }),
    )?;
    Ok(pass)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["qualify", out] => qualify(out),
        ["run", artifact, sequences, out] => run(artifact, sequences, out, false),
        ["run", artifact, sequences, out, "--verify"] => run(artifact, sequences, out, true),
        ["windows", readout, out] => windows(readout, out),
        ["singles", artifact, out] => singles(artifact, out),
        _ => Err(USAGE.to_owned()),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!("d11-fbs-eval: the GPU departs from a reference (see the output)");
            ExitCode::from(1)
        }
        Err(e) => {
            eprintln!("d11-fbs-eval: {e}");
            ExitCode::from(2)
        }
    }
}
