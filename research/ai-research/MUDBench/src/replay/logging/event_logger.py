"""Deterministic append-only replay event log writer pipeline."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from replay.logging.event_record import (
    ReplayEventParseResult,
    ReplayEventRecord,
    parse_replay_event_record,
)


@dataclass(frozen=True, slots=True)
class EventLogWriteResult:
    """Explicit accept/reject outcome for one event write attempt."""

    accepted: bool
    record: ReplayEventRecord | None = None
    reason: str | None = None
    parse_result: ReplayEventParseResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.record is None:
                raise ValueError("accepted write result must include record")
            if self.reason is not None:
                raise ValueError("accepted write result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected write result must include reason")
        if self.record is not None:
            raise ValueError("rejected write result must not include record")


class DeterministicEventLogWriter:
    """Append-only writer that preserves event order exactly as received."""

    def __init__(self) -> None:
        self._events: list[ReplayEventRecord] = []

    def append_event(self, event: object) -> EventLogWriteResult:
        if isinstance(event, ReplayEventRecord):
            self._events.append(event)
            return _accepted_result(record=event)

        if not isinstance(event, Mapping):
            return _rejected_result(reason="event_must_be_replay_event_record_or_mapping")

        parse_result = parse_replay_event_record(event)
        if not parse_result.accepted or parse_result.record is None:
            parse_reason = parse_result.reason or "unknown_parse_rejection"
            return _rejected_result(
                reason=f"invalid_event_payload:{parse_reason}",
                parse_result=parse_result,
            )

        self._events.append(parse_result.record)
        return _accepted_result(record=parse_result.record, parse_result=parse_result)

    def append_events(self, events: Sequence[object]) -> tuple[EventLogWriteResult, ...]:
        if isinstance(events, (str, bytes)) or not isinstance(events, Sequence):
            raise ValueError("events must be a sequence of replay events")
        return tuple(self.append_event(event) for event in events)

    def snapshot(self) -> tuple[ReplayEventRecord, ...]:
        return tuple(self._events)

    def to_dict(self) -> dict[str, Any]:
        return {"events": [event.to_dict() for event in self._events]}

    def to_canonical_jsonl(self) -> str:
        return "\n".join(event.to_canonical_json() for event in self._events)

    def to_canonical_bytes(self) -> bytes:
        return self.to_canonical_jsonl().encode("utf-8")


def _accepted_result(
    *,
    record: ReplayEventRecord,
    parse_result: ReplayEventParseResult | None = None,
) -> EventLogWriteResult:
    return EventLogWriteResult(accepted=True, record=record, parse_result=parse_result)


def _rejected_result(
    *,
    reason: str,
    parse_result: ReplayEventParseResult | None = None,
) -> EventLogWriteResult:
    return EventLogWriteResult(accepted=False, reason=reason, parse_result=parse_result)
