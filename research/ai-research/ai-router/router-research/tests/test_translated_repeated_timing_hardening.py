import unittest

from tools import translated_repeated_timing_hardening as audit


class TranslatedRepeatedTimingHardeningTest(unittest.TestCase):
    def _run(self, seed, top1, cand_frac, amortized, offline, online, route_index, route_query, search):
        return {
            "args": {"seed": seed, "query_repeats": 1},
            "metrics": {
                "test_top1_after": top1,
                "retrieval_candidate_fraction_mean": cand_frac,
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

    def test_build_payload_detects_within_seed_repeat_flips(self):
        analysis_a = {
            "experiment_id": "a",
            "results": {
                "BASE": [
                    self._run(0, 0.5, 0.2, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.5, 0.2, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.49, 0.19, 0.80, 0.38, 0.42, 0.19, 0.08, 0.34),
                    self._run(1, 0.49, 0.19, 1.10, 0.42, 0.68, 0.22, 0.14, 0.44),
                ],
            },
        }
        analysis_b = {
            "experiment_id": "b",
            "results": {
                "BASE": [
                    self._run(0, 0.5, 0.2, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                    self._run(1, 0.5, 0.2, 1.00, 0.40, 0.60, 0.20, 0.10, 0.40),
                ],
                "CAND": [
                    self._run(0, 0.49, 0.19, 1.05, 0.41, 0.64, 0.21, 0.12, 0.42),
                    self._run(1, 0.49, 0.19, 0.90, 0.39, 0.51, 0.19, 0.07, 0.44),
                ],
            },
        }

        payload = audit.build_payload(
            [analysis_a, analysis_b],
            [{"label": "x", "baseline_route_id": "BASE", "candidate_route_id": "CAND"}],
        )
        comparison = payload["comparisons"][0]
        seed0 = next(row for row in comparison["per_seed"] if row["seed"] == 0)
        seed1 = next(row for row in comparison["per_seed"] if row["seed"] == 1)
        self.assertEqual(
            seed0["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"],
            "mixed",
        )
        self.assertEqual(
            seed1["summaries"]["amortized_delta_candidate_minus_baseline_sec"]["sign_consistency"],
            "mixed",
        )
        self.assertIn("timing noise", comparison["interpretation"])


if __name__ == "__main__":
    unittest.main()
