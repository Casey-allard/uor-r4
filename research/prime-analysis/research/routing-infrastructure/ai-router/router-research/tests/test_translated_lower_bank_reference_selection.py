import unittest

from tools import translated_lower_bank_reference_selection as audit


class TranslatedLowerBankReferenceSelectionTest(unittest.TestCase):
    def _historical_backfill_analysis(self, top1=0.0444, cand=0.18936552, online=0.079535615, amortized=0.105716386):
        return {
            "experiment_id": "inc0104_confirm",
            "route_stats": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "mean_test_top1_after": top1,
                    "mean_retrieval_candidate_fraction": cand,
                    "mean_online_total_per_repeat_sec": online,
                    "mean_amortized_total_per_repeat_sec": amortized,
                }
            ],
        }

    def _soft_bias_screen_analysis(self):
        return {
            "experiment_id": "inc0130_screen",
            "route_stats": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0464,
                    "mean_retrieval_candidate_fraction": 0.189016,
                    "mean_online_total_per_repeat_sec": 0.099554,
                    "mean_amortized_total_per_repeat_sec": 0.132991,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0548,
                    "mean_retrieval_candidate_fraction": 0.189016,
                    "mean_online_total_per_repeat_sec": 0.195365,
                    "mean_amortized_total_per_repeat_sec": 0.244294,
                },
            ],
        }

    def _soft_bias_confirm_analysis(
        self,
        near_hard_top1=0.0446,
        near_hard_amortized=0.089899906,
        balanced_top1=0.0464,
        balanced_amortized=0.094196657,
        quality_top1=0.0524,
        quality_amortized=0.141591979,
        focused_backfill_top1=0.0452,
        focused_backfill_amortized=1.999071781,
    ):
        return {
            "experiment_id": "inc0131_confirm",
            "route_stats": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
                    "mean_test_top1_after": near_hard_top1,
                    "mean_retrieval_candidate_fraction": 0.19332824,
                    "mean_online_total_per_repeat_sec": 0.067324427,
                    "mean_amortized_total_per_repeat_sec": near_hard_amortized,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
                    "mean_test_top1_after": balanced_top1,
                    "mean_retrieval_candidate_fraction": 0.19332824,
                    "mean_online_total_per_repeat_sec": 0.072979062,
                    "mean_amortized_total_per_repeat_sec": balanced_amortized,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
                    "mean_test_top1_after": quality_top1,
                    "mean_retrieval_candidate_fraction": 0.19332824,
                    "mean_online_total_per_repeat_sec": 0.112685969,
                    "mean_amortized_total_per_repeat_sec": quality_amortized,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "mean_test_top1_after": focused_backfill_top1,
                    "mean_retrieval_candidate_fraction": 0.1901496,
                    "mean_online_total_per_repeat_sec": 1.968045239,
                    "mean_amortized_total_per_repeat_sec": focused_backfill_amortized,
                },
                {
                    "route_id": "DENSE_Q01_T2500",
                    "mean_test_top1_after": 0.0520,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_online_total_per_repeat_sec": 0.125753198,
                    "mean_amortized_total_per_repeat_sec": 0.125753198,
                },
            ],
        }

    def test_build_payload_promotes_near_hard_and_splits_soft_bias_roles(self):
        payload = audit.build_payload(
            self._historical_backfill_analysis(),
            self._soft_bias_screen_analysis(),
            self._soft_bias_confirm_analysis(),
        )

        self.assertEqual(payload["selection_mode"], "near_hard_promoted_over_stale_backfill")
        self.assertEqual(payload["carry_forward_contract"], "single_lower_bank_default_plus_two_comparators")
        self.assertEqual(
            payload["default_systems_reference"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["balanced_quality_comparator"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["quality_first_comparator"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["stale_historical_comparator"]["classification"],
            "stale_historical_comparator",
        )
        self.assertGreater(
            payload["stale_historical_comparator"]["packet_scope_amortized_ratio"],
            4.0,
        )

    def test_build_payload_can_retain_historical_backfill_when_it_stays_stable_and_faster(self):
        payload = audit.build_payload(
            self._historical_backfill_analysis(amortized=0.0900, top1=0.0450),
            self._soft_bias_screen_analysis(),
            self._soft_bias_confirm_analysis(
                near_hard_top1=0.0444,
                near_hard_amortized=0.0940,
                balanced_top1=0.0460,
                balanced_amortized=0.0980,
                quality_top1=0.0510,
                quality_amortized=0.1400,
                focused_backfill_top1=0.0450,
                focused_backfill_amortized=0.0920,
            ),
        )

        self.assertEqual(payload["selection_mode"], "historical_backfill_retained")
        self.assertEqual(
            payload["default_systems_reference"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["stale_historical_comparator"]["classification"],
            "retained_default",
        )


if __name__ == "__main__":
    unittest.main()
