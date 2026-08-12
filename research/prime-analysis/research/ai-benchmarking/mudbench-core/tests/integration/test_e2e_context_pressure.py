from __future__ import annotations

import json

from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import (
    BenchmarkRunnerConfig,
    run_benchmark_lifecycle,
)


def _scenario_payload() -> dict[str, object]:
    return dict(_SCENARIO_PRESETS["tiny-context-pressure"])


def _scenario_payload_with_consequence() -> dict[str, object]:
    """Return the context-pressure payload with the cadence-efficiency timing consequence active."""
    payload = dict(_SCENARIO_PRESETS["tiny-context-pressure"])
    scenario_vars = dict(payload["scenario_vars"])  # type: ignore[arg-type]
    scenario_vars["timing_consequence"] = "cadence_efficiency"
    payload = dict(payload)
    payload["scenario_vars"] = scenario_vars
    return payload


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-context-pressure-run",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-a", "agent-b"),
    )


def _world_config(scenario_name: str) -> dict[str, object]:
    scenario = _SCENARIO_PRESETS[scenario_name]
    scenario_vars = scenario["scenario_vars"]
    return json.loads(str(scenario_vars["world_config_json"]))


def _final_metric_sum(
    *,
    events: list[dict[str, object]],
    actor_id: str,
    metric_name: str,
) -> float:
    snapshots = [event for event in events if event["event_type"] == "state_snapshot"]
    if len(snapshots) == 0:
        return 0.0

    final_state_json = snapshots[-1]["payload"]["state_json"]  # type: ignore[index]
    state_payload = json.loads(str(final_state_json))
    for actor_state in state_payload.get("agent_states", []):
        if actor_state.get("actor_id") != actor_id:
            continue
        for metric in actor_state.get("metrics", []):
            if metric.get("metric_name") == metric_name:
                return float(metric.get("value_sum", 0.0))
        return 0.0
    return 0.0


def test_context_pressure_is_deterministic_and_completes() -> None:
    first = run_benchmark_lifecycle(_runner_config()).to_dict()
    second = run_benchmark_lifecycle(_runner_config()).to_dict()

    assert first["scorecard"] == second["scorecard"]
    assert first["replay_artifact"]["events"] == second["replay_artifact"]["events"]  # type: ignore[index]
    assert first["lifecycle_state"]["scenario_id"] == "tiny-context-pressure"
    assert first["lifecycle_state"]["status"] == "finalized"
    assert first["replay_parity_artifact"]["step_count"] == 12


def test_context_pressure_progression_reaches_archive_after_multiple_dependencies() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    coolant_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "coolant-cell"
    )
    valve_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "valve-handle"
    )
    service_unlock = next(
        event
        for event in events
        if event["event_type"] == "dependency_unlocked"
        and event["payload"].get("effect_id") == "service-bypass"
    )
    archive_unlock = next(
        event
        for event in events
        if event["event_type"] == "dependency_unlocked"
        and event["payload"].get("effect_id") == "archive-seal"
    )
    prism_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "archive-prism"
    )
    move_events = [event for event in events if event["event_type"] == "action_move"]
    rooms_visited = {
        room_id
        for event in move_events
        for room_id in (event["payload"]["source_room_id"], event["payload"]["destination_room_id"])
    }

    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert {"camp", "depot", "workshop", "service-bay", "spillway", "seal-door", "archive"} <= rooms_visited
    assert coolant_take["step_index"] <= archive_unlock["step_index"] <= prism_take["step_index"]
    assert valve_take["step_index"] <= service_unlock["step_index"] <= archive_unlock["step_index"]
    assert prism_take["step_index"] >= 10


def test_context_pressure_is_structurally_richer_than_current_richer_tiny_slices() -> None:
    scenario = _SCENARIO_PRESETS["tiny-context-pressure"]
    world = _world_config("tiny-context-pressure")
    richer_slice_ids = ("tiny-guarded-relic", "tiny-hazard-route", "tiny-delayed-cost")

    room_count = len(world["rooms"])
    item_and_npc_count = len(world["items"]) + len(world.get("npcs", []))
    unlock_count = len(world["unlock_effects"])

    assert scenario["max_steps"] == 12
    assert "relay-lift" in json.dumps(world["unlock_effects"], sort_keys=True)
    assert "archive-seal" in json.dumps(world["unlock_effects"], sort_keys=True)
    assert "signal-flare" in json.dumps(world["items"], sort_keys=True)

    for scenario_id in richer_slice_ids:
        other = _SCENARIO_PRESETS[scenario_id]
        other_world = _world_config(scenario_id)
        other_item_and_npc_count = len(other_world["items"]) + len(other_world.get("npcs", []))
        other_unlock_count = len(other_world.get("unlock_effects", []))

        assert scenario["max_steps"] > other["max_steps"]
        assert room_count > len(other_world["rooms"])
        assert item_and_npc_count > other_item_and_npc_count
        assert unlock_count > other_unlock_count

    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    events = payload["replay_artifact"]["events"]  # type: ignore[index]

    assert payload["scorecard"]["aggregate_score"] > 0.0
    assert payload["scorecard"]["metadata"]["scenario_id"] == "tiny-context-pressure"
    assert _final_metric_sum(events=events, actor_id="agent-a", metric_name="quest.completed") >= 2.0


def test_timing_consequence_zero_overhead_without_cadence() -> None:
    """Timing consequence is active but cadence=1 (default) produces zero overhead.
    Score must be identical to a run with no timing_consequence in scenario_vars."""
    result_baseline = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-no-cadence",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=None,
        )
    )
    result_with_consequence = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-consequence-no-cadence",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload_with_consequence(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=None,
        )
    )
    # cadence_interval=1 → overhead=0 → consequence leaves score unchanged
    assert result_baseline.scorecard.aggregate_score == result_with_consequence.scorecard.aggregate_score


def test_timing_consequence_penalizes_higher_cadence() -> None:
    """With the cadence-efficiency consequence active, cadence=3 must produce
    a strictly lower aggregate_score than cadence=1 (same scenario, same agent script)."""
    result_cadence1 = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-cadence1",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload_with_consequence(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=None,
        )
    )
    result_cadence3 = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-cadence3",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload_with_consequence(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=3,
        )
    )
    # cadence=3 adds overhead 2×actions_count to denominator → lower efficiency → lower aggregate
    assert result_cadence3.scorecard.aggregate_score < result_cadence1.scorecard.aggregate_score


def test_timing_consequence_ordering_stable_under_cadence() -> None:
    """The per-actor ordering from _runner_config must remain deterministic under cadence=3."""
    result = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-ordering",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload_with_consequence(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=3,
        )
    )
    payload = result.to_dict()
    assert payload["scorecard"]["metadata"]["scenario_id"] == "tiny-context-pressure"
    assert payload["scorecard"]["aggregate_score"] > 0.0
    # Run twice — must be deterministic
    result2 = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-tc-ordering",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload_with_consequence(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=3,
        )
    )
    assert result.scorecard.aggregate_score == result2.scorecard.aggregate_score


def test_cli_preset_activates_timing_consequence_under_cadence() -> None:
    """The live _SCENARIO_PRESETS['tiny-context-pressure'] preset (no manual injection)
    must activate the cadence-efficiency consequence when cadence > 1 is specified."""
    result_no_cadence = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-preset-sync-no-cadence",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=None,
        )
    )
    result_cadence3 = run_benchmark_lifecycle(
        BenchmarkRunnerConfig(
            run_id="e2e-preset-sync-cadence3",
            benchmark_id="e2e-benchmark",
            scenario=_scenario_payload(),
            actor_ids=("agent-a", "agent-b"),
            action_cadence_interval=3,
        )
    )
    # Preset now carries timing_consequence; cadence=3 must produce a lower score than cadence=1
    assert result_cadence3.scorecard.aggregate_score < result_no_cadence.scorecard.aggregate_score
