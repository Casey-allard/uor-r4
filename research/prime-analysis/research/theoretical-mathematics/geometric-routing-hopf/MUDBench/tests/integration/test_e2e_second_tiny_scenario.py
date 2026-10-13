from __future__ import annotations

import json
import re
from decimal import Decimal, ROUND_HALF_UP

from evaluation.benchmark_runner.runner import BenchmarkRunnerConfig, run_benchmark_lifecycle

_WORLD_CONFIG_JSON = (
    '{"items":[{"entity_id":"memory-token","entity_type":"item","location":"cache"},'
    '{"entity_id":"note-fragment","entity_type":"item","location":"start"}],'
    '"rooms":{"cache":{"description":"A quiet storage alcove.","entities":[],'
    '"exits":{"west":"junction"},"title":"Cache"},'
    '"junction":{"description":"A crossroads with worn stone marks.","entities":[],'
    '"exits":{"east":"cache","west":"start"},"title":"Junction"},'
    '"start":{"description":"A small starting chamber with etched markings.","entities":[],'
    '"exits":{"east":"junction"},"title":"Start"}}}'
)


def _scenario_payload() -> dict[str, object]:
    return {
        "scenario_id": "tiny-delayed-retrieval",
        "title": "Tiny Delayed Retrieval",
        "description": "Tiny memory-focused scenario with delayed target retrieval.",
        "start_room_id": "start",
        "max_steps": 6,
        "seed": 24,
        "version": "1.0",
        "scenario_vars": {
            "mode": "vertical-slice-memory",
            "agent_script_policy": "memory_delayed_retrieval_v1",
            "world_config_json": _WORLD_CONFIG_JSON,
        },
        "objectives": [
            {
                "objective_id": "retrieve-memory-token",
                "objective_type": "collect_item",
                "target_id": "memory-token",
                "required_count": 1,
            }
        ],
    }


def _runner_config() -> BenchmarkRunnerConfig:
    return BenchmarkRunnerConfig(
        run_id="e2e-second-tiny-run",
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


def test_second_tiny_scenario_executes_end_to_end() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()
    replay_events = payload["replay_artifact"]["events"]
    event_types = [event["event_type"] for event in replay_events]

    assert payload["lifecycle_state"]["status"] == "finalized"
    assert payload["lifecycle_state"]["scenario_id"] == "tiny-delayed-retrieval"
    assert event_types.count("step_completed") == 6
    assert event_types.count("state_snapshot") == 6
    assert "action_move" in event_types
    assert "action_take" in event_types
    assert "action_attack" not in event_types


def test_second_tiny_scenario_exercises_delayed_retrieval_behavior() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    replay_events = result.replay_artifact.to_dict()["events"]
    move_events = [event for event in replay_events if event["event_type"] == "action_move"]
    take_events = [event for event in replay_events if event["event_type"] == "action_take"]

    assert len(move_events) >= 3
    assert len(take_events) >= 1
    first_take = take_events[0]
    assert first_take["payload"]["item_id"] == "memory-token"
    assert first_take["payload"]["room_id"] == "cache"
    assert first_take["step_index"] >= 4


def test_second_tiny_scenario_emits_replay_scorecard_and_parity() -> None:
    result = run_benchmark_lifecycle(_runner_config())
    payload = result.to_dict()

    replay = payload["replay_artifact"]
    parity = payload["replay_parity_artifact"]
    refs = payload["replay_artifact_refs"]
    scorecard = payload["scorecard"]

    assert replay["envelope"]["scenario_id"] == "tiny-delayed-retrieval"
    assert scorecard["metadata"]["scenario_id"] == "tiny-delayed-retrieval"
    assert scorecard["aggregate_score"] > 0.0
    assert [entry["name"] for entry in refs] == ["replay_artifact", "replay_checksum"]
    assert refs[0]["ref"] == refs[1]["ref"]
    for entry in refs:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", entry["ref"]) is not None
    for field_name in ("terminal_state_hash", "applied_steps_hash", "score_summary_hash"):
        assert re.fullmatch(r"[0-9a-f]{64}", parity[field_name]) is not None


def test_second_tiny_scenario_replay_snapshot_contract_maps_memory_retrieval_to_scorecard() -> None:
    payload = run_benchmark_lifecycle(_runner_config()).to_dict()

    actor_a_projection = _recompute_quest_completion_contract(payload=payload, actor_id="agent-a")
    actor_b_projection = _recompute_quest_completion_contract(payload=payload, actor_id="agent-b")

    assert actor_a_projection["quest_completed"] == 1.0
    assert actor_a_projection["recomputed_normalized"] == actor_a_projection["scorecard_normalized"]
    assert actor_a_projection["recomputed_contribution"] == actor_a_projection["scorecard_contribution"]

    assert actor_b_projection["quest_completed"] == 0.0
    assert actor_b_projection["scorecard_normalized"] == 0.0
    assert actor_b_projection["scorecard_contribution"] == 0.0
    assert actor_b_projection["recomputed_normalized"] == actor_b_projection["scorecard_normalized"]
    assert actor_b_projection["recomputed_contribution"] == actor_b_projection["scorecard_contribution"]


def test_second_tiny_scenario_is_deterministic_for_same_seed() -> None:
    first = run_benchmark_lifecycle(_runner_config())
    second = run_benchmark_lifecycle(_runner_config())
    first_projection = _recompute_quest_completion_contract(payload=first.to_dict(), actor_id="agent-a")
    second_projection = _recompute_quest_completion_contract(payload=second.to_dict(), actor_id="agent-a")

    assert first.to_canonical_json() == second.to_canonical_json()
    assert first.replay_parity_artifact.to_canonical_json() == second.replay_parity_artifact.to_canonical_json()
    assert first_projection == second_projection
