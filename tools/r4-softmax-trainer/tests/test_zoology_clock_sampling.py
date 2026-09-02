from __future__ import annotations

import unittest
from collections.abc import Mapping
from typing import Any

import torch
from r4_softmax_trainer.zoology_clock.sampling import CyclingBatches
from torch import Tensor
from torch.utils.data import DataLoader, TensorDataset


def _tensors() -> dict[str, Tensor]:
    rows = torch.arange(64, dtype=torch.long).unsqueeze(1)
    return {
        "train_inputs": rows.repeat(1, 6),
        "train_positions": torch.tensor([[2, 4]]).repeat(64, 1),
        "train_targets": (rows + 100).repeat(1, 2),
    }


def _old_loader(tensors: Mapping[str, Tensor]) -> DataLoader[Any]:
    # Exact #1053 train-loader arguments, with only the test batch size reduced.
    return DataLoader(
        TensorDataset(
            tensors["train_inputs"],
            tensors["train_positions"],
            tensors["train_targets"],
        ),
        batch_size=4,
        shuffle=True,
        num_workers=0,
    )


def _consume_development() -> None:
    loader = DataLoader(
        TensorDataset(torch.arange(8)),
        batch_size=4,
        shuffle=True,
        num_workers=0,
    )
    list(loader)


def _consume_dropout(step: int) -> None:
    # Simulate interleaved training randomness without fitting a model.
    torch.rand(17 + step % 3)


def _trajectory(*, cycling: bool, steps: int, development: bool) -> list[Any]:
    torch.manual_seed(123)
    tensors = _tensors()
    batches = CyclingBatches(tensors, batch_size=4) if cycling else None
    old = _old_loader(tensors)
    iterator = None
    observed = []
    for step in range(steps):
        if batches is not None:
            batch = batches.next_batch()
        else:
            if iterator is None:
                iterator = iter(old)
            try:
                batch = next(iterator)
            except StopIteration:
                iterator = iter(old)
                batch = next(iterator)
        _consume_dropout(step)
        if development and (step + 1) % 196 == 0:
            _consume_development()
        observed.append(
            ([value.tolist() for value in batch], torch.get_rng_state().tolist())
        )
    return observed


class CyclingBatchesTests(unittest.TestCase):
    def test_exact_stock_order_and_rng_across_multiple_traversals(self) -> None:
        self.assertEqual(
            _trajectory(cycling=True, steps=53, development=False),
            _trajectory(cycling=False, steps=53, development=False),
        )

    def test_source_block_boundaries_preserve_partial_traversal_and_rng(self) -> None:
        self.assertEqual(
            _trajectory(cycling=True, steps=785, development=True),
            _trajectory(cycling=False, steps=785, development=True),
        )
        torch.manual_seed(123)
        batches = CyclingBatches(_tensors(), batch_size=4)
        for _ in range(196):
            batches.next_batch()
        state = batches.state_dict()
        self.assertEqual((state["cycles"], state["cursor"]), (13, 16))
        before = torch.get_rng_state().clone()
        batches.next_batch()
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        self.assertEqual(batches.state_dict()["cycles"], 13)

    def test_mid_traversal_restore_preserves_next_batches_and_global_rng(self) -> None:
        torch.manual_seed(123)
        tensors = _tensors()
        original = CyclingBatches(tensors, batch_size=4)
        for step in range(19):
            original.next_batch()
            _consume_dropout(step)
        state = original.state_dict()
        rng = torch.get_rng_state().clone()
        expected = []
        for step in range(19, 59):
            expected.append([value.tolist() for value in original.next_batch()])
            _consume_dropout(step)
        expected_rng = torch.get_rng_state().clone()

        before = torch.get_rng_state().clone()
        restored = CyclingBatches(tensors, batch_size=4)
        restored.load_state_dict(state)
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        torch.set_rng_state(rng)
        observed = []
        for step in range(19, 59):
            observed.append([value.tolist() for value in restored.next_batch()])
            _consume_dropout(step)
        self.assertEqual(observed, expected)
        self.assertTrue(torch.equal(expected_rng, torch.get_rng_state()))

    def test_initial_and_exhausted_states_are_lazy(self) -> None:
        torch.manual_seed(123)
        before = torch.get_rng_state().clone()
        batches = CyclingBatches(_tensors(), batch_size=4)
        self.assertEqual(
            batches.state_dict(), {"permutation": None, "cursor": 0, "cycles": 0}
        )
        batches.load_state_dict(batches.state_dict())
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        for _ in range(16):
            batches.next_batch()
        exhausted = batches.state_dict()
        self.assertEqual((exhausted["cycles"], exhausted["cursor"]), (1, 64))
        before = torch.get_rng_state().clone()
        restored = CyclingBatches(_tensors(), batch_size=4)
        restored.load_state_dict(exhausted)
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        _consume_development()
        after_development = torch.get_rng_state().clone()
        expected = batches.next_batch()[0]
        expected_rng = torch.get_rng_state().clone()
        torch.set_rng_state(after_development)
        observed = restored.next_batch()[0]
        self.assertTrue(torch.equal(observed, expected))
        self.assertTrue(torch.equal(expected_rng, torch.get_rng_state()))

    def test_invalid_state_rejected_without_randomness_or_partial_mutation(
        self,
    ) -> None:
        batches = CyclingBatches(_tensors(), batch_size=4)
        batches.next_batch()
        valid = batches.state_dict()
        invalid = [
            {**valid, "cursor": -4},
            {**valid, "cursor": 3},
            {**valid, "cursor": 68},
            {**valid, "cursor": True},
            {**valid, "cycles": 0},
            {**valid, "permutation": torch.zeros(64, dtype=torch.long)},
            {**valid, "permutation": torch.arange(64, dtype=torch.int32)},
            {**valid, "permutation": torch.arange(1, 65)},
            {**valid, "permutation": torch.arange(63)},
            {"permutation": None, "cursor": 4, "cycles": 0},
            {"permutation": None, "cursor": 0, "cycles": 1},
        ]
        before = torch.get_rng_state().clone()
        for state in invalid:
            with self.subTest(state=state), self.assertRaises(ValueError):
                batches.load_state_dict(state)
            current = batches.state_dict()
            self.assertEqual(current["cursor"], valid["cursor"])
            self.assertEqual(current["cycles"], valid["cycles"])
            self.assertTrue(torch.equal(current["permutation"], valid["permutation"]))
            self.assertTrue(torch.equal(before, torch.get_rng_state()))

    def test_incomplete_batches_are_not_silently_admitted(self) -> None:
        with self.assertRaisesRegex(ValueError, "full-size"):
            CyclingBatches(_tensors(), batch_size=3)


if __name__ == "__main__":
    unittest.main()
