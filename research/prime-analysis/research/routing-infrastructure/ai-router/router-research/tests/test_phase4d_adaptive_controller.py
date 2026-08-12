import unittest

import numpy as np

import hyperbolic_router_so8 as hr


class AdaptiveConvergeControllerTest(unittest.TestCase):
    def test_phi_ratio_reopens_growth_beyond_fixed_cap(self):
        z = np.array(
            [
                [0.8, 0.2, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.6, 0.4, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.2, 0.8, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [6.4, 1.2, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        common = dict(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.8,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=1.8,
            adaptive_shell_balance=0.0,
            adaptive_converge_lambda=1.0,
            adaptive_converge_target=0.8,
            adaptive_converge_hysteresis=0.1,
        )

        fixed = hr.phase4d_adaptive_components(
            adaptive_converge_mode="fixed",
            **common,
        )
        phi_ratio = hr.phase4d_adaptive_components(
            adaptive_converge_mode="phi_ratio",
            **common,
        )

        target_cap = np.exp(common["adaptive_converge_target"] + common["adaptive_converge_hysteresis"])
        overflow_mask = fixed["shell_overflow"] > 1e-9

        self.assertTrue(np.any(overflow_mask))
        self.assertTrue(np.all(fixed["shell_multiplier"] <= target_cap + 1e-9))
        self.assertGreater(
            float(np.max(phi_ratio["shell_multiplier"][overflow_mask])),
            float(np.max(fixed["shell_multiplier"][overflow_mask])),
        )
        self.assertGreater(float(np.max(phi_ratio["shell_ratio_pressure"])), 0.0)

    def test_phi_ladder_quantizes_convergence_in_phi_steps(self):
        z = np.array(
            [
                [0.8, 0.2, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.6, 0.4, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.2, 0.8, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
                [6.4, 1.2, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        common = dict(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.0,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.8,
            adaptive_balance=1.0,
            adaptive_angle_growth=0.35,
            adaptive_shell_growth=1.8,
            adaptive_shell_balance=0.0,
            adaptive_converge_lambda=1.0,
            adaptive_converge_target=0.8,
            adaptive_converge_hysteresis=0.1,
        )
        ladder = hr.phase4d_adaptive_components(
            adaptive_converge_mode="phi_ladder",
            **common,
        )
        overflow = ladder["shell_overflow"]
        expected_steps = np.ceil(overflow / hr.LOG_PHI).astype(np.int64)
        expected_steps = np.where(overflow > 1e-12, np.maximum(expected_steps, 1), 0)
        np.testing.assert_array_equal(ladder["shell_ladder_steps"], expected_steps)
        expected_converge = expected_steps.astype(np.float64) * hr.LOG_PHI
        np.testing.assert_allclose(ladder["shell_converge"], expected_converge)

    def test_phi_log_shell_metric_uses_geometric_boundaries(self):
        r_eff = np.array([0.0, 3.0 * (hr.PHI - 1e-6), 3.0 * hr.PHI, 3.0 * (hr.PHI ** 2)], dtype=np.float64)
        shell, shell_frac, shell_cont = hr.shell_metric_components(r_eff, delta_r=3.0, shell_mode="phi_log")
        expected_cont = np.log1p(r_eff / 3.0) / hr.LOG_PHI
        np.testing.assert_allclose(shell_cont, expected_cont)
        np.testing.assert_array_equal(shell, np.floor(expected_cont).astype(np.int64))
        self.assertTrue(np.all(shell_frac >= 0.0))
        self.assertTrue(np.all(shell_frac < 1.0))

    def test_phi_phase_shell_metric_applies_signed_phase_bias(self):
        np.random.seed(3)
        z = np.random.randn(16, 8)
        comp = hr.phase4d_adaptive_components(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
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
            shell_phase_coupling=0.3,
        )
        self.assertGreater(float(np.max(np.abs(comp["phase_pressure"]))), 0.0)
        self.assertGreater(float(np.max(np.abs(comp["shell_phase_bias"]))), 0.0)

        r_eff = comp["r"] * comp["shell_multiplier"]
        _shell_log, _frac_log, cont_log = hr.shell_metric_components(
            r_eff,
            delta_r=3.6,
            shell_mode="phi_log",
        )
        _shell_phase, _frac_phase, cont_phase = hr.shell_metric_components(
            r_eff,
            delta_r=3.6,
            shell_mode="phi_phase",
            shell_phase_bias=comp["shell_phase_bias"],
        )

        self.assertGreater(float(np.max(np.abs(cont_phase - cont_log))), 0.0)
        np.testing.assert_allclose(cont_phase - cont_log, comp["shell_phase_bias"])

    def test_hopf_diagnostics_track_s3_latitude_and_capacity(self):
        z = np.array(
            [
                [4.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [2.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [8.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        comp = hr.phase4d_adaptive_components(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
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
        )

        self.assertTrue(np.all(comp["chi"] >= 0.0))
        self.assertTrue(np.all(comp["chi"] <= 0.5 * np.pi))
        self.assertGreaterEqual(float(comp["chi_entropy"][0]), 0.0)
        self.assertLessEqual(float(comp["chi_entropy"][0]), np.log(hr.HOPF_CHI_BINS) + 1e-9)
        self.assertLess(comp["chi"][0], 0.5 * np.pi / 2.0)
        self.assertGreater(comp["chi"][1], 0.5 * np.pi / 2.0)

        self.assertGreaterEqual(float(np.min(comp["hopf_shell_capacity"])), 9.0)
        self.assertLessEqual(float(np.max(comp["hopf_shell_capacity"])), 25.0)
        self.assertLess(float(comp["r_alpha"][0]), float(comp["r_alpha"][-1]))
        self.assertLess(float(comp["hopf_shell_capacity"][0]), float(comp["hopf_shell_capacity"][-1]))

        self.assertGreaterEqual(int(comp["hopf_k1"][0]), int(comp["hopf_k2"][0]))
        self.assertLessEqual(int(comp["hopf_k1"][1]), int(comp["hopf_k2"][1]))
        self.assertGreater(float(np.mean(comp["hopf_k1_gap"] + comp["hopf_k2_gap"])), 0.0)

    def test_phase4d_hopf_sector_assignment_uses_hopf_pair_bins(self):
        z = np.array(
            [
                [4.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [2.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector, k1, k2 = hr.assign_sectors_phase4d_hopf(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
        )
        comp = hr.phase4d_adaptive_components(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=0.0,
            adaptive_shell_balance=0.0,
            adaptive_converge_lambda=0.0,
            adaptive_converge_target=1.0,
            adaptive_converge_hysteresis=0.1,
            adaptive_converge_mode="fixed",
        )
        np.testing.assert_array_equal(k1, comp["hopf_k1"])
        np.testing.assert_array_equal(k2, comp["hopf_k2"])
        self.assertTrue(np.all(sector >= 0))
        self.assertTrue(np.all(sector < 25))

    def test_phase4d_hopf_chi_assignment_splits_on_measure_aware_chi_bins(self):
        z = np.array(
            [
                [4.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 3.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.5, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector, kchi, k1, k2 = hr.assign_sectors_phase4d_hopf_chi(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            hopf_chi_bins=2,
        )
        self.assertTrue(np.all(kchi == 2))
        self.assertTrue(np.all(k1 >= 1))
        self.assertTrue(np.all(k2 >= 1))
        self.assertTrue(np.all(sector >= 0))
        self.assertTrue(np.all(sector < 25))

        comp = hr.phase4d_adaptive_components(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=0.0,
            adaptive_shell_balance=0.0,
            adaptive_converge_lambda=0.0,
            adaptive_converge_target=1.0,
            adaptive_converge_hysteresis=0.1,
            adaptive_converge_mode="fixed",
        )
        chi_bins = np.minimum((comp["chi_u"] * 2).astype(np.int64), 1)
        self.assertGreater(int(np.count_nonzero(chi_bins == 0)), 0)
        self.assertGreater(int(np.count_nonzero(chi_bins == 1)), 0)

    def test_phase4d_hopf_fib_assignment_uses_fibonacci_rungs(self):
        z = np.array(
            [
                [4.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 3.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.5, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector, kchi, k1, k2, fib_total = hr.assign_sectors_phase4d_hopf_fib(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
        )
        fib_vals = set(hr.fibonacci_values_upto(25))
        self.assertTrue(np.all(sector >= 0))
        self.assertTrue(np.all(sector < 25))
        self.assertTrue(all(int(v) in fib_vals for v in fib_total))
        self.assertTrue(all(int(v) in fib_vals for v in kchi))
        self.assertTrue(all(int(v) in fib_vals for v in k1))
        self.assertTrue(all(int(v) in fib_vals for v in k2))

    def test_phase4d_hopf_fib_rung_forces_nontrivial_chi_axis(self):
        kchi, k1, k2, forced_total = hr.allocate_phi2_rung_bins(
            total_cap=np.array([13, 13, 13], dtype=np.int64),
            min_pair_bins=3,
            chi_u=np.array([0.35, 0.50, 0.65], dtype=np.float64),
            rho1=np.array([2.0, 1.0, 0.8], dtype=np.float64),
            rho2=np.array([1.0, 1.0, 2.0], dtype=np.float64),
            div_score=np.array([0.8, 0.8, 0.8], dtype=np.float64),
            max_total=25,
        )
        fib_vals = set(hr.fibonacci_values_upto(25))
        self.assertTrue(all(int(v) in fib_vals for v in kchi))
        self.assertTrue(all(int(v) in fib_vals for v in k1))
        self.assertTrue(all(int(v) in fib_vals for v in k2))
        self.assertTrue(np.all(kchi >= 2))
        self.assertTrue(np.all(forced_total > 9))
        self.assertTrue(np.all(forced_total <= 25))

    def test_phase4d_hopf_fib_rung_gate_can_fall_back_to_pure_hopf(self):
        z = np.array(
            [
                [4.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 3.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.5, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        _sector_hopf, hopf_k1, hopf_k2 = hr.assign_sectors_phase4d_hopf(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
        )
        _sector, kchi, k1, k2, _fib_total, forced_total = hr.assign_sectors_phase4d_hopf_fib_rung(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            fib_rung_gate_threshold=10.0,
        )
        np.testing.assert_array_equal(kchi, np.ones_like(kchi))
        np.testing.assert_array_equal(k1, hopf_k1)
        np.testing.assert_array_equal(k2, hopf_k2)
        np.testing.assert_array_equal(forced_total, hopf_k1 * hopf_k2)

    def test_phase4d_hopf_fib_band_uses_small_shared_state_set(self):
        z = np.array(
            [
                [4.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [2.5, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 3.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.5, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector, kchi, k1, k2, _fib_total, state_total, band = hr.assign_sectors_phase4d_hopf_fib_band(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
        )
        states = np.stack([kchi, np.maximum(k1, k2), np.minimum(k1, k2)], axis=1)
        unique_states = np.unique(states, axis=0)
        self.assertTrue(np.all(sector >= 0))
        self.assertTrue(np.all(sector < 25))
        self.assertLessEqual(unique_states.shape[0], 3)
        self.assertGreaterEqual(np.max(state_total), np.min(state_total))
        self.assertGreaterEqual(len(np.unique(band)), 2)

    def test_phase4d_hopf_blend_lambda_zero_reduces_to_pure_hopf(self):
        z = np.array(
            [
                [4.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [3.5, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0, 0.0, 3.5, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.5, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        _sector_hopf, hopf_k1, hopf_k2 = hr.assign_sectors_phase4d_hopf(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
        )
        _sector, kchi, k1, k2, blend_total, _blend_score, _chi_pressure, _shell_pressure = hr.assign_sectors_phase4d_hopf_blend(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=2,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            shell_mode="linear",
            shell_phase_coupling=0.0,
            hopf_chi_bins=2,
            hopf_blend_lambda=0.0,
            hopf_blend_chi_weight=1.0,
            hopf_blend_shell_weight=0.5,
        )
        np.testing.assert_array_equal(kchi, np.ones_like(kchi))
        np.testing.assert_array_equal(k1, hopf_k1)
        np.testing.assert_array_equal(k2, hopf_k2)
        np.testing.assert_array_equal(blend_total, hopf_k1 * hopf_k2)

    def test_phase4d_hopf_blend_can_open_chi_axis_under_pressure(self):
        z = np.array(
            [
                [8.0, 0.0, 0.20, 0.0, 0.0, 0.0, 0.0, 0.0],
                [7.0, 0.0, 0.20, 0.0, 0.0, 0.0, 0.0, 0.0],
                [6.0, 0.0, 0.20, 0.0, 0.0, 0.0, 0.0, 0.0],
                [5.0, 0.0, 0.20, 0.0, 0.0, 0.0, 0.0, 0.0],
                [4.0, 0.0, 0.20, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.20, 0.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            ],
            dtype=np.float64,
        )
        sector, kchi, k1, k2, blend_total, blend_score, chi_pressure, shell_pressure = hr.assign_sectors_phase4d_hopf_blend(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            shell_mode="linear",
            shell_phase_coupling=0.0,
            hopf_chi_bins=2,
            hopf_blend_lambda=1.1,
            hopf_blend_chi_weight=1.5,
            hopf_blend_shell_weight=0.5,
        )
        comp = hr.phase4d_adaptive_components(
            z=z,
            K=25,
            dim_i=0,
            dim_j=1,
            dim_k=2,
            dim_l=3,
            delta_r=3.6,
            tau=1.0,
            adaptive_min_pair_bins=3,
            adaptive_time_growth=1.4,
            adaptive_balance=1.2,
            adaptive_angle_growth=0.5,
            adaptive_shell_growth=0.0,
            adaptive_shell_balance=0.0,
            adaptive_converge_lambda=0.0,
            adaptive_converge_target=1.0,
            adaptive_converge_hysteresis=0.1,
            adaptive_converge_mode="fixed",
        )
        self.assertTrue(np.all(sector >= 0))
        self.assertTrue(np.all(sector < 25))
        self.assertGreater(int(np.max(kchi)), 1)
        self.assertTrue(np.all(blend_total >= comp["hopf_k1"] * comp["hopf_k2"]))
        self.assertGreater(float(np.max(blend_score)), 0.0)
        self.assertGreater(float(np.max(chi_pressure)), 0.0)
        self.assertGreaterEqual(float(np.max(shell_pressure)), 0.0)


if __name__ == "__main__":
    unittest.main()
