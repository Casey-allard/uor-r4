from __future__ import annotations

import pytest

from world.items.item_state import InventoryState, ItemState
from world.state.state_types import WorldEntityState


def test_item_state_normalizes_and_serializes_deterministically() -> None:
    item = ItemState.from_dict(
        {
            "item_id": "item-1",
            "item_type": "quest",
            "name": "Ancient Key",
            "description": "A rusted key.",
            "location": "room-a",
            "owner_entity_id": None,
            "tags": ["rare", "quest", "rare"],
            "attributes": {"weight": 1, "durability": 100},
        }
    )

    serialized = item.to_dict()

    assert item.tags == ("quest", "rare")
    assert item.attributes == (("durability", 100), ("weight", 1))
    assert serialized["tags"] == ["quest", "rare"]
    assert serialized["attributes"] == {"durability": 100, "weight": 1}


def test_item_state_roundtrip_is_stable() -> None:
    original = ItemState.from_dict(
        {
            "item_id": "item-2",
            "item_type": "utility",
            "name": "Torch",
            "description": "A basic torch.",
            "location": None,
            "owner_entity_id": "agent-1",
            "tags": ["tool"],
            "attributes": {"charges": 3, "lit": False},
        }
    )

    restored = ItemState.from_dict(original.to_dict())

    assert restored == original
    assert restored.to_dict() == original.to_dict()


def test_item_state_rejects_non_scalar_attribute_values() -> None:
    with pytest.raises(ValueError, match="JSON scalar"):
        ItemState.from_dict(
            {
                "item_id": "item-3",
                "item_type": "quest",
                "attributes": {"metadata": {"nested": "invalid"}},
            }
        )


def test_inventory_state_normalizes_item_ids() -> None:
    inventory = InventoryState.from_dict(
        {
            "owner_entity_id": "agent-1",
            "item_ids": ["item-3", "item-1", "item-3", "item-2"],
            "capacity": 10,
            "weight_limit": 25,
        }
    )

    serialized = inventory.to_dict()

    assert inventory.item_ids == ("item-1", "item-2", "item-3")
    assert serialized["item_ids"] == ["item-1", "item-2", "item-3"]
    assert serialized["capacity"] == 10
    assert serialized["weight_limit"] == 25


def test_inventory_state_rejects_negative_limits() -> None:
    with pytest.raises(ValueError, match="greater than or equal to zero"):
        InventoryState.from_dict(
            {
                "owner_entity_id": "agent-1",
                "item_ids": ["item-1"],
                "capacity": -1,
            }
        )


def test_world_entity_inventory_is_deterministically_normalized() -> None:
    entity = WorldEntityState.from_dict(
        {
            "entity_id": "agent-1",
            "entity_type": "agent",
            "inventory": ["item-b", "item-a", "item-b"],
        }
    )

    assert entity.inventory == ("item-a", "item-b")
    assert entity.to_dict()["inventory"] == ["item-a", "item-b"]
