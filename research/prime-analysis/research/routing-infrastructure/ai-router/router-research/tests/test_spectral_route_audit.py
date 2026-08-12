import unittest

import numpy as np

from tools import spectral_route_audit as audit


class SpectralRouteAuditTest(unittest.TestCase):
    def test_build_knn_graph_is_symmetric(self):
        x = np.array(
            [
                [0.0, 0.0],
                [1.0, 0.0],
                [0.0, 1.0],
                [1.0, 1.0],
            ],
            dtype=np.float64,
        )
        a, sigma = audit.build_knn_graph(x, knn_k=2)
        self.assertGreater(sigma, 0.0)
        np.testing.assert_allclose(a, a.T)
        np.testing.assert_allclose(np.diag(a), 0.0)

    def test_normalized_laplacian_detects_disconnected_components(self):
        a = np.array(
            [
                [0.0, 1.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
            dtype=np.float64,
        )
        l = audit.normalized_laplacian(a)
        evals, _ = np.linalg.eigh(l)
        self.assertEqual(int(np.count_nonzero(evals <= 1e-8)), 2)

    def test_spectral_metrics_return_positive_gap_for_connected_graph(self):
        x = np.stack(
            [
                np.linspace(-1.0, 1.0, 16),
                np.zeros(16, dtype=np.float64),
            ],
            axis=1,
        )
        shell = np.arange(16, dtype=np.int64) // 4
        sector = np.arange(16, dtype=np.int64) % 4
        metrics = audit.spectral_metrics(x, shell, sector, knn_k=3, lowfreq_modes=4)
        self.assertEqual(metrics["spectral_zero_eigs"], 1)
        self.assertGreater(metrics["spectral_lambda2"], 0.0)
        self.assertGreaterEqual(metrics["shell_lowfreq_energy"], 0.0)
        self.assertLessEqual(metrics["shell_lowfreq_energy"], 1.0)
        self.assertGreaterEqual(metrics["sector_lowfreq_energy"], 0.0)
        self.assertLessEqual(metrics["sector_lowfreq_energy"], 1.0)

    def test_low_frequency_matrix_energy_stays_in_unit_interval(self):
        x = np.stack(
            [
                np.linspace(-1.0, 1.0, 12),
                np.zeros(12, dtype=np.float64),
            ],
            axis=1,
        )
        decomp = audit.spectral_decomposition(x, knn_k=3)
        labels = np.zeros((12, 3), dtype=np.float64)
        labels[:4, 0] = 1.0
        labels[4:8, 1] = 1.0
        labels[8:, 2] = 1.0
        energy = audit.low_frequency_matrix_energy(decomp["evecs"], labels, modes=4)
        self.assertGreaterEqual(energy, 0.0)
        self.assertLessEqual(energy, 1.0)

    def test_dirichlet_energy_prefers_smoother_signal(self):
        x = np.stack(
            [
                np.linspace(-1.0, 1.0, 20),
                np.zeros(20, dtype=np.float64),
            ],
            axis=1,
        )
        decomp = audit.spectral_decomposition(x, knn_k=3)
        smooth = np.linspace(-1.0, 1.0, 20, dtype=np.float64)[:, None]
        rough = np.sign(np.sin(np.arange(20, dtype=np.float64) * np.pi))[:, None]
        smooth_energy = audit.normalized_dirichlet_energy(decomp["laplacian"], smooth)
        rough_energy = audit.normalized_dirichlet_energy(decomp["laplacian"], rough)
        self.assertLessEqual(smooth_energy, rough_energy)


if __name__ == "__main__":
    unittest.main()
