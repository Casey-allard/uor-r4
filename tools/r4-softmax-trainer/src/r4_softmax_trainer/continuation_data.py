"""Fresh, non-replayed corpus tranches for the frozen #1017 continuation."""

from __future__ import annotations

import json
import os
import shutil
import sys
from array import array
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from blake3 import blake3

from .constants import (
    DATASET_MANIFEST_SCHEMA,
    FROZEN_MODEL_CONFIG,
    SEALED_PROMPT_COUNT,
    SEALED_PROMPT_TOKEN_COUNT,
    SEALED_PROMPT_TOKENS_PER_STORY,
    TEST_TOKEN_CAP,
    TINYSTORIES_BYTES,
    TINYSTORIES_FILENAME,
    TINYSTORIES_REPOSITORY,
    TINYSTORIES_REVISION,
    TINYSTORIES_SHA256,
    TINYSTORIES_URL,
)
from .data import (
    BOS_TOKEN_ID,
    EOS_TOKEN_ID,
    IndexWriter,
    StorySummary,
    U16Writer,
    iter_canonical_stories,
    story_split,
    validate_tokenizer_json,
    verify_source,
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


CONTINUATION_DATASET_MANIFEST_NAME = "continuation-dataset-manifest.json"
CONTINUATION_TRAINING_VIEW_MANIFEST_NAME = "continuation-training-view-manifest.json"
CONTINUATION_DATASET_MANIFEST_SCHEMA = "uor-r4-softmax-trainer-continuation-dataset/1"
CONTINUATION_TRAINING_VIEW_MANIFEST_SCHEMA = (
    "uor-r4-softmax-trainer-continuation-training-view/1"
)

INHERITED_CHECKPOINT_RELATIVE_PATH = "inherited/best.pt"
INHERITED_WEIGHTS_RELATIVE_PATH = "inherited/model.safetensors"
TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"
TOKEN_RELATIVE_PATHS = {
    "train": "tokens/train.u16",
    "dev": "tokens/dev.u16",
    "test": "sealed-confirmation/test.u16",
}
INDEX_RELATIVE_PATHS = {
    "train": "indexes/train.jsonl",
    "dev": "indexes/dev.jsonl",
    "test": "sealed-confirmation/test-index.jsonl",
}
SEALED_PROMPT_RELATIVE_PATH = "sealed-confirmation/prompts.json"
SEALED_DIRECTORY_RELATIVE_PATH = "sealed-confirmation"
SEALED_DENIAL_RELATIVE_PATH = "preflight/sealed-confirmation-denial.json"
SEALED_DENIAL_SCHEMA = "uor-r4-softmax-trainer-sealed-confirmation-denial/1"
FRESH_POPULATION_STATUS_RELATIVE_PATH = "preflight/fresh-population-status.json"

PREDECESSOR_DATASET_MANIFEST_CID = (
    "blake3:3e4d2ddb006771e5be0d4c76580c8971e6c67a23f8e223da8d81668d03bd9a01"
)
PREDECESSOR_SPLIT_POLICY_CID = (
    "blake3:54f0886d3e906a4aeeaa9328ff236440d61d9f16b2f92dcb8c05cac96e54d1aa"
)
PREDECESSOR_CHECKPOINT_CID = (
    "blake3:9c36e109d8dee67deec0362307ba4a471c967ff574835210f87653d628c95c91"
)
PREDECESSOR_WEIGHTS_CID = (
    "blake3:7d7c26e1a71866dc46973cea3b23b819f4b5060b345d2a0ec1bd067aa493bb7d"
)
PREDECESSOR_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
PREDECESSOR_PROMPT_CIDS = (
    "blake3:0000268631c8e69c7b66380e559bcf9b3cba35d034c3b8fea4d91a8995722d45",
    "blake3:000041b3c7fd5fc959421bcc59cf4b4bc5051f49f1e943d3938f31f2d5110e7d",
    "blake3:00006032bb21c0ed1ff8902efb50793bcd596341fcc6380f35aa7d2b33929e88",
    "blake3:0000ad9ff89a4a7cd105828d8e1ae952b49889444b3b3928fa3973e1ae7fe3d3",
    "blake3:00010aef5fb4f8edc9f7f9b29d644b4ab9c278bccd53b47b9584b2e01aca14ef",
)

PREDECESSOR_TOKEN_CAPS = {"train": 30_000_000, "dev": 250_000, "test": 249_880}
CONTINUATION_TOKEN_CAPS = {"train": 119_996_416, "dev": 250_000, "test": TEST_TOKEN_CAP}
PREDECESSOR_TOKEN_PATHS = {
    "train": "tokens/train.u16",
    "dev": "tokens/dev.u16",
    "test": "sealed-test/test.u16",
}


class FreshPopulationUnavailable(RuntimeError):
    """The pinned source cannot supply the exact frozen #1017 population."""

    terminal = "UNAVAILABLE_FRESH_POPULATION"


class _U16Digest:
    """Incrementally reproduce a little-endian uint16 artifact without writing it."""

    def __init__(self) -> None:
        self.digest = blake3()
        self.count = 0

    def append(self, values: list[int]) -> None:
        payload = array("H", values)
        if sys.byteorder != "little":
            payload.byteswap()
        self.digest.update(payload.tobytes())
        self.count += len(values)

    @property
    def cid(self) -> str:
        return f"blake3:{self.digest.hexdigest()}"


@dataclass(slots=True)
class _SplitState:
    predecessor: _U16Digest = field(default_factory=_U16Digest)
    predecessor_complete: bool = False
    split_story_ordinal: int = 0
    predecessor_stories_consumed: int = 0
    predecessor_last_story_cid: str | None = None
    predecessor_last_source_story_ordinal: int | None = None
    continuation_stories: int = 0
    first_story_cid: str | None = None
    last_story_cid: str | None = None
    first_source_story_ordinal: int | None = None
    last_source_story_ordinal: int | None = None


def _copy_verified(source: Path, destination: Path, expected_cid: str) -> None:
    if cid_file(source) != expected_cid:
        raise ValueError(f"inherited artifact CID mismatch: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = destination.with_name(f".{destination.name}.part")
    try:
        with source.open("rb") as reader, temporary.open("wb") as writer:
            shutil.copyfileobj(reader, writer, length=8 * 1024 * 1024)
            writer.flush()
            os.fsync(writer.fileno())
        if cid_file(temporary) != expected_cid:
            raise ValueError(f"copied inherited artifact CID mismatch: {destination}")
        os.replace(temporary, destination)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def _artifact_record(manifest: dict[str, Any], path: str) -> dict[str, Any]:
    matches = [record for record in manifest["artifacts"] if record.get("path") == path]
    if len(matches) != 1:
        raise ValueError(f"predecessor manifest does not bind exactly one {path}")
    return matches[0]


def _deny_sealed_confirmation(root: Path, *, dataset_manifest_cid: str) -> dict[str, Any]:
    """Make the fresh confirmation physically unreadable before training."""
    directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    directory.chmod(0)
    denial_verified = False
    try:
        with (root / TOKEN_RELATIVE_PATHS["test"]).open("rb"):
            pass
    except PermissionError:
        denial_verified = True
    if not denial_verified:
        directory.chmod(0o700)
        raise RuntimeError("sealed-confirmation permission denial did not hold")
    value: dict[str, Any] = {
        "schema": SEALED_DENIAL_SCHEMA,
        "issue": 1017,
        "continuation_dataset_manifest_cid": dataset_manifest_cid,
        "directory": SEALED_DIRECTORY_RELATIVE_PATH,
        "directory_mode": "000",
        "read_attempt": "PERMISSION_DENIED",
        "sealed_paths": [
            TOKEN_RELATIVE_PATHS["test"],
            INDEX_RELATIVE_PATHS["test"],
            SEALED_PROMPT_RELATIVE_PATH,
        ],
        "training_or_selection_reads": 0,
    }
    value["result_cid"] = cid_bytes(canonical_json_bytes(value))
    atomic_write_json(root / SEALED_DENIAL_RELATIVE_PATH, value)
    return value


def open_sealed_confirmation(root: Path) -> dict[str, Any]:
    """Unlock the fresh confirmation after the durable reveal marker exists."""
    denial = verify_manifest_envelope(root / CONTINUATION_TRAINING_VIEW_MANIFEST_NAME)
    verify_artifact_subset(
        denial,
        artifact_root=root,
        relative_paths=[SEALED_DENIAL_RELATIVE_PATH],
    )
    record = json.loads(
        (root / SEALED_DENIAL_RELATIVE_PATH).read_text(encoding="utf-8")
    )
    unsigned = dict(record)
    expected = unsigned.pop("result_cid", None)
    if (
        record.get("schema") != SEALED_DENIAL_SCHEMA
        or expected != cid_bytes(canonical_json_bytes(unsigned))
        or record.get("read_attempt") != "PERMISSION_DENIED"
        or record.get("training_or_selection_reads") != 0
    ):
        raise ValueError("sealed-confirmation denial record does not reproduce")
    directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    if directory.stat().st_mode & 0o777 != 0:
        raise ValueError("sealed-confirmation directory was unlocked before reveal")
    directory.chmod(0o700)
    return record


def _capped_ids(token_ids: list[int], remaining: int) -> tuple[list[int], bool]:
    """Mirror #1014's exact-cap rule and report whether the story was consumed."""
    if remaining < 1:
        return [], False
    if remaining == 1:
        return [EOS_TOKEN_ID], False
    if len(token_ids) > remaining:
        return [BOS_TOKEN_ID, *token_ids[1:-1][: remaining - 2], EOS_TOKEN_ID], True
    return token_ids, True


def _prompt_candidate(tokenizer: Any, story_cid: str, story: bytes) -> dict[str, Any] | None:
    text = story.decode("utf-8", errors="strict")
    content_ids = tokenizer.encode(text, add_special_tokens=False).ids
    if len(content_ids) < SEALED_PROMPT_TOKENS_PER_STORY:
        return None
    prompt_ids = content_ids[:SEALED_PROMPT_TOKENS_PER_STORY]
    prompt_text = tokenizer.decode(prompt_ids, skip_special_tokens=True)
    prompt_text.encode("utf-8", errors="strict")
    if tokenizer.encode(prompt_text, add_special_tokens=False).ids != prompt_ids:
        return None
    return {"story_cid": story_cid, "token_ids": prompt_ids, "text": prompt_text}


def _consider_prompt(
    selected: list[dict[str, Any]], tokenizer: Any, story_cid: str, story: bytes
) -> None:
    if story_cid in PREDECESSOR_PROMPT_CIDS:
        return
    if len(selected) == SEALED_PROMPT_COUNT and story_cid >= selected[-1]["story_cid"]:
        return
    candidate = _prompt_candidate(tokenizer, story_cid, story)
    if candidate is None:
        return
    selected.append(candidate)
    selected.sort(key=lambda record: str(record["story_cid"]))
    del selected[SEALED_PROMPT_COUNT:]


def _build_continuation_stores(
    source: Path,
    tokenizer: Any,
    root: Path,
    predecessor_records: dict[str, dict[str, Any]],
) -> dict[str, Any]:
    """Reproduce old stores in memory while writing the next story-aligned tranches."""
    writers = {name: U16Writer(root / path) for name, path in TOKEN_RELATIVE_PATHS.items()}
    indexes = {name: IndexWriter(root / path) for name, path in INDEX_RELATIVE_PATHS.items()}
    summaries = {name: StorySummary() for name in TOKEN_RELATIVE_PATHS}
    states = {name: _SplitState() for name in TOKEN_RELATIVE_PATHS}
    padding_tokens = {name: 0 for name in TOKEN_RELATIVE_PATHS}
    selected_prompts: list[dict[str, Any]] = []
    full_test_stories = 0
    full_test_story_bytes = 0
    ordered_test_cids = blake3()
    examined = 0
    try:
        for source_ordinal, story in enumerate(iter_canonical_stories(source)):
            examined += 1
            split = story_split(story)
            state = states[split]
            split_ordinal = state.split_story_ordinal
            state.split_story_ordinal += 1
            story_digest = blake3(story).digest()
            story_cid = f"blake3:{story_digest.hex()}"
            if split == "test":
                full_test_stories += 1
                full_test_story_bytes += len(story)
                ordered_test_cids.update(story_digest)

            writer = writers[split]
            if (
                state.predecessor_complete
                and writer.count >= CONTINUATION_TOKEN_CAPS[split]
            ):
                if split == "test":
                    _consider_prompt(selected_prompts, tokenizer, story_cid, story)
                continue

            text = story.decode("utf-8", errors="strict")
            content_ids = tokenizer.encode(text, add_special_tokens=False).ids
            token_ids = [BOS_TOKEN_ID, *content_ids, EOS_TOKEN_ID]
            include_current = state.predecessor_complete
            if not state.predecessor_complete:
                remaining = PREDECESSOR_TOKEN_CAPS[split] - state.predecessor.count
                old_ids, consumed_story = _capped_ids(token_ids, remaining)
                state.predecessor.append(old_ids)
                if consumed_story:
                    state.predecessor_stories_consumed += 1
                    state.predecessor_last_story_cid = story_cid
                    state.predecessor_last_source_story_ordinal = source_ordinal
                if state.predecessor.count == PREDECESSOR_TOKEN_CAPS[split]:
                    state.predecessor_complete = True
                    # A one-token EOS filler consumed no part of this story.
                    include_current = not consumed_story
                else:
                    continue
                if consumed_story:
                    continue

            if not include_current or writer.count >= CONTINUATION_TOKEN_CAPS[split]:
                continue
            remaining = CONTINUATION_TOKEN_CAPS[split] - writer.count
            next_ids, consumed_story = _capped_ids(token_ids, remaining)
            if not consumed_story:
                writer.append(next_ids)
                padding_tokens[split] += len(next_ids)
                continue
            truncated = len(next_ids) < len(token_ids)
            offset = writer.count
            writer.append(next_ids)
            summaries[split].add(story)
            summaries[split].truncated_final_story |= truncated
            state.continuation_stories += 1
            state.first_story_cid = state.first_story_cid or story_cid
            state.last_story_cid = story_cid
            state.first_source_story_ordinal = (
                source_ordinal
                if state.first_source_story_ordinal is None
                else state.first_source_story_ordinal
            )
            state.last_source_story_ordinal = source_ordinal
            indexes[split].append(
                {
                    "content_token_count": max(0, len(next_ids) - 2),
                    "content_token_offset": offset + 1,
                    "continuation_story_ordinal": state.continuation_stories - 1,
                    "source_story_ordinal": source_ordinal,
                    "split_story_ordinal": split_ordinal,
                    "story_cid": story_cid,
                    "story_token_count": len(next_ids),
                    "story_token_offset": offset,
                    "truncated": truncated,
                }
            )

        counts = {name: writer.count for name, writer in writers.items()}
        predecessor_reproduction: dict[str, Any] = {}
        for name, digest in ((name, state.predecessor) for name, state in states.items()):
            expected = predecessor_records[name]
            reproduced = {"bytes": digest.count * 2, "cid": digest.cid, "tokens": digest.count}
            if reproduced["tokens"] != PREDECESSOR_TOKEN_CAPS[name]:
                raise FreshPopulationUnavailable(
                    f"UNAVAILABLE_FRESH_POPULATION: predecessor {name} cap was unavailable"
                )
            if reproduced["bytes"] != expected.get("bytes") or reproduced[
                "cid"
            ] != expected.get("cid"):
                raise ValueError(f"reconstructed predecessor {name} store does not reproduce")
            predecessor_reproduction[name] = {
                "artifact_path": PREDECESSOR_TOKEN_PATHS[name],
                "expected_bytes": expected["bytes"],
                "expected_cid": expected["cid"],
                "reproduced_bytes": reproduced["bytes"],
                "reproduced_cid": reproduced["cid"],
                "reproduced_tokens": reproduced["tokens"],
                "stories_consumed": states[name].predecessor_stories_consumed,
                "last_consumed_story_cid": states[name].predecessor_last_story_cid,
                "last_consumed_source_story_ordinal": (
                    states[name].predecessor_last_source_story_ordinal
                ),
            }
        if counts != CONTINUATION_TOKEN_CAPS or len(selected_prompts) != SEALED_PROMPT_COUNT:
            raise FreshPopulationUnavailable(
                "UNAVAILABLE_FRESH_POPULATION: pinned source did not supply "
                "exact continuation caps/prompts"
            )
        for writer in writers.values():
            writer.finish()
        for index in indexes.values():
            index.finish()
        prompt_fixture: dict[str, Any] = {
            "schema": "uor-r4-softmax-trainer-continuation-sealed-prompts/1",
            "selection": (
                "first 24 content tokens of the five lowest eligible test-story CIDs "
                "strictly after both the #1014 and #1017 NLL-store populations, while "
                "also excluding the five published #1014 prompt CIDs"
            ),
            "excluded_predecessor_prompt_cids": list(PREDECESSOR_PROMPT_CIDS),
            "revealed_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "tokenizer_cid": PREDECESSOR_TOKENIZER_CID,
            "prompts": selected_prompts,
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

    split_payload: dict[str, Any] = {}
    for name in ("train", "dev", "test"):
        state = states[name]
        summary = summaries[name]
        if (
            state.first_source_story_ordinal is None
            or (
                state.predecessor_last_source_story_ordinal is not None
                and state.first_source_story_ordinal
                <= state.predecessor_last_source_story_ordinal
            )
        ):
            raise ValueError(f"{name} continuation is not source-order disjoint")
        split_payload[name] = {
            "tokens": writers[name].count,
            "token_cap": CONTINUATION_TOKEN_CAPS[name],
            "stories": state.continuation_stories,
            "story_bytes": summary.story_bytes,
            "ordered_story_bytes_cid": f"blake3:{summary._digest.hexdigest()}",
            "first_story_cid": state.first_story_cid,
            "last_story_cid": state.last_story_cid,
            "first_source_story_ordinal": state.first_source_story_ordinal,
            "last_source_story_ordinal": state.last_source_story_ordinal,
            "predecessor_last_source_story_ordinal": (
                state.predecessor_last_source_story_ordinal
            ),
            "source_ordinal_disjoint": True,
            "cap_padding_tokens": padding_tokens[name],
            "truncated_final_story": summary.truncated_final_story,
            "complete_context_scored_next_tokens": (
                (writers[name].count - 1) // FROZEN_MODEL_CONFIG.max_position_embeddings
            )
            * FROZEN_MODEL_CONFIG.max_position_embeddings,
            "boundary_policy": (
                "start at the first split story after the predecessor cap-consuming story; "
                "a predecessor one-token EOS filler consumes no part of the current story"
            ),
        }
    return {
        "counts": {name: writer.count for name, writer in writers.items()},
        "splits": split_payload,
        "predecessor_reproduction": predecessor_reproduction,
        "stories_examined": examined,
        "full_snapshot_test_population": {
            "stories": full_test_stories,
            "story_bytes": full_test_story_bytes,
            "ordered_full_story_cids_cid": f"blake3:{ordered_test_cids.hexdigest()}",
        },
    }


def prepare_continuation_dataset(
    root: Path,
    *,
    predecessor_root: Path,
    source: Path | None = None,
    force: bool = False,
) -> dict[str, Any]:
    """Build #1017 without ever opening a #1014 sealed-test artifact."""
    from tokenizers import Tokenizer

    root = root.resolve()
    predecessor_root = predecessor_root.resolve()
    if root == predecessor_root:
        raise ValueError("continuation root must differ from the immutable #1014 root")
    manifest_path = root / CONTINUATION_DATASET_MANIFEST_NAME
    training_view_path = root / CONTINUATION_TRAINING_VIEW_MANIFEST_NAME
    population_status_path = root / FRESH_POPULATION_STATUS_RELATIVE_PATH
    if manifest_path.is_file() and training_view_path.is_file() and not force:
        return {
            "dataset": verify_manifest_envelope(manifest_path),
            "training_view": load_continuation_training_view_manifest(root),
        }
    sealed_directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    if population_status_path.exists():
        if not force:
            raise FileExistsError("fresh population is terminal; use --force only for a new build")
        population_status_path.unlink()
    if sealed_directory.exists():
        sealed_directory.chmod(0o700)

    predecessor = verify_manifest_envelope(predecessor_root / "dataset-manifest.json")
    if predecessor.get("schema") != DATASET_MANIFEST_SCHEMA:
        raise ValueError("unsupported predecessor dataset manifest schema")
    if predecessor.get("manifest_cid") != PREDECESSOR_DATASET_MANIFEST_CID:
        raise ValueError("predecessor dataset manifest CID differs from frozen #1014")
    if predecessor.get("split_policy_cid") != PREDECESSOR_SPLIT_POLICY_CID:
        raise ValueError("predecessor split policy differs from frozen #1014")
    if predecessor.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("predecessor model contract differs from frozen #1014")

    tokenizer_source = predecessor_root / "tokenizer/tokenizer.json"
    checkpoint_source = predecessor_root / "checkpoints/best.pt"
    weights_source = predecessor_root / "export/model.safetensors"
    source_path = (source or predecessor_root / "raw" / TINYSTORIES_FILENAME).resolve()
    verify_source(source_path)
    if _artifact_record(predecessor, "tokenizer/tokenizer.json").get("cid") != (
        PREDECESSOR_TOKENIZER_CID
    ):
        raise ValueError("predecessor envelope binds an unexpected tokenizer")

    root.mkdir(parents=True, exist_ok=True)
    _copy_verified(tokenizer_source, root / TOKENIZER_RELATIVE_PATH, PREDECESSOR_TOKENIZER_CID)
    _copy_verified(
        checkpoint_source,
        root / INHERITED_CHECKPOINT_RELATIVE_PATH,
        PREDECESSOR_CHECKPOINT_CID,
    )
    _copy_verified(weights_source, root / INHERITED_WEIGHTS_RELATIVE_PATH, PREDECESSOR_WEIGHTS_CID)
    validate_tokenizer_json(root / TOKENIZER_RELATIVE_PATH)
    tokenizer = Tokenizer.from_file(str(root / TOKENIZER_RELATIVE_PATH))
    predecessor_records = {
        name: _artifact_record(predecessor, path) for name, path in PREDECESSOR_TOKEN_PATHS.items()
    }
    try:
        built = _build_continuation_stores(
            source_path, tokenizer, root, predecessor_records
        )
    except FreshPopulationUnavailable as error:
        status: dict[str, Any] = {
            "schema": "uor-r4-softmax-trainer-fresh-population-unavailable/1",
            "issue": 1017,
            "terminal": error.terminal,
            "reason": str(error),
            "training_permitted": False,
        }
        status["result_cid"] = cid_bytes(canonical_json_bytes(status))
        atomic_write_json(root / FRESH_POPULATION_STATUS_RELATIVE_PATH, status)
        raise
    if built["full_snapshot_test_population"] != predecessor.get(
        "full_snapshot_test_population"
    ):
        raise ValueError("full test population does not reproduce the frozen #1014 source/split")

    inherited = {
        "dataset_manifest_cid": PREDECESSOR_DATASET_MANIFEST_CID,
        "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
        "checkpoint_cid": PREDECESSOR_CHECKPOINT_CID,
        "weights_cid": PREDECESSOR_WEIGHTS_CID,
        "tokenizer_cid": PREDECESSOR_TOKENIZER_CID,
    }
    payload: dict[str, Any] = {
        "schema": CONTINUATION_DATASET_MANIFEST_SCHEMA,
        "predecessor": inherited,
        "source": {
            "repository": TINYSTORIES_REPOSITORY,
            "revision": TINYSTORIES_REVISION,
            "filename": TINYSTORIES_FILENAME,
            "url": TINYSTORIES_URL,
            "bytes": TINYSTORIES_BYTES,
            "sha256": TINYSTORIES_SHA256,
            "stories_examined": built["stories_examined"],
        },
        "split_policy": predecessor["split_policy"],
        "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
        "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
        "splits": built["splits"],
        "predecessor_reproduction": built["predecessor_reproduction"],
        "sealed_confirmation_budget": {
            "scored_store_token_ids": TEST_TOKEN_CAP,
            "prompt_token_ids": SEALED_PROMPT_TOKEN_COUNT,
            "total_revealed_token_ids": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
            "hard_cap": 250_000,
        },
        "full_snapshot_test_population": built["full_snapshot_test_population"],
        "freshness": {
            "population_order": (
                "canonical source order within the inherited pre-tokenization split"
            ),
            "predecessor_store_access": (
                "envelope-only; token bytes reconstructed from pinned raw source and tokenizer"
            ),
            "predecessor_sealed_paths_opened": 0,
        },
    }
    all_paths = [
        INHERITED_CHECKPOINT_RELATIVE_PATH,
        INHERITED_WEIGHTS_RELATIVE_PATH,
        TOKENIZER_RELATIVE_PATH,
        *TOKEN_RELATIVE_PATHS.values(),
        *INDEX_RELATIVE_PATHS.values(),
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    dataset = write_bound_manifest(
        manifest_path, payload, artifact_root=root, relative_paths=all_paths
    )
    denial = _deny_sealed_confirmation(
        root, dataset_manifest_cid=str(dataset["manifest_cid"])
    )
    records_by_path = {str(record["path"]): record for record in dataset["artifacts"]}
    training_paths = [
        TOKENIZER_RELATIVE_PATH,
        INHERITED_CHECKPOINT_RELATIVE_PATH,
        TOKEN_RELATIVE_PATHS["train"],
        TOKEN_RELATIVE_PATHS["dev"],
        INDEX_RELATIVE_PATHS["train"],
        INDEX_RELATIVE_PATHS["dev"],
        SEALED_DENIAL_RELATIVE_PATH,
    ]
    sealed_paths = [
        TOKEN_RELATIVE_PATHS["test"],
        INDEX_RELATIVE_PATHS["test"],
        SEALED_PROMPT_RELATIVE_PATH,
    ]
    training_view = write_bound_manifest(
        training_view_path,
        {
            "schema": CONTINUATION_TRAINING_VIEW_MANIFEST_SCHEMA,
            "continuation_dataset_manifest_cid": dataset["manifest_cid"],
            "predecessor": inherited,
            "split_policy": predecessor["split_policy"],
            "split_policy_cid": PREDECESSOR_SPLIT_POLICY_CID,
            "model_contract": FROZEN_MODEL_CONFIG.as_contract(),
            "sealed_confirmation_commitment": {
                "tokens": TEST_TOKEN_CAP,
                "prompt_tokens": SEALED_PROMPT_TOKEN_COUNT,
                "total_reveal_tokens": TEST_TOKEN_CAP + SEALED_PROMPT_TOKEN_COUNT,
                "full_snapshot_test_population": built["full_snapshot_test_population"],
                "artifacts": [records_by_path[path] for path in sealed_paths],
                "access_policy": (
                    "directory mode 000; training and selection verify this envelope "
                    "but do not open sealed-confirmation paths"
                ),
                "denial_result_cid": denial["result_cid"],
            },
        },
        artifact_root=root,
        relative_paths=training_paths,
    )
    return {"dataset": dataset, "training_view": training_view}


def load_continuation_dataset_manifest(root: Path) -> dict[str, Any]:
    manifest = verify_bound_manifest(
        root / CONTINUATION_DATASET_MANIFEST_NAME, artifact_root=root
    )
    if manifest.get("schema") != CONTINUATION_DATASET_MANIFEST_SCHEMA:
        raise ValueError("unsupported continuation dataset manifest schema")
    if manifest.get("predecessor", {}).get("dataset_manifest_cid") != (
        PREDECESSOR_DATASET_MANIFEST_CID
    ):
        raise ValueError("continuation dataset has the wrong predecessor")
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("continuation dataset/model contract differs from #1014")
    return manifest


def load_continuation_training_view_manifest(root: Path) -> dict[str, Any]:
    """Verify only continuation training inputs; never open sealed confirmation."""
    manifest = verify_manifest_envelope(root / CONTINUATION_TRAINING_VIEW_MANIFEST_NAME)
    if manifest.get("schema") != CONTINUATION_TRAINING_VIEW_MANIFEST_SCHEMA:
        raise ValueError("unsupported continuation training-view schema")
    if manifest.get("predecessor", {}).get("checkpoint_cid") != PREDECESSOR_CHECKPOINT_CID:
        raise ValueError("continuation training view has the wrong inherited checkpoint")
    if manifest.get("model_contract") != FROZEN_MODEL_CONFIG.as_contract():
        raise ValueError("continuation training-view/model contract differs from #1014")
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=[
            TOKENIZER_RELATIVE_PATH,
            INHERITED_CHECKPOINT_RELATIVE_PATH,
            TOKEN_RELATIVE_PATHS["train"],
            TOKEN_RELATIVE_PATHS["dev"],
            INDEX_RELATIVE_PATHS["train"],
            INDEX_RELATIVE_PATHS["dev"],
            SEALED_DENIAL_RELATIVE_PATH,
        ],
    )
    denial = json.loads((root / SEALED_DENIAL_RELATIVE_PATH).read_text(encoding="utf-8"))
    unsigned = dict(denial)
    expected = unsigned.pop("result_cid", None)
    commitment = manifest.get("sealed_confirmation_commitment", {})
    if (
        denial.get("schema") != SEALED_DENIAL_SCHEMA
        or expected != cid_bytes(canonical_json_bytes(unsigned))
        or denial.get("read_attempt") != "PERMISSION_DENIED"
        or denial.get("training_or_selection_reads") != 0
        or commitment.get("denial_result_cid") != denial.get("result_cid")
    ):
        raise ValueError("sealed-confirmation denial record does not reproduce")
    directory = root / SEALED_DIRECTORY_RELATIVE_PATH
    if directory.stat().st_mode & 0o777 != 0:
        raise ValueError("sealed-confirmation directory is readable during training")
    return manifest
