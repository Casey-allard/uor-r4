"""Frozen data and leakage boundary for issue #1043.

The construction loader deliberately verifies only public and construction
artifacts.  Terminal MQAR, English, and language payloads are generated and
committed during preparation, then placed below a mode-000 directory.  The
only function that opens that directory first verifies a real final model
artifact and records its CID in a create-once reveal envelope.
"""

from __future__ import annotations

import json
import math
import os
import re
import shutil
import stat
import struct
import tempfile
from collections.abc import Collection, Iterable, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Literal

import numpy as np
import torch
from blake3 import blake3
from tokenizers import Tokenizer
from torch import Tensor

from .language_path_generalization_data import (
    EXPECTED_SOURCE_TRAIN_STORE_CID,
    EXPECTED_TOKENIZER_CID,
    EXPECTED_TRAIN_SLICE_CID,
    LanguagePathWindowStore,
)
from .provenance import (
    artifact_records,
    canonical_json_bytes,
    cid_bytes,
    cid_file,
    tree_cid,
    verify_artifact_subset,
    verify_manifest_envelope,
)


ISSUE = 1043
POLICY = "R4PositionPreservingCausalKVBindingV1"
SEED = 10_043
VOCAB_SIZE = 4_096
CONTEXT = 120
WINDOW_TOKENS = CONTEXT + 1
IGNORE_INDEX = -100
BOS_TOKEN_ID = 0

NATURAL_SOURCE_WINDOWS = 43_680
NATURAL_CONSTRUCTION_WINDOWS = 21_840
NATURAL_CONSTRUCTION_DECISIONS = NATURAL_CONSTRUCTION_WINDOWS * CONTEXT
NATURAL_TERMINAL_WINDOWS = 2_066
NATURAL_TERMINAL_DECISIONS = NATURAL_TERMINAL_WINDOWS * CONTEXT

MQAR_RECORDS = 8
MQAR_QUERIES = 8
MQAR_CONSTRUCTION_SEQUENCES = 10_920
MQAR_CONSTRUCTION_DECISIONS = MQAR_CONSTRUCTION_SEQUENCES * MQAR_QUERIES
MQAR_TERMINAL_SEQUENCES = 1_024
MQAR_TERMINAL_DECISIONS = MQAR_TERMINAL_SEQUENCES * MQAR_QUERIES
MQAR_FILLER_MIN = 2
MQAR_FILLER_MAX = 255
MQAR_KEY_MIN = 256
MQAR_KEY_MAX = 2_047
MQAR_VALUE_MIN = 2_048
MQAR_VALUE_MAX = 4_095

ENGLISH_CONSTRUCTION_HISTORY = 8_190
ENGLISH_CONSTRUCTION_NO_HISTORY = 2_730
ENGLISH_TERMINAL_WORLDS = 256
ENGLISH_TERMINAL_QUERIES_PER_WORLD = 2
ENGLISH_TERMINAL_HISTORY = (
    ENGLISH_TERMINAL_WORLDS * ENGLISH_TERMINAL_QUERIES_PER_WORLD
)
ENGLISH_TERMINAL_NO_HISTORY = ENGLISH_TERMINAL_HISTORY
UNKNOWN_TOKEN_ID = 2_823

# Exact public V5 boundary and #1019 index identity.  Duplicating these three
# immutable identities avoids importing a prior campaign orchestration module
# into the new data boundary.
FRESH_HELDOUT_LAST_CAPACITY_STORY = 766_489
FRESH_HELDOUT_LAST_SOURCE_STORY = 851_190
FRESH_HELDOUT_TRAIN_INDEX_CID = (
    "blake3:0032889e32b38801476223c5bed7e401d77b61afbbd6cf9afddaceee18e2136e"
)

KEY_LEXICON = (
    "spoon",
    "marble",
    "kite",
    "boat",
    "bell",
    "key",
    "coin",
    "book",
    "ball",
    "cup",
    "hat",
    "drum",
    "doll",
    "ring",
    "lamp",
    "rope",
)
VALUE_LEXICON = (
    "garden",
    "kitchen",
    "attic",
    "basket",
    "drawer",
    "shelf",
    "pond",
    "cave",
    "barn",
    "forest",
    "beach",
    "table",
    "bed",
    "door",
    "tree",
    "box",
    "chair",
)

MANIFEST_RELATIVE_PATH = "position-kv-binding-data-manifest.json"
COMMITMENT_RELATIVE_PATH = "evaluation/commitment.json"
REVEAL_RELATIVE_PATH = "evaluation/reveal.json"
SEALED_DIRECTORY_RELATIVE_PATH = "evaluation/sealed"
CONSTRUCTION_NATURAL_RELATIVE_PATH = "construction/natural.u16"
CONSTRUCTION_NATURAL_SELECTION_RELATIVE_PATH = "construction/natural-selection.json"
CONSTRUCTION_MQAR_RELATIVE_PATH = "construction/mqar.json"
CONSTRUCTION_ENGLISH_RELATIVE_PATH = "construction/english.json"
TERMINAL_NATURAL_RELATIVE_PATH = "evaluation/sealed/natural.u16"
TERMINAL_NATURAL_SELECTION_RELATIVE_PATH = "evaluation/sealed/natural-selection.json"
TERMINAL_MQAR_RELATIVE_PATH = "evaluation/sealed/mqar.json"
TERMINAL_ENGLISH_RELATIVE_PATH = "evaluation/sealed/english.json"

MANIFEST_SCHEMA = "uor-r4.position-kv-binding-data/1"
COMMITMENT_SCHEMA = "uor-r4.position-kv-binding-commitment/1"
REVEAL_SCHEMA = "uor-r4.position-kv-binding-reveal/1"
MQAR_SCHEMA = "uor-r4.position-kv-binding-mqar/1"
ENGLISH_SCHEMA = "uor-r4.position-kv-binding-english/1"
NATURAL_SELECTION_SCHEMA = "uor-r4.position-kv-binding-natural-selection/1"

# The terminal-data boundary independently recognizes the campaign envelopes
# that are allowed to unlock it.  Keeping these literals here avoids a circular
# import from the campaign orchestrator while still making a loose file with a
# plausible ``.safetensors`` suffix insufficient to reveal the population.
CAMPAIGN_PREPARATION_RELATIVE_PATH = "position-kv-binding-preparation.json"
CAMPAIGN_PREFLIGHT_RELATIVE_PATH = "preflight/position-kv-binding-preflight.json"
CAMPAIGN_STARTED_RELATIVE_PATH = "run/position-kv-binding-started.json"
CAMPAIGN_FIT_RELATIVE_PATH = "run/position-kv-binding-fit.json"
CAMPAIGN_ARTIFACT_RELATIVE_PATH = "artifact/model.safetensors"
CAMPAIGN_PREPARATION_SCHEMA = "uor-r4.position-kv-binding-preparation/1"
CAMPAIGN_PREFLIGHT_SCHEMA = "uor-r4.position-kv-binding-preflight/1"
CAMPAIGN_STARTED_SCHEMA = "uor-r4.position-kv-binding-started/1"
CAMPAIGN_FIT_SCHEMA = "uor-r4.position-kv-binding-fit/1"
CAMPAIGN_OPTIMIZER_STEPS = 2_730
CAMPAIGN_ARTIFACT_BYTES = 1_010_800

CONSTRUCTION_ARTIFACT_PATHS = frozenset(
    {
        CONSTRUCTION_NATURAL_RELATIVE_PATH,
        CONSTRUCTION_NATURAL_SELECTION_RELATIVE_PATH,
        CONSTRUCTION_MQAR_RELATIVE_PATH,
        CONSTRUCTION_ENGLISH_RELATIVE_PATH,
    }
)
TERMINAL_ARTIFACT_PATHS = frozenset(
    {
        TERMINAL_NATURAL_RELATIVE_PATH,
        TERMINAL_NATURAL_SELECTION_RELATIVE_PATH,
        TERMINAL_MQAR_RELATIVE_PATH,
        TERMINAL_ENGLISH_RELATIVE_PATH,
    }
)
ALL_ARTIFACT_PATHS = CONSTRUCTION_ARTIFACT_PATHS | TERMINAL_ARTIFACT_PATHS

Split = Literal["construction", "terminal"]
Population = Literal["mqar", "english_history", "english_no_history"]


class PositionKVPopulationUnavailable(RuntimeError):
    """The exact frozen population cannot be obtained from the named inputs."""


def _cid(value: bytes) -> str:
    return cid_bytes(value)


def _is_cid(value: object) -> bool:
    if not isinstance(value, str) or not value.startswith("blake3:"):
        return False
    digest = value.removeprefix("blake3:")
    return len(digest) == 64 and all(character in "0123456789abcdef" for character in digest)


def _with_self_cid(value: Mapping[str, Any], field: str) -> dict[str, Any]:
    if field in value:
        raise ValueError(f"self-CID field already exists: {field}")
    result = dict(value)
    result[field] = _cid(canonical_json_bytes(value))
    return result


def _verify_self_cid(value: Mapping[str, Any], field: str) -> None:
    unsigned = dict(value)
    observed = unsigned.pop(field, None)
    if observed != _cid(canonical_json_bytes(unsigned)):
        raise ValueError(f"{field} does not reproduce")


def _read_canonical_json(path: Path) -> dict[str, Any]:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"expected a regular non-symlink JSON file: {path}")
    payload = path.read_bytes()
    try:
        value = json.loads(payload.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(f"cannot decode canonical JSON: {path}") from error
    if not isinstance(value, dict) or canonical_json_bytes(value) != payload:
        raise ValueError(f"JSON file is not canonical: {path}")
    return value


def _write_exclusive(path: Path, payload: bytes, *, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=True) as target:
            descriptor = -1
            target.write(payload)
            target.flush()
            os.fsync(target.fileno())
    finally:
        if descriptor >= 0:
            os.close(descriptor)


def _write_exclusive_json(path: Path, value: Any) -> None:
    _write_exclusive(path, canonical_json_bytes(value))


def _sequence_cid(input_ids: Sequence[int], label_ids: Sequence[int]) -> str:
    if len(input_ids) != len(label_ids):
        raise ValueError("input and label lengths differ")
    payload = bytearray(struct.pack(">I", len(input_ids)))
    for token in input_ids:
        payload.extend(struct.pack(">H", token))
    for label in label_ids:
        payload.extend(struct.pack(">i", label))
    return _cid(bytes(payload))


def _unique_input_serialization_identity(
    examples: Sequence[CausalBindingExample],
) -> dict[str, Any]:
    unique = tuple(sorted({example.input_ids for example in examples}))
    return {
        "rows": len(examples),
        "unique_inputs": len(unique),
        "repeated_rows": len(examples) - len(unique),
        "unique_inputs_cid": _cid(
            canonical_json_bytes([list(input_ids) for input_ids in unique])
        ),
    }


def _assignment_cid(keys: Sequence[int], values: Sequence[int]) -> str:
    return _cid(
        canonical_json_bytes(
            [[int(key), int(value)] for key, value in zip(keys, values, strict=True)]
        )
    )


@dataclass(frozen=True, slots=True)
class CausalBindingExample:
    """One causal row whose sparse labels align with input-token logits."""

    population: Population
    split: Split
    example_index: int
    world_index: int
    family_index: int
    input_ids: tuple[int, ...]
    label_ids: tuple[int, ...]
    query_positions: tuple[int, ...]
    query_keys: tuple[int, ...]
    answers: tuple[int, ...]
    binding_keys: tuple[int, ...]
    binding_values: tuple[int, ...]
    binding_names: tuple[tuple[str, str], ...]
    assignment_cid: str
    world_cid: str
    sequence_cid: str
    text: str | None = None

    def __post_init__(self) -> None:
        if self.population not in ("mqar", "english_history", "english_no_history"):
            raise ValueError("unknown causal-binding population")
        if self.split not in ("construction", "terminal"):
            raise ValueError("unknown causal-binding split")
        for value, label in (
            (self.example_index, "example index"),
            (self.world_index, "world index"),
            (self.family_index, "family index"),
        ):
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                raise ValueError(f"{label} must be a nonnegative integer")
        if not 1 <= len(self.input_ids) <= CONTEXT or len(self.label_ids) != len(
            self.input_ids
        ):
            raise ValueError("causal example length is outside the frozen context")
        if any(
            isinstance(token, bool)
            or not isinstance(token, int)
            or not 0 <= token < VOCAB_SIZE
            for token in self.input_ids
        ):
            raise ValueError("causal example contains an invalid input token")
        if any(
            isinstance(label, bool)
            or not isinstance(label, int)
            or (label != IGNORE_INDEX and not 0 <= label < VOCAB_SIZE)
            for label in self.label_ids
        ):
            raise ValueError("causal example contains an invalid label")
        widths = (len(self.query_positions), len(self.query_keys), len(self.answers))
        if len(set(widths)) != 1 or widths[0] < 1:
            raise ValueError("query positions, keys, and answers differ")
        if tuple(sorted(self.query_positions)) != self.query_positions or len(
            set(self.query_positions)
        ) != len(self.query_positions):
            raise ValueError("query positions must be strictly increasing")
        active = tuple(index for index, label in enumerate(self.label_ids) if label != IGNORE_INDEX)
        if active != self.query_positions:
            raise ValueError("causal label mask differs from query positions")
        if tuple(self.label_ids[index] for index in active) != self.answers:
            raise ValueError("causal labels differ from answer tokens")
        if (
            self.population == "mqar"
            and tuple(self.input_ids[index] for index in active) != self.query_keys
        ):
            raise ValueError("query positions do not contain their query keys")
        if len(self.binding_keys) != len(self.binding_values) or len(
            self.binding_names
        ) not in (0, len(self.binding_keys)):
            raise ValueError("binding fields differ in length")
        if self.assignment_cid != _assignment_cid(self.binding_keys, self.binding_values):
            raise ValueError("assignment CID does not reproduce")
        if not _is_cid(self.world_cid):
            raise ValueError("world CID is malformed")
        if self.sequence_cid != _sequence_cid(self.input_ids, self.label_ids):
            raise ValueError("sequence CID does not reproduce")
        if self.population == "mqar" and (
            len(self.input_ids) != CONTEXT
            or len(self.query_positions) != MQAR_QUERIES
            or len(self.binding_keys) != MQAR_RECORDS
            or self.binding_names
            or self.text is not None
        ):
            raise ValueError("MQAR row differs from its frozen shape")
        if self.population == "english_history" and (
            len(self.query_positions) != 1
            or len(self.binding_keys) != 4
            or len(self.binding_names) != 4
            or self.text is None
        ):
            raise ValueError("English history row differs from its frozen shape")
        if self.population == "english_no_history" and (
            len(self.query_positions) != 1
            or self.binding_keys
            or self.binding_values
            or self.binding_names
            or self.answers != (UNKNOWN_TOKEN_ID,)
            or self.text is None
        ):
            raise ValueError("English no-history row differs from its frozen shape")

    def record(self) -> dict[str, Any]:
        return {
            "population": self.population,
            "split": self.split,
            "example_index": self.example_index,
            "world_index": self.world_index,
            "family_index": self.family_index,
            "input_ids": list(self.input_ids),
            "label_ids": list(self.label_ids),
            "query_positions": list(self.query_positions),
            "query_keys": list(self.query_keys),
            "answers": list(self.answers),
            "binding_keys": list(self.binding_keys),
            "binding_values": list(self.binding_values),
            "binding_names": [list(value) for value in self.binding_names],
            "assignment_cid": self.assignment_cid,
            "world_cid": self.world_cid,
            "sequence_cid": self.sequence_cid,
            "text": self.text,
        }

    @classmethod
    def from_record(cls, value: Mapping[str, Any]) -> CausalBindingExample:
        expected = {
            "population",
            "split",
            "example_index",
            "world_index",
            "family_index",
            "input_ids",
            "label_ids",
            "query_positions",
            "query_keys",
            "answers",
            "binding_keys",
            "binding_values",
            "binding_names",
            "assignment_cid",
            "world_cid",
            "sequence_cid",
            "text",
        }
        if set(value) != expected:
            raise ValueError("causal example fields differ")
        names = value.get("binding_names")
        if not isinstance(names, list) or any(
            not isinstance(pair, list)
            or len(pair) != 2
            or not all(isinstance(item, str) for item in pair)
            for pair in names
        ):
            raise ValueError("causal example binding names are malformed")
        candidate = cls(
            population=value["population"],  # type: ignore[arg-type]
            split=value["split"],  # type: ignore[arg-type]
            example_index=value["example_index"],  # type: ignore[arg-type]
            world_index=value["world_index"],  # type: ignore[arg-type]
            family_index=value["family_index"],  # type: ignore[arg-type]
            input_ids=tuple(value["input_ids"]),  # type: ignore[arg-type]
            label_ids=tuple(value["label_ids"]),  # type: ignore[arg-type]
            query_positions=tuple(value["query_positions"]),  # type: ignore[arg-type]
            query_keys=tuple(value["query_keys"]),  # type: ignore[arg-type]
            answers=tuple(value["answers"]),  # type: ignore[arg-type]
            binding_keys=tuple(value["binding_keys"]),  # type: ignore[arg-type]
            binding_values=tuple(value["binding_values"]),  # type: ignore[arg-type]
            binding_names=tuple((pair[0], pair[1]) for pair in names),
            assignment_cid=value["assignment_cid"],  # type: ignore[arg-type]
            world_cid=value["world_cid"],  # type: ignore[arg-type]
            sequence_cid=value["sequence_cid"],  # type: ignore[arg-type]
            text=value["text"],  # type: ignore[arg-type]
        )
        if candidate.record() != dict(value):
            raise ValueError("causal example does not reproduce canonically")
        return candidate


@dataclass(frozen=True, slots=True)
class SerializationOracleResult:
    mqar_correct: int
    mqar_total: int
    english_correct: int
    english_total: int
    ambiguous_bindings: int
    missing_bindings: int
    overlength_sequences: int

    @property
    def passed(self) -> bool:
        return (
            self.mqar_correct == self.mqar_total == MQAR_TERMINAL_DECISIONS
            and self.english_correct == self.english_total == ENGLISH_TERMINAL_HISTORY
            and self.ambiguous_bindings == 0
            and self.missing_bindings == 0
            and self.overlength_sequences == 0
        )

    def record(self) -> dict[str, Any]:
        return {
            "mqar_correct": self.mqar_correct,
            "mqar_total": self.mqar_total,
            "english_correct": self.english_correct,
            "english_total": self.english_total,
            "ambiguous_bindings": self.ambiguous_bindings,
            "missing_bindings": self.missing_bindings,
            "overlength_sequences": self.overlength_sequences,
            "passed": self.passed,
        }


@dataclass(frozen=True, slots=True)
class CausalBindingBatch:
    input_ids: Tensor
    label_ids: Tensor
    lengths: Tensor


@dataclass(frozen=True, slots=True)
class PositionKVConstructionData:
    root: Path
    manifest: dict[str, Any]
    commitment: dict[str, Any]
    tokenizer_path: Path
    natural_windows: LanguagePathWindowStore
    natural_selection: dict[str, Any]
    mqar: tuple[CausalBindingExample, ...]
    english_history: tuple[CausalBindingExample, ...]
    english_no_history: tuple[CausalBindingExample, ...]


@dataclass(frozen=True, slots=True)
class PositionKVTerminalData:
    root: Path
    manifest: dict[str, Any]
    reveal: dict[str, Any]
    final_artifact_cid: str
    natural_windows: LanguagePathWindowStore
    natural_selection: dict[str, Any]
    mqar: tuple[CausalBindingExample, ...]
    mqar_binding_permuted: tuple[CausalBindingExample, ...]
    english_history: tuple[CausalBindingExample, ...]
    english_binding_permuted: tuple[CausalBindingExample, ...]
    english_no_history: tuple[CausalBindingExample, ...]


@dataclass(frozen=True, slots=True)
class PositionKVDataPreparation:
    root: Path
    manifest: dict[str, Any]
    commitment: dict[str, Any]
    construction: PositionKVConstructionData


def batch_causal_examples(
    examples: Sequence[CausalBindingExample],
    *,
    device: torch.device | str | None = None,
) -> CausalBindingBatch:
    """Right-pad causal rows; labels remain sparse and aligned to logits."""

    if not examples:
        raise ValueError("causal binding batch cannot be empty")
    maximum = max(len(example.input_ids) for example in examples)
    inputs = torch.full((len(examples), maximum), BOS_TOKEN_ID, dtype=torch.long)
    labels = torch.full((len(examples), maximum), IGNORE_INDEX, dtype=torch.long)
    lengths = torch.empty(len(examples), dtype=torch.long)
    for row, example in enumerate(examples):
        width = len(example.input_ids)
        inputs[row, :width] = torch.tensor(example.input_ids, dtype=torch.long)
        labels[row, :width] = torch.tensor(example.label_ids, dtype=torch.long)
        lengths[row] = width
    if device is not None:
        inputs = inputs.to(device=device)
        labels = labels.to(device=device)
        lengths = lengths.to(device=device)
    return CausalBindingBatch(inputs, labels, lengths)


def deterministic_natural_replay_ordinals(
    *,
    window_count: int = NATURAL_SOURCE_WINDOWS,
    take: int = NATURAL_CONSTRUCTION_WINDOWS,
) -> tuple[int, ...]:
    """Frozen ascending BLAKE3(seed || ordinal), without-replacement order."""

    if any(isinstance(value, bool) or not isinstance(value, int) for value in (window_count, take)):
        raise TypeError("natural replay coordinates must be integers")
    if not 0 < take <= window_count <= 2**64:
        raise ValueError("natural replay coordinates are outside the frozen domain")
    return tuple(
        sorted(
            range(window_count),
            key=lambda ordinal: (
                blake3(struct.pack(">QQ", SEED, ordinal)).digest(),
                ordinal,
            ),
        )[:take]
    )


def _hash_words(label: str, *values: int) -> bytes:
    payload = bytearray(label.encode("ascii"))
    for value in values:
        payload.extend(struct.pack(">Q", value))
    return blake3(bytes(payload)).digest()


def mqar_pair_partition(key: int, value: int) -> int:
    if not MQAR_KEY_MIN <= key <= MQAR_KEY_MAX or not MQAR_VALUE_MIN <= value <= MQAR_VALUE_MAX:
        raise ValueError("MQAR pair is outside its frozen token ranges")
    digest = blake3(b"1043/mqar/" + struct.pack(">HH", key, value)).digest()
    return int.from_bytes(digest, "big") % 5


def english_pair_partition(key: str, value: str) -> int:
    if key not in KEY_LEXICON or value not in VALUE_LEXICON:
        raise ValueError("English pair is outside the frozen lexicons")
    digest = blake3(b"1043/english/" + key.encode("ascii") + b"\0" + value.encode("ascii")).digest()
    return int.from_bytes(digest, "big") % 5


def _choose_weighted_position(
    candidates: Sequence[int],
    *,
    record_position: int,
    sequence_index: int,
    query_index: int,
    terminal: bool,
) -> int:
    weights = [(position - record_position) ** -0.1 for position in candidates]
    total = math.fsum(weights)
    digest = _hash_words(
        "1043/mqar/query-position/terminal" if terminal else "1043/mqar/query-position/construction",
        sequence_index,
        query_index,
    )
    draw = (int.from_bytes(digest[:8], "big") + 0.5) / 2**64 * total
    cumulative = 0.0
    for position, weight in zip(candidates, weights, strict=True):
        cumulative += weight
        if draw <= cumulative:
            return position
    return candidates[-1]


def _next_mqar_pair(
    *,
    sequence_index: int,
    record_index: int,
    terminal: bool,
    used: set[tuple[int, int]],
    sequence_keys: set[int],
    sequence_values: set[int],
) -> tuple[int, int]:
    desired_terminal = 0 if terminal else None
    for attempt in range(1_000_000):
        digest = _hash_words(
            "1043/mqar/pair/terminal" if terminal else "1043/mqar/pair/construction",
            sequence_index,
            record_index,
            attempt,
        )
        key = MQAR_KEY_MIN + int.from_bytes(digest[:4], "big") % (
            MQAR_KEY_MAX - MQAR_KEY_MIN + 1
        )
        value = MQAR_VALUE_MIN + int.from_bytes(digest[4:8], "big") % (
            MQAR_VALUE_MAX - MQAR_VALUE_MIN + 1
        )
        pair = (key, value)
        partition = mqar_pair_partition(*pair)
        if (
            pair not in used
            and key not in sequence_keys
            and value not in sequence_values
            and ((partition == desired_terminal) if terminal else (partition != 0))
        ):
            used.add(pair)
            sequence_keys.add(key)
            sequence_values.add(value)
            return pair
    raise PositionKVPopulationUnavailable("MQAR pair partition exhausted")


def _generate_mqar_examples(*, count: int, terminal: bool) -> tuple[CausalBindingExample, ...]:
    split: Split = "terminal" if terminal else "construction"
    used_pairs: set[tuple[int, int]] = set()
    used_assignments: set[str] = set()
    examples: list[CausalBindingExample] = []
    record_positions = tuple(range(0, MQAR_RECORDS * 4, 4))
    for sequence_index in range(count):
        sequence_keys: set[int] = set()
        sequence_values: set[int] = set()
        pairs = tuple(
            _next_mqar_pair(
                sequence_index=sequence_index,
                record_index=record_index,
                terminal=terminal,
                used=used_pairs,
                sequence_keys=sequence_keys,
                sequence_values=sequence_values,
            )
            for record_index in range(MQAR_RECORDS)
        )
        keys = tuple(pair[0] for pair in pairs)
        values = tuple(pair[1] for pair in pairs)
        assignment = _assignment_cid(keys, values)
        if assignment in used_assignments:
            raise PositionKVPopulationUnavailable("MQAR assignment map repeated")
        used_assignments.add(assignment)

        tokens = [0] * CONTEXT
        for record_position, (key, value) in zip(record_positions, pairs, strict=True):
            tokens[record_position] = key
            tokens[record_position + 1] = value
        available = list(range(MQAR_RECORDS * 4, CONTEXT))
        placements: list[tuple[int, int, int]] = []
        for record_index, record_position in enumerate(record_positions):
            position = _choose_weighted_position(
                available,
                record_position=record_position,
                sequence_index=sequence_index,
                query_index=record_index,
                terminal=terminal,
            )
            available.remove(position)
            placements.append((position, keys[record_index], values[record_index]))
        placements.sort()
        query_positions = tuple(value[0] for value in placements)
        query_keys = tuple(value[1] for value in placements)
        answers = tuple(value[2] for value in placements)
        for position, key, _answer in placements:
            tokens[position] = key
        for position, token in enumerate(tokens):
            if token == 0:
                digest = _hash_words(
                    "1043/mqar/filler/terminal" if terminal else "1043/mqar/filler/construction",
                    sequence_index,
                    position,
                )
                tokens[position] = MQAR_FILLER_MIN + int.from_bytes(digest[:4], "big") % (
                    MQAR_FILLER_MAX - MQAR_FILLER_MIN + 1
                )
        labels = [IGNORE_INDEX] * CONTEXT
        for position, answer in zip(query_positions, answers, strict=True):
            labels[position] = answer
        examples.append(
            CausalBindingExample(
                population="mqar",
                split=split,
                example_index=sequence_index,
                world_index=sequence_index,
                family_index=0,
                input_ids=tuple(tokens),
                label_ids=tuple(labels),
                query_positions=query_positions,
                query_keys=query_keys,
                answers=answers,
                binding_keys=keys,
                binding_values=values,
                binding_names=(),
                assignment_cid=assignment,
                world_cid=assignment,
                sequence_cid=_sequence_cid(tokens, labels),
            )
        )
    return tuple(examples)


def validate_tokenizer(tokenizer_path: Path) -> tuple[Tokenizer, dict[str, int]]:
    """Bind the exact tokenizer and prove every frozen answer is one token."""

    if tokenizer_path.is_symlink() or not tokenizer_path.is_file():
        raise ValueError("#1043 tokenizer must be a regular non-symlink file")
    if cid_file(tokenizer_path) != EXPECTED_TOKENIZER_CID:
        raise ValueError("#1043 tokenizer CID differs from the frozen identity")
    tokenizer = Tokenizer.from_file(str(tokenizer_path.resolve()))
    token_ids: dict[str, int] = {}
    for lexeme in (*KEY_LEXICON, *VALUE_LEXICON, "unknown"):
        encoded = tokenizer.encode(f" {lexeme}", add_special_tokens=False).ids
        if len(encoded) != 1 or not 0 <= encoded[0] < VOCAB_SIZE:
            raise ValueError(f"leading-space lexeme is not one in-vocabulary token: {lexeme}")
        token_ids[lexeme] = encoded[0]
    if token_ids["unknown"] != UNKNOWN_TOKEN_ID:
        raise ValueError("unknown-token identity differs from the freeze")
    return tokenizer, token_ids


def _eligible_english_pairs(*, terminal: bool) -> tuple[tuple[str, str], ...]:
    pairs = tuple(
        (key, value)
        for key in KEY_LEXICON
        for value in VALUE_LEXICON
        if (english_pair_partition(key, value) == 0) == terminal
    )
    return tuple(
        sorted(
            pairs,
            key=lambda pair: (
                blake3(
                    ("1043/english/order/terminal/" if terminal else "1043/english/order/construction/").encode("ascii")
                    + pair[0].encode("ascii")
                    + b"\0"
                    + pair[1].encode("ascii")
                ).digest(),
                pair,
            ),
        )
    )


def _english_world(
    *,
    world_index: int,
    terminal: bool,
    used_worlds: set[str],
) -> tuple[tuple[tuple[str, str], ...], str]:
    eligible = _eligible_english_pairs(terminal=terminal)
    for nonce in range(1_000_000):
        ordered = sorted(
            eligible,
            key=lambda pair: (
                blake3(
                    ("1043/english/world/terminal" if terminal else "1043/english/world/construction").encode("ascii")
                    + struct.pack(">QQ", world_index, nonce)
                    + pair[0].encode("ascii")
                    + b"\0"
                    + pair[1].encode("ascii")
                ).digest(),
                pair,
            ),
        )
        chosen: list[tuple[str, str]] = []
        keys: set[str] = set()
        values: set[str] = set()
        for pair in ordered:
            if pair[0] not in keys and pair[1] not in values:
                chosen.append(pair)
                keys.add(pair[0])
                values.add(pair[1])
                if len(chosen) == 4:
                    break
        if len(chosen) != 4:
            raise PositionKVPopulationUnavailable("English partition cannot form a four-pair world")
        world_cid = _cid(canonical_json_bytes([list(pair) for pair in chosen]))
        if world_cid not in used_worlds:
            used_worlds.add(world_cid)
            return tuple(chosen), world_cid
    raise PositionKVPopulationUnavailable("English world-assignment space exhausted")


def _construction_text(
    family: int,
    bindings: Sequence[tuple[str, str]],
    query_key: str,
    *,
    history: bool,
) -> str:
    if family == 0:
        facts = " ".join(f"The {key} is in the {value}." for key, value in bindings)
        return (
            f"Context: {facts} Question: Where is the {query_key}? Answer:"
            if history
            else f"Question: Where is the {query_key}? Answer:"
        )
    if family == 1:
        facts = " ".join(f"We put the {key} in the {value}." for key, value in bindings)
        return (
            f"Read these notes. {facts} Give the location of the {query_key}:"
            if history
            else f"Give the location of the {query_key}:"
        )
    if family == 2:
        facts = " ".join(f"{key} belongs in {value}." for key, value in bindings)
        return (
            f"Placement record: {facts} Look up {query_key}. Location:"
            if history
            else f"Look up {query_key}. Location:"
        )
    raise ValueError("unknown construction English family")


def _terminal_text(
    family: int,
    bindings: Sequence[tuple[str, str]],
    query_key: str,
    *,
    history: bool,
) -> str:
    if family == 0:
        facts = " ".join(f"Inside the {value} was the {key}." for key, value in bindings)
        return (
            f"Four objects were stored. {facts} Which place holds the {query_key}? Answer:"
            if history
            else f"Which place holds the {query_key}? Answer:"
        )
    if family == 1:
        facts = " ".join(
            f"The {key} can be found in {value}." for key, value in bindings
        )
        return (
            f"Today's list says that {facts} The {query_key} is where?"
            if history
            else f"The {query_key} is where?"
        )
    raise ValueError("unknown terminal English family")


def _english_example(
    *,
    tokenizer: Tokenizer,
    token_ids: Mapping[str, int],
    split: Split,
    example_index: int,
    world_index: int,
    family: int,
    bindings: Sequence[tuple[str, str]],
    world_cid: str,
    query_key: str,
    history: bool,
) -> CausalBindingExample:
    terminal = split == "terminal"
    text = (
        _terminal_text(family, bindings, query_key, history=history)
        if terminal
        else _construction_text(family, bindings, query_key, history=history)
    )
    inputs = tuple(tokenizer.encode(text, add_special_tokens=True).ids)
    if not 1 <= len(inputs) <= CONTEXT or any(not 0 <= token < VOCAB_SIZE for token in inputs):
        raise PositionKVPopulationUnavailable("English serialization exceeds the frozen model contract")
    answer = token_ids[dict(bindings)[query_key]] if history else UNKNOWN_TOKEN_ID
    labels = (IGNORE_INDEX,) * (len(inputs) - 1) + (answer,)
    binding_keys = tuple(token_ids[key] for key, _value in bindings) if history else ()
    binding_values = tuple(token_ids[value] for _key, value in bindings) if history else ()
    assignment = _assignment_cid(binding_keys, binding_values) if history else _assignment_cid((), ())
    if history and assignment != _assignment_cid(
        tuple(token_ids[key] for key, _value in bindings),
        tuple(token_ids[value] for _key, value in bindings),
    ):
        raise RuntimeError("English assignment identity drifted")
    # A no-history row remains linked to its matched world in ``world_index``;
    # its empty assignment is intentional because no binding enters the input.
    return CausalBindingExample(
        population="english_history" if history else "english_no_history",
        split=split,
        example_index=example_index,
        world_index=world_index,
        family_index=family,
        input_ids=inputs,
        label_ids=labels,
        query_positions=(len(inputs) - 1,),
        query_keys=(token_ids[query_key],),
        answers=(answer,),
        binding_keys=binding_keys,
        binding_values=binding_values,
        binding_names=tuple(bindings) if history else (),
        assignment_cid=assignment,
        world_cid=world_cid,
        sequence_cid=_sequence_cid(inputs, labels),
        text=text,
    )


def _generate_english_examples(
    *,
    tokenizer: Tokenizer,
    token_ids: Mapping[str, int],
    terminal: bool,
    history_count: int,
    no_history_count: int,
) -> tuple[tuple[CausalBindingExample, ...], tuple[CausalBindingExample, ...]]:
    split: Split = "terminal" if terminal else "construction"
    used_worlds: set[str] = set()
    history: list[CausalBindingExample] = []
    no_history: list[CausalBindingExample] = []
    world_count = history_count // (ENGLISH_TERMINAL_QUERIES_PER_WORLD if terminal else 1)
    if terminal and history_count != world_count * ENGLISH_TERMINAL_QUERIES_PER_WORLD:
        raise ValueError("terminal English history count is not world-aligned")
    for world_index in range(world_count):
        bindings, world_cid = _english_world(
            world_index=world_index,
            terminal=terminal,
            used_worlds=used_worlds,
        )
        family = world_index % (2 if terminal else 3)
        digest = _hash_words(
            "1043/english/query/terminal" if terminal else "1043/english/query/construction",
            world_index,
        )
        first_query = int.from_bytes(digest[:4], "big") % 4
        query_indices = (
            (first_query, (first_query + 1 + int.from_bytes(digest[4:8], "big") % 3) % 4)
            if terminal
            else (first_query,)
        )
        if terminal and query_indices[0] == query_indices[1]:
            raise RuntimeError("terminal English queries must be distinct")
        for query_index in query_indices:
            query_key = bindings[query_index][0]
            example_index = len(history)
            example = _english_example(
                tokenizer=tokenizer,
                token_ids=token_ids,
                split=split,
                example_index=example_index,
                world_index=world_index,
                family=family,
                bindings=bindings,
                world_cid=world_cid,
                query_key=query_key,
                history=True,
            )
            # Bind the semantic world identity independently from the token-ID
            # assignment stored on the row.
            if world_cid != _cid(canonical_json_bytes([list(pair) for pair in bindings])):
                raise RuntimeError("English semantic world CID drifted")
            history.append(example)
            if terminal:
                no_history.append(
                    _english_example(
                        tokenizer=tokenizer,
                        token_ids=token_ids,
                        split=split,
                        example_index=len(no_history),
                        world_index=world_index,
                        family=family,
                        bindings=bindings,
                        world_cid=world_cid,
                        query_key=query_key,
                        history=False,
                    )
                )
        if not terminal and world_index < no_history_count:
            query_key = bindings[first_query][0]
            no_history.append(
                _english_example(
                    tokenizer=tokenizer,
                    token_ids=token_ids,
                    split=split,
                    example_index=len(no_history),
                    world_index=world_index,
                    family=family,
                    bindings=bindings,
                    world_cid=world_cid,
                    query_key=query_key,
                    history=False,
                )
            )
    if len(history) != history_count or len(no_history) != no_history_count:
        raise RuntimeError("English population arithmetic drifted")
    return tuple(history), tuple(no_history)


def _active_labels(example: CausalBindingExample) -> tuple[tuple[int, int], ...]:
    """Return the labels physically serialized on the row, not its metadata."""

    return tuple(
        (position, label)
        for position, label in enumerate(example.label_ids)
        if label != IGNORE_INDEX
    )


def _english_serialization_patterns() -> tuple[
    tuple[Split, int, re.Pattern[str], tuple[tuple[str, str], ...]], ...
]:
    keys = "(?:" + "|".join(map(re.escape, KEY_LEXICON)) + ")"
    values = "(?:" + "|".join(map(re.escape, VALUE_LEXICON)) + ")"

    def facts(template: str) -> tuple[str, tuple[tuple[str, str], ...]]:
        pieces: list[str] = []
        groups: list[tuple[str, str]] = []
        for index in range(4):
            key_group = f"key_{index}"
            value_group = f"value_{index}"
            key_capture = f"(?P<{key_group}>{keys})"
            value_capture = f"(?P<{value_group}>{values})"
            pieces.append(
                template.format(
                    key=key_capture,
                    value=value_capture,
                )
            )
            groups.append((key_group, value_group))
        return " ".join(pieces), tuple(groups)

    construction_zero, c0_groups = facts(r"The {key} is in the {value}\.")
    construction_one, c1_groups = facts(r"We put the {key} in the {value}\.")
    construction_two, c2_groups = facts(r"{key} belongs in {value}\.")
    terminal_zero, t0_groups = facts(r"Inside the {value} was the {key}\.")
    terminal_one, t1_groups = facts(r"The {key} can be found in {value}\.")
    return (
        (
            "construction",
            0,
            re.compile(
                rf"^Context: {construction_zero} Question: Where is the (?P<query>{keys})\? Answer:$"
            ),
            c0_groups,
        ),
        (
            "construction",
            1,
            re.compile(
                rf"^Read these notes\. {construction_one} Give the location of the (?P<query>{keys}):$"
            ),
            c1_groups,
        ),
        (
            "construction",
            2,
            re.compile(
                rf"^Placement record: {construction_two} Look up (?P<query>{keys})\. Location:$"
            ),
            c2_groups,
        ),
        (
            "terminal",
            0,
            re.compile(
                rf"^Four objects were stored\. {terminal_zero} Which place holds the (?P<query>{keys})\? Answer:$"
            ),
            t0_groups,
        ),
        (
            "terminal",
            1,
            re.compile(
                rf"^Today's list says that {terminal_one} The (?P<query>{keys}) is where\?$"
            ),
            t1_groups,
        ),
    )


_ENGLISH_SERIALIZATION_PATTERNS = _english_serialization_patterns()


def _decode_english_bindings(
    example: CausalBindingExample,
    *,
    tokenizer: Tokenizer,
) -> tuple[tuple[tuple[str, str], ...], str] | None:
    """Parse facts and query from the actual input-token serialization."""

    try:
        text = tokenizer.decode(list(example.input_ids), skip_special_tokens=False)
    except (TypeError, ValueError, RuntimeError):
        return None
    for split, family, pattern, groups in _ENGLISH_SERIALIZATION_PATTERNS:
        match = pattern.fullmatch(text)
        if match is None:
            continue
        if split != example.split or family != example.family_index:
            return None
        bindings = tuple(
            (match.group(key_group), match.group(value_group))
            for key_group, value_group in groups
        )
        return bindings, match.group("query")
    return None


def serialization_oracle(
    mqar: Sequence[CausalBindingExample],
    english_history: Sequence[CausalBindingExample],
    *,
    tokenizer: Tokenizer | None = None,
    token_ids: Mapping[str, int] | None = None,
) -> SerializationOracleResult:
    """Recover every answer from serialized inputs, never binding metadata.

    MQAR records are decoded from their frozen key/value slots and queries from
    the active label positions.  English facts and the requested key are parsed
    from text decoded from ``input_ids``.  The row's ``binding_*``, ``text``,
    ``query_*``, and ``answers`` fields therefore cannot make a corrupted
    serialization pass this oracle.
    """

    if english_history and (tokenizer is None or token_ids is None):
        raise ValueError("English serialization oracle requires the frozen tokenizer map")
    if token_ids is not None and any(
        lexeme not in token_ids for lexeme in (*KEY_LEXICON, *VALUE_LEXICON)
    ):
        raise ValueError("English serialization oracle token map is incomplete")

    mqar_correct = 0
    english_correct = 0
    ambiguous = 0
    missing = 0
    overlength = 0
    mqar_total = 0
    english_total = 0
    for example in (*mqar, *english_history):
        if len(example.input_ids) > CONTEXT:
            overlength += 1

    record_positions = tuple(range(0, MQAR_RECORDS * 4, 4))
    for example in mqar:
        active = _active_labels(example)
        mqar_total += len(active)
        records: list[tuple[int, int, int]] = []
        for record_position in record_positions:
            if record_position + 1 >= len(example.input_ids):
                continue
            key = example.input_ids[record_position]
            value = example.input_ids[record_position + 1]
            if MQAR_KEY_MIN <= key <= MQAR_KEY_MAX and MQAR_VALUE_MIN <= value <= MQAR_VALUE_MAX:
                records.append((record_position, key, value))
        for position, expected in active:
            if position >= len(example.input_ids):
                missing += 1
                continue
            query_key = example.input_ids[position]
            candidates = [
                value
                for record_position, key, value in records
                if key == query_key and record_position < position
            ]
            if not candidates:
                missing += 1
            elif len(candidates) != 1:
                ambiguous += 1
            elif candidates[0] == expected:
                mqar_correct += 1

    assert tokenizer is not None or not english_history
    assert token_ids is not None or not english_history
    for example in english_history:
        active = _active_labels(example)
        english_total += len(active)
        decoded = _decode_english_bindings(example, tokenizer=tokenizer)  # type: ignore[arg-type]
        if len(active) != 1 or active[0][0] != len(example.input_ids) - 1 or decoded is None:
            missing += max(1, len(active))
            continue
        bindings, query_name = decoded
        mapping: dict[str, list[str]] = {}
        for key_name, value_name in bindings:
            mapping.setdefault(key_name, []).append(value_name)
        candidates = mapping.get(query_name, [])
        if not candidates:
            missing += 1
        elif len(candidates) != 1:
            ambiguous += 1
        elif token_ids[candidates[0]] == active[0][1]:  # type: ignore[index]
            english_correct += 1

    return SerializationOracleResult(
        mqar_correct=mqar_correct,
        mqar_total=mqar_total,
        english_correct=english_correct,
        english_total=english_total,
        ambiguous_bindings=ambiguous,
        missing_bindings=missing,
        overlength_sequences=overlength,
    )


def binding_permuted_examples(
    examples: Sequence[CausalBindingExample],
    *,
    tokenizer_path: Path | None = None,
) -> tuple[CausalBindingExample, ...]:
    """Derange admitted values while retaining labels and serialization widths.

    This is an evaluation intervention, not another committed population.  It
    can only be reached after the caller has already obtained revealed rows.
    """

    if not examples:
        raise ValueError("binding-permuted control cannot be empty")
    population = examples[0].population
    if population not in ("mqar", "english_history") or any(
        example.population != population or example.split != "terminal"
        for example in examples
    ):
        raise ValueError("binding-permuted control requires one terminal history population")
    tokenizer: Tokenizer | None = None
    token_ids: dict[str, int] | None = None
    if population == "english_history":
        if tokenizer_path is None:
            raise ValueError("English binding permutation requires the frozen tokenizer")
        tokenizer, token_ids = validate_tokenizer(tokenizer_path)
    controls: list[CausalBindingExample] = []
    for example in examples:
        values = example.binding_values[1:] + example.binding_values[:1]
        if len(values) < 2 or any(
            native == permuted
            for native, permuted in zip(example.binding_values, values, strict=True)
        ):
            raise ValueError("binding-value permutation is not a derangement")
        if population == "mqar":
            inputs = list(example.input_ids)
            first_query = min(example.query_positions)
            for key, value in zip(example.binding_keys, values, strict=True):
                positions = [
                    index
                    for index, token in enumerate(inputs[:first_query])
                    if token == key
                ]
                if len(positions) != 1 or positions[0] + 1 >= first_query:
                    raise ValueError("MQAR binding serialization is ambiguous")
                inputs[positions[0] + 1] = value
            names: tuple[tuple[str, str], ...] = ()
            text = None
        else:
            assert tokenizer is not None and token_ids is not None
            reverse_keys = {token_ids[key]: key for key in KEY_LEXICON}
            reverse_values = {token_ids[value]: value for value in VALUE_LEXICON}
            try:
                query_key = reverse_keys[example.query_keys[0]]
                names = tuple(
                    (reverse_keys[key], reverse_values[value])
                    for key, value in zip(example.binding_keys, values, strict=True)
                )
            except KeyError as error:
                raise ValueError("English binding tokens are outside the frozen lexicons") from error
            text = _terminal_text(
                example.family_index,
                names,
                query_key,
                history=True,
            )
            inputs = tokenizer.encode(text, add_special_tokens=True).ids
            if len(inputs) != len(example.input_ids):
                raise ValueError("English binding permutation changed serialization length")
        labels = list(example.label_ids)
        controls.append(
            CausalBindingExample(
                population=example.population,
                split=example.split,
                example_index=example.example_index,
                world_index=example.world_index,
                family_index=example.family_index,
                input_ids=tuple(inputs),
                label_ids=tuple(labels),
                query_positions=example.query_positions,
                query_keys=example.query_keys,
                answers=example.answers,
                binding_keys=example.binding_keys,
                binding_values=values,
                binding_names=names,
                assignment_cid=_assignment_cid(example.binding_keys, values),
                world_cid=(
                    _cid(canonical_json_bytes([list(pair) for pair in names]))
                    if names
                    else _assignment_cid(example.binding_keys, values)
                ),
                sequence_cid=_sequence_cid(inputs, labels),
                text=text,
            )
        )
    return tuple(controls)


def _mqar_payload(examples: Sequence[CausalBindingExample], *, split: Split) -> dict[str, Any]:
    return {
        "schema": MQAR_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "split": split,
        "sequence_count": len(examples),
        "query_decisions": sum(len(example.answers) for example in examples),
        "pair_partition": {
            "encoding": 'BLAKE3("1043/mqar/" || key_u16_be || value_u16_be) mod 5',
            "terminal_remainder": 0,
        },
        "assignment_map_cid": _cid(
            canonical_json_bytes([example.assignment_cid for example in examples])
        ),
        "world_cids_cid": _cid(
            canonical_json_bytes([example.world_cid for example in examples])
        ),
        "sequence_cids_cid": _cid(
            canonical_json_bytes([example.sequence_cid for example in examples])
        ),
        "examples": [example.record() for example in examples],
    }


def _english_payload(
    history: Sequence[CausalBindingExample],
    no_history: Sequence[CausalBindingExample],
    *,
    split: Split,
) -> dict[str, Any]:
    return {
        "schema": ENGLISH_SCHEMA,
        "issue": ISSUE,
        "policy": POLICY,
        "split": split,
        "history_count": len(history),
        "no_history_count": len(no_history),
        "pair_partition": {
            "encoding": 'BLAKE3("1043/english/" || key_utf8 || NUL || value_utf8) mod 5',
            "terminal_remainder": 0,
        },
        "history_assignment_cids_cid": _cid(
            canonical_json_bytes([example.assignment_cid for example in history])
        ),
        "history_world_cids_cid": _cid(
            canonical_json_bytes([example.world_cid for example in history])
        ),
        "history_sequence_cids_cid": _cid(
            canonical_json_bytes([example.sequence_cid for example in history])
        ),
        "no_history_sequence_cids_cid": _cid(
            canonical_json_bytes([example.sequence_cid for example in no_history])
        ),
        "history": [example.record() for example in history],
        "no_history": [example.record() for example in no_history],
    }


def _parse_mqar_payload(value: Mapping[str, Any], *, split: Split, expected_count: int) -> tuple[CausalBindingExample, ...]:
    examples_value = value.get("examples")
    if (
        value.get("schema") != MQAR_SCHEMA
        or value.get("issue") != ISSUE
        or value.get("policy") != POLICY
        or value.get("split") != split
        or value.get("sequence_count") != expected_count
        or value.get("query_decisions") != expected_count * MQAR_QUERIES
        or not isinstance(examples_value, list)
    ):
        raise ValueError("MQAR payload contract differs")
    examples = tuple(
        CausalBindingExample.from_record(example)
        for example in examples_value
        if isinstance(example, Mapping)
    )
    if len(examples) != expected_count or _mqar_payload(examples, split=split) != dict(value):
        raise ValueError("MQAR payload does not reproduce")
    return examples


def _parse_english_payload(
    value: Mapping[str, Any],
    *,
    split: Split,
    expected_history: int,
    expected_no_history: int,
) -> tuple[tuple[CausalBindingExample, ...], tuple[CausalBindingExample, ...]]:
    history_value = value.get("history")
    no_history_value = value.get("no_history")
    if (
        value.get("schema") != ENGLISH_SCHEMA
        or value.get("issue") != ISSUE
        or value.get("policy") != POLICY
        or value.get("split") != split
        or value.get("history_count") != expected_history
        or value.get("no_history_count") != expected_no_history
        or not isinstance(history_value, list)
        or not isinstance(no_history_value, list)
    ):
        raise ValueError("English payload contract differs")
    history = tuple(
        CausalBindingExample.from_record(example)
        for example in history_value
        if isinstance(example, Mapping)
    )
    no_history = tuple(
        CausalBindingExample.from_record(example)
        for example in no_history_value
        if isinstance(example, Mapping)
    )
    if (
        len(history) != expected_history
        or len(no_history) != expected_no_history
        or _english_payload(history, no_history, split=split) != dict(value)
    ):
        raise ValueError("English payload does not reproduce")
    return history, no_history


def _materialize_natural_construction(source_path: Path) -> tuple[bytes, dict[str, Any]]:
    if source_path.is_symlink() or not source_path.is_file():
        raise ValueError("retained natural source must be a regular non-symlink file")
    if cid_file(source_path) != EXPECTED_TRAIN_SLICE_CID:
        raise ValueError("retained natural source CID differs from the freeze")
    source = LanguagePathWindowStore(source_path, window_count=NATURAL_SOURCE_WINDOWS)
    ordinals = deterministic_natural_replay_ordinals()
    selected = np.array(source.windows[list(ordinals)], dtype="<u2", copy=True)
    payload = selected.tobytes(order="C")
    selection = {
        "schema": NATURAL_SELECTION_SCHEMA,
        "split": "construction",
        "seed": SEED,
        "source_cid": EXPECTED_TRAIN_SLICE_CID,
        "source_windows": NATURAL_SOURCE_WINDOWS,
        "windows": NATURAL_CONSTRUCTION_WINDOWS,
        "decisions": NATURAL_CONSTRUCTION_DECISIONS,
        "order_encoding": "ascending BLAKE3(pack_u64_be(10043)||pack_u64_be(window_ordinal)), then ordinal",
        "ordinals": list(ordinals),
        "payload_cid": _cid(payload),
    }
    return payload, selection


def _validated_story_cids(values: Collection[str]) -> tuple[str, ...]:
    ordered = tuple(sorted(values))
    if len(set(ordered)) != len(ordered) or any(not _is_cid(value) for value in ordered):
        raise ValueError("story-CID exclusion union is malformed")
    return ordered


def _terminal_natural_coordinates(
    index_path: Path,
    *,
    excluded_story_cids: Collection[str],
    count: int = NATURAL_TERMINAL_WINDOWS,
) -> tuple[tuple[int, str, int], ...]:
    """Select story-contained windows strictly after V5's final source story."""

    if index_path.is_symlink() or not index_path.is_file() or cid_file(index_path) != FRESH_HELDOUT_TRAIN_INDEX_CID:
        raise PositionKVPopulationUnavailable("#1019 train index differs from the frozen identity")
    excluded = frozenset(_validated_story_cids(excluded_story_cids))
    coordinates: list[tuple[int, str, int]] = []
    with index_path.open("rb") as source:
        for capacity_ordinal, line in enumerate(source):
            if capacity_ordinal <= FRESH_HELDOUT_LAST_CAPACITY_STORY:
                continue
            try:
                record = json.loads(line.decode("utf-8"))
            except (UnicodeError, json.JSONDecodeError) as error:
                raise PositionKVPopulationUnavailable("#1019 train index is malformed") from error
            if not isinstance(record, dict) or canonical_json_bytes(record) != line:
                raise PositionKVPopulationUnavailable("#1019 train index record is not canonical")
            source_ordinal = record.get("source_story_ordinal")
            story_cid = record.get("story_cid")
            offset = record.get("story_token_offset")
            tokens = record.get("story_token_count")
            if (
                record.get("capacity_story_ordinal") != capacity_ordinal
                or isinstance(source_ordinal, bool)
                or not isinstance(source_ordinal, int)
                or source_ordinal <= FRESH_HELDOUT_LAST_SOURCE_STORY
                or not _is_cid(story_cid)
                or isinstance(offset, bool)
                or not isinstance(offset, int)
                or isinstance(tokens, bool)
                or not isinstance(tokens, int)
                or offset < 0
                or tokens < 1
            ):
                raise PositionKVPopulationUnavailable("post-V5 train-story record differs")
            if story_cid in excluded:
                continue
            for within_story in range(0, tokens - WINDOW_TOKENS + 1, WINDOW_TOKENS):
                coordinates.append((offset + within_story, story_cid, source_ordinal))
                if len(coordinates) == count:
                    return tuple(coordinates)
    raise PositionKVPopulationUnavailable("post-V5 source cannot supply 2,066 exact windows")


def _materialize_natural_terminal(
    source_path: Path,
    index_path: Path,
    *,
    excluded_story_cids: Collection[str],
) -> tuple[bytes, dict[str, Any]]:
    if source_path.is_symlink() or not source_path.is_file() or cid_file(source_path) != EXPECTED_SOURCE_TRAIN_STORE_CID:
        raise PositionKVPopulationUnavailable("#1019 train-token store differs from the frozen identity")
    coordinates = _terminal_natural_coordinates(
        index_path,
        excluded_story_cids=excluded_story_cids,
    )
    rows = np.memmap(source_path, mode="r", dtype="<u2")
    output = np.empty((NATURAL_TERMINAL_WINDOWS, WINDOW_TOKENS), dtype="<u2")
    for row, (offset, _story_cid, _source_ordinal) in enumerate(coordinates):
        end = offset + WINDOW_TOKENS
        if end > rows.shape[0]:
            raise PositionKVPopulationUnavailable("terminal natural window crosses the source store")
        output[row] = rows[offset:end]
    payload = output.tobytes(order="C")
    exclusions = _validated_story_cids(excluded_story_cids)
    selected_story_cids = tuple(sorted({record[1] for record in coordinates}))
    if set(selected_story_cids).intersection(exclusions):
        raise RuntimeError("terminal natural selection intersects its exclusion union")
    selection = {
        "schema": NATURAL_SELECTION_SCHEMA,
        "split": "terminal",
        "source_store_cid": EXPECTED_SOURCE_TRAIN_STORE_CID,
        "source_index_cid": FRESH_HELDOUT_TRAIN_INDEX_CID,
        "strictly_after_capacity_story": FRESH_HELDOUT_LAST_CAPACITY_STORY,
        "strictly_after_source_story": FRESH_HELDOUT_LAST_SOURCE_STORY,
        "windows": NATURAL_TERMINAL_WINDOWS,
        "decisions": NATURAL_TERMINAL_DECISIONS,
        "selection": "source order; nonoverlapping 121-token windows wholly within each eligible story",
        "excluded_story_cids_count": len(exclusions),
        "excluded_story_cids_cid": _cid(canonical_json_bytes(list(exclusions))),
        "selected_story_cids": list(selected_story_cids),
        "selected_story_cids_cid": _cid(canonical_json_bytes(list(selected_story_cids))),
        "coordinates_cid": _cid(
            canonical_json_bytes(
                [
                    {"token_offset": offset, "story_cid": story_cid, "source_story_ordinal": ordinal}
                    for offset, story_cid, ordinal in coordinates
                ]
            )
        ),
        "first_token_offset": coordinates[0][0],
        "last_token_offset": coordinates[-1][0],
        "payload_cid": _cid(payload),
    }
    return payload, selection


def _population_disjointness(
    construction_mqar: Sequence[CausalBindingExample],
    terminal_mqar: Sequence[CausalBindingExample],
    construction_english: Sequence[CausalBindingExample],
    terminal_english: Sequence[CausalBindingExample],
) -> dict[str, Any]:
    construction_mqar_pairs = {
        (key, value)
        for example in construction_mqar
        for key, value in zip(example.binding_keys, example.binding_values, strict=True)
    }
    terminal_mqar_pairs = {
        (key, value)
        for example in terminal_mqar
        for key, value in zip(example.binding_keys, example.binding_values, strict=True)
    }
    construction_english_pairs = {
        pair for example in construction_english for pair in example.binding_names
    }
    terminal_english_pairs = {
        pair for example in terminal_english for pair in example.binding_names
    }
    witnesses = {
        "mqar_pairs_zero_intersection": not construction_mqar_pairs.intersection(terminal_mqar_pairs),
        "mqar_assignment_cids_zero_intersection": not {
            example.assignment_cid for example in construction_mqar
        }.intersection(example.assignment_cid for example in terminal_mqar),
        "mqar_world_cids_zero_intersection": not {
            example.world_cid for example in construction_mqar
        }.intersection(example.world_cid for example in terminal_mqar),
        "mqar_sequence_cids_zero_intersection": not {
            example.sequence_cid for example in construction_mqar
        }.intersection(example.sequence_cid for example in terminal_mqar),
        "english_pairs_zero_intersection": not construction_english_pairs.intersection(terminal_english_pairs),
        "english_assignment_cids_zero_intersection": not {
            example.assignment_cid for example in construction_english
        }.intersection(example.assignment_cid for example in terminal_english),
        "english_world_cids_zero_intersection": not {
            example.world_cid for example in construction_english
        }.intersection(example.world_cid for example in terminal_english),
        "english_sequence_cids_zero_intersection": not {
            example.sequence_cid for example in construction_english
        }.intersection(example.sequence_cid for example in terminal_english),
    }
    if not all(witnesses.values()):
        raise PositionKVPopulationUnavailable("construction and terminal populations overlap")
    return {
        **witnesses,
        "mqar_construction_pairs": len(construction_mqar_pairs),
        "mqar_terminal_pairs": len(terminal_mqar_pairs),
        "english_construction_pairs": len(construction_english_pairs),
        "english_terminal_pairs": len(terminal_english_pairs),
    }


def _validate_frozen_arithmetic() -> None:
    if (
        NATURAL_CONSTRUCTION_DECISIONS != 2_620_800
        or NATURAL_TERMINAL_DECISIONS != 247_920
        or MQAR_CONSTRUCTION_DECISIONS != 87_360
        or MQAR_TERMINAL_DECISIONS != 8_192
        or ENGLISH_CONSTRUCTION_HISTORY != 3 * ENGLISH_CONSTRUCTION_NO_HISTORY
        or ENGLISH_TERMINAL_HISTORY != 512
        or ENGLISH_TERMINAL_NO_HISTORY != 512
        or VOCAB_SIZE != MQAR_VALUE_MAX + 1
    ):
        raise RuntimeError("#1043 frozen population arithmetic drifted")


def _manifest_artifact_map(manifest: Mapping[str, Any]) -> dict[str, dict[str, Any]]:
    records = manifest.get("artifacts")
    if not isinstance(records, list):
        raise ValueError("#1043 data manifest has no artifact records")
    result: dict[str, dict[str, Any]] = {}
    for record in records:
        if not isinstance(record, dict) or set(record) != {"bytes", "cid", "path"}:
            raise ValueError("#1043 artifact record is malformed")
        path = record.get("path")
        if not isinstance(path, str) or path in result:
            raise ValueError("#1043 artifact paths are malformed")
        result[path] = record
    if set(result) != ALL_ARTIFACT_PATHS:
        raise ValueError("#1043 artifact set differs")
    return result


def _validate_public_envelopes(root: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    manifest = verify_manifest_envelope(root / MANIFEST_RELATIVE_PATH)
    if (
        manifest.get("schema") != MANIFEST_SCHEMA
        or manifest.get("issue") != ISSUE
        or manifest.get("policy") != POLICY
        or manifest.get("seed") != SEED
        or manifest.get("tokenizer_cid") != EXPECTED_TOKENIZER_CID
    ):
        raise ValueError("#1043 public data manifest differs")
    _manifest_artifact_map(manifest)
    commitment = _read_canonical_json(root / COMMITMENT_RELATIVE_PATH)
    _verify_self_cid(commitment, "commitment_cid")
    if (
        commitment.get("schema") != COMMITMENT_SCHEMA
        or commitment.get("issue") != ISSUE
        or commitment.get("policy") != POLICY
        or commitment.get("sealed_mode") != "000"
        or commitment.get("terminal_artifacts")
        != [
            _manifest_artifact_map(manifest)[path]
            for path in sorted(TERMINAL_ARTIFACT_PATHS)
        ]
        or manifest.get("commitment_cid") != commitment.get("commitment_cid")
    ):
        raise ValueError("#1043 terminal commitment differs")
    return manifest, commitment


def _load_examples_payload(path: Path, *, population: str, split: Split) -> Any:
    value = _read_canonical_json(path)
    if population == "mqar":
        return _parse_mqar_payload(
            value,
            split=split,
            expected_count=(MQAR_TERMINAL_SEQUENCES if split == "terminal" else MQAR_CONSTRUCTION_SEQUENCES),
        )
    return _parse_english_payload(
        value,
        split=split,
        expected_history=(ENGLISH_TERMINAL_HISTORY if split == "terminal" else ENGLISH_CONSTRUCTION_HISTORY),
        expected_no_history=(ENGLISH_TERMINAL_NO_HISTORY if split == "terminal" else ENGLISH_CONSTRUCTION_NO_HISTORY),
    )


def load_position_kv_binding_construction(root: Path) -> PositionKVConstructionData:
    """Load construction only; terminal artifact paths are never opened here."""

    if root.is_symlink():
        raise ValueError("#1043 data root must not be a symlink")
    root = root.resolve()
    manifest, commitment = _validate_public_envelopes(root)
    verify_artifact_subset(
        manifest,
        artifact_root=root,
        relative_paths=CONSTRUCTION_ARTIFACT_PATHS,
    )
    tokenizer_value = manifest.get("tokenizer")
    if not isinstance(tokenizer_value, Mapping) or tokenizer_value.get("path") is None:
        raise ValueError("#1043 tokenizer record is missing")
    tokenizer_path = Path(str(tokenizer_value["path"]))
    validate_tokenizer(tokenizer_path)
    natural_selection = _read_canonical_json(
        root / CONSTRUCTION_NATURAL_SELECTION_RELATIVE_PATH
    )
    natural = LanguagePathWindowStore(
        root / CONSTRUCTION_NATURAL_RELATIVE_PATH,
        window_count=NATURAL_CONSTRUCTION_WINDOWS,
    )
    if (
        natural_selection.get("schema") != NATURAL_SELECTION_SCHEMA
        or natural_selection.get("split") != "construction"
        or natural_selection.get("payload_cid")
        != _manifest_artifact_map(manifest)[CONSTRUCTION_NATURAL_RELATIVE_PATH]["cid"]
    ):
        raise ValueError("#1043 natural construction selection differs")
    mqar = _load_examples_payload(
        root / CONSTRUCTION_MQAR_RELATIVE_PATH,
        population="mqar",
        split="construction",
    )
    english_history, english_no_history = _load_examples_payload(
        root / CONSTRUCTION_ENGLISH_RELATIVE_PATH,
        population="english",
        split="construction",
    )
    return PositionKVConstructionData(
        root=root,
        manifest=manifest,
        commitment=commitment,
        tokenizer_path=tokenizer_path.resolve(),
        natural_windows=natural,
        natural_selection=natural_selection,
        mqar=mqar,
        english_history=english_history,
        english_no_history=english_no_history,
    )


def prepare_position_kv_binding_data(
    *,
    output_root: Path,
    retained_language_root: Path,
    source_root: Path,
    tokenizer_path: Path,
    excluded_story_cids: Collection[str],
) -> PositionKVDataPreparation:
    """Create all payloads once, seal terminal data, and return construction."""

    _validate_frozen_arithmetic()
    output_root = output_root.resolve()
    retained_language_root = retained_language_root.resolve()
    source_root = source_root.resolve()
    tokenizer_path = tokenizer_path.resolve()
    if output_root.exists() or output_root.is_symlink():
        raise FileExistsError("#1043 data root is create-once")
    tokenizer, token_ids = validate_tokenizer(tokenizer_path)

    natural_construction, natural_construction_selection = _materialize_natural_construction(
        retained_language_root / "data/train.u16"
    )
    natural_terminal, natural_terminal_selection = _materialize_natural_terminal(
        source_root / "tokens/train.u16",
        source_root / "indexes/train.jsonl",
        excluded_story_cids=excluded_story_cids,
    )
    construction_mqar = _generate_mqar_examples(
        count=MQAR_CONSTRUCTION_SEQUENCES,
        terminal=False,
    )
    terminal_mqar = _generate_mqar_examples(
        count=MQAR_TERMINAL_SEQUENCES,
        terminal=True,
    )
    construction_history, construction_no_history = _generate_english_examples(
        tokenizer=tokenizer,
        token_ids=token_ids,
        terminal=False,
        history_count=ENGLISH_CONSTRUCTION_HISTORY,
        no_history_count=ENGLISH_CONSTRUCTION_NO_HISTORY,
    )
    terminal_history, terminal_no_history = _generate_english_examples(
        tokenizer=tokenizer,
        token_ids=token_ids,
        terminal=True,
        history_count=ENGLISH_TERMINAL_HISTORY,
        no_history_count=ENGLISH_TERMINAL_NO_HISTORY,
    )
    disjointness = _population_disjointness(
        construction_mqar,
        terminal_mqar,
        construction_history,
        terminal_history,
    )
    oracle = serialization_oracle(
        terminal_mqar,
        terminal_history,
        tokenizer=tokenizer,
        token_ids=token_ids,
    )
    if not oracle.passed:
        raise PositionKVPopulationUnavailable("#1043 direct serialization oracle failed")

    payloads: dict[str, bytes] = {
        CONSTRUCTION_NATURAL_RELATIVE_PATH: natural_construction,
        CONSTRUCTION_NATURAL_SELECTION_RELATIVE_PATH: canonical_json_bytes(
            natural_construction_selection
        ),
        CONSTRUCTION_MQAR_RELATIVE_PATH: canonical_json_bytes(
            _mqar_payload(construction_mqar, split="construction")
        ),
        CONSTRUCTION_ENGLISH_RELATIVE_PATH: canonical_json_bytes(
            _english_payload(
                construction_history,
                construction_no_history,
                split="construction",
            )
        ),
        TERMINAL_NATURAL_RELATIVE_PATH: natural_terminal,
        TERMINAL_NATURAL_SELECTION_RELATIVE_PATH: canonical_json_bytes(
            natural_terminal_selection
        ),
        TERMINAL_MQAR_RELATIVE_PATH: canonical_json_bytes(
            _mqar_payload(terminal_mqar, split="terminal")
        ),
        TERMINAL_ENGLISH_RELATIVE_PATH: canonical_json_bytes(
            _english_payload(
                terminal_history,
                terminal_no_history,
                split="terminal",
            )
        ),
    }

    output_root.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{output_root.name}.preparing-", dir=output_root.parent)
    )
    try:
        for relative_path, payload in payloads.items():
            _write_exclusive(staging / relative_path, payload)
        records = artifact_records(staging, ALL_ARTIFACT_PATHS)
        records_by_path = {record["path"]: record for record in records}
        commitment = _with_self_cid(
            {
                "schema": COMMITMENT_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "terminal_artifacts": [
                    records_by_path[path] for path in sorted(TERMINAL_ARTIFACT_PATHS)
                ],
                "direct_serialization_oracle": oracle.record(),
                "english_no_history_serialization": {
                    **_unique_input_serialization_identity(terminal_no_history),
                    "role": (
                        "abstention control; scored rows are not independent "
                        "serialized inputs"
                    ),
                },
                "disjointness": disjointness,
                "terminal_payload_reads_before_final_artifact": 0,
                "sealed_mode": "000",
            },
            "commitment_cid",
        )
        _write_exclusive_json(staging / COMMITMENT_RELATIVE_PATH, commitment)
        manifest_body = {
            "schema": MANIFEST_SCHEMA,
            "issue": ISSUE,
            "policy": POLICY,
            "seed": SEED,
            "tokenizer_cid": EXPECTED_TOKENIZER_CID,
            "tokenizer": {"path": str(tokenizer_path), "cid": EXPECTED_TOKENIZER_CID},
            "retained_language_root": str(retained_language_root),
            "source_root": str(source_root),
            "population": {
                "natural_construction_windows": NATURAL_CONSTRUCTION_WINDOWS,
                "natural_terminal_windows": NATURAL_TERMINAL_WINDOWS,
                "mqar_construction_sequences": MQAR_CONSTRUCTION_SEQUENCES,
                "mqar_terminal_sequences": MQAR_TERMINAL_SEQUENCES,
                "english_construction_history": ENGLISH_CONSTRUCTION_HISTORY,
                "english_construction_no_history": ENGLISH_CONSTRUCTION_NO_HISTORY,
                "english_terminal_worlds": ENGLISH_TERMINAL_WORLDS,
                "english_terminal_history": ENGLISH_TERMINAL_HISTORY,
                "english_terminal_no_history": ENGLISH_TERMINAL_NO_HISTORY,
            },
            "commitment_cid": commitment["commitment_cid"],
            "artifacts": records,
            "tree_cid": tree_cid(records),
        }
        manifest = dict(manifest_body)
        manifest["manifest_cid"] = _cid(canonical_json_bytes(manifest_body))
        _write_exclusive_json(staging / MANIFEST_RELATIVE_PATH, manifest)
        sealed = staging / SEALED_DIRECTORY_RELATIVE_PATH
        sealed.chmod(0o000)
        if output_root.exists() or output_root.is_symlink():
            raise FileExistsError("#1043 data root appeared during preparation")
        staging.rename(output_root)
    except BaseException:
        sealed = staging / SEALED_DIRECTORY_RELATIVE_PATH
        if sealed.exists() and not sealed.is_symlink():
            sealed.chmod(0o700)
        if staging.exists() and not staging.is_symlink():
            shutil.rmtree(staging)
        raise
    construction = load_position_kv_binding_construction(output_root)
    return PositionKVDataPreparation(
        root=output_root,
        manifest=construction.manifest,
        commitment=construction.commitment,
        construction=construction,
    )


def _final_artifact_identity(path: Path) -> tuple[Path, str]:
    if (
        path.is_symlink()
        or not path.is_file()
        or path.stat().st_size != CAMPAIGN_ARTIFACT_BYTES
    ):
        raise ValueError(
            "final #1043 artifact must be the exact-size regular non-symlink artifact"
        )
    resolved = path.resolve()
    cid = cid_file(resolved)
    if not _is_cid(cid):
        raise RuntimeError("final #1043 artifact CID is malformed")
    return resolved, cid


def _completed_fit_identity(
    root: Path,
    *,
    manifest: Mapping[str, Any],
    final_path: Path,
    final_cid: str,
) -> dict[str, Any]:
    """Verify the exact completed campaign trajectory that permits reveal."""

    expected_artifact = (root / CAMPAIGN_ARTIFACT_RELATIVE_PATH).resolve()
    if final_path != expected_artifact:
        raise ValueError("#1043 reveal requires the campaign's bound final artifact path")

    preparation = _read_canonical_json(root / CAMPAIGN_PREPARATION_RELATIVE_PATH)
    _verify_self_cid(preparation, "preparation_cid")
    implementation = preparation.get("implementation")
    manifest_cid = manifest.get("manifest_cid")
    if (
        preparation.get("schema") != CAMPAIGN_PREPARATION_SCHEMA
        or preparation.get("issue") != ISSUE
        or preparation.get("policy") != POLICY
        or not isinstance(implementation, Mapping)
        or preparation.get("data_manifest") != dict(manifest)
        or preparation.get("data_manifest_cid") != manifest_cid
        or not _is_cid(manifest_cid)
    ):
        raise ValueError("#1043 campaign preparation does not bind this data contract")

    preflight = _read_canonical_json(root / CAMPAIGN_PREFLIGHT_RELATIVE_PATH)
    _verify_self_cid(preflight, "preflight_cid")
    if (
        preflight.get("schema") != CAMPAIGN_PREFLIGHT_SCHEMA
        or preflight.get("issue") != ISSUE
        or preflight.get("policy") != POLICY
        or preflight.get("preparation_cid") != preparation["preparation_cid"]
        or preflight.get("data_manifest_cid") != manifest_cid
        or preflight.get("implementation") != implementation
        or preflight.get("passed") is not True
        or preflight.get("terminal_payload_reads") != 0
    ):
        raise ValueError("#1043 campaign preflight is not a completed bound pass")

    started = _read_canonical_json(root / CAMPAIGN_STARTED_RELATIVE_PATH)
    _verify_self_cid(started, "started_cid")
    run_contract = started.get("run_contract")
    optimizer = run_contract.get("optimizer") if isinstance(run_contract, Mapping) else None
    if (
        started.get("schema") != CAMPAIGN_STARTED_SCHEMA
        or started.get("issue") != ISSUE
        or started.get("policy") != POLICY
        or started.get("preparation_cid") != preparation["preparation_cid"]
        or started.get("preflight_cid") != preflight["preflight_cid"]
        or started.get("implementation") != implementation
        or not isinstance(run_contract, Mapping)
        or run_contract.get("policy") != POLICY
        or run_contract.get("preparation_cid") != preparation["preparation_cid"]
        or run_contract.get("preflight_cid") != preflight["preflight_cid"]
        or run_contract.get("implementation") != implementation
        or run_contract.get("terminal_payload") != "SEALED_UNOPENED"
        or run_contract.get("cuda") != "FORBIDDEN"
        or run_contract.get("mps") != "FORBIDDEN"
        or not isinstance(optimizer, Mapping)
        or optimizer.get("steps") != CAMPAIGN_OPTIMIZER_STEPS
        or optimizer.get("batch_size") != 16
        or optimizer.get("composition")
        != {
            "natural": 8,
            "mqar": 4,
            "english_history": 3,
            "english_no_history": 1,
        }
        or optimizer.get("checkpoint_selection") != "NONE"
        or started.get("run_contract_cid") != _cid(canonical_json_bytes(run_contract))
        or started.get("terminal_payload_reads") != 0
    ):
        raise ValueError("#1043 campaign start envelope does not reproduce")

    fit = _read_canonical_json(root / CAMPAIGN_FIT_RELATIVE_PATH)
    _verify_self_cid(fit, "fit_cid")
    artifact = fit.get("artifact")
    work = fit.get("work")
    presentations = fit.get("presentations")
    expected_target_reads = (
        NATURAL_CONSTRUCTION_DECISIONS
        + MQAR_CONSTRUCTION_DECISIONS
        + ENGLISH_CONSTRUCTION_HISTORY
        + ENGLISH_CONSTRUCTION_NO_HISTORY
    )
    elapsed = fit.get("elapsed_seconds")
    if (
        fit.get("schema") != CAMPAIGN_FIT_SCHEMA
        or fit.get("issue") != ISSUE
        or fit.get("policy") != POLICY
        or fit.get("started_cid") != started["started_cid"]
        or fit.get("preparation_cid") != preparation["preparation_cid"]
        or fit.get("preflight_cid") != preflight["preflight_cid"]
        or fit.get("run_contract_cid") != started["run_contract_cid"]
        or fit.get("implementation") != implementation
        or fit.get("plan") != run_contract.get("plan")
        or fit.get("completed_steps") != CAMPAIGN_OPTIMIZER_STEPS
        or fit.get("optimizer_steps_after_reveal") != 0
        or fit.get("terminal_payload_reads_before_artifact_cid") != 0
        or presentations
        != {
            "natural": NATURAL_CONSTRUCTION_WINDOWS,
            "mqar": MQAR_CONSTRUCTION_SEQUENCES,
            "english_history": ENGLISH_CONSTRUCTION_HISTORY,
            "english_no_history": ENGLISH_CONSTRUCTION_NO_HISTORY,
        }
        or not isinstance(work, Mapping)
        or work.get("target_reads") != expected_target_reads
        or any(
            work.get(name, 0) != 0
            for name in ("provider_calls", "teacher_calls", "future_reads", "forbidden_reads")
        )
        or not isinstance(artifact, Mapping)
        or artifact
        != {
            "path": CAMPAIGN_ARTIFACT_RELATIVE_PATH,
            "bytes": final_path.stat().st_size,
            "cid": final_cid,
        }
        or not _is_cid(fit.get("loss_trace_cid"))
        or not isinstance(fit.get("first_loss"), Mapping)
        or not isinstance(fit.get("final_loss"), Mapping)
        or isinstance(elapsed, bool)
        or not isinstance(elapsed, (int, float))
        or not math.isfinite(float(elapsed))
        or not 0.0 <= float(elapsed) < 1_800.0
    ):
        raise ValueError("#1043 final artifact lacks its exact completed fit envelope")
    return fit


def reveal_position_kv_binding_terminal(
    root: Path,
    *,
    final_artifact_path: Path,
) -> PositionKVTerminalData:
    """Open terminal payloads only after binding a real final artifact CID."""

    root = root.resolve()
    manifest, _commitment = _validate_public_envelopes(root)
    final_path, final_cid = _final_artifact_identity(final_artifact_path)
    fit = _completed_fit_identity(
        root,
        manifest=manifest,
        final_path=final_path,
        final_cid=final_cid,
    )
    if final_path.is_relative_to(root / SEALED_DIRECTORY_RELATIVE_PATH):
        raise ValueError("final artifact must not live inside the sealed population")
    reveal_path = root / REVEAL_RELATIVE_PATH
    if reveal_path.exists() or reveal_path.is_symlink():
        reveal = _read_canonical_json(reveal_path)
        _verify_self_cid(reveal, "reveal_cid")
    else:
        reveal = _with_self_cid(
            {
                "schema": REVEAL_SCHEMA,
                "issue": ISSUE,
                "policy": POLICY,
                "commitment_cid": manifest["commitment_cid"],
                "final_artifact": {
                    "path": str(final_path),
                    "bytes": final_path.stat().st_size,
                    "cid": final_cid,
                },
                "final_artifact_cid": final_cid,
                "fit_cid": fit["fit_cid"],
                "reveal_count": 1,
            },
            "reveal_cid",
        )
    if (
        reveal.get("schema") != REVEAL_SCHEMA
        or reveal.get("issue") != ISSUE
        or reveal.get("policy") != POLICY
        or reveal.get("commitment_cid") != manifest["commitment_cid"]
        or reveal.get("fit_cid") != fit["fit_cid"]
        or reveal.get("final_artifact")
        != {
            "path": str(final_path),
            "bytes": final_path.stat().st_size,
            "cid": final_cid,
        }
        or reveal.get("final_artifact_cid") != final_cid
        or reveal.get("reveal_count") != 1
    ):
        raise ValueError("#1043 terminal population was revealed for another fit or artifact")
    sealed = root / SEALED_DIRECTORY_RELATIVE_PATH
    if sealed.is_symlink() or not sealed.is_dir():
        raise ValueError("#1043 sealed terminal directory differs")
    mode = stat.S_IMODE(sealed.stat().st_mode)
    if mode not in (0, 0o700):
        raise ValueError("#1043 sealed terminal directory mode differs")
    sealed.chmod(0o700)
    try:
        verify_artifact_subset(
            manifest,
            artifact_root=root,
            relative_paths=TERMINAL_ARTIFACT_PATHS,
        )
        natural_selection = _read_canonical_json(
            root / TERMINAL_NATURAL_SELECTION_RELATIVE_PATH
        )
        natural = LanguagePathWindowStore(
            root / TERMINAL_NATURAL_RELATIVE_PATH,
            window_count=NATURAL_TERMINAL_WINDOWS,
        )
        if (
            natural_selection.get("schema") != NATURAL_SELECTION_SCHEMA
            or natural_selection.get("split") != "terminal"
            or natural_selection.get("payload_cid")
            != _manifest_artifact_map(manifest)[TERMINAL_NATURAL_RELATIVE_PATH]["cid"]
        ):
            raise ValueError("#1043 natural terminal selection differs")
        mqar = _load_examples_payload(
            root / TERMINAL_MQAR_RELATIVE_PATH,
            population="mqar",
            split="terminal",
        )
        english_history, english_no_history = _load_examples_payload(
            root / TERMINAL_ENGLISH_RELATIVE_PATH,
            population="english",
            split="terminal",
        )
        tokenizer_value = manifest.get("tokenizer")
        if not isinstance(tokenizer_value, Mapping) or tokenizer_value.get("path") is None:
            raise ValueError("#1043 tokenizer record is missing")
        tokenizer_path = Path(str(tokenizer_value["path"]))
        tokenizer, token_ids = validate_tokenizer(tokenizer_path)
        oracle = serialization_oracle(
            mqar,
            english_history,
            tokenizer=tokenizer,
            token_ids=token_ids,
        )
        if not oracle.passed:
            raise ValueError("#1043 revealed serialization oracle differs")
        mqar_binding_permuted = binding_permuted_examples(mqar)
        english_binding_permuted = binding_permuted_examples(
            english_history,
            tokenizer_path=tokenizer_path,
        )
        if not reveal_path.exists():
            _write_exclusive_json(reveal_path, reveal)
    except BaseException:
        if not reveal_path.exists():
            sealed.chmod(0o000)
        raise
    return PositionKVTerminalData(
        root=root,
        manifest=manifest,
        reveal=reveal,
        final_artifact_cid=final_cid,
        natural_windows=natural,
        natural_selection=natural_selection,
        mqar=mqar,
        mqar_binding_permuted=mqar_binding_permuted,
        english_history=english_history,
        english_binding_permuted=english_binding_permuted,
        english_no_history=english_no_history,
    )


def load_revealed_position_kv_binding_terminal(
    root: Path,
    *,
    final_artifact_path: Path,
) -> PositionKVTerminalData:
    """Replay the one reveal; a different artifact CID remains forbidden."""

    if not (root.resolve() / REVEAL_RELATIVE_PATH).is_file():
        raise ValueError("#1043 terminal population has not been revealed")
    return reveal_position_kv_binding_terminal(
        root,
        final_artifact_path=final_artifact_path,
    )
