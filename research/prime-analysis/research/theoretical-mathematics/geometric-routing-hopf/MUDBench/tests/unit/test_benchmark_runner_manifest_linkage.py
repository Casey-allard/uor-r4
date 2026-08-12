from __future__ import annotations

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig
from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "phase3-manifest-linkage-scenario",
        "title": "Phase 3 Manifest Linkage Scenario",
        "description": "Deterministic runner manifest linkage scenario.",
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


def test_runner_result_includes_run_manifest_with_consistent_fields() -> None:
    config = BenchmarkRunConfig(
        run_id="phase3-manifest-linkage-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
        max_steps_override=3,
    )
    result = run_benchmark_lifecycle(config)
    payload = result.to_dict()

    assert "run_manifest" in payload
    manifest = payload["run_manifest"]
    assert manifest["run_id"] == "phase3-manifest-linkage-run"
    assert manifest["benchmark_id"] == "phase3-benchmark"
    assert manifest["benchmark_version"] == "0.1"
    assert manifest["scenario_id"] == "phase3-manifest-linkage-scenario"
    assert manifest["scenario_version"] == "1.0"
    assert manifest["scoring_version"] == "phase3-v1"
    assert manifest["effective_seed"] == result.lifecycle_state.seed == result.scorecard.metadata.seed
    assert manifest["seed_source"] == "run_seed"
    assert manifest["actor_ids"] == ["agent-a", "agent-b"]
    assert manifest["max_steps"] == 3


def test_runner_manifest_linkage_is_deterministic_in_canonical_artifact() -> None:
    config = BenchmarkRunConfig(
        run_id="phase3-manifest-linkage-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        run_seed=17,
        max_steps_override=3,
    )
    first = run_benchmark_lifecycle(config).to_canonical_json()
    second = run_benchmark_lifecycle(config).to_canonical_json()

    assert first == second


def test_runner_manifest_linkage_works_for_legacy_wrapper_config() -> None:
    config = BenchmarkRunnerConfig(
        run_id="phase3-manifest-linkage-legacy-run",
        benchmark_id="phase3-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-b", "agent-a"),
        seed_override=29,
    )
    result = run_benchmark_lifecycle(config)

    assert result.run_manifest.seed_source == "seed_override"
    assert result.run_manifest.effective_seed == 29
    assert result.scorecard.metadata.seed == 29
