from __future__ import annotations

import math
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_clock import development as clock
from r4_softmax_trainer.zoology_control.model import (
    ZoologyFigure2Config,
    ZoologyFigure2Model,
)


def tensors() -> dict[str, torch.Tensor]:
    inputs = torch.tensor([[2, 20, 4, 21, 2, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0]]).repeat(
        8, 1
    )
    inputs[:, 8] = torch.arange(8)
    positions = torch.tensor([[4, 6]]).repeat(8, 1)
    targets = torch.tensor([[20, 21]]).repeat(8, 1)
    return {
        f"{split}_{name}": value.clone()
        for split in ("train", "test")
        for name, value in (
            ("inputs", inputs),
            ("positions", positions),
            ("targets", targets),
        )
    }


def model() -> ZoologyFigure2Model:
    return ZoologyFigure2Model(
        ZoologyFigure2Config(
            vocab_size=32,
            d_model=8,
            n_layers=2,
            num_heads=1,
            max_position_embeddings=16,
            attention_dropout=0.1,
            embed_dropout=0.1,
        )
    )


class ClockDevelopmentTests(unittest.TestCase):
    def test_cached_primary_configures_cpu_before_control_recovery(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            record = clock.release._with_cid({"binding_cid": "binding"}, "primary_cid")
            clock.release._write_exclusive_json(root / "primary/result.json", record)
            with patch.object(clock.release, "_configure_cpu") as configure:
                self.assertEqual(
                    clock._primary(root, {}, threads=4, binding_cid="binding"), record
                )
            configure.assert_called_once_with(4)

    def small(self) -> ExitStack:
        stack = ExitStack()
        for name, value in (
            ("UPDATES_PER_BLOCK", 3),
            ("MAXIMUM_BLOCKS", 2),
            ("MAXIMUM_UPDATES", 6),
            ("CHECKPOINT_INTERVAL", 2),
            ("BATCH_SIZE", 2),
            ("QUERIES", 2),
            ("DEVELOPMENT_ROWS", 8),
        ):
            stack.enter_context(patch.object(clock, name, value))
        stack.enter_context(
            patch.object(clock.previous, "_new_model", side_effect=model)
        )
        return stack

    def test_interrupted_mid_block_matches_uninterrupted_model_and_rng(self) -> None:
        with self.small(), tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            full = clock._primary(
                root / "full", tensors(), threads=4, binding_cid="binding"
            )
            original = clock.previous._save_checkpoint

            def crash(path, value):
                original(path, value)
                raise RuntimeError("interrupted after durable checkpoint")

            with (
                patch.object(clock.previous, "_save_checkpoint", side_effect=crash),
                self.assertRaisesRegex(RuntimeError, "durable checkpoint"),
            ):
                clock._primary(
                    root / "resumed", tensors(), threads=4, binding_cid="binding"
                )
            saved = torch.load(
                root / "resumed/primary/checkpoint.pt", weights_only=True
            )
            self.assertEqual(saved["completed_updates"], 2)
            self.assertEqual(saved["history"], [])
            recovered = clock._primary(
                root / "resumed", tensors(), threads=4, binding_cid="binding"
            )
            self.assertEqual(full["artifact"], recovered["artifact"])
            self.assertEqual(full["evaluation_rng"], recovered["evaluation_rng"])
            self.assertEqual(full["final_development"], recovered["final_development"])
            self.assertEqual(full["work"], recovered["work"])
            self.assertEqual(full["completed_updates"], 6)
            self.assertEqual(full["history"][0]["learning_rate"], clock.LEARNING_RATE)
            expected = clock.LEARNING_RATE * (1 + math.cos(math.pi / 64)) / 2
            self.assertAlmostEqual(
                full["history"][1]["learning_rate"], expected, places=16
            )
            self.assertEqual(full["history"][0]["train"]["updates"], 3)

    def test_passing_checkpoint_finalizes_without_training_or_evaluation(self) -> None:
        score = {
            "decisions": 16,
            "top1_correct": 16,
            "top1_rate": 1.0,
            "nll_nats": 0.01,
            "selected_logits_cid": "fixture",
        }
        with self.small(), tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                patch.object(clock.previous, "_score", return_value=score),
                patch.object(
                    clock.previous,
                    "_write_or_match",
                    side_effect=RuntimeError("finalization crash"),
                ),
                self.assertRaisesRegex(RuntimeError, "finalization crash"),
            ):
                clock._primary(root, tensors(), threads=4, binding_cid="binding")
            with (
                patch.object(clock, "_step", side_effect=AssertionError("extra step")),
                patch.object(
                    clock.previous,
                    "_score",
                    side_effect=AssertionError("extra evaluation"),
                ),
            ):
                result = clock._primary(
                    root, tensors(), threads=4, binding_cid="binding"
                )
            self.assertTrue(result["passed"])
            self.assertEqual(result["completed_updates"], 3)
            saved = torch.load(root / "primary/checkpoint.pt", weights_only=True)
            self.assertEqual(saved["scheduler"]["last_epoch"], 0)

    def test_negative_primary_never_reads_binding_control(self) -> None:
        preparation = {
            "preparation_cid": "prep",
            "implementation": {},
            "reused_c0": {},
            "dataset": {},
            "read_ledger": {},
        }
        records = [
            {
                "plan": {"threads": threads},
                "stable": True,
                "repeat_deterministic": True,
                "projected_primary_seconds": cost,
                "peak_rss_bytes": 1000,
            }
            for threads, cost in ((1, 2000), (4, 1000), (8, 1100))
        ]
        admitted = clock.release._with_cid(
            {
                "preparation_cid": "prep",
                "implementation": {},
                "reused_c0": {},
                "plans": records,
                "selected": clock._select(records),
                "passed": True,
            },
            "preflight_cid",
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            clock.release._write_exclusive_json(root / clock.PREFLIGHT_PATH, admitted)
            with (
                patch.object(
                    clock.contract, "validate_preparation", return_value=preparation
                ),
                patch.object(clock.contract, "load_dataset", return_value=tensors()),
                patch.object(
                    clock,
                    "_primary",
                    return_value={
                        "passed": False,
                        "status": "CLOCK_MATCHED_TRANSFER_MISS",
                    },
                ),
                patch.object(
                    clock.contract,
                    "load_control",
                    side_effect=AssertionError("control opened"),
                ) as control,
            ):
                result = clock.run(root)
            control.assert_not_called()
            self.assertEqual(
                result["decision"]["verdict"], "CLOCK_MATCHED_TRANSFER_MISS"
            )
            self.assertEqual(result["read_ledger"]["control_query_decisions"], 0)

    def test_history_rejects_wrong_clock_and_post_pass_updates(self) -> None:
        row = {
            "block": 1,
            "completed_updates": 196,
            "development": {"top1_rate": 1.0},
            "strict_source_pass": True,
        }
        self.assertTrue(clock._history_pass([row]))
        with self.assertRaisesRegex(ValueError, "history differs"):
            clock._history_pass([{**row, "completed_updates": 195}])
        with self.assertRaisesRegex(ValueError, "after a passing"):
            clock._history_pass([row, {**row, "block": 2, "completed_updates": 392}])


if __name__ == "__main__":
    unittest.main()
