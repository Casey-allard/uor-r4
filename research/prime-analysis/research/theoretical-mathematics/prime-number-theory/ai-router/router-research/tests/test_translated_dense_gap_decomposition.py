import json
import tempfile
import unittest
from pathlib import Path

import numpy as np

from tools import translated_dense_gap_decomposition as audit


class TranslatedDenseGapDecompositionTest(unittest.TestCase):
    def _write_audit(self, path: Path, *, correct, target_present, topk_target_present):
        n = len(correct)
        payload = {
            "eval_local_idx": np.arange(n, dtype=np.int64),
            "eval_source_idx": np.arange(n, dtype=np.int64),
            "target_tok": np.full((n,), 20, dtype=np.int32),
            "pred_tok": np.where(np.array(correct, dtype=np.int8) == 1, 20, 10).astype(np.int32),
            "correct": np.array(correct, dtype=np.int8),
            "candidate_count": np.full((n,), 8, dtype=np.int32),
            "candidate_fraction": np.full((n,), 0.2, dtype=np.float64),
            "target_present": np.array(target_present, dtype=np.int8),
            "best_target_rank": np.where(np.array(target_present, dtype=np.int8) == 1, 2, 0).astype(np.int32),
            "topk_target_present": np.array(topk_target_present, dtype=np.int8),
        }
        np.savez_compressed(path, **payload)

    def _analysis(self, baseline_path: str, candidate_path: str):
        return {
            "experiment_id": "inc0113_test",
            "config": "test.json",
            "route_stats": [],
            "results": {
                "DENSE": [
                    {
                        "args": {"seed": 0},
                        "artifacts": {"retrieval_query_audit_file": baseline_path},
                    }
                ],
                "CAND": [
                    {
                        "args": {"seed": 0},
                        "artifacts": {"retrieval_query_audit_file": candidate_path},
                    }
                ],
            },
        }

    def test_build_payload_marks_omission_led_gap_as_candidate_recovery(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            baseline_path = Path(tmpdir) / "baseline.npz"
            candidate_path = Path(tmpdir) / "candidate.npz"
            self._write_audit(
                baseline_path,
                correct=[1, 1, 1, 0],
                target_present=[1, 1, 1, 1],
                topk_target_present=[1, 1, 1, 0],
            )
            self._write_audit(
                candidate_path,
                correct=[0, 0, 1, 0],
                target_present=[0, 0, 1, 1],
                topk_target_present=[0, 0, 1, 0],
            )
            payload = audit.build_payload(
                [self._analysis(str(baseline_path), str(candidate_path))],
                [{"label": "dense_vs_cand", "baseline_route_id": "DENSE", "candidate_route_id": "CAND"}],
                negligible_gap_threshold=0.001,
            )

        comparison = payload["comparisons"][0]
        self.assertEqual(comparison["next_branch_bias"], "candidate_recovery")
        self.assertAlmostEqual(comparison["mean_omission_rate_within_gap"], 1.0)
        self.assertAlmostEqual(comparison["mean_present_but_not_top1_rate_within_gap"], 0.0)

    def test_build_payload_marks_tiny_gap_as_operationally_negligible(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            baseline_path = Path(tmpdir) / "baseline.npz"
            candidate_path = Path(tmpdir) / "candidate.npz"
            self._write_audit(
                baseline_path,
                correct=[1, 0, 0, 0],
                target_present=[1, 1, 1, 1],
                topk_target_present=[1, 0, 0, 0],
            )
            self._write_audit(
                candidate_path,
                correct=[0, 0, 0, 0],
                target_present=[1, 1, 1, 1],
                topk_target_present=[0, 0, 0, 0],
            )
            payload = audit.build_payload(
                [self._analysis(str(baseline_path), str(candidate_path))],
                [{"label": "dense_vs_cand", "baseline_route_id": "DENSE", "candidate_route_id": "CAND"}],
                negligible_gap_threshold=0.30,
            )

        comparison = payload["comparisons"][0]
        self.assertEqual(comparison["next_branch_bias"], "operationally_negligible")
        self.assertLessEqual(comparison["mean_net_dense_advantage_rate"], 0.30)


if __name__ == "__main__":
    unittest.main()
