import unittest

from tools import translated_component_stability as audit


class TranslatedComponentStabilityTest(unittest.TestCase):
    def _run(self, seed, top1, cand_frac, amortized, offline, online, route_index, route_query, search):
        return {
            "args": {"seed": seed, "query_repeats": 2},
            "metrics": {
                "test_top1_after": top1,
                "retrieval_candidate_fraction_mean": cand_frac,
                "retrieval_total_amortized_per_repeat_sec": amortized,
                "retrieval_query_repeats": 2,
            },
            "timings_sec": {
                "offline_total": offline,
                "online_total": online,
                "route_index_build": route_index,
                "query_route": route_query,
                "retrieval_search": search,
            },
        }

    def test_build_payload_reports_mixed_route_query_and_stable_search(self):
        analysis = {
            "experiment_id": "exp",
            "config": "cfg",
            "results": {
                "BASE": [
                    self._run(0, 0.50, 0.20, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.50, 0.20, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(2, 0.50, 0.20, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.49, 0.19, 0.70, 0.44, 0.48, 0.24, 0.06, 0.30),
                    self._run(1, 0.50, 0.19, 1.08, 0.42, 0.66, 0.22, 0.14, 0.34),
                    self._run(2, 0.49, 0.18, 0.80, 0.38, 0.56, 0.18, 0.08, 0.30),
                ],
            },
        }

        payload = audit.build_payload(
            [analysis],
            [
                {
                    "label": "base_vs_candidate",
                    "baseline_route_id": "BASE",
                    "candidate_route_id": "CAND",
                }
            ],
        )
        comparison = payload["comparisons"][0]
        self.assertEqual(comparison["win_count"], 2)
        self.assertEqual(comparison["loss_count"], 1)
        self.assertEqual(
            comparison["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"],
            "mixed",
        )
        self.assertEqual(
            comparison["summaries"]["route_query_delta_candidate_minus_baseline_sec"]["sign_consistency"],
            "mixed",
        )
        self.assertEqual(
            comparison["summaries"]["retrieval_search_delta_candidate_minus_baseline_sec"]["sign_consistency"],
            "stable_improvement",
        )
        self.assertAlmostEqual(
            comparison["seed_entries"][0]["route_index_build_delta_candidate_minus_baseline_sec"],
            0.02,
        )
        self.assertAlmostEqual(
            comparison["seed_entries"][0]["route_query_delta_candidate_minus_baseline_sec"],
            -0.02,
        )
        self.assertIn("Route-query deltas change sign", comparison["interpretation"])


if __name__ == "__main__":
    unittest.main()
