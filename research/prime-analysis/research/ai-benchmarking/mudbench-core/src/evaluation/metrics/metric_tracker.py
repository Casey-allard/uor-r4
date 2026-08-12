"""Deterministic raw metric accumulation for benchmark evaluation."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Sequence

from evaluation.metrics.metric_signal import MetricSignal


@dataclass(frozen=True, slots=True)
class MetricAggregate:
    """Aggregated counters for one actor/metric pair."""

    sample_count: int
    value_sum: int | float
    min_value: int | float
    max_value: int | float
    last_step: int
    last_value: int | float

    def __post_init__(self) -> None:
        if not isinstance(self.sample_count, int) or self.sample_count <= 0:
            raise ValueError("sample_count must be a positive integer")
        if not isinstance(self.last_step, int) or self.last_step < 0:
            raise ValueError("last_step must be a non-negative integer")
        for field_name in ("value_sum", "min_value", "max_value", "last_value"):
            value = getattr(self, field_name)
            _require_numeric(value, field_name=field_name)
        if self.min_value > self.max_value:
            raise ValueError("min_value must be less than or equal to max_value")

    @classmethod
    def from_signal(cls, signal: MetricSignal) -> "MetricAggregate":
        return cls(
            sample_count=1,
            value_sum=signal.value,
            min_value=signal.value,
            max_value=signal.value,
            last_step=signal.step,
            last_value=signal.value,
        )

    def apply_signal(self, signal: MetricSignal) -> "MetricAggregate":
        """Apply one signal using explicit counter update semantics."""
        updated_sum = self.value_sum + signal.value
        return MetricAggregate(
            sample_count=self.sample_count + 1,
            value_sum=updated_sum,
            min_value=min(self.min_value, signal.value),
            max_value=max(self.max_value, signal.value),
            last_step=signal.step,
            last_value=signal.value,
        )

    def to_dict(self) -> dict[str, int | float]:
        return {
            "sample_count": self.sample_count,
            "value_sum": self.value_sum,
            "min_value": self.min_value,
            "max_value": self.max_value,
            "last_step": self.last_step,
            "last_value": self.last_value,
        }


@dataclass(frozen=True, slots=True)
class MetricTrackerSnapshot:
    """Deterministic immutable snapshot of tracked metric aggregates."""

    run_id: str
    total_signals: int
    actors: tuple[tuple[str, tuple[tuple[str, MetricAggregate], ...]], ...]

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.total_signals, int) or self.total_signals < 0:
            raise ValueError("total_signals must be a non-negative integer")

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "total_signals": self.total_signals,
            "actors": [
                {
                    "actor_id": actor_id,
                    "metrics": [
                        {"metric_name": metric_name, **aggregate.to_dict()}
                        for metric_name, aggregate in metrics
                    ],
                }
                for actor_id, metrics in self.actors
            ],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


class DeterministicMetricTracker:
    """Per-run, per-agent deterministic metric accumulation state."""

    def __init__(self, *, run_id: str) -> None:
        if not isinstance(run_id, str) or not run_id:
            raise ValueError("run_id must be a non-empty string")
        self._run_id = run_id
        self._total_signals = 0
        self._aggregates: dict[str, dict[str, MetricAggregate]] = {}

    @property
    def run_id(self) -> str:
        return self._run_id

    def apply_signal(self, signal: MetricSignal) -> None:
        if not isinstance(signal, MetricSignal):
            raise ValueError("signal must be a MetricSignal")
        if signal.run_id != self._run_id:
            raise ValueError(
                f"signal run_id mismatch: expected '{self._run_id}', received '{signal.run_id}'"
            )

        actor_metrics = self._aggregates.setdefault(signal.actor_id, {})
        current = actor_metrics.get(signal.metric_name)
        if current is None:
            actor_metrics[signal.metric_name] = MetricAggregate.from_signal(signal)
        else:
            actor_metrics[signal.metric_name] = current.apply_signal(signal)
        self._total_signals += 1

    def apply_signals(self, signals: Sequence[MetricSignal]) -> None:
        if isinstance(signals, (str, bytes)) or not isinstance(signals, Sequence):
            raise ValueError("signals must be a sequence of MetricSignal values")
        for signal in signals:
            self.apply_signal(signal)

    def snapshot(self) -> MetricTrackerSnapshot:
        ordered_actors = tuple(
            (
                actor_id,
                tuple(
                    (metric_name, metric_aggregate)
                    for metric_name, metric_aggregate in sorted(
                        actor_metrics.items(), key=lambda item: item[0]
                    )
                ),
            )
            for actor_id, actor_metrics in sorted(self._aggregates.items(), key=lambda item: item[0])
        )
        return MetricTrackerSnapshot(
            run_id=self._run_id,
            total_signals=self._total_signals,
            actors=ordered_actors,
        )


def _require_numeric(value: object, *, field_name: str) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be numeric")
    if isinstance(value, float) and not math.isfinite(value):
        raise ValueError(f"{field_name} must be finite")
