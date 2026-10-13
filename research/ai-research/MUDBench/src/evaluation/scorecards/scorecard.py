"""Deterministic scorecard schema and serialization."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from evaluation.scoring.composite_score import CompositeScoreResult

_CAPABILITY_KEYS = (
    "exploration_coverage",
    "quest_completion",
    "combat_performance",
    "survival_time",
    "efficiency",
)


@dataclass(frozen=True, slots=True)
class ScorecardMetadata:
    """Immutable run metadata for a scorecard artifact."""

    run_id: str
    benchmark_id: str
    scenario_id: str
    seed: int
    step_count: int
    benchmark_version: str = "0.1"
    scenario_version: str = "1.0"
    scorer_version: str = "phase3-v1"

    def __post_init__(self) -> None:
        for key in (
            "run_id",
            "benchmark_id",
            "benchmark_version",
            "scenario_id",
            "scenario_version",
            "scorer_version",
        ):
            value = getattr(self, key)
            if not isinstance(value, str) or not value:
                raise ValueError(f"{key} must be a non-empty string")
        if not isinstance(self.seed, int) or isinstance(self.seed, bool):
            raise ValueError("seed must be an integer")
        if not isinstance(self.step_count, int) or isinstance(self.step_count, bool) or self.step_count < 0:
            raise ValueError("step_count must be a non-negative integer")

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "ScorecardMetadata":
        scoring_version, reason = _resolve_scoring_version(payload)
        if reason is not None:
            raise ValueError(reason)
        if scoring_version is None:
            raise RuntimeError("internal error resolving scoring_version")
        return cls(
            run_id=_read_required_string(payload, "run_id"),
            benchmark_id=_read_required_string(payload, "benchmark_id"),
            benchmark_version=_read_required_string(payload, "benchmark_version"),
            scenario_id=_read_required_string(payload, "scenario_id"),
            scenario_version=_read_required_string(payload, "scenario_version"),
            seed=_read_required_int(payload, "seed"),
            step_count=_read_required_non_negative_int(payload, "step_count"),
            scorer_version=scoring_version,
        )

    @property
    def scoring_version(self) -> str:
        return self.scorer_version

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "benchmark_id": self.benchmark_id,
            "benchmark_version": self.benchmark_version,
            "scenario_id": self.scenario_id,
            "scenario_version": self.scenario_version,
            "seed": self.seed,
            "step_count": self.step_count,
            "scoring_version": self.scoring_version,
            "scorer_version": self.scorer_version,
        }


@dataclass(frozen=True, slots=True)
class ScorecardActorEntry:
    """Immutable actor score breakdown entry."""

    actor_id: str
    composite_score: float
    normalized_metrics: tuple[tuple[str, float], ...]
    contributions: tuple[tuple[str, float], ...]

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        _require_unit_interval(self.composite_score, field_name="composite_score")
        _validate_metric_tuple(self.normalized_metrics, field_name="normalized_metrics")
        _validate_metric_tuple(self.contributions, field_name="contributions")

    def to_dict(self) -> dict[str, Any]:
        return {
            "actor_id": self.actor_id,
            "composite_score": self.composite_score,
            "normalized_metrics": {key: value for key, value in self.normalized_metrics},
            "contributions": {key: value for key, value in self.contributions},
        }


@dataclass(frozen=True, slots=True)
class Scorecard:
    """Canonical immutable scorecard artifact."""

    metadata: ScorecardMetadata
    normalized_weights: tuple[tuple[str, float], ...]
    actors: tuple[ScorecardActorEntry, ...]
    aggregate_metrics: tuple[tuple[str, float], ...]
    aggregate_contributions: tuple[tuple[str, float], ...]
    aggregate_score: float

    def __post_init__(self) -> None:
        _validate_metric_tuple(self.normalized_weights, field_name="normalized_weights")
        _validate_metric_tuple(self.aggregate_metrics, field_name="aggregate_metrics")
        _validate_metric_tuple(self.aggregate_contributions, field_name="aggregate_contributions")
        _require_unit_interval(self.aggregate_score, field_name="aggregate_score")

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "Scorecard":
        metadata_raw = payload.get("metadata")
        if not isinstance(metadata_raw, Mapping):
            raise ValueError("metadata must be a mapping")
        metadata = ScorecardMetadata.from_mapping(metadata_raw)

        normalized_weights = _metric_tuple_from_mapping(
            payload.get("normalized_weights"),
            field_name="normalized_weights",
        )
        actors_raw = payload.get("actors")
        if not isinstance(actors_raw, Sequence) or isinstance(actors_raw, (str, bytes)):
            raise ValueError("actors must be a sequence")

        seen_actor_ids: set[str] = set()
        actors: list[ScorecardActorEntry] = []
        for actor_raw in actors_raw:
            if not isinstance(actor_raw, Mapping):
                raise ValueError("actor entries must be mappings")
            actor_id = _read_required_string(actor_raw, "actor_id")
            if actor_id in seen_actor_ids:
                raise ValueError(f"duplicate actor_id: {actor_id}")
            seen_actor_ids.add(actor_id)
            actors.append(
                ScorecardActorEntry(
                    actor_id=actor_id,
                    composite_score=_read_required_unit_interval(actor_raw, "composite_score"),
                    normalized_metrics=_metric_tuple_from_mapping(
                        actor_raw.get("normalized_metrics"),
                        field_name="normalized_metrics",
                    ),
                    contributions=_metric_tuple_from_mapping(
                        actor_raw.get("contributions"),
                        field_name="contributions",
                    ),
                )
            )

        aggregate_metrics = _metric_tuple_from_mapping(
            payload.get("aggregate_metrics"),
            field_name="aggregate_metrics",
        )
        aggregate_contributions = _metric_tuple_from_mapping(
            payload.get("aggregate_contributions"),
            field_name="aggregate_contributions",
        )
        aggregate_score = _read_required_unit_interval(payload, "aggregate_score")
        return cls(
            metadata=metadata,
            normalized_weights=normalized_weights,
            actors=tuple(sorted(actors, key=lambda actor: actor.actor_id)),
            aggregate_metrics=aggregate_metrics,
            aggregate_contributions=aggregate_contributions,
            aggregate_score=aggregate_score,
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "metadata": self.metadata.to_dict(),
            "normalized_weights": {key: value for key, value in self.normalized_weights},
            "actors": [actor.to_dict() for actor in self.actors],
            "aggregate_metrics": {key: value for key, value in self.aggregate_metrics},
            "aggregate_contributions": {key: value for key, value in self.aggregate_contributions},
            "aggregate_score": self.aggregate_score,
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def build_scorecard(
    *,
    metadata: ScorecardMetadata | Mapping[str, Any],
    composite_result: CompositeScoreResult | Mapping[str, Any],
) -> Scorecard:
    """Build a canonical scorecard from validated composite scoring output."""
    resolved_metadata = _normalize_metadata(metadata)
    composite_payload = _normalize_composite_payload(composite_result)
    composite_run_id = _read_required_string(composite_payload, "run_id")
    if resolved_metadata.run_id != composite_run_id:
        raise ValueError(
            "metadata.run_id must match composite_result.run_id "
            f"('{resolved_metadata.run_id}' != '{composite_run_id}')"
        )

    scorecard_payload = {
        "metadata": resolved_metadata.to_dict(),
        "normalized_weights": composite_payload.get("normalized_weights"),
        "actors": composite_payload.get("actors"),
        "aggregate_metrics": composite_payload.get("aggregate_metrics"),
        "aggregate_contributions": composite_payload.get("aggregate_contributions"),
        "aggregate_score": composite_payload.get("aggregate_score"),
    }
    return Scorecard.from_dict(scorecard_payload)


def _normalize_metadata(metadata: ScorecardMetadata | Mapping[str, Any]) -> ScorecardMetadata:
    if isinstance(metadata, ScorecardMetadata):
        return metadata
    if not isinstance(metadata, Mapping):
        raise ValueError("metadata must be ScorecardMetadata or mapping")
    return ScorecardMetadata.from_mapping(metadata)


def _normalize_composite_payload(
    composite_result: CompositeScoreResult | Mapping[str, Any],
) -> Mapping[str, Any]:
    if isinstance(composite_result, CompositeScoreResult):
        return composite_result.to_dict()
    if not isinstance(composite_result, Mapping):
        raise ValueError("composite_result must be CompositeScoreResult or mapping")
    return composite_result


def _metric_tuple_from_mapping(value: object, *, field_name: str) -> tuple[tuple[str, float], ...]:
    if not isinstance(value, Mapping):
        raise ValueError(f"{field_name} must be a mapping")

    missing = [key for key in _CAPABILITY_KEYS if key not in value]
    if missing:
        raise ValueError(f"{field_name} missing keys: {','.join(missing)}")

    extras = [key for key in value.keys() if key not in _CAPABILITY_KEYS]
    if extras:
        raise ValueError(
            f"{field_name} unexpected keys: {','.join(sorted(str(key) for key in extras))}"
        )

    return tuple((key, _read_required_unit_interval(value, key)) for key in _CAPABILITY_KEYS)


def _validate_metric_tuple(
    metrics: tuple[tuple[str, float], ...],
    *,
    field_name: str,
) -> None:
    if tuple(key for key, _ in metrics) != _CAPABILITY_KEYS:
        raise ValueError(f"{field_name} keys must match canonical capability key order")
    for key, value in metrics:
        _require_unit_interval(value, field_name=f"{field_name}.{key}")


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be an integer")
    return value


def _read_required_non_negative_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = _read_required_int(payload, field_name)
    if value < 0:
        raise ValueError(f"{field_name} must be a non-negative integer")
    return value


def _resolve_scoring_version(payload: Mapping[str, Any]) -> tuple[str | None, str | None]:
    scoring_version = payload.get("scoring_version")
    scorer_version = payload.get("scorer_version")
    if scoring_version is None and scorer_version is None:
        return None, "scoring_version must be a non-empty string"
    if scoring_version is not None and (not isinstance(scoring_version, str) or not scoring_version):
        return None, "scoring_version must be a non-empty string"
    if scorer_version is not None and (not isinstance(scorer_version, str) or not scorer_version):
        return None, "scorer_version must be a non-empty string"

    resolved = scoring_version if isinstance(scoring_version, str) else scorer_version
    if resolved is None:
        return None, "scoring_version must be a non-empty string"
    if isinstance(scoring_version, str) and isinstance(scorer_version, str) and scoring_version != scorer_version:
        return None, "scoring_version must match scorer_version when both are provided"
    return resolved, None


def _read_required_unit_interval(payload: Mapping[str, Any], field_name: str) -> float:
    value = payload.get(field_name)
    _require_unit_interval(value, field_name=field_name)
    return float(value)


def _require_unit_interval(value: Any, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{field_name} must be finite")
    if value < 0.0 or value > 1.0:
        raise ValueError(f"{field_name} must be within [0, 1]")
