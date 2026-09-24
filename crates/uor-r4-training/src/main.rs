#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::json;
use uor_r4_training::{run_integrity, IntegrityRequest};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 6 || args[0] != "integrity" {
        return Err("usage: uor-r4-training integrity SNAPSHOT TOKENS_U16 OUT_JSON {cpu|metal} WINDOW\nWINDOW is 8..=64; reads exactly WINDOW+1 retained tokens from offset zero; no optimization steps".into());
    }
    let request = IntegrityRequest {
        snapshot: args[1].clone().into(),
        tokens: args[2].clone().into(),
        device: args[4].clone(),
        window: args[5].parse()?,
    };
    request.validate()?;
    let output = PathBuf::from(&args[3]);
    // Single-file report is exclusively claimed before any model or token load.
    let mut file = fs::File::create_new(&output)?;
    match run_integrity(&request) {
        Ok(report) => {
            serde_json::to_writer_pretty(&mut file, &report)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            println!("{}: {}; loss={:.6}; max logit delta={:.8}; Q/K/V gradients={}; finite differences={}",
                report.status, output.display(), report.next_token_loss_nats,
                report.parity.max_absolute_delta, report.all_qkv_gradients_present_finite_nonzero,
                report.finite_differences_pass);
            if report.status != "INTEGRITY_PASS" {
                return Err("reference integrity criteria failed; complete report retained".into());
            }
            Ok(())
        }
        Err(error) => {
            serde_json::to_writer_pretty(
                &mut file,
                &json!({"schema":"uor-r4.training-reference-integrity/1", "status":"FAILED_ATTEMPT", "error":error.to_string(), "optimizer_steps":0, "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND")}),
            )?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            Err(error.into())
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
