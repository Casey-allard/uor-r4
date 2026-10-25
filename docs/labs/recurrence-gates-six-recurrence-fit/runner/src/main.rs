//! CLI runner for `recurrence-gates-six-recurrence-fit` (`#2890`).
//!
//! Usage:
//!   recurrence-six-fit --mode synthetic-probe --out <path>
//!   recurrence-six-fit --mode throughput-probe --data-root <dir> --out <path>
//!   recurrence-six-fit --mode full --data-root <dir> --out <path>

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use candle_core::Device;
use recurrence_gates_six_recurrence_fit_runner::{
    run_full_six_recurrence_sheet, run_synthetic_probe, run_throughput_probe,
};

fn main() -> ExitCode {
    let mut mode = String::from("full");
    let mut data_root = PathBuf::from("local/w2-inputs/2026-10-25/driver-data");
    let mut out_path: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                let Some(val) = args.next() else {
                    eprintln!("missing value for --mode");
                    return ExitCode::from(2);
                };
                mode = val;
            }
            "--data-root" => {
                let Some(val) = args.next() else {
                    eprintln!("missing value for --data-root");
                    return ExitCode::from(2);
                };
                data_root = PathBuf::from(val);
            }
            "--out" => {
                let Some(val) = args.next() else {
                    eprintln!("missing value for --out");
                    return ExitCode::from(2);
                };
                out_path = Some(PathBuf::from(val));
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::from(2);
            }
        }
    }

    if let Some(ref out) = out_path {
        if out.exists() {
            eprintln!(
                "refusing to overwrite existing output file {} (L-002)",
                out.display()
            );
            return ExitCode::from(2);
        }
    }

    let device = match Device::new_cuda(0) {
        Ok(d) => d,
        Err(err) => {
            eprintln!("CUDA device 0 required (CHARTER §3.5 / D24): {err}");
            return ExitCode::from(1);
        }
    };

    let result_json = match mode.as_str() {
        "synthetic-probe" => match run_synthetic_probe(&device) {
            Ok(v) => serde_json::to_string_pretty(&v),
            Err(e) => {
                eprintln!("synthetic-probe failed: {e}");
                return ExitCode::from(1);
            }
        },
        "throughput-probe" => match run_throughput_probe(Path::new(&data_root), &device) {
            Ok(v) => serde_json::to_string_pretty(&v),
            Err(e) => {
                eprintln!("throughput-probe failed: {e}");
                return ExitCode::from(1);
            }
        },
        "full" => match run_full_six_recurrence_sheet(Path::new(&data_root), &device) {
            Ok(receipt) => serde_json::to_string_pretty(&receipt),
            Err(e) => {
                eprintln!("full six-recurrence sheet failed: {e}");
                return ExitCode::from(1);
            }
        },
        other => {
            eprintln!(
                "unsupported --mode {other}; expected synthetic-probe, throughput-probe, or full"
            );
            return ExitCode::from(2);
        }
    };

    let mut text = match result_json {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JSON serialization error: {e}");
            return ExitCode::from(1);
        }
    };
    text.push('\n');

    if let Some(out) = out_path {
        if let Some(parent) = out.parent() {
            if !parent.as_os_str().is_empty() {
                if let Err(e) = fs::create_dir_all(parent) {
                    eprintln!(
                        "failed to create parent directory {}: {e}",
                        parent.display()
                    );
                    return ExitCode::from(1);
                }
            }
        }
        if let Err(e) = fs::write(&out, &text) {
            eprintln!("failed to write {}: {e}", out.display());
            return ExitCode::from(1);
        }
        println!("wrote {}", out.display());
    } else {
        print!("{text}");
    }

    ExitCode::SUCCESS
}
