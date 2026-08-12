import subprocess
import sys
import unittest


class TestCLIContract(unittest.TestCase):
    def test_required_flags_present(self):
        cmd = [sys.executable, "hyperbolic_router_so8.py", "--help"]
        p = subprocess.run(cmd, capture_output=True, text=True, check=True)
        out = p.stdout

        required = [
            "--sector_mode",
            "--phase_dims",
            "--phase4_dims",
            "--field4_dims",
            "--complex_dims",
            "--hybrid_local_k",
            "--hybrid_complex_roots",
            "--hybrid_local_min_k",
            "--hybrid_local_target",
            "--hybrid_local_hysteresis",
            "--hybrid_local_converge_lambda",
            "--adaptive_min_pair_bins",
            "--adaptive_time_growth",
            "--adaptive_balance",
            "--adaptive_angle_growth",
            "--adaptive_shell_growth",
            "--adaptive_shell_balance",
            "--adaptive_converge_lambda",
            "--adaptive_converge_target",
            "--adaptive_converge_hysteresis",
            "--adaptive_converge_mode",
            "--fib_rung_gate_threshold",
            "--route_scale_lambda",
            "--memory_coord_mode",
            "--shell_mode",
            "--shell_phase_coupling",
            "--product_shell_control_mode",
            "--product_shell_gate_threshold",
            "--hopf_blend_lambda",
            "--hopf_blend_chi_weight",
            "--hopf_blend_shell_weight",
            "--phase_transport_lambda",
            "--phase_field_lambda",
            "--time_pressure_lambda",
            "--train_route_mode",
            "--recluster_after_chart",
            "--fast_dev",
            "--early_stop_patience",
            "--early_stop_min_delta",
            "--cache_dir",
            "--cache_chart",
            "--cache_routes",
            "--run_tag",
        ]
        for flag in required:
            self.assertIn(flag, out)

        self.assertIn("{kmeans,phase2,phase4d,phase4d_adaptive,phase4d_hopf,phase4d_hopf_base,phase4d_hopf_transport,phase4d_hopf_transport_complex,phase4d_hopf_product_phase,phase4d_hopf_iso,phase4d_hopf_ball,phase4d_hopf_base_ball,phase4d_hopf_chi,phase4d_hopf_fib,phase4d_hopf_fib_rung,phase4d_hopf_fib_band,phase4d_hopf_fib_band_iso,phase4d_hopf_fib_band_bound,phase4d_hopf_blend,phase4d_complex_local,complex2}", out)
        self.assertIn("{fixed,phi_ratio,phi_ladder}", out)
        self.assertIn("{dynamic,final_static}", out)
        self.assertIn("{route_chart,full_chart}", out)
        self.assertIn("{linear,phi_log,phi_phase,h4_mass,h4_mass_phi}", out)
        self.assertIn("{continuous,gated,banded}", out)

    def test_retrieval_help_exposes_phase_field_routes(self):
        cmd = [sys.executable, "tasks/router_retrieval_eval.py", "--help"]
        p = subprocess.run(cmd, capture_output=True, text=True, check=True)
        out = p.stdout

        for flag in ["--field4_dims", "--phase_transport_lambda", "--phase_field_lambda", "--product_shell_control_mode", "--product_shell_gate_threshold", "--event_gate_mode", "--event_gate_threshold", "--event_gate_tau", "--retrieval_audit_dir"]:
            self.assertIn(flag, out)
        self.assertIn("--complex_rerank_margin_threshold", out)
        self.assertIn("phase4d_hopf_base", out)
        self.assertIn("phase4d_hopf_base_ball", out)
        self.assertIn("phase4d_hopf_transport", out)
        self.assertIn("phase4d_hopf_transport_complex", out)
        self.assertIn("phase4d_hopf_product_phase", out)
        self.assertIn("complex_plane_low_margin", out)
        self.assertIn("{off,soft_error}", out)

    def test_proxy_help_exposes_sparse_event_surface(self):
        cmd = [sys.executable, "tasks/router_proxy_eval.py", "--help"]
        p = subprocess.run(cmd, capture_output=True, text=True, check=True)
        out = p.stdout

        for flag in ["--event_gate_mode", "--event_gate_threshold", "--event_gate_tau"]:
            self.assertIn(flag, out)
        self.assertIn("{off,soft_error,hard_error}", out)
        self.assertIn("phase4d_hopf_base_ball", out)


if __name__ == "__main__":
    unittest.main()
