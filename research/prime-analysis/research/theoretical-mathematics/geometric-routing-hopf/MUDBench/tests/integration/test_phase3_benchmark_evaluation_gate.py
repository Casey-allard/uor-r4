from __future__ import annotations

import json
from typing import Any

from evaluation.benchmark_runner.lifecycle import BenchmarkLifecycleStatus
from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase3-gate-scenario",
        "title": "Phase 3 Gate Scenario",
        "description": "Deterministic benchmark evaluation gate scenario.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 41,
        "version": "1.0",
        "scenario_vars": {"mode": "gate"},
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
        run_id="phase3-gate-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
    )


def _build_phase3_gate_artifact() -> tuple[str, dict[str, Any]]:
    result = run_benchmark_lifecycle(_runner_config())
    scorecard = result.scorecard.to_dict()
    manifest = result.run_manifest.to_dict()
    payload: dict[str, Any] = {
        "lifecycle": {
            "status": result.lifecycle_state.status.value,
            "step_index": result.lifecycle_state.step_index,
            "max_steps": result.lifecycle_state.max_steps,
        },
        "metadata": scorecard["metadata"],
        "actor_scores": [
            {
                "actor_id": actor_entry["actor_id"],
                "composite_score": actor_entry["composite_score"],
            }
            for actor_entry in scorecard["actors"]
        ],
        "aggregate_score": scorecard["aggregate_score"],
        "seed_provenance": {
            "seed_source": manifest["seed_source"],
            "manifest_effective_seed": manifest["effective_seed"],
            "lifecycle_seed": result.lifecycle_state.seed,
            "scorecard_seed": scorecard["metadata"]["seed"],
            "manifest_max_steps": manifest["max_steps"],
            "lifecycle_step_index": result.lifecycle_state.step_index,
            "scorecard_step_count": scorecard["metadata"]["step_count"],
        },
    }
    artifact = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return artifact, payload


def test_phase3_benchmark_evaluation_gate_valid_scorecard_contract() -> None:
    artifact, payload = _build_phase3_gate_artifact()
    del artifact

    assert payload == {
        "lifecycle": {
            "status": BenchmarkLifecycleStatus.FINALIZED.value,
            "step_index": 3,
            "max_steps": 3,
        },
        "metadata": {
            "run_id": "phase3-gate-run",
            "benchmark_id": "phase3-benchmark",
            "benchmark_version": "0.1",
            "scenario_id": "phase3-gate-scenario",
            "scenario_version": "1.0",
            "seed": 41,
            "step_count": 3,
            "scoring_version": "phase3-v1",
            "scorer_version": "phase3-v1",
        },
        "actor_scores": [
            {"actor_id": "agent-a", "composite_score": 0.3},
            {"actor_id": "agent-b", "composite_score": 0.3},
        ],
        "aggregate_score": 0.3,
        "seed_provenance": {
            "seed_source": "scenario_seed",
            "manifest_effective_seed": 41,
            "lifecycle_seed": 41,
            "scorecard_seed": 41,
            "manifest_max_steps": 3,
            "lifecycle_step_index": 3,
            "scorecard_step_count": 3,
        },
    }


def test_phase3_benchmark_evaluation_gate_reruns_are_byte_identical() -> None:
    first_artifact, _ = _build_phase3_gate_artifact()
    second_artifact, _ = _build_phase3_gate_artifact()

    assert first_artifact == second_artifact
