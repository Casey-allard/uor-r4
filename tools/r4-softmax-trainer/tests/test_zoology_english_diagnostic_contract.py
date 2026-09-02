"""Direct provenance and permitted-read boundaries; no fitted inference."""

from __future__ import annotations

import copy
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

import torch
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes, tree_cid
from r4_softmax_trainer.zoology_english_diagnostic import contract


def envelope(body, field):
    value = copy.deepcopy(body)
    value[field] = cid_bytes(canonical_json_bytes(value))
    return value


def write(path, payload):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(payload)
    return {"path": path.name, "bytes": len(payload), "cid": cid_bytes(payload)}


class DirectBindingTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        temporary = self.stack.enter_context(tempfile.TemporaryDirectory())
        self.root = Path(temporary)
        self.repo, self.source, self.output = (
            self.root / name for name in ("repo", "source", "out")
        )
        self.stack.enter_context(
            patch.object(contract, "_repo", return_value=self.repo)
        )
        self.stack.enter_context(patch.object(contract, "SOURCE_FILE_COUNT", 1))
        write(self.repo / "historical.py", b"unchanged source\n")
        write(self.repo / contract.PACKAGE / "contract.py", b"new contract\n")
        write(
            self.repo
            / "tools/r4-softmax-trainer/tests/test_zoology_english_diagnostic_contract.py",
            b"focused checks\n",
        )
        files = contract.artifact_records(self.repo, ["historical.py"])
        construction = write(
            self.source / "data/construction.safetensors", b"construction fixture bytes"
        )
        vocabulary = write(
            self.source / "data/vocabulary.json", contract.data._vocabulary_bytes()
        )
        inventory = [
            construction,
            {
                "path": "development.safetensors",
                "bytes": 10,
                "cid": "unopened-development",
            },
            vocabulary,
        ]
        manifest = envelope(
            {
                "schema": contract.data.SCHEMA,
                "policy": contract.data.DATA_POLICY,
                "files": inventory,
                "tree_cid": tree_cid(inventory),
            },
            "manifest_cid",
        )
        write(self.source / "data/manifest.json", canonical_json_bytes(manifest))
        model = write(
            self.source / "fit/model.safetensors",
            b"model identity only; never deserialized",
        )
        model.update(
            path="fit/model.safetensors",
            config={"vocab_size": 4096},
            state_cid="retained-state",
        )
        preparation = envelope(
            {
                "training": {"total_updates": 3920},
                "model_config": model["config"],
                "dataset": manifest,
                "implementation": {"files": files, "tree_cid": tree_cid(files)},
            },
            "preparation_cid",
        )
        fitted = envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "training": preparation["training"],
                "artifact": model,
                "status": "FIT_COMPLETE",
                "completed_updates": 3920,
            },
            "fit_cid",
        )
        evidence = {
            "language": {
                "construction": {
                    "decisions": 8192,
                    "batches": 32,
                    "predictions_cid": "frozen-predictions",
                }
            }
        }
        result = envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "fit_cid": fitted["fit_cid"],
                "artifact": model,
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
                "implementation_cid": tree_cid(files),
            },
            "result_cid",
        )
        replay = envelope(
            {
                "preparation_cid": preparation["preparation_cid"],
                "fit_cid": fitted["fit_cid"],
                "result_cid": result["result_cid"],
                "artifact": model,
                "evidence_cid": result["evidence_cid"],
                "implementation_cid": tree_cid(files),
                "exact_replay": True,
                "fresh_process": True,
            },
            "replay_cid",
        )
        self.documents = {
            "preparation": preparation,
            "fit": fitted,
            "result": result,
            "replay": replay,
        }
        self.cids = {
            name: value[f"{name}_cid"] for name, value in self.documents.items()
        }
        self.stack.enter_context(patch.object(contract, "SOURCE_CIDS", self.cids))
        for name in self.documents:
            self.publish(name)

    def publish(self, name):
        value = self.documents[name]
        payload = canonical_json_bytes(value)
        write(self.source / contract.SOURCE_PATHS[name], payload)
        write(self.repo / f"{contract.PUBLIC_PREFIX}{name}.json", payload)
        self.cids[name] = value[f"{name}_cid"]

    def test_prepare_and_validate_only_read_explicit_allowed_files(self):
        allowed = {p.resolve() for p in self.root.rglob("*") if p.is_file()}
        allowed.add((self.output / "preparation.json").resolve())
        observed = []
        original = Path.read_bytes

        def permitted(path):
            self.assertIn(path.resolve(), allowed)
            observed.append(path.resolve())
            return original(path)

        with (
            patch.object(Path, "read_bytes", permitted),
            patch.object(
                contract.data,
                "validate",
                side_effect=AssertionError("full data validator forbidden"),
            ),
        ):
            preparation = contract.prepare(self.output, self.source)
            self.assertEqual(contract.validate_preparation(self.output), preparation)
        self.assertEqual(
            preparation["source"]["expected_construction"]["decisions"], 8192
        )
        paths = {record["path"] for record in preparation["implementation"]["files"]}
        self.assertIn("historical.py", paths)
        self.assertIn(f"{contract.PUBLIC_PREFIX}fit.json", paths)
        self.assertIn(f"{contract.PACKAGE}/contract.py", paths)
        self.assertFalse(
            any(
                path.name
                in (
                    "development.safetensors",
                    "checkpoint.pt",
                    "h4-frames.json",
                    "token-frames.json",
                )
                for path in observed
            )
        )
        with self.assertRaises(FileExistsError):
            contract.prepare(self.output, self.source)

    def test_rejects_changed_model_and_implementation(self):
        contract.prepare(self.output, self.source)
        model_path = self.source / "fit/model.safetensors"
        old = model_path.read_bytes()
        model_path.write_bytes(old + b"changed")
        with self.assertRaisesRegex(ValueError, "file identity"):
            contract.validate_preparation(self.output)
        model_path.write_bytes(old)
        (self.repo / "historical.py").write_text("changed source\n")
        with self.assertRaisesRegex(ValueError, "historical source"):
            contract.validate_preparation(self.output)

    def test_rejects_validly_resealed_but_unrelated_replay(self):
        replay = dict(self.documents["replay"])
        replay.pop("replay_cid")
        replay["fit_cid"] = "another-fit"
        self.documents["replay"] = envelope(replay, "replay_cid")
        self.publish("replay")
        with self.assertRaisesRegex(ValueError, "relationship"):
            contract.prepare(self.output, self.source)

    def test_rejects_noncanonical_json_and_escaped_path(self):
        path = self.source / "preparation.json"
        path.write_bytes(path.read_bytes() + b" ")
        with self.assertRaisesRegex(ValueError, "canonical"):
            contract.prepare(self.output, self.source)
        with self.assertRaisesRegex(ValueError, "escapes"):
            contract._read_record(self.source, {"path": "../forbidden"}, "../forbidden")


class ConstructionAccessTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.root = Path(self.stack.enter_context(tempfile.TemporaryDirectory()))
        self.stack.enter_context(patch.dict(contract.EVALUATION, rows=8))
        self.stack.enter_context(
            patch.dict(contract.data.DATA_POLICY, construction_groups=2)
        )
        rows, targets = [], []
        for group in range(2):
            facts = [(0, 0, 0), (0, 1, 1), (1, 0, 2), (2, 2, 3)]
            second = 1 if group == 0 else 2
            q0, q1 = facts[0][:2], facts[second][:2]
            swapped = list(facts)
            swapped[0] = (*facts[0][:2], facts[second][2])
            swapped[second] = (*facts[second][:2], facts[0][2])
            for history, query in (
                (facts, q0),
                (facts, q1),
                (swapped, q0),
                (swapped, q1),
                (facts, (3, 3)),
            ):
                row = contract.data._input(history, (0, 1, 2, 3), query)
                rows.append(row)
                targets.append([contract.data.oracle_target(row)])
        self.tensors = {
            "inputs": torch.tensor(rows),
            "positions": torch.full((10, 1), 40),
            "targets": torch.tensor(targets),
            "group_ids": torch.arange(2).repeat_interleave(5),
            "variant_ids": torch.arange(5).repeat(2),
            "pair_types": torch.arange(2).repeat_interleave(5),
        }
        self.install(self.tensors)

    def install(self, tensors):
        construction = write(
            self.root / "construction.safetensors",
            contract.data._canonical_safetensors(tensors),
        )
        vocabulary = write(
            self.root / "vocabulary.json", contract.data._vocabulary_bytes()
        )
        manifest = envelope({"construction": construction}, "manifest_cid")
        manifest_record = write(
            self.root / "manifest.json", canonical_json_bytes(manifest)
        )
        self.preparation = {
            "source": {
                "dataset": {
                    "root": str(self.root),
                    "construction": construction,
                    "vocabulary": vocabulary,
                    "manifest": manifest_record,
                    "manifest_cid": manifest["manifest_cid"],
                }
            }
        }

    def test_loads_only_construction_vocabulary_manifest_and_preserves_canonical_rows(
        self,
    ):
        original, observed = Path.read_bytes, []

        def permitted(path):
            self.assertEqual(path.parent, self.root)
            self.assertIn(
                path.name,
                ("construction.safetensors", "vocabulary.json", "manifest.json"),
            )
            observed.append(path.name)
            return original(path)

        with (
            patch.object(Path, "read_bytes", permitted),
            patch.object(
                contract.data,
                "load_development",
                side_effect=AssertionError("development forbidden"),
            ),
        ):
            tensors = contract.load_construction(self.preparation)
        self.assertEqual(
            set(observed),
            {"construction.safetensors", "vocabulary.json", "manifest.json"},
        )
        self.assertEqual(tensors["variant_ids"].tolist(), [0, 1, 2, 3] * 2)
        self.assertEqual(tensors["group_ids"].tolist(), [0] * 4 + [1] * 4)
        self.assertTrue(
            torch.equal(
                tensors["inputs"],
                self.tensors["inputs"][self.tensors["variant_ids"] < 4],
            )
        )

    def test_rejects_positions_labels_and_semantically_wrong_question_pair(self):
        for error in ("position", "label", "pair_metadata", "question_pair"):
            with self.subTest(error=error):
                tensors = {key: value.clone() for key, value in self.tensors.items()}
                if error == "position":
                    tensors["positions"][0, 0] = 39
                elif error == "label":
                    tensors["targets"][0, 0] = contract.data.UNKNOWN_ID
                elif error == "pair_metadata":
                    tensors["pair_types"][0] = 1
                else:
                    tensors["inputs"][1] = tensors["inputs"][0]
                    tensors["targets"][1] = tensors["targets"][0]
                self.install(tensors)
                with self.assertRaises(ValueError):
                    contract.load_construction(self.preparation)


if __name__ == "__main__":
    unittest.main()
