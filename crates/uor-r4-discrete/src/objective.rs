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
}
