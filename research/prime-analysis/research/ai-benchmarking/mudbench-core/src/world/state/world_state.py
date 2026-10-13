"""Deterministic world state manager implementation for MUDBench."""

from __future__ import annotations

import json
from typing import Any, Mapping

from core.world_state_manager import WorldDelta, WorldSnapshot, WorldStateManager

from .state_types import JsonScalar, WorldStateSnapshot


class DeterministicWorldStateManager(WorldStateManager):
    """Minimal deterministic world state manager.

    This implementation focuses on deterministic initialization, deterministic
    delta application, and stable serialization for Phase 1.
    """

    def __init__(self, initial_state: WorldStateSnapshot | None = None) -> None:
        self._initial_state = initial_state or WorldStateSnapshot(tick=0)
        self._state = self._initial_state

    def get_snapshot(self) -> WorldSnapshot:
        """Return a read-only snapshot as a plain mapping."""
        return self._state.to_dict()

    def apply_delta(self, delta: WorldDelta) -> None:
        """Apply deterministic state updates using a fixed key order."""
        if not isinstance(delta, Mapping):
            raise ValueError("delta must be a mapping")

        current_snapshot = self._state.to_dict()
        updated_tick = current_snapshot["tick"]
        updated_entities = dict(current_snapshot["entities"])
        updated_rooms = dict(current_snapshot["rooms"])
        updated_scenario_vars = dict(current_snapshot["scenario_vars"])

        if "tick" in delta:
            raw_tick = delta["tick"]
            if not isinstance(raw_tick, int):
                raise ValueError("delta.tick must be an integer")
            updated_tick = raw_tick

        if "entities" in delta:
            self._apply_entities_delta(updated_entities, delta["entities"])

        if "rooms" in delta:
            self._apply_rooms_delta(updated_rooms, delta["rooms"])

        if "scenario_vars" in delta:
            self._apply_scenario_vars_delta(updated_scenario_vars, delta["scenario_vars"])

        self._state = WorldStateSnapshot.from_dict(
            {
                "tick": updated_tick,
                "entities": updated_entities,
                "rooms": updated_rooms,
                "scenario_vars": updated_scenario_vars,
            }
        )

    def reset(self) -> None:
        """Reset world state to deterministic initial baseline."""
        self._state = self._initial_state

    def to_dict(self) -> dict[str, Any]:
        """Serialize the current world state to a plain dictionary."""
        return self._state.to_dict()

    def to_json(self) -> str:
        """Serialize current world state to stable JSON."""
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "DeterministicWorldStateManager":
        """Create a manager from a snapshot dictionary."""
        snapshot = WorldStateSnapshot.from_dict(payload)
        return cls(initial_state=snapshot)

    @classmethod
    def from_json(cls, payload: str) -> "DeterministicWorldStateManager":
        """Create a manager from serialized snapshot JSON."""
        parsed = json.loads(payload)
        if not isinstance(parsed, dict):
            raise ValueError("world state json payload must decode to an object")
        return cls.from_dict(parsed)

    @staticmethod
    def _apply_entities_delta(target: dict[str, Any], raw_entities: Any) -> None:
        if not isinstance(raw_entities, Mapping):
            raise ValueError("delta.entities must be a mapping")

        for entity_id, entity_state in sorted(raw_entities.items(), key=lambda item: str(item[0])):
            normalized_id = str(entity_id)
            if entity_state is None:
                target.pop(normalized_id, None)
                continue
            if not isinstance(entity_state, Mapping):
                raise ValueError("delta.entities values must be objects or null")
            normalized_payload = dict(entity_state)
            normalized_payload.setdefault("entity_id", normalized_id)
            target[normalized_id] = normalized_payload

    @staticmethod
    def _apply_rooms_delta(target: dict[str, Any], raw_rooms: Any) -> None:
        if not isinstance(raw_rooms, Mapping):
            raise ValueError("delta.rooms must be a mapping")

        for room_id, room_state in sorted(raw_rooms.items(), key=lambda item: str(item[0])):
            normalized_id = str(room_id)
            if room_state is None:
                target.pop(normalized_id, None)
                continue
            if not isinstance(room_state, Mapping):
                raise ValueError("delta.rooms values must be objects or null")
            normalized_payload = dict(room_state)
            normalized_payload.setdefault("room_id", normalized_id)
            target[normalized_id] = normalized_payload

    @staticmethod
    def _apply_scenario_vars_delta(target: dict[str, JsonScalar], raw_vars: Any) -> None:
        if not isinstance(raw_vars, Mapping):
            raise ValueError("delta.scenario_vars must be a mapping")

        for key, value in sorted(raw_vars.items(), key=lambda item: str(item[0])):
            normalized_key = str(key)
            if value is not None and not isinstance(value, (str, int, float, bool)):
                raise ValueError("delta.scenario_vars values must be JSON scalar values")
            target[normalized_key] = value

