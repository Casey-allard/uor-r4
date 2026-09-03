"""Synthetic source/frame provenance and preparation gates; no model execution."""

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
from r4_softmax_trainer.zoology_compound_r4 import contract


def _envelope(body, field):
    value = copy.deepcopy(body)
    value.pop(field, None)
    value[field] = cid_bytes(canonical_json_bytes(value))
    return value


class CompoundR4ContractTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.repo = Path(
            self.stack.enter_context(tempfile.TemporaryDirectory())
        ).resolve()
        self.source = self.repo / "retained"
        for folder in (self.repo / "docs", self.source / "fit", self.source / "data"):
            folder.mkdir(parents=True)
        (self.repo / "historical.py").write_text(
            "unchanged historical implementation\n"
        )
        files = artifact_records(self.repo, ["historical.py"])
        self.implementation = {
            "root": "/historical/worktree",
            "files": files,
            "tree_cid": tree_cid(files),
        }
        # Deliberately invalid tensor serialization: this preparation contract
        # may hash these fixture bytes, but must not deserialize fitted weights.
        self.model_bytes = b"x" * 1148672
        (self.source / "fit/model.safetensors").write_bytes(self.model_bytes)
        model = {
            **artifact_records(self.source, ["fit/model.safetensors"])[0],
            "state_cid": "fixture-learned-state",
            "config": contract.MODEL_CONFIG,
            "model_policy": contract.MODEL_POLICY,
        }
        for name in (
            "construction.safetensors",
            "development.safetensors",
            "vocabulary.json",
        ):
            (self.source / "data" / name).write_bytes(name.encode())
        self.dataset = {
            "manifest_cid": "fixture-data-manifest",
            "files": artifact_records(
                self.source / "data",
                [
                    "construction.safetensors",
                    "development.safetensors",
                    "vocabulary.json",
                ],
            ),
        }
        preparation = _envelope(
            {
                "model_config": contract.MODEL_CONFIG,
                "model_policy": contract.MODEL_POLICY,
                "implementation": self.implementation,
                "dataset": self.dataset,
            },
            "preparation_cid",
        )
        fitted = _envelope(
            {
                "status": "FIT_COMPLETE",
                "completed_updates": 3920,
                "work": {
                    "optimizer_updates": 3920,
                    "train_query_presentations": 2007040,
                },
                "preparation_cid": preparation["preparation_cid"],
                "artifact": model,
            },
            "fit_cid",
        )
        evidence = {
            "status": "COMPOUND_BINDING_FRESH_PASSED",
            "passed": True,
            "criteria": {
                "construction": True,
                "order": True,
                "value_binding_control": True,
                "fresh_binding": True,
            },
            "construction_views": [
                {"rotation": offset, "recorded": "full ordinary evidence"}
                for offset in range(4)
            ],
            "development": {
                "model_decisions": 5120,
                "views": [{"rotation": offset} for offset in range(4)],
            },
            "learned_state_before": model["state_cid"],
            "learned_state_after": model["state_cid"],
        }
        runtime = {"threads": 8, "interop_threads": 1}
        result = _envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "fit_cid": fitted["fit_cid"],
                "artifact": model,
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
                "runtime": runtime,
            },
            "result_cid",
        )
        replay = _envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "fit_cid": fitted["fit_cid"],
                "result_cid": result["result_cid"],
                "evidence_cid": result["evidence_cid"],
                "artifact": model,
                "runtime": runtime,
                "exact_replay": True,
                "fresh_process": True,
                "optimizer_updates": 0,
            },
            "replay_cid",
        )
        self.documents = {
            "preparation": preparation,
            "fit": fitted,
            "result": result,
            "replay": replay,
        }
        source_cids = {}
        for name, value in self.documents.items():
            source_cids[name] = value[f"{name}_cid"]
            payload = canonical_json_bytes(value)
            (
                self.repo / f"docs/r4_zoology_compound_binding_1073_{name}.json"
            ).write_bytes(payload)
            (self.source / contract.SOURCE_PATHS[name]).write_bytes(payload)
        self.stack.enter_context(
            patch.object(contract, "_repo", return_value=self.repo)
        )
        self.stack.enter_context(patch.object(contract, "SOURCE_FILE_COUNT", 1))
        self.stack.enter_context(patch.object(contract, "SOURCE_CIDS", source_cids))
        self.validate_data = self.stack.enter_context(
            patch.object(contract.data, "validate", return_value=self.dataset)
        )

    def test_published_source_and_retained_artifacts_are_bound_without_checkpoint_loads(
        self,
    ):
        original = Path.open

        def guarded_open(path, *args, **kwargs):
            if path.suffix == ".pt":
                raise AssertionError("checkpoint values must remain unread")
            return original(path, *args, **kwargs)

        with patch.object(Path, "open", guarded_open):
            source = contract._source_contract(self.source)
        self.validate_data.assert_called_once_with(
            self.source / "data", inspect_development=False
        )
        self.assertEqual(
            source["baseline_history"], self.documents["result"]["evidence"]
        )
        self.assertEqual(source["model"]["cid"], cid_bytes(self.model_bytes))
        self.assertEqual(len(source["documents"]), 4)
        self.assertEqual(source["checkpoint_reads"], 0)
        (self.source / "fit/model.safetensors").write_bytes(
            self.model_bytes + b"changed"
        )
        with self.assertRaisesRegex(ValueError, "model.safetensors"):
            contract._source_contract(self.source)
        (self.source / "fit/model.safetensors").write_bytes(self.model_bytes)
        (self.repo / "historical.py").write_text("changed implementation\n")
        with self.assertRaisesRegex(ValueError, "historical #1073 implementation"):
            contract._source_contract(self.source)

    def test_local_resealed_evidence_cannot_replace_published_baseline(self):
        changed = copy.deepcopy(self.documents["result"])
        changed["evidence"]["construction_views"][0]["recorded"] = (
            "different ordinary evidence"
        )
        changed["evidence_cid"] = cid_bytes(canonical_json_bytes(changed["evidence"]))
        changed = _envelope(changed, "result_cid")
        (self.source / "result.json").write_bytes(canonical_json_bytes(changed))
        with self.assertRaisesRegex(ValueError, "frozen predecessor"):
            contract._source_contract(self.source)

    def test_frame_relocation_preserves_qualified_integration_but_not_changed_files(
        self,
    ):
        frames = {
            "root": "/old/frame/location",
            "files": [{"path": "native.json", "cid": "native-cid", "bytes": 42}],
            "tree_cid": "native-tree",
            "vocabulary_size": 8192,
        }
        preparation = _envelope(
            {"frames": frames, "implementation": self.implementation}, "preparation_cid"
        )
        evidence = {
            "primary": {"passed": True},
            "control": {"strong_transport_sensitivity": True},
        }
        result = _envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
            },
            "result_cid",
        )
        replay = _envelope(
            {
                "exact_replay": True,
                "fresh_process": True,
                "result_cid": result["result_cid"],
                "evidence_cid": result["evidence_cid"],
            },
            "replay_cid",
        )
        documents = {"preparation": preparation, "result": result, "replay": replay}
        cids = {}
        for name, value in documents.items():
            (
                self.repo / f"docs/r4_zoology_exact_coherent_inference_1061_{name}.json"
            ).write_bytes(canonical_json_bytes(value))
            cids[name] = value[f"{name}_cid"]
        with patch.object(contract, "INTEGRATION_CIDS", cids):
            integration = contract._integration()
        relocated = {**frames, "root": str(self.repo / "frames")}
        with patch.object(contract.prior, "_frame_contract", return_value=relocated):
            self.assertEqual(
                contract._frames(self.repo / "frames", integration), relocated
            )
        changed = copy.deepcopy(relocated)
        changed["files"][0]["cid"] = "changed-native-frame"
        with (
            patch.object(contract.prior, "_frame_contract", return_value=changed),
            self.assertRaisesRegex(ValueError, "native frame bundle"),
        ):
            contract._frames(self.repo / "frames", integration)

    def test_frozen_policy_source_and_preflight_cannot_drift_after_prepare(self):
        bindings = {
            "source": {"root": str(self.source), "model": {"cid": "frozen-model"}},
            "frames": {"root": str(self.repo / "frames"), "tree_cid": "frozen-frames"},
            "integration": {"result_cid": "qualified-integration"},
            "implementation": {"tree_cid": "frozen-source"},
            "preflight": {"passed": True, "views": 8},
        }
        output = self.repo / "output"
        with patch.object(contract, "_bindings", return_value=bindings):
            body = contract.prepare(output, self.source, self.repo / "frames")
            self.assertEqual(contract.validate_preparation(output), body)
            with self.assertRaises(FileExistsError):
                contract.prepare(output, self.source, self.repo / "frames")
            for key in (
                "evaluation",
                "intervention",
                "source",
                "frames",
                "implementation",
                "preflight",
            ):
                with self.subTest(field=key):
                    changed = copy.deepcopy(body)
                    changed[key]["changed"] = True
                    (output / "preparation.json").write_bytes(
                        canonical_json_bytes(_envelope(changed, "preparation_cid"))
                    )
                    with self.assertRaises(ValueError):
                        contract.validate_preparation(output)


if __name__ == "__main__":
    unittest.main()
