"""Synthetic lineage and preparation checks without models or real populations."""

from __future__ import annotations

import copy
import sys
import tempfile
import types
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import Mock, patch

from r4_softmax_trainer import zoology_language_interface as package
from r4_softmax_trainer.provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    tree_cid,
)
from r4_softmax_trainer.zoology_language_interface import contract


def _envelope(body, field):
    unsigned = {key: value for key, value in body.items() if key != field}
    return {**unsigned, field: cid_bytes(canonical_json_bytes(unsigned))}


class LanguageInterfaceContractTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.repo = Path(
            self.stack.enter_context(tempfile.TemporaryDirectory())
        ).resolve()
        self.source_root, self.prior_root = (
            self.repo / "relocated-source",
            self.repo / "relocated-prior",
        )
        (self.repo / "docs").mkdir()
        self.prior_root.mkdir()
        (self.repo / "historical.py").write_text("frozen source\n")
        records = artifact_records(self.repo, ["historical.py"])
        self.implementation = {"files": records, "tree_cid": tree_cid(records)}
        self.source = {
            "root": str(self.source_root),
            "model": {"cid": "frozen-model", "state_cid": "frozen-state"},
            "result_cid": "source-result",
            "runtime": {"threads": 8, "interop_threads": 1},
        }
        preparation = _envelope(
            {
                "issue": 1075,
                "source": {
                    **copy.deepcopy(self.source),
                    "root": "/obsolete/source/root",
                },
                "frames": {
                    "root": "/unread/native/frames",
                    "tree_cid": contract.FRAME_TREE_CID,
                },
                "implementation": self.implementation,
            },
            "preparation_cid",
        )
        evidence = {
            "status": "COMPOUND_R4_PRESERVED",
            "preserved": True,
            "ordinary_exact_reproduction": True,
            "primary": {"passed": True},
            "control": {"valid": True, "strong_transport_sensitivity": True},
            "learned_state_before": "frozen-state",
            "learned_state_after": "frozen-state",
            "model_file_cid": "frozen-model",
            "parameter_count": 286976,
            "optimizer_updates": 0,
            "new_parameters": 0,
            "geometry_changes": 0,
            "native_geometry_exports": 0,
        }
        result = _envelope(
            {
                "issue": 1075,
                "preparation_cid": preparation["preparation_cid"],
                "source_result_cid": self.source["result_cid"],
                "model": self.source["model"],
                "frames": preparation["frames"],
                "runtime": self.source["runtime"],
                "implementation_cid": self.implementation["tree_cid"],
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
                "process_id": 100,
            },
            "result_cid",
        )
        replay = _envelope(
            {
                **{
                    key: result[key]
                    for key in (
                        "issue",
                        "preparation_cid",
                        "source_result_cid",
                        "model",
                        "frames",
                        "runtime",
                        "implementation_cid",
                        "evidence_cid",
                        "result_cid",
                    )
                },
                "exact_replay": True,
                "fresh_process": True,
                "optimizer_updates": 0,
                "process_id": 101,
            },
            "replay_cid",
        )
        self.documents = {
            "preparation": preparation,
            "result": result,
            "replay": replay,
        }
        self.source_cids = {}
        for name, value in self.documents.items():
            self._publish(name, value)
        self.stack.enter_context(
            patch.object(contract, "_repo", return_value=self.repo)
        )
        self.stack.enter_context(patch.object(contract, "SOURCE_FILE_COUNT", 1))
        self.stack.enter_context(
            patch.object(contract, "SOURCE_CIDS", self.source_cids)
        )
        self.source_check = self.stack.enter_context(
            patch.object(
                contract.previous, "_source_contract", return_value=self.source
            )
        )

    def _publish(self, name, value):
        value = _envelope(value, f"{name}_cid")
        self.source_cids[name] = value[f"{name}_cid"]
        payload = canonical_json_bytes(value)
        (self.repo / f"docs/r4_zoology_compound_r4_1075_{name}.json").write_bytes(
            payload
        )
        (self.prior_root / f"{name}.json").write_bytes(payload)

    def test_relocated_source_and_published_prior_bind_without_native_frame_reads(self):
        with (
            patch.object(
                contract.previous,
                "_preflight",
                side_effect=AssertionError("no frame execution"),
            ),
            patch.object(
                contract.previous.prior,
                "_frame_contract",
                side_effect=AssertionError("no frame reads"),
            ),
        ):
            lineage = contract._lineage(self.source_root, self.prior_root)
        self.source_check.assert_called_once_with(self.source_root)
        self.assertEqual(lineage["source"]["root"], str(self.source_root))
        self.assertEqual(lineage["source"]["model"], self.source["model"])
        self.assertEqual(lineage["prior"]["native_frame_reads"], 0)
        self.assertEqual(len(lineage["prior"]["public_documents"]), 3)
        (self.repo / "historical.py").write_text("changed source\n")
        with self.assertRaisesRegex(ValueError, "historical #1075 implementation"):
            contract._lineage(self.source_root, self.prior_root)

    def test_resealed_local_evidence_and_different_source_cannot_replace_publication(
        self,
    ):
        changed = copy.deepcopy(self.documents["result"])
        changed["evidence"]["preserved"] = False
        changed["evidence_cid"] = cid_bytes(canonical_json_bytes(changed["evidence"]))
        (self.prior_root / "result.json").write_bytes(
            canonical_json_bytes(_envelope(changed, "result_cid"))
        )
        with self.assertRaisesRegex(ValueError, "frozen predecessor"):
            contract._lineage(self.source_root, self.prior_root)
        self._publish("result", self.documents["result"])
        self.source["model"] = {"cid": "another-model", "state_cid": "another-state"}
        with self.assertRaisesRegex(ValueError, "preserved #1075 source"):
            contract._lineage(self.source_root, self.prior_root)

    def test_prior_replay_relationship_remains_required_with_valid_envelopes(self):
        for key, value in (
            ("fresh_process", False),
            ("process_id", 100),
            ("optimizer_updates", 1),
        ):
            with self.subTest(field=key):
                changed = {**self.documents["replay"], key: value}
                self._publish("replay", changed)
                with self.assertRaisesRegex(
                    ValueError, "source/result/replay relationship"
                ):
                    contract._lineage(self.source_root, self.prior_root)

    def test_prepare_is_exclusive_and_validation_rejects_policy_or_dataset_drift(self):
        fake = types.ModuleType(f"{package.__name__}.data")
        fake.DATA_POLICY = {"synthetic": True}
        fake.validate = Mock()

        def create(data_root, source_root):
            self.assertEqual(source_root, self.source_root)
            data_root.mkdir()
            for name in ("construction.safetensors", "development.safetensors"):
                (data_root / name).write_bytes(
                    b"invalid tensor bytes; hash-only fixture"
                )
            records = artifact_records(
                data_root, ["construction.safetensors", "development.safetensors"]
            )
            manifest = {"files": records, "tree_cid": tree_cid(records)}
            fake.validate.return_value = manifest
            return manifest

        fake.prepare = Mock(side_effect=create)
        root = self.repo / "experiment"
        with (
            patch.object(package, "data", fake, create=True),
            patch.dict(sys.modules, {fake.__name__: fake}),
            patch.object(contract, "_implementation", return_value=self.implementation),
        ):
            prepared = contract.prepare(root, self.source_root, self.prior_root)
            self.assertEqual(contract.validate_preparation(root), prepared)
            fake.validate.assert_called_with(root / "data", inspect_development=False)
            with self.assertRaises(FileExistsError):
                contract.prepare(root, self.source_root, self.prior_root)
            self.assertEqual(fake.prepare.call_count, 1)
            changed = copy.deepcopy(prepared)
            changed["training"]["updates"] = 513
            (root / "preparation.json").write_bytes(
                canonical_json_bytes(_envelope(changed, "preparation_cid"))
            )
            with self.assertRaisesRegex(ValueError, "policy or preparation phase"):
                contract.validate_preparation(root)
            (root / "preparation.json").write_bytes(canonical_json_bytes(prepared))
            (root / "data/development.safetensors").write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "dataset bytes or tree"):
                contract.validate_preparation(root)
            fake.prepare.side_effect = RuntimeError("synthetic interrupted preparation")
            interrupted = self.repo / "interrupted"
            with self.assertRaisesRegex(RuntimeError, "interrupted preparation"):
                contract.prepare(interrupted, self.source_root, self.prior_root)
            with self.assertRaises(FileExistsError):
                contract.prepare(interrupted, self.source_root, self.prior_root)
            self.assertEqual(fake.prepare.call_count, 2)


if __name__ == "__main__":
    unittest.main()
