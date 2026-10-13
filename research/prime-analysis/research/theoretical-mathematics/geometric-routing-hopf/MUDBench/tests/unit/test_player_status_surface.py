"""Unit tests for player-facing status surface improvements.

task_id: shared_world_player_status_and_objective_surface_v1

Tests cover:
- render_human_observation entity type separation (NPCs / Items / Other)
- render_human_observation message list format (one-per-line, not comma-joined)
- _format_quest_progress_compact helper
- render_human_observation backward compatibility with no entities/no messages
"""
from __future__ import annotations

import json
from typing import Any

from agents.protocol.observation import ObservedEntity, Observation


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _obs(
    *,
    entities: tuple[ObservedEntity, ...] = (),
    messages: tuple[str, ...] = (),
    run_id: str = "test-run",
    step: int = 0,
    location: str = "room",
    description: str = "A room.",
    exits: tuple[str, ...] = (),
    inventory: tuple[str, ...] = (),
    health: int = 100,
    action_space: tuple[str, ...] = ("wait", "look"),
    remaining_steps: int = 5,
) -> Observation:
    return Observation(
        run_id=run_id,
        step=step,
        location=location,
        description=description,
        exits=exits,
        entities=entities,
        inventory=inventory,
        health=health,
        messages=messages,
        action_space=action_space,
        remaining_steps=remaining_steps,
    )


# ---------------------------------------------------------------------------
# render_human_observation — entity type separation
# ---------------------------------------------------------------------------

class TestRenderEntityTypeSeparation:
    def test_npc_shows_as_npcs_section(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=(ObservedEntity(type="npc", name="guardian"),))
        rendered = render_human_observation(obs)
        assert "NPCs: guardian" in rendered
        assert "Entities:" not in rendered

    def test_item_shows_as_items_section(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=(ObservedEntity(type="item", name="ward-sigil"),))
        rendered = render_human_observation(obs)
        assert "Items: ward-sigil" in rendered
        assert "Entities:" not in rendered

    def test_consumable_shows_in_items_section(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=(ObservedEntity(type="consumable", name="potion"),))
        rendered = render_human_observation(obs)
        assert "Items: potion" in rendered

    def test_other_type_shows_as_other_section(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=(ObservedEntity(type="player", name="actor-b"),))
        rendered = render_human_observation(obs)
        assert "Other: actor-b" in rendered

    def test_mixed_entities_show_in_correct_sections(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=(
            ObservedEntity(type="npc", name="guardian"),
            ObservedEntity(type="item", name="ward-sigil"),
        ))
        rendered = render_human_observation(obs)
        assert "NPCs: guardian" in rendered
        assert "Items: ward-sigil" in rendered

    def test_no_entities_shows_entities_none(self):
        from human_console_client import render_human_observation
        obs = _obs(entities=())
        rendered = render_human_observation(obs)
        assert "Entities: none" in rendered

    def test_npc_and_item_sections_appear_before_inventory(self):
        from human_console_client import render_human_observation
        obs = _obs(
            entities=(
                ObservedEntity(type="npc", name="guardian"),
                ObservedEntity(type="item", name="relic"),
            ),
            inventory=("helm",),
        )
        rendered = render_human_observation(obs)
        npc_pos = rendered.index("NPCs:")
        item_pos = rendered.index("Items:")
        inv_pos = rendered.index("Inventory:")
        assert npc_pos < inv_pos
        assert item_pos < inv_pos


# ---------------------------------------------------------------------------
# render_human_observation — message list format
# ---------------------------------------------------------------------------

class TestRenderMessageList:
    def test_single_message_on_own_line(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=("The sentinel blocks the door.",))
        rendered = render_human_observation(obs)
        # "Messages:\n  The sentinel..." — not comma-joined on same line
        assert "Messages:\n  The sentinel blocks the door." in rendered

    def test_multiple_messages_each_on_own_line(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=("Message A.", "Message B."))
        rendered = render_human_observation(obs)
        assert "  Message A." in rendered
        assert "  Message B." in rendered
        # NOT comma-joined
        assert "Message A., Message B." not in rendered

    def test_no_messages_shows_none_placeholder(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=())
        rendered = render_human_observation(obs)
        assert "Messages:\n  (none)" in rendered

    def test_messages_header_present(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=("Hello.",))
        rendered = render_human_observation(obs)
        assert "Messages:" in rendered

    def test_message_section_appears_before_available_actions(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=("A message.",))
        rendered = render_human_observation(obs)
        msg_pos = rendered.index("Messages:")
        actions_pos = rendered.index("Available Actions:")
        assert msg_pos < actions_pos

    def test_deterministic_output(self):
        from human_console_client import render_human_observation
        obs = _obs(messages=("Msg 1.", "Msg 2."), entities=(ObservedEntity(type="npc", name="boss"),))
        assert render_human_observation(obs) == render_human_observation(obs)


# ---------------------------------------------------------------------------
# _format_quest_progress_compact
# ---------------------------------------------------------------------------

class TestFormatQuestProgressCompact:
    def _make_svars(
        self,
        quests: list[dict],
        completed_ids: list[str] | None = None,
        actor_id: str = "hero",
    ) -> dict:
        svars: dict[str, Any] = {"quest_objectives_json": json.dumps(quests)}
        for qid in (completed_ids or []):
            svars[f"quest.{qid}.{actor_id}"] = "complete"
        return svars

    def test_returns_none_when_no_objectives(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        assert _format_quest_progress_compact({}, "hero") is None

    def test_returns_none_for_malformed_json(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        assert _format_quest_progress_compact({"quest_objectives_json": "bad"}, "hero") is None

    def test_returns_none_for_empty_objectives(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        svars = {"quest_objectives_json": json.dumps([])}
        assert _format_quest_progress_compact(svars, "hero") is None

    def test_returns_string_when_objectives_exist(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [{"quest_id": "q1", "title": "Quest One", "trigger_event": "npc_defeated", "trigger_target_id": "g"}]
        svars = self._make_svars(quests)
        result = _format_quest_progress_compact(svars, "hero")
        assert result is not None
        assert "[Objectives]" in result

    def test_in_progress_quest_shows_circle_marker(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [{"quest_id": "q1", "title": "Find Key", "trigger_event": "action_take", "trigger_target_id": "key"}]
        svars = self._make_svars(quests)
        result = _format_quest_progress_compact(svars, "hero")
        assert result is not None
        assert "○ Find Key" in result

    def test_completed_quest_shows_check_marker(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [{"quest_id": "q1", "title": "Find Key", "trigger_event": "action_take", "trigger_target_id": "key"}]
        svars = self._make_svars(quests, completed_ids=["q1"])
        result = _format_quest_progress_compact(svars, "hero")
        assert result is not None
        assert "✓ Find Key" in result

    def test_mixed_completed_and_incomplete(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [
            {"quest_id": "q1", "title": "Quest A", "trigger_event": "npc_defeated", "trigger_target_id": "g"},
            {"quest_id": "q2", "title": "Quest B", "trigger_event": "npc_calmed", "trigger_target_id": "p"},
        ]
        svars = self._make_svars(quests, completed_ids=["q1"])
        result = _format_quest_progress_compact(svars, "hero")
        assert result is not None
        assert "✓ Quest A" in result
        assert "○ Quest B" in result

    def test_actor_specific_completion(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [{"quest_id": "q1", "title": "Quest A", "trigger_event": "npc_defeated", "trigger_target_id": "g"}]
        svars = {"quest_objectives_json": json.dumps(quests), "quest.q1.player-a": "complete"}
        result_a = _format_quest_progress_compact(svars, "player-a")
        result_b = _format_quest_progress_compact(svars, "player-b")
        assert result_a is not None and "✓ Quest A" in result_a
        assert result_b is not None and "○ Quest A" in result_b

    def test_output_is_single_line(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [
            {"quest_id": "q1", "title": "A", "trigger_event": "npc_defeated", "trigger_target_id": "x"},
            {"quest_id": "q2", "title": "B", "trigger_event": "npc_calmed", "trigger_target_id": "y"},
        ]
        svars = self._make_svars(quests)
        result = _format_quest_progress_compact(svars, "hero")
        assert result is not None
        assert "\n" not in result

    def test_output_is_deterministic(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        quests = [
            {"quest_id": "q1", "title": "A", "trigger_event": "npc_defeated", "trigger_target_id": "x"},
            {"quest_id": "q2", "title": "B", "trigger_event": "npc_calmed", "trigger_target_id": "y"},
        ]
        svars = self._make_svars(quests)
        assert _format_quest_progress_compact(svars, "hero") == _format_quest_progress_compact(svars, "hero")

    def test_returns_none_when_scenario_vars_is_none(self):
        from evaluation.benchmark_runner.runner import _format_quest_progress_compact  # type: ignore
        assert _format_quest_progress_compact(None, "hero") is None


# ---------------------------------------------------------------------------
# render_human_observation — backward compatibility
# ---------------------------------------------------------------------------

class TestRenderBackwardCompatibility:
    def test_core_fields_always_present(self):
        from human_console_client import render_human_observation
        obs = _obs()
        rendered = render_human_observation(obs)
        assert "Run: test-run" in rendered
        assert "Step: 1" in rendered
        assert "Location: room" in rendered
        assert "Health: 100" in rendered
        assert "Description: A room." in rendered
        assert "Remaining Steps: 5" in rendered
        assert "Exits: none" in rendered
        assert "Inventory: empty" in rendered
        assert "Available Actions:" in rendered
        assert "Allowed Targets:" in rendered

    def test_action_space_still_numbered(self):
        from human_console_client import render_human_observation
        obs = _obs(action_space=("wait", "look", "move north"))
        rendered = render_human_observation(obs)
        assert "  1. wait" in rendered
        assert "  2. look" in rendered
        assert "  3. move north" in rendered

    def test_prompt_hint_still_present(self):
        from human_console_client import render_human_observation
        obs = _obs()
        rendered = render_human_observation(obs)
        assert "Enter an exact action" in rendered
