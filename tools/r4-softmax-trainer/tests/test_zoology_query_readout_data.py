"""Data identity, stage-specific access and fresh-world semantics; no model."""

from __future__ import annotations

import random
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch
from r4_softmax_trainer.zoology_english_binding import data as source
from r4_softmax_trainer.zoology_query_readout import data
from safetensors.torch import load as load_safetensors


class QueryReadoutDataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory()
        cls.source_root = Path(cls.temporary.name) / "source"
        cls.root = Path(cls.temporary.name) / "new"
        cls.source_metadata = source.build(cls.source_root)
        cls.source_construction_bytes = (
            cls.source_root / "construction.safetensors"
        ).read_bytes()
        cls.source_construction = load_safetensors(cls.source_construction_bytes)
        cls.historical = source._build_split(development=True)
        read_bytes = Path.read_bytes
        historical_path = cls.source_root / "development.safetensors"

        def forbid_historical_payload(path):
            if path == historical_path:
                raise AssertionError(
                    "historical development payload must remain unopened"
                )
            return read_bytes(path)

        before = random.getstate()
        with patch.object(Path, "read_bytes", forbid_historical_payload):
            cls.metadata = data.build(cls.root, cls.source_root)
        if random.getstate() != before:
            raise AssertionError("fresh builder consumed global Python RNG")

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def test_construction_bytes_and_every_other_tensor_are_identical(self) -> None:
        self.assertEqual(self.source_metadata["manifest_cid"], data.SOURCE_MANIFEST_CID)
        self.assertEqual(
            (self.root / "construction.safetensors").read_bytes(),
            self.source_construction_bytes,
        )
        self.assertEqual(
            (self.root / "vocabulary.json").read_bytes(),
            (self.source_root / "vocabulary.json").read_bytes(),
        )
        construction = data.load_construction(self.root)
        self.assertEqual(tuple(construction["inputs"].shape), (10240, 41))
        for name, original in self.source_construction.items():
            if name == "positions":
                self.assertTrue(bool((original == 40).all()))
                self.assertTrue(bool((construction[name] == 37).all()))
            else:
                self.assertTrue(torch.equal(original, construction[name]), name)
        supported = data.load_training(self.root, mixed=False)
        self.assertEqual(tuple(supported["train_inputs"].shape), (8192, 41))
        self.assertFalse(bool((supported["train_targets"] == data.UNKNOWN_ID).any()))
        for row in construction["inputs"][:5].tolist():
            _, query, _ = source.parse_row(row)
            self.assertEqual(row[35], source.TOKEN_IDS[query[0]])
            self.assertEqual(row[37], source.TOKEN_IDS[query[1]])
        self.assertEqual(
            self.metadata["audit"]["construction"],
            self.source_metadata["audit"]["construction"],
        )

    def test_default_validation_and_training_do_not_open_or_regenerate_development(
        self,
    ) -> None:
        opened: list[str] = []
        original_payload = data._payload

        def construction_only(root, metadata, name):
            opened.append(name)
            if name == "development.safetensors":
                raise AssertionError("fitting stage opened development")
            return original_payload(root, metadata, name)

        with (
            patch.object(data, "_payload", side_effect=construction_only),
            patch.object(
                data,
                "_historical_development",
                side_effect=AssertionError("fitting stage regenerated development"),
            ),
        ):
            self.assertEqual(data.validate(self.root), self.metadata)
            mixed = data.load_training(self.root, mixed=True)
            data.load_construction(self.root)
        self.assertEqual(set(opened), {"construction.safetensors", "vocabulary.json"})
        self.assertEqual(tuple(mixed["train_inputs"].shape), (10240, 41))
        self.assertEqual(
            int(torch.count_nonzero(mixed["train_targets"] == data.UNKNOWN_ID)), 2048
        )
        opened.clear()

        def development_only(root, metadata, name):
            opened.append(name)
            return original_payload(root, metadata, name)

        with (
            patch.object(data, "_payload", side_effect=development_only),
            patch.object(
                data,
                "_historical_development",
                side_effect=AssertionError("development loader regenerated old data"),
            ),
        ):
            development = data.load_development(self.root)
        self.assertEqual(opened, ["development.safetensors"])
        self.assertEqual(tuple(development["inputs"].shape), (1280, 41))

    def test_fresh_all_variant_worlds_labels_and_balance_reproduce(self) -> None:
        fresh = data.load_development(self.root)

        def worlds(tensors):
            return {
                tuple(sorted(source.parse_row(row)[0]))
                for row in tensors["inputs"].tolist()
            }

        fresh_worlds = worlds(fresh)
        self.assertEqual(len(fresh_worlds), 256 * 3)
        self.assertFalse(fresh_worlds & worlds(self.historical))
        self.assertFalse(fresh_worlds & worlds(self.source_construction))
        fresh_inputs = {tuple(row) for row in fresh["inputs"].tolist()}
        old_inputs = {tuple(row) for row in self.historical["inputs"].tolist()}
        self.assertFalse(fresh_inputs & old_inputs)
        self.assertEqual(
            data.validate(self.root, inspect_development=True), self.metadata
        )
        audit = self.metadata["audit"]["development"]
        self.assertEqual(audit["owner_object_pair_count"], 64)
        self.assertEqual(
            {audit["target_counts"][location] for location in data.LOCATIONS}, {128}
        )
        self.assertEqual(audit["target_counts"]["unknown"], 256)
        for kind in source.PAIR_TYPES:
            for question in ("q0", "q1"):
                self.assertEqual(
                    set(audit["relevant_fact_slot_counts"][kind][question].values()),
                    {32},
                )
        self.assertEqual(
            data.build(Path(self.temporary.name) / "repeat", self.source_root),
            self.metadata,
        )
        corrupt = {name: tensor.clone() for name, tensor in fresh.items()}
        corrupt["targets"][0, 0] = data.UNKNOWN_ID
        with self.assertRaisesRegex(ValueError, "parsed oracle"):
            data._audit_population(corrupt, development=True)


if __name__ == "__main__":
    unittest.main()
