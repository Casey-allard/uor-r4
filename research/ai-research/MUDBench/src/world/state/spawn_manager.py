"""Deterministic spawn placement for agents and NPCs."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from .world_state import DeterministicWorldStateManager


@dataclass(frozen=True, slots=True)
class SpawnRequest:
    """Requested actor spawn details."""

    actor_id: str
    actor_type: str
    preferred_room_id: str | None = None
    health: int | None = None
    inventory: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class SpawnPlacement:
    """Resolved spawn mapping for an actor."""

    actor_id: str
    room_id: str


class DeterministicSpawnManager:
    """Places actors into world rooms using deterministic ordering and tie-breaking."""

    def __init__(self, *, seed: int = 0) -> None:
        if not isinstance(seed, int):
            raise ValueError("seed must be an integer")
        self._seed = seed

    def place_actors(
        self,
        world_state: DeterministicWorldStateManager,
        requests: Sequence[SpawnRequest],
    ) -> tuple[SpawnPlacement, ...]:
        """Apply deterministic spawn placement and return resolved mapping."""
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")

        snapshot = world_state.get_snapshot()
        rooms = snapshot.get("rooms")
        entities = snapshot.get("entities")
        if not isinstance(rooms, Mapping) or not rooms:
            raise ValueError("world state must contain at least one room before spawning")
        if not isinstance(entities, Mapping):
            raise ValueError("world state entities must be a mapping")

        room_ids = tuple(sorted(str(room_id) for room_id in rooms.keys()))
        entities_delta: dict[str, dict[str, Any]] = {}
        rooms_delta: dict[str, dict[str, Any]] = {}
        placements: list[SpawnPlacement] = []

        ordered_requests = tuple(
            sorted(
                requests,
                key=lambda req: (req.actor_id, req.actor_type, req.preferred_room_id or ""),
            )
        )

        next_auto_index = 0
        for request in ordered_requests:
            if request.actor_id in entities or request.actor_id in entities_delta:
                raise ValueError(f"actor_id already exists in world state: {request.actor_id}")

            if request.preferred_room_id is not None:
                if request.preferred_room_id not in rooms:
                    raise ValueError(
                        f"preferred_room_id '{request.preferred_room_id}' does not exist in world state"
                    )
                room_id = request.preferred_room_id
            else:
                room_id = room_ids[(self._seed + next_auto_index) % len(room_ids)]
                next_auto_index += 1

            placements.append(SpawnPlacement(actor_id=request.actor_id, room_id=room_id))
            entities_delta[request.actor_id] = _entity_payload(request=request, room_id=room_id)
            room_base_payload = rooms_delta.get(room_id, rooms.get(room_id))
            rooms_delta[room_id] = _room_payload_with_spawn(
                room_base_payload, new_actor_id=request.actor_id
            )

        world_state.apply_delta({"entities": entities_delta, "rooms": rooms_delta})

        return tuple(placements)


def _entity_payload(request: SpawnRequest, *, room_id: str) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "entity_id": request.actor_id,
        "entity_type": request.actor_type,
        "location": room_id,
    }
    if request.health is not None:
        payload["health"] = request.health
    if request.inventory:
        payload["inventory"] = list(request.inventory)
    if request.tags:
        payload["tags"] = list(request.tags)
    return payload


def _room_payload_with_spawn(room_payload: Any, *, new_actor_id: str) -> dict[str, Any]:
    if not isinstance(room_payload, Mapping):
        raise ValueError("room payload must be a mapping")

    normalized = dict(room_payload)
    raw_entities = normalized.get("entities_present", [])
    if isinstance(raw_entities, (str, bytes)) or not isinstance(raw_entities, Sequence):
        raise ValueError("room entities_present must be a sequence of strings")

    combined: set[str] = set()
    for actor_id in raw_entities:
        if not isinstance(actor_id, str):
            raise ValueError("room entities_present must contain only strings")
        combined.add(actor_id)
    combined.add(new_actor_id)

    normalized["entities_present"] = sorted(combined)
    return normalized
