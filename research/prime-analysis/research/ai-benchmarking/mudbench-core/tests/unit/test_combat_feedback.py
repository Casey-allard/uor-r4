"""Tests for combat feedback messages in _format_tick_event_messages.

Verifies:
- npc_counter_attack with defended=True shows [Combat] prefix and "defended" note
- npc_counter_attack with defended=False preserves original [World] format
- action_attack shows combat outcome only to the attacker (actor-specific)
- action_defend shows stance confirmation only to the defending actor
- No regressions in existing npc_calmed, actor_defeated, or other handlers
"""

from __future__ import annotations

from evaluation.benchmark_runner.runner import _format_tick_event_messages
from core.event_logger import EventRecord, normalize_payload


def _make_event(event_type: str, actor_id: str | None = None, **payload) -> EventRecord:
    return EventRecord(
        step_index=1,
        event_type=event_type,
        actor_id=actor_id,
        payload=normalize_payload(payload),
    )


# ---------------------------------------------------------------------------
# npc_counter_attack — undefended (backward compat)
# ---------------------------------------------------------------------------


def test_counter_attack_undefended_uses_world_prefix() -> None:
    event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=5,
        resulting_health=95,
        defended=False,
    )
    msgs = _format_tick_event_messages([event])
    assert len(msgs) == 1
    assert msgs[0].startswith("[World]")


def test_counter_attack_undefended_format_preserved() -> None:
    """Exact original format must be unchanged when defended=False."""
    event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=5,
        resulting_health=95,
        defended=False,
    )
    msgs = _format_tick_event_messages([event])
    assert msgs[0] == "[World] guardian attacks player-a for 5 damage! (you have 95 HP)"


def test_counter_attack_undefended_no_defended_key() -> None:
    """When defended key is absent entirely, behaves as undefended."""
    event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=5,
        resulting_health=95,
    )
    msgs = _format_tick_event_messages([event])
    assert msgs[0].startswith("[World]")
    assert "defended" not in msgs[0]


# ---------------------------------------------------------------------------
# npc_counter_attack — defended
# ---------------------------------------------------------------------------


def test_counter_attack_defended_uses_combat_prefix() -> None:
    event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=2,
        resulting_health=98,
        defended=True,
    )
    msgs = _format_tick_event_messages([event])
    assert len(msgs) == 1
    assert msgs[0].startswith("[Combat]")


def test_counter_attack_defended_message_contains_defended_note() -> None:
    event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=2,
        resulting_health=98,
        defended=True,
    )
    msgs = _format_tick_event_messages([event])
    assert "defended" in msgs[0].lower()
    assert "reduced" in msgs[0].lower()


def test_counter_attack_defended_shows_damage_and_hp() -> None:
    event = _make_event(
        "npc_counter_attack",
        npc_id="patrol",
        target_id="hero",
        damage=2,
        resulting_health=48,
        defended=True,
    )
    msgs = _format_tick_event_messages([event])
    assert "2" in msgs[0]
    assert "48" in msgs[0]
    assert "patrol" in msgs[0]
    assert "hero" in msgs[0]


def test_counter_attack_defended_differs_from_undefended() -> None:
    """Defended and undefended messages must be textually distinguishable."""
    defended_event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=2,
        resulting_health=98,
        defended=True,
    )
    normal_event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=5,
        resulting_health=95,
        defended=False,
    )
    msgs_defended = _format_tick_event_messages([defended_event])
    msgs_normal = _format_tick_event_messages([normal_event])
    assert msgs_defended[0] != msgs_normal[0]
    assert "[Combat]" in msgs_defended[0]
    assert "[World]" in msgs_normal[0]


# ---------------------------------------------------------------------------
# action_attack feedback (actor-specific)
# ---------------------------------------------------------------------------


def test_action_attack_shown_to_attacker() -> None:
    event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=10,
        resulting_health=15,
    )
    msgs = _format_tick_event_messages([event], actor_id="player-a")
    assert len(msgs) == 1
    assert "[Combat]" in msgs[0]
    assert "guardian" in msgs[0]
    assert "10" in msgs[0]


def test_action_attack_shows_remaining_hp() -> None:
    event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=10,
        resulting_health=15,
    )
    msgs = _format_tick_event_messages([event], actor_id="player-a")
    assert "15" in msgs[0]
    assert "guardian" in msgs[0]


def test_action_attack_not_shown_to_other_actor() -> None:
    event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=10,
        resulting_health=15,
    )
    msgs = _format_tick_event_messages([event], actor_id="player-b")
    assert len(msgs) == 0


def test_action_attack_not_shown_without_actor_id_filter() -> None:
    """Without actor_id filter, action_attack is not broadcast to all observers."""
    event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=10,
        resulting_health=15,
    )
    msgs = _format_tick_event_messages([event])
    assert len(msgs) == 0


def test_action_attack_no_hp_note_when_npc_defeated() -> None:
    """When resulting_health=0 (NPC defeated), HP note is omitted."""
    event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=25,
        resulting_health=0,
    )
    msgs = _format_tick_event_messages([event], actor_id="player-a")
    assert len(msgs) == 1
    assert "guardian" in msgs[0]
    assert "25" in msgs[0]
    # No "0 HP remaining" — the npc_defeated event covers this
    assert "0 HP remaining" not in msgs[0]


# ---------------------------------------------------------------------------
# action_defend feedback (actor-specific)
# ---------------------------------------------------------------------------


def test_action_defend_shown_to_defender() -> None:
    event = _make_event("action_defend", actor_id="player-a")
    msgs = _format_tick_event_messages([event], actor_id="player-a")
    assert len(msgs) == 1
    assert "[Combat]" in msgs[0]
    assert "brace" in msgs[0].lower() or "defend" in msgs[0].lower()


def test_action_defend_mentions_reduced_damage() -> None:
    event = _make_event("action_defend", actor_id="player-a")
    msgs = _format_tick_event_messages([event], actor_id="player-a")
    assert "reduced" in msgs[0].lower()


def test_action_defend_not_shown_to_other_actor() -> None:
    event = _make_event("action_defend", actor_id="player-a")
    msgs = _format_tick_event_messages([event], actor_id="player-b")
    assert len(msgs) == 0


def test_action_defend_not_shown_without_actor_id_filter() -> None:
    event = _make_event("action_defend", actor_id="player-a")
    msgs = _format_tick_event_messages([event])
    assert len(msgs) == 0


# ---------------------------------------------------------------------------
# npc_calmed shadowing fix (regression)
# ---------------------------------------------------------------------------


def test_npc_calmed_does_not_corrupt_actor_id_filter() -> None:
    """npc_calmed handler must NOT shadow the actor_id parameter.

    If the bug (local `actor_id = event.actor_id`) is still present, the
    quest_completed filter and action_attack/defend filter would be broken for
    any tick that contains a preceding npc_calmed event.
    """
    calm_event = _make_event(
        "npc_calmed",
        actor_id="player-a",
        target_id="guardian",
        item_id="ward-talisman",
    )
    attack_event = _make_event(
        "action_attack",
        actor_id="player-b",
        target_id="patrol",
        damage=8,
        resulting_health=12,
    )
    # player-b should see their own attack but not player-a's
    msgs = _format_tick_event_messages([calm_event, attack_event], actor_id="player-b")
    attack_msgs = [m for m in msgs if "[Combat]" in m and "patrol" in m]
    assert len(attack_msgs) == 1


def test_npc_calmed_after_quest_filter_still_works() -> None:
    """quest_completed filter should still apply after npc_calmed in same tick."""
    calm_event = _make_event(
        "npc_calmed",
        actor_id="player-a",
        target_id="npc-x",
        item_id="talisman",
    )
    quest_event = _make_event("quest_completed", actor_id="player-b", title="Side Quest", reward_message="")
    # player-a should NOT see player-b's quest completion
    msgs = _format_tick_event_messages([calm_event, quest_event], actor_id="player-a")
    quest_msgs = [m for m in msgs if "[Quest]" in m]
    assert len(quest_msgs) == 0


# ---------------------------------------------------------------------------
# Ordering determinism
# ---------------------------------------------------------------------------


def test_multiple_combat_events_ordering_deterministic() -> None:
    """Messages appear in event-list order, not sorted arbitrarily."""
    defend_event = _make_event("action_defend", actor_id="player-a")
    counter_event = _make_event(
        "npc_counter_attack",
        npc_id="guardian",
        target_id="player-a",
        damage=2,
        resulting_health=98,
        defended=True,
    )
    attack_event = _make_event(
        "action_attack",
        actor_id="player-a",
        target_id="guardian",
        damage=10,
        resulting_health=5,
    )
    msgs = _format_tick_event_messages(
        [defend_event, counter_event, attack_event], actor_id="player-a"
    )
    assert len(msgs) == 3
    # First: defend stance; second: counter-attack outcome; third: own attack
    assert "brace" in msgs[0].lower() or "defend" in msgs[0].lower()
    assert "defended" in msgs[1].lower()
    assert "guardian" in msgs[2] and "10" in msgs[2]
