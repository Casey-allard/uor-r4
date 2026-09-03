"""Synthetic source, opportunity and phase checks without fitted forwards."""

from __future__ import annotations

import copy
import tempfile
import types
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    tree_cid,
)
from r4_softmax_trainer.zoology_language_r4 import contract


def _envelope(body, field):
    unsigned = {key: value for key, value in body.items() if key != field}
    return {**unsigned, field: cid_bytes(canonical_json_bytes(unsigned))}


class LanguageR4ContractTests(unittest.TestCase):
    def setUp(self):
        self.stack = ExitStack()
        self.addCleanup(self.stack.close)
        self.repo = Path(
            self.stack.enter_context(tempfile.TemporaryDirectory())
        ).resolve()
        self.source_root = self.repo / "relocated-reader"
        (self.repo / "docs").mkdir()
        (self.source_root / "fit").mkdir(parents=True)
        (self.repo / "historical.py").write_text("frozen historical source\n")
        files = artifact_records(self.repo, ["historical.py"])
        self.implementation = {
            "root": "/obsolete/worktree",
            "files": files,
            "tree_cid": tree_cid(files),
        }
        self.reader_bytes = b"r" * 566692
        (self.source_root / "fit/reader.safetensors").write_bytes(self.reader_bytes)
        reader = {
            **artifact_records(self.source_root, ["fit/reader.safetensors"])[0],
            "parameter_count": 141571,
            "state_cid": "reader-state",
        }
        self.core = {
            "root": "/retained/core",
            "model": {"cid": "core-file", "state_cid": "core-state"},
        }
        self.lineage = {
            "source": self.core,
            "prior": {
                "root": "/retained/1075",
                "frame_tree_cid": contract.previous.FRAME_TREE_CID,
            },
        }
        self.dataset = {"manifest_cid": "observed-data", "files": []}
        preparation = _envelope(
            {
                "issue": 1077,
                "model_config": contract.MODEL_CONFIG,
                "model_policy": contract.MODEL_POLICY,
                "data_policy": contract.data.DATA_POLICY,
                "training": contract.previous.TRAINING,
                **self.lineage,
                "dataset": self.dataset,
                "implementation": self.implementation,
            },
            "preparation_cid",
        )
        runtime = {"threads": 4, "interop_threads": 1, "device": "cpu"}
        fitted = _envelope(
            {
                "issue": 1077,
                "status": "FIT_COMPLETE",
                "preparation_cid": preparation["preparation_cid"],
                "implementation_cid": self.implementation["tree_cid"],
                "optimizer_updates": 512,
                "row_presentations": 65536,
                "role_label_presentations": 917504,
                "core_optimizer_updates": 0,
                "development_tensor_reads": 0,
                "core_file_cid": "core-file",
                "core_state_cid": "core-state",
                "artifact": reader,
                "runtime": runtime,
            },
            "fit_cid",
        )
        evidence = {
            "status": "LANGUAGE_INTERFACE_HELDOUT_PASSED",
            "passed": True,
            "reader_state_before": "reader-state",
            "reader_state_after": "reader-state",
            "core_state_before": "core-state",
            "core_state_after": "core-state",
            "evaluation_optimizer_updates": 0,
            "core_optimizer_updates": 0,
            "r4_forwards": 0,
            "construction": [
                {"view_id": i, "qualification": {"passed": True}} for i in range(2)
            ],
            "development": [
                {"view_id": i, "qualification": {"passed": True}} for i in range(4)
            ],
        }
        result = _envelope(
            {
                "issue": 1077,
                "preparation_cid": preparation["preparation_cid"],
                "implementation_cid": self.implementation["tree_cid"],
                "fit_cid": fitted["fit_cid"],
                "reader": reader,
                "core": self.core["model"],
                "runtime": runtime,
                "process_id": 100,
                "evidence": evidence,
                "evidence_cid": cid_bytes(canonical_json_bytes(evidence)),
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
                        "implementation_cid",
                        "fit_cid",
                        "reader",
                        "core",
                        "runtime",
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
            "fit": fitted,
            "result": result,
            "replay": replay,
        }
        cids = {}
        for name, value in self.documents.items():
            payload = canonical_json_bytes(value)
            (
                self.repo / f"docs/r4_zoology_language_interface_1077_{name}.json"
            ).write_bytes(payload)
            (self.source_root / f"{name}.json").write_bytes(payload)
            cids[name] = value[f"{name}_cid"]
        self.stack.enter_context(
            patch.object(contract, "_repo", return_value=self.repo)
        )
        self.stack.enter_context(patch.object(contract, "SOURCE_CIDS", cids))
        self.stack.enter_context(patch.object(contract, "SOURCE_FILE_COUNT", 1))
        self.stack.enter_context(
            patch.object(contract.previous, "_lineage", return_value=self.lineage)
        )
        self.validate_data = self.stack.enter_context(
            patch.object(contract.data, "validate", return_value=self.dataset)
        )

    def test_source_binds_relocated_reader_and_historical_checkout_without_loading(
        self,
    ):
        with (
            patch.object(
                contract.previous,
                "validate_preparation",
                side_effect=AssertionError("obsolete root validation"),
            ),
            patch.object(
                contract,
                "load_source_model",
                side_effect=AssertionError("no fitted model loading"),
            ),
        ):
            source = contract._source_contract(self.source_root)
        self.assertEqual(source["root"], str(self.source_root))
        self.assertEqual(source["reader"]["cid"], cid_bytes(self.reader_bytes))
        self.assertEqual(source["core"], self.core)
        self.assertEqual(len(source["public_documents"]), 4)
        self.validate_data.assert_called_once_with(
            self.source_root / "data", inspect_development=False
        )

    def test_reader_source_and_resealed_local_evidence_tampering_fail_closed(self):
        reader = self.source_root / "fit/reader.safetensors"
        reader.write_bytes(self.reader_bytes + b"changed")
        with self.assertRaisesRegex(ValueError, "reader.safetensors"):
            contract._source_contract(self.source_root)
        reader.write_bytes(self.reader_bytes)
        (self.repo / "historical.py").write_text("changed source\n")
        with self.assertRaisesRegex(ValueError, "historical #1077 implementation"):
            contract._source_contract(self.source_root)
        (self.repo / "historical.py").write_text("frozen historical source\n")
        changed = {**self.documents["replay"], "exact_replay": False}
        (self.source_root / "replay.json").write_bytes(
            canonical_json_bytes(_envelope(changed, "replay_cid"))
        )
        with self.assertRaisesRegex(ValueError, "frozen predecessor"):
            contract._source_contract(self.source_root)

    def test_both_opportunities_count_actual_matrices_and_ignore_padding(self):
        from r4_softmax_trainer.zoology_language_r4 import attention

        matrices = torch.eye(4, dtype=torch.float64).repeat(120, 1, 1)
        matrices[1] = torch.diag(
            torch.tensor([-1.0, -1.0, 1.0, 1.0], dtype=torch.float64)
        )
        frames = types.SimpleNamespace(frame_matrices=matrices, identity_index=0)
        tokens = torch.zeros((2, 5, 3), dtype=torch.long)
        tokens[0, 0, 1], tokens[1, 0, 1] = 1, 2
        tokens[:, :, 2] = 1  # Distinct padding frames are never counted.
        ends = torch.tensor([[0, 1, 0, 0, 0], [0, 2, 0, 0, 0]])
        inputs = torch.zeros_like(tokens)
        lengths = torch.full((2, 5), 2, dtype=torch.long)
        with patch.object(attention, "frame_assignment", return_value=(tokens, ends)):
            view = contract._frame_view(
                inputs, lengths, torch.tensor([True, True]), frames
            )
            self.assertEqual(view["valid_tokens"], 20)
            for control in view["controls"].values():
                self.assertEqual(control["source_frame_matrices_changed"], 2)
                self.assertEqual(control["supported_rows_with_changed_source_frame"], 1)
                self.assertEqual(control["supported_loss_reachability_ceiling"], 0.5)
                self.assertTrue(control["passed"])
            self.assertEqual(
                view["controls"][contract.CONTROLS[0]][
                    "source_frame_positions_changed"
                ],
                20,
            )
            self.assertEqual(
                view["controls"][contract.CONTROLS[1]][
                    "source_frame_positions_changed"
                ],
                8,
            )
            missed = contract._frame_view(
                inputs, lengths, torch.tensor([False, True]), frames
            )
            self.assertFalse(any(v["passed"] for v in missed["controls"].values()))
        relocated = {
            "root": "/relocated/frames",
            "tree_cid": contract.previous.FRAME_TREE_CID,
        }
        with patch.object(contract.prior, "_frame_contract", return_value=relocated):
            self.assertEqual(
                contract._frames(
                    Path(relocated["root"]),
                    {"frame_tree_cid": contract.previous.FRAME_TREE_CID},
                ),
                relocated,
            )
        with (
            patch.object(
                contract.prior,
                "_frame_contract",
                return_value={**relocated, "tree_cid": "changed"},
            ),
            self.assertRaisesRegex(ValueError, "native frame bundle"),
        ):
            contract._frames(
                Path(relocated["root"]),
                {"frame_tree_cid": contract.previous.FRAME_TREE_CID},
            )

    def test_exclusive_preparation_and_frozen_policy_preflight_binding(self):
        output, frames = self.repo / "experiment", self.repo / "frames"
        binding = {
            "source": {"root": str(self.source_root)},
            "frames": {"root": str(frames)},
            "implementation": {"tree_cid": "fixed-source"},
            "preflight": {"passed": True},
        }
        with patch.object(contract, "_bindings", return_value=binding) as bound:
            prepared = contract.prepare(output, self.source_root, frames)
            self.assertEqual(contract.validate_preparation(output), prepared)
            with self.assertRaises(FileExistsError):
                contract.prepare(output, self.source_root, frames)
            self.assertEqual(bound.call_count, 2)
            for key in ("evaluation", "intervention", "preflight", "implementation"):
                with self.subTest(field=key):
                    changed = copy.deepcopy(prepared)
                    changed[key]["tampered"] = True
                    (output / "preparation.json").write_bytes(
                        canonical_json_bytes(_envelope(changed, "preparation_cid"))
                    )
                    with self.assertRaises(ValueError):
                        contract.validate_preparation(output)
        interrupted = self.repo / "interrupted"
        with patch.object(
            contract, "_bindings", side_effect=ValueError("preflight opportunity miss")
        ) as bound:
            with self.assertRaisesRegex(ValueError, "opportunity miss"):
                contract.prepare(interrupted, self.source_root, frames)
            with self.assertRaises(FileExistsError):
                contract.prepare(interrupted, self.source_root, frames)
            self.assertEqual(bound.call_count, 1)


if __name__ == "__main__":
    unittest.main()
