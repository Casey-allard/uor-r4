"""Deterministic capability metric normalization utilities."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from evaluation.metrics.capability_extractors import CapabilityExtractionResult

_CAPABILITY_KEYS = (
    "exploration_coverage",
    "quest_completion",
    "combat_performance",
    "survival_time",
    "efficiency",
)


@dataclass(frozen=True, slots=True)
class NormalizationProfile:
    """Explicit profile for bounded [0, 1] normalization."""

    minimum: float
    maximum: float

    def __post_init__(self) -> None:
        _require_finite_number(self.minimum, field_name="minimum")
        _require_finite_number(self.maximum, field_name="maximum")
        if self.maximum <= self.minimum:
            raise ValueError("maximum must be greater than minimum")

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "NormalizationProfile":
        return cls(
            minimum=_read_required_number(payload, "minimum"),
            maximum=_read_required_number(payload, "maximum"),
        )


@dataclass(frozen=True, slots=True)
class NormalizedActorMetrics:
    """Normalized capability metrics for a single actor."""

    actor_id: str
    exploration_coverage: float
    quest_completion: float
    combat_performance: float
    survival_time: float
    efficiency: float

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        for key in _CAPABILITY_KEYS:
            _require_unit_interval(getattr(self, key), field_name=key)

    def to_dict(self) -> dict[str, Any]:
        return {
            "actor_id": self.actor_id,
            "exploration_coverage": self.exploration_coverage,
            "quest_completion": self.quest_completion,
            "combat_performance": self.combat_performance,
            "survival_time": self.survival_time,
            "efficiency": self.efficiency,
        }


@dataclass(frozen=True, slots=True)
class NormalizedMetricResult:
    """Deterministic normalized capability output."""

    run_id: str
    actors: tuple[NormalizedActorMetrics, ...]
    aggregate: tuple[tuple[str, float], ...]

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if tuple(key for key, _ in self.aggregate) != _CAPABILITY_KEYS:
            raise ValueError("aggregate keys must match canonical capability key order")
        for _, value in self.aggregate:
            _require_unit_interval(value, field_name="aggregate_value")

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "actors": [actor.to_dict() for actor in self.actors],
            "aggregate": {key: value for key, value in self.aggregate},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def normalize_capability_metrics(
    capability_result: CapabilityExtractionResult | Mapping[str, Any],
    *,
    profiles: Mapping[str, NormalizationProfile | Mapping[str, Any]],
) -> NormalizedMetricResult:
    """Normalize capability metrics with explicit profile definitions."""
    capability_payload = _normalize_capability_payload(capability_result)
    run_id = _read_required_string(capability_payload, "run_id")
    profile_map = _normalize_profiles(profiles)

    actors_raw = capability_payload.get("actors")
    if not isinstance(actors_raw, Sequence) or isinstance(actors_raw, (str, bytes)):
        raise ValueError("actors must be a sequence")

    seen_actor_ids: set[str] = set()
    normalized_actors: list[NormalizedActorMetrics] = []
    for actor_raw in actors_raw:
        if not isinstance(actor_raw, Mapping):
            raise ValueError("actor entries must be mappings")
        actor_id = _read_required_string(actor_raw, "actor_id")
        if actor_id in seen_actor_ids:
            raise ValueError(f"duplicate actor_id: {actor_id}")
        seen_actor_ids.add(actor_id)
        normalized_actors.append(
            NormalizedActorMetrics(
                actor_id=actor_id,
                exploration_coverage=_normalize_value(
                    _read_required_number(actor_raw, "exploration_coverage"),
                    profile_map["exploration_coverage"],
                ),
                quest_completion=_normalize_value(
                    _read_required_number(actor_raw, "quest_completion"),
                    profile_map["quest_completion"],
                ),
                combat_performance=_normalize_value(
                    _read_required_number(actor_raw, "combat_performance"),
                    profile_map["combat_performance"],
                ),
                survival_time=_normalize_value(
                    _read_required_number(actor_raw, "survival_time"),
                    profile_map["survival_time"],
                ),
                efficiency=_normalize_value(
                    _read_required_number(actor_raw, "efficiency"),
                    profile_map["efficiency"],
                ),
            )
        )

    aggregate_raw = capability_payload.get("aggregate")
    if not isinstance(aggregate_raw, Mapping):
        raise ValueError("aggregate must be a mapping")
    normalized_aggregate = tuple(
        (
            key,
            _normalize_value(_read_required_number(aggregate_raw, key), profile_map[key]),
        )
        for key in _CAPABILITY_KEYS
    )

    return NormalizedMetricResult(
        run_id=run_id,
        actors=tuple(sorted(normalized_actors, key=lambda actor: actor.actor_id)),
        aggregate=normalized_aggregate,
    )


def _normalize_capability_payload(
    capability_result: CapabilityExtractionResult | Mapping[str, Any],
) -> Mapping[str, Any]:
    if isinstance(capability_result, CapabilityExtractionResult):
        return capability_result.to_dict()
    if not isinstance(capability_result, Mapping):
        raise ValueError("capability_result must be CapabilityExtractionResult or mapping")
    return capability_result


def _normalize_profiles(
    profiles: Mapping[str, NormalizationProfile | Mapping[str, Any]],
) -> dict[str, NormalizationProfile]:
    if not isinstance(profiles, Mapping):
        raise ValueError("profiles must be a mapping")

    missing = [key for key in _CAPABILITY_KEYS if key not in profiles]
    if missing:
        raise ValueError(f"missing normalization profiles: {','.join(missing)}")

    normalized: dict[str, NormalizationProfile] = {}
    for key in _CAPABILITY_KEYS:
        raw_profile = profiles[key]
        if isinstance(raw_profile, NormalizationProfile):
            normalized[key] = raw_profile
            continue
        if not isinstance(raw_profile, Mapping):
            raise ValueError(f"profile '{key}' must be NormalizationProfile or mapping")
        normalized[key] = NormalizationProfile.from_mapping(raw_profile)
    return normalized


def _normalize_value(value: float, profile: NormalizationProfile) -> float:
    normalized = (value - profile.minimum) / (profile.maximum - profile.minimum)
    if normalized <= 0.0:
        return 0.0
    if normalized >= 1.0:
        return 1.0
    return normalized


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_number(payload: Mapping[str, Any], field_name: str) -> float:
    value = payload.get(field_name)
    _require_finite_number(value, field_name=field_name)
    return float(value)


def _require_finite_number(value: Any, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{field_name} must be finite")


def _require_unit_interval(value: float, *, field_name: str) -> None:
    _require_finite_number(value, field_name=field_name)
    if value < 0.0 or value > 1.0:
        raise ValueError(f"{field_name} must be within [0, 1]")

