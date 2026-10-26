#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub const EVALUATION_SCHEMA: &str = "uor-r4.discrete-objective-evaluation/1";
pub const ROW_WORDS: usize = 14;
pub const HISTOGRAM_START: usize = 29;
pub const HISTOGRAM_BINS: [usize; 3] = [641, 73, 256];
pub const QUANTILE_START: usize = HISTOGRAM_START + 641 + 73 + 256;
pub const AGGREGATE_WORDS: usize = QUANTILE_START + 21;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectivePolicy {
    #[default]
    NearestAddressMarginV1,
    OccupiedCellRankHingeV1,
}

impl ObjectivePolicy {
    pub fn is_nearest_address_margin(&self) -> bool {
        *self == Self::NearestAddressMarginV1
    }

    pub const fn word(self) -> u32 {
        match self {
            Self::NearestAddressMarginV1 => 0,
            Self::OccupiedCellRankHingeV1 => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExactRatio {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HingeDistribution {
    pub min: u64,
    pub p25: u64,
    pub p50: u64,
    pub p75: u64,
    pub p90: u64,
    pub p99: u64,
    pub max: u64,
    pub mean: ExactRatio,
    pub zero_rows: u64,
    pub histogram: Vec<u64>,
}

pub const CONTROL_SCAFFOLD: [(&str, &str); 6] = [
    ("C-R1", "rank_hinge_exact_admission_equivalence"),
    ("C-R2", "single_count_per_occupied_cell_invariance"),
    (
        "C-R3",
        "virtual_vs_applied_candidate_identity_on_rank_and_occupancy",
    ),
    ("C-R4", "absent_rival_and_occupancy_under_k_bounds"),
    (
        "C-R5",
        "bounded_integer_arithmetic_dual_loss_and_distribution_schema",
    ),
    ("C-R6", "k16_baseline_guard_and_untrained_ceiling_check"),
];

pub fn control_scaffold() -> serde_json::Value {
    serde_json::json!({
        "schema": "uor-r4.admission-rank-control-scaffold/1",
        "status": "NOT_RUN",
        "qualification": "HELD_PENDING_EXACT_HEAD_REVIEW_AND_OWNER_RELEASE",
        "controls": CONTROL_SCAFFOLD.iter().map(|(id, name)| serde_json::json!({
            "id": id, "name": name, "status": "NOT_RUN"
        })).collect::<Vec<_>>(),
        "fixture_axes": {
            "kept_cells": (1..=16).collect::<Vec<u32>>(),
            "equal_distance_cell_ids": [0, 1, 15, 16, 255],
            "rival_cell_token_multiplicities": [1, 2, 4],
            "candidate_ordinals": (0..16).collect::<Vec<u32>>(),
            "seeds": [1, 2],
            "g_pre_minimum_target_admitted_rows": 4894,
            "qualification_model_operations": "GPU_ONLY_NOT_EXECUTED"
        },
        "authored_fixture_contracts": {
            "C-R1": {
                "ordered_distance_cell_pairs": [[0,0],[4,1],[4,15],[4,16],[8,255]],
                "target_cell": 15, "expected_rank": 3,
                "expected_hinge_for_k_1_through_16": [2,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
                "all_256_cells_tied_target_255": {"expected_rank":256,"k":16,"expected_hinge":240},
                "scope": "authored comparator inputs; full-writer GPU fixture qualification owed"
            },
            "C-R2": {
                "base_occupied_cells": [0,1,15,16,255],
                "crowded_rival_cell": 1,
                "expected_rank_with_1_2_4_tokens_in_rival_cell": [3,3,3],
                "new_preceding_cell": 2, "expected_rank_after_new_cell": 4
            },
            "C-R3": {
                "mutations": ["create_cell","empty_cell","move_target_cell","interior_block_only","identity_candidate"],
                "compare_fields": ["target_cell_rank","cell_rank_hinge","token_hinge","address_hinge","occupied_cells","admitted_token_count","loss_ta_numerator","loss_tr_numerator"],
                "method": "each virtual candidate versus independently restored then applied candidate; device results only",
                "status": "GPU_FIXTURE_EXECUTOR_OWED"
            },
            "C-R4": {
                "occupancy_under_k": {"occupied_cells":1,"k":16,"expected_rank":1,"expected_hinge":0},
                "zero_token_rivals": {"expected_row_eligible":false,"wholly_ineligible_status":8},
                "compare_policies": ["nearest_address_margin_v1","occupied_cell_rank_hinge_v1"]
            },
            "C-R5": {
                "raw_row_bounds": {"T":640,"A":72,"R":255},
                "k16_rank_hinge_bound":240,
                "overflow_status":12,
                "histogram_bins": HISTOGRAM_BINS,
                "quantile_rule":"nearest rank: ceil(percent * rows / 100); min is first occupied bin",
                "sum_overflow_inputs":["u64::MAX plus 1","weighted R overflow","quantile threshold overflow"],
                "mean_and_share_encoding":"exact numerator/denominator; denominator zero means undefined"
            },
            "C-R6": {
                "baseline_admitted_cells_range":[1,16],
                "seed_k_pairs":[[1,8],[1,16],[2,8],[2,16]],
                "g_pre_k16_minimum":4894,
                "updates_before_baseline":0,
                "status":"GPU_CORPUS_QUALIFICATION_OWED"
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_policy_is_strict_and_distinct_metadata() {
        for (policy, name) in [
            (
                ObjectivePolicy::NearestAddressMarginV1,
                "nearest_address_margin_v1",
            ),
            (
                ObjectivePolicy::OccupiedCellRankHingeV1,
                "occupied_cell_rank_hinge_v1",
            ),
        ] {
            let encoded = serde_json::to_value(policy).unwrap();
            assert_eq!(encoded, name);
            assert_eq!(
                serde_json::from_value::<ObjectivePolicy>(encoded).unwrap(),
                policy
            );
        }
        assert_ne!(
            ObjectivePolicy::NearestAddressMarginV1.word(),
            ObjectivePolicy::OccupiedCellRankHingeV1.word()
        );
        assert!(serde_json::from_str::<ObjectivePolicy>("\"rank\"").is_err());
    }

    #[test]
    fn controls_are_scaffolding_not_qualification_receipts() {
        let scaffold = control_scaffold();
        assert_eq!(scaffold["status"], "NOT_RUN");
        assert_eq!(scaffold["controls"].as_array().unwrap().len(), 6);
        assert!(scaffold["controls"]
            .as_array()
            .unwrap()
            .iter()
            .all(|entry| entry["status"] == "NOT_RUN"));
        assert_eq!(AGGREGATE_WORDS, 1020);
    }

    #[test]
    fn rank_occupancy_policies_and_finite_controls_c_r1_to_c_r7d_pass() {
        for (policy, expected_word, expected_name) in [
            (
                RankOccupancyPolicy::NearestAddressMarginV1,
                0u32,
                "nearest_address_margin_v1",
            ),
            (
                RankOccupancyPolicy::OccupiedCellRankHingeV1,
                1u32,
                "occupied_cell_rank_hinge_v1",
            ),
            (
                RankOccupancyPolicy::FixedGridRankHingeV1,
                2u32,
                "fixed_grid_rank_hinge_v1",
            ),
            (
                RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
                3u32,
                "occupied_cell_rank_crowd_hinge_v1",
            ),
        ] {
            assert_eq!(policy.word(), expected_word);
            assert_eq!(RankOccupancyPolicy::from_word(expected_word), Some(policy));
            let val = serde_json::to_value(policy).unwrap();
            assert_eq!(val, expected_name);
            let roundtrip: RankOccupancyPolicy = serde_json::from_value(val).unwrap();
            assert_eq!(roundtrip, policy);
        }
        assert_eq!(RankOccupancyPolicy::from_word(4), None);

        let receipt = verify_rank_occupancy_controls();
        assert!(receipt.c_r1_exact_admission_equivalence_pass);
        assert!(receipt.c_r2_occupied_multiplicity_vs_crowd_hinge_pass);
        assert!(receipt.c_r3_virtual_vs_applied_identity_pass);
        assert!(receipt.c_r4_occupancy_under_k_bounds_pass);
        assert!(receipt.c_r5_checked_u64_overflow_status_12_pass);
        assert!(receipt.c_r6_k16_baseline_guard_pass);
        assert!(receipt.c_r7a_fixed_grid_vacation_invariance_pass);
        assert!(receipt.c_r7b_integer_crowding_hinge_above_quota_pass);
        assert!(receipt.c_r7c_move_time_g_occ_floor_and_cap_refusal_pass);
        assert!(receipt.c_r7d_a8_uniform_equivalence_and_within_cell_invariance_pass);
        assert!(receipt.all_controls_pass);
    }
}

pub const RANK_OCCUPANCY_CONTROL_SCHEMA: &str = "uor-r4.rank-occupancy-controls/v1";
pub const GRID_CELLS: usize = 256;
pub const A8_UNIFORM_CELL_QUOTA: u32 = 16;
pub const CROWD_HINGE_QUOTA: u32 = 24;
pub const MOVE_TIME_MIN_OCCUPIED_CELLS: u32 = 128;
pub const MOVE_TIME_MAX_CELL_TOKENS: u32 = 32;
pub const G_OCC_HARD_FLOOR_CELLS: u64 = 17;
pub const G_OCC_D39_MIN_CELLS: u64 = 32;
pub const OVERFLOW_STATUS_CODE: u32 = 12;
pub const INELIGIBLE_CANDIDATE_COST: u64 = u64::MAX;

/// Extended rank-occupancy objective policy covering `s[11] in {0, 1, 2, 3}`
/// for `r-rank-objective-keeps-cells` (`#2883`).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RankOccupancyPolicy {
    #[default]
    NearestAddressMarginV1,
    OccupiedCellRankHingeV1,
    FixedGridRankHingeV1,
    OccupiedCellRankCrowdHingeV1,
}

impl RankOccupancyPolicy {
    pub const fn word(self) -> u32 {
        match self {
            Self::NearestAddressMarginV1 => 0,
            Self::OccupiedCellRankHingeV1 => 1,
            Self::FixedGridRankHingeV1 => 2,
            Self::OccupiedCellRankCrowdHingeV1 => 3,
        }
    }

    pub const fn from_word(word: u32) -> Option<Self> {
        match word {
            0 => Some(Self::NearestAddressMarginV1),
            1 => Some(Self::OccupiedCellRankHingeV1),
            2 => Some(Self::FixedGridRankHingeV1),
            3 => Some(Self::OccupiedCellRankCrowdHingeV1),
            _ => None,
        }
    }

    pub const fn uses_fixed_grid_rank(self) -> bool {
        matches!(
            self,
            Self::FixedGridRankHingeV1 | Self::OccupiedCellRankCrowdHingeV1
        )
    }

    pub const fn uses_crowd_hinge(self) -> bool {
        matches!(self, Self::OccupiedCellRankCrowdHingeV1)
    }

    pub const fn enforces_move_time_g_occ(self) -> bool {
        matches!(
            self,
            Self::FixedGridRankHingeV1 | Self::OccupiedCellRankCrowdHingeV1
        )
    }
}

impl From<ObjectivePolicy> for RankOccupancyPolicy {
    fn from(policy: ObjectivePolicy) -> Self {
        match policy {
            ObjectivePolicy::NearestAddressMarginV1 => Self::NearestAddressMarginV1,
            ObjectivePolicy::OccupiedCellRankHingeV1 => Self::OccupiedCellRankHingeV1,
        }
    }
}

/// Exact lexicographic cell precedence `(distance, cell) < (target_distance, target_cell)`
/// matching `cell_precedes` in `discrete.cu`.
pub const fn cell_precedes(
    distance: u32,
    cell: u32,
    target_distance: u32,
    target_cell: u32,
) -> bool {
    distance < target_distance || (distance == target_distance && cell < target_cell)
}

/// Compute `(rank_occ, occupied_cells)` for `OccupiedCellRankHingeV1` (`s[11] = 1`).
pub fn occupied_cell_rank(
    cell_distances: &[u32; GRID_CELLS],
    occupied: &[bool; GRID_CELLS],
    target_cell: usize,
) -> (u32, u32) {
    let target_cell_u32 = u32::try_from(target_cell).unwrap_or(0);
    let target_dist = cell_distances[target_cell];
    let mut rank = 1u32;
    let mut occ = 0u32;
    for cell in 0..GRID_CELLS {
        if occupied[cell] {
            occ += 1;
            let cell_u32 = u32::try_from(cell).unwrap_or(0);
            if cell_precedes(cell_distances[cell], cell_u32, target_dist, target_cell_u32) {
                rank += 1;
            }
        }
    }
    (rank, occ)
}

/// Compute `rank_grid` over all 256 cells of the fixed 16x16 root-pair grid (`s[11] in {2, 3}`),
/// invariant to whether `occupied[cell]` is true or false (`C-R7a`).
pub fn fixed_grid_cell_rank(cell_distances: &[u32; GRID_CELLS], target_cell: usize) -> u32 {
    let target_cell_u32 = u32::try_from(target_cell).unwrap_or(0);
    let target_dist = cell_distances[target_cell];
    let mut rank = 1u32;
    for (cell, &dist) in cell_distances.iter().enumerate() {
        let cell_u32 = u32::try_from(cell).unwrap_or(0);
        if cell_precedes(dist, cell_u32, target_dist, target_cell_u32) {
            rank += 1;
        }
    }
    rank
}

/// Compute `max(0, rank - kept_cells)`.
pub const fn rank_hinge(rank: u32, kept_cells: u32) -> u32 {
    rank.saturating_sub(kept_cells)
}

/// Select the top `kept_cells` occupied cells ordered by `(cell_distances[c], c)`.
pub fn admitted_cells_k(
    cell_distances: &[u32; GRID_CELLS],
    occupied: &[bool; GRID_CELLS],
    kept_cells: usize,
) -> Vec<usize> {
    let mut pairs: Vec<(u32, usize)> = (0..GRID_CELLS)
        .filter(|&c| occupied[c])
        .map(|c| (cell_distances[c], c))
        .collect();
    pairs.sort_unstable();
    pairs
        .into_iter()
        .take(kept_cells)
        .map(|(_, cell)| cell)
        .collect()
}

/// Compute the integer cell-crowding hinge `C_crowd(row)` above `CROWD_HINGE_QUOTA = 24` (`C-R7b`).
pub fn cell_crowding_hinge(
    cell_counts: &[u32; GRID_CELLS],
    target_cell: usize,
    admitted_cells: &[usize],
) -> u64 {
    let target_excess = u64::from(cell_counts[target_cell].saturating_sub(CROWD_HINGE_QUOTA));
    let admitted_excess: u64 = admitted_cells
        .iter()
        .map(|&c| u64::from(cell_counts[c].saturating_sub(CROWD_HINGE_QUOTA)))
        .sum();
    target_excess.saturating_add(admitted_excess)
}

/// Enforce the move-time `G-Occ` invariant (`occupied_cells >= 128`, `n(c_new) <= 32`) on endpoint moves (`C-R7c`).
pub const fn move_time_g_occ_allowed(
    policy: RankOccupancyPolicy,
    is_endpoint_block_move: bool,
    candidate_occupied_cells: u32,
    candidate_dest_cell_tokens: u32,
) -> bool {
    if policy.enforces_move_time_g_occ() && is_endpoint_block_move {
        candidate_occupied_cells >= MOVE_TIME_MIN_OCCUPIED_CELLS
            && candidate_dest_cell_tokens <= MOVE_TIME_MAX_CELL_TOKENS
    } else {
        true
    }
}

/// Checked `u64` integer row-loss numerator returning `Err(OVERFLOW_STATUS_CODE)` (`12`) on overflow (`C-R5`).
pub fn checked_row_loss_numerator(
    policy: RankOccupancyPolicy,
    token_hinge_t: u64,
    address_hinge_a: u64,
    rank_occ_hinge: u64,
    rank_grid_hinge: u64,
    crowd_hinge: u64,
    blocks_b: u64,
) -> std::result::Result<u64, u32> {
    let w = 8u64.checked_mul(blocks_b).ok_or(OVERFLOW_STATUS_CODE)?;
    let term_t = 8u64
        .checked_mul(token_hinge_t)
        .ok_or(OVERFLOW_STATUS_CODE)?;
    match policy {
        RankOccupancyPolicy::NearestAddressMarginV1 => {
            let term_a = w.checked_mul(address_hinge_a).ok_or(OVERFLOW_STATUS_CODE)?;
            term_t.checked_add(term_a).ok_or(OVERFLOW_STATUS_CODE)
        }
        RankOccupancyPolicy::OccupiedCellRankHingeV1 => {
            let term_r = w.checked_mul(rank_occ_hinge).ok_or(OVERFLOW_STATUS_CODE)?;
            term_t.checked_add(term_r).ok_or(OVERFLOW_STATUS_CODE)
        }
        RankOccupancyPolicy::FixedGridRankHingeV1 => {
            let term_r = w.checked_mul(rank_grid_hinge).ok_or(OVERFLOW_STATUS_CODE)?;
            term_t.checked_add(term_r).ok_or(OVERFLOW_STATUS_CODE)
        }
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1 => {
            let term_r = w.checked_mul(rank_grid_hinge).ok_or(OVERFLOW_STATUS_CODE)?;
            let term_c = 8u64.checked_mul(crowd_hinge).ok_or(OVERFLOW_STATUS_CODE)?;
            term_t
                .checked_add(term_r)
                .and_then(|acc| acc.checked_add(term_c))
                .ok_or(OVERFLOW_STATUS_CODE)
        }
    }
}

/// Evaluate the registered `G-Occ` contract (`occupied_cells > 16`, `>= max(32, ceil(occ_A / 2))`,
/// `mean_admitted_tokens_x10 <= 2 * mean_A_x10`, `max_cell_tokens <= 32`).
/// Returns `(g_occ_pass, min_required_cells, max_allowed_mean_tokens_x10)`.
pub const fn evaluate_g_occ_contract(
    occupied_cells: u64,
    control_a_occupied_cells: u64,
    mean_admitted_tokens_x10: u64,
    control_a_mean_admitted_tokens_x10: u64,
    max_cell_tokens: u64,
) -> (bool, u64, u64) {
    let half_a_ceil = control_a_occupied_cells.div_ceil(2);
    let min_required_cells = if half_a_ceil > G_OCC_D39_MIN_CELLS {
        half_a_ceil
    } else {
        G_OCC_D39_MIN_CELLS
    };
    let max_allowed_mean_tokens_x10 = control_a_mean_admitted_tokens_x10.saturating_mul(2);
    let pass = occupied_cells >= G_OCC_HARD_FLOOR_CELLS
        && occupied_cells >= min_required_cells
        && mean_admitted_tokens_x10 <= max_allowed_mean_tokens_x10
        && max_cell_tokens <= (MOVE_TIME_MAX_CELL_TOKENS as u64);
    (pass, min_required_cells, max_allowed_mean_tokens_x10)
}

/// Structured receipt of the finite controls `C-R1..C-R6` and `C-R7a..C-R7d` for `#2883`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RankOccupancyControlReceipt {
    pub schema: String,
    pub c_r1_exact_admission_equivalence_pass: bool,
    pub c_r2_occupied_multiplicity_vs_crowd_hinge_pass: bool,
    pub c_r3_virtual_vs_applied_identity_pass: bool,
    pub c_r4_occupancy_under_k_bounds_pass: bool,
    pub c_r5_checked_u64_overflow_status_12_pass: bool,
    pub c_r6_k16_baseline_guard_pass: bool,
    pub c_r7a_fixed_grid_vacation_invariance_pass: bool,
    pub c_r7b_integer_crowding_hinge_above_quota_pass: bool,
    pub c_r7c_move_time_g_occ_floor_and_cap_refusal_pass: bool,
    pub c_r7d_a8_uniform_equivalence_and_within_cell_invariance_pass: bool,
    pub all_controls_pass: bool,
}

/// Execute all ten finite controls (`C-R1..C-R6`, `C-R7a..C-R7d`).
pub fn verify_rank_occupancy_controls() -> RankOccupancyControlReceipt {
    // C-R1: Exact tie-breaking and admission equivalence:
    // Ordered pairs (0,0), (4,1), (4,15), (4,16), (8,255) with target_cell = 15 -> rank = 3,
    // hinge for K = 1..=16 is [2, 1, 0, ..., 0]; all 256 cells tied at distance 4 with target 255 -> rank = 256, hinge(K=16) = 240.
    let mut dists_r1 = [100u32; GRID_CELLS];
    let mut occ_r1 = [false; GRID_CELLS];
    for (cell, dist) in [(0usize, 0u32), (1, 4), (15, 4), (16, 4), (255, 8)] {
        dists_r1[cell] = dist;
        occ_r1[cell] = true;
    }
    let (rank_r1, occ_count_r1) = occupied_cell_rank(&dists_r1, &occ_r1, 15);
    let expected_hinges: [u32; 16] = [2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let hinges_match = (1u32..=16)
        .zip(expected_hinges)
        .all(|(k, exp)| rank_hinge(rank_r1, k) == exp);
    let tied_dists = [4u32; GRID_CELLS];
    let rank_tied_255 = fixed_grid_cell_rank(&tied_dists, 255);
    let c_r1_pass = rank_r1 == 3
        && occ_count_r1 == 5
        && hinges_match
        && rank_tied_255 == 256
        && rank_hinge(rank_tied_255, 16) == 240;

    // C-R2: Under OccupiedCellRankHingeV1 (s[11]=1), crowding rival cell 1 (1, 2, 4, 40 tokens) leaves rank_occ == 3 unchanged;
    // adding a new preceding cell 2 at distance 4 raises rank_occ to 4.
    let mut occ_r2_new = occ_r1;
    let mut dists_r2_new = dists_r1;
    occ_r2_new[2] = true;
    dists_r2_new[2] = 4;
    let (rank_r2_after_new, _) = occupied_cell_rank(&dists_r2_new, &occ_r2_new, 15);
    let mut counts_r2 = [0u32; GRID_CELLS];
    for cell in [0usize, 1, 15, 16, 255] {
        counts_r2[cell] = 1;
    }
    let admitted_r2 = admitted_cells_k(&dists_r1, &occ_r1, 16);
    let crowd_at_4 = {
        let mut c = counts_r2;
        c[1] = 4;
        cell_crowding_hinge(&c, 15, &admitted_r2)
    };
    let crowd_at_40 = {
        let mut c = counts_r2;
        c[1] = 40;
        cell_crowding_hinge(&c, 15, &admitted_r2)
    };
    let c_r2_pass = rank_r1 == 3 && rank_r2_after_new == 4 && crowd_at_4 == 0 && crowd_at_40 == 16;

    // C-R3: Virtual candidate state vs independently applied state across 5 mutation classes
    // (create_cell, empty_cell, move_target_cell, interior_block_only, identity_candidate).
    let mut c_r3_pass = true;
    for mutation in 0..5u32 {
        let mut v_occ = occ_r1;
        let mut v_dists = dists_r1;
        let mut v_counts = counts_r2;
        match mutation {
            0 => {
                // create_cell
                v_occ[2] = true;
                v_dists[2] = 4;
                v_counts[16] = 0;
                v_occ[16] = false;
                v_counts[2] = 1;
            }
            1 => {
                // empty_cell (vacate cell 1 into cell 0)
                v_occ[1] = false;
                v_counts[1] = 0;
                v_counts[0] = 2;
            }
            2 => {
                // move_target_cell closer
                v_dists[15] = 2;
            }
            3 => {
                // interior_block_only (endpoint cells and cell distances invariant)
            }
            _ => {
                // identity_candidate
            }
        }
        let applied_occ = v_occ;
        let applied_dists = v_dists;
        let applied_counts = v_counts;
        let (v_rocc, v_occcnt) = occupied_cell_rank(&v_dists, &v_occ, 15);
        let (a_rocc, a_occcnt) = occupied_cell_rank(&applied_dists, &applied_occ, 15);
        let v_rgrid = fixed_grid_cell_rank(&v_dists, 15);
        let a_rgrid = fixed_grid_cell_rank(&applied_dists, 15);
        let v_adm = admitted_cells_k(&v_dists, &v_occ, 16);
        let a_adm = admitted_cells_k(&applied_dists, &applied_occ, 16);
        let v_crowd = cell_crowding_hinge(&v_counts, 15, &v_adm);
        let a_crowd = cell_crowding_hinge(&applied_counts, 15, &a_adm);
        if v_rocc != a_rocc || v_occcnt != a_occcnt || v_rgrid != a_rgrid || v_crowd != a_crowd {
            c_r3_pass = false;
        }
    }

    // C-R4: Occupancy under K bounds (1 occupied cell, K = 16 -> rank_occ = 1, hinge_occ = 0,
    // whereas fixed_grid_cell_rank still counts all preceding fixed-grid cells).
    let mut occ_one = [false; GRID_CELLS];
    occ_one[15] = true;
    let (rank_occ_one, occ_count_one) = occupied_cell_rank(&dists_r1, &occ_one, 15);
    let rank_grid_one = fixed_grid_cell_rank(&dists_r1, 15);
    let c_r4_pass = occ_count_one == 1
        && rank_occ_one == 1
        && rank_hinge(rank_occ_one, 16) == 0
        && rank_grid_one == 3;

    // C-R5: Checked u64 integer arithmetic and overflow status 12.
    let ok_a = checked_row_loss_numerator(
        RankOccupancyPolicy::NearestAddressMarginV1,
        10,
        5,
        0,
        0,
        0,
        4,
    );
    let ok_rocc = checked_row_loss_numerator(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        10,
        0,
        0,
        5,
        3,
        4,
    );
    let overflow_t = checked_row_loss_numerator(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        u64::MAX,
        0,
        0,
        0,
        0,
        4,
    );
    let overflow_r = checked_row_loss_numerator(
        RankOccupancyPolicy::FixedGridRankHingeV1,
        0,
        0,
        0,
        u64::MAX,
        0,
        4,
    );
    let overflow_c = checked_row_loss_numerator(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        0,
        0,
        0,
        0,
        u64::MAX,
        4,
    );
    let c_r5_pass = ok_a == Ok(8 * 10 + 32 * 5)
        && ok_rocc == Ok(8 * 10 + 32 * 5 + 8 * 3)
        && overflow_t == Err(OVERFLOW_STATUS_CODE)
        && overflow_r == Err(OVERFLOW_STATUS_CODE)
        && overflow_c == Err(OVERFLOW_STATUS_CODE);

    // C-R6: K=16 G-Occ and G-Adm contract checks on K16-R (collapse fails G-Occ) vs Stage T A8 (passes G-Occ and G-Adm).
    let (k16_r_s1_pass, k16_r_s1_min_cells, k16_r_s1_max_n_x10) =
        evaluate_g_occ_contract(20, 209, 34_039, 3_518, 3_400);
    let (k16_r_s2_pass, k16_r_s2_min_cells, k16_r_s2_max_n_x10) =
        evaluate_g_occ_contract(19, 192, 34_807, 3_761, 3_480);
    let (stage_t_a8_pass, stage_t_min_cells, stage_t_max_n_x10) =
        evaluate_g_occ_contract(256, 256, 2_560, 2_560, 16);
    let c_r6_pass = !k16_r_s1_pass
        && k16_r_s1_min_cells == 105
        && k16_r_s1_max_n_x10 == 7_036
        && !k16_r_s2_pass
        && k16_r_s2_min_cells == 96
        && k16_r_s2_max_n_x10 == 7_522
        && stage_t_a8_pass
        && stage_t_min_cells == 128
        && stage_t_max_n_x10 == 5_120;

    // C-R7a: Fixed-grid rank R_grid is strictly invariant when preceding cells are vacated,
    // whereas OccupiedCellRankHingeV1 (R_occ) drops by 1 per vacated preceding cell.
    let mut grad_dists = [0u32; GRID_CELLS];
    for (c, slot) in grad_dists.iter_mut().enumerate() {
        *slot = u32::try_from(c / 4).unwrap_or(0);
    }
    let occ_all = [true; GRID_CELLS];
    let target_c = 80usize; // distance = 20, preceded by cells 0..80 (80 preceding cells -> rank = 81, hinge(K=16) = 65)
    let (r_occ_before, _) = occupied_cell_rank(&grad_dists, &occ_all, target_c);
    let r_grid_before = fixed_grid_cell_rank(&grad_dists, target_c);
    let mut occ_vacated = occ_all;
    for slot in occ_vacated.iter_mut().take(30) {
        *slot = false;
    }
    let (r_occ_after, _) = occupied_cell_rank(&grad_dists, &occ_vacated, target_c);
    let r_grid_after = fixed_grid_cell_rank(&grad_dists, target_c);
    let c_r7a_pass = r_occ_before == 81
        && r_grid_before == 81
        && r_occ_after == 51
        && r_grid_after == 81
        && rank_hinge(r_grid_before, 16) == 65
        && rank_hinge(r_grid_after, 16) == 65;

    // C-R7b: Integer cell-crowding hinge C_crowd is 0 at/below quota 24 and +1 per token above 24
    // (weighted by +8 in the row loss numerator).
    let counts_a8 = [A8_UNIFORM_CELL_QUOTA; GRID_CELLS];
    let adm_16 = admitted_cells_k(&grad_dists, &occ_all, 16);
    let crowd_a8 = cell_crowding_hinge(&counts_a8, target_c, &adm_16);
    let mut counts_at_24 = counts_a8;
    counts_at_24[target_c] = CROWD_HINGE_QUOTA;
    counts_at_24[0] = CROWD_HINGE_QUOTA;
    let crowd_24 = cell_crowding_hinge(&counts_at_24, target_c, &adm_16);
    let mut counts_crowded = counts_a8;
    counts_crowded[target_c] = 27; // +3 excess in target_cell
    counts_crowded[0] = 29; // +5 excess in admitted[0]
    let crowd_excess = cell_crowding_hinge(&counts_crowded, target_c, &adm_16);
    let loss_uncrowded = checked_row_loss_numerator(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        12,
        0,
        65,
        65,
        crowd_a8,
        4,
    );
    let loss_crowded = checked_row_loss_numerator(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        12,
        0,
        65,
        65,
        crowd_excess,
        4,
    );
    let c_r7b_pass = crowd_a8 == 0
        && crowd_24 == 0
        && crowd_excess == 8
        && loss_crowded == loss_uncrowded.map(|v| v + 64);

    // C-R7c: Move-time G-Occ floor (occ >= 128) and per-cell cap (n(c_new) <= 32) refusal on endpoint moves.
    let allow_128_32 = move_time_g_occ_allowed(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        true,
        128,
        32,
    );
    let refuse_occ_127 = !move_time_g_occ_allowed(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        true,
        127,
        16,
    );
    let refuse_cap_33 = !move_time_g_occ_allowed(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        true,
        256,
        33,
    );
    let allow_interior = move_time_g_occ_allowed(
        RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
        false,
        256,
        16,
    );
    let c_r7c_pass = allow_128_32 && refuse_occ_127 && refuse_cap_33 && allow_interior;

    // C-R7d: Under AddressCellsA8 (all 256 cells occupied, n(c) = 16 <= 24):
    // 1. r_grid == r_occ and C_crowd == 0 for every cell 0..256;
    // 2. For every admitted target cell (rank <= 16), R_grid == 0 and R_occ == 0,
    //    so L_{R-occ}(row) = L_{R-grid}(row) = 8 * T(row), providing zero within-cell rank difference
    //    between any two tokens sharing the same endpoint cell target_cell.
    let mut a8_all_cells_match = true;
    let mut admitted_zero_rank_loss = true;
    for cell in 0..GRID_CELLS {
        let (r_occ_c, occ_c) = occupied_cell_rank(&grad_dists, &occ_all, cell);
        let r_grid_c = fixed_grid_cell_rank(&grad_dists, cell);
        let c_crowd_c = cell_crowding_hinge(&counts_a8, cell, &adm_16);
        if occ_c != 256 || r_occ_c != r_grid_c || c_crowd_c != 0 {
            a8_all_cells_match = false;
        }
        if r_grid_c <= 16 {
            let hg = u64::from(rank_hinge(r_grid_c, 16));
            let loss_t10 = checked_row_loss_numerator(
                RankOccupancyPolicy::OccupiedCellRankCrowdHingeV1,
                10,
                0,
                hg,
                hg,
                c_crowd_c,
                4,
            );
            if hg != 0 || loss_t10 != Ok(80) {
                admitted_zero_rank_loss = false;
            }
        }
    }
    let c_r7d_pass = a8_all_cells_match && admitted_zero_rank_loss;

    let all_controls_pass = c_r1_pass
        && c_r2_pass
        && c_r3_pass
        && c_r4_pass
        && c_r5_pass
        && c_r6_pass
        && c_r7a_pass
        && c_r7b_pass
        && c_r7c_pass
        && c_r7d_pass;

    RankOccupancyControlReceipt {
        schema: RANK_OCCUPANCY_CONTROL_SCHEMA.to_owned(),
        c_r1_exact_admission_equivalence_pass: c_r1_pass,
        c_r2_occupied_multiplicity_vs_crowd_hinge_pass: c_r2_pass,
        c_r3_virtual_vs_applied_identity_pass: c_r3_pass,
        c_r4_occupancy_under_k_bounds_pass: c_r4_pass,
        c_r5_checked_u64_overflow_status_12_pass: c_r5_pass,
        c_r6_k16_baseline_guard_pass: c_r6_pass,
        c_r7a_fixed_grid_vacation_invariance_pass: c_r7a_pass,
        c_r7b_integer_crowding_hinge_above_quota_pass: c_r7b_pass,
        c_r7c_move_time_g_occ_floor_and_cap_refusal_pass: c_r7c_pass,
        c_r7d_a8_uniform_equivalence_and_within_cell_invariance_pass: c_r7d_pass,
        all_controls_pass,
    }
}
