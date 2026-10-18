//! Codebook coverage census for qualification packets (D39 item 2).
//! A separate binary so the training driver's source identity is unchanged.
//! Host-only: grouping and counting over stored data; no model execution.
//!
//! Usage: coverage-census CONFIG CHECKPOINT OUTPUT [--token-cells RECEIPT] [--bar-rows ROWS]
//! OUTPUT must not exist; it is created exclusively.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn run(args: &[String]) -> Result<(), String> {
    let (positional, flags) = args.split_at(args.len().min(3));
    if positional.len() != 3 {
        return Err(
            "usage: coverage-census CONFIG CHECKPOINT OUTPUT [--token-cells RECEIPT] [--bar-rows ROWS]"
                .into(),
        );
    }
    let mut cells = None;
    let mut bar = None;
    let mut rest = flags.iter();
    while let Some(flag) = rest.next() {
        let value = rest.next().ok_or(format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--token-cells" if cells.is_none() => {
                cells = Some(fs::read(value).map_err(|e| format!("{value}: {e}"))?)
            }
            "--bar-rows" if bar.is_none() => {
                bar = Some(
                    value
                        .parse::<u64>()
                        .map_err(|e| format!("--bar-rows {value}: {e}"))?,
                )
            }
            "--token-cells" | "--bar-rows" => return Err(format!("{flag} given twice")),
            other => return Err(format!("unknown flag {other}")),
        }
    }
    let read = |p: &str| fs::read(p).map_err(|e| format!("{p}: {e}"));
    let config = serde_json::from_slice(&read(&positional[0])?).map_err(|e| e.to_string())?;
    let checkpoint = read(&positional[1])?;
    // Reserve the report before loading any corpus or checkpoint content.
    let output = Path::new(&positional[2]);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("exclusive {}: {e}", output.display()))?;
    // A refusal is sealed into the reserved report, never left as an empty file.
    let result = uor_r4_discrete::coverage_census::run(config, &checkpoint, cells.as_deref(), bar)
        .map_err(|e| e.to_string());
    let report = match &result {
        Ok(report) => report.clone(),
        Err(error) => serde_json::json!({"schema": "uor-r4.coverage-census/1",
            "status": "FAILED", "error": error, "model_execution": false}),
    };
    let bytes = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    result.map(|_| println!("{}", String::from_utf8_lossy(&bytes)))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
