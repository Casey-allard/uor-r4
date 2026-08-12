import unittest

from tools import translated_lower_bank_contract_refresh as audit


class TranslatedLowerBankContractRefreshTest(unittest.TestCase):
    def _selection(self):
        return {
            "sources": {"soft_bias_confirm_analysis": "inc0131_confirm"},
            "default_systems_reference": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
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
        }

    def _broader_packet(self):
        return {
            "packet_id": "inc0116_packet",
            "default_route_ids": [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
            "optional_comparator_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
        }

    def _task_side_extension(self):
        return {
            "task_side_surface": "lm_proxy_real_task",
            "upper_anchor_task_side_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "upper default",
                "recommendation": {"verdict": "carry_as_task_side_default"},
            },
        }

    def _real_task_carry_forward(self):
        return {
            "carry_forward_contract": "dual_anchor_real_task_default_comparison",
            "upper_bank_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality-near",
                "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
            },
            "optional_upper_bank_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality-near",
                "recommendation": {"verdict": "optional_only"},
            },
        }

    def _downstream_extension(self):
        return {
            "downstream_surface": "lm_proxy_real_task_downstream",
            "upper_anchor_downstream_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality-near",
                "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
            },
            "optional_upper_downstream_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality-near",
                "recommendation": {"verdict": "optional_only"},
            },
        }

    def _downstream_carry_forward(self):
        return {
            "carry_forward_contract": "dual_anchor_downstream_real_task_default_comparison",
            "upper_bank_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality-near",
                "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
            },
        }

    def test_build_payload_refreshes_lower_bank_default_across_surfaces(self):
        payload = audit.build_payload(
            self._selection(),
            self._broader_packet(),
            self._task_side_extension(),
            self._real_task_carry_forward(),
            self._downstream_extension(),
            self._downstream_carry_forward(),
        )

        self.assertEqual(
            payload["broader_comparison_contract"]["default_route_ids"],
            [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
        )
        self.assertEqual(
            payload["task_side_contract"]["default_route_ids"][1],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["downstream_contract"]["default_route_ids"][1],
            "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["broader_comparison_contract"]["optional_comparator_route_ids"],
            [
                "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI030_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_TAU002_SBI080_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
        )
        self.assertIn(
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
            payload["broader_comparison_contract"]["excluded_by_default_route_ids"],
        )

    def test_build_payload_rejects_upper_bank_drift(self):
        downstream_carry_forward = self._downstream_carry_forward()
        downstream_carry_forward["upper_bank_default"]["route_id"] = "WRONG"

        with self.assertRaises(ValueError):
            audit.build_payload(
                self._selection(),
                self._broader_packet(),
                self._task_side_extension(),
                self._real_task_carry_forward(),
                self._downstream_extension(),
                downstream_carry_forward,
            )


if __name__ == "__main__":
    unittest.main()
