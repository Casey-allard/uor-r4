"""Pinned TinyStories intake, pre-tokenization split, and token-store build."""

from __future__ import annotations

import hashlib
import json
import os
import sys
import urllib.request
from array import array
from dataclasses import dataclass, field
from pathlib import Path
from typing import BinaryIO, Iterator

from blake3 import blake3

from .constants import (
    BOS_TOKEN,
    BOS_TOKEN_ID,
    DATASET_MANIFEST_SCHEMA,
    DEV_TOKEN_CAP,
    EOS_TOKEN,
    EOS_TOKEN_ID,
    FROZEN_MODEL_CONFIG,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
    STORY_DELIMITER,
    TEST_TOKEN_CAP,
    TEST_REVEAL_TOTAL_CAP,
    TINYSTORIES_BYTES,
    TINYSTORIES_FILENAME,
    TINYSTORIES_REPOSITORY,
    TINYSTORIES_REVISION,
    TINYSTORIES_SHA256,
    TINYSTORIES_URL,
    TOKENIZER_TRAIN_BYTES,
    TRAINING_VIEW_MANIFEST_SCHEMA,
    TRAIN_TOKEN_CAP,
    UNK_TOKEN,
    UNK_TOKEN_ID,
)
from .provenance import (
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    verify_artifact_subset,
    verify_bound_manifest,
    verify_manifest_envelope,
    write_bound_manifest,
)


DATASET_MANIFEST_NAME = "dataset-manifest.json"
TRAINING_VIEW_MANIFEST_NAME = "training-view-manifest.json"
TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"
TOKEN_RELATIVE_PATHS = {
    "train": "tokens/train.u16",
    "dev": "tokens/dev.u16",
    "test": "sealed-test/test.u16",
}
INDEX_RELATIVE_PATHS = {
    "train": "indexes/train.jsonl",
    "dev": "indexes/dev.jsonl",
    "test": "sealed-test/test-index.jsonl",
}
SEALED_PROMPT_RELATIVE_PATH = "sealed-test/prompts.json"
TOKEN_CAPS = {
    "train": TRAIN_TOKEN_CAP,
    "dev": DEV_TOKEN_CAP,
    "test": TEST_TOKEN_CAP,
}


@dataclass(slots=True)
class StorySummary:
    stories: int = 0
    story_bytes: int = 0
    truncated_final_story: bool = False
    _digest: object = field(init=False, repr=False)

    def __post_init__(self) -> None:
        self._digest = blake3()

    def add(self, story: bytes) -> None:
        self.stories += 1
        self.story_bytes += len(story)
        self._digest.update(len(story).to_bytes(8, "little"))
        self._digest.update(story)


class U16Writer:
    """Buffered, explicitly little-endian token writer."""

    def __init__(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        self.path = path
        self.temporary = path.with_name(f".{path.name}.part")
        self.handle: BinaryIO = self.temporary.open("wb")
        self.buffer = array("H")
        self.count = 0

    def append(self, values: list[int]) -> None:
        if any(value < 0 or value > 0xFFFF for value in values):
            raise ValueError("token id outside uint16")
        self.buffer.extend(values)
        self.count += len(values)
        if len(self.buffer) >= 1 << 20:
            self.flush()

    def flush(self) -> None:
        if not self.buffer:
            return
        if sys.byteorder != "little":
            self.buffer.byteswap()
        self.buffer.tofile(self.handle)
        if sys.byteorder != "little":
            self.buffer.byteswap()
        self.buffer = array("H")

    def finish(self) -> None:
        self.flush()
        self.handle.flush()
        os.fsync(self.handle.fileno())
        self.handle.close()
        os.replace(self.temporary, self.path)

    def abort(self) -> None:
        self.handle.close()
        self.temporary.unlink(missing_ok=True)


class IndexWriter:
    """Canonical JSON-lines story boundary/CID ledger."""

    def __init__(self, path: Path) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        self.path = path
        self.temporary = path.with_name(f".{path.name}.part")
        self.handle = self.temporary.open("wb")

    def append(self, record: dict[str, object]) -> None:
        self.handle.write(canonical_json_bytes(record))

    def finish(self) -> None:
        self.handle.flush()
        os.fsync(self.handle.fileno())
        self.handle.close()
        os.replace(self.temporary, self.path)

    def abort(self) -> None:
        self.handle.close()
        self.temporary.unlink(missing_ok=True)


def story_split(story: bytes) -> str:
    """Assign raw canonical story bytes to a stable 90/5/5 partition."""
    if not story:
        raise ValueError("empty story has no split")
    bucket = int.from_bytes(blake3(story).digest(), "big") % 100
    if bucket < 90:
        return "train"
    if bucket < 95:
        return "dev"
    return "test"


def iter_canonical_stories(source: Path, *, chunk_size: int = 4 * 1024 * 1024) -> Iterator[bytes]:
    """Yield exact nonempty payloads between TinyStories delimiters."""
    buffer = b""
    with source.open("rb") as stream:
        while chunk := stream.read(chunk_size):
            buffer += chunk
            while True:
                boundary = buffer.find(STORY_DELIMITER)
                if boundary < 0:
                    break
                story = buffer[:boundary].strip(b" \t\r\n")
                buffer = buffer[boundary + len(STORY_DELIMITER) :]
                if story:
                    yield story
            if len(buffer) > 16 * 1024 * 1024:
                raise ValueError("TinyStories record exceeds 16 MiB safety bound")
    story = buffer.strip(b" \t\r\n")
    if story:
        yield story


def _sha256_file(path: Path, *, chunk_size: int = 8 * 1024 * 1024) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(chunk_size):
            digest.update(chunk)
    return digest.hexdigest()


def verify_source(path: Path) -> None:
    size = path.stat().st_size
    if size != TINYSTORIES_BYTES:
        raise ValueError(f"TinyStories byte length {size} != pinned {TINYSTORIES_BYTES}")
    digest = _sha256_file(path)
    if digest != TINYSTORIES_SHA256:
        raise ValueError(f"TinyStories SHA-256 {digest} != pinned {TINYSTORIES_SHA256}")


def download_source(root: Path) -> Path:
    """Download and verify the exact pinned TinyStories source file."""
    destination = root / "raw" / TINYSTORIES_FILENAME
    if destination.is_file():
        verify_source(destination)
        return destination
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.part")
    request = urllib.request.Request(
        TINYSTORIES_URL,
        headers={"User-Agent": "uor-r4-softmax-trainer/0.1 (#1014)"},
    )
    digest = hashlib.sha256()
    total = 0
    try:
        with urllib.request.urlopen(request, timeout=60) as response, temporary.open("wb") as sink:
            while chunk := response.read(8 * 1024 * 1024):
                sink.write(chunk)
                digest.update(chunk)
                total += len(chunk)
                if total > TINYSTORIES_BYTES:
                    raise ValueError("TinyStories download exceeded pinned byte length")
            sink.flush()
            os.fsync(sink.fileno())
        if total != TINYSTORIES_BYTES:
            raise ValueError(f"TinyStories download length {total} != pinned {TINYSTORIES_BYTES}")
        if digest.hexdigest() != TINYSTORIES_SHA256:
            raise ValueError("TinyStories download SHA-256 did not reproduce the pin")
        os.replace(temporary, destination)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise
    return destination


def _training_story_iterator(source: Path, summary: StorySummary) -> Iterator[str]:
    for story in iter_canonical_stories(source):
        if story_split(story) != "train":
            continue
        if summary.story_bytes + len(story) > TOKENIZER_TRAIN_BYTES:
            break
        summary.add(story)
        yield story.decode("utf-8", errors="strict")


def validate_tokenizer_json(path: Path) -> dict[str, object]:
    """Validate the subset consumed by the repository `HfBpeTokenizer`."""
    definition = json.loads(path.read_text(encoding="utf-8"))
    if definition.get("normalizer") is not None:
        raise ValueError("tokenizer normalizer must be null")
    if definition.get("post_processor") is not None:
        raise ValueError("tokenizer post_processor must be null")
    pre_tokenizer = definition.get("pre_tokenizer")
    if not isinstance(pre_tokenizer, dict) or pre_tokenizer.get("type") != "ByteLevel":
        raise ValueError("tokenizer must use one ByteLevel pre-tokenizer")
    if pre_tokenizer.get("add_prefix_space") is not False:
        raise ValueError("ByteLevel add_prefix_space must be false")
    model = definition.get("model")
    if not isinstance(model, dict) or model.get("type") != "BPE":
        raise ValueError("tokenizer model must be BPE")
    vocab = model.get("vocab")
    if not isinstance(vocab, dict) or len(vocab) != FROZEN_MODEL_CONFIG.vocab_size:
        raise ValueError("tokenizer vocabulary must contain exactly 4096 entries")
    ids = sorted(vocab.values())
    if ids != list(range(FROZEN_MODEL_CONFIG.vocab_size)):
        raise ValueError("tokenizer model ids must be a dense 0..4095 prefix")
    for token, expected_id in [
        (BOS_TOKEN, BOS_TOKEN_ID),
        (EOS_TOKEN, EOS_TOKEN_ID),
        (UNK_TOKEN, UNK_TOKEN_ID),
    ]:
        if vocab.get(token) != expected_id:
            raise ValueError(f"special token {token} must have id {expected_id}")
    merges = model.get("merges")
    if not isinstance(merges, list) or not merges:
        raise ValueError("tokenizer BPE merges must be nonempty")
    return definition


def train_tokenizer(source: Path, root: Path) -> tuple[Path, StorySummary]:
    """Train the exact 4096-id byte-level BPE on train stories only."""
    from tokenizers import Tokenizer, decoders, models, pre_tokenizers, trainers

    summary = StorySummary()
    tokenizer = Tokenizer(models.BPE(unk_token=UNK_TOKEN))
    tokenizer.normalizer = None
    tokenizer.pre_tokenizer = pre_tokenizers.ByteLevel(add_prefix_space=False, use_regex=True)
    tokenizer.decoder = decoders.ByteLevel()
    tokenizer.post_processor = None
    trainer = trainers.BpeTrainer(
        vocab_size=FROZEN_MODEL_CONFIG.vocab_size,
        min_frequency=2,
        show_progress=True,
        special_tokens=[BOS_TOKEN, EOS_TOKEN, UNK_TOKEN],
        initial_alphabet=sorted(pre_tokenizers.ByteLevel.alphabet()),
    )
    tokenizer.train_from_iterator(_training_story_iterator(source, summary), trainer=trainer)
    destination = root / TOKENIZER_RELATIVE_PATH
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.part")
    tokenizer.save(str(temporary), pretty=False)
    os.replace(temporary, destination)
    validate_tokenizer_json(destination)
    return destination, summary


def _build_token_stores(
    source: Path,
    tokenizer_path: Path,
    root: Path,
) -> tuple[
    dict[str, int],
    dict[str, StorySummary],
    int,
    dict[str, int],
    dict[str, object],
]:
    from tokenizers import Tokenizer

    tokenizer = Tokenizer.from_file(str(tokenizer_path))
    writers = {name: U16Writer(root / relative) for name, relative in TOKEN_RELATIVE_PATHS.items()}
    indexes = {name: IndexWriter(root / relative) for name, relative in INDEX_RELATIVE_PATHS.items()}
    summaries = {name: StorySummary() for name in TOKEN_RELATIVE_PATHS}
    padding_tokens = {name: 0 for name in TOKEN_RELATIVE_PATHS}
    lowest_test_stories: list[tuple[str, bytes]] = []
    global_test_stories = 0
    global_test_story_bytes = 0
    ordered_test_cids = blake3()
    examined = 0
    try:
        for story in iter_canonical_stories(source):
            examined += 1
            split = story_split(story)
            if split == "test":
                story_digest = blake3(story).digest()
                story_cid = f"blake3:{story_digest.hex()}"
                global_test_stories += 1
                global_test_story_bytes += len(story)
                ordered_test_cids.update(story_digest)
                lowest_test_stories.append((story_cid, story))
                lowest_test_stories.sort(key=lambda candidate: candidate[0])
                del lowest_test_stories[SEALED_PROMPT_COUNT:]
            writer = writers[split]
            cap = TOKEN_CAPS[split]
            if writer.count >= cap:
                continue
            text = story.decode("utf-8", errors="strict")
            content_ids = tokenizer.encode(text, add_special_tokens=False).ids
            token_ids = [BOS_TOKEN_ID, *content_ids, EOS_TOKEN_ID]
            remaining = cap - writer.count
            if remaining == 1:
                # Keep every indexed story structurally complete. One EOS
                # filler reaches the exact cap but belongs to no story.
                writer.append([EOS_TOKEN_ID])
                padding_tokens[split] += 1
                continue
            truncated = False
            if len(token_ids) > remaining:
                content_ids = content_ids[: remaining - 2]
                token_ids = [BOS_TOKEN_ID, *content_ids, EOS_TOKEN_ID]
                summaries[split].truncated_final_story = True
                truncated = True
            offset = writer.count
            summaries[split].add(story)
            writer.append(token_ids)
            indexes[split].append(
                {
                    "content_token_count": len(content_ids),
                    "content_token_offset": offset + 1,
                    "story_cid": cid_bytes(story),
                    "story_token_count": len(token_ids),
                    "story_token_offset": offset,
                    "truncated": truncated,
                }
            )
        counts = {name: writer.count for name, writer in writers.items()}
        if counts != TOKEN_CAPS:
            raise RuntimeError(f"source exhausted before deterministic token caps: {counts}")
        for writer in writers.values():
            writer.finish()
        for index in indexes.values():
            index.finish()
        if len(lowest_test_stories) != SEALED_PROMPT_COUNT:
            raise RuntimeError("frozen snapshot did not contain five test stories")
        prompts: list[dict[str, object]] = []
        for story_cid, story in lowest_test_stories:
            content_ids = tokenizer.encode(story.decode("utf-8", errors="strict"), add_special_tokens=False).ids
            if len(content_ids) < SEALED_PROMPT_TOKENS_PER_STORY:
                raise RuntimeError(
                    f"lowest-CID test story {story_cid} has fewer than "
                    f"{SEALED_PROMPT_TOKENS_PER_STORY} tokens"
                )
            prompt_ids = content_ids[:SEALED_PROMPT_TOKENS_PER_STORY]
            prompt_text = tokenizer.decode(prompt_ids, skip_special_tokens=True)
            prompt_text.encode("utf-8", errors="strict")
            if tokenizer.encode(prompt_text, add_special_tokens=False).ids != prompt_ids:
                raise RuntimeError(
                    f"lowest-CID test prompt {story_cid} does not round-trip its 24 token ids"
                )
            prompts.append(
                {
                    "story_cid": story_cid,
                    "token_ids": prompt_ids,
                    "text": prompt_text,
                }
            )
        prompt_fixture: dict[str, object] = {
            "schema": "uor-r4-softmax-trainer-sealed-prompts/1",
            "selection": "first 24 content tokens of five lowest full-story CIDs",
            "revealed_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "full_snapshot_test_population": {
                "stories": global_test_stories,
                "story_bytes": global_test_story_bytes,
                "ordered_full_story_cids_cid": f"blake3:{ordered_test_cids.hexdigest()}",
            },
            "tokenizer_cid": cid_file(tokenizer_path),
            "prompts": prompts,
        }
        prompt_fixture["fixture_cid"] = cid_bytes(canonical_json_bytes(prompt_fixture))
        atomic_write_json(root / SEALED_PROMPT_RELATIVE_PATH, prompt_fixture)
    except BaseException:
        for writer in writers.values():
            if not writer.handle.closed:
                writer.abort()
        for index in indexes.values():
            if not index.handle.closed:
                index.abort()
        raise
    return (
        counts,
        summaries,
        examined,
        padding_tokens,
        {
            "stories": global_test_stories,
            "story_bytes": global_test_story_bytes,
            "ordered_full_story_cids_cid": f"blake3:{ordered_test_cids.hexdigest()}",
        },
    )


def prepare_dataset(root: Path, *, source: Path | None = None, force: bool = False) -> dict[str, object]:
    """Build the one CID-bound dataset/tokenizer snapshot used by #1014."""
    root = root.resolve()
    manifest_path = root / DATASET_MANIFEST_NAME
    training_view_path = root / TRAINING_VIEW_MANIFEST_NAME
    if manifest_path.is_file() and training_view_path.is_file() and not force:
        return {
            "dataset": verify_bound_manifest(manifest_path, artifact_root=root),
            "training_view": verify_bound_manifest(training_view_path, artifact_root=root),
        }
    root.mkdir(parents=True, exist_ok=True)
    source_path = source.resolve() if source is not None else download_source(root)
    verify_source(source_path)
    tokenizer_path, tokenizer_summary = train_tokenizer(source_path, root)
    (
        counts,
        split_summaries,
        examined,
        padding_tokens,
        global_test_population,
    ) = _build_token_stores(source_path, tokenizer_path, root)
    split_payload: dict[str, object] = {}
    for name in ["train", "dev", "test"]:
        summary = split_summaries[name]
        split_payload[name] = {
            "stories": summary.stories,
            "story_bytes": summary.story_bytes,
            "ordered_story_bytes_cid": f"blake3:{summary._digest.hexdigest()}",
            "tokens": counts[name],
            "token_cap": TOKEN_CAPS[name],
            "complete_context_scored_next_tokens": (
                (counts[name] - 1) // FROZEN_MODEL_CONFIG.max_position_embeddings
            )
            * FROZEN_MODEL_CONFIG.max_position_embeddings,
            "cap_padding_tokens": padding_tokens[name],
            "truncated_final_story": summary.truncated_final_story,
        }
    payload: dict[str, object] = {
        "schema": DATASET_MANIFEST_SCHEMA,
        "source": {
            "repository": TINYSTORIES_REPOSITORY,
            "revision": TINYSTORIES_REVISION,
            "filename": TINYSTORIES_FILENAME,
            "url": TINYSTORIES_URL,
            "bytes": TINYSTORIES_BYTES,
            "sha256": TINYSTORIES_SHA256,
            "stories_examined": examined,
        },
        "split_policy": {
            "input": "canonical story bytes before UTF-8 decoding or tokenization",
            "digest": "BLAKE3",
            "bucket": "big-endian integer(full 32-byte digest) mod 100",
            "train": "0..89",
            "dev": "90..94",
            "test": "95..99",
        },
        "tokenizer": {
            "family": "Hugging Face byte-level BPE",
            "vocab_size": FROZEN_MODEL_CONFIG.vocab_size,
            "training_split": "train only",
            "training_story_bytes_cap": TOKENIZER_TRAIN_BYTES,
            "training_stories": tokenizer_summary.stories,
            "training_story_bytes": tokenizer_summary.story_bytes,
            "training_ordered_story_bytes_cid": f"blake3:{tokenizer_summary._digest.hexdigest()}",
            "normalizer": None,
            "post_processor": None,
            "bos": {"token": BOS_TOKEN, "id": BOS_TOKEN_ID, "insertion": "explicit by corpus/generator"},
            "eos": {"token": EOS_TOKEN, "id": EOS_TOKEN_ID, "insertion": "explicit by corpus/generator"},
            "unk": {"token": UNK_TOKEN, "id": UNK_TOKEN_ID},
        },
        "splits": split_payload,
        "sealed_test_reveal_budget": {
            "scored_store_token_ids": TEST_TOKEN_CAP,
            "global_lowest_cid_prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_token_ids": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
            "hard_cap": TEST_REVEAL_TOTAL_CAP,
        },
        "full_snapshot_test_population": global_test_population,
        "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
    }
    payload["split_policy_cid"] = cid_bytes(canonical_json_bytes(payload["split_policy"]))
    relative_paths = [
        TOKENIZER_RELATIVE_PATH,
        *TOKEN_RELATIVE_PATHS.values(),
        *INDEX_RELATIVE_PATHS.values(),
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    dataset_manifest = write_bound_manifest(
        manifest_path,
        payload,
        artifact_root=root,
        relative_paths=relative_paths,
    )
    records_by_path = {
        str(record["path"]): record for record in dataset_manifest["artifacts"]
    }
    training_paths = [
        TOKENIZER_RELATIVE_PATH,
        TOKEN_RELATIVE_PATHS["train"],
        TOKEN_RELATIVE_PATHS["dev"],
        INDEX_RELATIVE_PATHS["train"],
        INDEX_RELATIVE_PATHS["dev"],
    ]
    sealed_paths = [
        TOKEN_RELATIVE_PATHS["test"],
        INDEX_RELATIVE_PATHS["test"],
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    training_payload: dict[str, object] = {
        "schema": TRAINING_VIEW_MANIFEST_SCHEMA,
        "dataset_manifest_cid": dataset_manifest["manifest_cid"],
        "split_policy": payload["split_policy"],
        "split_policy_cid": payload["split_policy_cid"],
        "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
        "sealed_test_commitment": {
            "tokens": counts["test"],
            "prompt_tokens": SEALED_PROMPT_TOKEN_COUNT,
            "total_reveal_tokens": counts["test"] + SEALED_PROMPT_TOKEN_COUNT,
            "full_snapshot_test_population": global_test_population,
            "artifacts": [records_by_path[path] for path in sealed_paths],
            "access_policy": "trainer does not open the full dataset manifest or sealed-test paths",
        },
    }
    training_view = write_bound_manifest(
        training_view_path,
        training_payload,
        artifact_root=root,
        relative_paths=training_paths,
    )
    return {"dataset": dataset_manifest, "training_view": training_view}


def load_dataset_manifest(root: Path) -> dict[str, object]:
    manifest = verify_bound_manifest(root / DATASET_MANIFEST_NAME, artifact_root=root)
    if manifest.get("schema") != DATASET_MANIFEST_SCHEMA:
        raise ValueError("unsupported dataset manifest schema")
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("dataset/model contract differs from frozen #1014 geometry")
    return manifest


def load_training_view_manifest(
    root: Path, *, verify_development: bool = True
) -> dict[str, object]:
    """Verify allowed training artifacts only; never open a sealed-test path."""
    manifest = verify_manifest_envelope(root / TRAINING_VIEW_MANIFEST_NAME)
    if manifest.get("schema") != TRAINING_VIEW_MANIFEST_SCHEMA:
        raise ValueError("unsupported training-view manifest schema")
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("training-view/model contract differs from frozen #1014 geometry")
    training_paths = [
        TOKENIZER_RELATIVE_PATH,
        TOKEN_RELATIVE_PATHS["train"],
        INDEX_RELATIVE_PATHS["train"],
    ]
    if verify_development:
        training_paths.extend(
            [TOKEN_RELATIVE_PATHS["dev"], INDEX_RELATIVE_PATHS["dev"]]
        )
    verify_artifact_subset(manifest, artifact_root=root, relative_paths=training_paths)
    return manifest


def read_story_index(path: Path) -> list[dict[str, object]]:
    records: list[dict[str, object]] = []
    with path.open("r", encoding="utf-8") as source:
        for line_number, line in enumerate(source, start=1):
            try:
                record = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValueError(f"invalid story index line {line_number}: {path}") from error
            records.append(record)
    return records
