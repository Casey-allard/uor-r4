"""Focused checks for #973's frozen nonsealed language-path data root."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import numpy as np
import torch

from r4_softmax_trainer import language_path_generalization_data as subject
from r4_softmax_trainer.provenance import cid_bytes


def _u16(values: list[int]) -> bytes:
    return np.asarray(values, dtype="<u2").tobytes()


class LanguagePathWindowStoreTests(unittest.TestCase):
    def test_store_is_read_only_and_returns_causal_pairs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tokens.u16"
            path.write_bytes(_u16(list(range(12))))
            with mock.patch.multiple(subject, WINDOW_TOKENS=4, CONTEXT=3):
                store = subject.LanguagePathWindowStore(path, window_count=3)
                self.assertEqual(store.windows.shape, (3, 4))
                self.assertFalse(store.windows.flags.writeable)
                self.assertEqual(store.window(1).tolist(), [4, 5, 6, 7])
                with self.assertRaises(ValueError):
                    store.windows[0, 0] = 99
                inputs, targets = store.batch([2, 0])
                self.assertEqual(inputs.dtype, torch.long)
                self.assertEqual(inputs.tolist(), [[8, 9, 10], [0, 1, 2]])
                self.assertEqual(targets.tolist(), [[9, 10, 11], [1, 2, 3]])
                batches = list(store.batches([2, 0, 1], batch_size=2))
                self.assertEqual([tuple(batch[0].shape) for batch in batches], [(2, 3), (1, 3)])

    def test_store_rejects_wrong_size_and_out_of_range_ordinal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "tokens.u16"
            path.write_bytes(_u16(list(range(11))))
            with (
                mock.patch.multiple(subject, WINDOW_TOKENS=4, CONTEXT=3),
                self.assertRaisesRegex(ValueError, "expected 24"),
            ):
                subject.LanguagePathWindowStore(path, window_count=3)
            path.write_bytes(_u16(list(range(12))))
            with mock.patch.multiple(subject, WINDOW_TOKENS=4, CONTEXT=3):
                store = subject.LanguagePathWindowStore(path, window_count=3)
                with self.assertRaises(IndexError):
                    store.window(3)


class LanguagePathOrderTests(unittest.TestCase):
    def test_order_is_a_stable_one_pass_blake3_permutation(self) -> None:
        expected = (3, 1, 4, 7, 5, 2, 0, 6)
        self.assertEqual(subject.deterministic_window_order(8), expected)
        self.assertEqual(subject.deterministic_window_order(8), expected)
        self.assertEqual(set(expected), set(range(8)))
        self.assertNotEqual(
            subject.deterministic_window_order(8, seed=9_739),
            expected,
        )


class LanguagePathPreparationTests(unittest.TestCase):
    def _tiny_contract(self, root: Path) -> tuple[dict[str, object], dict[str, object]]:
        source = root / "source"
        train_values = list(range(24))
        validation_values = list(range(100, 112))
        tokenizer_bytes = b'{"tiny":"tokenizer"}\n'
        geometry_bytes = b'{"tiny":"exact-geometry"}\n'
        train_store = _u16(train_values)
        validation_store = _u16(validation_values)
        train_slice = _u16(train_values[3:15])
        validation_slice = _u16(validation_values[:6])
        for relative, value in {
            subject.SOURCE_TRAIN_RELATIVE_PATH: train_store,
            subject.SOURCE_VALIDATION_RELATIVE_PATH: validation_store,
            subject.SOURCE_TOKENIZER_RELATIVE_PATH: tokenizer_bytes,
        }.items():
            path = source / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(value)
        geometry_path = root / "explicit-geometry.json"
        geometry_path.write_bytes(geometry_bytes)

        identities: dict[str, object] = {
            "WINDOW_TOKENS": 3,
            "CONTEXT": 2,
            "TRAIN_SOURCE_OFFSET_TOKENS": 3,
            "TRAIN_TOKENS": 12,
            "TRAIN_WINDOWS": 4,
            "TRAIN_DECISIONS": 8,
            "VALIDATION_SOURCE_OFFSET_TOKENS": 0,
            "VALIDATION_TOKENS": 6,
            "VALIDATION_WINDOWS": 2,
            "VALIDATION_DECISIONS": 4,
            "SOURCE_TRAIN_TOKENS": 24,
            "SOURCE_VALIDATION_TOKENS": 12,
            "EXPECTED_TRAIN_SLICE_CID": cid_bytes(train_slice),
            "EXPECTED_VALIDATION_SLICE_CID": cid_bytes(validation_slice),
            "EXPECTED_SOURCE_TRAIN_STORE_CID": cid_bytes(train_store),
            "EXPECTED_SOURCE_VALIDATION_STORE_CID": cid_bytes(validation_store),
            "EXPECTED_TOKENIZER_CID": cid_bytes(tokenizer_bytes),
            "EXPECTED_GEOMETRY_FILE_CID": cid_bytes(geometry_bytes),
            "EXPECTED_GEOMETRY_ARTIFACT_CID": "blake3:" + "a" * 64,
            "EXPECTED_SOURCE_TRAINING_VIEW_CID": "blake3:" + "b" * 64,
            "EXPECTED_SOURCE_DATASET_MANIFEST_CID": "blake3:" + "c" * 64,
            "EXPECTED_SOURCE_SPLIT_POLICY_CID": "blake3:" + "d" * 64,
        }
        manifest: dict[str, object] = {
            "manifest_cid": identities["EXPECTED_SOURCE_TRAINING_VIEW_CID"],
            "dataset_manifest_cid": identities[
                "EXPECTED_SOURCE_DATASET_MANIFEST_CID"
            ],
            "split_policy_cid": identities["EXPECTED_SOURCE_SPLIT_POLICY_CID"],
            "tokenizer_cid": identities["EXPECTED_TOKENIZER_CID"],
            "artifacts": [
                {
                    "path": subject.SOURCE_TRAIN_RELATIVE_PATH,
                    "bytes": len(train_store),
                    "cid": identities["EXPECTED_SOURCE_TRAIN_STORE_CID"],
                },
                {
                    "path": subject.SOURCE_VALIDATION_RELATIVE_PATH,
                    "bytes": len(validation_store),
                    "cid": identities[
                        "EXPECTED_SOURCE_VALIDATION_STORE_CID"
                    ],
                },
                {
                    "path": subject.SOURCE_TOKENIZER_RELATIVE_PATH,
                    "bytes": len(tokenizer_bytes),
                    "cid": identities["EXPECTED_TOKENIZER_CID"],
                },
            ],
        }
        return (
            {
                "source": source,
                "geometry_path": geometry_path,
                "train_slice": train_slice,
                "validation_slice": validation_slice,
                "manifest": manifest,
            },
            identities,
        )

    def test_prepare_copies_only_frozen_inputs_and_load_is_source_independent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture, identities = self._tiny_contract(root)
            output = root / "new-root"
            source_calls: list[Path] = []

            def source_loader(path: Path) -> dict[str, object]:
                source_calls.append(path)
                return fixture["manifest"]  # type: ignore[return-value]

            def geometry_loader(path: Path) -> SimpleNamespace:
                self.assertNotIn("sealed", str(path))
                return SimpleNamespace(
                    artifact_cid=identities["EXPECTED_GEOMETRY_ARTIFACT_CID"],
                    geometry_file_cid=identities["EXPECTED_GEOMETRY_FILE_CID"],
                )

            with mock.patch.multiple(subject, **identities):
                prepared = subject.prepare_language_path_data(
                    source_root=fixture["source"],  # type: ignore[arg-type]
                    output_root=output,
                    geometry_path=fixture["geometry_path"],  # type: ignore[arg-type]
                    _source_loader=source_loader,  # type: ignore[arg-type]
                    _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                )
                self.assertEqual(source_calls, [fixture["source"].resolve()])
                self.assertEqual(
                    {
                        str(path.relative_to(output))
                        for path in output.rglob("*")
                        if path.is_file()
                    },
                    subject.COPIED_ARTIFACT_PATHS | {subject.DATA_MANIFEST_NAME},
                )
                self.assertEqual(
                    (output / subject.TRAIN_RELATIVE_PATH).read_bytes(),
                    fixture["train_slice"],
                )
                self.assertEqual(
                    (output / subject.VALIDATION_RELATIVE_PATH).read_bytes(),
                    fixture["validation_slice"],
                )
                self.assertEqual(prepared.train_windows.windows.shape, (4, 3))
                self.assertEqual(prepared.validation_windows.windows.shape, (2, 3))
                self.assertEqual(set(prepared.train_order), set(range(4)))
                self.assertEqual(
                    prepared.manifest["access"],
                    {
                        "source_training_view_loader_calls": 1,
                        "source_dataset_loader_calls": 0,
                        "source_sealed_artifact_reads": 0,
                        "source_checkpoint_reads": 0,
                        "source_weight_reads": 0,
                        "teacher_logit_reads": 0,
                        "heldout_reveal_reads": 0,
                    },
                )

                with mock.patch.object(
                    subject,
                    "load_capacity_training_view_manifest",
                    side_effect=AssertionError("loader revisited #1019"),
                ):
                    loaded = subject.load_language_path_preparation(
                        output,
                        _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                    )
                self.assertEqual(loaded.train_windows.window(0).tolist(), [3, 4, 5])
                with self.assertRaises(FileExistsError):
                    subject.prepare_language_path_data(
                        source_root=fixture["source"],  # type: ignore[arg-type]
                        output_root=output,
                        geometry_path=fixture["geometry_path"],  # type: ignore[arg-type]
                        _source_loader=source_loader,  # type: ignore[arg-type]
                        _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                    )

    def test_loader_fails_closed_on_copied_artifact_tamper(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture, identities = self._tiny_contract(root)
            output = root / "new-root"

            def geometry_loader(path: Path) -> SimpleNamespace:
                return SimpleNamespace(
                    artifact_cid=identities["EXPECTED_GEOMETRY_ARTIFACT_CID"],
                    geometry_file_cid=identities["EXPECTED_GEOMETRY_FILE_CID"],
                )

            with mock.patch.multiple(subject, **identities):
                subject.prepare_language_path_data(
                    source_root=fixture["source"],  # type: ignore[arg-type]
                    output_root=output,
                    geometry_path=fixture["geometry_path"],  # type: ignore[arg-type]
                    _source_loader=lambda _: fixture["manifest"],  # type: ignore[return-value]
                    _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                )
                train = output / subject.TRAIN_RELATIVE_PATH
                changed = bytearray(train.read_bytes())
                changed[0] ^= 1
                train.write_bytes(changed)
                with self.assertRaisesRegex(ValueError, "artifact records do not reproduce"):
                    subject.load_language_path_preparation(
                        output,
                        _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                    )

    def test_interrupted_write_never_exposes_partial_final_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture, identities = self._tiny_contract(root)
            output = root / "new-root"
            real_atomic_write = subject.atomic_write
            writes: list[Path] = []

            def interrupted_write(path: Path, value: bytes) -> None:
                writes.append(path)
                if len(writes) == 2:
                    raise KeyboardInterrupt("injected interruption")
                real_atomic_write(path, value)

            def geometry_loader(path: Path) -> SimpleNamespace:
                return SimpleNamespace(
                    artifact_cid=identities["EXPECTED_GEOMETRY_ARTIFACT_CID"],
                    geometry_file_cid=identities["EXPECTED_GEOMETRY_FILE_CID"],
                )

            with (
                mock.patch.multiple(subject, **identities),
                mock.patch.object(subject, "atomic_write", interrupted_write),
                self.assertRaisesRegex(KeyboardInterrupt, "injected interruption"),
            ):
                subject.prepare_language_path_data(
                    source_root=fixture["source"],  # type: ignore[arg-type]
                    output_root=output,
                    geometry_path=fixture["geometry_path"],  # type: ignore[arg-type]
                    _source_loader=lambda _: fixture["manifest"],  # type: ignore[return-value]
                    _geometry_loader=geometry_loader,  # type: ignore[arg-type]
                )

            self.assertFalse(output.exists())
            self.assertEqual(len(writes), 2)
            self.assertEqual(
                list(root.glob(f".{output.name}.preparing-*")),
                [],
            )

    def test_small_slice_helper_checks_boundaries_without_large_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.u16"
            path.write_bytes(_u16(list(range(10))))
            self.assertEqual(
                subject._read_u16_slice(path, offset_tokens=3, token_count=4),
                _u16([3, 4, 5, 6]),
            )
            with self.assertRaisesRegex(ValueError, "crosses"):
                subject._read_u16_slice(path, offset_tokens=8, token_count=3)


if __name__ == "__main__":
    unittest.main()
