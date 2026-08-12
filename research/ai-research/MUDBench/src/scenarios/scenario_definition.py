"""Deterministic scenario schema definitions."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

ScenarioScalar = str | int | float | bool | None
_INVALID_VALUE = object()
_REQUIRED_SCENARIO_FIELDS = (
    "scenario_id",
    "title",
    "description",
    "start_room_id",
    "max_steps",
    "seed",
    "objectives",
)


@dataclass(frozen=True, slots=True)
class ScenarioObjective:
    """Single deterministic objective entry for a scenario."""

    objective_id: str
    objective_type: str
    target_id: str | None = None
    required_count: int = 1
    metadata: tuple[tuple[str, ScenarioScalar], ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.objective_id, str) or not self.objective_id:
            raise ValueError("objective_id must be a non-empty string")
        if not isinstance(self.objective_type, str) or not self.objective_type:
            raise ValueError("objective_type must be a non-empty string")
        if self.target_id is not None and (not isinstance(self.target_id, str) or not self.target_id):
            raise ValueError("target_id must be None or a non-empty string")
        if not isinstance(self.required_count, int) or isinstance(self.required_count, bool) or self.required_count <= 0:
            raise ValueError("required_count must be a positive integer")
        object.__setattr__(self, "metadata", _normalize_scalar_pairs(self.metadata, field_name="metadata"))

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "ScenarioObjective":
        objective_id = _read_required_string(payload, "objective_id")
        objective_type = _read_required_string(payload, "objective_type")
        raw_target_id = payload.get("target_id")
        target_id: str | None
        if raw_target_id is None:
            target_id = None
        elif isinstance(raw_target_id, str) and raw_target_id:
            target_id = raw_target_id
        else:
            raise ValueError("target_id must be None or a non-empty string")

        required_count = payload.get("required_count", 1)
        if not isinstance(required_count, int) or isinstance(required_count, bool) or required_count <= 0:
            raise ValueError("required_count must be a positive integer")

        raw_metadata = payload.get("metadata", {})
        if not isinstance(raw_metadata, Mapping):
            raise ValueError("metadata must be a mapping")
        return cls(
            objective_id=objective_id,
            objective_type=objective_type,
            target_id=target_id,
            required_count=required_count,
            metadata=tuple(raw_metadata.items()),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "objective_id": self.objective_id,
            "objective_type": self.objective_type,
            "target_id": self.target_id,
            "required_count": self.required_count,
            "metadata": {key: value for key, value in self.metadata},
        }


@dataclass(frozen=True, slots=True)
class ScenarioDefinition:
    """Canonical deterministic scenario definition."""

    scenario_id: str
    title: str
    description: str
    start_room_id: str
    max_steps: int
    seed: int
    objectives: tuple[ScenarioObjective, ...]
    scenario_vars: tuple[tuple[str, ScenarioScalar], ...] = ()
    version: str = "1.0"

    def __post_init__(self) -> None:
        if not isinstance(self.scenario_id, str) or not self.scenario_id:
            raise ValueError("scenario_id must be a non-empty string")
        if not isinstance(self.title, str) or not self.title:
            raise ValueError("title must be a non-empty string")
        if not isinstance(self.description, str) or not self.description:
            raise ValueError("description must be a non-empty string")
        if not isinstance(self.start_room_id, str) or not self.start_room_id:
            raise ValueError("start_room_id must be a non-empty string")
        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")
        if not isinstance(self.seed, int) or isinstance(self.seed, bool):
            raise ValueError("seed must be an integer")
        if not isinstance(self.version, str) or not self.version:
            raise ValueError("version must be a non-empty string")
        if isinstance(self.objectives, (str, bytes)) or not isinstance(self.objectives, Sequence):
            raise ValueError("objectives must be a sequence of ScenarioObjective values")
        if len(self.objectives) == 0:
            raise ValueError("objectives must contain at least one objective")

        normalized_objectives = _normalize_objectives(self.objectives)
        object.__setattr__(self, "objectives", normalized_objectives)
        object.__setattr__(
            self,
            "scenario_vars",
            _normalize_scalar_pairs(self.scenario_vars, field_name="scenario_vars"),
        )

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "ScenarioDefinition":
        missing = [field for field in _REQUIRED_SCENARIO_FIELDS if field not in payload]
        if missing:
            raise ValueError(f"missing required fields: {','.join(missing)}")

        raw_objectives = payload.get("objectives")
        if not isinstance(raw_objectives, Sequence) or isinstance(raw_objectives, (str, bytes)):
            raise ValueError("objectives must be a sequence")

        objectives: list[ScenarioObjective] = []
        for raw_objective in raw_objectives:
            if not isinstance(raw_objective, Mapping):
                raise ValueError("objective entries must be mappings")
            objectives.append(ScenarioObjective.from_mapping(raw_objective))

        raw_vars = payload.get("scenario_vars", {})
        if not isinstance(raw_vars, Mapping):
            raise ValueError("scenario_vars must be a mapping")

        version = payload.get("version", "1.0")
        if not isinstance(version, str) or not version:
            raise ValueError("version must be a non-empty string")

        return cls(
            scenario_id=_read_required_string(payload, "scenario_id"),
            title=_read_required_string(payload, "title"),
            description=_read_required_string(payload, "description"),
            start_room_id=_read_required_string(payload, "start_room_id"),
            max_steps=_read_required_positive_int(payload, "max_steps"),
            seed=_read_required_int(payload, "seed"),
            objectives=tuple(objectives),
            scenario_vars=tuple(raw_vars.items()),
            version=version,
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "scenario_id": self.scenario_id,
            "title": self.title,
            "description": self.description,
            "start_room_id": self.start_room_id,
            "max_steps": self.max_steps,
            "seed": self.seed,
            "version": self.version,
            "scenario_vars": {key: value for key, value in self.scenario_vars},
            "objectives": [objective.to_dict() for objective in self.objectives],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def _normalize_objectives(objectives: Sequence[ScenarioObjective]) -> tuple[ScenarioObjective, ...]:
    seen_ids: set[str] = set()
    normalized: list[ScenarioObjective] = []
    for objective in objectives:
        if not isinstance(objective, ScenarioObjective):
            raise ValueError("objectives must contain only ScenarioObjective values")
        if objective.objective_id in seen_ids:
            raise ValueError(f"duplicate objective_id: {objective.objective_id}")
        seen_ids.add(objective.objective_id)
        normalized.append(objective)
    normalized.sort(key=lambda objective: objective.objective_id)
    return tuple(normalized)


def _normalize_scalar_pairs(
    pairs: Sequence[tuple[str, Any]],
    *,
    field_name: str,
) -> tuple[tuple[str, ScenarioScalar], ...]:
    if isinstance(pairs, (str, bytes)) or not isinstance(pairs, Sequence):
        raise ValueError(f"{field_name} must be a sequence of key/value tuples")
    seen_keys: set[str] = set()
    normalized: list[tuple[str, ScenarioScalar]] = []
    for raw_pair in pairs:
        if not isinstance(raw_pair, tuple) or len(raw_pair) != 2:
            raise ValueError(f"{field_name} entries must be key/value tuples")
        key, raw_value = raw_pair
        if not isinstance(key, str) or not key:
            raise ValueError(f"{field_name} keys must be non-empty strings")
        if key in seen_keys:
            raise ValueError(f"duplicate key in {field_name}: {key}")
        seen_keys.add(key)
        normalized_value = _normalize_scalar_value(raw_value)
        if normalized_value is _INVALID_VALUE:
            raise ValueError(f"{field_name} values must be JSON scalar values")
        normalized.append((key, normalized_value))
    normalized.sort(key=lambda item: item[0])
    return tuple(normalized)


def _normalize_scalar_value(value: Any) -> ScenarioScalar | object:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            return _INVALID_VALUE
        return value
    return _INVALID_VALUE


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_positive_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ValueError(f"{field_name} must be a positive integer")
    return value


def _read_required_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be an integer")
    return value

