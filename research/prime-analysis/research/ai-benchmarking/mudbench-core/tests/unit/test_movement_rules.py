from __future__ import annotations

from world.rooms.movement_rules import DeterministicMovementRules
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-a": {
                    "title": "Room A",
                    "description": "Start room.",
                    "exits": {"east": "room-b"},
                    "entities": [],
                },
                "room-b": {
                    "title": "Room B",
                    "description": "Destination room.",
                    "exits": {"west": "room-a"},
                    "entities": ["npc-existing"],
                },
            }
        }
    )


def _world_with_spawned_actor(*, room_id: str = "room-a"):
    world = bootstrap_world_state_manager(_room_graph(), seed=7)
    spawn_manager = DeterministicSpawnManager(seed=0)
    spawn_manager.place_actors(
        world,
        (
            SpawnRequest(
                actor_id="agent-1",
                actor_type="agent",
                preferred_room_id=room_id,
            ),
        ),
    )
    return world


def _apply_resolution_delta(world, result) -> None:
    if result.world_delta is not None:
        world.apply_delta(result.world_delta)


def test_valid_move_updates_location_and_rooms_deterministically() -> None:
    graph = _room_graph()
    world = _world_with_spawned_actor(room_id="room-a")
    rules = DeterministicMovementRules()
    before = world.to_json()

    result = rules.apply_move(world, graph, actor_id="agent-1", direction="east")

    assert result.success is True
    assert result.reason is None
    assert result.source_room_id == "room-a"
    assert result.destination_room_id == "room-b"
    assert result.world_delta is not None
    assert world.to_json() == before

    _apply_resolution_delta(world, result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["agent-1"]["location"] == "room-b"
    assert snapshot["rooms"]["room-a"]["entities_present"] == []
    assert snapshot["rooms"]["room-b"]["entities_present"] == ["agent-1", "npc-existing"]


def test_invalid_exit_is_deterministic_no_op() -> None:
    graph = _room_graph()
    world = _world_with_spawned_actor(room_id="room-a")
    rules = DeterministicMovementRules()
    before = world.to_json()

    result = rules.apply_move(world, graph, actor_id="agent-1", direction="north")

    assert result.success is False
    assert result.reason == "exit_not_found"
    assert result.source_room_id == "room-a"
    assert result.destination_room_id is None
    assert world.to_json() == before


def test_unknown_actor_is_deterministic_no_op() -> None:
    graph = _room_graph()
    world = _world_with_spawned_actor(room_id="room-a")
    rules = DeterministicMovementRules()
    before = world.to_json()

    result = rules.apply_move(world, graph, actor_id="agent-missing", direction="east")

    assert result.success is False
    assert result.reason == "actor_not_found"
    assert result.source_room_id is None
    assert result.destination_room_id is None
    assert world.to_json() == before


def test_actor_without_location_is_deterministic_no_op() -> None:
    graph = _room_graph()
    world = bootstrap_world_state_manager(graph, seed=3)
    world.apply_delta({"entities": {"agent-1": {"entity_type": "agent", "location": None}}})
    rules = DeterministicMovementRules()
    before = world.to_json()

    result = rules.apply_move(world, graph, actor_id="agent-1", direction="east")

    assert result.success is False
    assert result.reason == "actor_has_no_location"
    assert world.to_json() == before


def test_missing_destination_room_is_deterministic_no_op() -> None:
    graph = _room_graph()
    world = _world_with_spawned_actor(room_id="room-a")
    world.apply_delta({"rooms": {"room-b": None}})
    rules = DeterministicMovementRules()
    before = world.to_json()

    result = rules.apply_move(world, graph, actor_id="agent-1", direction="east")

    assert result.success is False
    assert result.reason == "destination_room_missing"
    assert result.source_room_id == "room-a"
    assert result.destination_room_id == "room-b"
    assert world.to_json() == before


def test_same_input_produces_same_result_and_snapshot() -> None:
    graph = _room_graph()
    first_world = _world_with_spawned_actor(room_id="room-a")
    second_world = _world_with_spawned_actor(room_id="room-a")
    rules = DeterministicMovementRules()

    first_result = rules.apply_move(first_world, graph, actor_id="agent-1", direction="east")
    second_result = rules.apply_move(second_world, graph, actor_id="agent-1", direction="east")
    _apply_resolution_delta(first_world, first_result)
    _apply_resolution_delta(second_world, second_result)

    assert first_result == second_result
    assert first_world.to_json() == second_world.to_json()
