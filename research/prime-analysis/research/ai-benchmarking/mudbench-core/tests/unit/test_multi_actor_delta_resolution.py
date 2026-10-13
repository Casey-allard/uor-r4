"""Tests for multi-actor same-tick delta resolution correctness.

Verifies that when multiple actors submit actions in the same tick, the
SimulationController applies each delta before processing the next action,
preventing lost-update races on shared room state (entities_present).
"""
from __future__ import annotations

import json

import pytest

from core.action_processor import ActionRequest
from core.event_logger import EventRecord
from core.simulation_controller import SimulationController
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

class _InMemoryEventLogger:
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def _move(actor_id: str, direction: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="move",
        arguments=(("direction", direction),),
    )


def _make_two_actor_world(
    *,
    actor_a_location: str = "room-a",
    actor_b_location: str = "room-a",
) -> DeterministicWorldStateManager:
    """Three rooms in a line: room-a — east → room-b — east → room-c."""
    return DeterministicWorldStateManager(
        initial_state=WorldStateSnapshot(
            tick=0,
            entities=(
                WorldEntityState(
                    entity_id="actor-a",
                    entity_type="player",
                    location=actor_a_location,
                    health=100,
                    inventory=(),
                    tags=(),
                ),
                WorldEntityState(
                    entity_id="actor-b",
                    entity_type="player",
                    location=actor_b_location,
                    health=100,
                    inventory=(),
                    tags=(),
                ),
            ),
            rooms=(
                WorldRoomState(
                    room_id="room-a",
                    description="Room A",
                    exits=(("east", "room-b"),),
                    entities_present=tuple(
                        e for e in ("actor-a", "actor-b")
                        if (actor_a_location == "room-a" and e == "actor-a")
                        or (actor_b_location == "room-a" and e == "actor-b")
                    ),
                ),
                WorldRoomState(
                    room_id="room-b",
                    description="Room B",
                    exits=(("west", "room-a"), ("east", "room-c")),
                    entities_present=tuple(
                        e for e in ("actor-a", "actor-b")
                        if (actor_a_location == "room-b" and e == "actor-a")
                        or (actor_b_location == "room-b" and e == "actor-b")
                    ),
                ),
                WorldRoomState(
                    room_id="room-c",
                    description="Room C",
                    exits=(("west", "room-b"),),
                    entities_present=tuple(
                        e for e in ("actor-a", "actor-b")
                        if (actor_a_location == "room-c" and e == "actor-a")
                        or (actor_b_location == "room-c" and e == "actor-b")
                    ),
                ),
            ),
            scenario_vars={},
        )
    )


def _make_controller(
    world: DeterministicWorldStateManager,
    logger: _InMemoryEventLogger,
    *,
    max_steps: int = 10,
) -> SimulationController:
    ctrl = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=logger,
        seed=42,
        max_steps=max_steps,
    )
    ctrl.initialize()
    return ctrl


def _entities(world: DeterministicWorldStateManager, room_id: str) -> frozenset[str]:
    snap = world.get_snapshot()
    return frozenset(snap["rooms"][room_id].get("entities_present", []))


def _location(world: DeterministicWorldStateManager, actor_id: str) -> str:
    return world.get_snapshot()["entities"][actor_id]["location"]


# ---------------------------------------------------------------------------
# Core regression: both actors move FROM the same room in the same tick
# ---------------------------------------------------------------------------


def test_both_actors_move_out_of_same_room_no_actor_left_behind() -> None:
    """Both actor-a and actor-b move east from room-a in the same tick.

    After the tick:
    - room-a must be empty (no actor stranded by a stale delta overwrite)
    - actor-a and actor-b must both be in room-b
    """
    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])

    assert _entities(world, "room-a") == frozenset(), "room-a must be empty after both actors leave"
    assert "actor-a" in _entities(world, "room-b"), "actor-a must be in room-b"
    assert "actor-b" in _entities(world, "room-b"), "actor-b must be in room-b"
    assert _location(world, "actor-a") == "room-b"
    assert _location(world, "actor-b") == "room-b"


def test_both_actors_move_out_source_room_entity_membership_is_empty() -> None:
    """entities_present for the shared source room must be exactly empty after both leave."""
    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])

    snap = world.get_snapshot()
    room_a_entities = snap["rooms"]["room-a"].get("entities_present", [])
    assert list(room_a_entities) == [], f"room-a entities_present should be [] but got {room_a_entities}"


# ---------------------------------------------------------------------------
# Both actors move INTO the same room in the same tick
# ---------------------------------------------------------------------------


def test_both_actors_move_into_same_room_both_present() -> None:
    """actor-a is in room-a and actor-b is in room-c; both move toward room-b.

    After the tick both must be in room-b (no actor lost due to stale destination delta).
    """
    world = _make_two_actor_world(actor_a_location="room-a", actor_b_location="room-c")
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "west")])

    assert _entities(world, "room-b") == frozenset({"actor-a", "actor-b"}), (
        "Both actors must be present in room-b after converging"
    )
    assert _entities(world, "room-a") == frozenset()
    assert _entities(world, "room-c") == frozenset()


# ---------------------------------------------------------------------------
# Mixed movement: one actor moves in, one moves out of the same room
# ---------------------------------------------------------------------------


def test_one_actor_enters_other_leaves_same_room() -> None:
    """actor-a leaves room-b east → room-c; actor-b enters room-b west from room-a.

    room-b must end up with exactly actor-b (not actor-a, not empty).
    """
    world = _make_two_actor_world(actor_a_location="room-b", actor_b_location="room-a")
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])

    assert _entities(world, "room-b") == frozenset({"actor-b"}), (
        "room-b must contain only actor-b after a cross-move"
    )
    assert _entities(world, "room-c") == frozenset({"actor-a"})
    assert _entities(world, "room-a") == frozenset()


# ---------------------------------------------------------------------------
# Sequential ticks remain consistent
# ---------------------------------------------------------------------------


def test_multi_tick_movement_accumulates_correctly() -> None:
    """Two actors each advance one room per tick for two ticks — no state loss."""
    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    # Tick 1: both move east (room-a → room-b)
    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])
    assert _entities(world, "room-a") == frozenset()
    assert _entities(world, "room-b") == frozenset({"actor-a", "actor-b"})

    # Tick 2: both move east again (room-b → room-c)
    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])
    assert _entities(world, "room-b") == frozenset()
    assert _entities(world, "room-c") == frozenset({"actor-a", "actor-b"})


# ---------------------------------------------------------------------------
# Single-actor path is unaffected (regression guard)
# ---------------------------------------------------------------------------


def test_single_actor_move_still_correct() -> None:
    """Single-actor movement still produces correct room state after the fix."""
    world = _make_two_actor_world(actor_a_location="room-a", actor_b_location="room-c")
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    # Only actor-a moves
    ctrl.step([_move("actor-a", "east")])

    assert _location(world, "actor-a") == "room-b"
    assert _entities(world, "room-a") == frozenset()
    assert _entities(world, "room-b") == frozenset({"actor-a"})
    assert _entities(world, "room-c") == frozenset({"actor-b"})  # actor-b unmoved


# ---------------------------------------------------------------------------
# entities_present has no duplicate actors after simultaneous movement
# ---------------------------------------------------------------------------


def test_no_duplicate_actors_in_destination_room() -> None:
    """Destination room must not have duplicates when both actors arrive."""
    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])

    snap = world.get_snapshot()
    dest = snap["rooms"]["room-b"].get("entities_present", [])
    assert len(dest) == len(set(dest)), f"Duplicate actors in room-b: {dest}"
    assert sorted(dest) == ["actor-a", "actor-b"]


# ---------------------------------------------------------------------------
# save/load/reconnect preserves the corrected state
# ---------------------------------------------------------------------------


def test_save_load_preserves_corrected_entities_present(tmp_path: pytest.TempPathFactory) -> None:
    """After fixing a same-tick dual-move, save and reload preserves the correct state."""
    import os
    import tempfile

    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])

    # Serialize current world state
    snap_dict = world.to_dict()
    with tempfile.TemporaryDirectory() as tmpdir:
        slot_path = f"{tmpdir}/test-slot"
        import os
        os.makedirs(slot_path, exist_ok=True)
        snap_path = f"{slot_path}/world_snapshot.json"
        with open(snap_path, "w") as f:
            json.dump(snap_dict, f)
        with open(snap_path) as f:
            reloaded = json.load(f)

    reloaded_world = DeterministicWorldStateManager.from_dict(reloaded)
    assert frozenset(reloaded_world.get_snapshot()["rooms"]["room-a"].get("entities_present", [])) == frozenset()
    assert frozenset(reloaded_world.get_snapshot()["rooms"]["room-b"].get("entities_present", [])) == frozenset({"actor-a", "actor-b"})
    assert reloaded_world.get_snapshot()["entities"]["actor-a"]["location"] == "room-b"
    assert reloaded_world.get_snapshot()["entities"]["actor-b"]["location"] == "room-b"


# ---------------------------------------------------------------------------
# Ordering is deterministic: actor-a always processed before actor-b
# ---------------------------------------------------------------------------


def test_same_tick_action_ordering_is_deterministic() -> None:
    """Repeated calls with the same inputs produce identical world state."""
    def _run_and_snapshot() -> dict:
        world = _make_two_actor_world()
        logger = _InMemoryEventLogger()
        ctrl = _make_controller(world, logger)
        ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])
        return world.to_dict()

    snap1 = _run_and_snapshot()
    snap2 = _run_and_snapshot()
    assert snap1 == snap2


# ---------------------------------------------------------------------------
# StepOutcome event count is correct after fix
# ---------------------------------------------------------------------------


def test_step_outcome_processed_actions_count_correct() -> None:
    """StepOutcome.processed_actions must equal the number of submitted actions."""
    world = _make_two_actor_world()
    logger = _InMemoryEventLogger()
    ctrl = _make_controller(world, logger)

    outcome = ctrl.step([_move("actor-a", "east"), _move("actor-b", "east")])
    assert outcome.processed_actions == 2
