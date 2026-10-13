from __future__ import annotations

import json

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"relic-key","entity_type":"item","location":"armory"},'
    '{"entity_id":"relic","entity_type":"item","location":"reliquary"}],'
    '"npcs":[{"entity_id":"sentinel","entity_type":"npc","health":10,"location":"watch-post"}],'
    '"rooms":{"armory":{"description":"A cramped armory with a single iron key resting on a rack.",'
    '"entities":[],"exits":{"north":"watch-post","west":"camp"},"title":"Armory"},'
    '"camp":{"description":"A wind-beaten camp at the edge of the ruins.",'
    '"entities":[],"exits":{"east":"armory"},"title":"Camp"},'
    '"reliquary":{"description":"A sealed reliquary where the relic rests on a stone pedestal.",'
    '"entities":[],"exits":{"south":"seal-door"},"title":"Reliquary"},'
    '"seal-door":{"description":"A heavy sealed door blocks the final chamber.",'
    '"entities":[],"exits":{"west":"watch-post"},"title":"Seal Door"},'
    '"watch-post":{"description":"A narrow watch-post where a sentinel keeps vigil over the sealed door.",'
    '"entities":[],"exits":{"east":"seal-door","south":"armory"},"title":"Watch Post"}},'
    '"unlock_effects":[{"effect_id":"reliquary-seal","item_id":"relic-key","source_room_id":"seal-door",'
    '"direction":"north","destination_room_id":"reliquary","consume_item":false,"requires_actor_in_place":true}]}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-guarded-relic",
        "title": "Tiny Guarded Relic",
        "description": "Richer tiny scenario requiring navigation, defeating a sentinel, unlocking a sealed reliquary, and retrieving the relic.",
        "start_room_id": "camp",
        "max_steps": 8,
        "seed": 77,
        "version": "1.0",
        "scenario_vars": {
            "mode": "guarded-relic",
            "agent_script_policy": "guarded-relic-v1",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "collect-relic",
                "objective_type": "collect_item",
                "target_id": "relic",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-guarded-relic-run",
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


def test_guarded_relic_executes_with_non_trivial_multi_step_events() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.scenario_id == "tiny-guarded-relic"
    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_attack" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert "step_completed" in event_types
    assert "state_snapshot" in event_types


def test_guarded_relic_progression_reaches_unlock_and_relic_take_in_order() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    move_events = [event for event in events if event["event_type"] == "action_move"]
    rooms_visited = {
        room_id
        for event in move_events
        for room_id in (event["payload"]["source_room_id"], event["payload"]["destination_room_id"])
    }
    key_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "relic-key"
    )
    attack = next(
        event
        for event in events
        if event["event_type"] == "action_attack" and event["payload"].get("target_id") == "sentinel"
    )
    unlock = next(event for event in events if event["event_type"] == "dependency_unlocked")
    relic_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "relic"
    )

    assert len(rooms_visited) >= 4
    assert attack["payload"]["damage"] == 10
    assert key_take["step_index"] <= unlock["step_index"] <= relic_take["step_index"]
    assert attack["step_index"] <= unlock["step_index"]


def test_guarded_relic_scorecard_and_replay_are_meaningful() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    scorecard = payload["scorecard"]
    events = payload["replay_artifact"]["events"]  # type: ignore[index]

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-guarded-relic"
    assert payload["replay_parity_artifact"]["step_count"] == 8
    assert _final_metric_sum(events=events, actor_id="agent-a", metric_name="quest.completed") >= 2.0
