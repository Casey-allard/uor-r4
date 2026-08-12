"""Deterministic capability metric primitive extractors."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from evaluation.metrics.metric_tracker import MetricTrackerSnapshot

_CAPABILITY_KEYS = (
    "exploration_coverage",
    "quest_completion",
    "combat_performance",
    "survival_time",
    "efficiency",
)


@dataclass(frozen=True, slots=True)
class ActorCapabilityMetrics:
    """Deterministic capability primitives for one actor."""

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
            value = getattr(self, key)
            _require_finite_number(value, field_name=key)

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
class CapabilityExtractionResult:
    """Deterministic capability extraction output for one run."""

    run_id: str
    actors: tuple[ActorCapabilityMetrics, ...]
    aggregate: tuple[tuple[str, float], ...]

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if len(self.aggregate) != len(_CAPABILITY_KEYS):
            raise ValueError("aggregate must contain all capability keys")

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "actors": [actor.to_dict() for actor in self.actors],
            "aggregate": {key: value for key, value in self.aggregate},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def extract_capability_metrics(
    tracker_snapshot: MetricTrackerSnapshot | Mapping[str, Any],
) -> CapabilityExtractionResult:
    """Extract deterministic capability primitive metrics from a tracker snapshot."""
    normalized_snapshot = _normalize_snapshot(tracker_snapshot)
    run_id = _read_required_string(normalized_snapshot, "run_id")
    actors_raw = normalized_snapshot.get("actors")
    if not isinstance(actors_raw, Sequence) or isinstance(actors_raw, (str, bytes)):
        raise ValueError("snapshot.actors must be a sequence")

    seen_actor_ids: set[str] = set()
    actor_capabilities: list[ActorCapabilityMetrics] = []
    for actor_raw in actors_raw:
        if not isinstance(actor_raw, Mapping):
            raise ValueError("actor entries must be mappings")
        actor_id = _read_required_string(actor_raw, "actor_id")
        if actor_id in seen_actor_ids:
            raise ValueError(f"duplicate actor_id in snapshot: {actor_id}")
        seen_actor_ids.add(actor_id)

        metrics_by_name = _normalize_actor_metrics(actor_raw)
        actor_capabilities.append(
            ActorCapabilityMetrics(
                actor_id=actor_id,
                exploration_coverage=_non_negative(
                    _metric_field(metrics_by_name, "coverage.rooms", "value_sum"),
                    metric_name="coverage.rooms",
                ),
                quest_completion=_non_negative(
                    _metric_field(metrics_by_name, "quest.completed", "value_sum")
                    + _metric_field(metrics_by_name, "social.trade.completed", "value_sum"),
                    metric_name="quest.completed+social.trade.completed",
                ),
                combat_performance=(
                    _non_negative(
                        _metric_field(metrics_by_name, "combat.damage_dealt", "value_sum"),
                        metric_name="combat.damage_dealt",
                    )
                    - _non_negative(
                        _metric_field(metrics_by_name, "combat.damage_taken", "value_sum"),
                        metric_name="combat.damage_taken",
                    )
                ),
                survival_time=_non_negative(
                    _metric_field(metrics_by_name, "survival.steps_alive", "max_value"),
                    metric_name="survival.steps_alive",
                ),
                efficiency=_compute_efficiency(metrics_by_name),
            )
        )

    ordered_actors = tuple(sorted(actor_capabilities, key=lambda actor: actor.actor_id))
    aggregate = _aggregate_capabilities(ordered_actors)
    return CapabilityExtractionResult(
        run_id=run_id,
        actors=ordered_actors,
        aggregate=aggregate,
    )


def _normalize_snapshot(
    tracker_snapshot: MetricTrackerSnapshot | Mapping[str, Any],
) -> Mapping[str, Any]:
    if isinstance(tracker_snapshot, MetricTrackerSnapshot):
        return tracker_snapshot.to_dict()
    if not isinstance(tracker_snapshot, Mapping):
        raise ValueError("tracker_snapshot must be a MetricTrackerSnapshot or mapping")
    return tracker_snapshot


def _normalize_actor_metrics(actor_raw: Mapping[str, Any]) -> dict[str, Mapping[str, Any]]:
    metrics_raw = actor_raw.get("metrics")
    if not isinstance(metrics_raw, Sequence) or isinstance(metrics_raw, (str, bytes)):
        raise ValueError("actor.metrics must be a sequence")

    normalized: dict[str, Mapping[str, Any]] = {}
    for metric_raw in metrics_raw:
        if not isinstance(metric_raw, Mapping):
            raise ValueError("metric entries must be mappings")
        metric_name = _read_required_string(metric_raw, "metric_name")
        if metric_name in normalized:
            raise ValueError(f"duplicate metric_name for actor: {metric_name}")
        _read_required_positive_int(metric_raw, "sample_count")
        _read_required_non_negative_int(metric_raw, "last_step")
        for numeric_field in ("value_sum", "min_value", "max_value", "last_value"):
            _read_required_finite_number(metric_raw, numeric_field)
        normalized[metric_name] = metric_raw
    return normalized


def _metric_field(
    metrics_by_name: Mapping[str, Mapping[str, Any]],
    metric_name: str,
    field_name: str,
) -> float:
    metric_entry = metrics_by_name.get(metric_name)
    if metric_entry is None:
        return 0.0
    return _read_required_finite_number(metric_entry, field_name)


def _compute_efficiency(metrics_by_name: Mapping[str, Mapping[str, Any]]) -> float:
    progress = _non_negative(
        _metric_field(metrics_by_name, "objective.progress", "value_sum"),
        metric_name="objective.progress",
    )
    action_count = _non_negative(
        _metric_field(metrics_by_name, "actions.count", "value_sum"),
        metric_name="actions.count",
    )
    if action_count == 0.0:
        return 0.0
    return progress / action_count


def _aggregate_capabilities(
    actor_capabilities: Sequence[ActorCapabilityMetrics],
) -> tuple[tuple[str, float], ...]:
    if len(actor_capabilities) == 0:
        return tuple((key, 0.0) for key in _CAPABILITY_KEYS)

    actor_count = float(len(actor_capabilities))
    sums = {key: 0.0 for key in _CAPABILITY_KEYS}
    for actor in actor_capabilities:
        sums["exploration_coverage"] += actor.exploration_coverage
        sums["quest_completion"] += actor.quest_completion
        sums["combat_performance"] += actor.combat_performance
        sums["survival_time"] += actor.survival_time
        sums["efficiency"] += actor.efficiency
    return tuple((key, sums[key] / actor_count) for key in _CAPABILITY_KEYS)


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


def _read_required_non_negative_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise ValueError(f"{field_name} must be a non-negative integer")
    return value


def _read_required_finite_number(payload: Mapping[str, Any], field_name: str) -> float:
    value = payload.get(field_name)
    _require_finite_number(value, field_name=field_name)
    return float(value)


def _require_finite_number(value: Any, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{field_name} must be finite")


def _non_negative(value: float, *, metric_name: str) -> float:
    if value < 0:
        raise ValueError(f"{metric_name} must be non-negative")
    return value
