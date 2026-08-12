from __future__ import annotations

import pytest

from evaluation.benchmark_runner.lifecycle import BenchmarkLifecycleStatus
from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase3-flow-scenario",
        "title": "Phase 3 Flow Scenario",
        "description": "Deterministic benchmark flow scenario.",
        "start_room_id": "room-start",
        "max_steps": 4,
        "seed": 31,
        "version": "1.0",
        "scenario_vars": {"difficulty": "normal"},
        "objectives": [
            {
                "objective_id": "obj-b",
                "objective_type": "reach_room",
                "target_id": "room-goal",
                "required_count": 1,
            },
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            },
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="phase3-flow-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
    )


def test_phase3_benchmark_runner_flow_emits_structured_scorecard() -> None:
    result = run_benchmark_lifecycle(_runner_config())

    assert result.lifecycle_state.status is BenchmarkLifecycleStatus.FINALIZED
    assert result.lifecycle_state.step_index == 4
    assert result.lifecycle_state.max_steps == 4

    scorecard = result.scorecard.to_dict()
    assert scorecard["metadata"] == {
        "run_id": "phase3-flow-run",
        "benchmark_id": "phase3-benchmark",
        "benchmark_version": "0.1",
        "scenario_id": "phase3-flow-scenario",
        "scenario_version": "1.0",
        "seed": 31,
        "step_count": 4,
        "scoring_version": "phase3-v1",
        "scorer_version": "phase3-v1",
    }
    assert [actor["actor_id"] for actor in scorecard["actors"]] == ["agent-a", "agent-b"]
    assert 0.0 <= scorecard["aggregate_score"] <= 1.0
    assert result.composite_result.run_id == "phase3-flow-run"


def test_phase3_benchmark_runner_flow_is_byte_identical_for_repeated_runs() -> None:
    first = run_benchmark_lifecycle(_runner_config()).to_canonical_json()
    second = run_benchmark_lifecycle(_runner_config()).to_canonical_json()

    assert first == second


def test_phase3_benchmark_runner_flow_rejects_malformed_scenario() -> None:
    config = BenchmarkRunnerConfig(
        run_id="phase3-bad-run",
        benchmark_id="phase3-benchmark",
        scenario={"scenario_id": "bad-only"},
        actor_ids=("agent-a",),
    )

    with pytest.raises(ValueError, match="scenario load rejected"):
        run_benchmark_lifecycle(config)
