from __future__ import annotations

import pytest

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig
from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase3-adapter-scenario",
        "title": "Phase 3 Adapter Scenario",
        "description": "Deterministic runner adapter validation scenario.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": 41,
        "version": "1.0",
        "scenario_vars": {"difficulty": "normal"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    }


def test_runner_accepts_canonical_run_config_and_matches_legacy_wrapper_output() -> None:
    canonical_config = BenchmarkRunConfig(
        run_id="phase3-adapter-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
    )
    legacy_config = BenchmarkRunnerConfig(
        run_id="phase3-adapter-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
    )

    canonical = run_benchmark_lifecycle(canonical_config).to_canonical_json()
    legacy = run_benchmark_lifecycle(legacy_config).to_canonical_json()

    assert canonical == legacy


def test_runner_accepts_mapping_config_and_uses_canonical_seed_policy() -> None:
    result = run_benchmark_lifecycle(
        {
            "run_id": "phase3-adapter-mapping-run",
            "benchmark_id": "phase3-benchmark",
            "scenario": _scenario_payload(),
            "actor_ids": ["agent-a", "agent-b"],
            "run_seed": 17,
        }
    )

    assert result.lifecycle_state.seed == 17
    assert result.scorecard.metadata.seed == 17


def test_runner_rejects_invalid_config_input_type() -> None:
    with pytest.raises(
        ValueError, match="config must be BenchmarkRunConfig, BenchmarkRunnerConfig, or mapping"
    ):
        run_benchmark_lifecycle(object())  # type: ignore[arg-type]
