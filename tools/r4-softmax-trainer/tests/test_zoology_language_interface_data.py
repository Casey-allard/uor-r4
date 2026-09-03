"""Tiny lexical fixtures exercise rendering, supervision and evaluation access."""

from __future__ import annotations

import tempfile
import unittest
from collections import Counter
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.zoology_english_binding import data as english
from r4_softmax_trainer.zoology_language_interface import data


def _source(*, development: bool) -> dict:
    """Two explicit worlds only; no predecessor generator or retained data."""
    rows = []
    objects = (0, 4, 8) if development else (1, 5, 9)
    a, b, c = 0, 4, 8
    x, y, z = objects
    for pair_type in range(2):
        facts = [(a, x, 0), (a, y, 1), (b, x, 2), (c, z, 3)]
        second = 1 if pair_type == 0 else 2
        swapped = list(facts)
        swapped[0] = (*facts[0][:2], facts[second][2])
        swapped[second] = (*facts[second][:2], facts[0][2])
        absent = list(facts)
        absent[0], absent[3] = (a, z, 0), (c, x, 3)
        q0, q1 = (a, x), (a, y) if pair_type == 0 else (b, x)
        for world, query in (
            (facts, q0),
            (facts, q1),
            (swapped, q0),
            (swapped, q1),
            (absent, q0),
        ):
            rows.append(english._input(world, [2, 0, 3, 1], query))
    return {
        "inputs": torch.tensor(rows, dtype=torch.long),
        "targets": torch.tensor(
            [[english.oracle_target(row)] for row in rows], dtype=torch.long
        ),
        "group_ids": torch.tensor([index // 5 for index in range(10)]),
        "variant_ids": torch.tensor([index % 5 for index in range(10)]),
        "pair_types": torch.tensor([index // 5 for index in range(10)]),
    }


def _source_identity() -> dict:
    return {
        "manifest_cid": data.SOURCE_MANIFEST_CID,
        "tree_cid": data.SOURCE_TREE_CID,
        "files": [
            {
                "path": "construction.safetensors",
                "bytes": 3768792,
                "cid": "blake3:d767fafdf544f01db99d9acb317c76df55e9f9d28f99785d2a6ae62b663731a2",
            },
            {
                "path": "development.safetensors",
                "bytes": 471496,
                "cid": "blake3:0e343a7448098ea2a22d850d4d9fb31d75c55090807f4b2f00fa20531a422335",
            },
            {
                "path": "vocabulary.json",
                "bytes": 65283,
                "cid": "blake3:aa001c3a4369ad2f8bb3596a316270bd72b736f927158ee04403116b430c649d",
            },
        ],
    }


class LanguageInterfaceDataTests(unittest.TestCase):
    def test_templates_move_roles_and_preserve_canonical_semantics(self) -> None:
        source = _source(development=True)
        rendered = data._render_population(source, (0, 1, 2, 3))
        audit = data._audit_population(rendered, development=True)
        self.assertEqual(tuple(rendered["inputs"].shape), (40, 5, 13))
        self.assertTrue(
            torch.equal(rendered["canonical_inputs"], source["inputs"].repeat(4, 1))
        )
        self.assertTrue(
            torch.equal(rendered["targets"], source["targets"].repeat(4, 1))
        )
        self.assertEqual(audit["role_labels"], 560)
        expected = ((0, 7, 10), (7, 10, 2), (3, 6, 9), (4, 11, 2))
        for view, positions in enumerate(expected):
            self.assertEqual(
                rendered["role_positions"][view * 10, 0].tolist(), list(positions)
            )
            self.assertEqual(
                rendered["role_positions"][view * 10, 4].tolist(), [6, 3, -100]
            )
        self.assertEqual(
            [rendered["lengths"][view * 10, 0].item() for view in range(4)],
            [12, 12, 11, 13],
        )
        self.assertEqual(data.VOCABULARY[:52], english.VOCABULARY[:52])
        self.assertEqual(english.VOCABULARY[52], "<unused-0052>")

    def test_same_bag_counterfactuals_require_context_and_oracle_catches_corruption(
        self,
    ) -> None:
        rendered = data._render_population(_source(development=False), (0, 1))
        audit = data._audit_population(rendered, development=False)
        self.assertEqual(
            audit["views"]["0"]["same_object_identical_query_bag_pairs"], 2
        )
        q0, q1 = rendered["inputs"][5], rendered["inputs"][6]
        self.assertTrue(torch.equal(q0[:4], q1[:4]))
        self.assertEqual(Counter(q0[4].tolist()), Counter(q1[4].tolist()))
        self.assertNotEqual(
            rendered["targets"][5].item(), rendered["targets"][6].item()
        )
        self.assertEqual(rendered["targets"][9].item(), data.UNKNOWN_ID)
        wrong_target = {key: value.clone() for key, value in rendered.items()}
        wrong_target["targets"][0, 0] = data.UNKNOWN_ID
        with self.assertRaisesRegex(ValueError, "semantic oracle"):
            data._audit_population(wrong_target, development=False)
        wrong_role = {key: value.clone() for key, value in rendered.items()}
        wrong_role["role_positions"][0, 0, 0] = 3
        with self.assertRaisesRegex(ValueError, "role supervision"):
            data._audit_population(wrong_role, development=False)

    def test_partition_and_seen_heldout_combinations_are_explicit(self) -> None:
        construction = data._render_population(_source(development=False), (0, 1))
        development = data._render_population(_source(development=True), (0, 1, 2, 3))
        data._audit_population(construction, development=False)
        data._audit_population(development, development=True)
        joint = data._audit_joint(construction, development)
        self.assertEqual(joint["construction_development_exact_input_overlap"], 0)
        self.assertEqual(joint["fresh_semantic_worlds"], 0)
        self.assertTrue(
            set(data.VIEW_COMBINATIONS[:2]).isdisjoint(data.VIEW_COMBINATIONS[2:])
        )
        self.assertEqual(construction["view_ids"].tolist(), [0] * 10 + [1] * 10)
        # No other fact owner satisfies both non-held-out pairs here: use +4.
        choices, fallback = data._query_distractors(
            [("mara", "book", "drawer"), ("noah", "coin", "cabinet")],
            ("mara", "book"),
            ("mara", "coin"),
            0,
        )
        self.assertTrue(fallback)
        self.assertEqual(choices, ("iris", "iris"))

    def test_default_validation_and_construction_load_keep_development_closed(
        self,
    ) -> None:
        construction = data._render_population(_source(development=False), (0, 1))
        development = data._render_population(_source(development=True), (0, 1, 2, 3))
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / "data"
            manifest = data._write_package(
                root, _source_identity(), construction, development
            )
            original_load = data._load
            opened = []

            def construction_only(root, metadata, name):
                opened.append(name)
                if name == "development.safetensors":
                    raise AssertionError("default validation deserialized development")
                return original_load(root, metadata, name)

            with (
                patch.object(data, "_load", side_effect=construction_only),
                patch.object(
                    data.retained,
                    "load_development",
                    side_effect=AssertionError("historical population reopened"),
                ),
            ):
                self.assertEqual(data.validate(root), manifest)
                self.assertTrue(
                    torch.equal(
                        data.load_construction(root)["inputs"], construction["inputs"]
                    )
                )
            self.assertEqual(opened, ["construction.safetensors"] * 2)
            self.assertEqual(data.validate(root, inspect_development=True), manifest)
            payload = root / "development.safetensors"
            payload.write_bytes(payload.read_bytes() + b"changed")
            with self.assertRaisesRegex(ValueError, "payload identity"):
                data.validate(root)


if __name__ == "__main__":
    unittest.main()
