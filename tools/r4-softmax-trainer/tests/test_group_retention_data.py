"""Focused population, provenance, and physical-seal tests for #973."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from r4_softmax_trainer import group_retention_data as subject
from r4_softmax_trainer.provenance import (
    canonical_json_bytes,
    cid_bytes,
    write_bound_manifest,
)


class _AggregateGeometry:
    def population_signatures(
        self,
        *,
        fit_stories: tuple[tuple[int, ...], ...],
        heldout_stories: tuple[tuple[int, ...], ...],
    ) -> dict[str, object]:
        return {
            "fit": {
                "stories": len(fit_stories),
                "maximum_leaf_input": max(max(story) for story in fit_stories),
            },
            "heldout": {
                "stories": len(heldout_stories),
                "maximum_leaf_input": max(max(story) for story in heldout_stories),
            },
            "label_free": True,
        }


def _synthetic_store_and_index() -> tuple[bytes, bytes]:
    store_parts: list[bytes] = []
    index_parts: list[bytes] = []
    offset = 0
    # Two deliberately ineligible records plus 322 eligible records make the
    # lowest-CID cutoff observable without reading any broad external corpus.
    for ordinal in range(324):
        story_cid = f"blake3:{(10_000 - ordinal):064x}"
        if ordinal == 0:
            token_count = 258
            truncated = True
        elif ordinal == 1:
            token_count = 256
            truncated = False
        else:
            token_count = 258
            truncated = False
        values = [
            (ordinal + token_ordinal + 1) % subject.VOCAB_SIZE
            for token_ordinal in range(token_count)
        ]
        story_bytes = b"".join(value.to_bytes(2, "little") for value in values)
        store_parts.append(story_bytes)
        index_parts.append(
            canonical_json_bytes(
                {
                    "story_cid": story_cid,
                    "story_token_count": token_count,
                    "story_token_offset": offset,
                    "truncated": truncated,
                }
            )
        )
        offset += token_count
    return b"".join(store_parts), b"".join(index_parts)


def _write_source(root: Path) -> tuple[dict[str, object], dict[str, str]]:
    store, index = _synthetic_store_and_index()
    tokenizer = canonical_json_bytes({"kind": "synthetic-tokenizer"})
    paths = {
        subject.SOURCE_TRAIN_TOKENS_RELATIVE_PATH: store,
        subject.SOURCE_TRAIN_INDEX_RELATIVE_PATH: index,
        subject.SOURCE_TOKENIZER_RELATIVE_PATH: tokenizer,
    }
    for relative, value in paths.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value)
    manifest = write_bound_manifest(
        root / subject.SOURCE_MANIFEST_NAME,
        {"schema": "synthetic-continuation-dataset/1"},
        artifact_root=root,
        relative_paths=paths,
    )
    expected = {
        "EXPECTED_DATASET_MANIFEST_CID": str(manifest["manifest_cid"]),
        "EXPECTED_SOURCE_TREE_CID": str(manifest["tree_cid"]),
        "EXPECTED_TRAIN_STORE_CID": cid_bytes(store),
        "EXPECTED_TRAIN_INDEX_CID": cid_bytes(index),
        "EXPECTED_TOKENIZER_CID": cid_bytes(tokenizer),
    }
    return manifest, expected


class GroupRetentionPureBuilderTests(unittest.TestCase):
    def test_lowest_complete_cids_and_partition_bytes_are_deterministic(self) -> None:
        store, index = _synthetic_store_and_index()
        records = subject.parse_train_index_bytes(index)
        fit, heldout = subject.select_story_spans(records)

        selected = [record.story_cid for record in (*fit, *heldout)]
        eligible = sorted(
            record.story_cid
            for record in records
            if not record.truncated
            and record.story_token_count >= subject.TOKENS_PER_STORY
        )
        self.assertEqual(selected, eligible[: subject.SELECTED_STORY_COUNT])
        self.assertEqual(len(fit), subject.FIT_STORY_COUNT)
        self.assertEqual(len(heldout), subject.HELDOUT_STORY_COUNT)
        self.assertTrue(set(selected[:256]).isdisjoint(selected[256:]))

        first = subject.build_partition_bytes(store, fit, partition="fit")
        second = subject.build_partition_bytes(store, fit, partition="fit")
        self.assertEqual(first, second)
        self.assertEqual(
            len(first.tokens),
            subject.FIT_STORY_COUNT * subject.TOKENS_PER_STORY * 2,
        )
        self.assertEqual(len(first.records), subject.FIT_STORY_COUNT)
        self.assertTrue(all(record["truncated"] is False for record in first.records))
        self.assertTrue(
            all(token < subject.VOCAB_SIZE for story in first.stories for token in story)
        )
        self.assertEqual(
            first.records[0]["span_cid"],
            cid_bytes(first.tokens[: subject.TOKENS_PER_STORY * 2]),
        )

    def test_noncanonical_or_out_of_range_input_fails_closed(self) -> None:
        record = {
            "story_cid": "blake3:" + "1" * 64,
            "story_token_count": subject.TOKENS_PER_STORY,
            "story_token_offset": 0,
            "truncated": False,
        }
        noncanonical = (
            '{"story_cid":"blake3:'
            + "1" * 64
            + '", "story_token_count":257,"story_token_offset":0,"truncated":false}\n'
        ).encode()
        with self.assertRaisesRegex(ValueError, "not canonical JSON"):
            subject.parse_train_index_bytes(noncanonical)

        span = subject.parse_train_index_bytes(canonical_json_bytes(record))[0]
        store = (subject.VOCAB_SIZE).to_bytes(2, "little") * subject.TOKENS_PER_STORY
        with self.assertRaisesRegex(subject.PopulationUnavailable, "outside frozen range"):
            subject.build_partition_bytes(store, [span], partition="fit")


class GroupRetentionPopulationTests(unittest.TestCase):
    def _prepare(self, directory: str) -> tuple[Path, dict[str, object], dict[str, str]]:
        base = Path(directory)
        source_root = base / "source"
        output_root = base / "population"
        _, expected = _write_source(source_root)
        with mock.patch.multiple(subject, **expected):
            result = subject.prepare_group_retention_population(
                output_root,
                source_root,
                geometry=_AggregateGeometry(),
            )
        return output_root, result, expected

    def test_prepare_binds_exact_source_and_physically_seals_heldout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root, result, expected = self._prepare(directory)
            population = result["population"]
            self.assertEqual(population["source"]["train_store_cid"], expected["EXPECTED_TRAIN_STORE_CID"])
            self.assertEqual(population["population"]["fit_targets"], 65_536)
            self.assertEqual(population["population"]["heldout_targets"], 16_384)
            self.assertTrue(population["population"]["story_disjoint"])
            self.assertEqual(population["population"]["truncated_stories"], 0)
            self.assertLess(population["population"]["maximum_token_id"], 4096)
            self.assertEqual(population["geometry"]["status"], "COMPUTED")
            self.assertEqual(
                (root / subject.HELDOUT_DIRECTORY_RELATIVE_PATH).stat().st_mode & 0o777,
                0,
            )
            self.assertEqual(
                (root / subject.FIT_TOKENS_RELATIVE_PATH).stat().st_size,
                subject.FIT_STORY_COUNT * subject.TOKENS_PER_STORY * 2,
            )
            with mock.patch.multiple(subject, **expected):
                view = subject.load_group_retention_training_view(root)
            self.assertEqual(view["population_manifest_cid"], population["manifest_cid"])

    def test_training_view_verifies_fit_without_opening_heldout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root, _, expected = self._prepare(directory)
            original_open = Path.open

            def guarded_open(path: Path, *args: object, **kwargs: object):
                if subject.HELDOUT_DIRECTORY_RELATIVE_PATH in path.parts:
                    raise AssertionError(f"held-out path opened: {path}")
                return original_open(path, *args, **kwargs)

            with (
                mock.patch.multiple(subject, **expected),
                mock.patch.object(Path, "open", guarded_open),
            ):
                view = subject.load_group_retention_training_view(root)
            self.assertEqual(view["fit"]["stories"], subject.FIT_STORY_COUNT)

    def test_partial_heldout_materialization_remains_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            source_root = base / "source"
            output_root = base / "population"
            _, expected = _write_source(source_root)
            original_write = subject.atomic_write

            def fail_second_heldout_write(path: Path, value: bytes) -> None:
                if str(path).endswith(subject.HELDOUT_INDEX_RELATIVE_PATH):
                    raise OSError("synthetic held-out index failure")
                original_write(path, value)

            with (
                mock.patch.multiple(subject, **expected),
                mock.patch.object(
                    subject, "atomic_write", side_effect=fail_second_heldout_write
                ),
                self.assertRaisesRegex(OSError, "synthetic held-out index failure"),
            ):
                subject.prepare_group_retention_population(output_root, source_root)
            self.assertEqual(
                (output_root / subject.HELDOUT_DIRECTORY_RELATIVE_PATH).stat().st_mode
                & 0o777,
                0,
            )

    def test_terminal_package_exposes_no_heldout_open_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root, _, _ = self._prepare(directory)
            self.assertFalse(hasattr(subject, "build_fitted_artifact_commitment"))
            self.assertFalse(hasattr(subject, "open_group_retention_heldout"))
            self.assertEqual(
                (root / subject.HELDOUT_DIRECTORY_RELATIVE_PATH).stat().st_mode & 0o777,
                0,
            )


if __name__ == "__main__":
    unittest.main()
