from __future__ import annotations

import json
import re

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle
from replay.integrity.replay_verifier import verify_replay_reexecution_parity
from replay.reconstruction.reconstructor import reconstruct_replay_state
from replay.telemetry.collector import collect_replay_telemetry


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase4-runtime-replay-scenario",
        "title": "Phase 4 Runtime Replay Wiring Scenario",
        "description": "Validate runtime benchmark execution emits replay artifacts.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 51,
        "version": "1.0",
        "scenario_vars": {"mode": "runtime-replay"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    }


def _runner_config(*, max_steps_override: int | None = None) -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="phase4-runtime-replay-run",
        benchmark_id="phase4-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        run_seed=19,
        max_steps_override=max_steps_override,
    )


def test_phase4_runtime_replay_wiring_emits_runtime_artifact_with_step_events_and_refs() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    envelope = payload["replay_artifact"]["envelope"]
    replay_events = payload["replay_artifact"]["events"]

    assert envelope["run_id"] == result.run_manifest.run_id
    assert envelope["benchmark_id"] == result.run_manifest.benchmark_id
    assert envelope["scenario_id"] == result.run_manifest.scenario_id
    assert envelope["initial_seed"] == result.run_manifest.effective_seed
    assert envelope["seed_source"] == result.run_manifest.seed_source
    assert envelope["actor_ids"] == list(result.run_manifest.actor_ids)
    assert envelope["max_steps"] == result.run_manifest.max_steps
    assert envelope["run_metadata"] == {
        "lifecycle_status": "finalized",
        "runtime_source": "benchmark_runner",
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
        "step_count": result.lifecycle_state.step_index,
        "reconstruction_state_schema": "benchmark_runtime_state_v1",
        "reconstruction_state_event_type": "state_snapshot",
    }
    manifest = payload["run_manifest"]
    scorecard_metadata = payload["scorecard"]["metadata"]
    assert manifest["benchmark_version"] == envelope["run_metadata"]["benchmark_version"]
    assert manifest["scenario_version"] == envelope["run_metadata"]["scenario_version"]
    assert manifest["scoring_version"] == envelope["run_metadata"]["scoring_version"]
    assert scorecard_metadata["benchmark_version"] == manifest["benchmark_version"]
    assert scorecard_metadata["scenario_version"] == manifest["scenario_version"]
    assert scorecard_metadata["scoring_version"] == manifest["scoring_version"]

    event_types = [event["event_type"] for event in replay_events]
    assert event_types.count("state_snapshot") == 3
    assert event_types.count("step_completed") == 3
    assert [event["step_index"] for event in replay_events] == [0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2]
    assert [event["payload"] for event in replay_events if event["event_type"] == "step_completed"] == [
        {"domain_event_count": 2, "processed_actions": 2},
        {"domain_event_count": 2, "processed_actions": 2},
        {"domain_event_count": 2, "processed_actions": 2},
    ]
    assert event_types[0:4] == ["action_move", "action_move", "step_completed", "state_snapshot"]
    assert event_types[4:8] == ["action_move", "action_move", "step_completed", "state_snapshot"]
    assert event_types[8:12] == ["action_move", "action_move", "step_completed", "state_snapshot"]
    state_events = [event for event in replay_events if event["event_type"] == "state_snapshot"]
    for step_index, event in enumerate(state_events):
        assert event["payload"]["state_schema"] == "benchmark_runtime_state_v1"
        state_payload = json.loads(event["payload"]["state_json"])
        assert state_payload["schema_version"] == "benchmark_runtime_state_v1"
        assert state_payload["run_id"] == result.run_manifest.run_id
        assert state_payload["step_index"] == step_index
        assert isinstance(state_payload["agent_states"], list)
        assert isinstance(state_payload["item_states"], list)
        assert isinstance(state_payload["npc_states"], list)
        assert isinstance(state_payload["room_states"], list)
        assert state_payload["item_states"] == []
        assert state_payload["npc_states"] == []
        assert state_payload["room_states"] == []
        assert isinstance(state_payload["tracker_total_signals"], int)

    refs = payload["replay_artifact_refs"]
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    assert refs[0]["ref"] == refs[1]["ref"]
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None

    parity_artifact = payload["replay_parity_artifact"]
    assert parity_artifact["run_id"] == result.run_manifest.run_id
    assert parity_artifact["terminal_step"] == result.lifecycle_state.step_index - 1
    assert parity_artifact["step_count"] == result.lifecycle_state.step_index
    assert parity_artifact["scorecard_step_count"] == result.scorecard.metadata.step_count
    assert parity_artifact["aggregate_score"] == result.scorecard.aggregate_score
    for field_name in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity_artifact[field_name]) is not None


def test_phase4_runtime_replay_wiring_sparse_step_run_is_reconstructable() -> None:
    result = run_benchmark_lifecycle(_runner_config(max_steps_override=1))

    reconstructed_result = reconstruct_replay_state(result.replay_artifact)
    telemetry_result = collect_replay_telemetry(result.replay_artifact)
    parity_result = verify_replay_reexecution_parity(
        replay_artifact=result.replay_artifact,
        scorecard=result.scorecard,
        expected_parity_artifact=result.replay_parity_artifact,
    )

    assert reconstructed_result.accepted is True
    assert reconstructed_result.reconstructed_state is not None
    assert reconstructed_result.reconstructed_state.applied_steps == (0,)
    assert reconstructed_result.reconstructed_state.terminal_step == 0
    reconstructed_payload = reconstructed_result.reconstructed_state.to_dict()
    assert reconstructed_payload["canonical_state"]["schema_version"] == "benchmark_runtime_state_v1"
    assert reconstructed_payload["canonical_state"]["step_index"] == 0
    assert telemetry_result.accepted is True
    assert telemetry_result.snapshot is not None
    assert telemetry_result.snapshot.step_event_counts == ((0, 4),)
    assert parity_result.accepted is True
    assert parity_result.matched is True
    assert parity_result.actual_parity is not None
    assert parity_result.actual_parity.step_count == 1


def test_phase4_runtime_replay_wiring_is_byte_identical_for_repeated_runs() -> None:
    first_result = run_benchmark_lifecycle(_runner_config())
    second_result = run_benchmark_lifecycle(_runner_config())
    first = first_result.to_canonical_json()
    second = second_result.to_canonical_json()

    assert first == second
    assert (
        first_result.replay_parity_artifact.to_canonical_json()
        == second_result.replay_parity_artifact.to_canonical_json()
    )
