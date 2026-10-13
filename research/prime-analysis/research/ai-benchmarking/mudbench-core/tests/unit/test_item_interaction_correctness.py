"""Tests for shared-world same-tick item interaction correctness.

Verifies that take/drop/give-style state changes produce correct, deterministic
room/inventory state when multiple actors interact with items in the same tick,
including patterns where actors compete for the same item.
"""
from __future__ import annotations

import json

from core.action_processor import ActionRequest
from core.event_logger import EventLogger, EventRecord
from core.simulation_controller import SimulationController
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

class _Logger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def _take(actor_id: str, item_id: str) -> ActionRequest:
    return ActionRequest(actor_id=actor_id, action_type="take", arguments=(("item_id", item_id),))


def _drop(actor_id: str, item_id: str) -> ActionRequest:
    return ActionRequest(actor_id=actor_id, action_type="drop", arguments=(("item_id", item_id),))


def _give(actor_id: str, item_id: str, target_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="give",
        arguments=(("item_id", item_id), ("target_id", target_id)),
    )


def _move(actor_id: str, direction: str) -> ActionRequest:
    return ActionRequest(actor_id=actor_id, action_type="move", arguments=(("direction", direction),))


def _make_world(
    *,
    actor_a_location: str = "room-a",
    actor_b_location: str = "room-a",
    actor_a_inv: tuple[str, ...] = (),
    actor_b_inv: tuple[str, ...] = (),
    room_items: tuple[str, ...] = (),
    item_b_room: str | None = None,
) -> DeterministicWorldStateManager:
    """Build a two-room world with optional items on the floor and in inventories."""
    entities: list[WorldEntityState] = [
        WorldEntityState("actor-a", "player", actor_a_location, 100, actor_a_inv, ()),
        WorldEntityState("actor-b", "player", actor_b_location, 100, actor_b_inv, ()),
    ]
    room_a_floor = [
        e for e in ("actor-a", "actor-b")
        if (e == "actor-a" and actor_a_location == "room-a")
        or (e == "actor-b" and actor_b_location == "room-a")
    ]
    room_b_floor: list[str] = [
        e for e in ("actor-a", "actor-b")
        if (e == "actor-a" and actor_a_location == "room-b")
        or (e == "actor-b" and actor_b_location == "room-b")
    ]
    for iid in room_items:
        entities.append(WorldEntityState(iid, "item", "room-a", 0, (), ()))
        room_a_floor.append(iid)
    for iid in actor_a_inv:
        entities.append(WorldEntityState(iid, "item", None, 0, (), ()))
    for iid in actor_b_inv:
        if iid not in {e.entity_id for e in entities}:
            entities.append(WorldEntityState(iid, "item", None, 0, (), ()))
    if item_b_room is not None:
        room_b_floor.append(item_b_room)
        entities.append(WorldEntityState(item_b_room, "item", "room-b", 0, (), ()))
    return DeterministicWorldStateManager(
        initial_state=WorldStateSnapshot(
            tick=0,
            entities=tuple(entities),
            rooms=(
                WorldRoomState("room-a", "Room A", (("east", "room-b"),), tuple(sorted(room_a_floor))),
                WorldRoomState("room-b", "Room B", (("west", "room-a"),), tuple(sorted(room_b_floor))),
            ),
            scenario_vars={},
        )
    )


def _ctrl(world: DeterministicWorldStateManager, *, max_steps: int = 20) -> SimulationController:
    c = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=_Logger(),
        seed=42,
        max_steps=max_steps,
    )
    c.initialize()
    return c


def _room_entities(world: DeterministicWorldStateManager, room_id: str) -> frozenset[str]:
    return frozenset(world.get_snapshot()["rooms"][room_id].get("entities_present", []))


def _inv(world: DeterministicWorldStateManager, actor_id: str) -> frozenset[str]:
    return frozenset(world.get_snapshot()["entities"][actor_id].get("inventory", []))


def _item_location(world: DeterministicWorldStateManager, item_id: str) -> str | None:
    return world.get_snapshot()["entities"][item_id].get("location")


def _event_types(outcome) -> list[str]:
    return [e.event_type for e in outcome.emitted_events]


# ---------------------------------------------------------------------------
# Two actors compete for the same item
# ---------------------------------------------------------------------------


def test_only_first_actor_gets_contested_item() -> None:
    """When actor-a and actor-b both take item-x in the same tick, only actor-a (sorted first)
    succeeds — actor-b is rejected with no duplication."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)

    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])

    assert _inv(world, "actor-a") == frozenset({"item-x"}), "actor-a must hold item-x"
    assert _inv(world, "actor-b") == frozenset(), "actor-b must have empty inventory"
    assert _item_location(world, "item-x") is None, "item-x must have no room location"
    assert "item-x" not in _room_entities(world, "room-a"), "item-x must not remain in room"


def test_contested_item_not_duplicated() -> None:
    """Item-x must appear in exactly one place (actor-a inventory) after two concurrent takes."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)
    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])

    snap = world.get_snapshot()
    room_present = snap["rooms"]["room-a"].get("entities_present", [])
    a_inv = snap["entities"]["actor-a"].get("inventory", [])
    b_inv = snap["entities"]["actor-b"].get("inventory", [])
    item_location = snap["entities"]["item-x"].get("location")

    count = room_present.count("item-x") + a_inv.count("item-x") + b_inv.count("item-x")
    assert count == 1, f"item-x must appear exactly once; found {count} copies"
    assert item_location is None, "item-x entity location must be None when in inventory"


def test_second_actor_take_rejected_when_item_gone() -> None:
    """actor-b's take must be explicitly rejected (action_rejected event) not silently ignored."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)
    outcome = ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])

    event_types = _event_types(outcome)
    assert "action_take" in event_types, "actor-a's take must produce action_take"
    assert "action_rejected" in event_types, "actor-b's take must produce action_rejected"


# ---------------------------------------------------------------------------
# Drop then take (actor-a drops first, actor-b can pick up)
# ---------------------------------------------------------------------------


def test_drop_then_take_same_tick_succeeds() -> None:
    """actor-a drops item-x; actor-b takes it in the same tick.

    actor-a is processed first (a < b), drops the item, then actor-b sees the item
    in the room and successfully picks it up.
    """
    world = _make_world(actor_a_inv=("item-x",))
    ctrl = _ctrl(world)

    outcome = ctrl.step([_drop("actor-a", "item-x"), _take("actor-b", "item-x")])

    assert _inv(world, "actor-a") == frozenset(), "actor-a must no longer hold item-x"
    assert _inv(world, "actor-b") == frozenset({"item-x"}), "actor-b must hold item-x"
    assert "item-x" not in _room_entities(world, "room-a"), "item-x must not remain in room"
    events = _event_types(outcome)
    assert "action_drop" in events
    assert "action_take" in events
    assert "action_rejected" not in events


# ---------------------------------------------------------------------------
# Take attempted before drop (actor-a tries to take item still in actor-b's inventory)
# ---------------------------------------------------------------------------


def test_take_before_drop_fails_deterministically() -> None:
    """actor-a tries to take item-x while actor-b is dropping it in the same tick.

    actor-a is processed first. At that point item-x is still in actor-b's inventory
    (location=None), so actor-a is rejected. actor-b then successfully drops the item.
    Final state: item-x is in the room (actor-b dropped it), actor-a has nothing.
    """
    world = _make_world(actor_b_inv=("item-x",))
    ctrl = _ctrl(world)

    outcome = ctrl.step([_take("actor-a", "item-x"), _drop("actor-b", "item-x")])

    # actor-a rejected (item not in room when actor-a processed)
    assert _inv(world, "actor-a") == frozenset(), "actor-a must not have item-x"
    # actor-b successfully drops
    assert _inv(world, "actor-b") == frozenset(), "actor-b must no longer hold item-x"
    # item is now in room (actor-b dropped it)
    assert "item-x" in _room_entities(world, "room-a"), "item-x must be in room after actor-b drops it"
    assert _item_location(world, "item-x") == "room-a"

    events = _event_types(outcome)
    assert "action_rejected" in events, "actor-a's take must be rejected"
    assert "action_drop" in events, "actor-b's drop must succeed"


# ---------------------------------------------------------------------------
# Two actors each drop a different item (no conflict)
# ---------------------------------------------------------------------------


def test_two_actors_drop_different_items_both_succeed() -> None:
    """actor-a drops item-x, actor-b drops item-y in the same tick; both succeed."""
    world = _make_world(actor_a_inv=("item-x",), actor_b_inv=("item-y",))
    ctrl = _ctrl(world)

    outcome = ctrl.step([_drop("actor-a", "item-x"), _drop("actor-b", "item-y")])

    assert "item-x" in _room_entities(world, "room-a")
    assert "item-y" in _room_entities(world, "room-a")
    assert _inv(world, "actor-a") == frozenset()
    assert _inv(world, "actor-b") == frozenset()
    events = _event_types(outcome)
    assert events.count("action_drop") == 2, "Both drops must produce action_drop events"


def test_two_actors_drop_different_items_no_ghost_items() -> None:
    """No item should be duplicated when two actors drop simultaneously."""
    world = _make_world(actor_a_inv=("item-x",), actor_b_inv=("item-y",))
    ctrl = _ctrl(world)
    ctrl.step([_drop("actor-a", "item-x"), _drop("actor-b", "item-y")])

    snap = world.get_snapshot()
    room_present = snap["rooms"]["room-a"].get("entities_present", [])
    assert room_present.count("item-x") == 1, "item-x must appear exactly once in room"
    assert room_present.count("item-y") == 1, "item-y must appear exactly once in room"


# ---------------------------------------------------------------------------
# Move + take do not interfere
# ---------------------------------------------------------------------------


def test_actor_takes_item_while_other_moves_out_of_room() -> None:
    """actor-a takes item-x from room-a while actor-b moves east out of room-a.

    Both actions should succeed independently with no interference.
    """
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)

    outcome = ctrl.step([_take("actor-a", "item-x"), _move("actor-b", "east")])

    assert _inv(world, "actor-a") == frozenset({"item-x"})
    assert world.get_snapshot()["entities"]["actor-b"]["location"] == "room-b"
    assert "item-x" not in _room_entities(world, "room-a")
    events = _event_types(outcome)
    assert "action_take" in events
    assert "action_move" in events
    assert "action_rejected" not in events


def test_actor_takes_item_while_other_moves_into_room() -> None:
    """actor-b takes item in room-a while actor-a moves into room-a from room-b."""
    world = _make_world(
        actor_a_location="room-b",
        actor_b_location="room-a",
        room_items=("item-x",),
    )
    ctrl = _ctrl(world)

    ctrl.step([_move("actor-a", "west"), _take("actor-b", "item-x")])

    # actor-a arrives in room-a
    assert world.get_snapshot()["entities"]["actor-a"]["location"] == "room-a"
    # actor-b took item-x (b processed after a, so b sees the room correctly)
    assert _inv(world, "actor-b") == frozenset({"item-x"})
    assert "item-x" not in _room_entities(world, "room-a")


# ---------------------------------------------------------------------------
# Two actors take different items in the same tick (independent, no conflict)
# ---------------------------------------------------------------------------


def test_two_actors_take_different_items_both_succeed() -> None:
    """actor-a takes item-x, actor-b takes item-y simultaneously — no conflict, both succeed."""
    world = _make_world(room_items=("item-x", "item-y"))
    ctrl = _ctrl(world)

    outcome = ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-y")])

    assert _inv(world, "actor-a") == frozenset({"item-x"})
    assert _inv(world, "actor-b") == frozenset({"item-y"})
    assert "item-x" not in _room_entities(world, "room-a")
    assert "item-y" not in _room_entities(world, "room-a")
    events = _event_types(outcome)
    assert events.count("action_take") == 2, "Both takes must produce action_take events"
    assert "action_rejected" not in events


def test_two_actors_take_different_items_no_ghost_items() -> None:
    """No item should remain in the room when both actors took their respective items."""
    world = _make_world(room_items=("item-x", "item-y"))
    ctrl = _ctrl(world)
    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-y")])

    snap = world.get_snapshot()
    room_present = snap["rooms"]["room-a"].get("entities_present", [])
    assert "item-x" not in room_present
    assert "item-y" not in room_present
    assert snap["entities"]["item-x"].get("location") is None
    assert snap["entities"]["item-y"].get("location") is None


# ---------------------------------------------------------------------------
# No ghost items: items must not appear in multiple places
# ---------------------------------------------------------------------------


def test_no_ghost_item_after_contested_take() -> None:
    """After a contested take, item-x must not appear in both room AND inventory."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)
    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])

    snap = world.get_snapshot()
    room_present = set(snap["rooms"]["room-a"].get("entities_present", []))
    a_inv = set(snap["entities"]["actor-a"].get("inventory", []))
    b_inv = set(snap["entities"]["actor-b"].get("inventory", []))
    item_loc = snap["entities"]["item-x"].get("location")

    # Item must be in exactly one place
    in_room = "item-x" in room_present
    in_a_inv = "item-x" in a_inv
    in_b_inv = "item-x" in b_inv
    assert sum([in_room, in_a_inv, in_b_inv]) == 1, (
        f"item-x must be in exactly one place; room={in_room}, a_inv={in_a_inv}, b_inv={in_b_inv}"
    )
    if in_a_inv or in_b_inv:
        assert item_loc is None, "item in inventory must have location=None"
    if in_room:
        assert item_loc == "room-a", "item in room must have location='room-a'"


def test_no_item_loss_after_two_drops() -> None:
    """Both items must be findable in the world after two simultaneous drops."""
    world = _make_world(actor_a_inv=("item-x",), actor_b_inv=("item-y",))
    ctrl = _ctrl(world)
    ctrl.step([_drop("actor-a", "item-x"), _drop("actor-b", "item-y")])

    snap = world.get_snapshot()
    # Both items must have a room location
    assert snap["entities"]["item-x"].get("location") == "room-a"
    assert snap["entities"]["item-y"].get("location") == "room-a"


# ---------------------------------------------------------------------------
# Multi-tick item state consistency
# ---------------------------------------------------------------------------


def test_item_state_consistent_across_multiple_ticks() -> None:
    """Item state remains consistent across several ticks with alternating take/drop."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)

    # Tick 1: actor-a takes item-x
    ctrl.step([_take("actor-a", "item-x")])
    assert _inv(world, "actor-a") == frozenset({"item-x"})
    assert "item-x" not in _room_entities(world, "room-a")

    # Tick 2: actor-a drops item-x
    ctrl.step([_drop("actor-a", "item-x")])
    assert _inv(world, "actor-a") == frozenset()
    assert "item-x" in _room_entities(world, "room-a")

    # Tick 3: both try to take, actor-a wins again
    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])
    assert _inv(world, "actor-a") == frozenset({"item-x"})
    assert _inv(world, "actor-b") == frozenset()


# ---------------------------------------------------------------------------
# Ordering is deterministic
# ---------------------------------------------------------------------------


def test_contested_take_ordering_is_deterministic() -> None:
    """Repeated runs with same inputs must produce identical outcomes."""
    def _run() -> dict:
        w = _make_world(room_items=("item-x",))
        c = _ctrl(w)
        c.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])
        return w.to_dict()

    assert _run() == _run(), "Contested take must produce identical state on repeated runs"


# ---------------------------------------------------------------------------
# Save/load preserves item state after contested interaction
# ---------------------------------------------------------------------------


def test_save_load_preserves_item_state_after_contested_take() -> None:
    """Item state (actor-a holds item-x) must survive serialize/deserialize."""
    world = _make_world(room_items=("item-x",))
    ctrl = _ctrl(world)
    ctrl.step([_take("actor-a", "item-x"), _take("actor-b", "item-x")])

    serialized = json.dumps(world.to_dict())
    reloaded = DeterministicWorldStateManager.from_json(serialized)

    assert _inv(reloaded, "actor-a") == frozenset({"item-x"})
    assert _inv(reloaded, "actor-b") == frozenset()
    assert _item_location(reloaded, "item-x") is None
    assert "item-x" not in _room_entities(reloaded, "room-a")


def test_save_load_preserves_item_in_room_after_two_drops() -> None:
    """Both dropped items must survive serialize/deserialize."""
    world = _make_world(actor_a_inv=("item-x",), actor_b_inv=("item-y",))
    ctrl = _ctrl(world)
    ctrl.step([_drop("actor-a", "item-x"), _drop("actor-b", "item-y")])

    reloaded = DeterministicWorldStateManager.from_json(json.dumps(world.to_dict()))
    assert "item-x" in _room_entities(reloaded, "room-a")
    assert "item-y" in _room_entities(reloaded, "room-a")
