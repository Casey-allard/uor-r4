import sys
import unittest

from tasks import router_proxy_eval as proxy


def _proxy_args(argv):
    old_argv = sys.argv[:]
    try:
        sys.argv = argv
        return proxy.parse_args()
    finally:
        sys.argv = old_argv


class RouterProxyEvalEventGateTest(unittest.TestCase):
    def test_parse_args_exposes_sparse_event_surface(self):
        args = _proxy_args(
            [
                "router_proxy_eval.py",
                "--event_gate_mode",
                "soft_error",
                "--event_gate_threshold",
                "0.05",
                "--event_gate_tau",
                "0.02",
            ]
        )

        self.assertEqual(args.event_gate_mode, "soft_error")
        self.assertAlmostEqual(args.event_gate_threshold, 0.05)
        self.assertAlmostEqual(args.event_gate_tau, 0.02)

    def test_parse_args_accepts_hard_error_mode(self):
        args = _proxy_args(
            [
                "router_proxy_eval.py",
                "--event_gate_mode",
                "hard_error",
                "--event_gate_threshold",
                "0.07",
            ]
        )

        self.assertEqual(args.event_gate_mode, "hard_error")
        self.assertAlmostEqual(args.event_gate_threshold, 0.07)

    def test_event_gate_is_unity_when_disabled(self):
        gate = proxy.event_gate_value_from_error(
            error_mag=0.001,
            mode="off",
            threshold=0.05,
            tau=0.02,
        )
        self.assertAlmostEqual(gate, 1.0)

    def test_hard_error_gate_is_threshold_step(self):
        low = proxy.event_gate_value_from_error(
            error_mag=0.049,
            mode="hard_error",
            threshold=0.05,
            tau=0.01,
        )
        hit = proxy.event_gate_value_from_error(
            error_mag=0.05,
            mode="hard_error",
            threshold=0.05,
            tau=0.01,
        )
        high = proxy.event_gate_value_from_error(
            error_mag=0.08,
            mode="hard_error",
            threshold=0.05,
            tau=0.01,
        )

        self.assertEqual(low, 0.0)
        self.assertEqual(hit, 1.0)
        self.assertEqual(high, 1.0)

    def test_soft_error_gate_is_monotone_and_bounded(self):
        low = proxy.event_gate_value_from_error(
            error_mag=0.01,
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )
        mid = proxy.event_gate_value_from_error(
            error_mag=0.05,
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )
        high = proxy.event_gate_value_from_error(
            error_mag=0.10,
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )

        self.assertGreaterEqual(low, 0.0)
        self.assertLessEqual(high, 1.0)
        self.assertLess(low, mid)
        self.assertLess(mid, high)
        self.assertAlmostEqual(mid, 0.5, places=6)

    def test_event_gate_stats_report_error_and_activation(self):
        error_mag, gate, active = proxy.event_gate_stats(
            yhat=[0.0, 0.0],
            y=[0.2, 0.0],
            mode="soft_error",
            threshold=0.05,
            tau=0.01,
        )

        self.assertGreater(error_mag, 0.0)
        self.assertGreater(gate, 0.5)
        self.assertEqual(active, 1.0)

    def test_hard_error_gate_stats_report_binary_activation(self):
        low_error_mag, low_gate, low_active = proxy.event_gate_stats(
            yhat=[0.0, 0.0],
            y=[0.05, 0.0],
            mode="hard_error",
            threshold=0.05,
            tau=0.01,
        )
        high_error_mag, high_gate, high_active = proxy.event_gate_stats(
            yhat=[0.0, 0.0],
            y=[0.2, 0.0],
            mode="hard_error",
            threshold=0.05,
            tau=0.01,
        )

        self.assertLess(low_error_mag, 0.05)
        self.assertEqual(low_gate, 0.0)
        self.assertEqual(low_active, 0.0)
        self.assertGreater(high_error_mag, 0.05)
        self.assertEqual(high_gate, 1.0)
        self.assertEqual(high_active, 1.0)


if __name__ == "__main__":
    unittest.main()
