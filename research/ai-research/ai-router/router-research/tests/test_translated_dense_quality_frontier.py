import unittest

from tools import translated_dense_quality_frontier as audit


class TranslatedDenseQualityFrontierTest(unittest.TestCase):
    def _row(self, route_id, top1, cand_frac, amortized):
        return {
            "route_id": route_id,
            "mean_test_top1_after": top1,
            "mean_retrieval_candidate_fraction": cand_frac,
            "mean_amortized_total_per_repeat_sec": amortized,
        }

    def test_build_payload_classifies_lower_and_upper_frontier_reads(self):
        robust_analysis = {
            "comparisons": [
                {
                    "label": "lower_dense_vs_soft_sparse",
                    "systems_verdict": "pruning_only",
                    "interpretation": "lower soft read",
                    "robust_summaries": {
                        "top1_delta_candidate_minus_baseline": {
                            "median": -0.0072,
                            "trimmed_mean": -0.00736,
                            "robust_sign": "robust_regression",
                        },
                        "candidate_fraction_delta": {
                            "median": -0.8068,
                            "trimmed_mean": -0.8067,
                            "robust_sign": "robust_improvement",
                        },
                        "amortized_delta_candidate_minus_baseline_sec": {
                            "median": -0.0042,
                            "trimmed_mean": 0.0039,
                            "robust_sign": "mixed",
                        },
                    },
                },
                {
                    "label": "lower_dense_vs_backfill",
                    "systems_verdict": "promote",
                    "interpretation": "lower backfill read",
                    "robust_summaries": {
                        "top1_delta_candidate_minus_baseline": {
                            "median": -0.0068,
                            "trimmed_mean": -0.00744,
                            "robust_sign": "robust_regression",
                        },
                        "candidate_fraction_delta": {
                            "median": -0.8113,
                            "trimmed_mean": -0.8108,
                            "robust_sign": "robust_improvement",
                        },
                        "amortized_delta_candidate_minus_baseline_sec": {
                            "median": -0.0005,
                            "trimmed_mean": -0.0141,
                            "robust_sign": "robust_improvement",
                        },
                    },
                },
                {
                    "label": "upper_dense_vs_soft_sparse",
                    "systems_verdict": "promote",
                    "interpretation": "upper soft read",
                    "robust_summaries": {
                        "top1_delta_candidate_minus_baseline": {
                            "median": -0.0011,
                            "trimmed_mean": -0.00144,
                            "robust_sign": "robust_regression",
                        },
                        "candidate_fraction_delta": {
                            "median": -0.8130,
                            "trimmed_mean": -0.8156,
                            "robust_sign": "robust_improvement",
                        },
                        "amortized_delta_candidate_minus_baseline_sec": {
                            "median": -7.2076,
                            "trimmed_mean": -7.3189,
                            "robust_sign": "robust_improvement",
                        },
                    },
                },
                {
                    "label": "upper_dense_vs_backfill",
                    "systems_verdict": "promote",
                    "interpretation": "upper backfill read",
                    "robust_summaries": {
                        "top1_delta_candidate_minus_baseline": {
                            "median": -0.0011,
                            "trimmed_mean": -0.00147,
                            "robust_sign": "robust_regression",
                        },
                        "candidate_fraction_delta": {
                            "median": -0.8151,
                            "trimmed_mean": -0.8174,
                            "robust_sign": "robust_improvement",
                        },
                        "amortized_delta_candidate_minus_baseline_sec": {
                            "median": -7.0808,
                            "trimmed_mean": -7.3976,
                            "robust_sign": "robust_improvement",
                        },
                    },
                },
            ]
        }
        anchor_analyses = {
            "lower": {
                "experiment_id": "lower_confirm",
                "route_stats": [
                    self._row("DENSE_Q01_T2500", 0.0503, 1.0, 0.1078),
                    self._row("SOFT_T2500", 0.0446, 0.1933, 0.1527),
                    self._row("BACKFILL_T2500", 0.0444, 0.1894, 0.1057),
                ],
            },
            "upper": {
                "experiment_id": "upper_confirm",
                "route_stats": [
                    self._row("DENSE_Q01_T40000", 0.04885, 1.0, 11.7608),
                    self._row("SOFT_T40000", 0.047325, 0.1838, 3.5334),
                    self._row("BACKFILL_T40000", 0.0472875, 0.1820, 3.4702),
                ],
            },
        }
        specs = [
            {
                "label": "lower_dense_vs_soft_sparse",
                "anchor_key": "lower",
                "baseline_route_id": "DENSE_Q01_T2500",
                "candidate_route_id": "SOFT_T2500",
            },
            {
                "label": "lower_dense_vs_backfill",
                "anchor_key": "lower",
                "baseline_route_id": "DENSE_Q01_T2500",
                "candidate_route_id": "BACKFILL_T2500",
            },
            {
                "label": "upper_dense_vs_soft_sparse",
                "anchor_key": "upper",
                "baseline_route_id": "DENSE_Q01_T40000",
                "candidate_route_id": "SOFT_T40000",
            },
            {
                "label": "upper_dense_vs_backfill",
                "anchor_key": "upper",
                "baseline_route_id": "DENSE_Q01_T40000",
                "candidate_route_id": "BACKFILL_T40000",
            },
        ]

        payload = audit.build_payload(
            robust_analysis,
            anchor_analyses,
            specs,
            quality_tolerance=0.002,
        )

        by_label = {row["label"]: row for row in payload["comparisons"]}
        self.assertEqual(by_label["lower_dense_vs_soft_sparse"]["classification"], "pruning-only")
        self.assertEqual(by_label["lower_dense_vs_backfill"]["classification"], "systems-only")
        self.assertEqual(by_label["upper_dense_vs_soft_sparse"]["classification"], "quality-near systems promotion")
        self.assertEqual(by_label["upper_dense_vs_backfill"]["classification"], "quality-near systems promotion")
        self.assertEqual(payload["lower_bank_read"], "systems_only")
        self.assertEqual(payload["upper_bank_read"], "near_frontier")
        self.assertEqual(payload["overall_dense_frontier_claim"], "upper-bank near-frontier; lower-bank systems-only")

    def test_build_payload_supports_upper_only_frontier_hardening(self):
        robust_analysis = {
            "comparisons": [
                {
                    "label": "upper_dense_vs_soft_sparse",
                    "systems_verdict": "promote",
                    "interpretation": "upper soft read",
                    "robust_summaries": {
                        "top1_delta_candidate_minus_baseline": {
                            "median": -0.0010,
                            "trimmed_mean": -0.0013,
                            "robust_sign": "robust_regression",
                        },
                        "candidate_fraction_delta": {
                            "median": -0.81,
                            "trimmed_mean": -0.812,
                            "robust_sign": "robust_improvement",
                        },
                        "amortized_delta_candidate_minus_baseline_sec": {
                            "median": -7.0,
                            "trimmed_mean": -7.2,
                            "robust_sign": "robust_improvement",
                        },
                    },
                }
            ]
        }
        anchor_analyses = {
            "upper": {
                "experiment_id": "upper_only",
                "route_stats": [
                    self._row("DENSE_Q01_T40000", 0.0488, 1.0, 11.7),
                    self._row("SOFT_T40000", 0.0474, 0.1837, 3.5),
                ],
            }
        }
        specs = [
            {
                "label": "upper_dense_vs_soft_sparse",
                "anchor_key": "upper",
                "baseline_route_id": "DENSE_Q01_T40000",
                "candidate_route_id": "SOFT_T40000",
            }
        ]

        payload = audit.build_payload(
            robust_analysis,
            anchor_analyses,
            specs,
            quality_tolerance=0.002,
        )

        self.assertEqual(payload["lower_bank_read"], "not_evaluated")
        self.assertEqual(payload["upper_bank_read"], "near_frontier")
        self.assertEqual(payload["overall_dense_frontier_claim"], "upper-bank near-frontier")


if __name__ == "__main__":
    unittest.main()
