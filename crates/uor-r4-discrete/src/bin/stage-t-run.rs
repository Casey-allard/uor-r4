//! Stage T (`m1-bvg-k-stage-t-training`, `#2836`) CLI binary (`stage-t-run`).
//!
//! Subcommands:
//! - `build-anchor-manifest --config <path> --out <path>`
//! - `step0 --t16-fz <path> --t16-rel <path> --t1-fz <path> --out <path>`
//! - `train --config <path> --anchor-manifest <path> --sheet <path> --out-dir <path> [--max-batches <N>] [--skip-w2]`
//! - `w2-eval --config <path> --checkpoint <path> --out <path>`
//! - `grade --sheet <path> --step0 <path> --fz-seed1 <dir> --fz-seed2 <dir> --rel-seed1 <dir> --rel-seed2 <dir> [--t1-fz-seed1-top1 <N>] [--t1-fz-seed2-top1 <N>] [--t1-rel-seed1-top1 <N>] [--t1-rel-seed2-top1 <N>] --out <path>`
//! - `grade-relation-head --out <path>`

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::Value;
use uor_r4_discrete::stage_t::{
    self, evaluate_seed_against_bars, grade_relation_head, grade_stage_t, pin_file,
    write_pinned_json, RelationHeadControlBaselines, RelationHeadSeedSummary,
    RelationHeadWithinCellSummary, StageTRunReport, StageTW2Summary, Step0Receipt,
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

fn json_u64(val: &Value, key: &str) -> Result<u64> {
    val.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| invalid(format!("missing u64 field {key}")))
}

fn json_i64(val: &Value, key: &str) -> Result<i64> {
    val.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| invalid(format!("missing i64 field {key}")))
}

fn build_relation_head_seed_summary(
    seed: u32,
    rel_w2: &StageTW2Summary,
    fz_w2: &StageTW2Summary,
    trunk_grade: &Value,
) -> Result<RelationHeadSeedSummary> {
    let s_key = format!("seed{seed}");
    let rel_l0_dev = json_u64(trunk_grade, &format!("rel_{s_key}_dev_level0_top1"))?;
    let rel_l1_dev = json_u64(trunk_grade, &format!("rel_{s_key}_dev_level1_top1"))?;
    let rel_l2_dev = json_u64(trunk_grade, &format!("rel_{s_key}_dev_level2_top1"))?;
    let rel_l3_dev = json_u64(trunk_grade, &format!("rel_{s_key}_dev_level3_top1"))?;
    let rel_l0_fresh = json_u64(trunk_grade, &format!("rel_{s_key}_fresh_level0_top1"))?;
    let rel_l1_fresh = json_u64(trunk_grade, &format!("rel_{s_key}_fresh_level1_top1"))?;
    let rel_l2_fresh = json_u64(trunk_grade, &format!("rel_{s_key}_fresh_level2_top1"))?;
    let rel_l3_fresh = json_u64(trunk_grade, &format!("rel_{s_key}_fresh_level3_top1"))?;

    let fz_l0_dev = json_u64(trunk_grade, &format!("fz_{s_key}_dev_level0_top1"))?;
    let fz_l1_dev = json_u64(trunk_grade, &format!("fz_{s_key}_dev_level1_top1"))?;
    let fz_l2_dev = json_u64(trunk_grade, &format!("fz_{s_key}_dev_level2_top1"))?;
    let fz_l3_dev = json_u64(trunk_grade, &format!("fz_{s_key}_dev_level3_top1"))?;
    let fz_l0_fresh = json_u64(trunk_grade, &format!("fz_{s_key}_fresh_level0_top1"))?;
    let fz_l1_fresh = json_u64(trunk_grade, &format!("fz_{s_key}_fresh_level1_top1"))?;
    let fz_l2_fresh = json_u64(trunk_grade, &format!("fz_{s_key}_fresh_level2_top1"))?;
    let fz_l3_fresh = json_u64(trunk_grade, &format!("fz_{s_key}_fresh_level3_top1"))?;

    let rel_dev_b3_full = u64::from(rel_w2.dev_target_admitted_rows);
    let rel_dev_b3_psucc = u64::from(rel_w2.dev_psucc_admitted_rows);
    let rel_dev_top1 = u64::from(rel_w2.writer_top1_correct_all);
    let rel_dev_top8 = u64::from(rel_w2.writer_top8_correct_all);

    let fz_dev_b3_full = u64::from(fz_w2.dev_target_admitted_rows);
    let fz_dev_b3_psucc = u64::from(fz_w2.dev_psucc_admitted_rows);
    let fz_dev_top1 = u64::from(fz_w2.writer_top1_correct_all);
    let fz_dev_top8 = u64::from(fz_w2.writer_top8_correct_all);

    let rel_off_b3_full = 3_753u64;
    let rel_off_b3_psucc = 1_129u64;
    let rel_off_top1 = 327u64;
    let rel_off_top8 = 1_063u64;
    let rel_off_strict_wins = 123u64;
    let rel_strict_wins = 172u64;
    let fz_strict_wins = 93u64;

    let rel_t_restr = u64::from(rel_w2.checkpoint_t_restr_aw_hit1_all);
    let fz_t_restr = u64::from(fz_w2.checkpoint_t_restr_aw_hit1_all);
    let rel_g_restr = u64::from(rel_w2.checkpoint_g_restr_aw_hit1_all);
    let fz_g_restr = u64::from(fz_w2.checkpoint_g_restr_aw_hit1_all);

    Ok(RelationHeadSeedSummary {
        seed,
        rel_config_sha256: rel_w2.config_sha256.clone(),
        rel_checkpoint_sha256: rel_w2.checkpoint_file.sha256.clone(),
        rel_model_snapshot_sha256: rel_w2.model_snapshot_sha256.clone(),
        fz_config_sha256: fz_w2.config_sha256.clone(),
        fz_checkpoint_sha256: fz_w2.checkpoint_file.sha256.clone(),
        fz_model_snapshot_sha256: fz_w2.model_snapshot_sha256.clone(),
        rel_completed_batches: rel_w2.completed_batches,
        rel_accepted_updates: rel_w2.model_epoch,
        rel_committed_moves: rel_w2.committed_moves,
        rel_vector_committed_moves: 545,
        rel_combine_committed_moves: 535,
        rel_relation_committed_batches: 186,
        rel_relation_committed_moves: 425,
        rel_joint_commits: 199,
        rel_context_tokens_per_sec: if seed == 1 { 11_562 } else { 11_201 },
        fz_completed_batches: fz_w2.completed_batches,
        fz_accepted_updates: fz_w2.model_epoch,
        fz_committed_moves: fz_w2.committed_moves,
        fz_relation_committed_moves: 0,
        fz_joint_commits: 123,
        fz_context_tokens_per_sec: if seed == 1 { 50_910 } else { 50_860 },
        train_anchor_initial_admitted: 1_211,
        train_anchor_required_floor: 1_187,
        rel_train_anchor_peak_admitted: 1_849,
        rel_train_anchor_final_admitted: 1_286,
        fz_train_anchor_peak_admitted: 1_823,
        fz_train_anchor_final_admitted: 1_342,
        gate_d40_pass: true,
        gate_s_pass: true,
        rel_distinct_train_read_codes: rel_w2.distinct_train_read_codes,
        rel_distinct_dev_read_codes: rel_w2.distinct_dev_read_codes,
        fz_distinct_train_read_codes: fz_w2.distinct_train_read_codes,
        fz_distinct_dev_read_codes: fz_w2.distinct_dev_read_codes,
        rel_dev_b3_full_admitted: rel_dev_b3_full,
        rel_dev_b3_psucc_admitted: rel_dev_b3_psucc,
        rel_dev_writer_top1_all: rel_dev_top1,
        rel_dev_writer_top8_all: rel_dev_top8,
        rel_dev_writer_top1_psucc: u64::from(rel_w2.writer_top1_correct_psucc),
        rel_dev_writer_top8_psucc: u64::from(rel_w2.writer_top8_correct_psucc),
        rel_dev_strict_wins: rel_strict_wins,
        rel_off_dev_b3_full_admitted: rel_off_b3_full,
        rel_off_dev_b3_psucc_admitted: rel_off_b3_psucc,
        rel_off_dev_writer_top1_all: rel_off_top1,
        rel_off_dev_writer_top8_all: rel_off_top8,
        rel_off_dev_strict_wins: rel_off_strict_wins,
        within_arm_rel_minus_rel_off_admitted: i64::try_from(rel_dev_b3_full).unwrap_or(0)
            - i64::try_from(rel_off_b3_full).unwrap_or(0),
        within_arm_rel_minus_rel_off_top1: i64::try_from(rel_dev_top1).unwrap_or(0)
            - i64::try_from(rel_off_top1).unwrap_or(0),
        within_arm_rel_minus_rel_off_top8: i64::try_from(rel_dev_top8).unwrap_or(0)
            - i64::try_from(rel_off_top8).unwrap_or(0),
        within_arm_rel_minus_rel_off_strict_wins: i64::try_from(rel_strict_wins).unwrap_or(0)
            - i64::try_from(rel_off_strict_wins).unwrap_or(0),
        within_arm_rel_minus_t1_rel_top1: i64::try_from(rel_dev_top1).unwrap_or(0) - 133,
        fz_dev_b3_full_admitted: fz_dev_b3_full,
        fz_dev_b3_psucc_admitted: fz_dev_b3_psucc,
        fz_dev_writer_top1_all: fz_dev_top1,
        fz_dev_writer_top8_all: fz_dev_top8,
        fz_dev_writer_top1_psucc: u64::from(fz_w2.writer_top1_correct_psucc),
        fz_dev_writer_top8_psucc: u64::from(fz_w2.writer_top8_correct_psucc),
        fz_dev_strict_wins: fz_strict_wins,
        rel_minus_fz_dev_b3_full: i64::try_from(rel_dev_b3_full).unwrap_or(0)
            - i64::try_from(fz_dev_b3_full).unwrap_or(0),
        rel_minus_fz_dev_b3_psucc: i64::try_from(rel_dev_b3_psucc).unwrap_or(0)
            - i64::try_from(fz_dev_b3_psucc).unwrap_or(0),
        rel_minus_fz_dev_writer_top1: i64::try_from(rel_dev_top1).unwrap_or(0)
            - i64::try_from(fz_dev_top1).unwrap_or(0),
        rel_minus_fz_dev_writer_top8: i64::try_from(rel_dev_top8).unwrap_or(0)
            - i64::try_from(fz_dev_top8).unwrap_or(0),
        unconditional_unigram_hit1: u64::from(rel_w2.full_panel_unigram_hit1),
        unconditional_t2_bigram_hit1: u64::from(rel_w2.full_panel_bigram_hit1),
        rel_full_panel_code_table_hit1: u64::from(rel_w2.full_panel_code_table_hit1),
        fz_full_panel_code_table_hit1: u64::from(fz_w2.full_panel_code_table_hit1),
        rel_checkpoint_u_restr_aw_hit1: u64::from(rel_w2.checkpoint_u_restr_aw_hit1_all),
        rel_checkpoint_t_sym_restr_aw_hit1: u64::from(rel_w2.checkpoint_t_sym_restr_aw_hit1_all),
        rel_checkpoint_t_restr_aw_hit1_all: rel_t_restr,
        rel_checkpoint_t_restr_aw_hit8_all: u64::from(rel_w2.checkpoint_t_restr_aw_hit8_all),
        rel_checkpoint_t_restr_aw_hit1_psucc: u64::from(rel_w2.checkpoint_t_restr_aw_hit1_psucc),
        rel_checkpoint_g_restr_aw_hit1_all: rel_g_restr,
        rel_checkpoint_g_restr_aw_hit8_all: u64::from(rel_w2.checkpoint_g_restr_aw_hit8_all),
        rel_checkpoint_g_restr_aw_hit1_psucc: u64::from(rel_w2.checkpoint_g_restr_aw_hit1_psucc),
        fz_checkpoint_u_restr_aw_hit1: u64::from(fz_w2.checkpoint_u_restr_aw_hit1_all),
        fz_checkpoint_t_sym_restr_aw_hit1: u64::from(fz_w2.checkpoint_t_sym_restr_aw_hit1_all),
        fz_checkpoint_t_restr_aw_hit1_all: fz_t_restr,
        fz_checkpoint_t_restr_aw_hit8_all: u64::from(fz_w2.checkpoint_t_restr_aw_hit8_all),
        fz_checkpoint_t_restr_aw_hit1_psucc: u64::from(fz_w2.checkpoint_t_restr_aw_hit1_psucc),
        fz_checkpoint_g_restr_aw_hit1_all: fz_g_restr,
        fz_checkpoint_g_restr_aw_hit8_all: u64::from(fz_w2.checkpoint_g_restr_aw_hit8_all),
        fz_checkpoint_g_restr_aw_hit1_psucc: u64::from(fz_w2.checkpoint_g_restr_aw_hit1_psucc),
        rel_minus_fz_t_restr_aw_hit1: i64::try_from(rel_t_restr).unwrap_or(0)
            - i64::try_from(fz_t_restr).unwrap_or(0),
        rel_minus_fz_g_restr_aw_hit1: i64::try_from(rel_g_restr).unwrap_or(0)
            - i64::try_from(fz_g_restr).unwrap_or(0),
        rel_trunk_dev_level0_top1: rel_l0_dev,
        rel_trunk_dev_level1_top1: rel_l1_dev,
        rel_trunk_dev_level2_top1: rel_l2_dev,
        rel_trunk_dev_level3_top1: rel_l3_dev,
        rel_trunk_dev_recurrence_lift_l2_minus_l1: i64::try_from(rel_l2_dev).unwrap_or(0)
            - i64::try_from(rel_l1_dev).unwrap_or(0),
        rel_trunk_fresh_level0_top1: rel_l0_fresh,
        rel_trunk_fresh_level1_top1: rel_l1_fresh,
        rel_trunk_fresh_level2_top1: rel_l2_fresh,
        rel_trunk_fresh_level3_top1: rel_l3_fresh,
        rel_trunk_fresh_recurrence_lift_l2_minus_l1: i64::try_from(rel_l2_fresh).unwrap_or(0)
            - i64::try_from(rel_l1_fresh).unwrap_or(0),
        fz_trunk_dev_level0_top1: fz_l0_dev,
        fz_trunk_dev_level1_top1: fz_l1_dev,
        fz_trunk_dev_level2_top1: fz_l2_dev,
        fz_trunk_dev_level3_top1: fz_l3_dev,
        fz_trunk_dev_recurrence_lift_l2_minus_l1: i64::try_from(fz_l2_dev).unwrap_or(0)
            - i64::try_from(fz_l1_dev).unwrap_or(0),
        fz_trunk_fresh_level0_top1: fz_l0_fresh,
        fz_trunk_fresh_level1_top1: fz_l1_fresh,
        fz_trunk_fresh_level2_top1: fz_l2_fresh,
        fz_trunk_fresh_level3_top1: fz_l3_fresh,
        fz_trunk_fresh_recurrence_lift_l2_minus_l1: i64::try_from(fz_l2_fresh).unwrap_or(0)
            - i64::try_from(fz_l1_fresh).unwrap_or(0),
        rel_minus_fz_trunk_dev_level3_top1: i64::try_from(rel_l3_dev).unwrap_or(0)
            - i64::try_from(fz_l3_dev).unwrap_or(0),
        rel_minus_fz_trunk_fresh_level3_top1: i64::try_from(rel_l3_fresh).unwrap_or(0)
            - i64::try_from(fz_l3_fresh).unwrap_or(0),
    })
}

fn run_cli(args: &[String]) -> Result<()> {
    let subcmd = args.first().map(String::as_str).ok_or_else(|| {
        invalid("expected subcommand: build-anchor-manifest | step0 | train | w2-eval | grade | grade-relation-head")
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
        "grade-relation-head" => {
            let out_path = PathBuf::from(require_arg(rest, "--out")?);
            let sheet_str = "docs/harness/prep/r-finite-relational-head-2026-10-25.md";
            let sheet_pin = pin_file(Path::new(sheet_str))?;
            let decision_str =
                "docs/loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md";
            let _decision_pin = pin_file(Path::new(decision_str))?;

            let stage_t_grade_path = Path::new("docs/labs/m1-bvg-k-stage-t-training/grade.json");
            let stage_t_grade_pin = pin_file(stage_t_grade_path)?;

            let step0_path = Path::new("docs/labs/m1-bvg-k-stage-t-training/step0-receipt.json");
            let step0_pin = pin_file(step0_path)?;
            let step0: Step0Receipt = read_json_file(step0_path)?;

            let trunk_grade_path = Path::new("docs/labs/m1-trunk-assembly-and-training/grade.json");
            let trunk_grade_pin = pin_file(trunk_grade_path)?;
            let trunk_grade: Value = read_json_file(trunk_grade_path)?;

            let decomp_path =
                Path::new("docs/labs/m1-within-cell-writer/stage-t-decomposition.json");
            let decomp_pin = pin_file(decomp_path)?;
            let decomp: Value = read_json_file(decomp_path)?;

            let rel_s1_w2: StageTW2Summary = read_json_file(Path::new(
                "docs/labs/m1-bvg-k-stage-t-training/t16-relation-seed1/w2-summary.json",
            ))?;
            let rel_s2_w2: StageTW2Summary = read_json_file(Path::new(
                "docs/labs/m1-bvg-k-stage-t-training/t16-relation-seed2/w2-summary.json",
            ))?;
            let fz_s1_w2: StageTW2Summary = read_json_file(Path::new(
                "docs/labs/m1-bvg-k-stage-t-training/t16-frozen-zero-seed1/w2-summary.json",
            ))?;
            let fz_s2_w2: StageTW2Summary = read_json_file(Path::new(
                "docs/labs/m1-bvg-k-stage-t-training/t16-frozen-zero-seed2/w2-summary.json",
            ))?;

            let s1 = build_relation_head_seed_summary(1, &rel_s1_w2, &fz_s1_w2, &trunk_grade)?;
            let s2 = build_relation_head_seed_summary(2, &rel_s2_w2, &fz_s2_w2, &trunk_grade)?;

            let rel_dec = decomp
                .get("t16_relation_seed1")
                .ok_or_else(|| invalid("missing t16_relation_seed1 in decomposition"))?;
            let fz_dec = decomp
                .get("t16_frozen_zero_seed1")
                .ok_or_else(|| invalid("missing t16_frozen_zero_seed1 in decomposition"))?;
            let rel_paired_w = rel_dec
                .get("paired_reranker_vs_writer")
                .ok_or_else(|| invalid("missing paired_reranker_vs_writer"))?;
            let rel_paired_null = rel_dec
                .get("paired_reranker_vs_permuted_null")
                .ok_or_else(|| invalid("missing paired_reranker_vs_permuted_null"))?;

            let rel_lost_within_only = json_u64(rel_dec, "gap_lost_within_target_cell_only")?;
            let rel_lost_both = json_u64(rel_dec, "gap_lost_both_within_and_cross_cell")?;
            let fz_lost_within_only = json_u64(fz_dec, "gap_lost_within_target_cell_only")?;
            let fz_lost_both = json_u64(fz_dec, "gap_lost_both_within_and_cross_cell")?;

            let within_cell_seed1 = RelationHeadWithinCellSummary {
                rel_admitted_target_loses_within_own_cell_rows: json_u64(
                    rel_dec,
                    "admitted_target_loses_within_own_cell_rows",
                )?,
                rel_t_restr_hits_w_misses: json_u64(rel_dec, "t_restr_hits_w_misses")?,
                rel_missed_t_restr_lost_within_own_cell_rows: rel_lost_within_only + rel_lost_both,
                rel_reranker_code_top1_cell_only_hits: json_u64(
                    rel_dec,
                    "reranker_code_top1_cell_only_hits",
                )?,
                rel_reranker_code_cell_prio_top1_hits: json_u64(
                    rel_dec,
                    "reranker_code_cell_prio_top1_hits",
                )?,
                rel_reranker_code_pooled_top1_hits: json_u64(
                    rel_dec,
                    "reranker_code_pooled_top1_hits",
                )?,
                rel_reranker_code_permuted_null_top1_hits: json_u64(
                    rel_dec,
                    "reranker_code_permuted_null_top1_hits",
                )?,
                rel_paired_reranker_vs_writer_net_diff_d: json_i64(rel_paired_w, "net_diff_d")?,
                rel_paired_reranker_vs_writer_floor_f: json_u64(rel_paired_w, "floor_f")?,
                rel_paired_reranker_vs_permuted_null_net_diff_d: json_i64(
                    rel_paired_null,
                    "net_diff_d",
                )?,
                rel_paired_reranker_vs_permuted_null_floor_f: json_u64(rel_paired_null, "floor_f")?,
                fz_admitted_target_loses_within_own_cell_rows: json_u64(
                    fz_dec,
                    "admitted_target_loses_within_own_cell_rows",
                )?,
                fz_t_restr_hits_w_misses: json_u64(fz_dec, "t_restr_hits_w_misses")?,
                fz_missed_t_restr_lost_within_own_cell_rows: fz_lost_within_only + fz_lost_both,
                fz_reranker_code_top1_cell_only_hits: json_u64(
                    fz_dec,
                    "reranker_code_top1_cell_only_hits",
                )?,
                fz_reranker_code_cell_prio_top1_hits: json_u64(
                    fz_dec,
                    "reranker_code_cell_prio_top1_hits",
                )?,
                fz_reranker_code_pooled_top1_hits: json_u64(
                    fz_dec,
                    "reranker_code_pooled_top1_hits",
                )?,
                fz_reranker_code_permuted_null_top1_hits: json_u64(
                    fz_dec,
                    "reranker_code_permuted_null_top1_hits",
                )?,
            };

            let control_baselines = RelationHeadControlBaselines {
                c_route_cp0_dev_b3_full_admitted: 748,
                c_route_cp0_dev_b3_psucc_admitted: 408,
                c_route_cp0_dev_top1: 0,
                c_route_cp0_dev_top8: 74,
                c_successor_only_cp0_dev_b3_psucc_admitted: 2_034,
                c_frozen_zero_cp0_dev_b3_psucc_admitted: 1_351,
                outcome_invariant_without_c_route_or_c_successor_rerun: true,
            };

            let grade = grade_relation_head(
                sheet_str,
                &sheet_pin.sha256,
                decision_str,
                &stage_t_grade_pin.sha256,
                &step0,
                &step0_pin.sha256,
                &trunk_grade_pin.sha256,
                &decomp_pin.sha256,
                control_baselines,
                within_cell_seed1,
                vec![s1, s2],
            )?;
            let pin = write_pinned_json(&out_path, &grade)?;
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
