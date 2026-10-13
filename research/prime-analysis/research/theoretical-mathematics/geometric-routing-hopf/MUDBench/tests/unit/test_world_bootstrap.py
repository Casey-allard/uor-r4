from __future__ import annotations

from world.rooms.room_graph import DeterministicRoomGraph
from world.state.world_bootstrap import (
    bootstrap_snapshot_dict,
    bootstrap_snapshot_from_room_graph,
    bootstrap_world_state_manager,
)
from world.state.world_state import DeterministicWorldStateManager


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-b": {
                    "title": "Room B",
                    "description": "The second room.",
                    "exits": {"west": "room-a"},
                    "entities": ["npc-1"],
                },
                "room-a": {
                    "title": "Room A",
                    "description": "The first room.",
                    "exits": {"east": "room-b"},
                    "entities": [],
                },
            }
        }
    )


def test_bootstrap_snapshot_is_reproducible_for_same_seed() -> None:
    graph = _room_graph()

    first = bootstrap_snapshot_from_room_graph(graph, seed=11)
    second = bootstrap_snapshot_from_room_graph(graph, seed=11)

    assert first.to_dict() == second.to_dict()


def test_bootstrap_snapshot_respects_seed_value() -> None:
    graph = _room_graph()

    first = bootstrap_snapshot_from_room_graph(graph, seed=1)
    second = bootstrap_snapshot_from_room_graph(graph, seed=2)

    assert first.to_dict()["scenario_vars"]["bootstrap_seed"] == 1
    assert second.to_dict()["scenario_vars"]["bootstrap_seed"] == 2
    assert first.to_dict() != second.to_dict()


def test_bootstrap_output_contains_room_topology() -> None:
    graph = _room_graph()
    snapshot = bootstrap_snapshot_dict(graph, seed=99)

    assert snapshot["tick"] == 0
    assert sorted(snapshot["rooms"].keys()) == ["room-a", "room-b"]
    assert snapshot["rooms"]["room-a"]["exits"] == {"east": "room-b"}
    assert snapshot["rooms"]["room-b"]["entities_present"] == ["npc-1"]


def test_bootstrap_manager_serialization_roundtrip() -> None:
    graph = _room_graph()
    manager = bootstrap_world_state_manager(graph, seed=5)
    restored = DeterministicWorldStateManager.from_json(manager.to_json())

    assert restored.get_snapshot() == manager.get_snapshot()


def test_bootstrap_requires_integer_seed() -> None:
    graph = _room_graph()
    try:
        bootstrap_snapshot_from_room_graph(graph, seed="7")  # type: ignore[arg-type]
    except ValueError as exc:
        assert "seed must be an integer" in str(exc)
    else:
        raise AssertionError("bootstrap should reject non-integer seed values")

