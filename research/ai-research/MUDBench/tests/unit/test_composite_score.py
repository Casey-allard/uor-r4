from __future__ import annotations

import pytest

from evaluation.scoring.composite_score import calculate_composite_scores


def _sample_normalized_payload() -> dict[str, object]:
    return {
        "run_id": "run-1",
        "actors": [
            {
                "actor_id": "agent-b",
                "exploration_coverage": 0.2,
                "quest_completion": 0.25,
                "combat_performance": 0.0,
                "survival_time": 0.25,
                "efficiency": 0.25,
            },
            {
                "actor_id": "agent-a",
                "exploration_coverage": 0.8,
                "quest_completion": 0.75,
                "combat_performance": 1.0,
                "survival_time": 0.75,
                "efficiency": 1.0,
            },
        ],
        "aggregate": {
            "exploration_coverage": 0.5,
            "quest_completion": 0.5,
            "combat_performance": 0.5,
            "survival_time": 0.5,
            "efficiency": 0.625,
        },
    }


def _sample_weights() -> dict[str, float]:
    return {
        "exploration_coverage": 3.0,
        "quest_completion": 3.0,
        "combat_performance": 2.0,
        "survival_time": 1.0,
        "efficiency": 1.0,
    }


def test_calculate_composite_scores_happy_path() -> None:
    result = calculate_composite_scores(_sample_normalized_payload(), weights=_sample_weights())

    assert result.to_dict() == {
        "run_id": "run-1",
        "normalized_weights": {
            "exploration_coverage": 0.3,
            "quest_completion": 0.3,
            "combat_performance": 0.2,
            "survival_time": 0.1,
            "efficiency": 0.1,
        },
        "actors": [
            {
                "actor_id": "agent-a",
                "normalized_metrics": {
                    "exploration_coverage": 0.8,
                    "quest_completion": 0.75,
                    "combat_performance": 1.0,
                    "survival_time": 0.75,
                    "efficiency": 1.0,
                },
                "contributions": {
                    "exploration_coverage": 0.24,
                    "quest_completion": 0.225,
                    "combat_performance": 0.2,
                    "survival_time": 0.075,
                    "efficiency": 0.1,
                },
                "composite_score": 0.84,
            },
            {
                "actor_id": "agent-b",
                "normalized_metrics": {
                    "exploration_coverage": 0.2,
                    "quest_completion": 0.25,
                    "combat_performance": 0.0,
                    "survival_time": 0.25,
                    "efficiency": 0.25,
                },
                "contributions": {
                    "exploration_coverage": 0.06,
                    "quest_completion": 0.075,
                    "combat_performance": 0.0,
                    "survival_time": 0.025,
                    "efficiency": 0.025,
                },
                "composite_score": 0.185,
            },
        ],
        "aggregate_metrics": {
            "exploration_coverage": 0.5,
            "quest_completion": 0.5,
            "combat_performance": 0.5,
            "survival_time": 0.5,
            "efficiency": 0.625,
        },
        "aggregate_contributions": {
            "exploration_coverage": 0.15,
            "quest_completion": 0.15,
            "combat_performance": 0.1,
            "survival_time": 0.05,
            "efficiency": 0.0625,
        },
        "aggregate_score": 0.5125,
    }


def test_calculate_composite_scores_is_deterministic() -> None:
    first = calculate_composite_scores(_sample_normalized_payload(), weights=_sample_weights())
    second = calculate_composite_scores(_sample_normalized_payload(), weights=_sample_weights())

    assert first == second
    assert first.to_canonical_json() == second.to_canonical_json()


def test_calculate_composite_scores_rejects_missing_weight_key() -> None:
    weights = _sample_weights()
    del weights["efficiency"]

    with pytest.raises(ValueError, match="missing weight keys: efficiency"):
        calculate_composite_scores(_sample_normalized_payload(), weights=weights)


def test_calculate_composite_scores_rejects_unexpected_weight_key() -> None:
    weights = _sample_weights()
    weights["unknown_metric"] = 1.0

    with pytest.raises(ValueError, match="unexpected weight keys: unknown_metric"):
        calculate_composite_scores(_sample_normalized_payload(), weights=weights)


def test_calculate_composite_scores_rejects_non_positive_weight_total() -> None:
    weights = {
        "exploration_coverage": 0.0,
        "quest_completion": 0.0,
        "combat_performance": 0.0,
        "survival_time": 0.0,
        "efficiency": 0.0,
    }

    with pytest.raises(ValueError, match="weight total must be greater than zero"):
        calculate_composite_scores(_sample_normalized_payload(), weights=weights)


def test_calculate_composite_scores_rejects_out_of_range_normalized_values() -> None:
    payload = _sample_normalized_payload()
    actor = payload["actors"][0]  # type: ignore[index]
    actor["efficiency"] = 1.1  # type: ignore[index]

    with pytest.raises(ValueError, match="efficiency must be within \\[0, 1\\]"):
        calculate_composite_scores(payload, weights=_sample_weights())
