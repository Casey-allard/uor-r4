import unittest

from tools import translated_dual_anchor_real_task_downstream_carry_forward as audit


class TranslatedDualAnchorRealTaskDownstreamCarryForwardTest(unittest.TestCase):
    def _comparison(self):
        return {
            "comparison_surface": "lm_proxy_real_task_downstream",
            "packet_id": "inc0121_packet",
            "packet_mode": "downstream_dual_anchor_real_task_default_packet",
            "default_downstream_route_ids": [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
            "optional_downstream_comparator_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "excluded_by_default_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "inheritance_rules": ["rule 1", "rule 2"],
            "lower_anchor_downstream_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "baseline_route_id": "DENSE_Q01_T2500",
                "classification": "systems-only",
                "quality_read": "quality_negative",
            },
            "upper_anchor_downstream_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
            },
            "optional_upper_downstream_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
            },
            "promotion_recommendation": {
                "lower_bank_default": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "verdict": "carry_as_systems_only_default",
                },
                "upper_bank_default": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "verdict": "carry_as_promoted_real_task_default",
                },
                "upper_bank_optional_comparator": {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "verdict": "optional_only",
                },
            },
        }

    def test_build_payload_carries_explicit_downstream_defaults(self):
        payload = audit.build_payload(self._comparison())

        self.assertEqual(
            payload["carry_forward_contract"],
            "dual_anchor_downstream_real_task_default_comparison",
        )
        self.assertEqual(
            payload["lower_bank_default"]["recommendation"]["verdict"],
            "carry_as_systems_only_default",
        )
        self.assertEqual(
            payload["upper_bank_default"]["recommendation"]["verdict"],
            "carry_as_promoted_real_task_default",
        )
        self.assertEqual(
            payload["optional_upper_bank_comparator"]["recommendation"]["verdict"],
            "optional_only",
        )

    def test_build_payload_rejects_unexpected_default_routes(self):
        comparison = self._comparison()
        comparison["default_downstream_route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(comparison)


if __name__ == "__main__":
    unittest.main()
