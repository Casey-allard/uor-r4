//! Score one frozen reply-controller implementation on a raw-id control trace.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use one_model_end_to_end_panel::reply_control_controller::control_reply;
use one_model_end_to_end_panel::{TraceRow, EOS};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

const SCHEMA: &str = "uor-r4.reply-control-bakeoff/1";
const USAGE: &str =
    "usage: reply-control-bakeoff --trace FILE --samples FILE --policy NAME --out FILE";

type Result<T> = std::result::Result<T, String>;

#[derive(Deserialize)]
struct Sample {
    reference: Vec<u32>,
}

#[derive(Default)]
struct Tally {
    cases: usize,
    exact_complete: usize,
    truncated: usize,
    eos_position_error: usize,
    empty: usize,
    cap: usize,
    generated_tokens: usize,
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn sha256(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

fn at_eos(ids: &[u32]) -> &[u32] {
    let end = ids
        .iter()
        .position(|&token| token == EOS)
        .map_or(ids.len(), |index| index + 1);
    &ids[..end]
}

fn load_trace(path: &Path) -> Result<BTreeMap<usize, Vec<TraceRow>>> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut cases: BTreeMap<usize, Vec<TraceRow>> = BTreeMap::new();
    for (line, text) in BufReader::new(file).lines().enumerate() {
        let text = text.map_err(|e| format!("{} line {}: {e}", path.display(), line + 1))?;
        let row: TraceRow = serde_json::from_str(&text)
            .map_err(|e| format!("{} line {}: {e}", path.display(), line + 1))?;
        cases.entry(row.case).or_default().push(row);
    }
    Ok(cases)
}

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let need = |name: &str| flag(&args, name).ok_or_else(|| format!("missing {name}\n{USAGE}"));
    let trace_path = need("--trace")?;
    let samples_path = need("--samples")?;
    let policy = need("--policy")?;
    let out_path = need("--out")?;
    let trace_bytes =
        fs::read(&trace_path).map_err(|e| format!("{trace_path}: {e}"))?;
    let sample_bytes =
        fs::read(&samples_path).map_err(|e| format!("{samples_path}: {e}"))?;
    let samples: Vec<Sample> =
        serde_json::from_slice(&sample_bytes).map_err(|e| format!("{samples_path}: {e}"))?;
    let cases = load_trace(Path::new(&trace_path))?;
    if cases.len() != samples.len() {
        return Err(format!(
            "trace has {} cases, samples has {}",
            cases.len(),
            samples.len()
        ));
    }

    let mut tally = Tally::default();
    let mut rows = Vec::with_capacity(cases.len());
    for (case, trace) in &cases {
        let sample = samples
            .get(*case)
            .ok_or_else(|| format!("case {case}: no sample"))?;
        let reference = at_eos(&sample.reference);
        let output = control_reply(trace);
        let trimmed = at_eos(&output);
        let generated_eos = trimmed.iter().position(|&token| token == EOS);
        let reference_eos = reference.iter().position(|&token| token == EOS);
        let exact = trimmed == reference;
        let truncated = generated_eos.is_some()
            && reference_eos.is_some()
            && generated_eos < reference_eos;
        let eos_error = generated_eos != reference_eos;
        tally.cases += 1;
        tally.exact_complete += usize::from(exact);
        tally.truncated += usize::from(truncated);
        tally.eos_position_error += usize::from(eos_error);
        tally.empty += usize::from(trimmed.is_empty());
        tally.cap += usize::from(trimmed.len() == 48 && generated_eos.is_none());
        tally.generated_tokens += trimmed.len();
        rows.push(json!({
            "case": case,
            "exact_complete": exact,
            "truncated": truncated,
            "eos_position_error": eos_error,
            "generated": trimmed,
            "reference": reference,
        }));
    }
    let report = serde_json::to_vec_pretty(&json!({
        "schema": SCHEMA,
        "policy": policy,
        "trace_sha256": sha256(&trace_bytes),
        "samples_sha256": sha256(&sample_bytes),
        "totals": {
            "cases": tally.cases,
            "exact_complete": tally.exact_complete,
            "truncated": tally.truncated,
            "eos_position_error": tally.eos_position_error,
            "empty": tally.empty,
            "cap": tally.cap,
            "generated_tokens": tally.generated_tokens,
        },
        "rows": rows,
    }))
    .map_err(|e| e.to_string())?;
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&out_path)
        .map_err(|e| format!("{out_path}: {e}"))?;
    out.write_all(&report)
        .and_then(|_| out.write_all(b"\n"))
        .map_err(|e| format!("{out_path}: {e}"))?;
    println!(
        "{}",
        json!({
            "schema": SCHEMA,
            "policy": policy,
            "totals": {
                "cases": tally.cases,
                "exact_complete": tally.exact_complete,
                "truncated": tally.truncated,
                "eos_position_error": tally.eos_position_error,
            },
            "report_sha256": sha256(&report),
        })
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("reply-control-bakeoff: {error}");
        std::process::exit(1);
    }
}
