"""Unit tests for persistent shared room/object state mechanics.

Covers:
- item take removes from room (room-level object absence)
- item drop adds to room (visible to all actors)
- consumable exhausted on use (entity removed from world)
- item state survives save/load
"""

from __future__ import annotations

import json
from pathlib import Path

from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager
from world.state.world_persistence import load_world_snapshot, save_world_snapshot
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from core.action_processor import ActionRequest


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_room_item_world(
    *,
    actor_ids: tuple[str, ...] = ("hero",),
    actor_room: str = "hall",
    item_entity_type: str = "item",
) -> DeterministicWorldStateManager:
    """Minimal shared world with a room containing a takeable item."""
    entities = [
        WorldEntityState(
            entity_id=actor_id,
            entity_type="player",
            location=actor_room,
            health=100,
            inventory=(),
            tags=(),
        )
        for actor_id in actor_ids
    ]
    entities.append(
        WorldEntityState(
            entity_id="treasure",
            entity_type=item_entity_type,
            location="hall",
            health=0,
            inventory=(),
            tags=(),
        )
    )
    snapshot = WorldStateSnapshot(
        tick=0,
        entities=tuple(entities),
        rooms=(
            WorldRoomState(
                room_id="hall",
                description="A stone hall with a treasure.",
                exits=(("south", "exit"),),
                entities_present=tuple(sorted(list(actor_ids) + ["treasure"])),
            ),
            WorldRoomState(
                room_id="exit",
                description="Exit room.",
                exits=(("north", "hall"),),
                entities_present=(),
            ),
        ),
        scenario_vars=(),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


def _take(actor_id: str, item_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="take",
        arguments=(("item_id", item_id),),
    )


def _drop(actor_id: str, item_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="drop",
        arguments=(("item_id", item_id),),
    )


def _use(actor_id: str, item_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="use",
        arguments=(("item_id", item_id),),
    )


# ---------------------------------------------------------------------------
# Take removes item from room
# ---------------------------------------------------------------------------


def test_take_removes_item_from_room_entities_present() -> None:
    """After take, item is absent from room's entities_present."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    assert results[0].accepted
    ws.apply_delta(dict(results[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" not in snap["rooms"]["hall"]["entities_present"]


def test_take_adds_item_to_actor_inventory() -> None:
    """After take, item is in actor's inventory."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" in snap["entities"]["hero"]["inventory"]


def test_take_sets_item_location_to_none() -> None:
    """After take, item entity's location is None (not in any room)."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["treasure"]["location"] is None


def test_second_actor_cannot_take_already_taken_item() -> None:
    """After actor-a takes the item, actor-b's take is rejected (item no longer in room)."""
    ws = _make_room_item_world(actor_ids=("actor-a", "actor-b"))
    processor = BasicDeterministicActionProcessor()

    # actor-a takes treasure
    r1 = processor.process_actions([_take("actor-a", "treasure")], ws, step_index=0)
    assert r1[0].accepted
    ws.apply_delta(dict(r1[0].world_delta))

    # actor-b tries to take — rejected
    r2 = processor.process_actions([_take("actor-b", "treasure")], ws, step_index=1)
    assert not r2[0].accepted


def test_room_item_absence_visible_to_all_after_take() -> None:
    """Both actors' snapshots show the item absent from the room after one takes it."""
    ws = _make_room_item_world(actor_ids=("actor-a", "actor-b"))
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("actor-a", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    snap = ws.get_snapshot()
    room = snap["rooms"]["hall"]
    assert "treasure" not in room["entities_present"]
    # The single shared snapshot is authoritative for both actors
    assert snap["entities"]["actor-b"]["location"] == "hall"


# ---------------------------------------------------------------------------
# Drop adds item to room
# ---------------------------------------------------------------------------


def test_drop_adds_item_to_room_entities_present() -> None:
    """After drop, item appears in room's entities_present."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    # Take first
    r1 = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    # Drop
    r2 = processor.process_actions([_drop("hero", "treasure")], ws, step_index=1)
    assert r2[0].accepted
    ws.apply_delta(dict(r2[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" in snap["rooms"]["hall"]["entities_present"]
    assert snap["entities"]["treasure"]["location"] == "hall"


def test_dropped_item_can_be_taken_by_another_actor() -> None:
    """Actor-a drops item; actor-b in same room can take it."""
    ws = _make_room_item_world(actor_ids=("actor-a", "actor-b"))
    processor = BasicDeterministicActionProcessor()

    # actor-a takes
    r1 = processor.process_actions([_take("actor-a", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    # actor-a drops
    r2 = processor.process_actions([_drop("actor-a", "treasure")], ws, step_index=1)
    ws.apply_delta(dict(r2[0].world_delta))

    # actor-b takes
    r3 = processor.process_actions([_take("actor-b", "treasure")], ws, step_index=2)
    assert r3[0].accepted
    ws.apply_delta(dict(r3[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" in snap["entities"]["actor-b"]["inventory"]
    assert "treasure" not in snap["entities"]["actor-a"]["inventory"]


# ---------------------------------------------------------------------------
# Consumable exhausted on use
# ---------------------------------------------------------------------------


def test_consumable_removed_from_inventory_on_use() -> None:
    """After using a consumable, it is removed from actor's inventory."""
    ws = _make_room_item_world(item_entity_type="consumable")
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))
    r2 = processor.process_actions([_use("hero", "treasure")], ws, step_index=1)
    assert r2[0].accepted
    ws.apply_delta(dict(r2[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" not in snap["entities"]["hero"]["inventory"]


def test_consumable_entity_deleted_from_world_on_use() -> None:
    """After using a consumable, the entity is removed from the world state entirely."""
    ws = _make_room_item_world(item_entity_type="consumable")
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))
    r2 = processor.process_actions([_use("hero", "treasure")], ws, step_index=1)
    ws.apply_delta(dict(r2[0].world_delta))

    snap = ws.get_snapshot()
    assert "treasure" not in snap["entities"], "consumed entity should be removed from world"


# ---------------------------------------------------------------------------
# Persistence across save/load
# ---------------------------------------------------------------------------


def test_taken_item_persists_across_save_load(tmp_path: Path) -> None:
    """Item in actor inventory and absent from room survives save/load roundtrip."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    dest = str(tmp_path / "take_snap.json")
    save_result = save_world_snapshot(dest, ws, run_id="r-take", scenario_id="sc1")
    assert save_result.accepted

    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "treasure" not in snap["rooms"]["hall"]["entities_present"]
    assert "treasure" in snap["entities"]["hero"]["inventory"]
    assert snap["entities"]["treasure"]["location"] is None


def test_dropped_item_persists_across_save_load(tmp_path: Path) -> None:
    """Dropped item in room survives save/load roundtrip and remains takeable."""
    ws = _make_room_item_world(actor_ids=("actor-a", "actor-b"))
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("actor-a", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))
    r2 = processor.process_actions([_drop("actor-a", "treasure")], ws, step_index=1)
    ws.apply_delta(dict(r2[0].world_delta))

    dest = str(tmp_path / "drop_snap.json")
    save_world_snapshot(dest, ws, run_id="r-drop", scenario_id="sc1")
    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "treasure" in snap["rooms"]["hall"]["entities_present"]
    assert snap["entities"]["treasure"]["location"] == "hall"


def test_consumed_item_absence_persists_across_save_load(tmp_path: Path) -> None:
    """Consumed item entity being absent from world survives save/load."""
    ws = _make_room_item_world(item_entity_type="consumable")
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))
    r2 = processor.process_actions([_use("hero", "treasure")], ws, step_index=1)
    ws.apply_delta(dict(r2[0].world_delta))

    dest = str(tmp_path / "consumed_snap.json")
    save_world_snapshot(dest, ws, run_id="r-consumed", scenario_id="sc1")
    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "treasure" not in snap["entities"]
    assert "treasure" not in snap["rooms"]["hall"]["entities_present"]


def test_room_item_state_inspectable_in_snapshot_json(tmp_path: Path) -> None:
    """Saved snapshot JSON is inspectable and correctly reflects item state."""
    ws = _make_room_item_world()
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_take("hero", "treasure")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    dest = tmp_path / "inspect.json"
    save_world_snapshot(str(dest), ws, run_id="r-inspect", scenario_id="sc1")
    payload = json.loads(dest.read_text(encoding="utf-8"))

    assert payload["world_state"]["entities"]["hero"]["inventory"] == ["treasure"]
    assert payload["world_state"]["entities"]["treasure"]["location"] is None
    assert "treasure" not in payload["world_state"]["rooms"]["hall"]["entities_present"]
