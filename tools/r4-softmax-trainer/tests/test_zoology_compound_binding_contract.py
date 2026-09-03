"""Recorded-reference lineage and frozen prototype gate boundaries; no models."""

from __future__ import annotations

import copy
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

from r4_softmax_trainer.provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    tree_cid,
)
from r4_softmax_trainer.zoology_compound_binding import contract


def envelope(body, field):
    value = copy.deepcopy(body)
    value.pop(field, None)
    value[field] = cid_bytes(canonical_json_bytes(value))
    return value


class CompoundContractTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(
            self.stack.enter_context(tempfile.TemporaryDirectory())
        ).resolve()
        (self.root / "docs").mkdir()
        (self.root / "historical.py").write_text("unchanged source\n")
        files = artifact_records(self.root, ["historical.py"])
        self.historical = {
            "readout_cids": {"result": "retained-readout"},
            "construction_files": [],
            "baseline": {
                "construction": {"decisions": 8192, "top1_correct": 3735},
                "diagnostic": {"fixture": "canonical full diagnostic"},
                "fit_work": {"optimizer_updates": 3920},
                "model": {"cid": "retained-model-metadata-only"},
            },
        }
        preparation = envelope(
            {
                "training": contract.TRAINING,
                "model_config": contract.previous.MODEL_CONFIG,
                "evaluation": contract.previous.EVALUATION,
                "lineage": self.historical,
                "implementation": {"files": files, "tree_cid": tree_cid(files)},
            },
            "preparation_cid",
        )
        fitted = envelope(
            {
                "status": "FIT_COMPLETE",
                "completed_updates": 3920,
                "work": self.historical["baseline"]["fit_work"],
                "preparation_cid": preparation["preparation_cid"],
                "artifact": {"cid": "prior-negative-model-not-loaded"},
            },
            "fit_cid",
        )
        evidence = {
            "status": "CYCLIC_FACTS_PRESERVATION_MISS",
            "reference_canonical_exact_reproduction": True,
            "reference_views": [
                {
                    "rotation": offset,
                    "construction": self.historical["baseline"]["construction"],
                    "diagnostic": self.historical["baseline"]["diagnostic"],
                }
                for offset in range(4)
            ],
            "all_order_worlds": {"reference": {"worlds": 2048}},
        }
        result = envelope(
            {
                "fit_cid": fitted["fit_cid"],
                "artifact": fitted["artifact"],
                "preparation_cid": preparation["preparation_cid"],
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
            },
            "result_cid",
        )
        replay = envelope(
            {
                "exact_replay": True,
                "fresh_process": True,
                "result_cid": result["result_cid"],
                "evidence_cid": result["evidence_cid"],
            },
            "replay_cid",
        )
        self.documents = {
            "preparation": preparation,
            "fit": fitted,
            "result": result,
            "replay": replay,
        }
        self.cids = {}
        for name in self.documents:
            self.publish(name)
        self.stack.enter_context(
            patch.object(contract, "_repo", return_value=self.root)
        )
        self.stack.enter_context(patch.object(contract, "SOURCE_FILE_COUNT", 1))
        self.stack.enter_context(patch.object(contract, "SOURCE_CIDS", self.cids))
        self.stack.enter_context(
            patch.object(contract.previous, "_lineage", return_value=self.historical)
        )

    def publish(self, name):
        value = envelope(self.documents[name], f"{name}_cid")
        self.documents[name] = value
        self.cids[name] = value[f"{name}_cid"]
        path = self.root / f"docs/r4_zoology_cyclic_facts_1071_{name}.json"
        path.write_bytes(canonical_json_bytes(value))

    def test_new_architecture_retains_four_recorded_views_without_weight_reads(self):
        self.assertNotEqual(contract.MODEL_CONFIG, contract.previous.MODEL_CONFIG)
        original = Path.open

        def guarded_open(path, *args, **kwargs):
            if path.suffix in (".safetensors", ".pt"):
                raise AssertionError("no weights, development or checkpoint reads")
            return original(path, *args, **kwargs)

        with patch.object(Path, "open", guarded_open):
            lineage = contract._lineage()
        self.assertEqual(lineage["baseline"], self.historical["baseline"])
        self.assertEqual(
            [row["rotation"] for row in lineage["reference_views"]], [0, 1, 2, 3]
        )
        self.assertEqual(len(lineage["documents"]), 4)
        (self.root / "historical.py").write_text("changed historical source\n")
        with self.assertRaisesRegex(ValueError, "historical source"):
            contract._lineage()

    def test_self_consistent_new_envelopes_cannot_change_canonical_reference(self):
        result = copy.deepcopy(self.documents["result"])
        result["evidence"]["reference_views"][0]["diagnostic"] = {"changed": True}
        result["evidence_cid"] = cid_bytes(canonical_json_bytes(result["evidence"]))
        self.documents["result"] = result
        self.publish("result")
        self.documents["replay"]["result_cid"] = self.cids["result"]
        self.documents["replay"]["evidence_cid"] = result["evidence_cid"]
        self.publish("replay")
        with self.assertRaisesRegex(ValueError, "canonical reproduction"):
            contract._lineage()

    def test_model_control_order_and_reference_bindings_cannot_change_after_prepare(
        self,
    ):
        bindings = {
            "lineage": {"reference_views": [{"rotation": 0}]},
            "implementation": {"tree_cid": "synthetic-source"},
            "dataset": {"manifest_cid": "synthetic-data"},
        }
        body = {
            "schema": "uor-r4.zoology-compound-binding-preparation/1",
            "issue": contract.ISSUE,
            "policy": contract.POLICY,
            "model_config": contract.MODEL_CONFIG,
            "model_policy": contract.MODEL_POLICY,
            "training": contract.TRAINING,
            "evaluation": contract.EVALUATION,
            "behavior": contract.BEHAVIOR,
            "order": contract.ORDER,
            "control": contract.CONTROL,
            "intervention": contract.INTERVENTION,
            **bindings,
        }
        target = self.root / "preparation.json"

        def write(value):
            target.write_bytes(canonical_json_bytes(envelope(value, "preparation_cid")))

        with patch.object(contract, "_bindings", return_value=bindings):
            write(body)
            contract.validate_preparation(self.root)
            for key in ("model_config", "model_policy", "control", "order", "lineage"):
                with self.subTest(key=key):
                    changed = copy.deepcopy(body)
                    changed[key] = {}
                    write(changed)
                    with self.assertRaises(ValueError):
                        contract.validate_preparation(self.root)


if __name__ == "__main__":
    unittest.main()
