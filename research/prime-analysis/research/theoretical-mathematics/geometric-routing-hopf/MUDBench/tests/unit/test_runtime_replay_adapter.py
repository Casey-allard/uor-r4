from __future__ import annotations

from core.event_logger import EventRecord, normalize_payload
from replay.logging.runtime_adapter import adapt_runtime_events_to_replay


def test_adapt_runtime_events_to_replay_preserves_order_and_rejected_actions() -> None:
    events = (
        EventRecord(
            step_index=0,
            event_type="action_rejected",
            actor_id="agent-a",
            payload=normalize_payload({"action_type": "move", "reason": "blocked_exit"}),
        ),
        EventRecord(
            step_index=0,
            event_type="step_completed",
            payload=normalize_payload({"processed_actions": 1, "domain_event_count": 1}),
        ),
    )

    result = adapt_runtime_events_to_replay(run_id="run-adapter-1", events=events)

    assert result.accepted is True
    assert tuple(record.event_type for record in result.records) == (
        "action_rejected",
        "step_completed",
    )
    assert result.records[0].actor_id == "agent-a"
    assert dict(result.records[0].payload) == {
        "action_type": "move",
        "reason": "blocked_exit",
    }
    assert result.records[1].actor_id is None
    assert dict(result.records[1].payload) == {
        "domain_event_count": 1,
        "processed_actions": 1,
    }


def test_adapt_runtime_events_to_replay_serializes_structured_payload_values() -> None:
    events = (
        EventRecord(
            step_index=2,
            event_type="action_look",
            actor_id="agent-a",
            payload=normalize_payload({"visible_exits": ("east", "west")}),
        ),
    )

    result = adapt_runtime_events_to_replay(run_id="run-adapter-2", events=events)

    assert result.accepted is True
    assert dict(result.records[0].payload) == {"visible_exits": "[\"east\",\"west\"]"}


def test_adapt_runtime_events_to_replay_rejects_invalid_inputs_explicitly() -> None:
    result = adapt_runtime_events_to_replay(run_id="", events=())
    assert result.accepted is False
    assert result.reason == "run_id_must_be_non_empty_string"

    result = adapt_runtime_events_to_replay(run_id="run-adapter-3", events=3.14)
    assert result.accepted is False
    assert result.reason == "events_must_be_sequence"
