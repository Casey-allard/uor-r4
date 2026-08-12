"""Deterministic room state types for MUDBench world topology."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _coerce_string_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    if value is None:
        return ()
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")

    entries: list[str] = []
    for entry in value:
        if not isinstance(entry, str):
            raise ValueError(f"{field_name} must contain only strings")
        entries.append(entry)
    return tuple(entries)


@dataclass(frozen=True, slots=True)
class RoomDefinition:
    """Serializable deterministic room definition."""

    room_id: str
    title: str
    description: str
    exits: tuple[tuple[str, str], ...] = ()
    entities: tuple[str, ...] = ()

    @classmethod
    def from_dict(
        cls,
        payload: Mapping[str, Any],
        *,
        room_id: str | None = None,
    ) -> "RoomDefinition":
        resolved_room_id = room_id if room_id is not None else payload.get("room_id")
        resolved_title = payload.get("title", "")
        resolved_description = payload.get("description", "")
        raw_exits = payload.get("exits", {})
        raw_entities = payload.get("entities", ())

        if not isinstance(resolved_title, str):
            raise ValueError("title must be a string")
        if not isinstance(resolved_description, str):
            raise ValueError("description must be a string")
        if not isinstance(raw_exits, Mapping):
            raise ValueError("exits must be a mapping")

        exits: list[tuple[str, str]] = []
        for direction, destination in sorted(raw_exits.items(), key=lambda item: str(item[0])):
            exits.append(
                (
                    _require_non_empty_string(direction, field_name="exits.direction"),
                    _require_non_empty_string(destination, field_name="exits.destination"),
                )
            )

        return cls(
            room_id=_require_non_empty_string(resolved_room_id, field_name="room_id"),
            title=resolved_title,
            description=resolved_description,
            exits=tuple(exits),
            entities=_coerce_string_tuple(raw_entities, field_name="entities"),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "room_id": self.room_id,
            "title": self.title,
            "description": self.description,
            "exits": {direction: destination for direction, destination in self.exits},
            "entities": list(self.entities),
        }

