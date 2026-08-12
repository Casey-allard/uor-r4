from __future__ import annotations

import pytest

from world.state.state_types import WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager


def test_world_state_initialization_is_deterministic() -> None:
    first = DeterministicWorldStateManager()
    second = DeterministicWorldStateManager()

    assert first.get_snapshot() == second.get_snapshot()
    assert first.get_snapshot() == {"tick": 0, "entities": {}, "rooms": {}, "scenario_vars": {}}


def test_world_state_apply_delta_updates_state_deterministically() -> None:
    state = DeterministicWorldStateManager()
    state.apply_delta(
        {
            "tick": 1,
            "entities": {
                "agent-1": {
                    "entity_type": "agent",
                    "location": "room-1",
                    "health": 100,
                    "inventory": ["coin"],
                    "tags": ["alive"],
                }
            },
            "rooms": {
                "room-1": {
                    "description": "A starting room.",
                    "exits": {"north": "room-2"},
                    "entities_present": ["agent-1"],
                }
            },
            "scenario_vars": {"weather": "clear", "turn_limit": 100},
        }
    )

    snapshot = state.get_snapshot()
    assert snapshot["tick"] == 1
    assert snapshot["entities"]["agent-1"]["entity_type"] == "agent"
    assert snapshot["entities"]["agent-1"]["location"] == "room-1"
    assert snapshot["rooms"]["room-1"]["exits"] == {"north": "room-2"}
    assert snapshot["scenario_vars"] == {"turn_limit": 100, "weather": "clear"}


def test_world_state_delta_order_does_not_change_outcome() -> None:
    first = DeterministicWorldStateManager()
    second = DeterministicWorldStateManager()

    first.apply_delta(
        {
            "scenario_vars": {"b": 2, "a": 1},
            "entities": {
                "z": {"entity_type": "agent", "location": "room-z"},
                "a": {"entity_type": "agent", "location": "room-a"},
            },
            "tick": 2,
        }
    )
    second.apply_delta(
        {
            "tick": 2,
            "entities": {
                "a": {"entity_type": "agent", "location": "room-a"},
                "z": {"entity_type": "agent", "location": "room-z"},
            },
            "scenario_vars": {"a": 1, "b": 2},
        }
    )

    assert first.to_json() == second.to_json()


def test_world_state_reset_restores_initial_state() -> None:
    initial = WorldStateSnapshot.from_dict(
        {
            "tick": 5,
            "entities": {
                "agent-1": {
                    "entity_id": "agent-1",
                    "entity_type": "agent",
                    "location": "room-1",
                }
            },
            "rooms": {"room-1": {"room_id": "room-1", "description": "Room 1", "exits": {}}},
            "scenario_vars": {"phase": "bootstrap"},
        }
    )
    state = DeterministicWorldStateManager(initial_state=initial)
    state.apply_delta({"tick": 99, "scenario_vars": {"phase": "changed"}})
    assert state.get_snapshot()["tick"] == 99

    state.reset()

    assert state.get_snapshot() == initial.to_dict()


def test_world_state_json_roundtrip() -> None:
    state = DeterministicWorldStateManager()
    state.apply_delta(
        {
            "tick": 7,
            "entities": {
                "npc-1": {
                    "entity_type": "npc",
                    "location": "market",
                    "tags": ["trader"],
                }
            },
            "scenario_vars": {"market_open": True},
        }
    )
    serialized = state.to_json()
    restored = DeterministicWorldStateManager.from_json(serialized)

    assert restored.get_snapshot() == state.get_snapshot()


def test_world_state_rejects_non_scalar_scenario_vars() -> None:
    state = DeterministicWorldStateManager()

    with pytest.raises(ValueError, match="JSON scalar"):
        state.apply_delta({"scenario_vars": {"nested": {"not": "scalar"}}})

