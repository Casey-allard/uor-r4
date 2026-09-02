# SPDX-License-Identifier: Apache-2.0
# Adapted from HazyResearch/Zoology; Copyright 2021 The Meerkat Team.
"""MQAR data for the bounded Zoology integration control in issue #1047.

The source-native generator is a bounded-memory adaptation of Zoology's
ICLR24 ``_mqar`` implementation at revision
``de4e258784224e09909c257ff3ea040f089ed660``.  It retains the released
legacy NumPy RNG stream, integer layout, power-law gap sampling, shifted label
alignment, and zero fillers used by this control.  See ``NOTICE.md`` and
``LICENSE-APACHE-2.0.md`` in this directory for attribution and license terms.

The exact-#1045 adapter keeps the already-open input, position, and target
bytes unchanged.  Its categorical role sidecar is re-derived and validated,
but :class:`ZoologyMQARBatch` deliberately has no role tensor, so roles cannot
be supplied to the copied model through this data boundary.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np
import torch
from blake3 import blake3
from torch import Tensor

from ..position_kv_binding_data import (
    CONTEXT as EXACT_1045_CONTEXT,
    MQAR_QUERIES as EXACT_1045_QUERIES,
    MQAR_RECORDS as EXACT_1045_RECORDS,
    VOCAB_SIZE as EXACT_1045_VOCAB_SIZE,
)
from ..provenance import canonical_json_bytes, cid_bytes
from ..role_tagged_associative_data import (
    MQAR_DEVELOPMENT_ROWS as EXACT_1045_DEVELOPMENT_ROWS,
    MQAR_TRAIN_ROWS as EXACT_1045_TRAIN_ROWS,
    RoleCode,
    derive_mqar_role_ids,
    load_role_tagged_construction,
)
from .provenance import ZOOLOGY_RELEASE_REVISION


SOURCE_NATIVE_VOCAB_SIZE = 8_192
SOURCE_NATIVE_INPUT_SEQ_LEN = 64
SOURCE_NATIVE_KV_PAIRS = 4
SOURCE_NATIVE_TRAIN_ROWS = 8_192
SOURCE_NATIVE_DEVELOPMENT_ROWS = 1_024
SOURCE_NATIVE_TRAIN_SEED = 0
SOURCE_NATIVE_DEVELOPMENT_SEED = 10
SOURCE_NATIVE_POWER_A = 0.01

SOURCE_ROW_NAMESPACE = b"uor-r4/1047/zoology/source-native-row/v1\0"
PERMUTED_ROW_NAMESPACE = b"uor-r4/1047/zoology/exact-binding-permutation/v1\0"
SHUFFLE_NAMESPACE = b"uor-r4/1047/zoology/release-style-shuffle/v1\0"


def _is_cid(value: object) -> bool:
    if not isinstance(value, str) or not value.startswith("blake3:"):
        return False
    digest = value.removeprefix("blake3:")
    return len(digest) == 64 and all(character in "0123456789abcdef" for character in digest)


def _integer_tuple(values: Sequence[int], *, label: str) -> tuple[int, ...]:
    result = tuple(values)
    if any(isinstance(value, bool) or not isinstance(value, int) for value in result):
        raise ValueError(f"{label} must contain only integers")
    return result


@dataclass(frozen=True, slots=True)
class ZoologyMQARRow:
    """One fixed-width MQAR row with only query positions supervised."""

    input_ids: tuple[int, ...]
    selected_positions: tuple[int, ...]
    targets: tuple[int, ...]
    stable_id: str
    role_ids: tuple[int, ...] | None = None

    def __post_init__(self) -> None:
        inputs = _integer_tuple(self.input_ids, label="input IDs")
        positions = _integer_tuple(self.selected_positions, label="selected positions")
        targets = _integer_tuple(self.targets, label="targets")
        if inputs != self.input_ids or positions != self.selected_positions or targets != self.targets:
            raise ValueError("MQAR row fields must be canonical tuples")
        if not inputs or any(token < 0 for token in inputs):
            raise ValueError("MQAR input IDs must be nonnegative and nonempty")
        if (
            not positions
            or positions != tuple(sorted(positions))
            or len(set(positions)) != len(positions)
            or any(position < 0 or position >= len(inputs) for position in positions)
        ):
            raise ValueError("MQAR selected positions must be unique, ordered, and in bounds")
        if len(targets) != len(positions) or any(target < 0 for target in targets):
            raise ValueError("MQAR targets must align with selected positions")
        if not _is_cid(self.stable_id):
            raise ValueError("MQAR stable ID is malformed")
        if self.role_ids is not None:
            roles = _integer_tuple(self.role_ids, label="role IDs")
            if roles != self.role_ids or len(roles) != len(inputs) or any(
                role < int(RoleCode.TEXT) or role > int(RoleCode.QUERY) for role in roles
            ):
                raise ValueError("MQAR role sidecar is malformed")

    def record(self) -> dict[str, Any]:
        return {
            "input_ids": list(self.input_ids),
            "selected_positions": list(self.selected_positions),
            "targets": list(self.targets),
            "stable_id": self.stable_id,
            "role_ids": None if self.role_ids is None else list(self.role_ids),
        }

    @property
    def row_cid(self) -> str:
        """Commit the complete row, including its inherited stable identity."""

        return cid_bytes(canonical_json_bytes(self.record()))


def _population_body(
    *,
    train: Sequence[ZoologyMQARRow],
    development: Sequence[ZoologyMQARRow],
    name: str,
    vocab_size: int,
    input_seq_len: int,
    num_kv_pairs: int,
    train_seed: int | None,
    development_seed: int | None,
    source_split_cid: str | None,
) -> dict[str, Any]:
    return {
        "schema": "uor-r4/zoology-mqar-population/v1",
        "zoology_release_oracle_revision": ZOOLOGY_RELEASE_REVISION,
        "name": name,
        "vocab_size": vocab_size,
        "input_seq_len": input_seq_len,
        "num_kv_pairs": num_kv_pairs,
        "train_seed": train_seed,
        "development_seed": development_seed,
        "source_split_cid": source_split_cid,
        "train_row_cids": [row.row_cid for row in train],
        "development_row_cids": [row.row_cid for row in development],
    }


@dataclass(frozen=True, slots=True)
class ZoologyMQARPopulation:
    """One create-once train/development population and its exact commitment."""

    train: tuple[ZoologyMQARRow, ...]
    development: tuple[ZoologyMQARRow, ...]
    population_cid: str
    name: str
    vocab_size: int
    input_seq_len: int
    num_kv_pairs: int
    train_seed: int | None = None
    development_seed: int | None = None
    source_split_cid: str | None = None

    def __post_init__(self) -> None:
        if not self.train or not self.development:
            raise ValueError("MQAR population requires nonempty train and development rows")
        if (
            not self.name
            or isinstance(self.vocab_size, bool)
            or not isinstance(self.vocab_size, int)
            or self.vocab_size < 2
            or isinstance(self.input_seq_len, bool)
            or not isinstance(self.input_seq_len, int)
            or self.input_seq_len < 1
            or isinstance(self.num_kv_pairs, bool)
            or not isinstance(self.num_kv_pairs, int)
            or self.num_kv_pairs < 1
        ):
            raise ValueError("MQAR population dimensions are invalid")
        for seed in (self.train_seed, self.development_seed):
            if seed is not None and (
                isinstance(seed, bool) or not isinstance(seed, int) or seed < 0
            ):
                raise ValueError("MQAR population seed must be a nonnegative integer")
        if self.source_split_cid is not None and not _is_cid(self.source_split_cid):
            raise ValueError("MQAR source split CID is malformed")

        all_rows = self.train + self.development
        if any(
            len(row.input_ids) != self.input_seq_len
            or len(row.selected_positions) != self.num_kv_pairs
            or any(token >= self.vocab_size for token in (*row.input_ids, *row.targets))
            for row in all_rows
        ):
            raise ValueError("MQAR row differs from its population dimensions")
        stable_ids = [row.stable_id for row in all_rows]
        row_cids = [row.row_cid for row in all_rows]
        if len(set(stable_ids)) != len(stable_ids) or len(set(row_cids)) != len(row_cids):
            raise ValueError("MQAR rows repeat within or across population splits")

        expected = cid_bytes(canonical_json_bytes(self._unsigned_record()))
        if self.population_cid != expected:
            raise ValueError("MQAR population CID does not reproduce")

    def _unsigned_record(self) -> dict[str, Any]:
        return _population_body(
            train=self.train,
            development=self.development,
            name=self.name,
            vocab_size=self.vocab_size,
            input_seq_len=self.input_seq_len,
            num_kv_pairs=self.num_kv_pairs,
            train_seed=self.train_seed,
            development_seed=self.development_seed,
            source_split_cid=self.source_split_cid,
        )

    def record(self) -> dict[str, Any]:
        return {**self._unsigned_record(), "population_cid": self.population_cid}


@dataclass(frozen=True, slots=True)
class ZoologyMQARBatch:
    """The complete model-facing data ABI; role IDs are intentionally absent."""

    input_ids: Tensor
    selected_positions: Tensor
    targets: Tensor

    def __post_init__(self) -> None:
        if (
            self.input_ids.dtype != torch.long
            or self.selected_positions.dtype != torch.long
            or self.targets.dtype != torch.long
            or self.input_ids.ndim != 2
            or self.selected_positions.ndim != 2
            or self.targets.shape != self.selected_positions.shape
            or self.selected_positions.shape[0] != self.input_ids.shape[0]
        ):
            raise ValueError("Zoology MQAR batch tensor contract differs")
        if len({self.input_ids.device, self.selected_positions.device, self.targets.device}) != 1:
            raise ValueError("Zoology MQAR batch tensors are on different devices")
        if bool(
            (
                (self.selected_positions < 0)
                | (self.selected_positions >= self.input_ids.shape[1])
            ).any()
        ):
            raise ValueError("Zoology MQAR selected position is outside its row")
        if any(
            tuple(sorted(int(value) for value in row.tolist()))
            != tuple(int(value) for value in row.tolist())
            or len(set(int(value) for value in row.tolist())) != row.numel()
            for row in self.selected_positions.detach().cpu()
        ):
            raise ValueError("Zoology MQAR selected positions must be ordered and unique")


def _released_mqar(
    *,
    vocab_size: int,
    num_examples: int,
    input_seq_len: int,
    seed: int,
    power_a: float = SOURCE_NATIVE_POWER_A,
    num_kv_pairs: int = 8,
) -> tuple[Tensor, Tensor]:
    """Reproduce released ``_mqar(..., random_non_queries=False)`` integers.

    Zoology tiles each choice vocabulary once per row before applying legacy
    ``np.random.choice``.  The row arrays are read-only choice populations, so
    sampling them sequentially from a local ``RandomState`` preserves the
    exact released RNG calls while avoiding both large tiled intermediates.
    """

    if (
        isinstance(vocab_size, bool)
        or not isinstance(vocab_size, int)
        or isinstance(num_examples, bool)
        or not isinstance(num_examples, int)
        or isinstance(input_seq_len, bool)
        or not isinstance(input_seq_len, int)
        or isinstance(seed, bool)
        or not isinstance(seed, int)
        or isinstance(num_kv_pairs, bool)
        or not isinstance(num_kv_pairs, int)
        or num_examples < 1
        or seed < 0
        or input_seq_len % 2 != 0
        or vocab_size <= input_seq_len
        or num_kv_pairs < 1
        or num_kv_pairs * 4 > input_seq_len
        or not np.isfinite(power_a)
        or power_a <= 0.0
    ):
        raise ValueError("released MQAR dimensions or power law are invalid")

    rng = np.random.RandomState(seed)
    key_vocab_size = vocab_size // 2
    key_choices = np.arange(1, key_vocab_size)
    value_choices = np.arange(key_vocab_size, vocab_size)

    keys = np.empty((num_examples, num_kv_pairs), dtype=np.int64)
    values = np.empty_like(keys)
    for row_index in range(num_examples):
        keys[row_index] = rng.choice(key_choices, replace=False, size=num_kv_pairs)
    for row_index in range(num_examples):
        values[row_index] = rng.choice(value_choices, replace=False, size=num_kv_pairs)

    context_size = num_kv_pairs * 2
    space = (input_seq_len - context_size) // 2
    gap_choices = np.arange(space, dtype=int)
    probabilities = power_a * np.arange(1, space + 1) ** (power_a - 1)
    probabilities = probabilities / probabilities.sum()
    gaps = np.empty_like(keys)
    for row_index in range(num_examples):
        gaps[row_index] = rng.choice(
            gap_choices,
            replace=False,
            p=probabilities,
            size=num_kv_pairs,
        )

    inputs = np.zeros((num_examples, input_seq_len), dtype=np.int64)
    labels = np.full((num_examples, input_seq_len), -100, dtype=np.int64)
    inputs[:, :context_size:2] = keys
    inputs[:, 1:context_size:2] = values
    query_positions = context_size + gaps * 2
    np.put_along_axis(inputs, query_positions, values=keys, axis=1)
    np.put_along_axis(labels, query_positions, values=values, axis=1)
    return torch.from_numpy(inputs), torch.from_numpy(labels)


def _source_rows(
    *,
    split: str,
    count: int,
    seed: int,
) -> tuple[ZoologyMQARRow, ...]:
    inputs, labels = _released_mqar(
        vocab_size=SOURCE_NATIVE_VOCAB_SIZE,
        num_examples=count,
        input_seq_len=SOURCE_NATIVE_INPUT_SEQ_LEN,
        seed=seed,
        num_kv_pairs=SOURCE_NATIVE_KV_PAIRS,
    )
    result: list[ZoologyMQARRow] = []
    for row_index in range(count):
        input_ids = tuple(int(value) for value in inputs[row_index].tolist())
        label_ids = tuple(int(value) for value in labels[row_index].tolist())
        selected = tuple(index for index, label in enumerate(label_ids) if label != -100)
        targets = tuple(label_ids[index] for index in selected)
        identity = {
            "source_revision": ZOOLOGY_RELEASE_REVISION,
            "split": split,
            "seed": seed,
            "row_index": row_index,
            "input_ids": list(input_ids),
            "selected_positions": list(selected),
            "targets": list(targets),
        }
        stable_id = cid_bytes(SOURCE_ROW_NAMESPACE + canonical_json_bytes(identity))
        result.append(
            ZoologyMQARRow(
                input_ids=input_ids,
                selected_positions=selected,
                targets=targets,
                stable_id=stable_id,
            )
        )
    return tuple(result)


def _make_population(
    *,
    train: Sequence[ZoologyMQARRow],
    development: Sequence[ZoologyMQARRow],
    name: str,
    vocab_size: int,
    input_seq_len: int,
    num_kv_pairs: int,
    train_seed: int | None = None,
    development_seed: int | None = None,
    source_split_cid: str | None = None,
) -> ZoologyMQARPopulation:
    train_tuple = tuple(train)
    development_tuple = tuple(development)
    body = _population_body(
        train=train_tuple,
        development=development_tuple,
        name=name,
        vocab_size=vocab_size,
        input_seq_len=input_seq_len,
        num_kv_pairs=num_kv_pairs,
        train_seed=train_seed,
        development_seed=development_seed,
        source_split_cid=source_split_cid,
    )
    return ZoologyMQARPopulation(
        train=train_tuple,
        development=development_tuple,
        population_cid=cid_bytes(canonical_json_bytes(body)),
        name=name,
        vocab_size=vocab_size,
        input_seq_len=input_seq_len,
        num_kv_pairs=num_kv_pairs,
        train_seed=train_seed,
        development_seed=development_seed,
        source_split_cid=source_split_cid,
    )


def build_source_calibration() -> ZoologyMQARPopulation:
    """Build the declared scaled source-native train/development population."""

    return _make_population(
        train=_source_rows(
            split="train",
            count=SOURCE_NATIVE_TRAIN_ROWS,
            seed=SOURCE_NATIVE_TRAIN_SEED,
        ),
        development=_source_rows(
            split="development",
            count=SOURCE_NATIVE_DEVELOPMENT_ROWS,
            seed=SOURCE_NATIVE_DEVELOPMENT_SEED,
        ),
        name="scaled_source_native",
        vocab_size=SOURCE_NATIVE_VOCAB_SIZE,
        input_seq_len=SOURCE_NATIVE_INPUT_SEQ_LEN,
        num_kv_pairs=SOURCE_NATIVE_KV_PAIRS,
        train_seed=SOURCE_NATIVE_TRAIN_SEED,
        development_seed=SOURCE_NATIVE_DEVELOPMENT_SEED,
    )


def _adapt_exact_row(tagged: Any) -> ZoologyMQARRow:
    input_ids = tuple(int(value) for value in tagged.input_ids)
    role_ids = tuple(int(value) for value in tagged.role_ids)
    if derive_mqar_role_ids(input_ids) != role_ids:
        raise ValueError("#1045 role bytes do not reproduce from exact input bytes")
    selected = tuple(int(value) for value in tagged.source.query_positions)
    targets = tuple(int(value) for value in tagged.source.answers)
    label_ids = tuple(int(value) for value in tagged.label_ids)
    active = tuple(index for index, label in enumerate(label_ids) if label != -100)
    if (
        active != selected
        or tuple(label_ids[index] for index in selected) != targets
        or tuple(input_ids[index] for index in selected)
        != tuple(int(value) for value in tagged.source.query_keys)
    ):
        raise ValueError("#1045 query positions or targets do not reproduce")
    return ZoologyMQARRow(
        input_ids=input_ids,
        selected_positions=selected,
        targets=targets,
        stable_id=str(tagged.stable_id),
        role_ids=role_ids,
    )


def load_exact_1045_population(source_root: Path) -> ZoologyMQARPopulation:
    """Adapt only #1045's open 8,192/1,024 split without changing row bytes."""

    construction = load_role_tagged_construction(source_root)
    if (
        len(construction.mqar_train) != EXACT_1045_TRAIN_ROWS
        or len(construction.mqar_development) != EXACT_1045_DEVELOPMENT_ROWS
    ):
        raise ValueError("#1045 open train/development split counts differ")
    train = tuple(_adapt_exact_row(row) for row in construction.mqar_train)
    development = tuple(_adapt_exact_row(row) for row in construction.mqar_development)
    return _make_population(
        train=train,
        development=development,
        name="exact_1045_open_bytes",
        vocab_size=EXACT_1045_VOCAB_SIZE,
        input_seq_len=EXACT_1045_CONTEXT,
        num_kv_pairs=EXACT_1045_QUERIES,
        source_split_cid=str(construction.split_cid),
    )


def batch_rows(
    rows: Sequence[ZoologyMQARRow],
    device: torch.device | str | None = None,
) -> ZoologyMQARBatch:
    """Create the complete three-tensor model ABI from uniform MQAR rows."""

    if not rows:
        raise ValueError("Zoology MQAR batch cannot be empty")
    widths = {len(row.input_ids) for row in rows}
    query_counts = {len(row.selected_positions) for row in rows}
    if len(widths) != 1 or len(query_counts) != 1:
        raise ValueError("one Zoology MQAR batch requires uniform row dimensions")
    return ZoologyMQARBatch(
        input_ids=torch.tensor([row.input_ids for row in rows], dtype=torch.long, device=device),
        selected_positions=torch.tensor(
            [row.selected_positions for row in rows], dtype=torch.long, device=device
        ),
        targets=torch.tensor([row.targets for row in rows], dtype=torch.long, device=device),
    )


def deterministic_epoch_order(
    rows: Sequence[ZoologyMQARRow],
    epoch: int,
    namespace: str | bytes,
) -> tuple[ZoologyMQARRow, ...]:
    """Return an explicit, replayable ``torch.randperm`` release-style order."""

    if not rows:
        raise ValueError("release-style shuffle cannot order an empty population")
    if isinstance(epoch, bool) or not isinstance(epoch, int) or epoch < 0:
        raise ValueError("shuffle epoch must be a nonnegative integer")
    if isinstance(namespace, str):
        namespace_bytes = namespace.encode("utf-8")
    elif isinstance(namespace, bytes):
        namespace_bytes = namespace
    else:
        raise TypeError("shuffle namespace must be str or bytes")
    if not namespace_bytes:
        raise ValueError("shuffle namespace cannot be empty")
    stable_ids = [row.stable_id for row in rows]
    if len(set(stable_ids)) != len(stable_ids):
        raise ValueError("release-style shuffle requires unique stable row IDs")

    digest = blake3(
        SHUFFLE_NAMESPACE
        + namespace_bytes
        + b"\0"
        + epoch.to_bytes(8, "big")
        + canonical_json_bytes(stable_ids)
    ).digest()
    seed = int.from_bytes(digest[:8], "big") & ((1 << 63) - 1)
    generator = torch.Generator(device="cpu")
    generator.manual_seed(seed)
    order = torch.randperm(len(rows), generator=generator).tolist()
    return tuple(rows[index] for index in order)


def permute_exact_bindings(
    rows: Sequence[ZoologyMQARRow],
) -> tuple[ZoologyMQARRow, ...]:
    """Rotate serialized #1045 values while retaining queries and targets.

    This is a data-level intervention.  It intentionally makes the physical
    bindings disagree with the retained labels, and it never exposes roles in
    :func:`batch_rows`.
    """

    if not rows:
        raise ValueError("exact binding permutation cannot be empty")
    controls: list[ZoologyMQARRow] = []
    for row in rows:
        if row.role_ids is None or derive_mqar_role_ids(row.input_ids) != row.role_ids:
            raise ValueError("exact binding permutation requires valid #1045 role bytes")
        value_positions = tuple(
            index for index, role in enumerate(row.role_ids) if role == int(RoleCode.VALUE)
        )
        if len(value_positions) != EXACT_1045_RECORDS:
            raise ValueError("exact binding permutation requires every physical value slot")
        native_values = tuple(row.input_ids[index] for index in value_positions)
        permuted_values = native_values[1:] + native_values[:1]
        if len(set(native_values)) != len(native_values) or any(
            native == permuted
            for native, permuted in zip(native_values, permuted_values, strict=True)
        ):
            raise ValueError("exact binding permutation is not a derangement")

        changed = list(row.input_ids)
        for position, value in zip(value_positions, permuted_values, strict=True):
            changed[position] = value
        changed_inputs = tuple(changed)
        if derive_mqar_role_ids(changed_inputs) != row.role_ids:
            raise ValueError("binding permutation changed the categorical role serialization")
        identity = {
            "intervention": "rotate_physical_values_left_one",
            "source_stable_id": row.stable_id,
            "input_ids": list(changed_inputs),
            "selected_positions": list(row.selected_positions),
            "targets": list(row.targets),
            "role_ids": list(row.role_ids),
        }
        controls.append(
            ZoologyMQARRow(
                input_ids=changed_inputs,
                selected_positions=row.selected_positions,
                targets=row.targets,
                stable_id=cid_bytes(PERMUTED_ROW_NAMESPACE + canonical_json_bytes(identity)),
                role_ids=row.role_ids,
            )
        )
    return tuple(controls)


__all__ = [
    "ZoologyMQARBatch",
    "ZoologyMQARPopulation",
    "ZoologyMQARRow",
    "batch_rows",
    "build_source_calibration",
    "deterministic_epoch_order",
    "load_exact_1045_population",
    "permute_exact_bindings",
]
