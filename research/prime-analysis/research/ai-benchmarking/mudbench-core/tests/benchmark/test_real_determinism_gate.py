from __future__ import annotations

import json
from typing import Any

from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    BenchmarkRunnerResult,
    run_benchmark_lifecycle,
)
from replay.integrity.replay_verifier import verify_replay_reexecution_parity


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase4-real-determinism-gate-scenario",
        "title": "Phase 4 Real Determinism Gate Scenario",
        "description": "End-to-end deterministic parity gate scenario.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 63,
        "version": "1.0",
        "scenario_vars": {"mode": "real-determinism-gate"},
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
        run_id="phase4-real-determinism-gate-run",
        benchmark_id="phase4-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        run_seed=23,
        max_steps_override=max_steps_override,
    )


def _event_order_signature(result: BenchmarkRunnerResult) -> tuple[tuple[int, str], ...]:
    return tuple((event.step_index, event.event_type) for event in result.replay_artifact.events)


def test_real_determinism_gate_repeated_runs_preserve_parity_contract_outputs() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())

    assert first.replay_artifact.to_canonical_json() == second.replay_artifact.to_canonical_json()
    assert (
        first.replay_parity_artifact.to_canonical_json()
        == second.replay_parity_artifact.to_canonical_json()
    )
    assert (
        first.replay_parity_artifact.terminal_state_hash
        == second.replay_parity_artifact.terminal_state_hash
    )
    assert (
        first.replay_parity_artifact.applied_steps_hash
        == second.replay_parity_artifact.applied_steps_hash
    )
    assert (
        first.replay_parity_artifact.score_summary_hash
        == second.replay_parity_artifact.score_summary_hash
    )
    assert first.scorecard.to_canonical_json() == second.scorecard.to_canonical_json()
    assert first.scorecard.aggregate_score == second.scorecard.aggregate_score
    assert _event_order_signature(first) == _event_order_signature(second)


def test_real_determinism_gate_sparse_step_runs_remain_deterministic() -> None:
    first = run_benchmark_lifecycle(_runner_config(max_steps_override=1))
    second = run_benchmark_lifecycle(_runner_config(max_steps_override=1))

    assert first.replay_artifact.to_canonical_json() == second.replay_artifact.to_canonical_json()
    assert (
        first.replay_parity_artifact.to_canonical_json()
        == second.replay_parity_artifact.to_canonical_json()
    )
    assert first.replay_parity_artifact.step_count == second.replay_parity_artifact.step_count == 1
    assert first.replay_parity_artifact.terminal_step == second.replay_parity_artifact.terminal_step == 0
    assert _event_order_signature(first) == _event_order_signature(second)


def test_real_determinism_gate_reports_clear_divergence_on_mutated_artifact() -> None:
    baseline = run_benchmark_lifecycle(_runner_config())
    mutated_artifact = baseline.replay_artifact.to_dict()
    events = mutated_artifact["events"]

    snapshot_event: dict[str, Any] | None = None
    for event in reversed(events):
        if isinstance(event, dict) and event.get("event_type") == "state_snapshot":
            snapshot_event = event
            break

    assert snapshot_event is not None
    payload = snapshot_event.get("payload")
    assert isinstance(payload, dict)
    state_json = payload.get("state_json")
    assert isinstance(state_json, str)
    state_payload = json.loads(state_json)
    assert isinstance(state_payload, dict)
    tracker_total_signals = state_payload.get("tracker_total_signals")
    assert isinstance(tracker_total_signals, int)
    state_payload["tracker_total_signals"] = tracker_total_signals + 1
    payload["state_json"] = json.dumps(
        state_payload,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    )

    verification = verify_replay_reexecution_parity(
        replay_artifact=mutated_artifact,
        scorecard=baseline.scorecard,
        expected_parity_artifact=baseline.replay_parity_artifact,
    )

    assert verification.accepted is True
    assert verification.matched is False
    assert verification.mismatches
    assert any(path.startswith("$.terminal_state_hash:") for path in verification.mismatches)
