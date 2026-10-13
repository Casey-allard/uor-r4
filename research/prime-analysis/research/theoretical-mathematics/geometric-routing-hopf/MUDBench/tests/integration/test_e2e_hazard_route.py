from __future__ import annotations

import json

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"bridge-kit","entity_type":"item","location":"supply-cache"},'
    '{"entity_id":"storm-core","entity_type":"item","location":"vault"}],'
    '"npcs":[{"entity_id":"raider","entity_type":"npc","health":20,"location":"ember-pass"}],'
    '"rooms":{"bridge-approach":{"description":"A broken bridge spans the final ravine, but a bridge kit could make the crossing safe.",'
    '"entities":[],"exits":{"south":"supply-cache"},"title":"Bridge Approach"},'
    '"camp":{"description":"A storm-battered camp between a guarded choke point and an abandoned supply route.",'
    '"entities":[],"exits":{"east":"supply-cache","north":"ember-pass"},"title":"Storm Camp"},'
    '"ember-pass":{"description":"A charred pass watched by a raider; the route is short but dangerous.",'
    '"entities":[],"exits":{"north":"vault","south":"camp"},"title":"Ember Pass"},'
    '"supply-cache":{"description":"A half-collapsed cache with salvage that could make the longer route safe.",'
    '"entities":[],"exits":{"north":"bridge-approach","west":"camp"},"title":"Supply Cache"},'
    '"vault":{"description":"A sealed storm vault where the storm core hums behind the last approach.",'
    '"entities":[],"exits":{"south":"ember-pass"},"title":"Storm Vault"}},'
    '"unlock_effects":[{"effect_id":"bridge-span","item_id":"bridge-kit","source_room_id":"bridge-approach",'
    '"direction":"north","destination_room_id":"vault","consume_item":false,"requires_actor_in_place":true}]}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-hazard-route",
        "title": "Tiny Hazard Route",
        "description": "Tiny hazard-route scenario with a risky guarded shortcut and a safer bridge-kit bypass.",
        "start_room_id": "camp",
        "max_steps": 8,
        "seed": 88,
        "version": "1.0",
        "scenario_vars": {
            "mode": "hazard-tradeoff",
            "agent_script_policy": "hazard-tradeoff-v1",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "collect-storm-core",
                "objective_type": "collect_item",
                "target_id": "storm-core",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-hazard-route-run",
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


def test_hazard_route_executes_with_non_trivial_multi_step_events() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.scenario_id == "tiny-hazard-route"
    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert "step_completed" in event_types
    assert "state_snapshot" in event_types


def test_hazard_route_progression_uses_bridge_kit_and_reaches_vault() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    move_events = [event for event in events if event["event_type"] == "action_move"]
    rooms_visited = {
        room_id
        for event in move_events
        for room_id in (event["payload"]["source_room_id"], event["payload"]["destination_room_id"])
    }
    kit_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "bridge-kit"
    )
    unlock = next(event for event in events if event["event_type"] == "dependency_unlocked")
    core_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "storm-core"
    )

    assert {"camp", "supply-cache", "bridge-approach", "vault"} <= rooms_visited
    assert kit_take["step_index"] <= unlock["step_index"] <= core_take["step_index"]
    assert unlock["payload"]["effect_id"] == "bridge-span"


def test_hazard_route_scorecard_and_replay_are_meaningful() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    scorecard = payload["scorecard"]
    events = payload["replay_artifact"]["events"]  # type: ignore[index]

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-hazard-route"
    assert payload["replay_parity_artifact"]["step_count"] == 8
    assert _final_metric_sum(events=events, actor_id="agent-a", metric_name="quest.completed") >= 2.0
