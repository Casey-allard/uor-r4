from __future__ import annotations

import json
from pathlib import Path

from core.event_logger import EventRecord, normalize_payload
from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    _build_runtime_step_signals,
    _build_runtime_state_snapshot_json,
    _build_runtime_telemetry_summary,
    _normalize_runtime_telemetry_record,
    build_tiny_suite_baseline_report,
    run_benchmark_lifecycle,
)


def _metric_count(signals: tuple[object, ...], *, actor_id: str, metric_name: str) -> int:
    return sum(
        1
        for signal in signals
        if getattr(signal, "actor_id", None) == actor_id
        and getattr(signal, "metric_name", None) == metric_name
    )


def test_runtime_signals_emits_social_give_signal_but_not_trade_success_for_plain_give() -> None:
    signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=1,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=1,
                event_type="action_give",
                actor_id="agent-a",
                payload=normalize_payload({"item_id": "note", "target_id": "trader"}),
            ),
        ),
    )

    assert _metric_count(signals, actor_id="agent-a", metric_name="social.give.completed") == 1
    assert _metric_count(signals, actor_id="agent-a", metric_name="social.trade.completed") == 0


def test_runtime_signals_emits_social_trade_success_only_for_trade_completion_unlock() -> None:
    signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=2,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=2,
                event_type="dependency_unlocked",
                actor_id="agent-a",
                payload=normalize_payload(
                    {
                        "effect_id": "market-trade",
                        "item_id": "trade-token",
                        "target_id": "trader",
                        "reward_item_id": "artifact",
                    }
                ),
            ),
        ),
    )
    assert _metric_count(signals, actor_id="agent-a", metric_name="social.trade.completed") == 1

    non_trade_signals = _build_runtime_step_signals(
        run_id="run-social-signal",
        step=2,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(
            EventRecord(
                step_index=2,
                event_type="dependency_unlocked",
                actor_id="agent-a",
                payload=normalize_payload(
                    {
                        "effect_id": "sealed_gate",
                        "item_id": "brass-key",
                        "source_room_id": "lock-ante",
                        "direction": "north",
                        "destination_room_id": "treasure",
                    }
                ),
            ),
        ),
    )
    assert _metric_count(
        non_trade_signals, actor_id="agent-a", metric_name="social.trade.completed"
    ) == 0


def test_runtime_signals_emit_llm_runtime_metrics_from_sidecar_records() -> None:
    signals = _build_runtime_step_signals(
        run_id="run-llm-signal",
        step=3,
        actor_ids=("agent-a",),
        accepted_action_count=1,
        gateway_failures=(),
        emitted_events=(),
        runtime_telemetry_records=(
            {
                "actor_id": "agent-a",
                "run_id": "run-llm-signal",
                "step": 3,
                "repair_used": True,
                "fail_closed_used": False,
                "final_parse_status": "accepted_after_repair",
                "failure_reason": "invalid_json",
                "provider_request_count": 2,
                "provider_latency_ms": 25.0,
            },
        ),
    )

    assert _metric_count(signals, actor_id="agent-a", metric_name="llm.runtime.turns") == 1
    assert _metric_count(signals, actor_id="agent-a", metric_name="llm.runtime.repair_used") == 1
    assert _metric_count(signals, actor_id="agent-a", metric_name="llm.runtime.fail_closed_used") == 1
    assert (
        _metric_count(
            signals,
            actor_id="agent-a",
            metric_name="llm.runtime.final_parse_status.accepted_after_repair",
        )
        == 1
    )


def test_runtime_telemetry_summary_is_deterministic_and_actor_grouped() -> None:
    records = (
        {
            "actor_id": "agent-a",
            "run_id": "run-llm-signal",
            "step": 1,
            "repair_used": False,
            "fail_closed_used": False,
            "final_parse_status": "accepted_initial",
            "provider_request_count": 1,
            "provider_latency_ms": 12.0,
        },
        {
            "actor_id": "agent-a",
            "run_id": "run-llm-signal",
            "step": 2,
            "repair_used": True,
            "fail_closed_used": True,
            "final_parse_status": "fail_closed",
            "failure_reason": "model_output_rejected_after_repair:invalid_json",
            "provider_request_count": 2,
            "provider_latency_ms": 15.0,
        },
    )

    first = _build_runtime_telemetry_summary(records)
    second = _build_runtime_telemetry_summary(records)

    assert first == second
    assert first == [
        {
            "actor_id": "agent-a",
            "turn_count": 2,
            "repair_used_count": 1,
            "fail_closed_used_count": 1,
            "provider_request_count_total": 3,
            "provider_latency_ms_total": 27.0,
            "final_parse_status_counts": {
                "accepted_initial": 1,
                "fail_closed": 1,
            },
            "failure_reasons": ["model_output_rejected_after_repair:invalid_json"],
        }
    ]


def test_runtime_state_snapshot_includes_runtime_telemetry_summary() -> None:
    scenario = json.loads(Path("scenarios/canonical/tiny_fetch_quest.json").read_text(encoding="utf-8"))
    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="unit-runtime-telemetry-run",
            benchmark_id="unit-benchmark",
            scenario=scenario,
            actor_ids=("agent-a",),
        )
    )

    state_json = _build_runtime_state_snapshot_json(
        run_manifest=result.run_manifest,
        tracker_snapshot=result.tracker_snapshot,
        step_index=result.lifecycle_state.step_index,
        runtime_telemetry_summary=_build_runtime_telemetry_summary(
            (
                _normalize_runtime_telemetry_record(
                    {
                        "actor_id": "agent-a",
                        "run_id": "unit-runtime-telemetry-run",
                        "step": 0,
                        "repair_used": True,
                        "fail_closed_used": False,
                        "final_parse_status": "accepted_after_repair",
                        "failure_reason": "invalid_json",
                        "provider_name": "openai-chat-completions",
                        "provider_request_count": 2,
                        "provider_latency_ms": 10.0,
                    }
                ),
            )
        ),
    )
    payload = json.loads(state_json)

    assert payload["runtime_telemetry_summary"] == [
        {
            "actor_id": "agent-a",
            "turn_count": 1,
            "repair_used_count": 1,
            "fail_closed_used_count": 0,
            "provider_request_count_total": 2,
            "provider_latency_ms_total": 10.0,
            "final_parse_status_counts": {"accepted_after_repair": 1},
            "failure_reasons": ["invalid_json"],
        }
    ]


def test_tiny_suite_baseline_report_includes_required_fields_for_configured_agents_only() -> None:
    scenario = json.loads(Path("scenarios/canonical/tiny_fetch_quest.json").read_text(encoding="utf-8"))

    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="unit-suite-report-run",
            benchmark_id="unit-benchmark",
            scenario=scenario,
            actor_ids=("agent-a", "agent-b"),
        )
    )

    report = build_tiny_suite_baseline_report((result,))

    assert report["schema_version"] == "tiny_suite_baseline_report_v1"
    assert report["benchmark_ids"] == ["unit-benchmark"]
    assert report["scenario_count"] == 1
    assert report["entry_count"] == 2

    entries = report["entries"]
    assert [entry["agent_id"] for entry in entries] == ["agent-a", "agent-b"]
    assert all(entry["scenario_id"] == "tiny-fetch-quest" for entry in entries)
    assert all(isinstance(entry["aggregate_score"], float) for entry in entries)
    assert all(isinstance(entry["composite_score"], float) for entry in entries)
    assert all(entry["replay_ref"].startswith("sha256:") for entry in entries)
    assert all(
        set(entry["parity_ref"].keys()) == {"terminal_state_hash", "applied_steps_hash", "score_summary_hash"}
        for entry in entries
    )
    assert all("quest_completion" in entry["normalized_metrics"] for entry in entries)
    assert all("quest_completion" in entry["contributions"] for entry in entries)
