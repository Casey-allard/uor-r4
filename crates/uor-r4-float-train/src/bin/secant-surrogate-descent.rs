//! CLI runner for `r-secant-surrogate-descent` (`#2886`).
//!
//! Modes:
//! - `--mode synthetic-probe`: Probe (a) synthetic mechanism & control verification on CUDA.
//! - `--mode throughput-probe`: Probe (b) throughput & boundary-crossing projection on CUDA.
//! - `--mode full`: Full 6-arm × 2-seed × 256-step pre-registered run on CUDA.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use candle_core::Device;
use uor_r4_float_train::secant_descent::{
    run_single_arm, summarize_suite, DiagnosticSlice, SecantArm, PROBE_BATCH, PROBE_BATCHES,
    PROBE_CONTEXT, PROBE_STEPS,
};
use uor_r4_float_train::{FloatTrainError, Result};

fn parse_args() -> Result<(String, PathBuf, PathBuf, PathBuf, PathBuf, f64, f64)> {
    let mut mode = String::from("full");
    let mut tokens = PathBuf::from("tokens.u16");
    let mut mask = PathBuf::from("response_mask.u8");
    let mut tokenizer = PathBuf::from("tokenizer.json");
    let mut out = PathBuf::from("secant-surrogate-descent-receipt.json");
    let mut lr_rec = 0.005f64;
    let mut lr_fbs = 0.005f64;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--mode" => {
                mode = args
                    .next()
                    .ok_or_else(|| FloatTrainError::Invalid("missing --mode value".into()))?;
            }
            "--tokens" => {
                tokens =
                    PathBuf::from(args.next().ok_or_else(|| {
                        FloatTrainError::Invalid("missing --tokens value".into())
                    })?);
            }
            "--mask" => {
                mask = PathBuf::from(
                    args.next()
                        .ok_or_else(|| FloatTrainError::Invalid("missing --mask value".into()))?,
                );
            }
            "--tokenizer" => {
                tokenizer =
                    PathBuf::from(args.next().ok_or_else(|| {
                        FloatTrainError::Invalid("missing --tokenizer value".into())
                    })?);
            }
            "--out" => {
                out = PathBuf::from(
                    args.next()
                        .ok_or_else(|| FloatTrainError::Invalid("missing --out value".into()))?,
                );
            }
            "--lr-rec" => {
                let val = args
                    .next()
                    .ok_or_else(|| FloatTrainError::Invalid("missing --lr-rec value".into()))?;
                lr_rec = val
                    .parse::<f64>()
                    .map_err(|e| FloatTrainError::Invalid(format!("invalid --lr-rec: {e}")))?;
            }
            "--lr-fbs" => {
                let val = args
                    .next()
                    .ok_or_else(|| FloatTrainError::Invalid("missing --lr-fbs value".into()))?;
                lr_fbs = val
                    .parse::<f64>()
                    .map_err(|e| FloatTrainError::Invalid(format!("invalid --lr-fbs: {e}")))?;
            }
            other => {
                return Err(FloatTrainError::Invalid(format!(
                    "unrecognized argument: {other}"
                )));
            }
        }
    }
    Ok((mode, tokens, mask, tokenizer, out, lr_rec, lr_fbs))
}

fn main() -> Result<()> {
    let (mode, tokens_path, mask_path, tokenizer_path, out_path, base_lr_rec, base_lr_fbs) =
        parse_args()?;
    let device = Device::new_cuda(0)?;
    let suite_start = Instant::now();

    match mode.as_str() {
        "synthetic-probe" => {
            let slice = DiagnosticSlice::synthetic(4, PROBE_BATCH, PROBE_CONTEXT);
            let mut probe_runs = Vec::new();
            for arm in SecantArm::ALL {
                let lr = if arm.is_part_a() {
                    base_lr_rec
                } else {
                    base_lr_fbs
                };
                let r = run_single_arm(arm, 1, 8, lr, false, &slice, &device)?;
                println!(
                    "[synthetic-probe] arm={} loss_0={:.6} loss_8={:.6} delta={:+.6} ds_norm={:.6} dq_norm={:.6} rec_cross={:.4} fbs_cross={:.4} wall_ms={}",
                    r.arm,
                    r.loss_step_0,
                    r.loss_step_256,
                    r.delta_loss_0_to_256,
                    r.step0_ds_l2_norm,
                    r.step0_dq_l2_norm,
                    r.rec_active_lane_token_crossings_frac.max(r.rec_decay_param_crossings_frac),
                    r.fbs_query_stage_crossings_frac,
                    r.wall_ms
                );
                if matches!(
                    arm,
                    SecantArm::RecZeroCreditNull | SecantArm::FbsZeroCreditNull
                ) && r.delta_loss_0_to_256.abs() > 1e-7
                {
                    return Err(FloatTrainError::Invalid(format!(
                        "zero-credit null {} changed loss by {}",
                        arm.name(),
                        r.delta_loss_0_to_256
                    )));
                }
                probe_runs.push(r);
            }
            let payload = serde_json::to_vec_pretty(&serde_json::json!({
                "mode": "synthetic-probe",
                "runs": probe_runs,
                "wall_ms": suite_start.elapsed().as_millis() as u64,
            }))?;
            fs::write(&out_path, payload)?;
            Ok(())
        }
        "throughput-probe" => {
            let slice = DiagnosticSlice::load_pinned(
                &tokens_path,
                &mask_path,
                &tokenizer_path,
                PROBE_BATCHES,
            )?;
            let r_rec = run_single_arm(
                SecantArm::RecSecantActive,
                1,
                16,
                base_lr_rec,
                false,
                &slice,
                &device,
            )?;
            let r_fbs = run_single_arm(
                SecantArm::FbsSecantActive,
                1,
                16,
                base_lr_fbs,
                false,
                &slice,
                &device,
            )?;
            let projected_per_arm_s =
                ((r_rec.wall_ms.max(r_fbs.wall_ms) as f64) / 16.0 * (PROBE_STEPS as f64)) / 1000.0;
            println!(
                "[throughput-probe] rec_16step_ms={} fbs_16step_ms={} projected_256step_per_arm_s={:.2} projected_12runs_s={:.2}",
                r_rec.wall_ms,
                r_fbs.wall_ms,
                projected_per_arm_s,
                projected_per_arm_s * 12.0
            );
            let payload = serde_json::to_vec_pretty(&serde_json::json!({
                "mode": "throughput-probe",
                "rec_16step": r_rec,
                "fbs_16step": r_fbs,
                "projected_256step_per_arm_s": projected_per_arm_s,
                "projected_12runs_s": projected_per_arm_s * 12.0,
            }))?;
            fs::write(&out_path, payload)?;
            Ok(())
        }
        "full" => {
            let slice = DiagnosticSlice::load_pinned(
                &tokens_path,
                &mask_path,
                &tokenizer_path,
                PROBE_BATCHES,
            )?;

            // Part A (RecGates::Geometric): calibrate lr_rec if I-Boundary-Crossings < 1.0%.
            let mut lr_rec = base_lr_rec;
            let mut analytic_keep = false;
            let mut part_a_runs = Vec::new();
            for mult in [1.0f64, 2.5, 5.0, 10.0] {
                lr_rec = base_lr_rec * mult;
                let s1 = run_single_arm(
                    SecantArm::RecSecantActive,
                    1,
                    PROBE_STEPS,
                    lr_rec,
                    analytic_keep,
                    &slice,
                    &device,
                )?;
                let s2 = run_single_arm(
                    SecantArm::RecSecantActive,
                    2,
                    PROBE_STEPS,
                    lr_rec,
                    analytic_keep,
                    &slice,
                    &device,
                )?;
                let max_plat = s1
                    .keep_histograms
                    .iter()
                    .chain(s2.keep_histograms.iter())
                    .map(|h| h.plateau_12_15_fraction)
                    .fold(0.0f64, f64::max);
                if !analytic_keep
                    && max_plat > 0.80
                    && s1.step0_mean_abs_d_lambda.min(s2.step0_mean_abs_d_lambda) < 1e-7
                {
                    analytic_keep = true;
                    continue;
                }
                let c1 = s1
                    .rec_active_lane_token_crossings_frac
                    .max(s1.rec_decay_param_crossings_frac);
                let c2 = s2
                    .rec_active_lane_token_crossings_frac
                    .max(s2.rec_decay_param_crossings_frac);
                if c1.min(c2) >= 0.01 || mult == 10.0 {
                    part_a_runs.push(s1);
                    part_a_runs.push(s2);
                    break;
                }
            }
            for arm in [SecantArm::RecZeroCreditNull, SecantArm::RecRandomRefitNull] {
                for seed in [1u64, 2] {
                    part_a_runs.push(run_single_arm(
                        arm,
                        seed,
                        PROBE_STEPS,
                        lr_rec,
                        analytic_keep,
                        &slice,
                        &device,
                    )?);
                }
            }

            // Part B (QuaternionWeights::SpiralcoreFbs): calibrate lr_fbs if I-Boundary-Crossings < 1.0%.
            let mut lr_fbs = base_lr_fbs;
            let mut part_b_runs = Vec::new();
            for mult in [1.0f64, 2.5, 5.0, 10.0] {
                lr_fbs = base_lr_fbs * mult;
                let s1 = run_single_arm(
                    SecantArm::FbsSecantActive,
                    1,
                    PROBE_STEPS,
                    lr_fbs,
                    false,
                    &slice,
                    &device,
                )?;
                let s2 = run_single_arm(
                    SecantArm::FbsSecantActive,
                    2,
                    PROBE_STEPS,
                    lr_fbs,
                    false,
                    &slice,
                    &device,
                )?;
                let c1 = s1.fbs_query_stage_crossings_frac;
                let c2 = s2.fbs_query_stage_crossings_frac;
                if c1.min(c2) >= 0.01 || mult == 10.0 {
                    part_b_runs.push(s1);
                    part_b_runs.push(s2);
                    break;
                }
            }
            for arm in [SecantArm::FbsZeroCreditNull, SecantArm::FbsRandomRefitNull] {
                for seed in [1u64, 2] {
                    part_b_runs.push(run_single_arm(
                        arm,
                        seed,
                        PROBE_STEPS,
                        lr_fbs,
                        false,
                        &slice,
                        &device,
                    )?);
                }
            }

            let mut all_runs = part_a_runs;
            all_runs.extend(part_b_runs);
            for r in &all_runs {
                println!(
                    "[full] arm={} seed={} lr={} loss_0={:.6} loss_256={:.6} delta={:+.6} rec_cross={:.4} fbs_cross={:.4} fbs_flip={:.4} wall_ms={}",
                    r.arm,
                    r.seed,
                    r.lr,
                    r.loss_step_0,
                    r.loss_step_256,
                    r.delta_loss_0_to_256,
                    r.rec_active_lane_token_crossings_frac.max(r.rec_decay_param_crossings_frac),
                    r.fbs_query_stage_crossings_frac,
                    r.fbs_output_route_flip_rate,
                    r.wall_ms
                );
            }

            let total_wall_ms = suite_start.elapsed().as_millis() as u64;
            let receipt = summarize_suite(
                &all_runs,
                lr_rec,
                lr_fbs,
                analytic_keep,
                &slice,
                total_wall_ms,
            )?;
            println!(
                "[summary] G-Rec-Descent={} (noise={:.6}, margin_s1={:+.6}, margin_s2={:+.6}) | G-Fbs-Descent={} (noise={:.6}, margin_s1={:+.6}, margin_s2={:+.6})",
                receipt.gates.g_rec_descent_pass,
                receipt.gates.delta_noise_rec,
                receipt.gates.rec_margin_over_controls_seed1,
                receipt.gates.rec_margin_over_controls_seed2,
                receipt.gates.g_fbs_descent_pass,
                receipt.gates.delta_noise_fbs,
                receipt.gates.fbs_margin_over_controls_seed1,
                receipt.gates.fbs_margin_over_controls_seed2,
            );
            fs::write(&out_path, serde_json::to_vec_pretty(&receipt)?)?;
            Ok(())
        }
        other => Err(FloatTrainError::Invalid(format!(
            "unsupported --mode {other}"
        ))),
    }
}
