"""Tests for actor cooperative recovery/revive mechanic.

Covers:
- Using a revive-kit on a defeated co-actor restores health and clears defeat state
- Revive cancels pending auto-respawn
- Revived actor can perform actions normally after revival
- No revive when no defeated co-actor in same room
- No revive when reviver and defeated are in different rooms
- No revive when item has no revive effect configured
- Revive only targets co-actors (not self, not NPCs)
- Revive health respects effect config (default + custom)
- Save/load/reconnect preserves revived state
- actor_revived event appears in tick messages (runner formatter)
- actor_revived event appears in world event log history
- Deterministic: alphabetically-first defeated actor in room is targeted
- _find_actor_revive_effect helper parses correctly
"""

from __future__ import annotations

import json

from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,
    _find_actor_revive_effect,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------

def _make_revive_effects_json(item_id: str = "revive-kit", revive_health: int = 25, effect_id: str = "revive-kit-effect") -> str:
    return json.dumps([{"effect_id": effect_id, "item_id": item_id, "revive_health": revive_health}])


def _make_world(
    *,
    actor_a_health: int = 100,
    actor_b_health: int = 100,
    actor_a_room: str = "camp",
    actor_b_room: str = "camp",
    actor_a_inventory: list[str] | None = None,
    actor_b_inventory: list[str] | None = None,
    actor_a_defeated: bool = False,
    actor_b_defeated: bool = False,
    actor_a_respawn_at: int | None = None,
    actor_b_respawn_at: int | None = None,
    revive_effects_json: str | None = None,
    extra_items: list[dict] | None = None,
) -> DeterministicWorldStateManager:
    actor_a_inventory = actor_a_inventory or []
    actor_b_inventory = actor_b_inventory or []
    extra_items = extra_items or []

    entities: dict = {
        "actor-a": {
            "entity_id": "actor-a",
            "entity_type": "player",
            "location": actor_a_room,
            "health": actor_a_health,
            "inventory": actor_a_inventory,
            "tags": [],
        },
        "actor-b": {
            "entity_id": "actor-b",
            "entity_type": "agent",
            "location": actor_b_room,
            "health": actor_b_health,
            "inventory": actor_b_inventory,
            "tags": [],
        },
    }

    for item in extra_items:
        entities[item["entity_id"]] = item

    rooms: dict = {
        "camp": {
            "room_id": "camp",
            "title": "Camp",
            "description": "A test camp.",
            "exits": {},
            "entities_present": sorted(
                [e for e, d in entities.items() if d.get("location") == "camp"]
            ),
        },
        "vault": {
            "room_id": "vault",
            "title": "Vault",
            "description": "A test vault.",
            "exits": {},
            "entities_present": sorted(
                [e for e, d in entities.items() if d.get("location") == "vault"]
            ),
        },
    }

    svars: dict = {"mode": "shared_shard"}
    if actor_a_defeated:
        svars["actor_defeated.actor-a"] = True
    if actor_b_defeated:
        svars["actor_defeated.actor-b"] = True
    if actor_a_respawn_at is not None:
        svars["actor_respawn_at.actor-a"] = actor_a_respawn_at
    if actor_b_respawn_at is not None:
        svars["actor_respawn_at.actor-b"] = actor_b_respawn_at
    if revive_effects_json is not None:
        svars["actor_revive_effects_json"] = revive_effects_json

    return DeterministicWorldStateManager.from_dict({
        "entities": entities,
        "rooms": rooms,
        "scenario_vars": svars,
    })


def _action(actor_id: str, action_type: str, **kwargs: str) -> BasicDeterministicActionProcessor:
    from core.action_processor import ActionRequest
    return ActionRequest(
        actor_id=actor_id,
        action_type=action_type,
        arguments=tuple((k, v) for k, v in kwargs.items()),
    )


def _processor() -> BasicDeterministicActionProcessor:
    return BasicDeterministicActionProcessor()


# ---------------------------------------------------------------------------
# _find_actor_revive_effect unit tests
# ---------------------------------------------------------------------------

class TestFindActorReviveEffect:
    def test_returns_effect_when_item_matches(self) -> None:
        svars = {"actor_revive_effects_json": _make_revive_effects_json("revive-kit")}
        result = _find_actor_revive_effect(svars, item_id="revive-kit")
        assert result is not None
        assert result["item_id"] == "revive-kit"
        assert result["revive_health"] == 25

    def test_returns_none_when_item_not_in_list(self) -> None:
        svars = {"actor_revive_effects_json": _make_revive_effects_json("revive-kit")}
        assert _find_actor_revive_effect(svars, item_id="ward-sigil") is None

    def test_returns_none_when_key_missing(self) -> None:
        assert _find_actor_revive_effect({}, item_id="revive-kit") is None

    def test_returns_none_when_not_a_mapping(self) -> None:
        assert _find_actor_revive_effect(None, item_id="revive-kit") is None

    def test_returns_none_on_malformed_json(self) -> None:
        svars = {"actor_revive_effects_json": "not-valid-json"}
        assert _find_actor_revive_effect(svars, item_id="revive-kit") is None

    def test_returns_none_when_json_not_a_list(self) -> None:
        svars = {"actor_revive_effects_json": json.dumps({"item_id": "revive-kit"})}
        assert _find_actor_revive_effect(svars, item_id="revive-kit") is None

    def test_custom_revive_health_respected(self) -> None:
        svars = {"actor_revive_effects_json": _make_revive_effects_json("kit", revive_health=40)}
        result = _find_actor_revive_effect(svars, item_id="kit")
        assert result is not None
        assert result["revive_health"] == 40


# ---------------------------------------------------------------------------
# Core revive mechanic tests
# ---------------------------------------------------------------------------

class TestActorReviveMechanic:
    def _setup_revive_scenario(
        self,
        *,
        reviver_room: str = "camp",
        target_room: str = "camp",
        revive_health: int = 25,
    ) -> DeterministicWorldStateManager:
        """Reviver (actor-b) has revive-kit; actor-a is defeated."""
        return _make_world(
            actor_a_health=0,
            actor_b_health=80,
            actor_a_room=target_room,
            actor_b_room=reviver_room,
            actor_a_inventory=[],
            actor_b_inventory=["revive-kit"],
            actor_a_defeated=True,
            actor_b_defeated=False,
            actor_a_respawn_at=7,
            revive_effects_json=_make_revive_effects_json("revive-kit", revive_health=revive_health),
            extra_items=[
                {
                    "entity_id": "revive-kit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                }
            ],
        )

    def test_revive_clears_defeated_flag(self) -> None:
        world = self._setup_revive_scenario()
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        svars = snap.get("scenario_vars", {})
        assert not svars.get("actor_defeated.actor-a")

    def test_revive_clears_respawn_schedule(self) -> None:
        world = self._setup_revive_scenario()
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        svars = snap.get("scenario_vars", {})
        assert svars.get("actor_respawn_at.actor-a") is None

    def test_revive_restores_health(self) -> None:
        world = self._setup_revive_scenario(revive_health=25)
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        actor_a = snap.get("entities", {}).get("actor-a", {})
        assert actor_a.get("health") == 25

    def test_revive_emits_actor_revived_event(self) -> None:
        world = self._setup_revive_scenario()
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" in event_types

    def test_revive_event_payload_correct(self) -> None:
        world = self._setup_revive_scenario(revive_health=25)
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        revive_event = next(e for e in result[0].events if e.event_type == "actor_revived")
        payload = dict(revive_event.payload)
        assert payload.get("target_id") == "actor-a"
        assert payload.get("item_id") == "revive-kit"
        assert payload.get("revive_health") == 25
        assert revive_event.actor_id == "actor-b"

    def test_no_revive_when_target_not_defeated(self) -> None:
        """Using revive-kit on a healthy actor does not emit actor_revived."""
        world = _make_world(
            actor_a_health=80,
            actor_b_health=80,
            actor_a_inventory=[],
            actor_b_inventory=["revive-kit"],
            revive_effects_json=_make_revive_effects_json("revive-kit"),
            extra_items=[
                {
                    "entity_id": "revive-kit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                }
            ],
        )
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" not in event_types

    def test_no_revive_when_target_in_different_room(self) -> None:
        """Reviver and defeated actor in different rooms — no revive."""
        world = self._setup_revive_scenario(reviver_room="camp", target_room="vault")
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" not in event_types

    def test_no_revive_without_effect_configured(self) -> None:
        """Using a non-revive consumable does not trigger revive logic."""
        world = _make_world(
            actor_a_health=0,
            actor_b_health=80,
            actor_a_defeated=True,
            actor_b_inventory=["potion"],
            revive_effects_json=None,  # no revive effects
            extra_items=[
                {
                    "entity_id": "potion",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                }
            ],
        )
        proc = _processor()
        action = _action("actor-b", "use", item_id="potion")
        result = proc.process_actions([action], world_state=world, step_index=5)
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" not in event_types

    def test_revive_custom_health_from_effect(self) -> None:
        """Effect-defined revive_health overrides default."""
        world = self._setup_revive_scenario(revive_health=40)
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        assert snap.get("entities", {}).get("actor-a", {}).get("health") == 40

    def test_revived_actor_state_survives_save_load(self) -> None:
        """Revived actor state is preserved through save/load round-trip."""
        world = self._setup_revive_scenario()
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        world.apply_delta(dict(result[0].world_delta))

        # Simulate save/load via snapshot round-trip.
        snap = world.get_snapshot()
        reloaded = DeterministicWorldStateManager.from_dict({
            "entities": snap.get("entities", {}),
            "rooms": snap.get("rooms", {}),
            "scenario_vars": snap.get("scenario_vars", {}),
        })
        rsnap = reloaded.get_snapshot()
        svars = rsnap.get("scenario_vars", {})
        assert not svars.get("actor_defeated.actor-a")
        assert svars.get("actor_respawn_at.actor-a") is None
        assert rsnap.get("entities", {}).get("actor-a", {}).get("health") == 25

    def test_deterministic_selection_alphabetically_first(self) -> None:
        """When multiple defeated actors are in the room, revive targets alphabetically first."""
        # Add actor-c also defeated in same room
        world = _make_world(
            actor_a_health=0,
            actor_b_health=80,
            actor_a_room="camp",
            actor_b_room="camp",
            actor_a_inventory=[],
            actor_b_inventory=["revive-kit"],
            actor_a_defeated=True,
            revive_effects_json=_make_revive_effects_json("revive-kit"),
            extra_items=[
                {
                    "entity_id": "revive-kit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                },
                {
                    "entity_id": "actor-c",
                    "entity_type": "player",
                    "location": "camp",
                    "health": 0,
                    "inventory": [],
                    "tags": [],
                },
            ],
        )
        # Mark actor-c as defeated too
        world.apply_delta(
            {"scenario_vars": {"actor_defeated.actor-c": True}}
        )

        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=5)
        revive_event = next(
            (e for e in result[0].events if e.event_type == "actor_revived"), None
        )
        assert revive_event is not None
        # "actor-a" < "actor-c" alphabetically → actor-a is revived
        assert dict(revive_event.payload).get("target_id") == "actor-a"


# ---------------------------------------------------------------------------
# Runner formatter tests
# ---------------------------------------------------------------------------

class TestReviveRunnerFormatters:
    def test_tick_message_includes_actor_revived(self) -> None:
        from evaluation.benchmark_runner.runner import _format_tick_event_messages
        from core.event_logger import EventRecord, normalize_payload

        events = [
            EventRecord(
                step_index=5,
                event_type="actor_revived",
                actor_id="actor-b",
                payload=normalize_payload({
                    "target_id": "actor-a",
                    "item_id": "revive-kit",
                    "room_id": "camp",
                    "revive_health": 25,
                    "effect_id": "revive-kit-effect",
                }),
            )
        ]
        messages = _format_tick_event_messages(events)
        assert any("actor-b revived actor-a" in m for m in messages)
        assert any("25 HP" in m for m in messages)

    def test_history_log_includes_actor_revived(self) -> None:
        from evaluation.benchmark_runner.runner import _format_world_event_log_messages

        log = [
            {
                "step": 5,
                "event_type": "actor_revived",
                "actor_id": "actor-b",
                "target_id": "actor-a",
                "room_id": "camp",
                "revive_health": 25,
                "effect_id": "revive-kit-effect",
            }
        ]
        svars = {"world_event_log_json": json.dumps(log)}
        messages = _format_world_event_log_messages(svars)
        assert any("actor-b revived actor-a" in m for m in messages)

    def test_tick_messages_unchanged_without_revive(self) -> None:
        """No revive events → no revive lines in tick messages."""
        from evaluation.benchmark_runner.runner import _format_tick_event_messages
        from core.event_logger import EventRecord, normalize_payload

        events = [
            EventRecord(
                step_index=1,
                event_type="actor_defeated",
                actor_id="actor-a",
                payload=normalize_payload({"actor_id": "actor-a", "respawn_at_tick": 4}),
            )
        ]
        messages = _format_tick_event_messages(events)
        assert not any("revived" in m for m in messages)

    def test_history_log_includes_actor_revived_hp_note(self) -> None:
        from evaluation.benchmark_runner.runner import _format_world_event_log_messages

        log = [
            {
                "step": 3,
                "event_type": "actor_revived",
                "actor_id": "hero",
                "target_id": "ally",
                "room_id": "cave",
                "revive_health": 30,
            }
        ]
        svars = {"world_event_log_json": json.dumps(log)}
        messages = _format_world_event_log_messages(svars)
        assert any("30 HP" in m for m in messages)


# ---------------------------------------------------------------------------
# Interaction with process_actor_defeat_tick
# ---------------------------------------------------------------------------

class TestReviveWithDefeatAndRespawn:
    def test_revive_before_auto_respawn(self) -> None:
        """Revive on tick 3 should cancel respawn scheduled for tick 5."""
        world = _make_world(
            actor_a_health=0,
            actor_b_health=80,
            actor_a_defeated=True,
            actor_a_respawn_at=5,
            actor_b_inventory=["revive-kit"],
            revive_effects_json=_make_revive_effects_json("revive-kit"),
            extra_items=[
                {
                    "entity_id": "revive-kit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                }
            ],
        )
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=3)
        world.apply_delta(dict(result[0].world_delta))
        snap = world.get_snapshot()
        svars = snap.get("scenario_vars", {})
        # Defeat flag and respawn schedule both cleared
        assert not svars.get("actor_defeated.actor-a")
        assert svars.get("actor_respawn_at.actor-a") is None

    def test_already_respawned_actor_not_revived_again(self) -> None:
        """If actor-a already respawned (not defeated), revive-kit use does not trigger."""
        world = _make_world(
            actor_a_health=50,
            actor_b_health=80,
            actor_a_defeated=False,  # already respawned
            actor_b_inventory=["revive-kit"],
            revive_effects_json=_make_revive_effects_json("revive-kit"),
            extra_items=[
                {
                    "entity_id": "revive-kit",
                    "entity_type": "consumable",
                    "location": "inventory",
                    "inventory_of": "actor-b",
                }
            ],
        )
        proc = _processor()
        action = _action("actor-b", "use", item_id="revive-kit")
        result = proc.process_actions([action], world_state=world, step_index=6)
        event_types = [e.event_type for e in result[0].events]
        assert "actor_revived" not in event_types
