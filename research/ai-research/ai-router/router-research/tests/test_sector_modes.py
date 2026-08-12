import unittest

import numpy as np

import hyperbolic_router_so8 as hr


class TestSectorModes(unittest.TestCase):
    def test_all_sector_modes_produce_valid_ids(self):
        np.random.seed(0)
        n, d, k = 64, 8, 9
        hybrid_local_k = 4
        v = np.random.randn(n, d)
        chart = hr.Chart(R=np.eye(d), s_global=None, S_radial=None, scale_mode="global")
        U = hr.normalize_rows(v)
        C = hr.spherical_kmeans(U, K=k, iters=5, seed=0)

        modes = ["kmeans", "phase2", "phase4d", "phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local", "complex2"]
        for mode in modes:
            shell, sector, _u, _z = hr.route_addresses(
                v,
                delta_r=2.0,
                C=C,
                chart=chart,
                sector_mode=mode,
                phase_dim_i=0,
                phase_dim_j=1,
                phase4_dim_i=0,
                phase4_dim_j=2,
                phase4_dim_k=4,
                phase4_dim_l=6,
                field4_dim_i=1,
                field4_dim_j=3,
                field4_dim_k=5,
                field4_dim_l=7,
                complex_dim_i=1,
                complex_dim_j=3,
                K=k,
                time_pressure_lambda=0.0,
                tau=1.0,
                adaptive_min_pair_bins=2,
                adaptive_time_growth=1.0,
                adaptive_balance=1.0,
                adaptive_angle_growth=0.35,
                hybrid_local_k=hybrid_local_k,
                hybrid_complex_roots=4,
                hybrid_local_min_k=1,
                hybrid_local_target=0.60,
                hybrid_local_hysteresis=0.05,
                hybrid_local_converge_lambda=1.0,
            )
            self.assertEqual(shell.shape[0], n)
            self.assertEqual(sector.shape[0], n)
            self.assertTrue(np.all(sector >= 0))
            if mode == "phase4d_complex_local":
                sector_upper = k * hybrid_local_k
            else:
                sector_upper = k
            self.assertTrue(np.all(sector < sector_upper))

    def test_hopf_transport_phase_vanishes_at_balanced_amplitude(self):
        eta = 0.25 * np.pi
        delta = 1.1
        alpha = -0.4
        theta1 = hr.wrap_to_pi(alpha + 0.5 * delta)
        theta2 = hr.wrap_to_pi(alpha - 0.5 * delta)
        rho1 = np.cos(eta)
        rho2 = np.sin(eta)
        z = np.array(
            [[
                rho1 * np.cos(theta1),
                rho1 * np.sin(theta1),
                rho2 * np.cos(theta2),
                rho2 * np.sin(theta2),
                0.0, 0.0, 0.0, 0.0,
            ]],
            dtype=np.float64,
        )
        comp = hr.hopf_phase_transport_components(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            phase_transport_lambda=1.0,
        )
        self.assertAlmostEqual(float(comp["transport_phase_shift"][0]), 0.0, places=7)
        self.assertAlmostEqual(float(comp["transported_alpha"][0]), alpha, places=7)

    def test_hopf_transport_triplet_budget_keeps_alpha_live(self):
        kchi, kdelta, kalpha = hr.allocate_triplet_bins_budget(
            25,
            min_first=2,
            min_second=2,
            min_third=2,
        )
        self.assertGreaterEqual(kchi, 2)
        self.assertGreaterEqual(kdelta, 2)
        self.assertGreaterEqual(kalpha, 2)
        self.assertLessEqual(kchi * kdelta * kalpha, 25)

    def test_hopf_transport_complex_diagnostics_include_field_shift(self):
        z = np.array(
            [[
                1.0, 0.0, 0.0, 1.0,
                0.0, 2.0, 0.0, 0.0,
            ]],
            dtype=np.float64,
        )
        diag = hr.hopf_phase_transport_complex_diagnostics(
            z,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            complex_dim_i=4,
            complex_dim_j=5,
            phase_transport_lambda=0.0,
            phase_field_lambda=1.0,
        )
        self.assertGreater(diag["phase_transport_field_shift_abs_mean"], 0.0)
        self.assertGreater(diag["phase_transport_shift_abs_mean"], 0.0)

    def test_hopf_transport_complex_changes_sector_when_field_phase_is_live(self):
        z = np.array(
            [
                [1.0, 0.0, 0.0, 1.0, -2.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0, 1.0, -2.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector_base, *_ = hr.assign_sectors_phase4d_hopf_transport(
            z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            phase_transport_lambda=0.0,
            hopf_chi_bins=2,
        )
        sector_cplx, _kchi, _kdelta, kalpha = hr.assign_sectors_phase4d_hopf_transport_complex(
            z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            complex_dim_i=4,
            complex_dim_j=5,
            phase_transport_lambda=0.0,
            phase_field_lambda=1.0,
            hopf_chi_bins=2,
        )
        self.assertTrue(np.all(kalpha >= 2))
        self.assertTrue(np.all(sector_cplx >= 0))
        self.assertTrue(np.all(sector_cplx < 25))
        self.assertTrue(np.any(sector_cplx != sector_base))

    def test_hopf_product_phase_diagnostics_include_field_shift(self):
        rho = np.sqrt(0.5)
        z = np.array(
            [[
                rho, 0.0, rho, 0.0,
                0.0, rho, 0.0, rho,
            ]],
            dtype=np.float64,
        )
        diag = hr.hopf_product_phase_diagnostics(
            z,
            route_dim_i=0,
            route_dim_j=1,
            route_dim_k=2,
            route_dim_l=3,
            field_dim_i=4,
            field_dim_j=5,
            field_dim_k=6,
            field_dim_l=7,
            phase_transport_lambda=0.0,
            phase_field_lambda=1.0,
        )
        self.assertGreater(diag["phase_transport_field_shift_abs_mean"], 0.0)
        self.assertGreater(diag["phase_transport_field_weight_abs_mean"], 0.0)
        self.assertGreater(diag["phase_transport_shift_abs_mean"], 0.0)

    def test_hopf_product_phase_diagnostics_zero_field_term_when_disabled(self):
        rho = np.sqrt(0.5)
        z = np.array(
            [[
                rho, 0.0, rho, 0.0,
                0.0, rho, 0.0, rho,
            ]],
            dtype=np.float64,
        )
        diag = hr.hopf_product_phase_diagnostics(
            z,
            route_dim_i=0,
            route_dim_j=1,
            route_dim_k=2,
            route_dim_l=3,
            field_dim_i=4,
            field_dim_j=5,
            field_dim_k=6,
            field_dim_l=7,
            phase_transport_lambda=0.0,
            phase_field_lambda=0.0,
        )
        self.assertEqual(diag["phase_transport_field_shift_abs_mean"], 0.0)
        self.assertGreater(diag["phase_transport_field_weight_abs_mean"], 0.0)
        self.assertEqual(diag["phase_transport_shift_abs_mean"], 0.0)

    def test_hopf_product_phase_changes_sector_when_field_phase_is_live(self):
        rho = np.sqrt(0.5)
        v = np.array(
            [
                [rho, 0.0, rho, 0.0, 0.0, rho, 0.0, rho],
                [rho, 0.0, rho, 0.0, 0.0, rho, 0.0, rho],
            ],
            dtype=np.float64,
        )
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell_base, sector_base, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=4,
            complex_dim_j=5,
            K=25,
            shell_mode="phi_log",
        )
        shell_prod, sector_prod, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_product_phase",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            field4_dim_i=4,
            field4_dim_j=5,
            field4_dim_k=6,
            field4_dim_l=7,
            complex_dim_i=4,
            complex_dim_j=5,
            K=25,
            phase_transport_lambda=0.0,
            phase_field_lambda=1.0,
            hopf_chi_bins=2,
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(shell_base, shell_prod)
        self.assertTrue(np.all(sector_prod >= 0))
        self.assertTrue(np.all(sector_prod < 25))
        self.assertTrue(np.any(sector_prod != sector_base))

    def test_product_phase_shell_controller_can_fall_back_to_identity(self):
        z = np.array(
            [
                [2.0, 0.0, 1.5, 0.0, 0.5, 0.0, 0.25, 0.0],
                [1.8, 0.0, 1.2, 0.0, 0.4, 0.0, 0.2, 0.0],
            ],
            dtype=np.float64,
        )
        ctrl = hr.product_phase_shell_controller(
            z=z,
            K=25,
            dim_i=0,
            dim_j=2,
            dim_k=4,
            dim_l=6,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            product_shell_control_mode="gated",
            product_shell_gate_threshold=10.0,
        )
        np.testing.assert_allclose(ctrl["shell_multiplier"], np.ones_like(ctrl["shell_multiplier"]))
        self.assertFalse(np.any(ctrl["active_mask"]))
        self.assertTrue(np.all(ctrl["band_state"] == 0))

    def test_product_phase_shell_controller_banded_uses_small_state_set(self):
        z = np.array(
            [
                [2.0, 0.0, 1.5, 0.0, 0.5, 0.0, 0.25, 0.0],
                [0.9, 0.0, 0.8, 0.0, 0.5, 0.0, 0.4, 0.0],
                [0.4, 0.0, 0.6, 0.0, 0.3, 0.0, 0.8, 0.0],
            ],
            dtype=np.float64,
        )
        ctrl = hr.product_phase_shell_controller(
            z=z,
            K=25,
            dim_i=0,
            dim_j=2,
            dim_k=4,
            dim_l=6,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            product_shell_control_mode="banded",
            product_shell_gate_threshold=0.35,
        )
        states = np.unique(ctrl["band_state"])
        self.assertLessEqual(states.size, 3)
        self.assertTrue(np.all(states >= 0))
        self.assertTrue(np.all(states <= 2))

    def test_product_phase_shell_control_changes_shell_not_sector(self):
        v = np.array(
            [
                [2.0, 0.1, 1.4, 0.0, 1.1, 0.0, 0.7, 0.0],
                [1.7, 0.0, 1.1, 0.1, 0.9, 0.0, 0.5, 0.0],
                [1.3, 0.0, 0.9, 0.0, 0.7, 0.0, 0.6, 0.0],
            ],
            dtype=np.float64,
        )
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        kwargs = dict(
            delta_r=3.6,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_product_phase",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            field4_dim_i=1,
            field4_dim_j=3,
            field4_dim_k=5,
            field4_dim_l=7,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
            phase_transport_lambda=0.0,
            phase_field_lambda=1.5,
            hopf_chi_bins=2,
        )
        shell_cont, sector_cont, _u_cont, _z_cont = hr.route_addresses(
            v,
            product_shell_control_mode="continuous",
            product_shell_gate_threshold=0.0,
            **kwargs,
        )
        shell_gate, sector_gate, _u_gate, _z_gate = hr.route_addresses(
            v,
            product_shell_control_mode="gated",
            product_shell_gate_threshold=0.3,
            **kwargs,
        )
        np.testing.assert_array_equal(sector_cont, sector_gate)
        self.assertTrue(np.any(shell_cont != shell_gate))

    def test_route_coordinate_surface_matches_mode_family(self):
        v = np.array([[1.0, -2.0, 0.5, 1.5, 0.25, -0.75, 0.4, -0.2]], dtype=np.float64)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([0.1, -0.2, 0.3, 0.0, 0.2, -0.1, 0.05, -0.05], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        full_chart = hr.apply_chart(v, chart)
        iso_chart = hr.apply_chart_isometric(v, chart)

        for mode in ("phase4d_hopf_base", "phase4d_hopf_base_ball", "phase4d_hopf", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase"):
            np.testing.assert_allclose(
                hr.route_coordinate(v, chart, sector_mode=mode, route_scale_lambda=0.0),
                full_chart,
            )

        np.testing.assert_allclose(
            hr.route_coordinate(v, chart, sector_mode="phase4d_hopf_iso", route_scale_lambda=0.0),
            iso_chart,
        )
        np.testing.assert_allclose(
            hr.route_coordinate(v, chart, sector_mode="phase4d_hopf_fib_band_iso", route_scale_lambda=0.0),
            iso_chart,
        )
        np.testing.assert_allclose(
            hr.route_coordinate(v, chart, sector_mode="phase4d_hopf_fib_band_bound", route_scale_lambda=0.0),
            iso_chart,
        )

    def test_hybrid_route_one_matches_batch_route(self):
        np.random.seed(1)
        v = np.random.randn(8, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell, sector, _u, z = hr.route_addresses(
            v,
            delta_r=2.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_complex_local",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=4,
            complex_dim_j=5,
            K=9,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=0.8,
            adaptive_shell_balance=0.3,
            adaptive_converge_lambda=1.2,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ratio",
            shell_mode="linear",
            hybrid_local_k=4,
            hybrid_complex_roots=4,
            hybrid_local_min_k=1,
            hybrid_local_target=0.60,
            hybrid_local_hysteresis=0.05,
            hybrid_local_converge_lambda=1.0,
        )
        for i in range(v.shape[0]):
            key, z1 = hr.route_one(
                v[i],
                delta_r=2.0,
                C=None,
                chart=chart,
                sector_mode="phase4d_complex_local",
                phase_dim_i=0,
                phase_dim_j=1,
                phase4_dim_i=0,
                phase4_dim_j=1,
                phase4_dim_k=2,
                phase4_dim_l=3,
                complex_dim_i=4,
                complex_dim_j=5,
                K=9,
                time_pressure_lambda=0.0,
                tau=1.0,
                adaptive_min_pair_bins=2,
                adaptive_time_growth=1.0,
                adaptive_balance=1.0,
                adaptive_angle_growth=0.35,
                adaptive_shell_growth=0.8,
                adaptive_shell_balance=0.3,
                adaptive_converge_lambda=1.2,
                adaptive_converge_target=0.85,
                adaptive_converge_hysteresis=0.05,
                adaptive_converge_mode="phi_ratio",
                hybrid_local_k=4,
                hybrid_complex_roots=4,
                hybrid_local_min_k=1,
                hybrid_local_target=0.60,
                hybrid_local_hysteresis=0.05,
                hybrid_local_converge_lambda=1.0,
            )
            self.assertEqual(key, (int(shell[i]), int(sector[i])))
            np.testing.assert_allclose(z1, z[i])

    def test_hybrid_local_controller_can_open_bins(self):
        np.random.seed(7)
        z = np.random.randn(32, 8)
        z[:, 1] *= 4.0
        z[:, 3] *= 4.0
        hybrid = hr.phase4d_complex_local_components(
            z=z,
            K=9,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            complex_dim_i=1,
            complex_dim_j=3,
            delta_r=2.0,
            tau=1.0,
            time_pressure_lambda=0.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=0.8,
            adaptive_shell_balance=0.3,
            adaptive_converge_lambda=1.2,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ratio",
            shell_mode="linear",
            hybrid_local_k=4,
            hybrid_complex_roots=4,
            hybrid_local_min_k=1,
            hybrid_local_target=0.05,
            hybrid_local_hysteresis=0.0,
            hybrid_local_converge_lambda=0.0,
        )
        self.assertGreater(int(np.max(hybrid["local_k_eff"])), 1)

    def test_hopf_base_sector_is_invariant_to_common_fiber_phase(self):
        np.random.seed(9)
        v = np.random.randn(48, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell_a, sector_a, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=0,
            complex_dim_j=1,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            shell_mode="phi_log",
        )

        phi = 0.73
        rot = np.array([[np.cos(phi), -np.sin(phi)], [np.sin(phi), np.cos(phi)]], dtype=np.float64)
        v_shift = v.copy()
        v_shift[:, 0:2] = v_shift[:, 0:2] @ rot.T
        v_shift[:, 2:4] = v_shift[:, 2:4] @ rot.T

        shell_b, sector_b, _u2, _z2 = hr.route_addresses(
            v_shift,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=1,
            phase4_dim_k=2,
            phase4_dim_l=3,
            complex_dim_i=0,
            complex_dim_j=1,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.0,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            shell_mode="phi_log",
        )

        np.testing.assert_array_equal(shell_a, shell_b)
        np.testing.assert_array_equal(sector_a, sector_b)

    def test_phase_coupled_shell_mode_routes_valid_shells(self):
        np.random.seed(11)
        v = np.random.randn(32, 8)
        chart = hr.Chart(R=np.eye(8), s_global=None, S_radial=None, scale_mode="global")
        shell_log, sector_log, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.6,
            C=None,
            chart=chart,
            sector_mode="phase4d_adaptive",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_phase, sector_phase, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.6,
            C=None,
            chart=chart,
            sector_mode="phase4d_adaptive",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            time_pressure_lambda=0.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_phase",
            shell_phase_coupling=0.3,
        )
        self.assertTrue(np.all(shell_phase >= 0))
        self.assertTrue(np.all(sector_phase >= 0))
        self.assertTrue(np.all(sector_phase < 25))
        self.assertGreater(int(np.count_nonzero(shell_phase != shell_log)), 0)
        np.testing.assert_array_equal(sector_phase.shape, sector_log.shape)

    def test_phase4d_hopf_ball_keeps_sectors_but_changes_shells(self):
        np.random.seed(13)
        v = 0.4 * np.random.randn(32, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.full((8,), np.log(1.4), dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        shell_hopf, sector_hopf, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_ball, sector_ball, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_ball",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(sector_hopf, sector_ball)
        self.assertGreater(int(np.count_nonzero(shell_hopf != shell_ball)), 0)

    def test_phase4d_hopf_base_ball_keeps_base_sectors_but_changes_shells(self):
        np.random.seed(19)
        v = 0.4 * np.random.randn(32, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.full((8,), np.log(1.4), dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        shell_base, sector_base, _u1, _z1 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_base_ball, sector_base_ball, _u2, _z2 = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_base_ball",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(sector_base, sector_base_ball)
        self.assertGreater(int(np.count_nonzero(shell_base != shell_base_ball)), 0)

    def test_phase4d_hopf_iso_matches_hopf_when_chart_is_rotation_only(self):
        np.random.seed(17)
        v = 0.35 * np.random.randn(32, 8)
        q, _ = np.linalg.qr(np.random.randn(8, 8))
        chart = hr.Chart(R=q, s_global=None, S_radial=None, scale_mode="global")
        shell_hopf, sector_hopf, _u1, z_hopf = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u2, z_iso = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_iso",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(shell_hopf, shell_iso)
        np.testing.assert_array_equal(sector_hopf, sector_iso)
        np.testing.assert_allclose(z_hopf, z_iso)

    def test_phase4d_hopf_fib_band_iso_matches_band_when_chart_is_rotation_only(self):
        np.random.seed(19)
        v = 0.35 * np.random.randn(32, 8)
        q, _ = np.linalg.qr(np.random.randn(8, 8))
        chart = hr.Chart(R=q, s_global=None, S_radial=None, scale_mode="global")
        shell_band, sector_band, _u1, z_band = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_fib_band",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u2, z_iso = hr.route_addresses(
            v,
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_fib_band_iso",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        np.testing.assert_array_equal(shell_band, shell_iso)
        np.testing.assert_array_equal(sector_band, sector_iso)
        np.testing.assert_allclose(z_band, z_iso)

    def test_phase4d_hopf_fib_band_bound_lambda_zero_matches_iso(self):
        rs = np.random.RandomState(23)
        v = 0.25 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.6), np.log(0.8), np.log(1.3), np.log(0.9), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        kwargs = dict(
            delta_r=3.6,
            C=None,
            chart=chart,
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=1.6,
            adaptive_shell_balance=1.0,
            adaptive_converge_lambda=0.65,
            adaptive_converge_target=0.85,
            adaptive_converge_hysteresis=0.05,
            adaptive_converge_mode="phi_ladder",
            shell_mode="phi_log",
        )
        shell_iso, sector_iso, _u_iso, z_iso = hr.route_addresses(
            v,
            sector_mode="phase4d_hopf_fib_band_iso",
            **kwargs,
        )
        shell_bound, sector_bound, _u_bound, z_bound = hr.route_addresses(
            v,
            sector_mode="phase4d_hopf_fib_band_bound",
            route_scale_lambda=0.0,
            **kwargs,
        )
        np.testing.assert_array_equal(shell_iso, shell_bound)
        np.testing.assert_array_equal(sector_iso, sector_bound)
        np.testing.assert_allclose(z_iso, z_bound)


if __name__ == "__main__":
    unittest.main()
