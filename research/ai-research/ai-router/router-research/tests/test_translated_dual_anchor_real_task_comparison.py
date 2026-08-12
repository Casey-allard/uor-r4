import unittest

from tools import translated_dual_anchor_real_task_comparison as audit


class TranslatedDualAnchorRealTaskComparisonTest(unittest.TestCase):
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

    def _task_side(self):
        return {
            "packet_id": "inc0116_packet",
            "task_side_surface": "real_task_comparison",
            "task_side_extension_read": "task-side read",
            "default_task_side_route_ids": [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
            "optional_task_side_comparator_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "excluded_by_default_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "inheritance_rules": ["rule 1", "rule 2"],
            "lower_anchor_default_task_side_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "baseline_route_id": "DENSE_Q01_T2500",
                "classification": "systems-only",
                "systems_verdict": "promote",
                "quality_read": "quality_negative",
                "top1_tolerance_gap_abs": 0.00744,
            },
            "upper_anchor_default_task_side_read": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                "baseline_route_id": "DENSE_Q01_T40000",
                "classification": "quality-near systems promotion",
                "systems_verdict": "promote",
                "quality_read": "quality_near",
                "top1_tolerance_gap_abs": 0.00144,
            },
            "optional_upper_task_side_comparator": {
                "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                "classification": "quality-near systems promotion",
                "systems_verdict": "promote",
                "quality_read": "quality_near",
                "inclusion_mode": "optional",
            },
        }

    def test_build_payload_sets_real_task_recommendations(self):
        payload = audit.build_payload(self._packet(), self._task_side())

        self.assertEqual(payload["comparison_surface"], "lm_proxy_real_task")
        self.assertEqual(
            payload["promotion_recommendation"]["lower_bank_default"]["verdict"],
            "carry_as_systems_only_default",
        )
        self.assertEqual(
            payload["promotion_recommendation"]["upper_bank_default"]["verdict"],
            "carry_as_promoted_real_task_default",
        )
        self.assertEqual(
            payload["optional_upper_real_task_comparator"]["route_id"],
            "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
        )

    def test_build_payload_rejects_optional_comparator_mismatch(self):
        task_side = self._task_side()
        task_side["optional_task_side_comparator_route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(self._packet(), task_side)


if __name__ == "__main__":
    unittest.main()
