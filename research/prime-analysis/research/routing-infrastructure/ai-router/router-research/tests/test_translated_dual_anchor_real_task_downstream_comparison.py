import unittest

from tools import translated_dual_anchor_real_task_downstream_comparison as audit


class TranslatedDualAnchorRealTaskDownstreamComparisonTest(unittest.TestCase):
    def _packet_manifest(self):
        return {
            "packet_id": "inc0121_packet",
            "packet_mode": "downstream_dual_anchor_real_task_default_packet",
            "interpretation": "packet manifest read",
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

    def _downstream_extension(self):
        return {
            "downstream_surface": "lm_proxy_real_task_downstream",
            "downstream_extension_read": "downstream extension read",
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
            "lower_anchor_downstream_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "baseline_route_id": "DENSE_Q01_T2500",
                "classification": "systems-only",
                "quality_read": "quality_negative",
                "recommendation": {"verdict": "carry_as_systems_only_default"},
            },
            "upper_anchor_downstream_default": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
                "recommendation": {"verdict": "carry_as_promoted_real_task_default"},
            },
            "optional_upper_downstream_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "quality_read": "quality_near",
                "recommendation": {"verdict": "optional_only"},
            },
        }

    def test_build_payload_sets_downstream_recommendations(self):
        payload = audit.build_payload(self._packet_manifest(), self._downstream_extension())

        self.assertEqual(payload["comparison_surface"], "lm_proxy_real_task_downstream")
        self.assertEqual(
            payload["promotion_recommendation"]["lower_bank_default"]["verdict"],
            "carry_as_systems_only_default",
        )
        self.assertEqual(
            payload["promotion_recommendation"]["upper_bank_default"]["verdict"],
            "carry_as_promoted_real_task_default",
        )
        self.assertEqual(
            payload["optional_upper_downstream_comparator"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
        )

    def test_build_payload_rejects_optional_comparator_mismatch(self):
        downstream_extension = self._downstream_extension()
        downstream_extension["optional_downstream_comparator_route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(self._packet_manifest(), downstream_extension)


if __name__ == "__main__":
    unittest.main()
