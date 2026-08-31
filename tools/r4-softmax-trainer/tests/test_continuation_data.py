"""Focused contracts for the #1017 fresh-population builder."""

from __future__ import annotations

import json
import struct
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from blake3 import blake3

from r4_softmax_trainer import continuation_data as subject
from r4_softmax_trainer.constants import FROZEN_MODEL_CONFIG, STORY_DELIMITER
from r4_softmax_trainer.provenance import write_bound_manifest


class _CharacterTokenizer:
    def encode(self, text: str, *, add_special_tokens: bool) -> SimpleNamespace:
        if add_special_tokens:
            raise AssertionError("the continuation builder inserts BOS/EOS explicitly")
        return SimpleNamespace(ids=[ord(character) + 3 for character in text])

    def decode(self, token_ids: list[int], *, skip_special_tokens: bool) -> str:
        if not skip_special_tokens:
            raise AssertionError("prompt decoding must skip special tokens")
        return "".join(chr(token_id - 3) for token_id in token_ids)


def _split(story: bytes) -> str:
    prefix = story[:1]
    return {b"T": "train", b"D": "dev", b"S": "test"}[prefix]


def _tokens(tokenizer: _CharacterTokenizer, story: bytes) -> list[int]:
    content = tokenizer.encode(story.decode(), add_special_tokens=False).ids
    return [subject.BOS_TOKEN_ID, *content, subject.EOS_TOKEN_ID]


def _old_store(
    stories: list[bytes], split: str, cap: int, tokenizer: _CharacterTokenizer
) -> bytes:
    values: list[int] = []
    for story in stories:
        if _split(story) != split or len(values) >= cap:
            continue
        token_ids = _tokens(tokenizer, story)
        remaining = cap - len(values)
        if remaining == 1:
            values.append(subject.EOS_TOKEN_ID)
        elif len(token_ids) > remaining:
            values.extend(
                [
                    subject.BOS_TOKEN_ID,
                    *token_ids[1:-1][: remaining - 2],
                    subject.EOS_TOKEN_ID,
                ]
            )
        else:
            values.extend(token_ids)
    return b"".join(struct.pack("<H", value) for value in values)


def _records(
    stories: list[bytes], caps: dict[str, int], tokenizer: _CharacterTokenizer
) -> dict[str, dict[str, object]]:
    records = {}
    for split, cap in caps.items():
        payload = _old_store(stories, split, cap, tokenizer)
        records[split] = {"bytes": len(payload), "cid": f"blake3:{blake3(payload).hexdigest()}"}
    return records


class ContinuationBoundaryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tokenizer = _CharacterTokenizer()
        self.stories = [
            b"T predecessor train story that is deliberately truncated",
            b"D filler boundary story remains completely fresh",
            b"S predecessor test",
            b"T first continuation train",
            b"D second development story",
            b"S first continuation test story with enough prompt text",
            b"T another continuation train story",
            b"D third development story",
            b"S second eligible test story with enough prompt text",
            b"S third eligible test story with enough prompt text",
            b"S fourth eligible test story with enough prompt text",
            b"S fifth eligible test story with enough prompt text",
            b"S sixth eligible test story with enough prompt text",
        ]

    def _write_source(self, root: Path) -> Path:
        path = root / "source.txt"
        path.write_bytes(STORY_DELIMITER.join(self.stories) + STORY_DELIMITER)
        return path

    def test_story_aligned_boundary_and_exact_caps(self) -> None:
        predecessor_caps = {
            "train": 7,
            "dev": 1,
            "test": len(_tokens(self.tokenizer, self.stories[2])),
        }
        continuation_caps = {"train": 20, "dev": 20, "test": 20}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self._write_source(root)
            records = _records(self.stories, predecessor_caps, self.tokenizer)
            with (
                mock.patch.object(subject, "story_split", side_effect=_split),
                mock.patch.object(subject, "PREDECESSOR_TOKEN_CAPS", predecessor_caps),
                mock.patch.object(subject, "CONTINUATION_TOKEN_CAPS", continuation_caps),
            ):
                result = subject._build_continuation_stores(
                    source, self.tokenizer, root, records
                )

            self.assertEqual(result["counts"], continuation_caps)
            self.assertEqual(
                result["splits"]["train"]["first_story_cid"],
                f"blake3:{blake3(self.stories[3]).hexdigest()}",
            )
            # A one-token predecessor filler consumes none of the current dev story.
            self.assertEqual(
                result["splits"]["dev"]["first_story_cid"],
                f"blake3:{blake3(self.stories[1]).hexdigest()}",
            )
            self.assertEqual(
                result["splits"]["test"]["first_story_cid"],
                f"blake3:{blake3(self.stories[5]).hexdigest()}",
            )
            train_index = [
                json.loads(line)
                for line in (root / subject.INDEX_RELATIVE_PATHS["train"])
                .read_text(encoding="utf-8")
                .splitlines()
            ]
            self.assertNotEqual(
                train_index[0]["story_cid"], f"blake3:{blake3(self.stories[0]).hexdigest()}"
            )
            for split in ("train", "dev", "test"):
                self.assertEqual(
                    (root / subject.TOKEN_RELATIVE_PATHS[split]).stat().st_size,
                    continuation_caps[split] * 2,
                )
                self.assertEqual(
                    result["predecessor_reproduction"][split]["expected_cid"],
                    result["predecessor_reproduction"][split]["reproduced_cid"],
                )

    def test_repeated_builds_are_byte_identical(self) -> None:
        predecessor_caps = {
            "train": 7,
            "dev": 1,
            "test": len(_tokens(self.tokenizer, self.stories[2])),
        }
        continuation_caps = {"train": 20, "dev": 20, "test": 20}
        with tempfile.TemporaryDirectory() as first, tempfile.TemporaryDirectory() as second:
            roots = [Path(first), Path(second)]
            records = _records(self.stories, predecessor_caps, self.tokenizer)
            with (
                mock.patch.object(subject, "story_split", side_effect=_split),
                mock.patch.object(subject, "PREDECESSOR_TOKEN_CAPS", predecessor_caps),
                mock.patch.object(subject, "CONTINUATION_TOKEN_CAPS", continuation_caps),
            ):
                for root in roots:
                    subject._build_continuation_stores(
                        self._write_source(root), self.tokenizer, root, records
                    )
            paths = [
                *subject.TOKEN_RELATIVE_PATHS.values(),
                *subject.INDEX_RELATIVE_PATHS.values(),
                subject.SEALED_PROMPT_RELATIVE_PATH,
            ]
            for relative in paths:
                self.assertEqual(
                    (roots[0] / relative).read_bytes(),
                    (roots[1] / relative).read_bytes(),
                )

    def test_short_source_fails_closed(self) -> None:
        predecessor_caps = {
            "train": 7,
            "dev": 1,
            "test": len(_tokens(self.tokenizer, self.stories[2])),
        }
        continuation_caps = {"train": 1_000, "dev": 1_000, "test": 1_000}
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = self._write_source(root)
            records = _records(self.stories, predecessor_caps, self.tokenizer)
            with (
                mock.patch.object(subject, "story_split", side_effect=_split),
                mock.patch.object(subject, "PREDECESSOR_TOKEN_CAPS", predecessor_caps),
                mock.patch.object(subject, "CONTINUATION_TOKEN_CAPS", continuation_caps),
                self.assertRaisesRegex(
                    subject.FreshPopulationUnavailable, "UNAVAILABLE_FRESH_POPULATION"
                ),
            ):
                subject._build_continuation_stores(source, self.tokenizer, root, records)


class ContinuationPromptTests(unittest.TestCase):
    def test_excludes_published_prompts_and_selects_lowest_eligible(self) -> None:
        tokenizer = _CharacterTokenizer()
        selected: list[dict[str, object]] = []
        eligible_story = b"S this test story has substantially more than twenty four characters"
        subject._consider_prompt(
            selected, tokenizer, subject.PREDECESSOR_PROMPT_CIDS[0], eligible_story
        )
        self.assertEqual(selected, [])
        cids = [f"blake3:{value:064x}" for value in [9, 2, 7, 1, 8, 3, 6]]
        for cid in cids:
            subject._consider_prompt(selected, tokenizer, cid, eligible_story)
        self.assertEqual(
            [record["story_cid"] for record in selected],
            sorted(cids)[: subject.SEALED_PROMPT_COUNT],
        )
        for record in selected:
            self.assertEqual(len(record["token_ids"]), subject.SEALED_PROMPT_TOKENS_PER_STORY)
            self.assertEqual(
                tokenizer.encode(record["text"], add_special_tokens=False).ids,
                record["token_ids"],
            )


class ContinuationTrainingViewTests(unittest.TestCase):
    def test_loader_never_opens_sealed_confirmation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            training_paths = [
                subject.TOKENIZER_RELATIVE_PATH,
                subject.INHERITED_CHECKPOINT_RELATIVE_PATH,
                subject.TOKEN_RELATIVE_PATHS["train"],
                subject.TOKEN_RELATIVE_PATHS["dev"],
                subject.INDEX_RELATIVE_PATHS["train"],
                subject.INDEX_RELATIVE_PATHS["dev"],
            ]
            for relative in training_paths:
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(relative.encode())
            sealed = root / subject.TOKEN_RELATIVE_PATHS["test"]
            sealed.parent.mkdir(parents=True, exist_ok=True)
            sealed.write_bytes(b"must remain unopened")
            denial = {
                "schema": subject.SEALED_DENIAL_SCHEMA,
                "issue": 1017,
                "continuation_dataset_manifest_cid": "blake3:" + "0" * 64,
                "directory": subject.SEALED_DIRECTORY_RELATIVE_PATH,
                "directory_mode": "000",
                "read_attempt": "PERMISSION_DENIED",
                "sealed_paths": [subject.TOKEN_RELATIVE_PATHS["test"]],
                "training_or_selection_reads": 0,
            }
            denial["result_cid"] = subject.cid_bytes(subject.canonical_json_bytes(denial))
            subject.atomic_write_json(root / subject.SEALED_DENIAL_RELATIVE_PATH, denial)
            training_paths.append(subject.SEALED_DENIAL_RELATIVE_PATH)
            write_bound_manifest(
                root / subject.CONTINUATION_TRAINING_VIEW_MANIFEST_NAME,
                {
                    "schema": subject.CONTINUATION_TRAINING_VIEW_MANIFEST_SCHEMA,
                    "continuation_dataset_manifest_cid": "blake3:" + "0" * 64,
                    "predecessor": {
                        "checkpoint_cid": subject.PREDECESSOR_CHECKPOINT_CID,
                        "dataset_manifest_cid": subject.PREDECESSOR_DATASET_MANIFEST_CID,
                    },
                    "split_policy_cid": subject.PREDECESSOR_SPLIT_POLICY_CID,
                    "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
                    "sealed_confirmation_commitment": {
                        "path": subject.TOKEN_RELATIVE_PATHS["test"],
                        "denial_result_cid": denial["result_cid"],
                    },
                },
                artifact_root=root,
                relative_paths=training_paths,
            )
            sealed.parent.chmod(0)
            original_open = Path.open
            opened: list[Path] = []

            def audited_open(path: Path, *args: object, **kwargs: object):
                opened.append(path)
                if "sealed-confirmation" in path.parts:
                    raise AssertionError(f"sealed path opened: {path}")
                return original_open(path, *args, **kwargs)

            with mock.patch.object(Path, "open", audited_open):
                subject.load_continuation_training_view_manifest(root)
            self.assertFalse(any("sealed-confirmation" in path.parts for path in opened))
            sealed.parent.chmod(0o700)


if __name__ == "__main__":
    unittest.main()
