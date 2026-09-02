"""Fail-closed, create-once execution campaign for issue #1043.

The campaign has four monotonic phases: prepare and commit every population,
preflight the serialization/mechanics/CPU contract, fit the one frozen mixed
objective, then reveal and score the terminal population.  The terminal data
loader is never called before the fitted artifact CID exists, and no optimizer
can be constructed after a reveal marker exists.

Population construction lives in :mod:`position_kv_binding_data`; model
mechanics live in :mod:`position_kv_binding`.  The small bridge functions in
this module are intentionally the only place that knows their concrete data
containers.  That keeps phase and provenance enforcement independent of the
serialization implementation.
"""

from __future__ import annotations

import json
import math
import multiprocessing as mp
import os
import platform
import resource
import time
import traceback
from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass, fields, is_dataclass
from pathlib import Path
from typing import Any, Literal

import torch
from blake3 import blake3
from torch import Tensor
from torch.nn import functional as F

from .group_retention_campaign import load_group_geometry_artifacts
from .h4_spin_frame_sidecar import H4SpinFrameArtifactV1
from .language_path_generalization import (
    CONTEXT,
    HEAD_DIM,
    HEADS,
    LAYERS,
    PARAMETER_COUNT,
    VOCAB_SIZE,
)
from .position_kv_binding import R4PositionPreservingCausalKVBindingV1
from .provenance import (
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    trainer_implementation_contract,
)


ISSUE = 1043
POLICY = "R4PositionPreservingCausalKVBindingV1"
SEED = 10_043
IGNORE_INDEX = -100

OPTIMIZER_STEPS = 2_730
BATCH_SIZE = 16
NATURAL_BATCH = 8
MQAR_BATCH = 4
ENGLISH_BATCH = 4
ENGLISH_HISTORY_BATCH = 3
ENGLISH_NO_HISTORY_BATCH = 1
NATURAL_CONSTRUCTION_ROWS = OPTIMIZER_STEPS * NATURAL_BATCH
MQAR_CONSTRUCTION_ROWS = OPTIMIZER_STEPS * MQAR_BATCH
ENGLISH_HISTORY_CONSTRUCTION_ROWS = OPTIMIZER_STEPS * ENGLISH_HISTORY_BATCH
ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS = (
    OPTIMIZER_STEPS * ENGLISH_NO_HISTORY_BATCH
)

ADAM_BETAS = (0.9, 0.95)
ADAM_EPSILON = 1.0e-8
WEIGHT_DECAY = 0.1
GRADIENT_CLIP = 1.0
WARMUP_STEPS = 100
PEAK_LEARNING_RATE = 1.0e-4
FINAL_LEARNING_RATE = 1.0e-5
LOSS_WEIGHTS = {"natural": 0.50, "mqar": 0.25, "english": 0.25}

PROBE_WARMUP_STEPS = 1
PROBE_MEASURED_STEPS = 3
PROJECTION_SAFETY_FACTOR = 1.25
HARD_WALL_SECONDS = 1_800.0
MEMORY_CEILING_BYTES = 16 * 1024**3

INITIAL_ARTIFACT_BYTES = 1_010_800
INITIAL_ARTIFACT_CID = (
    "blake3:c1cd34b36c7df7c53915785a608ccd353a11de56eebb3ecc58e74092cb5d1933"
)
H4_FRAME_ARTIFACT_CID = (
    "blake3:f1f556d3c93a2e21593c4f48de13efd64705fec11f7660e0b6fac7ba49263099"
)
H4_FRAME_FILE_CID = (
    "blake3:9df624162d14ba133fed34c560e4828961a4dc8d6a9438c731e8f8c209c16ad4"
)
GEOMETRY_ARTIFACT_CID = (
    "blake3:55447c00c1eb86a1d05324d6c83d044407bdc89f653f46957bf6f0bccb6c000b"
)
GEOMETRY_FILE_CID = (
    "blake3:a812cf6749e637f4c486a6ad206b96c90d695b5c4bb2ed029df3c6bef147d702"
)

# The terminal TinyStories population must be story-disjoint from every
# natural-language story that informed the inherited ordinary checkpoint or a
# prior revealed #973 evaluation. These identities are public artifacts; the
# #1043 terminal payload remains sealed until the fitted artifact exists.
SOURCE_TRAIN_INDEX_CID = (
    "blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e"
)
SOURCE_DEV_INDEX_CID = (
    "blake3:bafcec396953c72c4caaed9529d9f6d6c45f039f27982edda4d762839b80f81a"
)
ORDINARY_TRAIN_SOURCE_OFFSET_TOKENS = 149_996_595
ORDINARY_TRAIN_TOKENS = 5_285_280
ORDINARY_TRAIN_STORY_COUNT = 25_879
ORDINARY_TRAIN_FIRST_CAPACITY_STORY = 734_500
ORDINARY_TRAIN_LAST_CAPACITY_STORY = 760_378
ORDINARY_TRAIN_FIRST_SOURCE_STORY = 815_766
ORDINARY_TRAIN_LAST_SOURCE_STORY = 844_443
ORDINARY_TRAIN_STORY_CIDS_CID = (
    "blake3:a20574441f3aa7bd29609c51502cf3325ae03d05dd21b6e8e46fa4ea7cf8878c"
)
ORDINARY_DEV_SOURCE_OFFSET_TOKENS = 0
ORDINARY_DEV_TOKENS = 249_986
ORDINARY_DEV_STORY_COUNT = 1_251
ORDINARY_DEV_FIRST_CAPACITY_STORY = 0
ORDINARY_DEV_LAST_CAPACITY_STORY = 1_250
ORDINARY_DEV_FIRST_SOURCE_STORY = 47_299
ORDINARY_DEV_LAST_SOURCE_STORY = 72_670
ORDINARY_DEV_STORY_CIDS_CID = (
    "blake3:18a2de8d3e955d190f7f19ff00b40bad783074773a45bd559b3e94922b08f509"
)
V5_POPULATION_RELATIVE_PATH = "evaluation/sealed/prompt-population.json"
V5_POPULATION_FILE_CID = (
    "blake3:120719d0984b33a63904b5d72cc8b5e831b77df2eceb2f2c75b9c75750cacd10"
)
V5_POPULATION_SCHEMA = "uor-r4.retained-prompt-swap-population/5"
PRIOR_PROMPT_STORY_COUNT = 2_048
PRIOR_PROMPT_STORY_CIDS_CID = (
    "blake3:c926c19deaae20a17b05fc3c5eddc099324d9b531bbfd83ac992a5ef02ede092"
)
V5_PROMPT_STORY_COUNT = 512
V5_PROMPT_STORY_CIDS_CID = (
    "blake3:e78a4ee75b470ee946f634ef4da2edeacac2dc7b70e97c9f30610a05e1aad4e0"
)
V5_FRESH_FIRST_CAPACITY_STORY = 765_248
V5_FRESH_LAST_CAPACITY_STORY = 766_489
V5_FRESH_FIRST_SOURCE_STORY = 849_803
V5_FRESH_LAST_SOURCE_STORY = 851_190
V5_FRESH_STORY_COUNT = 1_242
V5_FRESH_STORY_CIDS_CID = (
    "blake3:07a9f3c199172d491738a2cda018a605b1e30ecb152103f2f17f3d2d7919f4dc"
)
COMPLETE_STORY_EXCLUSION_COUNT = 30_932
COMPLETE_STORY_EXCLUSION_CID = (
    "blake3:3456b61b4e16bb7bc150c110d5eb077760e7aab5d9bf91abba47b0d097290e22"
)

TERMINAL_MQAR_DECISIONS = 8_192
TERMINAL_ENGLISH_HISTORY_DECISIONS = 512
TERMINAL_ENGLISH_NO_HISTORY_DECISIONS = 512
TERMINAL_NATURAL_DECISIONS = 247_920
MQAR_REQUIRED_CORRECT = 8_110
MQAR_CONTROL_DROP = 0.50
MQAR_TRANSPORT_DROP = 0.25
ENGLISH_HISTORY_REQUIRED_CORRECT = 461
ENGLISH_HISTORY_NO_HISTORY_DROP = 0.35
ENGLISH_BINDING_PERMUTED_DROP = 0.35
ENGLISH_UNKNOWN_REQUIRED_CORRECT = 461
ENGLISH_UNSUPPORTED_ALLOWED = 0
LANGUAGE_NLL_TOLERANCE = 0.05
LANGUAGE_TOP1_TOLERANCE = 0.01
ATTENTION_PARITY_TOLERANCE = 2.0e-6
LOGIT_PARITY_TOLERANCE = 2.0e-5
TERMINAL_NATURAL_WINDOWS = 2_066
TERMINAL_MQAR_SEQUENCES = 1_024
TERMINAL_ENGLISH_HISTORY_ROWS = 512
TERMINAL_ENGLISH_NO_HISTORY_ROWS = 512
TERMINAL_PARITY_DECISIONS = (
    TERMINAL_NATURAL_DECISIONS
    + TERMINAL_MQAR_DECISIONS
    + TERMINAL_ENGLISH_HISTORY_DECISIONS
    + TERMINAL_ENGLISH_NO_HISTORY_DECISIONS
)
TERMINAL_REPLAY_DECISIONS = 16

AUDIT_WORK_FIELDS = (
    "token_steps",
    "cache_writes",
    "materialized_attention_scores",
    "admitted_attention_scores",
    "transported_r4_blocks",
    "value_reads",
    "vocabulary_scores",
    "target_reads",
    "source_reads",
    "provider_calls",
    "teacher_calls",
    "future_reads",
    "forbidden_reads",
)
EXPECTED_EVALUATION_TARGET_READS = (
    5 * TERMINAL_MQAR_DECISIONS
    + 3 * TERMINAL_ENGLISH_HISTORY_DECISIONS
    + 2 * TERMINAL_NATURAL_DECISIONS
    + 3 * TERMINAL_PARITY_DECISIONS
    + 2 * TERMINAL_REPLAY_DECISIONS
)

NATURAL_SCORE_BATCHES = math.ceil(TERMINAL_NATURAL_WINDOWS / BATCH_SIZE)
MQAR_SCORE_BATCHES = math.ceil(TERMINAL_MQAR_SEQUENCES / BATCH_SIZE)
ENGLISH_HISTORY_SCORE_BATCHES = math.ceil(
    TERMINAL_ENGLISH_HISTORY_ROWS / BATCH_SIZE
)
ENGLISH_NO_HISTORY_SCORE_BATCHES = math.ceil(
    TERMINAL_ENGLISH_NO_HISTORY_ROWS / BATCH_SIZE
)
# The scorer below performs these exact model-call counts.  Plain and R4 full
# passes are timed separately, and real incremental passes get their own much
# more conservative measurement rather than being priced as one full square.
PROJECTED_PLAIN_FULL_BATCHES = (
    2 * NATURAL_SCORE_BATCHES
    + NATURAL_SCORE_BATCHES
    + MQAR_SCORE_BATCHES
    + ENGLISH_HISTORY_SCORE_BATCHES
    + ENGLISH_NO_HISTORY_SCORE_BATCHES
)
PROJECTED_R4_FULL_BATCHES = (
    5 * MQAR_SCORE_BATCHES
    + 3 * ENGLISH_HISTORY_SCORE_BATCHES
    + NATURAL_SCORE_BATCHES
    + MQAR_SCORE_BATCHES
    + ENGLISH_HISTORY_SCORE_BATCHES
    + ENGLISH_NO_HISTORY_SCORE_BATCHES
    + 2  # both artifact-replay forwards
)
PROJECTED_INCREMENTAL_BATCHES = (
    NATURAL_SCORE_BATCHES
    + MQAR_SCORE_BATCHES
    + ENGLISH_HISTORY_SCORE_BATCHES
    + ENGLISH_NO_HISTORY_SCORE_BATCHES
)

TERMINAL_PASS = "POSITION_KV_BINDING_PASS"
TERMINAL_SYNTHETIC_ONLY = "SYNTHETIC_ONLY_NO_NATURAL_TRANSFER"
TERMINAL_LANGUAGE_REGRESSION = "BINDING_LANGUAGE_REGRESSION"
TERMINAL_NOT_LEARNED = "POSITION_KV_BINDING_NOT_LEARNED"
TERMINAL_UNATTRIBUTED = "POSITION_KV_BINDING_UNATTRIBUTED"
TERMINAL_GEOMETRY_UNATTRIBUTED = (
    "POSITION_KV_BINDING_GEOMETRY_UNATTRIBUTED"
)
TERMINAL_INVALID = "INVALID_POSITION_KV_BINDING"
TERMINAL_UNAVAILABLE = "UNAVAILABLE_COMPUTE"

INPUT_INITIAL_ARTIFACT = "inputs/ordinary-initialization.safetensors"
INPUT_GEOMETRY = "inputs/r4-group-address-geometry.json"
INPUT_H4_FRAMES = "inputs/h4-spin-frame-sidecar.json"
PREPARATION_RELATIVE_PATH = "position-kv-binding-preparation.json"
PREFLIGHT_RELATIVE_PATH = "preflight/position-kv-binding-preflight.json"
STARTED_RELATIVE_PATH = "run/position-kv-binding-started.json"
ARTIFACT_RELATIVE_PATH = "artifact/model.safetensors"
FIT_RELATIVE_PATH = "run/position-kv-binding-fit.json"
REVEAL_RELATIVE_PATH = "evaluation/reveal.json"
SCORING_RELATIVE_PATH = "run/position-kv-binding-scoring.json"
RESULT_RELATIVE_PATH = "run/position-kv-binding-result.json"

PREPARATION_SCHEMA = "uor-r4.position-kv-binding-preparation/1"
PREFLIGHT_SCHEMA = "uor-r4.position-kv-binding-preflight/1"
STARTED_SCHEMA = "uor-r4.position-kv-binding-started/1"
FIT_SCHEMA = "uor-r4.position-kv-binding-fit/1"
REVEAL_SCHEMA = "uor-r4.position-kv-binding-reveal/1"
SCORING_SCHEMA = "uor-r4.position-kv-binding-scoring/1"
RESULT_SCHEMA = "uor-r4.position-kv-binding-result/1"


@dataclass(frozen=True, slots=True)
class ExecutionPlan:
    """One eligible one-worker Apple Accelerate CPU plan."""

    name: str
    threads: int

    def identity(self) -> dict[str, Any]:
        if self.threads not in (1, 4, 8):
            raise ValueError("#1043 CPU plan must use exactly 1, 4, or 8 threads")
        body = {
            "name": self.name,
            "backend": "cpu-apple-accelerate",
            "threads": self.threads,
            "workers": 1,
            "cuda": "FORBIDDEN",
            "mps": "FORBIDDEN",
        }
        body["plan_cid"] = cid_bytes(canonical_json_bytes(body))
        return body


ELIGIBLE_PLANS = (
    ExecutionPlan("cpu-accelerate-1t", 1),
    ExecutionPlan("cpu-accelerate-4t", 4),
    ExecutionPlan("cpu-accelerate-8t", 8),
)


@dataclass(frozen=True, slots=True)
class MixedBatch:
    """The exact 8/4/4 construction batch for one optimizer step."""

    natural_inputs: Tensor
    natural_labels: Tensor
    mqar_inputs: Tensor
    mqar_labels: Tensor
    english_inputs: Tensor
    english_labels: Tensor

    def validate(self) -> None:
        expected = (
            (self.natural_inputs, self.natural_labels, NATURAL_BATCH, "natural"),
            (self.mqar_inputs, self.mqar_labels, MQAR_BATCH, "MQAR"),
            (self.english_inputs, self.english_labels, ENGLISH_BATCH, "English"),
        )
        for inputs, labels, batch, label in expected:
            if (
                inputs.dtype != torch.long
                or labels.dtype != torch.long
                or inputs.ndim != 2
                or labels.shape != inputs.shape
                or int(inputs.shape[0]) != batch
                or not 1 <= int(inputs.shape[1]) <= CONTEXT
            ):
                raise ValueError(f"#1043 {label} batch differs from the freeze")
            selected = labels != IGNORE_INDEX
            if not bool(selected.any()):
                raise ValueError(f"#1043 {label} batch has no scored labels")


def learning_rate(step: int) -> float:
    """Return the frozen linear-warmup then cosine-decay learning rate."""

    if isinstance(step, bool) or not isinstance(step, int):
        raise TypeError("optimizer step must be an integer")
    if not 1 <= step <= OPTIMIZER_STEPS:
        raise ValueError("optimizer step is outside the frozen #1043 fit")
    if step <= WARMUP_STEPS:
        return PEAK_LEARNING_RATE * step / WARMUP_STEPS
    progress = (step - WARMUP_STEPS) / (OPTIMIZER_STEPS - WARMUP_STEPS)
    cosine = 0.5 * (1.0 + math.cos(math.pi * progress))
    return FINAL_LEARNING_RATE + cosine * (
        PEAK_LEARNING_RATE - FINAL_LEARNING_RATE
    )


def _with_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = cid_bytes(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != cid_bytes(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_json(path: Path, *, cid_field: str | None = None) -> dict[str, Any]:
    raw = path.read_bytes()
    value = json.loads(raw.decode("utf-8", errors="strict"))
    if not isinstance(value, dict) or canonical_json_bytes(value) != raw:
        raise ValueError(f"JSON evidence is not a canonical object: {path}")
    if cid_field is not None:
        _verify_self_cid(value, cid_field)
    return value


def _is_blake3_cid(value: object) -> bool:
    if not isinstance(value, str) or not value.startswith("blake3:"):
        return False
    digest = value.removeprefix("blake3:")
    return len(digest) == 64 and all(
        character in "0123456789abcdef" for character in digest
    )


def _story_set_cid(values: Sequence[str] | set[str]) -> str:
    return cid_bytes(canonical_json_bytes(sorted(values)))


def _validate_complete_story_exclusions(values: Sequence[str]) -> tuple[str, ...]:
    ordered = tuple(sorted(values))
    if (
        len(ordered) != COMPLETE_STORY_EXCLUSION_COUNT
        or len(set(ordered)) != len(ordered)
        or any(not _is_blake3_cid(value) for value in ordered)
        or _story_set_cid(ordered) != COMPLETE_STORY_EXCLUSION_CID
    ):
        raise ValueError("#1043 story-exclusion union differs from the complete freeze")
    return ordered


def _canonical_index_record(line: bytes, *, label: str) -> dict[str, Any]:
    try:
        value = json.loads(line.decode("utf-8", errors="strict"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"#1043 {label} index is malformed") from error
    if not isinstance(value, dict) or canonical_json_bytes(value) != line:
        raise ValueError(f"#1043 {label} index record is not canonical")
    return value


def _index_story_identity(
    records: Sequence[tuple[int, dict[str, Any]]],
    *,
    label: str,
    expected_count: int,
    expected_first_capacity: int,
    expected_last_capacity: int,
    expected_first_source: int,
    expected_last_source: int,
    expected_cid: str,
) -> set[str]:
    if len(records) != expected_count:
        raise ValueError(f"#1043 {label} story count differs from the freeze")
    first_ordinal, first = records[0]
    last_ordinal, last = records[-1]
    if (
        first_ordinal != expected_first_capacity
        or first.get("capacity_story_ordinal") != expected_first_capacity
        or first.get("source_story_ordinal") != expected_first_source
        or last_ordinal != expected_last_capacity
        or last.get("capacity_story_ordinal") != expected_last_capacity
        or last.get("source_story_ordinal") != expected_last_source
    ):
        raise ValueError(f"#1043 {label} story boundaries differ from the freeze")
    story_cids = {str(record.get("story_cid")) for _ordinal, record in records}
    if (
        len(story_cids) != expected_count
        or any(not _is_blake3_cid(value) for value in story_cids)
        or _story_set_cid(story_cids) != expected_cid
    ):
        raise ValueError(f"#1043 {label} story-CID set differs from the freeze")
    return story_cids


def _collect_index_story_exclusions(
    source_root: Path,
) -> tuple[set[str], set[str], set[str]]:
    """Bind checkpoint train/dev stories and V5's fresh-language stories."""

    index_root = source_root.resolve() / "indexes"
    train_path = index_root / "train.jsonl"
    dev_path = index_root / "dev.jsonl"
    for path, expected, label in (
        (train_path, SOURCE_TRAIN_INDEX_CID, "train"),
        (dev_path, SOURCE_DEV_INDEX_CID, "development"),
    ):
        if path.is_symlink() or not path.is_file() or cid_file(path) != expected:
            raise ValueError(f"#1043 {label} story index differs from the freeze")

    train_end = ORDINARY_TRAIN_SOURCE_OFFSET_TOKENS + ORDINARY_TRAIN_TOKENS
    ordinary_train_records: list[tuple[int, dict[str, Any]]] = []
    v5_fresh_records: list[tuple[int, dict[str, Any]]] = []
    with train_path.open("rb") as source:
        for ordinal, line in enumerate(source):
            if ordinal < ORDINARY_TRAIN_FIRST_CAPACITY_STORY - 1:
                continue
            if ordinal > V5_FRESH_LAST_CAPACITY_STORY:
                break
            record = _canonical_index_record(line, label="train")
            if record.get("capacity_story_ordinal") != ordinal:
                raise ValueError("#1043 train index ordinal differs")
            offset = record.get("story_token_offset")
            count = record.get("story_token_count")
            if (
                isinstance(offset, bool)
                or not isinstance(offset, int)
                or isinstance(count, bool)
                or not isinstance(count, int)
                or offset < 0
                or count < 1
            ):
                raise ValueError("#1043 train index token span is malformed")
            if (
                offset + count > ORDINARY_TRAIN_SOURCE_OFFSET_TOKENS
                and offset < train_end
            ):
                ordinary_train_records.append((ordinal, record))
            if V5_FRESH_FIRST_CAPACITY_STORY <= ordinal <= V5_FRESH_LAST_CAPACITY_STORY:
                v5_fresh_records.append((ordinal, record))

    ordinary_train = _index_story_identity(
        ordinary_train_records,
        label="ordinary-checkpoint train",
        expected_count=ORDINARY_TRAIN_STORY_COUNT,
        expected_first_capacity=ORDINARY_TRAIN_FIRST_CAPACITY_STORY,
        expected_last_capacity=ORDINARY_TRAIN_LAST_CAPACITY_STORY,
        expected_first_source=ORDINARY_TRAIN_FIRST_SOURCE_STORY,
        expected_last_source=ORDINARY_TRAIN_LAST_SOURCE_STORY,
        expected_cid=ORDINARY_TRAIN_STORY_CIDS_CID,
    )
    v5_fresh = _index_story_identity(
        v5_fresh_records,
        label="V5 fresh-language",
        expected_count=V5_FRESH_STORY_COUNT,
        expected_first_capacity=V5_FRESH_FIRST_CAPACITY_STORY,
        expected_last_capacity=V5_FRESH_LAST_CAPACITY_STORY,
        expected_first_source=V5_FRESH_FIRST_SOURCE_STORY,
        expected_last_source=V5_FRESH_LAST_SOURCE_STORY,
        expected_cid=V5_FRESH_STORY_CIDS_CID,
    )

    dev_end = ORDINARY_DEV_SOURCE_OFFSET_TOKENS + ORDINARY_DEV_TOKENS
    ordinary_dev_records: list[tuple[int, dict[str, Any]]] = []
    with dev_path.open("rb") as source:
        for ordinal, line in enumerate(source):
            record = _canonical_index_record(line, label="development")
            if record.get("capacity_story_ordinal") != ordinal:
                raise ValueError("#1043 development index ordinal differs")
            offset = record.get("story_token_offset")
            count = record.get("story_token_count")
            if (
                isinstance(offset, bool)
                or not isinstance(offset, int)
                or isinstance(count, bool)
                or not isinstance(count, int)
                or offset < 0
                or count < 1
            ):
                raise ValueError("#1043 development index token span is malformed")
            if offset + count > ORDINARY_DEV_SOURCE_OFFSET_TOKENS and offset < dev_end:
                ordinary_dev_records.append((ordinal, record))
            if offset >= dev_end:
                break
    ordinary_dev = _index_story_identity(
        ordinary_dev_records,
        label="ordinary-checkpoint development",
        expected_count=ORDINARY_DEV_STORY_COUNT,
        expected_first_capacity=ORDINARY_DEV_FIRST_CAPACITY_STORY,
        expected_last_capacity=ORDINARY_DEV_LAST_CAPACITY_STORY,
        expected_first_source=ORDINARY_DEV_FIRST_SOURCE_STORY,
        expected_last_source=ORDINARY_DEV_LAST_SOURCE_STORY,
        expected_cid=ORDINARY_DEV_STORY_CIDS_CID,
    )
    return ordinary_train, ordinary_dev, v5_fresh


def _collect_prompt_story_exclusions(v5_root: Path) -> tuple[set[str], set[str]]:
    population_path = v5_root.resolve() / V5_POPULATION_RELATIVE_PATH
    if (
        population_path.is_symlink()
        or not population_path.is_file()
        or cid_file(population_path) != V5_POPULATION_FILE_CID
    ):
        raise ValueError("#1043 V5 prompt population differs from the freeze")
    population = _read_json(population_path)
    prior = population.get("prior_population_exclusions")
    pairs = population.get("pairs")
    summary = population.get("population")
    if (
        population.get("schema") != V5_POPULATION_SCHEMA
        or not isinstance(prior, Mapping)
        or not isinstance(pairs, list)
        or len(pairs) != V5_PROMPT_STORY_COUNT // 2
        or not isinstance(summary, Mapping)
        or summary.get("pairs") != V5_PROMPT_STORY_COUNT // 2
        or summary.get("directions") != V5_PROMPT_STORY_COUNT
    ):
        raise ValueError("#1043 V5 prompt population contract differs")
    prior_values = prior.get("story_cids")
    if not isinstance(prior_values, list):
        raise ValueError("#1043 prior prompt exclusion list is absent")
    prior_story_cids = {str(value) for value in prior_values}
    if (
        len(prior_values) != PRIOR_PROMPT_STORY_COUNT
        or len(prior_story_cids) != PRIOR_PROMPT_STORY_COUNT
        or any(not _is_blake3_cid(value) for value in prior_story_cids)
        or prior.get("story_cid_count") != PRIOR_PROMPT_STORY_COUNT
        or prior.get("story_cid_set_cid") != PRIOR_PROMPT_STORY_CIDS_CID
        or _story_set_cid(prior_story_cids) != PRIOR_PROMPT_STORY_CIDS_CID
    ):
        raise ValueError("#1043 V1-through-V4 prompt exclusion union differs")

    v5_story_cids: set[str] = set()
    for pair_index, pair in enumerate(pairs):
        if not isinstance(pair, Mapping) or pair.get("pair_index") != pair_index:
            raise ValueError("#1043 V5 prompt pair ordering differs")
        for side in ("left", "right"):
            record = pair.get(side)
            story_cid = record.get("story_cid") if isinstance(record, Mapping) else None
            if not _is_blake3_cid(story_cid):
                raise ValueError("#1043 V5 prompt story CID is malformed")
            v5_story_cids.add(story_cid)
    if (
        len(v5_story_cids) != V5_PROMPT_STORY_COUNT
        or _story_set_cid(v5_story_cids) != V5_PROMPT_STORY_CIDS_CID
    ):
        raise ValueError("#1043 V5 prompt story-CID set differs")
    return prior_story_cids, v5_story_cids


def collect_position_kv_story_exclusions(
    *, source_root: Path, v5_root: Path
) -> tuple[str, ...]:
    """Return the complete CID-bound story exclusion union for #1043."""

    ordinary_train, ordinary_dev, v5_fresh = _collect_index_story_exclusions(
        source_root
    )
    prior_prompts, v5_prompts = _collect_prompt_story_exclusions(v5_root)
    sources = (
        ordinary_train,
        ordinary_dev,
        prior_prompts,
        v5_prompts,
        v5_fresh,
    )
    union = set().union(*sources)
    if len(union) != sum(len(values) for values in sources):
        raise ValueError("#1043 natural-language exclusion sources overlap")
    return _validate_complete_story_exclusions(tuple(union))


def _write_exclusive(path: Path, payload: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(payload)
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def _write_exclusive_json(path: Path, value: Mapping[str, Any]) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _copy_verified_input(
    source: Path,
    destination: Path,
    *,
    expected_cid: str | None = None,
    expected_bytes: int | None = None,
) -> dict[str, Any]:
    source = source.resolve()
    if source.is_symlink() or not source.is_file():
        raise ValueError(f"#1043 input must be a regular non-symlink file: {source}")
    if expected_bytes is not None and source.stat().st_size != expected_bytes:
        raise ValueError(f"#1043 input byte count differs: {source}")
    observed = cid_file(source)
    if expected_cid is not None and observed != expected_cid:
        raise ValueError(f"#1043 input CID differs: {source}")
    _write_exclusive(destination, source.read_bytes())
    copied = cid_file(destination)
    if copied != observed:
        raise RuntimeError("#1043 copied input differs from its source")
    return {
        "path": str(destination),
        "bytes": destination.stat().st_size,
        "cid": copied,
        "source_path": str(source),
    }


def _field(value: Any, *names: str) -> Any:
    for name in names:
        if isinstance(value, Mapping) and name in value:
            return value[name]
        if hasattr(value, name):
            return getattr(value, name)
    raise AttributeError(f"#1043 data lacks any of {names}")


def _optional_field(value: Any, *names: str, default: Any = None) -> Any:
    try:
        return _field(value, *names)
    except AttributeError:
        return default


def _record(value: Any) -> dict[str, Any]:
    if isinstance(value, Mapping):
        return dict(value)
    if is_dataclass(value):
        return asdict(value)
    if hasattr(value, "__dict__"):
        return dict(vars(value))
    raise TypeError("#1043 evidence object is not record-like")


def _data_module() -> Any:
    from . import position_kv_binding_data

    return position_kv_binding_data


def _data_manifest(value: Any) -> dict[str, Any]:
    manifest = _field(value, "manifest", "preparation_manifest")
    if not isinstance(manifest, Mapping):
        raise TypeError("#1043 data preparation manifest is not a mapping")
    return dict(manifest)


def _load_geometry_and_frames(root: Path) -> tuple[Any, H4SpinFrameArtifactV1]:
    geometry = load_group_geometry_artifacts(root / INPUT_GEOMETRY).exact_h4
    frames = H4SpinFrameArtifactV1.load(root / INPUT_H4_FRAMES)
    if (
        frames.artifact_cid != H4_FRAME_ARTIFACT_CID
        or frames.file_cid != H4_FRAME_FILE_CID
    ):
        raise ValueError("#1043 H4 sidecar differs from the frozen artifact")
    return geometry, frames


def _build_model(root: Path, *, device: torch.device) -> R4PositionPreservingCausalKVBindingV1:
    geometry, frames = _load_geometry_and_frames(root)
    payload = (root / INPUT_INITIAL_ARTIFACT).read_bytes()
    if len(payload) != INITIAL_ARTIFACT_BYTES or cid_bytes(payload) != INITIAL_ARTIFACT_CID:
        raise ValueError("#1043 ordinary initialization differs from the freeze")
    model = R4PositionPreservingCausalKVBindingV1.from_learned_artifact(
        payload,
        geometry=geometry,
        frames=frames,
    ).to(device)
    if model.parameter_count() != PARAMETER_COUNT:
        raise RuntimeError("#1043 model parameter count differs")
    return model


def _manifest_cid(manifest: Mapping[str, Any]) -> str:
    value = manifest.get("manifest_cid")
    if isinstance(value, str) and value.startswith("blake3:"):
        return value
    return cid_bytes(canonical_json_bytes(manifest))


def _implementation_contract() -> dict[str, Any]:
    """Bind every executable trainer module and the locked environment."""

    implementation = trainer_implementation_contract()
    return {
        **implementation,
        "torch": torch.__version__,
        "platform": platform.platform(),
    }


_IMPORTED_IMPLEMENTATION_CONTRACT = _implementation_contract()


def _capture_prepare_implementation() -> dict[str, Any]:
    """Bind preparation to the source tree that this process imported."""

    current = _implementation_contract()
    if current != _IMPORTED_IMPLEMENTATION_CONTRACT:
        raise ValueError(
            "current #1043 implementation differs from the process-imported source"
        )
    return current


def _require_current_implementation(bound: Any) -> dict[str, Any]:
    current = _implementation_contract()
    if bound != current:
        raise ValueError("current #1043 implementation differs from the frozen phase")
    return current


def _phase_paths(root: Path) -> dict[str, Path]:
    return {
        "preparation": root / PREPARATION_RELATIVE_PATH,
        "preflight": root / PREFLIGHT_RELATIVE_PATH,
        "started": root / STARTED_RELATIVE_PATH,
        "artifact": root / ARTIFACT_RELATIVE_PATH,
        "fit": root / FIT_RELATIVE_PATH,
        "reveal": root / REVEAL_RELATIVE_PATH,
        "scoring": root / SCORING_RELATIVE_PATH,
        "result": root / RESULT_RELATIVE_PATH,
    }


def _peak_rss_bytes() -> int:
    value = int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    return value if platform.system() == "Darwin" else value * 1_024


def _configure_cpu(plan: ExecutionPlan) -> torch.device:
    plan.identity()
    if platform.system() != "Darwin":
        raise RuntimeError("#1043 CPU probe requires Darwin")
    if torch.cuda.is_available() and os.environ.get("CUDA_VISIBLE_DEVICES") not in (
        None,
        "",
        "-1",
    ):
        raise RuntimeError("CUDA is forbidden by #1043")
    build = torch.__config__.show().lower()
    if "accelerate" not in build:
        raise RuntimeError("#1043 CPU probe requires Apple Accelerate")
    os.environ["OMP_NUM_THREADS"] = str(plan.threads)
    os.environ["VECLIB_MAXIMUM_THREADS"] = str(plan.threads)
    os.environ["OPENBLAS_NUM_THREADS"] = str(plan.threads)
    torch.set_num_threads(plan.threads)
    try:
        torch.set_num_interop_threads(plan.threads)
    except RuntimeError as error:
        if torch.get_num_interop_threads() != plan.threads:
            raise RuntimeError("#1043 could not establish CPU interop threads") from error
    torch.use_deterministic_algorithms(True)
    torch.manual_seed(SEED)
    return torch.device("cpu")


def _normalize_batch(value: Any, *, device: torch.device) -> tuple[Tensor, Tensor]:
    if isinstance(value, (tuple, list)) and len(value) == 2:
        inputs, labels = value
    else:
        inputs = _field(value, "input_ids", "inputs", "tokens")
        labels = _field(value, "label_ids", "labels", "targets")
    inputs = torch.as_tensor(inputs, dtype=torch.long, device=device)
    labels = torch.as_tensor(labels, dtype=torch.long, device=device)
    if (
        inputs.ndim != 2
        or labels.shape != inputs.shape
        or not 1 <= int(inputs.shape[1]) <= CONTEXT
    ):
        raise ValueError("#1043 causal batch must be matching [batch,time] tensors")
    return inputs, labels


def _causal_batch(
    data_api: Any,
    examples: Sequence[Any],
    *,
    device: torch.device,
) -> tuple[Tensor, Tensor]:
    batcher = getattr(data_api, "batch_causal_examples", None)
    if not callable(batcher):
        raise AttributeError("position_kv_binding_data lacks batch_causal_examples")
    return _normalize_batch(batcher(examples, device=device), device=device)


def _natural_batch(
    store: Any,
    ordinals: Sequence[int],
    *,
    device: torch.device,
) -> tuple[Tensor, Tensor]:
    if hasattr(store, "batch"):
        return _normalize_batch(store.batch(ordinals), device=device)
    raw = _field(store, "windows") if hasattr(store, "windows") else store
    selected = torch.as_tensor(raw[list(ordinals)], dtype=torch.long, device=device)
    if selected.ndim != 2 or int(selected.shape[1]) != CONTEXT + 1:
        raise ValueError("#1043 natural rows must contain exactly 121 tokens")
    return selected[:, :-1], selected[:, 1:]


def _sequence(value: Any, *names: str) -> Sequence[Any]:
    result = _field(value, *names)
    if not isinstance(result, Sequence) and not hasattr(result, "__len__"):
        raise TypeError(f"#1043 population {names[0]} is not indexable")
    return result


def _construction_parts(construction: Any) -> tuple[Any, Sequence[Any], Sequence[Any], Sequence[Any]]:
    natural = _field(
        construction,
        "natural_windows",
        "natural_language",
        "natural",
    )
    mqar = _sequence(construction, "mqar", "mqar_examples")
    english_history = _sequence(
        construction,
        "english_history",
        "english_history_examples",
    )
    english_no_history = _sequence(
        construction,
        "english_no_history",
        "english_no_history_examples",
    )
    counts = (
        len(natural),
        len(mqar),
        len(english_history),
        len(english_no_history),
    )
    expected = (
        NATURAL_CONSTRUCTION_ROWS,
        MQAR_CONSTRUCTION_ROWS,
        ENGLISH_HISTORY_CONSTRUCTION_ROWS,
        ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS,
    )
    if counts != expected:
        raise ValueError(
            f"#1043 construction population counts differ: {counts} != {expected}"
        )
    return natural, mqar, english_history, english_no_history


def _mixed_batch(
    construction: Any,
    step: int,
    *,
    device: torch.device,
    data_api: Any | None = None,
) -> MixedBatch:
    if not 1 <= step <= OPTIMIZER_STEPS:
        raise ValueError("#1043 mixed-batch step is outside the frozen epoch")
    api = _data_module() if data_api is None else data_api
    natural, mqar, english_history, english_no_history = _construction_parts(
        construction
    )
    offset = step - 1
    natural_ordinals = range(offset * NATURAL_BATCH, step * NATURAL_BATCH)
    mqar_rows = mqar[offset * MQAR_BATCH : step * MQAR_BATCH]
    history_rows = english_history[
        offset * ENGLISH_HISTORY_BATCH : step * ENGLISH_HISTORY_BATCH
    ]
    no_history_rows = english_no_history[offset:step]
    english_rows = tuple(history_rows) + tuple(no_history_rows)
    natural_inputs, natural_labels = _natural_batch(
        natural, natural_ordinals, device=device
    )
    mqar_inputs, mqar_labels = _causal_batch(api, mqar_rows, device=device)
    english_inputs, english_labels = _causal_batch(
        api, english_rows, device=device
    )
    result = MixedBatch(
        natural_inputs=natural_inputs,
        natural_labels=natural_labels,
        mqar_inputs=mqar_inputs,
        mqar_labels=mqar_labels,
        english_inputs=english_inputs,
        english_labels=english_labels,
    )
    result.validate()
    return result


def _component_loss(logits: Tensor, labels: Tensor) -> Tensor:
    selected = labels != IGNORE_INDEX
    if not bool(selected.any()):
        raise ValueError("#1043 component has no scored labels")
    return F.cross_entropy(
        logits.float().reshape(-1, VOCAB_SIZE),
        labels.reshape(-1),
        ignore_index=IGNORE_INDEX,
        reduction="sum",
    ) / selected.sum()


def _component_top1(logits: Tensor, labels: Tensor) -> dict[str, int]:
    selected = labels != IGNORE_INDEX
    selected_labels = labels[selected]
    if selected_labels.numel() < 1:
        raise ValueError("#1043 component has no scored labels")
    selected_logits = logits.float()[selected]
    return {
        "decisions": int(selected_labels.numel()),
        "top1_correct": int(
            (selected_logits.argmax(dim=-1) == selected_labels).sum().detach().cpu()
        ),
    }


def _optimizer(model: torch.nn.Module) -> torch.optim.AdamW:
    parameters = list(model.parameters())
    if (
        sum(parameter.numel() for parameter in parameters) != PARAMETER_COUNT
        or not all(parameter.requires_grad for parameter in parameters)
    ):
        raise RuntimeError("#1043 AdamW does not own all 252,160 parameters")
    return torch.optim.AdamW(
        parameters,
        lr=PEAK_LEARNING_RATE,
        betas=ADAM_BETAS,
        eps=ADAM_EPSILON,
        weight_decay=WEIGHT_DECAY,
        foreach=False,
        fused=False,
    )


def _train_step(
    model: R4PositionPreservingCausalKVBindingV1,
    optimizer: torch.optim.Optimizer,
    batch: MixedBatch,
    *,
    step: int,
) -> dict[str, Any]:
    batch.validate()
    optimizer.zero_grad(set_to_none=True)
    natural_output = model(
        batch.natural_inputs,
        batch.natural_labels,
        execution="plain",
        intervention="native",
    )
    mqar_output = model(
        batch.mqar_inputs,
        batch.mqar_labels,
        execution="plain",
        intervention="native",
    )
    english_output = model(
        batch.english_inputs,
        batch.english_labels,
        execution="plain",
        intervention="native",
    )
    losses = {
        "natural": _component_loss(natural_output.logits, batch.natural_labels),
        "mqar": _component_loss(mqar_output.logits, batch.mqar_labels),
        "english": _component_loss(english_output.logits, batch.english_labels),
    }
    total = sum(LOSS_WEIGHTS[name] * losses[name] for name in LOSS_WEIGHTS)
    if not bool(torch.isfinite(total)):
        raise RuntimeError("#1043 mixed loss is non-finite")
    total.backward()
    missing = [
        name
        for name, parameter in model.named_parameters()
        if parameter.grad is None or not bool(torch.isfinite(parameter.grad).all())
    ]
    if missing:
        raise RuntimeError(f"#1043 all-parameter gradient contract failed: {missing}")
    gradient_norm = torch.nn.utils.clip_grad_norm_(
        model.parameters(), GRADIENT_CLIP, error_if_nonfinite=True
    )
    rate = learning_rate(step)
    for group in optimizer.param_groups:
        group["lr"] = rate
    optimizer.step()
    return {
        "total": float(total.detach().cpu()),
        "natural": float(losses["natural"].detach().cpu()),
        "mqar": float(losses["mqar"].detach().cpu()),
        "english": float(losses["english"].detach().cpu()),
        "gradient_norm_before_clip": float(gradient_norm.detach().cpu()),
        "learning_rate": rate,
        "construction_top1": {
            "natural": _component_top1(
                natural_output.logits, batch.natural_labels
            ),
            "mqar": _component_top1(mqar_output.logits, batch.mqar_labels),
            "english": _component_top1(
                english_output.logits, batch.english_labels
            ),
        },
        "audits": {
            "natural": _validated_call_audit(
                natural_output.audit,
                batch=int(batch.natural_inputs.shape[0]),
                time_steps=int(batch.natural_inputs.shape[1]),
                target_reads=int(torch.count_nonzero(batch.natural_labels != IGNORE_INDEX)),
                execution="plain",
                intervention="native",
                full_square=True,
            ),
            "mqar": _validated_call_audit(
                mqar_output.audit,
                batch=int(batch.mqar_inputs.shape[0]),
                time_steps=int(batch.mqar_inputs.shape[1]),
                target_reads=int(torch.count_nonzero(batch.mqar_labels != IGNORE_INDEX)),
                execution="plain",
                intervention="native",
                full_square=True,
            ),
            "english": _validated_call_audit(
                english_output.audit,
                batch=int(batch.english_inputs.shape[0]),
                time_steps=int(batch.english_inputs.shape[1]),
                target_reads=int(torch.count_nonzero(batch.english_labels != IGNORE_INDEX)),
                execution="plain",
                intervention="native",
                full_square=True,
            ),
        },
    }


def _audit_record(audit: Any) -> dict[str, Any]:
    value = _record(audit)
    required = set(AUDIT_WORK_FIELDS)
    if not required.issubset(value):
        raise ValueError("#1043 model audit lacks frozen work/leak fields")
    for name in required:
        observed = value[name]
        if isinstance(observed, bool) or not isinstance(observed, int) or observed < 0:
            raise ValueError(f"#1043 audit field {name} is not a nonnegative integer")
    return value


def _expected_call_work(
    *,
    batch: int,
    time_steps: int,
    target_reads: int,
    execution: Literal["plain", "r4"],
    intervention: str,
    full_square: bool,
) -> dict[str, int]:
    if batch < 1 or time_steps < 1 or not 0 <= target_reads <= batch * time_steps:
        raise ValueError("#1043 expected-call dimensions are malformed")
    token_steps = batch * time_steps
    materialized = (
        batch * LAYERS * HEADS * time_steps * time_steps
        if full_square
        else batch * LAYERS * HEADS * time_steps * (time_steps + 1) // 2
    )
    admitted = (
        batch * LAYERS * HEADS * time_steps
        if intervention == "current_only"
        else batch * LAYERS * HEADS * time_steps * (time_steps + 1) // 2
    )
    return {
        "token_steps": token_steps,
        "cache_writes": token_steps * LAYERS * 2 * HEADS * HEAD_DIM,
        "materialized_attention_scores": materialized,
        "admitted_attention_scores": admitted,
        "transported_r4_blocks": (
            materialized * 2 * (HEAD_DIM // 4) if execution == "r4" else 0
        ),
        "value_reads": materialized * HEAD_DIM,
        "vocabulary_scores": token_steps * VOCAB_SIZE,
        "target_reads": target_reads,
        "source_reads": token_steps,
        "provider_calls": 0,
        "teacher_calls": 0,
        "future_reads": 0,
        "forbidden_reads": 0,
    }


def _validated_call_audit(
    audit: Any,
    *,
    batch: int,
    time_steps: int,
    target_reads: int,
    execution: Literal["plain", "r4"],
    intervention: str,
    full_square: bool,
) -> dict[str, Any]:
    record = _audit_record(audit)
    expected = _expected_call_work(
        batch=batch,
        time_steps=time_steps,
        target_reads=target_reads,
        execution=execution,
        intervention=intervention,
        full_square=full_square,
    )
    observed = {name: record[name] for name in AUDIT_WORK_FIELDS}
    if observed != expected:
        raise ValueError("#1043 executed work differs from exact causal arithmetic")
    if (
        record.get("execution") != execution
        or record.get("intervention") != intervention
        or record.get("batch_size") != batch
        or record.get("layers") != LAYERS
        or record.get("heads") != HEADS
    ):
        raise ValueError("#1043 audit policy identity differs")
    return record


def _maximum_delta(left: Tensor, right: Tensor) -> float:
    if left.shape != right.shape:
        return math.inf
    if left.numel() == 0:
        return 0.0
    return float((left.float() - right.float()).abs().max().detach().cpu())


def _mechanics_preflight(
    root: Path,
    construction: Any,
    *,
    data_api: Any | None = None,
) -> dict[str, Any]:
    api = _data_module() if data_api is None else data_api
    device = torch.device("cpu")
    model = _build_model(root, device=device).eval()
    _, mqar, _, _ = _construction_parts(construction)
    inputs, labels = _causal_batch(api, mqar[:2], device=device)
    with torch.no_grad():
        plain = model(inputs, labels, execution="plain", intervention="native")
        geometric = model(inputs, labels, execution="r4", intervention="native")
        incremental = model.forward_incremental(
            inputs,
            labels,
            execution="r4",
            intervention="native",
        )
        changed = inputs.clone()
        changed[:, -1] = (changed[:, -1] + 1) % VOCAB_SIZE
        future_control = model(
            changed,
            execution="plain",
            intervention="native",
        )
    logit_incremental_delta = _maximum_delta(
        geometric.logits, incremental.logits
    )
    logit_r4_delta = _maximum_delta(plain.logits, geometric.logits)
    attention_r4_delta = _maximum_delta(
        plain.attention_weights, geometric.attention_weights
    )
    prefix_delta = _maximum_delta(plain.logits[:, :-1], future_control.logits[:, :-1])
    batch = int(inputs.shape[0])
    time_steps = int(inputs.shape[1])
    target_reads = int(torch.count_nonzero(labels != IGNORE_INDEX))
    audits = {
        "plain": _validated_call_audit(
            plain.audit,
            batch=batch,
            time_steps=time_steps,
            target_reads=target_reads,
            execution="plain",
            intervention="native",
            full_square=True,
        ),
        "incremental": _validated_call_audit(
            incremental.audit,
            batch=batch,
            time_steps=time_steps,
            target_reads=target_reads,
            execution="r4",
            intervention="native",
            full_square=False,
        ),
        "r4": _validated_call_audit(
            geometric.audit,
            batch=batch,
            time_steps=time_steps,
            target_reads=target_reads,
            execution="r4",
            intervention="native",
            full_square=True,
        ),
        "future_control": _validated_call_audit(
            future_control.audit,
            batch=batch,
            time_steps=time_steps,
            target_reads=0,
            execution="plain",
            intervention="native",
            full_square=True,
        ),
    }
    forbidden = sum(
        int(value["future_reads"]) + int(value["forbidden_reads"])
        for value in audits.values()
    )
    provider_or_teacher = sum(
        int(value["provider_calls"]) + int(value["teacher_calls"])
        for value in audits.values()
    )
    passed = bool(
        logit_incremental_delta <= LOGIT_PARITY_TOLERANCE
        and logit_r4_delta <= LOGIT_PARITY_TOLERANCE
        and attention_r4_delta <= ATTENTION_PARITY_TOLERANCE
        and torch.equal(
            geometric.logits.argmax(dim=-1), incremental.logits.argmax(dim=-1)
        )
        and torch.equal(plain.logits.argmax(dim=-1), geometric.logits.argmax(dim=-1))
        and prefix_delta == 0.0
        and forbidden == 0
        and provider_or_teacher == 0
    )
    return {
        "passed": passed,
        "batch": 2,
        "time": int(inputs.shape[1]),
        "r4_full_incremental_logit_max_delta": logit_incremental_delta,
        "plain_r4_logit_max_delta": logit_r4_delta,
        "plain_r4_attention_weight_max_delta": attention_r4_delta,
        "future_suffix_prefix_logit_max_delta": prefix_delta,
        "r4_full_incremental_top1_identical": bool(
            torch.equal(
                geometric.logits.argmax(dim=-1),
                incremental.logits.argmax(dim=-1),
            )
        ),
        "plain_r4_top1_identical": bool(
            torch.equal(plain.logits.argmax(dim=-1), geometric.logits.argmax(dim=-1))
        ),
        "audits": audits,
    }


def _normalize_oracle(value: Mapping[str, Any]) -> dict[str, Any]:
    def integer(*names: str) -> int:
        observed = _field(value, *names)
        if isinstance(observed, bool) or not isinstance(observed, int):
            raise ValueError(f"#1043 serialization oracle {names[0]} is not an integer")
        return observed

    overlength = integer("overlength_sequences", "overlength")
    maximum_context = _optional_field(value, "maximum_context", "max_context")
    if maximum_context is None:
        maximum_context = CONTEXT if overlength == 0 else CONTEXT + 1
    if isinstance(maximum_context, bool) or not isinstance(maximum_context, int):
        raise ValueError("#1043 serialization oracle maximum_context is not an integer")
    result = {
        "mqar_correct": integer("mqar_correct", "mqar_recovered"),
        "mqar_total": integer("mqar_total", "mqar_labels"),
        "english_correct": integer("english_correct", "english_recovered"),
        "english_total": integer("english_total", "english_labels"),
        "ambiguous": integer("ambiguous", "ambiguous_bindings"),
        "missing": integer("missing", "missing_bindings"),
        "overlength_sequences": overlength,
        "maximum_context": maximum_context,
    }
    result["passed"] = bool(
        result["mqar_correct"] == TERMINAL_MQAR_DECISIONS
        and result["mqar_total"] == TERMINAL_MQAR_DECISIONS
        and result["english_correct"] == TERMINAL_ENGLISH_HISTORY_DECISIONS
        and result["english_total"] == TERMINAL_ENGLISH_HISTORY_DECISIONS
        and result["ambiguous"] == 0
        and result["missing"] == 0
        and result["overlength_sequences"] == 0
        and result["maximum_context"] <= CONTEXT
    )
    return result


def _probe_trajectory(
    root: Path,
    construction: Any,
    *,
    device: torch.device,
    data_api: Any,
) -> dict[str, Any]:
    torch.manual_seed(SEED)
    model = _build_model(root, device=device).train()
    optimizer = _optimizer(model)
    measured: list[float] = []
    loss_trace: list[float] = []
    work: dict[str, int] = {}
    total_steps = PROBE_WARMUP_STEPS + PROBE_MEASURED_STEPS
    for step in range(1, total_steps + 1):
        batch = _mixed_batch(construction, step, device=device, data_api=data_api)
        started = time.monotonic()
        record = _train_step(model, optimizer, batch, step=step)
        elapsed = time.monotonic() - started
        if step > PROBE_WARMUP_STEPS:
            measured.append(elapsed)
        loss_trace.append(float(record["total"]))
        for audit in record["audits"].values():
            if (
                audit["future_reads"] != 0
                or audit["forbidden_reads"] != 0
                or audit["provider_calls"] != 0
                or audit["teacher_calls"] != 0
            ):
                raise RuntimeError("#1043 probe observed a forbidden training read")
            for name, value in audit.items():
                if name in {
                    "execution",
                    "intervention",
                    "batch_size",
                    "layers",
                    "heads",
                }:
                    continue
                if isinstance(value, int):
                    work[name] = work.get(name, 0) + value
    model.eval()
    natural, _, _, _ = _construction_parts(construction)
    inputs, _ = _natural_batch(natural, range(BATCH_SIZE), device=device)
    started = time.monotonic()
    with torch.no_grad():
        plain = model(inputs, execution="plain", intervention="native")
    plain_seconds = time.monotonic() - started
    started = time.monotonic()
    with torch.no_grad():
        geometric = model(inputs, execution="r4", intervention="native")
    r4_seconds = time.monotonic() - started
    started = time.monotonic()
    with torch.no_grad():
        incremental = model.forward_incremental(
            inputs, execution="r4", intervention="native"
        )
    incremental_seconds = time.monotonic() - started
    for output, execution, full_square in (
        (plain, "plain", True),
        (geometric, "r4", True),
        (incremental, "r4", False),
    ):
        evaluation_audit = _validated_call_audit(
            output.audit,
            batch=int(inputs.shape[0]),
            time_steps=int(inputs.shape[1]),
            target_reads=0,
            execution=execution,  # type: ignore[arg-type]
            intervention="native",
            full_square=full_square,
        )
        if any(
            evaluation_audit[name] != 0
            for name in (
                "future_reads",
                "forbidden_reads",
                "provider_calls",
                "teacher_calls",
            )
        ):
            raise RuntimeError("#1043 probe evaluation observed a forbidden read")
    artifact = model.export_learned_artifact()
    return {
        "mean_train_step_seconds": sum(measured) / len(measured),
        "plain_full_batch_seconds": plain_seconds,
        "r4_full_batch_seconds": r4_seconds,
        "incremental_batch_seconds": incremental_seconds,
        "loss_trace": loss_trace,
        "artifact_bytes": artifact,
        "artifact_cid": cid_bytes(artifact),
        "peak_memory_bytes": _peak_rss_bytes(),
        "work": work,
    }


def _default_probe_runner(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    device = _configure_cpu(plan)
    data_api = _data_module()
    construction = data_api.load_position_kv_binding_construction(root)
    first = _probe_trajectory(
        root, construction, device=device, data_api=data_api
    )
    second = _probe_trajectory(
        root, construction, device=device, data_api=data_api
    )
    deterministic = bool(
        first["artifact_bytes"] == second["artifact_bytes"]
        and first["loss_trace"] == second["loss_trace"]
    )
    mean_step = max(
        float(first["mean_train_step_seconds"]),
        float(second["mean_train_step_seconds"]),
    )
    plain_seconds = max(
        float(first["plain_full_batch_seconds"]),
        float(second["plain_full_batch_seconds"]),
    )
    r4_seconds = max(
        float(first["r4_full_batch_seconds"]),
        float(second["r4_full_batch_seconds"]),
    )
    incremental_seconds = max(
        float(first["incremental_batch_seconds"]),
        float(second["incremental_batch_seconds"]),
    )
    raw_training_seconds = mean_step * OPTIMIZER_STEPS
    raw_scoring_seconds = (
        plain_seconds * PROJECTED_PLAIN_FULL_BATCHES
        + r4_seconds * PROJECTED_R4_FULL_BATCHES
        + incremental_seconds * PROJECTED_INCREMENTAL_BATCHES
    )
    projected_training_seconds = PROJECTION_SAFETY_FACTOR * raw_training_seconds
    projected_scoring_seconds = PROJECTION_SAFETY_FACTOR * raw_scoring_seconds
    projected = projected_training_seconds + projected_scoring_seconds
    return {
        "plan": plan.identity(),
        "deterministic_replay": deterministic,
        "probe_artifact_cid": first["artifact_cid"],
        "mean_train_step_seconds": mean_step,
        "plain_full_batch_seconds": plain_seconds,
        "r4_full_batch_seconds": r4_seconds,
        "incremental_batch_seconds": incremental_seconds,
        "projection": {
            "raw_training_seconds": raw_training_seconds,
            "raw_scoring_seconds": raw_scoring_seconds,
            "training_seconds": projected_training_seconds,
            "scoring_seconds": projected_scoring_seconds,
            "plain_full_batches": PROJECTED_PLAIN_FULL_BATCHES,
            "r4_full_batches": PROJECTED_R4_FULL_BATCHES,
            "incremental_batches": PROJECTED_INCREMENTAL_BATCHES,
            "safety_factor": PROJECTION_SAFETY_FACTOR,
            "total_seconds": projected,
        },
        "peak_memory_bytes": max(
            int(first["peak_memory_bytes"]), int(second["peak_memory_bytes"])
        ),
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
        "loss_trace": first["loss_trace"],
        "work": first["work"],
    }


def _probe_worker(root: str, plan_value: Mapping[str, Any], queue: Any) -> None:
    try:
        plan = ExecutionPlan(
            name=str(plan_value["name"]), threads=int(plan_value["threads"])
        )
        queue.put({"ok": True, "result": _default_probe_runner(Path(root), plan)})
    except BaseException as error:
        queue.put(
            {
                "ok": False,
                "plan": dict(plan_value),
                "error": {
                    "type": type(error).__name__,
                    "reason": str(error),
                    "traceback": traceback.format_exc(),
                },
            }
        )


def _spawned_probe_runner(root: Path, plan: ExecutionPlan) -> dict[str, Any]:
    context = mp.get_context("spawn")
    queue = context.Queue()
    process = context.Process(
        target=_probe_worker,
        args=(str(root), {"name": plan.name, "threads": plan.threads}, queue),
        name=f"position-kv-probe-{plan.threads}t",
    )
    process.start()
    process.join(timeout=300.0)
    if process.is_alive():
        process.terminate()
        process.join(timeout=10.0)
        return {
            "ok": False,
            "plan": plan.identity(),
            "error": {"type": "TimeoutError", "reason": "probe exceeded 300 seconds"},
        }
    if queue.empty():
        return {
            "ok": False,
            "plan": plan.identity(),
            "error": {
                "type": "RuntimeError",
                "reason": f"probe exited {process.exitcode} without evidence",
            },
        }
    return dict(queue.get())


def select_execution_plan(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    """Choose the fastest deterministic plan within both frozen hard bounds."""

    by_name = {
        str(record.get("result", record).get("plan", {}).get("name")): record
        for record in records
        if isinstance(record.get("result", record), Mapping)
    }
    expected_names = {plan.name for plan in ELIGIBLE_PLANS}
    if set(by_name) != expected_names:
        raise ValueError("#1043 probe does not contain exactly the 1/4/8 CPU plans")
    normalized: list[dict[str, Any]] = []
    eligible: list[dict[str, Any]] = []
    for plan in ELIGIBLE_PLANS:
        envelope = dict(by_name[plan.name])
        if envelope.get("ok") is False:
            normalized.append(envelope)
            continue
        result = dict(envelope.get("result", envelope))
        if result.get("plan") != plan.identity():
            raise ValueError("#1043 probe plan identity differs")
        projection = result.get("projection")
        total = projection.get("total_seconds") if isinstance(projection, Mapping) else None
        memory = result.get("peak_memory_bytes")
        values = (total, memory)
        finite = all(
            isinstance(value, (int, float))
            and not isinstance(value, bool)
            and math.isfinite(float(value))
            and float(value) >= 0.0
            for value in values
        )
        result["eligible"] = bool(
            finite
            and result.get("deterministic_replay") is True
            and float(total) <= HARD_WALL_SECONDS
            and int(memory) <= MEMORY_CEILING_BYTES
        )
        normalized.append({"ok": True, "result": result})
        if result["eligible"]:
            eligible.append(result)
    selected = min(
        eligible,
        key=lambda value: (
            float(value["projection"]["total_seconds"]),
            int(value["plan"]["threads"]),
        ),
        default=None,
    )
    return {
        "available": selected is not None,
        "plans": normalized,
        "selected_plan": None if selected is None else selected["plan"],
        "selected_projection": None if selected is None else selected["projection"],
        "hard_wall_seconds": HARD_WALL_SECONDS,
        "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
    }


def prepare_position_kv_binding_campaign(
    root: Path,
    *,
    retained_language_root: Path,
    source_root: Path,
    tokenizer_path: Path,
    geometry_path: Path,
    h4_sidecar_path: Path,
    excluded_story_cids: Sequence[str],
) -> dict[str, Any]:
    """Prepare and CID-seal all populations without exposing terminal payloads."""

    root = root.resolve()
    paths = _phase_paths(root)
    if root.exists() or root.is_symlink():
        raise FileExistsError("#1043 preparation requires a new empty campaign root")
    implementation = _capture_prepare_implementation()
    root.parent.mkdir(parents=True, exist_ok=True)
    exclusions = _validate_complete_story_exclusions(excluded_story_cids)
    data_api = _data_module()
    prepared = data_api.prepare_position_kv_binding_data(
        output_root=root,
        retained_language_root=retained_language_root.resolve(),
        source_root=source_root.resolve(),
        tokenizer_path=tokenizer_path.resolve(),
        excluded_story_cids=exclusions,
    )
    manifest = _data_manifest(prepared)
    initial_source = retained_language_root.resolve() / "arms/ordinary/model.safetensors"
    input_records = {
        "ordinary_initialization": _copy_verified_input(
            initial_source,
            root / INPUT_INITIAL_ARTIFACT,
            expected_cid=INITIAL_ARTIFACT_CID,
            expected_bytes=INITIAL_ARTIFACT_BYTES,
        ),
        "geometry": _copy_verified_input(
            geometry_path,
            root / INPUT_GEOMETRY,
            expected_cid=GEOMETRY_FILE_CID,
        ),
        "h4_frames": _copy_verified_input(
            h4_sidecar_path,
            root / INPUT_H4_FRAMES,
            expected_cid=H4_FRAME_FILE_CID,
        ),
    }
    geometry_bundle = load_group_geometry_artifacts(root / INPUT_GEOMETRY)
    frames = H4SpinFrameArtifactV1.load(root / INPUT_H4_FRAMES)
    if (
        geometry_bundle.artifact_cid != GEOMETRY_ARTIFACT_CID
        or frames.artifact_cid != H4_FRAME_ARTIFACT_CID
    ):
        raise ValueError("#1043 geometry identity differs from the freeze")
    _require_current_implementation(implementation)
    body = {
        "schema": PREPARATION_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "seed": SEED,
        "implementation": implementation,
        "data_manifest": manifest,
        "data_manifest_cid": _manifest_cid(manifest),
        "inputs": input_records,
        "population": {
            "natural_construction_rows": NATURAL_CONSTRUCTION_ROWS,
            "mqar_construction_rows": MQAR_CONSTRUCTION_ROWS,
            "english_history_construction_rows": ENGLISH_HISTORY_CONSTRUCTION_ROWS,
            "english_no_history_construction_rows": ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS,
            "terminal_payload_state": "SEALED_UNOPENED",
        },
        "leakage_exclusion": {
            "policy": (
                "all stories informing the inherited ordinary train/development "
                "slices plus all V1-through-V5 prompt and V5 fresh-language stories"
            ),
            "story_cids": len(exclusions),
            "story_cids_cid": _story_set_cid(exclusions),
        },
        "fit": {
            "steps": OPTIMIZER_STEPS,
            "batch": {
                "total": BATCH_SIZE,
                "natural": NATURAL_BATCH,
                "mqar": MQAR_BATCH,
                "english_history": ENGLISH_HISTORY_BATCH,
                "english_no_history": ENGLISH_NO_HISTORY_BATCH,
            },
            "all_parameters": PARAMETER_COUNT,
            "optimizer": "AdamW",
            "loss_weights": LOSS_WEIGHTS,
            "seed": SEED,
        },
        "reveal_after": "fitted artifact CID and fit CID are fixed",
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    result = _with_cid(body, "preparation_cid")
    _write_exclusive_json(paths["preparation"], result)
    return result


def preflight_position_kv_binding_campaign(
    root: Path,
) -> dict[str, Any]:
    """Run the binding oracle, mechanics fixture, and 1/4/8 CPU probe."""

    root = root.resolve()
    paths = _phase_paths(root)
    if any(paths[name].exists() for name in ("started", "artifact", "fit", "reveal", "result")):
        raise RuntimeError("#1043 preflight cannot run after fitting has started")
    if paths["preflight"].exists():
        value = _read_json(paths["preflight"], cid_field="preflight_cid")
        _require_current_implementation(value.get("implementation"))
        return value
    preparation = _read_json(paths["preparation"], cid_field="preparation_cid")
    implementation = _require_current_implementation(preparation.get("implementation"))
    data_api = _data_module()
    construction = data_api.load_position_kv_binding_construction(root)
    current_manifest = _data_manifest(construction)
    if (
        _manifest_cid(current_manifest) != preparation.get("data_manifest_cid")
        or current_manifest != preparation.get("data_manifest")
    ):
        raise ValueError("#1043 construction data differs from preparation")
    commitment = _field(construction, "commitment")
    oracle_source = _optional_field(
        commitment,
        "direct_serialization_oracle",
        "serialization_oracle",
        "oracle",
    )
    if not isinstance(oracle_source, Mapping):
        raise ValueError("#1043 preparation lacks the sealed serialization oracle")
    oracle = _normalize_oracle(oracle_source)
    mechanics = _mechanics_preflight(root, construction, data_api=data_api)
    probe_records: list[dict[str, Any]] = []
    if oracle["passed"] and mechanics["passed"]:
        for plan in ELIGIBLE_PLANS:
            try:
                record = dict(_spawned_probe_runner(root, plan))
                if "ok" not in record:
                    record = {"ok": True, "result": record}
            except BaseException as error:
                record = {
                    "ok": False,
                    "plan": plan.identity(),
                    "error": {"type": type(error).__name__, "reason": str(error)},
                }
            probe_records.append(record)
        selection = select_execution_plan(probe_records)
    else:
        selection = {
            "available": False,
            "plans": [],
            "selected_plan": None,
            "selected_projection": None,
            "hard_wall_seconds": HARD_WALL_SECONDS,
            "memory_ceiling_bytes": MEMORY_CEILING_BYTES,
        }
    body = {
        "schema": PREFLIGHT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "data_manifest_cid": preparation["data_manifest_cid"],
        "implementation": implementation,
        "serialization_oracle": oracle,
        "mechanics": mechanics,
        "selection": selection,
        "passed": bool(oracle["passed"] and mechanics["passed"] and selection["available"]),
        "terminal_payload_reads": 0,
    }
    result = _with_cid(body, "preflight_cid")
    _write_exclusive_json(paths["preflight"], result)
    return result


def _selected_plan(preflight: Mapping[str, Any]) -> ExecutionPlan | None:
    selected = _optional_field(preflight, "selection", default={})
    if not isinstance(selected, Mapping):
        return None
    identity = selected.get("selected_plan")
    if identity is None:
        return None
    if not isinstance(identity, Mapping):
        raise ValueError("#1043 selected execution plan is malformed")
    plan = ExecutionPlan(name=str(identity.get("name")), threads=int(identity.get("threads")))
    if plan.identity() != dict(identity):
        raise ValueError("#1043 selected execution plan identity differs")
    return plan


def _selected_projection(preflight: Mapping[str, Any]) -> dict[str, float] | None:
    selection = preflight.get("selection")
    if not isinstance(selection, Mapping):
        return None
    value = selection.get("selected_projection")
    if value is None:
        return None
    if not isinstance(value, Mapping):
        raise ValueError("#1043 selected execution projection is malformed")
    result: dict[str, float] = {}
    for name in ("training_seconds", "scoring_seconds", "total_seconds"):
        observed = value.get(name)
        if (
            isinstance(observed, bool)
            or not isinstance(observed, (int, float))
            or not math.isfinite(float(observed))
            or float(observed) < 0.0
        ):
            raise ValueError(f"#1043 selected projection {name} is malformed")
        result[name] = float(observed)
    if (
        not math.isclose(
            result["training_seconds"] + result["scoring_seconds"],
            result["total_seconds"],
            rel_tol=0.0,
            abs_tol=1.0e-9,
        )
        or result["total_seconds"] > HARD_WALL_SECONDS
    ):
        raise ValueError("#1043 selected projection exceeds or misstates the total wall")
    return result


def _write_result_once(root: Path, body: Mapping[str, Any]) -> dict[str, Any]:
    path = root / RESULT_RELATIVE_PATH
    if path.exists():
        existing = _read_json(path, cid_field="result_cid")
        candidate = _with_cid(body, "result_cid")
        if existing != candidate:
            raise FileExistsError("#1043 terminal result is already fixed and differs")
        return existing
    result = _with_cid(body, "result_cid")
    _write_exclusive_json(path, result)
    return result


def _unavailable_result(
    root: Path,
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    reason: str,
    phase: str,
) -> dict[str, Any]:
    return _write_result_once(
        root,
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "fit_cid": None,
            "reveal_cid": None,
            "scoring_cid": None,
            "artifact": None,
            "decision": {
                "verdict": TERMINAL_UNAVAILABLE,
                "phase": phase,
                "reason": reason,
                "action": "stop; do not substitute a population, backend, or fit",
            },
            "verdict": TERMINAL_UNAVAILABLE,
            "terminal_payload_reads": 0,
            "optimizer_steps_after_reveal": 0,
        },
    )


def _aggregate_training_work(
    totals: dict[str, int], audits: Mapping[str, Mapping[str, Any]]
) -> None:
    for audit in audits.values():
        for name, value in audit.items():
            if name in {"execution", "intervention", "batch_size", "layers", "heads"}:
                continue
            if isinstance(value, int):
                totals[name] = totals.get(name, 0) + value


def fit_position_kv_binding_campaign(root: Path) -> dict[str, Any]:
    """Run the sole 2,730-step all-parameter fit without opening terminal data."""

    root = root.resolve()
    paths = _phase_paths(root)
    if paths["result"].exists():
        return _read_json(paths["result"], cid_field="result_cid")
    if paths["reveal"].exists():
        raise RuntimeError("#1043 optimizer construction is forbidden after reveal")
    preparation = _read_json(paths["preparation"], cid_field="preparation_cid")
    preflight = _read_json(paths["preflight"], cid_field="preflight_cid")
    implementation = _require_current_implementation(preflight.get("implementation"))
    if (
        preflight.get("preparation_cid") != preparation["preparation_cid"]
        or preflight.get("data_manifest_cid") != preparation["data_manifest_cid"]
    ):
        raise ValueError("#1043 preflight does not bind the preparation")
    plan = _selected_plan(preflight)
    projection = _selected_projection(preflight)
    if preflight.get("passed") is not True or plan is None or projection is None:
        return _unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            reason="serialization, mechanics, or CPU projection preflight did not pass",
            phase="PREFLIGHT",
        )
    if paths["fit"].exists():
        fit = _read_json(paths["fit"], cid_field="fit_cid")
        if fit.get("preflight_cid") != preflight["preflight_cid"]:
            raise ValueError("#1043 cached fit binds another preflight")
        return fit
    if paths["started"].exists():
        raise RuntimeError(
            "#1043 fit already started without a final artifact; a retry is forbidden"
        )
    data_api = _data_module()
    construction = data_api.load_position_kv_binding_construction(root)
    manifest = _data_manifest(construction)
    if (
        manifest != preparation.get("data_manifest")
        or _manifest_cid(manifest) != preparation.get("data_manifest_cid")
    ):
        raise ValueError("#1043 construction changed after preflight")
    _construction_parts(construction)
    run_contract = {
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "implementation": implementation,
        "plan": plan.identity(),
        "optimizer": {
            "name": "AdamW",
            "steps": OPTIMIZER_STEPS,
            "batch_size": BATCH_SIZE,
            "composition": {
                "natural": NATURAL_BATCH,
                "mqar": MQAR_BATCH,
                "english_history": ENGLISH_HISTORY_BATCH,
                "english_no_history": ENGLISH_NO_HISTORY_BATCH,
            },
            "loss_weights": LOSS_WEIGHTS,
            "betas": list(ADAM_BETAS),
            "epsilon": ADAM_EPSILON,
            "weight_decay": WEIGHT_DECAY,
            "gradient_clip": GRADIENT_CLIP,
            "warmup_steps": WARMUP_STEPS,
            "peak_learning_rate": PEAK_LEARNING_RATE,
            "final_learning_rate": FINAL_LEARNING_RATE,
            "schedule": "linear warmup then cosine decay",
            "seed": SEED,
            "checkpoint_selection": "NONE",
        },
        "wall": {
            "total_seconds": HARD_WALL_SECONDS,
            "selected_projection": projection,
            "fit_ceiling_seconds": HARD_WALL_SECONDS
            - projection["scoring_seconds"],
            "scope": "fit plus reveal, all controls, parity, incremental replay, and result",
        },
        "terminal_payload": "SEALED_UNOPENED",
        "cuda": "FORBIDDEN",
        "mps": "FORBIDDEN",
    }
    run_contract_cid = cid_bytes(canonical_json_bytes(run_contract))
    started = _with_cid(
        {
            "schema": STARTED_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "implementation": implementation,
            "run_contract": run_contract,
            "run_contract_cid": run_contract_cid,
            "terminal_payload_reads": 0,
        },
        "started_cid",
    )
    _write_exclusive_json(paths["started"], started)
    began = time.monotonic()
    try:
        device = _configure_cpu(plan)
        torch.manual_seed(SEED)
        model = _build_model(root, device=device).train()
        if (
            model.export_learned_artifact()
            != (root / INPUT_INITIAL_ARTIFACT).read_bytes()
        ):
            raise RuntimeError(
                "#1043 loaded initialization does not replay byte-identically"
            )
        optimizer = _optimizer(model)
    except KeyboardInterrupt:
        return _unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            reason="the sole fit was interrupted during initialization",
            phase="FIT",
        )
    trace = blake3()
    work: dict[str, int] = {}
    construction_top1 = {
        name: {"decisions": 0, "top1_correct": 0}
        for name in ("natural", "mqar", "english")
    }
    first_loss: dict[str, Any] | None = None
    final_loss: dict[str, Any] | None = None
    fit_wall_ceiling = HARD_WALL_SECONDS - projection["scoring_seconds"]
    if fit_wall_ceiling <= 0.0:
        return _unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            reason="the selected projection leaves no positive fit wall",
            phase="FIT",
        )
    try:
        for step in range(1, OPTIMIZER_STEPS + 1):
            if time.monotonic() - began > fit_wall_ceiling:
                return _unavailable_result(
                    root,
                    preparation=preparation,
                    preflight=preflight,
                    reason="the fit exhausted its scoring-reserved share of the 1,800-second total wall",
                    phase="FIT",
                )
            batch = _mixed_batch(construction, step, device=device, data_api=data_api)
            loss = _train_step(model, optimizer, batch, step=step)
            numeric_loss = {
                name: loss[name]
                for name in (
                    "total",
                    "natural",
                    "mqar",
                    "english",
                    "gradient_norm_before_clip",
                    "learning_rate",
                )
            }
            step_top1 = loss.get("construction_top1")
            if not isinstance(step_top1, Mapping):
                raise RuntimeError("#1043 fit lacks construction top-1 evidence")
            for name in construction_top1:
                record = step_top1.get(name)
                if not isinstance(record, Mapping):
                    raise RuntimeError("#1043 construction top-1 record is absent")
                for field in ("decisions", "top1_correct"):
                    observed = record.get(field)
                    if (
                        isinstance(observed, bool)
                        or not isinstance(observed, int)
                        or observed < 0
                    ):
                        raise RuntimeError(
                            f"#1043 construction top-1 {name}.{field} is malformed"
                        )
                    construction_top1[name][field] += observed
            trace.update(
                canonical_json_bytes(
                    {"step": step, "loss": numeric_loss, "top1": step_top1}
                )
            )
            _aggregate_training_work(work, loss["audits"])
            if first_loss is None:
                first_loss = numeric_loss
            final_loss = numeric_loss
    except KeyboardInterrupt:
        return _unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            reason="the sole fit was interrupted",
            phase="FIT",
        )
    expected_target_reads = (
        NATURAL_CONSTRUCTION_ROWS * CONTEXT
        + MQAR_CONSTRUCTION_ROWS * 8
        + ENGLISH_HISTORY_CONSTRUCTION_ROWS
        + ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS
    )
    if (
        work.get("target_reads") != expected_target_reads
        or any(
            work.get(name, 0) != 0
            for name in ("provider_calls", "teacher_calls", "future_reads", "forbidden_reads")
        )
    ):
        raise RuntimeError("#1043 fit work/leak ledger differs from the freeze")
    expected_construction_decisions = {
        "natural": NATURAL_CONSTRUCTION_ROWS * CONTEXT,
        "mqar": MQAR_CONSTRUCTION_ROWS * 8,
        "english": ENGLISH_HISTORY_CONSTRUCTION_ROWS
        + ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS,
    }
    for name, expected in expected_construction_decisions.items():
        record = construction_top1[name]
        if (
            record["decisions"] != expected
            or record["top1_correct"] > record["decisions"]
        ):
            raise RuntimeError("#1043 construction top-1 ledger differs")
        record["top1_rate"] = record["top1_correct"] / record["decisions"]
    if paths["reveal"].exists():
        raise RuntimeError("#1043 terminal data was revealed during fitting")
    artifact = model.export_learned_artifact()
    _write_exclusive(paths["artifact"], artifact)
    artifact_record = {
        "path": ARTIFACT_RELATIVE_PATH,
        "bytes": len(artifact),
        "cid": cid_file(paths["artifact"]),
    }
    elapsed = time.monotonic() - began
    if elapsed > fit_wall_ceiling:
        return _unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            reason=(
                "the fit, work validation, and artifact export exhausted their "
                "scoring-reserved share of the 1,800-second total wall"
            ),
            phase="FIT",
        )
    body = {
        "schema": FIT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "started_cid": started["started_cid"],
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "run_contract_cid": run_contract_cid,
        "implementation": implementation,
        "plan": plan.identity(),
        "completed_steps": OPTIMIZER_STEPS,
        "presentations": {
            "natural": NATURAL_CONSTRUCTION_ROWS,
            "mqar": MQAR_CONSTRUCTION_ROWS,
            "english_history": ENGLISH_HISTORY_CONSTRUCTION_ROWS,
            "english_no_history": ENGLISH_NO_HISTORY_CONSTRUCTION_ROWS,
        },
        "first_loss": first_loss,
        "final_loss": final_loss,
        "construction_top1": construction_top1,
        "loss_trace_cid": f"blake3:{trace.hexdigest()}",
        "elapsed_seconds": elapsed,
        "fit_wall_ceiling_seconds": fit_wall_ceiling,
        "projected_scoring_seconds": projection["scoring_seconds"],
        "total_wall_seconds": HARD_WALL_SECONDS,
        "artifact": artifact_record,
        "work": work,
        "terminal_payload_reads_before_artifact_cid": 0,
        "optimizer_steps_after_reveal": 0,
    }
    fit = _with_cid(body, "fit_cid")
    _write_exclusive_json(paths["fit"], fit)
    return fit


def _load_fitted_model(
    root: Path, artifact_path: Path, *, device: torch.device
) -> R4PositionPreservingCausalKVBindingV1:
    model = _build_model(root, device=device)
    payload = artifact_path.read_bytes()
    model.load_learned_artifact(payload)
    if model.export_learned_artifact() != payload:
        raise RuntimeError("#1043 fitted artifact does not reload byte-identically")
    return model


def _accumulate_audit(totals: dict[str, int], audit: Any) -> None:
    record = _audit_record(audit)
    for name, value in record.items():
        if name in {"execution", "intervention", "batch_size", "layers", "heads"}:
            continue
        if isinstance(value, int):
            totals[name] = totals.get(name, 0) + value


def _finish_score(
    *,
    decisions: int,
    correct: int,
    loss_sum: float,
    digest: Any,
    work: Mapping[str, int],
    extra: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    if (
        isinstance(decisions, bool)
        or not isinstance(decisions, int)
        or decisions < 1
        or isinstance(correct, bool)
        or not isinstance(correct, int)
        or not 0 <= correct <= decisions
        or isinstance(loss_sum, bool)
        or not isinstance(loss_sum, (int, float))
        or not math.isfinite(float(loss_sum))
        or float(loss_sum) < 0.0
    ):
        raise ValueError("#1043 terminal score totals are malformed")
    result = {
        "decisions": decisions,
        "top1_correct": correct,
        "top1_rate": correct / decisions,
        "nll_nats": loss_sum / decisions,
        "selected_logits_cid": f"blake3:{digest.hexdigest()}",
        "work": dict(work),
    }
    if extra:
        result.update(extra)
    return result


def _check_deadline(deadline: float | None) -> None:
    if deadline is not None and time.monotonic() >= deadline:
        raise TimeoutError("#1043 total fit-plus-scoring wall was exhausted")


@torch.no_grad()
def _score_natural(
    model: R4PositionPreservingCausalKVBindingV1,
    store: Any,
    *,
    device: torch.device,
    batch_size: int = BATCH_SIZE,
    deadline: float | None = None,
) -> dict[str, Any]:
    model.eval()
    decisions = 0
    correct = 0
    loss_sum = 0.0
    digest = blake3()
    work: dict[str, int] = {}
    count = len(store)
    for start in range(0, count, batch_size):
        _check_deadline(deadline)
        ordinals = range(start, min(count, start + batch_size))
        inputs, labels = _natural_batch(store, ordinals, device=device)
        output = model(
            inputs,
            labels,
            execution="plain",
            intervention="native",
        )
        logits = output.logits.float()
        selected = labels != IGNORE_INDEX
        selected_logits = logits[selected]
        selected_labels = labels[selected]
        decisions += int(selected_labels.numel())
        correct += int((selected_logits.argmax(dim=-1) == selected_labels).sum().cpu())
        loss_sum += float(
            F.cross_entropy(selected_logits, selected_labels, reduction="sum").cpu()
        )
        digest.update(selected_logits.detach().cpu().contiguous().numpy().tobytes())
        _accumulate_audit(
            work,
            _validated_call_audit(
                output.audit,
                batch=int(inputs.shape[0]),
                time_steps=int(inputs.shape[1]),
                target_reads=int(selected_labels.numel()),
                execution="plain",
                intervention="native",
                full_square=True,
            ),
        )
        _check_deadline(deadline)
    return _finish_score(
        decisions=decisions,
        correct=correct,
        loss_sum=loss_sum,
        digest=digest,
        work=work,
    )


def _world_assignments(english_history: Sequence[Any]) -> dict[int, dict[int, int]]:
    result: dict[int, dict[int, int]] = {}
    for example in english_history:
        world = int(_field(example, "world_index"))
        keys = tuple(int(value) for value in _field(example, "binding_keys"))
        values = tuple(int(value) for value in _field(example, "binding_values"))
        mapping = dict(zip(keys, values, strict=True))
        if world in result and result[world] != mapping:
            raise ValueError("#1043 English world has inconsistent assignments")
        result[world] = mapping
    return result


@torch.no_grad()
def _score_examples(
    model: R4PositionPreservingCausalKVBindingV1,
    examples: Sequence[Any],
    *,
    device: torch.device,
    execution: Literal["plain", "r4"] = "plain",
    intervention: str = "native",
    data_api: Any | None = None,
    world_assignments: Mapping[int, Mapping[int, int]] | None = None,
    deadline: float | None = None,
) -> dict[str, Any]:
    api = _data_module() if data_api is None else data_api
    model.eval()
    decisions = 0
    correct = 0
    loss_sum = 0.0
    digest = blake3()
    work: dict[str, int] = {}
    assigned_answer_correct = 0
    unsupported_assigned_top1 = 0
    for start in range(0, len(examples), BATCH_SIZE):
        _check_deadline(deadline)
        rows = examples[start : start + BATCH_SIZE]
        inputs, labels = _causal_batch(api, rows, device=device)
        output = model(
            inputs,
            labels,
            execution=execution,
            intervention=intervention,  # type: ignore[arg-type]
        )
        logits = output.logits.float()
        selected = labels != IGNORE_INDEX
        selected_logits = logits[selected]
        selected_labels = labels[selected]
        predictions = selected_logits.argmax(dim=-1)
        decisions += int(selected_labels.numel())
        correct += int((predictions == selected_labels).sum().cpu())
        loss_sum += float(
            F.cross_entropy(selected_logits, selected_labels, reduction="sum").cpu()
        )
        digest.update(selected_logits.detach().cpu().contiguous().numpy().tobytes())
        _accumulate_audit(
            work,
            _validated_call_audit(
                output.audit,
                batch=int(inputs.shape[0]),
                time_steps=int(inputs.shape[1]),
                target_reads=int(selected_labels.numel()),
                execution=execution,
                intervention=intervention,
                full_square=True,
            ),
        )
        if world_assignments is not None:
            cursor = 0
            for row in rows:
                width = len(tuple(_field(row, "answers")))
                row_predictions = predictions[cursor : cursor + width].tolist()
                cursor += width
                world = int(_field(row, "world_index"))
                mapping = world_assignments.get(world)
                if mapping is None:
                    raise ValueError("#1043 no-history row lacks its world assignment")
                query_keys = tuple(int(value) for value in _field(row, "query_keys"))
                for prediction, query_key in zip(
                    row_predictions, query_keys, strict=True
                ):
                    expected = mapping.get(query_key)
                    if expected is None:
                        raise ValueError("#1043 no-history query is absent from its world")
                    assigned_answer_correct += int(prediction == expected)
                    unsupported_assigned_top1 += int(prediction in set(mapping.values()))
        _check_deadline(deadline)
    extra = None
    if world_assignments is not None:
        extra = {
            "assigned_answer_top1_correct": assigned_answer_correct,
            "assigned_answer_top1_rate": assigned_answer_correct / decisions,
            "unsupported_assigned_value_top1": unsupported_assigned_top1,
        }
    return _finish_score(
        decisions=decisions,
        correct=correct,
        loss_sum=loss_sum,
        digest=digest,
        work=work,
        extra=extra,
    )


def _selected_values(output: Any, labels: Tensor) -> tuple[Tensor, Tensor]:
    selected = labels != IGNORE_INDEX
    return output.logits.float()[selected], output.attention_weights.float()


@torch.no_grad()
def _parity_examples(
    model: R4PositionPreservingCausalKVBindingV1,
    examples: Sequence[Any],
    *,
    device: torch.device,
    data_api: Any,
    deadline: float | None = None,
) -> dict[str, Any]:
    logit_r4_delta = 0.0
    attention_r4_delta = 0.0
    logit_incremental_delta = 0.0
    plain_r4_top1 = True
    plain_incremental_top1 = True
    work: dict[str, int] = {}
    decisions = 0
    for start in range(0, len(examples), BATCH_SIZE):
        _check_deadline(deadline)
        rows = examples[start : start + BATCH_SIZE]
        inputs, labels = _causal_batch(data_api, rows, device=device)
        plain = model(inputs, labels, execution="plain", intervention="native")
        geometric = model(inputs, labels, execution="r4", intervention="native")
        incremental = model.forward_incremental(
            inputs,
            labels,
            execution="r4",
            intervention="native",
        )
        plain_logits, plain_weights = _selected_values(plain, labels)
        geometric_logits, geometric_weights = _selected_values(geometric, labels)
        incremental_logits, incremental_weights = _selected_values(incremental, labels)
        decisions += int(plain_logits.shape[0])
        logit_r4_delta = max(logit_r4_delta, _maximum_delta(plain_logits, geometric_logits))
        attention_r4_delta = max(
            attention_r4_delta, _maximum_delta(plain_weights, geometric_weights)
        )
        logit_incremental_delta = max(
            logit_incremental_delta,
            _maximum_delta(geometric_logits, incremental_logits),
        )
        plain_r4_top1 &= bool(
            torch.equal(plain_logits.argmax(dim=-1), geometric_logits.argmax(dim=-1))
        )
        plain_incremental_top1 &= bool(
            torch.equal(
                geometric_logits.argmax(dim=-1),
                incremental_logits.argmax(dim=-1),
            )
        )
        # The incremental weight tensor is checked for a stable public shape;
        # equality to the full path is not a frozen terminal criterion.
        if incremental_weights.shape != plain_weights.shape:
            raise ValueError("#1043 incremental attention-weight shape differs")
        target_reads = int(torch.count_nonzero(labels != IGNORE_INDEX))
        for output, execution, full_square in (
            (plain, "plain", True),
            (geometric, "r4", True),
            (incremental, "r4", False),
        ):
            _accumulate_audit(
                work,
                _validated_call_audit(
                    output.audit,
                    batch=int(inputs.shape[0]),
                    time_steps=int(inputs.shape[1]),
                    target_reads=target_reads,
                    execution=execution,  # type: ignore[arg-type]
                    intervention="native",
                    full_square=full_square,
                ),
            )
        _check_deadline(deadline)
    return {
        "decisions": decisions,
        "r4_plain_attention_weight_max_delta": attention_r4_delta,
        "r4_plain_logit_max_delta": logit_r4_delta,
        "r4_plain_top1_identical": plain_r4_top1,
        "full_incremental_logit_max_delta": logit_incremental_delta,
        "full_incremental_top1_identical": plain_incremental_top1,
        "work": work,
    }


@torch.no_grad()
def _parity_natural(
    model: R4PositionPreservingCausalKVBindingV1,
    store: Any,
    *,
    device: torch.device,
    deadline: float | None = None,
) -> dict[str, Any]:
    logit_r4_delta = 0.0
    attention_r4_delta = 0.0
    logit_incremental_delta = 0.0
    plain_r4_top1 = True
    plain_incremental_top1 = True
    work: dict[str, int] = {}
    decisions = 0
    for start in range(0, len(store), BATCH_SIZE):
        _check_deadline(deadline)
        ordinals = range(start, min(len(store), start + BATCH_SIZE))
        inputs, labels = _natural_batch(store, ordinals, device=device)
        plain = model(inputs, labels, execution="plain", intervention="native")
        geometric = model(inputs, labels, execution="r4", intervention="native")
        incremental = model.forward_incremental(
            inputs,
            labels,
            execution="r4",
            intervention="native",
        )
        plain_logits, plain_weights = _selected_values(plain, labels)
        geometric_logits, geometric_weights = _selected_values(geometric, labels)
        incremental_logits, incremental_weights = _selected_values(incremental, labels)
        decisions += int(plain_logits.shape[0])
        logit_r4_delta = max(logit_r4_delta, _maximum_delta(plain_logits, geometric_logits))
        attention_r4_delta = max(
            attention_r4_delta, _maximum_delta(plain_weights, geometric_weights)
        )
        logit_incremental_delta = max(
            logit_incremental_delta,
            _maximum_delta(geometric_logits, incremental_logits),
        )
        plain_r4_top1 &= bool(
            torch.equal(plain_logits.argmax(dim=-1), geometric_logits.argmax(dim=-1))
        )
        plain_incremental_top1 &= bool(
            torch.equal(
                geometric_logits.argmax(dim=-1),
                incremental_logits.argmax(dim=-1),
            )
        )
        if incremental_weights.shape != plain_weights.shape:
            raise ValueError("#1043 incremental attention-weight shape differs")
        target_reads = int(torch.count_nonzero(labels != IGNORE_INDEX))
        for output, execution, full_square in (
            (plain, "plain", True),
            (geometric, "r4", True),
            (incremental, "r4", False),
        ):
            _accumulate_audit(
                work,
                _validated_call_audit(
                    output.audit,
                    batch=int(inputs.shape[0]),
                    time_steps=int(inputs.shape[1]),
                    target_reads=target_reads,
                    execution=execution,  # type: ignore[arg-type]
                    intervention="native",
                    full_square=full_square,
                ),
            )
        _check_deadline(deadline)
    return {
        "decisions": decisions,
        "r4_plain_attention_weight_max_delta": attention_r4_delta,
        "r4_plain_logit_max_delta": logit_r4_delta,
        "r4_plain_top1_identical": plain_r4_top1,
        "full_incremental_logit_max_delta": logit_incremental_delta,
        "full_incremental_top1_identical": plain_incremental_top1,
        "work": work,
    }


def _combine_parity(records: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    work: dict[str, int] = {}
    for record in records:
        for name, value in dict(record.get("work", {})).items():
            work[name] = work.get(name, 0) + int(value)
    return {
        "decisions": sum(int(record["decisions"]) for record in records),
        "r4_plain_attention_weight_max_delta": max(
            float(record["r4_plain_attention_weight_max_delta"]) for record in records
        ),
        "r4_plain_logit_max_delta": max(
            float(record["r4_plain_logit_max_delta"]) for record in records
        ),
        "r4_plain_top1_identical": all(
            record.get("r4_plain_top1_identical") is True for record in records
        ),
        "full_incremental_logit_max_delta": max(
            float(record["full_incremental_logit_max_delta"]) for record in records
        ),
        "full_incremental_top1_identical": all(
            record.get("full_incremental_top1_identical") is True for record in records
        ),
        "work": work,
    }


def _all_work_records(metrics: Mapping[str, Any]) -> list[Mapping[str, int]]:
    records: list[Mapping[str, int]] = []

    def visit(value: Any) -> None:
        if isinstance(value, Mapping):
            work = value.get("work")
            if isinstance(work, Mapping):
                records.append(work)  # type: ignore[arg-type]
            for key, nested in value.items():
                if key != "work":
                    visit(nested)
        elif isinstance(value, (tuple, list)):
            for nested in value:
                visit(nested)

    visit(metrics)
    return records


def _aggregate_evaluation_work(metrics: Mapping[str, Any]) -> dict[str, int]:
    result: dict[str, int] = {}
    for record in _all_work_records(metrics):
        for name, value in record.items():
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                raise ValueError("#1043 evaluation work ledger is malformed")
            result[name] = result.get(name, 0) + value
    return result


@torch.no_grad()
def _artifact_replay(
    root: Path,
    artifact_path: Path,
    examples: Sequence[Any],
    *,
    device: torch.device,
    data_api: Any,
    deadline: float | None = None,
) -> dict[str, Any]:
    _check_deadline(deadline)
    payload = artifact_path.read_bytes()
    left = _load_fitted_model(root, artifact_path, device=device).eval()
    right = _load_fitted_model(root, artifact_path, device=device).eval()
    inputs, labels = _causal_batch(data_api, examples[:2], device=device)
    left_output = left(inputs, labels, execution="r4", intervention="native")
    right_output = right(inputs, labels, execution="r4", intervention="native")
    logits_identical = bool(torch.equal(left_output.logits, right_output.logits))
    attention_identical = bool(
        torch.equal(left_output.attention_weights, right_output.attention_weights)
    )
    artifact_identical = bool(
        left.export_learned_artifact() == payload
        and right.export_learned_artifact() == payload
    )
    decisions = int(torch.count_nonzero(labels != IGNORE_INDEX))
    work: dict[str, int] = {}
    for output in (left_output, right_output):
        _accumulate_audit(
            work,
            _validated_call_audit(
                output.audit,
                batch=int(inputs.shape[0]),
                time_steps=int(inputs.shape[1]),
                target_reads=decisions,
                execution="r4",
                intervention="native",
                full_square=True,
            ),
        )
    _check_deadline(deadline)
    return {
        "artifact_bytes_identical": artifact_identical,
        "logits_identical": logits_identical,
        "attention_weights_identical": attention_identical,
        "artifact_cid": cid_bytes(payload),
        "decisions": decisions,
        "replay_logits_cid": cid_bytes(
            left_output.logits.detach().cpu().contiguous().numpy().tobytes()
        ),
        "passed": artifact_identical and logits_identical and attention_identical,
        "work": work,
    }


def _default_scoring_runner(
    root: Path,
    artifact_path: Path,
    *,
    terminal: Any,
    plan: ExecutionPlan,
    deadline: float,
) -> dict[str, Any]:
    data_api = _data_module()
    device = _configure_cpu(plan)
    _check_deadline(deadline)
    fitted = _load_fitted_model(root, artifact_path, device=device).eval()
    initial = _build_model(root, device=device).eval()
    _check_deadline(deadline)
    assignments = _world_assignments(terminal.english_history)
    metrics: dict[str, Any] = {
        "mqar": {
            "native": _score_examples(
                fitted,
                terminal.mqar,
                device=device,
                execution="r4",
                data_api=data_api,
                deadline=deadline,
            ),
            "current_only": _score_examples(
                fitted,
                terminal.mqar,
                device=device,
                execution="r4",
                intervention="current_only",
                data_api=data_api,
                deadline=deadline,
            ),
            "value_permuted": _score_examples(
                fitted,
                terminal.mqar,
                device=device,
                execution="r4",
                intervention="value_permuted",
                data_api=data_api,
                deadline=deadline,
            ),
            "binding_permuted": _score_examples(
                fitted,
                terminal.mqar_binding_permuted,
                device=device,
                execution="r4",
                data_api=data_api,
                deadline=deadline,
            ),
            "transport_mismatch": _score_examples(
                fitted,
                terminal.mqar,
                device=device,
                execution="r4",
                intervention="transport_mismatch",
                data_api=data_api,
                deadline=deadline,
            ),
        },
        "english": {
            "history": _score_examples(
                fitted,
                terminal.english_history,
                device=device,
                execution="r4",
                data_api=data_api,
                deadline=deadline,
            ),
            "binding_permuted": _score_examples(
                fitted,
                terminal.english_binding_permuted,
                device=device,
                execution="r4",
                data_api=data_api,
                deadline=deadline,
            ),
            "no_history": _score_examples(
                fitted,
                terminal.english_no_history,
                device=device,
                execution="r4",
                data_api=data_api,
                world_assignments=assignments,
                deadline=deadline,
            ),
        },
        "language": {
            "initialization": _score_natural(
                initial, terminal.natural_windows, device=device, deadline=deadline
            ),
            "fitted": _score_natural(
                fitted, terminal.natural_windows, device=device, deadline=deadline
            ),
        },
    }
    parity_records = (
        _parity_natural(
            fitted, terminal.natural_windows, device=device, deadline=deadline
        ),
        _parity_examples(
            fitted,
            terminal.mqar,
            device=device,
            data_api=data_api,
            deadline=deadline,
        ),
        _parity_examples(
            fitted,
            terminal.english_history,
            device=device,
            data_api=data_api,
            deadline=deadline,
        ),
        _parity_examples(
            fitted,
            terminal.english_no_history,
            device=device,
            data_api=data_api,
            deadline=deadline,
        ),
    )
    metrics["parity"] = _combine_parity(parity_records)
    metrics["replay"] = _artifact_replay(
        root,
        artifact_path,
        terminal.mqar,
        device=device,
        data_api=data_api,
        deadline=deadline,
    )
    work = _aggregate_evaluation_work(metrics)
    metrics["work"] = work
    metrics["leakage"] = {
        "target_reads": work.get("target_reads", 0),
        "source_reads": work.get("source_reads", 0),
        "provider_calls": work.get("provider_calls", 0),
        "teacher_calls": work.get("teacher_calls", 0),
        "future_reads": work.get("future_reads", 0),
        "forbidden_reads": work.get("forbidden_reads", 0),
    }
    _validate_terminal_metrics(metrics)
    return metrics


def _rate(record: Mapping[str, Any], *, decisions: int) -> float:
    if record.get("decisions") != decisions:
        raise ValueError("#1043 terminal decision count differs")
    correct = record.get("top1_correct")
    rate = record.get("top1_rate")
    if (
        isinstance(correct, bool)
        or not isinstance(correct, int)
        or not 0 <= correct <= decisions
        or not isinstance(rate, (int, float))
        or isinstance(rate, bool)
        or not math.isclose(float(rate), correct / decisions, rel_tol=0.0, abs_tol=1e-15)
    ):
        raise ValueError("#1043 terminal accuracy record is malformed")
    return float(rate)


def _nonnegative_int(value: Any, *, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError(f"#1043 {label} is not an exact nonnegative integer")
    return value


def _finite_float(value: Any, *, label: str, nonnegative: bool = True) -> float:
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(float(value))
        or (nonnegative and float(value) < 0.0)
    ):
        raise ValueError(f"#1043 {label} is not a finite numeric value")
    return float(value)


def _strict_mapping(value: Any, *, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"#1043 {label} is not a mapping")
    return value


def _validate_metric_work(value: Any, *, label: str) -> dict[str, int]:
    record = _strict_mapping(value, label=label)
    if set(record) != set(AUDIT_WORK_FIELDS):
        raise ValueError(f"#1043 {label} does not contain the exact work fields")
    result = {
        name: _nonnegative_int(record[name], label=f"{label}.{name}")
        for name in AUDIT_WORK_FIELDS
    }
    if (
        result["source_reads"] != result["token_steps"]
        or result["cache_writes"]
        != result["token_steps"] * LAYERS * 2 * HEADS * HEAD_DIM
        or result["vocabulary_scores"]
        != result["token_steps"] * VOCAB_SIZE
        or result["value_reads"]
        != result["materialized_attention_scores"] * HEAD_DIM
        or result["admitted_attention_scores"]
        > result["materialized_attention_scores"]
    ):
        raise ValueError(f"#1043 {label} violates exact work arithmetic")
    return result


def _validate_score_record(
    value: Any,
    *,
    label: str,
    decisions: int,
    execution: Literal["plain", "r4"],
    no_history: bool = False,
) -> tuple[Mapping[str, Any], dict[str, int]]:
    record = _strict_mapping(value, label=label)
    expected_fields = {
        "decisions",
        "top1_correct",
        "top1_rate",
        "nll_nats",
        "selected_logits_cid",
        "work",
    }
    if no_history:
        expected_fields.update(
            {
                "assigned_answer_top1_correct",
                "assigned_answer_top1_rate",
                "unsupported_assigned_value_top1",
            }
        )
    if set(record) != expected_fields:
        raise ValueError(f"#1043 {label} does not contain the exact metric fields")
    rate = _rate(record, decisions=decisions)
    del rate
    _finite_float(record["nll_nats"], label=f"{label}.nll_nats")
    if not _is_blake3_cid(record["selected_logits_cid"]):
        raise ValueError(f"#1043 {label} selected-logit CID is malformed")
    work = _validate_metric_work(record["work"], label=f"{label}.work")
    if work["target_reads"] != decisions or work["source_reads"] < decisions:
        raise ValueError(f"#1043 {label} target/source work differs")
    expected_transport = (
        work["materialized_attention_scores"] * 2 * (HEAD_DIM // 4)
        if execution == "r4"
        else 0
    )
    if work["transported_r4_blocks"] != expected_transport:
        raise ValueError(f"#1043 {label} transport work differs")
    if no_history:
        assigned = _nonnegative_int(
            record["assigned_answer_top1_correct"],
            label=f"{label}.assigned_answer_top1_correct",
        )
        unsupported = _nonnegative_int(
            record["unsupported_assigned_value_top1"],
            label=f"{label}.unsupported_assigned_value_top1",
        )
        assigned_rate = _finite_float(
            record["assigned_answer_top1_rate"],
            label=f"{label}.assigned_answer_top1_rate",
        )
        if (
            assigned > decisions
            or unsupported > decisions
            or not math.isclose(
                assigned_rate,
                assigned / decisions,
                rel_tol=0.0,
                abs_tol=1.0e-15,
            )
        ):
            raise ValueError(f"#1043 {label} no-history metric differs")
    return record, work


def _sum_work(records: Sequence[Mapping[str, int]]) -> dict[str, int]:
    return {
        name: sum(record[name] for record in records)
        for name in AUDIT_WORK_FIELDS
    }


def _validate_terminal_metrics(metrics: Mapping[str, Any]) -> None:
    if set(metrics) != {
        "mqar",
        "english",
        "language",
        "parity",
        "replay",
        "work",
        "leakage",
    }:
        raise ValueError("#1043 terminal metrics have an unexpected top-level schema")
    mqar = _strict_mapping(metrics["mqar"], label="mqar")
    english = _strict_mapping(metrics["english"], label="english")
    language = _strict_mapping(metrics["language"], label="language")
    if set(mqar) != {
        "native",
        "current_only",
        "value_permuted",
        "binding_permuted",
        "transport_mismatch",
    }:
        raise ValueError("#1043 MQAR metric arms differ")
    if set(english) != {"history", "binding_permuted", "no_history"}:
        raise ValueError("#1043 English metric arms differ")
    if set(language) != {"initialization", "fitted"}:
        raise ValueError("#1043 language metric arms differ")

    work_records: list[dict[str, int]] = []
    for name in (
        "native",
        "current_only",
        "value_permuted",
        "binding_permuted",
        "transport_mismatch",
    ):
        _record_value, work = _validate_score_record(
            mqar[name],
            label=f"mqar.{name}",
            decisions=TERMINAL_MQAR_DECISIONS,
            execution="r4",
        )
        work_records.append(work)
    for name in ("history", "binding_permuted", "no_history"):
        _record_value, work = _validate_score_record(
            english[name],
            label=f"english.{name}",
            decisions=(
                TERMINAL_ENGLISH_NO_HISTORY_DECISIONS
                if name == "no_history"
                else TERMINAL_ENGLISH_HISTORY_DECISIONS
            ),
            execution="r4",
            no_history=name == "no_history",
        )
        work_records.append(work)
    for name in ("initialization", "fitted"):
        _record_value, work = _validate_score_record(
            language[name],
            label=f"language.{name}",
            decisions=TERMINAL_NATURAL_DECISIONS,
            execution="plain",
        )
        work_records.append(work)

    parity = _strict_mapping(metrics["parity"], label="parity")
    if set(parity) != {
        "decisions",
        "r4_plain_attention_weight_max_delta",
        "r4_plain_logit_max_delta",
        "r4_plain_top1_identical",
        "full_incremental_logit_max_delta",
        "full_incremental_top1_identical",
        "work",
    }:
        raise ValueError("#1043 parity metric fields differ")
    if (
        _nonnegative_int(parity["decisions"], label="parity.decisions")
        != TERMINAL_PARITY_DECISIONS
        or parity["r4_plain_top1_identical"] is not True
        and parity["r4_plain_top1_identical"] is not False
        or parity["full_incremental_top1_identical"] is not True
        and parity["full_incremental_top1_identical"] is not False
    ):
        raise ValueError("#1043 parity counts or booleans differ")
    for name in (
        "r4_plain_attention_weight_max_delta",
        "r4_plain_logit_max_delta",
        "full_incremental_logit_max_delta",
    ):
        _finite_float(parity[name], label=f"parity.{name}")
    parity_work = _validate_metric_work(parity["work"], label="parity.work")
    if parity_work["target_reads"] != 3 * TERMINAL_PARITY_DECISIONS:
        raise ValueError("#1043 parity target-read ledger differs")
    work_records.append(parity_work)

    replay = _strict_mapping(metrics["replay"], label="replay")
    if set(replay) != {
        "artifact_bytes_identical",
        "logits_identical",
        "attention_weights_identical",
        "artifact_cid",
        "decisions",
        "replay_logits_cid",
        "passed",
        "work",
    }:
        raise ValueError("#1043 replay metric fields differ")
    replay_flags = (
        replay["artifact_bytes_identical"],
        replay["logits_identical"],
        replay["attention_weights_identical"],
    )
    if (
        any(value is not True and value is not False for value in replay_flags)
        or replay["passed"] is not all(value is True for value in replay_flags)
        or _nonnegative_int(replay["decisions"], label="replay.decisions")
        != TERMINAL_REPLAY_DECISIONS
        or not _is_blake3_cid(replay["artifact_cid"])
        or not _is_blake3_cid(replay["replay_logits_cid"])
    ):
        raise ValueError("#1043 replay metric values differ")
    replay_work = _validate_metric_work(replay["work"], label="replay.work")
    if (
        replay_work["target_reads"] != 2 * TERMINAL_REPLAY_DECISIONS
        or replay_work["transported_r4_blocks"]
        != replay_work["materialized_attention_scores"] * 2 * (HEAD_DIM // 4)
    ):
        raise ValueError("#1043 replay work differs")
    work_records.append(replay_work)

    aggregate = _sum_work(work_records)
    observed_work = _validate_metric_work(metrics["work"], label="work")
    if (
        observed_work != aggregate
        or aggregate["target_reads"] != EXPECTED_EVALUATION_TARGET_READS
    ):
        raise ValueError("#1043 aggregate evaluation work ledger differs")
    leakage = _strict_mapping(metrics["leakage"], label="leakage")
    leakage_fields = (
        "target_reads",
        "source_reads",
        "provider_calls",
        "teacher_calls",
        "future_reads",
        "forbidden_reads",
    )
    if set(leakage) != set(leakage_fields):
        raise ValueError("#1043 leakage metric fields differ")
    for name in leakage_fields:
        if (
            _nonnegative_int(leakage[name], label=f"leakage.{name}")
            != aggregate[name]
        ):
            raise ValueError("#1043 leakage projection differs from exact work")


def decide_position_kv_binding(metrics: Mapping[str, Any]) -> dict[str, Any]:
    """Apply the frozen #1043 gates without fitting or threshold adjustment."""

    try:
        _validate_terminal_metrics(metrics)
        mqar = _field(metrics, "mqar")
        english = _field(metrics, "english")
        language = _field(metrics, "language")
        parity = _field(metrics, "parity")
        replay = _field(metrics, "replay")
        leakage = _field(metrics, "leakage")
        native = _field(mqar, "native")
        current = _field(mqar, "current_only")
        value_permuted = _field(mqar, "value_permuted")
        binding_permuted = _field(mqar, "binding_permuted")
        transport = _field(mqar, "transport_mismatch")
        history = _field(english, "history")
        english_binding_permuted = _field(english, "binding_permuted")
        no_history = _field(english, "no_history")
        initialization = _field(language, "initialization")
        fitted = _field(language, "fitted")
        native_rate = _rate(native, decisions=TERMINAL_MQAR_DECISIONS)
        current_rate = _rate(current, decisions=TERMINAL_MQAR_DECISIONS)
        value_rate = _rate(value_permuted, decisions=TERMINAL_MQAR_DECISIONS)
        binding_rate = _rate(binding_permuted, decisions=TERMINAL_MQAR_DECISIONS)
        transport_rate = _rate(transport, decisions=TERMINAL_MQAR_DECISIONS)
        history_rate = _rate(
            history, decisions=TERMINAL_ENGLISH_HISTORY_DECISIONS
        )
        english_binding_rate = _rate(
            english_binding_permuted,
            decisions=TERMINAL_ENGLISH_HISTORY_DECISIONS,
        )
        unknown_rate = _rate(
            no_history, decisions=TERMINAL_ENGLISH_NO_HISTORY_DECISIONS
        )
        initial_top1 = _rate(
            initialization, decisions=TERMINAL_NATURAL_DECISIONS
        )
        fitted_top1 = _rate(fitted, decisions=TERMINAL_NATURAL_DECISIONS)
    except (AttributeError, KeyError, TypeError, ValueError) as error:
        return {
            "verdict": TERMINAL_INVALID,
            "action": "repair evidence serialization; no scientific inference is allowed",
            "invalid_reason": str(error),
            "gates": {},
        }
    parity_gate = bool(
        float(parity.get("r4_plain_attention_weight_max_delta", math.inf))
        <= ATTENTION_PARITY_TOLERANCE
        and float(parity.get("r4_plain_logit_max_delta", math.inf))
        <= LOGIT_PARITY_TOLERANCE
        and parity.get("r4_plain_top1_identical") is True
        and float(parity.get("full_incremental_logit_max_delta", math.inf))
        <= LOGIT_PARITY_TOLERANCE
        and parity.get("full_incremental_top1_identical") is True
    )
    leakage_gate = bool(
        all(
            leakage.get(name) == 0
            for name in ("provider_calls", "teacher_calls", "future_reads", "forbidden_reads")
        )
        and int(leakage.get("target_reads", 0)) > 0
        and int(leakage.get("source_reads", 0)) > 0
    )
    replay_gate = bool(
        replay.get("artifact_bytes_identical") is True
        and replay.get("logits_identical") is True
        and replay.get("passed") is True
    )
    mechanics_gate = parity_gate and leakage_gate and replay_gate
    mqar_absolute_gate = int(native["top1_correct"]) >= MQAR_REQUIRED_CORRECT
    mqar_attribution_gates = {
        "current_only_drop": native_rate - current_rate >= MQAR_CONTROL_DROP,
        "value_permuted_drop": native_rate - value_rate >= MQAR_CONTROL_DROP,
        "binding_permuted_drop": native_rate - binding_rate >= MQAR_CONTROL_DROP,
    }
    geometry_attribution = {
        "transport_mismatch_drop": native_rate - transport_rate
        >= MQAR_TRANSPORT_DROP,
    }
    assigned_rate = float(no_history.get("assigned_answer_top1_rate", math.inf))
    english_gates = {
        "history_absolute": int(history["top1_correct"])
        >= ENGLISH_HISTORY_REQUIRED_CORRECT,
        "binding_permuted_drop": history_rate - english_binding_rate
        >= ENGLISH_BINDING_PERMUTED_DROP,
        "history_minus_no_history": history_rate - assigned_rate
        >= ENGLISH_HISTORY_NO_HISTORY_DROP,
        "unknown_absolute": int(no_history["top1_correct"])
        >= ENGLISH_UNKNOWN_REQUIRED_CORRECT,
        "unsupported_assigned_value": no_history.get(
            "unsupported_assigned_value_top1"
        )
        == ENGLISH_UNSUPPORTED_ALLOWED,
    }
    initial_nll = float(initialization.get("nll_nats", math.inf))
    fitted_nll = float(fitted.get("nll_nats", math.inf))
    language_gates = {
        "nll_nonregression": fitted_nll <= initial_nll + LANGUAGE_NLL_TOLERANCE,
        "top1_nonregression": fitted_top1
        >= initial_top1 - LANGUAGE_TOP1_TOLERANCE,
    }
    gates = {
        "mechanics": {
            "parity": parity_gate,
            "leakage": leakage_gate,
            "replay": replay_gate,
            "exact_work": True,
        },
        "mqar": {
            "absolute": mqar_absolute_gate,
            **mqar_attribution_gates,
        },
        "english": english_gates,
        "geometry_attribution": geometry_attribution,
        "language": language_gates,
    }
    if not mechanics_gate:
        verdict = TERMINAL_INVALID
        action = "repair the failed causal, parity, identity, replay, or leak gate; do not interpret metrics"
    elif not mqar_absolute_gate:
        verdict = TERMINAL_NOT_LEARNED
        action = "stop; reconsider role encoding or curriculum in a new frozen issue"
    elif not all(mqar_attribution_gates.values()):
        verdict = TERMINAL_UNATTRIBUTED
        action = "stop; binding accuracy is not causally attributed to the supplied key/value records"
    elif not all(english_gates.values()):
        verdict = TERMINAL_SYNTHETIC_ONLY
        action = "stop; isolate natural template and role transfer in a new issue"
    elif not all(language_gates.values()):
        verdict = TERMINAL_LANGUAGE_REGRESSION
        action = "stop; revisit joint language preservation in a new issue"
    elif not all(geometry_attribution.values()):
        verdict = TERMINAL_GEOMETRY_UNATTRIBUTED
        action = "stop; binding passed but causal dependence on coherent R4 transport was not established"
    else:
        verdict = TERMINAL_PASS
        action = "freeze a separate source-free context-conditioned generation rung"
    return {"verdict": verdict, "action": action, "gates": gates}


def _post_fit_unavailable_result(
    root: Path,
    *,
    preparation: Mapping[str, Any],
    preflight: Mapping[str, Any],
    fit: Mapping[str, Any],
    artifact: Mapping[str, Any],
    reason: str,
    reveal: Mapping[str, Any] | None,
) -> dict[str, Any]:
    return _write_result_once(
        root,
        {
            "schema": RESULT_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "preparation_cid": preparation["preparation_cid"],
            "preflight_cid": preflight["preflight_cid"],
            "fit_cid": fit["fit_cid"],
            "reveal_cid": None if reveal is None else reveal["reveal_cid"],
            "scoring_cid": None,
            "artifact": dict(artifact),
            "decision": {
                "verdict": TERMINAL_UNAVAILABLE,
                "phase": "SCORING",
                "reason": reason,
                "action": "stop; do not interpret partial metrics or alter the frozen run",
            },
            "verdict": TERMINAL_UNAVAILABLE,
            "terminal_payload_reads_before_artifact_cid": 0,
            "terminal_payload_revealed": reveal is not None,
            "optimizer_created_after_reveal": False,
            "optimizer_steps_after_reveal": 0,
        },
    )


def finalize_position_kv_binding_campaign(
    root: Path,
) -> dict[str, Any]:
    """Reveal once after artifact commitment, score, and create one result."""

    root = root.resolve()
    paths = _phase_paths(root)
    if paths["result"].exists():
        return validate_position_kv_binding_result(root)
    preparation = _read_json(paths["preparation"], cid_field="preparation_cid")
    preflight = _read_json(paths["preflight"], cid_field="preflight_cid")
    fit = _read_json(paths["fit"], cid_field="fit_cid")
    implementation = _require_current_implementation(fit.get("implementation"))
    if (
        fit.get("preparation_cid") != preparation["preparation_cid"]
        or fit.get("preflight_cid") != preflight["preflight_cid"]
        or fit.get("completed_steps") != OPTIMIZER_STEPS
        or fit.get("optimizer_steps_after_reveal") != 0
    ):
        raise ValueError("#1043 fit envelope differs from the frozen trajectory")
    artifact_record = fit.get("artifact")
    if not isinstance(artifact_record, Mapping):
        raise ValueError("#1043 fit has no artifact record")
    artifact_path = paths["artifact"]
    if (
        artifact_record.get("path") != ARTIFACT_RELATIVE_PATH
        or artifact_record.get("bytes") != artifact_path.stat().st_size
        or artifact_record.get("cid") != cid_file(artifact_path)
    ):
        raise ValueError("#1043 final artifact does not reproduce its fit CID")
    fit_elapsed = float(fit.get("elapsed_seconds", 0.0))
    if not math.isfinite(fit_elapsed) or fit_elapsed < 0.0:
        raise ValueError("#1043 fit elapsed time is malformed")
    if fit_elapsed >= HARD_WALL_SECONDS:
        return _post_fit_unavailable_result(
            root,
            preparation=preparation,
            preflight=preflight,
            fit=fit,
            artifact=artifact_record,
            reason="the fit consumed the complete 1,800-second total wall",
            reveal=None,
        )
    plan = _selected_plan(preflight)
    if plan is None or fit.get("plan") != plan.identity():
        raise ValueError("#1043 scoring plan differs from the selected CPU plan")
    if paths["scoring"].exists():
        data_api = _data_module()
        terminal = data_api.load_revealed_position_kv_binding_terminal(
            root, final_artifact_path=artifact_path
        )
        reveal = _record(_field(terminal, "reveal"))
        _verify_self_cid(reveal, "reveal_cid")
        if (
            reveal.get("final_artifact_cid") != artifact_record["cid"]
            or reveal.get("fit_cid") != fit["fit_cid"]
            or _field(terminal, "final_artifact_cid") != artifact_record["cid"]
        ):
            raise ValueError("#1043 reveal binds another final artifact")
        scoring = _read_json(paths["scoring"], cid_field="scoring_cid")
        if (
            scoring.get("fit_cid") != fit["fit_cid"]
            or scoring.get("reveal_cid") != reveal["reveal_cid"]
            or scoring.get("artifact_cid") != artifact_record["cid"]
            or scoring.get("plan") != plan.identity()
        ):
            raise ValueError("#1043 cached scoring binds another reveal")
        metrics = scoring.get("metrics")
        if not isinstance(metrics, Mapping):
            raise ValueError("#1043 cached scoring has no metrics")
        _validate_terminal_metrics(metrics)
        scoring_elapsed = float(scoring.get("scoring_elapsed_seconds", math.inf))
        total_elapsed = float(scoring.get("total_elapsed_seconds", math.inf))
        if (
            not math.isfinite(scoring_elapsed)
            or scoring_elapsed < 0.0
            or not math.isclose(
                total_elapsed, fit_elapsed + scoring_elapsed, rel_tol=0.0, abs_tol=1e-9
            )
            or total_elapsed > HARD_WALL_SECONDS
        ):
            raise ValueError("#1043 cached scoring wall ledger differs")
    else:
        scoring_started = time.monotonic()
        deadline = scoring_started + (HARD_WALL_SECONDS - fit_elapsed)
        reveal: dict[str, Any] | None = None
        try:
            data_api = _data_module()
            if paths["reveal"].exists():
                terminal = data_api.load_revealed_position_kv_binding_terminal(
                    root, final_artifact_path=artifact_path
                )
            else:
                terminal = data_api.reveal_position_kv_binding_terminal(
                    root, final_artifact_path=artifact_path
                )
            reveal = _record(_field(terminal, "reveal"))
            _verify_self_cid(reveal, "reveal_cid")
            if (
                reveal.get("final_artifact_cid") != artifact_record["cid"]
                or reveal.get("fit_cid") != fit["fit_cid"]
                or _field(terminal, "final_artifact_cid")
                != artifact_record["cid"]
            ):
                raise ValueError("#1043 reveal binds another final artifact")
            _check_deadline(deadline)
            metrics = dict(
                _default_scoring_runner(
                    root,
                    artifact_path,
                    terminal=terminal,
                    plan=plan,
                    deadline=deadline,
                )
            )
            _validate_terminal_metrics(metrics)
        except (TimeoutError, KeyboardInterrupt) as error:
            if reveal is None and paths["reveal"].exists():
                candidate = _read_json(paths["reveal"], cid_field="reveal_cid")
                if candidate.get("final_artifact_cid") == artifact_record["cid"]:
                    reveal = candidate
            return _post_fit_unavailable_result(
                root,
                preparation=preparation,
                preflight=preflight,
                fit=fit,
                artifact=artifact_record,
                reason=(
                    str(error)
                    if isinstance(error, TimeoutError)
                    else "terminal reveal or scoring was interrupted"
                ),
                reveal=reveal,
            )
        scoring_elapsed = time.monotonic() - scoring_started
        total_elapsed = fit_elapsed + scoring_elapsed
        if total_elapsed > HARD_WALL_SECONDS:
            return _post_fit_unavailable_result(
                root,
                preparation=preparation,
                preflight=preflight,
                fit=fit,
                artifact=artifact_record,
                reason="fit plus terminal scoring exceeded the 1,800-second total wall",
                reveal=reveal,
            )
        decision = decide_position_kv_binding(metrics)
        scoring = _with_cid(
            {
                "schema": SCORING_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "preparation_cid": preparation["preparation_cid"],
                "preflight_cid": preflight["preflight_cid"],
                "fit_cid": fit["fit_cid"],
                "reveal_cid": reveal["reveal_cid"],
                "artifact_cid": artifact_record["cid"],
                "implementation": implementation,
                "plan": plan.identity(),
                "metrics": metrics,
                "decision": decision,
                "fit_elapsed_seconds": fit_elapsed,
                "scoring_elapsed_seconds": scoring_elapsed,
                "total_elapsed_seconds": total_elapsed,
                "total_wall_seconds": HARD_WALL_SECONDS,
                "optimizer_created_after_reveal": False,
                "optimizer_steps_after_reveal": 0,
            },
            "scoring_cid",
        )
        _write_exclusive_json(paths["scoring"], scoring)
    decision = decide_position_kv_binding(metrics)
    if scoring.get("decision") != decision:
        raise ValueError("#1043 scoring decision does not reproduce")
    result_body = {
        "schema": RESULT_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "fit_cid": fit["fit_cid"],
        "reveal_cid": reveal["reveal_cid"],
        "scoring_cid": scoring["scoring_cid"],
        "artifact": dict(artifact_record),
        "implementation": implementation,
        "population": {
            "data_manifest_cid": preparation["data_manifest_cid"],
            "terminal_artifact_cid": artifact_record["cid"],
            "reveal_count": reveal.get("reveal_count"),
        },
        "fit": {
            "completed_steps": fit["completed_steps"],
            "presentations": fit["presentations"],
            "work": fit["work"],
            "loss_trace_cid": fit["loss_trace_cid"],
        },
        "metrics": metrics,
        "decision": decision,
        "verdict": decision["verdict"],
        "terminal_payload_reads_before_artifact_cid": 0,
        "optimizer_created_after_reveal": False,
        "optimizer_steps_after_reveal": 0,
        "fit_elapsed_seconds": fit_elapsed,
        "scoring_elapsed_seconds": scoring["scoring_elapsed_seconds"],
        "total_elapsed_seconds": scoring["total_elapsed_seconds"],
        "total_wall_seconds": HARD_WALL_SECONDS,
        "generation": "NOT_RUN",
        "reasoning": "NOT_RUN",
        "recurrence_compression": "NOT_RUN",
        "lowering": "NOT_RUN",
    }
    return _write_result_once(root, result_body)


def validate_position_kv_binding_result(root: Path) -> dict[str, Any]:
    """Reproduce the create-once terminal envelope and all phase bindings."""

    root = root.resolve()
    paths = _phase_paths(root)
    result = _read_json(paths["result"], cid_field="result_cid")
    if result.get("verdict") == TERMINAL_UNAVAILABLE:
        preparation = _read_json(paths["preparation"], cid_field="preparation_cid")
        preflight = _read_json(paths["preflight"], cid_field="preflight_cid")
        if (
            result.get("preparation_cid") != preparation["preparation_cid"]
            or result.get("preflight_cid") != preflight["preflight_cid"]
            or result.get("optimizer_steps_after_reveal") != 0
        ):
            raise ValueError("#1043 unavailable result binding differs")
        if result.get("fit_cid") is not None:
            fit = _read_json(paths["fit"], cid_field="fit_cid")
            artifact = result.get("artifact")
            if (
                result.get("fit_cid") != fit["fit_cid"]
                or not isinstance(artifact, Mapping)
                or artifact.get("cid") != cid_file(paths["artifact"])
            ):
                raise ValueError("#1043 post-fit unavailable binding differs")
            if result.get("reveal_cid") is not None:
                reveal = _read_json(paths["reveal"], cid_field="reveal_cid")
                if (
                    result.get("reveal_cid") != reveal["reveal_cid"]
                    or reveal.get("final_artifact_cid") != artifact.get("cid")
                    or reveal.get("fit_cid") != fit["fit_cid"]
                ):
                    raise ValueError("#1043 post-reveal unavailable binding differs")
        return result
    preparation = _read_json(paths["preparation"], cid_field="preparation_cid")
    preflight = _read_json(paths["preflight"], cid_field="preflight_cid")
    fit = _read_json(paths["fit"], cid_field="fit_cid")
    reveal = _read_json(paths["reveal"], cid_field="reveal_cid")
    scoring = _read_json(paths["scoring"], cid_field="scoring_cid")
    artifact = result.get("artifact")
    if not isinstance(artifact, Mapping):
        raise ValueError("#1043 result artifact record is malformed")
    bindings = {
        "preparation_cid": preparation["preparation_cid"],
        "preflight_cid": preflight["preflight_cid"],
        "fit_cid": fit["fit_cid"],
        "reveal_cid": reveal["reveal_cid"],
        "scoring_cid": scoring["scoring_cid"],
    }
    if any(result.get(name) != value for name, value in bindings.items()):
        raise ValueError("#1043 result phase-CID chain differs")
    if (
        artifact.get("path") != ARTIFACT_RELATIVE_PATH
        or artifact.get("cid") != cid_file(paths["artifact"])
        or artifact.get("bytes") != paths["artifact"].stat().st_size
        or reveal.get("final_artifact_cid") != artifact.get("cid")
        or reveal.get("fit_cid") != fit["fit_cid"]
        or scoring.get("artifact_cid") != artifact.get("cid")
        or scoring.get("plan") != fit.get("plan")
        or scoring.get("implementation") != fit.get("implementation")
        or result.get("metrics") != scoring.get("metrics")
        or result.get("decision") != scoring.get("decision")
        or result.get("decision") != decide_position_kv_binding(result["metrics"])
        or result.get("verdict") != result.get("decision", {}).get("verdict")
        or result.get("optimizer_steps_after_reveal") != 0
    ):
        raise ValueError("#1043 terminal artifact/reveal/result binding differs")
    _require_current_implementation(result.get("implementation"))
    return result


def run_position_kv_binding_campaign(root: Path) -> dict[str, Any]:
    """Fit the frozen prepared campaign and finalize it exactly once."""

    fit = fit_position_kv_binding_campaign(root)
    if fit.get("verdict") == TERMINAL_UNAVAILABLE:
        return fit
    return finalize_position_kv_binding_campaign(root)
