"""Construction-only fit slices for the #973 fuller retained decoder.

The predecessor loader deliberately consumes only the already verified fit view
from ``issue-973-group-retention``.  Construction validation is a disjoint slice
of that fit store; it is not the physically sealed 64-story model-evaluation
partition, and this module has no API that can open that partition.
"""

from __future__ import annotations

import json
import struct
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import torch
from torch import Tensor

from .group_retention_data import (
    FIT_INDEX_RELATIVE_PATH,
    FIT_STORY_COUNT,
    FIT_TOKENS_RELATIVE_PATH,
    TOKENS_PER_STORY as PREDECESSOR_TOKENS_PER_STORY,
    load_group_retention_training_view,
)
from .provenance import canonical_json_bytes, cid_bytes


ISSUE = 973
CONTEXT = 128
TOKENS_PER_STORY = CONTEXT + 1
STORIES_PER_PARTITION = 32
DECISIONS_PER_PARTITION = STORIES_PER_PARTITION * CONTEXT
EXCLUDED_PRIOR_SMOKE_ORDINALS = tuple(range(0, 8))
TRAIN_ORDINALS = tuple(range(8, 40))
VALIDATION_ORDINALS = tuple(range(40, 72))

TRAIN_TOKENS_RELATIVE_PATH = "construction/train.u16"
TRAIN_INDEX_RELATIVE_PATH = "construction/train-index.jsonl"
VALIDATION_TOKENS_RELATIVE_PATH = "construction/validation.u16"
VALIDATION_INDEX_RELATIVE_PATH = "construction/validation-index.jsonl"

EXPECTED_PREDECESSOR_TRAINING_VIEW_CID = (
    "blake3:ce26777ed9fa8d25410b3f27acf30a0b33d9d725d9d3fb1e614137bf91581f31"
)
EXPECTED_PREDECESSOR_POPULATION_CID = (
    "blake3:35af5002bfbe92d68403e2cf8742fae4a22b7d6b11109a3f861fab9e15d2b52e"
)
EXPECTED_PREDECESSOR_FIT_STORE_CID = (
    "blake3:3ce77ac0b15dd3173add6382dd070016a880e8258821951a9ba9bbffa03ea43c"
)
EXPECTED_PREDECESSOR_FIT_INDEX_CID = (
    "blake3:73ba637e007c404ab19084ddf627b4082c4d5ab93fe468dbe42b087e29d9c12b"
)
EXPECTED_TOKENIZER_CID = (
    "blake3:3f42bcfce7728512076549c63b88387e13c8156fe35c0f91d9b112439f3739cc"
)
EXPECTED_GEOMETRY_ARTIFACT_CID = (
    "blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b"
)


@dataclass(frozen=True, slots=True)
class PredecessorFitRecord:
    ordinal: int
    story_cid: str
    span_cid: str
    copied_token_offset: int
    copied_token_count: int


@dataclass(frozen=True, slots=True)
class ConstructionPartition:
    name: str
    ordinals: tuple[int, ...]
    tokens: bytes
    index: bytes
    story_cids: tuple[str, ...]
    span_cids: tuple[str, ...]

    @property
    def decisions(self) -> int:
        return len(self.ordinals) * CONTEXT


@dataclass(frozen=True, slots=True)
class ConstructionData:
    predecessor: Mapping[str, Any]
    train: ConstructionPartition
    validation: ConstructionPartition


def _artifact_record(manifest: Mapping[str, Any], path: str) -> Mapping[str, Any]:
    artifacts = manifest.get("artifacts")
    records = [
        record
        for record in artifacts if isinstance(record, Mapping) and record.get("path") == path
    ] if isinstance(artifacts, list) else []
    if len(records) != 1:
        raise ValueError(f"predecessor training view must bind exactly one {path}")
    return records[0]


def _parse_fit_index(value: bytes) -> tuple[PredecessorFitRecord, ...]:
    records: list[PredecessorFitRecord] = []
    for line_number, line in enumerate(value.splitlines(keepends=True), start=1):
        if not line.endswith(b"\n"):
            raise ValueError(f"predecessor fit-index line {line_number} lacks a newline")
        try:
            item = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ValueError(f"invalid predecessor fit-index line {line_number}") from error
        if not isinstance(item, dict) or canonical_json_bytes(item) != line:
            raise ValueError(f"predecessor fit-index line {line_number} is not canonical JSON")
        expected_ordinal = len(records)
        ordinal = item.get("partition_ordinal")
        offset = item.get("copied_token_offset")
        count = item.get("copied_token_count")
        story_cid = item.get("story_cid")
        span_cid = item.get("span_cid")
        if (
            isinstance(ordinal, bool)
            or not isinstance(ordinal, int)
            or ordinal != expected_ordinal
            or isinstance(offset, bool)
            or not isinstance(offset, int)
            or offset != expected_ordinal * PREDECESSOR_TOKENS_PER_STORY
            or count != PREDECESSOR_TOKENS_PER_STORY
            or not isinstance(story_cid, str)
            or not story_cid.startswith("blake3:")
            or not isinstance(span_cid, str)
            or not span_cid.startswith("blake3:")
        ):
            raise ValueError(f"predecessor fit-index line {line_number} differs from its contract")
        records.append(
            PredecessorFitRecord(
                ordinal=ordinal,
                story_cid=story_cid,
                span_cid=span_cid,
                copied_token_offset=offset,
                copied_token_count=count,
            )
        )
    if len(records) != FIT_STORY_COUNT:
        raise ValueError("predecessor fit index does not contain exactly 256 stories")
    return tuple(records)


def _partition(
    store: bytes,
    records: Sequence[PredecessorFitRecord],
    *,
    name: str,
    ordinals: tuple[int, ...],
) -> ConstructionPartition:
    token_parts: list[bytes] = []
    index_parts: list[bytes] = []
    story_cids: list[str] = []
    span_cids: list[str] = []
    for construction_ordinal, source_ordinal in enumerate(ordinals):
        record = records[source_ordinal]
        full_start = record.copied_token_offset * 2
        full_end = full_start + PREDECESSOR_TOKENS_PER_STORY * 2
        full_story = store[full_start:full_end]
        if len(full_story) != PREDECESSOR_TOKENS_PER_STORY * 2:
            raise ValueError(f"predecessor fit story {source_ordinal} has a short token span")
        if cid_bytes(full_story) != record.span_cid:
            raise ValueError(f"predecessor fit story {source_ordinal} span CID differs")
        selected = full_story[: TOKENS_PER_STORY * 2]
        values = tuple(item[0] for item in struct.iter_unpack("<H", selected))
        if len(values) != TOKENS_PER_STORY or any(value >= 4_096 for value in values):
            raise ValueError(f"construction story {source_ordinal} has invalid tokens")
        selected_cid = cid_bytes(selected)
        token_parts.append(selected)
        index_parts.append(
            canonical_json_bytes(
                {
                    "construction_ordinal": construction_ordinal,
                    "construction_partition": name,
                    "copied_token_count": TOKENS_PER_STORY,
                    "copied_token_offset": construction_ordinal * TOKENS_PER_STORY,
                    "scored_next_tokens": CONTEXT,
                    "selected_span_cid": selected_cid,
                    "source_fit_ordinal": source_ordinal,
                    "source_full_span_cid": record.span_cid,
                    "story_cid": record.story_cid,
                }
            )
        )
        story_cids.append(record.story_cid)
        span_cids.append(selected_cid)
    return ConstructionPartition(
        name=name,
        ordinals=ordinals,
        tokens=b"".join(token_parts),
        index=b"".join(index_parts),
        story_cids=tuple(story_cids),
        span_cids=tuple(span_cids),
    )


def build_decoder_construction_data(predecessor_root: Path) -> ConstructionData:
    """Verify the immutable predecessor and build two disjoint fit-only slices."""
    predecessor_root = predecessor_root.resolve()
    training_view = load_group_retention_training_view(predecessor_root)
    fit_store_record = _artifact_record(training_view, FIT_TOKENS_RELATIVE_PATH)
    fit_index_record = _artifact_record(training_view, FIT_INDEX_RELATIVE_PATH)
    if (
        training_view.get("manifest_cid") != EXPECTED_PREDECESSOR_TRAINING_VIEW_CID
        or training_view.get("population_manifest_cid") != EXPECTED_PREDECESSOR_POPULATION_CID
        or fit_store_record.get("cid") != EXPECTED_PREDECESSOR_FIT_STORE_CID
        or fit_index_record.get("cid") != EXPECTED_PREDECESSOR_FIT_INDEX_CID
        or training_view.get("source", {}).get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("fuller-decoder predecessor identities differ from the frozen contract")
    fit_store = (predecessor_root / FIT_TOKENS_RELATIVE_PATH).read_bytes()
    fit_index = (predecessor_root / FIT_INDEX_RELATIVE_PATH).read_bytes()
    if cid_bytes(fit_store) != EXPECTED_PREDECESSOR_FIT_STORE_CID:
        raise ValueError("fuller-decoder predecessor fit store CID differs")
    if cid_bytes(fit_index) != EXPECTED_PREDECESSOR_FIT_INDEX_CID:
        raise ValueError("fuller-decoder predecessor fit index CID differs")
    records = _parse_fit_index(fit_index)
    train = _partition(fit_store, records, name="train", ordinals=TRAIN_ORDINALS)
    validation = _partition(
        fit_store, records, name="validation", ordinals=VALIDATION_ORDINALS
    )
    if set(train.story_cids) & set(validation.story_cids):
        raise ValueError("construction train and validation story identities overlap")
    predecessor = {
        "training_view_manifest_cid": EXPECTED_PREDECESSOR_TRAINING_VIEW_CID,
        "population_manifest_cid": EXPECTED_PREDECESSOR_POPULATION_CID,
        "fit_store_cid": EXPECTED_PREDECESSOR_FIT_STORE_CID,
        "fit_index_cid": EXPECTED_PREDECESSOR_FIT_INDEX_CID,
        "tokenizer_cid": EXPECTED_TOKENIZER_CID,
    }
    return ConstructionData(predecessor=predecessor, train=train, validation=validation)


def decode_construction_tensor(value: bytes, *, partition: str) -> Tensor:
    expected_bytes = STORIES_PER_PARTITION * TOKENS_PER_STORY * 2
    if len(value) != expected_bytes:
        raise ValueError(f"{partition} construction store has the wrong byte length")
    tokens = tuple(item[0] for item in struct.iter_unpack("<H", value))
    if len(tokens) != STORIES_PER_PARTITION * TOKENS_PER_STORY or any(
        token >= 4_096 for token in tokens
    ):
        raise ValueError(f"{partition} construction store has invalid tokens")
    return torch.tensor(tokens, dtype=torch.long).view(
        STORIES_PER_PARTITION, TOKENS_PER_STORY
    )
