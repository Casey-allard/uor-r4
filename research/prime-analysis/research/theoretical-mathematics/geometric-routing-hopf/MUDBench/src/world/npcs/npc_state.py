"""Deterministic NPC state helpers."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _coerce_optional_location(value: Any, *, field_name: str) -> str | None:
    if value is None:
        return None
    return _require_non_empty_string(value, field_name=field_name)


def _coerce_string_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    if value is None:
        return ()
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")

    values: set[str] = set()
    for entry in value:
        if not isinstance(entry, str):
            raise ValueError(f"{field_name} must contain only strings")
        values.add(entry)
    return tuple(sorted(values))


@dataclass(frozen=True, slots=True)
class NpcStateView:
    """Normalized read-model for deterministic NPC ticking."""

    npc_id: str
    location: str | None = None
    inventory: tuple[str, ...] = ()
    tags: tuple[str, ...] = ()

    @classmethod
    def from_entity_payload(
        cls,
        payload: Mapping[str, Any],
        *,
        npc_id: str,
    ) -> "NpcStateView":
        if payload.get("entity_type") != "npc":
            raise ValueError("entity_type must be 'npc'")
        return cls(
            npc_id=_require_non_empty_string(npc_id, field_name="npc_id"),
            location=_coerce_optional_location(payload.get("location"), field_name="location"),
            inventory=_coerce_string_tuple(payload.get("inventory", ()), field_name="inventory"),
            tags=_coerce_string_tuple(payload.get("tags", ()), field_name="tags"),
        )
