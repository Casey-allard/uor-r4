"""Open, causal role-sidecar data for issue #1045.

This module is deliberately downstream of the frozen #1043 *construction*
loader.  It neither names nor opens #1043's fitted model, reveal envelope, or
sealed evaluation directory.  Role IDs are reconstructed from the physical
input serialization only; labels and binding metadata never enter either the
role annotator or the open-development split rank.
"""

from __future__ import annotations

import struct
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from enum import IntEnum
from pathlib import Path

import torch
from blake3 import blake3
from tokenizers import Tokenizer
from torch import Tensor

from .language_path_generalization_data import LanguagePathWindowStore
from .position_kv_binding_data import (
    BOS_TOKEN_ID,
    CONTEXT,
    IGNORE_INDEX,
    KEY_LEXICON,
    MQAR_FILLER_MAX,
    MQAR_FILLER_MIN,
    MQAR_KEY_MAX,
    MQAR_KEY_MIN,
    MQAR_QUERIES,
    MQAR_RECORDS,
    MQAR_VALUE_MAX,
    MQAR_VALUE_MIN,
    VALUE_LEXICON,
    CausalBindingExample,
    PositionKVConstructionData,
    load_position_kv_binding_construction,
    validate_tokenizer,
)


ISSUE = 1045
POLICY = "R4RoleTaggedAssociativeCurriculumV1"
SPLIT_NAMESPACE = b"uor-r4/1045/role-tagged-associative/open-mqar-split/v1\0"
ROW_NAMESPACE = b"uor-r4/1045/role-tagged-associative/row/v1\0"

MQAR_TRAIN_ROWS = 8_192
MQAR_DEVELOPMENT_ROWS = 1_024
MQAR_CONTROL_ROWS = 1_704
MQAR_TOTAL_ROWS = MQAR_TRAIN_ROWS + MQAR_DEVELOPMENT_ROWS + MQAR_CONTROL_ROWS
ENGLISH_FACTS = 4


class RoleCode(IntEnum):
    """The W8-compatible categorical role ABI.

    These values are identities, not distances or arithmetic residues.
    """

    TEXT = 0
    KEY = 1
    VALUE = 2
    QUERY = 3


ROLE_COUNT = len(RoleCode)


def _input_bytes(input_ids: Sequence[int]) -> bytes:
    payload = bytearray(struct.pack(">I", len(input_ids)))
    for token in input_ids:
        if isinstance(token, bool) or not isinstance(token, int) or not 0 <= token <= 0xFFFF:
            raise ValueError("role input token is outside the u16 serialization")
        payload.extend(struct.pack(">H", token))
    return bytes(payload)


def _role_bytes(role_ids: Sequence[int]) -> bytes:
    if any(
        isinstance(role, bool)
        or not isinstance(role, int)
        or not int(RoleCode.TEXT) <= role <= int(RoleCode.QUERY)
        for role in role_ids
    ):
        raise ValueError("role sidecar contains an unknown uint8 role")
    return bytes(role_ids)


def _row_id(population: str, input_ids: Sequence[int], role_ids: Sequence[int]) -> str:
    digest = blake3(
        ROW_NAMESPACE
        + population.encode("ascii")
        + b"\0"
        + _input_bytes(input_ids)
        + _role_bytes(role_ids)
    ).hexdigest()
    return f"blake3:{digest}"


def _physical_mqar_pairs(input_ids: Sequence[int]) -> tuple[tuple[int, int], ...]:
    """Decode the eight bindings from fixed physical record slots."""

    pairs: list[tuple[int, int]] = []
    for record_index in range(MQAR_RECORDS):
        position = record_index * 4
        if position + 1 >= len(input_ids):
            raise ValueError("MQAR prefix does not contain every record slot")
        key = input_ids[position]
        value = input_ids[position + 1]
        if not MQAR_KEY_MIN <= key <= MQAR_KEY_MAX:
            raise ValueError("MQAR physical key slot is outside the key range")
        if not MQAR_VALUE_MIN <= value <= MQAR_VALUE_MAX:
            raise ValueError("MQAR physical value slot is outside the value range")
        pairs.append((key, value))
    if len({key for key, _value in pairs}) != MQAR_RECORDS:
        raise ValueError("MQAR physical record keys are not unique")
    if len({value for _key, value in pairs}) != MQAR_RECORDS:
        raise ValueError("MQAR physical record values are not unique")
    return tuple(pairs)


def derive_mqar_role_ids(
    input_ids: Sequence[int],
    *,
    require_complete: bool = True,
) -> tuple[int, ...]:
    """Derive MQAR roles causally from fixed slots and disjoint token ranges."""

    if not input_ids or len(input_ids) > CONTEXT:
        raise ValueError("MQAR role input is outside the frozen context")
    roles: list[int] = []
    record_keys: set[int] = set()
    record_values: set[int] = set()
    query_keys: list[int] = []
    for position, token in enumerate(input_ids):
        record_offset = position % 4
        if position < MQAR_RECORDS * 4 and record_offset == 0:
            if not MQAR_KEY_MIN <= token <= MQAR_KEY_MAX:
                raise ValueError("MQAR record-key slot is outside the key range")
            if token in record_keys:
                raise ValueError("MQAR record key repeats")
            record_keys.add(token)
            roles.append(int(RoleCode.KEY))
        elif position < MQAR_RECORDS * 4 and record_offset == 1:
            if not MQAR_VALUE_MIN <= token <= MQAR_VALUE_MAX:
                raise ValueError("MQAR record-value slot is outside the value range")
            if token in record_values:
                raise ValueError("MQAR record value repeats")
            record_values.add(token)
            roles.append(int(RoleCode.VALUE))
        elif MQAR_KEY_MIN <= token <= MQAR_KEY_MAX:
            if position < MQAR_RECORDS * 4 or token not in record_keys:
                raise ValueError("MQAR query does not name a prior physical record")
            roles.append(int(RoleCode.QUERY))
            query_keys.append(token)
        else:
            if not MQAR_FILLER_MIN <= token <= MQAR_FILLER_MAX:
                raise ValueError("MQAR non-role slot is outside the filler range")
            roles.append(int(RoleCode.TEXT))
    if require_complete and (
        len(input_ids) != CONTEXT
        or len(record_keys) != MQAR_RECORDS
        or len(record_values) != MQAR_RECORDS
        or len(query_keys) != MQAR_QUERIES
        or set(query_keys) != record_keys
        or len(set(query_keys)) != len(query_keys)
    ):
        raise ValueError("MQAR row differs from the complete #1043 construction shape")
    return tuple(roles)


@dataclass(frozen=True, slots=True)
class EnglishRoleSchema:
    """Tokenizer-bound finite-state markers for construction English."""

    key_ids: frozenset[int]
    value_ids: frozenset[int]
    history_starts: tuple[tuple[int, ...], ...]
    no_history_starts: tuple[tuple[int, ...], ...]
    complete_endings_by_family: tuple[tuple[tuple[int, ...], ...], ...]

    def __post_init__(self) -> None:
        if (
            len(self.key_ids) != len(KEY_LEXICON)
            or len(self.value_ids) != len(VALUE_LEXICON)
            or self.key_ids.intersection(self.value_ids)
        ):
            raise ValueError("English role lexicon token IDs are not disjoint singletons")
        if (
            len(self.history_starts) != 3
            or len(self.no_history_starts) != 3
            or len(self.complete_endings_by_family) != 3
            or any(not pattern for pattern in (*self.history_starts, *self.no_history_starts))
        ):
            raise ValueError("English role schema contains an empty start marker")
        if any(not endings for endings in self.complete_endings_by_family):
            raise ValueError("English role schema contains no completion marker")


def _encode(tokenizer: Tokenizer, text: str) -> tuple[int, ...]:
    return tuple(tokenizer.encode(text, add_special_tokens=False).ids)


def build_english_role_schema(
    tokenizer: Tokenizer,
    token_ids: Mapping[str, int],
) -> EnglishRoleSchema:
    """Bind causal construction markers to the exact frozen tokenizer."""

    missing = set((*KEY_LEXICON, *VALUE_LEXICON)).difference(token_ids)
    if missing:
        raise ValueError("English role token map is incomplete")
    key_ids = frozenset(int(token_ids[value]) for value in KEY_LEXICON)
    value_ids = frozenset(int(token_ids[value]) for value in VALUE_LEXICON)
    history_starts = tuple(
        _encode(tokenizer, text)
        for text in (
            "Context: The",
            "Read these notes. We put the",
            "Placement record:",
        )
    )
    no_history_starts = tuple(
        _encode(tokenizer, text)
        for text in (
            "Question: Where is the",
            "Give the location of the",
            "Look up",
        )
    )
    endings_by_family = tuple(
        tuple(_encode(tokenizer, template.format(key=key)) for key in KEY_LEXICON)
        for template in (" {key}? Answer:", " {key}:", " {key}. Location:")
    )
    return EnglishRoleSchema(
        key_ids=key_ids,
        value_ids=value_ids,
        history_starts=history_starts,
        no_history_starts=no_history_starts,
        complete_endings_by_family=endings_by_family,
    )


def _starts_with_any(
    input_ids: Sequence[int], patterns: Sequence[Sequence[int]]
) -> bool:
    return any(
        len(input_ids) >= len(pattern)
        and tuple(input_ids[: len(pattern)]) == tuple(pattern)
        for pattern in patterns
    )


def _ends_with_any(
    input_ids: Sequence[int], patterns: Sequence[Sequence[int]]
) -> bool:
    return any(
        len(input_ids) >= len(pattern)
        and tuple(input_ids[-len(pattern) :]) == tuple(pattern)
        for pattern in patterns
    )


def derive_english_role_ids(
    input_ids: Sequence[int],
    schema: EnglishRoleSchema,
    *,
    require_complete: bool = True,
) -> tuple[int, ...]:
    """Annotate one construction-English prefix without labels or metadata.

    The serialized prompt identifies history versus no-history before the
    first variable token arrives.  History has exactly four physical fact
    keys; later key-lexicon tokens are queries.  A completion marker tags the
    answer-readout punctuation as QUERY as soon as that marker is observed.
    """

    if not input_ids or len(input_ids) > CONTEXT:
        raise ValueError("English role input is outside the frozen context")
    history_families = tuple(
        index
        for index, pattern in enumerate(schema.history_starts)
        if _starts_with_any(input_ids, (pattern,))
    )
    no_history_families = tuple(
        index
        for index, pattern in enumerate(schema.no_history_starts)
        if _starts_with_any(input_ids, (pattern,))
    )
    if len(history_families) + len(no_history_families) != 1:
        # A short prefix may not yet contain its entire start marker.  It has
        # no lexical role at that point, so TEXT is the only causal result.
        if not any(token in schema.key_ids or token in schema.value_ids for token in input_ids):
            return (int(RoleCode.TEXT),) * len(input_ids)
        raise ValueError("English physical serialization has no unique construction start")
    history = bool(history_families)
    family = (history_families or no_history_families)[0]

    roles: list[int] = []
    key_occurrence = 0
    value_occurrence = 0
    physical_keys: list[int] = []
    physical_values: list[int] = []
    for token in input_ids:
        if token in schema.key_ids:
            key_occurrence += 1
            physical_keys.append(token)
            role = (
                RoleCode.KEY
                if history and key_occurrence <= ENGLISH_FACTS
                else RoleCode.QUERY
            )
            roles.append(int(role))
        elif token in schema.value_ids:
            value_occurrence += 1
            physical_values.append(token)
            roles.append(int(RoleCode.VALUE))
        else:
            roles.append(int(RoleCode.TEXT))

    complete = _ends_with_any(input_ids, schema.complete_endings_by_family[family])
    if complete:
        roles[-1] = int(RoleCode.QUERY)
    if require_complete:
        expected_keys = ENGLISH_FACTS + 1 if history else 1
        expected_values = ENGLISH_FACTS if history else 0
        if (
            not complete
            or key_occurrence != expected_keys
            or value_occurrence != expected_values
            or roles.count(int(RoleCode.QUERY)) != 2
            or (
                history
                and (
                    len(set(physical_keys[:ENGLISH_FACTS])) != ENGLISH_FACTS
                    or len(set(physical_values)) != ENGLISH_FACTS
                    or physical_keys[-1] not in physical_keys[:ENGLISH_FACTS]
                )
            )
        ):
            raise ValueError("English row differs from a complete construction serialization")
    return tuple(roles)


def natural_role_ids(input_ids: Sequence[int]) -> tuple[int, ...]:
    """Natural-language replay has no synthetic side-channel roles."""

    _input_bytes(input_ids)
    if not input_ids:
        raise ValueError("natural role input cannot be empty")
    return (int(RoleCode.TEXT),) * len(input_ids)


@dataclass(frozen=True, slots=True)
class RoleTaggedExample:
    source: CausalBindingExample
    role_ids: tuple[int, ...]
    stable_id: str

    def __post_init__(self) -> None:
        if len(self.role_ids) != len(self.source.input_ids):
            raise ValueError("role sidecar length differs from its token row")
        _role_bytes(self.role_ids)
        expected = _row_id(self.source.population, self.source.input_ids, self.role_ids)
        if self.stable_id != expected:
            raise ValueError("role-tagged stable ID does not reproduce")

    @property
    def population(self) -> str:
        return self.source.population

    @property
    def input_ids(self) -> tuple[int, ...]:
        return self.source.input_ids

    @property
    def labels(self) -> tuple[int, ...]:
        return self.source.label_ids

    @property
    def label_ids(self) -> tuple[int, ...]:
        return self.source.label_ids


def tag_mqar_example(example: CausalBindingExample) -> RoleTaggedExample:
    if example.population != "mqar" or example.split != "construction":
        raise ValueError("#1045 accepts only open construction MQAR rows")
    roles = derive_mqar_role_ids(example.input_ids)
    return RoleTaggedExample(
        source=example,
        role_ids=roles,
        stable_id=_row_id(example.population, example.input_ids, roles),
    )


def tag_english_example(
    example: CausalBindingExample,
    schema: EnglishRoleSchema,
) -> RoleTaggedExample:
    if example.population not in ("english_history", "english_no_history"):
        raise ValueError("#1045 English tagger received another population")
    if example.split != "construction":
        raise ValueError("#1045 accepts only open construction English rows")
    roles = derive_english_role_ids(example.input_ids, schema)
    return RoleTaggedExample(
        source=example,
        role_ids=roles,
        stable_id=_row_id(example.population, example.input_ids, roles),
    )


def _mqar_rank(example: CausalBindingExample) -> tuple[bytes, bytes]:
    serialized = _input_bytes(example.input_ids)
    return blake3(SPLIT_NAMESPACE + serialized).digest(), serialized


@dataclass(frozen=True, slots=True)
class MQAROpenSplit:
    train: tuple[RoleTaggedExample, ...]
    development: tuple[RoleTaggedExample, ...]
    controls: tuple[RoleTaggedExample, ...]
    split_cid: str

    def __post_init__(self) -> None:
        if (
            len(self.train) != MQAR_TRAIN_ROWS
            or len(self.development) != MQAR_DEVELOPMENT_ROWS
            or len(self.controls) != MQAR_CONTROL_ROWS
        ):
            raise ValueError("#1045 MQAR split counts differ")
        expected = _split_cid(self.train, self.development, self.controls)
        if self.split_cid != expected:
            raise ValueError("#1045 MQAR split CID does not reproduce")


def _split_cid(
    train: Sequence[RoleTaggedExample],
    development: Sequence[RoleTaggedExample],
    controls: Sequence[RoleTaggedExample],
) -> str:
    payload = bytearray(SPLIT_NAMESPACE)
    for name, rows in (
        (b"train", train),
        (b"development", development),
        (b"controls", controls),
    ):
        payload.extend(name + b"\0")
        for row in rows:
            payload.extend(bytes.fromhex(row.stable_id.removeprefix("blake3:")))
    return f"blake3:{blake3(bytes(payload)).hexdigest()}"


def _assignment_and_pairs(
    row: RoleTaggedExample,
) -> tuple[tuple[tuple[int, int], ...], frozenset[tuple[int, int]]]:
    assignment = _physical_mqar_pairs(row.input_ids)
    return assignment, frozenset(assignment)


def _validate_mqar_disjointness(split: MQAROpenSplit) -> None:
    seen_rows: set[str] = set()
    seen_assignments: set[tuple[tuple[int, int], ...]] = set()
    population_pairs: list[set[tuple[int, int]]] = []
    for population in (split.train, split.development, split.controls):
        pairs: set[tuple[int, int]] = set()
        for row in population:
            if row.stable_id in seen_rows:
                raise ValueError("#1045 MQAR row repeats across open populations")
            seen_rows.add(row.stable_id)
            assignment, row_pairs = _assignment_and_pairs(row)
            if assignment in seen_assignments:
                raise ValueError("#1045 MQAR assignment repeats across open populations")
            seen_assignments.add(assignment)
            if pairs.intersection(row_pairs):
                raise ValueError("#1045 MQAR pair repeats within one open population")
            pairs.update(row_pairs)
        population_pairs.append(pairs)
    if any(
        population_pairs[left].intersection(population_pairs[right])
        for left in range(len(population_pairs))
        for right in range(left + 1, len(population_pairs))
    ):
        raise ValueError("#1045 MQAR pair crosses train/development/control")


def split_mqar_construction(
    examples: Sequence[CausalBindingExample],
) -> MQAROpenSplit:
    """Rank all 10,920 open rows using input bytes only, then split once."""

    if len(examples) != MQAR_TOTAL_ROWS:
        raise ValueError("#1045 requires all 10,920 #1043 construction MQAR rows")
    ranked = sorted(examples, key=_mqar_rank)
    tagged = tuple(tag_mqar_example(example) for example in ranked)
    train = tagged[:MQAR_TRAIN_ROWS]
    development = tagged[
        MQAR_TRAIN_ROWS : MQAR_TRAIN_ROWS + MQAR_DEVELOPMENT_ROWS
    ]
    controls = tagged[MQAR_TRAIN_ROWS + MQAR_DEVELOPMENT_ROWS :]
    split = MQAROpenSplit(
        train=train,
        development=development,
        controls=controls,
        split_cid=_split_cid(train, development, controls),
    )
    _validate_mqar_disjointness(split)
    return split


@dataclass(frozen=True, slots=True)
class RoleTaggedConstruction:
    source: PositionKVConstructionData
    mqar_train: tuple[RoleTaggedExample, ...]
    mqar_development: tuple[RoleTaggedExample, ...]
    mqar_controls: tuple[RoleTaggedExample, ...]
    english_history: tuple[RoleTaggedExample, ...]
    english_no_history: tuple[RoleTaggedExample, ...]
    english_schema: EnglishRoleSchema
    split_cid: str

    @property
    def natural_windows(self) -> LanguagePathWindowStore:
        return self.source.natural_windows


def load_role_tagged_construction(source_root: Path) -> RoleTaggedConstruction:
    """Load and annotate only the already-open #1043 construction population."""

    source = load_position_kv_binding_construction(source_root)
    tokenizer, token_ids = validate_tokenizer(source.tokenizer_path)
    schema = build_english_role_schema(tokenizer, token_ids)
    mqar = split_mqar_construction(source.mqar)
    english_history = tuple(
        tag_english_example(example, schema) for example in source.english_history
    )
    english_no_history = tuple(
        tag_english_example(example, schema) for example in source.english_no_history
    )
    return RoleTaggedConstruction(
        source=source,
        mqar_train=mqar.train,
        mqar_development=mqar.development,
        mqar_controls=mqar.controls,
        english_history=english_history,
        english_no_history=english_no_history,
        english_schema=schema,
        split_cid=mqar.split_cid,
    )


@dataclass(frozen=True, slots=True)
class RoleTaggedBatch:
    input_ids: Tensor
    role_ids: Tensor
    labels: Tensor
    lengths: Tensor
    selected_positions: Tensor
    targets: Tensor

    @property
    def label_ids(self) -> Tensor:
        return self.labels

    def __post_init__(self) -> None:
        if (
            self.input_ids.dtype != torch.long
            or self.labels.dtype != torch.long
            or self.role_ids.dtype != torch.uint8
            or self.lengths.dtype != torch.long
            or self.selected_positions.dtype != torch.long
            or self.targets.dtype != torch.long
            or self.input_ids.ndim != 2
            or self.labels.shape != self.input_ids.shape
            or self.role_ids.shape != self.input_ids.shape
            or self.lengths.shape != (self.input_ids.shape[0],)
            or self.selected_positions.ndim != 2
            or self.selected_positions.shape[0] != self.input_ids.shape[0]
            or self.targets.shape != self.selected_positions.shape
        ):
            raise ValueError("role-tagged batch tensor contract differs")
        tensors = (
            self.input_ids,
            self.role_ids,
            self.labels,
            self.lengths,
            self.selected_positions,
            self.targets,
        )
        if len({tensor.device for tensor in tensors}) != 1:
            raise ValueError("role-tagged batch tensors are on different devices")
        if bool((self.role_ids > int(RoleCode.QUERY)).any()):
            raise ValueError("role-tagged batch contains an unknown role")
        if bool(
            ((self.lengths < 1) | (self.lengths > self.input_ids.shape[1])).any()
        ):
            raise ValueError("role-tagged batch length is outside the padded width")
        if bool(
            (
                (self.selected_positions < 0)
                | (self.selected_positions >= self.lengths[:, None])
            ).any()
        ):
            raise ValueError("role-tagged selected position is outside its row")
        if any(
            len(set(int(value) for value in row.tolist())) != row.numel()
            for row in self.selected_positions.detach().cpu()
        ):
            raise ValueError("role-tagged selected positions repeat within a row")
        selected_labels = torch.gather(self.labels, 1, self.selected_positions)
        if bool((selected_labels == IGNORE_INDEX).any()) or not torch.equal(
            selected_labels, self.targets
        ):
            raise ValueError("role-tagged selected targets do not match labels")


def batch_role_tagged_examples(
    rows: Sequence[RoleTaggedExample],
    device: torch.device | str | None = None,
) -> RoleTaggedBatch:
    """Right-pad rows and expose uniform per-row scored positions ``[B,Q]``."""

    if not rows:
        raise ValueError("role-tagged batch cannot be empty")
    maximum = max(len(row.input_ids) for row in rows)
    input_ids = torch.full((len(rows), maximum), BOS_TOKEN_ID, dtype=torch.long)
    role_ids = torch.full(
        (len(rows), maximum), int(RoleCode.TEXT), dtype=torch.uint8
    )
    labels = torch.full((len(rows), maximum), IGNORE_INDEX, dtype=torch.long)
    lengths = torch.empty(len(rows), dtype=torch.long)
    for index, row in enumerate(rows):
        width = len(row.input_ids)
        input_ids[index, :width] = torch.tensor(row.input_ids, dtype=torch.long)
        role_ids[index, :width] = torch.tensor(row.role_ids, dtype=torch.uint8)
        labels[index, :width] = torch.tensor(row.labels, dtype=torch.long)
        lengths[index] = width
    selected_by_row = [
        tuple(index for index, label in enumerate(row.labels) if label != IGNORE_INDEX)
        for row in rows
    ]
    query_counts = {len(positions) for positions in selected_by_row}
    if len(query_counts) != 1 or next(iter(query_counts)) < 1:
        raise ValueError("one role-tagged batch requires a uniform positive query count")
    selected_positions = torch.tensor(selected_by_row, dtype=torch.long)
    targets = torch.gather(labels, 1, selected_positions)
    if device is not None:
        input_ids = input_ids.to(device=device)
        role_ids = role_ids.to(device=device)
        labels = labels.to(device=device)
        lengths = lengths.to(device=device)
        selected_positions = selected_positions.to(device=device)
        targets = targets.to(device=device)
    return RoleTaggedBatch(
        input_ids=input_ids,
        role_ids=role_ids,
        labels=labels,
        lengths=lengths,
        selected_positions=selected_positions,
        targets=targets,
    )


def select_labeled_logits(logits: Tensor, batch: RoleTaggedBatch) -> tuple[Tensor, Tensor]:
    """Select only physically aligned supervised logits in deterministic order."""

    if logits.ndim != 3 or logits.shape[:2] != batch.input_ids.shape:
        raise ValueError("logits do not align with the role-tagged batch")
    positions = batch.selected_positions.unsqueeze(-1).expand(
        -1, -1, logits.shape[-1]
    )
    return torch.gather(logits, 1, positions), batch.targets


@dataclass(frozen=True, slots=True)
class RoleOracleResult:
    rows: int
    positions: int
    prefix_checks: int
    exact_rows: int
    label_reads: int = 0
    metadata_reads: int = 0

    @property
    def passed(self) -> bool:
        return (
            self.rows > 0
            and self.exact_rows == self.rows
            and self.positions > 0
            and self.prefix_checks == self.positions
            and self.label_reads == 0
            and self.metadata_reads == 0
        )


def validate_role_oracle(
    rows: Sequence[RoleTaggedExample],
    *,
    english_schema: EnglishRoleSchema | None = None,
) -> RoleOracleResult:
    """Rebuild every role and prove prefix invariance without target access."""

    if not rows:
        raise ValueError("role oracle requires at least one row")
    exact_rows = 0
    prefix_checks = 0
    positions = 0
    for row in rows:
        if row.population == "mqar":
            derive = lambda values, complete: derive_mqar_role_ids(  # noqa: E731
                values, require_complete=complete
            )
        elif row.population in ("english_history", "english_no_history"):
            if english_schema is None:
                raise ValueError("English role oracle requires its tokenizer schema")
            derive = lambda values, complete: derive_english_role_ids(  # noqa: E731
                values, english_schema, require_complete=complete
            )
        else:
            raise ValueError("role oracle received an unsupported population")
        observed = derive(row.input_ids, True)
        if observed != row.role_ids:
            raise ValueError("role sidecar differs from the physical-input oracle")
        if row.stable_id != _row_id(row.population, row.input_ids, observed):
            raise ValueError("role stable ID differs from physical inputs")
        exact_rows += 1
        positions += len(row.input_ids)
        for width in range(1, len(row.input_ids) + 1):
            prefix = derive(row.input_ids[:width], width == len(row.input_ids))
            if prefix != observed[:width]:
                raise ValueError("role annotator changes a past tag after future input")
            prefix_checks += 1
    result = RoleOracleResult(
        rows=len(rows),
        positions=positions,
        prefix_checks=prefix_checks,
        exact_rows=exact_rows,
    )
    if not result.passed:
        raise ValueError("role oracle did not pass")
    return result
