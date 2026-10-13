from __future__ import annotations

import pytest

from world.rooms.room_graph import DeterministicRoomGraph
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-b": {
                    "title": "Room B",
                    "description": "The second room.",
                    "exits": {"west": "room-a"},
                    "entities": ["npc-existing"],
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


def test_spawn_mapping_is_deterministic_for_same_seed() -> None:
    graph = _room_graph()
    first_world = bootstrap_world_state_manager(graph, seed=17)
    second_world = bootstrap_world_state_manager(graph, seed=17)

    first_manager = DeterministicSpawnManager(seed=4)
    second_manager = DeterministicSpawnManager(seed=4)
    requests = (
        SpawnRequest(actor_id="npc-2", actor_type="npc"),
        SpawnRequest(actor_id="agent-1", actor_type="agent"),
    )

    first_mapping = first_manager.place_actors(first_world, requests)
    second_mapping = second_manager.place_actors(second_world, requests)

    assert first_mapping == second_mapping
    assert first_world.to_json() == second_world.to_json()


def test_spawn_mapping_reflected_in_world_snapshot() -> None:
    world = bootstrap_world_state_manager(_room_graph(), seed=2)
    manager = DeterministicSpawnManager(seed=0)
    mapping = manager.place_actors(
        world,
        (
            SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-b"),
            SpawnRequest(actor_id="npc-2", actor_type="npc"),
        ),
    )

    snapshot = world.get_snapshot()
    assert tuple((placement.actor_id, placement.room_id) for placement in mapping) == (
        ("agent-1", "room-b"),
        ("npc-2", "room-a"),
    )
    assert snapshot["entities"]["agent-1"]["location"] == "room-b"
    assert snapshot["entities"]["npc-2"]["location"] == "room-a"
    assert snapshot["rooms"]["room-b"]["entities_present"] == ["agent-1", "npc-existing"]


def test_spawn_seed_changes_auto_assignment_deterministically() -> None:
    graph = _room_graph()
    world_seed_0 = bootstrap_world_state_manager(graph, seed=1)
    world_seed_1 = bootstrap_world_state_manager(graph, seed=1)

    manager_seed_0 = DeterministicSpawnManager(seed=0)
    manager_seed_1 = DeterministicSpawnManager(seed=1)

    first = manager_seed_0.place_actors(
        world_seed_0,
        (SpawnRequest(actor_id="agent-1", actor_type="agent"),),
    )
    second = manager_seed_1.place_actors(
        world_seed_1,
        (SpawnRequest(actor_id="agent-1", actor_type="agent"),),
    )

    assert first[0].room_id == "room-a"
    assert second[0].room_id == "room-b"


def test_spawn_rejects_unknown_preferred_room() -> None:
    world = bootstrap_world_state_manager(_room_graph(), seed=3)
    manager = DeterministicSpawnManager(seed=0)

    with pytest.raises(ValueError, match="preferred_room_id"):
        manager.place_actors(
            world,
            (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-x"),),
        )


def test_spawn_rejects_when_world_has_no_rooms() -> None:
    world = DeterministicWorldStateManager()
    manager = DeterministicSpawnManager(seed=0)

    with pytest.raises(ValueError, match="at least one room"):
        manager.place_actors(world, (SpawnRequest(actor_id="agent-1", actor_type="agent"),))


def test_spawn_rejects_duplicate_actor_ids() -> None:
    world = bootstrap_world_state_manager(_room_graph(), seed=9)
    manager = DeterministicSpawnManager(seed=0)

    with pytest.raises(ValueError, match="actor_id already exists"):
        manager.place_actors(
            world,
            (
                SpawnRequest(actor_id="agent-1", actor_type="agent"),
                SpawnRequest(actor_id="agent-1", actor_type="agent"),
            ),
        )

