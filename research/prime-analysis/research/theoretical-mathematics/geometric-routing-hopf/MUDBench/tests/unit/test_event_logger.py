from __future__ import annotations

from copy import deepcopy

import pytest

from replay.logging.event_logger import DeterministicEventLogWriter, EventLogWriteResult
from replay.logging.event_record import ReplayEventRecord


def _event_payload(
    *,
    step_index: int,
    event_type: str = "action_wait",
    payload: dict[str, object] | None = None,
    metadata: dict[str, object] | None = None,
) -> dict[str, object]:
    return {
        "run_id": "run-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": "agent-a",
        "payload": payload or {"value": step_index},
        "metadata": metadata or {},
    }


def test_append_event_accepts_record_and_preserves_append_order() -> None:
    writer = DeterministicEventLogWriter()
    first = ReplayEventRecord.from_dict(_event_payload(step_index=2, event_type="action_move"))
    second = ReplayEventRecord.from_dict(_event_payload(step_index=1, event_type="action_pickup"))

    assert writer.append_event(first) == EventLogWriteResult(accepted=True, record=first)
    second_result = writer.append_event(second)
    assert second_result.accepted is True
    assert second_result.record == second

    assert writer.snapshot() == (first, second)


def test_append_event_from_mapping_does_not_mutate_source_payload() -> None:
    writer = DeterministicEventLogWriter()
    payload = _event_payload(
        step_index=3,
        payload={"z": "last", "a": "first"},
        metadata={"m2": 2, "m1": 1},
    )
    before = deepcopy(payload)

    result = writer.append_event(payload)

    assert result.accepted is True
    assert payload == before
    assert writer.snapshot() == (ReplayEventRecord.from_dict(before),)


def test_append_event_rejects_out_of_contract_event_input() -> None:
    writer = DeterministicEventLogWriter()

    result = writer.append_event(1.25)

    assert result == EventLogWriteResult(
        accepted=False,
        reason="event_must_be_replay_event_record_or_mapping",
    )
    assert writer.snapshot() == ()


def test_append_event_rejects_invalid_mapping_with_explicit_reason() -> None:
    writer = DeterministicEventLogWriter()
    payload = _event_payload(step_index=0)
    payload.pop("event_type")

    result = writer.append_event(payload)

    assert result.accepted is False
    assert result.record is None
    assert result.reason == "invalid_event_payload:missing_required_fields:event_type"
    assert result.parse_result is not None
    assert result.parse_result.reason == "missing_required_fields:event_type"
    assert writer.snapshot() == ()


def test_append_events_rejects_non_sequence_input() -> None:
    writer = DeterministicEventLogWriter()

    with pytest.raises(ValueError, match="events must be a sequence of replay events"):
        writer.append_events("not-a-sequence")
    with pytest.raises(ValueError, match="events must be a sequence of replay events"):
        writer.append_events(1)  # type: ignore[arg-type]


def test_append_events_processes_inputs_in_order_and_continues_after_rejection() -> None:
    writer = DeterministicEventLogWriter()
    valid_1 = _event_payload(step_index=0, event_type="action_wait")
    invalid = {"run_id": "run-1"}
    valid_2 = _event_payload(step_index=2, event_type="action_move")

    results = writer.append_events((valid_1, invalid, valid_2))

    assert tuple(result.accepted for result in results) == (True, False, True)
    assert writer.snapshot() == (
        ReplayEventRecord.from_dict(valid_1),
        ReplayEventRecord.from_dict(valid_2),
    )


def test_canonical_output_is_byte_identical_for_equivalent_ordered_events() -> None:
    writer_a = DeterministicEventLogWriter()
    writer_b = DeterministicEventLogWriter()

    writer_a.append_events(
        (
            _event_payload(
                step_index=0,
                payload={"b": 2, "a": 1},
                metadata={"y": True, "x": None},
            ),
            _event_payload(step_index=1, payload={"key": "value"}),
        )
    )
    writer_b.append_events(
        (
            _event_payload(
                step_index=0,
                payload={"a": 1, "b": 2},
                metadata={"x": None, "y": True},
            ),
            _event_payload(step_index=1, payload={"key": "value"}),
        )
    )

    assert writer_a.to_canonical_jsonl() == writer_b.to_canonical_jsonl()
    assert writer_a.to_canonical_bytes() == writer_b.to_canonical_bytes()


def test_canonical_output_changes_when_append_order_changes() -> None:
    writer_a = DeterministicEventLogWriter()
    writer_b = DeterministicEventLogWriter()
    first = _event_payload(step_index=0, event_type="action_wait")
    second = _event_payload(step_index=1, event_type="action_move")

    writer_a.append_events((first, second))
    writer_b.append_events((second, first))

    assert writer_a.to_canonical_jsonl() != writer_b.to_canonical_jsonl()
