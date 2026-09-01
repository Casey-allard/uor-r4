"""Deterministic, create-once natural-language population for #973.

This module derives the bounded ``R4GroupAddressedRetentionLMV1`` population
from the already frozen #1017 train store.  It deliberately keeps the full
population manifest and held-out bytes outside the training view: fitting can
reproduce every admitted byte without opening a held-out path, and the held-out
directory remains physically sealed.  The terminal package exposes no reveal
API because the construction gate emitted no main authorization.
"""

from __future__ import annotations

import json
import re
import struct
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Protocol

from .provenance import (
    atomic_write,
    atomic_write_json,
    canonical_json_bytes,
    cid_bytes,
    tree_cid,
    verify_artifact_subset,
    verify_manifest_envelope,
    write_bound_manifest,
)


ISSUE = 973
FIT_STORY_COUNT = 256
HELDOUT_STORY_COUNT = 64
SELECTED_STORY_COUNT = FIT_STORY_COUNT + HELDOUT_STORY_COUNT
TOKENS_PER_STORY = 257
TARGETS_PER_STORY = TOKENS_PER_STORY - 1
VOCAB_SIZE = 4096

EXPECTED_DATASET_MANIFEST_CID = (
    "blake3:5f709a9ef886801c55c799cd6f684774dc87a3e9e192f148d254c3d20a394aec"
)
EXPECTED_SOURCE_TREE_CID = (
    "blake3:3820b702f05dc23102b2a732d9255cd1965b1451755e0e3bb091b406ea8a795f"
)
EXPECTED_TRAIN_STORE_CID = (
    "blake3:b18679a2d8efc005ff96c5dc3f7652693fea461489f46afc19b29a87a74ad6c6"
)
EXPECTED_TRAIN_INDEX_CID = (
    "blake3:f422386cff6425e9b44336559942a16e4b286ddc41c64db798d77f488ba6d46a"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)

SOURCE_MANIFEST_NAME = "continuation-dataset-manifest.json"
SOURCE_TRAIN_TOKENS_RELATIVE_PATH = "tokens/train.u16"
SOURCE_TRAIN_INDEX_RELATIVE_PATH = "indexes/train.jsonl"
SOURCE_TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"

POPULATION_MANIFEST_NAME = "group-retention-population-manifest.json"
TRAINING_VIEW_MANIFEST_NAME = "group-retention-training-view-manifest.json"
FIT_TOKENS_RELATIVE_PATH = "population/fit.u16"
FIT_INDEX_RELATIVE_PATH = "population/fit-index.jsonl"
HELDOUT_DIRECTORY_RELATIVE_PATH = "sealed-heldout"
HELDOUT_TOKENS_RELATIVE_PATH = "sealed-heldout/heldout.u16"
HELDOUT_INDEX_RELATIVE_PATH = "sealed-heldout/heldout-index.jsonl"
TOKENIZER_RELATIVE_PATH = "tokenizer/tokenizer.json"
HELDOUT_DENIAL_RELATIVE_PATH = "preflight/heldout-denial.json"

POPULATION_MANIFEST_SCHEMA = "uor-r4-group-addressed-retention-population/1"
TRAINING_VIEW_MANIFEST_SCHEMA = "uor-r4-group-addressed-retention-training-view/1"
HELDOUT_DENIAL_SCHEMA = "uor-r4-group-addressed-retention-heldout-denial/1"

_CID_PATTERN = re.compile(r"blake3:[0-9a-f]{64}\Z")


class PopulationUnavailable(RuntimeError):
    """The frozen source cannot produce the exact admitted population."""

    terminal = "UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET"


@dataclass(frozen=True, slots=True)
class StorySpan:
    """One complete indexed story span in the frozen #1017 train store."""

    story_cid: str
    story_token_offset: int
    story_token_count: int
    truncated: bool
    index_line: int


@dataclass(frozen=True, slots=True)
class PartitionBytes:
    """Pure deterministic bytes for one selected population partition."""

    tokens: bytes
    index: bytes
    records: tuple[dict[str, Any], ...]
    stories: tuple[tuple[int, ...], ...]


class PopulationGeometry(Protocol):
    """Adapter for aggregate geometry facts owned by another implementation."""

    def population_signatures(
        self,
        *,
        fit_stories: tuple[tuple[int, ...], ...],
        heldout_stories: tuple[tuple[int, ...], ...],
    ) -> Mapping[str, Any]:
        """Return aggregate, label-free facts; never raw tokens, text, or targets."""


def _is_cid(value: object) -> bool:
    return isinstance(value, str) and _CID_PATTERN.fullmatch(value) is not None


def _json_object_without_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key in train index: {key}")
        value[key] = item
    return value


def _required_int(record: Mapping[str, Any], field: str, *, line_number: int) -> int:
    value = record.get(field)
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"train index line {line_number} has invalid {field}")
    return value


def parse_train_index_bytes(value: bytes) -> tuple[StorySpan, ...]:
    """Parse and validate canonical #1017 train-index metadata without token reads."""
    try:
        text = value.decode("utf-8", errors="strict")
    except UnicodeDecodeError as error:
        raise ValueError("train index is not valid UTF-8") from error
    if not text or not text.endswith("\n"):
        raise ValueError("train index must be nonempty canonical JSON-lines")

    spans: list[StorySpan] = []
    seen_cids: set[str] = set()
    prior_end = 0
    for line_number, line in enumerate(text.splitlines(), start=1):
        if not line:
            raise ValueError(f"train index line {line_number} is empty")
        try:
            record = json.loads(line, object_pairs_hook=_json_object_without_duplicate_keys)
        except (json.JSONDecodeError, ValueError) as error:
            raise ValueError(f"invalid train index line {line_number}") from error
        if not isinstance(record, dict):
            raise ValueError(f"train index line {line_number} is not an object")
        if canonical_json_bytes(record) != f"{line}\n".encode("utf-8"):
            raise ValueError(f"train index line {line_number} is not canonical JSON")

        story_cid = record.get("story_cid")
        if not _is_cid(story_cid):
            raise ValueError(f"train index line {line_number} has invalid story_cid")
        if story_cid in seen_cids:
            raise ValueError(f"train index repeats story CID {story_cid}")
        seen_cids.add(story_cid)

        offset = _required_int(record, "story_token_offset", line_number=line_number)
        count = _required_int(record, "story_token_count", line_number=line_number)
        truncated = record.get("truncated")
        if not isinstance(truncated, bool):
            raise ValueError(f"train index line {line_number} has invalid truncated flag")
        if count < 2:
            raise ValueError(f"train index line {line_number} has an incomplete story")
        if offset < prior_end:
            raise ValueError(f"train index line {line_number} overlaps a prior story")
        prior_end = offset + count
        spans.append(
            StorySpan(
                story_cid=story_cid,
                story_token_offset=offset,
                story_token_count=count,
                truncated=truncated,
                index_line=line_number,
            )
        )
    return tuple(spans)


def select_story_spans(
    records: Sequence[StorySpan],
) -> tuple[tuple[StorySpan, ...], tuple[StorySpan, ...]]:
    """Select the 320 lowest eligible CIDs, then freeze the 256/64 split."""
    eligible = sorted(
        (
            record
            for record in records
            if record.story_token_count >= TOKENS_PER_STORY and not record.truncated
        ),
        key=lambda record: record.story_cid,
    )
    if len(eligible) < SELECTED_STORY_COUNT:
        raise PopulationUnavailable(
            f"frozen train index has only {len(eligible)} eligible complete stories; "
            f"requires {SELECTED_STORY_COUNT}"
        )
    selected = eligible[:SELECTED_STORY_COUNT]
    fit = tuple(selected[:FIT_STORY_COUNT])
    heldout = tuple(selected[FIT_STORY_COUNT:])
    if {record.story_cid for record in fit} & {
        record.story_cid for record in heldout
    }:
        raise ValueError("fit and held-out stories overlap")
    return fit, heldout


def _decode_story_tokens(story_bytes: bytes, *, story_cid: str) -> tuple[int, ...]:
    expected_bytes = TOKENS_PER_STORY * 2
    if len(story_bytes) != expected_bytes:
        raise ValueError(f"story {story_cid} did not yield exactly {TOKENS_PER_STORY} u16 tokens")
    tokens = tuple(value[0] for value in struct.iter_unpack("<H", story_bytes))
    if len(tokens) != TOKENS_PER_STORY:
        raise ValueError(f"story {story_cid} token decode length differs")
    if any(token >= VOCAB_SIZE for token in tokens):
        raise PopulationUnavailable(
            f"story {story_cid} contains a token outside frozen range 0..{VOCAB_SIZE - 1}"
        )
    return tokens


def _build_partition_from_chunks(
    spans: Sequence[StorySpan],
    chunks: Sequence[bytes],
    *,
    partition: str,
) -> PartitionBytes:
    if partition not in {"fit", "heldout"}:
        raise ValueError("partition must be fit or heldout")
    if len(spans) != len(chunks):
        raise ValueError("story spans and token chunks differ in length")

    token_parts: list[bytes] = []
    index_parts: list[bytes] = []
    records: list[dict[str, Any]] = []
    stories: list[tuple[int, ...]] = []
    for ordinal, (span, chunk) in enumerate(zip(spans, chunks, strict=True)):
        tokens = _decode_story_tokens(chunk, story_cid=span.story_cid)
        copied_token_offset = ordinal * TOKENS_PER_STORY
        record: dict[str, Any] = {
            "copied_token_count": TOKENS_PER_STORY,
            "copied_token_offset": copied_token_offset,
            "index_line": span.index_line,
            "partition": partition,
            "partition_ordinal": ordinal,
            "scored_next_tokens": TARGETS_PER_STORY,
            "source_story_token_count": span.story_token_count,
            "source_story_token_offset": span.story_token_offset,
            "span_cid": cid_bytes(chunk),
            "story_cid": span.story_cid,
            "truncated": False,
        }
        token_parts.append(chunk)
        index_parts.append(canonical_json_bytes(record))
        records.append(record)
        stories.append(tokens)
    return PartitionBytes(
        tokens=b"".join(token_parts),
        index=b"".join(index_parts),
        records=tuple(records),
        stories=tuple(stories),
    )


def build_partition_bytes(
    store_bytes: bytes,
    spans: Sequence[StorySpan],
    *,
    partition: str,
) -> PartitionBytes:
    """Pure builder used to prove byte-identical population reconstruction."""
    chunks: list[bytes] = []
    for span in spans:
        start = span.story_token_offset * 2
        end = start + TOKENS_PER_STORY * 2
        if start < 0 or end > len(store_bytes):
            raise ValueError(f"story span escapes train store: {span.story_cid}")
        chunks.append(store_bytes[start:end])
    return _build_partition_from_chunks(spans, chunks, partition=partition)


def _read_partition(
    store_path: Path,
    spans: Sequence[StorySpan],
    *,
    partition: str,
) -> PartitionBytes:
    chunks: list[bytes] = []
    store_size = store_path.stat().st_size
    with store_path.open("rb") as source:
        for span in spans:
            start = span.story_token_offset * 2
            length = TOKENS_PER_STORY * 2
            if start < 0 or start + length > store_size:
                raise ValueError(f"story span escapes train store: {span.story_cid}")
            source.seek(start)
            chunk = source.read(length)
            if len(chunk) != length:
                raise ValueError(f"short train-store read for story {span.story_cid}")
            chunks.append(chunk)
    return _build_partition_from_chunks(spans, chunks, partition=partition)


def _artifact_record(manifest: Mapping[str, Any], path: str) -> Mapping[str, Any]:
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list):
        raise ValueError("source manifest has no artifact records")
    records = [
        record
        for record in artifacts
        if isinstance(record, dict) and record.get("path") == path
    ]
    if len(records) != 1:
        raise ValueError(f"source manifest does not bind exactly one {path}")
    return records[0]


def _verify_source(source_root: Path) -> dict[str, Any]:
    manifest = verify_manifest_envelope(source_root / SOURCE_MANIFEST_NAME)
    if manifest.get("manifest_cid") != EXPECTED_DATASET_MANIFEST_CID:
        raise ValueError("#1017 dataset manifest CID differs from live #973")
    if manifest.get("tree_cid") != EXPECTED_SOURCE_TREE_CID:
        raise ValueError("#1017 source tree CID differs from live #973")
    expected = {
        SOURCE_TRAIN_TOKENS_RELATIVE_PATH: EXPECTED_TRAIN_STORE_CID,
        SOURCE_TRAIN_INDEX_RELATIVE_PATH: EXPECTED_TRAIN_INDEX_CID,
        SOURCE_TOKENIZER_RELATIVE_PATH: EXPECTED_TOKENIZER_CID,
    }
    for path, expected_cid in expected.items():
        record = _artifact_record(manifest, path)
        if record.get("cid") != expected_cid:
            raise ValueError(f"#1017 manifest binds an unexpected {path} CID")
    verify_artifact_subset(
        manifest,
        artifact_root=source_root,
        relative_paths=expected,
    )
    return manifest


def _validate_aggregate(value: Any, *, path: str = "geometry") -> None:
    """Keep the optional pre-seal geometry record aggregate and label-free."""
    if isinstance(value, Mapping):
        for key, item in value.items():
            if not isinstance(key, str):
                raise ValueError(f"{path} contains a non-string key")
            if key in {"text", "token_ids", "tokens", "targets", "story_cids"}:
                raise ValueError(f"{path}.{key} would expose held-out content")
            _validate_aggregate(item, path=f"{path}.{key}")
        return
    if value is None or isinstance(value, (bool, int, str)):
        return
    if isinstance(value, float) and value == value and abs(value) != float("inf"):
        return
    raise ValueError(f"{path} must contain aggregate JSON scalars and mappings only")


def _geometry_record(
    geometry: PopulationGeometry | None,
    *,
    fit: PartitionBytes,
    heldout: PartitionBytes,
) -> dict[str, Any]:
    if geometry is None:
        return {"status": "NOT_COMPUTED"}
    summary = dict(
        geometry.population_signatures(
            fit_stories=fit.stories,
            heldout_stories=heldout.stories,
        )
    )
    _validate_aggregate(summary)
    return {
        "status": "COMPUTED",
        "summary": summary,
        "summary_cid": cid_bytes(canonical_json_bytes(summary)),
    }


def _read_tokenizer(source: Path) -> bytes:
    value = source.read_bytes()
    if cid_bytes(value) != EXPECTED_TOKENIZER_CID:
        raise ValueError("#1017 tokenizer changed during population construction")
    return value


def _seal_heldout(root: Path, *, population_manifest_cid: str) -> dict[str, Any]:
    directory = root / HELDOUT_DIRECTORY_RELATIVE_PATH
    directory.chmod(0)
    denied = False
    try:
        with (root / HELDOUT_TOKENS_RELATIVE_PATH).open("rb"):
            pass
    except PermissionError:
        denied = True
    if not denied:
        # Leave the bytes fail-closed even on a platform where the active
        # process can bypass ordinary permission bits.
        directory.chmod(0)
        raise RuntimeError("#973 held-out permission denial did not hold")
    record: dict[str, Any] = {
        "schema": HELDOUT_DENIAL_SCHEMA,
        "issue": ISSUE,
        "population_manifest_cid": population_manifest_cid,
        "directory": HELDOUT_DIRECTORY_RELATIVE_PATH,
        "directory_mode": "000",
        "read_attempt": "PERMISSION_DENIED",
        "sealed_paths": [
            HELDOUT_TOKENS_RELATIVE_PATH,
            HELDOUT_INDEX_RELATIVE_PATH,
        ],
        "training_reads": 0,
    }
    record["result_cid"] = cid_bytes(canonical_json_bytes(record))
    atomic_write_json(root / HELDOUT_DENIAL_RELATIVE_PATH, record)
    return record


def prepare_group_retention_population(
    root: Path,
    source_root: Path,
    geometry: PopulationGeometry | None = None,
) -> dict[str, Any]:
    """Create and physically seal the sole 256-fit/64-held-out #973 population."""
    root = root.resolve()
    source_root = source_root.resolve()
    if root == source_root:
        raise ValueError("#973 population root must differ from immutable #1017 source")
    managed_paths = [
        root / POPULATION_MANIFEST_NAME,
        root / TRAINING_VIEW_MANIFEST_NAME,
        root / "population",
        root / HELDOUT_DIRECTORY_RELATIVE_PATH,
        root / "tokenizer",
        root / "preflight",
        root / "reveal",
    ]
    if any(path.exists() or path.is_symlink() for path in managed_paths):
        raise FileExistsError("#973 population is create-once; use a new empty root")

    source_manifest = _verify_source(source_root)
    index_path = source_root / SOURCE_TRAIN_INDEX_RELATIVE_PATH
    records = parse_train_index_bytes(index_path.read_bytes())
    fit_spans, heldout_spans = select_story_spans(records)
    train_store = source_root / SOURCE_TRAIN_TOKENS_RELATIVE_PATH
    fit = _read_partition(train_store, fit_spans, partition="fit")
    heldout = _read_partition(train_store, heldout_spans, partition="heldout")
    if {record.story_cid for record in fit_spans} & {
        record.story_cid for record in heldout_spans
    }:
        raise ValueError("fit and held-out story identities overlap")

    tokenizer_bytes = _read_tokenizer(source_root / SOURCE_TOKENIZER_RELATIVE_PATH)
    selection = {
        "policy": (
            "320 lowest story CIDs with story_token_count>=257 and truncated=false; "
            "first 256 fit, next 64 heldout; first 257 u16 tokens per story"
        ),
        "fit": list(fit.records),
        "heldout": list(heldout.records),
    }
    payload: dict[str, Any] = {
        "schema": POPULATION_MANIFEST_SCHEMA,
        "issue": ISSUE,
        "source": {
            "dataset_manifest_cid": EXPECTED_DATASET_MANIFEST_CID,
            "tree_cid": EXPECTED_SOURCE_TREE_CID,
            "train_store_cid": EXPECTED_TRAIN_STORE_CID,
            "train_index_cid": EXPECTED_TRAIN_INDEX_CID,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "verified_manifest_schema": source_manifest.get("schema"),
        },
        "population": {
            "selected_stories": SELECTED_STORY_COUNT,
            "fit_stories": FIT_STORY_COUNT,
            "heldout_stories": HELDOUT_STORY_COUNT,
            "tokens_per_story": TOKENS_PER_STORY,
            "targets_per_story": TARGETS_PER_STORY,
            "fit_targets": FIT_STORY_COUNT * TARGETS_PER_STORY,
            "heldout_targets": HELDOUT_STORY_COUNT * TARGETS_PER_STORY,
            "vocab_size": VOCAB_SIZE,
            "story_disjoint": True,
            "truncated_stories": 0,
            "maximum_token_id": max(
                token for story in (*fit.stories, *heldout.stories) for token in story
            ),
        },
        "selection": selection,
        "selection_cid": cid_bytes(canonical_json_bytes(selection)),
        "geometry": _geometry_record(geometry, fit=fit, heldout=heldout),
    }
    artifact_values = {
        TOKENIZER_RELATIVE_PATH: tokenizer_bytes,
        FIT_TOKENS_RELATIVE_PATH: fit.tokens,
        FIT_INDEX_RELATIVE_PATH: fit.index,
        HELDOUT_TOKENS_RELATIVE_PATH: heldout.tokens,
        HELDOUT_INDEX_RELATIVE_PATH: heldout.index,
    }
    artifact_records = [
        {"bytes": len(value), "cid": cid_bytes(value), "path": path}
        for path, value in sorted(artifact_values.items())
    ]
    population = dict(payload)
    population["artifacts"] = artifact_records
    population["tree_cid"] = tree_cid(artifact_records)
    population["manifest_cid"] = cid_bytes(canonical_json_bytes(population))

    root.mkdir(parents=True, exist_ok=True)
    atomic_write(root / FIT_TOKENS_RELATIVE_PATH, fit.tokens)
    atomic_write(root / FIT_INDEX_RELATIVE_PATH, fit.index)
    atomic_write(root / TOKENIZER_RELATIVE_PATH, tokenizer_bytes)
    # Materialize both held-out files as one fail-closed operation.  The
    # population manifest was built from their pure bytes above, so no later
    # construction step needs to reopen this directory.
    try:
        atomic_write(root / HELDOUT_TOKENS_RELATIVE_PATH, heldout.tokens)
        atomic_write(root / HELDOUT_INDEX_RELATIVE_PATH, heldout.index)
    finally:
        directory = root / HELDOUT_DIRECTORY_RELATIVE_PATH
        if directory.exists() and not directory.is_symlink():
            directory.chmod(0)
    atomic_write_json(root / POPULATION_MANIFEST_NAME, population)
    try:
        denial = _seal_heldout(
            root, population_manifest_cid=str(population["manifest_cid"])
        )
        records_by_path = {
            str(record["path"]): record for record in population["artifacts"]
        }
        training_paths = [
            TOKENIZER_RELATIVE_PATH,
            FIT_TOKENS_RELATIVE_PATH,
            FIT_INDEX_RELATIVE_PATH,
            HELDOUT_DENIAL_RELATIVE_PATH,
        ]
        training_view = write_bound_manifest(
            root / TRAINING_VIEW_MANIFEST_NAME,
            {
                "schema": TRAINING_VIEW_MANIFEST_SCHEMA,
                "issue": ISSUE,
                "population_manifest_cid": population["manifest_cid"],
                "selection_cid": population["selection_cid"],
                "source": population["source"],
                "fit": {
                    "stories": FIT_STORY_COUNT,
                    "targets": FIT_STORY_COUNT * TARGETS_PER_STORY,
                    "story_cids": [record.story_cid for record in fit_spans],
                },
                "heldout_commitment": {
                    "stories": HELDOUT_STORY_COUNT,
                    "targets": HELDOUT_STORY_COUNT * TARGETS_PER_STORY,
                    "artifacts": [
                        records_by_path[HELDOUT_TOKENS_RELATIVE_PATH],
                        records_by_path[HELDOUT_INDEX_RELATIVE_PATH],
                    ],
                    "access_policy": (
                        "directory mode 000 until all three fitted artifact commitments "
                        "are self-CID-bound"
                    ),
                    "denial_result_cid": denial["result_cid"],
                },
                "geometry": population["geometry"],
            },
            artifact_root=root,
            relative_paths=training_paths,
        )
    except BaseException:
        directory = root / HELDOUT_DIRECTORY_RELATIVE_PATH
        if directory.exists() and not directory.is_symlink():
            directory.chmod(0)
        raise
    return {"population": population, "training_view": training_view}


def load_group_retention_training_view(root: Path) -> dict[str, Any]:
    """Verify fit artifacts and physical sealing without opening held-out bytes."""
    root = root.resolve()
    manifest = verify_manifest_envelope(root / TRAINING_VIEW_MANIFEST_NAME)
    if (
        manifest.get("schema") != TRAINING_VIEW_MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("source", {}).get("dataset_manifest_cid")
        != EXPECTED_DATASET_MANIFEST_CID
        or manifest.get("source", {}).get("tree_cid") != EXPECTED_SOURCE_TREE_CID
        or manifest.get("source", {}).get("train_store_cid")
        != EXPECTED_TRAIN_STORE_CID
        or manifest.get("source", {}).get("train_index_cid")
        != EXPECTED_TRAIN_INDEX_CID
        or manifest.get("source", {}).get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("#973 training-view identity differs from live contract")
    training_paths = [
        TOKENIZER_RELATIVE_PATH,
        FIT_TOKENS_RELATIVE_PATH,
        FIT_INDEX_RELATIVE_PATH,
        HELDOUT_DENIAL_RELATIVE_PATH,
    ]
    verify_artifact_subset(manifest, artifact_root=root, relative_paths=training_paths)

    denial = json.loads((root / HELDOUT_DENIAL_RELATIVE_PATH).read_text(encoding="utf-8"))
    unsigned = dict(denial)
    result_cid = unsigned.pop("result_cid", None)
    commitment = manifest.get("heldout_commitment")
    if (
        not isinstance(commitment, dict)
        or denial.get("schema") != HELDOUT_DENIAL_SCHEMA
        or denial.get("issue") != ISSUE
        or result_cid != cid_bytes(canonical_json_bytes(unsigned))
        or denial.get("population_manifest_cid")
        != manifest.get("population_manifest_cid")
        or denial.get("read_attempt") != "PERMISSION_DENIED"
        or denial.get("training_reads") != 0
        or commitment.get("denial_result_cid") != denial.get("result_cid")
    ):
        raise ValueError("#973 held-out denial record does not reproduce")
    if (root / HELDOUT_DIRECTORY_RELATIVE_PATH).stat().st_mode & 0o777 != 0:
        raise ValueError("#973 held-out directory is readable during fitting")
    return manifest
