from __future__ import annotations

import json
from decimal import Decimal, ROUND_HALF_UP

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"brass-key","entity_type":"item","location":"key-chamber"},'
    '{"entity_id":"artifact","entity_type":"item","location":"treasure"}],'
    '"rooms":{"entry":{"description":"The entry chamber smells of salt and stone. Paths lead east to the locksmiths and south to a sealed gate.","entities":[],'
    '"exits":{"east":"key-chamber","south":"lock-ante"},"title":"Entry Chamber"},'
    '"key-chamber":{"description":"A cramped chamber with a brass key resting on an iron stand.","entities":[],'
    '"exits":{"west":"entry"},"title":"Key Chamber"},'
    '"lock-ante":{"description":"A narrow ante-chamber with a barred northern door and a passage south.","entities":[],'
    '"exits":{"south":"entry"},"title":"Lock Ante-Chamber"},'
    '"treasure":{"description":"A tiny vault lit by phosphor moss; a prized artifact lies within.","entities":[],'
    '"exits":{"south":"lock-ante"},"title":"Treasure Vault"}},"unlock_effects":[{"effect_id":"sealed_gate","item_id":"brass-key","source_room_id":"lock-ante","direction":"north","destination_room_id":"treasure","consume_item":false,"requires_actor_in_place":true}]}'
)


def _scenario_payload(*, include_planning_policy: bool = True) -> dict[str, object]:
    scenario_vars: dict[str, object] = {
        "mode": "planning-dependency",
        "world_config_json": _WORLD_CONFIG_JSON,
    }
    if include_planning_policy:
        scenario_vars["agent_script_policy"] = "planning-dependency"

    return {
        "scenario_id": "tiny-locked-path",
        "title": "Tiny Locked Path",
        "description": "Tiny planning scenario that forces a key → unlock → artifact dependency chain.",
        "start_room_id": "entry",
        "max_steps": 8,
        "seed": 55,
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


def _runner_config(*, include_planning_policy: bool = True) -> BenchmarkRunnerConfig:
    policy_suffix = "planning" if include_planning_policy else "no-planning"
    return BenchmarkRunnerConfig(
        run_id=f"e2e-locked-path-run-{policy_suffix}",
        benchmark_id="e2e-benchmark",
        scenario=_scenario_payload(include_planning_policy=include_planning_policy),
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
    recomputed_normalized = min(max(quest_completed / max_steps, 0.0), 1.0)

    actor_scorecard = next(actor for actor in scorecard["actors"] if actor["actor_id"] == actor_id)  # type: ignore[index]
    quest_weight = float(scorecard["normalized_weights"]["quest_completion"])  # type: ignore[index]
    recomputed_contribution = _round_score(recomputed_normalized * quest_weight)
    scorecard_contribution = float(actor_scorecard["contributions"]["quest_completion"])
    scorecard_normalized = float(actor_scorecard["normalized_metrics"]["quest_completion"])
    return {
        "quest_completed": quest_completed,
        "recomputed_normalized": recomputed_normalized,
        "scorecard_normalized": scorecard_normalized,
        "recomputed_contribution": recomputed_contribution,
        "scorecard_contribution": scorecard_contribution,
    }


def test_locked_path_executes_and_emits_dependency_event() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]
    event_types = [event["event_type"] for event in events]

    assert result.lifecycle_state.status.value == "finalized"
    assert result.lifecycle_state.scenario_id == "tiny-locked-path"
    assert "action_move" in event_types
    assert "action_use" in event_types
    assert "dependency_unlocked" in event_types
    assert "action_take" in event_types


def test_locked_path_unlock_precedes_treasure_access() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    events = result.replay_artifact.to_dict()["events"]

    unlock_event = next(event for event in events if event["event_type"] == "dependency_unlocked")
    first_treasure_move = next(
        event
        for event in events
        if event["event_type"] == "action_move"
        and event["payload"].get("destination_room_id") == "treasure"
    )
    artifact_take = next(
        event
        for event in events
        if event["event_type"] == "action_take"
        and event["payload"].get("item_id") == "artifact"
    )

    assert first_treasure_move["step_index"] >= unlock_event["step_index"]
    assert artifact_take["step_index"] >= unlock_event["step_index"]


def test_locked_path_scorecard_parity_and_replay_are_valid() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    scorecard = payload["scorecard"]
    parity = payload["replay_parity_artifact"]

    assert scorecard["aggregate_score"] > 0.0
    assert scorecard["metadata"]["scenario_id"] == "tiny-locked-path"
    assert parity["step_count"] > 0
    for field in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert isinstance(parity[field], str)
        assert len(parity[field]) == 64


def test_locked_path_replay_snapshot_contract_maps_dependency_completion_to_scorecard() -> None:
    without_planning_payload = run_benchmark_lifecycle(
        _runner_config(include_planning_policy=False)
    ).to_dict()
    without_planning_projection = _recompute_quest_completion_contract(
        payload=without_planning_payload,
        actor_id="agent-a",
    )
    without_planning_events = without_planning_payload["replay_artifact"]["events"]  # type: ignore[index]

    with_planning_payload = run_benchmark_lifecycle(_runner_config(include_planning_policy=True)).to_dict()
    with_planning_projection = _recompute_quest_completion_contract(
        payload=with_planning_payload,
        actor_id="agent-a",
    )
    with_planning_events = with_planning_payload["replay_artifact"]["events"]  # type: ignore[index]

    assert "dependency_unlocked" not in [event["event_type"] for event in without_planning_events]
    assert "dependency_unlocked" in [event["event_type"] for event in with_planning_events]

    assert without_planning_projection["quest_completed"] == 1.0
    assert with_planning_projection["quest_completed"] == 2.0
    assert with_planning_projection["quest_completed"] > without_planning_projection["quest_completed"]

    assert without_planning_projection["recomputed_normalized"] == without_planning_projection["scorecard_normalized"]
    assert without_planning_projection["recomputed_contribution"] == without_planning_projection["scorecard_contribution"]
    assert with_planning_projection["recomputed_normalized"] == with_planning_projection["scorecard_normalized"]
    assert with_planning_projection["recomputed_contribution"] == with_planning_projection["scorecard_contribution"]


def test_locked_path_same_seed_is_deterministic() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())
    first_projection = _recompute_quest_completion_contract(payload=first.to_dict(), actor_id="agent-a")
    second_projection = _recompute_quest_completion_contract(payload=second.to_dict(), actor_id="agent-a")

    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()
    assert first_projection == second_projection
