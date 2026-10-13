from __future__ import annotations

import json

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle
from replay.integrity.replay_verifier import (
    compute_replay_parity_artifact,
    verify_replay_reexecution_parity,
    ReplayReconstructionVerificationResult,
    verify_replay_reconstruction,
)
from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.reconstruction.reconstructor import reconstruct_replay_state


def _envelope_payload(*, max_steps: int = 4) -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-verify-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-verifier",
        "initial_seed": 41,
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
    payload: dict[str, object],
) -> dict[str, object]:
    return {
        "run_id": "run-verify-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload,
        "metadata": {},
    }


def _artifact(events: tuple[dict[str, object], ...], *, max_steps: int = 4) -> ReplayArtifact:
    emit_result = emit_replay_artifact(
        envelope=_envelope_payload(max_steps=max_steps),
        events=events,
    )
    assert emit_result.accepted is True
    assert emit_result.artifact is not None
    return emit_result.artifact


def _parity_runner_config(*, max_steps_override: int | None = None) -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="parity-verify-run",
        benchmark_id="phase4-benchmark",
        scenario={
            "scenario_id": "parity-verify-scenario",
            "title": "Parity Verification Scenario",
            "description": "Deterministic replay parity verification scenario.",
            "start_room_id": "room-start",
            "max_steps": 3,
            "seed": 73,
            "version": "1.0",
            "scenario_vars": {"mode": "parity"},
            "objectives": [
                {
                    "objective_id": "obj-parity",
                    "objective_type": "collect_item",
                    "target_id": "item-key",
                    "required_count": 1,
                }
            ],
        },
        actor_ids=("agent-a", "agent-b"),
        run_seed=11,
        max_steps_override=max_steps_override,
    )


def _compute_runtime_parity():
    result = run_benchmark_lifecycle(_parity_runner_config())
    parity_result = compute_replay_parity_artifact(
        replay_artifact=result.replay_artifact,
        scorecard=result.scorecard,
    )
    assert parity_result.accepted is True
    assert parity_result.parity_artifact is not None
    return result, parity_result.parity_artifact


def _baseline_artifact() -> ReplayArtifact:
    return _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="action_move",
                actor_id="agent-a",
                payload={"source_room_id": "room-1", "destination_room_id": "room-2"},
            ),
            _event_payload(
                step_index=1,
                event_type="action_take",
                actor_id="agent-a",
                payload={"item_id": "item-key"},
            ),
            _event_payload(
                step_index=1,
                event_type="action_attack",
                actor_id="agent-a",
                payload={"target_id": "agent-b", "resulting_health": 8},
            ),
        )
    )


def test_verify_replay_reconstruction_matches_for_typed_and_mapping_expected_states() -> None:
    artifact = _baseline_artifact()
    reconstructed = reconstruct_replay_state(artifact)
    assert reconstructed.accepted is True
    assert reconstructed.reconstructed_state is not None
    expected_state = reconstructed.reconstructed_state

    typed_result = verify_replay_reconstruction(
        artifact=artifact,
        expected_terminal_state=expected_state,
    )
    mapping_result = verify_replay_reconstruction(
        artifact=artifact.to_dict(),
        expected_terminal_state=expected_state.to_dict(),
    )

    assert typed_result.accepted is True
    assert typed_result.matched is True
    assert typed_result.mismatches == ()
    assert typed_result.actual_state == expected_state
    assert typed_result.expected_state == expected_state

    assert mapping_result.accepted is True
    assert mapping_result.matched is True
    assert mapping_result.mismatches == ()
    assert mapping_result.actual_state == expected_state
    assert mapping_result.expected_state == expected_state


def test_verify_replay_reconstruction_reports_deterministic_mismatch_paths() -> None:
    artifact = _baseline_artifact()
    reconstructed = reconstruct_replay_state(artifact)
    assert reconstructed.reconstructed_state is not None
    expected_payload = reconstructed.reconstructed_state.to_dict()
    expected_payload["entities"][0]["location"] = "room-x"  # type: ignore[index]

    first = verify_replay_reconstruction(
        artifact=artifact,
        expected_terminal_state=expected_payload,
    )
    second = verify_replay_reconstruction(
        artifact=artifact,
        expected_terminal_state=expected_payload,
    )

    assert first.accepted is True
    assert first.matched is False
    assert first.mismatches == second.mismatches
    assert first.mismatches == ('$.entities[0].location:actual="room-2":expected="room-x"',)


def test_verify_replay_reconstruction_rejects_invalid_expected_state_type() -> None:
    result = verify_replay_reconstruction(
        artifact=_baseline_artifact(),
        expected_terminal_state=5,
    )
    assert result == ReplayReconstructionVerificationResult(
        accepted=False,
        reason="invalid_expected_state:expected_state_must_be_reconstructed_state_or_mapping",
    )


def test_verify_replay_reconstruction_rejects_invalid_expected_state_mapping() -> None:
    artifact = _baseline_artifact()
    reconstructed = reconstruct_replay_state(artifact)
    assert reconstructed.reconstructed_state is not None
    expected_payload = reconstructed.reconstructed_state.to_dict()
    expected_payload.pop("event_count")

    result = verify_replay_reconstruction(
        artifact=artifact,
        expected_terminal_state=expected_payload,
    )

    assert result.accepted is False
    assert result.reason == "invalid_expected_state:missing_required_fields:event_count"


def test_verify_replay_reconstruction_rejects_when_reconstruction_fails() -> None:
    expected_result = reconstruct_replay_state(_baseline_artifact())
    assert expected_result.reconstructed_state is not None

    broken_artifact = _artifact(
        (
            _event_payload(
                step_index=0,
                event_type="action_wait",
                actor_id="agent-a",
                payload={"result": "ok"},
            ),
            _event_payload(
                step_index=2,
                event_type="action_wait",
                actor_id="agent-a",
                payload={"result": "ok"},
            ),
        )
    )
    result = verify_replay_reconstruction(
        artifact=broken_artifact,
        expected_terminal_state=expected_result.reconstructed_state,
    )

    assert result.accepted is False
    assert result.reason == "cannot_reconstruct:unreconstructable_event_stream:step_gap_at_index:1:expected_step:1:step:2"
    assert result.reconstruction_result is not None
    assert result.reconstruction_result.accepted is False


def test_compute_replay_parity_artifact_is_stable_for_identical_inputs() -> None:
    result, typed_parity = _compute_runtime_parity()
    mapping_parity = compute_replay_parity_artifact(
        replay_artifact=result.replay_artifact.to_dict(),
        scorecard=result.scorecard.to_dict(),
    )
    assert mapping_parity.accepted is True
    assert mapping_parity.parity_artifact is not None

    assert typed_parity.to_canonical_json() == mapping_parity.parity_artifact.to_canonical_json()
    assert len(typed_parity.terminal_state_hash) == 64
    assert typed_parity.step_count == result.lifecycle_state.step_index
    assert typed_parity.scorecard_step_count == result.scorecard.metadata.step_count
    assert typed_parity.aggregate_score == result.scorecard.aggregate_score


def test_verify_replay_reexecution_parity_matches_on_runtime_artifacts() -> None:
    result, expected_parity = _compute_runtime_parity()

    verification = verify_replay_reexecution_parity(
        replay_artifact=result.replay_artifact.to_dict(),
        scorecard=result.scorecard.to_dict(),
        expected_parity_artifact=expected_parity.to_dict(),
    )

    assert verification.accepted is True
    assert verification.matched is True
    assert verification.mismatches == ()
    assert verification.actual_parity == expected_parity
    assert verification.expected_parity == expected_parity


def test_verify_replay_reexecution_parity_detects_terminal_state_divergence() -> None:
    result, expected_parity = _compute_runtime_parity()
    mutated_artifact = result.replay_artifact.to_dict()
    events = mutated_artifact["events"]  # type: ignore[index]
    mutated_snapshot_event = None
    for event in reversed(events):
        if isinstance(event, dict) and event.get("event_type") == "state_snapshot":
            mutated_snapshot_event = event
            break
    assert mutated_snapshot_event is not None
    state_payload = json.loads(mutated_snapshot_event["payload"]["state_json"])  # type: ignore[index]
    state_payload["tracker_total_signals"] = state_payload["tracker_total_signals"] + 1  # type: ignore[index]
    mutated_snapshot_event["payload"]["state_json"] = json.dumps(  # type: ignore[index]
        state_payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    )

    verification = verify_replay_reexecution_parity(
        replay_artifact=mutated_artifact,
        scorecard=result.scorecard,
        expected_parity_artifact=expected_parity,
    )

    assert verification.accepted is True
    assert verification.matched is False
    assert any(path.startswith("$.terminal_state_hash:") for path in verification.mismatches)


def test_verify_replay_reexecution_parity_detects_scorecard_divergence() -> None:
    result, expected_parity = _compute_runtime_parity()
    mutated_scorecard = result.scorecard.to_dict()
    aggregate_score = float(mutated_scorecard["aggregate_score"])
    mutated_scorecard["aggregate_score"] = round(aggregate_score - 0.1 if aggregate_score >= 0.1 else aggregate_score + 0.1, 6)

    verification = verify_replay_reexecution_parity(
        replay_artifact=result.replay_artifact,
        scorecard=mutated_scorecard,
        expected_parity_artifact=expected_parity,
    )

    assert verification.accepted is True
    assert verification.matched is False
    assert any(path.startswith("$.aggregate_score:") for path in verification.mismatches)
    assert any(path.startswith("$.score_summary_hash:") for path in verification.mismatches)


def test_verify_replay_reexecution_parity_supports_sparse_step_runs() -> None:
    result = run_benchmark_lifecycle(_parity_runner_config(max_steps_override=1))
    parity_result = compute_replay_parity_artifact(
        replay_artifact=result.replay_artifact,
        scorecard=result.scorecard,
    )
    assert parity_result.accepted is True
    assert parity_result.parity_artifact is not None

    verification = verify_replay_reexecution_parity(
        replay_artifact=result.replay_artifact,
        scorecard=result.scorecard,
        expected_parity_artifact=parity_result.parity_artifact,
    )

    assert verification.accepted is True
    assert verification.matched is True
    assert verification.actual_parity is not None
    assert verification.actual_parity.step_count == 1
    assert verification.actual_parity.terminal_step == 0
