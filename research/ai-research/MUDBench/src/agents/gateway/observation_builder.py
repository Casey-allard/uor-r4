"""Deterministic per-agent observation construction for gateway flows."""

from __future__ import annotations

import json
from typing import Any, Mapping, Sequence

from agents.protocol.observation import Observation

_DEFAULT_HEALTH = 100
_LLM_RUNTIME_MODES = frozenset({"benchmark_single_turn", "persistent_session"})


def build_observation_for_actor(
    snapshot: Mapping[str, Any],
    *,
    actor_id: str,
    run_id: str,
    step: int,
    max_steps: int,
    messages: Sequence[str] = (),
) -> Observation:
    """Build a deterministic observation for a single actor from a world snapshot."""
    if not isinstance(snapshot, Mapping):
        raise ValueError("snapshot must be a mapping")
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")
    if not isinstance(run_id, str) or not run_id:
        raise ValueError("run_id must be a non-empty string")
    if not isinstance(step, int) or step < 0:
        raise ValueError("step must be a non-negative integer")
    if not isinstance(max_steps, int) or max_steps < 0:
        raise ValueError("max_steps must be a non-negative integer")

    entities = _require_mapping(snapshot.get("entities"), field_name="snapshot.entities")
    rooms = _require_mapping(snapshot.get("rooms"), field_name="snapshot.rooms")
    scenario_vars = _require_mapping(snapshot.get("scenario_vars", {}), field_name="snapshot.scenario_vars")

    actor_payload = entities.get(actor_id)
    if not isinstance(actor_payload, Mapping):
        raise ValueError("actor_not_found")

    location = _require_non_empty_string(actor_payload.get("location"), field_name="actor.location")
    room_payload = rooms.get(location)
    if not isinstance(room_payload, Mapping):
        raise ValueError("actor_room_not_found")

    description_raw = room_payload.get("description", "")
    if not isinstance(description_raw, str):
        raise ValueError("room.description must be a string")
    description = _resolve_reactive_description(
        room_id=location,
        base_description=description_raw,
        rooms=rooms,
        entities=entities,
        scenario_vars=scenario_vars,
    )

    exits_map = _require_mapping(room_payload.get("exits", {}), field_name="room.exits")
    exits = _sorted_exits(exits_map)

    room_entity_ids = _entity_ids_in_room(room_payload, actor_id=actor_id)
    visible_room_entity_ids = _visible_room_entity_ids(
        room_entity_ids=room_entity_ids,
        entities=entities,
        scenario_vars=scenario_vars,
        actor_id=actor_id,
        room_id=location,
    )
    observed_entities = _observed_entities(visible_room_entity_ids, entities)
    inventory = _normalized_inventory(actor_payload)
    health = _resolved_health(actor_payload)
    action_space = _action_space(
        exits=exits,
        room_entity_ids=visible_room_entity_ids,
        entities=entities,
        inventory=inventory,
    )

    remaining_steps = max(max_steps - step, 0)
    normalized_messages = _normalize_string_tuple(messages, field_name="messages")
    return Observation.from_dict(
        {
            "run_id": run_id,
            "step": step,
            "location": location,
            "description": description,
            "exits": list(exits),
            "entities": list(observed_entities),
            "inventory": list(inventory),
            "health": health,
            "messages": list(normalized_messages),
            "action_space": list(action_space),
            "remaining_steps": remaining_steps,
        }
    )


def build_model_facing_observation_payload(
    observation: Observation,
    *,
    mode: str = "benchmark_single_turn",
    actor_id: str | None = None,
) -> dict[str, Any]:
    """Build the canonical model-facing observation payload for LLM runtime use."""
    if not isinstance(observation, Observation):
        raise ValueError("observation must be an Observation")
    if not isinstance(mode, str) or mode not in _LLM_RUNTIME_MODES:
        raise ValueError("mode must be one of: benchmark_single_turn, persistent_session")
    if actor_id is not None and (not isinstance(actor_id, str) or not actor_id):
        raise ValueError("actor_id must be a non-empty string when provided")

    normalized_action_space = list(observation.action_space)
    payload: dict[str, Any] = {
        "mode": mode,
        "session_frame": _build_session_frame(mode),
        "response_format": {
            "type": "json_object",
            "required_fields": ["action"],
            "additional_properties": False,
        },
        "output_contract": {
            "json_only": True,
            "single_action_only": True,
            "fail_closed_after_single_repair": True,
        },
        "action_selection_rule": "Return exactly one action from observation.action_space.",
        "allowed_actions": normalized_action_space,
        "allowed_targets": _build_allowed_targets(normalized_action_space),
        "observation": observation.to_dict(),
    }
    if actor_id is not None:
        payload["actor_id"] = actor_id
    return payload


def _build_session_frame(mode: str) -> dict[str, Any]:
    if mode == "benchmark_single_turn":
        return {
            "mode": mode,
            "turn_scope": "single_turn_only",
            "session_continuation_allowed": False,
            "history_policy": "no_cross_turn_memory",
        }
    return {
        "mode": mode,
        "turn_scope": "persistent_session_turn",
        "session_continuation_allowed": True,
        "history_policy": "caller_managed_session_history",
    }


def _build_allowed_targets(action_space: Sequence[str]) -> dict[str, Any]:
    directional_targets: list[str] = []
    single_argument_targets: dict[str, list[str]] = {
        "take": [],
        "drop": [],
        "use": [],
        "attack": [],
    }
    give_targets: list[dict[str, str]] = []

    for action in action_space:
        parts = action.split(" ")
        verb = parts[0]
        if verb == "move" and len(parts) == 2:
            _append_unique(directional_targets, parts[1])
            continue
        if verb in single_argument_targets and len(parts) == 2:
            _append_unique(single_argument_targets[verb], parts[1])
            continue
        if verb == "give" and len(parts) == 3:
            candidate = {"item_id": parts[1], "target_id": parts[2]}
            if candidate not in give_targets:
                give_targets.append(candidate)

    allowed_targets: dict[str, Any] = {}
    if directional_targets:
        allowed_targets["move"] = directional_targets
    for verb in ("take", "drop", "use", "attack"):
        if single_argument_targets[verb]:
            allowed_targets[verb] = single_argument_targets[verb]
    if give_targets:
        allowed_targets["give"] = give_targets
    return allowed_targets


def _build_loop_layers(
    observation: Observation,
    action_space: Sequence[str],
) -> dict[str, Any]:
    allowed_targets = _build_allowed_targets(action_space)
    local_objective_targets = sorted(
        target
        for verb in ("take", "use", "attack", "give")
        for target in _stringified_targets(allowed_targets.get(verb, ()))
    )
    multi_agent_entities = sorted(
        entity.name
        for entity in observation.entities
        if entity.type not in {"item", "consumable", "npc"}
    )
    return {
        "immediate_action": {
            "location": observation.location,
            "allowed_actions": list(action_space),
            "allowed_targets": allowed_targets,
            "exits": list(observation.exits),
            "visible_entities": [entity.name for entity in observation.entities],
        },
        "local_objective": {
            "candidate_targets": local_objective_targets,
            "inventory": list(observation.inventory),
            "interaction_actions": [
                action
                for action in action_space
                if action.startswith("take ")
                or action.startswith("use ")
                or action.startswith("attack ")
                or action.startswith("give ")
            ],
        },
        "temporal_world": {
            "messages": list(observation.messages),
            "step": observation.step,
            "remaining_steps": observation.remaining_steps,
        },
        "multi_agent": {
            "visible_other_entities": multi_agent_entities,
            "visible_other_entity_count": len(multi_agent_entities),
        },
        "persistence": {
            "inventory": list(observation.inventory),
            "health": observation.health,
            "step": observation.step,
            "remaining_steps": observation.remaining_steps,
        },
    }


def _build_routing_context(
    observation: Observation,
    action_space: Sequence[str],
) -> dict[str, Any]:
    allowed_targets = _build_allowed_targets(action_space)
    return {
        "available_loop_layers": [
            "immediate_action",
            "local_objective",
            "temporal_world",
            "multi_agent",
            "persistence",
        ],
        "inventory_count": len(observation.inventory),
        "message_count": len(observation.messages),
        "visible_entity_count": len(observation.entities),
        "visible_other_entity_count": sum(
            1
            for entity in observation.entities
            if entity.type not in {"item", "consumable", "npc"}
        ),
        "interaction_action_count": sum(
            1
            for action in action_space
            if action.startswith("take ")
            or action.startswith("use ")
            or action.startswith("attack ")
            or action.startswith("give ")
        ),
        "allowed_target_verbs": sorted(allowed_targets.keys()),
    }


def _stringified_targets(value: object) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, Sequence):
        items: list[str] = []
        for entry in value:
            if isinstance(entry, str):
                items.append(entry)
            elif isinstance(entry, Mapping):
                for mapping_value in entry.values():
                    if isinstance(mapping_value, str):
                        items.append(mapping_value)
        return items
    return []


def _append_unique(entries: list[str], value: str) -> None:
    if value not in entries:
        entries.append(value)


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _normalize_string_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")

    normalized: list[str] = []
    for entry in value:
        if not isinstance(entry, str):
            raise ValueError(f"{field_name} must contain only strings")
        normalized.append(entry)
    return tuple(normalized)


def _sorted_exits(exits_map: Mapping[str, Any]) -> tuple[str, ...]:
    exits: list[str] = []
    for direction, destination in exits_map.items():
        resolved_direction = _require_non_empty_string(direction, field_name="room.exits.direction")
        _require_non_empty_string(destination, field_name="room.exits.destination")
        exits.append(resolved_direction)
    return tuple(sorted(exits))


def _entity_ids_in_room(room_payload: Mapping[str, Any], *, actor_id: str) -> tuple[str, ...]:
    raw_entities = room_payload.get("entities_present", ())
    entity_ids = _normalize_string_tuple(raw_entities, field_name="room.entities_present")
    return tuple(sorted(entity_id for entity_id in entity_ids if entity_id != actor_id))


def _visible_room_entity_ids(
    *,
    room_entity_ids: Sequence[str],
    entities: Mapping[str, Any],
    scenario_vars: Mapping[str, Any],
    actor_id: str,
    room_id: str,
) -> tuple[str, ...]:
    hidden_items = _hidden_item_ids(scenario_vars)
    if len(hidden_items) == 0:
        return tuple(room_entity_ids)
    if _is_room_revealed(scenario_vars=scenario_vars, actor_id=actor_id, room_id=room_id):
        return tuple(room_entity_ids)

    visible: list[str] = []
    for entity_id in room_entity_ids:
        payload = entities.get(entity_id)
        if not isinstance(payload, Mapping):
            raise ValueError(f"room_entity_not_found:{entity_id}")
        entity_type = str(payload.get("entity_type", ""))
        if entity_id in hidden_items and entity_type in {"item", "consumable"}:
            continue
        visible.append(entity_id)
    return tuple(visible)


def _hidden_item_ids(scenario_vars: Mapping[str, Any]) -> frozenset[str]:
    if scenario_vars.get("observation_policy") != "look_reveals_hidden_items_v1":
        return frozenset()
    raw_ids = scenario_vars.get("hidden_item_ids_json")
    if not isinstance(raw_ids, str):
        return frozenset()
    try:
        parsed = json.loads(raw_ids)
    except json.JSONDecodeError:
        return frozenset()
    if not isinstance(parsed, list):
        return frozenset()
    return frozenset(value for value in parsed if isinstance(value, str) and value)


def _is_room_revealed(
    *,
    scenario_vars: Mapping[str, Any],
    actor_id: str,
    room_id: str,
) -> bool:
    reveal_key = f"reveal.{actor_id}.{room_id}"
    value = scenario_vars.get(reveal_key)
    if isinstance(value, bool):
        return value
    if isinstance(value, int):
        return value != 0
    if isinstance(value, str):
        return value in {"1", "true", "True"}
    return False


def _observed_entities(
    room_entity_ids: Sequence[str],
    entities: Mapping[str, Any],
) -> tuple[dict[str, str], ...]:
    observed: list[dict[str, str]] = []
    for entity_id in room_entity_ids:
        entity_payload = entities.get(entity_id)
        if not isinstance(entity_payload, Mapping):
            raise ValueError(f"room_entity_not_found:{entity_id}")
        entity_type = _require_non_empty_string(
            entity_payload.get("entity_type"),
            field_name=f"entity_type[{entity_id}]",
        )
        observed.append({"type": entity_type, "name": entity_id})
    return tuple(observed)


def _normalized_inventory(actor_payload: Mapping[str, Any]) -> tuple[str, ...]:
    raw_inventory = actor_payload.get("inventory", ())
    entries = _normalize_string_tuple(raw_inventory, field_name="actor.inventory")
    return tuple(sorted(set(entries)))


def _resolved_health(actor_payload: Mapping[str, Any]) -> int:
    raw_health = actor_payload.get("health")
    if raw_health is None:
        return _DEFAULT_HEALTH
    if not isinstance(raw_health, int):
        raise ValueError("actor.health must be an integer")
    return raw_health


def _action_space(
    *,
    exits: Sequence[str],
    room_entity_ids: Sequence[str],
    entities: Mapping[str, Any],
    inventory: Sequence[str],
) -> tuple[str, ...]:
    ordered_actions: list[str] = ["wait", "look"]
    ordered_actions.extend(f"move {direction}" for direction in exits)

    take_targets: list[str] = []
    attack_targets: list[str] = []
    give_targets: list[str] = []
    talk_targets: list[str] = []
    for entity_id in room_entity_ids:
        payload = entities.get(entity_id)
        if not isinstance(payload, Mapping):
            raise ValueError(f"room_entity_not_found:{entity_id}")
        entity_type = str(payload.get("entity_type", ""))
        tags = payload.get("tags") or []
        if entity_type in {"item", "consumable"}:
            take_targets.append(entity_id)
        if entity_type == "npc":
            # All NPCs are valid attack targets unless explicitly tagged "neutral" or "peaceful".
            # Neutral/peaceful NPCs represent allies, quest-givers, or non-combatants where
            # attacking is almost always a player error rather than intentional play.
            if "neutral" not in tags and "peaceful" not in tags:
                attack_targets.append(entity_id)
            # Non-hostile NPCs can be talked to.
            if "hostile" not in tags:
                talk_targets.append(entity_id)
            # Give targets include all NPCs for item trading, but NOT for consumables
            # which are activated with ``use`` rather than handed to an NPC.
            give_targets.append(entity_id)

    ordered_actions.extend(f"take {entity_id}" for entity_id in sorted(take_targets))
    ordered_actions.extend(f"drop {item_id}" for item_id in sorted(inventory))
    ordered_actions.extend(f"use {item_id}" for item_id in sorted(inventory))
    for item_id in sorted(inventory):
        # Consumables are activated with ``use``, not handed to NPCs — skip give for them.
        item_entity_type = str((entities.get(item_id) or {}).get("entity_type", ""))
        if item_entity_type != "consumable":
            ordered_actions.extend(
                f"give {item_id} {target_id}" for target_id in sorted(give_targets)
            )
    ordered_actions.extend(f"talk {entity_id}" for entity_id in sorted(talk_targets))
    # defend appears alongside attack options — it's only useful when hostile NPCs are present.
    if attack_targets:
        ordered_actions.append("defend")
    ordered_actions.extend(f"attack {entity_id}" for entity_id in sorted(attack_targets))

    deduplicated: list[str] = []
    seen: set[str] = set()
    for action in ordered_actions:
        if action in seen:
            continue
        seen.add(action)
        deduplicated.append(action)
    return tuple(deduplicated)


_VALID_REACTIVE_CONDITION_TYPES: frozenset[str] = frozenset(
    {"scenario_var_truthy", "entity_absent"}
)


def _resolve_reactive_description(
    *,
    room_id: str,
    base_description: str,
    rooms: Mapping[str, Any],
    entities: Mapping[str, Any],
    scenario_vars: Mapping[str, Any],
) -> str:
    """Return a state-reactive description for a room, or the base description.

    Reads ``room_description_overrides_json`` from ``scenario_vars`` (written
    there during world setup).  Each override entry specifies:

    - ``room_id``          — which room this override applies to
    - ``condition_type``   — ``"scenario_var_truthy"`` or ``"entity_absent"``
    - For ``scenario_var_truthy``: ``condition_key`` (key in scenario_vars)
    - For ``entity_absent``: ``condition_room_id`` + ``condition_entity_id``
    - ``description``      — the override text to use when the condition is met

    The first matching override for the current room wins.
    Falls back to ``base_description`` when no override matches.
    """
    raw = scenario_vars.get("room_description_overrides_json")
    if not isinstance(raw, str):
        return base_description
    try:
        overrides = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return base_description
    if not isinstance(overrides, list):
        return base_description

    for override in overrides:
        if not isinstance(override, Mapping):
            continue
        if override.get("room_id") != room_id:
            continue
        condition_type = override.get("condition_type", "")
        if condition_type not in _VALID_REACTIVE_CONDITION_TYPES:
            continue
        override_text = override.get("description")
        if not isinstance(override_text, str) or not override_text:
            continue

        if condition_type == "scenario_var_truthy":
            key = str(override.get("condition_key", ""))
            if key and scenario_vars.get(key):
                return override_text

        elif condition_type == "entity_absent":
            check_room_id = str(override.get("condition_room_id", room_id))
            entity_id = str(override.get("condition_entity_id", ""))
            if not entity_id:
                continue
            check_room = rooms.get(check_room_id)
            if isinstance(check_room, Mapping):
                present = check_room.get("entities_present", [])
                if entity_id not in present:
                    return override_text

    return base_description
