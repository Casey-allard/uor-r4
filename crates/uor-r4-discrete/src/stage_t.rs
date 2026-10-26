//! Stage T (`m1-bvg-k-stage-t-training`, `#2836`) batched-coordinate (`BVG-K`, `K = 16`)
//! D40-audited training controller, Step 0 GPU instrument controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`),
//! straddle-aware block evidence accounting (`Combine` gates `0..20` vs `Relation` gates `20..45`),
//! checkpoint-restricted W2 count-table evaluation (`T_restr(A_W)` and `G_restr(A_W)`),
//! and exact integer Stage 2 grading across two seeds (`seed 1`, `seed 2`).
//!
//! All model forward, scoring, verification, commit, and evaluation operations run on CUDA (`CHARTER §3`).
//! Host helpers perform only schedule accounting, integer grading, and JSON receipt serialization (`D32`).

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::address_cells::AddressMetric;
use crate::attention_anchor::{AnchorPolicy, TrainAnchorManifest};
use crate::controller::attention_audited::AuditPolicy;
use crate::corpus::{self, FilePin};
use crate::metadata::BinderPlan;
use crate::training::{
    Block, BlockReceipt, CodebookInitialization, CoordinateKind, CounterfactualPath, Cursor,
    DriverConfig, RelationLearningPolicy, SchedulePolicy, StepReceipt,
};
use crate::{Error, ObjectivePolicy, Parameter, ReadInitialization, ReadSpec, Result};

pub const PSUCC_ROWS: usize = 3_847;
pub const EXPECTED_ANCHOR_MANIFEST_SHA256: &str =
    "f85466ff339d8fb848fb1151a7d672fba5687296b81b1ad6fb39c36ddaca27de";
pub const EXPECTED_ANCHOR_CSV_SHA256: &str =
    "8c1b80e9b15e4e2bcc765037afa7800b785418be3e20b8cc80ce13ee2e32b3c1";

pub const STAGE_T_STEP0_SCHEMA: &str = "uor-r4.stage-t-step0-controls/v1";
pub const STAGE_T_RUN_SCHEMA: &str = "uor-r4.stage-t-audited-run/v1";
pub const STAGE_T_W2_SCHEMA: &str = "uor-r4.stage-t-w2-restricted-eval/v1";
pub const STAGE_T_GRADE_SCHEMA: &str = "uor-r4.stage-t-grade/v1";
pub const RELATION_HEAD_GRADE_SCHEMA: &str = "uor-r4.relation-head-grade/v1";
pub const RELATION_HEAD_MIN_COMMITS: u64 = 16;
pub const RELATION_HEAD_D40_TRAIN_FLOOR: u64 = 1_187;
pub const RELATION_HEAD_STAGE_T_ARCHIVE_SHA256: &str =
    "c004db506bf64903231c5c225a07e9969d5e3344c51a67eaec9429265e5f44b6";
pub const RELATION_HEAD_TRUNK_ARCHIVE_SHA256: &str =
    "4f4d4817e2750565f65bdb8a6c6f8ed07b44389f0757702cf6eba11b64e23d35";

pub const A8_STRUCTURED_TABLE_SHA256: &str =
    "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1";
pub const STAGE_T_MAX_BATCHES: u64 = 16_404;
pub const STAGE_T_Q1_BATCHES: u64 = 4_096;
pub const STAGE_T_AUDIT_CADENCE: u64 = 64;

/// Registered Stage 2 bars on the 14,287-row DEV panel (`3,847` `P_succ` rows).
pub const DEV_FULL_ADMISSION_BAR: u64 = 4_220;
pub const DEV_PSUCC_ADMISSION_BAR: u64 = 676;
pub const UNIGRAM_FULL_BAR: u64 = 1_226;
pub const UNIGRAM_PLUS_2SE_BAR: u64 = 1_344;
pub const W2_BASELINE_CODE_RESTR_BAR: u64 = 1_138;
pub const W2_BASELINE_BIGRAM_RESTR_BAR: u64 = 1_482;
pub const FULL_PANEL_T2_REFERENCE: u64 = 6_300;
pub const MCNEMAR_FLOOR_FULL_DEV: u64 = 286;

fn invalid(msg: impl Into<String>) -> Error {
    Error::Invalid(msg.into())
}

/// Extract total committed moves from a `Cursor` (`cursor.block.committed_moves` for `K > 1`, `accepted_updates` for `K = 1`).
pub fn cursor_committed_moves(cursor: &Cursor) -> u64 {
    cursor
        .block
        .map_or(cursor.accepted_updates, |b| b.committed_moves)
}

/// Exact integer `ceil(2 * sqrt(n))`: smallest `k >= 0` such that `k * k >= 4 * n`.
pub fn integer_ceil_two_sqrt(n: u64) -> u64 {
    if n == 0 {
        return 0;
    }
    let target = n.saturating_mul(4);
    let mut low = 1u64;
    let mut high = 2_000_000_000u64;
    while low < high {
        let mid = low + (high - low) / 2;
        if mid.saturating_mul(mid) >= target {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    low
}

/// Coordinate family inside the Relation/A8 parameter inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateFamily {
    Vector,
    Combine,
    Relation,
}

/// Classification of a coordinate block's family composition.
/// At `K = 16` under `InterleavedReadEvery8V1`, read blocks (`completed_batches % 8 == 7`)
/// draw 16 coordinates from the 45 gates (`0..20` Combine, `20..45` Relation):
/// - `o_read % 45 == 0` -> `Gate(0..16)` (`CombineOnly`)
/// - `o_read % 45 == 16` -> `Gate(16..32)` (`CombineRelationStraddle`: 4 Combine + 12 Relation)
/// - `o_read % 45 == 32` -> `Gate(32..45)` (`RelationOnly`: 13 Relation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockComposition {
    VectorOnly,
    CombineOnly,
    CombineRelationStraddle,
    RelationOnly,
}

/// Coordinate-level and batch-level counters for one parameter family (`Vector`, `Combine`, `Relation`).
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockFamilyCounters {
    pub attempted_batches: u64,
    pub attempted_coordinates: u64,
    pub eligible_coordinates: u64,
    pub policy_refused_coordinates: u64,
    pub sensitive_batches: u64,
    pub provisional_applied_batches: u64,
    pub provisional_committed_moves: u64,
    pub committed_applied_batches: u64,
    pub committed_moves: u64,
    pub rejected_applied_batches: u64,
    pub rejected_moves: u64,
}

/// Cumulative straddle-aware counters across all three families and block compositions.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTCounters {
    pub attempted_batches: u64,
    pub vector_only_batches: u64,
    pub combine_only_batches: u64,
    pub combine_relation_straddle_batches: u64,
    pub relation_only_batches: u64,
    pub provisional_accepted_updates: u64,
    pub provisional_committed_moves: u64,
    pub provisional_joint_commits: u64,
    pub committed_accepted_updates: u64,
    pub committed_moves: u64,
    pub committed_joint_commits: u64,
    pub rejected_accepted_updates: u64,
    pub rejected_moves: u64,
    pub rejected_joint_commits: u64,
    pub vector: BlockFamilyCounters,
    pub combine: BlockFamilyCounters,
    pub relation: BlockFamilyCounters,
}

/// Pending provisional counters accumulated within a single 64-batch interval before the D40 audit.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalDelta {
    pub batches: u64,
    pub accepted_updates: u64,
    pub committed_moves: u64,
    pub joint_commits: u64,
    pub vector_applied_batches: u64,
    pub vector_moves: u64,
    pub combine_applied_batches: u64,
    pub combine_moves: u64,
    pub relation_applied_batches: u64,
    pub relation_moves: u64,
}

/// Return `(combine_gates, total_gates)` for a `DriverConfig` (`(20, 45)` for C192, B4, Relation8/sweep1).
pub fn gate_inventory(config: &DriverConfig) -> Result<(u32, u32)> {
    let stages = BinderPlan::new(config.spec.context as usize)?.stages() as u32;
    let combine = config.spec.blocks * stages;
    let total = config.read_spec.gate_count(config.spec.blocks, stages)?;
    Ok((combine, total))
}

/// Classify a single `Parameter` into `Vector`, `Combine`, or `Relation`.
pub fn classify_parameter(config: &DriverConfig, parameter: Parameter) -> Result<CoordinateFamily> {
    let (combine, total) = gate_inventory(config)?;
    match parameter {
        Parameter::VectorBlock(idx) if idx < config.spec.vocabulary * config.spec.blocks => {
            Ok(CoordinateFamily::Vector)
        }
        Parameter::Gate(idx) if idx < combine => Ok(CoordinateFamily::Combine),
        Parameter::Gate(idx) if idx < total => Ok(CoordinateFamily::Relation),
        _ => Err(invalid(
            "Stage T coordinate is outside the admitted VectorBlock/Combine/Relation inventory",
        )),
    }
}

/// Compute the per-coordinate eligibility flags for a `Block` under `config.relation_learning_policy`.
pub fn block_eligibility(config: &DriverConfig, block: &Block) -> Result<Vec<bool>> {
    let mut eligible = Vec::with_capacity(block.coordinates.len());
    for &param in &block.coordinates {
        let fam = classify_parameter(config, param)?;
        let is_eligible = !matches!(
            (fam, config.relation_learning_policy),
            (
                CoordinateFamily::Relation,
                RelationLearningPolicy::FrozenZeroV1
            )
        );
        eligible.push(is_eligible);
    }
    Ok(eligible)
}

/// Classify a `Block`'s family composition.
pub fn classify_block(config: &DriverConfig, block: &Block) -> Result<BlockComposition> {
    let mut has_vector = false;
    let mut has_combine = false;
    let mut has_relation = false;
    for &param in &block.coordinates {
        match classify_parameter(config, param)? {
            CoordinateFamily::Vector => has_vector = true,
            CoordinateFamily::Combine => has_combine = true,
            CoordinateFamily::Relation => has_relation = true,
        }
    }
    match (has_vector, has_combine, has_relation) {
        (true, false, false) => Ok(BlockComposition::VectorOnly),
        (false, true, false) => Ok(BlockComposition::CombineOnly),
        (false, true, true) => Ok(BlockComposition::CombineRelationStraddle),
        (false, false, true) => Ok(BlockComposition::RelationOnly),
        _ => Err(invalid(
            "Stage T block mixes vector coordinates with read gates or is empty",
        )),
    }
}

impl StageTCounters {
    /// Record one `transition_block` step (`K = 1` or `K = 16`) into the cumulative and interval counters.
    pub fn record_block_step(
        &mut self,
        interval: &mut IntervalDelta,
        config: &DriverConfig,
        block: &Block,
        receipt: &BlockReceipt,
    ) -> Result<BlockComposition> {
        let comp = classify_block(config, block)?;
        let eligible = block_eligibility(config, block)?;
        self.attempted_batches += 1;
        interval.batches += 1;
        match comp {
            BlockComposition::VectorOnly => self.vector_only_batches += 1,
            BlockComposition::CombineOnly => self.combine_only_batches += 1,
            BlockComposition::CombineRelationStraddle => {
                self.combine_relation_straddle_batches += 1
            }
            BlockComposition::RelationOnly => self.relation_only_batches += 1,
        }

        let mut vec_in_block = false;
        let mut comb_in_block = false;
        let mut rel_in_block = false;
        for (&param, &elig) in block.coordinates.iter().zip(&eligible) {
            let fam = classify_parameter(config, param)?;
            let target = match fam {
                CoordinateFamily::Vector => {
                    vec_in_block = true;
                    &mut self.vector
                }
                CoordinateFamily::Combine => {
                    comb_in_block = true;
                    &mut self.combine
                }
                CoordinateFamily::Relation => {
                    rel_in_block = true;
                    &mut self.relation
                }
            };
            target.attempted_coordinates += 1;
            if elig {
                target.eligible_coordinates += 1;
            } else {
                target.policy_refused_coordinates += 1;
            }
        }
        if vec_in_block {
            self.vector.attempted_batches += 1;
        }
        if comb_in_block {
            self.combine.attempted_batches += 1;
        }
        if rel_in_block {
            self.relation.attempted_batches += 1;
        }

        let sensitive = receipt.s_len > 0 || receipt.l_best != receipt.l_before;
        if sensitive {
            if vec_in_block {
                self.vector.sensitive_batches += 1;
            }
            if comb_in_block {
                self.combine.sensitive_batches += 1;
            }
            if rel_in_block && config.relation_learning_policy == RelationLearningPolicy::LearnV1 {
                self.relation.sensitive_batches += 1;
            }
        }

        let moves = receipt.commit_ordinals.len() as u64;
        let mut vec_moves = 0u64;
        let mut comb_moves = 0u64;
        let mut rel_moves = 0u64;
        for &ord in &receipt.commit_ordinals {
            let idx = ord as usize;
            let param = *block
                .coordinates
                .get(idx)
                .ok_or_else(|| invalid("KId violation: commit ordinal out of block bounds"))?;
            let elig = *eligible
                .get(idx)
                .ok_or_else(|| invalid("KId violation: eligible ordinal out of block bounds"))?;
            if !elig {
                return Err(invalid(
                    "V8d/KId violation: committed an ineligible coordinate ordinal",
                ));
            }
            match classify_parameter(config, param)? {
                CoordinateFamily::Vector => vec_moves += 1,
                CoordinateFamily::Combine => comb_moves += 1,
                CoordinateFamily::Relation => {
                    if config.relation_learning_policy == RelationLearningPolicy::FrozenZeroV1 {
                        return Err(invalid(
                            "V8d violation: FrozenZeroV1 committed a relation coordinate",
                        ));
                    }
                    rel_moves += 1;
                }
            }
        }

        if moves > 0 {
            if receipt.committed_loss >= receipt.l_before {
                return Err(invalid(
                    "KId violation: committed block did not strictly decrease loss",
                ));
            }
            self.provisional_accepted_updates += 1;
            self.provisional_committed_moves += moves;
            interval.accepted_updates += 1;
            interval.committed_moves += moves;
            if receipt.joint {
                self.provisional_joint_commits += 1;
                interval.joint_commits += 1;
            }
            if vec_moves > 0 {
                self.vector.provisional_applied_batches += 1;
                self.vector.provisional_committed_moves += vec_moves;
                interval.vector_applied_batches += 1;
                interval.vector_moves += vec_moves;
            }
            if comb_moves > 0 {
                self.combine.provisional_applied_batches += 1;
                self.combine.provisional_committed_moves += comb_moves;
                interval.combine_applied_batches += 1;
                interval.combine_moves += comb_moves;
            }
            if rel_moves > 0 {
                self.relation.provisional_applied_batches += 1;
                self.relation.provisional_committed_moves += rel_moves;
                interval.relation_applied_batches += 1;
                interval.relation_moves += rel_moves;
            }
        }
        Ok(comp)
    }

    /// Record one ordinary `K = 1` `transition` step into the cumulative and interval counters.
    pub fn record_single_step(
        &mut self,
        interval: &mut IntervalDelta,
        config: &DriverConfig,
        receipt: &StepReceipt,
    ) -> Result<BlockComposition> {
        let param = match receipt.coordinate.kind {
            CoordinateKind::VectorBlock => Parameter::VectorBlock(receipt.coordinate.index),
            CoordinateKind::Route => Parameter::Route(receipt.coordinate.index),
            CoordinateKind::Gate => Parameter::Gate(receipt.coordinate.index),
        };
        let fam = classify_parameter(config, param)?;
        self.attempted_batches += 1;
        interval.batches += 1;
        let comp = match fam {
            CoordinateFamily::Vector => {
                self.vector_only_batches += 1;
                BlockComposition::VectorOnly
            }
            CoordinateFamily::Combine => {
                self.combine_only_batches += 1;
                BlockComposition::CombineOnly
            }
            CoordinateFamily::Relation => {
                self.relation_only_batches += 1;
                BlockComposition::RelationOnly
            }
        };
        let target = match fam {
            CoordinateFamily::Vector => &mut self.vector,
            CoordinateFamily::Combine => &mut self.combine,
            CoordinateFamily::Relation => &mut self.relation,
        };
        target.attempted_batches += 1;
        target.attempted_coordinates += 1;
        if receipt.apply_refused_by_policy {
            target.policy_refused_coordinates += 1;
        } else {
            target.eligible_coordinates += 1;
        }
        if receipt.any_candidate_cost_differs_from_identity {
            target.sensitive_batches += 1;
        }
        if receipt.applied {
            if fam == CoordinateFamily::Relation
                && config.relation_learning_policy == RelationLearningPolicy::FrozenZeroV1
            {
                return Err(invalid(
                    "V8d violation: FrozenZeroV1 applied a relation coordinate",
                ));
            }
            target.provisional_applied_batches += 1;
            target.provisional_committed_moves += 1;
            self.provisional_accepted_updates += 1;
            self.provisional_committed_moves += 1;
            interval.accepted_updates += 1;
            interval.committed_moves += 1;
            match fam {
                CoordinateFamily::Vector => {
                    interval.vector_applied_batches += 1;
                    interval.vector_moves += 1;
                }
                CoordinateFamily::Combine => {
                    interval.combine_applied_batches += 1;
                    interval.combine_moves += 1;
                }
                CoordinateFamily::Relation => {
                    interval.relation_applied_batches += 1;
                    interval.relation_moves += 1;
                }
            }
        }
        Ok(comp)
    }

    /// Settle a 64-batch interval after the D40 TRAIN anchor audit (`rejected == false` promotes to committed; `rejected == true` rolls back).
    pub fn settle_interval(&mut self, interval: &IntervalDelta, rejected: bool) {
        if !rejected {
            self.committed_accepted_updates += interval.accepted_updates;
            self.committed_moves += interval.committed_moves;
            self.committed_joint_commits += interval.joint_commits;
            self.vector.committed_applied_batches += interval.vector_applied_batches;
            self.vector.committed_moves += interval.vector_moves;
            self.combine.committed_applied_batches += interval.combine_applied_batches;
            self.combine.committed_moves += interval.combine_moves;
            self.relation.committed_applied_batches += interval.relation_applied_batches;
            self.relation.committed_moves += interval.relation_moves;
        } else {
            self.rejected_accepted_updates += interval.accepted_updates;
            self.rejected_moves += interval.committed_moves;
            self.rejected_joint_commits += interval.joint_commits;
            self.vector.rejected_applied_batches += interval.vector_applied_batches;
            self.vector.rejected_moves += interval.vector_moves;
            self.combine.rejected_applied_batches += interval.combine_applied_batches;
            self.combine.rejected_moves += interval.combine_moves;
            self.relation.rejected_applied_batches += interval.relation_applied_batches;
            self.relation.rejected_moves += interval.relation_moves;
        }
    }
}

/// Validate that a `DriverConfig` is an admitted Stage T (`K = 16` or `K = 1`, `seed = 1` or `seed = 2`) A8 configuration.
pub fn validate_stage_t_config(config: &DriverConfig) -> Result<()> {
    config.validate()?;
    if config.spec.vocabulary != 4_096
        || config.spec.blocks != 4
        || config.spec.context != 192
        || config.spec.kept_cells != 8
        || !matches!(config.read_spec, ReadSpec::Relation { kept: 8, sweeps: 1 })
        || config.read_initialization != ReadInitialization::ActiveLeafV1
        || config.schedule != SchedulePolicy::InterleavedReadEvery8V1
        || config.objective_policy != ObjectivePolicy::NearestAddressMarginV1
        || !config.normalization_mode.is_scale_matched_snap()
        || config.position.is_some()
        || config.address_metric != AddressMetric::OuterL1V1
        || config.counterfactual_path != CounterfactualPath::RowsV1
        || !matches!(config.coordinate_block.get(), 1 | 16)
        || !matches!(config.seed, 1 | 2)
        || config.batch_rows != 64
    {
        return Err(invalid(
            "Stage T requires V4096/B4/C192/K8, Relation8/sweep1, ActiveLeafV1, InterleavedReadEvery8V1, NearestAddressMarginV1, OuterL1V1, rows_v1, coordinate_block in {1, 16}, seed in {1, 2}, and batch_rows = 64",
        ));
    }
    match &config.codebook_initialization {
        CodebookInitialization::PinnedTableV1 { table }
            if table.sha256 == A8_STRUCTURED_TABLE_SHA256 => {}
        _ => {
            return Err(invalid(
                "Stage T requires the pinned A8 address-structured codebook table (6600bdb1...)",
            ))
        }
    }
    Ok(())
}

/// Receipt for the five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) and throughput probe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step0Receipt {
    pub schema: String,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub driver_source_sha256: String,
    pub t16_fz_config_sha256: String,
    pub t16_rel_config_sha256: String,
    pub t1_fz_config_sha256: String,
    pub akd_single_move_commits_checked: u32,
    pub akd_mismatches: u32,
    pub akd_pass: bool,
    pub v1d_batches_checked: u32,
    pub v1d_mismatches: u32,
    pub v1d_final_snapshot_sha256: String,
    pub v1d_pass: bool,
    pub kid_batches_checked: u32,
    pub kid_commits_rescored: u32,
    pub kid_ineligible_noop_verified: bool,
    pub kid_mismatches: u32,
    pub kid_pass: bool,
    pub v8d_batches_checked: u32,
    pub v8d_relation_commits_on_frozen_zero: u32,
    pub v8d_nonzero_relation_trits: u32,
    pub v8d_pass: bool,
    pub v9d_uninterrupted_128_snapshot_sha256: String,
    pub v9d_resumed_128_snapshot_sha256: String,
    pub v9d_epoch_128: u64,
    pub v9d_committed_moves_128: u64,
    pub v9d_pass: bool,
    pub t16_128_batches_elapsed_ms: u64,
    pub projected_16404_seconds: u64,
    pub all_pass: bool,
}

/// Summary of one 64-batch D40 interval audit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntervalAuditRecord {
    pub audit_index: u32,
    pub attempted_batches: u64,
    pub interval_start_epoch: u64,
    pub inspected_epoch: u64,
    pub post_audit_epoch: u64,
    pub interval_provisional_updates: u64,
    pub interval_provisional_moves: u64,
    pub interval_joint_commits: u64,
    pub train_anchor_admitted: u64,
    pub required_floor: u64,
    pub rejected: bool,
    pub consecutive_full_rejections: u32,
    pub post_audit_snapshot_sha256: String,
}

/// Extracted DEV evaluation metrics at a milestone checkpoint (`baseline`, `Q1`, `final`, `relation_off`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DevMilestoneSummary {
    pub label: String,
    pub completed_batches: u64,
    pub model_epoch: u64,
    pub committed_moves: u64,
    pub model_snapshot_sha256: String,
    pub evaluation_file: FilePin,
    pub rows: u64,
    pub target_admitted_rows: u64,
    pub psucc_rows: u64,
    pub psucc_target_admitted_rows: u64,
    pub complement_target_admitted_rows: u64,
    pub top1_correct_rows: u64,
    pub top8_correct_rows: u64,
    pub admitted_strict_win_rows: u64,
}

/// Terminal status of a Stage T audited run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StageTTerminalStatus {
    CompletedFullDose,
    CompletedProbe,
    DiagnosticStopConsecutiveRejections,
    DiagnosticStopQ1,
}

/// Checkpoint-restricted W2 table evaluation summary (`Step 3` of the sheet).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTW2Summary {
    pub schema: String,
    pub config_sha256: String,
    pub checkpoint_file: FilePin,
    pub model_snapshot_sha256: String,
    pub model_epoch: u64,
    pub completed_batches: u64,
    pub committed_moves: u64,
    pub train_rows_captured: usize,
    pub dev_rows_captured: usize,
    pub distinct_train_read_codes: usize,
    pub distinct_dev_read_codes: usize,
    pub dev_target_admitted_rows: u32,
    pub dev_psucc_admitted_rows: u32,
    pub writer_top1_correct_all: u32,
    pub writer_top8_correct_all: u32,
    pub writer_top1_correct_psucc: u32,
    pub writer_top8_correct_psucc: u32,
    pub full_panel_unigram_hit1: u32,
    pub full_panel_bigram_hit1: u32,
    pub full_panel_code_table_hit1: u32,
    pub w2_baseline_code_restr_q4_hit1: u32,
    pub w2_baseline_bigram_restr_q4_hit1: u32,
    pub checkpoint_t_restr_aw_hit1_all: u32,
    pub checkpoint_t_restr_aw_hit8_all: u32,
    pub checkpoint_u_restr_aw_hit1_all: u32,
    pub checkpoint_g_restr_aw_hit1_all: u32,
    pub checkpoint_g_restr_aw_hit8_all: u32,
    pub checkpoint_t_sym_restr_aw_hit1_all: u32,
    pub checkpoint_t_restr_aw_hit1_psucc: u32,
    pub checkpoint_g_restr_aw_hit1_psucc: u32,
    pub elapsed_ms: u64,
}

/// Full report written by `stage-t-run train`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTRunReport {
    pub schema: String,
    pub arm_label: String,
    pub seed: u32,
    pub coordinate_block: u32,
    pub relation_learning_policy: RelationLearningPolicy,
    pub is_full_dose: bool,
    pub max_batches: u64,
    pub terminal_status: StageTTerminalStatus,
    pub stop_reasons: Vec<String>,
    pub source_sha256: String,
    pub kernel_sha256: String,
    pub geometry_sha256: String,
    pub driver_source_sha256: String,
    pub config_file: FilePin,
    pub anchor_manifest_file: FilePin,
    pub sheet_file: FilePin,
    pub audit_policy: AuditPolicy,
    pub initial_train_anchor_admitted: u64,
    pub train_anchor_required_floor: u64,
    pub peak_train_anchor_admitted: u64,
    pub final_train_anchor_admitted: u64,
    pub total_audits: u32,
    pub rejected_audits: u32,
    pub counters: StageTCounters,
    pub baseline_dev: DevMilestoneSummary,
    pub q1_dev: Option<DevMilestoneSummary>,
    pub final_dev: DevMilestoneSummary,
    pub relation_off_dev: Option<DevMilestoneSummary>,
    pub final_checkpoint_file: FilePin,
    pub w2_summary_file: Option<FilePin>,
    pub w2_summary: Option<StageTW2Summary>,
    pub interval_audits: Vec<IntervalAuditRecord>,
    pub fresh_slice_status: String,
    pub fresh_evaluations: u32,
    pub elapsed_ms: u64,
}

/// Per-seed Stage 2 evaluation record used by `grade_stage_t`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeedStageTEval {
    pub arm_label: String,
    pub seed: u32,
    pub coordinate_block: u32,
    pub terminal_status: StageTTerminalStatus,
    pub completed_batches: u64,
    pub accepted_updates: u64,
    pub committed_moves: u64,
    pub dev_target_admitted_rows: u64,
    pub dev_psucc_admitted_rows: u64,
    pub dev_top1_correct: u64,
    pub dev_top8_correct: u64,
    pub t1_control_top1_correct: u64,
    pub t1_control_halted_early: bool,
    pub mcnemar_n10_t16_only: u64,
    pub mcnemar_n01_t1_only: u64,
    pub mcnemar_required_delta: u64,
    pub checkpoint_t_restr_aw_hit1: u64,
    pub checkpoint_g_restr_aw_hit1: u64,
    pub checkpoint_u_restr_aw_hit1: u64,
    pub g_adm_pass: bool,
    pub beats_unigram: bool,
    pub beats_unigram_plus_2se: bool,
    pub beats_w2_baseline_code_restr: bool,
    pub beats_w2_baseline_bigram_restr: bool,
    pub beats_checkpoint_t_restr_aw: bool,
    pub beats_checkpoint_g_restr_aw: bool,
    pub b_adm_table_pass: bool,
    pub b_block_gain_pass: bool,
    pub beats_full_panel_t2: bool,
}

/// Pre-registered Stage T outcome verdict (`docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md` Section 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StageTVerdict {
    PassStage2AdmTable,
    PartialUnigramOnly,
    DiagnosticStopConsecutiveRejections,
    HTie,
}

/// Complete two-seed Stage T grade report (`grade.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageTGradeReport {
    pub schema: String,
    pub item: String,
    pub issue: u32,
    pub sheet_path: String,
    pub sheet_sha256: String,
    pub step0_all_pass: bool,
    pub step0_receipt_sha256: String,
    pub primary_arm: String,
    pub verdict: StageTVerdict,
    pub stage2_bar_met_two_seeds: bool,
    pub unigram_bar_met_two_seeds: bool,
    pub w2_code_table_bar_met_two_seeds: bool,
    pub w2_bigram_table_bar_met_two_seeds: bool,
    pub m1_full_t2_bar_met_two_seeds: bool,
    pub mechanism_negative: bool,
    pub unlocked_next_items: Vec<String>,
    pub two_seed_noise_halfwidth_rows: u64,
    pub frozen_zero_seeds: Vec<SeedStageTEval>,
    pub relation_seeds: Vec<SeedStageTEval>,
    pub fresh_evaluations: u32,
}

/// Build a `SeedStageTEval` from a seed's `StageTRunReport`, its `StageTW2Summary`, and the matched `T1` control counts.
pub fn evaluate_seed_against_bars(
    report: &StageTRunReport,
    w2: &StageTW2Summary,
    t1_control_top1_correct: u64,
    t1_control_halted_early: bool,
    mcnemar_n10_t16_only: u64,
    mcnemar_n01_t1_only: u64,
) -> Result<SeedStageTEval> {
    if !report.is_full_dose {
        return Err(invalid(
            "Stage T grading refuses a short probe run; max_batches must be 16404",
        ));
    }
    if report.final_dev.model_snapshot_sha256 != w2.model_snapshot_sha256 {
        return Err(invalid(
            "Stage T W2 summary model_snapshot_sha256 differs from final_dev",
        ));
    }
    let dev_target_admitted_rows = report.final_dev.target_admitted_rows;
    let dev_psucc_admitted_rows = report.final_dev.psucc_target_admitted_rows;
    let dev_top1_correct = report.final_dev.top1_correct_rows;
    let dev_top8_correct = report.final_dev.top8_correct_rows;
    if dev_top1_correct != u64::from(w2.writer_top1_correct_all) {
        return Err(invalid(
            "Stage T W2 writer_top1_correct_all disagrees with final_dev top1_correct_rows",
        ));
    }

    let g_adm_pass = dev_target_admitted_rows >= DEV_FULL_ADMISSION_BAR
        && dev_psucc_admitted_rows >= DEV_PSUCC_ADMISSION_BAR;
    let beats_unigram = dev_top1_correct > UNIGRAM_FULL_BAR;
    let beats_unigram_plus_2se = dev_top1_correct >= UNIGRAM_PLUS_2SE_BAR;
    let beats_w2_baseline_code_restr = dev_top1_correct > W2_BASELINE_CODE_RESTR_BAR;
    let beats_w2_baseline_bigram_restr = dev_top1_correct > W2_BASELINE_BIGRAM_RESTR_BAR;

    let checkpoint_t_restr_aw_hit1 = u64::from(w2.checkpoint_t_restr_aw_hit1_all);
    let checkpoint_g_restr_aw_hit1 = u64::from(w2.checkpoint_g_restr_aw_hit1_all);
    let checkpoint_u_restr_aw_hit1 = u64::from(w2.checkpoint_u_restr_aw_hit1_all);

    let beats_checkpoint_t_restr_aw = dev_top1_correct > checkpoint_t_restr_aw_hit1;
    let beats_checkpoint_g_restr_aw = dev_top1_correct > checkpoint_g_restr_aw_hit1;

    let b_adm_table_pass = beats_w2_baseline_code_restr
        && beats_w2_baseline_bigram_restr
        && beats_checkpoint_t_restr_aw;

    let discordant = mcnemar_n10_t16_only.saturating_add(mcnemar_n01_t1_only);
    let mcnemar_required_delta = integer_ceil_two_sqrt(discordant).max(MCNEMAR_FLOOR_FULL_DEV);
    let diff_over_t1 = dev_top1_correct.saturating_sub(t1_control_top1_correct);
    let b_block_gain_pass = diff_over_t1 >= mcnemar_required_delta
        || (t1_control_halted_early && beats_unigram && b_adm_table_pass);

    let beats_full_panel_t2 = dev_top1_correct > FULL_PANEL_T2_REFERENCE;

    Ok(SeedStageTEval {
        arm_label: report.arm_label.clone(),
        seed: report.seed,
        coordinate_block: report.coordinate_block,
        terminal_status: report.terminal_status,
        completed_batches: report.final_dev.completed_batches,
        accepted_updates: report.final_dev.model_epoch,
        committed_moves: report.final_dev.committed_moves,
        dev_target_admitted_rows,
        dev_psucc_admitted_rows,
        dev_top1_correct,
        dev_top8_correct,
        t1_control_top1_correct,
        t1_control_halted_early,
        mcnemar_n10_t16_only,
        mcnemar_n01_t1_only,
        mcnemar_required_delta,
        checkpoint_t_restr_aw_hit1,
        checkpoint_g_restr_aw_hit1,
        checkpoint_u_restr_aw_hit1,
        g_adm_pass,
        beats_unigram,
        beats_unigram_plus_2se,
        beats_w2_baseline_code_restr,
        beats_w2_baseline_bigram_restr,
        beats_checkpoint_t_restr_aw,
        beats_checkpoint_g_restr_aw,
        b_adm_table_pass,
        b_block_gain_pass,
        beats_full_panel_t2,
    })
}

fn arm_verdict(seeds: &[SeedStageTEval]) -> Result<StageTVerdict> {
    if seeds.len() != 2 || seeds[0].seed != 1 || seeds[1].seed != 2 {
        return Err(invalid(
            "Stage T requires exactly two seeds (seed 1 and seed 2) per graded arm",
        ));
    }
    let both_pass_stage2 = seeds.iter().all(|s| {
        s.g_adm_pass && s.beats_unigram_plus_2se && s.b_adm_table_pass && s.b_block_gain_pass
    });
    if both_pass_stage2 {
        return Ok(StageTVerdict::PassStage2AdmTable);
    }
    let both_partial_unigram = seeds.iter().all(|s| {
        s.g_adm_pass
            && s.beats_unigram
            && s.beats_w2_baseline_code_restr
            && s.dev_top1_correct > s.t1_control_top1_correct
    });
    if both_partial_unigram {
        return Ok(StageTVerdict::PartialUnigramOnly);
    }
    let any_diag_stop = seeds.iter().any(|s| {
        matches!(
            s.terminal_status,
            StageTTerminalStatus::DiagnosticStopConsecutiveRejections
                | StageTTerminalStatus::DiagnosticStopQ1
        )
    });
    if any_diag_stop {
        Ok(StageTVerdict::DiagnosticStopConsecutiveRejections)
    } else {
        Ok(StageTVerdict::HTie)
    }
}

fn verdict_rank(v: StageTVerdict) -> u8 {
    match v {
        StageTVerdict::PassStage2AdmTable => 3,
        StageTVerdict::PartialUnigramOnly => 2,
        StageTVerdict::HTie => 1,
        StageTVerdict::DiagnosticStopConsecutiveRejections => 0,
    }
}

/// Grade two-seed Stage T runs (`T16_fz` and `T16_rel`) once Step 0 controls pass.
pub fn grade_stage_t(
    sheet_path: &str,
    sheet_sha256: &str,
    step0: &Step0Receipt,
    step0_receipt_sha256: &str,
    frozen_zero_seeds: Vec<SeedStageTEval>,
    relation_seeds: Vec<SeedStageTEval>,
) -> Result<StageTGradeReport> {
    if !step0.all_pass
        || !step0.akd_pass
        || !step0.v1d_pass
        || !step0.kid_pass
        || !step0.v8d_pass
        || !step0.v9d_pass
    {
        return Err(invalid(
            "Step 0 GPU control failed; CHARTER §4 forbids grading training metrics on a failed instrument",
        ));
    }
    let fz_verdict = arm_verdict(&frozen_zero_seeds)?;
    let rel_verdict = arm_verdict(&relation_seeds)?;
    let (primary_arm, verdict, chosen) = if verdict_rank(rel_verdict) > verdict_rank(fz_verdict) {
        ("T16_rel".to_owned(), rel_verdict, &relation_seeds)
    } else {
        ("T16_fz".to_owned(), fz_verdict, &frozen_zero_seeds)
    };

    let seed_gap = chosen[0]
        .dev_top1_correct
        .abs_diff(chosen[1].dev_top1_correct);
    let two_seed_noise_halfwidth_rows = 2 * seed_gap.max(59);

    let stage2_bar_met_two_seeds = verdict == StageTVerdict::PassStage2AdmTable;
    let unigram_bar_met_two_seeds = chosen.iter().all(|s| s.g_adm_pass && s.beats_unigram);
    let w2_code_table_bar_met_two_seeds = chosen
        .iter()
        .all(|s| s.g_adm_pass && s.beats_w2_baseline_code_restr && s.beats_checkpoint_t_restr_aw);
    let w2_bigram_table_bar_met_two_seeds = chosen
        .iter()
        .all(|s| s.g_adm_pass && s.beats_w2_baseline_bigram_restr);
    let m1_full_t2_bar_met_two_seeds = chosen.iter().all(|s| s.g_adm_pass && s.beats_full_panel_t2);

    let unlocked_next_items = match verdict {
        StageTVerdict::PassStage2AdmTable => {
            let mut items = vec![
                "m1-export-gap-and-toks (#2839)".to_owned(),
                "m1-fresh-panel-transfer (#2841)".to_owned(),
            ];
            if m1_full_t2_bar_met_two_seeds {
                items.push("m1-close-saved-model (#2837)".to_owned());
            } else {
                items.push(
                    "m1-address-key-full-vector (#2834 FV8-Env training registration)".to_owned(),
                );
                items.push("m1-wider-code-carrier (#2847)".to_owned());
            }
            items
        }
        StageTVerdict::PartialUnigramOnly
        | StageTVerdict::DiagnosticStopConsecutiveRejections
        | StageTVerdict::HTie => vec![
            "m1-address-key-full-vector (#2834 FV8-Env training registration)".to_owned(),
            "m1-wider-code-carrier (#2847)".to_owned(),
        ],
    };

    Ok(StageTGradeReport {
        schema: STAGE_T_GRADE_SCHEMA.to_owned(),
        item: "m1-bvg-k-stage-t-training".to_owned(),
        issue: 2836,
        sheet_path: sheet_path.to_owned(),
        sheet_sha256: sheet_sha256.to_owned(),
        step0_all_pass: step0.all_pass,
        step0_receipt_sha256: step0_receipt_sha256.to_owned(),
        primary_arm,
        verdict,
        stage2_bar_met_two_seeds,
        unigram_bar_met_two_seeds,
        w2_code_table_bar_met_two_seeds,
        w2_bigram_table_bar_met_two_seeds,
        m1_full_t2_bar_met_two_seeds,
        mechanism_negative: false,
        unlocked_next_items,
        two_seed_noise_halfwidth_rows,
        frozen_zero_seeds,
        relation_seeds,
        fresh_evaluations: 0,
    })
}

/// Verdict for the registered finite relational head (`r-finite-relational-head`, `#2877`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelationHeadVerdict {
    PassM1AndRelational,
    PassRelationalGainSubT2,
    RelationalWithinArmGainTrailsFrozenZeroAdmission,
}

/// Zero-update GPU control baselines (`#2885` `C_route` cp0 and `#2830` `C_successor_only` cp0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationHeadControlBaselines {
    pub c_route_cp0_dev_b3_full_admitted: u64,
    pub c_route_cp0_dev_b3_psucc_admitted: u64,
    pub c_route_cp0_dev_top1: u64,
    pub c_route_cp0_dev_top8: u64,
    pub c_successor_only_cp0_dev_b3_psucc_admitted: u64,
    pub c_frozen_zero_cp0_dev_b3_psucc_admitted: u64,
    pub outcome_invariant_without_c_route_or_c_successor_rerun: bool,
}

/// Seed-1 within-cell read-code reranker decomposition (`#3267`, `stage-t-decomposition.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationHeadWithinCellSummary {
    pub rel_admitted_target_loses_within_own_cell_rows: u64,
    pub rel_t_restr_hits_w_misses: u64,
    pub rel_missed_t_restr_lost_within_own_cell_rows: u64,
    pub rel_reranker_code_top1_cell_only_hits: u64,
    pub rel_reranker_code_cell_prio_top1_hits: u64,
    pub rel_reranker_code_pooled_top1_hits: u64,
    pub rel_reranker_code_permuted_null_top1_hits: u64,
    pub rel_paired_reranker_vs_writer_net_diff_d: i64,
    pub rel_paired_reranker_vs_writer_floor_f: u64,
    pub rel_paired_reranker_vs_permuted_null_net_diff_d: i64,
    pub rel_paired_reranker_vs_permuted_null_floor_f: u64,
    pub fz_admitted_target_loses_within_own_cell_rows: u64,
    pub fz_t_restr_hits_w_misses: u64,
    pub fz_missed_t_restr_lost_within_own_cell_rows: u64,
    pub fz_reranker_code_top1_cell_only_hits: u64,
    pub fz_reranker_code_cell_prio_top1_hits: u64,
    pub fz_reranker_code_pooled_top1_hits: u64,
    pub fz_reranker_code_permuted_null_top1_hits: u64,
}

/// Per-seed paired comparison of `H_relation` (`T16_rel`) and `C_frozen_zero` (`T16_fz`)
/// across training, D40 audits, `relation_off` ablation, count tables, and Stage 3 trunk transfer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationHeadSeedSummary {
    pub seed: u32,
    pub rel_config_sha256: String,
    pub rel_checkpoint_sha256: String,
    pub rel_model_snapshot_sha256: String,
    pub fz_config_sha256: String,
    pub fz_checkpoint_sha256: String,
    pub fz_model_snapshot_sha256: String,
    pub rel_completed_batches: u64,
    pub rel_accepted_updates: u64,
    pub rel_committed_moves: u64,
    pub rel_vector_committed_moves: u64,
    pub rel_combine_committed_moves: u64,
    pub rel_relation_committed_batches: u64,
    pub rel_relation_committed_moves: u64,
    pub rel_joint_commits: u64,
    pub rel_context_tokens_per_sec: u64,
    pub fz_completed_batches: u64,
    pub fz_accepted_updates: u64,
    pub fz_committed_moves: u64,
    pub fz_relation_committed_moves: u64,
    pub fz_joint_commits: u64,
    pub fz_context_tokens_per_sec: u64,
    pub train_anchor_initial_admitted: u64,
    pub train_anchor_required_floor: u64,
    pub rel_train_anchor_peak_admitted: u64,
    pub rel_train_anchor_final_admitted: u64,
    pub fz_train_anchor_peak_admitted: u64,
    pub fz_train_anchor_final_admitted: u64,
    pub gate_d40_pass: bool,
    pub gate_s_pass: bool,
    pub rel_distinct_train_read_codes: usize,
    pub rel_distinct_dev_read_codes: usize,
    pub fz_distinct_train_read_codes: usize,
    pub fz_distinct_dev_read_codes: usize,
    pub rel_dev_b3_full_admitted: u64,
    pub rel_dev_b3_psucc_admitted: u64,
    pub rel_dev_writer_top1_all: u64,
    pub rel_dev_writer_top8_all: u64,
    pub rel_dev_writer_top1_psucc: u64,
    pub rel_dev_writer_top8_psucc: u64,
    pub rel_dev_strict_wins: u64,
    pub rel_off_dev_b3_full_admitted: u64,
    pub rel_off_dev_b3_psucc_admitted: u64,
    pub rel_off_dev_writer_top1_all: u64,
    pub rel_off_dev_writer_top8_all: u64,
    pub rel_off_dev_strict_wins: u64,
    pub within_arm_rel_minus_rel_off_admitted: i64,
    pub within_arm_rel_minus_rel_off_top1: i64,
    pub within_arm_rel_minus_rel_off_top8: i64,
    pub within_arm_rel_minus_rel_off_strict_wins: i64,
    pub within_arm_rel_minus_t1_rel_top1: i64,
    pub fz_dev_b3_full_admitted: u64,
    pub fz_dev_b3_psucc_admitted: u64,
    pub fz_dev_writer_top1_all: u64,
    pub fz_dev_writer_top8_all: u64,
    pub fz_dev_writer_top1_psucc: u64,
    pub fz_dev_writer_top8_psucc: u64,
    pub fz_dev_strict_wins: u64,
    pub rel_minus_fz_dev_b3_full: i64,
    pub rel_minus_fz_dev_b3_psucc: i64,
    pub rel_minus_fz_dev_writer_top1: i64,
    pub rel_minus_fz_dev_writer_top8: i64,
    pub unconditional_unigram_hit1: u64,
    pub unconditional_t2_bigram_hit1: u64,
    pub rel_full_panel_code_table_hit1: u64,
    pub fz_full_panel_code_table_hit1: u64,
    pub rel_checkpoint_u_restr_aw_hit1: u64,
    pub rel_checkpoint_t_sym_restr_aw_hit1: u64,
    pub rel_checkpoint_t_restr_aw_hit1_all: u64,
    pub rel_checkpoint_t_restr_aw_hit8_all: u64,
    pub rel_checkpoint_t_restr_aw_hit1_psucc: u64,
    pub rel_checkpoint_g_restr_aw_hit1_all: u64,
    pub rel_checkpoint_g_restr_aw_hit8_all: u64,
    pub rel_checkpoint_g_restr_aw_hit1_psucc: u64,
    pub fz_checkpoint_u_restr_aw_hit1: u64,
    pub fz_checkpoint_t_sym_restr_aw_hit1: u64,
    pub fz_checkpoint_t_restr_aw_hit1_all: u64,
    pub fz_checkpoint_t_restr_aw_hit8_all: u64,
    pub fz_checkpoint_t_restr_aw_hit1_psucc: u64,
    pub fz_checkpoint_g_restr_aw_hit1_all: u64,
    pub fz_checkpoint_g_restr_aw_hit8_all: u64,
    pub fz_checkpoint_g_restr_aw_hit1_psucc: u64,
    pub rel_minus_fz_t_restr_aw_hit1: i64,
    pub rel_minus_fz_g_restr_aw_hit1: i64,
    pub rel_trunk_dev_level0_top1: u64,
    pub rel_trunk_dev_level1_top1: u64,
    pub rel_trunk_dev_level2_top1: u64,
    pub rel_trunk_dev_level3_top1: u64,
    pub rel_trunk_dev_recurrence_lift_l2_minus_l1: i64,
    pub rel_trunk_fresh_level0_top1: u64,
    pub rel_trunk_fresh_level1_top1: u64,
    pub rel_trunk_fresh_level2_top1: u64,
    pub rel_trunk_fresh_level3_top1: u64,
    pub rel_trunk_fresh_recurrence_lift_l2_minus_l1: i64,
    pub fz_trunk_dev_level0_top1: u64,
    pub fz_trunk_dev_level1_top1: u64,
    pub fz_trunk_dev_level2_top1: u64,
    pub fz_trunk_dev_level3_top1: u64,
    pub fz_trunk_dev_recurrence_lift_l2_minus_l1: i64,
    pub fz_trunk_fresh_level0_top1: u64,
    pub fz_trunk_fresh_level1_top1: u64,
    pub fz_trunk_fresh_level2_top1: u64,
    pub fz_trunk_fresh_level3_top1: u64,
    pub fz_trunk_fresh_recurrence_lift_l2_minus_l1: i64,
    pub rel_minus_fz_trunk_dev_level3_top1: i64,
    pub rel_minus_fz_trunk_fresh_level3_top1: i64,
}

/// Complete two-seed grade report for `r-finite-relational-head` (`#2877`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationHeadGradeReport {
    pub schema: String,
    pub item: String,
    pub issue: u32,
    pub sheet_path: String,
    pub sheet_sha256: String,
    pub decision_card_path: String,
    pub stage_t_archive_sha256: String,
    pub stage_t_grade_sha256: String,
    pub step0_receipt_sha256: String,
    pub trunk_archive_sha256: String,
    pub trunk_grade_sha256: String,
    pub within_cell_decomposition_sha256: String,
    pub gate_c1_to_c5_and_step0_pass: bool,
    pub gate_d40_two_seeds_pass: bool,
    pub gate_s_two_seeds_pass: bool,
    pub within_arm_relation_beats_relation_off_two_seeds: bool,
    pub within_arm_relation_beats_t1_rel_two_seeds: bool,
    pub within_arm_relation_expands_read_codes_two_seeds: bool,
    pub within_arm_relation_recurrence_lift_exceeds_fz_two_seeds: bool,
    pub between_arm_relation_beats_frozen_zero_two_seeds: bool,
    pub m1_closing_bar_met_two_seeds: bool,
    pub outcome_invariance_verified: bool,
    pub mechanism_negative: bool,
    pub verdict: RelationHeadVerdict,
    pub control_baselines: RelationHeadControlBaselines,
    pub within_cell_seed1: RelationHeadWithinCellSummary,
    pub seeds: Vec<RelationHeadSeedSummary>,
}

/// Grade the two-seed finite relational head (`r-finite-relational-head`, `#2877`) against `C_frozen_zero` and the count tables.
#[allow(clippy::too_many_arguments)]
pub fn grade_relation_head(
    sheet_path: &str,
    sheet_sha256: &str,
    decision_card_path: &str,
    stage_t_grade_sha256: &str,
    step0: &Step0Receipt,
    step0_receipt_sha256: &str,
    trunk_grade_sha256: &str,
    within_cell_decomposition_sha256: &str,
    control_baselines: RelationHeadControlBaselines,
    within_cell_seed1: RelationHeadWithinCellSummary,
    seeds: Vec<RelationHeadSeedSummary>,
) -> Result<RelationHeadGradeReport> {
    if !step0.all_pass
        || !step0.akd_pass
        || !step0.v1d_pass
        || !step0.kid_pass
        || !step0.v8d_pass
        || !step0.v9d_pass
    {
        return Err(invalid(
            "Step 0 GPU control failed; CHARTER §4 forbids grading relational head on a failed instrument",
        ));
    }
    if seeds.len() != 2 || seeds[0].seed != 1 || seeds[1].seed != 2 {
        return Err(invalid(
            "Relational head grading requires exactly two seeds (seed 1 and seed 2)",
        ));
    }

    for s in &seeds {
        let d40_ok = s.train_anchor_required_floor == RELATION_HEAD_D40_TRAIN_FLOOR
            && s.rel_train_anchor_final_admitted >= s.train_anchor_required_floor
            && s.fz_train_anchor_final_admitted >= s.train_anchor_required_floor
            && s.rel_dev_b3_psucc_admitted >= DEV_PSUCC_ADMISSION_BAR
            && s.fz_dev_b3_psucc_admitted >= DEV_PSUCC_ADMISSION_BAR;
        if !d40_ok || !s.gate_d40_pass {
            return Err(invalid(
                "Gate D40 breach in relational head grading: TRAIN anchor or DEV P_succ floor violated",
            ));
        }
        let gate_s_ok = s.rel_relation_committed_batches >= RELATION_HEAD_MIN_COMMITS
            && s.rel_relation_committed_moves >= RELATION_HEAD_MIN_COMMITS
            && s.fz_relation_committed_moves == 0;
        if !gate_s_ok || !s.gate_s_pass {
            return Err(invalid(
                "Gate S or V8d breach in relational head grading: relation commits below 16 or FrozenZero non-zero",
            ));
        }

        let expected_rel_minus_off_adm = i64::try_from(s.rel_dev_b3_full_admitted).unwrap_or(0)
            - i64::try_from(s.rel_off_dev_b3_full_admitted).unwrap_or(0);
        let expected_rel_minus_off_top1 = i64::try_from(s.rel_dev_writer_top1_all).unwrap_or(0)
            - i64::try_from(s.rel_off_dev_writer_top1_all).unwrap_or(0);
        let expected_rel_minus_fz_adm = i64::try_from(s.rel_dev_b3_full_admitted).unwrap_or(0)
            - i64::try_from(s.fz_dev_b3_full_admitted).unwrap_or(0);
        let expected_rel_minus_fz_top1 = i64::try_from(s.rel_dev_writer_top1_all).unwrap_or(0)
            - i64::try_from(s.fz_dev_writer_top1_all).unwrap_or(0);
        let expected_rel_minus_fz_trestr = i64::try_from(s.rel_checkpoint_t_restr_aw_hit1_all)
            .unwrap_or(0)
            - i64::try_from(s.fz_checkpoint_t_restr_aw_hit1_all).unwrap_or(0);
        let expected_rel_minus_fz_grestr = i64::try_from(s.rel_checkpoint_g_restr_aw_hit1_all)
            .unwrap_or(0)
            - i64::try_from(s.fz_checkpoint_g_restr_aw_hit1_all).unwrap_or(0);
        if s.within_arm_rel_minus_rel_off_admitted != expected_rel_minus_off_adm
            || s.within_arm_rel_minus_rel_off_top1 != expected_rel_minus_off_top1
            || s.rel_minus_fz_dev_b3_full != expected_rel_minus_fz_adm
            || s.rel_minus_fz_dev_writer_top1 != expected_rel_minus_fz_top1
            || s.rel_minus_fz_t_restr_aw_hit1 != expected_rel_minus_fz_trestr
            || s.rel_minus_fz_g_restr_aw_hit1 != expected_rel_minus_fz_grestr
        {
            return Err(invalid(
                "Relational head seed summary delta mismatch against raw counts",
            ));
        }
    }

    let within_arm_relation_beats_relation_off_two_seeds = seeds.iter().all(|s| {
        s.within_arm_rel_minus_rel_off_admitted > 0
            && s.within_arm_rel_minus_rel_off_top1 > 0
            && s.within_arm_rel_minus_rel_off_top8 > 0
            && s.within_arm_rel_minus_rel_off_strict_wins > 0
    });
    let within_arm_relation_beats_t1_rel_two_seeds =
        seeds.iter().all(|s| s.within_arm_rel_minus_t1_rel_top1 > 0);
    let within_arm_relation_expands_read_codes_two_seeds = seeds.iter().all(|s| {
        s.rel_distinct_train_read_codes > s.fz_distinct_train_read_codes
            && s.rel_distinct_dev_read_codes > s.fz_distinct_dev_read_codes
    });
    let within_arm_relation_recurrence_lift_exceeds_fz_two_seeds = seeds.iter().all(|s| {
        s.rel_trunk_dev_recurrence_lift_l2_minus_l1 > s.fz_trunk_dev_recurrence_lift_l2_minus_l1
            && s.rel_trunk_fresh_recurrence_lift_l2_minus_l1
                > s.fz_trunk_fresh_recurrence_lift_l2_minus_l1
    });

    let between_arm_relation_beats_frozen_zero_two_seeds = seeds.iter().all(|s| {
        s.rel_dev_writer_top1_all > s.fz_dev_writer_top1_all
            || (s.rel_dev_b3_psucc_admitted > s.fz_dev_b3_psucc_admitted
                && s.rel_dev_writer_top1_all >= s.fz_dev_writer_top1_all)
    });
    let m1_closing_bar_met_two_seeds = seeds.iter().all(|s| {
        s.rel_dev_writer_top1_all > UNIGRAM_FULL_BAR
            && s.rel_dev_writer_top1_all > FULL_PANEL_T2_REFERENCE
    });

    // Outcome invariance check: max(C_frozen_zero, C_route) >= C_frozen_zero > H_relation
    let fz_strictly_dominates_rel_all_metrics = seeds.iter().all(|s| {
        s.rel_dev_b3_full_admitted < s.fz_dev_b3_full_admitted
            && s.rel_dev_b3_psucc_admitted < s.fz_dev_b3_psucc_admitted
            && s.rel_dev_writer_top1_all < s.fz_dev_writer_top1_all
            && s.rel_checkpoint_t_restr_aw_hit1_all < s.fz_checkpoint_t_restr_aw_hit1_all
            && s.rel_checkpoint_g_restr_aw_hit1_all < s.fz_checkpoint_g_restr_aw_hit1_all
            && s.rel_trunk_dev_level3_top1 < s.fz_trunk_dev_level3_top1
            && s.rel_trunk_fresh_level3_top1 < s.fz_trunk_fresh_level3_top1
    });
    if control_baselines.outcome_invariant_without_c_route_or_c_successor_rerun
        && !fz_strictly_dominates_rel_all_metrics
    {
        return Err(invalid(
            "outcome invariance claimed when H_relation is not strictly below C_frozen_zero",
        ));
    }

    let verdict =
        if between_arm_relation_beats_frozen_zero_two_seeds && m1_closing_bar_met_two_seeds {
            RelationHeadVerdict::PassM1AndRelational
        } else if between_arm_relation_beats_frozen_zero_two_seeds {
            RelationHeadVerdict::PassRelationalGainSubT2
        } else {
            RelationHeadVerdict::RelationalWithinArmGainTrailsFrozenZeroAdmission
        };

    Ok(RelationHeadGradeReport {
        schema: RELATION_HEAD_GRADE_SCHEMA.to_owned(),
        item: "r-finite-relational-head".to_owned(),
        issue: 2877,
        sheet_path: sheet_path.to_owned(),
        sheet_sha256: sheet_sha256.to_owned(),
        decision_card_path: decision_card_path.to_owned(),
        stage_t_archive_sha256: RELATION_HEAD_STAGE_T_ARCHIVE_SHA256.to_owned(),
        stage_t_grade_sha256: stage_t_grade_sha256.to_owned(),
        step0_receipt_sha256: step0_receipt_sha256.to_owned(),
        trunk_archive_sha256: RELATION_HEAD_TRUNK_ARCHIVE_SHA256.to_owned(),
        trunk_grade_sha256: trunk_grade_sha256.to_owned(),
        within_cell_decomposition_sha256: within_cell_decomposition_sha256.to_owned(),
        gate_c1_to_c5_and_step0_pass: true,
        gate_d40_two_seeds_pass: true,
        gate_s_two_seeds_pass: true,
        within_arm_relation_beats_relation_off_two_seeds,
        within_arm_relation_beats_t1_rel_two_seeds,
        within_arm_relation_expands_read_codes_two_seeds,
        within_arm_relation_recurrence_lift_exceeds_fz_two_seeds,
        between_arm_relation_beats_frozen_zero_two_seeds,
        m1_closing_bar_met_two_seeds,
        outcome_invariance_verified: fz_strictly_dominates_rel_all_metrics,
        mechanism_negative: false,
        verdict,
        control_baselines,
        within_cell_seed1,
        seeds,
    })
}

/// Write a serializable value to `path` as pretty JSON and return its `FilePin`.
pub fn write_pinned_json<T: Serialize>(path: &Path, value: &T) -> Result<FilePin> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| invalid(format!("{}: {e}", parent.display())))?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| invalid(e.to_string()))?;
    fs::write(path, &bytes).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    let canonical =
        fs::canonicalize(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    Ok(FilePin {
        path: canonical,
        sha256: corpus::sha256(&bytes),
    })
}

/// Pin an existing file on disk.
pub fn pin_file(path: &Path) -> Result<FilePin> {
    let canonical =
        fs::canonicalize(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    let bytes =
        fs::read(&canonical).map_err(|e| invalid(format!("{}: {e}", canonical.display())))?;
    Ok(FilePin {
        path: canonical,
        sha256: corpus::sha256(&bytes),
    })
}

/// Build and verify the registered 3,847-row TRAIN anchor manifest (`f85466ff...`) from a `DriverConfig`'s TRAIN corpus.
pub fn build_train_anchor_manifest(config_path: &Path, out_path: &Path) -> Result<FilePin> {
    let bytes =
        fs::read(config_path).map_err(|e| invalid(format!("{}: {e}", config_path.display())))?;
    let config: DriverConfig = serde_json::from_slice(&bytes)
        .map_err(|e| invalid(format!("{}: {e}", config_path.display())))?;
    validate_stage_t_config(&config)?;
    let train_corpus =
        corpus::PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)?;
    let policy = AnchorPolicy {
        start_eligible_index: 1_049_856,
        count: 3_847,
        context: 192,
    };
    let manifest = TrainAnchorManifest::build(&train_corpus, policy)?;
    if manifest.manifest_sha256 != EXPECTED_ANCHOR_MANIFEST_SHA256
        || manifest.row_csv_sha256 != EXPECTED_ANCHOR_CSV_SHA256
    {
        return Err(invalid(format!(
            "built TRAIN anchor manifest ({}) or CSV ({}) differs from registered pins",
            manifest.manifest_sha256, manifest.row_csv_sha256
        )));
    }
    write_pinned_json(out_path, &manifest)
}

#[cfg(feature = "cuda")]
pub mod gpu {
    use super::*;
    use crate::attention_anchor_device::{
        PreparedTrainAnchor, RegisteredAnchorBinding, FIT_SHEET_SHA256, REGISTERED_CSV_SHA256,
    };
    use crate::controller::attention_learning::evidence;
    use crate::device::{snapshot_hash_domains, Device};
    use crate::training::{self, DriverCheckpoint, PreparedDriver};
    use crate::w2_capacity::{self, DevRow, TrainRow, DEV_ROWS, MATCHED_TRAIN_ROWS, VOCABULARY};
    use crate::w2_capture::{self, CELL_SLOTS};
    use serde_json::Value;
    use std::time::Instant;

    fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
        let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        serde_json::from_slice(&bytes).map_err(|e| invalid(format!("{}: {e}", path.display())))
    }

    fn evaluate_train_anchor(anchor: &PreparedTrainAnchor, model: &Device) -> Result<u64> {
        model.synchronize()?;
        let epoch = model.epoch()?;
        let before = snapshot_hash_domains(&model.snapshot()?)?;
        let mut evaluations = Vec::with_capacity(anchor.batches().len());
        for batch in anchor.batches() {
            let carried = model.carry_windows(batch.windows())?;
            let mut evaluation = model.evaluate(&carried, batch.targets())?;
            if evaluation.rows.len() != batch.rows().len() {
                return Err(invalid("anchor GPU row inventory differs"));
            }
            for (row, identity) in evaluation.rows.iter_mut().zip(batch.rows()) {
                row.target_position = Some(identity.target_position);
                row.target_token = Some(identity.target_token);
            }
            evaluations.push(evaluation);
        }
        let aggregate = model.aggregate_evaluations(&evaluations)?;
        model.synchronize()?;
        let after = snapshot_hash_domains(&model.snapshot()?)?;
        if model.epoch()? != epoch
            || after != before
            || aggregate.scored_rows != anchor.registration().policy.count
            || aggregate.target_admitted_rows > aggregate.scored_rows
        {
            return Err(invalid(
                "anchor evaluation changed state or returned wrong inventory",
            ));
        }
        Ok(aggregate.target_admitted_rows)
    }

    /// Build the ordered `(target_position, target_token, is_psucc)` vector for the 14,287 DEV rows.
    pub fn dev_row_identities(config: &DriverConfig) -> Result<Vec<(u64, u32, bool)>> {
        let source =
            corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
        let mut rows = Vec::with_capacity(source.eligible_len());
        let mut first = 0u64;
        while first < source.eligible_len() as u64 {
            let batch = source.batch(first, config.spec.context, config.batch_rows)?;
            for ((window, &position), &target) in batch
                .windows
                .iter()
                .zip(&batch.target_positions)
                .zip(&batch.targets)
            {
                rows.push((
                    position,
                    target,
                    evidence::is_copy_successor(window, target),
                ));
            }
            first = batch.next_target;
        }
        let psucc_count = rows.iter().filter(|&&(_, _, is_psucc)| is_psucc).count();
        if rows.len() != DEV_ROWS || psucc_count != PSUCC_ROWS {
            return Err(invalid(
                "Stage T DEV partition must reproduce 14287 total and 3847 P_succ rows",
            ));
        }
        Ok(rows)
    }

    fn extract_u64(val: &Value, field: &str) -> Result<u64> {
        val.get(field)
            .and_then(Value::as_u64)
            .ok_or_else(|| invalid(format!("missing u64 field {field} in evaluation aggregate")))
    }

    /// Evaluate a `DriverCheckpoint` on DEV, write `<out_dir>/<filename>`, and return `DevMilestoneSummary`.
    pub fn evaluate_and_record_dev(
        driver: &PreparedDriver,
        model: &Device,
        checkpoint: &DriverCheckpoint,
        dev_expected: &[(u64, u32, bool)],
        label: &str,
        out_path: &Path,
    ) -> Result<DevMilestoneSummary> {
        let eval_val = driver.evaluate_checkpoint(model, checkpoint)?;
        let b3 = evidence::summarize(&eval_val, dev_expected)?;
        let agg = eval_val
            .get("aggregate")
            .ok_or_else(|| invalid("DEV evaluation missing aggregate object"))?;
        let rows = extract_u64(agg, "scored_rows")?;
        let target_admitted_rows = extract_u64(agg, "target_admitted_rows")?;
        let top1_correct_rows = extract_u64(agg, "top1_correct")?;
        let top8_correct_rows = extract_u64(agg, "top8_correct")?;
        let admitted_strict_win_rows = extract_u64(agg, "admitted_strict_win_rows")?;
        if b3.full.rows != rows || b3.full.target_admitted != target_admitted_rows {
            return Err(invalid(
                "DEV B3 summary disagrees with aggregate target_admitted_rows",
            ));
        }
        let snap = model.snapshot()?;
        let hashes = snapshot_hash_domains(&snap)?;
        let evaluation_file = write_pinned_json(out_path, &eval_val)?;
        Ok(DevMilestoneSummary {
            label: label.to_owned(),
            completed_batches: checkpoint.cursor.completed_batches,
            model_epoch: checkpoint.cursor.accepted_updates,
            committed_moves: cursor_committed_moves(&checkpoint.cursor),
            model_snapshot_sha256: hashes.model_snapshot_sha256,
            evaluation_file,
            rows,
            target_admitted_rows,
            psucc_rows: b3.psucc.rows,
            psucc_target_admitted_rows: b3.psucc.target_admitted,
            complement_target_admitted_rows: b3.complement.target_admitted,
            top1_correct_rows,
            top8_correct_rows,
            admitted_strict_win_rows,
        })
    }

    /// Execute all five Step 0 GPU controls (`AKd`, `V1d`, `KId`, `V8d`, `V9d`) and throughput probe.
    pub fn run_step0(
        t16_fz_config_path: &Path,
        t16_rel_config_path: &Path,
        t1_fz_config_path: &Path,
        out_path: &Path,
    ) -> Result<Step0Receipt> {
        let t16_fz_pin = pin_file(t16_fz_config_path)?;
        let t16_rel_pin = pin_file(t16_rel_config_path)?;
        let t1_fz_pin = pin_file(t1_fz_config_path)?;

        let t16_fz_cfg: DriverConfig = read_json(t16_fz_config_path)?;
        let t16_rel_cfg: DriverConfig = read_json(t16_rel_config_path)?;
        let t1_fz_cfg: DriverConfig = read_json(t1_fz_config_path)?;

        validate_stage_t_config(&t16_fz_cfg)?;
        validate_stage_t_config(&t16_rel_cfg)?;
        validate_stage_t_config(&t1_fz_cfg)?;
        if t16_fz_cfg.coordinate_block.get() != 16
            || t16_fz_cfg.relation_learning_policy != RelationLearningPolicy::FrozenZeroV1
            || t16_rel_cfg.coordinate_block.get() != 16
            || t16_rel_cfg.relation_learning_policy != RelationLearningPolicy::LearnV1
            || t1_fz_cfg.coordinate_block.get() != 1
            || t1_fz_cfg.relation_learning_policy != RelationLearningPolicy::FrozenZeroV1
        {
            return Err(invalid(
                "step0 requires (t16_fz K=16 FrozenZero, t16_rel K=16 LearnV1, t1_fz K=1 FrozenZero)",
            ));
        }

        // 1. V1d: First 24 batches on production A8 config, transition_block(K=1) vs transition(K=1).
        let driver_t1 = PreparedDriver::load(t1_fz_cfg.clone())?;
        let adm_t1 = training::issue_carrier_arm_admission(&driver_t1, 128)?;
        let mut model_t1_std = driver_t1.new_model(0)?;
        let mut cur_t1_std = driver_t1.initial_cursor();
        let mut model_t1_blk = driver_t1.new_model(0)?;
        let mut cur_t1_blk = driver_t1.initial_cursor();
        let rowsk_t1 = model_t1_blk.rowsk_scoring()?;
        let bvgk_t1 = model_t1_blk.bvgk_kernels()?;

        let mut v1d_mismatches = 0u32;
        let mut akd_checked = 0u32;
        let mut akd_mismatches = 0u32;

        for _ in 0..24u32 {
            let pre_snap = model_t1_blk.snapshot()?;
            let block = driver_t1.block_at(&cur_t1_blk)?;
            let batch = driver_t1.batch(&cur_t1_blk)?;
            let out_std = driver_t1.transition(&mut model_t1_std, &cur_t1_std, &adm_t1)?;
            let out_blk = driver_t1.transition_block(
                &mut model_t1_blk,
                &cur_t1_blk,
                &adm_t1,
                &rowsk_t1,
                &bvgk_t1,
            )?;

            let snap_std = model_t1_std.snapshot()?;
            let snap_blk = model_t1_blk.snapshot()?;
            let h_std = snapshot_hash_domains(&snap_std)?;
            let h_blk = snapshot_hash_domains(&snap_blk)?;
            let blk_moves = out_blk.commit_ordinals.len();
            if out_std.cursor_after != out_blk.cursor_after
                || out_std.applied != (blk_moves == 1)
                || out_std.loss_before != out_blk.l_before
                || out_std.loss_after != out_blk.committed_loss
                || model_t1_std.epoch()? != model_t1_blk.epoch()?
                || h_std != h_blk
            {
                v1d_mismatches += 1;
            }
            if blk_moves == 1 && akd_checked < 32 {
                let mut shadow = Device::from_snapshot(pre_snap, 0)?;
                let rows_cf = shadow.rows_counterfactual()?;
                let carried = shadow.carry_windows(&batch.windows)?;
                let prop = shadow.propose_rows(
                    &carried,
                    &batch.targets,
                    block.coordinates[0],
                    &rows_cf,
                )?;
                let applied = Device::apply(&mut shadow, &prop)?;
                let snap_sh = shadow.snapshot()?;
                let h_sh = snapshot_hash_domains(&snap_sh)?;
                if !applied
                    || snap_blk.table != snap_sh.table
                    || snap_blk.routes != snap_sh.routes
                    || snap_blk.gates != snap_sh.gates
                    || snap_blk.epoch != snap_sh.epoch
                    || h_blk != h_sh
                {
                    akd_mismatches += 1;
                }
                akd_checked += 1;
            }
            cur_t1_std = out_std.cursor_after;
            cur_t1_blk = out_blk.cursor_after;
        }
        let v1d_final_snapshot_sha256 =
            snapshot_hash_domains(&model_t1_blk.snapshot()?)?.model_snapshot_sha256;

        // 2. Run 128 batches of T16_fz uninterrupted: verify KId, V8d, AKd (on single-move commits),
        //    capture batch-64 checkpoint and batch-128 checkpoint, and measure wall time.
        let driver_t16_fz = PreparedDriver::load(t16_fz_cfg.clone())?;
        let adm_t16_fz = training::issue_carrier_arm_admission(&driver_t16_fz, 128)?;
        let mut model_fz = driver_t16_fz.new_model(0)?;
        let mut cur_fz = driver_t16_fz.initial_cursor();
        let rowsk_fz = model_fz.rowsk_scoring()?;
        let bvgk_fz = model_fz.bvgk_kernels()?;
        let (combine_gates, _) = gate_inventory(&t16_fz_cfg)?;

        let mut kid_batches_checked = 0u32;
        let mut kid_commits_rescored = 0u32;
        let mut kid_ineligible_noop_verified = false;
        let mut kid_mismatches = 0u32;
        let mut v8d_batches_checked = 0u32;
        let mut v8d_relation_commits = 0u32;
        let mut v8d_nonzero_relation_trits = 0u32;
        let mut running_moves = 0u64;
        let mut ckpt_64: Option<DriverCheckpoint> = None;

        let t16_start = Instant::now();
        for batch_idx in 0..128u64 {
            let pre_snap = model_fz.snapshot()?;
            let pre_epoch = pre_snap.epoch;
            if pre_snap.gates[combine_gates as usize..]
                .iter()
                .any(|&g| g != 0)
            {
                v8d_nonzero_relation_trits += 1;
            }
            let block = driver_t16_fz.block_at(&cur_fz)?;
            let eligible = block_eligibility(&t16_fz_cfg, &block)?;
            let batch = driver_t16_fz.batch(&cur_fz)?;
            let all_ineligible = eligible.iter().all(|&e| !e);

            let out = driver_t16_fz.transition_block(
                &mut model_fz,
                &cur_fz,
                &adm_t16_fz,
                &rowsk_fz,
                &bvgk_fz,
            )?;
            let moves = out.commit_ordinals.len() as u64;
            running_moves += moves;
            let post_snap = model_fz.snapshot()?;
            let post_epoch = model_fz.epoch()?;

            // KId check
            if post_epoch != out.cursor_after.accepted_updates
                || running_moves != cursor_committed_moves(&out.cursor_after)
            {
                kid_mismatches += 1;
            }
            if all_ineligible {
                if moves != 0
                    || post_epoch != pre_epoch
                    || snapshot_hash_domains(&post_snap)? != snapshot_hash_domains(&pre_snap)?
                {
                    kid_mismatches += 1;
                } else {
                    kid_ineligible_noop_verified = true;
                }
            }
            if moves > 0 {
                if out.committed_loss >= out.l_before {
                    kid_mismatches += 1;
                }
                let post_carried = model_fz.carry_windows(&batch.windows)?;
                let rescored = model_fz.score_rows_k(
                    &post_carried,
                    &batch.targets,
                    &[block.coordinates[0]],
                    &rowsk_fz,
                )?;
                if rescored[0].losses[0] != out.committed_loss {
                    kid_mismatches += 1;
                }
                kid_commits_rescored += 1;
            }
            kid_batches_checked += 1;

            // V8d check
            for &ord in &out.commit_ordinals {
                if let Some(&Parameter::Gate(idx)) = block.coordinates.get(ord as usize) {
                    if idx >= combine_gates {
                        v8d_relation_commits += 1;
                    }
                }
            }
            if post_snap.gates[combine_gates as usize..]
                .iter()
                .any(|&g| g != 0)
            {
                v8d_nonzero_relation_trits += 1;
            }
            v8d_batches_checked += 1;

            // AKd check on natural single-move commits in T16_fz, plus 1-coord block shadow when s_len >= 1
            if moves == 1 && akd_checked < 32 {
                let ord = out.commit_ordinals[0] as usize;
                let param = block.coordinates[ord];
                let mut shadow = Device::from_snapshot(pre_snap, 0)?;
                let rows_cf = shadow.rows_counterfactual()?;
                let carried = shadow.carry_windows(&batch.windows)?;
                let prop = shadow.propose_rows(&carried, &batch.targets, param, &rows_cf)?;
                let applied = Device::apply(&mut shadow, &prop)?;
                let snap_sh = shadow.snapshot()?;
                if !applied
                    || post_snap.table != snap_sh.table
                    || post_snap.routes != snap_sh.routes
                    || post_snap.gates != snap_sh.gates
                    || post_snap.epoch != snap_sh.epoch
                    || snapshot_hash_domains(&post_snap)? != snapshot_hash_domains(&snap_sh)?
                {
                    akd_mismatches += 1;
                }
                akd_checked += 1;
            } else if out.s_len >= 1 && akd_checked < 32 {
                let ord = out.commit_ordinals[0] as usize;
                let param = block.coordinates[ord];
                let mut shadow_bvgk = Device::from_snapshot(pre_snap.clone(), 0)?;
                let sb_rowsk = shadow_bvgk.rowsk_scoring()?;
                let sb_bvgk = shadow_bvgk.bvgk_kernels()?;
                let carried_bvgk = shadow_bvgk.carry_windows(&batch.windows)?;
                let prop_bvgk = shadow_bvgk.propose_block(
                    &carried_bvgk,
                    &batch.targets,
                    &[param],
                    &[true],
                    &sb_rowsk,
                    &sb_bvgk,
                )?;
                let commit_bvgk = shadow_bvgk.apply_block(&prop_bvgk)?;
                let snap_bvgk = shadow_bvgk.snapshot()?;

                let mut shadow_std = Device::from_snapshot(pre_snap, 0)?;
                let rows_std = shadow_std.rows_counterfactual()?;
                let carried_std = shadow_std.carry_windows(&batch.windows)?;
                let prop_std =
                    shadow_std.propose_rows(&carried_std, &batch.targets, param, &rows_std)?;
                let applied_std = Device::apply(&mut shadow_std, &prop_std)?;
                let snap_std = shadow_std.snapshot()?;
                if commit_bvgk.moves != 1
                    || !applied_std
                    || snap_bvgk.table != snap_std.table
                    || snap_bvgk.routes != snap_std.routes
                    || snap_bvgk.gates != snap_std.gates
                    || snap_bvgk.epoch != snap_std.epoch
                    || snapshot_hash_domains(&snap_bvgk)? != snapshot_hash_domains(&snap_std)?
                {
                    akd_mismatches += 1;
                }
                akd_checked += 1;
            }

            cur_fz = out.cursor_after;
            if batch_idx + 1 == 64 {
                ckpt_64 = Some(driver_t16_fz.checkpoint(&model_fz, cur_fz)?);
            }
        }
        let t16_128_batches_elapsed_ms =
            u64::try_from(t16_start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let projected_16404_seconds =
            (t16_128_batches_elapsed_ms.saturating_mul(STAGE_T_MAX_BATCHES)) / (128 * 1_000);

        let ckpt_128_uninterrupted = driver_t16_fz.checkpoint(&model_fz, cur_fz)?;
        let v9d_uninterrupted_128_snapshot_sha256 =
            snapshot_hash_domains(&model_fz.snapshot()?)?.model_snapshot_sha256;

        // 3. V9d: Restore batch-64 checkpoint of T16_fz and run 64 additional batches to 128.
        let ckpt_64 = ckpt_64.ok_or_else(|| invalid("missing batch-64 checkpoint in step0"))?;
        let (mut model_resumed, mut cur_resumed) = driver_t16_fz.restore(ckpt_64, 0)?;
        let rowsk_resumed = model_resumed.rowsk_scoring()?;
        let bvgk_resumed = model_resumed.bvgk_kernels()?;
        for _ in 64..128u64 {
            let out = driver_t16_fz.transition_block(
                &mut model_resumed,
                &cur_resumed,
                &adm_t16_fz,
                &rowsk_resumed,
                &bvgk_resumed,
            )?;
            cur_resumed = out.cursor_after;
        }
        let ckpt_128_resumed = driver_t16_fz.checkpoint(&model_resumed, cur_resumed)?;
        let v9d_resumed_128_snapshot_sha256 =
            snapshot_hash_domains(&model_resumed.snapshot()?)?.model_snapshot_sha256;
        let v9d_pass = ckpt_128_uninterrupted.cursor == ckpt_128_resumed.cursor
            && ckpt_128_uninterrupted.model_sha256 == ckpt_128_resumed.model_sha256
            && v9d_uninterrupted_128_snapshot_sha256 == v9d_resumed_128_snapshot_sha256;

        // 4. If akd_checked < 32, continue on T16_rel until 32 single-move commits are checked.
        if akd_checked < 32 {
            let driver_rel = PreparedDriver::load(t16_rel_cfg)?;
            let adm_rel = training::issue_carrier_arm_admission(&driver_rel, 256)?;
            let mut model_rel = driver_rel.new_model(0)?;
            let mut cur_rel = driver_rel.initial_cursor();
            let rowsk_rel = model_rel.rowsk_scoring()?;
            let bvgk_rel = model_rel.bvgk_kernels()?;
            while akd_checked < 32 && cur_rel.completed_batches < 256 {
                let pre_snap = model_rel.snapshot()?;
                let block = driver_rel.block_at(&cur_rel)?;
                let batch = driver_rel.batch(&cur_rel)?;
                let out = driver_rel.transition_block(
                    &mut model_rel,
                    &cur_rel,
                    &adm_rel,
                    &rowsk_rel,
                    &bvgk_rel,
                )?;
                if out.s_len >= 1 {
                    let ord = out.commit_ordinals[0] as usize;
                    let param = block.coordinates[ord];
                    let mut shadow_bvgk = Device::from_snapshot(pre_snap.clone(), 0)?;
                    let sb_rowsk = shadow_bvgk.rowsk_scoring()?;
                    let sb_bvgk = shadow_bvgk.bvgk_kernels()?;
                    let carried_bvgk = shadow_bvgk.carry_windows(&batch.windows)?;
                    let prop_bvgk = shadow_bvgk.propose_block(
                        &carried_bvgk,
                        &batch.targets,
                        &[param],
                        &[true],
                        &sb_rowsk,
                        &sb_bvgk,
                    )?;
                    let commit_bvgk = shadow_bvgk.apply_block(&prop_bvgk)?;
                    let snap_bvgk = shadow_bvgk.snapshot()?;

                    let mut shadow_std = Device::from_snapshot(pre_snap, 0)?;
                    let rows_std = shadow_std.rows_counterfactual()?;
                    let carried_std = shadow_std.carry_windows(&batch.windows)?;
                    let prop_std =
                        shadow_std.propose_rows(&carried_std, &batch.targets, param, &rows_std)?;
                    let applied_std = Device::apply(&mut shadow_std, &prop_std)?;
                    let snap_std = shadow_std.snapshot()?;
                    if commit_bvgk.moves != 1
                        || !applied_std
                        || snap_bvgk.table != snap_std.table
                        || snap_bvgk.routes != snap_std.routes
                        || snap_bvgk.gates != snap_std.gates
                        || snap_bvgk.epoch != snap_std.epoch
                        || snapshot_hash_domains(&snap_bvgk)? != snapshot_hash_domains(&snap_std)?
                    {
                        akd_mismatches += 1;
                    }
                    akd_checked += 1;
                }
                cur_rel = out.cursor_after;
            }
        }

        let akd_pass = akd_checked >= 32 && akd_mismatches == 0;
        let v1d_pass = v1d_mismatches == 0;
        let kid_pass =
            kid_batches_checked == 128 && kid_ineligible_noop_verified && kid_mismatches == 0;
        let v8d_pass = v8d_batches_checked == 128
            && v8d_relation_commits == 0
            && v8d_nonzero_relation_trits == 0;
        let all_pass = akd_pass && v1d_pass && kid_pass && v8d_pass && v9d_pass;

        let compiled = crate::device::compiled_identity();
        let receipt = Step0Receipt {
            schema: STAGE_T_STEP0_SCHEMA.to_owned(),
            source_sha256: compiled["source_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            kernel_sha256: compiled["kernel_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            geometry_sha256: compiled["geometry_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            driver_source_sha256: training::driver_source_sha256(),
            t16_fz_config_sha256: t16_fz_pin.sha256,
            t16_rel_config_sha256: t16_rel_pin.sha256,
            t1_fz_config_sha256: t1_fz_pin.sha256,
            akd_single_move_commits_checked: akd_checked,
            akd_mismatches,
            akd_pass,
            v1d_batches_checked: 24,
            v1d_mismatches,
            v1d_final_snapshot_sha256,
            v1d_pass,
            kid_batches_checked,
            kid_commits_rescored,
            kid_ineligible_noop_verified,
            kid_mismatches,
            kid_pass,
            v8d_batches_checked,
            v8d_relation_commits_on_frozen_zero: v8d_relation_commits,
            v8d_nonzero_relation_trits,
            v8d_pass,
            v9d_uninterrupted_128_snapshot_sha256,
            v9d_resumed_128_snapshot_sha256,
            v9d_epoch_128: cur_fz.accepted_updates,
            v9d_committed_moves_128: cursor_committed_moves(&cur_fz),
            v9d_pass,
            t16_128_batches_elapsed_ms,
            projected_16404_seconds,
            all_pass,
        };
        write_pinned_json(out_path, &receipt)?;
        if !all_pass {
            return Err(invalid(
                "Step 0 GPU control failed; inspect step0 receipt JSON",
            ));
        }
        Ok(receipt)
    }

    /// Capture the 16,404-batch TRAIN and 224-batch DEV read codes and admitted cells for a checkpoint on GPU
    /// and compute its checkpoint-restricted W2 count tables (`T_restr(A_W)`, `G_restr(A_W)`, `U_restr(A_W)`, `T_sym_restr(A_W)`).
    pub fn run_w2_eval_on_device(
        driver: &PreparedDriver,
        model: &Device,
        checkpoint: &DriverCheckpoint,
        config_sha256: &str,
        checkpoint_file: FilePin,
        out_path: &Path,
    ) -> Result<StageTW2Summary> {
        let start = Instant::now();
        driver.validate_checkpoint(checkpoint)?;
        let config = driver.config();
        let snap = model.snapshot()?;
        let hashes = snapshot_hash_domains(&snap)?;
        let token_cells = model.export_token_cells()?;
        if token_cells.len() != VOCABULARY {
            return Err(invalid("token_cells length differs from VOCABULARY"));
        }

        // 1. DEV capture (14,287 rows)
        let dev_corpus =
            corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
        let mut dev_rows = Vec::with_capacity(DEV_ROWS);
        let mut dev_slots: Vec<Vec<u16>> = Vec::with_capacity(DEV_ROWS);
        let mut dev_is_psucc: Vec<bool> = Vec::with_capacity(DEV_ROWS);
        let mut first = 0u64;
        let mut psucc_seen = 0usize;
        let mut dev_admitted_count = 0u32;
        let mut dev_psucc_admitted_count = 0u32;

        while first < dev_corpus.eligible_len() as u64 {
            let batch = dev_corpus.batch(first, config.spec.context, config.batch_rows)?;
            let carried = model.carry_windows(&batch.windows)?;
            let forward = model.forward(&carried, &batch.targets)?;
            let vectors = forward.vectors()?;
            let evaluated = model.evaluate(&carried, &batch.targets)?;
            let n = batch.targets.len();
            if vectors.len() != n * 32
                || forward.admitted_cells.len() != n * CELL_SLOTS
                || evaluated.rows.len() != n
            {
                return Err(invalid("DEV W2 forward/evaluate row shape mismatch"));
            }
            for r in 0..n {
                let window = &batch.windows[r];
                let query = window
                    .last()
                    .ok_or_else(|| invalid("empty DEV window"))?
                    .token;
                let target = batch.targets[r];
                let is_psucc = evidence::is_copy_successor(window, target);
                if is_psucc {
                    psucc_seen += 1;
                }
                let eval_row = &evaluated.rows[r];
                let target_admitted = match eval_row
                    .admitted_cells
                    .checked_sub(eval_row.address_rivals)
                {
                    Some(0) => false,
                    Some(1) => true,
                    _ => return Err(invalid("DEV admitted_cells - address_rivals not in 0..=1")),
                };
                if target_admitted {
                    dev_admitted_count += 1;
                    if is_psucc {
                        dev_psucc_admitted_count += 1;
                    }
                }
                let q: &[i32; 32] = vectors[r * 32..(r + 1) * 32]
                    .try_into()
                    .map_err(|_| invalid("DEV read slice length != 32"))?;
                let code = w2_capacity::decode_read_vector(q)
                    .map_err(|e| invalid(format!("DEV decode_read_vector: {e}")))?;
                let slot_slice = &forward.admitted_cells[r * CELL_SLOTS..(r + 1) * CELL_SLOTS];
                let (slots_arr, _) = w2_capture::cell_slots(slot_slice).map_err(invalid)?;
                dev_rows.push(DevRow {
                    code,
                    query_token: u16::try_from(query).map_err(|_| invalid("query > u16"))?,
                    target_token: u16::try_from(target).map_err(|_| invalid("target > u16"))?,
                    writer_hit1: eval_row.top1_correct == 1,
                    writer_hit8: eval_row.top8_correct == 1,
                });
                dev_slots.push(slots_arr.to_vec());
                dev_is_psucc.push(is_psucc);
            }
            first = batch.next_target;
        }
        if dev_rows.len() != DEV_ROWS || psucc_seen != PSUCC_ROWS {
            return Err(invalid("DEV W2 capture row count mismatch"));
        }

        // 2. TRAIN capture (16,404 batches = 1,049,856 rows)
        let train_corpus =
            corpus::PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)?;
        let mut train_rows = Vec::with_capacity(MATCHED_TRAIN_ROWS);
        let mut train_first = 0u64;
        for _ in 0..STAGE_T_MAX_BATCHES {
            let batch = train_corpus.batch(train_first, config.spec.context, config.batch_rows)?;
            let carried = model.carry_windows(&batch.windows)?;
            let forward = model.forward(&carried, &batch.targets)?;
            let vectors = forward.vectors()?;
            let n = batch.targets.len();
            if vectors.len() != n * 32 {
                return Err(invalid("TRAIN W2 forward vector shape mismatch"));
            }
            for r in 0..n {
                let window = &batch.windows[r];
                let query = window
                    .last()
                    .ok_or_else(|| invalid("empty TRAIN window"))?
                    .token;
                let target = batch.targets[r];
                let q: &[i32; 32] = vectors[r * 32..(r + 1) * 32]
                    .try_into()
                    .map_err(|_| invalid("TRAIN read slice length != 32"))?;
                let code = w2_capacity::decode_read_vector(q)
                    .map_err(|e| invalid(format!("TRAIN decode_read_vector: {e}")))?;
                train_rows.push(TrainRow {
                    code,
                    query_token: u16::try_from(query).map_err(|_| invalid("query > u16"))?,
                    target_token: u16::try_from(target).map_err(|_| invalid("target > u16"))?,
                });
            }
            train_first = batch.next_target;
        }
        if train_rows.len() != MATCHED_TRAIN_ROWS {
            return Err(invalid("TRAIN W2 capture row count mismatch"));
        }

        let actions = w2_capacity::symmetry_root_actions().map_err(invalid)?;
        let sym_keys = w2_capacity::symmetry_keys(&actions);
        let tables = w2_capacity::build_tables(&train_rows).map_err(|e| invalid(e.to_string()))?;
        let t_sym_table = w2_capacity::build_sym_table(&train_rows, &sym_keys);

        let full_scores =
            w2_capacity::score_dev(&tables, &dev_rows).map_err(|e| invalid(e.to_string()))?;
        let full_agg = w2_capacity::aggregate(&tables, &full_scores);

        let admitted_scores = w2_capacity::score_admitted(
            &tables,
            &t_sym_table,
            &sym_keys,
            &dev_rows,
            &token_cells,
            &dev_slots,
        );
        let am_all = w2_capacity::admission_matched(&admitted_scores, &dev_rows);

        let mut psucc_admitted_scores = Vec::with_capacity(PSUCC_ROWS);
        let mut psucc_dev_rows = Vec::with_capacity(PSUCC_ROWS);
        for ((score, row), &is_ps) in admitted_scores.iter().zip(&dev_rows).zip(&dev_is_psucc) {
            if is_ps {
                psucc_admitted_scores.push(score.clone());
                psucc_dev_rows.push(*row);
            }
        }
        let am_psucc = w2_capacity::admission_matched(&psucc_admitted_scores, &psucc_dev_rows);

        let mut train_codes: Vec<u16> = train_rows.iter().map(|r| r.code).collect();
        train_codes.sort_unstable();
        train_codes.dedup();
        let mut dev_codes: Vec<u16> = dev_rows.iter().map(|r| r.code).collect();
        dev_codes.sort_unstable();
        dev_codes.dedup();

        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let summary = StageTW2Summary {
            schema: STAGE_T_W2_SCHEMA.to_owned(),
            config_sha256: config_sha256.to_owned(),
            checkpoint_file,
            model_snapshot_sha256: hashes.model_snapshot_sha256,
            model_epoch: checkpoint.cursor.accepted_updates,
            completed_batches: checkpoint.cursor.completed_batches,
            committed_moves: cursor_committed_moves(&checkpoint.cursor),
            train_rows_captured: train_rows.len(),
            dev_rows_captured: dev_rows.len(),
            distinct_train_read_codes: train_codes.len(),
            distinct_dev_read_codes: dev_codes.len(),
            dev_target_admitted_rows: dev_admitted_count,
            dev_psucc_admitted_rows: dev_psucc_admitted_count,
            writer_top1_correct_all: am_all.all[4].hit1 as u32,
            writer_top8_correct_all: am_all.all[4].hit8 as u32,
            writer_top1_correct_psucc: am_psucc.all[4].hit1 as u32,
            writer_top8_correct_psucc: am_psucc.all[4].hit8 as u32,
            full_panel_unigram_hit1: full_agg.u_top1 as u32,
            full_panel_bigram_hit1: full_agg.g_top1 as u32,
            full_panel_code_table_hit1: full_agg.t_top1 as u32,
            w2_baseline_code_restr_q4_hit1: W2_BASELINE_CODE_RESTR_BAR as u32,
            w2_baseline_bigram_restr_q4_hit1: W2_BASELINE_BIGRAM_RESTR_BAR as u32,
            checkpoint_t_restr_aw_hit1_all: am_all.all[0].hit1 as u32,
            checkpoint_t_restr_aw_hit8_all: am_all.all[0].hit8 as u32,
            checkpoint_u_restr_aw_hit1_all: am_all.all[1].hit1 as u32,
            checkpoint_g_restr_aw_hit1_all: am_all.all[2].hit1 as u32,
            checkpoint_g_restr_aw_hit8_all: am_all.all[2].hit8 as u32,
            checkpoint_t_sym_restr_aw_hit1_all: am_all.all[3].hit1 as u32,
            checkpoint_t_restr_aw_hit1_psucc: am_psucc.all[0].hit1 as u32,
            checkpoint_g_restr_aw_hit1_psucc: am_psucc.all[2].hit1 as u32,
            elapsed_ms,
        };
        write_pinned_json(out_path, &summary)?;
        Ok(summary)
    }

    /// Standalone `w2-eval` entrypoint from a saved config and checkpoint file.
    pub fn run_w2_eval(
        config_path: &Path,
        checkpoint_path: &Path,
        out_path: &Path,
    ) -> Result<StageTW2Summary> {
        let cfg_pin = pin_file(config_path)?;
        let ckpt_pin = pin_file(checkpoint_path)?;
        let config: DriverConfig = read_json(config_path)?;
        validate_stage_t_config(&config)?;
        let ckpt: DriverCheckpoint = read_json(checkpoint_path)?;
        let driver = PreparedDriver::load(config)?;
        let (model, _) = driver.restore(ckpt.clone(), 0)?;
        run_w2_eval_on_device(&driver, &model, &ckpt, &cfg_pin.sha256, ckpt_pin, out_path)
    }

    /// Execute a D40-audited Stage T training run (`K = 16` or `K = 1`, `seed = 1` or `seed = 2`).
    pub fn run_stage_t_train(
        config_path: &Path,
        anchor_manifest_path: &Path,
        sheet_path: &Path,
        out_dir: &Path,
        max_batches: u64,
        run_w2_at_end: bool,
    ) -> Result<StageTRunReport> {
        let run_start = Instant::now();
        if max_batches == 0 || max_batches > STAGE_T_MAX_BATCHES {
            return Err(invalid("max_batches must be in 1..=16404"));
        }
        fs::create_dir_all(out_dir).map_err(|e| invalid(format!("{}: {e}", out_dir.display())))?;

        let config_file = pin_file(config_path)?;
        let anchor_manifest_file = pin_file(anchor_manifest_path)?;
        let sheet_file = pin_file(sheet_path)?;

        let config: DriverConfig = read_json(config_path)?;
        validate_stage_t_config(&config)?;
        let manifest: TrainAnchorManifest = read_json(anchor_manifest_path)?;
        if manifest.row_csv_sha256 != REGISTERED_CSV_SHA256
            || manifest.manifest_sha256 != EXPECTED_ANCHOR_MANIFEST_SHA256
        {
            return Err(invalid(
                "Stage T TRAIN anchor manifest SHA-256 or CSV SHA-256 mismatch",
            ));
        }

        let driver = PreparedDriver::load(config.clone())?;
        let admission = training::issue_carrier_arm_admission(&driver, max_batches)?;
        let expected_anchor = RegisteredAnchorBinding {
            corpus: driver.train().binding().clone(),
            policy: AnchorPolicy {
                start_eligible_index: 1_049_856,
                count: 3_847,
                context: 192,
            },
            manifest_sha256: EXPECTED_ANCHOR_MANIFEST_SHA256.to_owned(),
            csv_sha256: REGISTERED_CSV_SHA256.to_owned(),
            sheet_sha256: FIT_SHEET_SHA256.to_owned(),
            plan_sha256: sheet_file.sha256.clone(),
        };
        let prepared_anchor =
            PreparedTrainAnchor::prepare(driver.train(), &manifest, &expected_anchor)?;

        let dev_expected = dev_row_identities(&config)?;
        let policy = AuditPolicy::registered();
        let (combine_gates, _) = gate_inventory(&config)?;

        let mut model = driver.new_model(0)?;
        let mut cursor = driver.initial_cursor();
        let mut rowsk = model.rowsk_scoring()?;
        let mut bvgk = model.bvgk_kernels()?;

        // Baseline TRAIN anchor evaluation (batch 0)
        let initial_train_anchor_admitted = evaluate_train_anchor(&prepared_anchor, &model)?;
        if initial_train_anchor_admitted < policy.minimum_baseline {
            return Err(invalid(
                "initial TRAIN anchor admitted < minimum_baseline 500",
            ));
        }
        let required_floor = initial_train_anchor_admitted.saturating_sub(policy.margin);
        let mut peak_train_anchor_admitted = initial_train_anchor_admitted;
        let mut final_train_anchor_admitted = initial_train_anchor_admitted;

        let mut passing_checkpoint = driver.checkpoint(&model, cursor)?;
        write_pinned_json(&out_dir.join("checkpoint-00000.json"), &passing_checkpoint)?;

        let baseline_dev = evaluate_and_record_dev(
            &driver,
            &model,
            &passing_checkpoint,
            &dev_expected,
            "baseline",
            &out_dir.join("baseline-dev.json"),
        )?;
        if baseline_dev.target_admitted_rows != 4_244
            || baseline_dev.psucc_target_admitted_rows != 1_351
            || baseline_dev.top1_correct_rows != 133
        {
            return Err(invalid(
                "Stage T initial DEV evaluation did not reproduce A8 baseline 4244/1351/133",
            ));
        }

        let mut counters = StageTCounters::default();
        let mut interval_audits = Vec::new();
        let mut consecutive_full_rejections = 0u32;
        let mut total_audits = 1u32; // baseline audit = 1
        let mut rejected_audits = 0u32;
        let mut q1_dev: Option<DevMilestoneSummary> = None;
        let mut terminal_status = if max_batches == STAGE_T_MAX_BATCHES {
            StageTTerminalStatus::CompletedFullDose
        } else {
            StageTTerminalStatus::CompletedProbe
        };
        let mut stop_reasons = Vec::new();

        while cursor.completed_batches < max_batches {
            let interval_start_batch = cursor.completed_batches;
            let interval_end_batch =
                (interval_start_batch + STAGE_T_AUDIT_CADENCE).min(max_batches);
            let interval_start_epoch = cursor.accepted_updates;
            let mut interval_delta = IntervalDelta::default();

            while cursor.completed_batches < interval_end_batch {
                if config.coordinate_block.get() == 16 {
                    let block = driver.block_at(&cursor)?;
                    let batch = driver.batch(&cursor)?;
                    let out =
                        driver.transition_block(&mut model, &cursor, &admission, &rowsk, &bvgk)?;
                    counters.record_block_step(&mut interval_delta, &config, &block, &out)?;
                    // Per-batch KId check: verify epoch and post-commit loss on every commit
                    if model.epoch()? != out.cursor_after.accepted_updates {
                        return Err(invalid("KId violation: model.epoch() != accepted_updates"));
                    }
                    if !out.commit_ordinals.is_empty() {
                        let post_carried = model.carry_windows(&batch.windows)?;
                        let rescored = model.score_rows_k(
                            &post_carried,
                            &batch.targets,
                            &[block.coordinates[0]],
                            &rowsk,
                        )?;
                        if rescored[0].losses[0] != out.committed_loss {
                            return Err(invalid(
                                "KId violation: post-commit rescored loss != committed_loss",
                            ));
                        }
                    }
                    cursor = out.cursor_after;
                } else {
                    let out = driver.transition(&mut model, &cursor, &admission)?;
                    counters.record_single_step(&mut interval_delta, &config, &out)?;
                    cursor = out.cursor_after;
                }
            }

            // 64-batch (or final tail) D40 TRAIN anchor audit
            let inspected_epoch = model.epoch()?;
            if config.relation_learning_policy == RelationLearningPolicy::FrozenZeroV1 {
                let snap = model.snapshot()?;
                if snap.gates[combine_gates as usize..].iter().any(|&g| g != 0) {
                    return Err(invalid(
                        "V8d violation: non-zero relation trit detected at interval audit",
                    ));
                }
            }

            let admitted = evaluate_train_anchor(&prepared_anchor, &model)?;
            total_audits += 1;
            let rejected = admitted < required_floor;

            if !rejected {
                counters.settle_interval(&interval_delta, false);
                consecutive_full_rejections = 0;
                if admitted > peak_train_anchor_admitted {
                    peak_train_anchor_admitted = admitted;
                }
                final_train_anchor_admitted = admitted;
                passing_checkpoint = driver.checkpoint(&model, cursor)?;
            } else {
                counters.settle_interval(&interval_delta, true);
                rejected_audits += 1;
                consecutive_full_rejections += 1;
                // Restore last passing checkpoint and advance cursor across rejected interval without replay
                let (restored_model, mut advanced) =
                    driver.restore(passing_checkpoint.clone(), 0)?;
                model = restored_model;
                rowsk = model.rowsk_scoring()?;
                bvgk = model.bvgk_kernels()?;
                while advanced.completed_batches < interval_end_batch {
                    advanced = if config.coordinate_block.is_one() {
                        driver.advance(&advanced, false)?
                    } else {
                        driver.advance_block(&advanced, false, 0)?
                    };
                }
                cursor = advanced;
                passing_checkpoint = driver.checkpoint(&model, cursor)?;
            }

            let post_audit_epoch = model.epoch()?;
            let post_snap_sha = snapshot_hash_domains(&model.snapshot()?)?.model_snapshot_sha256;
            interval_audits.push(IntervalAuditRecord {
                audit_index: total_audits - 1,
                attempted_batches: interval_end_batch,
                interval_start_epoch,
                inspected_epoch,
                post_audit_epoch,
                interval_provisional_updates: interval_delta.accepted_updates,
                interval_provisional_moves: interval_delta.committed_moves,
                interval_joint_commits: interval_delta.joint_commits,
                train_anchor_admitted: admitted,
                required_floor,
                rejected,
                consecutive_full_rejections,
                post_audit_snapshot_sha256: post_snap_sha,
            });

            // Q1 evaluation at batch 4,096 (evaluated on committed state after batch-4096 audit)
            if interval_end_batch == STAGE_T_Q1_BATCHES {
                write_pinned_json(&out_dir.join("checkpoint-04096.json"), &passing_checkpoint)?;
                let q1_summary = evaluate_and_record_dev(
                    &driver,
                    &model,
                    &passing_checkpoint,
                    &dev_expected,
                    "q1",
                    &out_dir.join("q1-dev.json"),
                )?;
                let q1_breach = q1_summary.target_admitted_rows < DEV_FULL_ADMISSION_BAR
                    || q1_summary.psucc_target_admitted_rows < DEV_PSUCC_ADMISSION_BAR
                    || (counters.combine.sensitive_batches == 0
                        && counters.relation.sensitive_batches == 0);
                q1_dev = Some(q1_summary);
                if q1_breach {
                    terminal_status = StageTTerminalStatus::DiagnosticStopQ1;
                    stop_reasons.push("q1_admission_or_sensitivity_stop".to_owned());
                    break;
                }
            }

            if u64::from(consecutive_full_rejections) >= policy.consecutive_full_rejections {
                terminal_status = StageTTerminalStatus::DiagnosticStopConsecutiveRejections;
                stop_reasons.push("consecutive_rejections".to_owned());
                break;
            }
        }

        let final_checkpoint_file =
            write_pinned_json(&out_dir.join("final-checkpoint.json"), &passing_checkpoint)?;
        let final_dev = evaluate_and_record_dev(
            &driver,
            &model,
            &passing_checkpoint,
            &dev_expected,
            "final",
            &out_dir.join("final-dev.json"),
        )?;

        let relation_off_dev = if config.relation_learning_policy == RelationLearningPolicy::LearnV1
        {
            let mut rel_off_snap = model.snapshot()?;
            for g in &mut rel_off_snap.gates[combine_gates as usize..] {
                *g = 0;
            }
            let rel_off_model = Device::from_snapshot(rel_off_snap, 0)?;
            let rel_off_ckpt = driver.checkpoint(&rel_off_model, passing_checkpoint.cursor)?;
            Some(evaluate_and_record_dev(
                &driver,
                &rel_off_model,
                &rel_off_ckpt,
                &dev_expected,
                "relation_off",
                &out_dir.join("relation-off-dev.json"),
            )?)
        } else {
            None
        };

        let (w2_summary_file, w2_summary) = if run_w2_at_end {
            let w2_path = out_dir.join("w2-summary.json");
            let w2 = run_w2_eval_on_device(
                &driver,
                &model,
                &passing_checkpoint,
                &config_file.sha256,
                final_checkpoint_file.clone(),
                &w2_path,
            )?;
            let w2_pin = pin_file(&w2_path)?;
            (Some(w2_pin), Some(w2))
        } else {
            (None, None)
        };

        let arm_prefix = if config.coordinate_block.get() == 16 {
            "T16"
        } else {
            "T1"
        };
        let pol_suffix = match config.relation_learning_policy {
            RelationLearningPolicy::FrozenZeroV1 => "fz",
            RelationLearningPolicy::LearnV1 => "rel",
        };
        let arm_label = format!("{arm_prefix}_{pol_suffix}_seed{}", config.seed);
        let compiled = crate::device::compiled_identity();
        let elapsed_ms = u64::try_from(run_start.elapsed().as_millis()).unwrap_or(u64::MAX);

        let report = StageTRunReport {
            schema: STAGE_T_RUN_SCHEMA.to_owned(),
            arm_label,
            seed: config.seed,
            coordinate_block: config.coordinate_block.get(),
            relation_learning_policy: config.relation_learning_policy,
            is_full_dose: max_batches == STAGE_T_MAX_BATCHES,
            max_batches,
            terminal_status,
            stop_reasons,
            source_sha256: compiled["source_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            kernel_sha256: compiled["kernel_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            geometry_sha256: compiled["geometry_sha256"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            driver_source_sha256: training::driver_source_sha256(),
            config_file,
            anchor_manifest_file,
            sheet_file,
            audit_policy: policy,
            initial_train_anchor_admitted,
            train_anchor_required_floor: required_floor,
            peak_train_anchor_admitted,
            final_train_anchor_admitted,
            total_audits,
            rejected_audits,
            counters,
            baseline_dev,
            q1_dev,
            final_dev,
            relation_off_dev,
            final_checkpoint_file,
            w2_summary_file,
            w2_summary,
            interval_audits,
            fresh_slice_status: "SEALED_PENDING_PAIRED_DEV_DECISION".to_owned(),
            fresh_evaluations: 0,
            elapsed_ms,
        };
        write_pinned_json(&out_dir.join("report.json"), &report)?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::training::{self, Coordinate};
    use std::path::PathBuf;

    fn load_committed_config(name: &str) -> DriverConfig {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let path = root
            .join("docs/labs/m1-bvg-k-stage-t-training/configs")
            .join(name);
        let bytes = fs::read(&path).expect("read committed Stage T config");
        serde_json::from_slice(&bytes).expect("parse committed Stage T config")
    }

    #[test]
    fn integer_ceil_two_sqrt_is_exact() {
        assert_eq!(integer_ceil_two_sqrt(0), 0);
        assert_eq!(integer_ceil_two_sqrt(1), 2);
        assert_eq!(integer_ceil_two_sqrt(2), 3);
        assert_eq!(integer_ceil_two_sqrt(4), 4);
        assert_eq!(integer_ceil_two_sqrt(5), 5);
        assert_eq!(integer_ceil_two_sqrt(25), 10);
        assert_eq!(integer_ceil_two_sqrt(20_449), 286);
        assert_eq!(integer_ceil_two_sqrt(20_450), 287);
    }

    #[test]
    fn committed_stage_t_configs_validate_across_seeds_and_blocks() -> Result<()> {
        for (name, expected_k, expected_seed, expected_pol) in [
            (
                "t16-frozen-zero-seed1.json",
                16,
                1,
                RelationLearningPolicy::FrozenZeroV1,
            ),
            (
                "t16-relation-seed1.json",
                16,
                1,
                RelationLearningPolicy::LearnV1,
            ),
            (
                "t16-frozen-zero-seed2.json",
                16,
                2,
                RelationLearningPolicy::FrozenZeroV1,
            ),
            (
                "t16-relation-seed2.json",
                16,
                2,
                RelationLearningPolicy::LearnV1,
            ),
            (
                "t1-frozen-zero-seed1.json",
                1,
                1,
                RelationLearningPolicy::FrozenZeroV1,
            ),
            (
                "t1-relation-seed1.json",
                1,
                1,
                RelationLearningPolicy::LearnV1,
            ),
            (
                "t1-frozen-zero-seed2.json",
                1,
                2,
                RelationLearningPolicy::FrozenZeroV1,
            ),
            (
                "t1-relation-seed2.json",
                1,
                2,
                RelationLearningPolicy::LearnV1,
            ),
        ] {
            let cfg = load_committed_config(name);
            validate_stage_t_config(&cfg)?;
            assert_eq!(cfg.coordinate_block.get(), expected_k);
            assert_eq!(cfg.seed, expected_seed);
            assert_eq!(cfg.relation_learning_policy, expected_pol);
        }

        // Refusal checks on invalid configs
        let mut bad_seed = load_committed_config("t16-frozen-zero-seed1.json");
        bad_seed.seed = 3;
        assert!(validate_stage_t_config(&bad_seed).is_err());

        let mut bad_rows = load_committed_config("t16-frozen-zero-seed1.json");
        bad_rows.batch_rows = 32;
        assert!(validate_stage_t_config(&bad_rows).is_err());
        Ok(())
    }

    #[test]
    fn straddle_aware_block_evidence_handles_all_three_read_windows_and_rollback() -> Result<()> {
        let cfg_rel = load_committed_config("t16-relation-seed1.json");
        let cfg_fz = load_committed_config("t16-frozen-zero-seed1.json");
        assert_eq!(gate_inventory(&cfg_rel)?, (20, 45));

        // Window 1: Gate(0..16) -> CombineOnly (16 Combine coordinates)
        let block_w1 = Block {
            kind: CoordinateKind::Gate,
            coordinates: (0..16).map(Parameter::Gate).collect(),
        };
        assert_eq!(
            classify_block(&cfg_rel, &block_w1)?,
            BlockComposition::CombineOnly
        );

        // Window 2: Gate(16..32) -> CombineRelationStraddle (4 Combine 16..20 + 12 Relation 20..32)
        let block_w2 = Block {
            kind: CoordinateKind::Gate,
            coordinates: (16..32).map(Parameter::Gate).collect(),
        };
        assert_eq!(
            classify_block(&cfg_rel, &block_w2)?,
            BlockComposition::CombineRelationStraddle
        );
        let elig_fz_w2 = block_eligibility(&cfg_fz, &block_w2)?;
        assert_eq!(elig_fz_w2.iter().filter(|&&e| e).count(), 4);
        assert_eq!(elig_fz_w2.iter().filter(|&&e| !e).count(), 12);

        // Window 3: Gate(32..45) -> RelationOnly (13 Relation coordinates)
        let block_w3 = Block {
            kind: CoordinateKind::Gate,
            coordinates: (32..45).map(Parameter::Gate).collect(),
        };
        assert_eq!(
            classify_block(&cfg_fz, &block_w3)?,
            BlockComposition::RelationOnly
        );
        let elig_fz_w3 = block_eligibility(&cfg_fz, &block_w3)?;
        assert!(elig_fz_w3.iter().all(|&e| !e));

        // Joint commit across the straddle boundary (ordinal 2 = Gate(18) Combine, ordinal 6 = Gate(22) Relation)
        let mut counters = StageTCounters::default();
        let mut interval = IntervalDelta::default();
        let straddle_receipt = BlockReceipt {
            lineage_sha256: "0".repeat(64),
            cursor_before: Cursor::initial(),
            cursor_after: Cursor::initial(),
            kind: CoordinateKind::Gate,
            k: 16,
            s_len: 2,
            joint: true,
            l_before: 500,
            l_joint: Some(420),
            l_best: 450,
            committed_loss: 420,
            commit_ordinals: vec![2, 6],
            o_vec: 112,
            o_read: 16,
            completed_batches: 15,
            model_epoch: 1,
        };
        counters.record_block_step(&mut interval, &cfg_rel, &block_w2, &straddle_receipt)?;
        assert_eq!(counters.combine_relation_straddle_batches, 1);
        assert_eq!(counters.combine.attempted_coordinates, 4);
        assert_eq!(counters.relation.attempted_coordinates, 12);
        assert_eq!(counters.combine.provisional_committed_moves, 1);
        assert_eq!(counters.relation.provisional_committed_moves, 1);
        assert_eq!(interval.accepted_updates, 1);
        assert_eq!(interval.committed_moves, 2);
        assert_eq!(interval.joint_commits, 1);

        // Settle passing interval
        counters.settle_interval(&interval, false);
        assert_eq!(counters.committed_accepted_updates, 1);
        assert_eq!(counters.committed_moves, 2);
        assert_eq!(counters.combine.committed_moves, 1);
        assert_eq!(counters.relation.committed_moves, 1);

        // Next interval breaches -> rolled back into rejected counters
        let mut interval2 = IntervalDelta::default();
        counters.record_block_step(&mut interval2, &cfg_rel, &block_w1, &straddle_receipt)?;
        counters.settle_interval(&interval2, true);
        assert_eq!(counters.committed_accepted_updates, 1);
        assert_eq!(counters.rejected_accepted_updates, 1);
        assert_eq!(counters.rejected_moves, 2);
        assert_eq!(counters.combine.rejected_moves, 2);

        // V8d refusal: FrozenZeroV1 must refuse any commit of a Relation ordinal (ordinal 6 = Gate(22))
        let mut fz_counters = StageTCounters::default();
        let mut fz_interval = IntervalDelta::default();
        assert!(fz_counters
            .record_block_step(&mut fz_interval, &cfg_fz, &block_w2, &straddle_receipt)
            .is_err());

        // Also verify K = 1 single-step accounting and rollback
        let cfg_t1 = load_committed_config("t1-frozen-zero-seed1.json");
        let mut t1_counters = StageTCounters::default();
        let mut t1_interval = IntervalDelta::default();
        let single_receipt = StepReceipt {
            lineage_sha256: "0".repeat(64),
            cursor_before: Cursor::initial(),
            cursor_after: Cursor::initial(),
            coordinate: Coordinate {
                kind: CoordinateKind::Gate,
                index: 22,
            },
            target_positions: vec![0; 64],
            candidate: 1,
            loss_before: 500,
            loss_after: 500,
            model_epoch: 0,
            applied: false,
            apply_refused_by_policy: true,
            any_candidate_cost_differs_from_identity: true,
            carrier: None,
        };
        t1_counters.record_single_step(&mut t1_interval, &cfg_t1, &single_receipt)?;
        assert_eq!(t1_counters.relation.policy_refused_coordinates, 1);
        assert_eq!(t1_counters.relation.provisional_committed_moves, 0);
        Ok(())
    }

    #[test]
    fn stage2_grader_verifies_pass_partial_and_diagnostic_stop_outcomes() -> Result<()> {
        let step0 = Step0Receipt {
            schema: STAGE_T_STEP0_SCHEMA.to_owned(),
            source_sha256: "a".repeat(64),
            kernel_sha256: "b".repeat(64),
            geometry_sha256: "c".repeat(64),
            driver_source_sha256: training::driver_source_sha256(),
            t16_fz_config_sha256: "d".repeat(64),
            t16_rel_config_sha256: "e".repeat(64),
            t1_fz_config_sha256: "f".repeat(64),
            akd_single_move_commits_checked: 32,
            akd_mismatches: 0,
            akd_pass: true,
            v1d_batches_checked: 24,
            v1d_mismatches: 0,
            v1d_final_snapshot_sha256: "1".repeat(64),
            v1d_pass: true,
            kid_batches_checked: 128,
            kid_commits_rescored: 20,
            kid_ineligible_noop_verified: true,
            kid_mismatches: 0,
            kid_pass: true,
            v8d_batches_checked: 128,
            v8d_relation_commits_on_frozen_zero: 0,
            v8d_nonzero_relation_trits: 0,
            v8d_pass: true,
            v9d_uninterrupted_128_snapshot_sha256: "2".repeat(64),
            v9d_resumed_128_snapshot_sha256: "2".repeat(64),
            v9d_epoch_128: 20,
            v9d_committed_moves_128: 44,
            v9d_pass: true,
            t16_128_batches_elapsed_ms: 20_000,
            projected_16404_seconds: 2_563,
            all_pass: true,
        };

        let make_seed = |seed: u32, top1: u64, status: StageTTerminalStatus| SeedStageTEval {
            arm_label: format!("T16_fz_seed{seed}"),
            seed,
            coordinate_block: 16,
            terminal_status: status,
            completed_batches: 16_404,
            accepted_updates: 500,
            committed_moves: 1_200,
            dev_target_admitted_rows: 4_290,
            dev_psucc_admitted_rows: 1_100,
            dev_top1_correct: top1,
            dev_top8_correct: 2_200,
            t1_control_top1_correct: 871,
            t1_control_halted_early: true,
            mcnemar_n10_t16_only: top1.saturating_sub(800),
            mcnemar_n01_t1_only: 71,
            mcnemar_required_delta: 286,
            checkpoint_t_restr_aw_hit1: 1_150,
            checkpoint_g_restr_aw_hit1: 1_480,
            checkpoint_u_restr_aw_hit1: 420,
            g_adm_pass: true,
            beats_unigram: top1 > 1_226,
            beats_unigram_plus_2se: top1 >= 1_344,
            beats_w2_baseline_code_restr: top1 > 1_138,
            beats_w2_baseline_bigram_restr: top1 > 1_482,
            beats_checkpoint_t_restr_aw: top1 > 1_150,
            beats_checkpoint_g_restr_aw: top1 > 1_480,
            b_adm_table_pass: top1 > 1_482,
            b_block_gain_pass: top1.saturating_sub(871) >= 286,
            beats_full_panel_t2: top1 > 6_300,
        };

        // 1. PASS_STAGE2_ADM_TABLE when both seeds beat unigram (1,226) and W2 admission-matched tables (1,138 / 1,482)
        let pass_grade = grade_stage_t(
            "docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md",
            &"9".repeat(64),
            &step0,
            &"8".repeat(64),
            vec![
                make_seed(1, 1_600, StageTTerminalStatus::CompletedFullDose),
                make_seed(2, 1_550, StageTTerminalStatus::CompletedFullDose),
            ],
            vec![
                make_seed(
                    1,
                    1_100,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
                make_seed(
                    2,
                    1_050,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
            ],
        )?;
        assert_eq!(pass_grade.verdict, StageTVerdict::PassStage2AdmTable);
        assert!(pass_grade.stage2_bar_met_two_seeds);
        assert_eq!(pass_grade.primary_arm, "T16_fz");

        // 2. PARTIAL_UNIGRAM_ONLY when both seeds beat unigram (1,226) and T_restr (1,138) but not G_restr (1,482)
        let partial_grade = grade_stage_t(
            "docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md",
            &"9".repeat(64),
            &step0,
            &"8".repeat(64),
            vec![
                make_seed(1, 1_360, StageTTerminalStatus::CompletedFullDose),
                make_seed(2, 1_350, StageTTerminalStatus::CompletedFullDose),
            ],
            vec![
                make_seed(
                    1,
                    900,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
                make_seed(
                    2,
                    910,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
            ],
        )?;
        assert_eq!(partial_grade.verdict, StageTVerdict::PartialUnigramOnly);
        assert!(!partial_grade.stage2_bar_met_two_seeds);
        assert!(partial_grade.unigram_bar_met_two_seeds);
        assert!(partial_grade
            .unlocked_next_items
            .iter()
            .all(|item| !item.contains("2846")));

        // 3. DIAGNOSTIC_STOP_CONSECUTIVE_REJECTIONS when consecutive rejections stop before clearing unigram
        let stop_grade = grade_stage_t(
            "docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md",
            &"9".repeat(64),
            &step0,
            &"8".repeat(64),
            vec![
                make_seed(
                    1,
                    950,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
                make_seed(
                    2,
                    920,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
            ],
            vec![
                make_seed(
                    1,
                    880,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
                make_seed(
                    2,
                    860,
                    StageTTerminalStatus::DiagnosticStopConsecutiveRejections,
                ),
            ],
        )?;
        assert_eq!(
            stop_grade.verdict,
            StageTVerdict::DiagnosticStopConsecutiveRejections
        );
        assert!(!stop_grade.mechanism_negative);

        // 4. Failed Step 0 control must refuse grading
        let mut broken_step0 = step0;
        broken_step0.akd_pass = false;
        broken_step0.all_pass = false;
        assert!(grade_stage_t(
            "docs/harness/prep/m1-bvg-k-stage-t-training-2026-10-25.md",
            &"9".repeat(64),
            &broken_step0,
            &"8".repeat(64),
            vec![
                make_seed(1, 1_600, StageTTerminalStatus::CompletedFullDose),
                make_seed(2, 1_550, StageTTerminalStatus::CompletedFullDose),
            ],
            vec![
                make_seed(1, 1_600, StageTTerminalStatus::CompletedFullDose),
                make_seed(2, 1_550, StageTTerminalStatus::CompletedFullDose),
            ],
        )
        .is_err());
        Ok(())
    }

    #[test]
    fn relation_head_grader_verifies_gates_within_arm_lift_and_outcome_invariance() -> Result<()> {
        let step0 = Step0Receipt {
            schema: STAGE_T_STEP0_SCHEMA.to_owned(),
            source_sha256: "be02d3eff523d73bde5a5e15533cd0a1e6e65eb596416fa85f21184e8fad6be8"
                .to_owned(),
            kernel_sha256: "0f9ccfa7b4748292f83a40227e30ea7b69c893a1649fa8afa3f7d547802429d6"
                .to_owned(),
            geometry_sha256: "46ddbb76b7fbe6ac6222ea24779fc10055b1d6cea6a8da786895676fd29fe327"
                .to_owned(),
            driver_source_sha256: training::driver_source_sha256(),
            t16_fz_config_sha256:
                "77d61451125b04827821216e221d548885af188bad6d9d301234d126fefa3b7d".to_owned(),
            t16_rel_config_sha256:
                "ab41788a5e72a1f11370f875b090653e0d1b36be091393861b0e147688e9d746".to_owned(),
            t1_fz_config_sha256: "cf7ebcc85d2e463ae8f6691b63d90a0e005701cb38b8249a8d0d5460d8508479"
                .to_owned(),
            akd_single_move_commits_checked: 32,
            akd_mismatches: 0,
            akd_pass: true,
            v1d_batches_checked: 24,
            v1d_mismatches: 0,
            v1d_final_snapshot_sha256:
                "5de50c024f5507a2805aeda6942d78effc8b5d12396d298d7deea368cc13bdad".to_owned(),
            v1d_pass: true,
            kid_batches_checked: 128,
            kid_commits_rescored: 39,
            kid_ineligible_noop_verified: true,
            kid_mismatches: 0,
            kid_pass: true,
            v8d_batches_checked: 128,
            v8d_relation_commits_on_frozen_zero: 0,
            v8d_nonzero_relation_trits: 0,
            v8d_pass: true,
            v9d_uninterrupted_128_snapshot_sha256:
                "3545fe65c5c35f748dc6f9115d30b3fd5ec415c1253a63a4cd2f47466585d12d".to_owned(),
            v9d_resumed_128_snapshot_sha256:
                "3545fe65c5c35f748dc6f9115d30b3fd5ec415c1253a63a4cd2f47466585d12d".to_owned(),
            v9d_epoch_128: 39,
            v9d_committed_moves_128: 79,
            v9d_pass: true,
            t16_128_batches_elapsed_ms: 27_519,
            projected_16404_seconds: 3_526,
            all_pass: true,
        };

        let controls = RelationHeadControlBaselines {
            c_route_cp0_dev_b3_full_admitted: 748,
            c_route_cp0_dev_b3_psucc_admitted: 408,
            c_route_cp0_dev_top1: 0,
            c_route_cp0_dev_top8: 74,
            c_successor_only_cp0_dev_b3_psucc_admitted: 2_034,
            c_frozen_zero_cp0_dev_b3_psucc_admitted: 1_351,
            outcome_invariant_without_c_route_or_c_successor_rerun: true,
        };

        let within_cell = RelationHeadWithinCellSummary {
            rel_admitted_target_loses_within_own_cell_rows: 3_226,
            rel_t_restr_hits_w_misses: 1_291,
            rel_missed_t_restr_lost_within_own_cell_rows: 1_190,
            rel_reranker_code_top1_cell_only_hits: 1_095,
            rel_reranker_code_cell_prio_top1_hits: 1_597,
            rel_reranker_code_pooled_top1_hits: 1_602,
            rel_reranker_code_permuted_null_top1_hits: 554,
            rel_paired_reranker_vs_writer_net_diff_d: 1_231,
            rel_paired_reranker_vs_writer_floor_f: 74,
            rel_paired_reranker_vs_permuted_null_net_diff_d: 1_048,
            rel_paired_reranker_vs_permuted_null_floor_f: 74,
            fz_admitted_target_loses_within_own_cell_rows: 4_208,
            fz_t_restr_hits_w_misses: 1_765,
            fz_missed_t_restr_lost_within_own_cell_rows: 1_597,
            fz_reranker_code_top1_cell_only_hits: 1_706,
            fz_reranker_code_cell_prio_top1_hits: 2_142,
            fz_reranker_code_pooled_top1_hits: 2_140,
            fz_reranker_code_permuted_null_top1_hits: 753,
        };

        let make_rel_seed = |seed: u32,
                             rel_l2_dev: u64,
                             rel_l3_dev: u64,
                             rel_l2_fresh: u64,
                             fz_l2_fresh: u64,
                             fz_l3_dev: u64| RelationHeadSeedSummary {
            seed,
            rel_config_sha256: "a".repeat(64),
            rel_checkpoint_sha256: "b".repeat(64),
            rel_model_snapshot_sha256: "c".repeat(64),
            fz_config_sha256: "d".repeat(64),
            fz_checkpoint_sha256: "e".repeat(64),
            fz_model_snapshot_sha256: "f".repeat(64),
            rel_completed_batches: 4_096,
            rel_accepted_updates: 678,
            rel_committed_moves: 1_505,
            rel_vector_committed_moves: 545,
            rel_combine_committed_moves: 535,
            rel_relation_committed_batches: 186,
            rel_relation_committed_moves: 425,
            rel_joint_commits: 199,
            rel_context_tokens_per_sec: if seed == 1 { 11_562 } else { 11_201 },
            fz_completed_batches: 3_008,
            fz_accepted_updates: 420,
            fz_committed_moves: 854,
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
            rel_distinct_train_read_codes: 27_132,
            rel_distinct_dev_read_codes: 4_429,
            fz_distinct_train_read_codes: 19_385,
            fz_distinct_dev_read_codes: 4_067,
            rel_dev_b3_full_admitted: 3_827,
            rel_dev_b3_psucc_admitted: 1_144,
            rel_dev_writer_top1_all: 371,
            rel_dev_writer_top8_all: 1_124,
            rel_dev_writer_top1_psucc: 78,
            rel_dev_writer_top8_psucc: 243,
            rel_dev_strict_wins: 172,
            rel_off_dev_b3_full_admitted: 3_753,
            rel_off_dev_b3_psucc_admitted: 1_129,
            rel_off_dev_writer_top1_all: 327,
            rel_off_dev_writer_top8_all: 1_063,
            rel_off_dev_strict_wins: 123,
            within_arm_rel_minus_rel_off_admitted: 74,
            within_arm_rel_minus_rel_off_top1: 44,
            within_arm_rel_minus_rel_off_top8: 61,
            within_arm_rel_minus_rel_off_strict_wins: 49,
            within_arm_rel_minus_t1_rel_top1: 238,
            fz_dev_b3_full_admitted: 5_009,
            fz_dev_b3_psucc_admitted: 1_216,
            fz_dev_writer_top1_all: 455,
            fz_dev_writer_top8_all: 1_484,
            fz_dev_writer_top1_psucc: 107,
            fz_dev_writer_top8_psucc: 318,
            fz_dev_strict_wins: 93,
            rel_minus_fz_dev_b3_full: -1_182,
            rel_minus_fz_dev_b3_psucc: -72,
            rel_minus_fz_dev_writer_top1: -84,
            rel_minus_fz_dev_writer_top8: -360,
            unconditional_unigram_hit1: 1_226,
            unconditional_t2_bigram_hit1: 6_300,
            rel_full_panel_code_table_hit1: 3_680,
            fz_full_panel_code_table_hit1: 4_085,
            rel_checkpoint_u_restr_aw_hit1: 942,
            rel_checkpoint_t_sym_restr_aw_hit1: 1_377,
            rel_checkpoint_t_restr_aw_hit1_all: 1_602,
            rel_checkpoint_t_restr_aw_hit8_all: 3_007,
            rel_checkpoint_t_restr_aw_hit1_psucc: 327,
            rel_checkpoint_g_restr_aw_hit1_all: 2_364,
            rel_checkpoint_g_restr_aw_hit8_all: 3_529,
            rel_checkpoint_g_restr_aw_hit1_psucc: 496,
            fz_checkpoint_u_restr_aw_hit1: 1_127,
            fz_checkpoint_t_sym_restr_aw_hit1: 1_909,
            fz_checkpoint_t_restr_aw_hit1_all: 2_140,
            fz_checkpoint_t_restr_aw_hit8_all: 4_006,
            fz_checkpoint_t_restr_aw_hit1_psucc: 376,
            fz_checkpoint_g_restr_aw_hit1_all: 3_084,
            fz_checkpoint_g_restr_aw_hit8_all: 4_674,
            fz_checkpoint_g_restr_aw_hit1_psucc: 517,
            rel_minus_fz_t_restr_aw_hit1: -538,
            rel_minus_fz_g_restr_aw_hit1: -720,
            rel_trunk_dev_level0_top1: 371,
            rel_trunk_dev_level1_top1: 1_557,
            rel_trunk_dev_level2_top1: rel_l2_dev,
            rel_trunk_dev_level3_top1: rel_l3_dev,
            rel_trunk_dev_recurrence_lift_l2_minus_l1: i64::try_from(rel_l2_dev).unwrap() - 1_557,
            rel_trunk_fresh_level0_top1: 439,
            rel_trunk_fresh_level1_top1: 1_559,
            rel_trunk_fresh_level2_top1: rel_l2_fresh,
            rel_trunk_fresh_level3_top1: 2_427,
            rel_trunk_fresh_recurrence_lift_l2_minus_l1: i64::try_from(rel_l2_fresh).unwrap()
                - 1_559,
            fz_trunk_dev_level0_top1: 455,
            fz_trunk_dev_level1_top1: 2_054,
            fz_trunk_dev_level2_top1: 2_087,
            fz_trunk_dev_level3_top1: fz_l3_dev,
            fz_trunk_dev_recurrence_lift_l2_minus_l1: 33,
            fz_trunk_fresh_level0_top1: 457,
            fz_trunk_fresh_level1_top1: 2_187,
            fz_trunk_fresh_level2_top1: fz_l2_fresh,
            fz_trunk_fresh_level3_top1: 3_163,
            fz_trunk_fresh_recurrence_lift_l2_minus_l1: i64::try_from(fz_l2_fresh).unwrap() - 2_187,
            rel_minus_fz_trunk_dev_level3_top1: i64::try_from(rel_l3_dev).unwrap()
                - i64::try_from(fz_l3_dev).unwrap(),
            rel_minus_fz_trunk_fresh_level3_top1: -736,
        };

        let s1 = make_rel_seed(1, 1_624, 2_227, 1_594, 2_207, 2_988);
        let s2 = make_rel_seed(2, 1_610, 2_226, 1_602, 2_225, 2_989);

        let report = grade_relation_head(
            "docs/harness/prep/r-finite-relational-head-2026-10-25.md",
            &"1".repeat(64),
            "docs/loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md",
            &"2".repeat(64),
            &step0,
            &"3".repeat(64),
            &"4".repeat(64),
            &"5".repeat(64),
            controls.clone(),
            within_cell.clone(),
            vec![s1.clone(), s2.clone()],
        )?;

        assert_eq!(report.schema, RELATION_HEAD_GRADE_SCHEMA);
        assert_eq!(report.item, "r-finite-relational-head");
        assert_eq!(report.issue, 2877);
        assert!(report.gate_c1_to_c5_and_step0_pass);
        assert!(report.gate_d40_two_seeds_pass);
        assert!(report.gate_s_two_seeds_pass);
        assert!(report.within_arm_relation_beats_relation_off_two_seeds);
        assert!(report.within_arm_relation_beats_t1_rel_two_seeds);
        assert!(report.within_arm_relation_expands_read_codes_two_seeds);
        assert!(report.within_arm_relation_recurrence_lift_exceeds_fz_two_seeds);
        assert!(!report.between_arm_relation_beats_frozen_zero_two_seeds);
        assert!(!report.m1_closing_bar_met_two_seeds);
        assert!(report.outcome_invariance_verified);
        assert!(!report.mechanism_negative);
        assert_eq!(
            report.verdict,
            RelationHeadVerdict::RelationalWithinArmGainTrailsFrozenZeroAdmission
        );

        // Gate S failure must be refused
        let mut bad_gate_s = s1.clone();
        bad_gate_s.rel_relation_committed_batches = 8;
        assert!(grade_relation_head(
            "docs/harness/prep/r-finite-relational-head-2026-10-25.md",
            &"1".repeat(64),
            "docs/loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md",
            &"2".repeat(64),
            &step0,
            &"3".repeat(64),
            &"4".repeat(64),
            &"5".repeat(64),
            controls.clone(),
            within_cell.clone(),
            vec![bad_gate_s, s2.clone()],
        )
        .is_err());

        // Gate D40 failure must be refused
        let mut bad_d40 = s1;
        bad_d40.rel_train_anchor_final_admitted = 1_100;
        assert!(grade_relation_head(
            "docs/harness/prep/r-finite-relational-head-2026-10-25.md",
            &"1".repeat(64),
            "docs/loop/tasks/r-finite-relational-head/antigravity-96/DECISION.md",
            &"2".repeat(64),
            &step0,
            &"3".repeat(64),
            &"4".repeat(64),
            &"5".repeat(64),
            controls,
            within_cell,
            vec![bad_d40, s2],
        )
        .is_err());
        Ok(())
    }
}
