import unittest

from tools import translated_promoted_carry_forward_contract as audit


class TranslatedPromotedCarryForwardContractTest(unittest.TestCase):
    def _selection_analysis(self):
        return {
            "carry_forward_contract": "single_promoted_reference",
            "selection_mode": "tie_break_within_tolerance",
            "promoted_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            "supporting_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            "pair_deltas": {
                "soft_minus_backfill_top1": 0.000038,
                "soft_minus_backfill_candidate_fraction": 0.001761,
                "soft_minus_backfill_amortized_per_repeat_sec": -0.043834,
            },
            "sources": {"selection_analysis": "inc0114"},
        }

    def _lower_confirm(self):
        return {
            "experiment_id": "inc0104_confirm",
            "route_stats": [
                {
                    "route_id": "DENSE_Q01_T2500",
                    "mean_test_top1_after": 0.0520,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_amortized_total_per_repeat_sec": 0.125365,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0444,
                    "mean_retrieval_candidate_fraction": 0.189366,
                    "mean_amortized_total_per_repeat_sec": 0.105716,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0446,
                    "mean_retrieval_candidate_fraction": 0.193328,
                    "mean_amortized_total_per_repeat_sec": 0.152708,
                },
            ],
        }

    def _upper_confirm(self):
        return {
            "experiment_id": "inc0113_confirm",
            "route_stats": [
                {
                    "route_id": "DENSE_Q01_T40000",
                    "mean_test_top1_after": 0.04885,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_amortized_total_per_repeat_sec": 16.770385,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "mean_test_top1_after": 0.047325,
                    "mean_retrieval_candidate_fraction": 0.183764,
                    "mean_amortized_total_per_repeat_sec": 3.426262,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "mean_test_top1_after": 0.0472875,
                    "mean_retrieval_candidate_fraction": 0.182003,
                    "mean_amortized_total_per_repeat_sec": 3.470096,
                },
            ],
        }

    def test_build_payload_uses_promoted_upper_bank_reference_and_systems_only_lower_bank(self):
        payload = audit.build_payload(
            self._selection_analysis(),
            self._lower_confirm(),
            self._upper_confirm(),
        )

        self.assertEqual(payload["carry_forward_contract"], "promoted_upper_bank_single_reference")
        self.assertEqual(
            payload["promoted_upper_bank_route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
        )
        self.assertEqual(
            payload["supporting_upper_bank_route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
        )
        self.assertEqual(
            payload["default_broader_comparison_packet"]["route_ids"],
            [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
        )
        self.assertEqual(
            payload["excluded_by_default"]["route_ids"],
            [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
        )

    def test_build_payload_requires_single_promoted_upper_bank_reference(self):
        selection = self._selection_analysis()
        selection["carry_forward_contract"] = "explicit_pair"

        with self.assertRaises(ValueError):
            audit.build_payload(selection, self._lower_confirm(), self._upper_confirm())


if __name__ == "__main__":
    unittest.main()
