import unittest

from tools import spectral_route_sweep as sweep


class SpectralRouteSweepTest(unittest.TestCase):
    def test_aggregate_route_stats_collects_means(self):
        seed_payloads = [
            {
                "seed": 0,
                "results": [
                    {
                        "route_id": "A",
                        "metrics": {
                            "spectral_lambda2": 0.1,
                            "spectral_mode_participation_mean": 0.2,
                            "sector_lowfreq_energy": 0.3,
                            "shell_lowfreq_energy": 0.4,
                            "spectral_zero_eigs": 1,
                            "graph_edges": 12,
                            "graph_sigma": 0.7,
                            "spectral_lambda3": 0.2,
                            "spectral_lambda4": 0.3,
                            "spectral_lowfreq_mass_ratio": 0.5,
                        },
                    }
                ],
            },
            {
                "seed": 1,
                "results": [
                    {
                        "route_id": "A",
                        "metrics": {
                            "spectral_lambda2": 0.3,
                            "spectral_mode_participation_mean": 0.4,
                            "sector_lowfreq_energy": 0.5,
                            "shell_lowfreq_energy": 0.6,
                            "spectral_zero_eigs": 1,
                            "graph_edges": 16,
                            "graph_sigma": 0.9,
                            "spectral_lambda3": 0.4,
                            "spectral_lambda4": 0.5,
                            "spectral_lowfreq_mass_ratio": 0.7,
                        },
                    }
                ],
            },
        ]
        stats = sweep.aggregate_route_stats("A", {"sector_mode": "phase4d_hopf"}, seed_payloads)
        self.assertAlmostEqual(stats["mean_spectral_lambda2"], 0.2)
        self.assertAlmostEqual(stats["mean_spectral_mode_participation_mean"], 0.3)
        self.assertAlmostEqual(stats["mean_sector_lowfreq_energy"], 0.4)
        self.assertEqual(stats["n_seeds"], 2)

    def test_evaluate_acceptance_detects_distinct_product_signature(self):
        route_stats = [
            {
                "route_id": "P",
                "args": {"sector_mode": "phase4d_hopf_product_phase"},
                "mean_spectral_zero_eigs": 1,
                "mean_spectral_mode_participation_mean": 0.30,
                "mean_sector_lowfreq_energy": 0.05,
                "mean_spectral_lambda2": 0.08,
                "mean_shell_lowfreq_energy": 0.15,
            },
            {
                "route_id": "C1",
                "args": {"sector_mode": "phase4d_hopf"},
                "mean_spectral_zero_eigs": 1,
                "mean_spectral_mode_participation_mean": 0.20,
                "mean_sector_lowfreq_energy": 0.20,
                "mean_spectral_lambda2": 0.09,
                "mean_shell_lowfreq_energy": 0.11,
            },
            {
                "route_id": "C2",
                "args": {"sector_mode": "phase4d_hopf_base"},
                "mean_spectral_zero_eigs": 1,
                "mean_spectral_mode_participation_mean": 0.22,
                "mean_sector_lowfreq_energy": 0.18,
                "mean_spectral_lambda2": 0.07,
                "mean_shell_lowfreq_energy": 0.10,
            },
        ]
        acceptance = sweep.evaluate_acceptance(route_stats)
        self.assertTrue(acceptance["all_graphs_connected"])
        self.assertTrue(acceptance["distinct_signature"])
        self.assertGreater(acceptance["participation_gap_product_minus_control"], 0.0)
        self.assertGreater(acceptance["sector_lowfreq_gap_control_minus_product"], 0.0)

    def test_choose_recommendation_handles_inconclusive_screen(self):
        route_stats = [
            {
                "route_id": "P",
                "mean_spectral_mode_participation_mean": 0.25,
                "mean_sector_lowfreq_energy": 0.08,
            },
            {
                "route_id": "C",
                "mean_spectral_mode_participation_mean": 0.23,
                "mean_sector_lowfreq_energy": 0.19,
            },
        ]
        acceptance = {
            "all_graphs_connected": True,
            "distinct_signature": False,
        }
        recommendation = sweep.choose_recommendation(route_stats, acceptance, stage="screen")
        self.assertIn("inconclusive", recommendation)


if __name__ == "__main__":
    unittest.main()
