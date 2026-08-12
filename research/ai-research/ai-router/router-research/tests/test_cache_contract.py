import copy
import sys
import unittest

import hyperbolic_router_so8 as hr
from tasks import router_proxy_eval as proxy
from tasks import router_retrieval_eval as retrieval


def _router_args():
    old_argv = sys.argv[:]
    try:
        sys.argv = ["hyperbolic_router_so8.py"]
        return hr.parse_args()
    finally:
        sys.argv = old_argv


def _proxy_args():
    old_argv = sys.argv[:]
    try:
        sys.argv = ["router_proxy_eval.py"]
        return proxy.parse_args()
    finally:
        sys.argv = old_argv


def _retrieval_args():
    old_argv = sys.argv[:]
    try:
        sys.argv = ["router_retrieval_eval.py"]
        return retrieval.parse_args()
    finally:
        sys.argv = old_argv


class CacheContractTest(unittest.TestCase):
    def test_router_cache_payload_changes_when_product_phase_args_change(self):
        args = _router_args()
        base_chart = hr.chart_cache_payload(args)
        base_route = hr.route_cache_payload(args, chart_fp="chart", c_fp="centers")

        for key, value in (
            ("field4_dims", "0,1,2,3"),
            ("phase_transport_lambda", 0.25),
            ("phase_field_lambda", 0.75),
            ("product_shell_control_mode", "gated"),
            ("product_shell_gate_threshold", 0.35),
        ):
            mutated = copy.deepcopy(args)
            setattr(mutated, key, value)
            self.assertNotEqual(hr.stable_hash(base_chart), hr.stable_hash(hr.chart_cache_payload(mutated)))
            self.assertNotEqual(
                hr.stable_hash(base_route),
                hr.stable_hash(hr.route_cache_payload(mutated, chart_fp="chart", c_fp="centers")),
            )

    def test_proxy_cache_payload_changes_when_product_phase_args_change(self):
        args = _proxy_args()
        base_chart = proxy.proxy_chart_cache_payload(args, d=8, dy=4, n_train=32)
        base_route = proxy.proxy_route_cache_payload(args, chart_fp="chart", c_fp="centers", n_train=32, n_eval=16)

        for key, value in (
            ("field4_dims", "0,1,2,3"),
            ("phase_transport_lambda", 0.25),
            ("phase_field_lambda", 0.75),
            ("product_shell_control_mode", "gated"),
            ("product_shell_gate_threshold", 0.35),
        ):
            mutated = copy.deepcopy(args)
            setattr(mutated, key, value)
            self.assertNotEqual(hr.stable_hash(base_chart), hr.stable_hash(proxy.proxy_chart_cache_payload(mutated, d=8, dy=4, n_train=32)))
            self.assertNotEqual(
                hr.stable_hash(base_route),
                hr.stable_hash(
                    proxy.proxy_route_cache_payload(mutated, chart_fp="chart", c_fp="centers", n_train=32, n_eval=16)
                ),
            )

    def test_retrieval_cache_payload_changes_when_product_phase_args_change(self):
        args = _retrieval_args()
        base_chart = retrieval.retrieval_chart_cache_payload(args, d=8, dy=4, n_train=32)
        base_route = retrieval.retrieval_route_cache_payload(args, chart_fp="chart", n_train=32)

        for key, value in (
            ("field4_dims", "0,1,2,3"),
            ("phase_transport_lambda", 0.25),
            ("phase_field_lambda", 0.75),
            ("product_shell_control_mode", "gated"),
            ("product_shell_gate_threshold", 0.35),
        ):
            mutated = copy.deepcopy(args)
            setattr(mutated, key, value)
            self.assertNotEqual(
                hr.stable_hash(base_chart),
                hr.stable_hash(retrieval.retrieval_chart_cache_payload(mutated, d=8, dy=4, n_train=32)),
            )
            self.assertNotEqual(
                hr.stable_hash(base_route),
                hr.stable_hash(retrieval.retrieval_route_cache_payload(mutated, chart_fp="chart", n_train=32)),
            )

    def test_retrieval_train_route_cache_payload_ignores_query_repeats(self):
        args = _retrieval_args()
        base = hr.stable_hash(retrieval.retrieval_route_cache_payload(args, chart_fp="chart", n_train=32))

        mutated = copy.deepcopy(args)
        mutated.query_repeats = int(args.query_repeats) + 4

        self.assertEqual(
            base,
            hr.stable_hash(retrieval.retrieval_route_cache_payload(mutated, chart_fp="chart", n_train=32)),
        )


if __name__ == "__main__":
    unittest.main()
