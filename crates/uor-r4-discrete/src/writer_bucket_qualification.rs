//! Finite CUDA qualification for the CSR bucketed writer (`m1-faster-writer`, #2840).
//!
//! Executes five control groups on the GPU:
//! 1. `device_end_to_end_parity`: seeded `Device` models across `V in {64, 256, 4096}`,
//!    `B = 4`, `K in {8, 16}`, and both `ObjectivePolicy` modes (`NearestAddressMarginV1`
//!    and `OccupiedCellRankHingeV1`), checking exact parity across `forward`, `evaluate`,
//!    and `propose` (`VectorBlock`, `Gate`).
//! 2. `direct_seeded_sweep`: direct GPU writer sweep (`discrete_fullscan_writer_eval` vs
//!    `discrete_bucket_writer_eval` in `IdOrder` and `ServingPruned` modes) across
//!    `V in {64, 256, 4096, 16384}`, `B in {2, 4}`, `K in {1, 8, 16}`, `seed in {1, 2, 3}`,
//!    plus virtual moves `kind = 1`.
//! 3. `edge_cases_and_incremental_update`: single-cell collapse (`|B[c]| = V`), uniform
//!    occupancy (`|B[c]| = V / 256` at `V = 4096` and `V = 16384`), fewer than `K` occupied
//!    cells, `ABSENT` non-root token refusal (`status == 8`), constructed cell and token
//!    ties, virtual moves across cells, and a `64`-step committed move sequence comparing
//!    incremental `discrete_bucket_update` against fresh `discrete_bucket_build` at every step.
//! 4. `real_read_dev_panel`: full `14,287`-row `DEV` panel (`ft/dev/tokens.u16` and
//!    `ft/dev/response_mask.u8`) on both pinned `A8` (`6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1`)
//!    and pinned `PrimeZeta` (`83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b`),
//!    verifying `0` mismatches across `forward`, `evaluate`, and direct `IdOrder` /
//!    `ServingPruned` writer evaluations, and recording exact comparison counts and `tokens/s`.
//! 5. `planted_faults`: four negative controls (`FAULT_STALE_BUCKET`, `FAULT_REVERSED_TIE`,
//!    `FAULT_OVER_PRUNED`, `FAULT_CORRUPTED_CSR`), verifying each produces a non-zero mismatch
//!    caught by the parity check.

#![forbid(unsafe_code)]

use crate::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use crate::device::{
    bucket_extends_ordinary_source, compiled_bucket_image_sha256, compiled_identity,
    BucketWriterEngine, Device, DirectTripleTimed, DirectWriterOutput, FAULT_CORRUPTED_CSR,
    FAULT_NONE, FAULT_OVER_PRUNED, FAULT_REVERSED_TIE, FAULT_STALE_BUCKET, WALK_ID_ORDER,
    WALK_SERVING_PRUNED,
};
use crate::metadata::ABSENT;
use crate::training::{driver_source_sha256, CodebookTable, CODEBOOK_TABLE_SCHEMA};
use crate::{
    Error, Identity, ObjectivePolicy, Parameter, ReadInitialization, ReadSpec, Result, Spec,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::time::Instant;

mod geometry {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}

pub const QUALIFICATION_SCHEMA: &str = "uor-r4.writer-bucket-qualification/1";
pub const DEV_TOKENS_SHA256: &str =
    "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b";
pub const DEV_MASK_SHA256: &str =
    "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078";
pub const A8_FILE_SHA256: &str = "6600bdb1306dbaea4d77f5bf021e717a6bef791e44c1ec7d7c9bb9f0e8cd3cd1";
pub const A8_TABLE_SHA256: &str =
    "4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74";
pub const PZ_FILE_SHA256: &str = "83c1380a8f507e51d0342faa1b089dea6a20561a03552208c093fa640afc124b";
pub const DEV_ELIGIBLE_ROWS: u64 = 14_287;
pub const DEV_PAYLOAD_TOKENS: u64 = 42_856;
pub const A8_EXPECTED_TOP1: u64 = 133;
pub const PZ_EXPECTED_TOP1: u64 = 38;
pub const MIN_REDUCTION_X100: u64 = 1_000;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceParityCase {
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
    pub seed: u32,
    pub objective_policy: ObjectivePolicy,
    pub occupied_cells: u32,
    pub max_bucket_size: u32,
    pub forward_rows: u32,
    pub forward_mismatches: u32,
    pub evaluate_mismatches: u32,
    pub propose_cases: u32,
    pub propose_mismatches: u32,
    pub build_us: u64,
    pub fullscan_forward_us: u64,
    pub bucket_forward_us: u64,
    pub fullscan_evaluate_us: u64,
    pub bucket_evaluate_us: u64,
    pub fullscan_propose_us: u64,
    pub bucket_propose_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaleCounterSummary {
    pub vocabulary: u32,
    pub blocks: u32,
    pub kept_cells: u32,
    pub cases: u32,
    pub total_rows: u64,
    pub mismatches: u64,
    pub max_bucket_size: u32,
    pub fullscan_token_cell_per_row: u64,
    pub fullscan_cell_distance_per_row: u64,
    pub fullscan_token_distance_per_row: u64,
    pub fullscan_total_evals_per_row: u64,
    pub id_order_token_cell_per_row: u64,
    pub id_order_cell_distance_per_row: u64,
    pub id_order_token_distance_per_row: u64,
    pub id_order_max_token_distance_per_row: u32,
    pub id_order_total_evals_per_row: u64,
    pub id_order_reduction_x100: u64,
    pub id_order_reduction_whole: u64,
    pub serving_pruned_token_distance_per_row: u64,
    pub serving_pruned_max_token_distance_per_row: u32,
    pub serving_pruned_total_evals_per_row: u64,
    pub serving_pruned_reduction_x100: u64,
    pub serving_pruned_reduction_whole: u64,
    pub build_us: u64,
    pub fullscan_us: u64,
    pub id_order_us: u64,
    pub serving_pruned_us: u64,
    pub fullscan_tokens_per_s: u64,
    pub id_order_tokens_per_s: u64,
    pub serving_pruned_tokens_per_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectSweepSummary {
    pub total_cases: u32,
    pub total_rows: u64,
    pub virtual_move_cases: u32,
    pub cell_changing_virtual_moves: u32,
    pub total_mismatches: u64,
    pub scales: Vec<ScaleCounterSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeCaseSummary {
    pub single_cell_collapse_mismatches: u32,
    pub single_cell_collapse_token_distance_evals: u32,
    pub uniform_v4096_mismatches: u32,
    pub uniform_v4096_id_order_total_evals: u64,
    pub uniform_v4096_fullscan_total_evals: u64,
    pub uniform_v4096_reduction_x100: u64,
    pub uniform_v16384_mismatches: u32,
    pub uniform_v16384_id_order_total_evals: u64,
    pub uniform_v16384_fullscan_total_evals: u64,
    pub uniform_v16384_reduction_x100: u64,
    pub fewer_than_k_cells_mismatches: u32,
    pub fewer_than_k_admitted_count: u32,
    pub absent_refusal_cases: u32,
    pub absent_refusal_mismatches: u32,
    pub tie_cases_mismatches: u32,
    pub virtual_move_edge_mismatches: u32,
    pub incremental_update_steps: u32,
    pub incremental_update_accepted_commits: u32,
    pub incremental_update_cell_changes: u32,
    pub incremental_update_csr_mismatches: u32,
    pub incremental_update_writer_mismatches: u32,
    pub incremental_update_max_entries_written: u32,
    pub incremental_update_bound: u32,
    pub incremental_update_total_us: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealReadCodebookSummary {
    pub name: String,
    pub file_sha256: String,
    pub table_sha256: String,
    pub vocabulary: u32,
    pub blocks: u32,
    pub context: u32,
    pub kept_cells: u32,
    pub occupied_cells: u32,
    pub min_nonempty_bucket_size: u32,
    pub max_bucket_size: u32,
    pub scored_rows: u64,
    pub batches: u32,
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub forward_mismatches: u64,
    pub evaluate_mismatches: u64,
    pub direct_id_order_mismatches: u64,
    pub direct_serving_pruned_mismatches: u64,
    pub fullscan_token_cell_per_row: u64,
    pub fullscan_cell_distance_per_row: u64,
    pub fullscan_token_distance_per_row: u64,
    pub fullscan_total_evals_per_row: u64,
    pub id_order_token_cell_per_row: u64,
    pub id_order_cell_distance_per_row: u64,
    pub id_order_token_distance_per_row: u64,
    pub id_order_max_token_distance_per_row: u32,
    pub id_order_total_evals_per_row: u64,
    pub id_order_reduction_x100: u64,
    pub id_order_reduction_whole: u64,
    pub serving_pruned_token_distance_per_row: u64,
    pub serving_pruned_max_token_distance_per_row: u32,
    pub serving_pruned_total_evals_per_row: u64,
    pub serving_pruned_reduction_x100: u64,
    pub serving_pruned_reduction_whole: u64,
    pub build_us: u64,
    pub fullscan_forward_us: u64,
    pub bucket_forward_us: u64,
    pub fullscan_evaluate_us: u64,
    pub bucket_evaluate_us: u64,
    pub direct_fullscan_us: u64,
    pub direct_id_order_us: u64,
    pub direct_serving_pruned_us: u64,
    pub fullscan_forward_tokens_per_s: u64,
    pub bucket_forward_tokens_per_s: u64,
    pub fullscan_evaluate_tokens_per_s: u64,
    pub bucket_evaluate_tokens_per_s: u64,
    pub direct_fullscan_tokens_per_s: u64,
    pub direct_id_order_tokens_per_s: u64,
    pub direct_serving_pruned_tokens_per_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlantedFaultCase {
    pub fault_mode: u32,
    pub fault_name: String,
    pub rows: u32,
    pub mismatches_detected: u32,
    pub caught: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BucketQualificationReport {
    pub schema: String,
    pub status: String,
    pub b1_parity_pass: bool,
    pub b2_comparison_reduction_pass: bool,
    pub negative_controls_pass: bool,
    pub driver_source_sha256: String,
    pub compiled_identity: Value,
    pub bucket_image: Value,
    pub dev_tokens_sha256: String,
    pub dev_mask_sha256: String,
    pub device_parity_cases: Vec<DeviceParityCase>,
    pub direct_seeded_sweep: DirectSweepSummary,
    pub edge_cases: EdgeCaseSummary,
    pub real_read_dev_panel: Vec<RealReadCodebookSummary>,
    pub planted_faults: Vec<PlantedFaultCase>,
}

// ---------------------------------------------------------------------------
// Deterministic integer table and query generators for direct GPU controls
// ---------------------------------------------------------------------------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    fn below(&mut self, n: usize) -> usize {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x % (n as u64)) as usize
    }
}

fn root(r: u32) -> [i32; 8] {
    geometry::ROOTS[r as usize].map(i32::from)
}

fn action(a: u8, x: &[i32; 8]) -> [i32; 8] {
    let mut y = [0; 8];
    for (d, out) in y.iter_mut().enumerate() {
        let axis = geometry::AXES[usize::from(a) * 8 + d];
        let v = x[usize::from(axis & 7)];
        *out = if axis & 8 != 0 { -v } else { v };
    }
    y
}

fn move_action(candidate: u32) -> u8 {
    if candidate != 0 {
        geometry::GENERATORS[candidate as usize - 1]
    } else {
        geometry::IDENTITY
    }
}

fn root_id(x: &[i32; 8]) -> u32 {
    (0..16).find(|&r| root(r) == *x).unwrap_or(ABSENT)
}

fn root_cost(x: &[i32; 8], r: u32) -> u32 {
    x.iter().zip(root(r)).map(|(&a, b)| a.abs_diff(b)).sum()
}

fn non_root() -> [i32; 8] {
    [9; 8]
}

fn perturb(rng: &mut Rng, mut x: [i32; 8]) -> [i32; 8] {
    x[rng.below(8)] += if rng.below(2) == 0 { 1 } else { -1 };
    x
}

fn flatten_tokens(tokens: &[Vec<[i32; 8]>]) -> Vec<i32> {
    let mut table = Vec::new();
    for token in tokens {
        for block in token {
            table.extend_from_slice(block);
        }
    }
    table
}

fn random_valid_vocab(rng: &mut Rng, palette: &[u32], n: usize, blocks: usize) -> Vec<i32> {
    let mut tokens: Vec<Vec<[i32; 8]>> = Vec::with_capacity(n);
    for _ in 0..n {
        if !tokens.is_empty() && rng.below(4) == 0 {
            let copy = tokens[rng.below(tokens.len())].clone();
            tokens.push(copy);
            continue;
        }
        let token = (0..blocks)
            .map(|b| {
                let x = root(palette[rng.below(palette.len())]);
                let interior = b != 0 && b != blocks - 1;
                if interior && rng.below(3) == 0 {
                    perturb(rng, x)
                } else {
                    x
                }
            })
            .collect();
        tokens.push(token);
    }
    flatten_tokens(&tokens)
}

fn random_queries_and_targets(
    rng: &mut Rng,
    palette: &[u32],
    rows: usize,
    blocks: usize,
    vocabulary: usize,
) -> (Vec<i32>, Vec<u32>) {
    let mut queries = Vec::with_capacity(rows * blocks * 8);
    let mut targets = Vec::with_capacity(rows);
    for _ in 0..rows {
        for _ in 0..blocks {
            let x = root(palette[rng.below(palette.len())]);
            let q = if rng.below(3) == 0 {
                perturb(rng, x)
            } else {
                x
            };
            queries.extend_from_slice(&q);
        }
        targets.push(rng.below(vocabulary) as u32);
    }
    (queries, targets)
}

fn equal_distance_pair(from: u32) -> Result<(u32, u32)> {
    for x in 0..16 {
        for y in (x + 1)..16 {
            if x != from && y != from && root_cost(&root(from), x) == root_cost(&root(from), y) {
                return Ok((x, y));
            }
        }
    }
    Err(invalid("no equal-distance root pair found"))
}

fn crossing_move() -> Result<(u32, u32, u32)> {
    for r in 0..16 {
        for candidate in 1..16 {
            let to = root_id(&action(move_action(candidate), &root(r)));
            if to != ABSENT && to != r {
                return Ok((r, candidate, to));
            }
        }
    }
    Err(invalid("no generator moves a root to another root"))
}

fn tokens_per_second(rows: u64, elapsed_us: u64) -> u64 {
    (rows * 1_000_000) / elapsed_us.max(1)
}

/// Counts row-level mismatches between two `DirectWriterOutput` records.
/// Note: for `WALK_SERVING_PRUNED`, `losses`, `token_rivals`, `top8`, and `metrics`
/// reflect the pruned walk over admitted cells, whereas `status`, `predictions`,
/// and `admitted` (and `address_rivals` / `address_hinge`) must match the full-scan
/// writer identically on every row. For `WALK_ID_ORDER`, every field including
/// `losses`, `token_rivals`, `top8`, and `metrics` must match `full` identically.
fn count_direct_mismatches(
    full: &DirectWriterOutput,
    bucket: &DirectWriterOutput,
    kept_cells: u32,
    check_full_loss_and_top8: bool,
) -> u32 {
    if full.status != bucket.status {
        return full.predictions.len().max(1) as u32;
    }
    if full.status != 0 {
        return u32::from(full.predictions != bucket.predictions);
    }
    let rows = full.predictions.len();
    let k = kept_cells as usize;
    let mut mismatches = 0u32;
    for row in 0..rows {
        let full_cells = &full.admitted[row * 16..row * 16 + k];
        let bucket_cells = &bucket.admitted[row * 16..row * 16 + k];
        let mut row_bad = full.predictions[row] != bucket.predictions[row]
            || full.address_rivals[row] != bucket.address_rivals[row]
            || full_cells != bucket_cells;
        if check_full_loss_and_top8 {
            let full_top8 = &full.top8[row * 8..(row + 1) * 8];
            let bucket_top8 = &bucket.top8[row * 8..(row + 1) * 8];
            let full_metrics = &full.metrics[row * 14..(row + 1) * 14];
            let bucket_metrics = &bucket.metrics[row * 14..(row + 1) * 14];
            row_bad = row_bad
                || full.losses[row] != bucket.losses[row]
                || full.token_rivals[row] != bucket.token_rivals[row]
                || full_top8 != bucket_top8
                || full_metrics != bucket_metrics;
        }
        if row_bad {
            mismatches += 1;
        }
    }
    mismatches
}

// ---------------------------------------------------------------------------
// Control Group 1: Seeded Device End-to-End Parity
// ---------------------------------------------------------------------------

fn run_device_parity_controls(ordinal: usize) -> Result<Vec<DeviceParityCase>> {
    let configs = [
        (
            64u32,
            4u32,
            8u32,
            8u32,
            11u32,
            ObjectivePolicy::NearestAddressMarginV1,
        ),
        (64, 4, 8, 8, 12, ObjectivePolicy::OccupiedCellRankHingeV1),
        (64, 4, 8, 16, 13, ObjectivePolicy::NearestAddressMarginV1),
        (64, 4, 8, 16, 14, ObjectivePolicy::OccupiedCellRankHingeV1),
        (256, 4, 8, 8, 21, ObjectivePolicy::NearestAddressMarginV1),
        (256, 4, 8, 8, 22, ObjectivePolicy::OccupiedCellRankHingeV1),
        (256, 4, 8, 16, 23, ObjectivePolicy::NearestAddressMarginV1),
        (256, 4, 8, 16, 24, ObjectivePolicy::OccupiedCellRankHingeV1),
        (4096, 4, 8, 8, 31, ObjectivePolicy::NearestAddressMarginV1),
        (4096, 4, 8, 8, 32, ObjectivePolicy::OccupiedCellRankHingeV1),
        (4096, 4, 8, 16, 33, ObjectivePolicy::NearestAddressMarginV1),
        (4096, 4, 8, 16, 34, ObjectivePolicy::OccupiedCellRankHingeV1),
    ];
    let mut cases = Vec::with_capacity(configs.len());
    for (vocabulary, blocks, context, kept_cells, seed, objective_policy) in configs {
        let spec = Spec {
            vocabulary,
            blocks,
            context,
            kept_cells,
        };
        let model = Device::new_with_initialization(
            spec,
            ReadSpec::Flock { kept: 8 },
            objective_policy,
            ReadInitialization::ActiveLeafV1,
            seed,
            ordinal,
        )?;
        let engine = BucketWriterEngine::from_device(&model)?;
        model.synchronize()?;

        let t_build = Instant::now();
        let csr = engine.build_index(&model)?;
        engine.synchronize()?;
        let build_us = t_build.elapsed().as_micros() as u64;

        let csr_snap = csr.snapshot(engine.stream())?;
        csr_snap.validate(vocabulary)?;
        if csr_snap.absent_count != 0 {
            return Err(invalid("seeded Device model has non-zero absent_count"));
        }
        let mut occupied_cells = 0u32;
        let mut max_bucket_size = 0u32;
        for c in 0..256usize {
            let count = csr_snap.offsets[c + 1] - csr_snap.offsets[c];
            if count > 0 {
                occupied_cells += 1;
                if count > max_bucket_size {
                    max_bucket_size = count;
                }
            }
        }

        let rows = 16usize;
        let mut windows = Vec::with_capacity(rows);
        let mut targets = Vec::with_capacity(rows);
        for r in 0..rows {
            let len = 2 + (r % (context as usize - 1));
            let win = (0..len)
                .map(|pos| Identity {
                    token: ((r * 13 + pos * 7 + (seed as usize)) % (vocabulary as usize)) as u32,
                    occurrence: (r * 16 + pos + 1) as u64,
                    record: 1,
                    version: 0,
                })
                .collect::<Vec<_>>();
            windows.push(win);
            targets.push(((r * 29 + 3) % (vocabulary as usize)) as u32);
        }
        let carried = model.carry_windows(&windows)?;
        model.synchronize()?;

        let t_ff = Instant::now();
        let full_fwd = model.forward(&carried, &targets)?;
        model.synchronize()?;
        let fullscan_forward_us = t_ff.elapsed().as_micros() as u64;

        let t_bf = Instant::now();
        let bucket_fwd = engine.forward(&model, &csr, &carried, &targets)?;
        engine.synchronize()?;
        let bucket_forward_us = t_bf.elapsed().as_micros() as u64;

        let mut forward_mismatches = 0u32;
        if full_fwd.predictions != bucket_fwd.predictions
            || full_fwd.losses != bucket_fwd.losses
            || full_fwd.token_rivals != bucket_fwd.token_rivals
            || full_fwd.address_rivals != bucket_fwd.address_rivals
            || full_fwd.admitted_cells != bucket_fwd.admitted_cells
            || full_fwd.vectors()? != bucket_fwd.vectors()?
        {
            forward_mismatches += 1;
        }

        let t_fe = Instant::now();
        let full_eval = model.evaluate(&carried, &targets)?;
        model.synchronize()?;
        let fullscan_evaluate_us = t_fe.elapsed().as_micros() as u64;

        let t_be = Instant::now();
        let bucket_eval = engine.evaluate(&model, &csr, &carried, &targets)?;
        engine.synchronize()?;
        let bucket_evaluate_us = t_be.elapsed().as_micros() as u64;

        let mut evaluate_mismatches = 0u32;
        let full_agg =
            serde_json::to_vec(&full_eval.aggregate).map_err(|e| invalid(e.to_string()))?;
        let bucket_agg =
            serde_json::to_vec(&bucket_eval.aggregate).map_err(|e| invalid(e.to_string()))?;
        if full_agg != bucket_agg || full_eval.rows.len() != bucket_eval.rows.len() {
            evaluate_mismatches += 1;
        }
        for (a, b) in full_eval.rows.iter().zip(&bucket_eval.rows) {
            if a.top8 != b.top8
                || a.token_hinge != b.token_hinge
                || a.address_hinge != b.address_hinge
                || a.cell_rank_hinge != b.cell_rank_hinge
                || a.target_cell_rank != b.target_cell_rank
                || a.occupied_cells != b.occupied_cells
                || a.admitted_token_count != b.admitted_token_count
                || a.loss_ta_numerator != b.loss_ta_numerator
                || a.loss_tr_numerator != b.loss_tr_numerator
                || a.objective_loss_numerator != b.objective_loss_numerator
                || a.total_loss_numerator != b.total_loss_numerator
                || a.token_rivals != b.token_rivals
                || a.address_rivals != b.address_rivals
                || a.top1_correct != b.top1_correct
                || a.top8_correct != b.top8_correct
                || a.admitted_cells != b.admitted_cells
            {
                evaluate_mismatches += 1;
            }
        }

        let target0 = targets[0];
        let params = [
            Parameter::VectorBlock(target0 * blocks),
            Parameter::VectorBlock(target0 * blocks + 1),
            Parameter::VectorBlock(target0 * blocks + (blocks - 1)),
            Parameter::Gate(0),
            Parameter::Gate(1),
        ];
        let mut propose_mismatches = 0u32;
        let mut fullscan_propose_us = 0u64;
        let mut bucket_propose_us = 0u64;
        for param in params {
            let t_fp = Instant::now();
            let full_prop = model.propose(&carried, &targets, param)?;
            model.synchronize()?;
            fullscan_propose_us += t_fp.elapsed().as_micros() as u64;

            let t_bp = Instant::now();
            let bucket_prop = engine.propose(&model, &csr, &carried, &targets, param)?;
            engine.synchronize()?;
            bucket_propose_us += t_bp.elapsed().as_micros() as u64;

            if full_prop.observation()? != bucket_prop.observation()?
                || full_prop.candidate_losses()? != bucket_prop.candidate_losses()?
            {
                propose_mismatches += 1;
            }
        }

        if forward_mismatches != 0 || evaluate_mismatches != 0 || propose_mismatches != 0 {
            return Err(invalid(format!(
                "device parity mismatch at V={vocabulary} K={kept_cells}: fwd={forward_mismatches} eval={evaluate_mismatches} prop={propose_mismatches}"
            )));
        }

        cases.push(DeviceParityCase {
            vocabulary,
            blocks,
            context,
            kept_cells,
            seed,
            objective_policy,
            occupied_cells,
            max_bucket_size,
            forward_rows: rows as u32,
            forward_mismatches,
            evaluate_mismatches,
            propose_cases: params.len() as u32,
            propose_mismatches,
            build_us,
            fullscan_forward_us,
            bucket_forward_us,
            fullscan_evaluate_us,
            bucket_evaluate_us,
            fullscan_propose_us,
            bucket_propose_us,
        });
    }
    Ok(cases)
}

// ---------------------------------------------------------------------------
// Control Group 2: Direct GPU Writer Seeded Sweep (V in {64, 256, 4096, 16384})
// ---------------------------------------------------------------------------

fn run_direct_seeded_sweep(engine: &BucketWriterEngine) -> Result<DirectSweepSummary> {
    let mut total_cases = 0u32;
    let mut total_rows = 0u64;
    let mut virtual_move_cases = 0u32;
    let mut cell_changing_virtual_moves = 0u32;
    let mut total_mismatches = 0u64;
    let mut scales = Vec::new();

    for vocabulary in [64u32, 256, 4096, 16384] {
        for blocks in [2u32, 4] {
            for kept_cells in [1u32, 8, 16] {
                let mut scale_cases = 0u32;
                let mut scale_rows = 0u64;
                let mut scale_mismatches = 0u64;
                let mut scale_max_bucket = 0u32;
                let mut sum_full_tc = 0u64;
                let mut sum_full_cd = 0u64;
                let mut sum_full_td = 0u64;
                let mut sum_id_tc = 0u64;
                let mut sum_id_cd = 0u64;
                let mut sum_id_td = 0u64;
                let mut max_id_td = 0u32;
                let mut sum_pruned_td = 0u64;
                let mut max_pruned_td = 0u32;
                let mut sum_build_us = 0u64;
                let mut sum_full_us = 0u64;
                let mut sum_id_us = 0u64;
                let mut sum_pruned_us = 0u64;

                for seed in [1u64, 2, 3] {
                    let mut rng = Rng::new(
                        seed * 1_000_003
                            + u64::from(vocabulary) * 97
                            + u64::from(blocks) * 13
                            + u64::from(kept_cells),
                    );
                    let palette_len = if vocabulary <= 256 {
                        2 + rng.below(15)
                    } else {
                        16
                    };
                    let palette: Vec<u32> = if palette_len == 16 {
                        (0..16).collect()
                    } else {
                        (0..palette_len).map(|_| rng.below(16) as u32).collect()
                    };
                    let table = random_valid_vocab(
                        &mut rng,
                        &palette,
                        vocabulary as usize,
                        blocks as usize,
                    );
                    let rows = 32usize;
                    let (queries, targets) = random_queries_and_targets(
                        &mut rng,
                        &palette,
                        rows,
                        blocks as usize,
                        vocabulary as usize,
                    );
                    let timed: DirectTripleTimed = engine.eval_direct_triple_timed(
                        vocabulary, blocks, kept_cells, 0, &table, &queries, &targets, 0, 0, 0,
                    )?;
                    timed.csr.validate(vocabulary)?;
                    let m_id =
                        count_direct_mismatches(&timed.full, &timed.id_order, kept_cells, true);
                    let m_pruned = count_direct_mismatches(
                        &timed.full,
                        &timed.serving_pruned,
                        kept_cells,
                        false,
                    );
                    let mismatches = u64::from(m_id + m_pruned);
                    if mismatches != 0 {
                        return Err(invalid(format!(
                            "direct sweep mismatch at V={vocabulary} B={blocks} K={kept_cells} seed={seed}: id={m_id} pruned={m_pruned}"
                        )));
                    }

                    for ((fc, ic), pc) in timed
                        .full
                        .counters
                        .iter()
                        .zip(&timed.id_order.counters)
                        .zip(&timed.serving_pruned.counters)
                    {
                        if ic.token_cell_calls != 0
                            || pc.token_cell_calls != 0
                            || ic.cell_distance_evals != 256
                            || pc.cell_distance_evals != 256
                            || ic.token_distance_evals > kept_cells * ic.max_bucket_size
                            || pc.token_distance_evals > ic.token_distance_evals
                        {
                            return Err(invalid(format!(
                                "counter invariant failed at V={vocabulary} B={blocks} K={kept_cells}: ic={ic:?} pc={pc:?}"
                            )));
                        }
                        if ic.max_bucket_size > scale_max_bucket {
                            scale_max_bucket = ic.max_bucket_size;
                        }
                        if ic.token_distance_evals > max_id_td {
                            max_id_td = ic.token_distance_evals;
                        }
                        if pc.token_distance_evals > max_pruned_td {
                            max_pruned_td = pc.token_distance_evals;
                        }
                        sum_full_tc += u64::from(fc.token_cell_calls);
                        sum_full_cd += u64::from(fc.cell_distance_evals);
                        sum_full_td += u64::from(fc.token_distance_evals);
                        sum_id_tc += u64::from(ic.token_cell_calls);
                        sum_id_cd += u64::from(ic.cell_distance_evals);
                        sum_id_td += u64::from(ic.token_distance_evals);
                        sum_pruned_td += u64::from(pc.token_distance_evals);
                    }

                    scale_cases += 1;
                    scale_rows += rows as u64;
                    scale_mismatches += mismatches;
                    sum_build_us += timed.build_us;
                    sum_full_us += timed.fullscan_us;
                    sum_id_us += timed.id_order_us;
                    sum_pruned_us += timed.serving_pruned_us;

                    total_cases += 1;
                    total_rows += rows as u64;
                    total_mismatches += mismatches;

                    if vocabulary <= 256 {
                        let (_, candidate, _) = crossing_move()?;
                        let mv_token = (seed as u32) % vocabulary;
                        let mv_block = if seed % 2 == 0 { 0 } else { blocks - 1 };
                        let mv_index = mv_token * blocks + mv_block;
                        let mv_timed = engine.eval_direct_triple_timed(
                            vocabulary,
                            blocks,
                            kept_cells,
                            0,
                            &table,
                            &queries[..8 * (blocks as usize) * 8],
                            &targets[..8],
                            1,
                            mv_index,
                            candidate,
                        )?;
                        let mv_m_id = count_direct_mismatches(
                            &mv_timed.full,
                            &mv_timed.id_order,
                            kept_cells,
                            true,
                        );
                        let mv_m_pruned = count_direct_mismatches(
                            &mv_timed.full,
                            &mv_timed.serving_pruned,
                            kept_cells,
                            false,
                        );
                        if mv_m_id != 0 || mv_m_pruned != 0 {
                            return Err(invalid(format!(
                                "virtual-move sweep mismatch at V={vocabulary} B={blocks} K={kept_cells} seed={seed}"
                            )));
                        }
                        for ic in &mv_timed.id_order.counters {
                            if ic.token_cell_calls != 1
                                || ic.cell_distance_evals != 256
                                || ic.token_distance_evals > kept_cells * ic.max_bucket_size + 1
                            {
                                return Err(invalid(
                                    "virtual-move counter invariant failed on GPU",
                                ));
                            }
                        }
                        virtual_move_cases += 1;
                        cell_changing_virtual_moves += 1;
                    }
                }

                let full_tc_row = sum_full_tc / scale_rows;
                let full_cd_row = sum_full_cd / scale_rows;
                let full_td_row = sum_full_td / scale_rows;
                let full_total_row = (sum_full_tc + sum_full_cd + sum_full_td) / scale_rows;

                let id_tc_row = sum_id_tc / scale_rows;
                let id_cd_row = sum_id_cd / scale_rows;
                let id_td_row = sum_id_td / scale_rows;
                let id_total_row = (sum_id_tc + sum_id_cd + sum_id_td) / scale_rows;
                let id_red_x100 = ((sum_full_tc + sum_full_cd + sum_full_td) * 100)
                    / (sum_id_tc + sum_id_cd + sum_id_td).max(1);
                let id_red_whole = id_red_x100 / 100;

                let pruned_td_row = sum_pruned_td / scale_rows;
                let pruned_total_row = (sum_id_tc + sum_id_cd + sum_pruned_td) / scale_rows;
                let pruned_red_x100 = ((sum_full_tc + sum_full_cd + sum_full_td) * 100)
                    / (sum_id_tc + sum_id_cd + sum_pruned_td).max(1);
                let pruned_red_whole = pruned_red_x100 / 100;

                if (vocabulary == 4096 || vocabulary == 16384) && kept_cells == 8 && blocks == 4 {
                    if id_red_x100 < MIN_REDUCTION_X100 || pruned_red_x100 < MIN_REDUCTION_X100 {
                        return Err(invalid(format!(
                            "B2 reduction bar missed at V={vocabulary} B={blocks} K={kept_cells}: id_red_x100={id_red_x100}"
                        )));
                    }
                }

                scales.push(ScaleCounterSummary {
                    vocabulary,
                    blocks,
                    kept_cells,
                    cases: scale_cases,
                    total_rows: scale_rows,
                    mismatches: scale_mismatches,
                    max_bucket_size: scale_max_bucket,
                    fullscan_token_cell_per_row: full_tc_row,
                    fullscan_cell_distance_per_row: full_cd_row,
                    fullscan_token_distance_per_row: full_td_row,
                    fullscan_total_evals_per_row: full_total_row,
                    id_order_token_cell_per_row: id_tc_row,
                    id_order_cell_distance_per_row: id_cd_row,
                    id_order_token_distance_per_row: id_td_row,
                    id_order_max_token_distance_per_row: max_id_td,
                    id_order_total_evals_per_row: id_total_row,
                    id_order_reduction_x100: id_red_x100,
                    id_order_reduction_whole: id_red_whole,
                    serving_pruned_token_distance_per_row: pruned_td_row,
                    serving_pruned_max_token_distance_per_row: max_pruned_td,
                    serving_pruned_total_evals_per_row: pruned_total_row,
                    serving_pruned_reduction_x100: pruned_red_x100,
                    serving_pruned_reduction_whole: pruned_red_whole,
                    build_us: sum_build_us,
                    fullscan_us: sum_full_us,
                    id_order_us: sum_id_us,
                    serving_pruned_us: sum_pruned_us,
                    fullscan_tokens_per_s: tokens_per_second(scale_rows, sum_full_us),
                    id_order_tokens_per_s: tokens_per_second(scale_rows, sum_id_us),
                    serving_pruned_tokens_per_s: tokens_per_second(scale_rows, sum_pruned_us),
                });
            }
        }
    }

    Ok(DirectSweepSummary {
        total_cases,
        total_rows,
        virtual_move_cases,
        cell_changing_virtual_moves,
        total_mismatches,
        scales,
    })
}

// ---------------------------------------------------------------------------
// Control Group 3: Constructed Edge Cases and 64-Step Incremental Update
// ---------------------------------------------------------------------------

fn run_edge_cases_and_incremental_update(
    ordinal: usize,
    engine: &BucketWriterEngine,
) -> Result<EdgeCaseSummary> {
    // (a) Single-cell collapse: all V=256 tokens in cell 0 (root 0, root 0).
    let mut rng = Rng::new(777);
    let collapse_tokens: Vec<Vec<[i32; 8]>> = (0..256usize)
        .map(|i| {
            let mut mid = root((i % 16) as u32);
            mid[i % 8] += if (i / 8) % 2 == 0 { 1 } else { -1 };
            vec![root(0), mid, root(0)]
        })
        .collect();
    let collapse_table = flatten_tokens(&collapse_tokens);
    let palette16: Vec<u32> = (0..16).collect();
    let (collapse_q, collapse_t) = random_queries_and_targets(&mut rng, &palette16, 16, 3, 256);
    let collapse_res = engine.eval_direct_triple_timed(
        256,
        3,
        8,
        0,
        &collapse_table,
        &collapse_q,
        &collapse_t,
        0,
        0,
        0,
    )?;
    let single_cell_collapse_mismatches =
        count_direct_mismatches(&collapse_res.full, &collapse_res.id_order, 8, true)
            + count_direct_mismatches(&collapse_res.full, &collapse_res.serving_pruned, 8, false);
    let single_cell_collapse_token_distance_evals =
        collapse_res.id_order.counters[0].token_distance_evals;
    if single_cell_collapse_mismatches != 0
        || collapse_res.id_order.counters[0].max_bucket_size != 256
        || single_cell_collapse_token_distance_evals != 256
    {
        return Err(invalid("single-cell collapse control failed on GPU"));
    }

    // (b) Uniform occupancy: |B[c]| = V / 256 at V = 4,096 (16/cell) and V = 16,384 (64/cell).
    let build_uniform_table = |v: usize, rng: &mut Rng| -> Vec<i32> {
        let mut tokens = Vec::with_capacity(v);
        for t in 0..v {
            let cell = (t % 256) as u32;
            let r_first = cell / 16;
            let r_last = cell % 16;
            let b1 = perturb(rng, root(((t / 256) % 16) as u32));
            let b2 = perturb(rng, root(((t / 16) % 16) as u32));
            tokens.push(vec![root(r_first), b1, b2, root(r_last)]);
        }
        flatten_tokens(&tokens)
    };

    let u4096_table = build_uniform_table(4096, &mut rng);
    let (u4096_q, u4096_t) = random_queries_and_targets(&mut rng, &palette16, 64, 4, 4096);
    let u4096_res = engine.eval_direct_triple_timed(
        4096,
        4,
        8,
        0,
        &u4096_table,
        &u4096_q,
        &u4096_t,
        0,
        0,
        0,
    )?;
    let uniform_v4096_mismatches =
        count_direct_mismatches(&u4096_res.full, &u4096_res.id_order, 8, true)
            + count_direct_mismatches(&u4096_res.full, &u4096_res.serving_pruned, 8, false);
    let uniform_v4096_id_order_total_evals = u4096_res.id_order.counters[0].total_evals();
    let uniform_v4096_fullscan_total_evals = u4096_res.full.counters[0].total_evals();
    let uniform_v4096_reduction_x100 =
        (uniform_v4096_fullscan_total_evals * 100) / uniform_v4096_id_order_total_evals.max(1);
    if uniform_v4096_mismatches != 0
        || u4096_res.id_order.counters[0].max_bucket_size != 16
        || u4096_res.id_order.counters[0].token_distance_evals != 128
        || uniform_v4096_id_order_total_evals != 384
    {
        return Err(invalid("uniform V=4096 control failed on GPU"));
    }

    let u16384_table = build_uniform_table(16384, &mut rng);
    let (u16384_q, u16384_t) = random_queries_and_targets(&mut rng, &palette16, 64, 4, 16384);
    let u16384_res = engine.eval_direct_triple_timed(
        16384,
        4,
        8,
        0,
        &u16384_table,
        &u16384_q,
        &u16384_t,
        0,
        0,
        0,
    )?;
    let uniform_v16384_mismatches =
        count_direct_mismatches(&u16384_res.full, &u16384_res.id_order, 8, true)
            + count_direct_mismatches(&u16384_res.full, &u16384_res.serving_pruned, 8, false);
    let uniform_v16384_id_order_total_evals = u16384_res.id_order.counters[0].total_evals();
    let uniform_v16384_fullscan_total_evals = u16384_res.full.counters[0].total_evals();
    let uniform_v16384_reduction_x100 =
        (uniform_v16384_fullscan_total_evals * 100) / uniform_v16384_id_order_total_evals.max(1);
    if uniform_v16384_mismatches != 0
        || u16384_res.id_order.counters[0].max_bucket_size != 64
        || u16384_res.id_order.counters[0].token_distance_evals != 512
        || uniform_v16384_id_order_total_evals != 768
    {
        return Err(invalid("uniform V=16384 control failed on GPU"));
    }

    // (c) Fewer than K occupied cells: 3 occupied cells with K = 8.
    let sparse_tokens = vec![
        vec![root(0), root(1)],
        vec![root(0), root(2)],
        vec![root(0), root(3)],
        vec![root(0), root(1)],
    ];
    let sparse_table = flatten_tokens(&sparse_tokens);
    let sparse_q = flatten_tokens(&[vec![root(0), root(1)]]);
    let sparse_t = vec![0u32];
    let sparse_res = engine.eval_direct_triple_timed(
        4,
        2,
        8,
        0,
        &sparse_table,
        &sparse_q,
        &sparse_t,
        0,
        0,
        0,
    )?;
    let fewer_than_k_cells_mismatches =
        count_direct_mismatches(&sparse_res.full, &sparse_res.id_order, 8, true)
            + count_direct_mismatches(&sparse_res.full, &sparse_res.serving_pruned, 8, false);
    let fewer_than_k_admitted_count = sparse_res.id_order.admitted[..16]
        .iter()
        .filter(|&&c| c != ABSENT)
        .count() as u32;
    if fewer_than_k_cells_mismatches != 0 || fewer_than_k_admitted_count != 3 {
        return Err(invalid("fewer-than-K occupied cells control failed on GPU"));
    }

    // (d) ABSENT non-root token refusal (status == 8, predictions == ABSENT).
    let absent_tokens = vec![
        vec![root(1), root(2), root(3)],
        vec![root(4), root(5), root(6)],
        vec![root(1), root(2), root(3)],
        vec![root(7), root(8), non_root()],
    ];
    let absent_table = flatten_tokens(&absent_tokens);
    let absent_q = flatten_tokens(&[vec![root(1), root(2), root(3)]]);
    let absent_t = vec![0u32];
    let (_, cross_cand, _) = crossing_move()?;
    let absent_moves = [
        (0u32, 0u32, 0u32),
        (1, 1, cross_cand),
        (1, 3, cross_cand),
        (1, 10, cross_cand),
        (1, 11, cross_cand),
    ];
    let mut absent_refusal_cases = 0u32;
    let mut absent_refusal_mismatches = 0u32;
    for (kind, index, candidate) in absent_moves {
        for walk in [WALK_ID_ORDER, WALK_SERVING_PRUNED] {
            let (full, bucket, csr_snap) = engine.eval_direct_pair(
                4,
                3,
                8,
                0,
                &absent_table,
                &absent_q,
                &absent_t,
                kind,
                index,
                candidate,
                walk,
                FAULT_NONE,
                None,
            )?;
            csr_snap.validate(4)?;
            absent_refusal_cases += 1;
            if csr_snap.absent_count != 1
                || full.status != 8
                || bucket.status != 8
                || full.predictions != vec![ABSENT]
                || bucket.predictions != vec![ABSENT]
            {
                absent_refusal_mismatches += 1;
            }
        }
    }
    if absent_refusal_mismatches != 0 {
        return Err(invalid("ABSENT non-root refusal control failed on GPU"));
    }

    // (e) Constructed ties: admission tie + cross-cell token tie + same-cell token tie.
    let (rx, ry) = equal_distance_pair(0)?;
    let tie_tokens = vec![vec![root(0), root(ry)], vec![root(0), root(rx)]];
    let tie_table = flatten_tokens(&tie_tokens);
    let tie_q = flatten_tokens(&[vec![root(0), root(0)]]);
    let tie_t = vec![0u32];
    let mut tie_cases_mismatches = 0u32;
    let tie_k1 =
        engine.eval_direct_triple_timed(2, 2, 1, 0, &tie_table, &tie_q, &tie_t, 0, 0, 0)?;
    if count_direct_mismatches(&tie_k1.full, &tie_k1.id_order, 1, true) != 0
        || count_direct_mismatches(&tie_k1.full, &tie_k1.serving_pruned, 1, false) != 0
        || tie_k1.id_order.predictions != vec![1]
        || tie_k1.serving_pruned.predictions != vec![1]
        || tie_k1.id_order.admitted[0] != rx
    {
        tie_cases_mismatches += 1;
    }
    let tie_k2 =
        engine.eval_direct_triple_timed(2, 2, 2, 0, &tie_table, &tie_q, &tie_t, 0, 0, 0)?;
    if count_direct_mismatches(&tie_k2.full, &tie_k2.id_order, 2, true) != 0
        || count_direct_mismatches(&tie_k2.full, &tie_k2.serving_pruned, 2, false) != 0
        || tie_k2.id_order.predictions != vec![0]
        || tie_k2.serving_pruned.predictions != vec![0]
        || tie_k2.id_order.admitted[0] != rx
        || tie_k2.id_order.admitted[1] != ry
    {
        tie_cases_mismatches += 1;
    }

    let same_near = vec![root(3), root(4), root(5)];
    let same_far = vec![root(9), root(10), root(11)];
    let same_other = vec![root(12), root(13), root(14)];
    let same_tokens = vec![
        same_far.clone(),
        same_far.clone(),
        same_near.clone(),
        same_far,
        same_other,
        same_near,
    ];
    let same_table = flatten_tokens(&same_tokens);
    let same_q = flatten_tokens(&[vec![root(3), root(4), root(5)]]);
    for k in [1u32, 2, 16] {
        let same_res =
            engine.eval_direct_triple_timed(6, 3, k, 0, &same_table, &same_q, &[0], 0, 0, 0)?;
        if count_direct_mismatches(&same_res.full, &same_res.id_order, k, true) != 0
            || count_direct_mismatches(&same_res.full, &same_res.serving_pruned, k, false) != 0
            || same_res.id_order.predictions != vec![2]
            || same_res.serving_pruned.predictions != vec![2]
        {
            tie_cases_mismatches += 1;
        }
    }
    if tie_cases_mismatches != 0 {
        return Err(invalid("constructed tie control failed on GPU"));
    }

    // (f) Virtual moves into and out of admitted cell.
    let (r_from, cand, r_to) = crossing_move()?;
    let r_other = (0..16)
        .find(|&o| o != r_from && o != r_to)
        .ok_or_else(|| invalid("missing third root"))?;
    let into_tokens = vec![
        vec![root(r_from), root(r_other)],
        vec![root(r_from), root(r_other)],
    ];
    let into_table = flatten_tokens(&into_tokens);
    let into_q = flatten_tokens(&[vec![root(r_to), root(r_other)]]);
    let into_res =
        engine.eval_direct_triple_timed(2, 2, 1, 0, &into_table, &into_q, &[0], 1, 2, cand)?;
    let mut virtual_move_edge_mismatches =
        count_direct_mismatches(&into_res.full, &into_res.id_order, 1, true)
            + count_direct_mismatches(&into_res.full, &into_res.serving_pruned, 1, false);
    if into_res.id_order.predictions != vec![1]
        || into_res.id_order.admitted[0] != r_to * 16 + r_other
        || into_res.id_order.counters[0].token_cell_calls != 1
    {
        virtual_move_edge_mismatches += 1;
    }

    let out_tokens = vec![
        vec![root(r_from), root(r_other)],
        vec![root(r_other), root(r_other)],
        vec![root(r_from), root(r_other)],
    ];
    let out_table = flatten_tokens(&out_tokens);
    let out_q = flatten_tokens(&[vec![root(r_from), root(r_other)]]);
    let out_res =
        engine.eval_direct_triple_timed(3, 2, 1, 0, &out_table, &out_q, &[0], 1, 0, cand)?;
    virtual_move_edge_mismatches +=
        count_direct_mismatches(&out_res.full, &out_res.id_order, 1, true)
            + count_direct_mismatches(&out_res.full, &out_res.serving_pruned, 1, false);
    if out_res.id_order.predictions != vec![2]
        || out_res.id_order.admitted[0] != r_from * 16 + r_other
    {
        virtual_move_edge_mismatches += 1;
    }

    let empty_tokens = vec![
        vec![root(r_from), root(r_other)],
        vec![root(r_other), root(r_other)],
    ];
    let empty_table = flatten_tokens(&empty_tokens);
    let empty_res =
        engine.eval_direct_triple_timed(2, 2, 1, 0, &empty_table, &out_q, &[0], 1, 0, cand)?;
    virtual_move_edge_mismatches +=
        count_direct_mismatches(&empty_res.full, &empty_res.id_order, 1, true)
            + count_direct_mismatches(&empty_res.full, &empty_res.serving_pruned, 1, false);
    if empty_res.id_order.admitted[0] == r_from * 16 + r_other {
        virtual_move_edge_mismatches += 1;
    }
    if virtual_move_edge_mismatches != 0 {
        return Err(invalid("virtual-move edge control failed on GPU"));
    }

    // (g) 64-step incremental update sequence on GPU (discrete_apply + discrete_bucket_update).
    let inc_spec = Spec {
        vocabulary: 256,
        blocks: 4,
        context: 8,
        kept_cells: 8,
    };
    let mut model = Device::new_with_initialization(
        inc_spec,
        ReadSpec::Flock { kept: 8 },
        ObjectivePolicy::NearestAddressMarginV1,
        ReadInitialization::ZeroGatesV1,
        99,
        ordinal,
    )?;
    let model_engine = BucketWriterEngine::from_device(&model)?;
    let mut csr = model_engine.build_index(&model)?;
    let (inc_q, inc_t) = random_queries_and_targets(&mut rng, &palette16, 16, 4, 256);

    let mut windows = Vec::with_capacity(8);
    let mut batch_targets = Vec::with_capacity(8);
    for r in 0..8usize {
        let win = (0..8usize)
            .map(|pos| Identity {
                token: ((r * 7 + pos * 3 + 1) % 256) as u32,
                occurrence: (r * 8 + pos + 1) as u64,
                record: 1,
                version: 0,
            })
            .collect::<Vec<_>>();
        windows.push(win);
        batch_targets.push((r % 16) as u32);
    }

    let incremental_update_steps = 64u32;
    let incremental_update_bound = 2 * inc_spec.vocabulary + 258;
    let mut incremental_update_accepted_commits = 0u32;
    let mut incremental_update_cell_changes = 0u32;
    let mut incremental_update_csr_mismatches = 0u32;
    let mut incremental_update_writer_mismatches = 0u32;
    let mut incremental_update_max_entries_written = 0u32;
    let mut incremental_update_total_us = 0u64;

    for step in 0..incremental_update_steps {
        let token = batch_targets[(step as usize) % batch_targets.len()];
        let block = if step % 2 == 0 {
            0
        } else {
            inc_spec.blocks - 1
        };
        let param = Parameter::VectorBlock(token * inc_spec.blocks + block);
        let carried = model.carry_windows(&windows)?;
        let prop = model_engine.propose(&model, &csr, &carried, &batch_targets, param)?;
        let pre_snap = csr.snapshot(model_engine.stream())?;
        let pre_cell = pre_snap.cells[token as usize];

        let committed = model_engine.commit_proposal_in_qualification(&mut model, &prop)?;
        if committed {
            incremental_update_accepted_commits += 1;
        }
        model.synchronize()?;
        let t_up = Instant::now();
        let written = model_engine.update_index(&model, &mut csr, token)?;
        model_engine.synchronize()?;
        incremental_update_total_us += t_up.elapsed().as_micros() as u64;

        if written > incremental_update_max_entries_written {
            incremental_update_max_entries_written = written;
        }
        if written > incremental_update_bound {
            incremental_update_csr_mismatches += 1;
        }

        let updated_snap = csr.snapshot(model_engine.stream())?;
        updated_snap.validate(inc_spec.vocabulary)?;
        if updated_snap.cells[token as usize] != pre_cell {
            incremental_update_cell_changes += 1;
        }

        let fresh_csr = model_engine.build_index(&model)?;
        let fresh_snap = fresh_csr.snapshot(model_engine.stream())?;
        if updated_snap != fresh_snap {
            incremental_update_csr_mismatches += 1;
        }

        let step_eval = model_engine.eval_model_queries(&model, &csr, &inc_q, &inc_t)?;
        incremental_update_writer_mismatches +=
            count_direct_mismatches(&step_eval.full, &step_eval.id_order, 8, true)
                + count_direct_mismatches(&step_eval.full, &step_eval.serving_pruned, 8, false);
    }

    if incremental_update_csr_mismatches != 0
        || incremental_update_writer_mismatches != 0
        || incremental_update_accepted_commits == 0
    {
        return Err(invalid(format!(
            "incremental 64-step update control failed: csr_mismatches={incremental_update_csr_mismatches} writer_mismatches={incremental_update_writer_mismatches} accepted={incremental_update_accepted_commits}"
        )));
    }

    Ok(EdgeCaseSummary {
        single_cell_collapse_mismatches,
        single_cell_collapse_token_distance_evals,
        uniform_v4096_mismatches,
        uniform_v4096_id_order_total_evals,
        uniform_v4096_fullscan_total_evals,
        uniform_v4096_reduction_x100,
        uniform_v16384_mismatches,
        uniform_v16384_id_order_total_evals,
        uniform_v16384_fullscan_total_evals,
        uniform_v16384_reduction_x100,
        fewer_than_k_cells_mismatches,
        fewer_than_k_admitted_count,
        absent_refusal_cases,
        absent_refusal_mismatches,
        tie_cases_mismatches,
        virtual_move_edge_mismatches,
        incremental_update_steps,
        incremental_update_accepted_commits,
        incremental_update_cell_changes,
        incremental_update_csr_mismatches,
        incremental_update_writer_mismatches,
        incremental_update_max_entries_written,
        incremental_update_bound,
        incremental_update_total_us,
    })
}

// ---------------------------------------------------------------------------
// Control Group 4: Real-Read 14,287-Row DEV Panel on Pinned A8 and PrimeZeta
// ---------------------------------------------------------------------------

fn load_pinned_codebook(
    path: &Path,
    expected_file_sha256: &str,
    expected_table_sha256: Option<&str>,
) -> Result<(String, CodebookTable)> {
    let bytes =
        std::fs::read(path).map_err(|e| invalid(format!("read {}: {e}", path.display())))?;
    let file_sha = sha256(&bytes);
    if file_sha != expected_file_sha256 {
        return Err(invalid(format!(
            "codebook {} file sha256 {file_sha} != expected {expected_file_sha256}",
            path.display()
        )));
    }
    let table: CodebookTable = serde_json::from_slice(&bytes)
        .map_err(|e| invalid(format!("parse {}: {e}", path.display())))?;
    if table.schema != CODEBOOK_TABLE_SCHEMA {
        return Err(invalid(format!(
            "codebook {} schema {} != {CODEBOOK_TABLE_SCHEMA}",
            path.display(),
            table.schema
        )));
    }
    let computed_table_sha = CodebookTable::words_sha256(&table.table);
    if computed_table_sha != table.table_sha256 {
        return Err(invalid("codebook table_sha256 disagrees with words_sha256"));
    }
    if let Some(exp_tab) = expected_table_sha256 {
        if table.table_sha256 != exp_tab {
            return Err(invalid(format!(
                "codebook table_sha256 {} != expected {exp_tab}",
                table.table_sha256
            )));
        }
    }
    Ok((file_sha, table))
}

fn run_real_read_codebook_panel(
    ordinal: usize,
    name: &str,
    codebook_path: &Path,
    expected_file_sha256: &str,
    expected_table_sha256: Option<&str>,
    expected_top1: u64,
    corpus: &PreparedCorpus,
) -> Result<RealReadCodebookSummary> {
    let (file_sha256, codebook) =
        load_pinned_codebook(codebook_path, expected_file_sha256, expected_table_sha256)?;
    let spec = codebook.spec;
    let seed_model = Device::new_with_initialization(
        spec,
        ReadSpec::Relation { kept: 8, sweeps: 1 },
        ObjectivePolicy::NearestAddressMarginV1,
        ReadInitialization::ActiveLeafV1,
        1,
        ordinal,
    )?;
    let mut snapshot = seed_model.snapshot()?;
    snapshot.table = codebook.table.clone();
    let model = Device::from_snapshot(snapshot, ordinal)?;
    let engine = BucketWriterEngine::from_device(&model)?;
    model.synchronize()?;

    let t_build = Instant::now();
    let csr = engine.build_index(&model)?;
    engine.synchronize()?;
    let build_us = t_build.elapsed().as_micros() as u64;

    let csr_snap = csr.snapshot(engine.stream())?;
    csr_snap.validate(spec.vocabulary)?;
    if csr_snap.absent_count != 0 {
        return Err(invalid(format!("{name} has non-zero absent_count")));
    }

    let mut occupied_cells = 0u32;
    let mut min_nonempty_bucket_size = u32::MAX;
    let mut max_bucket_size = 0u32;
    for c in 0..256usize {
        let count = csr_snap.offsets[c + 1] - csr_snap.offsets[c];
        if count > 0 {
            occupied_cells += 1;
            if count < min_nonempty_bucket_size {
                min_nonempty_bucket_size = count;
            }
            if count > max_bucket_size {
                max_bucket_size = count;
            }
        }
    }

    let total_eligible = corpus.eligible_len() as u64;
    if total_eligible != DEV_ELIGIBLE_ROWS {
        return Err(invalid("DEV eligible target count mismatch"));
    }

    let mut cursor = 0u64;
    let mut batches = 0u32;
    let mut scored_rows = 0u64;
    let mut top1_correct = 0u64;
    let mut top8_correct = 0u64;
    let mut forward_mismatches = 0u64;
    let mut evaluate_mismatches = 0u64;
    let mut direct_id_order_mismatches = 0u64;
    let mut direct_serving_pruned_mismatches = 0u64;

    let mut sum_full_tc = 0u64;
    let mut sum_full_cd = 0u64;
    let mut sum_full_td = 0u64;
    let mut sum_id_tc = 0u64;
    let mut sum_id_cd = 0u64;
    let mut sum_id_td = 0u64;
    let mut max_id_td = 0u32;
    let mut sum_pruned_td = 0u64;
    let mut max_pruned_td = 0u32;

    let mut fullscan_forward_us = 0u64;
    let mut bucket_forward_us = 0u64;
    let mut fullscan_evaluate_us = 0u64;
    let mut bucket_evaluate_us = 0u64;
    let mut direct_fullscan_us = 0u64;
    let mut direct_id_order_us = 0u64;
    let mut direct_serving_pruned_us = 0u64;

    while cursor < total_eligible {
        let batch_rows = (total_eligible - cursor).min(64) as u32;
        let batch = corpus.batch(cursor, spec.context, batch_rows)?;
        let carried = model.carry_windows(&batch.windows)?;
        model.synchronize()?;

        let t_ff = Instant::now();
        let full_fwd = model.forward(&carried, &batch.targets)?;
        model.synchronize()?;
        fullscan_forward_us += t_ff.elapsed().as_micros() as u64;

        let t_bf = Instant::now();
        let bucket_fwd = engine.forward(&model, &csr, &carried, &batch.targets)?;
        engine.synchronize()?;
        bucket_forward_us += t_bf.elapsed().as_micros() as u64;

        let q_vectors = full_fwd.vectors()?;
        if full_fwd.predictions != bucket_fwd.predictions
            || full_fwd.losses != bucket_fwd.losses
            || full_fwd.token_rivals != bucket_fwd.token_rivals
            || full_fwd.address_rivals != bucket_fwd.address_rivals
            || full_fwd.admitted_cells != bucket_fwd.admitted_cells
            || q_vectors != bucket_fwd.vectors()?
        {
            forward_mismatches += 1;
        }

        let t_fe = Instant::now();
        let full_eval = model.evaluate(&carried, &batch.targets)?;
        model.synchronize()?;
        fullscan_evaluate_us += t_fe.elapsed().as_micros() as u64;

        let t_be = Instant::now();
        let bucket_eval = engine.evaluate(&model, &csr, &carried, &batch.targets)?;
        engine.synchronize()?;
        bucket_evaluate_us += t_be.elapsed().as_micros() as u64;

        let full_agg =
            serde_json::to_vec(&full_eval.aggregate).map_err(|e| invalid(e.to_string()))?;
        let bucket_agg =
            serde_json::to_vec(&bucket_eval.aggregate).map_err(|e| invalid(e.to_string()))?;
        if full_agg != bucket_agg || full_eval.rows.len() != bucket_eval.rows.len() {
            evaluate_mismatches += 1;
        }
        for (a, b) in full_eval.rows.iter().zip(&bucket_eval.rows) {
            if a.top8 != b.top8
                || a.token_hinge != b.token_hinge
                || a.address_hinge != b.address_hinge
                || a.cell_rank_hinge != b.cell_rank_hinge
                || a.target_cell_rank != b.target_cell_rank
                || a.occupied_cells != b.occupied_cells
                || a.admitted_token_count != b.admitted_token_count
                || a.loss_ta_numerator != b.loss_ta_numerator
                || a.loss_tr_numerator != b.loss_tr_numerator
                || a.objective_loss_numerator != b.objective_loss_numerator
                || a.total_loss_numerator != b.total_loss_numerator
                || a.token_rivals != b.token_rivals
                || a.address_rivals != b.address_rivals
                || a.top1_correct != b.top1_correct
                || a.top8_correct != b.top8_correct
                || a.admitted_cells != b.admitted_cells
            {
                evaluate_mismatches += 1;
            }
            top1_correct += a.top1_correct;
            top8_correct += a.top8_correct;
        }

        let direct = engine.eval_model_queries(&model, &csr, &q_vectors, &batch.targets)?;
        direct_fullscan_us += direct.fullscan_us;
        direct_id_order_us += direct.id_order_us;
        direct_serving_pruned_us += direct.serving_pruned_us;

        direct_id_order_mismatches += u64::from(count_direct_mismatches(
            &direct.full,
            &direct.id_order,
            spec.kept_cells,
            true,
        ));
        direct_serving_pruned_mismatches += u64::from(count_direct_mismatches(
            &direct.full,
            &direct.serving_pruned,
            spec.kept_cells,
            false,
        ));

        for ((fc, ic), pc) in direct
            .full
            .counters
            .iter()
            .zip(&direct.id_order.counters)
            .zip(&direct.serving_pruned.counters)
        {
            if ic.token_cell_calls != 0
                || pc.token_cell_calls != 0
                || ic.cell_distance_evals != 256
                || pc.cell_distance_evals != 256
                || ic.token_distance_evals > spec.kept_cells * ic.max_bucket_size
                || pc.token_distance_evals > ic.token_distance_evals
            {
                return Err(invalid(format!("{name} DEV row counter invariant failed")));
            }
            if ic.token_distance_evals > max_id_td {
                max_id_td = ic.token_distance_evals;
            }
            if pc.token_distance_evals > max_pruned_td {
                max_pruned_td = pc.token_distance_evals;
            }
            sum_full_tc += u64::from(fc.token_cell_calls);
            sum_full_cd += u64::from(fc.cell_distance_evals);
            sum_full_td += u64::from(fc.token_distance_evals);
            sum_id_tc += u64::from(ic.token_cell_calls);
            sum_id_cd += u64::from(ic.cell_distance_evals);
            sum_id_td += u64::from(ic.token_distance_evals);
            sum_pruned_td += u64::from(pc.token_distance_evals);
        }

        cursor = batch.next_target;
        batches += 1;
        scored_rows += u64::from(batch_rows);
    }

    if scored_rows != DEV_ELIGIBLE_ROWS
        || top1_correct != expected_top1
        || forward_mismatches != 0
        || evaluate_mismatches != 0
        || direct_id_order_mismatches != 0
        || direct_serving_pruned_mismatches != 0
    {
        return Err(invalid(format!(
            "{name} DEV panel verification failed: rows={scored_rows} top1={top1_correct} (expected {expected_top1}) fwd_err={forward_mismatches} eval_err={evaluate_mismatches} id_err={direct_id_order_mismatches} pruned_err={direct_serving_pruned_mismatches}"
        )));
    }

    let full_total = sum_full_tc + sum_full_cd + sum_full_td;
    let id_total = sum_id_tc + sum_id_cd + sum_id_td;
    let pruned_total = sum_id_tc + sum_id_cd + sum_pruned_td;
    let id_red_x100 = (full_total * 100) / id_total.max(1);
    let pruned_red_x100 = (full_total * 100) / pruned_total.max(1);

    if id_red_x100 < MIN_REDUCTION_X100 || pruned_red_x100 < MIN_REDUCTION_X100 {
        return Err(invalid(format!(
            "{name} DEV panel missed 10x reduction bar: id_red_x100={id_red_x100}"
        )));
    }

    Ok(RealReadCodebookSummary {
        name: name.to_owned(),
        file_sha256,
        table_sha256: codebook.table_sha256,
        vocabulary: spec.vocabulary,
        blocks: spec.blocks,
        context: spec.context,
        kept_cells: spec.kept_cells,
        occupied_cells,
        min_nonempty_bucket_size,
        max_bucket_size,
        scored_rows,
        batches,
        top1_correct,
        top8_correct,
        forward_mismatches,
        evaluate_mismatches,
        direct_id_order_mismatches,
        direct_serving_pruned_mismatches,
        fullscan_token_cell_per_row: sum_full_tc / scored_rows,
        fullscan_cell_distance_per_row: sum_full_cd / scored_rows,
        fullscan_token_distance_per_row: sum_full_td / scored_rows,
        fullscan_total_evals_per_row: full_total / scored_rows,
        id_order_token_cell_per_row: sum_id_tc / scored_rows,
        id_order_cell_distance_per_row: sum_id_cd / scored_rows,
        id_order_token_distance_per_row: sum_id_td / scored_rows,
        id_order_max_token_distance_per_row: max_id_td,
        id_order_total_evals_per_row: id_total / scored_rows,
        id_order_reduction_x100: id_red_x100,
        id_order_reduction_whole: id_red_x100 / 100,
        serving_pruned_token_distance_per_row: sum_pruned_td / scored_rows,
        serving_pruned_max_token_distance_per_row: max_pruned_td,
        serving_pruned_total_evals_per_row: pruned_total / scored_rows,
        serving_pruned_reduction_x100: pruned_red_x100,
        serving_pruned_reduction_whole: pruned_red_x100 / 100,
        build_us,
        fullscan_forward_us,
        bucket_forward_us,
        fullscan_evaluate_us,
        bucket_evaluate_us,
        direct_fullscan_us,
        direct_id_order_us,
        direct_serving_pruned_us,
        fullscan_forward_tokens_per_s: tokens_per_second(scored_rows, fullscan_forward_us),
        bucket_forward_tokens_per_s: tokens_per_second(scored_rows, bucket_forward_us),
        fullscan_evaluate_tokens_per_s: tokens_per_second(scored_rows, fullscan_evaluate_us),
        bucket_evaluate_tokens_per_s: tokens_per_second(scored_rows, bucket_evaluate_us),
        direct_fullscan_tokens_per_s: tokens_per_second(scored_rows, direct_fullscan_us),
        direct_id_order_tokens_per_s: tokens_per_second(scored_rows, direct_id_order_us),
        direct_serving_pruned_tokens_per_s: tokens_per_second(
            scored_rows,
            direct_serving_pruned_us,
        ),
    })
}

// ---------------------------------------------------------------------------
// Control Group 5: Negative Controls / Four Planted Faults
// ---------------------------------------------------------------------------

fn run_planted_fault_controls(engine: &BucketWriterEngine) -> Result<Vec<PlantedFaultCase>> {
    let mut faults = Vec::with_capacity(4);

    // Fault 1: FAULT_STALE_BUCKET (virtual move changes cell, but bucket occupancy ignores move).
    let (r_from, cand, r_to) = crossing_move()?;
    let r_other = (0..16)
        .find(|&o| o != r_from && o != r_to)
        .ok_or_else(|| invalid("missing third root"))?;
    let f1_tokens = vec![
        vec![root(r_from), root(r_other)],
        vec![root(r_from), root(r_other)],
    ];
    let f1_table = flatten_tokens(&f1_tokens);
    let f1_q = flatten_tokens(&[vec![root(r_to), root(r_other)]]);
    let (f1_full, f1_bucket, _) = engine.eval_direct_pair(
        2,
        2,
        1,
        0,
        &f1_table,
        &f1_q,
        &[0],
        1,
        2,
        cand,
        WALK_ID_ORDER,
        FAULT_STALE_BUCKET,
        None,
    )?;
    let f1_mismatches = count_direct_mismatches(&f1_full, &f1_bucket, 1, true);
    faults.push(PlantedFaultCase {
        fault_mode: FAULT_STALE_BUCKET,
        fault_name: "stale_bucket_after_move".to_owned(),
        rows: 1,
        mismatches_detected: f1_mismatches,
        caught: f1_mismatches > 0,
    });

    // Fault 2: FAULT_REVERSED_TIE (equal-distance cell and token ties prefer larger ID instead of smaller).
    let (rx, ry) = equal_distance_pair(0)?;
    let f2_tokens = vec![vec![root(0), root(ry)], vec![root(0), root(rx)]];
    let f2_table = flatten_tokens(&f2_tokens);
    let f2_q = flatten_tokens(&[vec![root(0), root(0)]]);
    let (f2_full, f2_bucket, _) = engine.eval_direct_pair(
        2,
        2,
        1,
        0,
        &f2_table,
        &f2_q,
        &[0],
        0,
        0,
        0,
        WALK_ID_ORDER,
        FAULT_REVERSED_TIE,
        None,
    )?;
    let f2_mismatches = count_direct_mismatches(&f2_full, &f2_bucket, 1, true);
    faults.push(PlantedFaultCase {
        fault_mode: FAULT_REVERSED_TIE,
        fault_name: "reversed_tie_break".to_owned(),
        rows: 1,
        mismatches_detected: f2_mismatches,
        caught: f2_mismatches > 0,
    });

    // Fault 3: FAULT_OVER_PRUNED (ServingPruned skips when cell_dist >= best_distance instead of >).
    // Token 1 is in cell rx (admitted first) and Token 0 is in cell ry (admitted second) at equal
    // address_distance and equal token_distance from f3_q. Under exact pruning (>), cell ry is searched
    // and Token 0 wins the tie (0 < 1). Under FAULT_OVER_PRUNED (>=), cell ry is skipped because
    // dist_cell[ry] >= best_distance, leaving Token 1!
    let f3_tokens = vec![
        vec![root(0), root(0), root(ry)],
        vec![root(0), root(0), root(rx)],
    ];
    let f3_table = flatten_tokens(&f3_tokens);
    let f3_q = flatten_tokens(&[vec![root(0), root(0), root(0)]]);
    let (f3_full, f3_bucket, _) = engine.eval_direct_pair(
        2,
        3,
        2,
        0,
        &f3_table,
        &f3_q,
        &[0],
        0,
        0,
        0,
        WALK_SERVING_PRUNED,
        FAULT_OVER_PRUNED,
        None,
    )?;
    let f3_mismatches = count_direct_mismatches(&f3_full, &f3_bucket, 2, false);
    faults.push(PlantedFaultCase {
        fault_mode: FAULT_OVER_PRUNED,
        fault_name: "over_aggressive_serving_prune".to_owned(),
        rows: 1,
        mismatches_detected: f3_mismatches,
        caught: f3_mismatches > 0,
    });

    // Fault 4: FAULT_CORRUPTED_CSR (offsets[c + 1] - 1 drops the last member of each bucket).
    let f4_tokens = vec![vec![root(2), root(3)], vec![root(0), root(1)]];
    let f4_table = flatten_tokens(&f4_tokens);
    let f4_q = flatten_tokens(&[vec![root(0), root(1)]]);
    let (f4_full, f4_bucket, _) = engine.eval_direct_pair(
        2,
        2,
        1,
        0,
        &f4_table,
        &f4_q,
        &[1],
        0,
        0,
        0,
        WALK_ID_ORDER,
        FAULT_CORRUPTED_CSR,
        None,
    )?;
    let f4_mismatches = count_direct_mismatches(&f4_full, &f4_bucket, 1, true);
    faults.push(PlantedFaultCase {
        fault_mode: FAULT_CORRUPTED_CSR,
        fault_name: "corrupted_csr_offset".to_owned(),
        rows: 1,
        mismatches_detected: f4_mismatches,
        caught: f4_mismatches > 0,
    });

    if faults.iter().any(|f| !f.caught) {
        return Err(invalid(format!(
            "planted fault control failed to catch a planted fault: {faults:?}"
        )));
    }
    Ok(faults)
}

/// Execute the complete finite CUDA qualification suite for `m1-faster-writer` (#2840).
pub fn run_qualification(
    ordinal: usize,
    dev_tokens_path: &Path,
    dev_mask_path: &Path,
    a8_table_path: &Path,
    pz_table_path: &Path,
) -> Result<BucketQualificationReport> {
    bucket_extends_ordinary_source()?;
    let engine = BucketWriterEngine::new(ordinal)?;

    let source = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: dev_tokens_path.to_path_buf(),
            sha256: DEV_TOKENS_SHA256.to_owned(),
        },
        mask: FilePin {
            path: dev_mask_path.to_path_buf(),
            sha256: DEV_MASK_SHA256.to_owned(),
        },
        expected_payload_tokens: DEV_PAYLOAD_TOKENS,
        expected_eligible_targets: DEV_ELIGIBLE_ROWS,
    };
    let bos = BosIdentity {
        token: "<|bos|>".to_owned(),
        id: 0,
    };
    let corpus = PreparedCorpus::load(&source, 4096, &bos)?;

    let device_parity_cases = run_device_parity_controls(ordinal)?;
    let direct_seeded_sweep = run_direct_seeded_sweep(&engine)?;
    let edge_cases = run_edge_cases_and_incremental_update(ordinal, &engine)?;
    let a8_summary = run_real_read_codebook_panel(
        ordinal,
        "A8_address_structured_v1",
        a8_table_path,
        A8_FILE_SHA256,
        Some(A8_TABLE_SHA256),
        A8_EXPECTED_TOP1,
        &corpus,
    )?;
    let pz_summary = run_real_read_codebook_panel(
        ordinal,
        "PrimeZeta_v4096_b4_c192_k8",
        pz_table_path,
        PZ_FILE_SHA256,
        None,
        PZ_EXPECTED_TOP1,
        &corpus,
    )?;
    let real_read_dev_panel = vec![a8_summary, pz_summary];
    let planted_faults = run_planted_fault_controls(&engine)?;

    let b1_parity_pass = device_parity_cases.iter().all(|c| {
        c.forward_mismatches == 0 && c.evaluate_mismatches == 0 && c.propose_mismatches == 0
    }) && direct_seeded_sweep.total_mismatches == 0
        && edge_cases.single_cell_collapse_mismatches == 0
        && edge_cases.uniform_v4096_mismatches == 0
        && edge_cases.uniform_v16384_mismatches == 0
        && edge_cases.fewer_than_k_cells_mismatches == 0
        && edge_cases.absent_refusal_mismatches == 0
        && edge_cases.tie_cases_mismatches == 0
        && edge_cases.virtual_move_edge_mismatches == 0
        && edge_cases.incremental_update_csr_mismatches == 0
        && edge_cases.incremental_update_writer_mismatches == 0
        && real_read_dev_panel.iter().all(|p| {
            p.forward_mismatches == 0
                && p.evaluate_mismatches == 0
                && p.direct_id_order_mismatches == 0
                && p.direct_serving_pruned_mismatches == 0
        });

    let b2_comparison_reduction_pass = edge_cases.uniform_v4096_reduction_x100
        >= MIN_REDUCTION_X100
        && edge_cases.uniform_v16384_reduction_x100 >= MIN_REDUCTION_X100
        && real_read_dev_panel.iter().all(|p| {
            p.id_order_token_cell_per_row == 0
                && p.id_order_cell_distance_per_row == 256
                && p.id_order_reduction_x100 >= MIN_REDUCTION_X100
                && p.serving_pruned_reduction_x100 >= MIN_REDUCTION_X100
        });

    let negative_controls_pass =
        planted_faults.len() == 4 && planted_faults.iter().all(|f| f.caught);

    if !b1_parity_pass || !b2_comparison_reduction_pass || !negative_controls_pass {
        return Err(invalid(
            "qualification bars not cleared despite passing individual stage checks",
        ));
    }

    Ok(BucketQualificationReport {
        schema: QUALIFICATION_SCHEMA.to_owned(),
        status: "PASSED".to_owned(),
        b1_parity_pass,
        b2_comparison_reduction_pass,
        negative_controls_pass,
        driver_source_sha256: driver_source_sha256(),
        compiled_identity: compiled_identity(),
        bucket_image: compiled_bucket_image_sha256(),
        dev_tokens_sha256: DEV_TOKENS_SHA256.to_owned(),
        dev_mask_sha256: DEV_MASK_SHA256.to_owned(),
        device_parity_cases,
        direct_seeded_sweep,
        edge_cases,
        real_read_dev_panel,
        planted_faults,
    })
}
