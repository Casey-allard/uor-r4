"""Deterministic replay artifact emitter from envelope and event stream."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from replay.logging.event_record import (
    ReplayEventParseResult,
    ReplayEventRecord,
    parse_replay_event_record,
)
from replay.logging.replay_log_format import (
    ReplayLogEnvelope,
    ReplayLogEnvelopeParseResult,
    parse_replay_log_envelope,
)

REQUIRED_REPLAY_ARTIFACT_FIELDS = ("envelope", "events")
RECONSTRUCTION_STATE_SCHEMA_KEY = "reconstruction_state_schema"
RECONSTRUCTION_STATE_EVENT_TYPE_KEY = "reconstruction_state_event_type"
_DEFAULT_RECONSTRUCTION_STATE_EVENT_TYPE = "state_snapshot"


@dataclass(frozen=True, slots=True)
class ReplayArtifact:
    """Immutable replay artifact with explicit envelope/events boundaries."""

    envelope: ReplayLogEnvelope
    events: tuple[ReplayEventRecord, ...]

    def __post_init__(self) -> None:
        if not isinstance(self.envelope, ReplayLogEnvelope):
            raise ValueError("envelope must be a ReplayLogEnvelope")
        if isinstance(self.events, (str, bytes)) or not isinstance(self.events, Sequence):
            raise ValueError("events must be a sequence of ReplayEventRecord values")

        normalized_events: list[ReplayEventRecord] = []
        for index, event in enumerate(self.events):
            if not isinstance(event, ReplayEventRecord):
                raise ValueError("events must contain ReplayEventRecord values")
            if event.run_id != self.envelope.run_id:
                raise ValueError(
                    f"event run_id mismatch at index {index}: expected '{self.envelope.run_id}', got '{event.run_id}'"
                )
            normalized_events.append(event)
        object.__setattr__(self, "events", tuple(normalized_events))
        _validate_reconstruction_state_contract(
            envelope=self.envelope,
            events=tuple(normalized_events),
        )

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "ReplayArtifact":
        result = parse_replay_artifact(payload)
        if not result.accepted or result.artifact is None:
            raise ValueError(f"Invalid replay artifact payload: {result.reason}")
        return result.artifact

    def to_dict(self) -> dict[str, Any]:
        return {
            "envelope": self.envelope.to_dict(),
            "events": [event.to_dict() for event in self.events],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)

    def to_canonical_bytes(self) -> bytes:
        return self.to_canonical_json().encode("utf-8")


@dataclass(frozen=True, slots=True)
class ReplayArtifactEmitResult:
    """Explicit accept/reject result for replay artifact emission."""

    accepted: bool
    artifact: ReplayArtifact | None = None
    reason: str | None = None
    envelope_result: ReplayLogEnvelopeParseResult | None = None
    event_results: tuple[ReplayEventParseResult, ...] = ()

    def __post_init__(self) -> None:
        if self.accepted:
            if self.artifact is None:
                raise ValueError("accepted artifact result must include artifact")
            if self.reason is not None:
                raise ValueError("accepted artifact result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected artifact result must include reason")
        if self.artifact is not None:
            raise ValueError("rejected artifact result must not include artifact")


def emit_replay_artifact(*, envelope: object, events: object) -> ReplayArtifactEmitResult:
    resolved_envelope, envelope_result, envelope_reason = _resolve_envelope(envelope)
    if envelope_reason is not None:
        return _rejected_result(
            reason=envelope_reason,
            envelope_result=envelope_result,
        )

    resolved_events, event_results, events_reason = _resolve_events(events)
    if events_reason is not None:
        return _rejected_result(
            reason=events_reason,
            envelope_result=envelope_result,
            event_results=event_results,
        )

    if resolved_envelope is None or resolved_events is None:
        raise RuntimeError("internal error resolving replay artifact inputs")

    try:
        artifact = ReplayArtifact(envelope=resolved_envelope, events=resolved_events)
    except ValueError as exc:
        return _rejected_result(
            reason=f"artifact_validation_error:{exc}",
            envelope_result=envelope_result,
            event_results=event_results,
        )
    return _accepted_result(
        artifact=artifact,
        envelope_result=envelope_result,
        event_results=event_results,
    )


def parse_replay_artifact(payload: object) -> ReplayArtifactEmitResult:
    if not isinstance(payload, Mapping):
        return _rejected_result(reason="payload_not_mapping")

    missing_fields = [field for field in REQUIRED_REPLAY_ARTIFACT_FIELDS if field not in payload]
    if missing_fields:
        return _rejected_result(f"missing_required_fields:{','.join(missing_fields)}")

    return emit_replay_artifact(
        envelope=payload.get("envelope"),
        events=payload.get("events"),
    )


def _resolve_envelope(
    envelope: object,
) -> tuple[ReplayLogEnvelope | None, ReplayLogEnvelopeParseResult | None, str | None]:
    if isinstance(envelope, ReplayLogEnvelope):
        return envelope, None, None
    if isinstance(envelope, Mapping):
        parse_result = parse_replay_log_envelope(envelope)
        if not parse_result.accepted or parse_result.envelope is None:
            parse_reason = parse_result.reason or "unknown_parse_rejection"
            return None, parse_result, f"invalid_envelope:{parse_reason}"
        return parse_result.envelope, parse_result, None
    return None, None, "envelope_must_be_replay_log_envelope_or_mapping"


def _resolve_events(
    events: object,
) -> tuple[tuple[ReplayEventRecord, ...] | None, tuple[ReplayEventParseResult, ...], str | None]:
    if isinstance(events, (str, bytes)) or not isinstance(events, Sequence):
        return None, (), "events_must_be_sequence"

    resolved_events: list[ReplayEventRecord] = []
    parse_results: list[ReplayEventParseResult] = []
    for index, event in enumerate(events):
        if isinstance(event, ReplayEventRecord):
            parse_result = ReplayEventParseResult(accepted=True, record=event)
            parse_results.append(parse_result)
            resolved_events.append(event)
            continue
        if not isinstance(event, Mapping):
            return (
                None,
                tuple(parse_results),
                f"event_at_index:{index}:must_be_replay_event_record_or_mapping",
            )

        parse_result = parse_replay_event_record(event)
        parse_results.append(parse_result)
        if not parse_result.accepted or parse_result.record is None:
            parse_reason = parse_result.reason or "unknown_parse_rejection"
            return None, tuple(parse_results), f"invalid_event_at_index:{index}:{parse_reason}"
        resolved_events.append(parse_result.record)
    return tuple(resolved_events), tuple(parse_results), None


def _validate_reconstruction_state_contract(
    *,
    envelope: ReplayLogEnvelope,
    events: tuple[ReplayEventRecord, ...],
) -> None:
    run_metadata = {key: value for key, value in envelope.run_metadata}
    declared_schema = run_metadata.get(RECONSTRUCTION_STATE_SCHEMA_KEY)
    if declared_schema is None:
        return
    if not isinstance(declared_schema, str) or not declared_schema:
        raise ValueError(
            "run_metadata.reconstruction_state_schema must be a non-empty string when provided"
        )

    snapshot_event_type = run_metadata.get(
        RECONSTRUCTION_STATE_EVENT_TYPE_KEY,
        _DEFAULT_RECONSTRUCTION_STATE_EVENT_TYPE,
    )
    if not isinstance(snapshot_event_type, str) or not snapshot_event_type:
        raise ValueError(
            "run_metadata.reconstruction_state_event_type must be a non-empty string when provided"
        )

    found_snapshot = False
    for index, event in enumerate(events):
        if event.event_type != snapshot_event_type:
            continue

        payload = {key: value for key, value in event.payload}
        payload_schema = payload.get("state_schema")
        if payload_schema != declared_schema:
            raise ValueError(
                f"state snapshot schema mismatch at index {index}: "
                f"expected '{declared_schema}', got '{payload_schema}'"
            )

        state_json = payload.get("state_json")
        if not isinstance(state_json, str) or not state_json:
            raise ValueError(f"state snapshot missing state_json at index {index}")
        try:
            parsed = json.loads(state_json)
        except json.JSONDecodeError as exc:
            raise ValueError(f"state snapshot state_json invalid at index {index}") from exc
        if not isinstance(parsed, Mapping):
            raise ValueError(f"state snapshot state_json must decode to object at index {index}")
        canonical = json.dumps(parsed, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
        if canonical != state_json:
            raise ValueError(f"state snapshot state_json must be canonical JSON at index {index}")
        found_snapshot = True

    if not found_snapshot:
        raise ValueError(
            f"missing_state_snapshot_events_for_declared_schema:{declared_schema}"
        )


def _accepted_result(
    *,
    artifact: ReplayArtifact,
    envelope_result: ReplayLogEnvelopeParseResult | None = None,
    event_results: tuple[ReplayEventParseResult, ...] = (),
) -> ReplayArtifactEmitResult:
    return ReplayArtifactEmitResult(
        accepted=True,
        artifact=artifact,
        envelope_result=envelope_result,
        event_results=event_results,
    )


def _rejected_result(
    reason: str,
    *,
    envelope_result: ReplayLogEnvelopeParseResult | None = None,
    event_results: tuple[ReplayEventParseResult, ...] = (),
) -> ReplayArtifactEmitResult:
    return ReplayArtifactEmitResult(
        accepted=False,
        reason=reason,
        envelope_result=envelope_result,
        event_results=event_results,
    )
