"""Tests for the defend combat action.

Covers:
- defend appears in action space when hostile NPC is present
- defend does NOT appear when no hostile NPC is in actor's room
- defend reduces counter-attack damage this tick
- non-defending actor in same room still takes full damage
- defend is per-tick: next tick returns to full damage
- two actors can defend simultaneously
- action parser accepts "defend" as a no-argument verb
- save/load/reconnect: defending state is stored in scenario_vars and readable
- no regressions: attack still appears alongside defend
- no regressions: defend does not appear in room with only neutral NPCs
"""

from __future__ import annotations

from agents.gateway.action_parser import parse_action_command
from agents.gateway.observation_builder import build_observation_for_actor
from core.action_processor import ActionRequest
from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,
    _ACTOR_DEFENDING_KEY_PREFIX,
    _NPC_COUNTER_DAMAGE,
    _NPC_COUNTER_DAMAGE_DEFENDED,
    process_npc_tick,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_world(
    *,
    actor_health: int = 100,
    actor_room: str = "camp",
    npc_health: int = 20,
    npc_room: str = "camp",
    npc_tags: list[str] | None = None,
    extra_actors: list[dict] | None = None,
    scenario_vars: dict | None = None,
) -> DeterministicWorldStateManager:
    if npc_tags is None:
        npc_tags = ["hostile"]

    entities: dict = {
        "actor-a": {
            "entity_id": "actor-a",
            "entity_type": "player",
            "location": actor_room,
            "health": actor_health,
            "inventory": [],
            "tags": [],
        },
        "guardian": {
            "entity_id": "guardian",
            "entity_type": "npc",
            "location": npc_room,
            "health": npc_health,
            "tags": npc_tags,
        },
    }
    for actor in (extra_actors or []):
        entities[actor["entity_id"]] = actor

    rooms: dict = {
        "camp": {
            "room_id": "camp",
            "title": "Camp",
            "description": "A test camp.",
            "exits": {"east": "vault"},
            "entities_present": sorted(
                e for e, d in entities.items() if d.get("location") == "camp"
            ),
        },
        "vault": {
            "room_id": "vault",
            "title": "Vault",
            "description": "A test vault.",
            "exits": {"west": "camp"},
            "entities_present": sorted(
                e for e, d in entities.items() if d.get("location") == "vault"
            ),
        },
    }

    svars: dict = {"mode": "shared_shard"}
    if scenario_vars:
        svars.update(scenario_vars)

    return DeterministicWorldStateManager.from_dict({
        "entities": entities,
        "rooms": rooms,
        "scenario_vars": svars,
    })


def _action(actor_id: str, action_type: str, **kwargs: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type=action_type,
        arguments=tuple((k, v) for k, v in kwargs.items()),
    )


def _processor() -> BasicDeterministicActionProcessor:
    return BasicDeterministicActionProcessor()


def _snap(world: DeterministicWorldStateManager) -> dict:
    return world.get_snapshot()


# ---------------------------------------------------------------------------
# Action space tests
# ---------------------------------------------------------------------------

class TestDefendActionSpace:
    def test_defend_in_action_space_when_hostile_npc_in_room(self) -> None:
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        snap = _snap(world)
        obs = build_observation_for_actor(
            snap, actor_id="actor-a", run_id="r1", step=0, max_steps=10
        )
        assert "defend" in obs.action_space

    def test_defend_not_in_action_space_when_no_hostile_npc(self) -> None:
        """No hostile NPC in room → defend not needed → not shown."""
        world = _make_world(actor_room="camp", npc_room="vault", npc_tags=["hostile"])
        snap = _snap(world)
        obs = build_observation_for_actor(
            snap, actor_id="actor-a", run_id="r2", step=0, max_steps=10
        )
        assert "defend" not in obs.action_space
        assert "attack guardian" not in obs.action_space

    def test_defend_not_in_action_space_with_neutral_npc_only(self) -> None:
        """NPC tagged neutral → not an attack target → defend not shown."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["neutral"])
        snap = _snap(world)
        obs = build_observation_for_actor(
            snap, actor_id="actor-a", run_id="r3", step=0, max_steps=10
        )
        assert "defend" not in obs.action_space
        assert "attack guardian" not in obs.action_space

    def test_defend_appears_before_attack_in_action_space(self) -> None:
        """defend is listed before attack entries."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        snap = _snap(world)
        obs = build_observation_for_actor(
            snap, actor_id="actor-a", run_id="r4", step=0, max_steps=10
        )
        actions = list(obs.action_space)
        defend_idx = actions.index("defend")
        attack_idx = actions.index("attack guardian")
        assert defend_idx < attack_idx

    def test_attack_still_present_alongside_defend(self) -> None:
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        snap = _snap(world)
        obs = build_observation_for_actor(
            snap, actor_id="actor-a", run_id="r5", step=0, max_steps=10
        )
        assert "defend" in obs.action_space
        assert "attack guardian" in obs.action_space


# ---------------------------------------------------------------------------
# Action parser tests
# ---------------------------------------------------------------------------

class TestDefendParser:
    def test_defend_parses_as_no_argument_verb(self) -> None:
        result = parse_action_command(actor_id="actor-a", action="defend")
        assert result.accepted
        assert result.action_request is not None
        assert result.action_request.action_type == "defend"
        assert result.action_request.arguments == ()

    def test_defend_with_argument_is_rejected(self) -> None:
        result = parse_action_command(actor_id="actor-a", action="defend guardian")
        assert not result.accepted

    def test_defend_is_case_sensitive(self) -> None:
        result = parse_action_command(actor_id="actor-a", action="Defend")
        assert not result.accepted


# ---------------------------------------------------------------------------
# Defend action processing tests
# ---------------------------------------------------------------------------

class TestDefendActionProcessing:
    def test_defend_action_accepted(self) -> None:
        world = _make_world()
        proc = _processor()
        results = proc.process_actions(
            (_action("actor-a", "defend"),),
            world,
            step_index=0,
        )
        assert len(results) == 1
        assert results[0].accepted

    def test_defend_sets_defending_key_in_scenario_vars(self) -> None:
        world = _make_world()
        proc = _processor()
        results = proc.process_actions(
            (_action("actor-a", "defend"),),
            world,
            step_index=3,
        )
        result = results[0]
        assert result.accepted
        delta = dict(result.world_delta)
        svars_delta = dict(delta.get("scenario_vars", ()))
        defend_key = f"{_ACTOR_DEFENDING_KEY_PREFIX}actor-a"
        assert svars_delta.get(defend_key) == 3  # stores the step_index

    def test_defend_emits_action_defend_event(self) -> None:
        world = _make_world()
        proc = _processor()
        results = proc.process_actions(
            (_action("actor-a", "defend"),),
            world,
            step_index=1,
        )
        events = results[0].events
        assert len(events) == 1
        assert events[0].event_type == "action_defend"
        assert events[0].actor_id == "actor-a"


# ---------------------------------------------------------------------------
# Defend reduces counter-attack damage
# ---------------------------------------------------------------------------

class TestDefendReducesCounterDamage:
    def test_defending_actor_takes_reduced_damage(self) -> None:
        """Actor who defends takes _NPC_COUNTER_DAMAGE_DEFENDED instead of full damage."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        proc = _processor()
        # Apply defend action (sets defending.actor-a = step 0)
        def_results = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=0
        )
        assert def_results[0].accepted
        world.apply_delta(dict(def_results[0].world_delta))

        # Run NPC counter-attack tick
        tick_result = process_npc_tick(world, step_index=0)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        actor_delta = dict(entities_delta.get("actor-a", ()))
        new_hp = actor_delta.get("health")
        assert new_hp == 100 - _NPC_COUNTER_DAMAGE_DEFENDED

    def test_non_defending_actor_takes_full_damage(self) -> None:
        """Actor who does NOT defend takes full _NPC_COUNTER_DAMAGE."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        tick_result = process_npc_tick(world, step_index=0)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        actor_delta = dict(entities_delta.get("actor-a", ()))
        new_hp = actor_delta.get("health")
        assert new_hp == 100 - _NPC_COUNTER_DAMAGE

    def test_defend_event_marks_defended_true_in_payload(self) -> None:
        """Counter-attack event marks defended=True when actor is defending."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        proc = _processor()
        def_results = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=0
        )
        world.apply_delta(dict(def_results[0].world_delta))

        tick_result = process_npc_tick(world, step_index=0)
        assert tick_result is not None
        events = tick_result.events
        counter_events = [e for e in events if e.event_type == "npc_counter_attack"]
        assert counter_events
        payload_dict = dict(counter_events[0].payload)
        assert payload_dict.get("defended") is True

    def test_defend_damage_constants_are_correct_values(self) -> None:
        """Sanity check: reduced damage is less than full damage."""
        assert _NPC_COUNTER_DAMAGE_DEFENDED < _NPC_COUNTER_DAMAGE


# ---------------------------------------------------------------------------
# Per-tick: defending state does not carry over
# ---------------------------------------------------------------------------

class TestDefendIsPerTick:
    def test_defending_state_ignored_on_next_tick(self) -> None:
        """Defending state from tick N does not reduce damage on tick N+1."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        proc = _processor()

        # Defend on tick 0
        def_results = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=0
        )
        world.apply_delta(dict(def_results[0].world_delta))

        # Counter-attack on tick 1 (different step_index → defending state mismatch)
        tick_result = process_npc_tick(world, step_index=1)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        actor_delta = dict(entities_delta.get("actor-a", ()))
        new_hp = actor_delta.get("health")
        # Full damage — defend state is stale
        assert new_hp == 100 - _NPC_COUNTER_DAMAGE


# ---------------------------------------------------------------------------
# Two actors can defend simultaneously
# ---------------------------------------------------------------------------

class TestTwoActorsDefend:
    def test_both_actors_defending_both_take_reduced_damage(self) -> None:
        extra = {
            "entity_id": "actor-b",
            "entity_type": "agent",
            "location": "camp",
            "health": 100,
            "inventory": [],
            "tags": [],
        }
        world = _make_world(
            actor_room="camp", npc_room="camp", npc_tags=["hostile"],
            extra_actors=[extra],
        )
        # Update camp entities_present to include actor-b
        snap = _snap(world)
        rooms = snap["rooms"]
        camp = dict(rooms["camp"])
        camp["entities_present"] = sorted(
            e for e, d in snap["entities"].items() if d.get("location") == "camp"
        )
        world.apply_delta({"rooms": {"camp": camp}})

        proc = _processor()
        def_a = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=5
        )
        world.apply_delta(dict(def_a[0].world_delta))
        def_b = proc.process_actions(
            (_action("actor-b", "defend"),), world, step_index=5
        )
        world.apply_delta(dict(def_b[0].world_delta))

        tick_result = process_npc_tick(world, step_index=5)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        hp_a = dict(entities_delta.get("actor-a", {})).get("health")
        hp_b = dict(entities_delta.get("actor-b", {})).get("health")
        assert hp_a == 100 - _NPC_COUNTER_DAMAGE_DEFENDED
        assert hp_b == 100 - _NPC_COUNTER_DAMAGE_DEFENDED

    def test_one_defending_one_not(self) -> None:
        """Only the defending actor takes reduced damage; the other takes full damage."""
        extra = {
            "entity_id": "actor-b",
            "entity_type": "agent",
            "location": "camp",
            "health": 100,
            "inventory": [],
            "tags": [],
        }
        world = _make_world(
            actor_room="camp", npc_room="camp", npc_tags=["hostile"],
            extra_actors=[extra],
        )
        snap = _snap(world)
        rooms = snap["rooms"]
        camp = dict(rooms["camp"])
        camp["entities_present"] = sorted(
            e for e, d in snap["entities"].items() if d.get("location") == "camp"
        )
        world.apply_delta({"rooms": {"camp": camp}})

        proc = _processor()
        # Only actor-a defends
        def_a = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=7
        )
        world.apply_delta(dict(def_a[0].world_delta))

        tick_result = process_npc_tick(world, step_index=7)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        hp_a = dict(entities_delta.get("actor-a", {})).get("health")
        hp_b = dict(entities_delta.get("actor-b", {})).get("health")
        assert hp_a == 100 - _NPC_COUNTER_DAMAGE_DEFENDED
        assert hp_b == 100 - _NPC_COUNTER_DAMAGE


# ---------------------------------------------------------------------------
# Save/load/reconnect: defending state persists correctly
# ---------------------------------------------------------------------------

class TestDefendSaveLoadReconnect:
    def test_defending_key_persists_after_apply_delta(self) -> None:
        """Defending state is stored in scenario_vars and visible in snapshot."""
        world = _make_world()
        proc = _processor()
        results = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=4
        )
        world.apply_delta(dict(results[0].world_delta))

        snap = _snap(world)
        defend_key = f"{_ACTOR_DEFENDING_KEY_PREFIX}actor-a"
        assert snap["scenario_vars"].get(defend_key) == 4

    def test_defending_key_reload_and_counter_attack(self) -> None:
        """A world reloaded from snapshot with defending key still reduces damage."""
        world = _make_world(actor_room="camp", npc_room="camp", npc_tags=["hostile"])
        proc = _processor()
        # Apply defend
        results = proc.process_actions(
            (_action("actor-a", "defend"),), world, step_index=2
        )
        world.apply_delta(dict(results[0].world_delta))

        # Simulate save/load: take snapshot and reload
        snap = _snap(world)
        reloaded = DeterministicWorldStateManager.from_dict(snap)

        # Counter-attack on same step_index → defending state still active
        tick_result = process_npc_tick(reloaded, step_index=2)
        assert tick_result is not None
        delta = dict(tick_result.world_delta)
        entities_delta = dict(delta.get("entities", ()))
        new_hp = dict(entities_delta.get("actor-a", {})).get("health")
        assert new_hp == 100 - _NPC_COUNTER_DAMAGE_DEFENDED


# ---------------------------------------------------------------------------
# Integration: defend via shared shard loop
# ---------------------------------------------------------------------------

class TestDefendIntegration:
    def test_defend_accepted_in_shared_shard_session(self) -> None:
        """defend is accepted as a valid action in a shared shard session."""
        from cli.main import _SCENARIO_PRESETS
        from evaluation.benchmark_runner.runner import build_shared_shard_loop_session

        session = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=["player-a", "player-b"],
            run_id="test-defend-shard",
            max_steps_override=15,
        )
        # Move into vault-entrance (where guardian is)
        session.advance_tick({"player-a": "move east", "player-b": "move east"})
        obs = session.get_observation("player-a")
        assert "defend" in obs.action_space
        # Should not raise
        session.advance_tick({"player-a": "defend", "player-b": "wait"})
        obs2 = session.get_observation("player-a")
        # Player is still alive and can act
        assert obs2.health is not None

    def test_defend_reduces_counter_damage_in_session(self) -> None:
        """Actor who defends takes less damage from hostile NPC than one who waits."""
        from cli.main import _SCENARIO_PRESETS
        from evaluation.benchmark_runner.runner import build_shared_shard_loop_session

        # Session A: actor waits (takes full counter damage from patrol)
        session_wait = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=["player-a", "player-b"],
            run_id="test-defend-wait",
            max_steps_override=15,
        )
        session_wait.advance_tick({"player-a": "move east", "player-b": "move east"})
        # Patrol is hostile — counter-attacks on entry tick; now wait
        session_wait.advance_tick({"player-a": "wait", "player-b": "wait"})
        obs_wait = session_wait.get_observation("player-a")
        health_after_wait = obs_wait.health

        # Session B: actor defends (takes reduced damage)
        session_defend = build_shared_shard_loop_session(
            scenario=_SCENARIO_PRESETS["tiny-shared-combat"],
            actor_ids=["player-c", "player-d"],
            run_id="test-defend-defend",
            max_steps_override=15,
        )
        session_defend.advance_tick({"player-c": "move east", "player-d": "move east"})
        session_defend.advance_tick({"player-c": "defend", "player-d": "defend"})
        obs_defend = session_defend.get_observation("player-c")
        health_after_defend = obs_defend.health

        # Defending should result in higher (or equal) health
        assert health_after_defend is not None
        assert health_after_wait is not None
        assert health_after_defend >= health_after_wait
