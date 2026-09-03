"""Synthetic fact/label preservation and the exact traversal-tail ledger; no fitting."""

from __future__ import annotations

import unittest

import torch

from r4_softmax_trainer.zoology_cyclic_facts.augmentation import (
    augment_training_batch,
    rotate_inputs,
    rotation_ledger,
    rotation_offset,
)
from r4_softmax_trainer.zoology_english_binding import data as lexical


class CyclicFactAugmentationTests(unittest.TestCase):
    def test_all_rotations_preserve_intact_counterfactual_facts_labels_and_rng(
        self,
    ) -> None:
        facts = [(0, 0, 0), (0, 1, 1), (1, 0, 2), (2, 2, 3)]
        swapped = [(0, 0, 1), (0, 1, 0), (1, 0, 2), (2, 2, 3)]
        absent = [(0, 2, 0), (0, 1, 1), (1, 0, 2), (2, 0, 3)]
        order = [2, 0, 3, 1]
        inputs = torch.tensor(
            [
                lexical._input(world, order, query)
                for world, query in (
                    (facts, (0, 0)),
                    (facts, (0, 1)),
                    (swapped, (0, 0)),
                    (swapped, (0, 1)),
                    (absent, (0, 0)),
                )
            ]
        )
        original = inputs.clone()
        positions = torch.full((5, 1), 37, dtype=torch.long)
        targets = torch.tensor([[lexical.oracle_target(row)] for row in inputs])
        rng = torch.get_rng_state().clone()
        for offset in range(4):
            with self.subTest(offset=offset):
                rotated, observed_positions, observed_targets = augment_training_batch(
                    (inputs, positions, targets), completed_updates=offset * 16
                )
                self.assertIs(observed_positions, positions)
                self.assertIs(observed_targets, targets)
                self.assertNotEqual(rotated.data_ptr(), inputs.data_ptr())
                self.assertTrue(torch.equal(rotated[:, 0], inputs[:, 0]))
                self.assertTrue(torch.equal(rotated[:, 33:], inputs[:, 33:]))
                self.assertTrue(
                    torch.equal(rotated.sort(dim=1).values, inputs.sort(dim=1).values)
                )
                for before, after, target in zip(inputs, rotated, targets, strict=True):
                    old_facts, old_query, _ = lexical.parse_row(before)
                    new_facts, new_query, _ = lexical.parse_row(after)
                    expected = (
                        old_facts[-offset:] + old_facts[:-offset]
                        if offset
                        else old_facts
                    )
                    self.assertEqual(new_facts, expected)
                    self.assertEqual(new_query, old_query)
                    self.assertEqual(lexical.oracle_target(after), int(target[0]))
        self.assertTrue(torch.equal(inputs, original))
        self.assertTrue(torch.equal(torch.get_rng_state(), rng))

    def test_schedule_boundaries_phase_reset_and_invalid_inputs(self) -> None:
        expected = {
            0: 0,
            15: 0,
            16: 1,
            63: 3,
            64: 0,
            2351: 2,
            2352: 0,
            2371: 0,
            2372: 1,
            2431: 3,
            2432: 0,
            3919: 2,
        }
        for updates, offset in expected.items():
            self.assertEqual(rotation_offset(updates), offset)
        for updates in (-1, 3920, True, 1.0):
            with self.assertRaises(ValueError):
                rotation_offset(updates)
        for offset in (-1, 4, True):
            with self.assertRaises(ValueError):
                rotate_inputs(torch.zeros((1, 41), dtype=torch.long), offset)
        for inputs in (
            torch.zeros((1, 40), dtype=torch.long),
            torch.zeros((1, 41)),
            torch.zeros((0, 41), dtype=torch.long),
        ):
            with self.assertRaises(ValueError):
                rotate_inputs(inputs, 0)

    def test_ledger_uses_measured_partial_unknown_count_and_preserves_all_totals(
        self,
    ) -> None:
        ledger = rotation_ledger(3920, 160588)
        self.assertEqual(
            [
                row["optimizer_updates"]
                for row in ledger["supported_phase"]["by_offset"]
            ],
            [592, 592, 592, 576],
        )
        self.assertEqual(
            [row["optimizer_updates"] for row in ledger["mixed_phase"]["by_offset"]],
            [400, 400, 388, 380],
        )
        self.assertEqual(
            [row["presentations"] for row in ledger["totals_by_offset"]],
            [507904, 507904, 501760, 489472],
        )
        self.assertEqual(
            [row["unknown_presentations"] for row in ledger["totals_by_offset"]],
            [40960, 40960, 39756, 38912],
        )
        self.assertEqual(
            [row["supported_presentations"] for row in ledger["totals_by_offset"]],
            [466944, 466944, 462004, 450560],
        )
        self.assertEqual(ledger["mixed_phase"]["complete_traversals"], 78)
        self.assertEqual(
            ledger["mixed_phase"]["partial_traversal"],
            {
                "offset": 2,
                "updates": 8,
                "presentations": 4096,
                "unknown_presentations": 844,
            },
        )
        self.assertEqual(
            (
                ledger["train_query_presentations"],
                ledger["supported_presentations"],
                ledger["unknown_presentations"],
            ),
            (2007040, 1846452, 160588),
        )
        self.assertEqual(rotation_ledger(0, 0)["train_query_presentations"], 0)
        stopped = rotation_ledger(2352 + 8, 844)
        self.assertEqual(
            stopped["mixed_phase"]["by_offset"][0]["supported_presentations"], 3252
        )
        complete = rotation_ledger(2352 + 20, 2048)
        self.assertIsNone(complete["mixed_phase"]["partial_traversal"]["offset"])
        changed = rotation_ledger(3920, 160600)
        self.assertEqual(
            changed["mixed_phase"]["partial_traversal"]["unknown_presentations"], 856
        )
        self.assertEqual(changed["totals_by_offset"][2]["unknown_presentations"], 39768)
        for updates, unknown in (
            (2352, 1),
            (2352 + 17, 0),
            (2352 + 20, 2047),
            (3921, 160588),
            (0, True),
            (0, -1),
        ):
            with self.assertRaises(ValueError):
                rotation_ledger(updates, unknown)


if __name__ == "__main__":
    unittest.main()
