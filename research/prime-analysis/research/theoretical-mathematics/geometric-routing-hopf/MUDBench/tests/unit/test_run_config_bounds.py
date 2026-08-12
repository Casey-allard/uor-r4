from __future__ import annotations

import pytest

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig

_MAX_SEED = 2_147_483_647


def _scenario_payload(*, seed: int | bool | None = 41) -> dict[str, object]:
    payload: dict[str, object] = {
        "scenario_id": "scenario-bounds",
        "title": "Bounds Scenario",
        "description": "Scenario for strict seed/max-step bounds tests.",
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


def test_run_config_bounds_accept_valid_seed_and_step_boundaries() -> None:
    lower_bound = BenchmarkRunConfig(
        run_id="run-lower",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=0),
        actor_ids=("agent-a",),
        max_steps_override=1,
    )
    assert lower_bound.seed_source == "scenario_seed"
    assert lower_bound.effective_seed == 0
    assert lower_bound.max_steps_override == 1

    upper_bound = BenchmarkRunConfig(
        run_id="run-upper",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=0),
        actor_ids=("agent-a",),
        run_seed=0,
        seed_override=_MAX_SEED,
        max_steps_override=999,
    )
    assert upper_bound.seed_source == "seed_override"
    assert upper_bound.effective_seed == _MAX_SEED

    scenario_upper = BenchmarkRunConfig(
        run_id="run-scenario-upper",
        benchmark_id="benchmark-1",
        scenario=_scenario_payload(seed=_MAX_SEED),
        actor_ids=("agent-a",),
    )
    assert scenario_upper.seed_source == "scenario_seed"
    assert scenario_upper.effective_seed == _MAX_SEED


@pytest.mark.parametrize("field_name,bad_value", [("run_seed", -1), ("run_seed", _MAX_SEED + 1)])
def test_run_config_bounds_rejects_out_of_range_run_seed(field_name: str, bad_value: int) -> None:
    with pytest.raises(ValueError, match=r"run_seed must be within \[0, 2147483647\]"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            **{field_name: bad_value},
        )


@pytest.mark.parametrize("field_name,bad_value", [("seed_override", -1), ("seed_override", _MAX_SEED + 1)])
def test_run_config_bounds_rejects_out_of_range_seed_override(field_name: str, bad_value: int) -> None:
    with pytest.raises(ValueError, match=r"seed_override must be within \[0, 2147483647\]"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            **{field_name: bad_value},
        )


@pytest.mark.parametrize("bad_seed", [-1, _MAX_SEED + 1])
def test_run_config_bounds_rejects_out_of_range_scenario_seed(bad_seed: int) -> None:
    with pytest.raises(ValueError, match=r"seed must be within \[0, 2147483647\]"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=bad_seed),
            actor_ids=("agent-a",),
        )


def test_run_config_bounds_rejects_boolean_integer_fields() -> None:
    with pytest.raises(ValueError, match="run_seed must be None or an integer"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            run_seed=True,  # type: ignore[arg-type]
        )

    with pytest.raises(ValueError, match="seed_override must be None or an integer"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            seed_override=False,  # type: ignore[arg-type]
        )

    with pytest.raises(ValueError, match="max_steps_override must be None or a positive integer"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            max_steps_override=True,  # type: ignore[arg-type]
        )

    with pytest.raises(ValueError, match="seed must be an integer when provided"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=True),  # type: ignore[arg-type]
            actor_ids=("agent-a",),
        )


@pytest.mark.parametrize("bad_step_budget", [0, -1])
def test_run_config_bounds_rejects_non_positive_max_steps_override(bad_step_budget: int) -> None:
    with pytest.raises(ValueError, match="max_steps_override must be None or a positive integer"):
        BenchmarkRunConfig(
            run_id="run-1",
            benchmark_id="benchmark-1",
            scenario=_scenario_payload(seed=41),
            actor_ids=("agent-a",),
            max_steps_override=bad_step_budget,
        )


@pytest.mark.parametrize("bad_step_budget", ["10", 1.5])
def test_run_config_bounds_rejects_non_integer_max_steps_override(bad_step_budget: object) -> None:
    with pytest.raises(ValueError, match="max_steps_override must be None or an integer"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(seed=41),
                "actor_ids": ["agent-a"],
                "max_steps_override": bad_step_budget,
            }
        )
