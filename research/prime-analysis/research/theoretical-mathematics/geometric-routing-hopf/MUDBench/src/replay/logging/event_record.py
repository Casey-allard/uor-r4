"""Deterministic replay event record schema and parsing."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

ReplayScalar = str | int | float | bool | None
_INVALID_VALUE = object()
REQUIRED_REPLAY_EVENT_FIELDS = ("run_id", "step_index", "event_type", "actor_id", "payload")


@dataclass(frozen=True, slots=True)
class ReplayEventRecord:
    """Canonical immutable replay event record."""

    run_id: str
    step_index: int
    event_type: str
    actor_id: str | None
    payload: tuple[tuple[str, ReplayScalar], ...]
    metadata: tuple[tuple[str, ReplayScalar], ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.step_index, int) or isinstance(self.step_index, bool) or self.step_index < 0:
            raise ValueError("step_index must be a non-negative integer")
        if not isinstance(self.event_type, str) or not self.event_type:
            raise ValueError("event_type must be a non-empty string")
        if self.actor_id is not None and (not isinstance(self.actor_id, str) or not self.actor_id):
            raise ValueError("actor_id must be None or a non-empty string")

        normalized_payload = _normalize_key_value_pairs(self.payload, field_name="payload")
        normalized_metadata = _normalize_key_value_pairs(self.metadata, field_name="metadata")
        object.__setattr__(self, "payload", normalized_payload)
        object.__setattr__(self, "metadata", normalized_metadata)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "ReplayEventRecord":
        result = parse_replay_event_record(payload)
        if not result.accepted or result.record is None:
            raise ValueError(f"Invalid replay event payload: {result.reason}")
        return result.record

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "step_index": self.step_index,
            "event_type": self.event_type,
            "actor_id": self.actor_id,
            "payload": {key: value for key, value in self.payload},
            "metadata": {key: value for key, value in self.metadata},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class ReplayEventParseResult:
    """Explicit accept/reject parse result for replay event records."""

    accepted: bool
    record: ReplayEventRecord | None = None
    reason: str | None = None


def parse_replay_event_record(payload: object) -> ReplayEventParseResult:
    """Parse replay event payload with explicit accept/reject semantics."""
    if not isinstance(payload, Mapping):
        return _rejected_result("payload_not_mapping")

    missing_fields = [field for field in REQUIRED_REPLAY_EVENT_FIELDS if field not in payload]
    if missing_fields:
        return _rejected_result(f"missing_required_fields:{','.join(missing_fields)}")

    run_id = payload.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        return _rejected_result("run_id_must_be_non_empty_string")

    step_index = payload.get("step_index")
    if not isinstance(step_index, int) or isinstance(step_index, bool) or step_index < 0:
        return _rejected_result("step_index_must_be_non_negative_integer")

    event_type = payload.get("event_type")
    if not isinstance(event_type, str) or not event_type:
        return _rejected_result("event_type_must_be_non_empty_string")

    actor_id = payload.get("actor_id")
    if actor_id is not None and (not isinstance(actor_id, str) or not actor_id):
        return _rejected_result("actor_id_must_be_none_or_non_empty_string")

    raw_payload = payload.get("payload")
    normalized_payload = _normalize_mapping(raw_payload, field_name="payload")
    if normalized_payload is _INVALID_VALUE:
        return _rejected_result("payload_must_be_mapping_with_scalar_values")

    raw_metadata = payload.get("metadata", {})
    normalized_metadata = _normalize_mapping(raw_metadata, field_name="metadata")
    if normalized_metadata is _INVALID_VALUE:
        return _rejected_result("metadata_must_be_mapping_with_scalar_values")

    return _accepted_result(
        ReplayEventRecord(
            run_id=run_id,
            step_index=step_index,
            event_type=event_type,
            actor_id=actor_id,
            payload=normalized_payload,  # type: ignore[arg-type]
            metadata=normalized_metadata,  # type: ignore[arg-type]
        )
    )


def _normalize_mapping(
    value: object,
    *,
    field_name: str,
) -> tuple[tuple[str, ReplayScalar], ...] | object:
    if not isinstance(value, Mapping):
        return _INVALID_VALUE
    normalized_pairs: list[tuple[str, ReplayScalar]] = []
    keys = tuple(value.keys())
    for key in keys:
        if not isinstance(key, str) or not key:
            return _INVALID_VALUE

    for key in sorted(keys):
        normalized_value = _normalize_scalar(value[key])
        if normalized_value is _INVALID_VALUE:
            return _INVALID_VALUE
        normalized_pairs.append((key, normalized_value))
    return tuple(normalized_pairs)


def _normalize_key_value_pairs(
    pairs: Sequence[tuple[str, ReplayScalar]],
    *,
    field_name: str,
) -> tuple[tuple[str, ReplayScalar], ...]:
    if isinstance(pairs, (str, bytes)) or not isinstance(pairs, Sequence):
        raise ValueError(f"{field_name} must be a sequence of key/value pairs")

    normalized_pairs: list[tuple[str, ReplayScalar]] = []
    seen_keys: set[str] = set()
    for pair in pairs:
        if not isinstance(pair, tuple) or len(pair) != 2:
            raise ValueError(f"{field_name} entries must be key/value tuples")
        key, raw_value = pair
        if not isinstance(key, str) or not key:
            raise ValueError(f"{field_name} keys must be non-empty strings")
        if key in seen_keys:
            raise ValueError(f"duplicate key in {field_name}: {key}")
        seen_keys.add(key)

        normalized_value = _normalize_scalar(raw_value)
        if normalized_value is _INVALID_VALUE:
            raise ValueError(f"{field_name} values must be JSON scalar values")
        normalized_pairs.append((key, normalized_value))

    normalized_pairs.sort(key=lambda item: item[0])
    return tuple(normalized_pairs)


def _normalize_scalar(value: Any) -> ReplayScalar | object:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            return _INVALID_VALUE
        return value
    return _INVALID_VALUE


def _accepted_result(record: ReplayEventRecord) -> ReplayEventParseResult:
    return ReplayEventParseResult(accepted=True, record=record)


def _rejected_result(reason: str) -> ReplayEventParseResult:
    return ReplayEventParseResult(accepted=False, reason=reason)
