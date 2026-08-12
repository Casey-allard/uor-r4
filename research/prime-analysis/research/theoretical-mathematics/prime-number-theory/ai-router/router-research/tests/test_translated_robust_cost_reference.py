import unittest

from tools import translated_robust_cost_reference as audit


class TranslatedRobustCostReferenceTest(unittest.TestCase):
    def _run(self, seed, top1, cand_frac, cand_count, amortized, offline, online, route_index, route_query, search):
        return {
            "args": {"seed": seed, "query_repeats": 1},
            "metrics": {
                "test_top1_after": top1,
                "retrieval_candidate_fraction_mean": cand_frac,
                "retrieval_candidate_count_mean": cand_count,
                "retrieval_total_amortized_per_repeat_sec": amortized,
                "retrieval_query_repeats": 1,
            },
            "timings_sec": {
                "offline_total": offline,
                "online_total": online,
                "route_index_build": route_index,
                "query_route": route_query,
                "retrieval_search": search,
            },
        }

    def test_build_payload_marks_pruning_only_when_median_and_trimmed_mean_disagree(self):
        analysis_a = {
            "experiment_id": "a",
            "results": {
                "BASE": [
                    self._run(0, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.50, 0.18, 180, 0.50, 0.20, 0.30, 0.10, 0.05, 0.25),
                    self._run(1, 0.50, 0.18, 180, 0.40, 0.12, 0.28, 0.08, 0.05, 0.23),
                ],
            },
        }
        analysis_b = {
            "experiment_id": "b",
            "results": {
                "BASE": [
                    self._run(0, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.50, 0.18, 180, 1.01, 0.41, 0.60, 0.21, 0.10, 0.40),
                    self._run(1, 0.50, 0.18, 180, 1.02, 0.42, 0.61, 0.22, 0.10, 0.41),
                ],
            },
        }
        analysis_c = {
            "experiment_id": "c",
            "results": {
                "BASE": [
                    self._run(0, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.50, 0.18, 180, 1.04, 0.43, 0.61, 0.23, 0.10, 0.41),
                    self._run(1, 0.50, 0.18, 180, 1.05, 0.44, 0.61, 0.24, 0.10, 0.41),
                ],
            },
        }

        payload = audit.build_payload(
            [analysis_a, analysis_b, analysis_c],
            [{"label": "x", "baseline_route_id": "BASE", "candidate_route_id": "CAND"}],
            trim_count=1,
        )
        comparison = payload["comparisons"][0]
        amortized = comparison["robust_summaries"]["amortized_delta_candidate_minus_baseline_sec"]
        cand_frac = comparison["robust_summaries"]["candidate_fraction_delta"]
        cand_count = comparison["robust_summaries"]["candidate_count_delta"]

        self.assertGreater(amortized["median"], 0.0)
        self.assertLess(amortized["trimmed_mean"], 0.0)
        self.assertEqual(amortized["robust_sign"], "mixed")
        self.assertEqual(cand_frac["robust_sign"], "robust_improvement")
        self.assertEqual(cand_count["robust_sign"], "robust_improvement")
        self.assertEqual(comparison["systems_verdict"], "pruning_only")
        self.assertIn("Candidate-fraction and candidate-count reduction both remain robust.", comparison["interpretation"])

    def test_top1_uses_higher_is_better_polarity(self):
        analysis = {
            "experiment_id": "a",
            "results": {
                "BASE": [
                    self._run(0, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.50, 0.20, 200, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.48, 0.18, 180, 0.90, 0.30, 0.60, 0.10, 0.05, 0.55),
                    self._run(1, 0.49, 0.18, 180, 0.92, 0.31, 0.61, 0.11, 0.06, 0.55),
                ],
            },
        }

        payload = audit.build_payload(
            [analysis],
            [{"label": "x", "baseline_route_id": "BASE", "candidate_route_id": "CAND"}],
            trim_count=1,
        )
        top1 = payload["comparisons"][0]["robust_summaries"]["top1_delta_candidate_minus_baseline"]
        self.assertEqual(top1["sign_consistency"], "stable_regression")
        self.assertEqual(top1["robust_sign"], "robust_regression")


if __name__ == "__main__":
    unittest.main()
