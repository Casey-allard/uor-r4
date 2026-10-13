"""Deterministic per-step NPC tick updates."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from world.state.world_state import DeterministicWorldStateManager

from .npc_state import NpcStateView


@dataclass(frozen=True, slots=True)
class NpcTickOutcome:
    """Single NPC tick outcome."""

    npc_id: str
    moved: bool
    source_room_id: str | None = None
    destination_room_id: str | None = None
    reason: str | None = None


class DeterministicNpcTicker:
    """Applies deterministic NPC movement updates over room exits."""

    def tick(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        step_index: int,
    ) -> tuple[NpcTickOutcome, ...]:
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")
        if not isinstance(step_index, int) or step_index < 0:
            raise ValueError("step_index must be a non-negative integer")

        snapshot = world_state.get_snapshot()
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")
        rooms = _require_mapping(snapshot.get("rooms"), field_name="world.rooms")
        room_payloads = {
            room_id: _normalized_room_payload(room_payload)
            for room_id, room_payload in sorted(rooms.items(), key=lambda item: str(item[0]))
        }

        npc_ids = tuple(
            sorted(
                entity_id
                for entity_id, payload in entities.items()
                if isinstance(payload, Mapping) and payload.get("entity_type") == "npc"
            )
        )

        entity_updates: dict[str, dict[str, Any]] = {}
        room_updates: dict[str, dict[str, Any]] = {}
        outcomes: list[NpcTickOutcome] = []

        for npc_id in npc_ids:
            raw_payload = entities.get(npc_id)
            if not isinstance(raw_payload, Mapping):
                outcomes.append(NpcTickOutcome(npc_id=npc_id, moved=False, reason="npc_payload_invalid"))
                continue
            npc_state = NpcStateView.from_entity_payload(raw_payload, npc_id=npc_id)
            source_room_id = npc_state.location
            if source_room_id is None:
                outcomes.append(
                    NpcTickOutcome(npc_id=npc_id, moved=False, reason="npc_has_no_location")
                )
                continue
            if source_room_id not in room_payloads:
                outcomes.append(
                    NpcTickOutcome(
                        npc_id=npc_id,
                        moved=False,
                        source_room_id=source_room_id,
                        reason="source_room_missing",
                    )
                )
                continue

            source_room = room_payloads[source_room_id]
            exits = _sorted_exits(source_room.get("exits", {}))
            if not exits:
                outcomes.append(
                    NpcTickOutcome(
                        npc_id=npc_id,
                        moved=False,
                        source_room_id=source_room_id,
                        reason="no_exits",
                    )
                )
                continue

            _, destination_room_id = exits[step_index % len(exits)]
            if destination_room_id not in room_payloads:
                outcomes.append(
                    NpcTickOutcome(
                        npc_id=npc_id,
                        moved=False,
                        source_room_id=source_room_id,
                        destination_room_id=destination_room_id,
                        reason="destination_room_missing",
                    )
                )
                continue

            destination_room = room_payloads[destination_room_id]
            source_room["entities_present"] = sorted(
                entity_id
                for entity_id in source_room["entities_present"]
                if entity_id != npc_id
            )
            destination_room["entities_present"] = sorted(
                set(destination_room["entities_present"]) | {npc_id}
            )
            room_updates[source_room_id] = source_room
            room_updates[destination_room_id] = destination_room

            updated_npc_payload = dict(raw_payload)
            updated_npc_payload["entity_id"] = npc_id
            updated_npc_payload["entity_type"] = "npc"
            updated_npc_payload["location"] = destination_room_id
            entity_updates[npc_id] = updated_npc_payload

            outcomes.append(
                NpcTickOutcome(
                    npc_id=npc_id,
                    moved=True,
                    source_room_id=source_room_id,
                    destination_room_id=destination_room_id,
                )
            )

        if entity_updates or room_updates:
            world_state.apply_delta({"entities": entity_updates, "rooms": room_updates})

        return tuple(outcomes)


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value


def _normalized_room_payload(room_payload: Any) -> dict[str, Any]:
    if not isinstance(room_payload, Mapping):
        raise ValueError("room payload must be a mapping")
    normalized = dict(room_payload)
    raw_entities = normalized.get("entities_present", ())
    if isinstance(raw_entities, (str, bytes)) or not isinstance(raw_entities, Sequence):
        raise ValueError("room entities_present must be a sequence of strings")

    entities_present: set[str] = set()
    for entity_id in raw_entities:
        if not isinstance(entity_id, str):
            raise ValueError("room entities_present must contain only strings")
        entities_present.add(entity_id)
    normalized["entities_present"] = sorted(entities_present)
    return normalized


def _sorted_exits(raw_exits: Any) -> tuple[tuple[str, str], ...]:
    exits = _require_mapping(raw_exits, field_name="room.exits")
    pairs: list[tuple[str, str]] = []
    for direction, destination in sorted(exits.items(), key=lambda item: str(item[0])):
        if not isinstance(direction, str) or not direction:
            raise ValueError("room exit direction must be a non-empty string")
        if not isinstance(destination, str) or not destination:
            raise ValueError("room exit destination must be a non-empty string")
        pairs.append((direction, destination))
    return tuple(pairs)
