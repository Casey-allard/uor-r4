"""Unit tests for persistent NPC respawn/reset mechanic.

task_id: persistent_npc_respawn_and_world_reset_v1

Tests cover:
- _find_npc_respawn_rule helper returns correct rule or None
- process_npc_respawn_tick fires at exactly the scheduled tick
- process_npc_respawn_tick restores NPC health, tags, and location
- defeated.<npc_id> cleared after respawn
- respawn_at_tick.<npc_id> cleared after respawn
- calmed.<effect_id> cleared via clears_calm_effects on respawn
- idempotency (respawn only fires once)
- two NPCs can respawn at different ticks
- respawn state persists through save/load round-trip
- _build_npc_respawn_rules_scenario_delta builder validation
"""
from __future__ import annotations

import json
from typing import Any


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_respawn_world(
    *,
    patrol_health: int = 0,
    patrol_tags: list[str] | None = None,
    patrol_location: str = "vault-entrance",
    patrol_defeated: bool = True,
    patrol_respawn_at: int | None = None,
    patrol_calmed: bool = False,
    guardian_health: int = 0,
    guardian_defeated: bool = False,
    guardian_respawn_at: int | None = None,
    npc_respawn_rules: list[dict[str, Any]] | None = None,
) -> Any:
    """Build a minimal world with respawn rules for unit tests."""
    from world.state.world_state import DeterministicWorldStateManager

    rooms: dict[str, Any] = {
        "vault-entrance": {
            "title": "Vault Entrance",
            "description": "Entrance.",
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
        "patrol": {
            "entity_id": "patrol",
            "entity_type": "npc",
            "health": patrol_health,
            "location": patrol_location,
            "tags": list(patrol_tags or []),
            "inventory": [],
        },
        "guardian": {
            "entity_id": "guardian",
            "entity_type": "npc",
            "health": guardian_health,
            "location": "vault-entrance",
            "tags": [],
            "inventory": [],
        },
    }

    # Wire entities_present (only living NPCs appear in room list)
    ve_present = []
    if patrol_health > 0 and patrol_location == "vault-entrance":
        ve_present.append("patrol")
    if guardian_health > 0:
        ve_present.append("guardian")
    rooms["vault-entrance"]["entities_present"] = sorted(ve_present)

    scenario_vars: dict[str, Any] = {}

    if npc_respawn_rules is None:
        npc_respawn_rules = [
            {
                "npc_id": "patrol",
                "respawn_delay_ticks": 3,
                "respawn_health": 15,
                "respawn_room_id": "vault-entrance",
                "respawn_tags": ["hostile"],
                "clears_calm_effects": ["patrol-calmed"],
            },
            {
                "npc_id": "guardian",
                "respawn_delay_ticks": 5,
                "respawn_health": 20,
                "respawn_room_id": "vault-entrance",
                "respawn_tags": [],
                "clears_calm_effects": [],
            },
        ]
    scenario_vars["npc_respawn_rules_json"] = json.dumps(npc_respawn_rules, sort_keys=True)

    if patrol_defeated:
        scenario_vars["defeated.patrol"] = True
    if patrol_respawn_at is not None:
        scenario_vars["respawn_at_tick.patrol"] = patrol_respawn_at
    if guardian_defeated:
        scenario_vars["defeated.guardian"] = True
    if guardian_respawn_at is not None:
        scenario_vars["respawn_at_tick.guardian"] = guardian_respawn_at
    if patrol_calmed:
        scenario_vars["calmed.patrol-calmed"] = True

    world = DeterministicWorldStateManager.from_dict(
        {"rooms": rooms, "entities": entities, "scenario_vars": scenario_vars}
    )
    return world


def _snap_vars(world: Any) -> dict[str, Any]:
    return dict(world.get_snapshot().get("scenario_vars", {}))


def _snap_entity(world: Any, entity_id: str) -> dict[str, Any]:
    return dict(world.get_snapshot()["entities"].get(entity_id, {}))


def _snap_room_present(world: Any, room_id: str) -> list[str]:
    return list(world.get_snapshot()["rooms"].get(room_id, {}).get("entities_present", []))


# ---------------------------------------------------------------------------
# _find_npc_respawn_rule helper
# ---------------------------------------------------------------------------

class TestFindNpcRespawnRule:
    def test_returns_none_when_no_rules_json(self):
        from world.state.basic_action_processor import _find_npc_respawn_rule  # type: ignore[attr-defined]
        assert _find_npc_respawn_rule({}, "patrol") is None

    def test_returns_none_when_rules_json_none(self):
        from world.state.basic_action_processor import _find_npc_respawn_rule  # type: ignore[attr-defined]
        assert _find_npc_respawn_rule({"npc_respawn_rules_json": None}, "patrol") is None

    def test_returns_none_when_npc_not_in_rules(self):
        from world.state.basic_action_processor import _find_npc_respawn_rule  # type: ignore[attr-defined]
        rules = [{"npc_id": "guardian", "respawn_delay_ticks": 5, "respawn_health": 20, "respawn_room_id": "ve"}]
        svars = {"npc_respawn_rules_json": json.dumps(rules)}
        assert _find_npc_respawn_rule(svars, "patrol") is None

    def test_returns_correct_rule(self):
        from world.state.basic_action_processor import _find_npc_respawn_rule  # type: ignore[attr-defined]
        rules = [
            {"npc_id": "patrol", "respawn_delay_ticks": 3, "respawn_health": 15, "respawn_room_id": "vault-entrance"},
            {"npc_id": "guardian", "respawn_delay_ticks": 5, "respawn_health": 20, "respawn_room_id": "vault-entrance"},
        ]
        svars = {"npc_respawn_rules_json": json.dumps(rules)}
        rule = _find_npc_respawn_rule(svars, "patrol")
        assert rule is not None
        assert rule["npc_id"] == "patrol"
        assert rule["respawn_delay_ticks"] == 3
        assert rule["respawn_health"] == 15

    def test_returns_none_for_malformed_json(self):
        from world.state.basic_action_processor import _find_npc_respawn_rule  # type: ignore[attr-defined]
        assert _find_npc_respawn_rule({"npc_respawn_rules_json": "bad json"}, "patrol") is None


# ---------------------------------------------------------------------------
# process_npc_respawn_tick — timing
# ---------------------------------------------------------------------------

class TestProcessNpcRespawnTick:
    def test_no_respawn_when_no_respawn_at_tick(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=None)
        result = process_npc_respawn_tick(world, step_index=10)
        assert result is None

    def test_no_respawn_before_scheduled_tick(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=4)
        assert result is None

    def test_respawn_fires_at_exact_tick(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        assert result.accepted

    def test_respawn_fires_at_later_tick(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=99)
        assert result is not None
        assert result.accepted

    def test_respawn_event_type_is_npc_respawn(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        events = [e for e in result.events if e.event_type == "npc_respawn"]
        assert len(events) == 1
        assert events[0].actor_id is None


# ---------------------------------------------------------------------------
# process_npc_respawn_tick — world state changes
# ---------------------------------------------------------------------------

class TestProcessNpcRespawnWorldState:
    def _apply(self, world: Any, result: Any) -> None:
        world.apply_delta(dict(result.world_delta))

    def test_npc_health_restored(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_health=0, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        entity = _snap_entity(world, "patrol")
        assert entity["health"] == 15

    def test_npc_tags_restored_to_hostile(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_tags=[], patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        entity = _snap_entity(world, "patrol")
        assert "hostile" in entity["tags"]

    def test_npc_location_set_to_respawn_room(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_location="camp", patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        entity = _snap_entity(world, "patrol")
        assert entity["location"] == "vault-entrance"

    def test_npc_added_to_room_entities_present(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_health=0, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        assert "patrol" in _snap_room_present(world, "vault-entrance")

    def test_defeated_flag_cleared_after_respawn(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        svars = _snap_vars(world)
        assert not bool(svars.get("defeated.patrol"))

    def test_respawn_at_tick_cleared_after_respawn(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        svars = _snap_vars(world)
        assert not isinstance(svars.get("respawn_at_tick.patrol"), int)

    def test_calm_effect_cleared_on_respawn(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(
            patrol_defeated=False, patrol_health=0, patrol_respawn_at=5, patrol_calmed=True
        )
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        svars = _snap_vars(world)
        assert not bool(svars.get("calmed.patrol-calmed"))

    def test_respawn_is_idempotent(self):
        """After applying the respawn delta, a second call at the same tick is a no-op."""
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result1 = process_npc_respawn_tick(world, step_index=5)
        assert result1 is not None
        world.apply_delta(dict(result1.world_delta))
        result2 = process_npc_respawn_tick(world, step_index=5)
        assert result2 is None


# ---------------------------------------------------------------------------
# Two NPCs with different respawn timings
# ---------------------------------------------------------------------------

class TestTwoNpcRespawnTiming:
    def _apply(self, world: Any, result: Any) -> None:
        world.apply_delta(dict(result.world_delta))

    def test_patrol_respawns_independently_of_guardian(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(
            patrol_defeated=True, patrol_respawn_at=3,
            guardian_defeated=True, guardian_respawn_at=6,
        )
        # At tick=3 only patrol should respawn
        result = process_npc_respawn_tick(world, step_index=3)
        assert result is not None
        self._apply(world, result)

        patrol_health = _snap_entity(world, "patrol")["health"]
        guardian_health = _snap_entity(world, "guardian")["health"]
        assert patrol_health == 15
        assert guardian_health == 0  # not yet respawned

    def test_guardian_respawns_after_patrol(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(
            patrol_defeated=True, patrol_respawn_at=3,
            guardian_defeated=True, guardian_respawn_at=6,
        )
        # Fast-forward: apply patrol respawn at tick=3
        r = process_npc_respawn_tick(world, step_index=3)
        if r:
            self._apply(world, r)
        # At tick=6 guardian should respawn
        result = process_npc_respawn_tick(world, step_index=6)
        assert result is not None
        self._apply(world, result)
        assert _snap_entity(world, "guardian")["health"] == 20

    def test_both_respawn_same_tick_when_scheduled_identically(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(
            patrol_defeated=True, patrol_respawn_at=5,
            guardian_defeated=True, guardian_respawn_at=5,
        )
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        self._apply(world, result)
        assert _snap_entity(world, "patrol")["health"] == 15
        assert _snap_entity(world, "guardian")["health"] == 20


# ---------------------------------------------------------------------------
# Respawn event payload
# ---------------------------------------------------------------------------

class TestRespawnEventPayload:
    def test_event_payload_contains_npc_id_and_room(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        events = [e for e in result.events if e.event_type == "npc_respawn"]
        assert events
        payload = dict(events[0].payload)
        assert payload.get("npc_id") == "patrol"
        assert payload.get("room_id") == "vault-entrance"
        assert payload.get("health") == 15

    def test_event_payload_tags_match_respawn_tags(self):
        from world.state.basic_action_processor import process_npc_respawn_tick
        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        events = [e for e in result.events if e.event_type == "npc_respawn"]
        payload = dict(events[0].payload)
        assert "hostile" in payload.get("tags", [])


# ---------------------------------------------------------------------------
# Save/load round-trip persistence
# ---------------------------------------------------------------------------

class TestRespawnStatePersistence:
    def test_respawn_at_tick_persists_through_save_load(self, tmp_path):
        from world.state.basic_action_processor import process_npc_respawn_tick
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_respawn_world(
            patrol_defeated=True, patrol_respawn_at=7
        )
        save_world_slot("test-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("test-slot", str(tmp_path))
        assert loaded.accepted
        assert loaded.world_state is not None

        # Respawn should NOT have fired yet (step_index=6 < 7)
        result_early = process_npc_respawn_tick(loaded.world_state, step_index=6)
        assert result_early is None

        # At step_index=7 it should fire
        result = process_npc_respawn_tick(loaded.world_state, step_index=7)
        assert result is not None
        assert result.accepted

    def test_respawn_clears_persist_through_save_load(self, tmp_path):
        from world.state.basic_action_processor import process_npc_respawn_tick
        from world.state.world_persistence import save_world_slot, load_world_slot

        world = _make_respawn_world(patrol_defeated=True, patrol_respawn_at=5)
        result = process_npc_respawn_tick(world, step_index=5)
        assert result is not None
        world.apply_delta(dict(result.world_delta))

        save_world_slot("test-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("test-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        svars = snap.get("scenario_vars", {})
        assert not bool(svars.get("defeated.patrol"))
        assert not isinstance(svars.get("respawn_at_tick.patrol"), int)
        assert snap["entities"]["patrol"]["health"] == 15


# ---------------------------------------------------------------------------
# _build_npc_respawn_rules_scenario_delta builder
# ---------------------------------------------------------------------------

class TestBuildNpcRespawnRulesScenarioDelta:
    def test_empty_when_no_world_config(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_respawn_rules_scenario_delta(None) == {}

    def test_empty_when_no_npc_respawn_rules_key(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_respawn_rules_scenario_delta({"rooms": {}}) == {}

    def test_empty_when_rules_list_is_empty(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        assert _build_npc_respawn_rules_scenario_delta({"npc_respawn_rules": []}) == {}

    def test_skips_rule_with_zero_delay(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        result = _build_npc_respawn_rules_scenario_delta({
            "npc_respawn_rules": [
                {"npc_id": "patrol", "respawn_delay_ticks": 0, "respawn_health": 15, "respawn_room_id": "ve"}
            ]
        })
        assert result == {}

    def test_skips_rule_with_zero_health(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        result = _build_npc_respawn_rules_scenario_delta({
            "npc_respawn_rules": [
                {"npc_id": "patrol", "respawn_delay_ticks": 3, "respawn_health": 0, "respawn_room_id": "ve"}
            ]
        })
        assert result == {}

    def test_builds_valid_scenario_delta(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        rules = [
            {
                "npc_id": "patrol",
                "respawn_delay_ticks": 3,
                "respawn_health": 15,
                "respawn_room_id": "vault-entrance",
                "respawn_tags": ["hostile"],
                "clears_calm_effects": ["patrol-calmed"],
            }
        ]
        result = _build_npc_respawn_rules_scenario_delta({"npc_respawn_rules": rules})
        assert "npc_respawn_rules_json" in result
        parsed = json.loads(result["npc_respawn_rules_json"])
        assert len(parsed) == 1
        assert parsed[0]["npc_id"] == "patrol"
        assert parsed[0]["respawn_delay_ticks"] == 3
        assert parsed[0]["clears_calm_effects"] == ["patrol-calmed"]

    def test_output_is_deterministic(self):
        from evaluation.benchmark_runner.runner import _build_npc_respawn_rules_scenario_delta  # type: ignore[attr-defined]
        rules = [
            {"npc_id": "b", "respawn_delay_ticks": 2, "respawn_health": 10, "respawn_room_id": "r"},
            {"npc_id": "a", "respawn_delay_ticks": 1, "respawn_health": 5, "respawn_room_id": "r"},
        ]
        result1 = _build_npc_respawn_rules_scenario_delta({"npc_respawn_rules": rules})
        result2 = _build_npc_respawn_rules_scenario_delta({"npc_respawn_rules": rules})
        assert result1 == result2
