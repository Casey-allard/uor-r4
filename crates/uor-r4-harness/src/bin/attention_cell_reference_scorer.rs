//! Standalone host-only CLI for validating GPU-captured attention preflight
//! and Dose 1 paired DEV receipts against the sealed D34 reference record.
//!
//! No CUDA context, model forward, routing, or training is executed by this binary.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use uor_r4_harness::attention_reference::{
    run_bundle_candidate_cells_files_with_checkpoint, run_paired_dev_files_with_checkpoints,
    run_preflight_dev_files_with_checkpoint, CorpusPaths,
};

fn usage() -> &'static str {
    "usage:
  attention-cell-reference-scorer preflight-dev \\
    --panel <panel.json> \\
    --capture <capture.json> \\
    --eval <eval.json> \\
    --d34 <d34-successor-table-record.json> \\
    [--candidate-cells <candidate-cells.json>] \\
    [--token-cells <token_cells.u16|report.json> [--checkpoint <checkpoint.json>] --train-tokens <train/tokens.u16> --train-mask <train/response_mask.u8> --dev-tokens <dev/tokens.u16> --dev-mask <dev/response_mask.u8>] \\
    [--out <out.json>]

  attention-cell-reference-scorer paired-dev \\
    --panel <panel.json> \\
    --relation-capture <rel-capture.json> \\
    --relation-eval <rel-eval.json> \\
    --control-capture <ctrl-capture.json> \\
    --control-eval <ctrl-eval.json> \\
    [--ablation-eval <ablation-eval.json>] \\
    --d34 <d34-successor-table-record.json> \\
    [--relation-candidate-cells <rel-cells.json> --control-candidate-cells <ctrl-cells.json>] \\
    [--relation-token-cells <rel-token_cells.u16|report.json> --control-token-cells <ctrl-token_cells.u16|report.json> [--relation-checkpoint <rel-checkpoint.json>] [--control-checkpoint <ctrl-checkpoint.json>] --train-tokens <train/tokens.u16> --train-mask <train/response_mask.u8> --dev-tokens <dev/tokens.u16> --dev-mask <dev/response_mask.u8>] \\
    [--out <out.json>]

  attention-cell-reference-scorer bundle-candidate-cells \\
    --token-cells <token_cells.u16|report.json> \\
    [--checkpoint <checkpoint.json>] \\
    [--checkpoint-sha256 <driver-checkpoint-sha256>] \\
    --d34 <d34-successor-table-record.json> \\
    --train-tokens <train/tokens.u16> \\
    --train-mask <train/response_mask.u8> \\
    --dev-tokens <dev/tokens.u16> \\
    --dev-mask <dev/response_mask.u8> \\
    [--out <candidate-cells.json>]"
}

fn require_arg(opt: Option<PathBuf>, flag: &str) -> Result<PathBuf, String> {
    opt.ok_or_else(|| format!("missing required flag {flag}\n{}", usage()))
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(subcmd) = args.next() else {
        return Err(usage().to_string());
    };
    if subcmd == "--help" || subcmd == "-h" {
        println!("{}", usage());
        return Ok(());
    }

    match subcmd.as_str() {
        "preflight-dev" => {
            let mut panel = None;
            let mut capture = None;
            let mut eval = None;
            let mut d34 = None;
            let mut token_cells = None;
            let mut checkpoint = None;
            let mut train_tokens = None;
            let mut train_mask = None;
            let mut dev_tokens = None;
            let mut dev_mask = None;
            let mut candidate_cells = None;
            let mut out = None;

            while let Some(flag) = args.next() {
                if flag == "--help" || flag == "-h" {
                    println!("{}", usage());
                    return Ok(());
                }
                let val = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))?;
                match flag.as_str() {
                    "--panel" => panel = Some(val),
                    "--capture" => capture = Some(val),
                    "--eval" => eval = Some(val),
                    "--d34" => d34 = Some(val),
                    "--token-cells" => token_cells = Some(val),
                    "--checkpoint" => checkpoint = Some(val),
                    "--train-tokens" => train_tokens = Some(val),
                    "--train-mask" => train_mask = Some(val),
                    "--dev-tokens" => dev_tokens = Some(val),
                    "--dev-mask" => dev_mask = Some(val),
                    "--candidate-cells" => candidate_cells = Some(val),
                    "--out" => out = Some(val),
                    other => return Err(format!("unknown flag '{other}'\n{}", usage())),
                }
            }

            let tc_and_corpus = if token_cells.is_some()
                || train_tokens.is_some()
                || train_mask.is_some()
                || dev_tokens.is_some()
                || dev_mask.is_some()
            {
                Some((
                    require_arg(token_cells, "--token-cells")?,
                    require_arg(train_tokens, "--train-tokens")?,
                    require_arg(train_mask, "--train-mask")?,
                    require_arg(dev_tokens, "--dev-tokens")?,
                    require_arg(dev_mask, "--dev-mask")?,
                ))
            } else {
                None
            };
            let corpus_ref = tc_and_corpus
                .as_ref()
                .map(|(_, tt, tm, dt, dm)| CorpusPaths {
                    train_tokens: tt.as_path(),
                    train_mask: tm.as_path(),
                    dev_tokens: dt.as_path(),
                    dev_mask: dm.as_path(),
                });

            let result = run_preflight_dev_files_with_checkpoint(
                &require_arg(panel, "--panel")?,
                &require_arg(capture, "--capture")?,
                &require_arg(eval, "--eval")?,
                &require_arg(d34, "--d34")?,
                candidate_cells.as_deref(),
                tc_and_corpus
                    .as_ref()
                    .zip(corpus_ref.as_ref())
                    .map(|((tc, _, _, _, _), cp)| (tc.as_path(), cp)),
                checkpoint.as_deref(),
            )?;
            let formatted = serde_json::to_string_pretty(&result)
                .map_err(|e| format!("serialize output: {e}"))?;
            if let Some(out_path) = out {
                fs::write(&out_path, format!("{formatted}\n"))
                    .map_err(|e| format!("write {}: {e}", out_path.display()))?;
            } else {
                println!("{formatted}");
            }
            Ok(())
        }
        "paired-dev" => {
            let mut panel = None;
            let mut rel_capture = None;
            let mut rel_eval = None;
            let mut ctrl_capture = None;
            let mut ctrl_eval = None;
            let mut ablation_eval = None;
            let mut d34 = None;
            let mut rel_cand = None;
            let mut ctrl_cand = None;
            let mut rel_tc = None;
            let mut ctrl_tc = None;
            let mut rel_checkpoint = None;
            let mut ctrl_checkpoint = None;
            let mut train_tokens = None;
            let mut train_mask = None;
            let mut dev_tokens = None;
            let mut dev_mask = None;
            let mut out = None;

            while let Some(flag) = args.next() {
                if flag == "--help" || flag == "-h" {
                    println!("{}", usage());
                    return Ok(());
                }
                let val = args
                    .next()
                    .map(PathBuf::from)
                    .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))?;
                match flag.as_str() {
                    "--panel" => panel = Some(val),
                    "--relation-capture" => rel_capture = Some(val),
                    "--relation-eval" => rel_eval = Some(val),
                    "--control-capture" => ctrl_capture = Some(val),
                    "--control-eval" => ctrl_eval = Some(val),
                    "--ablation-eval" => ablation_eval = Some(val),
                    "--d34" => d34 = Some(val),
                    "--relation-candidate-cells" => rel_cand = Some(val),
                    "--control-candidate-cells" => ctrl_cand = Some(val),
                    "--relation-token-cells" => rel_tc = Some(val),
                    "--control-token-cells" => ctrl_tc = Some(val),
                    "--relation-checkpoint" => rel_checkpoint = Some(val),
                    "--control-checkpoint" => ctrl_checkpoint = Some(val),
                    "--train-tokens" => train_tokens = Some(val),
                    "--train-mask" => train_mask = Some(val),
                    "--dev-tokens" => dev_tokens = Some(val),
                    "--dev-mask" => dev_mask = Some(val),
                    "--out" => out = Some(val),
                    other => return Err(format!("unknown flag '{other}'\n{}", usage())),
                }
            }

            let cand_pair = if rel_cand.is_some() || ctrl_cand.is_some() {
                Some((
                    require_arg(rel_cand, "--relation-candidate-cells")?,
                    require_arg(ctrl_cand, "--control-candidate-cells")?,
                ))
            } else {
                None
            };

            let tcs_and_corpus = if rel_tc.is_some()
                || ctrl_tc.is_some()
                || train_tokens.is_some()
                || train_mask.is_some()
                || dev_tokens.is_some()
                || dev_mask.is_some()
            {
                Some((
                    require_arg(rel_tc, "--relation-token-cells")?,
                    require_arg(ctrl_tc, "--control-token-cells")?,
                    require_arg(train_tokens, "--train-tokens")?,
                    require_arg(train_mask, "--train-mask")?,
                    require_arg(dev_tokens, "--dev-tokens")?,
                    require_arg(dev_mask, "--dev-mask")?,
                ))
            } else {
                None
            };
            let corpus_ref = tcs_and_corpus
                .as_ref()
                .map(|(_, _, tt, tm, dt, dm)| CorpusPaths {
                    train_tokens: tt.as_path(),
                    train_mask: tm.as_path(),
                    dev_tokens: dt.as_path(),
                    dev_mask: dm.as_path(),
                });

            let result = run_paired_dev_files_with_checkpoints(
                &require_arg(panel, "--panel")?,
                &require_arg(rel_capture, "--relation-capture")?,
                &require_arg(rel_eval, "--relation-eval")?,
                &require_arg(ctrl_capture, "--control-capture")?,
                &require_arg(ctrl_eval, "--control-eval")?,
                ablation_eval.as_deref(),
                &require_arg(d34, "--d34")?,
                cand_pair
                    .as_ref()
                    .map(|(rc, cc)| (rc.as_path(), cc.as_path())),
                tcs_and_corpus
                    .as_ref()
                    .zip(corpus_ref.as_ref())
                    .map(|((rtc, ctc, _, _, _, _), cp)| (rtc.as_path(), ctc.as_path(), cp)),
                rel_checkpoint.as_deref(),
                ctrl_checkpoint.as_deref(),
            )?;
            let formatted = serde_json::to_string_pretty(&result)
                .map_err(|e| format!("serialize output: {e}"))?;
            if let Some(out_path) = out {
                fs::write(&out_path, format!("{formatted}\n"))
                    .map_err(|e| format!("write {}: {e}", out_path.display()))?;
            } else {
                println!("{formatted}");
            }
            Ok(())
        }
        "bundle-candidate-cells" => {
            let mut token_cells = None;
            let mut checkpoint = None;
            let mut checkpoint_sha256: Option<String> = None;
            let mut d34 = None;
            let mut train_tokens = None;
            let mut train_mask = None;
            let mut dev_tokens = None;
            let mut dev_mask = None;
            let mut out = None;

            while let Some(flag) = args.next() {
                if flag == "--help" || flag == "-h" {
                    println!("{}", usage());
                    return Ok(());
                }
                let val_str = args
                    .next()
                    .ok_or_else(|| format!("missing value for {flag}\n{}", usage()))?;
                match flag.as_str() {
                    "--token-cells" => token_cells = Some(PathBuf::from(val_str)),
                    "--checkpoint" => checkpoint = Some(PathBuf::from(val_str)),
                    "--checkpoint-sha256" => checkpoint_sha256 = Some(val_str),
                    "--d34" => d34 = Some(PathBuf::from(val_str)),
                    "--train-tokens" => train_tokens = Some(PathBuf::from(val_str)),
                    "--train-mask" => train_mask = Some(PathBuf::from(val_str)),
                    "--dev-tokens" => dev_tokens = Some(PathBuf::from(val_str)),
                    "--dev-mask" => dev_mask = Some(PathBuf::from(val_str)),
                    "--out" => out = Some(PathBuf::from(val_str)),
                    other => return Err(format!("unknown flag '{other}'\n{}", usage())),
                }
            }

            let tt = require_arg(train_tokens, "--train-tokens")?;
            let tm = require_arg(train_mask, "--train-mask")?;
            let dt = require_arg(dev_tokens, "--dev-tokens")?;
            let dm = require_arg(dev_mask, "--dev-mask")?;
            let corpus_paths = CorpusPaths {
                train_tokens: &tt,
                train_mask: &tm,
                dev_tokens: &dt,
                dev_mask: &dm,
            };
            let table = run_bundle_candidate_cells_files_with_checkpoint(
                &require_arg(token_cells, "--token-cells")?,
                checkpoint.as_deref(),
                checkpoint_sha256.as_deref(),
                &require_arg(d34, "--d34")?,
                &corpus_paths,
            )?;
            let formatted = serde_json::to_string_pretty(&table)
                .map_err(|e| format!("serialize output: {e}"))?;
            if let Some(out_path) = out {
                fs::write(&out_path, format!("{formatted}\n"))
                    .map_err(|e| format!("write {}: {e}", out_path.display()))?;
            } else {
                println!("{formatted}");
            }
            Ok(())
        }
        other => Err(format!("unknown subcommand '{other}'\n{}", usage())),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
    }
}
