"""Unit tests for persistent quest/objective tracking mechanic.

task_id: persistent_world_quest_objective_tracking_v1

Tests cover:
- _find_quest_objectives helper
- process_quest_objective_tick fires for matching events
- quest_completed event has correct actor_id, quest_id, title, reward_message
- quest state set to "complete" in scenario_vars delta
- idempotency: already-completed quest not re-completed
- different actors get independent quest state
- multiple quests can complete in one tick
- trigger types: npc_defeated, npc_calmed, action_take, route_unlocked
- _build_quest_objectives_scenario_delta builder validation
- quest status persists through save/load
- _format_quest_status_messages shows correct completed/in-progress breakdown
"""
from __future__ import annotations

import json
from typing import Any


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_quest_world(
    *,
    quest_objectives: list[dict[str, Any]] | None = None,
    extra_scenario_vars: dict[str, Any] | None = None,
) -> Any:
    """Build a minimal world with quest objectives for unit tests."""
    from world.state.world_state import DeterministicWorldStateManager

    if quest_objectives is None:
        quest_objectives = [
            {
                "quest_id": "defeat-guardian",
                "title": "Defeat the Guardian",
                "trigger_event": "npc_defeated",
                "trigger_target_id": "guardian",
                "reward_message": "Guardian falls.",
            },
            {
                "quest_id": "calm-patrol",
                "title": "Calm the Patrol",
                "trigger_event": "npc_calmed",
                "trigger_target_id": "patrol",
                "reward_message": "Patrol calmed.",
            },
            {
                "quest_id": "take-relic",
                "title": "Take the Relic",
                "trigger_event": "action_take",
                "trigger_target_id": "relic",
                "reward_message": "Relic taken.",
            },
        ]

    scenario_vars: dict[str, Any] = {
        "quest_objectives_json": json.dumps(quest_objectives, sort_keys=True),
    }
    if extra_scenario_vars:
        scenario_vars.update(extra_scenario_vars)

    return DeterministicWorldStateManager.from_dict(
        {
            "rooms": {
                "start": {
                    "title": "Start",
                    "description": "Start room.",
                    "exits": {},
                    "entities": [],
                    "entities_present": [],
                }
            },
            "entities": {},
            "scenario_vars": scenario_vars,
        }
    )


def _make_event(
    event_type: str,
    actor_id: str | None,
    payload: dict[str, Any],
    step: int = 0,
) -> Any:
    from core import EventRecord, normalize_payload
    return EventRecord(
        step_index=step,
        event_type=event_type,
        actor_id=actor_id,
        payload=normalize_payload(payload),
    )


def _run_quest_tick(world: Any, events: list[Any], step: int = 0) -> Any:
    from world.state.basic_action_processor import process_quest_objective_tick
    return process_quest_objective_tick(world, events, step_index=step)


# ---------------------------------------------------------------------------
# _find_quest_objectives helper
# ---------------------------------------------------------------------------

class TestFindQuestObjectives:
    def test_returns_empty_when_no_json(self):
        from world.state.basic_action_processor import _find_quest_objectives  # type: ignore[attr-defined]
        assert _find_quest_objectives({}) == []

    def test_returns_empty_for_malformed_json(self):
        from world.state.basic_action_processor import _find_quest_objectives  # type: ignore[attr-defined]
        assert _find_quest_objectives({"quest_objectives_json": "bad"}) == []

    def test_returns_valid_entries(self):
        from world.state.basic_action_processor import _find_quest_objectives  # type: ignore[attr-defined]
        objs = [
            {
                "quest_id": "q1",
                "title": "Quest 1",
                "trigger_event": "npc_defeated",
                "trigger_target_id": "guardian",
            }
        ]
        svars = {"quest_objectives_json": json.dumps(objs)}
        result = _find_quest_objectives(svars)
        assert len(result) == 1
        assert result[0]["quest_id"] == "q1"

    def test_skips_unknown_trigger_event(self):
        from world.state.basic_action_processor import _find_quest_objectives  # type: ignore[attr-defined]
        objs = [
            {
                "quest_id": "q1",
                "title": "Quest 1",
                "trigger_event": "unknown_event",
                "trigger_target_id": "guard",
            }
        ]
        svars = {"quest_objectives_json": json.dumps(objs)}
        assert _find_quest_objectives(svars) == []

    def test_returns_empty_for_none_scenario_vars(self):
        from world.state.basic_action_processor import _find_quest_objectives  # type: ignore[attr-defined]
        assert _find_quest_objectives(None) == []


# ---------------------------------------------------------------------------
# process_quest_objective_tick — event matching
# ---------------------------------------------------------------------------

class TestQuestObjectiveTick:
    def test_returns_none_when_no_objectives(self):
        from world.state.world_state import DeterministicWorldStateManager
        world = DeterministicWorldStateManager.from_dict({
            "rooms": {"r": {"title": "R", "description": "d.", "exits": {}, "entities": [], "entities_present": []}},
            "entities": {},
            "scenario_vars": {},
        })
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is None

    def test_returns_none_when_no_events(self):
        world = _make_quest_world()
        result = _run_quest_tick(world, [])
        assert result is None

    def test_fires_for_npc_defeated(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        assert result.accepted

    def test_fires_for_npc_calmed(self):
        world = _make_quest_world()
        event = _make_event("npc_calmed", "hero", {"target_id": "patrol"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        assert result.accepted

    def test_fires_for_action_take(self):
        world = _make_quest_world()
        event = _make_event("action_take", "hero", {"item_id": "relic"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        assert result.accepted

    def test_does_not_fire_for_wrong_target(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", "hero", {"target_id": "wrong-npc"})
        result = _run_quest_tick(world, [event])
        assert result is None

    def test_does_not_fire_when_no_actor_id(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", None, {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is None

    def test_idempotent_already_completed(self):
        world = _make_quest_world(
            extra_scenario_vars={"quest.defeat-guardian.hero": "complete"}
        )
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is None

    def test_two_quests_complete_same_tick(self):
        world = _make_quest_world()
        events = [
            _make_event("npc_defeated", "hero", {"target_id": "guardian"}),
            _make_event("npc_calmed", "hero", {"target_id": "patrol"}),
        ]
        result = _run_quest_tick(world, events)
        assert result is not None
        quest_events = [e for e in result.events if e.event_type == "quest_completed"]
        assert len(quest_events) == 2
        quest_ids = {dict(e.payload)["quest_id"] for e in quest_events}
        assert "defeat-guardian" in quest_ids
        assert "calm-patrol" in quest_ids


# ---------------------------------------------------------------------------
# process_quest_objective_tick — world state changes
# ---------------------------------------------------------------------------

class TestQuestObjectiveWorldState:
    def _apply(self, world: Any, result: Any) -> None:
        world.apply_delta(dict(result.world_delta))

    def test_sets_quest_complete_in_scenario_vars(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        self._apply(world, result)
        svars = world.get_snapshot()["scenario_vars"]
        assert svars.get("quest.defeat-guardian.hero") == "complete"

    def test_different_actors_have_independent_state(self):
        world = _make_quest_world()
        event_a = _make_event("npc_defeated", "player-a", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event_a])
        assert result is not None
        self._apply(world, result)
        svars = world.get_snapshot()["scenario_vars"]
        assert svars.get("quest.defeat-guardian.player-a") == "complete"
        assert svars.get("quest.defeat-guardian.player-b") is None

    def test_event_actor_id_is_correct(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", "player-x", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        events = [e for e in result.events if e.event_type == "quest_completed"]
        assert events[0].actor_id == "player-x"

    def test_event_payload_contains_quest_metadata(self):
        world = _make_quest_world()
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        events = [e for e in result.events if e.event_type == "quest_completed"]
        payload = dict(events[0].payload)
        assert payload["quest_id"] == "defeat-guardian"
        assert payload["title"] == "Defeat the Guardian"
        assert "Guardian falls" in payload.get("reward_message", "")


# ---------------------------------------------------------------------------
# Quest state persistence through save/load
# ---------------------------------------------------------------------------

class TestQuestPersistence:
    def test_quest_state_persists_through_save_load(self, tmp_path):
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_quest_world()
        event = _make_event("npc_defeated", "hero", {"target_id": "guardian"})
        result = _run_quest_tick(world, [event])
        assert result is not None
        world.apply_delta(dict(result.world_delta))

        assert world.get_snapshot()["scenario_vars"].get("quest.defeat-guardian.hero") == "complete"

        save_world_slot("quest-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("quest-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        assert snap["scenario_vars"].get("quest.defeat-guardian.hero") == "complete"

    def test_incomplete_quest_not_complete_after_reload(self, tmp_path):
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_quest_world()
        save_world_slot("no-quest-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("no-quest-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        assert snap["scenario_vars"].get("quest.defeat-guardian.hero") is None

    def test_objectives_json_survives_reload(self, tmp_path):
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_quest_world()
        save_world_slot("obj-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("obj-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        assert "quest_objectives_json" in snap["scenario_vars"]


# ---------------------------------------------------------------------------
# _build_quest_objectives_scenario_delta builder
# ---------------------------------------------------------------------------

class TestBuildQuestObjectivesScenarioDelta:
    def test_empty_when_no_world_config(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        assert _build_quest_objectives_scenario_delta(None) == {}

    def test_empty_when_no_quest_objectives_key(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        assert _build_quest_objectives_scenario_delta({"rooms": {}}) == {}

    def test_empty_when_list_is_empty(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        assert _build_quest_objectives_scenario_delta({"quest_objectives": []}) == {}

    def test_skips_unknown_trigger_event(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        result = _build_quest_objectives_scenario_delta({
            "quest_objectives": [
                {"quest_id": "q1", "title": "T", "trigger_event": "bad_event", "trigger_target_id": "x"}
            ]
        })
        assert result == {}

    def test_builds_valid_scenario_delta(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        quests = [
            {
                "quest_id": "q1",
                "title": "Quest 1",
                "trigger_event": "npc_defeated",
                "trigger_target_id": "guardian",
                "reward_message": "Done.",
            }
        ]
        result = _build_quest_objectives_scenario_delta({"quest_objectives": quests})
        assert "quest_objectives_json" in result
        parsed = json.loads(result["quest_objectives_json"])
        assert len(parsed) == 1
        assert parsed[0]["quest_id"] == "q1"
        assert parsed[0]["reward_message"] == "Done."

    def test_output_is_deterministic(self):
        from evaluation.benchmark_runner.runner import _build_quest_objectives_scenario_delta  # type: ignore[attr-defined]
        quests = [
            {"quest_id": "b", "title": "B", "trigger_event": "npc_calmed", "trigger_target_id": "x"},
            {"quest_id": "a", "title": "A", "trigger_event": "action_take", "trigger_target_id": "y"},
        ]
        r1 = _build_quest_objectives_scenario_delta({"quest_objectives": quests})
        r2 = _build_quest_objectives_scenario_delta({"quest_objectives": quests})
        assert r1 == r2


# ---------------------------------------------------------------------------
# _format_quest_status_messages
# ---------------------------------------------------------------------------

class TestFormatQuestStatusMessages:
    def test_returns_empty_when_no_objectives(self):
        from evaluation.benchmark_runner.runner import _format_quest_status_messages  # type: ignore[attr-defined]
        assert _format_quest_status_messages({}, "hero") == ()

    def test_shows_completed_quest(self):
        from evaluation.benchmark_runner.runner import _format_quest_status_messages  # type: ignore[attr-defined]
        quests = [{"quest_id": "q1", "title": "Quest 1", "trigger_event": "npc_defeated", "trigger_target_id": "x"}]
        svars = {
            "quest_objectives_json": json.dumps(quests),
            "quest.q1.hero": "complete",
        }
        msgs = _format_quest_status_messages(svars, "hero")
        assert any("✓" in m and "Quest 1" in m for m in msgs)

    def test_shows_in_progress_quest(self):
        from evaluation.benchmark_runner.runner import _format_quest_status_messages  # type: ignore[attr-defined]
        quests = [{"quest_id": "q1", "title": "Quest 1", "trigger_event": "npc_defeated", "trigger_target_id": "x"}]
        svars = {"quest_objectives_json": json.dumps(quests)}
        msgs = _format_quest_status_messages(svars, "hero")
        assert any("○" in m and "Quest 1" in m for m in msgs)

    def test_actor_specific_state(self):
        from evaluation.benchmark_runner.runner import _format_quest_status_messages  # type: ignore[attr-defined]
        quests = [{"quest_id": "q1", "title": "Quest 1", "trigger_event": "npc_defeated", "trigger_target_id": "x"}]
        svars = {
            "quest_objectives_json": json.dumps(quests),
            "quest.q1.player-a": "complete",
        }
        msgs_a = _format_quest_status_messages(svars, "player-a")
        msgs_b = _format_quest_status_messages(svars, "player-b")
        assert any("✓" in m for m in msgs_a)
        assert any("○" in m for m in msgs_b)
