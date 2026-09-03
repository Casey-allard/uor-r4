"""Compound-binding data identity, four-history exclusion and access boundaries."""

from __future__ import annotations

import copy
import random
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch
from safetensors.torch import load as load_safetensors

from r4_softmax_trainer.zoology_compound_binding import data
from r4_softmax_trainer.zoology_cyclic_facts import data as previous
from r4_softmax_trainer.zoology_english_binding import data as source


class CompoundBindingDataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.temporary = tempfile.TemporaryDirectory()
        cls.source_root = Path(cls.temporary.name) / "source"
        cls.root = Path(cls.temporary.name) / "compound"
        cls.source_metadata = source.build(cls.source_root)
        cls.source_bytes = (cls.source_root / "construction.safetensors").read_bytes()
        cls.original = load_safetensors(cls.source_bytes)
        read_bytes = Path.read_bytes

        def no_development_payload(path):
            if path.name == "development.safetensors":
                raise AssertionError("build opened a retained development payload")
            return read_bytes(path)

        before = random.getstate()
        source_policy, previous_policy = (
            copy.deepcopy(source.DATA_POLICY),
            copy.deepcopy(previous.DATA_POLICY),
        )
        with patch.object(Path, "read_bytes", no_development_payload):
            cls.metadata = data.build(cls.root, cls.source_root)
        if random.getstate() != before:
            raise AssertionError("builder consumed global Python RNG")
        if (
            source.DATA_POLICY != source_policy
            or previous.DATA_POLICY != previous_policy
        ):
            raise AssertionError("builder changed a predecessor policy")
        cls.construction = data.load_construction(cls.root)
        cls.historical = data._historical_development(cls.metadata, cls.construction)

    @classmethod
    def tearDownClass(cls) -> None:
        cls.temporary.cleanup()

    def test_copied_bytes_and_training_tensors_match_original_except_positions(
        self,
    ) -> None:
        self.assertEqual(
            (self.root / "construction.safetensors").read_bytes(), self.source_bytes
        )
        self.assertEqual(
            (self.root / "vocabulary.json").read_bytes(),
            (self.source_root / "vocabulary.json").read_bytes(),
        )
        self.assertEqual(self.source_metadata["manifest_cid"], data.SOURCE_MANIFEST_CID)
        self.assertEqual(tuple(self.construction["inputs"].shape), (10240, 41))
        for name, original in self.original.items():
            if name == "positions":
                self.assertTrue(bool((original == 40).all()))
                self.assertTrue(bool((self.construction[name] == 37).all()))
            else:
                self.assertTrue(torch.equal(original, self.construction[name]), name)
        supported = data.load_training(self.root, mixed=False)
        self.assertEqual(tuple(supported["train_inputs"].shape), (8192, 41))
        self.assertFalse(bool((supported["train_targets"] == data.UNKNOWN_ID).any()))
        self.assertEqual(
            self.metadata["audit"]["construction"],
            self.source_metadata["audit"]["construction"],
        )
        for row in self.construction["inputs"][:5].tolist():
            _, query, _ = source.parse_row(row)
            self.assertEqual(row[35], source.TOKEN_IDS[query[0]])
            self.assertEqual(row[37], source.TOKEN_IDS[query[1]])

    def test_training_and_default_validation_never_open_or_regenerate_development(
        self,
    ) -> None:
        opened: list[str] = []
        payload = data._payload

        def construction_only(root, metadata, name):
            opened.append(name)
            if name == "development.safetensors":
                raise AssertionError("fitting stage opened development")
            return payload(root, metadata, name)

        with (
            patch.object(data, "_payload", side_effect=construction_only),
            patch.object(
                data,
                "_historical_development",
                side_effect=AssertionError("fitting stage regenerated history"),
            ),
            patch.object(
                data,
                "_published_previous",
                side_effect=AssertionError("fitting stage reopened published history"),
            ),
        ):
            self.assertEqual(data.validate(self.root), self.metadata)
            mixed = data.load_training(self.root, mixed=True)
            data.load_construction(self.root)
        self.assertEqual(set(opened), {"construction.safetensors", "vocabulary.json"})
        self.assertEqual(
            int(torch.count_nonzero(mixed["train_targets"] == data.UNKNOWN_ID)), 2048
        )
        opened.clear()

        def track(root, metadata, name):
            opened.append(name)
            return payload(root, metadata, name)

        with (
            patch.object(data, "_payload", side_effect=track),
            patch.object(
                data,
                "_historical_development",
                side_effect=AssertionError("loader regenerated history"),
            ),
        ):
            fresh = data.load_development(self.root)
        self.assertEqual(opened, ["development.safetensors"])
        self.assertEqual(tuple(fresh["inputs"].shape), (1280, 41))

    def test_four_histories_and_union_exclusion_preserve_valid_balanced_worlds(
        self,
    ) -> None:
        fresh = data.load_development(self.root)

        def worlds(tensors):
            return {
                tuple(sorted(source.parse_row(row)[0]))
                for row in tensors["inputs"].tolist()
            }

        fresh_worlds = worlds(fresh)
        fresh_inputs = {tuple(row) for row in fresh["inputs"].tolist()}
        self.assertEqual(set(self.historical), {"1063", "1067", "1069", "1071"})
        union_worlds = set().union(
            *(worlds(tensors) for tensors in self.historical.values())
        )
        union_inputs = set().union(
            *(
                {tuple(row) for row in tensors["inputs"].tolist()}
                for tensors in self.historical.values()
            )
        )
        audit_union = self.metadata["audit"]["historical_development_union"]
        self.assertEqual(audit_union["canonical_worlds"], len(union_worlds))
        self.assertEqual(audit_union["unique_input_rows"], len(union_inputs))
        self.assertEqual(audit_union["rows"], 5120)
        self.assertEqual(audit_union["populations"], 4)
        self.assertEqual(audit_union["canonical_world_overlap"], 0)
        self.assertEqual(audit_union["exact_input_overlap"], 0)
        all_union = self.metadata["audit"]["all_exclusions_union"]
        self.assertEqual(
            all_union["canonical_worlds"], len(union_worlds | worlds(self.original))
        )
        self.assertEqual(all_union["rows"], 15360)
        self.assertEqual(all_union["populations"], 5)
        self.assertEqual(len(fresh_worlds), 768)
        self.assertFalse(fresh_worlds & worlds(self.original))
        for issue, historical in self.historical.items():
            with self.subTest(issue=issue):
                history_audit = self.metadata["audit"]["historical_development"][issue]
                self.assertEqual(
                    history_audit["canonical_worlds"], len(worlds(historical))
                )
                self.assertEqual(history_audit["rows"], 1280)
                self.assertEqual(history_audit["unique_input_rows"], 1280)
                self.assertFalse(fresh_worlds & worlds(historical))
                self.assertFalse(
                    fresh_inputs & {tuple(row) for row in historical["inputs"].tolist()}
                )
                self.assertEqual(
                    data._record(
                        "development.safetensors",
                        data._canonical_safetensors(historical),
                    ),
                    self.metadata["historical_development"][issue]["development"],
                )
        self.assertEqual(
            self.metadata["historical_development"]["1071"]["manifest_cid"],
            data.PREVIOUS_MANIFEST_CID,
        )
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
        altered = copy.deepcopy(self.metadata)
        altered["historical_development"]["1071"]["development"]["cid"] = "blake3:wrong"
        with self.assertRaisesRegex(ValueError, "#1071 development differs"):
            data._historical_development(altered, self.construction)
        altered = copy.deepcopy(self.metadata)
        altered["historical_development"]["1067"]["development"]["cid"] = "blake3:wrong"
        altered.pop("manifest_cid")
        altered["manifest_cid"] = data.cid_bytes(data.canonical_json_bytes(altered))
        changed_root = Path(self.temporary.name) / "changed-history"
        changed_root.mkdir()
        (changed_root / data.MANIFEST).write_bytes(data.canonical_json_bytes(altered))
        with self.assertRaisesRegex(ValueError, "published chain"):
            data._manifest(changed_root)


if __name__ == "__main__":
    unittest.main()
