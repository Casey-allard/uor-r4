from __future__ import annotations

import pytest

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig
from evaluation.benchmark_runner.run_manifest import RunManifest, build_run_manifest


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "scenario-manifest",
        "title": "Manifest Scenario",
        "description": "Scenario for canonical run manifest tests.",
        "start_room_id": "room-start",
        "max_steps": 4,
        "seed": 41,
        "version": "1.0",
        "scenario_vars": {"difficulty": "normal"},
        "objectives": [
            {
                "objective_id": "obj-a",
                "objective_type": "collect_item",
                "target_id": "item-key",
                "required_count": 1,
            }
        ],
    }


def _run_config() -> BenchmarkRunConfig:
    return BenchmarkRunConfig(
        run_id="run-manifest-1",
        benchmark_id="benchmark-phase3",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
    )


def test_build_run_manifest_is_immutable_roundtrippable_and_deterministic() -> None:
    manifest_first = build_run_manifest(
        run_config=_run_config(),
        scenario_id="scenario-manifest",
        scenario_version="1.0",
        benchmark_version="0.1",
        scoring_version="phase3-v1",
        max_steps=4,
    )
    manifest_second = build_run_manifest(
        run_config=_run_config(),
        scenario_id="scenario-manifest",
        scenario_version="1.0",
        benchmark_version="0.1",
        scoring_version="phase3-v1",
        max_steps=4,
    )

    assert manifest_first == manifest_second
    assert manifest_first.to_canonical_json() == manifest_second.to_canonical_json()

    parsed = RunManifest.from_mapping(manifest_first.to_dict())
    assert parsed == manifest_first
    assert parsed.to_canonical_json() == manifest_first.to_canonical_json()


def test_run_manifest_from_mapping_canonicalizes_config_and_actor_order() -> None:
    manifest = RunManifest.from_mapping(
        {
                "run_id": "run-manifest-2",
                "benchmark_id": "benchmark-phase3",
                "benchmark_version": "0.1",
                "scenario_id": "scenario-manifest",
                "scenario_version": "1.0",
                "scoring_version": "phase3-v1",
                "effective_seed": 41,
                "seed_source": "scenario_seed",
                "actor_ids": ["agent-b", "agent-a"],
                "max_steps": 4,
            "config": {
                "z_field": 9,
                "a_field": {"k2": 2, "k1": 1},
            },
        }
    )
    payload = manifest.to_dict()

    assert payload["actor_ids"] == ["agent-a", "agent-b"]
    assert manifest.config_json == '{"a_field":{"k1":1,"k2":2},"z_field":9}'


def test_run_manifest_rejects_invalid_payloads() -> None:
    with pytest.raises(ValueError, match="seed_source must be one of"):
        RunManifest.from_mapping(
            {
                "run_id": "run-manifest-1",
                "benchmark_id": "benchmark-phase3",
                "benchmark_version": "0.1",
                "scenario_id": "scenario-manifest",
                "scenario_version": "1.0",
                "scoring_version": "phase3-v1",
                "effective_seed": 41,
                "seed_source": "invalid",
                "actor_ids": ["agent-a"],
                "max_steps": 4,
                "config": {"a": 1},
            }
        )

    with pytest.raises(ValueError, match=r"effective_seed must be within \[0, 2147483647\]"):
        RunManifest.from_mapping(
            {
                "run_id": "run-manifest-1",
                "benchmark_id": "benchmark-phase3",
                "benchmark_version": "0.1",
                "scenario_id": "scenario-manifest",
                "scenario_version": "1.0",
                "scoring_version": "phase3-v1",
                "effective_seed": -1,
                "seed_source": "derived",
                "actor_ids": ["agent-a"],
                "max_steps": 4,
                "config": {"a": 1},
            }
        )

    with pytest.raises(ValueError, match="max_steps must be a positive integer"):
        RunManifest.from_mapping(
            {
                "run_id": "run-manifest-1",
                "benchmark_id": "benchmark-phase3",
                "benchmark_version": "0.1",
                "scenario_id": "scenario-manifest",
                "scenario_version": "1.0",
                "scoring_version": "phase3-v1",
                "effective_seed": 41,
                "seed_source": "derived",
                "actor_ids": ["agent-a"],
                "max_steps": 0,
                "config": {"a": 1},
            }
        )

    with pytest.raises(ValueError, match="run_config must be BenchmarkRunConfig or mapping"):
        build_run_manifest(
            run_config=object(),  # type: ignore[arg-type]
            scenario_id="scenario-manifest",
            scenario_version="1.0",
            benchmark_version="0.1",
            scoring_version="phase3-v1",
            max_steps=4,
        )
