from __future__ import annotations

from world.npcs.npc_tick import DeterministicNpcTicker
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-a": {
                    "title": "Room A",
                    "description": "Hub room.",
                    "exits": {"east": "room-b", "north": "room-c"},
                    "entities": [],
                },
                "room-b": {
                    "title": "Room B",
                    "description": "Side room.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
                "room-c": {
                    "title": "Room C",
                    "description": "Upper room.",
                    "exits": {"south": "room-a"},
                    "entities": [],
                },
                "room-d": {
                    "title": "Room D",
                    "description": "Dead-end room.",
                    "exits": {},
                    "entities": [],
                },
            }
        }
    )


def _world_with_spawns(spawns: tuple[SpawnRequest, ...]) -> DeterministicWorldStateManager:
    world = bootstrap_world_state_manager(_room_graph(), seed=9)
    DeterministicSpawnManager(seed=0).place_actors(world, spawns)
    return DeterministicWorldStateManager.from_json(world.to_json())


def test_tick_processes_npcs_in_sorted_id_order() -> None:
    world = _world_with_spawns(
        (
            SpawnRequest(actor_id="npc-b", actor_type="npc", preferred_room_id="room-a"),
            SpawnRequest(actor_id="npc-a", actor_type="npc", preferred_room_id="room-a"),
        )
    )
    ticker = DeterministicNpcTicker()

    outcomes = ticker.tick(world, step_index=0)
    snapshot = world.get_snapshot()

    assert tuple(outcome.npc_id for outcome in outcomes) == ("npc-a", "npc-b")
    assert all(outcome.moved for outcome in outcomes)
    assert snapshot["entities"]["npc-a"]["location"] == "room-b"
    assert snapshot["entities"]["npc-b"]["location"] == "room-b"
    assert sorted(snapshot["rooms"]["room-b"]["entities_present"]) == ["npc-a", "npc-b"]
    assert snapshot["rooms"]["room-a"]["entities_present"] == []


def test_tick_uses_step_index_modulo_for_exit_selection() -> None:
    first_world = _world_with_spawns(
        (SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),)
    )
    second_world = _world_with_spawns(
        (SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),)
    )
    ticker = DeterministicNpcTicker()

    first_outcome = ticker.tick(first_world, step_index=0)[0]
    second_outcome = ticker.tick(second_world, step_index=1)[0]

    assert first_outcome.destination_room_id == "room-b"
    assert second_outcome.destination_room_id == "room-c"
    assert first_world.get_snapshot()["entities"]["npc-1"]["location"] == "room-b"
    assert second_world.get_snapshot()["entities"]["npc-1"]["location"] == "room-c"


def test_tick_no_exits_is_deterministic_no_op() -> None:
    world = _world_with_spawns(
        (SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-d"),)
    )
    ticker = DeterministicNpcTicker()
    before = world.to_json()

    outcome = ticker.tick(world, step_index=0)[0]

    assert outcome.moved is False
    assert outcome.reason == "no_exits"
    assert world.to_json() == before


def test_tick_destination_missing_is_deterministic_no_op() -> None:
    world = _world_with_spawns(
        (SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),)
    )
    world.apply_delta({"rooms": {"room-b": None}})
    ticker = DeterministicNpcTicker()
    before = world.to_json()

    outcome = ticker.tick(world, step_index=0)[0]

    assert outcome.moved is False
    assert outcome.reason == "destination_room_missing"
    assert outcome.source_room_id == "room-a"
    assert outcome.destination_room_id == "room-b"
    assert world.to_json() == before


def test_tick_ignores_non_npc_entities() -> None:
    world = _world_with_spawns(
        (
            SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),
            SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),
        )
    )
    ticker = DeterministicNpcTicker()

    outcomes = ticker.tick(world, step_index=0)
    snapshot = world.get_snapshot()

    assert len(outcomes) == 1
    assert outcomes[0].npc_id == "npc-1"
    assert snapshot["entities"]["agent-1"]["location"] == "room-a"
    assert snapshot["entities"]["npc-1"]["location"] == "room-b"


def test_tick_is_deterministic_for_identical_input_state() -> None:
    first_world = _world_with_spawns(
        (
            SpawnRequest(actor_id="npc-2", actor_type="npc", preferred_room_id="room-a"),
            SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),
        )
    )
    second_world = _world_with_spawns(
        (
            SpawnRequest(actor_id="npc-2", actor_type="npc", preferred_room_id="room-a"),
            SpawnRequest(actor_id="npc-1", actor_type="npc", preferred_room_id="room-a"),
        )
    )
    ticker = DeterministicNpcTicker()

    first_outcomes = ticker.tick(first_world, step_index=1)
    second_outcomes = ticker.tick(second_world, step_index=1)

    assert first_outcomes == second_outcomes
    assert first_world.to_json() == second_world.to_json()
