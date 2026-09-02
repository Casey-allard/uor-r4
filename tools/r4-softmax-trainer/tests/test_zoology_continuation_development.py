from __future__ import annotations

import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_continuation import development as continuation
from test_zoology_clock_development import model, tensors

clock = continuation.clock
TINY = continuation._Limits(
    inherited_blocks=1,
    maximum_blocks=3,
    updates_per_block=3,
    batch_size=2,
    queries=2,
    train_rows=8,
    development_rows=8,
    checkpoint_interval=2,
)


class ContinuationDevelopmentTests(unittest.TestCase):
    def fixture(self, root: Path):
        """Only tiny synthetic parents; never read the fitted research artifacts."""
        stack = ExitStack()
        self.addCleanup(stack.close)
        for name, value in (
            ("UPDATES_PER_BLOCK", 3),
            ("MAXIMUM_BLOCKS", 1),
            ("MAXIMUM_UPDATES", 3),
            ("CHECKPOINT_INTERVAL", 2),
            ("BATCH_SIZE", 2),
            ("QUERIES", 2),
            ("DEVELOPMENT_ROWS", 8),
        ):
            stack.enter_context(patch.object(clock, name, value))
        stack.enter_context(
            patch.object(clock.previous, "_new_model", side_effect=model)
        )
        parent_root = root / "parent"
        parent = clock._primary(parent_root, tensors(), threads=4, binding_cid="parent")
        preparation = {
            "preparation_cid": "fixture-preparation",
            "parent_primary": parent,
            "parent_checkpoint": {"cid": "fixture-checkpoint"},
            "parent_binding_cid": "parent",
            "parent_history_cid": "fixture-history",
            "cpu_plan": {"threads": 4},
            "control": {},
        }

        def load(_):
            return torch.load(parent_root / "primary/checkpoint.pt", weights_only=True)

        stack.enter_context(
            patch.object(continuation.contract, "load_checkpoint", side_effect=load)
        )
        return preparation, parent_root

    def test_split_and_midblock_restart_match_uninterrupted_trajectory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation, _ = self.fixture(root)
            with (
                patch.object(clock, "MAXIMUM_UPDATES", 9),
                patch.object(clock, "MAXIMUM_BLOCKS", 3),
            ):
                whole = clock._primary(
                    root / "whole", tensors(), threads=4, binding_cid="whole"
                )
            save = continuation.previous._save_checkpoint

            def interrupt(path, state):
                save(path, state)
                raise RuntimeError("durable interruption")

            with (
                patch.object(
                    continuation.previous, "_save_checkpoint", side_effect=interrupt
                ),
                self.assertRaisesRegex(RuntimeError, "durable interruption"),
            ):
                continuation._primary(
                    root / "split", tensors(), preparation, limits=TINY
                )
            partial = torch.load(
                root / "split/primary/checkpoint.pt", weights_only=True
            )
            self.assertEqual(partial["completed_updates"], 4)
            self.assertEqual(partial["accumulator"]["updates"], 1)
            resumed = continuation._primary(
                root / "split", tensors(), preparation, limits=TINY
            )
            self.assertEqual(resumed["artifact"], whole["artifact"])
            self.assertEqual(resumed["evaluation_rng"], whole["evaluation_rng"])
            self.assertEqual(resumed["final_development"], whole["final_development"])
            self.assertEqual(
                resumed["history"][:1], preparation["parent_primary"]["history"]
            )
            self.assertEqual(resumed["work"]["inherited"]["optimizer_updates"], 3)
            self.assertEqual(resumed["work"]["additional"]["optimizer_updates"], 6)
            self.assertEqual(
                resumed["work"]["additional"]["train_query_presentations"], 24
            )
            self.assertEqual(
                resumed["work"]["total"]["development_query_presentations"], 48
            )
            split_state = torch.load(
                root / "split/primary/checkpoint.pt", weights_only=True
            )
            whole_state = torch.load(
                root / "whole/primary/checkpoint.pt", weights_only=True
            )
            for name in ("torch_rng_state", "evaluation_rng"):
                self.assertTrue(torch.equal(split_state[name], whole_state[name]))
            self.assertEqual(split_state["scheduler"], whole_state["scheduler"])
            for key, state in split_state["optimizer"]["state"].items():
                for name, value in state.items():
                    self.assertTrue(
                        torch.equal(value, whole_state["optimizer"]["state"][key][name])
                    )
            self.assertEqual(
                split_state["sampler"]["cursor"], whole_state["sampler"]["cursor"]
            )
            self.assertTrue(
                torch.equal(
                    split_state["sampler"]["permutation"],
                    whole_state["sampler"]["permutation"],
                )
            )
            for row, original in zip(resumed["history"][1:], whole["history"][1:]):
                for name in ("learning_rate", "train", "development"):
                    self.assertEqual(row[name], original[name])

    def test_inherited_time_is_free_but_additional_time_survives_restart(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation, parent_root = self.fixture(root)
            parent = torch.load(
                parent_root / "primary/checkpoint.pt", weights_only=True
            )
            parent["elapsed_seconds"] = 9000.0
            now = [0.0]
            original_step, original_save = (
                clock._step,
                continuation.previous._save_checkpoint,
            )

            def step(*args):
                measured = original_step(*args)
                now[0] += 1000.0
                return measured

            def interrupt(path, state):
                original_save(path, state)
                raise RuntimeError("durable interruption")

            with (
                patch.object(
                    continuation.contract, "load_checkpoint", return_value=parent
                ),
                patch.object(
                    continuation.time, "monotonic", side_effect=lambda: now[0]
                ),
                patch.object(clock, "_step", side_effect=step),
                patch.object(
                    continuation.previous, "_save_checkpoint", side_effect=interrupt
                ),
                self.assertRaisesRegex(RuntimeError, "durable interruption"),
            ):
                continuation._primary(
                    root / "added", tensors(), preparation, limits=TINY
                )
            saved = torch.load(root / "added/primary/checkpoint.pt", weights_only=True)
            self.assertEqual(saved["inherited_elapsed_seconds"], 9000)
            self.assertEqual(saved["additional_elapsed_seconds"], 1000)
            # A new monotonic origin must not erase the 1000 seconds already spent.
            ticks = iter((100000.0, 100901.0, 100902.0, 100903.0))
            with (
                patch.object(
                    continuation.contract, "load_checkpoint", return_value=parent
                ),
                patch.object(
                    continuation.time, "monotonic", side_effect=lambda: next(ticks)
                ),
                patch.object(
                    clock, "_step", side_effect=AssertionError("budget reset")
                ),
            ):
                result = continuation._primary(
                    root / "added", tensors(), preparation, limits=TINY
                )
            self.assertEqual(result["status"], "INCOMPLETE")
            self.assertEqual(result["completed_updates"], 4)
            self.assertEqual(result["additional_elapsed_seconds"], 1903)
            self.assertEqual(result["elapsed_seconds"], 10903)
            self.assertIsNone(result["final_development"])

    def test_passing_checkpoint_finalizes_without_another_step_or_evaluation(
        self,
    ) -> None:
        score = {
            "decisions": 16,
            "top1_correct": 16,
            "top1_rate": 1.0,
            "nll_nats": 0.01,
            "selected_logits_cid": "fixture",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            preparation, _ = self.fixture(root)
            with (
                patch.object(continuation.previous, "_score", return_value=score),
                patch.object(
                    continuation.previous,
                    "_write_or_match",
                    side_effect=RuntimeError("finalization interruption"),
                ),
                self.assertRaisesRegex(RuntimeError, "finalization interruption"),
            ):
                continuation._primary(
                    root / "passing", tensors(), preparation, limits=TINY
                )
            with (
                patch.object(
                    clock, "_step", side_effect=AssertionError("post-pass step")
                ),
                patch.object(
                    continuation.previous,
                    "_score",
                    side_effect=AssertionError("post-pass evaluation"),
                ),
            ):
                result = continuation._primary(
                    root / "passing", tensors(), preparation, limits=TINY
                )
            self.assertTrue(result["passed"])
            self.assertEqual(result["completed_updates"], 6)
            state = torch.load(
                root / "passing/primary/checkpoint.pt", weights_only=True
            )
            self.assertEqual(state["scheduler"]["last_epoch"], 1)
            with patch.object(continuation.release, "_configure_cpu") as configure:
                self.assertEqual(
                    continuation._primary(
                        root / "passing", {}, preparation, limits=TINY
                    ),
                    result,
                )
            configure.assert_called_once_with(4)

    def test_miss_never_opens_control_and_history_cannot_change(self) -> None:
        with patch.object(
            continuation.contract,
            "load_control",
            side_effect=AssertionError("control opened"),
        ) as control:
            self.assertEqual(
                continuation._control(Path("unused"), {"passed": False}, {}),
                {"status": "NOT_RUN_PRIMARY_MISS"},
            )
        control.assert_not_called()
        self.assertFalse(continuation.release._source_pass(0.99))
        row = {
            "block": 1,
            "completed_updates": 3,
            "strict_source_pass": False,
            "development": {"top1_rate": 0.5, "decisions": 16},
        }
        prep = {"parent_primary": {"history": [row]}}
        self.assertFalse(continuation._history_pass([row], prep, TINY))
        with self.assertRaisesRegex(ValueError, "inherited history"):
            continuation._history_pass([{**row, "completed_updates": 2}], prep, TINY)
        passed = {
            **row,
            "block": 2,
            "completed_updates": 6,
            "strict_source_pass": True,
            "development": {"top1_rate": 1.0, "decisions": 16},
        }
        with self.assertRaisesRegex(ValueError, "after a passing"):
            continuation._history_pass(
                [row, passed, {**passed, "block": 3, "completed_updates": 9}],
                prep,
                TINY,
            )


if __name__ == "__main__":
    unittest.main()
