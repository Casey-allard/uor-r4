from __future__ import annotations

import json
import re
from decimal import Decimal, ROUND_HALF_UP

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"trade-token","entity_type":"item","location":"token-room"}],'
    '"npcs":[{"entity_id":"trader","entity_type":"npc","health":30,"location":"market"}],'
    '"rooms":{"market":{"description":"A compact market stall where a trader watches quietly.","entities":[],'
    '"exits":{"west":"start"},"title":"Market"},'
    '"start":{"description":"A small crossroads linking the token room and market.","entities":[],'
    '"exits":{"east":"market","west":"token-room"},"title":"Crossroads"},'
    '"token-room":{"description":"A narrow supply room with a single trade token on a shelf.","entities":[],'
    '"exits":{"east":"start"},"title":"Token Room"}},'
    '"trade_effects":[{"effect_id":"market-trade","item_id":"trade-token","target_id":"trader",'
    '"reward_item_id":"artifact","reward_entity_type":"item"}]}'
)


def _scenario_payload(*, include_social_policy: bool) -> dict[str, object]:
    scenario_vars: dict[str, object] = {
        "mode": "social-trade-dependency",
        "world_config_json": _WORLD_CONFIG_JSON,
    }
    if include_social_policy:
        scenario_vars["agent_script_policy"] = "social-trade-dependency"
    return {
        "scenario_id": "tiny-social-trade",
        "title": "Tiny Social Trade",
        "description": "Tiny social/trade scenario requiring token handoff to an NPC before objective retrieval.",
        "start_room_id": "start",
        "max_steps": 7,
        "seed": 66,
        "version": "1.0",
        "scenario_vars": scenario_vars,
        "objectives": [
            {
                "objective_id": "collect-artifact",
                "objective_type": "collect_item",
                "target_id": "artifact",
                "required_count": 1,
            }
        ],
    }


def _runner_config(*, include_social_policy: bool) -> BenchmarkRunnerConfig:
    policy_suffix = "social" if include_social_policy else "no-social"
    return BenchmarkRunnerConfig(
        run_id=f"e2e-tiny-social-trade-{policy_suffix}",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(include_social_policy=include_social_policy),
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


def _round_score(value: float, *, precision: int = 6) -> float:
    quant = Decimal("1").scaleb(-precision)
    decimal_value = Decimal(str(value)).quantize(quant, rounding=ROUND_HALF_UP)
    return float(decimal_value)


def _recompute_quest_completion_contract(
    *,
    payload: dict[str, object],
    actor_id: str,
) -> dict[str, float]:
    replay = payload["replay_artifact"]  # type: ignore[index]
    scorecard = payload["scorecard"]  # type: ignore[index]
    events = replay["events"]  # type: ignore[index]
    max_steps = int(replay["envelope"]["max_steps"])  # type: ignore[index]
    quest_completed = _final_metric_sum(events=events, actor_id=actor_id, metric_name="quest.completed")
    social_trade_completed = _final_metric_sum(
        events=events, actor_id=actor_id, metric_name="social.trade.completed"
    )
    recomputed_raw = quest_completed + social_trade_completed
    recomputed_normalized = min(max(recomputed_raw / max_steps, 0.0), 1.0)

    actor_scorecard = next(actor for actor in scorecard["actors"] if actor["actor_id"] == actor_id)  # type: ignore[index]
    quest_weight = float(scorecard["normalized_weights"]["quest_completion"])  # type: ignore[index]
    recomputed_contribution = _round_score(recomputed_normalized * quest_weight)
    scorecard_contribution = float(actor_scorecard["contributions"]["quest_completion"])
    scorecard_normalized = float(actor_scorecard["normalized_metrics"]["quest_completion"])
    return {
        "quest_completed": quest_completed,
        "social_trade_completed": social_trade_completed,
        "recomputed_raw": recomputed_raw,
        "recomputed_normalized": recomputed_normalized,
        "scorecard_normalized": scorecard_normalized,
        "recomputed_contribution": recomputed_contribution,
        "scorecard_contribution": scorecard_contribution,
    }


def test_tiny_social_trade_requires_correct_social_action_for_completion() -> None:
    without_social = run_benchmark_lifecycle(_runner_config(include_social_policy=False))
    without_events = without_social.replay_artifact.to_dict()["events"]
    without_types = [event["event_type"] for event in without_events]
    artifact_take_without = [
        event
        for event in without_events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "artifact"
    ]
    assert "action_give" not in without_types
    assert len(artifact_take_without) == 0
    assert (
        _final_metric_sum(
            events=without_events,
            actor_id="agent-a",
            metric_name="social.trade.completed",
        )
        == 0.0
    )

    with_social = run_benchmark_lifecycle(_runner_config(include_social_policy=True))
    with_events = with_social.replay_artifact.to_dict()["events"]
    with_types = [event["event_type"] for event in with_events]
    artifact_take_with = [
        event
        for event in with_events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "artifact"
    ]
    assert "action_give" in with_types
    assert len(artifact_take_with) >= 1
    assert (
        _final_metric_sum(
            events=with_events,
            actor_id="agent-a",
            metric_name="social.give.completed",
        )
        >= 1.0
    )
    assert (
        _final_metric_sum(
            events=with_events,
            actor_id="agent-a",
            metric_name="social.trade.completed",
        )
        >= 1.0
    )


def test_tiny_social_trade_replay_contains_interaction_chain() -> None:
    result = run_benchmark_lifecycle(_runner_config(include_social_policy=True))
    events = result.replay_artifact.to_dict()["events"]

    give_event = next(
        event
        for event in events
        if event["event_type"] == "action_give"
        and event["payload"].get("item_id") == "trade-token"
        and event["payload"].get("target_id") == "trader"
    )
    unlock_event = next(
        event
        for event in events
        if event["event_type"] == "dependency_unlocked"
        and event["payload"].get("effect_id") == "market-trade"
    )
    artifact_take = next(
        event
        for event in events
        if event["event_type"] == "action_take" and event["payload"].get("item_id") == "artifact"
    )

    assert unlock_event["step_index"] >= give_event["step_index"]
    assert artifact_take["step_index"] >= unlock_event["step_index"]


def test_tiny_social_trade_scorecard_and_parity_are_valid() -> None:
    result = run_benchmark_lifecycle(_runner_config(include_social_policy=True))
    payload = result.to_dict()
    replay = payload["replay_artifact"]
    scorecard = payload["scorecard"]
    parity = payload["replay_parity_artifact"]
    refs = payload["replay_artifact_refs"]

    assert replay["envelope"]["scenario_id"] == "tiny-social-trade"
    assert scorecard["metadata"]["scenario_id"] == "tiny-social-trade"
    assert scorecard["aggregate_score"] > 0.0
    actor_a = next(actor for actor in scorecard["actors"] if actor["actor_id"] == "agent-a")
    assert actor_a["normalized_metrics"]["quest_completion"] > 0.0
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None
    for field_name in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity[field_name]) is not None


def test_tiny_social_trade_replay_snapshot_contract_maps_to_scorecard_quest_completion() -> None:
    without_social_payload = run_benchmark_lifecycle(_runner_config(include_social_policy=False)).to_dict()
    without_projection = _recompute_quest_completion_contract(
        payload=without_social_payload,
        actor_id="agent-a",
    )

    with_social_payload = run_benchmark_lifecycle(_runner_config(include_social_policy=True)).to_dict()
    with_projection = _recompute_quest_completion_contract(
        payload=with_social_payload,
        actor_id="agent-a",
    )

    assert without_projection["social_trade_completed"] == 0.0
    assert with_projection["social_trade_completed"] >= 1.0
    assert with_projection["recomputed_raw"] > without_projection["recomputed_raw"]

    assert without_projection["recomputed_normalized"] == without_projection["scorecard_normalized"]
    assert without_projection["recomputed_contribution"] == without_projection["scorecard_contribution"]
    assert with_projection["recomputed_normalized"] == with_projection["scorecard_normalized"]
    assert with_projection["recomputed_contribution"] == with_projection["scorecard_contribution"]


def test_tiny_social_trade_is_same_seed_deterministic() -> None:
    first = run_benchmark_lifecycle(_runner_config(include_social_policy=True))
    second = run_benchmark_lifecycle(_runner_config(include_social_policy=True))
    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()

    first_projection = _recompute_quest_completion_contract(payload=first.to_dict(), actor_id="agent-a")
    second_projection = _recompute_quest_completion_contract(payload=second.to_dict(), actor_id="agent-a")
    assert first_projection == second_projection
