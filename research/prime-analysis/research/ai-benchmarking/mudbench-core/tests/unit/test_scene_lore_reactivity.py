"""Unit tests for state-reactive scene/lore description layer.

task_id: shared_world_scene_and_lore_depth_v1

Tests cover:
- _resolve_reactive_description: base description returned when no overrides
- _resolve_reactive_description: scenario_var_truthy condition matches and overrides
- _resolve_reactive_description: scenario_var_truthy condition skipped when falsy
- _resolve_reactive_description: entity_absent condition matches when entity gone
- _resolve_reactive_description: entity_absent condition skipped when entity present
- _resolve_reactive_description: first matching override wins
- _resolve_reactive_description: room_id mismatch skipped
- _resolve_reactive_description: invalid JSON falls back to base
- _resolve_reactive_description: malformed entry skipped
- _build_room_description_overrides_scenario_delta: empty when no world_config
- _build_room_description_overrides_scenario_delta: skips invalid condition types
- _build_room_description_overrides_scenario_delta: builds valid delta
- _build_room_description_overrides_scenario_delta: output deterministic
- Reactive description persists across save/load
"""
from __future__ import annotations

import json
from typing import Any


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_scenario_vars(overrides: list[dict], **extra: Any) -> dict:
    svars: dict[str, Any] = {
        "room_description_overrides_json": json.dumps(overrides, sort_keys=True)
    }
    svars.update(extra)
    return svars


def _make_rooms(room_id: str = "room-a", entities_present: list[str] | None = None) -> dict:
    return {
        room_id: {
            "title": "Room A",
            "description": "Base description.",
            "exits": {},
            "entities": [],
            "entities_present": entities_present or [],
        }
    }


def _call(
    *,
    room_id: str = "room-a",
    base_description: str = "Base description.",
    rooms: dict | None = None,
    entities: dict | None = None,
    scenario_vars: dict | None = None,
) -> str:
    from agents.gateway.observation_builder import _resolve_reactive_description  # type: ignore
    return _resolve_reactive_description(
        room_id=room_id,
        base_description=base_description,
        rooms=rooms if rooms is not None else _make_rooms(room_id),
        entities=entities if entities is not None else {},
        scenario_vars=scenario_vars if scenario_vars is not None else {},
    )


# ---------------------------------------------------------------------------
# _resolve_reactive_description — no override JSON
# ---------------------------------------------------------------------------

class TestReactiveDescriptionNoOverrides:
    def test_returns_base_when_no_json_key(self):
        assert _call(scenario_vars={}) == "Base description."

    def test_returns_base_when_json_is_none(self):
        assert _call(scenario_vars={"room_description_overrides_json": None}) == "Base description."

    def test_returns_base_when_json_malformed(self):
        assert _call(scenario_vars={"room_description_overrides_json": "bad"}) == "Base description."

    def test_returns_base_when_overrides_list_empty(self):
        svars = _make_scenario_vars([])
        assert _call(scenario_vars=svars) == "Base description."


# ---------------------------------------------------------------------------
# _resolve_reactive_description — scenario_var_truthy
# ---------------------------------------------------------------------------

class TestScenarioVarTruthyCondition:
    def test_overrides_when_key_is_true(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "scenario_var_truthy",
            "condition_key": "defeated.guardian",
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides, **{"defeated.guardian": True})
        assert _call(scenario_vars=svars) == "Override description."

    def test_returns_base_when_key_is_missing(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "scenario_var_truthy",
            "condition_key": "defeated.guardian",
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides)
        assert _call(scenario_vars=svars) == "Base description."

    def test_returns_base_when_key_is_none(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "scenario_var_truthy",
            "condition_key": "defeated.guardian",
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides, **{"defeated.guardian": None})
        assert _call(scenario_vars=svars) == "Base description."

    def test_returns_base_when_key_is_false(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "scenario_var_truthy",
            "condition_key": "defeated.guardian",
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides, **{"defeated.guardian": False})
        assert _call(scenario_vars=svars) == "Base description."

    def test_skips_entry_when_room_id_mismatch(self):
        overrides = [{
            "room_id": "other-room",
            "condition_type": "scenario_var_truthy",
            "condition_key": "defeated.guardian",
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides, **{"defeated.guardian": True})
        assert _call(room_id="room-a", scenario_vars=svars) == "Base description."

    def test_skips_entry_with_missing_condition_key_field(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "scenario_var_truthy",
            # condition_key missing
            "description": "Override description.",
        }]
        svars = _make_scenario_vars(overrides)
        assert _call(scenario_vars=svars) == "Base description."


# ---------------------------------------------------------------------------
# _resolve_reactive_description — entity_absent
# ---------------------------------------------------------------------------

class TestEntityAbsentCondition:
    def test_overrides_when_entity_not_in_room(self):
        overrides = [{
            "room_id": "vault",
            "condition_type": "entity_absent",
            "condition_room_id": "vault",
            "condition_entity_id": "relic",
            "description": "The vault is empty.",
        }]
        # relic NOT in entities_present → absent
        rooms = _make_rooms("vault", entities_present=[])
        svars = _make_scenario_vars(overrides)
        assert _call(room_id="vault", rooms=rooms, scenario_vars=svars) == "The vault is empty."

    def test_returns_base_when_entity_still_present(self):
        overrides = [{
            "room_id": "vault",
            "condition_type": "entity_absent",
            "condition_room_id": "vault",
            "condition_entity_id": "relic",
            "description": "The vault is empty.",
        }]
        rooms = _make_rooms("vault", entities_present=["relic"])
        svars = _make_scenario_vars(overrides)
        assert _call(room_id="vault", rooms=rooms, scenario_vars=svars) == "Base description."

    def test_skips_entry_with_missing_condition_entity_id(self):
        overrides = [{
            "room_id": "vault",
            "condition_type": "entity_absent",
            "condition_room_id": "vault",
            # condition_entity_id missing
            "description": "Override.",
        }]
        rooms = _make_rooms("vault", entities_present=[])
        svars = _make_scenario_vars(overrides)
        assert _call(room_id="vault", rooms=rooms, scenario_vars=svars) == "Base description."


# ---------------------------------------------------------------------------
# _resolve_reactive_description — ordering and precedence
# ---------------------------------------------------------------------------

class TestReactiveDescriptionOrdering:
    def test_first_matching_override_wins(self):
        overrides = [
            {
                "room_id": "room-a",
                "condition_type": "scenario_var_truthy",
                "condition_key": "key1",
                "description": "First override.",
            },
            {
                "room_id": "room-a",
                "condition_type": "scenario_var_truthy",
                "condition_key": "key2",
                "description": "Second override.",
            },
        ]
        svars = _make_scenario_vars(overrides, key1=True, key2=True)
        assert _call(scenario_vars=svars) == "First override."

    def test_falls_through_to_matching_when_first_does_not_match(self):
        overrides = [
            {
                "room_id": "room-a",
                "condition_type": "scenario_var_truthy",
                "condition_key": "key1",
                "description": "First override.",
            },
            {
                "room_id": "room-a",
                "condition_type": "scenario_var_truthy",
                "condition_key": "key2",
                "description": "Second override.",
            },
        ]
        svars = _make_scenario_vars(overrides, key2=True)  # only key2 truthy
        assert _call(scenario_vars=svars) == "Second override."

    def test_unknown_condition_type_skipped(self):
        overrides = [{
            "room_id": "room-a",
            "condition_type": "unknown_type",
            "condition_key": "some_key",
            "description": "Should not appear.",
        }]
        svars = _make_scenario_vars(overrides, some_key=True)
        assert _call(scenario_vars=svars) == "Base description."


# ---------------------------------------------------------------------------
# _build_room_description_overrides_scenario_delta
# ---------------------------------------------------------------------------

class TestBuildRoomDescriptionOverridesDelta:
    def _call(self, world_config: Any) -> dict:
        from evaluation.benchmark_runner.runner import _build_room_description_overrides_scenario_delta  # type: ignore
        return _build_room_description_overrides_scenario_delta(world_config)

    def test_returns_empty_for_none(self):
        assert self._call(None) == {}

    def test_returns_empty_when_no_overrides_key(self):
        assert self._call({"rooms": {}}) == {}

    def test_returns_empty_for_empty_list(self):
        assert self._call({"room_description_overrides": []}) == {}

    def test_skips_invalid_condition_type(self):
        result = self._call({"room_description_overrides": [
            {"room_id": "r", "condition_type": "bad_type", "description": "D",
             "condition_key": "k"}
        ]})
        assert result == {}

    def test_skips_missing_room_id(self):
        result = self._call({"room_description_overrides": [
            {"condition_type": "scenario_var_truthy", "condition_key": "k", "description": "D"}
        ]})
        assert result == {}

    def test_skips_missing_description(self):
        result = self._call({"room_description_overrides": [
            {"room_id": "r", "condition_type": "scenario_var_truthy", "condition_key": "k"}
        ]})
        assert result == {}

    def test_builds_valid_scenario_var_truthy_entry(self):
        result = self._call({"room_description_overrides": [
            {"room_id": "vault-entrance", "condition_type": "scenario_var_truthy",
             "condition_key": "defeated.guardian", "description": "Override text."}
        ]})
        assert "room_description_overrides_json" in result
        parsed = json.loads(result["room_description_overrides_json"])
        assert len(parsed) == 1
        assert parsed[0]["room_id"] == "vault-entrance"
        assert parsed[0]["condition_key"] == "defeated.guardian"

    def test_builds_valid_entity_absent_entry(self):
        result = self._call({"room_description_overrides": [
            {"room_id": "vault", "condition_type": "entity_absent",
             "condition_room_id": "vault", "condition_entity_id": "relic",
             "description": "Empty vault."}
        ]})
        assert "room_description_overrides_json" in result
        parsed = json.loads(result["room_description_overrides_json"])
        assert parsed[0]["condition_entity_id"] == "relic"

    def test_output_is_deterministic(self):
        wc = {"room_description_overrides": [
            {"room_id": "r", "condition_type": "scenario_var_truthy",
             "condition_key": "k", "description": "D"}
        ]}
        r1 = self._call(wc)
        r2 = self._call(wc)
        assert r1 == r2


# ---------------------------------------------------------------------------
# Reactive description persists through save/load
# ---------------------------------------------------------------------------

class TestReactiveDescriptionPersistence:
    def test_overrides_json_survives_save_load(self, tmp_path):
        from world.state.world_state import DeterministicWorldStateManager
        from world.state.world_persistence import save_world_slot, load_world_slot

        overrides = [
            {"room_id": "vault-entrance", "condition_type": "scenario_var_truthy",
             "condition_key": "defeated.guardian", "description": "Post-battle."}
        ]
        world = DeterministicWorldStateManager.from_dict({
            "rooms": {
                "vault-entrance": {
                    "title": "Entrance", "description": "Base.", "exits": {},
                    "entities": [], "entities_present": []
                }
            },
            "entities": {},
            "scenario_vars": {
                "room_description_overrides_json": json.dumps(overrides),
            },
        })

        save_world_slot("desc-slot", str(tmp_path), world, scenario_id="test", run_id="r1")
        loaded = load_world_slot("desc-slot", str(tmp_path))
        assert loaded.accepted

        svars = loaded.world_state.get_snapshot()["scenario_vars"]
        assert "room_description_overrides_json" in svars

    def test_reactive_description_correct_after_reload(self, tmp_path):
        from world.state.world_state import DeterministicWorldStateManager
        from world.state.world_persistence import save_world_slot, load_world_slot
        from agents.gateway.observation_builder import _resolve_reactive_description

        overrides = [
            {"room_id": "vault-entrance", "condition_type": "scenario_var_truthy",
             "condition_key": "defeated.guardian", "description": "Post-battle override."}
        ]
        rooms = {"vault-entrance": {
            "title": "Entrance", "description": "Base.", "exits": {},
            "entities": [], "entities_present": []
        }}
        world = DeterministicWorldStateManager.from_dict({
            "rooms": rooms,
            "entities": {},
            "scenario_vars": {
                "room_description_overrides_json": json.dumps(overrides),
                "defeated.guardian": True,
            },
        })

        save_world_slot("desc-reload-slot", str(tmp_path), world,
                        scenario_id="test", run_id="r1")
        loaded = load_world_slot("desc-reload-slot", str(tmp_path))
        assert loaded.accepted

        snap = loaded.world_state.get_snapshot()
        result = _resolve_reactive_description(
            room_id="vault-entrance",
            base_description="Base.",
            rooms=snap["rooms"],
            entities=snap["entities"],
            scenario_vars=snap["scenario_vars"],
        )
        assert result == "Post-battle override."
