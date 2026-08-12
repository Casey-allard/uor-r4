import unittest

from tools import translated_dual_anchor_broader_comparison as audit


class TranslatedDualAnchorBroaderComparisonTest(unittest.TestCase):
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
            "default_routes": [
                {"route_id": "DENSE_Q01_T2500", "anchor": "lower_bank", "role": "dense_baseline"},
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "anchor": "lower_bank",
                    "role": "default_routed_systems_reference",
                },
                {"route_id": "DENSE_Q01_T40000", "anchor": "upper_bank", "role": "dense_baseline"},
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "anchor": "upper_bank",
                    "role": "promoted_dense_near_reference",
                },
            ],
            "optional_comparators": [
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "anchor": "upper_bank",
                    "role": "supporting_comparator",
                    "inclusion_mode": "optional",
                }
            ],
            "inheritance_rules": ["rule 1", "rule 2"],
        }

    def _frontier(self):
        robust = {
            "candidate_fraction_delta": {"median": -0.81, "robust_sign": "robust_improvement"},
            "amortized_delta_candidate_minus_baseline_sec": {"median": -7.2, "robust_sign": "robust_improvement"},
        }
        return {
            "overall_dense_frontier_claim": "upper-bank near-frontier; lower-bank systems-only",
            "source_robust_analysis": ["inc0111"],
            "comparisons": [
                {
                    "label": "lower_dense_vs_backfill",
                    "baseline_route_id": "DENSE_Q01_T2500",
                    "candidate_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500",
                    "classification": "systems-only",
                    "systems_verdict": "promote",
                    "quality_read": "quality_negative",
                    "interpretation": "lower read",
                    "top1_gap_abs_for_tolerance": 0.00744,
                    "robust_summaries": robust,
                },
                {
                    "label": "upper_dense_vs_soft_sparse",
                    "baseline_route_id": "DENSE_Q01_T40000",
                    "candidate_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
                    "classification": "quality-near systems promotion",
                    "systems_verdict": "promote",
                    "quality_read": "quality_near",
                    "interpretation": "upper read",
                    "top1_gap_abs_for_tolerance": 0.00144,
                    "robust_summaries": robust,
                },
                {
                    "label": "upper_dense_vs_backfill",
                    "baseline_route_id": "DENSE_Q01_T40000",
                    "candidate_route_id": "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
                    "classification": "quality-near systems promotion",
                    "systems_verdict": "promote",
                    "quality_read": "quality_near",
                    "interpretation": "upper comparator read",
                    "top1_gap_abs_for_tolerance": 0.00147,
                    "robust_summaries": robust,
                },
            ],
        }

    def test_build_payload_uses_default_packet_routes_for_lower_and_upper_reads(self):
        payload = audit.build_payload(self._packet(), self._frontier())

        self.assertEqual(payload["lower_anchor_default_read"]["route_id"], "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T2500")
        self.assertEqual(payload["lower_anchor_default_read"]["classification"], "systems-only")
        self.assertEqual(payload["upper_anchor_default_read"]["route_id"], "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000")
        self.assertEqual(payload["upper_anchor_default_read"]["classification"], "quality-near systems promotion")
        self.assertEqual(payload["optional_upper_comparator"]["route_id"], "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000")

    def test_build_payload_rejects_packet_frontier_route_mismatch(self):
        frontier = self._frontier()
        frontier["comparisons"][1]["candidate_route_id"] = "wrong"

        with self.assertRaises(ValueError):
            audit.build_payload(self._packet(), frontier)


if __name__ == "__main__":
    unittest.main()
