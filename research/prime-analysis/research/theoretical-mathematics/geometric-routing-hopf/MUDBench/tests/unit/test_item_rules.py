from __future__ import annotations

from core.action_processor import ActionRequest, normalize_arguments
from world.items.item_rules import DeterministicItemRules
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


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
                    "description": "Second room.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
            }
        }
    )


def _world_with_actor(*, room_id: str = "room-a") -> DeterministicWorldStateManager:
    world = bootstrap_world_state_manager(_room_graph(), seed=3)
    DeterministicSpawnManager(seed=0).place_actors(
        world,
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id=room_id),),
    )
    return DeterministicWorldStateManager.from_json(world.to_json())


def _add_npc(world: DeterministicWorldStateManager, *, npc_id: str, room_id: str) -> None:
    snapshot = world.get_snapshot()
    room_payload = snapshot["rooms"][room_id]
    entities_present = sorted(set(room_payload.get("entities_present", [])) | {npc_id})
    world.apply_delta(
        {
            "entities": {
                npc_id: {
                    "entity_id": npc_id,
                    "entity_type": "npc",
                    "location": room_id,
                    "inventory": [],
                }
            },
            "rooms": {
                room_id: {
                    **room_payload,
                    "entities_present": entities_present,
                }
            },
        }
    )


def _add_item(
    world: DeterministicWorldStateManager,
    *,
    item_id: str,
    room_id: str,
    tags: tuple[str, ...] = (),
) -> None:
    snapshot = world.get_snapshot()
    room_payload = snapshot["rooms"][room_id]
    entities_present = sorted(set(room_payload.get("entities_present", [])) | {item_id})
    world.apply_delta(
        {
            "entities": {
                item_id: {
                    "entity_type": "item",
                    "location": room_id,
                    "tags": list(tags),
                }
            },
            "rooms": {
                room_id: {
                    **room_payload,
                    "entities_present": entities_present,
                }
            },
        }
    )


def _apply_resolution_delta(world: DeterministicWorldStateManager, result) -> None:
    if result.world_delta is not None:
        world.apply_delta(result.world_delta)


def test_take_moves_item_to_inventory_and_room_state() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-a")
    rules = DeterministicItemRules()
    before = world.to_json()

    result = rules.apply_take(world, actor_id="agent-1", item_id="item-1")

    assert result.success is True
    assert result.world_delta is not None
    assert world.to_json() == before

    _apply_resolution_delta(world, result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["agent-1"]["inventory"] == ["item-1"]
    assert snapshot["entities"]["item-1"]["location"] is None
    assert "item-1" not in snapshot["rooms"]["room-a"]["entities_present"]


def test_drop_moves_item_back_to_room() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-a")
    rules = DeterministicItemRules()
    take_result = rules.apply_take(world, actor_id="agent-1", item_id="item-1")
    assert take_result.success is True
    _apply_resolution_delta(world, take_result)
    before_drop = world.to_json()

    drop_result = rules.apply_drop(world, actor_id="agent-1", item_id="item-1")

    assert drop_result.success is True
    assert drop_result.world_delta is not None
    assert world.to_json() == before_drop

    _apply_resolution_delta(world, drop_result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["agent-1"]["inventory"] == []
    assert snapshot["entities"]["item-1"]["location"] == "room-a"
    assert sorted(snapshot["rooms"]["room-a"]["entities_present"]) == ["agent-1", "item-1"]


def test_use_consumable_removes_item_entity_and_inventory_entry() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-a", tags=("consumable",))
    rules = DeterministicItemRules()
    take_result = rules.apply_take(world, actor_id="agent-1", item_id="item-1")
    assert take_result.success
    _apply_resolution_delta(world, take_result)
    before_use = world.to_json()

    use_result = rules.apply_use(world, actor_id="agent-1", item_id="item-1")

    assert use_result.success is True
    assert use_result.consumed is True
    assert use_result.world_delta is not None
    assert world.to_json() == before_use

    _apply_resolution_delta(world, use_result)
    snapshot = world.get_snapshot()
    assert "item-1" not in snapshot["entities"]
    assert snapshot["entities"]["agent-1"]["inventory"] == []


def test_use_non_consumable_is_accepted_no_op() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-a", tags=("tool",))
    rules = DeterministicItemRules()
    take_result = rules.apply_take(world, actor_id="agent-1", item_id="item-1")
    assert take_result.success
    _apply_resolution_delta(world, take_result)
    before = world.to_json()

    use_result = rules.apply_use(world, actor_id="agent-1", item_id="item-1")

    assert use_result.success is True
    assert use_result.consumed is False
    assert use_result.world_delta is None
    assert world.to_json() == before


def test_invalid_take_from_other_room_is_deterministic_no_op() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-b")
    rules = DeterministicItemRules()
    before = world.to_json()

    result = rules.apply_take(world, actor_id="agent-1", item_id="item-1")

    assert result.success is False
    assert result.reason == "item_not_in_room"
    assert result.world_delta is None
    assert world.to_json() == before


def test_action_processor_processes_take_use_drop_with_explicit_invalid_behavior() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_item(world, item_id="item-1", room_id="room-a", tags=("consumable",))
    processor = BasicDeterministicActionProcessor()

    take_result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="take",
                arguments=normalize_arguments({"item_id": "item-1"}),
            ),
        ),
        world,
        step_index=0,
    )[0]
    if take_result.world_delta:
        world.apply_delta(dict(take_result.world_delta))
    use_result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="use",
                arguments=normalize_arguments({"item_id": "item-1"}),
            ),
        ),
        world,
        step_index=1,
    )[0]
    if use_result.world_delta:
        world.apply_delta(dict(use_result.world_delta))
    drop_result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="drop",
                arguments=normalize_arguments({"item_id": "item-1"}),
            ),
        ),
        world,
        step_index=2,
    )[0]

    assert take_result.accepted is True
    assert take_result.events[0].event_type == "action_take"
    assert use_result.accepted is True
    assert use_result.events[0].event_type == "action_use"
    assert dict(use_result.events[0].payload)["consumed"] is True
    assert drop_result.accepted is False
    assert drop_result.events[0].event_type == "action_rejected"
    assert dict(drop_result.events[0].payload)["reason"] == "item_not_found"


def test_item_interactions_are_deterministic_for_identical_inputs() -> None:
    first_world = _world_with_actor(room_id="room-a")
    second_world = _world_with_actor(room_id="room-a")
    _add_item(first_world, item_id="item-1", room_id="room-a")
    _add_item(second_world, item_id="item-1", room_id="room-a")
    rules = DeterministicItemRules()

    first_take = rules.apply_take(first_world, actor_id="agent-1", item_id="item-1")
    second_take = rules.apply_take(second_world, actor_id="agent-1", item_id="item-1")
    _apply_resolution_delta(first_world, first_take)
    _apply_resolution_delta(second_world, second_take)
    first_drop = rules.apply_drop(first_world, actor_id="agent-1", item_id="item-1")
    second_drop = rules.apply_drop(second_world, actor_id="agent-1", item_id="item-1")
    _apply_resolution_delta(first_world, first_drop)
    _apply_resolution_delta(second_world, second_drop)

    assert first_take == second_take
    assert first_drop == second_drop
    assert first_world.to_json() == second_world.to_json()


def test_give_transfers_item_to_npc_inventory() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_npc(world, npc_id="trader", room_id="room-a")
    _add_item(world, item_id="trade-token", room_id="room-a")
    rules = DeterministicItemRules()
    take_result = rules.apply_take(world, actor_id="agent-1", item_id="trade-token")
    assert take_result.success is True
    _apply_resolution_delta(world, take_result)

    give_result = rules.apply_give(
        world,
        actor_id="agent-1",
        item_id="trade-token",
        target_id="trader",
    )

    assert give_result.success is True
    assert give_result.world_delta is not None
    _apply_resolution_delta(world, give_result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["agent-1"]["inventory"] == []
    assert snapshot["entities"]["trader"]["inventory"] == ["trade-token"]
    assert snapshot["entities"]["trade-token"]["location"] is None


def test_give_with_trade_effect_spawns_reward_item() -> None:
    world = _world_with_actor(room_id="room-a")
    _add_npc(world, npc_id="trader", room_id="room-a")
    _add_item(world, item_id="trade-token", room_id="room-a")
    world.apply_delta(
        {
            "scenario_vars": {
                "trade_effects_json": (
                    '{"trade-token|trader":{"effect_id":"tiny-trade",'
                    '"reward_item_id":"artifact","reward_entity_type":"item"}}'
                )
            }
        }
    )
    rules = DeterministicItemRules()
    take_result = rules.apply_take(world, actor_id="agent-1", item_id="trade-token")
    assert take_result.success is True
    _apply_resolution_delta(world, take_result)

    give_result = rules.apply_give(
        world,
        actor_id="agent-1",
        item_id="trade-token",
        target_id="trader",
    )

    assert give_result.success is True
    assert give_result.metadata == (
        ("effect_id", "tiny-trade"),
        ("effect_target_id", "trader"),
        ("effect_reward_item_id", "artifact"),
    )
    assert give_result.world_delta is not None
    _apply_resolution_delta(world, give_result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["artifact"]["location"] == "room-a"
    assert "artifact" in snapshot["rooms"]["room-a"]["entities_present"]
    assert snapshot["scenario_vars"]["trade.tiny-trade"] is True
