import unittest

from tools.proxy_sweep import apply_health_gate, build_run_plan, choose_recommendation, summarize_route


def make_summary(route_id, seed, eval_shells, shell_pmax, buckets=20, pmax_after=0.4, unseen=0.0, mse=0.00393, total=30.0):
    return {
        "route_id": route_id,
        "args": {"seed": seed},
        "metrics": {
            "buckets": buckets,
            "eval_shells": eval_shells,
            "pmax_after": pmax_after,
            "shell_pmax": shell_pmax,
            "sector_pmax": 0.4,
            "entropy_after": 2.0,
            "shell_entropy": 0.6,
            "sector_entropy": 1.5,
            "test_unseen_rate": unseen,
            "test_mse_after": mse,
        },
        "timings_sec": {"total": total},
    }


class ProxySweepHealthGateTest(unittest.TestCase):
    def test_seed_major_run_order_interleaves_routes(self):
        routes = [{"route_id": "A"}, {"route_id": "B"}]
        seeds = [0, 1]
        plan = build_run_plan(routes, seeds, run_order="seed_major")
        self.assertEqual(
            [(route["route_id"], seed) for route, seed in plan],
            [("A", 0), ("B", 0), ("A", 1), ("B", 1)],
        )

    def test_seed_health_can_fail_even_when_mean_passes(self):
        results = {
            "R0": [
                make_summary("R0", 0, eval_shells=1, shell_pmax=1.0, buckets=8, pmax_after=0.2, mse=0.0040, total=30.0),
                make_summary("R0", 1, eval_shells=1, shell_pmax=1.0, buckets=8, pmax_after=0.2, mse=0.0040, total=30.0),
            ],
            "R1": [
                make_summary("R1", 0, eval_shells=1, shell_pmax=1.0, mse=0.0039, total=29.0),
                make_summary("R1", 1, eval_shells=5, shell_pmax=0.68, mse=0.0039, total=29.0),
            ],
        }
        route_stats = [summarize_route(route_id, summaries) for route_id, summaries in results.items()]
        route_stats.sort(key=lambda row: row["route_id"])
        apply_health_gate(
            route_stats,
            results,
            {
                "min_buckets": 4,
                "min_eval_shells": 2,
                "max_pmax_after": 0.65,
                "max_shell_pmax": 0.85,
                "max_unseen_rate": 0.01,
                "max_mse_ratio_vs_r0": 1.03,
                "max_runtime_ratio_vs_r0": 1.15,
                "enforce_seed_health": True,
            },
        )
        by_route = {row["route_id"]: row for row in route_stats}
        self.assertFalse(by_route["R1"]["passes_health_gate"])
        self.assertIn("seed0_eval_shells<2.000", by_route["R1"]["health_gate_reasons"])
        self.assertIn("seed0_shell_pmax>0.850", by_route["R1"]["health_gate_reasons"])

    def test_hardware_efficiency_lead_can_be_promoted_within_quality_tolerance(self):
        route_stats = [
            {
                "route_id": "R0",
                "mean_test_mse_after": 0.00390,
                "mean_total_sec": 80.0,
                "passes_health_gate": False,
            },
            {
                "route_id": "R1",
                "mean_test_mse_after": 0.00391,
                "mean_total_sec": 50.0,
                "passes_health_gate": True,
                "mse_ratio_vs_r0": 1.0026,
                "runtime_ratio_vs_r0": 0.625,
            },
        ]
        recommendation = choose_recommendation(
            route_stats,
            health_gate={"max_mse_ratio_vs_r0": 1.03},
        )
        self.assertIn("hardware-efficiency transfer lead", recommendation)
        self.assertIn("R1", recommendation)

    def test_non_r0_baseline_is_supported(self):
        route_stats = [
            {
                "route_id": "DENSE",
                "mean_test_mse_after": 0.00400,
                "mean_total_sec": 20.0,
                "passes_health_gate": False,
            },
            {
                "route_id": "HOPF",
                "mean_test_mse_after": 0.00395,
                "mean_total_sec": 18.0,
                "passes_health_gate": True,
                "mse_ratio_vs_r0": 0.9875,
                "runtime_ratio_vs_r0": 0.9,
            },
        ]
        recommendation = choose_recommendation(
            route_stats,
            health_gate={"max_mse_ratio_vs_r0": 1.03},
            baseline_route_id="DENSE",
        )
        self.assertIn("DENSE", recommendation)
        self.assertIn("HOPF", recommendation)

    def test_retrieval_recommendation_tracks_systems_lead_when_top1_regresses(self):
        route_stats = [
            {
                "route_id": "HOPF",
                "mean_test_mse_after": 0.00431,
                "mean_total_sec": 26.2,
                "mean_test_top1_after": 0.0510,
                "mean_retrieval_candidate_fraction": 0.3368,
                "mean_online_total_per_repeat_sec": 0.6290,
                "mean_amortized_total_per_repeat_sec": 1.0681,
                "passes_health_gate": True,
            },
            {
                "route_id": "PRODUCT_CPX",
                "mean_test_mse_after": 0.00432,
                "mean_total_sec": 19.7,
                "mean_test_top1_after": 0.0487,
                "mean_retrieval_candidate_fraction": 0.1903,
                "mean_online_total_per_repeat_sec": 0.3804,
                "mean_amortized_total_per_repeat_sec": 0.7949,
                "passes_health_gate": True,
            },
        ]
        recommendation = choose_recommendation(
            route_stats,
            health_gate={"min_buckets": 4},
            baseline_route_id="HOPF",
        )
        self.assertIn("translated systems lead", recommendation)
        self.assertIn("PRODUCT_CPX", recommendation)
        self.assertIn("top-1 regressed", recommendation)

    def test_retrieval_recommendation_promotes_systems_lead_when_top1_improves(self):
        route_stats = [
            {
                "route_id": "DENSE_Q24",
                "mean_test_mse_after": 0.00432,
                "mean_total_sec": 30.0,
                "mean_test_top1_after": 0.0480,
                "mean_retrieval_candidate_fraction": 1.0,
                "mean_online_total_per_repeat_sec": 0.9000,
                "mean_amortized_total_per_repeat_sec": 0.9000,
                "passes_health_gate": False,
            },
            {
                "route_id": "PRODUCT_CPX",
                "mean_test_mse_after": 0.00431,
                "mean_total_sec": 20.0,
                "mean_test_top1_after": 0.0485,
                "mean_retrieval_candidate_fraction": 0.1900,
                "mean_online_total_per_repeat_sec": 0.3800,
                "mean_amortized_total_per_repeat_sec": 0.7900,
                "passes_health_gate": True,
            },
        ]
        recommendation = choose_recommendation(
            route_stats,
            baseline_route_id="DENSE_Q24",
        )
        self.assertIn("Promote PRODUCT_CPX as translated systems lead", recommendation)
        self.assertIn("improving top-1", recommendation)

    def test_summarize_route_includes_sparse_event_metrics(self):
        summaries = [
            {
                "route_id": "PRODUCT_EVT",
                "args": {"seed": 0},
                "metrics": {
                    "test_mse_after": 0.0040,
                    "event_gate_error_mean": 0.07,
                    "event_gate_mean": 0.45,
                    "event_gate_active_frac": 0.30,
                    "event_gate_cost_proxy": 0.45,
                },
                "timings_sec": {"total": 10.0},
            },
            {
                "route_id": "PRODUCT_EVT",
                "args": {"seed": 1},
                "metrics": {
                    "test_mse_after": 0.0042,
                    "event_gate_error_mean": 0.05,
                    "event_gate_mean": 0.25,
                    "event_gate_active_frac": 0.20,
                    "event_gate_cost_proxy": 0.25,
                },
                "timings_sec": {"total": 11.0},
            },
        ]

        stats = summarize_route("PRODUCT_EVT", summaries)

        self.assertAlmostEqual(stats["mean_event_gate_error_mean"], 0.06)
        self.assertAlmostEqual(stats["mean_event_gate_mean"], 0.35)
        self.assertAlmostEqual(stats["mean_event_gate_active_frac"], 0.25)
        self.assertAlmostEqual(stats["mean_event_gate_cost_proxy"], 0.35)


if __name__ == "__main__":
    unittest.main()
