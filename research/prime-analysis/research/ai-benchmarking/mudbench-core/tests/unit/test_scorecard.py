from __future__ import annotations

import pytest

from evaluation.scorecards.scorecard import Scorecard, ScorecardMetadata, build_scorecard
from evaluation.scoring.composite_score import calculate_composite_scores


def _normalized_payload() -> dict[str, object]:
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


def _weights() -> dict[str, float]:
    return {
        "exploration_coverage": 3.0,
        "quest_completion": 3.0,
        "combat_performance": 2.0,
        "survival_time": 1.0,
        "efficiency": 1.0,
    }


def _metadata() -> ScorecardMetadata:
    return ScorecardMetadata(
        run_id="run-1",
        benchmark_id="benchmark-phase3",
        scenario_id="scenario-alpha",
        benchmark_version="0.1",
        scenario_version="1.0",
        seed=41,
        step_count=12,
        scorer_version="phase3-v1",
    )


def _composite_result_dict() -> dict[str, object]:
    result = calculate_composite_scores(_normalized_payload(), weights=_weights())
    return result.to_dict()


def test_build_scorecard_roundtrips_and_is_deterministic() -> None:
    composite_payload = _composite_result_dict()
    first = build_scorecard(metadata=_metadata(), composite_result=composite_payload)
    second = build_scorecard(metadata=_metadata(), composite_result=composite_payload)

    assert first == second
    assert first.to_canonical_json() == second.to_canonical_json()
    assert [actor.actor_id for actor in first.actors] == ["agent-a", "agent-b"]

    parsed = Scorecard.from_dict(first.to_dict())
    assert parsed == first


def test_build_scorecard_rejects_metadata_run_id_mismatch() -> None:
    metadata = ScorecardMetadata(
        run_id="run-2",
        benchmark_id="benchmark-phase3",
        scenario_id="scenario-alpha",
        benchmark_version="0.1",
        scenario_version="1.0",
        seed=41,
        step_count=12,
        scorer_version="phase3-v1",
    )
    with pytest.raises(ValueError, match="metadata.run_id must match composite_result.run_id"):
        build_scorecard(metadata=metadata, composite_result=_composite_result_dict())


def test_scorecard_from_dict_rejects_missing_metric_key() -> None:
    scorecard = build_scorecard(metadata=_metadata(), composite_result=_composite_result_dict())
    payload = scorecard.to_dict()
    del payload["aggregate_metrics"]["efficiency"]

    with pytest.raises(ValueError, match="aggregate_metrics missing keys: efficiency"):
        Scorecard.from_dict(payload)


def test_scorecard_from_dict_rejects_out_of_range_actor_score() -> None:
    scorecard = build_scorecard(metadata=_metadata(), composite_result=_composite_result_dict())
    payload = scorecard.to_dict()
    payload["actors"][0]["composite_score"] = 1.2

    with pytest.raises(ValueError, match="composite_score must be within \\[0, 1\\]"):
        Scorecard.from_dict(payload)


def test_scorecard_metadata_rejects_invalid_fields() -> None:
    with pytest.raises(ValueError, match="benchmark_id must be a non-empty string"):
        ScorecardMetadata(
            run_id="run-1",
            benchmark_id="",
            scenario_id="scenario-alpha",
            benchmark_version="0.1",
            scenario_version="1.0",
            seed=41,
            step_count=12,
            scorer_version="phase3-v1",
        )

    with pytest.raises(ValueError, match="seed must be an integer"):
        ScorecardMetadata(
            run_id="run-1",
            benchmark_id="benchmark-phase3",
            scenario_id="scenario-alpha",
            benchmark_version="0.1",
            scenario_version="1.0",
            seed=True,  # type: ignore[arg-type]
            step_count=12,
            scorer_version="phase3-v1",
        )


def test_build_scorecard_accepts_mapping_metadata_input() -> None:
    metadata_mapping = {
        "run_id": "run-1",
        "benchmark_id": "benchmark-phase3",
        "benchmark_version": "0.1",
        "scenario_id": "scenario-alpha",
        "scenario_version": "1.0",
        "seed": 41,
        "step_count": 12,
        "scoring_version": "phase3-v1",
        "scorer_version": "phase3-v1",
    }
    scorecard = build_scorecard(
        metadata=metadata_mapping,
        composite_result=_composite_result_dict(),
    )

    assert scorecard.metadata.to_dict() == metadata_mapping


def test_scorecard_metadata_rejects_mismatched_scoring_version_aliases() -> None:
    with pytest.raises(ValueError, match="scoring_version must match scorer_version when both are provided"):
        ScorecardMetadata.from_mapping(
            {
                "run_id": "run-1",
                "benchmark_id": "benchmark-phase3",
                "benchmark_version": "0.1",
                "scenario_id": "scenario-alpha",
                "scenario_version": "1.0",
                "seed": 41,
                "step_count": 12,
                "scoring_version": "phase3-v2",
                "scorer_version": "phase3-v1",
            }
        )
