"""Adapters for converting runtime core events into replay event records."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from core.event_logger import EventRecord
from replay.logging.event_record import (
    ReplayEventParseResult,
    ReplayEventRecord,
    ReplayScalar,
    parse_replay_event_record,
)

_INVALID = object()


@dataclass(frozen=True, slots=True)
class RuntimeEventAdaptResult:
    """Explicit accept/reject result for one runtime->replay event adaptation."""

    accepted: bool
    record: ReplayEventRecord | None = None
    reason: str | None = None
    parse_result: ReplayEventParseResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.record is None:
                raise ValueError("accepted adaptation result must include record")
            if self.reason is not None:
                raise ValueError("accepted adaptation result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected adaptation result must include reason")
        if self.record is not None:
            raise ValueError("rejected adaptation result must not include record")


@dataclass(frozen=True, slots=True)
class RuntimeEventBatchAdaptResult:
    """Explicit accept/reject result for adapting a runtime event sequence."""

    accepted: bool
    records: tuple[ReplayEventRecord, ...] = ()
    reason: str | None = None
    event_results: tuple[RuntimeEventAdaptResult, ...] = ()

    def __post_init__(self) -> None:
        if self.accepted:
            if self.reason is not None:
                raise ValueError("accepted batch adaptation result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected batch adaptation result must include reason")
        if self.records:
            raise ValueError("rejected batch adaptation result must not include records")


def adapt_runtime_event_to_replay(
    *,
    run_id: str,
    event: object,
) -> RuntimeEventAdaptResult:
    """Adapt one core EventRecord into one schema-validated ReplayEventRecord."""
    if not isinstance(run_id, str) or not run_id:
        return _rejected_event(reason="run_id_must_be_non_empty_string")
    if not isinstance(event, EventRecord):
        return _rejected_event(reason="event_must_be_core_event_record")

    payload, payload_reason = _normalize_payload_items(event.payload)
    if payload_reason is not None:
        return _rejected_event(reason=f"invalid_payload:{payload_reason}")

    parse_result = parse_replay_event_record(
        {
            "run_id": run_id,
            "step_index": event.step_index,
            "event_type": event.event_type,
            "actor_id": event.actor_id,
            "payload": payload,
            "metadata": {},
        }
    )
    if not parse_result.accepted or parse_result.record is None:
        parse_reason = parse_result.reason or "unknown_parse_rejection"
        return _rejected_event(
            reason=f"invalid_replay_event:{parse_reason}",
            parse_result=parse_result,
        )
    return _accepted_event(record=parse_result.record, parse_result=parse_result)


def adapt_runtime_events_to_replay(
    *,
    run_id: str,
    events: object,
) -> RuntimeEventBatchAdaptResult:
    """Adapt runtime event sequence into deterministic replay event sequence."""
    if not isinstance(run_id, str) or not run_id:
        return _rejected_batch(reason="run_id_must_be_non_empty_string")
    if isinstance(events, (str, bytes)) or not isinstance(events, Sequence):
        return _rejected_batch(reason="events_must_be_sequence")

    records: list[ReplayEventRecord] = []
    event_results: list[RuntimeEventAdaptResult] = []
    for index, event in enumerate(events):
        result = adapt_runtime_event_to_replay(run_id=run_id, event=event)
        event_results.append(result)
        if not result.accepted or result.record is None:
            reason = result.reason or "unknown_adaptation_failure"
            return _rejected_batch(
                reason=f"event_at_index:{index}:{reason}",
                event_results=tuple(event_results),
            )
        records.append(result.record)

    return RuntimeEventBatchAdaptResult(
        accepted=True,
        records=tuple(records),
        event_results=tuple(event_results),
    )


def _normalize_payload_items(
    payload_items: Sequence[tuple[str, Any]],
) -> tuple[dict[str, ReplayScalar], str | None]:
    if isinstance(payload_items, (str, bytes)) or not isinstance(payload_items, Sequence):
        return {}, "payload_must_be_sequence_of_pairs"

    payload: dict[str, ReplayScalar] = {}
    for pair in payload_items:
        if not isinstance(pair, tuple) or len(pair) != 2:
            return {}, "payload_pair_invalid"
        key, value = pair
        if not isinstance(key, str) or not key:
            return {}, "payload_key_invalid"
        normalized_value = _normalize_payload_value(value)
        if normalized_value is _INVALID:
            return {}, f"payload_value_invalid_for_key:{key}"
        payload[key] = normalized_value
    return payload, None


def _normalize_payload_value(value: Any) -> ReplayScalar | object:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        if math.isfinite(value):
            return value
        return _INVALID

    normalized_structured = _normalize_json_like(value)
    if normalized_structured is _INVALID:
        return _INVALID
    # Replay payload values are scalar-only; structured values are encoded as canonical JSON.
    return json.dumps(
        normalized_structured,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    )


def _normalize_json_like(value: Any) -> Any | object:
    if value is None or isinstance(value, (bool, str, int)):
        return value
    if isinstance(value, float):
        if math.isfinite(value):
            return value
        return _INVALID
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes)):
        normalized_items: list[Any] = []
        for item in value:
            normalized_item = _normalize_json_like(item)
            if normalized_item is _INVALID:
                return _INVALID
            normalized_items.append(normalized_item)
        return normalized_items
    if isinstance(value, Mapping):
        normalized_pairs: list[tuple[str, Any]] = []
        for raw_key, raw_value in value.items():
            if not isinstance(raw_key, str) or not raw_key:
                return _INVALID
            normalized_value = _normalize_json_like(raw_value)
            if normalized_value is _INVALID:
                return _INVALID
            normalized_pairs.append((raw_key, normalized_value))
        return {key: item for key, item in sorted(normalized_pairs, key=lambda pair: pair[0])}
    return _INVALID


def _accepted_event(
    *,
    record: ReplayEventRecord,
    parse_result: ReplayEventParseResult | None = None,
) -> RuntimeEventAdaptResult:
    return RuntimeEventAdaptResult(
        accepted=True,
        record=record,
        parse_result=parse_result,
    )


def _rejected_event(
    *,
    reason: str,
    parse_result: ReplayEventParseResult | None = None,
) -> RuntimeEventAdaptResult:
    return RuntimeEventAdaptResult(
        accepted=False,
        reason=reason,
        parse_result=parse_result,
    )


def _rejected_batch(
    *,
    reason: str,
    event_results: tuple[RuntimeEventAdaptResult, ...] = (),
) -> RuntimeEventBatchAdaptResult:
    return RuntimeEventBatchAdaptResult(
        accepted=False,
        reason=reason,
        event_results=event_results,
    )
