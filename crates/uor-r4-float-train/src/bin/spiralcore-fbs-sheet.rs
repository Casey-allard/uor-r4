//! `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full|d11-export-gap|d11-artifact-windows|d11-artifact-gap|d11-train-probe|d11-train-export --data-root DIR --out FILE [--save-dir DIR] [options below]`
//!
//! The runner of `docs/harness/prep/spiralcore-fbs-offline-float-sheet-2026-10-25.md`
//! (spiralcore-fbs-offline-float-sheet, #2891). GPU only. `--data-root` holds `mw-balp/train/`,
//! `ft/dev/` and (for `fresh_v2`, read only when its gates pass) `fresh_v2/`, each with
//! `tokens.u16` and `response_mask.u8`. The receipt is written to `--out`, never over a file.
//!
//! `d11-export-gap` is Stage 4b of d11-exporter-float-trunk
//! (`docs/harness/prep/d11-exporter-float-trunk-stage4b-2026-10-26.md`, claude-126): it saves each
//! seed's exported model, snapshot and readouts under `--save-dir`, which must not exist yet.
//!
//! `d11-artifact-windows` writes the DEV panel's windows for the GPU evaluator of the exported
//! artifact (`d11-fbs-eval run`). `d11-artifact-gap` scores the artifact's own export gap on one
//! seed from that evaluator's run (claude-129,
//! `docs/harness/prep/d11-exporter-float-trunk-artifact-gap-2026-10-26.md`), with `--model-dir`,
//! `--evaluator`, `--windows-sha256`, `--artifact-sha256` (the seed's pinned artifact),
//! `--stage4b`, `--stored-preds` and `--seed`.
//!
//! `d11-train-probe` runs 32 D11-trained steps and projects the full run.
//! `d11-train-export` trains the export arm on both seeds and saves each model, snapshot and float
//! predictions under `--save-dir`, with Stage 4b's noise band from `--stage4b`. Both train as
//! `--train-forward` says: `d11` (the default; claude-130,
//! `docs/harness/prep/d11-exporter-float-trunk-d11-train-2026-10-26.md`) or `dual`, the plain
//! loss plus the D11-mode loss (claude-131,
//! `docs/harness/prep/d11-exporter-float-trunk-dual-train-2026-10-26.md`).

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use candle_core::Device;
use uor_r4_float_train::spiralcore_fbs_sheet::{
    d11_artifact_windows, run_d11_artifact_gap, run_d11_export_gap, run_d11_train_export,
    run_d11_train_probe, run_full_sheet, run_synthetic_probe, run_throughput_probe, write_receipt,
    TrainForward,
};
use uor_r4_float_train::{FloatTrainError, Result};

/// The options of `d11-artifact-gap`, `d11-train-probe` and `d11-train-export`.
const GAP_OPTIONS: [&str; 8] = [
    "--model-dir",
    "--evaluator",
    "--windows-sha256",
    "--artifact-sha256",
    "--stage4b",
    "--stored-preds",
    "--seed",
    "--train-forward",
];

type Args = (
    String,
    PathBuf,
    PathBuf,
    Option<PathBuf>,
    BTreeMap<String, String>,
);

fn parse_args() -> Result<Args> {
    let mut mode = String::from("full");
    let mut data_root = PathBuf::from(".");
    let mut out = PathBuf::from("spiralcore-fbs-sheet-receipt.json");
    let mut save_dir = None;
    let mut extra = BTreeMap::new();
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
            "--save-dir" => save_dir = Some(PathBuf::from(value("--save-dir")?)),
            name if GAP_OPTIONS.contains(&name) => {
                let v = value(name)?;
                extra.insert(name.to_owned(), v);
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )))
            }
        }
    }
    Ok((mode, data_root, out, save_dir, extra))
}

fn main() -> Result<()> {
    let (mode, data_root, out, save_dir, extra) = parse_args()?;
    let option = |name: &str| {
        extra
            .get(name)
            .cloned()
            .ok_or_else(|| FloatTrainError::Invalid(format!("{mode} needs {name}")))
    };
    let train_forward =
        TrainForward::parse(extra.get("--train-forward").map_or("d11", String::as_str))?;
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
        "d11-export-gap" => {
            let save_dir = save_dir.ok_or_else(|| {
                FloatTrainError::Invalid("d11-export-gap needs --save-dir".to_owned())
            })?;
            let value = run_d11_export_gap(&data_root, &save_dir, &device)?;
            for s in value["seeds"].as_array().into_iter().flatten() {
                println!(
                    "[d11-export-gap] seed={} top1_float={} top1_d11={} b={} c={} net={} F_I={} loss_float={} loss_d11={} within_gpu_half={} save_load={} determinism={} dev={} q12={}",
                    s["seed"],
                    s["float"]["top1_hits"],
                    s["d11"]["top1_hits"],
                    s["paired_float_vs_d11"]["a_hit_b_miss_c"],
                    s["paired_float_vs_d11"]["a_miss_b_hit_b"],
                    s["net_gap_rows"],
                    s["f_i"],
                    s["loss_float"],
                    s["loss_d11"],
                    s["within_band"],
                    s["i_save_load"],
                    s["i_d_determinism"],
                    s["i_dev_denominator"],
                    s["i_q12_oracle"],
                );
            }
            println!(
                "[summary] noise={} gpu_instruments={} gap_inside_band_on_both_seeds={} (the gate also needs the artifact parity)",
                value["noise_band_dev_loss"], value["gpu_instruments_pass"], value["gap_inside_band_on_both_seeds"]
            );
            write_receipt(&out, &value)
        }
        "d11-train-probe" => {
            let value = run_d11_train_probe(&data_root, train_forward, &device)?;
            println!(
                "[d11-train-probe] forward={} train_learns={} learns={} train_loss_0={} train_loss_31={} dev_loss_0={} dev_loss_32={} s_per_step={} projected_full_s={} under_one_hour_per_seed={}",
                value["train_forward"],
                value["train_learns"],
                value["learns"],
                value["train_loss_step_0"],
                value["train_loss_step_31"],
                value["dev_loss_float_step_0"],
                value["dev_loss_float_step_32"],
                value["seconds_per_step"],
                value["projected_full_seconds"],
                value["under_one_hour_per_seed"],
            );
            write_receipt(&out, &value)
        }
        "d11-train-export" => {
            let save_dir = save_dir.ok_or_else(|| {
                FloatTrainError::Invalid("d11-train-export needs --save-dir".to_owned())
            })?;
            let value = run_d11_train_export(
                &data_root,
                &save_dir,
                &PathBuf::from(option("--stage4b")?),
                train_forward,
                &device,
            )?;
            for s in value["seeds"].as_array().into_iter().flatten() {
                println!(
                    "[d11-train-export] forward={} seed={} top1_float={} top1_export_mode={} loss_float={} save_load={}",
                    s["train_forward"],
                    s["seed"],
                    s["float"]["top1_hits"],
                    s["export_mode"]["top1_hits"],
                    s["loss_float"],
                    s["i_save_load"],
                );
            }
            write_receipt(&out, &value)
        }
        "d11-artifact-windows" => {
            let value = d11_artifact_windows(&data_root)?;
            println!(
                "[d11-artifact-windows] windows={} rows={}",
                value["dev_windows"], value["scored_rows"]
            );
            write_receipt(&out, &value)
        }
        "d11-artifact-gap" => {
            let seed: u64 = option("--seed")?
                .parse()
                .map_err(|_| FloatTrainError::Invalid("--seed must be an integer".to_owned()))?;
            let value = run_d11_artifact_gap(
                &data_root,
                &PathBuf::from(option("--model-dir")?),
                &PathBuf::from(option("--evaluator")?),
                &option("--windows-sha256")?,
                &option("--artifact-sha256")?,
                &PathBuf::from(option("--stage4b")?),
                &PathBuf::from(option("--stored-preds")?),
                seed,
                &device,
            )?;
            println!(
                "[d11-artifact-gap] seed={} top1_float={} top1_artifact={} b={} c={} net={} F_I={} loss_float={} loss_artifact={} noise={} within={} save_load={} inputs={} verify_all={} dev={} gap_inside_band={}",
                value["seed"],
                value["float"]["top1_hits"],
                value["top1_artifact"],
                value["paired_float_vs_artifact"]["a_hit_b_miss_c"],
                value["paired_float_vs_artifact"]["a_miss_b_hit_b"],
                value["net_gap_rows"],
                value["f_i"],
                value["loss_float"],
                value["loss_artifact"],
                value["noise_band_dev_loss"],
                value["within_band"],
                value["i_save_load"],
                value["i_inputs"],
                value["i_verify_all"],
                value["i_dev_denominator"],
                value["gap_inside_band"],
            );
            write_receipt(&out, &value)
        }
        other => Err(FloatTrainError::Invalid(format!(
            "unsupported --mode {other}"
        ))),
    }
}
