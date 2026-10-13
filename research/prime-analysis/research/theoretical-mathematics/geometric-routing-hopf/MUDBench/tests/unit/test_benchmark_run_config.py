from __future__ import annotations

import pytest

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "scenario-seed-policy",
        "title": "Seed Policy Scenario",
        "description": "Scenario for deterministic run config tests.",
        "start_room_id": "room-start",
        "max_steps": 5,
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


def test_run_config_from_mapping_normalizes_actor_order_and_reads_scenario_seed() -> None:
    config = BenchmarkRunConfig.from_mapping(
        {
            "run_id": "run-1",
            "benchmark_id": "benchmark-1",
            "scenario": _scenario_payload(),
            "actor_ids": ["agent-b", "agent-a"],
        }
    )

    assert config.actor_ids == ("agent-a", "agent-b")
    assert config.scenario_seed == 41
    assert config.seed_source == "scenario_seed"
    assert config.effective_seed == 41

    payload = config.to_dict()
    assert payload["actor_ids"] == ["agent-a", "agent-b"]
    assert payload["effective_seed"] == 41
    assert payload["seed_source"] == "scenario_seed"


def test_run_config_effective_seed_precedence_and_derived_seed_stability() -> None:
    base_payload = {
        "run_id": "run-1",
        "benchmark_id": "benchmark-1",
        "scenario": _scenario_payload(),
        "actor_ids": ["agent-b", "agent-a"],
        "run_seed": 17,
        "seed_override": 23,
    }
    override_config = BenchmarkRunConfig.from_mapping(base_payload)
    assert override_config.seed_source == "seed_override"
    assert override_config.effective_seed == 23

    run_seed_payload = dict(base_payload)
    del run_seed_payload["seed_override"]
    run_seed_config = BenchmarkRunConfig.from_mapping(run_seed_payload)
    assert run_seed_config.seed_source == "run_seed"
    assert run_seed_config.effective_seed == 17

    scenario_seed_payload = dict(run_seed_payload)
    del scenario_seed_payload["run_seed"]
    scenario_seed_config = BenchmarkRunConfig.from_mapping(scenario_seed_payload)
    assert scenario_seed_config.seed_source == "scenario_seed"
    assert scenario_seed_config.effective_seed == 41

    scenario_without_seed = _scenario_payload()
    del scenario_without_seed["seed"]
    derived_a = BenchmarkRunConfig.from_mapping(
        {
            "run_id": "run-1",
            "benchmark_id": "benchmark-1",
            "scenario": scenario_without_seed,
            "actor_ids": ["agent-b", "agent-a"],
        }
    )
    derived_b = BenchmarkRunConfig.from_mapping(
        {
            "run_id": "run-1",
            "benchmark_id": "benchmark-1",
            "scenario": {
                "title": "Seed Policy Scenario",
                "scenario_id": "scenario-seed-policy",
                "description": "Scenario for deterministic run config tests.",
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
            "actor_ids": ["agent-a", "agent-b"],
        }
    )
    assert derived_a.seed_source == "derived"
    assert derived_b.seed_source == "derived"
    assert derived_a.effective_seed == derived_b.effective_seed


def test_run_config_derived_seed_changes_when_logical_inputs_change() -> None:
    scenario_without_seed = _scenario_payload()
    del scenario_without_seed["seed"]

    first = BenchmarkRunConfig.from_mapping(
        {
            "run_id": "run-1",
            "benchmark_id": "benchmark-1",
            "scenario": scenario_without_seed,
            "actor_ids": ["agent-a", "agent-b"],
        }
    )
    second = BenchmarkRunConfig.from_mapping(
        {
            "run_id": "run-2",
            "benchmark_id": "benchmark-1",
            "scenario": scenario_without_seed,
            "actor_ids": ["agent-a", "agent-b"],
        }
    )

    assert first.seed_source == "derived"
    assert second.seed_source == "derived"
    assert first.effective_seed != second.effective_seed


def test_run_config_rejects_invalid_schema_and_seed_values() -> None:
    with pytest.raises(ValueError, match="run_id must be a non-empty string"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(),
                "actor_ids": ["agent-a"],
            }
        )

    with pytest.raises(ValueError, match="duplicate actor_id in actor_ids: agent-a"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(),
                "actor_ids": ["agent-a", "agent-a"],
            }
        )

    with pytest.raises(ValueError, match="seed_override must be within \\[0, 2147483647\\]"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(),
                "actor_ids": ["agent-a"],
                "seed_override": -1,
            }
        )

    with pytest.raises(ValueError, match="run_seed must be None or an integer"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(),
                "actor_ids": ["agent-a"],
                "run_seed": True,
            }
        )

    with pytest.raises(ValueError, match="max_steps_override must be None or a positive integer"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": _scenario_payload(),
                "actor_ids": ["agent-a"],
                "max_steps_override": 0,
            }
        )

    with pytest.raises(ValueError, match="scenario JSON string is invalid"):
        BenchmarkRunConfig.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-1",
                "scenario": "{not-json}",
                "actor_ids": ["agent-a"],
            }
        )
