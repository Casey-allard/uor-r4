from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch
from r4_softmax_trainer.provenance import cid_bytes
from r4_softmax_trainer.role_tagged_associative_data import derive_mqar_role_ids
from r4_softmax_trainer.zoology_control import data as exact_data
from r4_softmax_trainer.zoology_release import development as release
from r4_softmax_trainer.zoology_transfer import contract


def _row(index: int) -> exact_data.ZoologyMQARRow:
    inputs = [2] * 120
    keys = tuple(256 + index * 16 + offset for offset in range(8))
    values = tuple(2_048 + index * 16 + offset for offset in range(8))
    for offset, (key, value) in enumerate(zip(keys, values, strict=True)):
        inputs[offset * 4] = key
        inputs[offset * 4 + 1] = value
    positions = tuple(32 + offset * 8 for offset in range(8))
    for position, key in zip(positions, reversed(keys), strict=True):
        inputs[position] = key
    return exact_data.ZoologyMQARRow(
        input_ids=tuple(inputs),
        selected_positions=positions,
        targets=tuple(reversed(values)),
        stable_id=cid_bytes(f"row-{index}".encode()),
        role_ids=derive_mqar_role_ids(inputs),
    )


def _population() -> exact_data.ZoologyMQARPopulation:
    return exact_data._make_population(
        train=(_row(0), _row(1)),
        development=(_row(2),),
        name="exact_1045_open_bytes",
        vocab_size=4096,
        input_seq_len=120,
        num_kv_pairs=8,
        source_split_cid=contract.control_development.EXPECTED_1045_SPLIT_CID,
    )


class ZoologyTransferContractTests(unittest.TestCase):
    def test_implementation_binds_transitive_source_and_environment_mutations(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            files = {
                "src/r4_softmax_trainer/__init__.py": "",
                "src/r4_softmax_trainer/zoology_transfer/__init__.py": "from .contract import value\n",
                "src/r4_softmax_trainer/zoology_transfer/contract.py": "from ..adapter import value\n",
                "src/r4_softmax_trainer/adapter.py": "from .dependency import value\n",
                "src/r4_softmax_trainer/dependency.py": "value = 1\n",
                "src/r4_softmax_trainer/zoology_control/NOTICE.md": "notice\n",
                "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md": "license\n",
                "tests/test_zoology_transfer_contract.py": "# declared check\n",
                "pyproject.toml": "# environment\n",
                "uv.lock": "# locked environment\n",
            }
            for relative, contents in files.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(contents)
            baseline = contract.implementation_contract(root)
            bound = {record["path"] for record in baseline["files"]}
            self.assertEqual(bound, set(files))
            for relative in (
                "src/r4_softmax_trainer/dependency.py",
                "tests/test_zoology_transfer_contract.py",
                "pyproject.toml",
                "uv.lock",
            ):
                path = root / relative
                path.write_text(files[relative] + "# mutation\n")
                self.assertNotEqual(
                    baseline["implementation_cid"],
                    contract.implementation_contract(root)["implementation_cid"],
                )
                path.write_text(files[relative])
            self.assertEqual(baseline, contract.implementation_contract(root))

    def test_primary_and_control_preserve_the_exact_three_tensor_model_abi(
        self,
    ) -> None:
        population = _population()
        primary, control = contract._population_tensors(population)
        self.assertEqual(
            set(primary),
            {
                f"{split}_{field}"
                for split in ("train", "test")
                for field in ("inputs", "positions", "targets")
            },
        )
        self.assertEqual(
            set(control), {"test_inputs", "test_positions", "test_targets"}
        )
        self.assertEqual(
            primary["train_inputs"].tolist(),
            [list(row.input_ids) for row in population.train],
        )
        self.assertEqual(
            primary["test_inputs"].tolist(),
            [list(row.input_ids) for row in population.development],
        )
        torch.testing.assert_close(primary["test_positions"], control["test_positions"])
        torch.testing.assert_close(primary["test_targets"], control["test_targets"])
        changed = torch.nonzero(primary["test_inputs"] != control["test_inputs"])
        self.assertEqual(changed[:, 1].tolist(), list(range(1, 32, 4)))
        values = primary["test_inputs"][0, 1:32:4]
        self.assertEqual(
            control["test_inputs"][0, 1:32:4].tolist(), values.roll(-1).tolist()
        )

    def test_bound_dataset_and_control_loads_remain_separate_and_detect_changes(
        self,
    ) -> None:
        primary, control = contract._population_tensors(_population())
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(contract, "TRAIN_ROWS", 2),
            patch.object(contract, "TEST_ROWS", 1),
        ):
            root = Path(directory)
            preparation = {}
            for key, tensors, relative in (
                ("dataset", primary, contract.DATA_RELATIVE_PATH),
                ("control", control, contract.CONTROL_RELATIVE_PATH),
            ):
                payload = release._canonical_safetensors(tensors)
                release._write_exclusive(root / relative, payload)
                preparation[key] = contract._tensor_record(tensors, payload, relative)
            loaded = contract.load_dataset(root, preparation)
            self.assertEqual(set(loaded), set(primary))
            self.assertEqual(
                set(contract.load_control(root, preparation)), set(control)
            )
            control_path = root / contract.CONTROL_RELATIVE_PATH
            control_path.write_bytes(b"invalid control")
            # Even a corrupt control is not opened on the primary path.
            contract.load_dataset(root, preparation)
            with self.assertRaisesRegex(ValueError, "file bytes/CID"):
                contract.load_control(root, preparation)
            (root / contract.DATA_RELATIVE_PATH).write_bytes(b"invalid primary")
            with self.assertRaisesRegex(ValueError, "file bytes/CID"):
                contract.load_dataset(root, preparation)

    def test_shape_validation_rejects_roles_and_out_of_range_queries(self) -> None:
        primary, _ = contract._population_tensors(_population())
        with (
            patch.object(contract, "TRAIN_ROWS", 2),
            patch.object(contract, "TEST_ROWS", 1),
        ):
            contract._validate_shapes(primary, control=False)
            with self.assertRaisesRegex(ValueError, "model ABI"):
                contract._validate_shapes(
                    {**primary, "train_roles": torch.zeros(2, 120)}, control=False
                )
            bad = {**primary, "test_positions": primary["test_positions"].clone()}
            bad["test_positions"][0, -1] = 120
            with self.assertRaisesRegex(ValueError, "outside its domain"):
                contract._validate_shapes(bad, control=False)

    def test_narrow_loader_requests_only_the_open_mqar_payload(self) -> None:
        population = _population()
        source_root = Path("unused-open-source")
        split = SimpleNamespace(
            train=population.train,
            development=population.development,
            controls=(),
            split_cid=population.source_split_cid,
        )
        manifest = {"manifest_cid": contract.EXPECTED_1043_MANIFEST_CID}
        commitment = {"commitment_cid": contract.EXPECTED_1043_COMMITMENT_CID}
        with (
            patch.object(
                contract.position_data,
                "_validate_public_envelopes",
                return_value=(manifest, commitment),
            ),
            patch.object(contract, "verify_artifact_subset") as verify_subset,
            patch.object(
                release,
                "_file_record",
                return_value={"file_cid": contract.EXPECTED_1043_MQAR_CID},
            ),
            patch.object(
                contract.position_data, "_load_examples_payload", return_value=()
            ) as load_examples,
            patch.object(
                contract.role_data, "split_mqar_construction", return_value=split
            ),
            patch.object(exact_data, "_adapt_exact_row", side_effect=lambda row: row),
            patch.object(
                exact_data,
                "load_role_tagged_construction",
                side_effect=AssertionError("broad loader called"),
            ),
            patch.object(
                contract, "EXPECTED_1045_POPULATION_CID", population.population_cid
            ),
        ):
            loaded, record = contract._load_exact_population_narrow(source_root)
        self.assertEqual(loaded.population_cid, population.population_cid)
        verify_subset.assert_called_once_with(
            manifest,
            artifact_root=source_root,
            relative_paths=(contract.position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,),
        )
        load_examples.assert_called_once_with(
            source_root / contract.position_data.CONSTRUCTION_MQAR_RELATIVE_PATH,
            population="mqar",
            split="construction",
        )
        self.assertEqual(record["sealed_payload_access"], "FORBIDDEN_NOT_READ")
        self.assertNotIn("construction/english.json", record["files_read"])

    def test_prepare_validation_never_opens_the_control(self) -> None:
        primary, _ = contract._population_tensors(_population())
        implementation = {"implementation_cid": cid_bytes(b"implementation")}
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(contract, "TRAIN_ROWS", 2),
            patch.object(contract, "TEST_ROWS", 1),
            patch.object(
                contract, "implementation_contract", return_value=implementation
            ),
        ):
            root = Path(directory)
            payload = release._canonical_safetensors(primary)
            release._write_exclusive(root / contract.DATA_RELATIVE_PATH, payload)
            preparation = release._with_cid(
                {
                    "schema": contract.PREPARATION_SCHEMA,
                    "issue": 1053,
                    "policy": contract.POLICY,
                    "implementation": implementation,
                    "training_contract": contract.training_contract(),
                    "dataset": contract._tensor_record(
                        primary, payload, contract.DATA_RELATIVE_PATH
                    ),
                    "control": {"path": "missing-control"},
                    "source_split_cid": contract.control_development.EXPECTED_1045_SPLIT_CID,
                    "source_population_cid": contract.EXPECTED_1045_POPULATION_CID,
                    "release_1050": {"result_cid": contract.EXPECTED_1050_RESULT_CID},
                    "predecessor_1045": {
                        "result_cid": contract.control_development.EXPECTED_1045_RESULT_CID
                    },
                },
                "preparation_cid",
            )
            release._write_exclusive_json(
                root / contract.PREPARATION_RELATIVE_PATH, preparation
            )
            self.assertEqual(contract.validate_preparation(root), preparation)


if __name__ == "__main__":
    unittest.main()
