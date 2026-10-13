"""Deterministic take/drop/use item transition rules."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from world.state.world_state import DeterministicWorldStateManager


@dataclass(frozen=True, slots=True)
class ItemActionResolution:
    """Deterministic outcome for an item interaction attempt."""

    action_type: str
    actor_id: str
    item_id: str
    success: bool
    reason: str | None = None
    room_id: str | None = None
    consumed: bool = False
    world_delta: dict[str, Any] | None = None
    metadata: tuple[tuple[str, str], ...] = ()


class DeterministicItemRules:
    """Applies deterministic item interaction transitions to world state."""

    def apply_take(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        actor_id: str,
        item_id: str,
    ) -> ItemActionResolution:
        snapshot = _snapshot(world_state)
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")

        actor_payload = entities.get(actor_id)
        if not isinstance(actor_payload, Mapping):
            return ItemActionResolution("take", actor_id, item_id, False, reason="actor_not_found")

        actor_room = actor_payload.get("location")
        if not isinstance(actor_room, str) or not actor_room:
            return ItemActionResolution(
                "take", actor_id, item_id, False, reason="actor_has_no_location"
            )
        if actor_room not in rooms:
            return ItemActionResolution(
                "take", actor_id, item_id, False, reason="source_room_missing", room_id=actor_room
            )

        item_payload = entities.get(item_id)
        if not isinstance(item_payload, Mapping):
            return ItemActionResolution("take", actor_id, item_id, False, reason="item_not_found")

        inventory = _entity_inventory(actor_payload)
        if item_id in inventory:
            return ItemActionResolution(
                "take", actor_id, item_id, False, reason="item_already_in_inventory", room_id=actor_room
            )

        item_location = item_payload.get("location")
        if not isinstance(item_location, str) or item_location != actor_room:
            return ItemActionResolution(
                "take", actor_id, item_id, False, reason="item_not_in_room", room_id=actor_room
            )

        updated_actor = dict(actor_payload)
        updated_actor["entity_id"] = actor_id
        updated_actor["inventory"] = sorted(set(inventory + (item_id,)))

        updated_item = dict(item_payload)
        updated_item["entity_id"] = item_id
        updated_item["location"] = None

        updated_room = _room_payload_without_entity(rooms[actor_room], entity_id=item_id)
        world_delta = {
            "entities": {actor_id: updated_actor, item_id: updated_item},
            "rooms": {actor_room: updated_room},
        }
        return ItemActionResolution(
            "take",
            actor_id,
            item_id,
            True,
            room_id=actor_room,
            world_delta=world_delta,
        )

    def apply_drop(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        actor_id: str,
        item_id: str,
    ) -> ItemActionResolution:
        snapshot = _snapshot(world_state)
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")

        actor_payload = entities.get(actor_id)
        if not isinstance(actor_payload, Mapping):
            return ItemActionResolution("drop", actor_id, item_id, False, reason="actor_not_found")

        actor_room = actor_payload.get("location")
        if not isinstance(actor_room, str) or not actor_room:
            return ItemActionResolution(
                "drop", actor_id, item_id, False, reason="actor_has_no_location"
            )
        if actor_room not in rooms:
            return ItemActionResolution(
                "drop", actor_id, item_id, False, reason="source_room_missing", room_id=actor_room
            )

        item_payload = entities.get(item_id)
        if not isinstance(item_payload, Mapping):
            return ItemActionResolution("drop", actor_id, item_id, False, reason="item_not_found")

        inventory = _entity_inventory(actor_payload)
        if item_id not in inventory:
            return ItemActionResolution(
                "drop", actor_id, item_id, False, reason="item_not_in_inventory", room_id=actor_room
            )

        updated_actor = dict(actor_payload)
        updated_actor["entity_id"] = actor_id
        updated_actor["inventory"] = sorted(current_item for current_item in inventory if current_item != item_id)

        updated_item = dict(item_payload)
        updated_item["entity_id"] = item_id
        updated_item["location"] = actor_room

        updated_room = _room_payload_with_entity(rooms[actor_room], entity_id=item_id)
        world_delta = {
            "entities": {actor_id: updated_actor, item_id: updated_item},
            "rooms": {actor_room: updated_room},
        }
        return ItemActionResolution(
            "drop",
            actor_id,
            item_id,
            True,
            room_id=actor_room,
            world_delta=world_delta,
        )

    def apply_use(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        actor_id: str,
        item_id: str,
    ) -> ItemActionResolution:
        snapshot = _snapshot(world_state)
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")
        scenario_vars = snapshot.get("scenario_vars")

        actor_payload = entities.get(actor_id)
        if not isinstance(actor_payload, Mapping):
            return ItemActionResolution("use", actor_id, item_id, False, reason="actor_not_found")

        actor_room = actor_payload.get("location")
        if not isinstance(actor_room, str) or not actor_room:
            return ItemActionResolution(
                "use", actor_id, item_id, False, reason="actor_has_no_location"
            )

        item_payload = entities.get(item_id)
        if not isinstance(item_payload, Mapping):
            return ItemActionResolution("use", actor_id, item_id, False, reason="item_not_found")

        inventory = _entity_inventory(actor_payload)
        if item_id not in inventory:
            return ItemActionResolution("use", actor_id, item_id, False, reason="item_not_in_inventory")

        effect_definition = _get_unlock_effect_definition(scenario_vars, item_id)
        effect_metadata: tuple[tuple[str, str], ...] = ()
        effect_delta: dict[str, Any] | None = None
        effect_consume_override: bool | None = None

        if effect_definition is not None:
            effect_id = str(effect_definition.get("effect_id") or "")
            source_room = str(effect_definition.get("source_room_id") or "")
            direction = str(effect_definition.get("direction") or "")
            destination_room = str(effect_definition.get("destination_room_id") or "")
            if (
                not effect_id
                or not source_room
                or not direction
                or not destination_room
                or _is_effect_already_triggered(scenario_vars, effect_id)
            ):
                effect_definition = None
            else:
                requires_actor = effect_definition.get("requires_actor_in_place", True)
                if requires_actor and actor_room != source_room:
                    return ItemActionResolution(
                        "use",
                        actor_id,
                        item_id,
                        False,
                        reason="unlock_requires_actor_in_place",
                    )
                room_delta = _build_room_exit_delta(
                    rooms,
                    source_room_id=source_room,
                    direction=direction,
                    destination_room_id=destination_room,
                )
                if room_delta is None:
                    return ItemActionResolution("use", actor_id, item_id, False, reason="unlock_room_missing")
                unlock_marker = f"unlock.{effect_id}"
                effect_delta = {
                    "rooms": room_delta,
                    "scenario_vars": {unlock_marker: True},
                }
                effect_metadata = (
                    ("effect_id", effect_id),
                    ("effect_source_room_id", source_room),
                    ("effect_direction", direction),
                    ("effect_destination_room_id", destination_room),
                )
                effect_consume_override = effect_definition.get("consume_item")

        consume_flag = (
            bool(effect_consume_override)
            if effect_consume_override is not None
            else _is_consumable_item(item_payload)
        )

        base_delta: dict[str, Any] | None = None
        if consume_flag:
            updated_actor = dict(actor_payload)
            updated_actor["entity_id"] = actor_id
            updated_actor["inventory"] = sorted(
                current_item for current_item in inventory if current_item != item_id
            )
            base_delta = {
                "entities": {actor_id: updated_actor, item_id: None},
            }

        world_delta = _merge_world_deltas(base_delta, effect_delta)
        return ItemActionResolution(
            "use",
            actor_id,
            item_id,
            True,
            consumed=consume_flag,
            room_id=actor_room,
            world_delta=world_delta,
            metadata=effect_metadata,
        )

    def apply_give(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        actor_id: str,
        item_id: str,
        target_id: str,
    ) -> ItemActionResolution:
        snapshot = _snapshot(world_state)
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")
        scenario_vars = snapshot.get("scenario_vars")

        actor_payload = entities.get(actor_id)
        if not isinstance(actor_payload, Mapping):
            return ItemActionResolution("give", actor_id, item_id, False, reason="actor_not_found")
        actor_room = actor_payload.get("location")
        if not isinstance(actor_room, str) or not actor_room:
            return ItemActionResolution("give", actor_id, item_id, False, reason="actor_has_no_location")
        if actor_room not in rooms:
            return ItemActionResolution(
                "give", actor_id, item_id, False, reason="source_room_missing", room_id=actor_room
            )

        target_payload = entities.get(target_id)
        if not isinstance(target_payload, Mapping):
            return ItemActionResolution("give", actor_id, item_id, False, reason="target_not_found")
        if target_payload.get("entity_type") != "npc":
            return ItemActionResolution("give", actor_id, item_id, False, reason="target_not_npc")
        target_room = target_payload.get("location")
        if not isinstance(target_room, str) or target_room != actor_room:
            return ItemActionResolution("give", actor_id, item_id, False, reason="target_not_in_room")

        item_payload = entities.get(item_id)
        if not isinstance(item_payload, Mapping):
            return ItemActionResolution("give", actor_id, item_id, False, reason="item_not_found")

        actor_inventory = _entity_inventory(actor_payload)
        if item_id not in actor_inventory:
            return ItemActionResolution("give", actor_id, item_id, False, reason="item_not_in_inventory")

        updated_actor = dict(actor_payload)
        updated_actor["entity_id"] = actor_id
        updated_actor["inventory"] = sorted(
            current_item for current_item in actor_inventory if current_item != item_id
        )
        target_inventory = _entity_inventory(target_payload)
        updated_target = dict(target_payload)
        updated_target["entity_id"] = target_id
        updated_target["inventory"] = sorted(set(target_inventory + (item_id,)))
        updated_item = dict(item_payload)
        updated_item["entity_id"] = item_id
        updated_item["location"] = None

        world_delta: dict[str, Any] = {
            "entities": {
                actor_id: updated_actor,
                target_id: updated_target,
                item_id: updated_item,
            }
        }
        metadata: tuple[tuple[str, str], ...] = ()

        trade_effect = _get_trade_effect_definition(
            scenario_vars=scenario_vars,
            item_id=item_id,
            target_id=target_id,
        )
        if trade_effect is not None:
            effect_id = str(trade_effect.get("effect_id") or "")
            reward_item_id = str(trade_effect.get("reward_item_id") or "")
            if (
                effect_id
                and reward_item_id
                and not _is_effect_already_triggered(scenario_vars, effect_id)
            ):
                reward_entity_type = str(trade_effect.get("reward_entity_type") or "item")
                reward_item_payload = entities.get(reward_item_id)
                if isinstance(reward_item_payload, Mapping):
                    reward_payload = dict(reward_item_payload)
                    reward_payload["entity_id"] = reward_item_id
                else:
                    reward_payload = {
                        "entity_id": reward_item_id,
                        "entity_type": reward_entity_type,
                    }
                reward_payload["location"] = actor_room
                world_delta["entities"][reward_item_id] = reward_payload
                room_payload = rooms.get(actor_room)
                if isinstance(room_payload, Mapping):
                    world_delta["rooms"] = {
                        actor_room: _room_payload_with_entity(room_payload, entity_id=reward_item_id)
                    }
                world_delta["scenario_vars"] = {f"trade.{effect_id}": True}
                metadata = (
                    ("effect_id", effect_id),
                    ("effect_target_id", target_id),
                    ("effect_reward_item_id", reward_item_id),
                )

        return ItemActionResolution(
            "give",
            actor_id,
            item_id,
            True,
            room_id=actor_room,
            world_delta=world_delta,
            metadata=metadata,
        )


def _snapshot(world_state: DeterministicWorldStateManager) -> Mapping[str, Any]:
    if not isinstance(world_state, DeterministicWorldStateManager):
        raise ValueError("world_state must be a DeterministicWorldStateManager")
    return world_state.get_snapshot()


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _entity_inventory(entity_payload: Mapping[str, Any]) -> tuple[str, ...]:
    raw_inventory = entity_payload.get("inventory", ())
    if isinstance(raw_inventory, (str, bytes)) or not isinstance(raw_inventory, Sequence):
        raise ValueError("entity inventory must be a sequence of strings")
    values: set[str] = set()
    for item_id in raw_inventory:
        if not isinstance(item_id, str):
            raise ValueError("entity inventory must contain only strings")
        values.add(item_id)
    return tuple(sorted(values))


def _room_payload_with_entity(room_payload: Any, *, entity_id: str) -> dict[str, Any]:
    normalized = _room_payload(room_payload)
    entities = set(normalized["entities_present"])
    entities.add(entity_id)
    normalized["entities_present"] = sorted(entities)
    return normalized


def _room_payload_without_entity(room_payload: Any, *, entity_id: str) -> dict[str, Any]:
    normalized = _room_payload(room_payload)
    entities = set(normalized["entities_present"])
    entities.discard(entity_id)
    normalized["entities_present"] = sorted(entities)
    return normalized


def _room_payload(room_payload: Any) -> dict[str, Any]:
    if not isinstance(room_payload, Mapping):
        raise ValueError("room payload must be a mapping")
    normalized = dict(room_payload)
    raw_entities = normalized.get("entities_present", ())
    if isinstance(raw_entities, (str, bytes)) or not isinstance(raw_entities, Sequence):
        raise ValueError("room entities_present must be a sequence of strings")

    entities: set[str] = set()
    for entity_id in raw_entities:
        if not isinstance(entity_id, str):
            raise ValueError("room entities_present must contain only strings")
        entities.add(entity_id)
    normalized["entities_present"] = sorted(entities)
    return normalized


def _is_consumable_item(item_payload: Mapping[str, Any]) -> bool:
    if str(item_payload.get("entity_type", "")) == "consumable":
        return True

    raw_tags = item_payload.get("tags", ())
    if isinstance(raw_tags, (str, bytes)) or not isinstance(raw_tags, Sequence):
        return False
    for tag in raw_tags:
        if isinstance(tag, str) and tag.lower() == "consumable":
            return True
    return False


def _get_unlock_effect_definition(
    scenario_vars: Mapping[str, Any] | None,
    item_id: str,
) -> dict[str, Any] | None:
    if not isinstance(scenario_vars, Mapping):
        return None
    raw_payload = scenario_vars.get("unlock_effects_json")
    if not isinstance(raw_payload, str):
        return None
    try:
        parsed = json.loads(raw_payload)
    except json.JSONDecodeError:
        return None
    if not isinstance(parsed, Mapping):
        return None
    raw_effect = parsed.get(item_id)
    if not isinstance(raw_effect, Mapping):
        return None
    return raw_effect


def _get_trade_effect_definition(
    *,
    scenario_vars: Mapping[str, Any] | None,
    item_id: str,
    target_id: str,
) -> dict[str, Any] | None:
    if not isinstance(scenario_vars, Mapping):
        return None
    raw_payload = scenario_vars.get("trade_effects_json")
    if not isinstance(raw_payload, str):
        return None
    try:
        parsed = json.loads(raw_payload)
    except json.JSONDecodeError:
        return None
    if not isinstance(parsed, Mapping):
        return None
    raw_effect = parsed.get(f"{item_id}|{target_id}")
    if not isinstance(raw_effect, Mapping):
        return None
    return dict(raw_effect)


def _is_effect_already_triggered(scenario_vars: Mapping[str, Any] | None, effect_id: str) -> bool:
    if not isinstance(scenario_vars, Mapping):
        return False
    return bool(scenario_vars.get(f"unlock.{effect_id}"))


def _build_room_exit_delta(
    rooms: Mapping[str, Any],
    *,
    source_room_id: str,
    direction: str,
    destination_room_id: str,
) -> dict[str, dict[str, Any]] | None:
    source_room = rooms.get(source_room_id)
    if not isinstance(source_room, Mapping):
        return None
    destination_room = rooms.get(destination_room_id)
    if not isinstance(destination_room, Mapping):
        return None
    raw_exits = source_room.get("exits", {})
    if not isinstance(raw_exits, Mapping):
        return None
    if direction in raw_exits:
        return None
    updated_exits = dict(raw_exits)
    updated_exits[direction] = destination_room_id
    updated_room = dict(source_room)
    updated_room["exits"] = updated_exits
    return {source_room_id: updated_room}


def _merge_world_deltas(
    base: dict[str, Any] | None,
    extra: dict[str, Any] | None,
) -> dict[str, Any] | None:
    if base is None:
        return extra
    if extra is None:
        return base
    merged: dict[str, Any] = {}
    for key, value in base.items():
        merged[key] = dict(value) if isinstance(value, Mapping) else value
    for key, value in extra.items():
        if isinstance(value, Mapping) and key in merged and isinstance(merged[key], Mapping):
            merged_value = dict(merged[key])
            merged_value.update(value)
            merged[key] = merged_value
            continue
        merged[key] = dict(value) if isinstance(value, Mapping) else value
    return merged
