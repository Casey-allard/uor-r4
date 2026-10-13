from __future__ import annotations

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="phase3-traceability-run",
        benchmark_id="phase3-benchmark",
        scenario={
            "scenario_id": "phase3-traceability-scenario",
            "title": "Phase 3 Traceability Scenario",
            "description": "Traceability contract integration scenario.",
            "start_room_id": "room-start",
            "max_steps": 4,
            "seed": 41,
            "version": "1.0",
            "scenario_vars": {"mode": "traceability"},
            "objectives": [
                {
                    "objective_id": "obj-a",
                    "objective_type": "collect_item",
                    "target_id": "item-key",
                    "required_count": 1,
                }
            ],
        },
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
        max_steps_override=3,
    )


def test_phase3_scorecard_seed_traceability_matches_manifest_and_lifecycle() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()

    lifecycle = payload["lifecycle_state"]
    manifest = payload["run_manifest"]
    metadata = payload["scorecard"]["metadata"]

    assert metadata["run_id"] == lifecycle["run_id"] == manifest["run_id"]
    assert metadata["benchmark_id"] == manifest["benchmark_id"]
    assert metadata["benchmark_version"] == manifest["benchmark_version"]
    assert metadata["scenario_id"] == lifecycle["scenario_id"] == manifest["scenario_id"]
    assert metadata["scenario_version"] == manifest["scenario_version"]
    assert metadata["seed"] == lifecycle["seed"] == manifest["effective_seed"]
    assert metadata["step_count"] == lifecycle["step_index"] == manifest["max_steps"]
    assert metadata["scoring_version"] == manifest["scoring_version"]


def test_phase3_scorecard_seed_traceability_is_byte_identical_for_repeated_runs() -> None:
    first = run_benchmark_lifecycle(_runner_config()).to_canonical_json()
    second = run_benchmark_lifecycle(_runner_config()).to_canonical_json()

    assert first == second
