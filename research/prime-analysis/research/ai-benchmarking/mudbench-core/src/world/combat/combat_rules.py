"""Deterministic basic combat resolution rules."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Mapping

from world.state.world_state import DeterministicWorldStateManager


@dataclass(frozen=True, slots=True)
class AttackResolution:
    """Deterministic resolution for a single attack action."""

    attacker_id: str
    target_id: str
    success: bool
    damage: int = 0
    resulting_health: int | None = None
    room_id: str | None = None
    reason: str | None = None
    world_delta: dict[str, Any] | None = None


class DeterministicCombatRules:
    """Resolve basic deterministic attack transitions."""

    def __init__(self, *, damage: int = 10) -> None:
        if not isinstance(damage, int) or damage <= 0:
            raise ValueError("damage must be a positive integer")
        self._damage = damage

    @property
    def damage(self) -> int:
        return self._damage

    def apply_attack(
        self,
        world_state: DeterministicWorldStateManager,
        *,
        attacker_id: str,
        target_id: str,
    ) -> AttackResolution:
        if not isinstance(world_state, DeterministicWorldStateManager):
            raise ValueError("world_state must be a DeterministicWorldStateManager")
        if not isinstance(attacker_id, str) or not attacker_id:
            raise ValueError("attacker_id must be a non-empty string")
        if not isinstance(target_id, str) or not target_id:
            raise ValueError("target_id must be a non-empty string")

        snapshot = world_state.get_snapshot()
        entities = _require_mapping(snapshot.get("entities"), field_name="world.entities")

        attacker_payload = entities.get(attacker_id)
        if not isinstance(attacker_payload, Mapping):
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                reason="attacker_not_found",
            )

        target_payload = entities.get(target_id)
        if not isinstance(target_payload, Mapping):
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                reason="target_not_found",
            )

        attacker_room = attacker_payload.get("location")
        if not isinstance(attacker_room, str) or not attacker_room:
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                reason="attacker_has_no_location",
            )

        target_room = target_payload.get("location")
        if not isinstance(target_room, str) or not target_room:
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                room_id=attacker_room,
                reason="target_has_no_location",
            )

        if attacker_room != target_room:
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                room_id=attacker_room,
                reason="target_not_in_same_room",
            )

        raw_health = target_payload.get("health")
        if not isinstance(raw_health, int):
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                room_id=attacker_room,
                reason="target_health_invalid",
            )
        if raw_health <= 0:
            return AttackResolution(
                attacker_id=attacker_id,
                target_id=target_id,
                success=False,
                room_id=attacker_room,
                reason="target_already_defeated",
            )

        resulting_health = max(0, raw_health - self._damage)
        updated_target = dict(target_payload)
        updated_target["entity_id"] = target_id
        updated_target["health"] = resulting_health
        world_delta = {"entities": {target_id: updated_target}}

        return AttackResolution(
            attacker_id=attacker_id,
            target_id=target_id,
            success=True,
            damage=self._damage,
            resulting_health=resulting_health,
            room_id=attacker_room,
            world_delta=world_delta,
        )


def _require_mapping(value: Any, *, field_name: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")
    return value
