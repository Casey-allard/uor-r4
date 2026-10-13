"""Deterministic world state datatypes for MUDBench Phase 1."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

JsonScalar = str | int | float | bool | None


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _require_int(value: Any, *, field_name: str) -> int:
    if not isinstance(value, int):
        raise ValueError(f"{field_name} must be an integer")
    return value


def _coerce_string_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    if value is None:
        return ()
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")

    values: list[str] = []
    for item in value:
        if not isinstance(item, str):
            raise ValueError(f"{field_name} must contain only strings")
        values.append(item)
    return tuple(values)


def _coerce_inventory_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    entries = _coerce_string_tuple(value, field_name=field_name)
    return tuple(sorted(set(entries)))


def _coerce_json_scalar(value: Any, *, field_name: str) -> JsonScalar:
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    raise ValueError(f"{field_name} values must be JSON scalar values")


@dataclass(frozen=True, slots=True)
class WorldEntityState:
    """Serializable entity state used in deterministic world snapshots."""

    entity_id: str
    entity_type: str
    location: str | None = None
    health: int | None = None
    inventory: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return {
            "entity_id": self.entity_id,
            "entity_type": self.entity_type,
            "location": self.location,
            "health": self.health,
            "inventory": list(self.inventory),
            "tags": list(self.tags),
        }

    @classmethod
    def from_dict(
        cls,
        payload: Mapping[str, Any],
        *,
        entity_id: str | None = None,
    ) -> "WorldEntityState":
        resolved_entity_id = entity_id if entity_id is not None else payload.get("entity_id")
        resolved_entity_type = payload.get("entity_type")
        raw_location = payload.get("location")
        raw_health = payload.get("health")
        raw_inventory = payload.get("inventory", ())
        raw_tags = payload.get("tags", ())

        entity_location: str | None
        if raw_location is None:
            entity_location = None
        else:
            entity_location = _require_non_empty_string(raw_location, field_name="location")

        entity_health: int | None
        if raw_health is None:
            entity_health = None
        else:
            entity_health = _require_int(raw_health, field_name="health")

        return cls(
            entity_id=_require_non_empty_string(resolved_entity_id, field_name="entity_id"),
            entity_type=_require_non_empty_string(resolved_entity_type, field_name="entity_type"),
            location=entity_location,
            health=entity_health,
            inventory=_coerce_inventory_tuple(raw_inventory, field_name="inventory"),
            tags=_coerce_string_tuple(raw_tags, field_name="tags"),
        )


@dataclass(frozen=True, slots=True)
class WorldRoomState:
    """Serializable room state for deterministic world snapshots."""

    room_id: str
    description: str = ""
    exits: tuple[tuple[str, str], ...] = ()
    entities_present: tuple[str, ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return {
            "room_id": self.room_id,
            "description": self.description,
            "exits": {direction: destination for direction, destination in self.exits},
            "entities_present": list(self.entities_present),
        }

    @classmethod
    def from_dict(
        cls,
        payload: Mapping[str, Any],
        *,
        room_id: str | None = None,
    ) -> "WorldRoomState":
        resolved_room_id = room_id if room_id is not None else payload.get("room_id")
        resolved_description = payload.get("description", "")
        raw_exits = payload.get("exits", {})
        raw_entities = payload.get("entities_present", ())

        if not isinstance(resolved_description, str):
            raise ValueError("description must be a string")
        if isinstance(raw_exits, Mapping):
            exits_pairs: list[tuple[str, str]] = []
            for direction, destination in raw_exits.items():
                exits_pairs.append(
                    (
                        _require_non_empty_string(direction, field_name="exits.direction"),
                        _require_non_empty_string(destination, field_name="exits.destination"),
                    )
                )
            exits = tuple(sorted(exits_pairs, key=lambda item: item[0]))
        else:
            raise ValueError("exits must be a mapping of direction to destination room_id")

        return cls(
            room_id=_require_non_empty_string(resolved_room_id, field_name="room_id"),
            description=resolved_description,
            exits=exits,
            entities_present=_coerce_string_tuple(raw_entities, field_name="entities_present"),
        )


@dataclass(frozen=True, slots=True)
class WorldStateSnapshot:
    """Immutable world snapshot with deterministic ordering."""

    tick: int
    entities: tuple[WorldEntityState, ...] = ()
    rooms: tuple[WorldRoomState, ...] = ()
    scenario_vars: tuple[tuple[str, JsonScalar], ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return {
            "tick": self.tick,
            "entities": {
                entity.entity_id: entity.to_dict()
                for entity in sorted(self.entities, key=lambda item: item.entity_id)
            },
            "rooms": {
                room.room_id: room.to_dict()
                for room in sorted(self.rooms, key=lambda item: item.room_id)
            },
            "scenario_vars": {key: value for key, value in self.scenario_vars},
        }

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "WorldStateSnapshot":
        raw_tick = payload.get("tick", 0)
        raw_entities = payload.get("entities", {})
        raw_rooms = payload.get("rooms", {})
        raw_scenario_vars = payload.get("scenario_vars", {})

        if not isinstance(raw_entities, Mapping):
            raise ValueError("entities must be a mapping")
        if not isinstance(raw_rooms, Mapping):
            raise ValueError("rooms must be a mapping")
        if not isinstance(raw_scenario_vars, Mapping):
            raise ValueError("scenario_vars must be a mapping")

        entities = tuple(
            WorldEntityState.from_dict(entity_payload, entity_id=str(entity_id))
            for entity_id, entity_payload in sorted(
                raw_entities.items(), key=lambda item: str(item[0])
            )
        )
        rooms = tuple(
            WorldRoomState.from_dict(room_payload, room_id=str(room_id))
            for room_id, room_payload in sorted(raw_rooms.items(), key=lambda item: str(item[0]))
        )
        scenario_vars = tuple(
            (
                _require_non_empty_string(name, field_name="scenario_vars.key"),
                _coerce_json_scalar(value, field_name=f"scenario_vars[{name}]"),
            )
            for name, value in sorted(raw_scenario_vars.items(), key=lambda item: str(item[0]))
        )
        return cls(
            tick=_require_int(raw_tick, field_name="tick"),
            entities=entities,
            rooms=rooms,
            scenario_vars=scenario_vars,
        )
