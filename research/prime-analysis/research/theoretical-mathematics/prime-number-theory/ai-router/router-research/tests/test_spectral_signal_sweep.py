import unittest

from tools import spectral_signal_sweep as sweep


class SpectralSignalSweepTest(unittest.TestCase):
    def test_evaluate_acceptance_detects_task_signal_gain(self):
        route_stats = [
            {
                "route_id": "P",
                "args": {"sector_mode": "phase4d_hopf_product_phase"},
                "mean_spectral_zero_eigs": 1,
                "mean_label_onehot_lowfreq_energy": 0.12,
                "mean_label_onehot_dirichlet_energy": 0.34,
                "mean_label_indicator_lowfreq_mean": 0.09,
                "mean_shell_lowfreq_energy": 0.20,
                "mean_sector_lowfreq_energy": 0.03,
            },
            {
                "route_id": "HB",
                "args": {"sector_mode": "phase4d_hopf_base"},
                "mean_spectral_zero_eigs": 1,
                "mean_label_onehot_lowfreq_energy": 0.08,
                "mean_label_onehot_dirichlet_energy": 0.40,
                "mean_label_indicator_lowfreq_mean": 0.07,
                "mean_shell_lowfreq_energy": 0.10,
                "mean_sector_lowfreq_energy": 0.05,
            },
            {
                "route_id": "H",
                "args": {"sector_mode": "phase4d_hopf"},
                "mean_spectral_zero_eigs": 1,
                "mean_label_onehot_lowfreq_energy": 0.09,
                "mean_label_onehot_dirichlet_energy": 0.42,
                "mean_label_indicator_lowfreq_mean": 0.07,
                "mean_shell_lowfreq_energy": 0.08,
                "mean_sector_lowfreq_energy": 0.09,
            },
        ]
        acceptance = sweep.evaluate_acceptance(route_stats)
        self.assertTrue(acceptance["all_graphs_connected"])
        self.assertTrue(acceptance["distinct_task_signal"])
        self.assertGreater(acceptance["hopf_label_lowfreq_gap"], 0.0)
        self.assertGreater(acceptance["hopf_label_dirichlet_gap"], 0.0)

    def test_choose_recommendation_handles_inconclusive_signal(self):
        acceptance = {
            "all_graphs_connected": True,
            "distinct_task_signal": False,
        }
        recommendation = sweep.choose_recommendation(acceptance, stage="screen")
        self.assertIn("inconclusive", recommendation)


if __name__ == "__main__":
    unittest.main()
