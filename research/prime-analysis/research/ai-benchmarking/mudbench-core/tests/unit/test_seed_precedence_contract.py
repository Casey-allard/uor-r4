from __future__ import annotations

import pytest

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig


def _scenario_payload(*, seed: int | None = 41) -> dict[str, object]:
    payload: dict[str, object] = {
        "scenario_id": "scenario-seed-precedence",
        "title": "Seed Precedence Scenario",
        "description": "Scenario for explicit seed precedence contract tests.",
        "start_room_id": "room-start",
        "max_steps": 5,
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
    if seed is not None:
        payload["seed"] = seed
    return payload


def test_seed_precedence_uses_seed_override_first() -> None:
    config = BenchmarkRunConfig(
        run_id="run-1",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=41),
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
        seed_override=23,
    )

    assert config.seed_source == "seed_override"
    assert config.effective_seed == 23


def test_seed_precedence_uses_run_seed_when_no_override() -> None:
    config = BenchmarkRunConfig(
        run_id="run-1",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=41),
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
    )

    assert config.seed_source == "run_seed"
    assert config.effective_seed == 17


def test_seed_precedence_uses_scenario_seed_when_no_run_or_override() -> None:
    config = BenchmarkRunConfig(
        run_id="run-1",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=41),
        actor_ids=("agent-b", "agent-a"),
    )

    assert config.seed_source == "scenario_seed"
    assert config.effective_seed == 41


def test_seed_precedence_uses_derived_when_no_explicit_sources() -> None:
    base_config = BenchmarkRunConfig(
        run_id="run-1",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=None),
        actor_ids=("agent-a", "agent-b"),
    )
    equivalent_config = BenchmarkRunConfig(
        run_id="run-1",
        benchmark_id="benchmark-1",
        scenario={
            "title": "Seed Precedence Scenario",
            "scenario_id": "scenario-seed-precedence",
            "description": "Scenario for explicit seed precedence contract tests.",
            "start_room_id": "room-start",
            "max_steps": 5,
            "version": "1.0",
            "scenario_vars": {"difficulty": "normal"},
            "objectives": [
                {
                    "objective_type": "collect_item",
                    "objective_id": "obj-a",
                    "target_id": "item-key",
                    "required_count": 1,
                }
            ],
        },
        actor_ids=("agent-b", "agent-a"),
    )

    assert base_config.seed_source == "derived"
    assert equivalent_config.seed_source == "derived"
    assert base_config.effective_seed == equivalent_config.effective_seed


def test_seed_precedence_rejects_invalid_explicit_inputs_without_fallback() -> None:
    with pytest.raises(ValueError, match="seed_override must be within \\[0, 2147483647\\]"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            run_seed=17,
            seed_override=-1,
        )

    with pytest.raises(ValueError, match="run_seed must be None or an integer"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            run_seed=True,  # type: ignore[arg-type]
        )

    with pytest.raises(ValueError, match="seed must be an integer when provided"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed="bad"),  # type: ignore[arg-type]
            actor_ids=("agent-a",),
        )
