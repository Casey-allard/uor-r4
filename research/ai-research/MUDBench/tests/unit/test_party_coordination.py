"""Tests for shared-world party coordination status surface.

Covers:
- _format_party_status_messages: alive co-actor with health+location
- _format_party_status_messages: defeated co-actor shows DEFEATED state
- _format_party_status_messages: self excluded
- _format_party_status_messages: multiple co-actors, sorted deterministically
- _format_party_status_messages: missing entity (graceful degradation)
- _format_party_status_messages: empty actor list
- _format_party_status_messages: non-mapping inputs
- Integration via get_observation: party status appears in messages
- Reconnect coherence: party status works after save/load
- No regressions to existing defeat/respawn visibility
"""

from __future__ import annotations

from evaluation.benchmark_runner.runner import (
    _format_party_status_messages,
)


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _snapshot(actors: dict) -> dict:
    """Build a minimal world snapshot with actor entities."""
    entities = {}
    for actor_id, data in actors.items():
        entities[actor_id] = {
            "entity_id": actor_id,
            "entity_type": "player",
            "location": data.get("location", "camp"),
            "health": data.get("health", 100),
            "inventory": [],
            "tags": [],
        }
    return {"entities": entities, "rooms": {}, "scenario_vars": {}}


# ---------------------------------------------------------------------------
# _format_party_status_messages — basic correctness
# ---------------------------------------------------------------------------


def test_alive_co_actor_shows_health_and_location():
    """Alive co-actor shows their current location and HP."""
    snap = _snapshot({"alice": {"location": "vault", "health": 45}})
    msgs = _format_party_status_messages(snap, {}, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "[Party] alice" in msgs[0]
    assert "vault" in msgs[0]
    assert "45 HP" in msgs[0]


def test_defeated_co_actor_shows_defeated_state_with_respawn_tick():
    """Defeated co-actor shows DEFEATED and respawn tick instead of HP."""
    snap = _snapshot({"alice": {"location": "camp", "health": 0}})
    svars = {"actor_defeated.alice": True, "actor_respawn_at.alice": 12}
    msgs = _format_party_status_messages(snap, svars, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "DEFEATED" in msgs[0]
    assert "12" in msgs[0]
    assert "[Party] alice" in msgs[0]


def test_defeated_co_actor_no_respawn_tick():
    """Defeated co-actor with no respawn tick shows 'awaiting respawn'."""
    snap = _snapshot({"alice": {"location": "camp", "health": 0}})
    svars = {"actor_defeated.alice": True}
    msgs = _format_party_status_messages(snap, svars, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "DEFEATED" in msgs[0]
    assert "awaiting respawn" in msgs[0]


def test_self_excluded():
    """The calling actor does not appear in their own party status."""
    snap = _snapshot({"bob": {"location": "vault", "health": 80}})
    msgs = _format_party_status_messages(snap, {}, ["bob"], "bob")
    assert msgs == ()


def test_self_excluded_with_co_actors():
    """Self is excluded but co-actors still appear."""
    snap = _snapshot({
        "bob": {"location": "vault", "health": 80},
        "alice": {"location": "camp", "health": 30},
    })
    msgs = _format_party_status_messages(snap, {}, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "[Party] alice" in msgs[0]
    assert "bob" not in msgs[0]


def test_multiple_co_actors_sorted_deterministically():
    """Multiple co-actors appear in sorted order each time."""
    snap = _snapshot({
        "charlie": {"location": "vault", "health": 20},
        "alice": {"location": "camp", "health": 100},
        "bob": {"location": "vault-entrance", "health": 50},
    })
    msgs1 = _format_party_status_messages(snap, {}, ["alice", "bob", "charlie", "zara"], "zara")
    msgs2 = _format_party_status_messages(snap, {}, ["charlie", "alice", "bob", "zara"], "zara")
    assert msgs1 == msgs2
    assert len(msgs1) == 3
    # Sorted: alice, bob, charlie
    assert msgs1[0].startswith("[Party] alice")
    assert msgs1[1].startswith("[Party] bob")
    assert msgs1[2].startswith("[Party] charlie")


def test_missing_entity_graceful():
    """Co-actor in actor_ids but not in snapshot entities shows unknown location."""
    snap = {"entities": {}, "rooms": {}}
    msgs = _format_party_status_messages(snap, {}, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "[Party] alice" in msgs[0]
    assert "unknown" in msgs[0]


def test_empty_actor_ids_returns_empty():
    """Empty actor list returns no messages."""
    snap = _snapshot({"bob": {"location": "camp", "health": 100}})
    msgs = _format_party_status_messages(snap, {}, [], "bob")
    assert msgs == ()


def test_only_self_in_actor_ids_returns_empty():
    """Single actor (only self) returns no messages."""
    snap = _snapshot({"bob": {"location": "camp", "health": 100}})
    msgs = _format_party_status_messages(snap, {}, ["bob"], "bob")
    assert msgs == ()


def test_non_mapping_snapshot_returns_empty():
    """Non-mapping snapshot returns empty gracefully."""
    msgs = _format_party_status_messages(None, {}, ["alice", "bob"], "bob")  # type: ignore[arg-type]
    assert msgs == ()


def test_non_mapping_scenario_vars_handled():
    """Non-mapping scenario_vars is handled gracefully."""
    snap = _snapshot({"alice": {"location": "vault", "health": 45}})
    msgs = _format_party_status_messages(snap, None, ["alice", "bob"], "bob")  # type: ignore[arg-type]
    # Should still show alive status (defeat defaults to False)
    assert len(msgs) == 1
    assert "45 HP" in msgs[0]


def test_alive_actor_not_showing_defeated_when_defeat_false():
    """Actor with actor_defeated.X = False is shown as alive."""
    snap = _snapshot({"alice": {"location": "vault", "health": 45}})
    svars = {"actor_defeated.alice": False}
    msgs = _format_party_status_messages(snap, svars, ["alice", "bob"], "bob")
    assert len(msgs) == 1
    assert "DEFEATED" not in msgs[0]
    assert "45 HP" in msgs[0]


def test_mixed_alive_and_defeated():
    """Party with one alive and one defeated shows both correctly."""
    snap = _snapshot({
        "alice": {"location": "vault", "health": 40},
        "carol": {"location": "camp", "health": 0},
    })
    svars = {"actor_defeated.carol": True, "actor_respawn_at.carol": 15}
    msgs = _format_party_status_messages(snap, svars, ["alice", "carol", "bob"], "bob")
    assert len(msgs) == 2
    alice_msg = next(m for m in msgs if "alice" in m)
    carol_msg = next(m for m in msgs if "carol" in m)
    assert "40 HP" in alice_msg
    assert "DEFEATED" in carol_msg
    assert "15" in carol_msg


# ---------------------------------------------------------------------------
# Integration via get_observation
# ---------------------------------------------------------------------------

_MINIMAL_SCENARIO = {
    "scenario_id": "test-party-coord",
    "title": "Test Party Coordination",
    "description": "Party coordination test.",
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


def _build_session():
    from evaluation.benchmark_runner.runner import build_shared_shard_loop_session
    return build_shared_shard_loop_session(
        scenario=_MINIMAL_SCENARIO,
        run_id="party-test-001",
        actor_ids=["actor-a", "actor-b"],
    )


def test_party_status_in_observation_shows_co_actor():
    """actor-b's observation includes a [Party] line for actor-a."""
    session = _build_session()
    obs = session.get_observation(actor_id="actor-b")
    assert any("[Party] actor-a" in m for m in obs.messages)


def test_party_status_shows_health_and_location():
    """[Party] line includes location and HP for alive co-actor."""
    session = _build_session()
    obs = session.get_observation(actor_id="actor-b")
    party_lines = [m for m in obs.messages if "[Party] actor-a" in m]
    assert len(party_lines) == 1
    assert "HP" in party_lines[0]
    assert "camp" in party_lines[0]  # default start room


def test_party_status_self_not_shown():
    """The observing actor does not see themselves in [Party] lines."""
    session = _build_session()
    obs = session.get_observation(actor_id="actor-b")
    assert not any("[Party] actor-b" in m for m in obs.messages)


def test_party_status_defeated_co_actor_in_observation():
    """When actor-a is defeated, actor-b sees DEFEATED in party status."""
    session = _build_session()
    session.world_state.apply_delta({
        "scenario_vars": {
            "actor_defeated.actor-a": True,
            "actor_respawn_at.actor-a": 10,
        }
    })
    obs = session.get_observation(actor_id="actor-b")
    party_lines = [m for m in obs.messages if "[Party] actor-a" in m]
    assert len(party_lines) == 1
    assert "DEFEATED" in party_lines[0]
    assert "10" in party_lines[0]


def test_party_status_updates_after_move():
    """Party status reflects updated location after actor-a moves."""
    session = _build_session()
    # Force actor-a's location to vault
    session.world_state.apply_delta({
        "entities": {
            "actor-a": {
                "entity_id": "actor-a",
                "entity_type": "player",
                "location": "vault",
                "health": 100,
                "inventory": [],
                "tags": [],
            }
        }
    })
    obs = session.get_observation(actor_id="actor-b")
    party_lines = [m for m in obs.messages if "[Party] actor-a" in m]
    assert len(party_lines) == 1
    assert "vault" in party_lines[0]


def test_party_status_reconnect_coherent():
    """After save/load, party status is still surfaced correctly."""
    import tempfile
    import os
    from evaluation.benchmark_runner.runner import build_shared_shard_loop_session
    from world.state.world_persistence import save_world_snapshot

    session = _build_session()
    # Force actor-a to vault
    session.world_state.apply_delta({
        "entities": {
            "actor-a": {
                "entity_id": "actor-a",
                "entity_type": "player",
                "location": "vault",
                "health": 75,
                "inventory": [],
                "tags": [],
            }
        }
    })

    with tempfile.TemporaryDirectory() as tmpdir:
        save_path = os.path.join(tmpdir, "world.json")
        save_result = save_world_snapshot(
            save_path, session.world_state,
            run_id="party-save-001", scenario_id="test-party-coord",
        )
        assert save_result.accepted

        # Reload into a new session
        session2 = build_shared_shard_loop_session(
            scenario=_MINIMAL_SCENARIO,
            run_id="party-reload-001",
            actor_ids=["actor-a", "actor-b"],
            world_load_path=save_path,
        )
        obs = session2.get_observation(actor_id="actor-b")
        party_lines = [m for m in obs.messages if "[Party] actor-a" in m]
        assert len(party_lines) == 1
        assert "vault" in party_lines[0]
        assert "75 HP" in party_lines[0]


def test_party_status_deterministic_across_calls():
    """Repeated calls with same state produce identical messages."""
    session = _build_session()
    msgs1 = [m for m in session.get_observation(actor_id="actor-b").messages if "[Party]" in m]
    msgs2 = [m for m in session.get_observation(actor_id="actor-b").messages if "[Party]" in m]
    assert msgs1 == msgs2
