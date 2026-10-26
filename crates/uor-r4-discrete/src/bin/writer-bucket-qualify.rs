//! Finite CUDA qualification runner for the CSR bucketed writer (`m1-faster-writer`, #2840).
//! A separate binary so `driver_source_sha256` is unchanged.
//!
//! Usage: writer-bucket-qualify DEV_TOKENS DEV_MASK A8_TABLE PZ_TABLE OUTPUT_JSON [--ordinal N]
//! OUTPUT_JSON must not exist; it is created exclusively.

#![forbid(unsafe_code)]
#![deny(warnings, clippy::all)]

use std::process::ExitCode;

#[cfg(feature = "cuda")]
fn run_cuda(args: &[String]) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::Path;
    use uor_r4_discrete::writer_bucket_qualification::{run_qualification, QUALIFICATION_SCHEMA};

    if args.len() != 5 && args.len() != 7 {
        return Err(
            "usage: writer-bucket-qualify DEV_TOKENS DEV_MASK A8_TABLE PZ_TABLE OUTPUT_JSON [--ordinal N]"
                .to_owned(),
        );
    }
    let ordinal = if args.len() == 7 {
        if args[5] != "--ordinal" {
            return Err(format!("unknown flag {}", args[5]));
        }
        args[6]
            .parse::<usize>()
            .map_err(|e| format!("--ordinal {}: {e}", args[6]))?
    } else {
        0usize
    };

    let dev_tokens = Path::new(&args[0]);
    let dev_mask = Path::new(&args[1]);
    let a8_table = Path::new(&args[2]);
    let pz_table = Path::new(&args[3]);
    let output = Path::new(&args[4]);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|e| format!("exclusive {}: {e}", output.display()))?;

    let result = run_qualification(ordinal, dev_tokens, dev_mask, a8_table, pz_table)
        .map_err(|e| e.to_string());
    let payload = match &result {
        Ok(report) => serde_json::to_value(report).map_err(|e| e.to_string())?,
        Err(error) => serde_json::json!({
            "schema": QUALIFICATION_SCHEMA,
            "status": "FAILED",
            "error": error,
        }),
    };
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    result.map(|_| ())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    #[cfg(feature = "cuda")]
    {
        match run_cuda(&args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        let _ = args;
        eprintln!("writer-bucket-qualify requires --features cuda");
        ExitCode::FAILURE
    }
}
