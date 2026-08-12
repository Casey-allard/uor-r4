import json
import os
import subprocess
import tempfile
import unittest

import numpy as np

from tasks.dynamic_h4_state_eval import (
    augment_route_keys_with_complex,
    build_flow_state,
    knn_state_predict_bucketed,
    pairwise_poincare_distance,
    pointwise_state_distance,
)


class DynamicH4StateEvalTest(unittest.TestCase):
    def test_pairwise_poincare_distance_matches_pointwise_diagonal(self):
        x = np.array([[0.05, 0.0], [0.1, 0.02], [0.0, 0.08]], dtype=np.float64)
        dmat = pairwise_poincare_distance(x, x)
        ddiag = np.diag(dmat)
        self.assertTrue(np.all(ddiag >= 0.0))
        self.assertTrue(np.allclose(ddiag, 0.0, atol=1e-9))

    def test_build_flow_state_scales_by_q95(self):
        z = np.array(
            [
                [0.0, 0.0],
                [1.0, 0.0],
                [2.0, 0.0],
                [3.0, 0.0],
            ],
            dtype=np.float64,
        )
        idx = np.arange(z.shape[0], dtype=np.int64)
        flow, flow_ball, q95 = build_flow_state(z, idx, flow_step=1, flow_scale=1.0)
        self.assertGreater(q95, 0.0)
        self.assertEqual(flow.shape, z.shape)
        self.assertEqual(flow_ball.shape, z.shape)
        self.assertAlmostEqual(float(flow[0, 0]), 0.0)

    def test_pointwise_state_distance_supports_all_modes(self):
        pos_a = np.array([[0.01, 0.0], [0.02, 0.01]], dtype=np.float64)
        pos_b = np.array([[0.02, 0.0], [0.01, 0.01]], dtype=np.float64)
        flow_a = np.array([[0.0, 0.0], [0.1, 0.0]], dtype=np.float64)
        flow_b = np.array([[0.0, 0.0], [0.0, 0.1]], dtype=np.float64)
        flow_ball_a = np.tanh(flow_a)
        flow_ball_b = np.tanh(flow_b)
        for mode in ("static_h4", "tangent_h4", "product_h4x_h4"):
            d = pointwise_state_distance(pos_a, pos_b, flow_a, flow_b, flow_ball_a, flow_ball_b, mode, 0.5)
            self.assertEqual(d.shape, (2,))
            self.assertTrue(np.all(d >= 0.0))

    def test_bucketed_knn_reduces_candidate_fraction(self):
        train_pos = np.array(
            [
                [0.01, 0.00],
                [0.02, 0.00],
                [0.40, 0.00],
                [0.41, 0.00],
            ],
            dtype=np.float64,
        )
        eval_pos = np.array(
            [
                [0.015, 0.00],
                [0.405, 0.00],
            ],
            dtype=np.float64,
        )
        train_flow = np.zeros_like(train_pos)
        eval_flow = np.zeros_like(eval_pos)
        train_flow_ball = np.zeros_like(train_pos)
        eval_flow_ball = np.zeros_like(eval_pos)
        train_y = np.eye(4, dtype=np.float64)
        train_keys = [(0, 0), (0, 0), (0, 1), (0, 1)]
        eval_keys = [(0, 0), (0, 1)]

        _, pred, _, cand_mean, cand_frac, probe_mean, fallback_rate = knn_state_predict_bucketed(
            train_pos=train_pos,
            train_flow=train_flow,
            train_flow_ball=train_flow_ball,
            train_y=train_y,
            train_keys=train_keys,
            eval_pos=eval_pos,
            eval_flow=eval_flow,
            eval_flow_ball=eval_flow_ball,
            eval_keys=eval_keys,
            dynamic_state_mode="static_h4",
            flow_weight=0.0,
            topk=1,
            block_size=8,
        )
        self.assertEqual(pred.shape, (2,))
        self.assertLess(cand_mean, float(train_pos.shape[0]))
        self.assertLess(cand_frac, 1.0)
        self.assertEqual(probe_mean, 1.0)
        self.assertEqual(fallback_rate, 0.0)

    def test_complex_route_key_augmentation_adds_secondary_component(self):
        base_keys = [(0, 0), (0, 0), (1, 2)]
        field = np.array(
            [
                [0.5, 0.0],
                [0.0, 0.5],
                [-0.5, 0.0],
            ],
            dtype=np.float64,
        )
        keys, n_secondary = augment_route_keys_with_complex(
            base_keys=base_keys,
            field=field,
            dim_i=0,
            dim_j=1,
            roots=4,
            radius_bins=1,
        )
        self.assertEqual(len(keys[0]), 3)
        self.assertGreaterEqual(n_secondary, 2)
        self.assertNotEqual(keys[0], keys[1])

    def test_script_emits_json_summary(self):
        rs = np.random.RandomState(0)
        x_train = rs.normal(size=(96, 16)).astype(np.float32)
        x_val = rs.normal(size=(64, 16)).astype(np.float32)
        x_test = rs.normal(size=(64, 16)).astype(np.float32)
        y_train = np.eye(8, dtype=np.float32)[rs.randint(0, 8, size=96)]
        y_val = np.eye(8, dtype=np.float32)[rs.randint(0, 8, size=64)]
        y_test = np.eye(8, dtype=np.float32)[rs.randint(0, 8, size=64)]

        with tempfile.TemporaryDirectory() as td:
            data_path = os.path.join(td, "proxy.npz")
            np.savez_compressed(
                data_path,
                x_train=x_train,
                y_train=y_train,
                x_val=x_val,
                y_val=y_val,
                x_test=x_test,
                y_test=y_test,
            )
            cmd = [
                "python",
                "/Users/adminamn/ai-router/router-research/tasks/dynamic_h4_state_eval.py",
                "--input",
                data_path,
                "--eval_split",
                "test",
                "--max_train",
                "64",
                "--max_eval",
                "32",
                "--learn_so8",
                "0",
                "--learn_scale",
                "0",
                "--chart_iters",
                "1",
                "--sector_mode",
                "phase4d_hopf",
                "--candidate_mode",
                "static_bucket_knn",
                "--route_key_mode",
                "hopf_plus_complex",
                "--complex_key_roots",
                "4",
                "--dynamic_state_mode",
                "product_h4x_h4",
            ]
            proc = subprocess.run(cmd, capture_output=True, text=True, check=True)
            summary_line = None
            for line in proc.stdout.splitlines():
                if line.startswith("__JSON_SUMMARY__"):
                    summary_line = line.split("__JSON_SUMMARY__", 1)[1].strip()
            self.assertIsNotNone(summary_line)
            payload = json.loads(summary_line)
            self.assertEqual(payload["task"], "dynamic_h4_state_eval")
            self.assertIn("test_mse_after", payload["metrics"])
            self.assertIn("dynamic_step_to_random_ratio", payload["metrics"])
            self.assertIn("retrieval_candidate_count_mean", payload["metrics"])
            self.assertIn("retrieval_candidate_fraction_mean", payload["metrics"])
            self.assertIn("retrieval_secondary_key_count", payload["metrics"])


if __name__ == "__main__":
    unittest.main()
