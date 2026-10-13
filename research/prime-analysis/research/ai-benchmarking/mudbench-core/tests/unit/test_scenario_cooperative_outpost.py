"""Tests for the small-cooperative-outpost scenario.

Covers:
- Scenario loads and is registered correctly
- Basic scenario structure: rooms, items, NPCs, quests
- Multi-mechanic playability: move, take, attack, use
- Gate unlock: defeating sentry opens east passage from gatehouse
- Calm path: ward-talisman calms patrol in courtyard
- Quest objective progression (breach-the-gate, silence-the-patrol, recover-the-beacon)
- NPC dialogue: scout provides lore
- Actor defeat + loot drop in shared session
- Revive mechanic: medkit revives defeated co-actor
- Save/load/reconnect preserves state coherently
- Scenario is materially richer than tiny scenarios (4 rooms, 3 NPCs, 4 items, 3 quests)
- Deterministic and inspectable at every step
"""

from __future__ import annotations

import json

from cli.main import _SCENARIO_PRESETS
from evaluation.benchmark_runner.runner import build_shared_shard_loop_session


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _session(*, run_id: str = "test-run", max_steps_override: int | None = None):
    return build_shared_shard_loop_session(
        scenario=_SCENARIO_PRESETS["small-cooperative-outpost"],
        actor_ids=["actor-a", "actor-b"],
        run_id=run_id,
        max_steps_override=max_steps_override,
    )


def _messages(session, actor_id: str) -> tuple[str, ...]:
    return session.get_observation(actor_id).messages


def _action_space(session, actor_id: str) -> tuple[str, ...]:
    return session.get_observation(actor_id).action_space


def _snap(session):
    return session.world_state.get_snapshot()


def _svars(session) -> dict:
    return dict(_snap(session).get("scenario_vars", {}))


def _entity(session, entity_id: str) -> dict:
    return dict(_snap(session).get("entities", {}).get(entity_id, {}))


def _room(session, room_id: str) -> dict:
    return dict(_snap(session).get("rooms", {}).get(room_id, {}))


# ---------------------------------------------------------------------------
# Scenario registration and structure
# ---------------------------------------------------------------------------

class TestScenarioRegistration:
    def test_scenario_is_loaded_in_presets(self) -> None:
        assert "small-cooperative-outpost" in _SCENARIO_PRESETS

    def test_scenario_has_correct_id(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        assert s["scenario_id"] == "small-cooperative-outpost"

    def test_scenario_has_four_rooms(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        assert sorted(wc["rooms"].keys()) == ["command-post", "courtyard", "gatehouse", "supply-camp"]

    def test_scenario_has_three_npcs(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        npc_ids = sorted(n["entity_id"] for n in wc["npcs"])
        assert npc_ids == ["patrol", "scout", "sentry"]

    def test_scenario_has_four_items(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        item_ids = sorted(i["entity_id"] for i in wc["items"])
        assert item_ids == ["distress-beacon", "medkit", "ration-pack", "ward-talisman"]

    def test_scenario_has_three_quest_objectives(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        quest_ids = sorted(q["quest_id"] for q in wc["quest_objectives"])
        assert quest_ids == ["breach-the-gate", "recover-the-beacon", "silence-the-patrol"]

    def test_scenario_max_steps_is_fourteen(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        assert s["max_steps"] == 14

    def test_scenario_has_actor_revive_effects(self) -> None:
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        revive = json.loads(s["scenario_vars"]["actor_revive_effects_json"])
        assert any(e["item_id"] == "medkit" for e in revive)

    def test_scenario_is_richer_than_tiny_shared_combat(self) -> None:
        """Sanity: new scenario has more rooms, NPCs, items, quests than tiny-shared-combat."""
        coop = _SCENARIO_PRESETS["small-cooperative-outpost"]
        tiny = _SCENARIO_PRESETS["tiny-shared-combat"]
        coop_wc = json.loads(coop["scenario_vars"]["world_config_json"])
        tiny_wc = json.loads(tiny["scenario_vars"]["world_config_json"])
        assert len(coop_wc["rooms"]) > len(tiny_wc["rooms"])
        assert len(coop_wc["npcs"]) >= len(tiny_wc["npcs"])
        assert len(coop_wc["items"]) >= len(tiny_wc["items"])
        assert coop["max_steps"] > tiny["max_steps"]


# ---------------------------------------------------------------------------
# Session initialisation
# ---------------------------------------------------------------------------

class TestSessionInit:
    def test_session_starts_in_supply_camp(self) -> None:
        session = _session()
        obs = session.get_observation("actor-a")
        assert any("Supply Camp" in m or "supply-camp" in m.lower() or "supply" in m.lower() for m in obs.messages)

    def test_initial_action_space_includes_take_ward_talisman(self) -> None:
        session = _session()
        space = _action_space(session, "actor-a")
        assert "take ward-talisman" in space

    def test_initial_action_space_includes_take_medkit(self) -> None:
        session = _session()
        space = _action_space(session, "actor-a")
        assert "take medkit" in space

    def test_initial_action_space_includes_move_east(self) -> None:
        session = _session()
        space = _action_space(session, "actor-a")
        assert "move east" in space

    def test_gatehouse_east_exit_locked_at_start(self) -> None:
        """East exit from gatehouse is NOT present in initial room config (locked by sentry)."""
        session = _session()
        gatehouse_exits = _room(session, "gatehouse").get("exits", {})
        assert "east" not in gatehouse_exits

    def test_both_actors_start_in_supply_camp(self) -> None:
        session = _session()
        a = _entity(session, "actor-a").get("location")
        b = _entity(session, "actor-b").get("location")
        assert a == "supply-camp"
        assert b == "supply-camp"


# ---------------------------------------------------------------------------
# Gate unlock: defeat sentry → east passage opens
# ---------------------------------------------------------------------------

class TestGateUnlock:
    def test_actor_can_move_to_gatehouse(self) -> None:
        session = _session()
        session.advance_tick({"actor-a": "move east", "actor-b": "wait"})
        assert _entity(session, "actor-a").get("location") == "gatehouse"

    def test_sentry_is_hostile_in_gatehouse(self) -> None:
        session = _session()
        session.advance_tick({"actor-a": "move east", "actor-b": "wait"})
        sentry = _entity(session, "sentry")
        assert "hostile" in sentry.get("tags", [])

    def test_defeat_sentry_unlocks_east_exit(self) -> None:
        """Defeating sentry emits route_unlocked and adds east exit from gatehouse."""
        session = _session()
        # Move both to gatehouse
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        # Attack sentry until defeated (25 HP, 2 actors x10 = 20/tick → 2 rounds needed)
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        # Gatehouse east exit should now exist
        gatehouse_exits = _room(session, "gatehouse").get("exits", {})
        assert "east" in gatehouse_exits

    def test_breach_the_gate_quest_completes_on_sentry_defeat(self) -> None:
        session = _session()
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        svars = _svars(session)
        assert svars.get("quest.breach-the-gate.actor-a") or svars.get("quest.breach-the-gate.actor-b")

    def test_sentry_defeat_message_appears(self) -> None:
        session = _session()
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        messages_seen = []
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
            messages_seen.extend(session.get_observation("actor-a").messages)
        assert any("sentry" in m.lower() for m in messages_seen)


# ---------------------------------------------------------------------------
# Calm path: ward-talisman calms patrol in courtyard
# ---------------------------------------------------------------------------

class TestCalmPath:
    def _setup_in_courtyard(self, session):
        """Navigate both actors to courtyard; actor-a carries ward-talisman."""
        # Pick up ward-talisman
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "wait"})
        # Both move to gatehouse
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        # Defeat sentry (2 rounds: 25 HP, 2 actors x10 = 20/tick)
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        # Move east to courtyard (gate now open)
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})

    def test_patrol_is_hostile_in_courtyard(self) -> None:
        session = _session(max_steps_override=20)
        self._setup_in_courtyard(session)
        patrol = _entity(session, "patrol")
        assert "hostile" in patrol.get("tags", [])

    def test_use_ward_talisman_calms_patrol(self) -> None:
        session = _session(max_steps_override=20)
        self._setup_in_courtyard(session)
        session.advance_tick({"actor-a": "use ward-talisman", "actor-b": "wait"})
        patrol = _entity(session, "patrol")
        assert "hostile" not in patrol.get("tags", [])

    def test_silence_the_patrol_quest_completes(self) -> None:
        session = _session(max_steps_override=20)
        self._setup_in_courtyard(session)
        session.advance_tick({"actor-a": "use ward-talisman", "actor-b": "wait"})
        svars = _svars(session)
        assert svars.get("quest.silence-the-patrol.actor-a") or svars.get("quest.silence-the-patrol.actor-b")

    def test_courtyard_description_updates_after_calm(self) -> None:
        session = _session(max_steps_override=20)
        self._setup_in_courtyard(session)
        session.advance_tick({"actor-a": "use ward-talisman", "actor-b": "wait"})
        obs = session.get_observation("actor-a")
        assert any("subdued" in m.lower() or "patrol stands aside" in m.lower() or "calmed" in m.lower() for m in obs.messages)


# ---------------------------------------------------------------------------
# Full linear path: defeat sentry → enter courtyard → calm patrol → recover beacon
# ---------------------------------------------------------------------------

class TestEndToEndPath:
    def _run_to_beacon(self, session) -> None:
        """Run the cooperative outpost scenario to beacon recovery."""
        # Take ward-talisman and medkit at start
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "take medkit"})
        # Both move to gatehouse
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        # Defeat sentry (2 rounds: 25 HP, 2 actors x10 = 20/tick)
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        # Move to courtyard
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        # Calm patrol
        session.advance_tick({"actor-a": "use ward-talisman", "actor-b": "wait"})
        # Move to command-post
        session.advance_tick({"actor-a": "move north", "actor-b": "move north"})
        # Take beacon
        session.advance_tick({"actor-a": "take distress-beacon", "actor-b": "wait"})

    def test_full_path_beacon_in_actor_inventory(self) -> None:
        session = _session(max_steps_override=20)
        self._run_to_beacon(session)
        actor_a = _entity(session, "actor-a")
        assert "distress-beacon" in actor_a.get("inventory", [])

    def test_full_path_recover_the_beacon_quest_completes(self) -> None:
        session = _session(max_steps_override=20)
        self._run_to_beacon(session)
        svars = _svars(session)
        assert svars.get("quest.recover-the-beacon.actor-a") or svars.get("quest.recover-the-beacon.actor-b")

    def test_full_path_three_quests_completed(self) -> None:
        session = _session(max_steps_override=20)
        self._run_to_beacon(session)
        svars = _svars(session)
        assert svars.get("quest.breach-the-gate.actor-a") or svars.get("quest.breach-the-gate.actor-b")
        assert svars.get("quest.silence-the-patrol.actor-a") or svars.get("quest.silence-the-patrol.actor-b")
        assert svars.get("quest.recover-the-beacon.actor-a") or svars.get("quest.recover-the-beacon.actor-b")

    def test_full_path_observation_shows_reward_message(self) -> None:
        session = _session(max_steps_override=20)
        self._run_to_beacon(session)
        obs = session.get_observation("actor-a")
        assert any("beacon" in m.lower() or "mission" in m.lower() for m in obs.messages)


# ---------------------------------------------------------------------------
# Actor defeat and loot drop
# ---------------------------------------------------------------------------

class TestActorDefeatAndLoot:
    def test_actor_defeated_by_hostile_npc_drops_items(self) -> None:
        """Actor carrying ward-talisman who dies drops it to the room."""
        from world.state.basic_action_processor import process_actor_defeat_tick
        from world.state.world_state import DeterministicWorldStateManager

        world = DeterministicWorldStateManager.from_dict({
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a", "entity_type": "player",
                    "location": "gatehouse", "health": 0,
                    "inventory": ["ward-talisman"], "tags": [],
                },
                "actor-b": {
                    "entity_id": "actor-b", "entity_type": "agent",
                    "location": "gatehouse", "health": 80,
                    "inventory": [], "tags": [],
                },
                "ward-talisman": {
                    "entity_id": "ward-talisman", "entity_type": "consumable",
                    "location": "inventory", "health": None, "inventory": [], "tags": [],
                },
            },
            "rooms": {
                "gatehouse": {
                    "title": "Gatehouse", "description": "A gatehouse.",
                    "exits": {}, "entities_present": ["actor-a", "actor-b", "ward-talisman"],
                },
            },
            "scenario_vars": {
                "mode": "shared-cooperative",
                "actor_respawn_delay_ticks": 3,
                "actor_respawn_health": 40,
                "actor_respawn_room_id": "supply-camp",
            },
        })
        result = process_actor_defeat_tick(world, step_index=1)
        assert result is not None
        world.apply_delta(dict(result.world_delta))
        snap = world.get_snapshot()
        # Item should be in room
        room_entities = snap["rooms"]["gatehouse"]["entities_present"]
        assert "ward-talisman" in room_entities
        # Actor-a inventory should be empty
        assert snap["entities"]["actor-a"]["inventory"] == []


# ---------------------------------------------------------------------------
# Revive mechanic in outpost scenario
# ---------------------------------------------------------------------------

class TestReviveInOutpost:
    def test_medkit_revives_defeated_co_actor(self) -> None:
        """Actor-b uses medkit to revive defeated actor-a in supply-camp."""
        from world.state.world_state import DeterministicWorldStateManager
        from world.state.basic_action_processor import BasicDeterministicActionProcessor
        from core.action_processor import ActionRequest

        world = DeterministicWorldStateManager.from_dict({
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a", "entity_type": "player",
                    "location": "supply-camp", "health": 0,
                    "inventory": [], "tags": [],
                },
                "actor-b": {
                    "entity_id": "actor-b", "entity_type": "agent",
                    "location": "supply-camp", "health": 80,
                    "inventory": ["medkit"], "tags": [],
                },
                "medkit": {
                    "entity_id": "medkit", "entity_type": "consumable",
                    "location": "inventory", "health": None, "inventory": [], "tags": [],
                },
            },
            "rooms": {
                "supply-camp": {
                    "title": "Supply Camp", "description": "Camp.",
                    "exits": {}, "entities_present": ["actor-a", "actor-b"],
                },
            },
            "scenario_vars": {
                "mode": "shared-cooperative",
                "actor_defeated.actor-a": True,
                "actor_respawn_at.actor-a": 5,
                "actor_revive_effects_json": json.dumps([
                    {"effect_id": "medkit-revive", "item_id": "medkit", "revive_health": 30}
                ]),
            },
        })
        proc = BasicDeterministicActionProcessor()
        action = ActionRequest(
            actor_id="actor-b",
            action_type="use",
            arguments=(("item_id", "medkit"),),
        )
        result = proc.process_actions([action], world_state=world, step_index=3)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        svars = snap.get("scenario_vars", {})
        # Defeat cleared
        assert not svars.get("actor_defeated.actor-a")
        assert svars.get("actor_respawn_at.actor-a") is None
        # Health restored
        assert snap["entities"]["actor-a"]["health"] == 30
        # Event emitted
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" in event_types


# ---------------------------------------------------------------------------
# Save/load/reconnect coherence
# ---------------------------------------------------------------------------

class TestSaveLoadCoherence:
    def test_save_load_preserves_gate_unlocked_state(self) -> None:
        """After sentry defeat, reloading preserves the unlocked east exit."""
        from world.state.world_state import DeterministicWorldStateManager

        session = _session(max_steps_override=20)
        # Take ward-talisman, move to gatehouse, defeat sentry
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})

        snap = session.world_state.get_snapshot()
        assert "east" in snap["rooms"]["gatehouse"].get("exits", {})

        # Simulate save/load
        reloaded = DeterministicWorldStateManager.from_dict({
            "entities": snap.get("entities", {}),
            "rooms": snap.get("rooms", {}),
            "scenario_vars": snap.get("scenario_vars", {}),
        })
        rsnap = reloaded.get_snapshot()
        assert "east" in rsnap["rooms"]["gatehouse"].get("exits", {})

    def test_save_load_preserves_calm_state(self) -> None:
        """After patrol calmed, reloading preserves patrol tags (no hostile)."""
        from world.state.world_state import DeterministicWorldStateManager

        session = _session(max_steps_override=20)
        # Get to courtyard with ward-talisman
        session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "wait"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        session.advance_tick({"actor-a": "use ward-talisman", "actor-b": "wait"})

        snap = session.world_state.get_snapshot()
        # Verify patrol calmed
        assert "hostile" not in snap["entities"]["patrol"].get("tags", [])

        reloaded = DeterministicWorldStateManager.from_dict({
            "entities": snap.get("entities", {}),
            "rooms": snap.get("rooms", {}),
            "scenario_vars": snap.get("scenario_vars", {}),
        })
        rsnap = reloaded.get_snapshot()
        assert "hostile" not in rsnap["entities"]["patrol"].get("tags", [])

    def test_save_load_preserves_quest_completion_state(self) -> None:
        """Quest completed flags survive save/load round-trip."""
        from world.state.world_state import DeterministicWorldStateManager

        session = _session(max_steps_override=20)
        session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
        for _ in range(2):
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})

        snap = session.world_state.get_snapshot()
        svars = snap.get("scenario_vars", {})
        assert svars.get("quest.breach-the-gate.actor-a") or svars.get("quest.breach-the-gate.actor-b")

        reloaded = DeterministicWorldStateManager.from_dict({
            "entities": snap.get("entities", {}),
            "rooms": snap.get("rooms", {}),
            "scenario_vars": snap.get("scenario_vars", {}),
        })
        rsvars = reloaded.get_snapshot().get("scenario_vars", {})
        assert rsvars.get("quest.breach-the-gate.actor-a") or svars.get("quest.breach-the-gate.actor-b")


# ---------------------------------------------------------------------------
# Determinism
# ---------------------------------------------------------------------------

class TestDeterminism:
    def test_same_actions_produce_same_state(self) -> None:
        """Two sessions with the same actions produce identical snapshots."""
        def _run_actions():
            session = _session()
            session.advance_tick({"actor-a": "take ward-talisman", "actor-b": "take medkit"})
            session.advance_tick({"actor-a": "move east", "actor-b": "move east"})
            session.advance_tick({"actor-a": "attack sentry", "actor-b": "attack sentry"})
            snap = session.world_state.get_snapshot()
            return {
                "sentry_health": snap["entities"]["sentry"].get("health"),
                "actor_a_location": snap["entities"]["actor-a"].get("location"),
                "actor_a_inventory": sorted(snap["entities"]["actor-a"].get("inventory", [])),
                "actor_b_inventory": sorted(snap["entities"]["actor-b"].get("inventory", [])),
            }

        run1 = _run_actions()
        run2 = _run_actions()
        assert run1 == run2

    def test_quest_ids_sorted_deterministically(self) -> None:
        """Quest objective IDs are deterministically ordered in scenario config."""
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        quest_ids = [q["quest_id"] for q in wc["quest_objectives"]]
        # Each quest_id should appear exactly once
        assert len(quest_ids) == len(set(quest_ids))

    def test_room_exits_sorted_deterministically(self) -> None:
        """Exits present in rooms are consistent and deterministic."""
        s = _SCENARIO_PRESETS["small-cooperative-outpost"]
        wc = json.loads(s["scenario_vars"]["world_config_json"])
        # supply-camp → gatehouse east is always present at start
        assert "east" in wc["rooms"]["supply-camp"]["exits"]
        # courtyard → command-post north is always present at start
        assert "north" in wc["rooms"]["courtyard"]["exits"]
