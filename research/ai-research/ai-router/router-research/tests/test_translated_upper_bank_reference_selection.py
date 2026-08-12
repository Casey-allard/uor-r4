import unittest

from tools import translated_upper_bank_reference_selection as audit


class TranslatedUpperBankReferenceSelectionTest(unittest.TestCase):
    def _quality_analysis(self):
        return {
            "comparisons": [
                {
                    "label": "upper_dense_vs_soft_sparse",
                    "classification": "quality-near systems promotion",
                },
                {
                    "label": "upper_dense_vs_backfill",
                    "classification": "quality-near systems promotion",
                },
            ]
        }

    def _gap_analysis(self):
        return {
            "comparisons": [
                {
                    "label": "upper_dense_vs_soft_sparse",
                    "next_branch_bias": "operationally_negligible",
                },
                {
                    "label": "upper_dense_vs_backfill",
                    "next_branch_bias": "operationally_negligible",
                },
            ]
        }

    def _confirm_analysis(self, soft_top1=0.047325, soft_cand=0.183764, soft_amortized=3.426262,
                         backfill_top1=0.0472875, backfill_cand=0.182003, backfill_amortized=3.470096):
        return {
            "experiment_id": "inc0113_confirm",
            "route_stats": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "mean_test_top1_after": soft_top1,
                    "mean_retrieval_candidate_fraction": soft_cand,
                    "mean_amortized_total_per_repeat_sec": soft_amortized,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "mean_test_top1_after": backfill_top1,
                    "mean_retrieval_candidate_fraction": backfill_cand,
                    "mean_amortized_total_per_repeat_sec": backfill_amortized,
                },
            ],
        }

    def test_build_payload_promotes_soft_sparse_within_tolerance_tie_break(self):
        payload = audit.build_payload(
            self._quality_analysis(),
            self._gap_analysis(),
            self._confirm_analysis(),
        )

        self.assertEqual(payload["carry_forward_contract"], "single_promoted_reference")
        self.assertEqual(payload["selection_mode"], "tie_break_within_tolerance")
        self.assertEqual(payload["promoted_route_id"], "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000")
        self.assertEqual(payload["supporting_route_id"], "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000")

    def test_build_payload_keeps_pair_when_backfill_pruning_advantage_exceeds_tolerance(self):
        payload = audit.build_payload(
            self._quality_analysis(),
            self._gap_analysis(),
            self._confirm_analysis(backfill_cand=0.1790, backfill_amortized=3.55),
        )

        self.assertEqual(payload["carry_forward_contract"], "explicit_pair")
        self.assertEqual(payload["selection_mode"], "keep_pair")
        self.assertEqual(payload["promoted_route_id"], "")


if __name__ == "__main__":
    unittest.main()
