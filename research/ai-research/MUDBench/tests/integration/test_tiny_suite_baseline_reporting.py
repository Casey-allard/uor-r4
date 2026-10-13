from __future__ import annotations

import json
from pathlib import Path

from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    build_tiny_suite_baseline_report,
    run_benchmark_lifecycle,
)

_SCENARIO_PATHS = (
    "scenarios/canonical/tiny_delayed_retrieval.json",
    "scenarios/canonical/tiny_fetch_quest.json",
    "scenarios/canonical/tiny_hidden_key.json",
    "scenarios/canonical/tiny_locked_path.json",
    "scenarios/canonical/tiny_social_trade.json",
)


def _suite_results() -> tuple[object, ...]:
    results = []
    for scenario_path in _SCENARIO_PATHS:
        scenario_name = scenario_path.rsplit("/", 1)[-1].removesuffix(".json")
        scenario = json.loads(Path(scenario_path).read_text(encoding="utf-8"))
        results.append(
            run_benchmark_lifecycle(
                BenchmarkRunnerConfig(
                    run_id=f"tiny-suite-{scenario_name}",
                    benchmark_id="tiny-suite-benchmark",
                    scenario=scenario,
                    actor_ids=("agent-a", "agent-b"),
                )
            )
        )
    return tuple(results)


def test_tiny_suite_baseline_report_covers_five_canonical_scenarios() -> None:
    report = build_tiny_suite_baseline_report(_suite_results())

    assert report["schema_version"] == "tiny_suite_baseline_report_v1"
    assert report["benchmark_ids"] == ["tiny-suite-benchmark"]
    assert report["scenario_count"] == 5
    assert report["entry_count"] == 10

    scenario_ids = [entry["scenario_id"] for entry in report["entries"]]
    assert sorted(set(scenario_ids)) == [
        "tiny-delayed-retrieval",
        "tiny-fetch-quest",
        "tiny-hidden-key",
        "tiny-locked-path",
        "tiny-social-trade",
    ]

    for entry in report["entries"]:
        assert entry["agent_id"] in {"agent-a", "agent-b"}
        assert isinstance(entry["aggregate_score"], float)
        assert isinstance(entry["composite_score"], float)
        assert set(entry["normalized_metrics"].keys()) == {
            "combat_performance",
            "efficiency",
            "exploration_coverage",
            "quest_completion",
            "survival_time",
        }
        assert set(entry["contributions"].keys()) == {
            "combat_performance",
            "efficiency",
            "exploration_coverage",
            "quest_completion",
            "survival_time",
        }
        assert entry["replay_ref"].startswith("sha256:")
        assert set(entry["parity_ref"].keys()) == {
            "applied_steps_hash",
            "score_summary_hash",
            "terminal_state_hash",
        }


def test_tiny_suite_baseline_report_is_deterministic() -> None:
    first = build_tiny_suite_baseline_report(_suite_results())
    second = build_tiny_suite_baseline_report(_suite_results())

    assert first == second
