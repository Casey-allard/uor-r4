"""Counterfactual English split, oracle and lexical-integrity checks only."""

from __future__ import annotations

import random
import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_english_binding import data


class EnglishBindingDataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory()
        cls.root = Path(cls.temporary.name) / "data"
        random.seed(17)
        before = random.getstate()
        cls.metadata = data.build(cls.root)
        if before != random.getstate():
            raise AssertionError("builder consumed global Python RNG")

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def test_split_oracle_and_exact_target_slot_balance_reproduce(self) -> None:
        metadata = data.validate(self.root)
        self.assertEqual(metadata, self.metadata)
        audit = metadata["audit"]
        self.assertEqual(audit["canonical_world_overlap"], 0)
        self.assertEqual(audit["owner_object_pair_overlap"], 0)
        self.assertTrue(audit["active_vocabulary_covered"])
        self.assertEqual(audit["construction"]["supported_rows"], 8192)
        self.assertEqual(audit["development"]["supported_rows"], 1024)
        self.assertEqual(audit["development"]["unknown_rows"], 256)
        self.assertEqual(audit["construction"]["owner_object_pair_count"], 192)
        self.assertEqual(audit["development"]["owner_object_pair_count"], 64)
        for split, expected in (("construction", 1024), ("development", 128)):
            self.assertEqual(
                {
                    audit[split]["target_counts"][location]
                    for location in data.LOCATIONS
                },
                {expected},
            )

    def test_bag_matched_missing_binding_and_lexical_roundtrip(self) -> None:
        population = data.load_development(self.root)
        for start in range(0, 1280, 5):
            rows = population["inputs"][start : start + 5]
            self.assertEqual(Counter(rows[0].tolist()), Counter(rows[2].tolist()))
            self.assertEqual(Counter(rows[0].tolist()), Counter(rows[4].tolist()))
            self.assertEqual(data.oracle_target(rows[4]), data.UNKNOWN_ID)
            self.assertNotEqual(data.oracle_target(rows[0]), data.UNKNOWN_ID)
            self.assertEqual(
                data.encode(data.decode(rows[0], skip_bos=True)), rows[0].tolist()
            )
        reserved = [0, 11, 51, 4095]
        self.assertEqual(data.encode(data.decode(reserved), add_bos=False), reserved)

    def test_training_never_opens_development_and_parser_rejects_corrupt_targets(
        self,
    ) -> None:
        original = data._payload
        opened = []

        def track(root, manifest, name):
            opened.append(name)
            return original(root, manifest, name)

        with patch.object(data, "_payload", side_effect=track):
            supported = data.load_training(self.root, mixed=False)
            mixed = data.load_training(self.root, mixed=True)
        self.assertEqual(
            opened, ["construction.safetensors", "construction.safetensors"]
        )
        self.assertEqual(tuple(supported["train_inputs"].shape), (8192, 41))
        self.assertEqual(tuple(mixed["train_inputs"].shape), (10240, 41))
        self.assertFalse(bool((supported["train_targets"] == data.UNKNOWN_ID).any()))
        self.assertEqual(
            int(torch.count_nonzero(mixed["train_targets"] == data.UNKNOWN_ID)), 2048
        )
        population = data.load_development(self.root)
        population["targets"][0, 0] = data.UNKNOWN_ID
        with self.assertRaisesRegex(ValueError, "parsed oracle"):
            data._audit_split(population, development=True)


if __name__ == "__main__":
    unittest.main()
