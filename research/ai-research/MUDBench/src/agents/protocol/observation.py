"""Structured observation model for MUDBench agents."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence


REQUIRED_OBSERVATION_FIELDS = (
    "run_id",
    "step",
    "location",
    "description",
    "exits",
    "entities",
    "inventory",
    "health",
    "messages",
    "action_space",
    "remaining_steps",
)
SUPPORTED_PROTOCOL_VERSIONS = ("1.0",)
DEFAULT_PROTOCOL_VERSION = SUPPORTED_PROTOCOL_VERSIONS[0]


@dataclass(frozen=True, slots=True)
class ObservedEntity:
    """Entity listed in the observation payload."""

    type: str
    name: str

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "ObservedEntity":
        entity_type = payload.get("type")
        entity_name = payload.get("name")
        if not isinstance(entity_type, str) or not entity_type:
            raise ValueError("Observation entity requires non-empty 'type'")
        if not isinstance(entity_name, str) or not entity_name:
            raise ValueError("Observation entity requires non-empty 'name'")
        return cls(type=entity_type, name=entity_name)

    def to_dict(self) -> dict[str, str]:
        return {"type": self.type, "name": self.name}


@dataclass(frozen=True, slots=True)
class Observation:
    """Canonical observation object delivered to an agent each step."""

    run_id: str
    step: int
    location: str
    description: str
    exits: tuple[str, ...]
    entities: tuple[ObservedEntity, ...]
    inventory: tuple[str, ...]
    health: int
    messages: tuple[str, ...]
    action_space: tuple[str, ...]
    remaining_steps: int
    protocol_version: str = DEFAULT_PROTOCOL_VERSION

    def __post_init__(self) -> None:
        normalized = require_supported_protocol_version(self.protocol_version)
        object.__setattr__(self, "protocol_version", normalized)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "Observation":
        missing = [field for field in REQUIRED_OBSERVATION_FIELDS if field not in payload]
        if missing:
            raise ValueError(f"Observation missing required fields: {', '.join(missing)}")

        return cls(
            run_id=_require_non_empty_string(payload, "run_id"),
            step=_require_int(payload, "step"),
            location=_require_non_empty_string(payload, "location"),
            description=_require_non_empty_string(payload, "description"),
            exits=_require_string_sequence(payload, "exits"),
            entities=_require_entities(payload, "entities"),
            inventory=_require_string_sequence(payload, "inventory"),
            health=_require_int(payload, "health"),
            messages=_require_string_sequence(payload, "messages"),
            action_space=_require_string_sequence(payload, "action_space"),
            remaining_steps=_require_int(payload, "remaining_steps"),
            protocol_version=_require_protocol_version(payload),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "step": self.step,
            "location": self.location,
            "description": self.description,
            "exits": list(self.exits),
            "entities": [entity.to_dict() for entity in self.entities],
            "inventory": list(self.inventory),
            "health": self.health,
            "messages": list(self.messages),
            "action_space": list(self.action_space),
            "remaining_steps": self.remaining_steps,
            "protocol_version": self.protocol_version,
        }


def _require_non_empty_string(payload: Mapping[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"Observation field '{key}' must be a non-empty string")
    return value


def _require_int(payload: Mapping[str, Any], key: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        raise ValueError(f"Observation field '{key}' must be an integer")
    return value


def _require_string_sequence(payload: Mapping[str, Any], key: str) -> tuple[str, ...]:
    value = payload.get(key)
    if not isinstance(value, Sequence) or isinstance(value, (str, bytes)):
        raise ValueError(f"Observation field '{key}' must be a sequence of strings")

    result: list[str] = []
    for item in value:
        if not isinstance(item, str):
            raise ValueError(f"Observation field '{key}' must contain only strings")
        result.append(item)
    return tuple(result)


def _require_entities(payload: Mapping[str, Any], key: str) -> tuple[ObservedEntity, ...]:
    value = payload.get(key)
    if not isinstance(value, Sequence) or isinstance(value, (str, bytes)):
        raise ValueError(f"Observation field '{key}' must be a sequence of objects")

    entities: list[ObservedEntity] = []
    for item in value:
        if not isinstance(item, Mapping):
            raise ValueError(f"Observation field '{key}' must contain object values")
        entities.append(ObservedEntity.from_mapping(item))
    return tuple(entities)


def _require_protocol_version(payload: Mapping[str, Any]) -> str:
    return require_supported_protocol_version(payload.get("protocol_version", DEFAULT_PROTOCOL_VERSION))


def require_supported_protocol_version(value: object) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError("protocol_version_must_be_non_empty_string")
    if value not in SUPPORTED_PROTOCOL_VERSIONS:
        supported = ",".join(SUPPORTED_PROTOCOL_VERSIONS)
        raise ValueError(f"unsupported_protocol_version:{value}:supported={supported}")
    return value
