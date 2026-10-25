//! CLI runner for `hamilton-gate-offline-float-sheet` (`#2824`).
//!
//! Modes:
//! - `--mode synthetic-probe --out <json>`: Probe (a) 4-step synthetic CUDA check across all 4 arms + `I-Icosian-Closure` + `I-Q12-Int-Parity`.
//! - `--mode throughput-probe --data-root <driver-data> --out <json>`: Probe (b) 16-step training + `14,287`-row DEV evaluation timing projection.
//! - `--mode full --data-root <driver-data> --out <json>`: Probe (c) full 4-arm × 2-seed × 256-step pre-registered run.

use std::env;
use std::fs;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::hamilton_sheet::{
    run_full_hamilton_sheet, run_synthetic_probe, run_throughput_probe,
};
use uor_r4_float_train::{FloatTrainError, Result};

fn parse_args() -> Result<(String, PathBuf, PathBuf)> {
    let mut mode = String::from("full");
    let mut data_root = PathBuf::from(".");
    let mut out = PathBuf::from("hamilton-gate-sheet-receipt.json");

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                mode = args
                    .next()
                    .ok_or_else(|| FloatTrainError::Invalid("missing --mode value".into()))?;
            }
            "--data-root" => {
                data_root =
                    PathBuf::from(args.next().ok_or_else(|| {
                        FloatTrainError::Invalid("missing --data-root value".into())
                    })?);
            }
            "--out" => {
                out = PathBuf::from(
                    args.next()
                        .ok_or_else(|| FloatTrainError::Invalid("missing --out value".into()))?,
                );
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )));
            }
        }
    }
    Ok((mode, data_root, out))
}

fn main() -> Result<()> {
    let (mode, data_root, out_path) = parse_args()?;
    let device = Device::new_cuda(0)?;

    match mode.as_str() {
        "synthetic-probe" => {
            let val = run_synthetic_probe(&device)?;
            println!("[synthetic-probe] {}", serde_json::to_string(&val)?);
            fs::write(&out_path, serde_json::to_vec_pretty(&val)?)?;
            Ok(())
        }
        "throughput-probe" => {
            let val = run_throughput_probe(&data_root, &device)?;
            println!("[throughput-probe] {}", serde_json::to_string(&val)?);
            fs::write(&out_path, serde_json::to_vec_pretty(&val)?)?;
            Ok(())
        }
        "full" => {
            let receipt = run_full_hamilton_sheet(&data_root, &device)?;
            for r in &receipt.runs {
                println!(
                    "[full] arm={} seed={} loss_0={:.6} loss_256={:.6} delta={:+.6} dev_f32_top1={}/{} dev_int_top1={}/{} dev_int_top8={}/{} gap_top1={} gap_top8={} min_rho={:.6} down_gain={:.4} tok_s={:.1}",
                    r.arm,
                    r.seed,
                    r.train_loss_step0,
                    r.train_loss_step256,
                    r.train_loss_delta_256,
                    r.dev_step256_float.top1_hits,
                    r.dev_step256_float.scored_rows,
                    r.dev_step256_int_export.top1_hits,
                    r.dev_step256_int_export.scored_rows,
                    r.dev_step256_int_export.top8_hits,
                    r.dev_step256_int_export.scored_rows,
                    r.export_gap_step256.delta_top1_rows,
                    r.export_gap_step256.delta_top8_rows,
                    r.suppression_step256.min_amplitude_ratio_rho,
                    r.suppression_step256.mean_downstream_gain_ratio,
                    r.dev_step256_int_export.eval_tokens_per_second,
                );
            }
            println!(
                "[summary] I-Finite={} I-DEV-Denom={} I-Icosian={} I-Q12={} | G-Fit-Unigram={} G-Beats-Random={} G-IcosianHard-Parity={} G-ExportGap={} | Suppression={} | Verdict={}",
                receipt.i_finite_cuda_pass,
                receipt.i_dev_denominator_pass,
                receipt.i_icosian_closure_pass,
                receipt.i_q12_int_parity_pass,
                receipt.g_hamilton_fit_pass_unigram,
                receipt.g_hamilton_beats_random_null,
                receipt.g_hamilton_icosian_hard_matches_or_beats_active,
                receipt.g_hamilton_export_gap_pass,
                receipt.gate_suppression_verdict,
                receipt.overall_verdict,
            );
            fs::write(&out_path, serde_json::to_vec_pretty(&receipt)?)?;
            Ok(())
        }
        other => Err(FloatTrainError::Invalid(format!(
            "unsupported --mode {other}"
        ))),
    }
}
