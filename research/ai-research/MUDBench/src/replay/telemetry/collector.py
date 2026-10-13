"""Deterministic telemetry metrics collector over replay artifacts."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping

from replay.logging.replay_artifact import (
    ReplayArtifact,
    ReplayArtifactEmitResult,
    parse_replay_artifact,
)

TELEMETRY_CATEGORIES = ("action", "rejection", "state_change", "observation", "unknown")


@dataclass(frozen=True, slots=True)
class ActorTelemetryMetrics:
    """Per-actor deterministic telemetry summary."""

    actor_id: str
    event_count: int
    action_count: int
    rejected_count: int
    unknown_event_count: int

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        for field_name in ("event_count", "action_count", "rejected_count", "unknown_event_count"):
            value = getattr(self, field_name)
            if not isinstance(value, int) or isinstance(value, bool) or value < 0:
                raise ValueError(f"{field_name} must be a non-negative integer")

    def to_dict(self) -> dict[str, Any]:
        return {
            "actor_id": self.actor_id,
            "event_count": self.event_count,
            "action_count": self.action_count,
            "rejected_count": self.rejected_count,
            "unknown_event_count": self.unknown_event_count,
        }


@dataclass(frozen=True, slots=True)
class ReplayTelemetrySnapshot:
    """Immutable deterministic telemetry snapshot for one replay artifact."""

    run_id: str
    initial_seed: int
    seed_source: str
    max_steps: int
    event_count: int
    step_event_counts: tuple[tuple[int, int], ...]
    event_type_counts: tuple[tuple[str, int], ...]
    category_counts: tuple[tuple[str, int], ...]
    unknown_event_types: tuple[str, ...]
    actor_metrics: tuple[ActorTelemetryMetrics, ...]

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.initial_seed, int) or isinstance(self.initial_seed, bool) or self.initial_seed < 0:
            raise ValueError("initial_seed must be a non-negative integer")
        if not isinstance(self.seed_source, str) or not self.seed_source:
            raise ValueError("seed_source must be a non-empty string")
        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")
        if not isinstance(self.event_count, int) or isinstance(self.event_count, bool) or self.event_count < 0:
            raise ValueError("event_count must be a non-negative integer")

        self._validate_step_counts()
        self._validate_event_type_counts()
        self._validate_category_counts()
        self._validate_unknown_event_types()
        self._validate_actor_metrics()

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "initial_seed": self.initial_seed,
            "seed_source": self.seed_source,
            "max_steps": self.max_steps,
            "event_count": self.event_count,
            "step_event_counts": [
                {"step_index": step_index, "count": count}
                for step_index, count in self.step_event_counts
            ],
            "event_type_counts": [
                {"event_type": event_type, "count": count}
                for event_type, count in self.event_type_counts
            ],
            "category_counts": [
                {"category": category, "count": count}
                for category, count in self.category_counts
            ],
            "unknown_event_types": list(self.unknown_event_types),
            "actor_metrics": [item.to_dict() for item in self.actor_metrics],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)

    def _validate_step_counts(self) -> None:
        previous_step = -1
        for step_index, count in self.step_event_counts:
            if not isinstance(step_index, int) or isinstance(step_index, bool) or step_index < 0:
                raise ValueError("step_event_counts step_index must be a non-negative integer")
            if step_index <= previous_step:
                raise ValueError("step_event_counts must be strictly sorted by step_index")
            previous_step = step_index
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                raise ValueError("step_event_counts count must be a non-negative integer")

    def _validate_event_type_counts(self) -> None:
        previous_type = ""
        for event_type, count in self.event_type_counts:
            if not isinstance(event_type, str) or not event_type:
                raise ValueError("event_type_counts event_type must be a non-empty string")
            if previous_type and event_type <= previous_type:
                raise ValueError("event_type_counts must be strictly sorted by event_type")
            previous_type = event_type
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                raise ValueError("event_type_counts count must be a non-negative integer")

    def _validate_category_counts(self) -> None:
        categories = tuple(category for category, _ in self.category_counts)
        if categories != TELEMETRY_CATEGORIES:
            raise ValueError("category_counts must include all telemetry categories in canonical order")
        for _, count in self.category_counts:
            if not isinstance(count, int) or isinstance(count, bool) or count < 0:
                raise ValueError("category_counts count must be a non-negative integer")

    def _validate_unknown_event_types(self) -> None:
        previous = ""
        for event_type in self.unknown_event_types:
            if not isinstance(event_type, str) or not event_type:
                raise ValueError("unknown_event_types must contain non-empty strings")
            if previous and event_type <= previous:
                raise ValueError("unknown_event_types must be strictly sorted")
            previous = event_type

    def _validate_actor_metrics(self) -> None:
        previous_actor = ""
        for metrics in self.actor_metrics:
            if not isinstance(metrics, ActorTelemetryMetrics):
                raise ValueError("actor_metrics must contain ActorTelemetryMetrics values")
            if previous_actor and metrics.actor_id <= previous_actor:
                raise ValueError("actor_metrics must be strictly sorted by actor_id")
            previous_actor = metrics.actor_id


@dataclass(frozen=True, slots=True)
class ReplayTelemetryCollectionResult:
    """Explicit accept/reject telemetry collection result."""

    accepted: bool
    snapshot: ReplayTelemetrySnapshot | None = None
    reason: str | None = None
    artifact_result: ReplayArtifactEmitResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.snapshot is None:
                raise ValueError("accepted telemetry result must include snapshot")
            if self.reason is not None:
                raise ValueError("accepted telemetry result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected telemetry result must include reason")
        if self.snapshot is not None:
            raise ValueError("rejected telemetry result must not include snapshot")


def collect_replay_telemetry(artifact: object) -> ReplayTelemetryCollectionResult:
    """Collect deterministic telemetry metrics from replay artifact events."""
    resolved_artifact, artifact_result, reason = _resolve_artifact(artifact)
    if reason is not None:
        return _rejected_result(reason=reason, artifact_result=artifact_result)
    if resolved_artifact is None:
        raise RuntimeError("internal error resolving replay artifact")

    snapshot = _collect_snapshot(resolved_artifact)
    return _accepted_result(snapshot=snapshot, artifact_result=artifact_result)


def _collect_snapshot(artifact: ReplayArtifact) -> ReplayTelemetrySnapshot:
    step_event_counts: dict[int, int] = {}
    event_type_counts: dict[str, int] = {}
    category_counts: dict[str, int] = {category: 0 for category in TELEMETRY_CATEGORIES}
    unknown_event_types: set[str] = set()
    actor_metrics: dict[str, dict[str, int]] = {}

    for actor_id in artifact.envelope.actor_ids:
        actor_metrics[actor_id] = {
            "event_count": 0,
            "action_count": 0,
            "rejected_count": 0,
            "unknown_event_count": 0,
        }

    for event in artifact.events:
        step_event_counts[event.step_index] = step_event_counts.get(event.step_index, 0) + 1
        event_type_counts[event.event_type] = event_type_counts.get(event.event_type, 0) + 1

        category = _categorize_event_type(event.event_type)
        category_counts[category] += 1
        if category == "unknown":
            unknown_event_types.add(event.event_type)

        if event.actor_id is None:
            continue
        metrics = actor_metrics.setdefault(
            event.actor_id,
            {
                "event_count": 0,
                "action_count": 0,
                "rejected_count": 0,
                "unknown_event_count": 0,
            },
        )
        metrics["event_count"] += 1
        if category == "action":
            metrics["action_count"] += 1
        elif category == "rejection":
            metrics["rejected_count"] += 1
        elif category == "unknown":
            metrics["unknown_event_count"] += 1

    ordered_actor_ids = tuple(sorted(actor_metrics.keys()))
    ordered_actor_metrics = tuple(
        ActorTelemetryMetrics(
            actor_id=actor_id,
            event_count=actor_metrics[actor_id]["event_count"],
            action_count=actor_metrics[actor_id]["action_count"],
            rejected_count=actor_metrics[actor_id]["rejected_count"],
            unknown_event_count=actor_metrics[actor_id]["unknown_event_count"],
        )
        for actor_id in ordered_actor_ids
    )

    return ReplayTelemetrySnapshot(
        run_id=artifact.envelope.run_id,
        initial_seed=artifact.envelope.initial_seed,
        seed_source=artifact.envelope.seed_source,
        max_steps=artifact.envelope.max_steps,
        event_count=len(artifact.events),
        step_event_counts=tuple(sorted(step_event_counts.items(), key=lambda item: item[0])),
        event_type_counts=tuple(sorted(event_type_counts.items(), key=lambda item: item[0])),
        category_counts=tuple(
            (category, category_counts[category]) for category in TELEMETRY_CATEGORIES
        ),
        unknown_event_types=tuple(sorted(unknown_event_types)),
        actor_metrics=ordered_actor_metrics,
    )


def _categorize_event_type(event_type: str) -> str:
    if event_type == "action_rejected":
        return "rejection"
    if event_type.startswith("action_"):
        return "action"
    if event_type.startswith("state_"):
        return "state_change"
    if event_type.startswith("observation_"):
        return "observation"
    return "unknown"


def _resolve_artifact(
    artifact: object,
) -> tuple[ReplayArtifact | None, ReplayArtifactEmitResult | None, str | None]:
    if isinstance(artifact, ReplayArtifact):
        return artifact, None, None
    if isinstance(artifact, Mapping):
        artifact_result = parse_replay_artifact(artifact)
        if not artifact_result.accepted or artifact_result.artifact is None:
            parse_reason = artifact_result.reason or "unknown_parse_rejection"
            return None, artifact_result, f"invalid_artifact:{parse_reason}"
        return artifact_result.artifact, artifact_result, None
    return None, None, "artifact_must_be_replay_artifact_or_mapping"


def _accepted_result(
    *,
    snapshot: ReplayTelemetrySnapshot,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayTelemetryCollectionResult:
    return ReplayTelemetryCollectionResult(
        accepted=True,
        snapshot=snapshot,
        artifact_result=artifact_result,
    )


def _rejected_result(
    *,
    reason: str,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> ReplayTelemetryCollectionResult:
    return ReplayTelemetryCollectionResult(
        accepted=False,
        reason=reason,
        artifact_result=artifact_result,
    )
