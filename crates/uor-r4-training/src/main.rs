#![forbid(unsafe_code)]

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::json;
use uor_r4_training::{run_integrity, IntegrityRequest};

fn run_comparison(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    use uor_r4_core::report_output;
    use uor_r4_training::{baseline_counts, baseline_protocol as protocol, reference_campaign};
    if args.len() == 2 && args[0] == "verify" {
        report_output::verify(std::path::Path::new(&args[1]))?;
        println!("complete report file set verified: {}", args[1]);
        return Ok(());
    }
    let (out_index, batch) = match args.first().map(String::as_str) {
        Some("ngram-fit") if args.len() == 3 => (2, 0),
        Some("ngram-evaluate") if args.len() == 4 => (3, 0),
        Some("reference-evaluate") if args.len() == 5 => {
            let batch: usize = args[4].parse()?;
            if !(1..=16).contains(&batch) || !["cpu", "metal"].contains(&args[3].as_str()) {
                return Err("reference device must be cpu|metal and batch 1..=16".into());
            }
            (2, batch)
        }
        Some("reference-generate") if args.len() == 4 => {
            if !["cpu", "metal"].contains(&args[3].as_str()) {
                return Err("reference device must be cpu|metal".into());
            }
            (2, 0)
        }
        _ => return Err("usage:\n  uor-r4-training ngram-fit EVALUATOR_JSON NEW_REPORT_DIR\n  uor-r4-training ngram-evaluate EVALUATOR_JSON SEALED_FIT_DIR NEW_REPORT_DIR\n  uor-r4-training reference-evaluate EVALUATOR_JSON NEW_REPORT_DIR {cpu|metal} BATCH\n  uor-r4-training reference-generate EVALUATOR_JSON NEW_REPORT_DIR {cpu|metal}\n  uor-r4-training verify SEALED_REPORT_DIR".into()),
    };
    let evaluator = protocol::load_evaluator(std::path::Path::new(&args[1]))?;
    let out = PathBuf::from(&args[out_index]);
    report_output::claim(&out)?;
    let result = (|| -> uor_r4_training::Result<()> {
        protocol::save_json(
            &out.join("run-provenance.json"),
            &protocol::base_report(&evaluator, &args[0])?,
        )?;
        protocol::save_json(&out.join("evaluator.json"), &evaluator.document)?;
        match args[0].as_str() {
            "ngram-fit" => baseline_counts::run_fit(&evaluator, &out),
            "ngram-evaluate" => {
                baseline_counts::run_evaluate(&evaluator, std::path::Path::new(&args[2]), &out)
            }
            "reference-evaluate" => {
                reference_campaign::run_evaluate(&evaluator, &out, &args[3], batch)
            }
            "reference-generate" => reference_campaign::run_generate(&evaluator, &out, &args[3]),
            _ => Err(uor_r4_training::TrainingError::Invalid(
                "unknown comparison mode".into(),
            )),
        }
    })();
    if let Err(error) = &result {
        protocol::save_json(
            &out.join("failed-attempt.json"),
            &json!({
                "status":"FAILED_ATTEMPT", "error":error.to_string(),
                "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND")
            }),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    println!("COMPLETE: {}; sealed and verified", out.display());
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|mode| mode != "integrity") {
        return run_comparison(&args);
    }
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
