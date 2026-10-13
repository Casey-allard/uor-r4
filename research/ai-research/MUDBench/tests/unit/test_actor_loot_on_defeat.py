"""Tests for actor loot-on-defeat mechanic.

Covers:
- Items drop to room when actor is defeated
- Actor inventory is cleared on defeat
- Room entities_present updated correctly
- Other actors can see and take dropped items
- No item duplication or loss
- Save/load preserves dropped-item state
- Deterministic ordering of dropped items
- No items dropped when inventory is empty
- Reconnect history shows loot_dropped event
- Tick messages surface loot_dropped event
"""

from __future__ import annotations

from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,
    ActionRequest,
    process_actor_defeat_tick,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _make_world(
    *,
    actor_a_inventory: list[str] | None = None,
    actor_b_inventory: list[str] | None = None,
    actor_a_health: int = 5,
    actor_b_health: int = 100,
    actor_a_room: str = "camp",
    actor_b_room: str = "camp",
    items: list[dict] | None = None,
) -> DeterministicWorldStateManager:
    """Build a minimal world with two actors and optional inventory items."""
    actor_a_inventory = actor_a_inventory or []
    actor_b_inventory = actor_b_inventory or []
    items = items or []

    entities: dict = {
        "actor-a": {
            "entity_id": "actor-a",
            "entity_type": "player",
            "location": actor_a_room,
            "health": actor_a_health,
            "inventory": actor_a_inventory,
            "tags": [],
        },
        "actor-b": {
            "entity_id": "actor-b",
            "entity_type": "agent",
            "location": actor_b_room,
            "health": actor_b_health,
            "inventory": actor_b_inventory,
            "tags": [],
        },
    }
    for item in items:
        entities[item["entity_id"]] = item

    return DeterministicWorldStateManager.from_dict({
        "entities": entities,
        "rooms": {
            "camp": {
                "title": "Camp",
                "description": "A test camp.",
                "exits": {},
                "entities_present": sorted(
                    [e["entity_id"] for e in items
                     if e.get("location") == "camp"]
                    + ["actor-a"] * (actor_a_room == "camp")
                    + ["actor-b"] * (actor_b_room == "camp")
                ),
            },
            "vault": {
                "title": "Vault",
                "description": "A test vault.",
                "exits": {},
                "entities_present": sorted(
                    [e["entity_id"] for e in items
                     if e.get("location") == "vault"]
                    + ["actor-a"] * (actor_a_room == "vault")
                    + ["actor-b"] * (actor_b_room == "vault")
                ),
            },
        },
        "scenario_vars": {
            "mode": "shared_shard",
            "actor_respawn_delay_ticks": 3,
        },
    })


def _item(item_id: str, location: str = "camp") -> dict:
    return {
        "entity_id": item_id,
        "entity_type": "item",
        "location": location,
        "health": None,
        "inventory": [],
        "tags": [],
    }


# ---------------------------------------------------------------------------
# process_actor_defeat_tick — loot drop
# ---------------------------------------------------------------------------


def test_defeated_actor_drops_item_to_room():
    """When actor-a is defeated, their held item appears in the room."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["sword"],
        items=[_item("sword", location="camp")],  # item owned by actor-a (in inventory)
    )
    # Put sword in actor-a's inventory (it was placed in camp entities, move ownership)
    world.apply_delta({
        "entities": {
            "sword": {"entity_id": "sword", "entity_type": "item",
                      "location": "camp", "health": None, "inventory": [], "tags": []},
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"], "inventory": ["sword"]},
        },
        "rooms": {
            "camp": {**world.get_snapshot()["rooms"]["camp"],
                     "entities_present": sorted({"actor-a", "actor-b"})},
        },
    })

    result = process_actor_defeat_tick(world, step_index=1)

    assert result is not None
    assert result.accepted
    snap = world.get_snapshot()

    # Apply the delta
    world.apply_delta(dict(result.world_delta))
    snap = world.get_snapshot()

    # Sword should be in the room
    assert "sword" in snap["rooms"]["camp"]["entities_present"]
    # Actor-a's inventory should be empty
    assert snap["entities"]["actor-a"]["inventory"] == []
    # Sword location should be camp
    assert snap["entities"]["sword"]["location"] == "camp"


def test_actor_inventory_cleared_on_defeat():
    """Defeated actor's inventory is emptied."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["key", "potion"],
        items=[_item("key"), _item("potion")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["key", "potion"]},
            "key": _item("key"),
            "potion": _item("potion"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    result = process_actor_defeat_tick(world, step_index=2)
    assert result is not None
    world.apply_delta(dict(result.world_delta))
    snap = world.get_snapshot()

    assert snap["entities"]["actor-a"]["inventory"] == []


def test_multiple_items_all_dropped():
    """All items from inventory are dropped, not just the first."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["key", "potion", "torch"],
        items=[_item("key"), _item("potion"), _item("torch")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["key", "potion", "torch"]},
            "key": _item("key"), "potion": _item("potion"), "torch": _item("torch"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    result = process_actor_defeat_tick(world, step_index=3)
    assert result is not None
    world.apply_delta(dict(result.world_delta))
    snap = world.get_snapshot()

    assert "key" in snap["rooms"]["camp"]["entities_present"]
    assert "potion" in snap["rooms"]["camp"]["entities_present"]
    assert "torch" in snap["rooms"]["camp"]["entities_present"]
    assert snap["entities"]["actor-a"]["inventory"] == []


def test_no_loot_drop_when_inventory_empty():
    """No loot_dropped event when actor has nothing to drop."""
    world = _make_world(actor_a_health=0, actor_a_inventory=[])

    result = process_actor_defeat_tick(world, step_index=1)
    assert result is not None  # defeat still happens
    event_types = [e.event_type for e in result.events]
    assert "actor_defeated" in event_types
    assert "actor_loot_dropped" not in event_types


def test_loot_dropped_event_emitted():
    """An actor_loot_dropped event is emitted when items are dropped."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["sword"],
        items=[_item("sword")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["sword"]},
            "sword": _item("sword"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    result = process_actor_defeat_tick(world, step_index=1)
    assert result is not None
    event_types = [e.event_type for e in result.events]
    assert "actor_loot_dropped" in event_types

    loot_event = next(e for e in result.events if e.event_type == "actor_loot_dropped")
    payload = dict(loot_event.payload)
    assert payload["actor_id"] == "actor-a"
    assert payload["room_id"] == "camp"
    assert "sword" in payload["item_ids"]


def test_loot_dropped_items_sorted_deterministically():
    """Dropped item list is sorted for deterministic ordering."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["zzz-item", "aaa-item", "mmm-item"],
        items=[_item("zzz-item"), _item("aaa-item"), _item("mmm-item")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["zzz-item", "aaa-item", "mmm-item"]},
            "zzz-item": _item("zzz-item"),
            "aaa-item": _item("aaa-item"),
            "mmm-item": _item("mmm-item"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    result = process_actor_defeat_tick(world, step_index=1)
    assert result is not None
    loot_event = next(e for e in result.events if e.event_type == "actor_loot_dropped")
    item_ids = list(dict(loot_event.payload)["item_ids"])
    assert item_ids == sorted(item_ids)


def test_alive_actor_no_loot_drop():
    """Alive actor (health > 0) does not trigger loot drop."""
    world = _make_world(actor_a_health=50, actor_a_inventory=["sword"])

    result = process_actor_defeat_tick(world, step_index=1)
    assert result is None  # no defeat at all


def test_already_defeated_actor_not_processed_twice():
    """Actor already marked defeated is not processed again (no double drop)."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["sword"],
        items=[_item("sword")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["sword"]},
            "sword": _item("sword"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
        "scenario_vars": {
            **world.get_snapshot()["scenario_vars"],
            "actor_defeated.actor-a": True,  # already defeated
        },
    })

    result = process_actor_defeat_tick(world, step_index=2)
    assert result is None  # nothing to do


def test_item_location_set_to_actor_room():
    """Dropped item's location field is set to the actor's room."""
    world = _make_world(
        actor_a_health=0,
        actor_a_room="vault",
        actor_a_inventory=["relic"],
        items=[_item("relic", location="vault")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "location": "vault", "inventory": ["relic"]},
            "relic": _item("relic", location="vault"),
        },
        "rooms": {
            "vault": {**world.get_snapshot()["rooms"]["vault"],
                      "entities_present": ["actor-a"]},
        },
    })

    result = process_actor_defeat_tick(world, step_index=1)
    assert result is not None
    world.apply_delta(dict(result.world_delta))
    snap = world.get_snapshot()

    assert snap["entities"]["relic"]["location"] == "vault"
    assert "relic" in snap["rooms"]["vault"]["entities_present"]


# ---------------------------------------------------------------------------
# Other actor can see and take dropped items
# ---------------------------------------------------------------------------


def test_other_actor_can_take_dropped_item():
    """actor-b can take an item that actor-a dropped upon defeat."""
    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["key"],
        items=[_item("key")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["key"]},
            "key": _item("key"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    # Defeat actor-a → drops key
    defeat_result = process_actor_defeat_tick(world, step_index=1)
    assert defeat_result is not None
    world.apply_delta(dict(defeat_result.world_delta))

    snap = world.get_snapshot()
    assert "key" in snap["rooms"]["camp"]["entities_present"]

    # actor-b takes the key
    processor = BasicDeterministicActionProcessor()
    take_result = processor.process_actions(
        [ActionRequest(actor_id="actor-b", action_type="take",
                       arguments=(("item_id", "key"),))],
        world,
        step_index=2,
    )
    assert take_result[0].accepted
    world.apply_delta(dict(take_result[0].world_delta))

    snap = world.get_snapshot()
    assert "key" in snap["entities"]["actor-b"]["inventory"]
    assert "key" not in snap["rooms"]["camp"]["entities_present"]


# ---------------------------------------------------------------------------
# Tick event and history messages
# ---------------------------------------------------------------------------


def test_tick_message_shows_loot_dropped():
    """_format_tick_event_messages renders actor_loot_dropped events."""
    from core.event_logger import EventRecord, normalize_payload
    from evaluation.benchmark_runner.runner import _format_tick_event_messages

    events = (
        EventRecord(
            step_index=5,
            event_type="actor_loot_dropped",
            actor_id="actor-a",
            payload=normalize_payload({
                "actor_id": "actor-a",
                "room_id": "camp",
                "item_ids": ["sword", "potion"],
            }),
        ),
    )
    msgs = _format_tick_event_messages(events, actor_id="actor-b")
    assert any("actor-a" in m and "dropped" in m for m in msgs)
    assert any("sword" in m for m in msgs)
    assert any("potion" in m for m in msgs)
    assert any("camp" in m for m in msgs)


def test_history_message_shows_loot_dropped():
    """_format_world_event_log_messages renders actor_loot_dropped log entries."""
    import json
    from evaluation.benchmark_runner.runner import _format_world_event_log_messages

    log = [
        {"event_type": "actor_loot_dropped", "step": 5,
         "actor_id": "actor-a", "room_id": "camp", "item_ids": ["key"]},
    ]
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps(log)})
    assert any("actor-a" in m and "dropped" in m for m in msgs)
    assert any("key" in m for m in msgs)


def test_tick_message_loot_dropped_nothing():
    """When item_ids is empty, shows 'nothing'."""
    from core.event_logger import EventRecord, normalize_payload
    from evaluation.benchmark_runner.runner import _format_tick_event_messages

    events = (
        EventRecord(
            step_index=5,
            event_type="actor_loot_dropped",
            actor_id="actor-a",
            payload=normalize_payload({
                "actor_id": "actor-a",
                "room_id": "camp",
                "item_ids": [],
            }),
        ),
    )
    msgs = _format_tick_event_messages(events, actor_id="actor-b")
    assert any("nothing" in m for m in msgs)


# ---------------------------------------------------------------------------
# Save/load persistence
# ---------------------------------------------------------------------------


def test_loot_drop_persists_across_save_load(tmp_path):
    """Dropped items remain in room state after save/load."""
    from world.state.world_persistence import save_world_snapshot, load_world_snapshot

    world = _make_world(
        actor_a_health=0,
        actor_a_inventory=["relic"],
        items=[_item("relic")],
    )
    world.apply_delta({
        "entities": {
            "actor-a": {**world.get_snapshot()["entities"]["actor-a"],
                        "inventory": ["relic"]},
            "relic": _item("relic"),
        },
        "rooms": {"camp": {**world.get_snapshot()["rooms"]["camp"],
                           "entities_present": ["actor-a", "actor-b"]}},
    })

    # Defeat actor-a
    result = process_actor_defeat_tick(world, step_index=1)
    assert result is not None
    world.apply_delta(dict(result.world_delta))

    # Save
    save_path = str(tmp_path / "loot_snap.json")
    save_result = save_world_snapshot(save_path, world, run_id="r1", scenario_id="s1")
    assert save_result.accepted

    # Load
    load_result = load_world_snapshot(save_path)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "relic" in snap["rooms"]["camp"]["entities_present"]
    assert snap["entities"]["actor-a"]["inventory"] == []
    assert snap["entities"]["relic"]["location"] == "camp"


# ---------------------------------------------------------------------------
# Integration via shared shard loop
# ---------------------------------------------------------------------------

_MINIMAL_SCENARIO = {
    "scenario_id": "test-loot-on-defeat",
    "title": "Test Loot On Defeat",
    "description": "Loot-on-defeat test.",
    "version": "1",
    "seed": 42,
    "max_steps": 20,
    "start_room_id": "camp",
    "objectives": [{"objective_id": "test-obj", "objective_type": "collect_item",
                    "required_count": 1, "target_id": "relic"}],
    "scenario_vars": {
        "mode": "shared_shard",
        "actor_respawn_delay_ticks": 3,
        "actor_respawn_health": 50,
        "actor_respawn_room_id": "camp",
        "world_config_json": (
            '{"rooms":{"camp":{"title":"Camp","description":"A camp.","exits":{},"entities":[]}},'
            '"npcs":[{"entity_id":"npc-x","entity_type":"npc","location":"camp","health":20,'
            '"tags":["hostile"]}],"items":[{"entity_id":"relic","entity_type":"item",'
            '"location":"camp","health":null,"inventory":[],"tags":[]}]}'
        ),
    },
}


def _build_session():
    from evaluation.benchmark_runner.runner import build_shared_shard_loop_session
    return build_shared_shard_loop_session(
        scenario=_MINIMAL_SCENARIO,
        run_id="loot-test-001",
        actor_ids=["actor-a", "actor-b"],
    )


def test_loot_dropped_in_shared_session_on_defeat():
    """In a shared session, defeated actor drops items visible to co-actors."""
    session = _build_session()

    # Give actor-a the relic
    session.world_state.apply_delta({
        "entities": {
            "actor-a": {
                **session.world_state.get_snapshot()["entities"]["actor-a"],
                "inventory": ["relic"],
                "health": 5,  # will be reduced to 0 by NPC on first tick
            },
        },
        "rooms": {
            "camp": {
                **session.world_state.get_snapshot()["rooms"]["camp"],
                "entities_present": sorted({"actor-a", "actor-b", "npc-x", "relic"}),
            }
        },
    })

    # Advance tick — NPC hits actor-a (5 HP → 0), defeat processed
    session.advance_tick({"actor-a": "wait", "actor-b": "wait"})

    snap = session.world_state.get_snapshot()
    # relic should now be in the room (dropped by actor-a)
    assert "relic" in snap["rooms"]["camp"]["entities_present"]
    assert snap["entities"]["actor-a"]["inventory"] == []

    # actor-b's observation should mention the drop
    obs_b = session.get_observation(actor_id="actor-b")
    assert any("dropped" in m and "relic" in m for m in obs_b.messages)


def test_no_regressions_on_defeat_without_inventory():
    """Defeat still works correctly when actor has no items (no loot event)."""
    session = _build_session()

    # Reduce actor-a health to 0 directly
    session.world_state.apply_delta({
        "entities": {
            "actor-a": {
                **session.world_state.get_snapshot()["entities"]["actor-a"],
                "health": 0,
                "inventory": [],
            }
        }
    })

    result = process_actor_defeat_tick(session.world_state, step_index=1)
    assert result is not None
    event_types = [e.event_type for e in result.events]
    assert "actor_defeated" in event_types
    assert "actor_loot_dropped" not in event_types
