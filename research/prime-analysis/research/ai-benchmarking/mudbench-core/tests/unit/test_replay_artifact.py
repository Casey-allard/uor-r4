from __future__ import annotations

import json

import pytest

from replay.logging.replay_artifact import (
    ReplayArtifact,
    ReplayArtifactEmitResult,
    emit_replay_artifact,
    parse_replay_artifact,
)
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION, ReplayLogEnvelope


def _envelope_payload() -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-cavern",
        "initial_seed": 17,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": 10,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
            "priority": 1,
        },
    }


def _event_payload(*, step_index: int, event_type: str) -> dict[str, object]:
    return {
        "run_id": "run-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": "agent-a",
        "payload": {"index": step_index, "event": event_type},
        "metadata": {"valid": True},
    }


def _state_snapshot_event_payload(*, step_index: int = 0) -> dict[str, object]:
    state_payload = {
        "schema_version": "benchmark_runtime_state_v1",
        "run_id": "run-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-cavern",
        "step_index": step_index,
        "agent_states": [],
        "item_states": [],
        "npc_states": [],
        "room_states": [],
        "tracker_total_signals": 0,
    }
    return {
        "run_id": "run-1",
        "step_index": step_index,
        "event_type": "state_snapshot",
        "actor_id": None,
        "payload": {
            "state_schema": "benchmark_runtime_state_v1",
            "state_json": json.dumps(state_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True),
        },
        "metadata": {},
    }


def test_emit_replay_artifact_accepts_valid_inputs_and_roundtrips() -> None:
    envelope = ReplayLogEnvelope.from_mapping(_envelope_payload())
    event_one = _event_payload(step_index=0, event_type="action_wait")
    event_two = _event_payload(step_index=1, event_type="action_move")

    result = emit_replay_artifact(
        envelope=envelope,
        events=(event_one, event_two),
    )

    assert result.accepted is True
    assert result.reason is None
    assert result.artifact is not None
    assert result.artifact.to_dict() == {
        "envelope": envelope.to_dict(),
        "events": [event_one, event_two],
    }

    reparsed = parse_replay_artifact(result.artifact.to_dict())
    assert reparsed.accepted is True
    assert reparsed.artifact == result.artifact
    assert reparsed.artifact is not None
    assert reparsed.artifact.to_canonical_json() == result.artifact.to_canonical_json()


def test_emit_replay_artifact_canonical_output_is_identical_for_equivalent_inputs() -> None:
    result_a = emit_replay_artifact(
        envelope=_envelope_payload(),
        events=(
            {
                "run_id": "run-1",
                "step_index": 0,
                "event_type": "action_wait",
                "actor_id": "agent-a",
                "payload": {"b": 2, "a": 1},
                "metadata": {"y": True, "x": None},
            },
        ),
    )
    result_b = emit_replay_artifact(
        envelope={
            **_envelope_payload(),
            "actor_ids": ["agent-a", "agent-b"],
            "run_metadata": {
                "benchmark_version": "0.1",
                "scenario_version": "1.0",
                "scoring_version": "phase3-v1",
                "priority": 1,
                "phase": "phase4",
            },
        },
        events=(
            {
                "run_id": "run-1",
                "step_index": 0,
                "event_type": "action_wait",
                "actor_id": "agent-a",
                "payload": {"a": 1, "b": 2},
                "metadata": {"x": None, "y": True},
            },
        ),
    )

    assert result_a.accepted is True
    assert result_b.accepted is True
    assert result_a.artifact is not None
    assert result_b.artifact is not None
    assert result_a.artifact.to_canonical_bytes() == result_b.artifact.to_canonical_bytes()


def test_emit_replay_artifact_changes_output_when_event_order_changes() -> None:
    first = _event_payload(step_index=0, event_type="action_wait")
    second = _event_payload(step_index=1, event_type="action_move")
    result_a = emit_replay_artifact(envelope=_envelope_payload(), events=(first, second))
    result_b = emit_replay_artifact(envelope=_envelope_payload(), events=(second, first))

    assert result_a.accepted is True
    assert result_b.accepted is True
    assert result_a.artifact is not None
    assert result_b.artifact is not None
    assert result_a.artifact.to_canonical_json() != result_b.artifact.to_canonical_json()


def test_emit_replay_artifact_rejects_invalid_envelope_with_explicit_reason() -> None:
    bad_envelope = _envelope_payload()
    bad_envelope["seed_source"] = "manual"

    result = emit_replay_artifact(
        envelope=bad_envelope,
        events=(),
    )

    assert result.accepted is False
    assert result.reason == "invalid_envelope:seed_source_must_be_one_of:seed_override,run_seed,scenario_seed,derived"
    assert result.envelope_result is not None
    assert result.envelope_result.reason == "seed_source_must_be_one_of:seed_override,run_seed,scenario_seed,derived"


def test_emit_replay_artifact_rejects_invalid_event_with_indexed_reason() -> None:
    bad_event = _event_payload(step_index=0, event_type="action_wait")
    bad_event.pop("event_type")

    result = emit_replay_artifact(
        envelope=_envelope_payload(),
        events=(bad_event,),
    )

    assert result.accepted is False
    assert result.reason == "invalid_event_at_index:0:missing_required_fields:event_type"
    assert len(result.event_results) == 1
    assert result.event_results[0].accepted is False
    assert result.event_results[0].reason == "missing_required_fields:event_type"


def test_emit_replay_artifact_rejects_event_run_id_mismatch() -> None:
    mismatched_event = _event_payload(step_index=0, event_type="action_wait")
    mismatched_event["run_id"] = "run-other"

    result = emit_replay_artifact(
        envelope=_envelope_payload(),
        events=(mismatched_event,),
    )

    assert result.accepted is False
    assert (
        result.reason
        == "artifact_validation_error:event run_id mismatch at index 0: expected 'run-1', got 'run-other'"
    )


def test_parse_replay_artifact_rejects_non_mapping_and_missing_fields() -> None:
    result = parse_replay_artifact(["not", "a", "mapping"])
    assert result == ReplayArtifactEmitResult(accepted=False, reason="payload_not_mapping")

    result = parse_replay_artifact({"envelope": _envelope_payload()})
    assert result.accepted is False
    assert result.reason == "missing_required_fields:events"


def test_replay_artifact_from_mapping_raises_explicit_reason() -> None:
    payload = {
        "envelope": {**_envelope_payload(), "run_id": ""},
        "events": (),
    }

    with pytest.raises(ValueError, match="invalid_envelope:run_id_must_be_non_empty_string"):
        ReplayArtifact.from_mapping(payload)


def test_emit_replay_artifact_accepts_declared_reconstruction_schema_with_snapshot_events() -> None:
    envelope = _envelope_payload()
    envelope["run_metadata"] = {
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
        "phase": "phase4",
        "priority": 1,
        "reconstruction_state_schema": "benchmark_runtime_state_v1",
        "reconstruction_state_event_type": "state_snapshot",
    }

    result = emit_replay_artifact(
        envelope=envelope,
        events=(_state_snapshot_event_payload(step_index=0),),
    )

    assert result.accepted is True
    assert result.artifact is not None


def test_emit_replay_artifact_rejects_declared_reconstruction_schema_without_snapshots() -> None:
    envelope = _envelope_payload()
    envelope["run_metadata"] = {
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
        "phase": "phase4",
        "priority": 1,
        "reconstruction_state_schema": "benchmark_runtime_state_v1",
    }

    result = emit_replay_artifact(
        envelope=envelope,
        events=(_event_payload(step_index=0, event_type="step_completed"),),
    )

    assert result.accepted is False
    assert (
        result.reason
        == "artifact_validation_error:missing_state_snapshot_events_for_declared_schema:benchmark_runtime_state_v1"
    )
