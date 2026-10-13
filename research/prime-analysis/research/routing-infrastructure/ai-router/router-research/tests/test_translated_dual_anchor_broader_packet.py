import unittest

from tools import translated_dual_anchor_broader_packet as packet


class TranslatedDualAnchorBroaderPacketTest(unittest.TestCase):
    def _contract(self):
        return {
            "carry_forward_contract": "promoted_upper_bank_single_reference",
            "selection_mode": "tie_break_within_tolerance",
            "promoted_upper_bank_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            "supporting_upper_bank_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            "default_broader_comparison_packet": {
                "route_ids": [
                    "DENSE_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "DENSE_Q01_T40000",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                ]
            },
            "optional_comparators": {
                "route_ids": [
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"
                ]
            },
            "excluded_by_default": {
                "route_ids": [
                    "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                ]
            },
            "inclusion_rules": ["rule a", "rule b"],
            "sources": {"contract": "inc0115"},
        }

    def _lower_config(self):
        return {
            "experiment_id": "inc0104_confirm",
            "common_args": {
                "query_repeats": 1,
                "event_gate_mode": "off",
                "phase_field_lambda": 1.5,
            },
            "routes": [
                {
                    "route_id": "DENSE_Q01_T2500",
                    "args": {"retrieval_backend": "dense_exact", "max_train": 2500},
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "args": {"max_train": 2500, "event_gate_mode": "soft_error", "complex_backfill_items": 2},
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                    "args": {"max_train": 2500, "event_gate_mode": "soft_error"},
                },
            ],
        }

    def _upper_config(self):
        return {
            "experiment_id": "inc0113_confirm",
            "common_args": {
                "query_repeats": 1,
                "event_gate_mode": "off",
                "phase_field_lambda": 1.5,
            },
            "routes": [
                {
                    "route_id": "DENSE_Q01_T40000",
                    "args": {"retrieval_backend": "dense_exact", "max_train": 40000},
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "args": {"max_train": 40000, "event_gate_mode": "soft_error"},
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "args": {"max_train": 40000, "event_gate_mode": "soft_error", "complex_backfill_items": 2},
                },
            ],
        }

    def test_build_payload_freezes_dual_anchor_default_routes(self):
        payload = packet.build_payload(
            self._contract(),
            self._lower_config(),
            self._upper_config(),
            lower_config_path="lower.json",
            upper_config_path="upper.json",
        )

        self.assertEqual(
            payload["default_route_ids"],
            [
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                "DENSE_Q01_T40000",
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
            ],
        )
        self.assertEqual(
            payload["optional_comparator_route_ids"],
            ["CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000"],
        )
        self.assertEqual(
            payload["excluded_by_default_route_ids"],
            [
                "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
            ],
        )
        self.assertEqual(payload["default_routes"][1]["resolved_args"]["complex_backfill_items"], 2)
        self.assertEqual(payload["default_routes"][3]["resolved_args"]["max_train"], 40000)

    def test_build_payload_rejects_unexpected_default_packet(self):
        contract = self._contract()
        contract["default_broader_comparison_packet"]["route_ids"] = ["wrong"]

        with self.assertRaises(ValueError):
            packet.build_payload(
                contract,
                self._lower_config(),
                self._upper_config(),
                lower_config_path="lower.json",
                upper_config_path="upper.json",
            )


if __name__ == "__main__":
    unittest.main()
