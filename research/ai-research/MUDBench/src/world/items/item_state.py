"""Deterministic item and inventory state models."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping, Sequence

JsonScalar = str | int | float | bool | None


def _require_non_empty_string(value: Any, *, field_name: str) -> str:
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _coerce_optional_string(value: Any, *, field_name: str) -> str | None:
    if value is None:
        return None
    return _require_non_empty_string(value, field_name=field_name)


def _coerce_sorted_unique_string_tuple(value: Any, *, field_name: str) -> tuple[str, ...]:
    if value is None:
        return ()
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")

    entries: set[str] = set()
    for entry in value:
        if not isinstance(entry, str):
            raise ValueError(f"{field_name} must contain only strings")
        entries.add(entry)
    return tuple(sorted(entries))


def _coerce_json_scalar(value: Any, *, field_name: str) -> JsonScalar:
    if value is None or isinstance(value, (str, int, float, bool)):
        return value
    raise ValueError(f"{field_name} values must be JSON scalar values")


def _coerce_optional_non_negative_int(value: Any, *, field_name: str) -> int | None:
    if value is None:
        return None
    if not isinstance(value, int):
        raise ValueError(f"{field_name} must be an integer")
    if value < 0:
        raise ValueError(f"{field_name} must be greater than or equal to zero")
    return value


@dataclass(frozen=True, slots=True)
class ItemState:
    """Serializable item state for deterministic snapshots."""

    item_id: str
    item_type: str
    name: str = ""
    description: str = ""
    location: str | None = None
    owner_entity_id: str | None = None
    tags: tuple[str, ...] = ()
    attributes: tuple[tuple[str, JsonScalar], ...] = ()

    def to_dict(self) -> dict[str, Any]:
        return {
            "item_id": self.item_id,
            "item_type": self.item_type,
            "name": self.name,
            "description": self.description,
            "location": self.location,
            "owner_entity_id": self.owner_entity_id,
            "tags": list(self.tags),
            "attributes": {name: value for name, value in self.attributes},
        }

    @classmethod
    def from_dict(
        cls,
        payload: Mapping[str, Any],
        *,
        item_id: str | None = None,
    ) -> "ItemState":
        resolved_item_id = item_id if item_id is not None else payload.get("item_id")
        resolved_item_type = payload.get("item_type")
        resolved_name = payload.get("name", "")
        resolved_description = payload.get("description", "")
        raw_location = payload.get("location")
        raw_owner_entity_id = payload.get("owner_entity_id")
        raw_tags = payload.get("tags", ())
        raw_attributes = payload.get("attributes", {})

        if not isinstance(resolved_name, str):
            raise ValueError("name must be a string")
        if not isinstance(resolved_description, str):
            raise ValueError("description must be a string")
        if not isinstance(raw_attributes, Mapping):
            raise ValueError("attributes must be a mapping")

        attributes = tuple(
            (
                _require_non_empty_string(attribute_name, field_name="attributes.key"),
                _coerce_json_scalar(attribute_value, field_name=f"attributes[{attribute_name}]"),
            )
            for attribute_name, attribute_value in sorted(
                raw_attributes.items(), key=lambda item: str(item[0])
            )
        )

        return cls(
            item_id=_require_non_empty_string(resolved_item_id, field_name="item_id"),
            item_type=_require_non_empty_string(resolved_item_type, field_name="item_type"),
            name=resolved_name,
            description=resolved_description,
            location=_coerce_optional_string(raw_location, field_name="location"),
            owner_entity_id=_coerce_optional_string(
                raw_owner_entity_id, field_name="owner_entity_id"
            ),
            tags=_coerce_sorted_unique_string_tuple(raw_tags, field_name="tags"),
            attributes=attributes,
        )


@dataclass(frozen=True, slots=True)
class InventoryState:
    """Serializable inventory state with deterministic item ordering."""

    owner_entity_id: str
    item_ids: tuple[str, ...] = ()
    capacity: int | None = None
    weight_limit: int | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "owner_entity_id": self.owner_entity_id,
            "item_ids": list(self.item_ids),
            "capacity": self.capacity,
            "weight_limit": self.weight_limit,
        }

    @classmethod
    def from_dict(
        cls,
        payload: Mapping[str, Any],
        *,
        owner_entity_id: str | None = None,
    ) -> "InventoryState":
        resolved_owner_entity_id = (
            owner_entity_id if owner_entity_id is not None else payload.get("owner_entity_id")
        )
        raw_item_ids = payload.get("item_ids", ())
        raw_capacity = payload.get("capacity")
        raw_weight_limit = payload.get("weight_limit")

        return cls(
            owner_entity_id=_require_non_empty_string(
                resolved_owner_entity_id, field_name="owner_entity_id"
            ),
            item_ids=_coerce_sorted_unique_string_tuple(raw_item_ids, field_name="item_ids"),
            capacity=_coerce_optional_non_negative_int(raw_capacity, field_name="capacity"),
            weight_limit=_coerce_optional_non_negative_int(
                raw_weight_limit, field_name="weight_limit"
            ),
        )
