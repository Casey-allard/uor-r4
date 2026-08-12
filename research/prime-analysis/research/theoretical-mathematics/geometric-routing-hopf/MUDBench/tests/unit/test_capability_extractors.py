from __future__ import annotations

import pytest

from evaluation.metrics.capability_extractors import extract_capability_metrics
from evaluation.metrics.metric_signal import MetricSignal
from evaluation.metrics.metric_tracker import DeterministicMetricTracker


def _signal(
    *,
    step: int,
    actor_id: str,
    metric_name: str,
    value: int | float,
    run_id: str = "run-1",
) -> MetricSignal:
    return MetricSignal(
        run_id=run_id,
        step=step,
        actor_id=actor_id,
        metric_name=metric_name,
        value=value,
    )


def _build_snapshot() -> dict[str, object]:
    tracker = DeterministicMetricTracker(run_id="run-1")
    tracker.apply_signals(
        (
            _signal(step=0, actor_id="agent-a", metric_name="coverage.rooms", value=5),
            _signal(step=1, actor_id="agent-a", metric_name="quest.completed", value=1),
            _signal(step=2, actor_id="agent-a", metric_name="combat.damage_dealt", value=30),
            _signal(step=3, actor_id="agent-a", metric_name="combat.damage_taken", value=10),
            _signal(step=4, actor_id="agent-a", metric_name="survival.steps_alive", value=20),
            _signal(step=5, actor_id="agent-a", metric_name="objective.progress", value=8),
            _signal(step=6, actor_id="agent-a", metric_name="actions.count", value=4),
            _signal(step=0, actor_id="agent-b", metric_name="coverage.rooms", value=3),
            _signal(step=1, actor_id="agent-b", metric_name="quest.completed", value=0),
            _signal(step=2, actor_id="agent-b", metric_name="combat.damage_dealt", value=5),
            _signal(step=3, actor_id="agent-b", metric_name="combat.damage_taken", value=15),
            _signal(step=4, actor_id="agent-b", metric_name="survival.steps_alive", value=18),
            _signal(step=5, actor_id="agent-b", metric_name="objective.progress", value=2),
            _signal(step=6, actor_id="agent-b", metric_name="actions.count", value=4),
        )
    )
    return tracker.snapshot().to_dict()


def test_extract_capability_metrics_computes_expected_primitives() -> None:
    snapshot = _build_snapshot()
    result = extract_capability_metrics(snapshot)

    assert result.to_dict() == {
        "run_id": "run-1",
        "actors": [
            {
                "actor_id": "agent-a",
                "exploration_coverage": 5.0,
                "quest_completion": 1.0,
                "combat_performance": 20.0,
                "survival_time": 20.0,
                "efficiency": 2.0,
            },
            {
                "actor_id": "agent-b",
                "exploration_coverage": 3.0,
                "quest_completion": 0.0,
                "combat_performance": -10.0,
                "survival_time": 18.0,
                "efficiency": 0.5,
            },
        ],
        "aggregate": {
            "exploration_coverage": 4.0,
            "quest_completion": 0.5,
            "combat_performance": 5.0,
            "survival_time": 19.0,
            "efficiency": 1.25,
        },
    }


def test_extract_capability_metrics_accepts_tracker_snapshot_object() -> None:
    tracker = DeterministicMetricTracker(run_id="run-1")
    tracker.apply_signal(_signal(step=0, actor_id="agent-z", metric_name="coverage.rooms", value=2))

    result = extract_capability_metrics(tracker.snapshot())
    assert result.to_dict()["actors"][0]["actor_id"] == "agent-z"
    assert result.to_dict()["actors"][0]["exploration_coverage"] == 2.0


def test_extract_capability_metrics_is_deterministic_for_identical_inputs() -> None:
    snapshot = _build_snapshot()
    first = extract_capability_metrics(snapshot)
    second = extract_capability_metrics(snapshot)

    assert first == second
    assert first.to_canonical_json() == second.to_canonical_json()


def test_extract_capability_metrics_rejects_non_snapshot_input() -> None:
    with pytest.raises(ValueError, match="MetricTrackerSnapshot or mapping"):
        extract_capability_metrics("invalid")  # type: ignore[arg-type]


def test_extract_capability_metrics_rejects_duplicate_actor_entries() -> None:
    snapshot = _build_snapshot()
    snapshot["actors"] = [
        snapshot["actors"][0],  # type: ignore[index]
        snapshot["actors"][0],  # type: ignore[index]
    ]

    with pytest.raises(ValueError, match="duplicate actor_id"):
        extract_capability_metrics(snapshot)


def test_extract_capability_metrics_rejects_invalid_metric_structure() -> None:
    snapshot = _build_snapshot()
    actor = snapshot["actors"][0]  # type: ignore[index]
    actor["metrics"] = [{"metric_name": "coverage.rooms"}]  # type: ignore[index]

    with pytest.raises(ValueError, match="sample_count must be a positive integer"):
        extract_capability_metrics(snapshot)


def test_extract_capability_metrics_rejects_negative_nonnegative_metrics() -> None:
    snapshot = _build_snapshot()
    actor = snapshot["actors"][0]  # type: ignore[index]
    metrics = actor["metrics"]  # type: ignore[index]
    for metric in metrics:  # type: ignore[assignment]
        if metric["metric_name"] == "actions.count":
            metric["value_sum"] = -1
            metric["min_value"] = -1
            metric["max_value"] = -1
            metric["last_value"] = -1
            break

    with pytest.raises(ValueError, match="actions.count must be non-negative"):
        extract_capability_metrics(snapshot)


def test_extract_capability_metrics_includes_social_trade_completion_in_quest_completion() -> None:
    snapshot = _build_snapshot()
    actor = snapshot["actors"][0]  # type: ignore[index]
    metrics = actor["metrics"]  # type: ignore[index]
    metrics.append(
        {
            "metric_name": "social.trade.completed",
            "sample_count": 1,
            "value_sum": 2,
            "min_value": 2,
            "max_value": 2,
            "last_step": 2,
            "last_value": 2,
        }
    )
    metrics.append(
        {
            "metric_name": "social.give.completed",
            "sample_count": 1,
            "value_sum": 5,
            "min_value": 5,
            "max_value": 5,
            "last_step": 2,
            "last_value": 5,
        }
    )

    result = extract_capability_metrics(snapshot).to_dict()
    actor_a = next(entry for entry in result["actors"] if entry["actor_id"] == "agent-a")
    assert actor_a["quest_completion"] == 3.0
