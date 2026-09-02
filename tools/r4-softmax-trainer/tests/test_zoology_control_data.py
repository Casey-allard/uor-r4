from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import torch

from r4_softmax_trainer.provenance import cid_bytes, cid_file
from r4_softmax_trainer.role_tagged_associative_data import derive_mqar_role_ids
from r4_softmax_trainer.zoology_control import data
from r4_softmax_trainer.zoology_control.data import (
    ZoologyMQARRow,
    batch_rows,
    deterministic_epoch_order,
    load_exact_1045_population,
    permute_exact_bindings,
)
from r4_softmax_trainer.zoology_control.provenance import (
    ZOOLOGY_RELEASE_LICENSE_CID,
    ZOOLOGY_RELEASE_REVISION,
    zoology_control_implementation_contract,
    zoology_source_attribution,
)


def _cid(label: str) -> str:
    return cid_bytes(label.encode("ascii"))


def _exact_tagged_row(index: int) -> SimpleNamespace:
    inputs = [2] * data.EXACT_1045_CONTEXT
    keys = tuple(256 + index * 16 + offset for offset in range(8))
    values = tuple(2_048 + index * 16 + offset for offset in range(8))
    for record_index, (key, value) in enumerate(zip(keys, values, strict=True)):
        inputs[record_index * 4] = key
        inputs[record_index * 4 + 1] = value
    positions = tuple(32 + offset * 8 for offset in range(8))
    for position, key in zip(positions, reversed(keys), strict=True):
        inputs[position] = key
    mapping = dict(zip(keys, values, strict=True))
    answers = tuple(mapping[inputs[position]] for position in positions)
    labels = [-100] * len(inputs)
    for position, answer in zip(positions, answers, strict=True):
        labels[position] = answer
    input_ids = tuple(inputs)
    return SimpleNamespace(
        input_ids=input_ids,
        role_ids=derive_mqar_role_ids(input_ids),
        label_ids=tuple(labels),
        stable_id=_cid(f"exact-row-{index}"),
        source=SimpleNamespace(
            query_positions=positions,
            query_keys=tuple(input_ids[position] for position in positions),
            answers=answers,
        ),
    )


class ZoologyControlDataTests(unittest.TestCase):
    def test_released_mqar_zero_filler_integer_golden(self) -> None:
        inputs, labels = data._released_mqar(
            vocab_size=32,
            num_examples=3,
            input_seq_len=16,
            seed=0,
            num_kv_pairs=2,
        )
        self.assertEqual(inputs.dtype, torch.long)
        self.assertEqual(labels.dtype, torch.long)
        self.assertEqual(
            inputs.tolist(),
            [
                [2, 29, 7, 22, 2, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [3, 21, 5, 16, 3, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                [5, 17, 13, 18, 13, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0],
            ],
        )
        self.assertEqual(
            labels.tolist(),
            [
                [
                    -100,
                    -100,
                    -100,
                    -100,
                    29,
                    -100,
                    22,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                ],
                [
                    -100,
                    -100,
                    -100,
                    -100,
                    21,
                    -100,
                    16,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                ],
                [
                    -100,
                    -100,
                    -100,
                    -100,
                    18,
                    -100,
                    -100,
                    -100,
                    17,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                    -100,
                ],
            ],
        )

    def test_source_calibration_declares_scaled_release_parameters(self) -> None:
        calls: list[tuple[str, int, int]] = []

        def fake_source_rows(*, split: str, count: int, seed: int):
            calls.append((split, count, seed))
            inputs = tuple([1] * data.SOURCE_NATIVE_INPUT_SEQ_LEN)
            positions = tuple(range(8, 16, 2))
            return (
                ZoologyMQARRow(
                    input_ids=inputs,
                    selected_positions=positions,
                    targets=(4_096, 4_097, 4_098, 4_099),
                    stable_id=_cid(split),
                ),
            )

        with patch.object(data, "_source_rows", fake_source_rows):
            population = data.build_source_calibration()
        self.assertEqual(
            calls,
            [
                ("train", 8_192, 0),
                ("development", 1_024, 10),
            ],
        )
        self.assertEqual(population.name, "scaled_source_native")
        self.assertEqual(population.vocab_size, 8_192)
        self.assertEqual(population.input_seq_len, 64)
        self.assertEqual(population.num_kv_pairs, 4)
        self.assertEqual(population.train_seed, 0)
        self.assertEqual(population.development_seed, 10)

    def test_exact_1045_adapter_keeps_bytes_and_roles_out_of_batch(self) -> None:
        train = (_exact_tagged_row(0), _exact_tagged_row(1))
        development = (_exact_tagged_row(2),)
        construction = SimpleNamespace(
            mqar_train=train,
            mqar_development=development,
            split_cid=_cid("1045-split"),
        )
        with (
            patch.object(data, "EXACT_1045_TRAIN_ROWS", len(train)),
            patch.object(data, "EXACT_1045_DEVELOPMENT_ROWS", len(development)),
            patch.object(
                data,
                "load_role_tagged_construction",
                return_value=construction,
            ),
        ):
            population = load_exact_1045_population(Path("unused"))
        self.assertEqual(population.source_split_cid, construction.split_cid)
        self.assertEqual(
            tuple(row.input_ids for row in population.train),
            tuple(row.input_ids for row in train),
        )
        self.assertEqual(
            tuple(row.selected_positions for row in population.development),
            tuple(row.source.query_positions for row in development),
        )
        self.assertEqual(
            tuple(row.targets for row in population.development),
            tuple(row.source.answers for row in development),
        )

        batch = batch_rows(population.train)
        self.assertEqual(
            tuple(batch.__dataclass_fields__),
            ("input_ids", "selected_positions", "targets"),
        )
        self.assertFalse(hasattr(batch, "role_ids"))
        self.assertTrue(torch.equal(batch.input_ids[0], torch.tensor(train[0].input_ids)))

    def test_exact_adapter_rejects_role_byte_drift(self) -> None:
        native = _exact_tagged_row(0)
        corrupted = SimpleNamespace(
            **{
                **native.__dict__,
                "role_ids": (0,) + native.role_ids[1:],
            }
        )
        construction = SimpleNamespace(
            mqar_train=(corrupted,),
            mqar_development=(_exact_tagged_row(1),),
            split_cid=_cid("1045-split-corrupt"),
        )
        with (
            patch.object(data, "EXACT_1045_TRAIN_ROWS", 1),
            patch.object(data, "EXACT_1045_DEVELOPMENT_ROWS", 1),
            patch.object(
                data,
                "load_role_tagged_construction",
                return_value=construction,
            ),
            self.assertRaisesRegex(ValueError, "role bytes"),
        ):
            load_exact_1045_population(Path("unused"))

    def test_binding_permutation_changes_only_physical_values_and_identity(self) -> None:
        native = data._adapt_exact_row(_exact_tagged_row(0))
        (control,) = permute_exact_bindings((native,))
        value_positions = tuple(range(1, 32, 4))
        native_values = tuple(native.input_ids[position] for position in value_positions)
        self.assertEqual(
            tuple(control.input_ids[position] for position in value_positions),
            native_values[1:] + native_values[:1],
        )
        self.assertTrue(
            all(
                control.input_ids[position] == native.input_ids[position]
                for position in range(len(native.input_ids))
                if position not in value_positions
            )
        )
        self.assertEqual(control.selected_positions, native.selected_positions)
        self.assertEqual(control.targets, native.targets)
        self.assertEqual(control.role_ids, native.role_ids)
        self.assertNotEqual(control.stable_id, native.stable_id)
        self.assertFalse(hasattr(batch_rows((control,)), "role_ids"))

    def test_release_style_shuffle_is_replayable_and_data_bound(self) -> None:
        rows = tuple(
            ZoologyMQARRow(
                input_ids=(index, 0, 0, 0),
                selected_positions=(0,),
                targets=(index,),
                stable_id=_cid(f"shuffle-{index}"),
            )
            for index in range(12)
        )
        first = deterministic_epoch_order(rows, 3, "train")
        replay = deterministic_epoch_order(rows, 3, "train")
        next_epoch = deterministic_epoch_order(rows, 4, "train")
        self.assertEqual(first, replay)
        self.assertEqual(set(first), set(rows))
        self.assertNotEqual(first, next_epoch)

    def test_source_attribution_and_recursive_tree_binding(self) -> None:
        attribution = zoology_source_attribution()
        self.assertEqual(
            attribution["release_oracle"]["revision"],
            ZOOLOGY_RELEASE_REVISION,
        )
        files = {
            record["path"]: record
            for record in attribution["release_oracle"]["files"]
        }
        self.assertEqual(
            set(files),
            {
                "zoology/data/associative_recall.py",
                "zoology/data/utils.py",
                "zoology/mixers/attention.py",
                "zoology/model.py",
                "zoology/train.py",
                "zoology/experiments/paper/figure2.py",
                "LICENSE.md",
            },
        )
        local_license = Path(data.__file__).with_name("LICENSE-APACHE-2.0.md")
        local_license_bytes = local_license.read_bytes()
        self.assertTrue(local_license_bytes.endswith(b"\n"))
        self.assertEqual(
            cid_bytes(local_license_bytes[:-1]),
            ZOOLOGY_RELEASE_LICENSE_CID,
        )
        live_contract = zoology_control_implementation_contract()
        live_files = {
            record["path"]: record for record in live_contract["files"]
        }
        local_license_path = (
            "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md"
        )
        self.assertEqual(live_files[local_license_path]["cid"], cid_file(local_license))

        with tempfile.TemporaryDirectory() as directory:
            trainer_root = Path(directory)
            package = (
                trainer_root / "src" / "r4_softmax_trainer" / "zoology_control"
            )
            nested = package / "nested"
            nested.mkdir(parents=True)
            (package / "data.py").write_text("VALUE = 1\n", encoding="utf-8")
            (nested / "helper.py").write_text("VALUE = 2\n", encoding="utf-8")
            (package / "NOTICE.md").write_text("notice\n", encoding="utf-8")
            (package / "LICENSE-APACHE-2.0.md").write_text(
                "license\n", encoding="utf-8"
            )
            test_dir = trainer_root / "tests" / "nested"
            test_dir.mkdir(parents=True)
            (test_dir / "test_zoology_control_nested.py").write_text(
                "def test_placeholder(): pass\n", encoding="utf-8"
            )
            (trainer_root / "pyproject.toml").write_text(
                "[project]\n", encoding="utf-8"
            )
            (trainer_root / "uv.lock").write_text("version = 1\n", encoding="utf-8")

            first = zoology_control_implementation_contract(trainer_root)
            paths = {record["path"] for record in first["files"]}
            self.assertIn(
                "src/r4_softmax_trainer/zoology_control/nested/helper.py",
                paths,
            )
            self.assertIn("tests/nested/test_zoology_control_nested.py", paths)
            self.assertIn(
                "src/r4_softmax_trainer/zoology_control/NOTICE.md",
                paths,
            )
            self.assertIn(
                "src/r4_softmax_trainer/zoology_control/LICENSE-APACHE-2.0.md",
                paths,
            )
            (nested / "helper.py").write_text("VALUE = 3\n", encoding="utf-8")
            second = zoology_control_implementation_contract(trainer_root)
            self.assertNotEqual(first["tree_cid"], second["tree_cid"])
            self.assertNotEqual(
                first["implementation_cid"], second["implementation_cid"]
            )


if __name__ == "__main__":
    unittest.main()
