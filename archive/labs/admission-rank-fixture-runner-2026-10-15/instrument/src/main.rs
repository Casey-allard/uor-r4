#[cfg(feature = "cuda")]
mod cuda_aggregate;
mod fixtures;

use fixtures::{build_suite_plan, run_host_checks, sha256_hex};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use uor_r4_discrete::corpus::FilePin;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlReceipt {
    pub id: String,
    pub status: String,
    pub receipt: FilePin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteExecutionSummary {
    pub schema: String,
    pub status: String,
    pub model_compute_executed: bool,
    pub apply_calls: u64,
    pub cpu_model_reference_used: bool,
    pub controls: Vec<ControlReceipt>,
}

/// The METRICS object (the aggregate) of a receipt, in either producer shape.
///
/// The producer puts the aggregate metrics AND the zero-update flags only under `aggregate`: at the
/// top level for a raw driver receipt, under `evaluation.aggregate` for a completion envelope
/// (completion.rs:580-581). Reading them one level high is the blocker this repairs.
///
/// Completion reports wrap a driver evaluation under `evaluation` by design. Raw eval-corpus
/// receipts emit it at the TOP LEVEL (aggregate, objective_evaluation_schema, ...). The runner read
/// only the wrapped shape, so on a raw receipt the lookup produced null and the K8/K16 guards failed
/// for a reason unrelated to the measured counts (Codex, #2029 6207186927).
///
/// Both shapes are named explicitly rather than probed for anything object-shaped, and a receipt that
/// is neither is REFUSED: a missing key must not be able to read as a failed bar.
pub fn driver_evaluation<'a>(
    val: &'a serde_json::Value,
    receipt: &str,
) -> Result<&'a serde_json::Value, String> {
    const DRIVER: &str = "uor-r4.discrete-corpus-driver/1";
    const OBJECTIVE: &str = "uor-r4.discrete-objective-evaluation/1";
    const COMPLETION: &str = "uor-r4.completion-evaluation/1";
    let schema = val.get("schema").and_then(|v| v.as_str());
    if schema == Some(COMPLETION) {
        let driver = val
            .get("evaluation")
            .filter(|e| e.is_object())
            .ok_or_else(|| format!("{receipt}: completion envelope has no evaluation object"))?;
        let nested = driver.get("schema").and_then(|v| v.as_str());
        if nested != Some(DRIVER) {
            return Err(format!(
                "{receipt}: nested evaluation schema is {nested:?}, expected {DRIVER}"
            ));
        }
        let nested_objective = driver
            .get("objective_evaluation_schema")
            .and_then(|v| v.as_str());
        if nested_objective != Some(OBJECTIVE) {
            return Err(format!(
                "{receipt}: nested objective_evaluation_schema is {nested_objective:?}, expected {OBJECTIVE}"
            ));
        }
        return driver
            .get("aggregate")
            .filter(|agg| agg.is_object())
            .ok_or_else(|| {
                format!("{receipt}: completion envelope has no evaluation.aggregate object")
            });
    }
    if schema == Some(DRIVER) {
        let objective = val
            .get("objective_evaluation_schema")
            .and_then(|v| v.as_str());
        if objective != Some(OBJECTIVE) {
            return Err(format!(
                "{receipt}: objective_evaluation_schema is {objective:?}, expected {OBJECTIVE}"
            ));
        }
        return val
            .get("aggregate")
            .filter(|agg| agg.is_object())
            .ok_or_else(|| format!("{receipt}: driver receipt has no aggregate object"));
    }
    Err(format!(
        "{receipt}: unrecognized receipt schema {schema:?}; expected {DRIVER} or {COMPLETION}"
    ))
}
/// The object carrying the zero-update flags, in either producer shape.
///
/// The flags are SIBLINGS of the aggregate, not inside it: at the receipt root for a raw driver
/// receipt, and under `evaluation` for a completion envelope. Reading them from the metrics object
/// rejected genuine zero-update reports, which is what Codex caught in pass 2 (6207702446).
pub fn driver_zero_update<'a>(
    val: &'a serde_json::Value,
    receipt: &str,
) -> Result<&'a serde_json::Value, String> {
    const COMPLETION: &str = "uor-r4.completion-evaluation/1";
    if val.get("schema").and_then(|v| v.as_str()) == Some(COMPLETION) {
        return val
            .get("evaluation")
            .filter(|e| e.is_object())
            .ok_or_else(|| format!("{receipt}: completion envelope has no evaluation object"));
    }
    Ok(val)
}

pub fn write_new_json<T: Serialize>(path: &Path, value: &T) -> Result<FilePin, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("failed to create directory {}: {e}", parent.display()))?;
    }
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("serialize JSON: {e}"))?;
    bytes.push(b'\n');
    let sha256 = sha256_hex(&bytes);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| {
            format!(
                "refusing to overwrite or failed to create {}: {e}",
                path.display()
            )
        })?;
    file.write_all(&bytes)
        .map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    file.sync_all()
        .map_err(|e| format!("failed to sync {}: {e}", path.display()))?;
    Ok(FilePin {
        path: path.to_path_buf(),
        sha256,
    })
}

fn default_repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../..")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("."))
}

use uor_r4_discrete::objective::{ExactRatio, HingeDistribution};

pub fn verify_hinge_distribution(
    name: &str,
    dist: &HingeDistribution,
    expected_bins: usize,
    expected_sum: u64,
    scored_rows: u64,
) -> Result<(), String> {
    if dist.histogram.len() != expected_bins {
        return Err(format!(
            "C-R5 5A {name} histogram bin inventory {} != {expected_bins}",
            dist.histogram.len()
        ));
    }
    let bin_sum: u64 = dist.histogram.iter().copied().sum();
    if bin_sum != scored_rows {
        return Err(format!(
            "C-R5 5A {name} sum(bins)={bin_sum} != scored_rows={scored_rows}"
        ));
    }
    if dist.zero_rows != dist.histogram[0] {
        return Err(format!(
            "C-R5 5A {name} zero_rows={} != bins[0]={}",
            dist.zero_rows, dist.histogram[0]
        ));
    }
    if dist.mean
        != (ExactRatio {
            numerator: expected_sum,
            denominator: scored_rows,
        })
    {
        return Err(format!(
            "C-R5 5A {name} mean {:?} != {expected_sum}/{scored_rows}",
            dist.mean
        ));
    }
    let actual_quantiles = [
        dist.min, dist.p25, dist.p50, dist.p75, dist.p90, dist.p99, dist.max,
    ];
    for (idx, percent) in [0u64, 25, 50, 75, 90, 99, 100].into_iter().enumerate() {
        let threshold = if percent == 0 {
            1
        } else {
            (scored_rows * percent).div_ceil(100)
        };
        let mut cumulative = 0u64;
        let mut expected_bin = 0u64;
        for (value, &count) in dist.histogram.iter().enumerate() {
            cumulative += count;
            if cumulative >= threshold {
                expected_bin = value as u64;
                break;
            }
        }
        if actual_quantiles[idx] != expected_bin {
            return Err(format!(
                "C-R5 5A {name} nearest-rank quantile p{percent}={} != expected {expected_bin}",
                actual_quantiles[idx]
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "cuda")]
mod gpu_runner {
    use super::*;
    use crate::cuda_aggregate::{identity_field, run_status12_overflow_checks};
    use crate::driver_evaluation;
    use crate::driver_zero_update;
    use crate::fixtures::{
        c_r1_all_256_cells_table, c_r1_equal_distance_routes, c_r1_equal_distance_tie_table,
        c_r1_mixed_distance_table, c_r2_crowded_rival_cell_table, c_r2_new_preceding_cell_table,
        c_r3_empty_cell_base_table, c_r4_absent_rival_table, c_r4_single_occupied_cell_table,
        c_r4_sixteen_occupied_cells_table, candidate_root_image, one_token_window,
        three_token_window, COMPOSITION_KAPPA, CONTROL_REPORT_SCHEMA, EXPECTED_C_R1_ADMITTED_BY_K,
        EXPECTED_C_R1_HINGE_BY_K, G_PRE_K16_MINIMUM, G_PRE_K8_SEED1_EXPECTED_ADDRESS_HINGE_SUM,
        G_PRE_K8_SEED1_EXPECTED_ADMITTED, G_PRE_K8_SEED1_EXPECTED_STRICT_WINS,
        G_PRE_K8_SEED1_EXPECTED_TOP1, G_PRE_K8_SEED1_EXPECTED_TOP8, OPERATOR_KAPPA, ROOTS,
        RUNNER_SCHEMA,
    };
    use uor_r4_discrete::{
        device::{compiled_identity, Device, Evaluation, Snapshot},
        objective::{AGGREGATE_WORDS, EVALUATION_SCHEMA, HISTOGRAM_BINS, ROW_WORDS},
        Identity, ObjectivePolicy, Parameter, ReadSpec, Spec, SCHEMA,
    };

    fn make_route_snapshot(
        spec: Spec,
        objective_policy: ObjectivePolicy,
        table: Vec<i32>,
        routes: Vec<u32>,
    ) -> Result<Snapshot, String> {
        let compiled = compiled_identity();
        let stages = routes.len() / (spec.blocks as usize);
        Ok(Snapshot {
            schema: SCHEMA.into(),
            spec,
            read_spec: ReadSpec::Route,
            objective_policy,
            read_initialization: uor_r4_discrete::ReadInitialization::ZeroGatesV1,
            seed: 1,
            epoch: 0,
            table,
            routes,
            gates: vec![0i32; (spec.blocks as usize) * stages],
            operator_kappa: OPERATOR_KAPPA.into(),
            composition_kappa: COMPOSITION_KAPPA.into(),
            source_sha256: identity_field(&compiled, "source_sha256")?,
            kernel_sha256: identity_field(&compiled, "kernel_sha256")?,
            geometry_sha256: identity_field(&compiled, "geometry_sha256")?,
        })
    }

    fn eval_single_window(
        spec: Spec,
        policy: ObjectivePolicy,
        table: Vec<i32>,
        routes: Vec<u32>,
        window: &[Identity],
        target: u32,
    ) -> Result<(Device, Evaluation), String> {
        let snap = make_route_snapshot(spec, policy, table, routes)?;
        let device = Device::from_snapshot(snap, 0).map_err(|e| format!("from_snapshot: {e}"))?;
        let batch = device
            .carry(window)
            .map_err(|e| format!("carry window: {e}"))?;
        let eval = device
            .evaluate(&batch, &[target])
            .map_err(|e| format!("evaluate: {e}"))?;
        if device.epoch().map_err(|e| e.to_string())? != 0 {
            return Err("device epoch mutated during evaluation".into());
        }
        Ok((device, eval))
    }

    fn run_c_r1(out_dir: &Path) -> Result<ControlReceipt, String> {
        let mut mixed_hinges = Vec::with_capacity(16);
        let mut mixed_admitted = Vec::with_capacity(16);
        let mut tie_hinges = Vec::with_capacity(16);
        let mut tie_admitted = Vec::with_capacity(16);
        let win3 = three_token_window(0);

        for k in 1..=16u32 {
            let spec = Spec {
                vocabulary: 16,
                blocks: 4,
                context: 3,
                kept_cells: k,
            };
            let (_, eval_mixed) = eval_single_window(
                spec,
                ObjectivePolicy::OccupiedCellRankHingeV1,
                c_r1_mixed_distance_table(),
                vec![0u32; 8],
                &win3,
                2,
            )?;
            let row_m = &eval_mixed.rows[0];
            if row_m.occupied_cells != 5 || row_m.target_cell_rank != 3 {
                return Err(format!(
                    "C-R1 1A-i K={k}: expected occupied_cells=5, rank=3, got occ={}, rank={}",
                    row_m.occupied_cells, row_m.target_cell_rank
                ));
            }
            mixed_hinges.push(row_m.cell_rank_hinge);
            mixed_admitted.push(eval_mixed.aggregate.target_admitted_rows == 1);

            let (_, eval_tie) = eval_single_window(
                spec,
                ObjectivePolicy::OccupiedCellRankHingeV1,
                c_r1_equal_distance_tie_table(),
                c_r1_equal_distance_routes(),
                &win3,
                2,
            )?;
            let row_t = &eval_tie.rows[0];
            if row_t.occupied_cells != 5 || row_t.target_cell_rank != 3 {
                return Err(format!(
                    "C-R1 1A-ii K={k}: expected occupied_cells=5, rank=3, got occ={}, rank={}",
                    row_t.occupied_cells, row_t.target_cell_rank
                ));
            }
            tie_hinges.push(row_t.cell_rank_hinge);
            tie_admitted.push(eval_tie.aggregate.target_admitted_rows == 1);
        }

        if mixed_hinges != EXPECTED_C_R1_HINGE_BY_K
            || tie_hinges != EXPECTED_C_R1_HINGE_BY_K
            || mixed_admitted != EXPECTED_C_R1_ADMITTED_BY_K
            || tie_admitted != EXPECTED_C_R1_ADMITTED_BY_K
        {
            return Err("C-R1 K=1..=16 hinge or admission vector mismatch".into());
        }

        // Fixture 1B: 256-cell full-occupancy boundary at K=16
        let spec_256 = Spec {
            vocabulary: 256,
            blocks: 4,
            context: 1,
            kept_cells: 16,
        };
        let win1 = one_token_window(34);
        let (_, eval_256) = eval_single_window(
            spec_256,
            ObjectivePolicy::OccupiedCellRankHingeV1,
            c_r1_all_256_cells_table(),
            vec![0u32; 4],
            &win1,
            255,
        )?;
        let row_256 = &eval_256.rows[0];
        if row_256.occupied_cells != 256
            || row_256.admitted_cells != 16
            || eval_256.aggregate.target_admitted_rows != 0
            || row_256.target_cell_rank != 256
            || row_256.cell_rank_hinge != 240
            || eval_256.aggregate.weighted_cell_rank_hinge_sum != 7680
        {
            return Err(format!("C-R1 1B 256-cell boundary mismatch: {:?}", row_256));
        }

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R1",
            "name": "rank_hinge_exact_admission_equivalence",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "fixture_1a_mixed_distance": {
                "occupied_cells": 5,
                "target_cell": 15,
                "target_cell_rank": 3,
                "cell_rank_hinge_k1_to_16": mixed_hinges,
                "target_admitted_k1_to_16": mixed_admitted,
            },
            "fixture_1a_equal_distance_tie": {
                "occupied_cells": 5,
                "target_cell": 15,
                "target_cell_rank": 3,
                "cell_rank_hinge_k1_to_16": tie_hinges,
                "target_admitted_k1_to_16": tie_admitted,
            },
            "fixture_1b_all_256_cells_k16": {
                "occupied_cells": row_256.occupied_cells,
                "admitted_cells": row_256.admitted_cells,
                "target_cell_rank": row_256.target_cell_rank,
                "cell_rank_hinge": row_256.cell_rank_hinge,
                "weighted_cell_rank_hinge_sum": eval_256.aggregate.weighted_cell_rank_hinge_sum,
            }
        });
        let pin = write_new_json(&out_dir.join("c-r1/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R1".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    fn run_c_r2(out_dir: &Path) -> Result<ControlReceipt, String> {
        let win3 = three_token_window(0);
        let routes = c_r1_equal_distance_routes();
        let mut ranks_m = Vec::new();
        let mut hinges_k1 = Vec::new();
        let mut hinges_k2 = Vec::new();
        let mut hinges_k3 = Vec::new();
        let mut admitted_tokens_k3 = Vec::new();

        for m in [1usize, 2, 4] {
            let table = c_r2_crowded_rival_cell_table(m);
            for (k, bucket) in [
                (1u32, &mut hinges_k1),
                (2u32, &mut hinges_k2),
                (3u32, &mut hinges_k3),
            ] {
                let spec = Spec {
                    vocabulary: 16,
                    blocks: 4,
                    context: 3,
                    kept_cells: k,
                };
                let (_, eval) = eval_single_window(
                    spec,
                    ObjectivePolicy::OccupiedCellRankHingeV1,
                    table.clone(),
                    routes.clone(),
                    &win3,
                    2,
                )?;
                let row = &eval.rows[0];
                if row.occupied_cells != 5 || row.target_cell_rank != 3 {
                    return Err(format!(
                        "C-R2 2A m={m} K={k}: expected occ=5, rank=3, got occ={}, rank={}",
                        row.occupied_cells, row.target_cell_rank
                    ));
                }
                bucket.push(row.cell_rank_hinge);
                if k == 3 {
                    ranks_m.push(row.target_cell_rank);
                    admitted_tokens_k3.push(row.admitted_token_count);
                }
            }
        }

        if ranks_m != vec![3, 3, 3]
            || hinges_k1 != vec![2, 2, 2]
            || hinges_k2 != vec![1, 1, 1]
            || hinges_k3 != vec![0, 0, 0]
            || admitted_tokens_k3 != vec![3, 4, 6]
        {
            return Err("C-R2 2A rival cell crowding invariance failed".into());
        }

        // Part 2B: moving one spare token into newly occupied preceding cell 2 across K in 1..=4
        let new_cell_table = c_r2_new_preceding_cell_table();
        let mut new_cell_hinges = Vec::new();
        for k in 1..=4u32 {
            let spec_k = Spec {
                vocabulary: 16,
                blocks: 4,
                context: 3,
                kept_cells: k,
            };
            let (_, eval_new_cell) = eval_single_window(
                spec_k,
                ObjectivePolicy::OccupiedCellRankHingeV1,
                new_cell_table.clone(),
                routes.clone(),
                &win3,
                2,
            )?;
            let row_new = &eval_new_cell.rows[0];
            if row_new.occupied_cells != 6 || row_new.target_cell_rank != 4 {
                return Err(format!(
                    "C-R2 2B K={k} new preceding cell mismatch: occ={}, rank={}",
                    row_new.occupied_cells, row_new.target_cell_rank
                ));
            }
            new_cell_hinges.push(row_new.cell_rank_hinge);
        }
        if new_cell_hinges != vec![3, 2, 1, 0] {
            return Err(format!(
                "C-R2 2B K in 1..=4 hinge mismatch: got {new_cell_hinges:?}, expected [3, 2, 1, 0]"
            ));
        }

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R2",
            "name": "single_count_per_occupied_cell_invariance",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "part_2a_crowded_rival_cell": {
                "rival_cell": 1,
                "multiplicities": [1, 2, 4],
                "target_cell_rank": ranks_m,
                "cell_rank_hinge_k1": hinges_k1,
                "cell_rank_hinge_k2": hinges_k2,
                "cell_rank_hinge_k3": hinges_k3,
                "admitted_token_count_k3": admitted_tokens_k3,
            },
            "part_2b_new_preceding_cell": {
                "new_preceding_cell": 2,
                "kept_cells_swept": [1, 2, 3, 4],
                "occupied_cells_before": 5,
                "occupied_cells_after": 6,
                "target_cell_rank_before": 3,
                "target_cell_rank_after": 4,
                "cell_rank_hinge_before_k1_to_k4": [2, 1, 0, 0],
                "cell_rank_hinge_after_k1_to_k4": new_cell_hinges,
            }
        });
        let pin = write_new_json(&out_dir.join("c-r2/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R2".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    fn run_c_r3(out_dir: &Path) -> Result<ControlReceipt, String> {
        let spec = Spec {
            vocabulary: 16,
            blocks: 4,
            context: 3,
            kept_cells: 2,
        };
        let win3 = three_token_window(0);
        let routes = c_r1_equal_distance_routes();

        // 5 mutation fixtures covering all 5 classes in objective.rs:100
        struct MutationCase {
            name: &'static str,
            base_table: Vec<i32>,
            base_routes: Vec<u32>,
            parameter: Parameter,
            token_block_root: Option<(usize, usize, usize)>,
            route_index: Option<usize>,
        }

        let cases = vec![
            MutationCase {
                name: "create_cell",
                base_table: c_r1_equal_distance_tie_table(),
                base_routes: routes.clone(),
                parameter: Parameter::VectorBlock(4 * 4 + 3),
                token_block_root: Some((4, 3, 15)),
                route_index: None,
            },
            MutationCase {
                name: "empty_cell",
                base_table: c_r3_empty_cell_base_table(),
                base_routes: routes.clone(),
                // Block 3, cell 0 -> index 12. Written without the identity `+ 0` the lint gate denied.
                parameter: Parameter::VectorBlock(3 * 4),
                token_block_root: Some((3, 0, 1)),
                route_index: None,
            },
            MutationCase {
                name: "move_target_cell",
                base_table: c_r1_equal_distance_tie_table(),
                base_routes: routes.clone(),
                parameter: Parameter::VectorBlock(2 * 4 + 3),
                token_block_root: Some((2, 3, 15)),
                route_index: None,
            },
            MutationCase {
                name: "interior_block_only",
                base_table: c_r1_equal_distance_tie_table(),
                base_routes: routes.clone(),
                parameter: Parameter::VectorBlock(2 * 4 + 1),
                token_block_root: Some((2, 1, 0)),
                route_index: None,
            },
            MutationCase {
                name: "identity_candidate",
                base_table: c_r1_equal_distance_tie_table(),
                base_routes: vec![0u32; 8],
                parameter: Parameter::Route(0),
                token_block_root: None,
                route_index: Some(0),
            },
        ];

        let mut mutation_summaries = Vec::new();
        let mut total_candidate_checks = 0usize;

        for case in &cases {
            let mut created_cell_observed = false;
            let mut emptied_cell_observed = false;
            let mut moved_target_rank_observed = false;
            let mut interior_invariance_verified = true;

            for policy in [
                ObjectivePolicy::NearestAddressMarginV1,
                ObjectivePolicy::OccupiedCellRankHingeV1,
            ] {
                let (base_dev, base_eval) = eval_single_window(
                    spec,
                    policy,
                    case.base_table.clone(),
                    case.base_routes.clone(),
                    &win3,
                    2,
                )?;
                let base_batch = base_dev.carry(&win3).map_err(|e| format!("carry: {e}"))?;
                let proposal = base_dev
                    .propose(&base_batch, &[2], case.parameter)
                    .map_err(|e| format!("propose {}: {e}", case.name))?;
                let cand_losses = proposal
                    .candidate_losses()
                    .map_err(|e| format!("candidate_losses {}: {e}", case.name))?;
                let obs = proposal
                    .observation()
                    .map_err(|e| format!("observation {}: {e}", case.name))?;
                let base_row = &base_eval.rows[0];

                if obs.before != base_eval.aggregate.objective_loss_numerator_sum
                    || cand_losses[0] != base_eval.aggregate.objective_loss_numerator_sum
                {
                    return Err(format!(
                        "C-R3 {} ({:?}): baseline/identity candidate loss mismatch",
                        case.name, policy
                    ));
                }

                for (c, &virtual_loss) in cand_losses.iter().enumerate() {
                    let mut cand_table = case.base_table.clone();
                    let mut cand_routes = case.base_routes.clone();
                    if let Some((tok, blk, base_root)) = case.token_block_root {
                        let new_root = candidate_root_image(base_root, c)
                            .ok_or("missing candidate root image")?;
                        let dst = (tok * 4 + blk) * 8;
                        cand_table[dst..dst + 8].copy_from_slice(&ROOTS[new_root]);
                    }
                    if let Some(r_idx) = case.route_index {
                        cand_routes[r_idx] = c as u32;
                    }
                    let (_, cand_eval) =
                        eval_single_window(spec, policy, cand_table, cand_routes, &win3, 2)?;
                    let crow = &cand_eval.rows[0];
                    if virtual_loss != cand_eval.aggregate.objective_loss_numerator_sum {
                        return Err(format!(
                            "C-R3 {} ({:?}) cand {c}: virtual_loss={virtual_loss} != eval_loss={}",
                            case.name, policy, cand_eval.aggregate.objective_loss_numerator_sum
                        ));
                    }
                    if c == 0
                        && (crow.target_cell_rank != base_row.target_cell_rank
                            || crow.cell_rank_hinge != base_row.cell_rank_hinge
                            || crow.token_hinge != base_row.token_hinge
                            || crow.address_hinge != base_row.address_hinge
                            || crow.occupied_cells != base_row.occupied_cells
                            || crow.admitted_token_count != base_row.admitted_token_count
                            || crow.loss_ta_numerator != base_row.loss_ta_numerator
                            || crow.loss_tr_numerator != base_row.loss_tr_numerator)
                    {
                        return Err(format!(
                            "C-R3 {} ({:?}): candidate 0 compare_fields mismatch",
                            case.name, policy
                        ));
                    }
                    if case.name == "create_cell" && crow.occupied_cells == 6 {
                        created_cell_observed = true;
                    }
                    if case.name == "empty_cell" && crow.occupied_cells == 4 {
                        emptied_cell_observed = true;
                    }
                    if case.name == "move_target_cell"
                        && crow.target_cell_rank != base_row.target_cell_rank
                    {
                        moved_target_rank_observed = true;
                    }
                    if case.name == "interior_block_only"
                        && (crow.occupied_cells != base_row.occupied_cells
                            || crow.target_cell_rank != base_row.target_cell_rank
                            || crow.cell_rank_hinge != base_row.cell_rank_hinge
                            || crow.address_hinge != base_row.address_hinge)
                    {
                        interior_invariance_verified = false;
                    }
                    total_candidate_checks += 1;
                }
            }

            if case.name == "create_cell" && !created_cell_observed {
                return Err("C-R3 create_cell did not observe 5->6 occupancy expansion".into());
            }
            if case.name == "empty_cell" && !emptied_cell_observed {
                return Err("C-R3 empty_cell did not observe 5->4 occupancy reduction".into());
            }
            if case.name == "move_target_cell" && !moved_target_rank_observed {
                return Err("C-R3 move_target_cell did not observe target_cell_rank change".into());
            }
            if case.name == "interior_block_only" && !interior_invariance_verified {
                return Err("C-R3 interior_block_only violated endpoint/rank invariance".into());
            }

            mutation_summaries.push(serde_json::json!({
                "mutation": case.name,
                "candidates_verified_per_policy": 16,
                "policies_verified": ["nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"],
                "passed": true,
            }));
        }

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R3",
            "name": "virtual_vs_applied_candidate_identity_on_rank_and_occupancy",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "total_candidate_evaluations_verified": total_candidate_checks,
            "mutations": mutation_summaries,
        });
        let pin = write_new_json(&out_dir.join("c-r3/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R3".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    fn run_c_r4(out_dir: &Path) -> Result<ControlReceipt, String> {
        let win1 = one_token_window(0);
        // 4A-i: M = 1 occupied cell at K = 16
        let spec_k16 = Spec {
            vocabulary: 16,
            blocks: 4,
            context: 1,
            kept_cells: 16,
        };
        let (_, eval_m1) = eval_single_window(
            spec_k16,
            ObjectivePolicy::OccupiedCellRankHingeV1,
            c_r4_single_occupied_cell_table(),
            vec![0u32; 4],
            &win1,
            0,
        )?;
        let row_m1 = &eval_m1.rows[0];
        if row_m1.occupied_cells != 1
            || row_m1.target_cell_rank != 1
            || row_m1.cell_rank_hinge != 0
            || eval_m1.aggregate.target_admitted_rows != 1
        {
            return Err(format!("C-R4 4A-i M=1 mismatch: {:?}", row_m1));
        }

        // 4A-ii: M = 16 occupied cells at K = 16 across all 16 target tokens
        let snap_m16 = make_route_snapshot(
            spec_k16,
            ObjectivePolicy::OccupiedCellRankHingeV1,
            c_r4_sixteen_occupied_cells_table(),
            vec![0u32; 4],
        )?;
        let dev_m16 = Device::from_snapshot(snap_m16, 0).map_err(|e| e.to_string())?;
        let windows_16 = vec![vec![win1[0]]; 16];
        let batch_16 = dev_m16
            .carry_windows(&windows_16)
            .map_err(|e| e.to_string())?;
        let targets_16: Vec<u32> = (0..16).collect();
        let eval_m16 = dev_m16
            .evaluate(&batch_16, &targets_16)
            .map_err(|e| e.to_string())?;
        if eval_m16.aggregate.occupied_cells != 16
            || eval_m16.aggregate.target_admitted_rows != 16
            || eval_m16.aggregate.cell_rank_hinge_sum != 0
            || eval_m16
                .rows
                .iter()
                .any(|r| !(1..=16).contains(&r.target_cell_rank) || r.cell_rank_hinge != 0)
        {
            return Err("C-R4 4A-ii M=16 at K=16 failed".into());
        }

        // 4B: tr == 0 row skipping and wholly ineligible status-8 refusal under both policies
        let spec_k1 = Spec {
            vocabulary: 16,
            blocks: 4,
            context: 1,
            kept_cells: 1,
        };
        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let snap =
                make_route_snapshot(spec_k1, policy, c_r4_absent_rival_table(), vec![0u32; 4])?;
            let dev = Device::from_snapshot(snap, 0).map_err(|e| e.to_string())?;
            let batch_ineligible = dev.carry(&win1).map_err(|e| e.to_string())?;
            let Err(err) = dev.propose(&batch_ineligible, &[0], Parameter::VectorBlock(0)) else {
                return Err("wholly ineligible batch must refuse with status 8".into());
            };
            if !err.to_string().contains("status 8") {
                return Err(format!("expected CUDA status 8 refusal, got: {err}"));
            }

            let batch_eligible_only = dev.carry(&win1).map_err(|e| e.to_string())?;
            let prop_1 = dev
                .propose(&batch_eligible_only, &[1], Parameter::VectorBlock(4))
                .map_err(|e| e.to_string())?;
            let batch_mixed = dev
                .carry_windows(&[vec![win1[0]], vec![win1[0]]])
                .map_err(|e| e.to_string())?;
            let prop_2 = dev
                .propose(&batch_mixed, &[0, 1], Parameter::VectorBlock(4))
                .map_err(|e| e.to_string())?;
            if prop_1.candidate_losses().map_err(|e| e.to_string())?
                != prop_2.candidate_losses().map_err(|e| e.to_string())?
            {
                return Err(format!(
                    "C-R4 4B-i tr==0 row changed candidate_losses under {:?}",
                    policy
                ));
            }
        }

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R4",
            "name": "absent_rival_and_occupancy_under_k_bounds",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "occupancy_under_k": {
                "m1_at_k16": {
                    "occupied_cells": row_m1.occupied_cells,
                    "target_cell_rank": row_m1.target_cell_rank,
                    "cell_rank_hinge": row_m1.cell_rank_hinge,
                    "target_admitted": eval_m1.aggregate.target_admitted_rows == 1,
                },
                "m16_at_k16": {
                    "occupied_cells": eval_m16.aggregate.occupied_cells,
                    "target_admitted_rows": eval_m16.aggregate.target_admitted_rows,
                    "cell_rank_hinge_sum": eval_m16.aggregate.cell_rank_hinge_sum,
                }
            },
            "absent_rival_checks": {
                "wholly_ineligible_refusal_status": 8,
                "mixed_batch_skips_tr_zero_identically": true,
                "policies_verified": ["nearest_address_margin_v1", "occupied_cell_rank_hinge_v1"],
            }
        });
        let pin = write_new_json(&out_dir.join("c-r4/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R4".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    fn run_c_r5(out_dir: &Path) -> Result<ControlReceipt, String> {
        // Part 5A: multi-row GPU Device::evaluate + Device::aggregate_evaluations
        let spec_256 = Spec {
            vocabulary: 256,
            blocks: 4,
            context: 1,
            kept_cells: 16,
        };
        let table_256 = c_r1_all_256_cells_table();
        let win_a = [
            vec![Identity {
                token: 34,
                occurrence: 1,
                version: 1,
                record: 1,
            }],
            vec![Identity {
                token: 34,
                occurrence: 2,
                version: 1,
                record: 1,
            }],
        ];
        let win_b = [
            vec![Identity {
                token: 34,
                occurrence: 3,
                version: 1,
                record: 1,
            }],
            vec![Identity {
                token: 34,
                occurrence: 4,
                version: 1,
                record: 1,
            }],
        ];
        let targets_a = [34u32, 255u32];
        let targets_b = [0u32, 128u32];

        for policy in [
            ObjectivePolicy::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1,
        ] {
            let snap = make_route_snapshot(spec_256, policy, table_256.clone(), vec![0u32; 4])?;
            let dev = Device::from_snapshot(snap, 0).map_err(|e| e.to_string())?;
            let batch_a = dev.carry_windows(&win_a).map_err(|e| e.to_string())?;
            let batch_b = dev.carry_windows(&win_b).map_err(|e| e.to_string())?;
            let eval_a = dev
                .evaluate(&batch_a, &targets_a)
                .map_err(|e| e.to_string())?;
            let eval_b = dev
                .evaluate(&batch_b, &targets_b)
                .map_err(|e| e.to_string())?;
            let combined = dev
                .aggregate_evaluations(&[eval_a, eval_b])
                .map_err(|e| e.to_string())?;

            let win_all = [
                win_a[0].clone(),
                win_a[1].clone(),
                win_b[0].clone(),
                win_b[1].clone(),
            ];
            let batch_all = dev.carry_windows(&win_all).map_err(|e| e.to_string())?;
            let eval_all = dev
                .evaluate(&batch_all, &[34, 255, 0, 128])
                .map_err(|e| e.to_string())?;
            let agg = &eval_all.aggregate;

            if eval_all.schema != EVALUATION_SCHEMA
                || serde_json::to_value(&combined).map_err(|e| e.to_string())?
                    != serde_json::to_value(agg).map_err(|e| e.to_string())?
            {
                return Err(
                    "C-R5 5A combined aggregate_evaluations != single-pass aggregate".into(),
                );
            }

            if agg.weighted_token_hinge_sum != 8 * agg.token_hinge_sum
                || agg.weighted_address_hinge_sum != 32 * agg.address_hinge_sum
                || agg.weighted_cell_rank_hinge_sum != 32 * agg.cell_rank_hinge_sum
                || agg.loss_ta_numerator_sum
                    != agg.weighted_token_hinge_sum + agg.weighted_address_hinge_sum
                || agg.loss_tr_numerator_sum
                    != agg.weighted_token_hinge_sum + agg.weighted_cell_rank_hinge_sum
                || agg.address_loss_share_ta
                    != (ExactRatio {
                        numerator: agg.weighted_address_hinge_sum,
                        denominator: agg.loss_ta_numerator_sum,
                    })
                || agg.rank_loss_share_tr
                    != (ExactRatio {
                        numerator: agg.weighted_cell_rank_hinge_sum,
                        denominator: agg.loss_tr_numerator_sum,
                    })
                || agg.mean_admitted_token_count
                    != (ExactRatio {
                        numerator: agg.admitted_token_count_sum,
                        denominator: agg.scored_rows,
                    })
            {
                return Err(
                    "C-R5 5A dual-loss, ExactRatio share, or mean_admitted_token_count mismatch"
                        .into(),
                );
            }

            let expected_obj = match policy {
                ObjectivePolicy::NearestAddressMarginV1 => agg.loss_ta_numerator_sum,
                ObjectivePolicy::OccupiedCellRankHingeV1 => agg.loss_tr_numerator_sum,
            };
            if agg.objective_loss_numerator_sum != expected_obj {
                return Err("C-R5 5A objective_loss_numerator_sum policy routing mismatch".into());
            }

            verify_hinge_distribution(
                "token_hinge",
                &agg.token_hinge_distribution,
                HISTOGRAM_BINS[0],
                agg.token_hinge_sum,
                agg.scored_rows,
            )?;
            verify_hinge_distribution(
                "address_hinge",
                &agg.address_hinge_distribution,
                HISTOGRAM_BINS[1],
                agg.address_hinge_sum,
                agg.scored_rows,
            )?;
            verify_hinge_distribution(
                "cell_rank_hinge",
                &agg.cell_rank_hinge_distribution,
                HISTOGRAM_BINS[2],
                agg.cell_rank_hinge_sum,
                agg.scored_rows,
            )?;

            if dev.aggregate_evaluations(&[]).is_ok() {
                return Err("C-R5 5A empty aggregate_evaluations must fail".into());
            }
        }

        // Part 5B: GPU kernel-level status-12 bounds and u64 overflow checks
        let overflow_report = run_status12_overflow_checks()?;

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R5",
            "name": "bounded_integer_arithmetic_dual_loss_and_distribution_schema",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "evaluation_schema": EVALUATION_SCHEMA,
            "row_words": ROW_WORDS,
            "aggregate_words": AGGREGATE_WORDS,
            "histogram_bins": HISTOGRAM_BINS,
            "dual_loss_and_summary_aggregation_verified": true,
            "status12_overflow_checks": overflow_report,
        });
        let pin = write_new_json(&out_dir.join("c-r5/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R5".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    fn run_c_r6(
        out_dir: &Path,
        repo_root: &Path,
        g_pre_dir: &Path,
    ) -> Result<ControlReceipt, String> {
        let host_check = run_host_checks(repo_root)?;

        // Part 6B: GPU V4096/B4/C192 checkpoint-0 Device::new_with_objective across seed in {1,2}, K in {8,16}
        let mut device_checks = Vec::new();
        let sample_win: Vec<Identity> = (0..192u32)
            .map(|i| Identity {
                token: i % 16,
                occurrence: u64::from(i + 1),
                version: 1,
                record: 1,
            })
            .collect();

        for (seed, k) in [(1u32, 8u32), (1, 16), (2, 8), (2, 16)] {
            let spec = Spec {
                vocabulary: 4096,
                blocks: 4,
                context: 192,
                kept_cells: k,
            };
            let dev = Device::new_with_objective(
                spec,
                ReadSpec::Flock { kept: 3 },
                ObjectivePolicy::OccupiedCellRankHingeV1,
                seed,
                0,
            )
            .map_err(|e| format!("C-R6 Device::new_with_objective seed={seed} K={k}: {e}"))?;
            let batch = dev.carry(&sample_win).map_err(|e| e.to_string())?;
            let eval = dev.evaluate(&batch, &[0]).map_err(|e| e.to_string())?;
            if eval.aggregate.occupied_cells != 16 {
                return Err(format!(
                    "C-R6 seed={seed} K={k}: expected checkpoint-0 occupied_cells=16, got {}",
                    eval.aggregate.occupied_cells
                ));
            }
            if k == 16
                && (eval.rows[0].admitted_cells != 16
                    || eval.aggregate.target_admitted_rows != 1
                    || eval.aggregate.cell_rank_hinge_sum != 0)
            {
                return Err(format!(
                    "C-R6 seed={seed} K=16 checkpoint-0 failed 100% admission"
                ));
            }
            device_checks.push(serde_json::json!({
                "seed": seed,
                "kept_cells": k,
                "occupied_cells": eval.aggregate.occupied_cells,
                "admitted_cells": eval.rows[0].admitted_cells,
                "target_admitted_rows": eval.aggregate.target_admitted_rows,
                "cell_rank_hinge_sum": eval.aggregate.cell_rank_hinge_sum,
            }));
        }

        // Part 6C: Verify G-Pre zero-update corpus evaluation receipts in g_pre_dir
        let read_eval = |rel: &str| -> Result<(FilePin, serde_json::Value), String> {
            let path = g_pre_dir.join(rel);
            let bytes = fs::read(&path)
                .map_err(|e| format!("C-R6 missing G-Pre receipt {}: {e}", path.display()))?;
            let sha256 = sha256_hex(&bytes);
            let val: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            // The zero-update flags are SIBLINGS of the aggregate: at the receipt root for a raw
            // driver receipt, under `evaluation` for a completion envelope.
            let flags = driver_zero_update(&val, &path.display().to_string())?;
            if flags["updates_applied_by_evaluation"] != 0
                || flags["accepted_updates_in_model"] != 0
                || flags["evaluation_mutates_model"] != false
            {
                return Err(format!(
                    "C-R6 G-Pre receipt {} violated zero-update invariant",
                    path.display()
                ));
            }
            Ok((
                FilePin {
                    path: path.to_path_buf(),
                    sha256,
                },
                val,
            ))
        };

        let (k8_pin, k8_val) = read_eval("g-pre-k8-A-seed1-eval/report.json")?;
        let k8_eval = driver_evaluation(&k8_val, "g-pre-k8-A-seed1-eval/report.json")?;
        if k8_eval["target_admitted_rows"].as_u64() != Some(G_PRE_K8_SEED1_EXPECTED_ADMITTED)
            || k8_eval["top1_correct"].as_u64() != Some(G_PRE_K8_SEED1_EXPECTED_TOP1)
            || k8_eval["top8_correct"].as_u64() != Some(G_PRE_K8_SEED1_EXPECTED_TOP8)
            || k8_eval["token_strict_win_eligible_rows"].as_u64()
                != Some(G_PRE_K8_SEED1_EXPECTED_STRICT_WINS)
            || k8_eval["address_hinge_sum"].as_u64()
                != Some(G_PRE_K8_SEED1_EXPECTED_ADDRESS_HINGE_SUM)
            || k8_eval["occupied_cells"].as_u64() != Some(16)
        {
            return Err(
                "C-R6 G-Pre seed=1 K=8 failed to match sealed pilot checkpoint-0 counts".into(),
            );
        }

        let mut k16_receipts = Vec::new();
        for run in [
            "g-pre-k16-A-seed1-eval/report.json",
            "g-pre-k16-R-seed1-eval/report.json",
            "g-pre-k16-A-seed2-eval/report.json",
            "g-pre-k16-R-seed2-eval/report.json",
        ] {
            let (pin, val) = read_eval(run)?;
            let ev = driver_evaluation(&val, run)?;
            let target_admitted = ev["target_admitted_rows"]
                .as_u64()
                .ok_or("missing target_admitted_rows")?;
            if target_admitted < G_PRE_K16_MINIMUM {
                return Err(format!(
                    "C-R6 G-Pre FAIL on {run}: target_admitted_rows={target_admitted} < {G_PRE_K16_MINIMUM}"
                ));
            }
            k16_receipts.push(serde_json::json!({
                "receipt": pin,
                "target_admitted_rows": target_admitted,
                "occupied_cells": ev["occupied_cells"],
                "admitted_cells": ev["admitted_cells"],
                "cell_rank_hinge_sum": ev["cell_rank_hinge_sum"],
            }));
        }

        let report = serde_json::json!({
            "schema": CONTROL_REPORT_SCHEMA,
            "id": "C-R6",
            "name": "k16_baseline_guard_and_untrained_ceiling_check",
            "status": "PASS",
            "apply_calls": 0,
            "cpu_model_reference_used": false,
            "stage_a_sweep_sha256": host_check.stage_a_sweep_sha256,
            "baseline_admitted_cells_guard_range": [1, 16],
            "g_pre_k16_minimum": G_PRE_K16_MINIMUM,
            "v4096_checkpoint0_device_checks": device_checks,
            "g_pre_k8_seed1_receipt": k8_pin,
            "g_pre_k16_receipts": k16_receipts,
        });
        let pin = write_new_json(&out_dir.join("c-r6/report.json"), &report)?;
        Ok(ControlReceipt {
            id: "C-R6".into(),
            status: "PASS".into(),
            receipt: pin,
        })
    }

    pub fn run_all_controls(
        out_dir: &Path,
        repo_root: &Path,
        g_pre_dir: &Path,
    ) -> Result<SuiteExecutionSummary, String> {
        if out_dir.exists() {
            return Err(format!(
                "refusing to write into existing directory {}",
                out_dir.display()
            ));
        }
        let controls = vec![
            run_c_r1(out_dir)?,
            run_c_r2(out_dir)?,
            run_c_r3(out_dir)?,
            run_c_r4(out_dir)?,
            run_c_r5(out_dir)?,
            run_c_r6(out_dir, repo_root, g_pre_dir)?,
        ];
        let summary = SuiteExecutionSummary {
            schema: RUNNER_SCHEMA.into(),
            status: "PASS".into(),
            model_compute_executed: true,
            apply_calls: 0,
            cpu_model_reference_used: false,
            controls,
        };
        write_new_json(&out_dir.join("c-r-suite-summary.json"), &summary)?;
        Ok(summary)
    }
}

fn usage() -> &'static str {
    "Usage:
  admission-rank-fixture-runner plan
  admission-rank-fixture-runner check [--repo-root DIR]
  admission-rank-fixture-runner run NEW_OUT_DIR --g-pre-dir G_PRE_DIR [--repo-root DIR]"
}

fn run_cli(args: &[String]) -> Result<String, String> {
    match args.first().map(String::as_str) {
        Some("plan") | Some("list") => {
            if args.len() != 1 {
                return Err(usage().into());
            }
            serde_json::to_string_pretty(&build_suite_plan()).map_err(|e| e.to_string())
        }
        Some("check") => {
            let mut repo_root = default_repo_root();
            let mut i = 1usize;
            while i < args.len() {
                if args[i] == "--repo-root" {
                    i += 1;
                    repo_root = PathBuf::from(args.get(i).ok_or_else(|| usage().to_string())?);
                } else {
                    return Err(usage().into());
                }
                i += 1;
            }
            let report = run_host_checks(&repo_root)?;
            serde_json::to_string_pretty(&report).map_err(|e| e.to_string())
        }
        Some("run") => {
            let out_dir = PathBuf::from(args.get(1).ok_or_else(|| usage().to_string())?);
            let mut g_pre_dir: Option<PathBuf> = None;
            let mut repo_root = default_repo_root();
            let mut i = 2usize;
            while i < args.len() {
                match args[i].as_str() {
                    "--g-pre-dir" => {
                        i += 1;
                        g_pre_dir = Some(PathBuf::from(
                            args.get(i).ok_or_else(|| usage().to_string())?,
                        ));
                    }
                    "--repo-root" => {
                        i += 1;
                        repo_root = PathBuf::from(args.get(i).ok_or_else(|| usage().to_string())?);
                    }
                    _ => return Err(usage().into()),
                }
                i += 1;
            }
            let g_pre_dir = g_pre_dir.ok_or_else(|| {
                "run requires --g-pre-dir G_PRE_DIR containing G-Pre zero-update evaluation receipts".to_string()
            })?;
            #[cfg(feature = "cuda")]
            {
                let summary = gpu_runner::run_all_controls(&out_dir, &repo_root, &g_pre_dir)?;
                serde_json::to_string_pretty(&summary).map_err(|e| e.to_string())
            }
            #[cfg(not(feature = "cuda"))]
            {
                let _ = (out_dir, repo_root, g_pre_dir);
                Err("GPU fixture execution requires compiling with `--features cuda` on an authorized CUDA 12.8+ pod (D24/D32: no CPU model fallback)".into())
            }
        }
        _ => Err(usage().into()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run_cli(&args) {
        Ok(out) => println!("{out}"),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::{
        c_r1_all_256_cells_table, c_r1_equal_distance_tie_table, c_r1_mixed_distance_table,
        c_r2_crowded_rival_cell_table, c_r2_new_preceding_cell_table, c_r3_empty_cell_base_table,
        c_r4_absent_rival_table, c_r4_single_occupied_cell_table,
        c_r4_sixteen_occupied_cells_table, candidate_root_image, ROOTS,
    };
    use uor_r4_discrete::objective::CONTROL_SCAFFOLD;

    fn assert_valid_e8_table(table: &[i32], expected_tokens: usize) {
        assert_eq!(table.len(), expected_tokens * 32);
        for chunk in table.chunks_exact(8) {
            assert!(
                ROOTS.iter().any(|r| r == chunk),
                "block is not one of the 16 E8 anchor-orbit roots: {chunk:?}"
            );
        }
    }

    #[test]
    fn suite_plan_matches_objective_control_scaffold_and_reports_not_run() {
        let plan = build_suite_plan();
        assert_eq!(plan.status, "NOT_RUN");
        assert!(!plan.model_compute_executed);
        assert_eq!(plan.apply_calls, 0);
        assert_eq!(plan.controls.len(), CONTROL_SCAFFOLD.len());
        for (idx, (id, name)) in CONTROL_SCAFFOLD.iter().enumerate() {
            assert_eq!(plan.controls[idx].id, *id);
            assert_eq!(plan.controls[idx].name, *name);
            assert_eq!(plan.controls[idx].status, "NOT_RUN");
            assert_eq!(
                plan.controls[idx].relative_report_path,
                format!("c-r{}/report.json", idx + 1)
            );
        }
    }

    #[test]
    fn host_checks_verify_specs_geometry_and_stage_a_sweep_without_cuda() {
        let report = run_host_checks(&default_repo_root()).expect("host checks must pass");
        assert_eq!(report.status, "HOST_CONTRACT_VERIFIED_GPU_NOT_RUN");
        assert!(!report.cuda_context_created);
        assert!(!report.model_compute_executed);
        assert_eq!(report.apply_calls, 0);
        assert_eq!(report.scaffold_controls_verified, 6);
        assert_eq!(report.fixture_specs_validated, 19);
        assert!(report.stage_a_sweep_verified);
        assert!(report.baseline_admitted_cells_guard_verified);
    }

    #[test]
    fn c_r1_and_c_r2_fixture_tables_place_all_tokens_on_valid_e8_orbit_roots() {
        assert_valid_e8_table(&c_r1_mixed_distance_table(), 16);
        assert_valid_e8_table(&c_r1_equal_distance_tie_table(), 16);
        assert_valid_e8_table(&c_r1_all_256_cells_table(), 256);
        for m in [1, 2, 4] {
            assert_valid_e8_table(&c_r2_crowded_rival_cell_table(m), 16);
        }
        assert_valid_e8_table(&c_r2_new_preceding_cell_table(), 16);
    }

    #[test]
    fn c_r3_candidate_root_images_cover_create_empty_move_interior_and_identity() {
        assert_valid_e8_table(&c_r3_empty_cell_base_table(), 16);
        // create_cell: base root 15 moves to non-15 roots for all c in 1..16
        for c in 1..16 {
            let img = candidate_root_image(15, c).unwrap();
            assert_ne!(img, 15, "candidate {c} on root 15 must create a fresh cell");
        }
        // empty_cell: base root 1 maps to root 15 at c in {7, 15}, merging cell 31 into cell 255
        assert_eq!(candidate_root_image(1, 7), Some(15));
        assert_eq!(candidate_root_image(1, 15), Some(15));
        // move_target_cell: base root 15 maps to root 3 at c in {3, 12}
        assert_eq!(candidate_root_image(15, 3), Some(3));
        assert_eq!(candidate_root_image(15, 12), Some(3));
    }

    #[test]
    fn c_r4_fixture_tables_cover_single_cell_sixteen_cells_and_absent_rival() {
        assert_valid_e8_table(&c_r4_single_occupied_cell_table(), 16);
        assert_valid_e8_table(&c_r4_sixteen_occupied_cells_table(), 16);
        assert_valid_e8_table(&c_r4_absent_rival_table(), 16);
    }

    #[test]
    fn cli_rejects_unknown_subcommands_and_refuses_gpu_run_without_cuda_feature() {
        assert!(run_cli(&["plan".to_string()]).is_ok());
        assert!(run_cli(&["check".to_string()]).is_ok());
        assert!(run_cli(&["unknown".to_string()]).is_err());
        #[cfg(not(feature = "cuda"))]
        {
            let err = run_cli(&[
                "run".to_string(),
                "/tmp/nonexistent-out".to_string(),
                "--g-pre-dir".to_string(),
                "/tmp/nonexistent-gpre".to_string(),
            ])
            .unwrap_err();
            assert!(err.contains("--features cuda"));
        }
    }

    #[test]
    fn atomic_json_writer_rejects_existing_target_and_computes_exact_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("c-r1/report.json");
        let payload = serde_json::json!({"id": "C-R1", "status": "PASS"});
        let pin = write_new_json(&target, &payload).unwrap();
        let disk_bytes = fs::read(&target).unwrap();
        assert_eq!(pin.sha256, sha256_hex(&disk_bytes));
        assert!(
            write_new_json(&target, &payload).is_err(),
            "write_new_json must refuse overwriting an existing file"
        );
    }

    #[test]
    fn c_r5_hinge_distribution_verifier_checks_bins_zero_rows_mean_and_nearest_rank_quantiles() {
        let mut hist = vec![0u64; 17];
        hist[0] = 1;
        hist[2] = 1;
        hist[4] = 1;
        hist[16] = 1;
        let dist = HingeDistribution {
            min: 0,
            p25: 0,
            p50: 2,
            p75: 4,
            p90: 16,
            p99: 16,
            max: 16,
            mean: ExactRatio {
                numerator: 22,
                denominator: 4,
            },
            zero_rows: 1,
            histogram: hist,
        };
        assert!(verify_hinge_distribution("test_rank", &dist, 17, 22, 4).is_ok());
        let mut bad_p50 = dist.clone();
        bad_p50.p50 = 4;
        assert!(verify_hinge_distribution("test_rank", &bad_p50, 17, 22, 4).is_err());
        let mut bad_zero = dist;
        bad_zero.zero_rows = 2;
        assert!(verify_hinge_distribution("test_rank", &bad_zero, 17, 22, 4).is_err());
    }
    #[test]
    fn a_raw_driver_receipt_reads_its_aggregate_metrics() {
        // Producer-faithful: metrics ONLY under aggregate, real schema identifiers, no root alias.
        let raw = serde_json::json!({
            "schema": "uor-r4.discrete-corpus-driver/1",
            "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
            "aggregate": {
                "target_admitted_rows": 4894,
                "top1_correct": 0
            },
            "updates_applied_by_evaluation": 0,
            "accepted_updates_in_model": 0,
            "evaluation_mutates_model": false
        });
        let ev = driver_evaluation(&raw, "raw.json").expect("raw aggregate accepted");
        let flags = driver_zero_update(&raw, "raw.json").expect("raw flags beside the aggregate");
        assert_eq!(ev["target_admitted_rows"].as_u64(), Some(4894));
        assert_eq!(flags["updates_applied_by_evaluation"], serde_json::json!(0));
    }

    #[test]
    fn a_completion_envelope_passes_with_its_nested_driver_evaluation() {
        // A completion envelope carries the driver evaluation nested; its flags live inside it.
        let wrapped = serde_json::json!({
            "schema": "uor-r4.completion-evaluation/1",
            "evaluation": {
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "aggregate": {"target_admitted_rows": 4894},
                "updates_applied_by_evaluation": 0,
                "accepted_updates_in_model": 0,
                "evaluation_mutates_model": false
            }
        });
        let ev = driver_evaluation(&wrapped, "wrapped.json").expect("completion envelope accepted");
        let flags = driver_zero_update(&wrapped, "wrapped.json").expect("nested flags accepted");
        assert_eq!(ev["target_admitted_rows"].as_u64(), Some(4894));
        assert_eq!(flags["evaluation_mutates_model"], serde_json::json!(false));
    }

    #[test]
    fn a_missing_aggregate_or_unknown_schema_is_refused_by_name() {
        for bad in [
            // Driver schema with the metrics at the ROOT only: the descent must FAIL, not fall back.
            // This is the case the previous regression masked with an invented root alias.
            serde_json::json!({
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.discrete-objective-evaluation/1",
                "target_admitted_rows": 4894
            }),
            // Named objective schema is wrong.
            serde_json::json!({
                "schema": "uor-r4.discrete-corpus-driver/1",
                "objective_evaluation_schema": "uor-r4.objective-evaluation/v1",
                "aggregate": {}
            }),
            // Completion envelope without the aggregate descent.
            serde_json::json!({
                "schema": "uor-r4.completion-evaluation/1",
                "evaluation": {"target_admitted_rows": 4894}
            }),
            serde_json::json!({}),
            serde_json::json!({"schema": "something-else"}),
        ] {
            let err = driver_evaluation(&bad, "bad.json").expect_err("refused");
            assert!(
                err.contains("bad.json"),
                "the refusal names the receipt: {err}"
            );
        }
    }
}
