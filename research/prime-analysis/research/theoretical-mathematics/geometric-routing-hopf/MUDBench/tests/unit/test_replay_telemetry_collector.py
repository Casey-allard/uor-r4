from __future__ import annotations

from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.telemetry.collector import (
    ReplayTelemetryCollectionResult,
    collect_replay_telemetry,
)


def _envelope_payload(*, max_steps: int = 5) -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-telemetry-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-telemetry",
        "initial_seed": 77,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": max_steps,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
        },
    }


def _event_payload(
    *,
    step_index: int,
    event_type: str,
    actor_id: str | None,
    payload: dict[str, object] | None = None,
) -> dict[str, object]:
    return {
        "run_id": "run-telemetry-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload or {},
        "metadata": {},
    }


def _artifact(events: tuple[dict[str, object], ...], *, max_steps: int = 5) -> ReplayArtifact:
    emit_result = emit_replay_artifact(
        envelope=_envelope_payload(max_steps=max_steps),
        events=events,
    )
    assert emit_result.accepted is True
    assert emit_result.artifact is not None
    return emit_result.artifact


def test_collect_replay_telemetry_is_deterministic_for_typed_and_mapping_inputs() -> None:
    artifact = _artifact(
        (
            _event_payload(step_index=0, event_type="action_move", actor_id="agent-a"),
            _event_payload(step_index=0, event_type="observation_room", actor_id="agent-a"),
            _event_payload(step_index=1, event_type="action_rejected", actor_id="agent-b"),
            _event_payload(step_index=1, event_type="state_change_hp", actor_id="agent-b"),
            _event_payload(step_index=2, event_type="telemetry_custom", actor_id="agent-a"),
        )
    )

    typed_result = collect_replay_telemetry(artifact)
    mapping_result = collect_replay_telemetry(artifact.to_dict())
    repeated_result = collect_replay_telemetry(artifact)

    assert typed_result.accepted is True
    assert mapping_result.accepted is True
    assert repeated_result.accepted is True
    assert typed_result.snapshot is not None
    assert mapping_result.snapshot is not None
    assert repeated_result.snapshot is not None
    assert typed_result.snapshot == mapping_result.snapshot
    assert typed_result.snapshot.to_canonical_json() == repeated_result.snapshot.to_canonical_json()

    snapshot = typed_result.snapshot.to_dict()
    assert snapshot == {
        "run_id": "run-telemetry-1",
        "initial_seed": 77,
        "seed_source": "run_seed",
        "max_steps": 5,
        "event_count": 5,
        "step_event_counts": [
            {"step_index": 0, "count": 2},
            {"step_index": 1, "count": 2},
            {"step_index": 2, "count": 1},
        ],
        "event_type_counts": [
            {"event_type": "action_move", "count": 1},
            {"event_type": "action_rejected", "count": 1},
            {"event_type": "observation_room", "count": 1},
            {"event_type": "state_change_hp", "count": 1},
            {"event_type": "telemetry_custom", "count": 1},
        ],
        "category_counts": [
            {"category": "action", "count": 1},
            {"category": "rejection", "count": 1},
            {"category": "state_change", "count": 1},
            {"category": "observation", "count": 1},
            {"category": "unknown", "count": 1},
        ],
        "unknown_event_types": ["telemetry_custom"],
        "actor_metrics": [
            {
                "actor_id": "agent-a",
                "event_count": 3,
                "action_count": 1,
                "rejected_count": 0,
                "unknown_event_count": 1,
            },
            {
                "actor_id": "agent-b",
                "event_count": 2,
                "action_count": 0,
                "rejected_count": 1,
                "unknown_event_count": 0,
            },
        ],
    }


def test_collect_replay_telemetry_includes_envelope_actors_with_zero_counts() -> None:
    artifact = _artifact(())
    result = collect_replay_telemetry(artifact)

    assert result.accepted is True
    assert result.snapshot is not None
    assert result.snapshot.to_dict() == {
        "run_id": "run-telemetry-1",
        "initial_seed": 77,
        "seed_source": "run_seed",
        "max_steps": 5,
        "event_count": 0,
        "step_event_counts": [],
        "event_type_counts": [],
        "category_counts": [
            {"category": "action", "count": 0},
            {"category": "rejection", "count": 0},
            {"category": "state_change", "count": 0},
            {"category": "observation", "count": 0},
            {"category": "unknown", "count": 0},
        ],
        "unknown_event_types": [],
        "actor_metrics": [
            {
                "actor_id": "agent-a",
                "event_count": 0,
                "action_count": 0,
                "rejected_count": 0,
                "unknown_event_count": 0,
            },
            {
                "actor_id": "agent-b",
                "event_count": 0,
                "action_count": 0,
                "rejected_count": 0,
                "unknown_event_count": 0,
            },
        ],
    }


def test_collect_replay_telemetry_changes_when_event_stream_changes() -> None:
    first = _artifact((_event_payload(step_index=0, event_type="action_wait", actor_id="agent-a"),))
    second = _artifact((_event_payload(step_index=0, event_type="action_move", actor_id="agent-a"),))

    first_result = collect_replay_telemetry(first)
    second_result = collect_replay_telemetry(second)

    assert first_result.accepted is True
    assert second_result.accepted is True
    assert first_result.snapshot is not None
    assert second_result.snapshot is not None
    assert first_result.snapshot.to_canonical_json() != second_result.snapshot.to_canonical_json()


def test_collect_replay_telemetry_rejects_invalid_artifact_inputs() -> None:
    result = collect_replay_telemetry(4.2)
    assert result == ReplayTelemetryCollectionResult(
        accepted=False,
        reason="artifact_must_be_replay_artifact_or_mapping",
    )

    result = collect_replay_telemetry({"envelope": _envelope_payload()})
    assert result.accepted is False
    assert result.reason == "invalid_artifact:missing_required_fields:events"
    assert result.artifact_result is not None
    assert result.artifact_result.reason == "missing_required_fields:events"
