from __future__ import annotations

import json
from typing import Any

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig
from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _runner_scenario(*, seed: int) -> dict[str, object]:
    return {
        "scenario_id": "phase3-seed-matrix-scenario",
        "title": "Phase 3 Seed Matrix Scenario",
        "description": "Integration seed determinism matrix scenario.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "seed": seed,
        "version": "1.0",
        "scenario_vars": {"mode": "seed-matrix"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    }


def _derived_seed_scenario(label: str) -> dict[str, object]:
    return {
        "scenario_id": "phase3-derived-seed-scenario",
        "title": "Phase 3 Derived Seed Scenario",
        "description": f"Derived seed scenario {label}.",
        "start_room_id": "room-start",
        "max_steps": 3,
        "version": "1.0",
        "scenario_vars": {"mode": f"derived-{label}"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    }


def _to_canonical_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _runner_artifact(
    *,
    run_id: str,
    scenario_seed: int,
    run_seed: int | None = None,
    seed_override: int | None = None,
) -> tuple[str, int, str]:
    config = BenchmarkRunnerConfig(
        run_id=run_id,
        benchmark_id="phase3-benchmark",
        scenario=_runner_scenario(seed=scenario_seed),
        actor_ids=("agent-b", "agent-a"),
        run_seed=run_seed,
        seed_override=seed_override,
        max_steps_override=3,
    )
    result = run_benchmark_lifecycle(config)
    return result.to_canonical_json(), result.run_manifest.effective_seed, result.run_manifest.seed_source


def _derived_config_artifact(*, run_id: str, label: str) -> tuple[str, int, str]:
    config = BenchmarkRunConfig(
        run_id=run_id,
        benchmark_id="phase3-benchmark",
        scenario=_derived_seed_scenario(label),
        actor_ids=("agent-b", "agent-a"),
        max_steps_override=3,
    )
    return _to_canonical_json(config.to_dict()), config.effective_seed, config.seed_source


def test_phase3_seed_determinism_matrix_same_seed_identity() -> None:
    first_override, first_override_seed, first_override_source = _runner_artifact(
        run_id="phase3-matrix-override",
        scenario_seed=41,
        run_seed=13,
        seed_override=17,
    )
    second_override, second_override_seed, second_override_source = _runner_artifact(
        run_id="phase3-matrix-override",
        scenario_seed=41,
        run_seed=13,
        seed_override=17,
    )
    assert first_override_source == second_override_source == "seed_override"
    assert first_override_seed == second_override_seed == 17
    assert first_override == second_override

    first_scenario, first_scenario_seed, first_scenario_source = _runner_artifact(
        run_id="phase3-matrix-scenario-seed",
        scenario_seed=41,
    )
    second_scenario, second_scenario_seed, second_scenario_source = _runner_artifact(
        run_id="phase3-matrix-scenario-seed",
        scenario_seed=41,
    )
    assert first_scenario_source == second_scenario_source == "scenario_seed"
    assert first_scenario_seed == second_scenario_seed == 41
    assert first_scenario == second_scenario

    first_derived, first_derived_seed, first_derived_source = _derived_config_artifact(
        run_id="phase3-matrix-derived",
        label="identity",
    )
    second_derived, second_derived_seed, second_derived_source = _derived_config_artifact(
        run_id="phase3-matrix-derived",
        label="identity",
    )
    assert first_derived_source == second_derived_source == "derived"
    assert first_derived_seed == second_derived_seed
    assert first_derived == second_derived


def test_phase3_seed_determinism_matrix_different_seed_divergence() -> None:
    override_a_artifact, override_a_seed, _ = _runner_artifact(
        run_id="phase3-matrix-override-diff",
        scenario_seed=41,
        run_seed=13,
        seed_override=17,
    )
    override_b_artifact, override_b_seed, _ = _runner_artifact(
        run_id="phase3-matrix-override-diff",
        scenario_seed=41,
        run_seed=13,
        seed_override=18,
    )
    assert override_a_seed != override_b_seed
    assert override_a_artifact != override_b_artifact

    scenario_a_artifact, scenario_a_seed, _ = _runner_artifact(
        run_id="phase3-matrix-scenario-diff",
        scenario_seed=41,
    )
    scenario_b_artifact, scenario_b_seed, _ = _runner_artifact(
        run_id="phase3-matrix-scenario-diff",
        scenario_seed=42,
    )
    assert scenario_a_seed != scenario_b_seed
    assert scenario_a_artifact != scenario_b_artifact

    derived_base_artifact, derived_base_seed, _ = _derived_config_artifact(
        run_id="phase3-matrix-derived-diff",
        label="base",
    )
    derived_changed_artifact, derived_changed_seed, _ = _derived_config_artifact(
        run_id="phase3-matrix-derived-diff",
        label="changed",
    )
    if derived_base_seed == derived_changed_seed:
        derived_changed_artifact, derived_changed_seed, _ = _derived_config_artifact(
            run_id="phase3-matrix-derived-diff-alt",
            label="changed",
        )

    assert derived_base_seed != derived_changed_seed
    assert derived_base_artifact != derived_changed_artifact
