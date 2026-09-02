from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.zoology_clock import contract


class ZoologyClockContractTests(unittest.TestCase):
    def test_clock_contract_keeps_cell_and_matches_3920_updates(self) -> None:
        inherited = contract.previous.training_contract()
        current = contract.training_contract()
        for key in (
            "vocab_size",
            "context",
            "query_positions_per_row",
            "train_rows",
            "development_rows",
            "d_model",
            "n_layers",
            "num_heads",
            "seed",
            "learning_rate",
            "learning_rate_float_hex",
            "batch_size",
            "optimizer",
            "weight_decay",
            "betas",
            "epsilon",
            "strict_early_stop",
            "scheduler_step",
        ):
            self.assertEqual(current[key], inherited[key], key)
        self.assertNotIn("maximum_epochs", current)
        self.assertEqual(current["maximum_source_blocks"], 20)
        self.assertEqual(current["updates_per_source_block"], 196)
        self.assertEqual(current["maximum_optimizer_updates"], 3920)
        self.assertEqual(current["maximum_train_query_presentations"], 16_056_320)
        self.assertEqual(current["full_run_complete_training_permutations"], 245)
        self.assertIn("64_source_blocks", current["scheduler"])

    def test_new_sources_and_tests_are_bound_and_unbound_local_import_is_refused(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {
                "src/r4_softmax_trainer/__init__.py": "",
                "src/r4_softmax_trainer/inherited.py": "value = 1\n",
                "pyproject.toml": "# project\n",
                "uv.lock": "# locked\n",
                "src/r4_softmax_trainer/zoology_clock/__init__.py": "",
                "src/r4_softmax_trainer/zoology_clock/contract.py": "from ..inherited import value\n",
                "tests/test_zoology_clock_contract.py": "# declared test\n",
            }
            for relative, contents in files.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents)
            inherited_paths = tuple(list(files)[:4])
            inherited = {
                "implementation_cid": contract.EXPECTED_IMPLEMENTATION_CID,
                "files": [
                    {
                        "path": path,
                        "bytes": len(files[path].encode()),
                        "cid": cid_bytes(files[path].encode()),
                    }
                    for path in inherited_paths
                ],
            }
            with patch.object(
                contract.previous, "implementation_contract", return_value=inherited
            ):
                initial = contract.implementation_contract(root)
                self.assertEqual(
                    {record["path"] for record in initial["files"]}, set(files)
                )
                self.assertEqual(initial["inherited_file_count"], 4)
                self.assertEqual(initial["new_file_count"], 3)
                test = root / "tests/test_zoology_clock_contract.py"
                test.write_text("# changed declared test\n")
                self.assertNotEqual(
                    initial["implementation_cid"],
                    contract.implementation_contract(root)["implementation_cid"],
                )
                foreign = root / "src/r4_softmax_trainer/foreign.py"
                foreign.write_text("value = 2\n")
                clock_source = root / "src/r4_softmax_trainer/zoology_clock/contract.py"
                clock_source.write_text("from ..foreign import value\n")
                with self.assertRaisesRegex(
                    ValueError, "outside its bound source closure"
                ):
                    contract.implementation_contract(root)
                clock_source.write_text("__import__('r4_softmax_trainer.foreign')\n")
                with self.assertRaisesRegex(ValueError, "dynamic imports"):
                    contract.implementation_contract(root)
            with (
                patch.object(
                    contract.previous,
                    "implementation_contract",
                    return_value={
                        **inherited,
                        "implementation_cid": cid_bytes(b"changed lock"),
                    },
                ),
                self.assertRaisesRegex(
                    ValueError, "implementation or lockfile changed"
                ),
            ):
                contract.implementation_contract(root)

    def test_prepare_writes_only_json_and_validation_never_opens_control(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "clock"
            predecessor = Path(directory) / "previous"
            predecessor.mkdir()
            bound = {
                "dataset": {
                    "path": "data/exact-1045.safetensors",
                    "file_cid": cid_bytes(b"unchanged primary"),
                },
                "control": {
                    "path": "data/binding-permuted.safetensors",
                    "file_cid": cid_bytes(b"unchanged control"),
                },
                "predecessor_root": str(predecessor),
                "predecessor_preparation_cid": contract.EXPECTED_PREPARATION_CID,
                "predecessor_preflight_cid": contract.EXPECTED_PREFLIGHT_CID,
                "predecessor_result_cid": contract.EXPECTED_RESULT_CID,
                "reused_c0": {"passed": True, "c0_cid": contract.EXPECTED_C0_CID},
                "source_1050_result_cid": contract.previous.EXPECTED_1050_RESULT_CID,
            }
            implementation = {"implementation_cid": cid_bytes(b"clock implementation")}
            with (
                patch.object(contract, "_load_predecessor", return_value=bound),
                patch.object(
                    contract, "implementation_contract", return_value=implementation
                ),
                patch.object(
                    contract.previous,
                    "load_control",
                    side_effect=AssertionError("control opened"),
                ),
            ):
                preparation = contract.prepare(root, predecessor)
                self.assertEqual(contract.validate_preparation(root), preparation)
                with self.assertRaises(FileExistsError):
                    contract.prepare(root, predecessor)
            self.assertEqual(
                [str(path.relative_to(root)) for path in root.rglob("*")],
                [contract.PREPARATION_PATH],
            )
            self.assertEqual(preparation["dataset"], bound["dataset"])
            self.assertEqual(preparation["control"], bound["control"])
            self.assertEqual(preparation["read_ledger"]["predecessor_weight_reads"], 0)
            self.assertEqual(preparation["read_ledger"]["c0_training_updates"], 0)

    def test_tensor_loaders_delegate_to_existing_predecessor_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            predecessor = Path(directory)
            preparation = {"predecessor_root": str(predecessor)}
            tensors = {"test_inputs": torch.tensor([[1]])}
            with (
                patch.object(
                    contract.previous, "load_dataset", return_value=tensors
                ) as primary,
                patch.object(
                    contract.previous, "load_control", return_value=tensors
                ) as control,
            ):
                self.assertIs(
                    contract.load_dataset(predecessor / "unused-new-root", preparation),
                    tensors,
                )
                primary.assert_called_once_with(predecessor, preparation)
                control.assert_not_called()
                self.assertIs(
                    contract.load_control(predecessor / "unused-new-root", preparation),
                    tensors,
                )
                control.assert_called_once_with(predecessor, preparation)


if __name__ == "__main__":
    unittest.main()
