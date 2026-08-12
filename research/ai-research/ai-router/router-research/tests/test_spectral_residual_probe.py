import math
import unittest

import numpy as np

from tools import spectral_residual_probe as probe


class SpectralResidualProbeTest(unittest.TestCase):
    def test_task_residual_metrics_emits_expected_keys(self):
        laplacian = np.array([[1.0, -1.0], [-1.0, 1.0]], dtype=np.float64)
        evecs = np.array(
            [
                [1.0 / math.sqrt(2.0), 1.0 / math.sqrt(2.0)],
                [1.0 / math.sqrt(2.0), -1.0 / math.sqrt(2.0)],
            ],
            dtype=np.float64,
        )
        y_eval = np.array([[1.0, 0.0], [0.0, 1.0]], dtype=np.float64)
        yhat_eval = np.array([[0.8, 0.2], [0.7, 0.3]], dtype=np.float64)

        metrics = probe.task_residual_metrics(
            laplacian,
            evecs,
            y_eval=y_eval,
            yhat_eval=yhat_eval,
            lowfreq_modes=1,
        )

        self.assertAlmostEqual(metrics["task_error_rate"], 0.5)
        self.assertIn("residual_l2_lowfreq_energy", metrics)
        self.assertIn("error_indicator_dirichlet_energy", metrics)
        self.assertIn("true_margin_lowfreq_energy", metrics)
        self.assertAlmostEqual(metrics["task_true_margin_mean"], 0.1)


if __name__ == "__main__":
    unittest.main()
