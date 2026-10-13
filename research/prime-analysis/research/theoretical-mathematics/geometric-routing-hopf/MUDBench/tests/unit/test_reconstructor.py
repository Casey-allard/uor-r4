from __future__ import annotations

import json

from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.reconstruction.reconstructor import (
    ReplayReconstructionResult,
    reconstruct_replay_state,
)


def _envelope_payload(*, max_steps: int = 4) -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-reconstruct-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-reconstruction",
        "initial_seed": 21,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": max_steps,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
            "purpose": "reconstruction-test",
        },
    }


def _event_payload(
    *,
    step_index: int,
    event_type: str,
    actor_id: str | None,
    payload: dict[str, object],
) -> dict[str, object]:
    return {
        "run_id": "run-reconstruct-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload,
        "metadata": {},
    }


def _artifact(
    events: tuple[dict[str, object], ...],
    *,
    max_steps: int = 4,
    envelope: dict[str, object] | None = None,
) -> ReplayArtifact:
    emit_result = emit_replay_artifact(
        envelope=envelope if envelope is not None else _envelope_payload(max_steps=max_steps),
        events=events,
    )
    assert emit_result.accepted is True
    assert emit_result.artifact is not None
    return emit_result.artifact


def test_reconstruct_replay_state_deterministic_roundtrip_for_typed_and_mapping_inputs() -> None:
    artifact = _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="action_move",
                actor_id="agent-a",
                payload={"source_room_id": "room-1", "destination_room_id": "room-2"},
            ),
            _event_payload(
                step_index=0,
                event_type="action_look",
                actor_id="agent-b",
                payload={"location": "room-1"},
            ),
            _event_payload(
                step_index=1,
                event_type="action_take",
                actor_id="agent-a",
                payload={"item_id": "item-key"},
            ),
            _event_payload(
                step_index=2,
                event_type="action_attack",
                actor_id="agent-a",
                payload={"target_id": "agent-b", "resulting_health": 7, "damage": 2},
            ),
            _event_payload(
                step_index=2,
                event_type="action_rejected",
                actor_id="agent-b",
                payload={"action_type": "move", "reason": "blocked_exit"},
            ),
        )
    )

    typed_result = reconstruct_replay_state(artifact)
    mapping_result = reconstruct_replay_state(artifact.to_dict())
    repeated_result = reconstruct_replay_state(artifact)

    assert typed_result.accepted is True
    assert mapping_result.accepted is True
    assert repeated_result.accepted is True
    assert typed_result.reconstructed_state is not None
    assert mapping_result.reconstructed_state is not None
    assert repeated_result.reconstructed_state is not None
    assert typed_result.reconstructed_state == mapping_result.reconstructed_state
    assert typed_result.reconstructed_state.to_canonical_json() == repeated_result.reconstructed_state.to_canonical_json()

    reconstructed = typed_result.reconstructed_state.to_dict()
    assert reconstructed["run_id"] == "run-reconstruct-1"
    assert reconstructed["event_count"] == 5
    assert reconstructed["terminal_step"] == 2
    assert reconstructed["applied_steps"] == [0, 1, 2]
    assert reconstructed["event_type_counts"] == [
        {"event_type": "action_attack", "count": 1},
        {"event_type": "action_look", "count": 1},
        {"event_type": "action_move", "count": 1},
        {"event_type": "action_rejected", "count": 1},
        {"event_type": "action_take", "count": 1},
    ]
    assert reconstructed["entities"] == [
        {
            "entity_id": "agent-a",
            "location": "room-2",
            "health": None,
            "inventory": ["item-key"],
            "last_event_type": "action_attack",
        },
        {
            "entity_id": "agent-b",
            "location": "room-1",
            "health": 7,
            "inventory": [],
            "last_event_type": "action_rejected",
        },
    ]
    assert reconstructed["rejected_actions"] == [
        {
            "step_index": 2,
            "actor_id": "agent-b",
            "action_type": "move",
            "reason": "blocked_exit",
        }
    ]


def test_reconstruct_replay_state_accepts_sparse_steps_with_explicit_step_completed_markers() -> None:
    artifact = _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="action_wait",
                actor_id="agent-a",
                payload={"result": "no_state_change"},
            ),
            _event_payload(
                step_index=0,
                event_type="step_completed",
                actor_id=None,
                payload={"processed_actions": 1, "domain_event_count": 1},
            ),
            _event_payload(
                step_index=1,
                event_type="step_completed",
                actor_id=None,
                payload={"processed_actions": 0, "domain_event_count": 0},
            ),
            _event_payload(
                step_index=2,
                event_type="action_wait",
                actor_id="agent-b",
                payload={"result": "no_state_change"},
            ),
            _event_payload(
                step_index=2,
                event_type="step_completed",
                actor_id=None,
                payload={"processed_actions": 1, "domain_event_count": 1},
            ),
        )
    )

    result = reconstruct_replay_state(artifact)
    repeated = reconstruct_replay_state(artifact)

    assert result.accepted is True
    assert repeated.accepted is True
    assert result.reconstructed_state is not None
    assert repeated.reconstructed_state is not None
    assert result.reconstructed_state == repeated.reconstructed_state
    assert result.reconstructed_state.applied_steps == (0, 1, 2)
    assert result.reconstructed_state.terminal_step == 2
    assert ("step_completed", 3) in result.reconstructed_state.event_type_counts


def test_reconstruct_replay_state_uses_declared_canonical_state_snapshot() -> None:
    envelope = _envelope_payload(max_steps=1)
    envelope["run_metadata"] = {
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
        "phase": "phase4",
        "purpose": "reconstruction-test",
        "reconstruction_state_schema": "benchmark_runtime_state_v1",
        "reconstruction_state_event_type": "state_snapshot",
    }
    state_payload = {
        "schema_version": "benchmark_runtime_state_v1",
        "run_id": "run-reconstruct-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-reconstruction",
        "step_index": 0,
        "agent_states": [
            {
                "actor_id": "agent-a",
                "metrics": [
                    {
                        "metric_name": "actions.count",
                        "sample_count": 1,
                        "value_sum": 1,
                        "min_value": 1,
                        "max_value": 1,
                        "last_step": 0,
                        "last_value": 1,
                    }
                ],
            }
        ],
        "item_states": [],
        "npc_states": [],
        "room_states": [],
        "tracker_total_signals": 1,
    }
    state_json = json.dumps(state_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    artifact = _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="state_snapshot",
                actor_id=None,
                payload={
                    "state_schema": "benchmark_runtime_state_v1",
                    "state_json": state_json,
                },
            ),
            _event_payload(
                step_index=0,
                event_type="step_completed",
                actor_id=None,
                payload={"processed_actions": 0, "domain_event_count": 0},
            ),
        ),
        max_steps=1,
        envelope=envelope,
    )

    result = reconstruct_replay_state(artifact)

    assert result.accepted is True
    assert result.reconstructed_state is not None
    reconstructed = result.reconstructed_state.to_dict()
    assert reconstructed["terminal_step"] == 0
    assert reconstructed["canonical_state"] == state_payload


def test_reconstruct_replay_state_rejects_invalid_declared_canonical_state_snapshot() -> None:
    envelope = _envelope_payload(max_steps=1)
    envelope["run_metadata"] = {
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
        "phase": "phase4",
        "purpose": "reconstruction-test",
        "reconstruction_state_schema": "benchmark_runtime_state_v1",
    }
    invalid_state_payload = {
        "schema_version": "benchmark_runtime_state_v1",
        "run_id": "run-reconstruct-1",
        "step_index": 0,
    }
    invalid_state_json = json.dumps(
        invalid_state_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    )
    artifact = _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="state_snapshot",
                actor_id=None,
                payload={
                    "state_schema": "benchmark_runtime_state_v1",
                    "state_json": invalid_state_json,
                },
            ),
            _event_payload(
                step_index=0,
                event_type="step_completed",
                actor_id=None,
                payload={"processed_actions": 0, "domain_event_count": 0},
            ),
        ),
        max_steps=1,
        envelope=envelope,
    )

    result = reconstruct_replay_state(artifact)

    assert result.accepted is False
    assert (
        result.reason
        == "unreconstructable_event_stream:invalid_state_snapshot_at_index:0:"
        "missing_required_fields:benchmark_id,scenario_id,agent_states,item_states,npc_states,room_states,tracker_total_signals"
    )


def test_reconstruct_replay_state_accepts_empty_event_stream() -> None:
    artifact = _artifact((), max_steps=3)
    result = reconstruct_replay_state(artifact)

    assert result.accepted is True
    assert result.reconstructed_state is not None
    assert result.reconstructed_state.to_dict() == {
        "run_id": "run-reconstruct-1",
        "initial_seed": 21,
        "seed_source": "run_seed",
        "max_steps": 3,
        "terminal_step": None,
        "event_count": 0,
        "applied_steps": [],
        "entities": [
            {
                "entity_id": "agent-a",
                "location": None,
                "health": None,
                "inventory": [],
                "last_event_type": None,
            },
            {
                "entity_id": "agent-b",
                "location": None,
                "health": None,
                "inventory": [],
                "last_event_type": None,
            },
        ],
        "event_type_counts": [],
        "rejected_actions": [],
    }


def test_reconstruct_replay_state_rejects_step_regression() -> None:
    artifact = _artifact(
        (
            _event_payload(step_index=0, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
            _event_payload(step_index=1, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
            _event_payload(step_index=0, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
        )
    )

    result = reconstruct_replay_state(artifact)
    assert result.accepted is False
    assert result.reason == "unreconstructable_event_stream:step_regression_at_index:2:previous_step:1:step:0"


def test_reconstruct_replay_state_rejects_step_gap() -> None:
    artifact = _artifact(
        (
            _event_payload(step_index=0, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
            _event_payload(step_index=2, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
        )
    )

    result = reconstruct_replay_state(artifact)
    assert result.accepted is False
    assert result.reason == "unreconstructable_event_stream:step_gap_at_index:1:expected_step:1:step:2"


def test_reconstruct_replay_state_rejects_step_overflow() -> None:
    artifact = _artifact(
        (
            _event_payload(step_index=0, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
            _event_payload(step_index=2, event_type="action_wait", actor_id="agent-a", payload={"result": "ok"}),
        ),
        max_steps=1,
    )

    result = reconstruct_replay_state(artifact)
    assert result.accepted is False
    assert result.reason == "unreconstructable_event_stream:step_overflow_at_index:1:step:2:max_steps:1"


def test_reconstruct_replay_state_rejects_invalid_artifact_input() -> None:
    result = reconstruct_replay_state(9.5)
    assert result == ReplayReconstructionResult(
        accepted=False,
        reason="artifact_must_be_replay_artifact_or_mapping",
    )

    result = reconstruct_replay_state({"envelope": _envelope_payload()})
    assert result.accepted is False
    assert result.reason == "invalid_artifact:missing_required_fields:events"
