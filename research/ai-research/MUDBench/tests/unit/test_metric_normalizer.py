from __future__ import annotations

import pytest

from evaluation.normalization.metric_normalizer import (
    NormalizationProfile,
    normalize_capability_metrics,
)


def _sample_capability_payload() -> dict[str, object]:
    return {
        "run_id": "run-1",
        "actors": [
            {
                "actor_id": "agent-b",
                "exploration_coverage": 2.0,
                "quest_completion": 1.0,
                "combat_performance": -20.0,
                "survival_time": 5.0,
                "efficiency": 0.5,
            },
            {
                "actor_id": "agent-a",
                "exploration_coverage": 8.0,
                "quest_completion": 3.0,
                "combat_performance": 40.0,
                "survival_time": 15.0,
                "efficiency": 2.0,
            },
        ],
        "aggregate": {
            "exploration_coverage": 5.0,
            "quest_completion": 2.0,
            "combat_performance": 10.0,
            "survival_time": 10.0,
            "efficiency": 1.25,
        },
    }


def _sample_profiles() -> dict[str, NormalizationProfile]:
    return {
        "exploration_coverage": NormalizationProfile(minimum=0, maximum=10),
        "quest_completion": NormalizationProfile(minimum=0, maximum=4),
        "combat_performance": NormalizationProfile(minimum=-20, maximum=40),
        "survival_time": NormalizationProfile(minimum=0, maximum=20),
        "efficiency": NormalizationProfile(minimum=0, maximum=2),
    }


def test_normalize_capability_metrics_happy_path_is_deterministic() -> None:
    payload = _sample_capability_payload()
    profiles = _sample_profiles()

    first = normalize_capability_metrics(payload, profiles=profiles)
    second = normalize_capability_metrics(payload, profiles=profiles)

    assert first == second
    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.to_dict() == {
        "run_id": "run-1",
        "actors": [
            {
                "actor_id": "agent-a",
                "exploration_coverage": 0.8,
                "quest_completion": 0.75,
                "combat_performance": 1.0,
                "survival_time": 0.75,
                "efficiency": 1.0,
            },
            {
                "actor_id": "agent-b",
                "exploration_coverage": 0.2,
                "quest_completion": 0.25,
                "combat_performance": 0.0,
                "survival_time": 0.25,
                "efficiency": 0.25,
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


def test_normalize_capability_metrics_clamps_out_of_range_values() -> None:
    payload = _sample_capability_payload()
    payload["aggregate"] = {
        "exploration_coverage": -5.0,
        "quest_completion": 99.0,
        "combat_performance": -999.0,
        "survival_time": 500.0,
        "efficiency": 3.0,
    }
    profiles = _sample_profiles()

    result = normalize_capability_metrics(payload, profiles=profiles)
    assert result.to_dict()["aggregate"] == {
        "exploration_coverage": 0.0,
        "quest_completion": 1.0,
        "combat_performance": 0.0,
        "survival_time": 1.0,
        "efficiency": 1.0,
    }


def test_normalize_capability_metrics_rejects_missing_profiles() -> None:
    payload = _sample_capability_payload()
    profiles = _sample_profiles()
    del profiles["efficiency"]

    with pytest.raises(ValueError, match="missing normalization profiles: efficiency"):
        normalize_capability_metrics(payload, profiles=profiles)


def test_normalization_profile_rejects_invalid_range() -> None:
    with pytest.raises(ValueError, match="greater than minimum"):
        NormalizationProfile(minimum=1.0, maximum=1.0)


def test_normalize_capability_metrics_rejects_invalid_actor_payload() -> None:
    payload = _sample_capability_payload()
    actor = payload["actors"][0]  # type: ignore[index]
    actor["exploration_coverage"] = "bad"  # type: ignore[index]

    with pytest.raises(ValueError, match="exploration_coverage must be numeric"):
        normalize_capability_metrics(payload, profiles=_sample_profiles())


def test_normalize_capability_metrics_rejects_duplicate_actor_ids() -> None:
    payload = _sample_capability_payload()
    payload["actors"] = [payload["actors"][0], payload["actors"][0]]  # type: ignore[index]

    with pytest.raises(ValueError, match="duplicate actor_id"):
        normalize_capability_metrics(payload, profiles=_sample_profiles())

