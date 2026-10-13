import unittest

from tools import translated_hardware_profile as profile


class TranslatedHardwareProfileTest(unittest.TestCase):
    def test_derive_group_profiles_computes_dense_comparisons(self):
        analysis = {
            "experiment_id": "exp",
            "route_stats": [
                {
                    "route_id": "DENSE_Q16",
                    "mean_test_top1_after": 0.050,
                    "mean_retrieval_candidate_count": 100.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 16.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 16.0,
                    "mean_online_total_per_repeat_sec": 1.0,
                    "mean_amortized_total_per_repeat_sec": 1.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "PRODUCT_Q16",
                    "mean_test_top1_after": 0.050,
                    "mean_retrieval_candidate_count": 30.0,
                    "mean_retrieval_candidate_fraction": 0.30,
                    "mean_total_sec": 12.0,
                    "mean_offline_total_sec": 8.0,
                    "mean_online_total_sec": 4.0,
                    "mean_online_total_per_repeat_sec": 0.25,
                    "mean_amortized_total_per_repeat_sec": 0.75,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
            ],
            "results": {
                "DENSE_Q16": [{"args": {"query_repeats": 16, "max_train": 12000, "retrieval_backend": "dense_exact"}}],
                "PRODUCT_Q16": [{"args": {"query_repeats": 16, "max_train": 12000, "retrieval_backend": "routed_probe"}}],
            },
        }
        payload = profile.build_payload([analysis])
        by_id = {row["route_id"]: row for row in payload["route_profiles"]}
        product = by_id["PRODUCT_Q16"]
        self.assertAlmostEqual(product["search_work_ratio_vs_dense"], 0.30)
        self.assertAlmostEqual(product["search_work_saved_fraction_vs_dense"], 0.70)
        self.assertAlmostEqual(product["offline_share"], 8.0 / 12.0)
        self.assertAlmostEqual(product["online_share"], 4.0 / 12.0)
        self.assertTrue(product["is_crossover_vs_dense"])
        self.assertAlmostEqual(product["amortized_margin_vs_dense"], 0.25)

    def test_group_summary_picks_quality_and_systems_crossover(self):
        analysis = {
            "experiment_id": "exp",
            "route_stats": [
                {
                    "route_id": "DENSE_Q16",
                    "mean_test_top1_after": 0.049,
                    "mean_retrieval_candidate_count": 100.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 16.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 16.0,
                    "mean_online_total_per_repeat_sec": 1.0,
                    "mean_amortized_total_per_repeat_sec": 1.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 4,
                },
                {
                    "route_id": "A150_Q16",
                    "mean_test_top1_after": 0.049,
                    "mean_retrieval_candidate_count": 35.0,
                    "mean_retrieval_candidate_fraction": 0.35,
                    "mean_total_sec": 14.0,
                    "mean_offline_total_sec": 9.0,
                    "mean_online_total_sec": 5.0,
                    "mean_online_total_per_repeat_sec": 0.3125,
                    "mean_amortized_total_per_repeat_sec": 0.875,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 4,
                },
                {
                    "route_id": "A150_CPX8_Q16",
                    "mean_test_top1_after": 0.0485,
                    "mean_retrieval_candidate_count": 20.0,
                    "mean_retrieval_candidate_fraction": 0.20,
                    "mean_total_sec": 13.0,
                    "mean_offline_total_sec": 9.0,
                    "mean_online_total_sec": 4.0,
                    "mean_online_total_per_repeat_sec": 0.25,
                    "mean_amortized_total_per_repeat_sec": 0.8125,
                    "mean_retrieval_bucket_fallback_rate": 0.002,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 4,
                },
            ],
            "results": {
                "DENSE_Q16": [{"args": {"query_repeats": 16, "max_train": 12000, "retrieval_backend": "dense_exact"}}],
                "A150_Q16": [{"args": {"query_repeats": 16, "max_train": 12000, "retrieval_backend": "routed_probe"}}],
                "A150_CPX8_Q16": [{"args": {"query_repeats": 16, "max_train": 12000, "retrieval_backend": "routed_probe"}}],
            },
        }
        payload = profile.build_payload([analysis])
        summary = payload["group_summaries"][0]
        self.assertTrue(summary["has_crossover"])
        self.assertEqual(summary["quality_matched_route_id"], "A150_Q16")
        self.assertEqual(summary["systems_route_id"], "A150_CPX8_Q16")

    def test_bank_summary_tracks_first_crossover_and_ratio_stability(self):
        analysis = {
            "experiment_id": "exp",
            "route_stats": [
                {
                    "route_id": "DENSE_Q12_T6000",
                    "mean_test_top1_after": 0.049,
                    "mean_retrieval_candidate_count": 100.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 12.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 12.0,
                    "mean_online_total_per_repeat_sec": 1.0,
                    "mean_amortized_total_per_repeat_sec": 1.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "A150_CPX8_Q12_T6000",
                    "mean_test_top1_after": 0.047,
                    "mean_retrieval_candidate_count": 19.0,
                    "mean_retrieval_candidate_fraction": 0.19,
                    "mean_total_sec": 13.2,
                    "mean_offline_total_sec": 9.6,
                    "mean_online_total_sec": 3.6,
                    "mean_online_total_per_repeat_sec": 0.30,
                    "mean_amortized_total_per_repeat_sec": 1.10,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "DENSE_Q16_T6000",
                    "mean_test_top1_after": 0.049,
                    "mean_retrieval_candidate_count": 100.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 16.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 16.0,
                    "mean_online_total_per_repeat_sec": 1.0,
                    "mean_amortized_total_per_repeat_sec": 1.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "A150_CPX8_Q16_T6000",
                    "mean_test_top1_after": 0.047,
                    "mean_retrieval_candidate_count": 18.8,
                    "mean_retrieval_candidate_fraction": 0.188,
                    "mean_total_sec": 12.8,
                    "mean_offline_total_sec": 8.8,
                    "mean_online_total_sec": 4.0,
                    "mean_online_total_per_repeat_sec": 0.25,
                    "mean_amortized_total_per_repeat_sec": 0.80,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "DENSE_Q24_T6000",
                    "mean_test_top1_after": 0.049,
                    "mean_retrieval_candidate_count": 100.0,
                    "mean_retrieval_candidate_fraction": 1.0,
                    "mean_total_sec": 24.0,
                    "mean_offline_total_sec": 0.0,
                    "mean_online_total_sec": 24.0,
                    "mean_online_total_per_repeat_sec": 1.0,
                    "mean_amortized_total_per_repeat_sec": 1.0,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 0.0,
                    "n_runs": 2,
                },
                {
                    "route_id": "A150_CPX8_Q24_T6000",
                    "mean_test_top1_after": 0.047,
                    "mean_retrieval_candidate_count": 18.7,
                    "mean_retrieval_candidate_fraction": 0.187,
                    "mean_total_sec": 12.0,
                    "mean_offline_total_sec": 7.2,
                    "mean_online_total_sec": 4.8,
                    "mean_online_total_per_repeat_sec": 0.20,
                    "mean_amortized_total_per_repeat_sec": 0.50,
                    "mean_retrieval_bucket_fallback_rate": 0.0,
                    "mean_retrieval_secondary_key_count": 4.0,
                    "n_runs": 2,
                },
            ],
            "results": {
                "DENSE_Q12_T6000": [{"args": {"query_repeats": 12, "max_train": 6000, "retrieval_backend": "dense_exact"}}],
                "A150_CPX8_Q12_T6000": [{"args": {"query_repeats": 12, "max_train": 6000, "retrieval_backend": "routed_probe"}}],
                "DENSE_Q16_T6000": [{"args": {"query_repeats": 16, "max_train": 6000, "retrieval_backend": "dense_exact"}}],
                "A150_CPX8_Q16_T6000": [{"args": {"query_repeats": 16, "max_train": 6000, "retrieval_backend": "routed_probe"}}],
                "DENSE_Q24_T6000": [{"args": {"query_repeats": 24, "max_train": 6000, "retrieval_backend": "dense_exact"}}],
                "A150_CPX8_Q24_T6000": [{"args": {"query_repeats": 24, "max_train": 6000, "retrieval_backend": "routed_probe"}}],
            },
        }
        payload = profile.build_payload([analysis])
        summary = payload["bank_summaries"][0]
        self.assertEqual(summary["max_train"], 6000)
        self.assertEqual(summary["first_any_crossover_query_repeats"], 16)
        self.assertEqual(summary["first_systems_crossover_query_repeats"], 16)
        self.assertEqual(summary["systems_route_family"], "A150_CPX8")
        self.assertAlmostEqual(summary["systems_search_work_ratio_min"], 0.187)
        self.assertAlmostEqual(summary["systems_search_work_ratio_max"], 0.19)
        self.assertAlmostEqual(summary["systems_search_work_ratio_span"], 0.003)
        self.assertAlmostEqual(summary["systems_margin_first"], -0.10)
        self.assertAlmostEqual(summary["systems_margin_last"], 0.50)
        self.assertAlmostEqual(summary["systems_margin_slope_per_repeat"], 0.05)


if __name__ == "__main__":
    unittest.main()
