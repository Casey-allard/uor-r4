"""Unit tests for persistent NPC dialogue/lore mechanic.

task_id: persistent_npc_dialogue_and_lore_v1

Tests cover:
- _find_npc_dialogue helper
- talk action accepted for non-hostile NPC in same room
- talk action rejected when NPC is hostile
- talk action rejected when NPC is not in same room
- talk action rejected when NPC is defeated
- dialogue lines cycle via talk_count.<npc_id>
- dialogue falls back gracefully when no lines configured
- talk_count persists through save/load
- npc_talked event payload
- _build_npc_dialogue_scenario_delta builder validation
- action_space includes talk for non-hostile NPC
- action_space excludes talk for hostile NPC
"""
from __future__ import annotations

import json
from typing import Any


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_dialogue_world(
    *,
    guardian_tags: list[str] | None = None,
    guardian_health: int = 20,
    guardian_in_room: bool = True,
    patrol_tags: list[str] | None = None,
    patrol_health: int = 15,
    patrol_in_room: bool = True,
    dialogue_entries: list[dict[str, Any]] | None = None,
    talk_count_guardian: int | None = None,
) -> Any:
    """Build a minimal world for dialogue unit tests."""
    from world.state.world_state import DeterministicWorldStateManager

    rooms: dict[str, Any] = {
        "vault-entrance": {
            "title": "Vault Entrance",
            "description": "Guarded entrance.",
            "exits": {"west": "camp"},
            "entities": [],
        },
        "camp": {
            "title": "Camp",
            "description": "Camp.",
            "exits": {"east": "vault-entrance"},
            "entities": [],
        },
    }

    entities: dict[str, Any] = {
        "player-a": {
            "entity_id": "player-a",
            "entity_type": "agent",
            "location": "vault-entrance",
            "health": 100,
            "inventory": [],
            "tags": [],
        },
        "guardian": {
            "entity_id": "guardian",
            "entity_type": "npc",
            "health": guardian_health,
            "location": "vault-entrance",
            "tags": list(guardian_tags or []),
            "inventory": [],
        },
        "patrol": {
            "entity_id": "patrol",
            "entity_type": "npc",
            "health": patrol_health,
            "location": "vault-entrance",
            "tags": list(patrol_tags or ["hostile"]),
            "inventory": [],
        },
    }

    ve_present = ["player-a"]
    if guardian_in_room and guardian_health > 0:
        ve_present.append("guardian")
    if patrol_in_room and patrol_health > 0:
        ve_present.append("patrol")
    rooms["vault-entrance"]["entities_present"] = sorted(ve_present)

    if dialogue_entries is None:
        dialogue_entries = [
            {
                "npc_id": "guardian",
                "lines": [
                    "The vault holds something precious.",
                    "Many have sought the relic.",
                    "Prove your strength first.",
                ],
            },
            {
                "npc_id": "patrol",
                "lines": [
                    "The ward-sigil humbles me.",
                ],
            },
        ]

    scenario_vars: dict[str, Any] = {
        "npc_dialogue_json": json.dumps(dialogue_entries, sort_keys=True),
    }
    if talk_count_guardian is not None:
        scenario_vars["talk_count.guardian"] = talk_count_guardian

    return DeterministicWorldStateManager.from_dict(
        {"rooms": rooms, "entities": entities, "scenario_vars": scenario_vars}
    )


def _make_talk_action(actor_id: str, target_id: str) -> Any:
    from core.action_processor import ActionRequest, normalize_arguments
    return ActionRequest(
        actor_id=actor_id,
        action_type="talk",
        arguments=normalize_arguments({"target_id": target_id}),
    )


def _process_talk(world: Any, actor_id: str, target_id: str) -> Any:
    from world.state.basic_action_processor import BasicDeterministicActionProcessor
    processor = BasicDeterministicActionProcessor()
    action = _make_talk_action(actor_id, target_id)
    results = processor.process_actions([action], world, step_index=0)
    return results[0]


# ---------------------------------------------------------------------------
# _find_npc_dialogue helper
# ---------------------------------------------------------------------------

class TestFindNpcDialogue:
    def test_returns_empty_when_no_dialogue_json(self):
        from world.state.basic_action_processor import _find_npc_dialogue  # type: ignore[attr-defined]
        assert _find_npc_dialogue({}, "guardian") == []

    def test_returns_empty_when_npc_not_in_list(self):
        from world.state.basic_action_processor import _find_npc_dialogue  # type: ignore[attr-defined]
        entries = [{"npc_id": "guardian", "lines": ["hello"]}]
        svars = {"npc_dialogue_json": json.dumps(entries)}
        assert _find_npc_dialogue(svars, "patrol") == []

    def test_returns_lines_for_matching_npc(self):
        from world.state.basic_action_processor import _find_npc_dialogue  # type: ignore[attr-defined]
        entries = [{"npc_id": "guardian", "lines": ["line1", "line2"]}]
        svars = {"npc_dialogue_json": json.dumps(entries)}
        lines = _find_npc_dialogue(svars, "guardian")
        assert lines == ["line1", "line2"]

    def test_returns_empty_for_malformed_json(self):
        from world.state.basic_action_processor import _find_npc_dialogue  # type: ignore[attr-defined]
        assert _find_npc_dialogue({"npc_dialogue_json": "bad json"}, "guardian") == []

    def test_returns_empty_for_none_scenario_vars(self):
        from world.state.basic_action_processor import _find_npc_dialogue  # type: ignore[attr-defined]
        assert _find_npc_dialogue(None, "guardian") == []


# ---------------------------------------------------------------------------
# talk action — acceptance / rejection
# ---------------------------------------------------------------------------

class TestTalkActionAcceptance:
    def test_talk_accepted_for_non_hostile_npc_in_room(self):
        world = _make_dialogue_world(guardian_tags=[])
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted

    def test_talk_rejected_when_npc_is_hostile(self):
        world = _make_dialogue_world(patrol_tags=["hostile"])
        result = _process_talk(world, "player-a", "patrol")
        assert not result.accepted
        events = [e for e in result.events if e.event_type == "action_rejected"]
        assert any("hostile" in str(dict(e.payload)) for e in events)

    def test_talk_rejected_when_npc_not_in_room(self):
        world = _make_dialogue_world(guardian_in_room=False)
        result = _process_talk(world, "player-a", "guardian")
        assert not result.accepted

    def test_talk_rejected_when_npc_health_zero(self):
        world = _make_dialogue_world(guardian_health=0, guardian_in_room=False)
        result = _process_talk(world, "player-a", "guardian")
        assert not result.accepted

    def test_talk_rejected_when_actor_not_found(self):
        from world.state.basic_action_processor import BasicDeterministicActionProcessor
        from core.action_processor import ActionRequest, normalize_arguments
        world = _make_dialogue_world()
        processor = BasicDeterministicActionProcessor()
        action = ActionRequest(
            actor_id="nobody",
            action_type="talk",
            arguments=normalize_arguments({"target_id": "guardian"}),
        )
        results = processor.process_actions([action], world, step_index=0)
        assert not results[0].accepted


# ---------------------------------------------------------------------------
# Dialogue line selection and cycling
# ---------------------------------------------------------------------------

class TestDialogueCycling:
    def test_returns_first_line_on_first_talk(self):
        world = _make_dialogue_world(guardian_tags=[], talk_count_guardian=None)
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        assert len(events) == 1
        payload = dict(events[0].payload)
        assert payload["line"] == "The vault holds something precious."

    def test_returns_second_line_on_second_talk(self):
        world = _make_dialogue_world(guardian_tags=[], talk_count_guardian=1)
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        payload = dict(events[0].payload)
        assert payload["line"] == "Many have sought the relic."

    def test_cycles_back_to_first_after_all_lines(self):
        world = _make_dialogue_world(guardian_tags=[], talk_count_guardian=3)  # 3 lines, idx=0 again
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        payload = dict(events[0].payload)
        assert payload["line"] == "The vault holds something precious."

    def test_talk_count_incremented_in_world_delta(self):
        world = _make_dialogue_world(guardian_tags=[], talk_count_guardian=None)
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        world.apply_delta(dict(result.world_delta))
        snap = world.get_snapshot()
        assert snap["scenario_vars"].get("talk_count.guardian") == 1

    def test_fallback_message_when_no_dialogue_configured(self):
        world = _make_dialogue_world(guardian_tags=[], dialogue_entries=[])
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        payload = dict(events[0].payload)
        assert "nothing to say" in payload["line"]


# ---------------------------------------------------------------------------
# npc_talked event payload
# ---------------------------------------------------------------------------

class TestNpcTalkedEventPayload:
    def test_event_payload_contains_target_and_room(self):
        world = _make_dialogue_world(guardian_tags=[])
        result = _process_talk(world, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        assert events
        payload = dict(events[0].payload)
        assert payload["target_id"] == "guardian"
        assert payload["room_id"] == "vault-entrance"
        assert "line" in payload

    def test_event_actor_id_is_correct(self):
        world = _make_dialogue_world(guardian_tags=[])
        result = _process_talk(world, "player-a", "guardian")
        events = [e for e in result.events if e.event_type == "npc_talked"]
        assert events[0].actor_id == "player-a"


# ---------------------------------------------------------------------------
# Persistence: talk_count survives save/load
# ---------------------------------------------------------------------------

class TestDialoguePersistence:
    def test_talk_count_persists_through_save_load(self, tmp_path):
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_dialogue_world(guardian_tags=[], talk_count_guardian=None)
        result = _process_talk(world, "player-a", "guardian")
        world.apply_delta(dict(result.world_delta))

        assert world.get_snapshot()["scenario_vars"].get("talk_count.guardian") == 1

        save_world_slot("talk-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("talk-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        assert snap["scenario_vars"].get("talk_count.guardian") == 1

    def test_dialogue_available_after_save_load(self, tmp_path):
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_dialogue_world(guardian_tags=[])
        save_world_slot("dlg-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("dlg-slot", str(tmp_path))
        assert loaded.accepted

        result = _process_talk(loaded.world_state, "player-a", "guardian")
        assert result.accepted
        events = [e for e in result.events if e.event_type == "npc_talked"]
        assert events


# ---------------------------------------------------------------------------
# _build_npc_dialogue_scenario_delta builder
# ---------------------------------------------------------------------------

class TestBuildNpcDialogueScenarioDelta:
    def test_empty_when_no_world_config(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_dialogue_scenario_delta(None) == {}

    def test_empty_when_no_npc_dialogue_key(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_dialogue_scenario_delta({"rooms": {}}) == {}

    def test_empty_when_dialogue_list_is_empty(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_dialogue_scenario_delta({"npc_dialogue": []}) == {}

    def test_skips_entry_with_empty_lines(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        result = _build_npc_dialogue_scenario_delta({
            "npc_dialogue": [{"npc_id": "guardian", "lines": []}]
        })
        assert result == {}

    def test_builds_valid_scenario_delta(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        entries = [{"npc_id": "guardian", "lines": ["Hello.", "Farewell."]}]
        result = _build_npc_dialogue_scenario_delta({"npc_dialogue": entries})
        assert "npc_dialogue_json" in result
        parsed = json.loads(result["npc_dialogue_json"])
        assert len(parsed) == 1
        assert parsed[0]["npc_id"] == "guardian"
        assert parsed[0]["lines"] == ["Hello.", "Farewell."]

    def test_output_is_deterministic(self):
        from evaluation.benchmark_runner.runner import _build_npc_dialogue_scenario_delta  # type: ignore[attr-defined]
        entries = [
            {"npc_id": "b", "lines": ["b line"]},
            {"npc_id": "a", "lines": ["a line"]},
        ]
        r1 = _build_npc_dialogue_scenario_delta({"npc_dialogue": entries})
        r2 = _build_npc_dialogue_scenario_delta({"npc_dialogue": entries})
        assert r1 == r2


# ---------------------------------------------------------------------------
# Action space: talk appears for non-hostile, not for hostile
# ---------------------------------------------------------------------------

class TestActionSpaceTalk:
    def test_talk_in_action_space_for_non_hostile_npc(self):
        from agents.gateway.observation_builder import build_observation_for_actor

        world = _make_dialogue_world(guardian_tags=[], patrol_tags=["hostile"])
        snap = world.get_snapshot()
        obs = build_observation_for_actor(
            snap, actor_id="player-a", run_id="r1", step=0, max_steps=10
        )
        assert "talk guardian" in obs.action_space

    def test_talk_not_in_action_space_for_hostile_npc(self):
        from agents.gateway.observation_builder import build_observation_for_actor

        world = _make_dialogue_world(guardian_tags=[], patrol_tags=["hostile"])
        snap = world.get_snapshot()
        obs = build_observation_for_actor(
            snap, actor_id="player-a", run_id="r1", step=0, max_steps=10
        )
        assert "talk patrol" not in obs.action_space

    def test_talk_appears_after_calm_removes_hostile_tag(self):
        """Once patrol's hostile tag is removed, talk patrol should appear."""
        from agents.gateway.observation_builder import build_observation_for_actor

        world = _make_dialogue_world(patrol_tags=["hostile"])
        # Remove hostile tag from patrol
        world.apply_delta({"entities": {"patrol": {
            "entity_id": "patrol", "entity_type": "npc",
            "health": 15, "location": "vault-entrance", "tags": [], "inventory": [],
        }}})
        snap = world.get_snapshot()
        obs = build_observation_for_actor(
            snap, actor_id="player-a", run_id="r1", step=0, max_steps=10
        )
        assert "talk patrol" in obs.action_space
