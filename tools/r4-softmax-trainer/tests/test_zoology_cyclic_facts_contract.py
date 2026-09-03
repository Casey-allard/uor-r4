"""Pinned comparison artifact and no-reference-read validation boundaries."""

from __future__ import annotations

import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes
from r4_softmax_trainer.zoology_cyclic_facts import contract


def envelope(body):
    value = copy.deepcopy(body)
    value["preparation_cid"] = cid_bytes(canonical_json_bytes(value))
    return canonical_json_bytes(value)


class CyclicReferenceBindingTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.reference = self.root / "reference"
        self.payload = b"synthetic plain reference; no tensors or model execution"
        self.model = {
            "path": "fit/model.safetensors",
            "bytes": len(self.payload),
            "cid": cid_bytes(self.payload),
            "config": copy.deepcopy(contract.MODEL_CONFIG),
            "state_cid": "synthetic-state",
        }
        self.lineage = {"baseline": {"model": self.model}}

    def test_preparation_reference_check_rejects_changed_bytes_or_owner_policy(self):
        target = self.reference / self.model["path"]
        target.parent.mkdir(parents=True)
        target.write_bytes(self.payload)
        observed = contract._reference(self.reference, self.lineage, inspect_model=True)
        self.assertEqual(observed["model"], self.model)
        target.write_bytes(b"changed bytes must fail before fitting")
        with self.assertRaisesRegex(ValueError, "reference model bytes"):
            contract._reference(self.reference, self.lineage, inspect_model=True)
        changed = copy.deepcopy(self.lineage)
        changed["baseline"]["model"]["query_encoding"] = {"owner_residual": True}
        with (
            patch.object(
                contract, "_record", side_effect=AssertionError("no model read")
            ),
            self.assertRaisesRegex(ValueError, "plain #1067"),
        ):
            contract._reference(self.reference, changed, inspect_model=True)

    def test_validation_binds_reference_and_policy_without_opening_weights(self):
        bindings = {
            "lineage": self.lineage,
            "implementation": {"files": [], "tree_cid": "synthetic-source"},
            "dataset": {"manifest_cid": "synthetic-data"},
        }
        body = {
            "schema": "uor-r4.zoology-cyclic-facts-preparation/1",
            "issue": contract.ISSUE,
            "policy": contract.POLICY,
            "model_config": contract.MODEL_CONFIG,
            "training": contract.TRAINING,
            "evaluation": contract.EVALUATION,
            "behavior": contract.BEHAVIOR,
            "intervention": contract.INTERVENTION,
            "augmentation": contract.AUGMENTATION,
            "reference": {"root": str(self.reference), "model": self.model},
            **bindings,
        }
        target = self.root / "preparation.json"
        target.write_bytes(envelope(body))
        with (
            patch.object(contract, "_bindings", return_value=bindings),
            patch.object(
                contract, "_record", side_effect=AssertionError("no model read")
            ) as reader,
        ):
            validated = contract.validate_preparation(self.root)
            self.assertEqual(validated["reference"], body["reference"])
            self.assertFalse(self.reference.exists())
            for key, replacement, error in (
                ("reference", {**body["reference"], "model": {}}, "reference"),
                ("augmentation", {}, "augmentation"),
            ):
                with self.subTest(key=key):
                    changed = copy.deepcopy(body)
                    changed[key] = replacement
                    target.write_bytes(envelope(changed))
                    with self.assertRaisesRegex(ValueError, error):
                        contract.validate_preparation(self.root)
            reader.assert_not_called()


if __name__ == "__main__":
    unittest.main()
