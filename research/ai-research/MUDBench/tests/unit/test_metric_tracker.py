from __future__ import annotations

import pytest

from evaluation.metrics.metric_signal import MetricSignal
from evaluation.metrics.metric_tracker import DeterministicMetricTracker


def _signal(
    *,
    run_id: str = "run-1",
    step: int,
    actor_id: str,
    metric_name: str,
    value: int | float,
) -> MetricSignal:
    return MetricSignal(
        run_id=run_id,
        step=step,
        actor_id=actor_id,
        metric_name=metric_name,
        value=value,
    )


def test_metric_tracker_accumulates_metric_with_explicit_counter_semantics() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")
    tracker.apply_signal(
        _signal(step=0, actor_id="agent-a", metric_name="coverage.rooms", value=1)
    )
    tracker.apply_signal(
        _signal(step=2, actor_id="agent-a", metric_name="coverage.rooms", value=3)
    )

    snapshot = tracker.snapshot().to_dict()
    assert snapshot == {
        "run_id": "run-1",
        "total_signals": 2,
        "actors": [
            {
                "actor_id": "agent-a",
                "metrics": [
                    {
                        "metric_name": "coverage.rooms",
                        "sample_count": 2,
                        "value_sum": 4,
                        "min_value": 1,
                        "max_value": 3,
                        "last_step": 2,
                        "last_value": 3,
                    }
                ],
            }
        ],
    }


def test_metric_tracker_snapshot_orders_actor_and_metric_keys_deterministically() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")
    tracker.apply_signals(
        (
            _signal(step=1, actor_id="agent-z", metric_name="efficiency.steps", value=7),
            _signal(step=0, actor_id="agent-a", metric_name="coverage.rooms", value=2),
            _signal(step=2, actor_id="agent-a", metric_name="efficiency.steps", value=5),
        )
    )

    snapshot = tracker.snapshot().to_dict()
    assert [actor["actor_id"] for actor in snapshot["actors"]] == ["agent-a", "agent-z"]
    assert [metric["metric_name"] for metric in snapshot["actors"][0]["metrics"]] == [
        "coverage.rooms",
        "efficiency.steps",
    ]


def test_metric_tracker_is_deterministic_for_identical_ordered_inputs() -> None:
    signals = (
        _signal(step=0, actor_id="agent-a", metric_name="coverage.rooms", value=1),
        _signal(step=1, actor_id="agent-b", metric_name="efficiency.steps", value=4),
        _signal(step=2, actor_id="agent-a", metric_name="coverage.rooms", value=2),
    )
    first = DeterministicMetricTracker(run_id="run-1")
    second = DeterministicMetricTracker(run_id="run-1")
    first.apply_signals(signals)
    second.apply_signals(signals)

    first_snapshot = first.snapshot()
    second_snapshot = second.snapshot()
    assert first_snapshot == second_snapshot
    assert first_snapshot.to_canonical_json() == second_snapshot.to_canonical_json()


def test_metric_tracker_rejects_signal_with_run_id_mismatch() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")

    with pytest.raises(ValueError, match="run_id mismatch"):
        tracker.apply_signal(
            _signal(run_id="run-2", step=0, actor_id="agent-a", metric_name="score.raw", value=1)
        )


def test_metric_tracker_rejects_invalid_signal_values() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")

    with pytest.raises(ValueError, match="MetricSignal"):
        tracker.apply_signal(object())  # type: ignore[arg-type]

    with pytest.raises(ValueError, match="sequence"):
        tracker.apply_signals("not-a-sequence")  # type: ignore[arg-type]


def test_metric_tracker_empty_snapshot_is_stable() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")
    first = tracker.snapshot()
    second = tracker.snapshot()

    assert first == second
    assert first.to_dict() == {"run_id": "run-1", "total_signals": 0, "actors": []}
    assert first.to_canonical_json() == second.to_canonical_json()
