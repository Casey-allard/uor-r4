import unittest

from tools import translated_dual_anchor_real_task_packet_manifest as audit


class TranslatedDualAnchorRealTaskPacketManifestTest(unittest.TestCase):
    def _carry_forward(self):
        return {
            "carry_forward_contract": "dual_anchor_real_task_default_comparison",
            "comparison_surface": "lm_proxy_real_task",
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
            "inheritance_rules": ["rule 1", "rule 2"],
            "interpretation": "carry-forward read",
        }

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
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"
            ],
            "excluded_by_default_route_ids": [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
            "default_routes": [
                {"route_id": "DENSE_Q01_T2500", "anchor": "lower_bank", "role": "dense_baseline", "source_experiment_id": "inc0104"},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500", "anchor": "lower_bank", "role": "default_routed_systems_reference", "source_experiment_id": "inc0104"},
                {"route_id": "DENSE_Q01_T40000", "anchor": "upper_bank", "role": "dense_baseline", "source_experiment_id": "inc0113"},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000", "anchor": "upper_bank", "role": "promoted_dense_near_reference", "source_experiment_id": "inc0113"},
            ],
            "optional_comparators": [
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000", "anchor": "upper_bank", "role": "supporting_comparator", "source_experiment_id": "inc0113"}
            ],
            "excluded_by_default_routes": [
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500", "anchor": "lower_bank", "role": "nondefault_pruning_quality_reference"},
                {"route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000", "anchor": "upper_bank", "role": "supporting_comparator"},
            ],
        }

    def test_build_payload_carries_packet_forward(self):
        payload = audit.build_payload(self._carry_forward(), self._packet())

        self.assertEqual(
            payload["packet_mode"],
            "downstream_dual_anchor_real_task_default_packet",
        )
        self.assertEqual(
            payload["default_route_ids"],
            self._packet()["default_route_ids"],
        )
        self.assertIn("Downstream real-task branches should inherit", payload["inheritance_rules"][-1])

    def test_build_payload_rejects_default_route_mismatch(self):
        carry_forward = self._carry_forward()
        carry_forward["default_downstream_real_task_routes"]["route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            audit.build_payload(carry_forward, self._packet())


if __name__ == "__main__":
    unittest.main()
