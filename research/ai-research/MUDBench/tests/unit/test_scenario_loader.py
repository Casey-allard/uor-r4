from __future__ import annotations

import json

import pytest

from scenarios.scenario_definition import ScenarioDefinition
from scenarios.scenario_loader import (
    ScenarioLoadResult,
    build_scenario_initialization,
    load_scenario_batch,
    load_scenario_definition,
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "retrieval-alpha",
        "title": "Retrieve the torch",
        "description": "Find and secure the lost torch.",
        "start_room_id": "outskirts-01",
        "max_steps": 40,
        "seed": 41,
        "version": "1.0",
        "scenario_vars": {"difficulty": "easy", "hint_count": 2},
        "objectives": [
            {
                "objective_id": "obj-b",
                "objective_type": "reach_room",
                "target_id": "gate-01",
                "required_count": 1,
            },
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "torch-01",
                "required_count": 1,
                "metadata": {"item_kind": "utility"},
            },
        ],
    }


def test_load_scenario_definition_accepts_mapping_and_json_deterministically() -> None:
    payload = _scenario_payload()
    first = load_scenario_definition(payload)
    second = load_scenario_definition(json.dumps(payload))

    assert first.accepted is True
    assert second.accepted is True
    assert first.reason is None
    assert second.reason is None
    assert first.scenario == second.scenario
    assert first.scenario is not None
    assert tuple(obj.objective_id for obj in first.scenario.objectives) == ("obj-a", "obj-b")
    assert first.scenario.to_canonical_json() == second.scenario.to_canonical_json()


def test_load_scenario_definition_rejects_invalid_json() -> None:
    result = load_scenario_definition("{bad-json")
    assert result == ScenarioLoadResult(accepted=False, reason="invalid_json")


def test_load_scenario_definition_rejects_missing_required_fields() -> None:
    payload = _scenario_payload()
    del payload["title"]
    result = load_scenario_definition(payload)

    assert result.accepted is False
    assert result.scenario is None
    assert result.reason == "missing required fields: title"


def test_load_scenario_definition_rejects_duplicate_objective_ids() -> None:
    payload = _scenario_payload()
    payload["objectives"] = [
        {
            "objective_id": "dup",
            "objective_type": "collect_item",
            "target_id": "torch-01",
            "required_count": 1,
        },
        {
            "objective_id": "dup",
            "objective_type": "reach_room",
            "target_id": "gate-01",
            "required_count": 1,
        },
    ]
    result = load_scenario_definition(payload)
    assert result.accepted is False
    assert result.reason == "duplicate objective_id: dup"


def test_load_scenario_batch_preserves_input_order() -> None:
    first_payload = _scenario_payload()
    second_payload = _scenario_payload()
    second_payload["scenario_id"] = "retrieval-beta"
    second_payload["seed"] = 42

    batch = load_scenario_batch((first_payload, second_payload))
    assert len(batch) == 2
    assert batch[0].accepted is True
    assert batch[1].accepted is True
    assert batch[0].scenario is not None
    assert batch[1].scenario is not None
    assert batch[0].scenario.scenario_id == "retrieval-alpha"
    assert batch[1].scenario.scenario_id == "retrieval-beta"


def test_build_scenario_initialization_is_deterministic_and_respects_seed_override() -> None:
    loaded = load_scenario_definition(_scenario_payload())
    assert loaded.accepted is True
    assert loaded.scenario is not None

    baseline = build_scenario_initialization(loaded.scenario)
    override = build_scenario_initialization(loaded.scenario, seed_override=99)
    repeated = build_scenario_initialization(loaded.scenario)

    assert baseline.run_seed == 41
    assert override.run_seed == 99
    assert baseline == repeated
    assert baseline.to_canonical_json() == repeated.to_canonical_json()


def test_build_scenario_initialization_rejects_invalid_inputs() -> None:
    with pytest.raises(ValueError, match="scenario must be a ScenarioDefinition"):
        build_scenario_initialization(object())  # type: ignore[arg-type]

    scenario = ScenarioDefinition.from_dict(_scenario_payload())
    with pytest.raises(ValueError, match="seed_override must be None or an integer"):
        build_scenario_initialization(scenario, seed_override=True)  # type: ignore[arg-type]

