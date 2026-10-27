//! `spiralcore-fbs-sheet --mode synthetic-probe|throughput-probe|full|d11-export-gap|d11-artifact-windows|d11-artifact-gap|d11-train-probe|d11-train-export|d11-residual|d11-snap-trace|d11-forced-snaps|d11-forced-summary --data-root DIR --out FILE [--save-dir DIR] [options below]`
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
//!
//! `d11-residual` is the residual check (claude-132,
//! `docs/harness/prep/d11-exporter-float-trunk-residual-2026-10-26.md`): one saved model's D11
//! panel, plain and export-mode panels, and head on the artifact's queries, paired against the
//! artifact's stored evaluator run. It takes `--model-dir`, `--evaluator`, `--windows-sha256`,
//! `--artifact-sha256`, `--model-sha256`, `--stored-preds`, `--seed`, `--label` (the configuration's name) and,
//! where the D11 panel was stored at training (Stage 4b), `--stored-d11-preds`.
//!
//! `d11-snap-trace` is snap-flip localisation (claude-134,
//! `docs/harness/prep/d11-exporter-float-trunk-snap-trace-2026-10-26.md`): over the first
//! `--windows-count` DEV windows, a saved model's D11 snap choices against the artifact's trace
//! (`d11-fbs-eval trace`, `--trace`), with `--model-dir`, `--windows-sha256`, `--artifact-sha256`,
//! `--model-sha256`, `--stored-run` (the qualified run of the same artifact, I-Old-Run), `--label`
//! and `--seed`.
//!
//! `d11-forced-snaps` is the forced-snap control (claude-135,
//! `docs/harness/prep/d11-exporter-float-trunk-forced-snaps-2026-10-26.md`): the same options as
//! `d11-snap-trace` plus `--expect-unforced` (the unforced D's rows against A, `I-Unforced`), over
//! the full panel, with every snap forced to the artifact's trace. `d11-forced-summary` applies its
//! bar to `--receipts` (comma-separated receipt paths) against the envelope of `--faithful`
//! (comma-separated `label:seed`).

use std::collections::BTreeMap;
use std::env;
use std::path::{Path, PathBuf};

use candle_core::Device;
use uor_r4_float_train::spiralcore_fbs_sheet::{
    d11_artifact_windows, run_d11_artifact_gap, run_d11_export_gap, run_d11_forced_snaps,
    run_d11_residual, run_d11_snap_trace, run_d11_train_export, run_d11_train_probe,
    run_full_sheet, run_synthetic_probe, run_throughput_probe, summarize_forced, write_receipt,
    TrainForward,
};
use uor_r4_float_train::{FloatTrainError, Result};

/// The options of `d11-artifact-gap`, `d11-train-probe`, `d11-train-export`, `d11-residual`,
/// `d11-snap-trace`, `d11-forced-snaps` and `d11-forced-summary`.
const GAP_OPTIONS: [&str; 20] = [
    "--expect-unforced",
    "--receipts",
    "--faithful",
    "--model-dir",
    "--evaluator",
    "--windows-sha256",
    "--artifact-sha256",
    "--stage4b",
    "--stored-preds",
    "--seed",
    "--train-forward",
    "--label",
    "--stored-d11-preds",
    "--model-sha256",
    "--trace",
    "--windows-count",
    "--stored-run",
    "--steps",
    "--preds-d11-out",
    "--preds-artifact-out",
];

/// The one dose the longer-dual sheet registers besides the default.
const LONGER_STEPS: usize = 2_048;

fn train_steps(extra: &BTreeMap<String, String>, mode: &str) -> Result<usize> {
    if extra.contains_key("--steps")
        && !matches!(
            mode,
            "d11-train-probe" | "d11-train-export" | "d11-artifact-gap" | "d11-residual"
        )
    {
        return Err(FloatTrainError::Invalid(format!(
            "--steps is not valid for {mode}"
        )));
    }
    let steps = extra
        .get("--steps")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| FloatTrainError::Invalid("--steps must be an integer".to_owned()))
        })
        .transpose()?
        .unwrap_or(1_024);
    match steps {
        1_024 | LONGER_STEPS => Ok(steps),
        _ => Err(FloatTrainError::Invalid(
            "--steps must be 1024 or 2048".to_owned(),
        )),
    }
}

type Args = (
    String,
    PathBuf,
    PathBuf,
    Option<PathBuf>,
    BTreeMap<String, String>,
);

/// The options of `d11-residual` (claude-132): every one required but `--stored-d11-preds` and
/// `--stage4b`.
#[derive(Debug, PartialEq)]
struct ResidualArgs {
    model_dir: PathBuf,
    evaluator: PathBuf,
    windows_sha256: String,
    artifact_sha256: String,
    model_sha256: String,
    stored_preds: PathBuf,
    stored_d11_preds: Option<PathBuf>,
    stage4b: Option<PathBuf>,
    seed: u64,
    label: String,
}

fn residual_args(extra: &BTreeMap<String, String>) -> Result<ResidualArgs> {
    let option = |name: &str| {
        extra
            .get(name)
            .cloned()
            .ok_or_else(|| FloatTrainError::Invalid(format!("d11-residual needs {name}")))
    };
    Ok(ResidualArgs {
        model_dir: PathBuf::from(option("--model-dir")?),
        evaluator: PathBuf::from(option("--evaluator")?),
        windows_sha256: option("--windows-sha256")?,
        artifact_sha256: option("--artifact-sha256")?,
        model_sha256: option("--model-sha256")?,
        stored_preds: PathBuf::from(option("--stored-preds")?),
        stored_d11_preds: extra.get("--stored-d11-preds").map(PathBuf::from),
        stage4b: extra.get("--stage4b").map(PathBuf::from),
        seed: option("--seed")?
            .parse()
            .map_err(|_| FloatTrainError::Invalid("--seed must be an integer".to_owned()))?,
        label: option("--label")?,
    })
}

/// A prediction vector option's path, if the caller supplied it.
fn prediction_path(extra: &BTreeMap<String, String>, name: &str) -> Option<PathBuf> {
    extra.get(name).map(PathBuf::from)
}

/// Writes a receipt's prediction vector to an explicitly requested output, never over a file.
fn write_prediction(path: &Path, value: &serde_json::Value, option: &str) -> Result<()> {
    if !value.is_array() {
        return Err(FloatTrainError::Invalid(format!(
            "{option} has no prediction vector in this receipt"
        )));
    }
    write_receipt(path, value)
}

/// `--faithful`'s models, `label:seed` separated by commas (claude-135).
fn forced_faithful(text: &str) -> Result<Vec<(String, u64)>> {
    text.split(',')
        .map(|m| {
            let (label, seed) = m.rsplit_once(':').ok_or_else(|| {
                FloatTrainError::Invalid(format!("--faithful wants label:seed, not {m}"))
            })?;
            let seed = seed
                .parse()
                .map_err(|_| FloatTrainError::Invalid(format!("--faithful seed in {m}")))?;
            Ok((label.to_owned(), seed))
        })
        .collect()
}

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
    // Bind and validate the step count before the CUDA device is opened.
    let steps = train_steps(&extra, &mode)?;
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
            let value = run_d11_train_probe(&data_root, train_forward, steps, &device)?;
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
                steps,
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
                &option("--model-sha256")?,
                &PathBuf::from(option("--stage4b")?),
                &PathBuf::from(option("--stored-preds")?),
                seed,
                steps,
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
            write_receipt(&out, &value)?;
            if let Some(path) = prediction_path(&extra, "--preds-d11-out") {
                return Err(FloatTrainError::Invalid(format!(
                    "d11-artifact-gap has no D11 predictions; {} would be empty",
                    path.display()
                )));
            }
            if let Some(path) = prediction_path(&extra, "--preds-artifact-out") {
                write_prediction(&path, &value["preds_artifact"], "--preds-artifact-out")?;
            }
            Ok(())
        }
        "d11-residual" => {
            let a = residual_args(&extra)?;
            let value = run_d11_residual(
                &data_root,
                &a.model_dir,
                &a.evaluator,
                &a.windows_sha256,
                &a.artifact_sha256,
                &a.model_sha256,
                &a.stored_preds,
                a.stored_d11_preds.as_deref(),
                a.stage4b.as_deref(),
                a.seed,
                &a.label,
                steps,
                &device,
            )?;
            let p = &value["pairs"]["d11_vs_artifact"];
            println!(
                "[d11-residual] label={} seed={} top1_float={} top1_export_mode={} top1_d11={} top1_head={} top1_artifact={} d11_vs_artifact_b={} c={} net={} F_I={} differ={} head_vs_artifact_differ={} max_drift={} instruments={} classification={}",
                value["label"],
                value["seed"],
                value["top1"]["float"],
                value["top1"]["export_mode"],
                value["top1"]["d11"],
                value["top1"]["head_on_artifact_queries"],
                value["top1"]["artifact"],
                p["b"],
                p["c"],
                p["net"],
                p["f_i"],
                p["rows_differ"],
                value["pairs"]["head_vs_artifact"]["rows_differ"],
                value["q12_drift_d11_vs_artifact"]["all"]["max_codes"],
                value["instruments_pass"],
                value["classification"],
            );
            write_receipt(&out, &value)?;
            if let Some(path) = prediction_path(&extra, "--preds-d11-out") {
                write_prediction(&path, &value["preds_d11"], "--preds-d11-out")?;
            }
            if let Some(path) = prediction_path(&extra, "--preds-artifact-out") {
                write_prediction(&path, &value["preds_artifact"], "--preds-artifact-out")?;
            }
            Ok(())
        }
        "d11-snap-trace" => {
            let number = |name: &str| -> Result<u64> {
                option(name)?
                    .parse()
                    .map_err(|_| FloatTrainError::Invalid(format!("{name} must be an integer")))
            };
            let value = run_d11_snap_trace(
                &data_root,
                &PathBuf::from(option("--model-dir")?),
                &PathBuf::from(option("--trace")?),
                number("--windows-count")? as usize,
                &option("--windows-sha256")?,
                &option("--artifact-sha256")?,
                &option("--model-sha256")?,
                &PathBuf::from(option("--stored-run")?),
                &option("--label")?,
                number("--seed")?,
                &device,
            )?;
            println!(
                "[d11-snap-trace] label={} seed={} windows={} rows_differ={} preceded_by_root={} agree_preceded_by_root={} lead={} of {} primaries={} classes={} instruments={} classification={}",
                value["label"],
                value["seed"],
                value["windows"],
                value["rows_differ"],
                value["rows_differ_preceded_by_a_root_disagreement"],
                value["rows_agree_preceded_by_a_root_disagreement"],
                value["windows_whose_first_flip_leads"],
                value["windows_with_rows_differ"],
                value["primaries_windows_with_rows_differ"]["primaries"],
                value["snap_disagreements_per_class"],
                value["instruments_pass"],
                value["classification"],
            );
            write_receipt(&out, &value)
        }
        "d11-forced-snaps" => {
            let number = |name: &str| -> Result<u64> {
                option(name)?
                    .parse()
                    .map_err(|_| FloatTrainError::Invalid(format!("{name} must be an integer")))
            };
            let value = run_d11_forced_snaps(
                &data_root,
                &PathBuf::from(option("--model-dir")?),
                &PathBuf::from(option("--trace")?),
                number("--windows-count")? as usize,
                &option("--windows-sha256")?,
                &option("--artifact-sha256")?,
                &option("--model-sha256")?,
                &PathBuf::from(option("--stored-run")?),
                number("--expect-unforced")? as usize,
                &option("--label")?,
                number("--seed")?,
                &device,
            )?;
            println!(
                "[d11-forced-snaps] label={} seed={} readings={} self_force={} wrong_root={} unforced={} layout={} instruments={}",
                value["label"],
                value["seed"],
                value["readings"],
                value["i_self_force"],
                value["i_wrong_root"],
                value["i_unforced"],
                value["i_layout"],
                value["instruments_pass"],
            );
            write_receipt(&out, &value)
        }
        "d11-forced-summary" => {
            let receipts = option("--receipts")?
                .split(',')
                .map(|p| Ok(serde_json::from_slice(&std::fs::read(p)?)?))
                .collect::<Result<Vec<serde_json::Value>>>()?;
            let faithful = forced_faithful(&option("--faithful")?)?;
            let value = summarize_forced(&receipts, &faithful)?;
            println!(
                "[d11-forced-summary] envelope={} models={}",
                value["envelope_would_flip_rate"], value["models"]
            );
            write_receipt(&out, &value)
        }
        other => Err(FloatTrainError::Invalid(format!(
            "unsupported --mode {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn residual_extra() -> BTreeMap<String, String> {
        [
            ("--model-dir", "m"),
            ("--evaluator", "e.json"),
            ("--windows-sha256", "w"),
            ("--artifact-sha256", "a"),
            ("--model-sha256", "s"),
            ("--stored-preds", "p.json"),
            ("--seed", "2"),
            ("--label", "dual-c131"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
    }

    #[test]
    fn the_train_step_option_accepts_only_the_two_registered_doses() -> Result<()> {
        let mut extra = BTreeMap::new();
        assert_eq!(train_steps(&extra, "d11-train-export")?, 1_024);
        extra.insert("--steps".to_owned(), "2048".to_owned());
        assert_eq!(train_steps(&extra, "d11-train-export")?, 2_048);
        for invalid in [
            "0",
            "1",
            "512",
            "1023",
            "1025",
            "2047",
            "2049",
            "4096",
            "not-a-number",
        ] {
            extra.insert("--steps".to_owned(), invalid.to_owned());
            assert!(
                train_steps(&extra, "d11-train-export").is_err(),
                "{invalid} must be refused"
            );
        }
        Ok(())
    }

    #[test]
    fn the_train_step_option_is_rejected_by_modes_that_do_not_consume_it() -> Result<()> {
        let mut extra = BTreeMap::new();
        extra.insert("--steps".to_owned(), "2048".to_owned());
        for mode in [
            "d11-train-probe",
            "d11-train-export",
            "d11-artifact-gap",
            "d11-residual",
        ] {
            assert_eq!(train_steps(&extra, mode)?, 2_048);
        }
        for mode in ["full", "d11-export-gap", "d11-snap-trace"] {
            assert!(train_steps(&extra, mode).is_err(), "{mode}");
        }
        Ok(())
    }

    #[test]
    fn every_cli_option_the_prediction_and_stage4b_paths_need_is_parsed() -> Result<()> {
        for option in [
            "--stage4b",
            "--steps",
            "--model-sha256",
            "--preds-d11-out",
            "--preds-artifact-out",
        ] {
            assert!(GAP_OPTIONS.contains(&option), "{option} is a CLI option");
        }
        let mut extra = residual_extra();
        extra.insert("--stage4b".to_owned(), "gap.json".to_owned());
        extra.insert("--preds-d11-out".to_owned(), "d11.json".to_owned());
        extra.insert(
            "--preds-artifact-out".to_owned(),
            "artifact.json".to_owned(),
        );
        assert_eq!(
            residual_args(&extra)?.stage4b,
            Some(PathBuf::from("gap.json"))
        );
        assert_eq!(
            prediction_path(&extra, "--preds-d11-out"),
            Some(PathBuf::from("d11.json"))
        );
        assert_eq!(
            prediction_path(&extra, "--preds-artifact-out"),
            Some(PathBuf::from("artifact.json"))
        );
        // The prediction outputs are optional, not residual-required fields.
        assert_eq!(prediction_path(&BTreeMap::new(), "--preds-d11-out"), None);
        Ok(())
    }

    #[test]
    fn the_residual_options_parse_and_only_the_optional_panels_are_optional() -> Result<()> {
        let mut extra = residual_extra();
        // Every option the mode reads is one the parser accepts.
        assert!(extra.keys().all(|k| GAP_OPTIONS.contains(&k.as_str())));
        assert!(GAP_OPTIONS.contains(&"--stored-d11-preds"));
        assert!(GAP_OPTIONS.contains(&"--stage4b"));
        // Each option lands in its own field, so no two can be swapped unnoticed.
        assert_eq!(
            residual_args(&extra)?,
            ResidualArgs {
                model_dir: PathBuf::from("m"),
                evaluator: PathBuf::from("e.json"),
                windows_sha256: "w".to_owned(),
                artifact_sha256: "a".to_owned(),
                model_sha256: "s".to_owned(),
                stored_preds: PathBuf::from("p.json"),
                stored_d11_preds: None,
                stage4b: None,
                seed: 2,
                label: "dual-c131".to_owned(),
            }
        );
        extra.insert("--stored-d11-preds".to_owned(), "d.json".to_owned());
        assert_eq!(
            residual_args(&extra)?.stored_d11_preds,
            Some(PathBuf::from("d.json"))
        );
        extra.insert("--stage4b".to_owned(), "gap.json".to_owned());
        assert_eq!(
            residual_args(&extra)?.stage4b,
            Some(PathBuf::from("gap.json"))
        );
        for required in residual_extra().keys() {
            let mut missing = residual_extra();
            missing.remove(required);
            assert!(residual_args(&missing).is_err(), "{required} is required");
        }
        let mut bad_seed = residual_extra();
        bad_seed.insert("--seed".to_owned(), "two".to_owned());
        assert!(residual_args(&bad_seed).is_err());
        Ok(())
    }
}
