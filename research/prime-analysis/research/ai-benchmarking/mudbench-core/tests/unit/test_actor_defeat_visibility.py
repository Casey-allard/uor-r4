"""Tests for shared-world actor defeat/respawn visibility to other actors.

Covers:
- Live-tick: other actors see actor_defeated / actor_respawned event messages
- Persistent: _format_co_actor_defeat_status_messages returns correct messages
- Reconnect history: _format_world_event_log_messages renders actor_defeated/respawned
- Self-vs-co-actor separation: self messages stay private, co-actor messages stay public
- No regressions to existing defeat/respawn behavior
"""

from __future__ import annotations

import json

from evaluation.benchmark_runner.runner import (
    _format_actor_defeat_status_message,
    _format_co_actor_defeat_status_messages,
    _format_tick_event_messages,
    _format_world_event_log_messages,
)
from core.event_logger import EventRecord, normalize_payload


# ---------------------------------------------------------------------------
# _format_tick_event_messages — live-tick visibility
# ---------------------------------------------------------------------------


def _defeat_event(actor_id: str, respawn_at: int) -> EventRecord:
    return EventRecord(
        step_index=5,
        event_type="actor_defeated",
        actor_id=actor_id,
        payload=normalize_payload({"actor_id": actor_id, "room_id": "camp", "respawn_at_tick": respawn_at}),
    )


def _respawn_event(actor_id: str, room_id: str, health: int) -> EventRecord:
    return EventRecord(
        step_index=8,
        event_type="actor_respawned",
        actor_id=actor_id,
        payload=normalize_payload({"actor_id": actor_id, "room_id": room_id, "health": health}),
    )


def test_defeat_event_visible_to_defeated_actor():
    """The defeated actor sees their own defeat event."""
    msgs = _format_tick_event_messages((_defeat_event("alice", 8),), actor_id="alice")
    assert any("alice has been defeated" in m for m in msgs)
    assert any("tick 8" in m for m in msgs)


def test_defeat_event_visible_to_other_actor():
    """A co-actor sees another actor's defeat event."""
    msgs = _format_tick_event_messages((_defeat_event("alice", 8),), actor_id="bob")
    assert any("alice has been defeated" in m for m in msgs)
    assert any("tick 8" in m for m in msgs)


def test_respawn_event_visible_to_respawned_actor():
    """The respawned actor sees their own respawn event."""
    msgs = _format_tick_event_messages((_respawn_event("alice", "vault", 50),), actor_id="alice")
    assert any("alice has returned" in m for m in msgs)
    assert any("vault" in m for m in msgs)
    assert any("50 HP" in m for m in msgs)


def test_respawn_event_visible_to_other_actor():
    """A co-actor sees another actor's respawn event."""
    msgs = _format_tick_event_messages((_respawn_event("alice", "vault", 50),), actor_id="bob")
    assert any("alice has returned" in m for m in msgs)


def test_no_defeat_events_returns_no_extra_messages():
    """No spurious defeat messages when no defeat events are present."""
    from core.event_logger import EventRecord, normalize_payload

    events = (
        EventRecord(
            step_index=1,
            event_type="npc_alert",
            actor_id="alice",
            payload=normalize_payload({"target_id": "guardian", "remaining_health": 10}),
        ),
    )
    msgs = _format_tick_event_messages(events, actor_id="bob")
    assert all("defeated" not in m.lower() for m in msgs if "[World]" in m and "guardian" not in m)


# ---------------------------------------------------------------------------
# _format_co_actor_defeat_status_messages — persistent co-actor visibility
# ---------------------------------------------------------------------------


def test_co_actor_defeat_status_visible_when_co_actor_defeated():
    """Active actor sees defeat status for a defeated co-actor."""
    scenario_vars = {
        "actor_defeated.alice": True,
        "actor_respawn_at.alice": 10,
    }
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert len(msgs) == 1
    assert "alice" in msgs[0]
    assert "tick 10" in msgs[0]
    assert "currently defeated" in msgs[0]


def test_co_actor_defeat_status_empty_when_no_co_actors_defeated():
    """No co-actor defeat messages when all actors are alive."""
    scenario_vars = {
        "actor_defeated.alice": False,
    }
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert msgs == ()


def test_co_actor_defeat_status_excludes_self():
    """Self-defeat is not surfaced via co-actor messages."""
    scenario_vars = {
        "actor_defeated.bob": True,
        "actor_respawn_at.bob": 10,
    }
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert msgs == ()


def test_co_actor_defeat_status_no_respawn_at():
    """Shows 'awaiting respawn' when respawn tick is not set."""
    scenario_vars = {
        "actor_defeated.alice": True,
    }
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert len(msgs) == 1
    assert "awaiting respawn" in msgs[0]


def test_co_actor_defeat_status_multiple_defeated():
    """Multiple defeated co-actors each get their own status line."""
    scenario_vars = {
        "actor_defeated.alice": True,
        "actor_respawn_at.alice": 10,
        "actor_defeated.carol": True,
        "actor_respawn_at.carol": 15,
    }
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert len(msgs) == 2
    ids = {m.split()[1] for m in msgs}
    assert "alice" in ids
    assert "carol" in ids


def test_co_actor_defeat_status_empty_when_no_defeat_keys():
    """Returns empty when scenario_vars has no defeat keys at all."""
    scenario_vars = {"some_other_key": "value"}
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert msgs == ()


def test_co_actor_defeat_status_empty_on_non_mapping():
    """Returns empty when scenario_vars is not a mapping."""
    assert _format_co_actor_defeat_status_messages(None, actor_id="bob") == ()  # type: ignore[arg-type]
    assert _format_co_actor_defeat_status_messages("not-a-dict", actor_id="bob") == ()  # type: ignore[arg-type]


def test_self_defeat_message_unchanged_by_co_actor_function():
    """_format_actor_defeat_status_message still works correctly in isolation."""
    scenario_vars = {
        "actor_defeated.bob": True,
        "actor_respawn_at.bob": 12,
    }
    msg = _format_actor_defeat_status_message(scenario_vars, actor_id="bob")
    assert msg is not None
    assert "You have been defeated" in msg
    assert "tick 12" in msg

    # Co-actor function must NOT generate anything for bob from bob's perspective
    co_msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="bob")
    assert co_msgs == ()


# ---------------------------------------------------------------------------
# _format_world_event_log_messages — reconnect history
# ---------------------------------------------------------------------------


def test_history_includes_actor_defeated():
    """Reconnecting actor sees prior actor defeat events in history."""
    log = [
        {"event_type": "actor_defeated", "step": 5, "actor_id": "alice", "respawn_at_tick": 8},
    ]
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps(log)})
    assert len(msgs) > 1
    assert any("alice" in m and "defeated" in m for m in msgs)
    assert any("tick 8" in m for m in msgs)


def test_history_includes_actor_respawned():
    """Reconnecting actor sees prior actor respawn events in history."""
    log = [
        {"event_type": "actor_respawned", "step": 8, "actor_id": "alice", "room_id": "camp", "health": 50},
    ]
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps(log)})
    assert len(msgs) > 1
    assert any("alice" in m and "respawned" in m for m in msgs)
    assert any("camp" in m for m in msgs)
    assert any("50 HP" in m for m in msgs)


def test_history_includes_actor_defeated_no_respawn_tick():
    """History entry works even when respawn_at_tick is absent."""
    log = [
        {"event_type": "actor_defeated", "step": 5, "actor_id": "carol"},
    ]
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps(log)})
    assert any("carol" in m and "defeated" in m for m in msgs)


def test_history_mixed_event_types():
    """History renders all loggable event types including new actor defeat/respawn."""
    log = [
        {"event_type": "npc_defeated", "step": 3, "target_id": "guardian"},
        {"event_type": "actor_defeated", "step": 5, "actor_id": "alice", "respawn_at_tick": 8},
        {"event_type": "actor_respawned", "step": 8, "actor_id": "alice", "room_id": "camp", "health": 50},
    ]
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps(log)})
    assert any("guardian" in m for m in msgs)
    assert any("alice" in m and "defeated" in m for m in msgs)
    assert any("alice" in m and "respawned" in m for m in msgs)


def test_history_empty_log_returns_empty():
    """Empty log returns no messages."""
    msgs = _format_world_event_log_messages({"world_event_log_json": json.dumps([])})
    assert msgs == ()


def test_history_no_log_key_returns_empty():
    """Missing log key returns empty."""
    msgs = _format_world_event_log_messages({})
    assert msgs == ()


# ---------------------------------------------------------------------------
# Integration: build a shared session and verify visibility end-to-end
# ---------------------------------------------------------------------------


_MINIMAL_SCENARIO = {
    "scenario_id": "test-defeat-vis",
    "title": "Test Defeat Visibility",
    "description": "Defeat visibility test scenario.",
    "version": "1",
    "seed": 42,
    "max_steps": 20,
    "start_room_id": "camp",
    "objectives": [{"objective_id": "test-obj", "objective_type": "collect_item", "required_count": 1, "target_id": "relic"}],
    "scenario_vars": {
        "mode": "shared_shard",
        "world_config_json": '{"rooms":{"camp":{"title":"Camp","description":"A test camp.","exits":{},"entities":[]},"vault":{"title":"Vault","description":"A test vault.","exits":{},"entities":[]}},"npcs":[],"items":[]}',
    },
}


def _build_minimal_shared_session():
    """Build a minimal shared-shard session with two actors."""
    from evaluation.benchmark_runner.runner import build_shared_shard_loop_session
    return build_shared_shard_loop_session(
        scenario=_MINIMAL_SCENARIO,
        run_id="vis-test-001",
        actor_ids=["actor-a", "actor-b"],
    )


def test_defeat_visible_in_observation_of_co_actor():
    """After actor-a is defeated, actor-b's observation mentions the defeat."""
    session = _build_minimal_shared_session()

    # Set actor-a as defeated by applying a scenario_var delta
    session.world_state.apply_delta({
        "scenario_vars": {
            "actor_defeated.actor-a": True,
            "actor_respawn_at.actor-a": 10,
        }
    })

    obs_b = session.get_observation(actor_id="actor-b")
    # actor-b should see co-actor defeat status
    assert any("actor-a" in line and "defeated" in line for line in obs_b.messages)


def test_defeat_status_not_shown_for_self_via_co_actor_path():
    """After actor-a is defeated, actor-a's observation does NOT get co-actor defeat message for self."""
    scenario_vars = {
        "actor_defeated.actor-a": True,
        "actor_respawn_at.actor-a": 10,
    }
    co_msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="actor-a")
    assert co_msgs == ()
    # But the self-defeat message is present
    self_msg = _format_actor_defeat_status_message(scenario_vars, actor_id="actor-a")
    assert self_msg is not None


def test_no_co_actor_defeat_messages_when_all_alive():
    """When no actors are defeated, no extra co-actor messages appear."""
    scenario_vars = {}
    msgs = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="actor-b")
    assert msgs == ()


def test_co_actor_defeat_messages_deterministic():
    """Co-actor defeat messages are ordered deterministically."""
    scenario_vars = {
        "actor_defeated.charlie": True,
        "actor_respawn_at.charlie": 20,
        "actor_defeated.alice": True,
        "actor_respawn_at.alice": 10,
        "actor_defeated.bob": True,
        "actor_respawn_at.bob": 15,
    }
    # Run twice and confirm order is consistent
    msgs1 = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="zara")
    msgs2 = _format_co_actor_defeat_status_messages(scenario_vars, actor_id="zara")
    assert msgs1 == msgs2
    assert len(msgs1) == 3
