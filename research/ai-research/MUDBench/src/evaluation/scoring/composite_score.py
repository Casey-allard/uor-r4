"""Deterministic composite score calculation from normalized metrics."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from decimal import Decimal, ROUND_HALF_UP
from typing import Any, Mapping, Sequence

from evaluation.normalization.metric_normalizer import NormalizedMetricResult

_CAPABILITY_KEYS = (
    "exploration_coverage",
    "quest_completion",
    "combat_performance",
    "survival_time",
    "efficiency",
)


@dataclass(frozen=True, slots=True)
class CompositeActorScore:
    """Composite score and contribution breakdown for one actor."""

    actor_id: str
    normalized_metrics: tuple[tuple[str, float], ...]
    contributions: tuple[tuple[str, float], ...]
    composite_score: float

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        if tuple(key for key, _ in self.normalized_metrics) != _CAPABILITY_KEYS:
            raise ValueError("normalized_metrics keys must match canonical capability key order")
        if tuple(key for key, _ in self.contributions) != _CAPABILITY_KEYS:
            raise ValueError("contributions keys must match canonical capability key order")
        _require_unit_interval(self.composite_score, field_name="composite_score")

    def to_dict(self) -> dict[str, Any]:
        return {
            "actor_id": self.actor_id,
            "normalized_metrics": {key: value for key, value in self.normalized_metrics},
            "contributions": {key: value for key, value in self.contributions},
            "composite_score": self.composite_score,
        }


@dataclass(frozen=True, slots=True)
class CompositeScoreResult:
    """Deterministic composite scoring output for one run."""

    run_id: str
    normalized_weights: tuple[tuple[str, float], ...]
    actors: tuple[CompositeActorScore, ...]
    aggregate_metrics: tuple[tuple[str, float], ...]
    aggregate_contributions: tuple[tuple[str, float], ...]
    aggregate_score: float

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if tuple(key for key, _ in self.normalized_weights) != _CAPABILITY_KEYS:
            raise ValueError("normalized_weights keys must match canonical capability key order")
        if tuple(key for key, _ in self.aggregate_metrics) != _CAPABILITY_KEYS:
            raise ValueError("aggregate_metrics keys must match canonical capability key order")
        if tuple(key for key, _ in self.aggregate_contributions) != _CAPABILITY_KEYS:
            raise ValueError("aggregate_contributions keys must match canonical capability key order")
        _require_unit_interval(self.aggregate_score, field_name="aggregate_score")

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "normalized_weights": {key: value for key, value in self.normalized_weights},
            "actors": [actor.to_dict() for actor in self.actors],
            "aggregate_metrics": {key: value for key, value in self.aggregate_metrics},
            "aggregate_contributions": {key: value for key, value in self.aggregate_contributions},
            "aggregate_score": self.aggregate_score,
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def calculate_composite_scores(
    normalized_result: NormalizedMetricResult | Mapping[str, Any],
    *,
    weights: Mapping[str, float],
    precision: int = 6,
) -> CompositeScoreResult:
    """Calculate weighted deterministic composite scores."""
    if not isinstance(precision, int) or precision < 0:
        raise ValueError("precision must be a non-negative integer")

    payload = _normalize_result_payload(normalized_result)
    run_id = _read_required_string(payload, "run_id")
    normalized_weights = _normalize_weights(weights, precision=precision)

    actors_raw = payload.get("actors")
    if not isinstance(actors_raw, Sequence) or isinstance(actors_raw, (str, bytes)):
        raise ValueError("actors must be a sequence")

    seen_actor_ids: set[str] = set()
    actor_scores: list[CompositeActorScore] = []
    for actor_raw in actors_raw:
        if not isinstance(actor_raw, Mapping):
            raise ValueError("actor entries must be mappings")
        actor_id = _read_required_string(actor_raw, "actor_id")
        if actor_id in seen_actor_ids:
            raise ValueError(f"duplicate actor_id: {actor_id}")
        seen_actor_ids.add(actor_id)

        normalized_metrics = tuple(
            (key, _read_required_unit_interval(actor_raw, key)) for key in _CAPABILITY_KEYS
        )
        contributions = tuple(
            (
                key,
                _round_score(metric_value * normalized_weight, precision=precision),
            )
            for (key, metric_value), (_, normalized_weight) in zip(
                normalized_metrics, normalized_weights, strict=True
            )
        )
        composite_score = _round_score(
            sum(value for _, value in contributions),
            precision=precision,
        )
        actor_scores.append(
            CompositeActorScore(
                actor_id=actor_id,
                normalized_metrics=normalized_metrics,
                contributions=contributions,
                composite_score=composite_score,
            )
        )

    aggregate_raw = payload.get("aggregate")
    if not isinstance(aggregate_raw, Mapping):
        raise ValueError("aggregate must be a mapping")
    aggregate_metrics = tuple(
        (key, _read_required_unit_interval(aggregate_raw, key)) for key in _CAPABILITY_KEYS
    )
    aggregate_contributions = tuple(
        (
            key,
            _round_score(metric_value * normalized_weight, precision=precision),
        )
        for (key, metric_value), (_, normalized_weight) in zip(
            aggregate_metrics, normalized_weights, strict=True
        )
    )
    aggregate_score = _round_score(
        sum(value for _, value in aggregate_contributions),
        precision=precision,
    )

    return CompositeScoreResult(
        run_id=run_id,
        normalized_weights=normalized_weights,
        actors=tuple(sorted(actor_scores, key=lambda actor: actor.actor_id)),
        aggregate_metrics=aggregate_metrics,
        aggregate_contributions=aggregate_contributions,
        aggregate_score=aggregate_score,
    )


def _normalize_result_payload(
    normalized_result: NormalizedMetricResult | Mapping[str, Any],
) -> Mapping[str, Any]:
    if isinstance(normalized_result, NormalizedMetricResult):
        return normalized_result.to_dict()
    if not isinstance(normalized_result, Mapping):
        raise ValueError("normalized_result must be NormalizedMetricResult or mapping")
    return normalized_result


def _normalize_weights(
    weights: Mapping[str, float],
    *,
    precision: int,
) -> tuple[tuple[str, float], ...]:
    if not isinstance(weights, Mapping):
        raise ValueError("weights must be a mapping")

    missing_keys = [key for key in _CAPABILITY_KEYS if key not in weights]
    if missing_keys:
        raise ValueError(f"missing weight keys: {','.join(missing_keys)}")

    extra_keys = [key for key in weights.keys() if key not in _CAPABILITY_KEYS]
    if extra_keys:
        raise ValueError(f"unexpected weight keys: {','.join(sorted(str(key) for key in extra_keys))}")

    raw_weights: list[tuple[str, float]] = []
    for key in _CAPABILITY_KEYS:
        raw_value = weights[key]
        _require_finite_number(raw_value, field_name=f"weight[{key}]")
        if raw_value < 0:
            raise ValueError(f"weight[{key}] must be non-negative")
        raw_weights.append((key, float(raw_value)))

    total = sum(weight for _, weight in raw_weights)
    if total <= 0:
        raise ValueError("weight total must be greater than zero")

    return tuple(
        (key, _round_score(weight / total, precision=precision))
        for key, weight in raw_weights
    )


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_unit_interval(payload: Mapping[str, Any], field_name: str) -> float:
    value = payload.get(field_name)
    _require_unit_interval(value, field_name=field_name)
    return float(value)


def _require_unit_interval(value: Any, *, field_name: str) -> None:
    _require_finite_number(value, field_name=field_name)
    if value < 0.0 or value > 1.0:
        raise ValueError(f"{field_name} must be within [0, 1]")


def _require_finite_number(value: Any, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{field_name} must be finite")


def _round_score(value: float, *, precision: int) -> float:
    quant = Decimal("1").scaleb(-precision)
    decimal_value = Decimal(str(value)).quantize(quant, rounding=ROUND_HALF_UP)
    return float(decimal_value)

