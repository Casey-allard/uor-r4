"""Deterministic movement rule evaluation over room exits."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from world.state.world_state import DeterministicWorldStateManager

from .room_graph import DeterministicRoomGraph


@dataclass(frozen=True, slots=True)
class MovementResolution:
    """Result of evaluating a movement action."""

    actor_id: str
    direction: str
    success: bool
    reason: str | None = None
    source_room_id: str | None = None
    destination_room_id: str | None = None
    world_delta: dict[str, Any] | None = None


class DeterministicMovementRules:
    """Apply deterministic movement updates against room exits and world state."""

    def apply_move(
        self,
        world_state: DeterministicWorldStateManager,
        room_graph: DeterministicRoomGraph,
        *,
        actor_id: str,
        direction: str,
    ) -> MovementResolution:
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")
        if not isinstance(room_graph, DeterministicRoomGraph):
            raise ValueError("room_graph must be a DeterministicRoomGraph")
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError("actor_id must be a non-empty string")
        if not isinstance(direction, str) or not direction:
            raise ValueError("direction must be a non-empty string")

        snapshot = world_state.get_snapshot()
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")

        actor_payload = entities.get(actor_id)
        if not isinstance(actor_payload, Mapping):
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="actor_not_found",
            )

        source_room_id = actor_payload.get("location")
        if not isinstance(source_room_id, str) or not source_room_id:
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="actor_has_no_location",
            )
        if source_room_id not in rooms:
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="source_room_missing",
                source_room_id=source_room_id,
            )

        try:
            destination_room_id = room_graph.resolve_exit(source_room_id, direction)
        except KeyError:
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="source_room_not_in_graph",
                source_room_id=source_room_id,
            )

        if destination_room_id is None:
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="exit_not_found",
                source_room_id=source_room_id,
            )
        if destination_room_id not in rooms:
            return MovementResolution(
                actor_id=actor_id,
                direction=direction,
                success=False,
                reason="destination_room_missing",
                source_room_id=source_room_id,
                destination_room_id=destination_room_id,
            )

        updated_entity_payload = dict(actor_payload)
        updated_entity_payload["entity_id"] = actor_id
        updated_entity_payload["location"] = destination_room_id

        rooms_delta: dict[str, dict[str, Any]] = {
            source_room_id: _room_payload_without_actor(rooms[source_room_id], actor_id=actor_id),
            destination_room_id: _room_payload_with_actor(rooms[destination_room_id], actor_id=actor_id),
        }
        world_delta = {"entities": {actor_id: updated_entity_payload}, "rooms": rooms_delta}

        return MovementResolution(
            actor_id=actor_id,
            direction=direction,
            success=True,
            source_room_id=source_room_id,
            destination_room_id=destination_room_id,
            world_delta=world_delta,
        )


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _room_payload_with_actor(room_payload: Any, *, actor_id: str) -> dict[str, Any]:
    normalized = _room_payload_with_normalized_entities(room_payload)
    entities = set(normalized["entities_present"])
    entities.add(actor_id)
    normalized["entities_present"] = sorted(entities)
    return normalized


def _room_payload_without_actor(room_payload: Any, *, actor_id: str) -> dict[str, Any]:
    normalized = _room_payload_with_normalized_entities(room_payload)
    entities = set(normalized["entities_present"])
    entities.discard(actor_id)
    normalized["entities_present"] = sorted(entities)
    return normalized


def _room_payload_with_normalized_entities(room_payload: Any) -> dict[str, Any]:
    if not isinstance(room_payload, Mapping):
        raise ValueError("room payload must be a mapping")

    normalized = dict(room_payload)
    raw_entities = normalized.get("entities_present", [])
    if isinstance(raw_entities, (str, bytes)) or not isinstance(raw_entities, Sequence):
        raise ValueError("room entities_present must be a sequence of strings")

    entities: set[str] = set()
    for current_actor_id in raw_entities:
        if not isinstance(current_actor_id, str):
            raise ValueError("room entities_present must contain only strings")
        entities.add(current_actor_id)

    normalized["entities_present"] = sorted(entities)
    return normalized
