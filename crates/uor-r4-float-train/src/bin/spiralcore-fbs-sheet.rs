//! `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full --data-root DIR --out FILE`
//!
//! The runner of `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`
//! (spiralcore-fbs-offline-float-sheet, #2891). GPU only. `--data-root` holds `mw-balp/train/`,
//! `ft/dev/` and (for `fresh_v2`, read only when its gates pass) `fresh_v2/`, each with
//! `tokens.u16` and `response_mask.u8`. The receipt is written to `--out`, never over a file.

use std::env;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::spiralcore_fbs_sheet::{
    run_full_sheet, run_synthetic_probe, run_throughput_probe, write_receipt,
};
use uor_r4_float_train::{FloatTrainError, Result};

fn parse_args() -> Result<(String, PathBuf, PathBuf)> {
    let mut mode = String::from("full");
    let mut data_root = PathBuf::from(".");
    let mut out = PathBuf::from("spiralcore-fbs-sheet-receipt.json");
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| {
            args.next()
                .ok_or_else(|| FloatTrainError::Invalid(format!("missing {name} value")))
        };
        match arg.as_str() {
            "--mode" => mode = value("--mode")?,
            "--data-root" => data_root = PathBuf::from(value("--data-root")?),
            "--out" => out = PathBuf::from(value("--out")?),
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )))
            }
        }
    }
    Ok((mode, data_root, out))
}

fn main() -> Result<()> {
    let (mode, data_root, out) = parse_args()?;
    let device = Device::new_cuda(0)?;
    match mode.as_str() {
        "synthetic-probe" => {
            let value = run_synthetic_probe(&device)?;
            println!("[synthetic-probe] pass={}", value["pass"]);
            write_receipt(&out, &value)
        }
        "throughput-probe" => {
            let value = run_throughput_probe(&data_root, &device)?;
            println!(
                "[throughput-probe] dev_denominator={} projected_full_s={} under_one_hour_per_arm={}",
                value["i_dev_denominator_pass"],
                value["projected_full_seconds"],
                value["under_one_hour_per_arm"]
            );
            write_receipt(&out, &value)
        }
        "full" => {
            let receipt = run_full_sheet(&data_root, &device)?;
            for run in &receipt.runs {
                let last = run.final_point()?;
                println!(
                    "[full] arm={} seed={} dev_loss_f32={:.6} dev_loss_int={:.6} top1_f32={}/{} top1_int={} mobility={:.4} churn={:.4} train_tok_s={:.0} decode_tok_s={:.1}",
                    run.arm,
                    run.seed,
                    last.dev_loss_float,
                    last.dev_loss_int,
                    last.dev_float.top1_hits,
                    last.dev_float.scored_rows,
                    last.dev_int.top1_hits,
                    run.route_mobility_fraction,
                    run.route_churn_last_step,
                    run.train_tokens_per_second,
                    run.decode_tokens_per_second,
                );
            }
            println!(
                "[summary] I-Finite={} I-DEV={} I-Route={} I-Export={} | G-Fit={} ({:?}) G-Capacity={} G-ExportGap={} G-Transfer={:?} | noise={:.6} results={}",
                receipt.i_finite_cuda_pass,
                receipt.i_dev_denominator_pass,
                receipt.i_route_mobility_pass,
                receipt.i_export_exactness.pass,
                receipt.g_sheet_fit,
                receipt.g_sheet_fit_arm,
                receipt.g_capacity_winner,
                receipt.g_export_gap,
                receipt.g_transfer,
                receipt.noise_band_dev_loss,
                receipt.results_sha256,
            );
            write_receipt(&out, &serde_json::to_value(&receipt)?)
        }
        other => Err(FloatTrainError::Invalid(format!(
            "unsupported --mode {other}"
        ))),
    }
}
