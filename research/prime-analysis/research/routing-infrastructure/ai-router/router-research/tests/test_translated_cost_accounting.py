import unittest

from tools import translated_cost_accounting as audit


class TranslatedCostAccountingTest(unittest.TestCase):
    def _analysis(self):
        return {
            "experiment_id": "exp",
            "route_stats": [
                {
                    "route_id": "DENSE_Q04_T36000",
                    "mean_test_top1_after": 0.0480,
                    "mean_retrieval_candidate_count": 36000.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 48.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 48.0,
                    "mean_online_total_per_repeat_sec": 12.0,
                    "mean_amortized_total_per_repeat_sec": 12.0,
                    "mean_retrieval_search_sec": 48.0,
                    "mean_query_route_sec": 0.0,
                    "mean_route_index_build_sec": 0.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_CPX8_Q04_T36000",
                    "mean_test_top1_after": 0.0472,
                    "mean_retrieval_candidate_count": 6840.0,
                    "mean_retrieval_candidate_fraction": 0.19,
                    "mean_total_sec": 38.0,
                    "mean_offline_total_sec": 29.0,
                    "mean_online_total_sec": 9.0,
                    "mean_online_total_per_repeat_sec": 2.25,
                    "mean_amortized_total_per_repeat_sec": 9.5,
                    "mean_retrieval_search_sec": 7.2,
                    "mean_query_route_sec": 1.8,
                    "mean_route_index_build_sec": 29.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "DENSE_Q04_T40000",
                    "mean_test_top1_after": 0.0490,
                    "mean_retrieval_candidate_count": 40000.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 36.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 36.0,
                    "mean_online_total_per_repeat_sec": 9.0,
                    "mean_amortized_total_per_repeat_sec": 9.0,
                    "mean_retrieval_search_sec": 36.0,
                    "mean_query_route_sec": 0.0,
                    "mean_route_index_build_sec": 0.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_CPX8_Q04_T40000",
                    "mean_test_top1_after": 0.0474,
                    "mean_retrieval_candidate_count": 7360.0,
                    "mean_retrieval_candidate_fraction": 0.184,
                    "mean_total_sec": 40.0,
                    "mean_offline_total_sec": 32.0,
                    "mean_online_total_sec": 8.0,
                    "mean_online_total_per_repeat_sec": 2.0,
                    "mean_amortized_total_per_repeat_sec": 10.0,
                    "mean_retrieval_search_sec": 6.4,
                    "mean_query_route_sec": 1.6,
                    "mean_route_index_build_sec": 32.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "DENSE_Q08_T40000",
                    "mean_test_top1_after": 0.0490,
                    "mean_retrieval_candidate_count": 40000.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 72.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 72.0,
                    "mean_online_total_per_repeat_sec": 9.0,
                    "mean_amortized_total_per_repeat_sec": 9.0,
                    "mean_retrieval_search_sec": 72.0,
                    "mean_query_route_sec": 0.0,
                    "mean_route_index_build_sec": 0.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "H4XH4_FIELD_A150_CPX8_Q08_T40000",
                    "mean_test_top1_after": 0.0474,
                    "mean_retrieval_candidate_count": 7360.0,
                    "mean_retrieval_candidate_fraction": 0.184,
                    "mean_total_sec": 48.0,
                    "mean_offline_total_sec": 32.0,
                    "mean_online_total_sec": 16.0,
                    "mean_online_total_per_repeat_sec": 2.0,
                    "mean_amortized_total_per_repeat_sec": 6.0,
                    "mean_retrieval_search_sec": 12.8,
                    "mean_query_route_sec": 3.2,
                    "mean_route_index_build_sec": 32.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
            ],
            "results": {
                "DENSE_Q04_T36000": [{"args": {"query_repeats": 4, "max_train": 36000, "retrieval_backend": "dense_exact"}}],
                "H4XH4_FIELD_A150_CPX8_Q04_T36000": [{"args": {"query_repeats": 4, "max_train": 36000, "retrieval_backend": "routed_probe"}}],
                "DENSE_Q04_T40000": [{"args": {"query_repeats": 4, "max_train": 40000, "retrieval_backend": "dense_exact"}}],
                "H4XH4_FIELD_A150_CPX8_Q04_T40000": [{"args": {"query_repeats": 4, "max_train": 40000, "retrieval_backend": "routed_probe"}}],
                "DENSE_Q08_T40000": [{"args": {"query_repeats": 8, "max_train": 40000, "retrieval_backend": "dense_exact"}}],
                "H4XH4_FIELD_A150_CPX8_Q08_T40000": [{"args": {"query_repeats": 8, "max_train": 40000, "retrieval_backend": "routed_probe"}}],
            },
        }

    def test_build_payload_decomposes_pair_costs(self):
        payload = audit.build_payload(
            self._analysis(),
            route_ids=[
                "DENSE_Q04_T36000",
                "H4XH4_FIELD_A150_CPX8_Q04_T36000",
            ],
            meta={"d": 128},
        )
        pair = payload["pair_audits"][0]
        self.assertTrue(pair["has_crossover"])
        self.assertAlmostEqual(pair["online_gain_per_repeat_vs_dense_sec"], 9.75)
        self.assertAlmostEqual(pair["offline_penalty_per_repeat_vs_dense_sec"], 7.25)
        self.assertAlmostEqual(pair["amortized_margin_vs_dense_sec"], 2.5)
        self.assertAlmostEqual(pair["route_lookup_penalty_per_repeat_sec"], 0.45)
        self.assertAlmostEqual(pair["search_gain_per_repeat_vs_dense_sec"], 10.2)
        self.assertAlmostEqual(pair["candidate_vector_bytes_saved_fraction_vs_dense"], 0.81)

    def test_threshold_finding_reports_non_monotone_split(self):
        payload = audit.build_payload(
            self._analysis(),
            route_ids=[
                "DENSE_Q04_T36000",
                "H4XH4_FIELD_A150_CPX8_Q04_T36000",
                "DENSE_Q04_T40000",
                "H4XH4_FIELD_A150_CPX8_Q04_T40000",
                "DENSE_Q08_T40000",
                "H4XH4_FIELD_A150_CPX8_Q08_T40000",
            ],
            meta={"d": 128},
        )
        finding = payload["threshold_finding"]
        self.assertEqual(finding["q04_crossing_route_id"], "H4XH4_FIELD_A150_CPX8_Q04_T36000")
        self.assertEqual(finding["q04_miss_route_id"], "H4XH4_FIELD_A150_CPX8_Q04_T40000")
        self.assertGreater(finding["q04_crossing_amortized_margin_vs_dense_sec"], 0.0)
        self.assertLess(finding["q04_miss_amortized_margin_vs_dense_sec"], 0.0)
        self.assertIn("online savings", finding["explanation"])

    def test_build_payload_merges_multiple_analyses_and_compares_non_dense_pairs(self):
        lower_chart = {
            "experiment_id": "lower_chart",
            "route_stats": [
                {
                    "route_id": "DENSE_Q01_T2500",
                    "mean_test_top1_after": 0.0503,
                    "mean_retrieval_candidate_count": 2500.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 0.42,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 0.115,
                    "mean_online_total_per_repeat_sec": 0.115,
                    "mean_amortized_total_per_repeat_sec": 0.115,
                    "mean_retrieval_search_sec": 0.115,
                    "mean_query_route_sec": 0.0,
                    "mean_route_index_build_sec": 0.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0463,
                    "mean_retrieval_candidate_count": 500.0,
                    "mean_retrieval_candidate_fraction": 0.2,
                    "mean_total_sec": 0.46,
                    "mean_offline_total_sec": 0.020,
                    "mean_online_total_sec": 0.058,
                    "mean_online_total_per_repeat_sec": 0.058,
                    "mean_amortized_total_per_repeat_sec": 0.078,
                    "mean_retrieval_search_sec": 0.048,
                    "mean_query_route_sec": 0.010,
                    "mean_route_index_build_sec": 0.015,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 8.0,
                    "n_runs": 2,
                },
            ],
            "results": {
                "DENSE_Q01_T2500": [{"args": {"query_repeats": 1, "max_train": 2500, "retrieval_backend": "dense_exact"}}],
                "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500": [{"args": {"query_repeats": 1, "max_train": 2500, "retrieval_backend": "routed_probe"}}],
            },
        }
        lower_full = {
            "experiment_id": "lower_full",
            "route_stats": [
                {
                    "route_id": "FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                    "mean_test_top1_after": 0.0463,
                    "mean_retrieval_candidate_count": 500.0,
                    "mean_retrieval_candidate_fraction": 0.2,
                    "mean_total_sec": 0.41,
                    "mean_offline_total_sec": 0.007,
                    "mean_online_total_sec": 0.048,
                    "mean_online_total_per_repeat_sec": 0.048,
                    "mean_amortized_total_per_repeat_sec": 0.055,
                    "mean_retrieval_search_sec": 0.040,
                    "mean_query_route_sec": 0.008,
                    "mean_route_index_build_sec": 0.005,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 8.0,
                    "n_runs": 2,
                },
            ],
            "results": {
                "FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500": [{"args": {"query_repeats": 1, "max_train": 2500, "retrieval_backend": "routed_probe"}}],
            },
        }
        payload = audit.build_payload(
            [lower_chart, lower_full],
            route_ids=[
                "DENSE_Q01_T2500",
                "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                "FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500",
            ],
            meta={"d": 128},
            comparison_specs=[
                {
                    "label": "chart_vs_full_lower",
                    "baseline_route_id": "FULL_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                    "candidate_route_id": "CHART_H4XH4_FIELD_A150_CPX8_Q01_T2500",
                }
            ],
        )
        self.assertEqual(payload["source_experiments"], ["lower_chart", "lower_full"])
        self.assertEqual(len(payload["comparison_audits"]), 1)
        comparison = payload["comparison_audits"][0]
        self.assertEqual(comparison["label"], "chart_vs_full_lower")
        self.assertAlmostEqual(comparison["route_index_build_penalty_candidate_vs_baseline_sec"], 0.01)
        self.assertAlmostEqual(comparison["route_query_penalty_candidate_vs_baseline_sec"], 0.002)
        self.assertAlmostEqual(comparison["retrieval_search_penalty_candidate_vs_baseline_sec"], 0.008)
        self.assertAlmostEqual(comparison["amortized_delta_candidate_minus_baseline_sec"], 0.023)
        self.assertEqual(comparison["dominant_penalty_component"], "route_index_build")


if __name__ == "__main__":
    unittest.main()
