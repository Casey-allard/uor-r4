import unittest

from tools import translated_dual_anchor_task_side_extension as audit


class TranslatedDualAnchorTaskSideExtensionTest(unittest.TestCase):
    def _packet(self):
        return {
            "packet_id": "inc0116_packet",
            "packet_mode": "dual_anchor_default_packet",
            "sources": {"packet": "inc0116"},
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

    def _broader(self):
        return {
            "packet_id": "inc0116_packet",
            "overall_dense_frontier_claim": "upper-bank near-frontier; lower-bank systems-only",
            "overall_broader_comparison_read": "broader read",
            "default_packet_route_ids": [
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
            "inheritance_rules": ["rule 1", "rule 2"],
            "lower_anchor_default_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "baseline_route_id": "DENSE_Q01_T2500",
                "classification": "systems-only",
                "systems_verdict": "promote",
                "quality_read": "quality_negative",
                "top1_tolerance_gap_abs": 0.00744,
            },
            "upper_anchor_default_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "systems_verdict": "promote",
                "quality_read": "quality_near",
                "top1_tolerance_gap_abs": 0.00144,
            },
            "optional_upper_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "systems_verdict": "promote",
                "quality_read": "quality_near",
                "inclusion_mode": "optional",
            },
        }

    def test_build_payload_carries_dual_anchor_defaults_into_task_side_surface(self):
        payload = audit.build_payload(self._packet(), self._broader())

        self.assertEqual(payload["task_side_surface"], "real_task_comparison")
        self.assertEqual(
            payload["lower_anchor_default_task_side_read"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
        )
        self.assertEqual(
            payload["upper_anchor_default_task_side_read"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
        )
        self.assertIn("REAL_TASK_COMPARISON.md", payload["inheritance_rules"][-1])

    def test_build_payload_rejects_default_route_mismatch(self):
        broader = self._broader()
        broader["default_packet_route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(self._packet(), broader)


if __name__ == "__main__":
    unittest.main()
