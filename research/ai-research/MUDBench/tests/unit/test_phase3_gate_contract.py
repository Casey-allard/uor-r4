from __future__ import annotations

import pytest

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _valid_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="phase3-gate-contract-run",
        benchmark_id="phase3-benchmark",
        scenario={
            "scenario_id": "phase3-gate-contract-scenario",
            "title": "Phase 3 Gate Contract Scenario",
            "description": "Deterministic scorecard contract scenario.",
            "start_room_id": "room-start",
            "max_steps": 4,
            "seed": 73,
            "version": "1.0",
            "scenario_vars": {"mode": "contract"},
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
    )


def test_phase3_gate_contract_scorecard_shape_and_metric_keys() -> None:
    payload = run_benchmark_lifecycle(_valid_config()).to_dict()
    scorecard = payload["scorecard"]
    lifecycle = payload["lifecycle_state"]
    manifest = payload["run_manifest"]

    assert set(scorecard.keys()) == {
        "metadata",
        "normalized_weights",
        "actors",
        "aggregate_metrics",
        "aggregate_contributions",
        "aggregate_score",
    }
    assert set(scorecard["metadata"].keys()) == {
        "run_id",
        "benchmark_id",
        "benchmark_version",
        "scenario_id",
        "scenario_version",
        "seed",
        "step_count",
        "scoring_version",
        "scorer_version",
    }

    expected_metric_keys = [
        "exploration_coverage",
        "quest_completion",
        "combat_performance",
        "survival_time",
        "efficiency",
    ]
    assert list(scorecard["normalized_weights"].keys()) == expected_metric_keys
    assert list(scorecard["aggregate_metrics"].keys()) == expected_metric_keys
    assert list(scorecard["aggregate_contributions"].keys()) == expected_metric_keys

    assert [entry["actor_id"] for entry in scorecard["actors"]] == ["agent-a", "agent-b"]
    for entry in scorecard["actors"]:
        assert list(entry["normalized_metrics"].keys()) == expected_metric_keys
        assert list(entry["contributions"].keys()) == expected_metric_keys
        assert 0.0 <= entry["composite_score"] <= 1.0

    assert 0.0 <= scorecard["aggregate_score"] <= 1.0
    assert manifest["seed_source"] == "scenario_seed"
    assert scorecard["metadata"]["seed"] == lifecycle["seed"] == manifest["effective_seed"]
    assert scorecard["metadata"]["step_count"] == lifecycle["step_index"] == manifest["max_steps"]
    assert scorecard["metadata"]["scenario_id"] == lifecycle["scenario_id"] == manifest["scenario_id"]
    assert scorecard["metadata"]["scenario_version"] == manifest["scenario_version"]
    assert scorecard["metadata"]["run_id"] == lifecycle["run_id"] == manifest["run_id"]
    assert scorecard["metadata"]["benchmark_id"] == manifest["benchmark_id"]
    assert scorecard["metadata"]["benchmark_version"] == manifest["benchmark_version"]
    assert scorecard["metadata"]["scoring_version"] == manifest["scoring_version"]


def test_phase3_gate_contract_scorecard_canonical_json_is_deterministic() -> None:
    first = run_benchmark_lifecycle(_valid_config()).scorecard.to_canonical_json()
    second = run_benchmark_lifecycle(_valid_config()).scorecard.to_canonical_json()

    assert first == second


def test_phase3_gate_contract_rejects_malformed_scenario() -> None:
    config = BenchmarkRunnerConfig(
        run_id="phase3-gate-contract-bad-run",
        benchmark_id="phase3-benchmark",
        scenario={"scenario_id": "missing-required-fields"},
        actor_ids=("agent-a",),
    )

    with pytest.raises(ValueError, match="scenario load rejected"):
        run_benchmark_lifecycle(config)
