from __future__ import annotations

import json

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"power-cell","entity_type":"item","location":"depot"},'
    '{"entity_id":"archive-ledger","entity_type":"item","location":"vault"}],'
    '"rooms":{"camp":{"description":"A split base camp between a dormant tram spur and a maintenance route into the vault wing.",'
    '"entities":[],"exits":{"east":"depot","north":"tram-hub"},"title":"Camp"},'
    '"depot":{"description":"A supply depot where a single power cell rests on a charging cradle.",'
    '"entities":[],"exits":{"east":"service-tunnel","west":"camp"},"title":"Depot"},'
    '"service-tunnel":{"description":"A cramped service tunnel that reaches the vault threshold without spending the cell early.",'
    '"entities":[],"exits":{"north":"vault-door","west":"depot"},"title":"Service Tunnel"},'
    '"tram-hub":{"description":"An inactive tram hub where the power cell can energize a shortcut, but doing so will drain it.",'
    '"entities":[],"exits":{"south":"camp"},"title":"Tram Hub"},'
    '"vault":{"description":"The inner archive vault, where the ledger waits once the final seal is powered.",'
    '"entities":[],"exits":{"south":"vault-door"},"title":"Vault"},'
    '"vault-door":{"description":"A sealed vault threshold whose northern lock also requires the same power cell.",'
    '"entities":[],"exits":{"south":"service-tunnel"},"title":"Vault Door"}},'
    '"unlock_effects":[{"consume_item":true,"destination_room_id":"vault-door","direction":"east","effect_id":"tram-bypass",'
    '"item_id":"power-cell","requires_actor_in_place":true,"source_room_id":"tram-hub"},'
    '{"consume_item":true,"destination_room_id":"vault","direction":"north","effect_id":"vault-seal",'
    '"item_id":"power-cell","requires_actor_in_place":true,"source_room_id":"vault-door"}]}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-delayed-cost",
        "title": "Tiny Delayed Cost",
        "description": "Tiny delayed-cost scenario where one power cell can be spent early for a shortcut or saved for the final vault seal.",
        "start_room_id": "camp",
        "max_steps": 8,
        "seed": 93,
        "version": "1.0",
        "scenario_vars": {
            "mode": "delayed-cost-planning",
            "agent_script_policy": "delayed-cost-v1",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "collect-archive-ledger",
                "objective_type": "collect_item",
                "target_id": "archive-ledger",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-delayed-cost-run",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(),
        actor_ids=("agent-a", "agent-b"),
    )


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


def test_delayed_cost_executes_with_non_trivial_multi_step_events() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.scenario_id == "tiny-delayed-cost"
    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert "step_completed" in event_types
    assert "state_snapshot" in event_types


def test_delayed_cost_progression_saves_power_cell_for_later_gate() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    move_events = [event for event in events if event["event_type"] == "action_move"]
    rooms_visited = {
        room_id
        for event in move_events
        for room_id in (event["payload"]["source_room_id"], event["payload"]["destination_room_id"])
    }
    cell_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "power-cell"
    )
    unlock = next(event for event in events if event["event_type"] == "dependency_unlocked")
    ledger_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "archive-ledger"
    )

    assert {"camp", "depot", "service-tunnel", "vault-door", "vault"} <= rooms_visited
    assert "tram-hub" not in rooms_visited
    assert cell_take["step_index"] <= unlock["step_index"] <= ledger_take["step_index"]
    assert unlock["payload"]["effect_id"] == "vault-seal"


def test_delayed_cost_scorecard_and_replay_are_meaningful() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    scorecard = payload["scorecard"]
    events = payload["replay_artifact"]["events"]  # type: ignore[index]

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-delayed-cost"
    assert payload["replay_parity_artifact"]["step_count"] == 8
    assert _final_metric_sum(events=events, actor_id="agent-a", metric_name="quest.completed") >= 2.0
