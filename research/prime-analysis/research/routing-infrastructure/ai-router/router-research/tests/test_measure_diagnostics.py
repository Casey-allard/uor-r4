import unittest

import numpy as np

import hyperbolic_router_so8 as hr


class MeasureDiagnosticsTest(unittest.TestCase):
    def test_shell_boundary_tangent_matches_linear_and_phi_modes(self):
        shell_ids = np.array([0.0, 1.0, 2.0, 3.0], dtype=np.float64)
        linear = hr.shell_boundary_tangent(shell_ids, delta_r=3.0, shell_mode="linear")
        phi_log = hr.shell_boundary_tangent(shell_ids, delta_r=3.0, shell_mode="phi_log")
        phi_phase = hr.shell_boundary_tangent(shell_ids, delta_r=3.0, shell_mode="phi_phase")

        np.testing.assert_allclose(linear, np.array([0.0, 3.0, 6.0, 9.0], dtype=np.float64))
        np.testing.assert_allclose(phi_log, 3.0 * (np.power(hr.PHI, shell_ids) - 1.0))
        np.testing.assert_allclose(phi_phase, phi_log)

    def test_h4_mass_shell_metric_matches_its_boundaries(self):
        delta_r = 0.35
        shell_ids = np.array([0.0, 1.0, 2.0, 3.0], dtype=np.float64)
        boundaries = hr.shell_boundary_tangent(shell_ids, delta_r=delta_r, shell_mode="h4_mass")
        shell, shell_frac, shell_cont = hr.shell_metric_components(boundaries, delta_r=delta_r, shell_mode="h4_mass")

        np.testing.assert_array_equal(shell, shell_ids.astype(np.int64))
        np.testing.assert_allclose(shell_cont, shell_ids, atol=1e-6)
        np.testing.assert_allclose(shell_frac, 0.0, atol=1e-6)

    def test_h4_mass_phi_shell_metric_matches_its_boundaries(self):
        delta_r = 0.35
        shell_ids = np.array([0.0, 1.0, 2.0, 3.0], dtype=np.float64)
        boundaries = hr.shell_boundary_tangent(shell_ids, delta_r=delta_r, shell_mode="h4_mass_phi")
        shell, shell_frac, shell_cont = hr.shell_metric_components(boundaries, delta_r=delta_r, shell_mode="h4_mass_phi")

        np.testing.assert_allclose(shell_cont, shell_ids, atol=1e-6)
        self.assertTrue(np.all(shell_frac < 1e-6))
        np.testing.assert_array_equal(shell, np.floor(shell_cont + 1e-8).astype(np.int64))

    def test_shell_measure_diagnostics_tracks_h4_mass_profile(self):
        shell_ids = np.arange(4, dtype=np.int64)
        lower = hr.shell_boundary_tangent(shell_ids, delta_r=0.15, shell_mode="linear")
        upper = hr.shell_boundary_tangent(shell_ids + 1, delta_r=0.15, shell_mode="linear")
        expected_mass = hr.h4_shell_mass(2.0 * lower, 2.0 * upper)
        counts = np.maximum(1, np.round(4000.0 * expected_mass / np.sum(expected_mass)).astype(np.int64))
        shell = np.repeat(shell_ids, counts)

        diag = hr.shell_measure_diagnostics(shell, delta_r=0.15, shell_mode="linear")

        self.assertEqual(diag["shell_mass_shells_used"], 4)
        self.assertLess(diag["shell_mass_error_l1"], 0.03)
        self.assertLess(diag["shell_mass_error_max"], 0.02)
        self.assertLess(diag["shell_mass_kl"], 0.01)
        self.assertGreater(diag["shell_mass_corr"], 0.999)

    def test_shell_measure_diagnostics_is_uniform_under_h4_mass_shells(self):
        shell_ids = np.arange(5, dtype=np.int64)
        shell = np.repeat(shell_ids, 200)
        diag = hr.shell_measure_diagnostics(shell, delta_r=0.25, shell_mode="h4_mass")

        self.assertEqual(diag["shell_mass_shells_used"], 5)
        self.assertLess(diag["shell_mass_error_l1"], 1e-10)
        self.assertLess(diag["shell_mass_error_max"], 1e-10)
        self.assertLess(diag["shell_mass_kl"], 1e-10)
        self.assertGreater(diag["shell_mass_corr"], 0.999999)

    def test_shell_measure_diagnostics_tracks_h4_mass_phi_profile(self):
        shell_ids = np.arange(5, dtype=np.int64)
        lower = hr.shell_boundary_tangent(shell_ids, delta_r=0.25, shell_mode="h4_mass_phi")
        upper = hr.shell_boundary_tangent(shell_ids + 1, delta_r=0.25, shell_mode="h4_mass_phi")
        expected_mass = hr.h4_shell_mass(2.0 * lower, 2.0 * upper)
        counts = np.maximum(1, np.round(4000.0 * expected_mass / np.sum(expected_mass)).astype(np.int64))
        shell = np.repeat(shell_ids, counts)
        diag = hr.shell_measure_diagnostics(shell, delta_r=0.25, shell_mode="h4_mass_phi")

        self.assertEqual(diag["shell_mass_shells_used"], 5)
        self.assertLess(diag["shell_mass_error_l1"], 0.03)
        self.assertLess(diag["shell_mass_error_max"], 0.02)
        self.assertLess(diag["shell_mass_kl"], 0.01)
        self.assertGreater(diag["shell_mass_corr"], 0.999)

    def test_hopf_angular_measure_diagnostics_detects_uniform_bins(self):
        chi_bins = 4
        theta_bins = 8
        rows = []
        for c_idx in range(chi_bins):
            chi_u = (c_idx + 0.5) / chi_bins
            chi = np.arcsin(np.sqrt(chi_u))
            for t1_idx in range(theta_bins):
                theta1 = -np.pi + (2.0 * np.pi) * (t1_idx + 0.5) / theta_bins
                for t2_idx in range(theta_bins):
                    theta2 = -np.pi + (2.0 * np.pi) * (t2_idx + 0.5) / theta_bins
                    rho1 = np.cos(chi)
                    rho2 = np.sin(chi)
                    rows.append(
                        [
                            rho1 * np.cos(theta1),
                            rho1 * np.sin(theta1),
                            rho2 * np.cos(theta2),
                            rho2 * np.sin(theta2),
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                        ]
                    )
        z = np.asarray(rows, dtype=np.float64)
        diag = hr.hopf_angular_measure_diagnostics(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            chi_bins=chi_bins,
            theta_bins=theta_bins,
        )

        self.assertLess(diag["hopf_chi_mass_error"], 1e-10)
        self.assertLess(diag["hopf_theta1_mass_error"], 1e-10)
        self.assertLess(diag["hopf_theta2_mass_error"], 1e-10)
        self.assertLess(diag["hopf_angular_mass_error"], 1e-10)
        self.assertAlmostEqual(diag["hopf_theta1_entropy"], np.log(theta_bins), places=6)
        self.assertAlmostEqual(diag["hopf_theta2_entropy"], np.log(theta_bins), places=6)

    def test_hopf_base_measure_diagnostics_detects_uniform_base_bins(self):
        chi_bins = 4
        delta_bins = 8
        alpha_bins = 8
        rows = []
        chi_edges = np.linspace(0.0, 1.0, chi_bins + 1)
        delta_edges = np.linspace(-np.pi, np.pi, delta_bins + 1)
        alpha_edges = np.linspace(-np.pi, np.pi, alpha_bins + 1)
        for c_idx in range(chi_bins):
            chi_u = 0.5 * (chi_edges[c_idx] + chi_edges[c_idx + 1])
            chi = np.arcsin(np.sqrt(chi_u))
            for d_idx in range(delta_bins):
                delta = 0.5 * (delta_edges[d_idx] + delta_edges[d_idx + 1])
                for a_idx in range(alpha_bins):
                    alpha = 0.5 * (alpha_edges[a_idx] + alpha_edges[a_idx + 1])
                    theta1 = hr.wrap_to_pi(alpha + 0.5 * delta)
                    theta2 = hr.wrap_to_pi(alpha - 0.5 * delta)
                    rho1 = np.cos(chi)
                    rho2 = np.sin(chi)
                    rows.append(
                        [
                            rho1 * np.cos(theta1),
                            rho1 * np.sin(theta1),
                            rho2 * np.cos(theta2),
                            rho2 * np.sin(theta2),
                            0.0,
                            0.0,
                            0.0,
                            0.0,
                        ]
                    )
        z = np.asarray(rows, dtype=np.float64)
        diag = hr.hopf_base_measure_diagnostics(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            chi_bins=chi_bins,
            delta_bins=delta_bins,
            alpha_bins=alpha_bins,
        )

        self.assertLess(diag["hopf_base_mass_error"], 1e-10)
        self.assertLess(diag["hopf_delta_mass_error"], 1e-10)
        self.assertGreater(diag["hopf_alpha_entropy"], np.log(alpha_bins) - 0.2)

    def test_hopf_phase_transport_diagnostics_follow_connection_law(self):
        eta = np.array([0.10 * np.pi, 0.25 * np.pi, 0.40 * np.pi], dtype=np.float64)
        delta = np.array([0.8, -1.2, 0.5], dtype=np.float64)
        alpha = np.array([-0.2, 0.1, 0.7], dtype=np.float64)
        rows = []
        for e, dlt, alp in zip(eta, delta, alpha):
            theta1 = hr.wrap_to_pi(alp + 0.5 * dlt)
            theta2 = hr.wrap_to_pi(alp - 0.5 * dlt)
            rho1 = np.cos(e)
            rho2 = np.sin(e)
            rows.append(
                [
                    rho1 * np.cos(theta1),
                    rho1 * np.sin(theta1),
                    rho2 * np.cos(theta2),
                    rho2 * np.sin(theta2),
                    0.0,
                    0.0,
                    0.0,
                    0.0,
                ]
            )
        z = np.asarray(rows, dtype=np.float64)
        comp = hr.hopf_phase_transport_components(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            phase_transport_lambda=1.0,
        )
        expected = hr.wrap_to_pi(0.5 * np.cos(2.0 * eta) * delta)
        np.testing.assert_allclose(comp["transport_phase_shift"], expected, atol=1e-8)
        diag = hr.hopf_phase_transport_diagnostics(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            phase_transport_lambda=1.0,
        )
        self.assertGreaterEqual(diag["phase_transport_coherence"], -1.0)
        self.assertLessEqual(diag["phase_transport_coherence"], 1.0)
        self.assertGreater(diag["phase_transport_shift_abs_max"], 0.0)

    def test_hopf_sector_routing_diagnostics_detect_sector_compactness(self):
        z = np.array(
            [
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector = np.array([0, 0, 1, 1], dtype=np.int64)
        diag = hr.hopf_sector_routing_diagnostics(
            z,
            sector,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            alpha_bins=8,
        )

        self.assertEqual(diag["hopf_sector_groups_used"], 2)
        self.assertLess(diag["hopf_sector_chi_std_mean"], 1e-10)
        self.assertLess(diag["hopf_sector_delta_cvar_mean"], 1e-10)
        self.assertGreaterEqual(diag["hopf_sector_alpha_entropy_gap"], 0.0)

    def test_route_entropy_radius_diagnostics_detects_increasing_entropy(self):
        shell = np.repeat(np.array([0, 1, 2, 3], dtype=np.int64), 40)
        sector = np.concatenate(
            [
                np.zeros(40, dtype=np.int64),
                np.tile(np.array([0, 1], dtype=np.int64), 20),
                np.tile(np.array([0, 1, 2, 3], dtype=np.int64), 10),
                np.tile(np.arange(8, dtype=np.int64), 5),
            ]
        )

        diag = hr.route_entropy_radius_diagnostics(shell, sector)

        self.assertEqual(diag["route_entropy_shells_used"], 4)
        self.assertGreater(diag["route_entropy_radius_corr"], 0.95)
        self.assertGreater(diag["route_entropy_radius_slope"], 0.1)

    def test_geodesic_neighborhood_diagnostics_is_identity_for_same_coordinates(self):
        rs = np.random.RandomState(7)
        v = 0.15 * rs.randn(64, 8)
        diag = hr.geodesic_neighborhood_diagnostics(v, v.copy(), max_points=64, k=6, seed=5)

        self.assertEqual(diag["geodesic_knn_points_used"], 64)
        self.assertEqual(diag["geodesic_knn_overlap_k"], 6.0)
        self.assertAlmostEqual(diag["geodesic_knn_overlap_mean"], 1.0, places=7)
        self.assertAlmostEqual(diag["geodesic_knn_jaccard_mean"], 1.0, places=7)


if __name__ == "__main__":
    unittest.main()
