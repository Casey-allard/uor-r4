from __future__ import annotations

from typing import Any, Mapping

import pytest

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _runner_payload() -> dict[str, Any]:
    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="phase3-traceability-contract-run",
            benchmark_id="phase3-benchmark",
            scenario={
                "scenario_id": "phase3-traceability-contract-scenario",
                "title": "Phase 3 Traceability Contract Scenario",
                "description": "Traceability contract unit scenario.",
                "start_room_id": "room-start",
                "max_steps": 4,
                "seed": 41,
                "version": "1.0",
                "scenario_vars": {"mode": "traceability-contract"},
                "objectives": [
                    {
                        "objective_id": "obj-a",
                        "objective_type": "collect_item",
                        "target_id": "item-key",
                        "required_count": 1,
                    }
                ],
            },
            actor_ids=("agent-b", "agent-a"),
            run_seed=17,
            max_steps_override=3,
        )
    )
    return result.to_dict()


def _validate_seed_traceability(payload: Mapping[str, Any]) -> None:
    lifecycle = payload.get("lifecycle_state")
    if not isinstance(lifecycle, Mapping):
        raise ValueError("lifecycle_state must be a mapping")
    manifest = payload.get("run_manifest")
    if not isinstance(manifest, Mapping):
        raise ValueError("run_manifest must be a mapping")
    scorecard = payload.get("scorecard")
    if not isinstance(scorecard, Mapping):
        raise ValueError("scorecard must be a mapping")
    metadata = scorecard.get("metadata")
    if not isinstance(metadata, Mapping):
        raise ValueError("scorecard.metadata must be a mapping")

    if metadata.get("seed") != lifecycle.get("seed") or metadata.get("seed") != manifest.get("effective_seed"):
        raise ValueError("seed traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest")
    if metadata.get("step_count") != lifecycle.get("step_index") or metadata.get("step_count") != manifest.get(
        "max_steps"
    ):
        raise ValueError("step_count traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest")
    if metadata.get("scenario_id") != lifecycle.get("scenario_id") or metadata.get("scenario_id") != manifest.get(
        "scenario_id"
    ):
        raise ValueError("scenario_id traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest")
    if metadata.get("run_id") != lifecycle.get("run_id") or metadata.get("run_id") != manifest.get("run_id"):
        raise ValueError("run_id traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest")
    if metadata.get("benchmark_id") != manifest.get("benchmark_id"):
        raise ValueError("benchmark_id traceability mismatch across scorecard.metadata/run_manifest")
    if metadata.get("benchmark_version") != manifest.get("benchmark_version"):
        raise ValueError("benchmark_version traceability mismatch across scorecard.metadata/run_manifest")
    if metadata.get("scenario_version") != manifest.get("scenario_version"):
        raise ValueError("scenario_version traceability mismatch across scorecard.metadata/run_manifest")
    if metadata.get("scoring_version") != manifest.get("scoring_version"):
        raise ValueError("scoring_version traceability mismatch across scorecard.metadata/run_manifest")


def test_scorecard_seed_traceability_contract_accepts_valid_runner_payload() -> None:
    payload = _runner_payload()

    _validate_seed_traceability(payload)


def test_scorecard_seed_traceability_contract_rejects_malformed_metadata_path() -> None:
    payload = _runner_payload()
    payload["scorecard"]["metadata"] = "invalid"  # type: ignore[index]

    with pytest.raises(ValueError, match="scorecard.metadata must be a mapping"):
        _validate_seed_traceability(payload)


def test_scorecard_seed_traceability_contract_rejects_seed_mismatch() -> None:
    payload = _runner_payload()
    payload["scorecard"]["metadata"]["seed"] = payload["scorecard"]["metadata"]["seed"] + 1  # type: ignore[index]

    with pytest.raises(
        ValueError, match="seed traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest"
    ):
        _validate_seed_traceability(payload)


def test_scorecard_seed_traceability_contract_rejects_step_count_mismatch() -> None:
    payload = _runner_payload()
    payload["scorecard"]["metadata"]["step_count"] = payload["scorecard"]["metadata"]["step_count"] + 1  # type: ignore[index]

    with pytest.raises(
        ValueError,
        match="step_count traceability mismatch across scorecard.metadata/lifecycle_state/run_manifest",
    ):
        _validate_seed_traceability(payload)
