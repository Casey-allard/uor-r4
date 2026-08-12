import unittest

from tools import translated_dual_anchor_real_task_refresh_comparison as audit


class TranslatedDualAnchorRealTaskRefreshComparisonTest(unittest.TestCase):
    def _contract(self):
        return {
            "contract_id": "inc0133_contract",
            "refresh_mode": "single_contract_refresh_from_inc0132",
            "interpretation": "contract read",
            "inheritance_rules": ["rule 1", "rule 2"],
            "lower_bank_contract": {
                "default_systems_reference": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
                    "baseline_route_id": "DENSE_Q01_T2500",
                },
                "balanced_quality_comparator": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
                },
                "quality_first_comparator": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
                },
                "stale_historical_comparator": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                },
            },
            "upper_bank_contract": {
                "default_reference": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "baseline_route_id": "DENSE_Q01_T40000",
                    "classification": "quality-near systems promotion",
                    "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
                },
                "optional_comparator": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "classification": "quality-near systems promotion",
                    "recommendation": {"verdict": "optional_only"},
                },
            },
            "real_task_contract": {
                "default_route_ids": [
                    "DENSE_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
                    "DENSE_Q01_T40000",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                ],
                "optional_comparator_route_ids": [
                    "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                ],
                "excluded_by_default_route_ids": [
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                ],
            },
        }

    def _lower_confirm(self):
        return {
            "experiment_id": "inc0131_confirm",
            "route_stats": [
                {"route_id": "DENSE_Q01_T2500", "mean_test_top1_after": 0.0520, "mean_retrieval_candidate_fraction": 1.0, "mean_online_total_per_repeat_sec": 0.1258, "mean_amortized_total_per_repeat_sec": 0.1258},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500", "mean_test_top1_after": 0.0446, "mean_retrieval_candidate_fraction": 0.1933, "mean_online_total_per_repeat_sec": 0.0673, "mean_amortized_total_per_repeat_sec": 0.0899},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500", "mean_test_top1_after": 0.0464, "mean_retrieval_candidate_fraction": 0.1933, "mean_online_total_per_repeat_sec": 0.0730, "mean_amortized_total_per_repeat_sec": 0.0942},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500", "mean_test_top1_after": 0.0524, "mean_retrieval_candidate_fraction": 0.1933, "mean_online_total_per_repeat_sec": 0.1127, "mean_amortized_total_per_repeat_sec": 0.1416},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500", "mean_test_top1_after": 0.0452, "mean_retrieval_candidate_fraction": 0.1901, "mean_online_total_per_repeat_sec": 1.9680, "mean_amortized_total_per_repeat_sec": 1.9991},
            ],
        }

    def _upper_confirm(self):
        return {
            "experiment_id": "inc0113_confirm",
            "route_stats": [
                {"route_id": "DENSE_Q01_T40000", "mean_test_top1_after": 0.04885, "mean_retrieval_candidate_fraction": 1.0, "mean_online_total_per_repeat_sec": 16.7704, "mean_amortized_total_per_repeat_sec": 16.7704},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000", "mean_test_top1_after": 0.047325, "mean_retrieval_candidate_fraction": 0.183764, "mean_online_total_per_repeat_sec": 3.1358, "mean_amortized_total_per_repeat_sec": 3.4263},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000", "mean_test_top1_after": 0.0472875, "mean_retrieval_candidate_fraction": 0.182003, "mean_online_total_per_repeat_sec": 3.1090, "mean_amortized_total_per_repeat_sec": 3.4701},
            ],
        }

    def _upper_selection(self):
        return {
            "selection_mode": "tie_break_within_tolerance",
            "interpretation": "upper selection read",
        }

    def test_build_payload_reaffirms_refreshed_default_and_comparators(self):
        payload = audit.build_payload(
            self._contract(),
            self._lower_confirm(),
            self._upper_confirm(),
            self._upper_selection(),
        )

        self.assertEqual(payload["comparison_surface"], "lm_proxy_real_task_refreshed")
        self.assertEqual(
            payload["lower_anchor_default"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["lower_anchor_balanced_quality_comparator"]["recommendation"]["verdict"],
            "optional_balanced_quality_comparator",
        )
        self.assertAlmostEqual(
            payload["lower_anchor_quality_first_comparator"]["delta_vs_dense"]["top1"],
            0.0004,
            places=7,
        )
        self.assertEqual(
            payload["upper_anchor_default"]["recommendation"]["verdict"],
            "carry_as_promoted_real_task_default",
        )

    def test_build_payload_rejects_default_route_mismatch(self):
        contract = self._contract()
        contract["real_task_contract"]["default_route_ids"][1] = "WRONG"

        with self.assertRaises(ValueError):
            audit.build_payload(
                contract,
                self._lower_confirm(),
                self._upper_confirm(),
                self._upper_selection(),
            )


if __name__ == "__main__":
    unittest.main()
