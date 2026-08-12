import unittest

import numpy as np

import hyperbolic_router_so8 as hr


class TestPoincareAlignment(unittest.TestCase):
    def test_rotation_preserves_poincare_alignment(self):
        rs = np.random.RandomState(0)
        v = 0.25 * rs.randn(64, 8)
        q, _ = np.linalg.qr(rs.randn(8, 8))
        chart = hr.Chart(R=q, s_global=None, S_radial=None, scale_mode="global")
        z = hr.apply_chart(v, chart)
        diag = hr.poincare_alignment_diagnostics(v, z, max_pairs=256, seed=7)

        self.assertEqual(diag["poincare_alignment_pairs_used"], 256)
        self.assertLess(diag["poincare_alignment_radial_mae"], 1e-10)
        self.assertLess(diag["poincare_alignment_pair_mae"], 1e-10)
        self.assertGreater(diag["poincare_alignment_radial_corr"], 0.999999)
        self.assertGreater(diag["poincare_alignment_pair_corr"], 0.999999)

    def test_global_scale_breaks_poincare_alignment(self):
        rs = np.random.RandomState(1)
        v = 0.20 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.full((8,), np.log(1.35), dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        z = hr.apply_chart(v, chart)
        diag = hr.poincare_alignment_diagnostics(v, z, max_pairs=256, seed=11)

        self.assertEqual(diag["poincare_alignment_pairs_used"], 256)
        self.assertGreater(diag["poincare_alignment_radial_mae"], 1e-3)
        self.assertGreater(diag["poincare_alignment_pair_mae"], 1e-3)
        self.assertGreater(diag["poincare_alignment_radial_rel_mean"], 1e-3)
        self.assertGreater(diag["poincare_alignment_pair_rel_mean"], 1e-3)

    def test_isometric_chart_ignores_scale_distortion(self):
        rs = np.random.RandomState(2)
        v = 0.20 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.8), np.log(0.8), np.log(1.4), np.log(0.7), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        z_scaled = hr.apply_chart(v, chart)
        z_iso = hr.apply_chart_isometric(v, chart)
        diag_scaled = hr.poincare_alignment_diagnostics(v, z_scaled, max_pairs=256, seed=13)
        diag_iso = hr.poincare_alignment_diagnostics(v, z_iso, max_pairs=256, seed=13)

        self.assertLess(diag_iso["poincare_alignment_radial_mae"], 1e-10)
        self.assertLess(diag_iso["poincare_alignment_pair_mae"], 1e-10)
        self.assertGreater(diag_scaled["poincare_alignment_radial_mae"], 1e-3)
        self.assertGreater(diag_scaled["poincare_alignment_pair_mae"], 1e-3)

    def test_route_blend_interpolates_between_isometric_and_scaled(self):
        rs = np.random.RandomState(5)
        v = 0.20 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.8), np.log(0.8), np.log(1.5), np.log(0.75), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        z_scaled = hr.apply_chart(v, chart)
        z_iso = hr.apply_chart_isometric(v, chart)
        z_mid = hr.apply_chart_route_blend(v, chart, route_scale_lambda=0.5)
        diag_scaled = hr.poincare_alignment_diagnostics(v, z_scaled, max_pairs=256, seed=23)
        diag_iso = hr.poincare_alignment_diagnostics(v, z_iso, max_pairs=256, seed=23)
        diag_mid = hr.poincare_alignment_diagnostics(v, z_mid, max_pairs=256, seed=23)

        np.testing.assert_allclose(z_iso, hr.apply_chart_route_blend(v, chart, route_scale_lambda=0.0))
        np.testing.assert_allclose(z_scaled, hr.apply_chart_route_blend(v, chart, route_scale_lambda=1.0))
        self.assertLess(diag_mid["poincare_alignment_pair_mae"], diag_scaled["poincare_alignment_pair_mae"])
        self.assertGreater(diag_mid["poincare_alignment_pair_mae"], diag_iso["poincare_alignment_pair_mae"])

    def test_phase4d_hopf_iso_route_uses_isometric_coordinate(self):
        rs = np.random.RandomState(3)
        v = 0.30 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.7), np.log(0.75), np.log(1.5), np.log(0.8), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        _shell_hopf, _sector_hopf, _u_hopf, z_hopf = hr.route_addresses(
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
        _shell_iso, _sector_iso, _u_iso, z_iso = hr.route_addresses(
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
        diag_hopf = hr.poincare_alignment_diagnostics(v, z_hopf, max_pairs=256, seed=17)
        diag_iso = hr.poincare_alignment_diagnostics(v, z_iso, max_pairs=256, seed=17)

        self.assertLess(diag_iso["poincare_alignment_radial_mae"], diag_hopf["poincare_alignment_radial_mae"])
        self.assertLess(diag_iso["poincare_alignment_pair_mae"], diag_hopf["poincare_alignment_pair_mae"])

    def test_phase4d_hopf_fib_band_iso_route_uses_isometric_coordinate(self):
        rs = np.random.RandomState(4)
        v = 0.30 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.7), np.log(0.75), np.log(1.5), np.log(0.8), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        _shell_band, _sector_band, _u_band, z_band = hr.route_addresses(
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
        _shell_iso, _sector_iso, _u_iso, z_iso = hr.route_addresses(
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
        diag_band = hr.poincare_alignment_diagnostics(v, z_band, max_pairs=256, seed=19)
        diag_iso = hr.poincare_alignment_diagnostics(v, z_iso, max_pairs=256, seed=19)

        self.assertLess(diag_iso["poincare_alignment_radial_mae"], diag_band["poincare_alignment_radial_mae"])
        self.assertLess(diag_iso["poincare_alignment_pair_mae"], diag_band["poincare_alignment_pair_mae"])

    def test_phase4d_hopf_fib_band_bound_sits_between_iso_and_band(self):
        rs = np.random.RandomState(6)
        v = 0.30 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.7), np.log(0.75), np.log(1.5), np.log(0.8), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        kwargs = dict(
            delta_r=3.0,
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
        _s_band, _c_band, _u_band, z_band = hr.route_addresses(v, sector_mode="phase4d_hopf_fib_band", **kwargs)
        _s_iso, _c_iso, _u_iso, z_iso = hr.route_addresses(v, sector_mode="phase4d_hopf_fib_band_iso", **kwargs)
        _s_bound, _c_bound, _u_bound, z_bound = hr.route_addresses(
            v,
            sector_mode="phase4d_hopf_fib_band_bound",
            route_scale_lambda=0.5,
            **kwargs,
        )
        diag_band = hr.poincare_alignment_diagnostics(v, z_band, max_pairs=256, seed=29)
        diag_iso = hr.poincare_alignment_diagnostics(v, z_iso, max_pairs=256, seed=29)
        diag_bound = hr.poincare_alignment_diagnostics(v, z_bound, max_pairs=256, seed=29)

        self.assertLess(diag_bound["poincare_alignment_pair_mae"], diag_band["poincare_alignment_pair_mae"])
        self.assertGreater(diag_bound["poincare_alignment_pair_mae"], diag_iso["poincare_alignment_pair_mae"])

    def test_full_chart_memory_keeps_route_keys_but_returns_full_chart_coordinate(self):
        rs = np.random.RandomState(7)
        v = 0.30 * rs.randn(64, 8)
        chart = hr.Chart(
            R=np.eye(8),
            s_global=np.array([np.log(1.7), np.log(0.75), np.log(1.5), np.log(0.8), 0.0, 0.0, 0.0, 0.0], dtype=np.float64),
            S_radial=None,
            scale_mode="global",
        )
        kwargs = dict(
            delta_r=3.0,
            C=None,
            chart=chart,
            sector_mode="phase4d_hopf_fib_band_bound",
            phase_dim_i=0,
            phase_dim_j=1,
            phase4_dim_i=0,
            phase4_dim_j=2,
            phase4_dim_k=4,
            phase4_dim_l=6,
            complex_dim_i=1,
            complex_dim_j=3,
            K=25,
            route_scale_lambda=0.5,
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
        shell_route, sector_route, _u_route, z_route = hr.route_addresses(
            v,
            memory_coord_mode="route_chart",
            **kwargs,
        )
        shell_full, sector_full, _u_full, z_full = hr.route_addresses(
            v,
            memory_coord_mode="full_chart",
            **kwargs,
        )
        np.testing.assert_array_equal(shell_route, shell_full)
        np.testing.assert_array_equal(sector_route, sector_full)
        np.testing.assert_allclose(z_full, hr.apply_chart(v, chart))
        self.assertGreater(
            hr.poincare_alignment_diagnostics(v, z_full, max_pairs=256, seed=31)["poincare_alignment_pair_mae"],
            hr.poincare_alignment_diagnostics(v, z_route, max_pairs=256, seed=31)["poincare_alignment_pair_mae"],
        )


if __name__ == "__main__":
    unittest.main()
