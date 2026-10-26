//! Stage T (`m1-bvg-k-stage-t-training`, `#2836`) CLI binary (`stage-t-run`).
//!
//! Subcommands:
//! - `build-anchor-manifest --config <path> --out <path>`
//! - `step0 --t16-fz <path> --t16-rel <path> --t1-fz <path> --out <path>`
//! - `train --config <path> --anchor-manifest <path> --sheet <path> --out-dir <path> [--max-batches <N>] [--skip-w2]`
//! - `w2-eval --config <path> --checkpoint <path> --out <path>`
//! - `grade --sheet <path> --step0 <path> --fz-seed1 <dir> --fz-seed2 <dir> --rel-seed1 <dir> --rel-seed2 <dir> [--t1-fz-seed1-top1 <N>] [--t1-fz-seed2-top1 <N>] [--t1-rel-seed1-top1 <N>] [--t1-rel-seed2-top1 <N>] --out <path>`

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use uor_r4_discrete::stage_t::{
    self, evaluate_seed_against_bars, grade_stage_t, pin_file, write_pinned_json, StageTRunReport,
    StageTW2Summary, Step0Receipt,
};
use uor_r4_discrete::{Error, Result};

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

fn require_arg(args: &[String], flag: &str) -> Result<String> {
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == flag {
            let val = args
                .get(idx + 1)
                .ok_or_else(|| invalid(format!("missing value after {flag}")))?;
            return Ok(val.clone());
        }
        idx += 1;
    }
    Err(invalid(format!("missing required flag {flag}")))
}

fn optional_u64(args: &[String], flag: &str, default: u64) -> Result<u64> {
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == flag {
            let val = args
                .get(idx + 1)
                .ok_or_else(|| invalid(format!("missing value after {flag}")))?;
            return val
                .parse::<u64>()
                .map_err(|e| invalid(format!("invalid u64 for {flag}: {e}")));
        }
        idx += 1;
    }
    Ok(default)
}

#[cfg(feature = "cuda")]
fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    serde_json::from_slice(&bytes).map_err(|e| invalid(format!("{}: {e}", path.display())))
}

fn load_seed_eval_from_dir(
    dir: &Path,
    t1_control_top1: u64,
    t1_halted_early: bool,
) -> Result<stage_t::SeedStageTEval> {
    let report_path = dir.join("report.json");
    let report: StageTRunReport = read_json_file(&report_path)?;
    let w2: StageTW2Summary = if let Some(w2) = report.w2_summary.clone() {
        w2
    } else {
        let w2_path = dir.join("w2-summary.json");
        read_json_file(&w2_path)?
    };
    let n10 = report
        .final_dev
        .top1_correct_rows
        .saturating_sub(t1_control_top1);
    let n01 = 0u64;
    evaluate_seed_against_bars(&report, &w2, t1_control_top1, t1_halted_early, n10, n01)
}

fn run_cli(args: &[String]) -> Result<()> {
    let subcmd = args.first().map(String::as_str).ok_or_else(|| {
        invalid("expected subcommand: build-anchor-manifest | step0 | train | w2-eval | grade")
    })?;
    let rest = &args[1..];
    match subcmd {
        "build-anchor-manifest" => {
            let cfg = PathBuf::from(require_arg(rest, "--config")?);
            let out = PathBuf::from(require_arg(rest, "--out")?);
            let pin = stage_t::build_train_anchor_manifest(&cfg, &out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&pin).map_err(|e| invalid(e.to_string()))?
            );
            Ok(())
        }
        "grade" => {
            let sheet_str = require_arg(rest, "--sheet")?;
            let sheet_pin = pin_file(Path::new(&sheet_str))?;
            let step0_path = PathBuf::from(require_arg(rest, "--step0")?);
            let step0_pin = pin_file(&step0_path)?;
            let step0: Step0Receipt = read_json_file(&step0_path)?;

            let fz_s1_dir = PathBuf::from(require_arg(rest, "--fz-seed1")?);
            let fz_s2_dir = PathBuf::from(require_arg(rest, "--fz-seed2")?);
            let rel_s1_dir = PathBuf::from(require_arg(rest, "--rel-seed1")?);
            let rel_s2_dir = PathBuf::from(require_arg(rest, "--rel-seed2")?);
            let out_path = PathBuf::from(require_arg(rest, "--out")?);

            // Default seed-1 T1 controls from #2831 (antigravity-57): FrozenZero Q1 = 871, Relation = 133
            let t1_fz_s1 = optional_u64(rest, "--t1-fz-seed1-top1", 871)?;
            let t1_fz_s2 = optional_u64(rest, "--t1-fz-seed2-top1", 871)?;
            let t1_rel_s1 = optional_u64(rest, "--t1-rel-seed1-top1", 133)?;
            let t1_rel_s2 = optional_u64(rest, "--t1-rel-seed2-top1", 133)?;

            let fz_seed1 = load_seed_eval_from_dir(&fz_s1_dir, t1_fz_s1, true)?;
            let fz_seed2 = load_seed_eval_from_dir(&fz_s2_dir, t1_fz_s2, true)?;
            let rel_seed1 = load_seed_eval_from_dir(&rel_s1_dir, t1_rel_s1, true)?;
            let rel_seed2 = load_seed_eval_from_dir(&rel_s2_dir, t1_rel_s2, true)?;

            let grade = grade_stage_t(
                &sheet_str,
                &sheet_pin.sha256,
                &step0,
                &step0_pin.sha256,
                vec![fz_seed1, fz_seed2],
                vec![rel_seed1, rel_seed2],
            )?;
            let pin = write_pinned_json(&out_path, &grade)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&pin).map_err(|e| invalid(e.to_string()))?
            );
            Ok(())
        }
        "step0" | "train" | "w2-eval" => {
            #[cfg(feature = "cuda")]
            {
                match subcmd {
                    "step0" => {
                        let t16_fz = PathBuf::from(require_arg(rest, "--t16-fz")?);
                        let t16_rel = PathBuf::from(require_arg(rest, "--t16-rel")?);
                        let t1_fz = PathBuf::from(require_arg(rest, "--t1-fz")?);
                        let out = PathBuf::from(require_arg(rest, "--out")?);
                        let receipt = stage_t::gpu::run_step0(&t16_fz, &t16_rel, &t1_fz, &out)?;
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&receipt)
                                .map_err(|e| invalid(e.to_string()))?
                        );
                        Ok(())
                    }
                    "train" => {
                        let config = PathBuf::from(require_arg(rest, "--config")?);
                        let anchor = PathBuf::from(require_arg(rest, "--anchor-manifest")?);
                        let sheet = PathBuf::from(require_arg(rest, "--sheet")?);
                        let out_dir = PathBuf::from(require_arg(rest, "--out-dir")?);
                        let max_batches =
                            optional_u64(rest, "--max-batches", stage_t::STAGE_T_MAX_BATCHES)?;
                        let run_w2 = !has_flag(rest, "--skip-w2");
                        let report = stage_t::gpu::run_stage_t_train(
                            &config,
                            &anchor,
                            &sheet,
                            &out_dir,
                            max_batches,
                            run_w2,
                        )?;
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report.final_dev)
                                .map_err(|e| invalid(e.to_string()))?
                        );
                        Ok(())
                    }
                    "w2-eval" => {
                        let config = PathBuf::from(require_arg(rest, "--config")?);
                        let ckpt = PathBuf::from(require_arg(rest, "--checkpoint")?);
                        let out = PathBuf::from(require_arg(rest, "--out")?);
                        let summary = stage_t::gpu::run_w2_eval(&config, &ckpt, &out)?;
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&summary)
                                .map_err(|e| invalid(e.to_string()))?
                        );
                        Ok(())
                    }
                    _ => unreachable!(),
                }
            }
            #[cfg(not(feature = "cuda"))]
            {
                Err(invalid(
                    "stage-t-run GPU subcommands (step0, train, w2-eval) require --features cuda",
                ))
            }
        }
        other => Err(invalid(format!("unknown stage-t-run subcommand: {other}"))),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run_cli(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("stage-t-run error: {err}");
            ExitCode::FAILURE
        }
    }
}
