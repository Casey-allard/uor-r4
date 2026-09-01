"""Focused fit-only population checks for the #973 fuller decoder."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest import mock

from r4_softmax_trainer import group_retention_decoder_data as subject
from r4_softmax_trainer.group_retention_data import (
    FIT_INDEX_RELATIVE_PATH,
    FIT_TOKENS_RELATIVE_PATH,
)
from r4_softmax_trainer.provenance import canonical_json_bytes, cid_bytes


def _synthetic_predecessor(root: Path) -> tuple[dict[str, object], dict[str, str]]:
    store_parts: list[bytes] = []
    index_parts: list[bytes] = []
    for ordinal in range(256):
        values = [
            (ordinal * 131 + token_ordinal) % 4_096
            for token_ordinal in range(subject.PREDECESSOR_TOKENS_PER_STORY)
        ]
        story = b"".join(value.to_bytes(2, "little") for value in values)
        store_parts.append(story)
        index_parts.append(
            canonical_json_bytes(
                {
                    "copied_token_count": subject.PREDECESSOR_TOKENS_PER_STORY,
                    "copied_token_offset": ordinal
                    * subject.PREDECESSOR_TOKENS_PER_STORY,
                    "partition": "fit",
                    "partition_ordinal": ordinal,
                    "scored_next_tokens": subject.PREDECESSOR_TOKENS_PER_STORY - 1,
                    "source_story_token_count": 300,
                    "source_story_token_offset": ordinal * 300,
                    "span_cid": cid_bytes(story),
                    "story_cid": f"blake3:{ordinal + 1:064x}",
                    "truncated": False,
                }
            )
        )
    store = b"".join(store_parts)
    index = b"".join(index_parts)
    for relative, value in {
        FIT_TOKENS_RELATIVE_PATH: store,
        FIT_INDEX_RELATIVE_PATH: index,
    }.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value)
    identities = {
        "EXPECTED_PREDECESSOR_FIT_STORE_CID": cid_bytes(store),
        "EXPECTED_PREDECESSOR_FIT_INDEX_CID": cid_bytes(index),
    }
    manifest: dict[str, object] = {
        "manifest_cid": subject.EXPECTED_PREDECESSOR_TRAINING_VIEW_CID,
        "population_manifest_cid": subject.EXPECTED_PREDECESSOR_POPULATION_CID,
        "source": {"tokenizer_cid": subject.EXPECTED_TOKENIZER_CID},
        "artifacts": [
            {"path": FIT_TOKENS_RELATIVE_PATH, "cid": identities["EXPECTED_PREDECESSOR_FIT_STORE_CID"]},
            {"path": FIT_INDEX_RELATIVE_PATH, "cid": identities["EXPECTED_PREDECESSOR_FIT_INDEX_CID"]},
        ],
    }
    return manifest, identities


class DecoderConstructionDataTests(unittest.TestCase):
    def test_build_uses_only_disjoint_post_smoke_fit_ordinals(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, identities = _synthetic_predecessor(root)
            original_read_bytes = Path.read_bytes
            opened: list[str] = []

            def guarded_read_bytes(path: Path) -> bytes:
                opened.append(str(path))
                self.assertNotIn("sealed", str(path))
                self.assertNotIn("heldout", str(path))
                return original_read_bytes(path)

            with (
                mock.patch.object(
                    subject, "load_group_retention_training_view", return_value=manifest
                ),
                mock.patch.multiple(subject, **identities),
                mock.patch.object(Path, "read_bytes", guarded_read_bytes),
            ):
                result = subject.build_decoder_construction_data(root)

            self.assertEqual(result.train.ordinals, tuple(range(8, 40)))
            self.assertEqual(result.validation.ordinals, tuple(range(40, 72)))
            self.assertTrue(set(result.train.story_cids).isdisjoint(result.validation.story_cids))
            self.assertEqual(result.train.decisions, 4_096)
            self.assertEqual(result.validation.decisions, 4_096)
            self.assertEqual(
                len(result.train.tokens),
                subject.STORIES_PER_PARTITION * subject.TOKENS_PER_STORY * 2,
            )
            self.assertEqual(
                subject.decode_construction_tensor(
                    result.validation.tokens, partition="validation"
                ).shape,
                (32, 129),
            )
            self.assertEqual(
                set(opened),
                {
                    str((root / FIT_TOKENS_RELATIVE_PATH).resolve()),
                    str((root / FIT_INDEX_RELATIVE_PATH).resolve()),
                },
            )

    def test_full_predecessor_span_cid_is_reproduced_before_slicing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, identities = _synthetic_predecessor(root)
            path = root / FIT_TOKENS_RELATIVE_PATH
            value = bytearray(path.read_bytes())
            value[8 * subject.PREDECESSOR_TOKENS_PER_STORY * 2] ^= 1
            path.write_bytes(value)
            identities["EXPECTED_PREDECESSOR_FIT_STORE_CID"] = cid_bytes(bytes(value))
            manifest["artifacts"][0]["cid"] = identities[
                "EXPECTED_PREDECESSOR_FIT_STORE_CID"
            ]
            with (
                mock.patch.object(
                    subject, "load_group_retention_training_view", return_value=manifest
                ),
                mock.patch.multiple(subject, **identities),
                self.assertRaisesRegex(ValueError, "span CID differs"),
            ):
                subject.build_decoder_construction_data(root)


if __name__ == "__main__":
    unittest.main()
