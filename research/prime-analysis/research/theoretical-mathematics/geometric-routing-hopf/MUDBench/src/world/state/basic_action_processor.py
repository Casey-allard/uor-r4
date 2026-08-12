"""Concrete deterministic action processor for core Phase-1 actions."""

from __future__ import annotations

import json
from typing import Any, Mapping, Sequence

from core.action_processor import ActionProcessingResult, ActionProcessor, ActionRequest
from core.event_logger import EventRecord, normalize_payload
from core.world_state_manager import WorldStateManager
from world.combat.combat_rules import DeterministicCombatRules
from world.items.item_rules import DeterministicItemRules
from world.rooms.movement_rules import DeterministicMovementRules
from world.rooms.room_graph import DeterministicRoomGraph

from .world_state import DeterministicWorldStateManager

_SUPPORTED_ACTION_TYPES = frozenset(
    {"wait", "look", "move", "take", "drop", "use", "give", "attack", "talk", "defend"}
)

_WORLD_EVENT_LOG_KEY = "world_event_log_json"
_WORLD_EVENT_LOG_MAX_ENTRIES = 20
_LOGGABLE_EVENT_TYPES = frozenset(
    {
        "npc_defeated",
        "route_unlocked",
        "npc_alert",
        "npc_calmed",
        "npc_respawn",
        "actor_defeated",
        "actor_respawned",
        "actor_loot_dropped",
        "actor_revived",
    }
)

# Damage dealt by a hostile NPC to each actor in its room each world tick.
_NPC_COUNTER_DAMAGE = 5
# Reduced damage dealt to a defending actor (one who took the defend action this tick).
_NPC_COUNTER_DAMAGE_DEFENDED = 2
# Scenario-var key prefix for actor defending state (value = step_index of the defend action).
_ACTOR_DEFENDING_KEY_PREFIX = "defending."
# Entity types that are treated as attackable actors (can take counter-damage).
_ACTOR_ENTITY_TYPES = frozenset({"player", "agent"})
# Default actor health assumed when entity.health is None (matches observation_builder default).
_DEFAULT_ACTOR_HEALTH = 100

class BasicDeterministicActionProcessor(ActionProcessor):
    """Deterministic processor for a minimal Phase-1 action set."""

    def __init__(self) -> None:
        self._movement_rules = DeterministicMovementRules()
        self._item_rules = DeterministicItemRules()
        self._combat_rules = DeterministicCombatRules(damage=10)

    def validate_action(self, action: ActionRequest) -> bool:
        return self._validation_error(action) is None

    def process_actions(
        self,
        actions: Sequence[ActionRequest],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")
        if not isinstance(step_index, int) or step_index < 0:
            raise ValueError("step_index must be a non-negative integer")

        ordered_actions = tuple(sorted(actions, key=_action_sort_key))
        results: list[ActionProcessingResult] = []
        for action in ordered_actions:
            validation_error = self._validation_error(action)
            if validation_error is not None:
                results.append(
                    _rejected_action_result(
                        step_index=step_index,
                        actor_id=action.actor_id,
                        action_type=action.action_type,
                        reason=validation_error,
                    )
                )
                continue

            if action.action_type == "wait":
                results.append(
                    ActionProcessingResult(
                        accepted=True,
                        events=(
                            EventRecord(
                                step_index=step_index,
                                event_type="action_wait",
                                actor_id=action.actor_id,
                                payload=normalize_payload({"result": "no_state_change"}),
                            ),
                        ),
                    )
                )
                continue

            if action.action_type == "look":
                results.append(self._resolve_look(action, world_state=world_state, step_index=step_index))
                continue

            if action.action_type == "move":
                results.append(self._resolve_move(action, world_state=world_state, step_index=step_index))
                continue

            if action.action_type == "attack":
                results.append(
                    self._resolve_attack(action, world_state=world_state, step_index=step_index)
                )
                continue

            if action.action_type == "defend":
                results.append(
                    self._resolve_defend(action, world_state=world_state, step_index=step_index)
                )
                continue

            if action.action_type in {"take", "drop", "use", "give"}:
                results.append(
                    self._resolve_item_action(action, world_state=world_state, step_index=step_index)
                )
                continue

            if action.action_type == "talk":
                results.append(
                    self._resolve_talk(action, world_state=world_state, step_index=step_index)
                )
                continue

            results.append(
                _rejected_action_result(
                    step_index=step_index,
                    actor_id=action.actor_id,
                    action_type=action.action_type,
                    reason="unsupported_action_type",
                )
            )

        return tuple(results)

    def _resolve_look(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        snapshot = world_state.get_snapshot()
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")

        actor_payload = entities.get(action.actor_id)
        if not isinstance(actor_payload, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="actor_not_found",
            )

        location = actor_payload.get("location")
        if not isinstance(location, str) or not location:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="actor_has_no_location",
            )

        room_payload = rooms.get(location)
        if not isinstance(room_payload, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="source_room_missing",
            )

        exits = _require_mapping(room_payload.get("exits", {}), field_name="room.exits")
        visible_exits = tuple(sorted(str(direction) for direction in exits.keys()))
        event = EventRecord(
            step_index=step_index,
            event_type="action_look",
            actor_id=action.actor_id,
            payload=normalize_payload(
                {"location": location, "visible_exits": visible_exits}
            ),
        )
        reveal_key = f"reveal.{action.actor_id}.{location}"
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items({"scenario_vars": {reveal_key: 1}}),
        )

    def _resolve_move(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        direction = args["direction"]
        room_graph = _room_graph_from_snapshot(world_state.get_snapshot())
        resolution = self._movement_rules.apply_move(
            world_state,
            room_graph,
            actor_id=action.actor_id,
            direction=direction,
        )

        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "move_rejected",
            )

        event = EventRecord(
            step_index=step_index,
            event_type="action_move",
            actor_id=action.actor_id,
            payload=normalize_payload(
                {
                    "direction": direction,
                    "source_room_id": resolution.source_room_id,
                    "destination_room_id": resolution.destination_room_id,
                }
            ),
        )
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items(resolution.world_delta),
        )

    def _resolve_item_action(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        item_id = args["item_id"]
        if action.action_type == "take":
            resolution = self._item_rules.apply_take(
                world_state, actor_id=action.actor_id, item_id=item_id
            )
        elif action.action_type == "drop":
            resolution = self._item_rules.apply_drop(
                world_state, actor_id=action.actor_id, item_id=item_id
            )
        elif action.action_type == "give":
            resolution = self._item_rules.apply_give(
                world_state,
                actor_id=action.actor_id,
                item_id=item_id,
                target_id=str(args["target_id"]),
            )
        else:
            resolution = self._item_rules.apply_use(
                world_state, actor_id=action.actor_id, item_id=item_id
            )

        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "item_action_rejected",
            )

        payload: dict[str, Any] = {"item_id": item_id}
        if action.action_type == "give":
            payload["target_id"] = str(args["target_id"])
        if resolution.room_id is not None:
            payload["room_id"] = resolution.room_id
        if action.action_type == "use":
            payload["consumed"] = resolution.consumed

        event = EventRecord(
            step_index=step_index,
            event_type=f"action_{action.action_type}",
            actor_id=action.actor_id,
            payload=normalize_payload(payload),
        )
        events: list[EventRecord] = [event]
        metadata = {key: value for key, value in resolution.metadata}
        effect_id = metadata.get("effect_id")
        if isinstance(effect_id, str) and effect_id:
            unlock_payload = {"effect_id": effect_id, "item_id": item_id}
            if action.action_type == "give":
                unlock_payload["target_id"] = metadata.get("effect_target_id")
                unlock_payload["reward_item_id"] = metadata.get("effect_reward_item_id")
            else:
                unlock_payload["source_room_id"] = metadata.get("effect_source_room_id")
                unlock_payload["direction"] = metadata.get("effect_direction")
                unlock_payload["destination_room_id"] = metadata.get("effect_destination_room_id")
            events.append(
                EventRecord(
                    step_index=step_index,
                    event_type="dependency_unlocked",
                    actor_id=action.actor_id,
                    payload=normalize_payload(unlock_payload),
                )
            )

        # Calm-NPC effect: when a consumable is used, check whether the scenario defines a
        # ``calm_npc_effects_json`` entry that matches this item + room combination.  If found
        # and not already triggered, remove ``"hostile"`` from the target NPC and emit
        # ``npc_calmed``.  The effect is idempotent (guarded by ``calmed.<effect_id>``).
        extended_world_delta: dict[str, Any] = dict(resolution.world_delta) if resolution.world_delta else {}
        if action.action_type == "use" and resolution.consumed:
            snapshot_for_calm = world_state.get_snapshot()
            calm_effect = _find_calm_npc_effect(
                snapshot_for_calm.get("scenario_vars", {}),
                item_id=item_id,
                room_id=resolution.room_id,
            )
            if calm_effect is not None:
                calm_effect_id = str(calm_effect.get("effect_id", ""))
                target_npc_id = str(calm_effect.get("target_npc_id", ""))
                if calm_effect_id and target_npc_id:
                    already_calmed = bool(
                        snapshot_for_calm.get("scenario_vars", {}).get(f"calmed.{calm_effect_id}")
                    )
                    entities_snap: Mapping[str, Any] = snapshot_for_calm.get("entities", {})
                    target_npc = entities_snap.get(target_npc_id)
                    if (
                        not already_calmed
                        and isinstance(target_npc, Mapping)
                        and target_npc.get("entity_type") == "npc"
                        and "hostile" in target_npc.get("tags", [])
                    ):
                        npc_health = target_npc.get("health")
                        npc_alive = not isinstance(npc_health, int) or npc_health > 0
                        npc_room = target_npc.get("location")
                        if npc_alive and npc_room == resolution.room_id:
                            updated_npc_tags = sorted(
                                t for t in target_npc.get("tags", []) if t != "hostile"
                            )
                            entities_delta: dict[str, Any] = dict(extended_world_delta.get("entities") or {})
                            entities_delta[target_npc_id] = {
                                **dict(target_npc),
                                "entity_id": target_npc_id,
                                "tags": updated_npc_tags,
                            }
                            extended_world_delta["entities"] = entities_delta

                            # Mark calm as triggered (idempotent guard).
                            svars_delta: dict[str, Any] = dict(
                                extended_world_delta.get("scenario_vars") or {}
                            )
                            svars_delta[f"calmed.{calm_effect_id}"] = True

                            # Schedule respawn for the calmed NPC if a rule exists.
                            calm_respawn_rule = _find_npc_respawn_rule(
                                snapshot_for_calm.get("scenario_vars", {}), target_npc_id
                            )
                            if calm_respawn_rule is not None:
                                calm_delay = calm_respawn_rule.get("respawn_delay_ticks", 0)
                                if isinstance(calm_delay, int) and calm_delay > 0:
                                    svars_delta[f"respawn_at_tick.{target_npc_id}"] = (
                                        step_index + calm_delay
                                    )

                            calm_event = EventRecord(
                                step_index=step_index,
                                event_type="npc_calmed",
                                actor_id=action.actor_id,
                                payload=normalize_payload(
                                    {
                                        "target_id": target_npc_id,
                                        "item_id": item_id,
                                        "room_id": npc_room,
                                    }
                                ),
                            )
                            events.append(calm_event)

                            # Persist npc_calmed to the world event log.
                            svars_base = snapshot_for_calm.get("scenario_vars", {})
                            _update_world_event_log(svars_base, svars_delta, [calm_event])
                            extended_world_delta["scenario_vars"] = svars_delta

        # Actor-revive effect: when a consumable is used, check whether the scenario defines an
        # ``actor_revive_effects_json`` entry that matches this item.  If found and there is a
        # defeated actor in the same room, restore that actor to limited health, clear the defeat
        # state, cancel the pending respawn, and emit ``actor_revived``.  The revival targets the
        # alphabetically-first defeated co-actor in the room (deterministic selection).
        if action.action_type == "use" and resolution.consumed:
            snapshot_for_revive = world_state.get_snapshot()
            revive_effect = _find_actor_revive_effect(
                snapshot_for_revive.get("scenario_vars", {}),
                item_id=item_id,
            )
            if revive_effect is not None:
                revive_effect_id = str(revive_effect.get("effect_id", ""))
                revive_hp_raw = revive_effect.get("revive_health", _DEFAULT_ACTOR_REVIVE_HEALTH)
                revive_hp = int(revive_hp_raw) if isinstance(revive_hp_raw, int) else _DEFAULT_ACTOR_REVIVE_HEALTH
                revive_room = resolution.room_id
                svars_snap = snapshot_for_revive.get("scenario_vars", {})
                entities_snap2: Mapping[str, Any] = snapshot_for_revive.get("entities", {})

                # Find defeated co-actors in the same room (sorted for determinism).
                defeated_in_room = sorted(
                    eid
                    for eid, edata in entities_snap2.items()
                    if isinstance(edata, Mapping)
                    and edata.get("entity_type") in _ACTOR_ENTITY_TYPES
                    and eid != action.actor_id
                    and bool(svars_snap.get(f"actor_defeated.{eid}"))
                    and edata.get("location") == revive_room
                )
                if defeated_in_room:
                    target_actor_id = defeated_in_room[0]

                    rev_entities_delta: dict[str, Any] = dict(
                        extended_world_delta.get("entities") or {}
                    )
                    existing_entity: Mapping[str, Any] = (
                        rev_entities_delta.get(target_actor_id)
                        or dict(entities_snap2.get(target_actor_id) or {})
                    )
                    rev_entities_delta[target_actor_id] = {
                        **dict(existing_entity),
                        "entity_id": target_actor_id,
                        "health": revive_hp,
                    }
                    extended_world_delta["entities"] = rev_entities_delta

                    rev_svars_delta: dict[str, Any] = dict(
                        extended_world_delta.get("scenario_vars") or {}
                    )
                    rev_svars_delta[f"actor_defeated.{target_actor_id}"] = None
                    rev_svars_delta[f"actor_respawn_at.{target_actor_id}"] = None

                    revive_event = EventRecord(
                        step_index=step_index,
                        event_type="actor_revived",
                        actor_id=action.actor_id,
                        payload=normalize_payload(
                            {
                                "target_id": target_actor_id,
                                "item_id": item_id,
                                "room_id": revive_room,
                                "revive_health": revive_hp,
                                "effect_id": revive_effect_id,
                            }
                        ),
                    )
                    events.append(revive_event)

                    svars_base2 = snapshot_for_revive.get("scenario_vars", {})
                    _update_world_event_log(svars_base2, rev_svars_delta, [revive_event])
                    extended_world_delta["scenario_vars"] = rev_svars_delta

        return ActionProcessingResult(
            accepted=True,
            events=tuple(events),
            world_delta=_world_delta_to_items(extended_world_delta),
        )

    def _resolve_attack(
        self,
        action: ActionRequest,
        *,
        world_state: DeterministicWorldStateManager,
        step_index: int,
    ) -> ActionProcessingResult:
        args = _arguments_to_dict(action.arguments)
        if args is None:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason="invalid_arguments",
            )

        target_id = args["target_id"]
        resolution = self._combat_rules.apply_attack(
            world_state,
            attacker_id=action.actor_id,
            target_id=target_id,
        )
        if not resolution.success:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type=action.action_type,
                reason=resolution.reason or "attack_rejected",
            )

        payload = {
            "target_id": target_id,
            "damage": resolution.damage,
            "resulting_health": resolution.resulting_health,
            "room_id": resolution.room_id,
        }
        event = EventRecord(
            step_index=step_index,
            event_type="action_attack",
            actor_id=action.actor_id,
            payload=normalize_payload(payload),
        )

        # Build extended world delta: base entity health update from combat_rules, plus
        # shared persistent defeat state when health reaches zero.
        extended_delta: dict[str, Any] = dict(resolution.world_delta) if resolution.world_delta else {}
        events: list[EventRecord] = [event]

        # Persistent NPC hostile/alert state: mark NPC hostile on first attack that does not
        # defeat it. Stored in the entity's ``tags`` tuple (persists through full snapshot
        # roundtrip). Emit ``npc_alert`` once (idempotent: skip if already hostile).
        if (
            resolution.resulting_health is not None
            and resolution.resulting_health > 0
            and resolution.room_id is not None
        ):
            snapshot_for_hostile = world_state.get_snapshot()
            target_entity_pre = snapshot_for_hostile.get("entities", {}).get(target_id, {})
            existing_tags = list(target_entity_pre.get("tags", []))
            already_hostile = "hostile" in existing_tags
            if not already_hostile:
                entities_delta = extended_delta.get("entities")
                if not isinstance(entities_delta, dict):
                    entities_delta = {}
                existing_target_delta = dict(entities_delta.get(target_id, target_entity_pre))
                existing_target_delta["entity_id"] = target_id
                updated_tags = sorted(set(existing_tags) | {"hostile"})
                existing_target_delta["tags"] = updated_tags
                entities_delta[target_id] = existing_target_delta
                extended_delta["entities"] = entities_delta
                events.append(
                    EventRecord(
                        step_index=step_index,
                        event_type="npc_alert",
                        actor_id=action.actor_id,
                        payload=normalize_payload(
                            {
                                "target_id": target_id,
                                "room_id": resolution.room_id,
                                "remaining_health": resolution.resulting_health,
                            }
                        ),
                    )
                )
                # Persist npc_alert to the world event log.
                svars_base = snapshot_for_hostile.get("scenario_vars", {})
                svars_delta = extended_delta.get("scenario_vars")
                if not isinstance(svars_delta, dict):
                    svars_delta = {}
                _update_world_event_log(
                    svars_base, svars_delta,
                    [events[-1]],  # the npc_alert event just appended
                )
                extended_delta["scenario_vars"] = svars_delta

        if resolution.resulting_health == 0 and resolution.room_id is not None:
            snapshot = world_state.get_snapshot()
            rooms = _require_mapping(snapshot.get("rooms", {}), field_name="world.rooms")
            room_payload = rooms.get(resolution.room_id)
            if isinstance(room_payload, Mapping):
                updated_room = dict(room_payload)
                updated_room["entities_present"] = sorted(
                    e for e in room_payload.get("entities_present", []) if e != target_id
                )
                rooms_delta = extended_delta.get("rooms")
                if not isinstance(rooms_delta, dict):
                    rooms_delta = {}
                rooms_delta[resolution.room_id] = updated_room
                extended_delta["rooms"] = rooms_delta

            scenario_vars_delta = extended_delta.get("scenario_vars")
            if not isinstance(scenario_vars_delta, dict):
                scenario_vars_delta = {}
            scenario_vars_delta[f"defeated.{target_id}"] = True
            extended_delta["scenario_vars"] = scenario_vars_delta

            # Schedule respawn if this NPC has a respawn rule.
            respawn_rule = _find_npc_respawn_rule(snapshot.get("scenario_vars", {}), target_id)
            if respawn_rule is not None:
                delay = respawn_rule.get("respawn_delay_ticks", 0)
                if isinstance(delay, int) and delay > 0:
                    scenario_vars_delta = extended_delta.get("scenario_vars")
                    if not isinstance(scenario_vars_delta, dict):
                        scenario_vars_delta = {}
                    scenario_vars_delta[f"respawn_at_tick.{target_id}"] = step_index + delay
                    extended_delta["scenario_vars"] = scenario_vars_delta

            events.append(
                EventRecord(
                    step_index=step_index,
                    event_type="npc_defeated",
                    actor_id=action.actor_id,
                    payload=normalize_payload(
                        {"target_id": target_id, "room_id": resolution.room_id}
                    ),
                )
            )

            # Defeat-triggered route unlocks
            defeat_effect = _find_defeat_unlock_effect(snapshot.get("scenario_vars", {}), target_id)
            if defeat_effect is not None:
                effect_id = defeat_effect.get("effect_id", "")
                src_room_id = defeat_effect.get("source_room_id", "")
                direction = defeat_effect.get("direction", "")
                dst_room_id = defeat_effect.get("destination_room_id", "")
                if effect_id and src_room_id and direction and dst_room_id:
                    already_triggered = bool(
                        snapshot.get("scenario_vars", {}).get(f"unlock.{effect_id}")
                    )
                    if not already_triggered:
                        rooms_delta = extended_delta.get("rooms")
                        if not isinstance(rooms_delta, dict):
                            rooms_delta = {}
                        # Prefer already-modified room (e.g. NPC removed) over snapshot
                        src_room_payload = rooms_delta.get(src_room_id) or rooms.get(src_room_id)
                        if isinstance(src_room_payload, Mapping):
                            existing_exits = src_room_payload.get("exits", {})
                            if direction not in existing_exits:
                                updated_src = dict(src_room_payload)
                                updated_src["exits"] = {**existing_exits, direction: dst_room_id}
                                rooms_delta[src_room_id] = updated_src
                                extended_delta["rooms"] = rooms_delta
                        scenario_vars_delta = extended_delta.get("scenario_vars")
                        if not isinstance(scenario_vars_delta, dict):
                            scenario_vars_delta = {}
                        scenario_vars_delta[f"unlock.{effect_id}"] = True
                        extended_delta["scenario_vars"] = scenario_vars_delta
                        events.append(
                            EventRecord(
                                step_index=step_index,
                                event_type="route_unlocked",
                                actor_id=action.actor_id,
                                payload=normalize_payload({
                                    "effect_id": effect_id,
                                    "source_room_id": src_room_id,
                                    "direction": direction,
                                    "destination_room_id": dst_room_id,
                                }),
                            )
                        )

            # Persist loggable events (npc_defeated, route_unlocked) to world event log
            log_events = [e for e in events if e.event_type in _LOGGABLE_EVENT_TYPES]
            if log_events:
                svars_delta = extended_delta.get("scenario_vars", {})
                if not isinstance(svars_delta, dict):
                    svars_delta = {}
                _update_world_event_log(snapshot.get("scenario_vars", {}), svars_delta, log_events)
                extended_delta["scenario_vars"] = svars_delta

        return ActionProcessingResult(
            accepted=True,
            events=tuple(events),
            world_delta=_world_delta_to_items(extended_delta),
        )

    def _resolve_defend(
        self,
        action: ActionRequest,
        *,
        world_state: "DeterministicWorldStateManager",
        step_index: int,
    ) -> "ActionProcessingResult":
        """Process a ``defend`` stance action.

        The actor braces for incoming attacks this tick.  Any hostile NPC
        counter-attack against this actor this tick deals ``_NPC_COUNTER_DAMAGE_DEFENDED``
        instead of the normal ``_NPC_COUNTER_DAMAGE``.

        The defending state is stored as ``defending.{actor_id} = step_index`` in
        scenario_vars.  Because it encodes the step index, it is automatically
        per-tick with no cleanup required — stale entries from earlier ticks are
        silently ignored by the counter-attack sweep.

        Always succeeds (no target validation needed).
        """
        defend_key = f"{_ACTOR_DEFENDING_KEY_PREFIX}{action.actor_id}"
        world_delta: dict[str, Any] = {
            "scenario_vars": {defend_key: step_index},
        }
        event = EventRecord(
            step_index=step_index,
            event_type="action_defend",
            actor_id=action.actor_id,
            payload=normalize_payload({"actor_id": action.actor_id, "step_index": step_index}),
        )
        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items(world_delta),
        )

    def _resolve_talk(
        self,
        action: ActionRequest,
        *,
        world_state: "DeterministicWorldStateManager",
        step_index: int,
    ) -> "ActionProcessingResult":
        """Process a ``talk <npc_id>`` interaction.

        Succeeds when the target NPC is:
        - present in the actor's current room
        - not hostile (``"hostile"`` not in NPC tags)
        - not defeated (health > 0)

        On success, returns one line from ``npc_dialogue_json`` and increments
        ``talk_count.<npc_id>`` in scenario_vars to cycle through dialogue.
        """
        args = _arguments_to_dict(action.arguments) or {}
        target_id = str(args.get("target_id", ""))

        snapshot = world_state.get_snapshot()
        entities: Mapping[str, Any] = snapshot.get("entities", {})
        rooms: Mapping[str, Any] = snapshot.get("rooms", {})
        scenario_vars: Mapping[str, Any] = snapshot.get("scenario_vars", {})

        actor = entities.get(action.actor_id)
        if not isinstance(actor, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="actor_not_found",
            )
        actor_location = str(actor.get("location", ""))

        room_payload = rooms.get(actor_location)
        if not isinstance(room_payload, Mapping):
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="actor_room_not_found",
            )
        room_present = list(room_payload.get("entities_present", []))
        if target_id not in room_present:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="target_not_in_room",
            )

        npc_payload = entities.get(target_id)
        if not isinstance(npc_payload, Mapping) or str(npc_payload.get("entity_type", "")) != "npc":
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="target_not_an_npc",
            )
        npc_tags = list(npc_payload.get("tags") or [])
        if "hostile" in npc_tags:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="npc_is_hostile",
            )
        npc_health = npc_payload.get("health")
        if isinstance(npc_health, int) and npc_health <= 0:
            return _rejected_action_result(
                step_index=step_index,
                actor_id=action.actor_id,
                action_type="talk",
                reason="npc_is_defeated",
            )

        dialogue = _find_npc_dialogue(scenario_vars, target_id)
        if not dialogue:
            line = f"{target_id} has nothing to say."
        else:
            count_key = f"talk_count.{target_id}"
            count = scenario_vars.get(count_key)
            idx = int(count) % len(dialogue) if isinstance(count, int) else 0
            line = dialogue[idx]
            next_count = idx + 1

        svars_delta: dict[str, Any] = {}
        if dialogue:
            svars_delta[f"talk_count.{target_id}"] = next_count

        event = EventRecord(
            step_index=step_index,
            event_type="npc_talked",
            actor_id=action.actor_id,
            payload=normalize_payload(
                {
                    "target_id": target_id,
                    "line": line,
                    "room_id": actor_location,
                }
            ),
        )

        world_delta: dict[str, Any] = {}
        if svars_delta:
            world_delta["scenario_vars"] = svars_delta

        return ActionProcessingResult(
            accepted=True,
            events=(event,),
            world_delta=_world_delta_to_items(world_delta) if world_delta else (),
        )

    @staticmethod
    def _validation_error(action: ActionRequest) -> str | None:
        if not isinstance(action.actor_id, str) or not action.actor_id:
            return "invalid_actor_id"
        if not isinstance(action.action_type, str) or action.action_type not in _SUPPORTED_ACTION_TYPES:
            return "unsupported_action_type"

        args = _arguments_to_dict(action.arguments)
        if args is None:
            return "invalid_arguments"

        if action.action_type in {"wait", "look", "defend"}:
            if args:
                return "unexpected_arguments"
            return None

        if action.action_type == "move":
            if set(args.keys()) != {"direction"}:
                return "move_requires_direction"
            if not isinstance(args["direction"], str) or not args["direction"]:
                return "move_direction_invalid"
            return None

        if action.action_type == "attack":
            if set(args.keys()) != {"target_id"}:
                return "attack_requires_target_id"
            if not isinstance(args["target_id"], str) or not args["target_id"]:
                return "target_id_invalid"
            return None

        if action.action_type == "talk":
            if set(args.keys()) != {"target_id"}:
                return "talk_requires_target_id"
            if not isinstance(args["target_id"], str) or not args["target_id"]:
                return "target_id_invalid"
            return None

        if action.action_type == "give":
            if set(args.keys()) != {"item_id", "target_id"}:
                return "give_requires_item_and_target"
            if not isinstance(args["item_id"], str) or not args["item_id"]:
                return "item_id_invalid"
            if not isinstance(args["target_id"], str) or not args["target_id"]:
                return "target_id_invalid"
            return None

        if set(args.keys()) != {"item_id"}:
            return "item_action_requires_item_id"
        if not isinstance(args["item_id"], str) or not args["item_id"]:
            return "item_id_invalid"
        return None


def _action_sort_key(action: ActionRequest) -> tuple[str, str, tuple[tuple[str, str], ...]]:
    return (
        str(action.actor_id),
        str(action.action_type),
        tuple((str(key), repr(value)) for key, value in action.arguments),
    )


def _arguments_to_dict(arguments: Any) -> dict[str, Any] | None:
    if not isinstance(arguments, Sequence):
        return None
    if isinstance(arguments, (str, bytes)):
        return None

    parsed: dict[str, Any] = {}
    for item in arguments:
        if not isinstance(item, tuple) or len(item) != 2:
            return None
        key, value = item
        if not isinstance(key, str) or not key:
            return None
        if key in parsed:
            return None
        parsed[key] = value
    return parsed


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _room_graph_from_snapshot(snapshot: Mapping[str, Any]) -> DeterministicRoomGraph:
    rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")
    return DeterministicRoomGraph.from_dict({"rooms": dict(rooms)})


def _update_world_event_log(
    scenario_vars: Any,
    scenario_vars_delta: dict[str, Any],
    new_events: Sequence[EventRecord],
) -> None:
    """Append loggable events to the persistent world event log stored in scenario_vars_delta.

    The log is capped at ``_WORLD_EVENT_LOG_MAX_ENTRIES`` entries; oldest entries are
    dropped when the cap is exceeded.  Only event types in ``_LOGGABLE_EVENT_TYPES``
    are written — all others are silently ignored.
    """
    loggable = [e for e in new_events if e.event_type in _LOGGABLE_EVENT_TYPES]
    if not loggable:
        return

    raw = scenario_vars.get(_WORLD_EVENT_LOG_KEY) if isinstance(scenario_vars, Mapping) else None
    try:
        existing: list[Any] = json.loads(raw) if isinstance(raw, str) else []
    except (json.JSONDecodeError, ValueError):
        existing = []
    if not isinstance(existing, list):
        existing = []

    new_entries: list[dict[str, Any]] = []
    for event in loggable:
        entry: dict[str, Any] = {
            "event_type": event.event_type,
            "step": event.step_index,
        }
        if event.actor_id:
            entry["actor_id"] = event.actor_id
        for k, v in (event.payload or ()):
            if k not in entry:
                entry[k] = v
        new_entries.append(entry)

    combined = existing + new_entries
    if len(combined) > _WORLD_EVENT_LOG_MAX_ENTRIES:
        combined = combined[-_WORLD_EVENT_LOG_MAX_ENTRIES:]
    scenario_vars_delta[_WORLD_EVENT_LOG_KEY] = json.dumps(
        combined, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    )


def _find_defeat_unlock_effect(
    scenario_vars: Any, npc_id: str
) -> dict[str, Any] | None:
    """Return the defeat-unlock effect definition for *npc_id*, or None if absent."""
    if not isinstance(scenario_vars, Mapping):
        return None
    raw = scenario_vars.get("defeat_unlock_effects_json")
    if not isinstance(raw, str):
        return None
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(parsed, Mapping):
        return None
    effect = parsed.get(npc_id)
    if not isinstance(effect, Mapping):
        return None
    return dict(effect)


def _find_calm_npc_effect(
    scenario_vars: Any,
    *,
    item_id: str,
    room_id: str | None,
) -> dict[str, Any] | None:
    """Return the calm-NPC effect definition for *item_id* used in *room_id*, or None.

    Looks up ``calm_npc_effects_json`` in *scenario_vars* (a JSON-encoded list of
    effect dicts).  Each entry must have:
    - ``"item_id"``       — the calming consumable
    - ``"source_room_id"`` — the room where the item must be used
    - ``"target_npc_id"``  — the NPC to de-escalate
    - ``"effect_id"``      — unique idempotency key (``calmed.<effect_id>``)
    """
    if not isinstance(scenario_vars, Mapping):
        return None
    if not isinstance(room_id, str) or not room_id:
        return None
    raw = scenario_vars.get("calm_npc_effects_json")
    if not isinstance(raw, str):
        return None
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(parsed, list):
        return None
    for entry in parsed:
        if not isinstance(entry, Mapping):
            continue
        if str(entry.get("item_id", "")) == item_id and str(entry.get("source_room_id", "")) == room_id:
            return dict(entry)
    return None


def _find_actor_revive_effect(
    scenario_vars: Any,
    *,
    item_id: str,
) -> dict[str, Any] | None:
    """Return the actor-revive effect definition for *item_id*, or None.

    Looks up ``actor_revive_effects_json`` in *scenario_vars* (a JSON-encoded list
    of effect dicts).  Each entry must have:

    - ``"item_id"``    — the revive consumable
    - ``"effect_id"``  — unique idempotency key (not reused per-use; just for logging)
    - ``"revive_health"`` — (optional) HP to restore; defaults to
      ``_DEFAULT_ACTOR_REVIVE_HEALTH``

    Unlike calm-NPC effects there is no ``source_room_id`` restriction — a revive
    kit can be used in any room where a defeated actor is present.
    """
    if not isinstance(scenario_vars, Mapping):
        return None
    raw = scenario_vars.get("actor_revive_effects_json")
    if not isinstance(raw, str):
        return None
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(parsed, list):
        return None
    for entry in parsed:
        if not isinstance(entry, Mapping):
            continue
        if str(entry.get("item_id", "")) == item_id:
            return dict(entry)
    return None


def _find_npc_dialogue(
    scenario_vars: Any,
    npc_id: str,
) -> list[str]:
    """Return the ordered dialogue lines for *npc_id* from ``npc_dialogue_json``.

    Returns an empty list when no dialogue is configured for the NPC.

    Dialogue is stored as ``npc_dialogue_json`` in scenario_vars — a JSON-encoded
    list of ``{"npc_id": ..., "lines": [...]}`` entries.
    """
    if not isinstance(scenario_vars, Mapping):
        return []
    raw = scenario_vars.get("npc_dialogue_json")
    if not isinstance(raw, str):
        return []
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return []
    if not isinstance(parsed, list):
        return []
    for entry in parsed:
        if isinstance(entry, Mapping) and str(entry.get("npc_id", "")) == npc_id:
            lines = entry.get("lines")
            if isinstance(lines, list):
                return [str(line) for line in lines if isinstance(line, str) and line]
    return []


def process_npc_tick(
    world_state: "DeterministicWorldStateManager",
    *,
    step_index: int,
) -> "ActionProcessingResult | None":
    """Execute one hostile-NPC counter-attack sweep for the current world tick.

    For every hostile NPC (``"hostile"`` in entity tags, health > 0) that shares
    a room with one or more actor entities (entity_type in ``_ACTOR_ENTITY_TYPES``),
    deal ``_NPC_COUNTER_DAMAGE`` HP to each such actor.

    Returns a single ``ActionProcessingResult`` bundling all health deltas and
    ``npc_counter_attack`` events, or ``None`` when no hostile NPCs are present.
    The caller is responsible for applying the delta to the world state.
    """
    from .world_state import DeterministicWorldStateManager as _DSM  # local to avoid circular

    if not isinstance(world_state, _DSM):
        return None

    snapshot = world_state.get_snapshot()
    entities: Mapping[str, Any] = snapshot.get("entities", {})

    # Build room → actor mapping once for efficiency.
    room_to_actors: dict[str, list[str]] = {}
    for entity_id, entity in entities.items():
        if not isinstance(entity, Mapping):
            continue
        if entity.get("entity_type") not in _ACTOR_ENTITY_TYPES:
            continue
        loc = entity.get("location")
        if isinstance(loc, str) and loc:
            health = entity.get("health")
            # Treat None health as default (actor has not taken damage yet).
            effective_health = health if isinstance(health, int) else _DEFAULT_ACTOR_HEALTH
            if effective_health > 0:
                room_to_actors.setdefault(loc, []).append(entity_id)

    if not room_to_actors:
        return None

    entity_delta: dict[str, Any] = {}
    events: list[EventRecord] = []

    for npc_id, npc in sorted(entities.items()):
        if not isinstance(npc, Mapping):
            continue
        if npc.get("entity_type") != "npc":
            continue
        npc_health = npc.get("health")
        if not isinstance(npc_health, int) or npc_health <= 0:
            continue
        if "hostile" not in npc.get("tags", []):
            continue

        npc_room = npc.get("location")
        if not isinstance(npc_room, str) or not npc_room:
            continue

        actors_here = room_to_actors.get(npc_room, [])
        if not actors_here:
            continue

        for actor_id in sorted(actors_here):
            actor = entities.get(actor_id)
            if not isinstance(actor, Mapping):
                continue
            # Use already-accumulated delta HP if this actor was hit by a previous NPC.
            # Fall back to _DEFAULT_ACTOR_HEALTH when entity.health is None.
            raw_hp = entity_delta.get(actor_id, {}).get("health", actor.get("health"))
            current_hp = raw_hp if isinstance(raw_hp, int) else _DEFAULT_ACTOR_HEALTH
            if current_hp <= 0:
                continue
            # Defending actors take reduced damage this tick.
            defend_key = f"{_ACTOR_DEFENDING_KEY_PREFIX}{actor_id}"
            is_defending = snapshot.get("scenario_vars", {}).get(defend_key) == step_index
            damage = _NPC_COUNTER_DAMAGE_DEFENDED if is_defending else _NPC_COUNTER_DAMAGE
            new_hp = max(0, current_hp - damage)
            actor_payload = dict(entity_delta.get(actor_id, dict(actor)))
            actor_payload["entity_id"] = actor_id
            actor_payload["health"] = new_hp
            entity_delta[actor_id] = actor_payload
            events.append(
                EventRecord(
                    step_index=step_index,
                    event_type="npc_counter_attack",
                    actor_id=npc_id,
                    payload=normalize_payload(
                        {
                            "npc_id": npc_id,
                            "target_id": actor_id,
                            "damage": damage,
                            "resulting_health": new_hp,
                            "room_id": npc_room,
                            "defended": is_defending,
                        }
                    ),
                )
            )

    if not events:
        return None

    world_delta = {"entities": entity_delta}
    return ActionProcessingResult(
        accepted=True,
        events=tuple(events),
        world_delta=_world_delta_to_items(world_delta),
    )


def _find_npc_respawn_rule(
    scenario_vars: Any,
    npc_id: str,
) -> dict[str, Any] | None:
    """Return the respawn rule for *npc_id* from ``npc_respawn_rules_json``, or None.

    Each rule entry must have at minimum:
    - ``"npc_id"``               — identifies the NPC
    - ``"respawn_delay_ticks"``  — ticks after defeat/calm before respawn
    - ``"respawn_health"``        — HP the NPC is restored to
    - ``"respawn_room_id"``       — room the NPC reappears in

    Optional:
    - ``"respawn_tags"``          — tags for the respawned NPC (default: [])
    """
    if not isinstance(scenario_vars, Mapping):
        return None
    raw = scenario_vars.get("npc_respawn_rules_json")
    if not isinstance(raw, str):
        return None
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return None
    if not isinstance(parsed, list):
        return None
    for entry in parsed:
        if isinstance(entry, Mapping) and str(entry.get("npc_id", "")) == npc_id:
            return dict(entry)
    return None


def process_npc_respawn_tick(
    world_state: "DeterministicWorldStateManager",
    *,
    step_index: int,
) -> "ActionProcessingResult | None":
    """Restore NPCs whose scheduled respawn tick has arrived.

    For each NPC with ``respawn_at_tick.<npc_id>`` in scenario_vars where
    ``step_index >= respawn_at_tick.<npc_id>``:

    1. Restore the NPC entity to its configured health and tags.
    2. Move it back to its respawn room (adding it to the room's entities_present).
    3. Clear ``defeated.<npc_id>`` and ``respawn_at_tick.<npc_id>`` from scenario_vars.
    4. Clear any ``calmed.<effect_id>`` markers that targeted this NPC (via
       ``npc_respawn_rules_json`` ``clears_calm_effects`` list).
    5. Emit ``npc_respawn`` event.

    Returns ``None`` when no respawn is due this tick.
    """
    from .world_state import DeterministicWorldStateManager as _DSM  # local to avoid circular

    if not isinstance(world_state, _DSM):
        return None

    snapshot = world_state.get_snapshot()
    scenario_vars: Mapping[str, Any] = snapshot.get("scenario_vars", {})
    entities: Mapping[str, Any] = snapshot.get("entities", {})
    rooms: Mapping[str, Any] = snapshot.get("rooms", {})

    # Collect all npc_ids with a scheduled respawn at or before this tick.
    due: list[str] = []
    for key, value in scenario_vars.items():
        if not isinstance(key, str) or not key.startswith("respawn_at_tick."):
            continue
        npc_id = key[len("respawn_at_tick."):]
        if isinstance(value, int) and step_index >= value:
            due.append(npc_id)

    if not due:
        return None

    entity_delta: dict[str, Any] = {}
    rooms_delta: dict[str, Any] = {}
    svars_delta: dict[str, Any] = {}
    events: list[EventRecord] = []

    for npc_id in sorted(due):
        rule = _find_npc_respawn_rule(scenario_vars, npc_id)
        if rule is None:
            continue

        respawn_health = rule.get("respawn_health")
        respawn_room_id = str(rule.get("respawn_room_id", ""))
        respawn_tags = list(rule.get("respawn_tags", []))

        if not isinstance(respawn_health, int) or respawn_health <= 0:
            continue
        if not respawn_room_id:
            continue

        # Restore NPC entity.
        npc_current = entities.get(npc_id)
        npc_base: dict[str, Any] = dict(npc_current) if isinstance(npc_current, Mapping) else {}
        npc_base["entity_id"] = npc_id
        npc_base["entity_type"] = "npc"
        npc_base["health"] = respawn_health
        npc_base["location"] = respawn_room_id
        npc_base["tags"] = sorted(respawn_tags)
        npc_base["inventory"] = []
        entity_delta[npc_id] = npc_base

        # Add NPC back to room's entities_present.
        room_payload = rooms_delta.get(respawn_room_id) or rooms.get(respawn_room_id)
        if isinstance(room_payload, Mapping):
            updated_room = dict(room_payload)
            existing_present = list(updated_room.get("entities_present", []))
            if npc_id not in existing_present:
                updated_room["entities_present"] = sorted(existing_present + [npc_id])
            rooms_delta[respawn_room_id] = updated_room

        # Clear defeat/calm/respawn markers from scenario_vars.
        svars_delta[f"defeated.{npc_id}"] = None  # None → key deleted on apply
        svars_delta[f"respawn_at_tick.{npc_id}"] = None
        # Also clear any calm markers listed in the rule.
        for calm_effect_id in rule.get("clears_calm_effects", []):
            if isinstance(calm_effect_id, str) and calm_effect_id:
                svars_delta[f"calmed.{calm_effect_id}"] = None

        event = EventRecord(
            step_index=step_index,
            event_type="npc_respawn",
            actor_id=None,
            payload=normalize_payload(
                {
                    "npc_id": npc_id,
                    "room_id": respawn_room_id,
                    "health": respawn_health,
                    "tags": respawn_tags,
                }
            ),
        )
        events.append(event)

    if not events:
        return None

    # Persist npc_respawn events to the world event log.
    _update_world_event_log(scenario_vars, svars_delta, events)

    world_delta: dict[str, Any] = {"entities": entity_delta, "scenario_vars": svars_delta}
    if rooms_delta:
        world_delta["rooms"] = rooms_delta

    return ActionProcessingResult(
        accepted=True,
        events=tuple(events),
        world_delta=_world_delta_to_items(world_delta),
    )


# ---------------------------------------------------------------------------
# Quest objective tracking
# ---------------------------------------------------------------------------

# Map from event_type → key in event payload that identifies the "target" for
# trigger matching.  Only event types listed here can serve as quest triggers.
_QUEST_TRIGGER_PAYLOAD_KEYS: dict[str, str] = {
    "npc_defeated": "target_id",
    "npc_calmed": "target_id",
    "action_take": "item_id",
    "route_unlocked": "destination_room_id",
}


def _find_quest_objectives(scenario_vars: Any) -> list[dict[str, Any]]:
    """Return the ordered list of quest objective defs from ``quest_objectives_json``.

    Each entry must have at minimum:
    - ``"quest_id"``           — unique identifier
    - ``"title"``              — human-readable quest name
    - ``"trigger_event"``      — event_type that completes the quest
    - ``"trigger_target_id"``  — expected payload value that must match

    Optional:
    - ``"reward_message"``     — shown to actor on completion

    Returns empty list when no quest objectives are configured.
    """
    if not isinstance(scenario_vars, Mapping):
        return []
    raw = scenario_vars.get("quest_objectives_json")
    if not isinstance(raw, str):
        return []
    try:
        parsed = json.loads(raw)
    except (json.JSONDecodeError, ValueError):
        return []
    if not isinstance(parsed, list):
        return []
    result: list[dict[str, Any]] = []
    for entry in parsed:
        if not isinstance(entry, Mapping):
            continue
        quest_id = entry.get("quest_id")
        title = entry.get("title")
        trigger_event = entry.get("trigger_event")
        trigger_target_id = entry.get("trigger_target_id")
        if (
            not isinstance(quest_id, str) or not quest_id
            or not isinstance(title, str) or not title
            or not isinstance(trigger_event, str) or trigger_event not in _QUEST_TRIGGER_PAYLOAD_KEYS
            or not isinstance(trigger_target_id, str) or not trigger_target_id
        ):
            continue
        result.append(dict(entry))
    return result


def process_quest_objective_tick(
    world_state: "DeterministicWorldStateManager",
    events: "Sequence[EventRecord]",
    *,
    step_index: int,
) -> "ActionProcessingResult | None":
    """Check each event against configured quest objectives and update per-actor state.

    For each event that matches a quest objective's trigger:
    1. If the event has an ``actor_id``, mark that actor's quest as ``"complete"``.
    2. Emit a ``quest_completed`` event with quest metadata.

    Returns ``None`` when no quest objectives are configured or no events match.

    Quest state is stored in ``scenario_vars`` as ``quest.<quest_id>.<actor_id>``.
    The value ``"complete"`` means completed; absent / other values mean in-progress.
    """
    from .world_state import DeterministicWorldStateManager as _DSM
    if not isinstance(world_state, _DSM):
        return None

    snapshot = world_state.get_snapshot()
    scenario_vars: Mapping[str, Any] = snapshot.get("scenario_vars", {})
    objectives = _find_quest_objectives(scenario_vars)
    if not objectives:
        return None

    if not events:
        return None

    svars_delta: dict[str, Any] = {}
    emitted: list[EventRecord] = []

    for event in events:
        if event.actor_id is None:
            continue
        event_type = event.event_type
        payload_key = _QUEST_TRIGGER_PAYLOAD_KEYS.get(event_type)
        if payload_key is None:
            continue
        payload = {k: v for k, v in (event.payload or ())}
        trigger_value = payload.get(payload_key)
        if not isinstance(trigger_value, str):
            continue

        for obj in objectives:
            if obj.get("trigger_event") != event_type:
                continue
            if obj.get("trigger_target_id") != trigger_value:
                continue
            quest_id = str(obj["quest_id"])
            actor_id = str(event.actor_id)
            quest_key = f"quest.{quest_id}.{actor_id}"
            # Don't re-complete an already completed quest.
            if scenario_vars.get(quest_key) == "complete":
                continue
            if svars_delta.get(quest_key) == "complete":
                continue
            svars_delta[quest_key] = "complete"
            emitted.append(
                EventRecord(
                    step_index=step_index,
                    event_type="quest_completed",
                    actor_id=actor_id,
                    payload=normalize_payload(
                        {
                            "quest_id": quest_id,
                            "title": str(obj.get("title", quest_id)),
                            "reward_message": str(obj.get("reward_message", "")),
                        }
                    ),
                )
            )

    if not emitted:
        return None

    return ActionProcessingResult(
        accepted=True,
        events=tuple(emitted),
        world_delta=_world_delta_to_items({"scenario_vars": svars_delta}),
    )


# ---------------------------------------------------------------------------
# Actor defeat detection and respawn
# ---------------------------------------------------------------------------

# Scenario var keys used by actor defeat/respawn mechanics.
_ACTOR_RESPAWN_DELAY_KEY = "actor_respawn_delay_ticks"
_ACTOR_RESPAWN_HEALTH_KEY = "actor_respawn_health"
_ACTOR_RESPAWN_ROOM_KEY = "actor_respawn_room_id"

_DEFAULT_ACTOR_RESPAWN_DELAY = 3
_DEFAULT_ACTOR_RESPAWN_HEALTH = 50
_DEFAULT_ACTOR_REVIVE_HEALTH = 25


def process_actor_defeat_tick(
    world_state: "DeterministicWorldStateManager",
    *,
    step_index: int,
) -> "ActionProcessingResult | None":
    """Detect actors newly reduced to 0 HP and mark them as defeated.

    For every player/agent entity with ``health <= 0`` that is not already
    marked ``actor_defeated.{actor_id}=True``:

    1. Sets ``actor_defeated.{actor_id}=True`` in scenario_vars.
    2. Schedules respawn at ``actor_respawn_at.{actor_id}=step_index + delay``.
    3. Emits ``actor_defeated`` event.

    Respawn delay is read from ``actor_respawn_delay_ticks`` scenario_var
    (default: ``_DEFAULT_ACTOR_RESPAWN_DELAY``).

    Returns ``None`` when no actors need defeat processing.
    """
    from .world_state import DeterministicWorldStateManager as _DSM

    if not isinstance(world_state, _DSM):
        return None

    snapshot = world_state.get_snapshot()
    entities: Mapping[str, Any] = snapshot.get("entities", {})
    scenario_vars: Mapping[str, Any] = snapshot.get("scenario_vars", {})

    delay_raw = scenario_vars.get(_ACTOR_RESPAWN_DELAY_KEY)
    delay = delay_raw if isinstance(delay_raw, int) and delay_raw > 0 else _DEFAULT_ACTOR_RESPAWN_DELAY

    svars_delta: dict[str, Any] = {}
    entities_delta: dict[str, Any] = {}
    rooms_delta: dict[str, Any] = {}
    events: list[EventRecord] = []

    for entity_id in sorted(entities):
        entity = entities.get(entity_id)
        if not isinstance(entity, Mapping):
            continue
        if entity.get("entity_type") not in _ACTOR_ENTITY_TYPES:
            continue
        health = entity.get("health")
        # Treat None health as default — actor not yet damaged.
        effective_health = health if isinstance(health, int) else _DEFAULT_ACTOR_HEALTH
        if effective_health > 0:
            continue
        if scenario_vars.get(f"actor_defeated.{entity_id}"):
            continue  # already processed

        svars_delta[f"actor_defeated.{entity_id}"] = True
        svars_delta[f"actor_respawn_at.{entity_id}"] = step_index + delay
        events.append(
            EventRecord(
                step_index=step_index,
                event_type="actor_defeated",
                actor_id=entity_id,
                payload=normalize_payload(
                    {
                        "actor_id": entity_id,
                        "room_id": entity.get("location"),
                        "respawn_at_tick": step_index + delay,
                    }
                ),
            )
        )

        # Loot-on-defeat: drop all held items into the current room.
        actor_room = entity.get("location")
        inventory: list[str] = list(entity.get("inventory") or [])
        if inventory and isinstance(actor_room, str) and actor_room:
            # Clear actor inventory.
            actor_delta = dict(entity)
            actor_delta["entity_id"] = entity_id
            actor_delta["inventory"] = []
            entities_delta[entity_id] = actor_delta

            dropped_item_ids: list[str] = []
            for item_id in sorted(inventory):  # sorted for determinism
                item_entity = entities.get(item_id)
                if not isinstance(item_entity, Mapping):
                    continue
                # Update item location to the actor's room.
                item_delta = dict(item_entity)
                item_delta["entity_id"] = item_id
                item_delta["location"] = actor_room
                entities_delta[item_id] = item_delta
                dropped_item_ids.append(item_id)

                # Add item to room's entities_present (accumulate if room updated twice).
                room_payload = rooms_delta.get(actor_room) or dict(
                    entities.get(actor_room) or {}
                ) or dict(snapshot.get("rooms", {}).get(actor_room) or {})
                updated_room = dict(room_payload)
                present = list(updated_room.get("entities_present") or [])
                if item_id not in present:
                    present.append(item_id)
                updated_room["entities_present"] = sorted(present)
                rooms_delta[actor_room] = updated_room

            if dropped_item_ids:
                events.append(
                    EventRecord(
                        step_index=step_index,
                        event_type="actor_loot_dropped",
                        actor_id=entity_id,
                        payload=normalize_payload(
                            {
                                "actor_id": entity_id,
                                "room_id": actor_room,
                                "item_ids": dropped_item_ids,
                            }
                        ),
                    )
                )

    if not events:
        return None

    _update_world_event_log(scenario_vars, svars_delta, events)
    world_delta: dict[str, Any] = {"scenario_vars": svars_delta}
    if entities_delta:
        world_delta["entities"] = entities_delta
    if rooms_delta:
        world_delta["rooms"] = rooms_delta
    return ActionProcessingResult(
        accepted=True,
        events=tuple(events),
        world_delta=_world_delta_to_items(world_delta),
    )


def process_actor_respawn_tick(
    world_state: "DeterministicWorldStateManager",
    *,
    step_index: int,
) -> "ActionProcessingResult | None":
    """Restore actors whose respawn timer has elapsed.

    For each ``actor_respawn_at.{actor_id}`` in scenario_vars where
    ``step_index >= actor_respawn_at.{actor_id}``:

    1. Restores actor health from ``actor_respawn_health`` scenario_var
       (default: ``_DEFAULT_ACTOR_RESPAWN_HEALTH``).
    2. Moves actor to ``actor_respawn_room_id`` scenario_var (default: current location).
    3. Clears ``actor_defeated.{actor_id}`` and ``actor_respawn_at.{actor_id}``.
    4. Updates room ``entities_present`` if the room changed.
    5. Emits ``actor_respawned`` event.

    Returns ``None`` when no actor respawn is due this tick.
    """
    from .world_state import DeterministicWorldStateManager as _DSM

    if not isinstance(world_state, _DSM):
        return None

    snapshot = world_state.get_snapshot()
    entities: Mapping[str, Any] = snapshot.get("entities", {})
    rooms: Mapping[str, Any] = snapshot.get("rooms", {})
    scenario_vars: Mapping[str, Any] = snapshot.get("scenario_vars", {})

    respawn_health_raw = scenario_vars.get(_ACTOR_RESPAWN_HEALTH_KEY)
    respawn_health = (
        respawn_health_raw
        if isinstance(respawn_health_raw, int) and respawn_health_raw > 0
        else _DEFAULT_ACTOR_RESPAWN_HEALTH
    )
    respawn_room_raw = scenario_vars.get(_ACTOR_RESPAWN_ROOM_KEY)
    default_respawn_room: str | None = (
        str(respawn_room_raw) if isinstance(respawn_room_raw, str) and respawn_room_raw else None
    )

    # Collect actors due for respawn this tick.
    due: list[str] = []
    for key, value in scenario_vars.items():
        if not isinstance(key, str) or not key.startswith("actor_respawn_at."):
            continue
        actor_id = key[len("actor_respawn_at."):]
        if isinstance(value, int) and step_index >= value:
            due.append(actor_id)

    if not due:
        return None

    entity_delta: dict[str, Any] = {}
    rooms_delta: dict[str, Any] = {}
    svars_delta: dict[str, Any] = {}
    events: list[EventRecord] = []

    for actor_id in sorted(due):
        actor = entities.get(actor_id)
        if not isinstance(actor, Mapping):
            continue

        old_room = str(actor.get("location", ""))
        target_room = default_respawn_room if default_respawn_room else old_room
        if not target_room:
            continue

        # Restore actor entity.
        restored: dict[str, Any] = dict(actor)
        restored["entity_id"] = actor_id
        restored["health"] = respawn_health
        restored["location"] = target_room
        restored["tags"] = [t for t in restored.get("tags", []) if t]
        entity_delta[actor_id] = restored

        # Update room membership when room changed.
        if target_room != old_room:
            # Remove from old room.
            if old_room:
                old_room_payload = rooms_delta.get(old_room) or rooms.get(old_room)
                if isinstance(old_room_payload, Mapping):
                    updated_old = dict(old_room_payload)
                    updated_old["entities_present"] = sorted(
                        e for e in old_room_payload.get("entities_present", []) if e != actor_id
                    )
                    rooms_delta[old_room] = updated_old
            # Add to new room.
            new_room_payload = rooms_delta.get(target_room) or rooms.get(target_room)
            if isinstance(new_room_payload, Mapping):
                updated_new = dict(new_room_payload)
                existing = list(updated_new.get("entities_present", []))
                if actor_id not in existing:
                    existing.append(actor_id)
                    existing.sort()
                updated_new["entities_present"] = existing
                rooms_delta[target_room] = updated_new

        svars_delta[f"actor_defeated.{actor_id}"] = None  # None → key deleted on apply
        svars_delta[f"actor_respawn_at.{actor_id}"] = None

        events.append(
            EventRecord(
                step_index=step_index,
                event_type="actor_respawned",
                actor_id=actor_id,
                payload=normalize_payload(
                    {
                        "actor_id": actor_id,
                        "room_id": target_room,
                        "health": respawn_health,
                    }
                ),
            )
        )

    if not events:
        return None

    _update_world_event_log(scenario_vars, svars_delta, events)
    world_delta: dict[str, Any] = {"entities": entity_delta, "scenario_vars": svars_delta}
    if rooms_delta:
        world_delta["rooms"] = rooms_delta
    return ActionProcessingResult(
        accepted=True,
        events=tuple(events),
        world_delta=_world_delta_to_items(world_delta),
    )


def _world_delta_to_items(delta: Mapping[str, Any] | None) -> tuple[tuple[str, Any], ...]:
    if delta is None:
        return ()
    return tuple(sorted(delta.items(), key=lambda item: item[0]))


def _rejected_action_result(
    *,
    step_index: int,
    actor_id: str,
    action_type: str,
    reason: str,
) -> ActionProcessingResult:
    normalized_actor_id = actor_id if isinstance(actor_id, str) and actor_id else None
    return ActionProcessingResult(
        accepted=False,
        events=(
            EventRecord(
                step_index=step_index,
                event_type="action_rejected",
                actor_id=normalized_actor_id,
                payload=normalize_payload({"action_type": action_type, "reason": reason}),
            ),
        ),
    )
