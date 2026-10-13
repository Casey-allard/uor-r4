import unittest

from tools import translated_dual_anchor_real_task_downstream_extension as audit


class TranslatedDualAnchorRealTaskDownstreamExtensionTest(unittest.TestCase):
    def _packet_manifest(self):
        return {
            "packet_id": "inc0121_packet",
            "packet_mode": "downstream_dual_anchor_real_task_default_packet",
            "interpretation": "packet manifest read",
            "inheritance_rules": ["rule 1", "rule 2"],
            "default_route_ids": [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
            "optional_comparator_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "excluded_by_default_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
        }

    def _carry_forward(self):
        return {
            "carry_forward_contract": "dual_anchor_real_task_default_comparison",
            "comparison_surface": "lm_proxy_real_task",
            "interpretation": "carry-forward read",
            "default_downstream_real_task_routes": {
                "route_ids": [
                    "DENSE_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "DENSE_Q01_T40000",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                ]
            },
            "optional_comparators": {
                "route_ids": ["CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"]
            },
            "excluded_by_default": {
                "route_ids": [
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                ]
            },
            "lower_bank_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "baseline_route_id": "DENSE_Q01_T2500",
                "classification": "systems-only",
                "quality_read": "quality_negative",
                "recommendation": {"verdict": "carry_as_systems_only_default"},
            },
            "upper_bank_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
                "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
            },
            "optional_upper_bank_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
                "recommendation": {"verdict": "optional_only"},
            },
        }

    def test_build_payload_carries_packet_manifest_into_downstream_extension(self):
        payload = audit.build_payload(self._packet_manifest(), self._carry_forward())

        self.assertEqual(payload["downstream_surface"], "lm_proxy_real_task_downstream")
        self.assertEqual(
            payload["lower_anchor_downstream_default"]["recommendation"]["verdict"],
            "carry_as_systems_only_default",
        )
        self.assertEqual(
            payload["upper_anchor_downstream_default"]["recommendation"]["verdict"],
            "carry_as_promoted_real_task_default",
        )
        self.assertIn("record any optional comparator reintroduction explicitly", payload["inheritance_rules"][-1])

    def test_build_payload_rejects_packet_default_mismatch(self):
        carry_forward = self._carry_forward()
        carry_forward["default_downstream_real_task_routes"]["route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(self._packet_manifest(), carry_forward)


if __name__ == "__main__":
    unittest.main()
