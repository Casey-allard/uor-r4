import unittest

from tools import spectral_residual_sweep as sweep


class SpectralResidualSweepTest(unittest.TestCase):
    def test_evaluate_acceptance_detects_promoted_signal(self):
        route_stats = [
            {
                "route_id": "P",
                "args": {"sector_mode": "phase4d_hopf_product_phase"},
                "mean_spectral_zero_eigs": 1,
                "mean_residual_l2_lowfreq_energy": 0.12,
                "mean_residual_l2_dirichlet_energy": 0.31,
                "mean_error_indicator_lowfreq_energy": 0.10,
                "mean_error_indicator_dirichlet_energy": 0.35,
                "mean_true_margin_lowfreq_energy": 0.05,
                "mean_true_margin_dirichlet_energy": 0.40,
            },
            {
                "route_id": "HB",
                "args": {"sector_mode": "phase4d_hopf_base"},
                "mean_spectral_zero_eigs": 1,
                "mean_residual_l2_lowfreq_energy": 0.08,
                "mean_residual_l2_dirichlet_energy": 0.36,
                "mean_error_indicator_lowfreq_energy": 0.07,
                "mean_error_indicator_dirichlet_energy": 0.40,
                "mean_true_margin_lowfreq_energy": 0.06,
                "mean_true_margin_dirichlet_energy": 0.39,
            },
            {
                "route_id": "H",
                "args": {"sector_mode": "phase4d_hopf"},
                "mean_spectral_zero_eigs": 1,
                "mean_residual_l2_lowfreq_energy": 0.09,
                "mean_residual_l2_dirichlet_energy": 0.37,
                "mean_error_indicator_lowfreq_energy": 0.06,
                "mean_error_indicator_dirichlet_energy": 0.39,
                "mean_true_margin_lowfreq_energy": 0.04,
                "mean_true_margin_dirichlet_energy": 0.41,
            },
        ]

        acceptance = sweep.evaluate_acceptance(route_stats)

        self.assertTrue(acceptance["all_graphs_connected"])
        self.assertTrue(acceptance["distinct_task_signal"])
        self.assertIn("residual_l2", acceptance["promoted_signals"])
        self.assertGreater(acceptance["hopf_residual_l2_lowfreq_gap"], 0.0)
        self.assertGreater(acceptance["hopf_residual_l2_dirichlet_gap"], 0.0)

    def test_choose_recommendation_handles_inconclusive_signal(self):
        acceptance = {
            "all_graphs_connected": True,
            "distinct_task_signal": False,
            "promoted_signals": [],
        }
        recommendation = sweep.choose_recommendation(acceptance, stage="screen")
        self.assertIn("inconclusive", recommendation)


if __name__ == "__main__":
    unittest.main()
