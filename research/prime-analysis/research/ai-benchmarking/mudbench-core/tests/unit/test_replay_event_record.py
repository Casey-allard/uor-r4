from __future__ import annotations

import pytest

from replay.logging.event_record import (
    ReplayEventParseResult,
    ReplayEventRecord,
    parse_replay_event_record,
)


def _valid_payload() -> dict[str, object]:
    return {
        "run_id": "run-1",
        "step_index": 2,
        "event_type": "action_move",
        "actor_id": "agent-a",
        "payload": {"direction": "east", "destination_room_id": "room-b"},
        "metadata": {"phase": "replay", "priority": 1, "success": True},
    }


def test_parse_replay_event_record_accepts_valid_payload_and_roundtrips() -> None:
    payload = _valid_payload()
    result = parse_replay_event_record(payload)

    assert result.accepted is True
    assert result.reason is None
    assert result.record == ReplayEventRecord(
        run_id="run-1",
        step_index=2,
        event_type="action_move",
        actor_id="agent-a",
        payload=(("destination_room_id", "room-b"), ("direction", "east")),
        metadata=(("phase", "replay"), ("priority", 1), ("success", True)),
    )
    assert result.record is not None
    assert result.record.to_dict() == {
        "run_id": "run-1",
        "step_index": 2,
        "event_type": "action_move",
        "actor_id": "agent-a",
        "payload": {"destination_room_id": "room-b", "direction": "east"},
        "metadata": {"phase": "replay", "priority": 1, "success": True},
    }

    reparsed = parse_replay_event_record(result.record.to_dict())
    assert reparsed == result


def test_parse_replay_event_record_rejects_non_mapping_payload() -> None:
    result = parse_replay_event_record(["not", "a", "mapping"])
    assert result == ReplayEventParseResult(accepted=False, reason="payload_not_mapping")


def test_parse_replay_event_record_rejects_missing_required_field() -> None:
    payload = _valid_payload()
    payload.pop("event_type")

    result = parse_replay_event_record(payload)
    assert result.accepted is False
    assert result.record is None
    assert result.reason == "missing_required_fields:event_type"


@pytest.mark.parametrize("bad_step_index", (-1, 1.5, True, "2"))
def test_parse_replay_event_record_rejects_invalid_step_index(bad_step_index: object) -> None:
    payload = _valid_payload()
    payload["step_index"] = bad_step_index

    result = parse_replay_event_record(payload)
    assert result.accepted is False
    assert result.reason == "step_index_must_be_non_negative_integer"


@pytest.mark.parametrize("bad_actor_id", ("", 12, False))
def test_parse_replay_event_record_rejects_invalid_actor_id(bad_actor_id: object) -> None:
    payload = _valid_payload()
    payload["actor_id"] = bad_actor_id

    result = parse_replay_event_record(payload)
    assert result.accepted is False
    assert result.reason == "actor_id_must_be_none_or_non_empty_string"


def test_parse_replay_event_record_rejects_invalid_payload_and_metadata_shapes() -> None:
    payload = _valid_payload()
    payload["payload"] = {"ok": 1, 2: "bad"}  # type: ignore[dict-item]
    result = parse_replay_event_record(payload)
    assert result.accepted is False
    assert result.reason == "payload_must_be_mapping_with_scalar_values"

    payload = _valid_payload()
    payload["metadata"] = {"ok": 1, "nested": {"bad": "value"}}
    result = parse_replay_event_record(payload)
    assert result.accepted is False
    assert result.reason == "metadata_must_be_mapping_with_scalar_values"


def test_replay_event_record_to_canonical_json_is_stable_for_key_order() -> None:
    first = ReplayEventRecord(
        run_id="run-9",
        step_index=0,
        event_type="action_wait",
        actor_id=None,
        payload=(("z", "last"), ("a", "first")),
        metadata=(("y", 2), ("x", 1)),
    )
    second = ReplayEventRecord(
        run_id="run-9",
        step_index=0,
        event_type="action_wait",
        actor_id=None,
        payload=(("a", "first"), ("z", "last")),
        metadata=(("x", 1), ("y", 2)),
    )

    assert first.payload == second.payload == (("a", "first"), ("z", "last"))
    assert first.metadata == second.metadata == (("x", 1), ("y", 2))
    assert first.to_canonical_json() == second.to_canonical_json()


def test_replay_event_record_from_dict_raises_explicit_reason() -> None:
    payload = _valid_payload()
    payload["run_id"] = ""

    with pytest.raises(ValueError, match="run_id_must_be_non_empty_string"):
        ReplayEventRecord.from_dict(payload)
