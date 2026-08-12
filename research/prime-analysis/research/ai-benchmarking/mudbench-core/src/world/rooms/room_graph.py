"""Deterministic room graph representation and validation."""

from __future__ import annotations

import json
from typing import Any, Mapping, Sequence

from .room_types import RoomDefinition


class DeterministicRoomGraph:
    """Serializable room graph with deterministic ordering and validation."""

    def __init__(self, rooms: Sequence[RoomDefinition]) -> None:
        seen_room_ids: set[str] = set()
        room_lookup: dict[str, RoomDefinition] = {}
        for room in rooms:
            if room.room_id in seen_room_ids:
                raise ValueError(f"duplicate room_id detected: {room.room_id}")
            seen_room_ids.add(room.room_id)
            room_lookup[room.room_id] = room

        for room_id, room in room_lookup.items():
            for _, destination in room.exits:
                if destination not in room_lookup:
                    raise ValueError(
                        f"room '{room_id}' has exit to unknown destination '{destination}'"
                    )

        self._rooms_by_id = room_lookup
        self._room_ids = tuple(sorted(room_lookup.keys()))

    @property
    def room_ids(self) -> tuple[str, ...]:
        """Return room identifiers in deterministic order."""
        return self._room_ids

    def get_room(self, room_id: str) -> RoomDefinition:
        """Return room definition for an ID."""
        return self._rooms_by_id[room_id]

    def resolve_exit(self, room_id: str, direction: str) -> str | None:
        """Resolve destination room for an exit direction."""
        room = self.get_room(room_id)
        for current_direction, destination in room.exits:
            if current_direction == direction:
                return destination
        return None

    def to_dict(self) -> dict[str, Any]:
        """Serialize graph with stable ordering."""
        return {
            "rooms": {room_id: self._rooms_by_id[room_id].to_dict() for room_id in self._room_ids}
        }

    def to_json(self) -> str:
        """Serialize graph to stable JSON."""
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "DeterministicRoomGraph":
        """Build graph from a serialized mapping."""
        raw_rooms = payload.get("rooms")
        if not isinstance(raw_rooms, Mapping):
            raise ValueError("rooms must be a mapping")

        rooms = tuple(
            RoomDefinition.from_dict(room_payload, room_id=str(room_id))
            for room_id, room_payload in sorted(raw_rooms.items(), key=lambda item: str(item[0]))
        )
        return cls(rooms=rooms)

    @classmethod
    def from_json(cls, payload: str) -> "DeterministicRoomGraph":
        """Build graph from JSON payload."""
        parsed = json.loads(payload)
        if not isinstance(parsed, dict):
            raise ValueError("room graph json payload must decode to an object")
        return cls.from_dict(parsed)

