"""Unit tests for persistent shared-world interaction: NPC defeat clears room state."""

from __future__ import annotations

import json
from pathlib import Path

from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_persistence import load_world_snapshot, save_world_snapshot
from world.state.world_state import DeterministicWorldStateManager
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from core.action_processor import ActionRequest


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_combat_world(
    *,
    npc_health: int = 20,
    attacker_location: str = "arena",
    npc_location: str = "arena",
) -> DeterministicWorldStateManager:
    """Build a minimal world with one attacker actor and one NPC in the same room."""
    snapshot = WorldStateSnapshot(
        tick=0,
        entities=(
            WorldEntityState(
                entity_id="fighter",
                entity_type="player",
                location=attacker_location,
                health=100,
                inventory=(),
                tags=(),
            ),
            WorldEntityState(
                entity_id="troll",
                entity_type="npc",
                location=npc_location,
                health=npc_health,
                inventory=(),
                tags=(),
            ),
        ),
        rooms=(
            WorldRoomState(
                room_id="arena",
                description="A combat arena",
                exits=(("north", "exit-hall"),),
                entities_present=("fighter", "troll"),
            ),
            WorldRoomState(
                room_id="exit-hall",
                description="Exit hall",
                exits=(("south", "arena"),),
                entities_present=(),
            ),
        ),
        scenario_vars=(("quest_stage", "start"),),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


def _attack_request(attacker_id: str, target_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=attacker_id,
        action_type="attack",
        arguments=(("target_id", target_id),),
    )


# ---------------------------------------------------------------------------
# NPC partial damage — no cleanup
# ---------------------------------------------------------------------------


def test_attack_partial_damage_does_not_clear_room() -> None:
    """NPC at health > 0 after attack is still in entities_present."""
    ws = _make_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    assert len(results) == 1
    result = results[0]
    assert result.accepted

    ws.apply_delta(dict(result.world_delta))
    snap = ws.get_snapshot()

    assert snap["entities"]["troll"]["health"] == 10
    assert "troll" in snap["rooms"]["arena"]["entities_present"]
    assert snap["scenario_vars"].get("defeated.troll") is None


def test_attack_partial_damage_emits_attack_and_alert_events() -> None:
    """action_attack and npc_alert events are emitted when NPC survives first attack."""
    ws = _make_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    event_types = [e.event_type for e in results[0].events]
    assert "action_attack" in event_types
    assert "npc_alert" in event_types
    assert "npc_defeated" not in event_types


# ---------------------------------------------------------------------------
# NPC defeat — room cleared, marker set, event emitted
# ---------------------------------------------------------------------------


def test_defeat_removes_npc_from_room_entities_present() -> None:
    """When NPC health reaches 0, it is removed from entities_present."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    assert results[0].accepted
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    assert snap["entities"]["troll"]["health"] == 0
    assert "troll" not in snap["rooms"]["arena"]["entities_present"]


def test_defeat_sets_defeated_marker_in_scenario_vars() -> None:
    """Defeating an NPC sets `defeated.{npc_id}: True` in scenario_vars."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    assert snap["scenario_vars"].get("defeated.troll") is True


def test_defeat_emits_npc_defeated_event() -> None:
    """Defeating an NPC emits both action_attack and npc_defeated events."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    event_types = [e.event_type for e in results[0].events]
    assert "action_attack" in event_types
    assert "npc_defeated" in event_types


def test_defeat_npc_defeated_event_contains_target_and_room() -> None:
    """npc_defeated event payload has target_id and room_id fields."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("fighter", "troll")],
        ws,
        step_index=0,
    )
    defeat_events = [e for e in results[0].events if e.event_type == "npc_defeated"]
    assert len(defeat_events) == 1
    payload = dict(defeat_events[0].payload)
    assert payload.get("target_id") == "troll"
    assert payload.get("room_id") == "arena"


def test_multi_hit_defeat_removes_npc_on_final_hit() -> None:
    """NPC with health=20 requires two hits; removal happens on second hit only."""
    ws = _make_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    # First hit: health 20 → 10, still in room
    r1 = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))
    snap1 = ws.get_snapshot()
    assert snap1["entities"]["troll"]["health"] == 10
    assert "troll" in snap1["rooms"]["arena"]["entities_present"]
    assert snap1["scenario_vars"].get("defeated.troll") is None

    # Second hit: health 10 → 0, removed from room, marker set
    r2 = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=1)
    ws.apply_delta(dict(r2[0].world_delta))
    snap2 = ws.get_snapshot()
    assert snap2["entities"]["troll"]["health"] == 0
    assert "troll" not in snap2["rooms"]["arena"]["entities_present"]
    assert snap2["scenario_vars"].get("defeated.troll") is True


def test_attack_on_already_defeated_npc_is_rejected() -> None:
    """Attacking an NPC at health=0 is rejected (target_already_defeated)."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    # Defeat the NPC
    r1 = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    # Try to attack again
    r2 = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=1)
    assert not r2[0].accepted
    reject_event = r2[0].events[0]
    assert reject_event.event_type == "action_rejected"
    assert "defeated" in dict(reject_event.payload).get("reason", "")


def test_attacker_remains_in_room_after_defeat() -> None:
    """The attacker is not removed from entities_present when they defeat an NPC."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    assert "fighter" in snap["rooms"]["arena"]["entities_present"]


# ---------------------------------------------------------------------------
# Defeat state persists across save/load
# ---------------------------------------------------------------------------


def test_defeat_state_persists_across_save_load(tmp_path: Path) -> None:
    """Defeated NPC absent from room after save/load — persistent shared state survives reload."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    dest = str(tmp_path / "combat_snap.json")
    save_result = save_world_snapshot(
        dest, ws, run_id="r1", scenario_id="sc1", actor_ids=["fighter"]
    )
    assert save_result.accepted

    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert snap["entities"]["troll"]["health"] == 0
    assert "troll" not in snap["rooms"]["arena"]["entities_present"]
    assert snap["scenario_vars"].get("defeated.troll") is True


def test_partial_damage_state_persists_across_save_load(tmp_path: Path) -> None:
    """NPC at reduced but positive health is still in room after save/load."""
    ws = _make_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    dest = str(tmp_path / "partial_snap.json")
    save_world_snapshot(dest, ws, run_id="r1", scenario_id="sc1")
    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert snap["entities"]["troll"]["health"] == 10
    assert "troll" in snap["rooms"]["arena"]["entities_present"]
    assert snap["scenario_vars"].get("defeated.troll") is None


def test_defeat_state_persists_across_slot_save_load(tmp_path: Path) -> None:
    """Defeated NPC absent from room after slot save/load cycle."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    save_world_slot("combat-save", tmp_path, ws, run_id="r1", scenario_id="sc1")
    loaded = load_world_slot("combat-save", tmp_path)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "troll" not in snap["rooms"]["arena"]["entities_present"]
    assert snap["scenario_vars"].get("defeated.troll") is True


def test_defeat_scenario_vars_json_is_inspectable(tmp_path: Path) -> None:
    """Saved snapshot has `defeated.troll: true` in the scenario_vars JSON — inspectable."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))

    dest = tmp_path / "inspect.json"
    save_world_snapshot(str(dest), ws, run_id="r1", scenario_id="sc1")
    payload = json.loads(dest.read_text(encoding="utf-8"))
    scenario_vars = payload["world_state"]["scenario_vars"]
    assert scenario_vars.get("defeated.troll") is True
    entities_present = payload["world_state"]["rooms"]["arena"]["entities_present"]
    assert "troll" not in entities_present
    assert "fighter" in entities_present


# ---------------------------------------------------------------------------
# Helpers for access-control tests
# ---------------------------------------------------------------------------


def _make_gated_combat_world(
    *,
    npc_health: int = 20,
    actor_location: str = "vault-entrance",
    second_actor_location: str | None = None,
) -> DeterministicWorldStateManager:
    """Build a world where the vault is gated behind guardian defeat.

    - vault-entrance has NO east exit initially
    - defeating the guardian should unlock east: vault
    - defeat_unlock_effects_json drives that unlock
    """
    defeat_unlock = json.dumps(
        {
            "guardian": {
                "destination_room_id": "vault",
                "direction": "east",
                "effect_id": "guardian-vault-opens",
                "source_room_id": "vault-entrance",
            }
        },
        sort_keys=True,
        separators=(",", ":"),
    )

    entities = [
        WorldEntityState(
            entity_id="hero",
            entity_type="player",
            location=actor_location,
            health=100,
            inventory=(),
            tags=(),
        ),
        WorldEntityState(
            entity_id="guardian",
            entity_type="npc",
            location="vault-entrance",
            health=npc_health,
            inventory=(),
            tags=(),
        ),
    ]
    if second_actor_location is not None:
        entities.append(
            WorldEntityState(
                entity_id="hero2",
                entity_type="player",
                location=second_actor_location,
                health=100,
                inventory=(),
                tags=(),
            )
        )

    snapshot = WorldStateSnapshot(
        tick=0,
        entities=tuple(entities),
        rooms=(
            WorldRoomState(
                room_id="camp",
                description="Camp",
                exits=(("east", "vault-entrance"),),
                entities_present=(),
            ),
            WorldRoomState(
                room_id="vault-entrance",
                description="Vault entrance — east passage sealed until guardian defeated.",
                exits=(("west", "camp"),),  # no east exit yet
                entities_present=(
                    ("hero", "guardian")
                    if second_actor_location is None
                    else ("hero", "guardian", "hero2")
                ),
            ),
            WorldRoomState(
                room_id="vault",
                description="The relic vault.",
                exits=(("west", "vault-entrance"),),
                entities_present=(),
            ),
        ),
        scenario_vars=(
            ("mode", "shared-combat"),
            ("defeat_unlock_effects_json", defeat_unlock),
        ),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


def _move_request(actor_id: str, direction: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="move",
        arguments=(("direction", direction),),
    )


# ---------------------------------------------------------------------------
# Access-control: movement blocked before guardian defeat
# ---------------------------------------------------------------------------


def test_vault_east_blocked_before_guardian_defeat() -> None:
    """Moving east from vault-entrance is rejected when guardian is alive (no exit)."""
    ws = _make_gated_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_move_request("hero", "east")],
        ws,
        step_index=0,
    )
    assert len(results) == 1
    result = results[0]
    assert not result.accepted


def test_vault_east_accessible_after_guardian_defeat() -> None:
    """After defeating the guardian, east exit is added to vault-entrance."""
    ws = _make_gated_combat_world(npc_health=10)  # one hit = defeated
    processor = BasicDeterministicActionProcessor()

    # Attack guardian
    attack_results = processor.process_actions(
        [_attack_request("hero", "guardian")],
        ws,
        step_index=0,
    )
    assert attack_results[0].accepted
    ws.apply_delta(dict(attack_results[0].world_delta))

    # East exit should now be present in vault-entrance
    snap_before_move = ws.get_snapshot()
    assert "east" in snap_before_move["rooms"]["vault-entrance"]["exits"]

    # Now move east should succeed
    move_results = processor.process_actions(
        [_move_request("hero", "east")],
        ws,
        step_index=1,
    )
    assert move_results[0].accepted
    ws.apply_delta(dict(move_results[0].world_delta))
    snap = ws.get_snapshot()
    assert snap["entities"]["hero"]["location"] == "vault"


def test_guardian_defeat_emits_route_unlocked_event() -> None:
    """`route_unlocked` event is emitted when guardian defeat opens the vault passage."""
    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("hero", "guardian")],
        ws,
        step_index=0,
    )
    assert results[0].accepted
    event_types = [e.event_type for e in results[0].events]
    assert "route_unlocked" in event_types

    route_event = next(e for e in results[0].events if e.event_type == "route_unlocked")
    payload = dict(route_event.payload)
    assert payload.get("effect_id") == "guardian-vault-opens"
    assert payload.get("direction") == "east"
    assert payload.get("destination_room_id") == "vault"


def test_guardian_defeat_sets_unlock_scenario_var() -> None:
    """`unlock.guardian-vault-opens` is True in scenario_vars after defeat."""
    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions(
        [_attack_request("hero", "guardian")],
        ws,
        step_index=0,
    )
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()
    assert snap["scenario_vars"].get("unlock.guardian-vault-opens") is True


def test_defeat_unlock_is_idempotent() -> None:
    """A second defeat attempt on an already-defeated NPC does not re-emit route_unlocked."""
    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    # First defeat
    r1 = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    # Second attack on same target (health is now 0, NPC absent from room — attack rejected)
    r2 = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=1)
    event_types = [e.event_type for e in r2[0].events] if r2[0].events else []
    assert "route_unlocked" not in event_types


def test_second_actor_sees_unlocked_exit_after_defeat() -> None:
    """Both actors in the shared room can traverse the unlocked exit after one defeats guardian."""
    ws = _make_gated_combat_world(
        npc_health=10, actor_location="vault-entrance", second_actor_location="vault-entrance"
    )
    processor = BasicDeterministicActionProcessor()

    # hero defeats guardian
    r1 = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=0)
    assert r1[0].accepted
    ws.apply_delta(dict(r1[0].world_delta))

    # hero2 can now move east
    r2 = processor.process_actions([_move_request("hero2", "east")], ws, step_index=1)
    assert r2[0].accepted
    ws.apply_delta(dict(r2[0].world_delta))
    snap = ws.get_snapshot()
    assert snap["entities"]["hero2"]["location"] == "vault"


# ---------------------------------------------------------------------------
# Access-control persists across save/load
# ---------------------------------------------------------------------------


def test_unlocked_access_persists_across_save_load(tmp_path: Path) -> None:
    """After guardian defeat, saving and reloading preserves the unlocked east exit."""
    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    dest = str(tmp_path / "gated_snap.json")
    save_result = save_world_snapshot(
        dest, ws, run_id="r-ac", scenario_id="tiny-shared-combat", actor_ids=["hero"]
    )
    assert save_result.accepted

    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "east" in snap["rooms"]["vault-entrance"]["exits"]
    assert snap["scenario_vars"].get("unlock.guardian-vault-opens") is True


def test_unlocked_access_persists_across_slot_save_load(tmp_path: Path) -> None:
    """Slot save/load cycle preserves east exit and unlock marker."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    save_world_slot(
        "ac-slot",
        tmp_path,
        ws,
        run_id="r-slot-ac",
        scenario_id="tiny-shared-combat",
    )
    loaded = load_world_slot("ac-slot", tmp_path)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "east" in snap["rooms"]["vault-entrance"]["exits"]
    assert snap["scenario_vars"].get("unlock.guardian-vault-opens") is True


def test_gated_world_without_defeat_unlock_no_east_exit(tmp_path: Path) -> None:
    """Saving and reloading a world where guardian was NOT defeated keeps the gate closed."""
    ws = _make_gated_combat_world(npc_health=20)  # NPC still alive

    dest = str(tmp_path / "gated_undefeated.json")
    save_world_snapshot(dest, ws, run_id="r-nd", scenario_id="tiny-shared-combat")
    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "east" not in snap["rooms"]["vault-entrance"]["exits"]
    assert snap["scenario_vars"].get("unlock.guardian-vault-opens") is None


def test_find_defeat_unlock_effect_helper() -> None:
    """_find_defeat_unlock_effect returns the correct effect dict for a matching npc_id."""
    from world.state.basic_action_processor import _find_defeat_unlock_effect

    payload = json.dumps(
        {"boss": {"destination_room_id": "treasure", "direction": "north",
                  "effect_id": "boss-unlock", "source_room_id": "entrance"}},
        sort_keys=True,
        separators=(",", ":"),
    )
    effect = _find_defeat_unlock_effect({"defeat_unlock_effects_json": payload}, "boss")
    assert effect is not None
    assert effect["effect_id"] == "boss-unlock"
    assert effect["direction"] == "north"

    # Wrong npc_id → None
    assert _find_defeat_unlock_effect({"defeat_unlock_effects_json": payload}, "goblin") is None

    # Missing key → None
    assert _find_defeat_unlock_effect({}, "boss") is None


# ---------------------------------------------------------------------------
# _format_tick_event_messages unit tests
# ---------------------------------------------------------------------------


def test_format_tick_event_messages_npc_defeated() -> None:
    """npc_defeated event produces a readable [World] message."""
    from evaluation.benchmark_runner.runner import _format_tick_event_messages
    from core.event_logger import EventRecord, normalize_payload

    event = EventRecord(
        step_index=0,
        event_type="npc_defeated",
        actor_id="hero",
        payload=normalize_payload({"target_id": "guardian", "room_id": "vault-entrance"}),
    )
    messages = _format_tick_event_messages((event,))
    assert len(messages) == 1
    assert "guardian" in messages[0]
    assert "defeated" in messages[0].lower()


def test_format_tick_event_messages_route_unlocked() -> None:
    """route_unlocked event produces a readable [World] passage message."""
    from evaluation.benchmark_runner.runner import _format_tick_event_messages
    from core.event_logger import EventRecord, normalize_payload

    event = EventRecord(
        step_index=0,
        event_type="route_unlocked",
        actor_id="hero",
        payload=normalize_payload({
            "effect_id": "guardian-vault-opens",
            "direction": "east",
            "destination_room_id": "vault",
            "source_room_id": "vault-entrance",
        }),
    )
    messages = _format_tick_event_messages((event,))
    assert len(messages) == 1
    assert "east" in messages[0]
    assert "vault" in messages[0]


def test_format_tick_event_messages_unrelated_events_ignored() -> None:
    """Events other than npc_defeated/route_unlocked produce no messages."""
    from evaluation.benchmark_runner.runner import _format_tick_event_messages
    from core.event_logger import EventRecord, normalize_payload

    events = (
        EventRecord(step_index=0, event_type="action_attack", actor_id="hero",
                    payload=normalize_payload({"damage": 10})),
        EventRecord(step_index=0, event_type="action_move", actor_id="hero",
                    payload=normalize_payload({"direction": "east"})),
    )
    assert _format_tick_event_messages(events) == ()


def test_format_tick_event_messages_empty_input() -> None:
    """Empty events tuple returns empty messages."""
    from evaluation.benchmark_runner.runner import _format_tick_event_messages
    assert _format_tick_event_messages(()) == ()


def test_format_tick_event_messages_multiple_events() -> None:
    """Both npc_defeated and route_unlocked in same tick both produce messages."""
    from evaluation.benchmark_runner.runner import _format_tick_event_messages
    from core.event_logger import EventRecord, normalize_payload

    events = (
        EventRecord(step_index=0, event_type="npc_defeated", actor_id="hero",
                    payload=normalize_payload({"target_id": "guardian", "room_id": "vault-entrance"})),
        EventRecord(step_index=0, event_type="route_unlocked", actor_id="hero",
                    payload=normalize_payload({"direction": "east", "destination_room_id": "vault",
                                               "effect_id": "guardian-vault-opens",
                                               "source_room_id": "vault-entrance"})),
    )
    messages = _format_tick_event_messages(events)
    assert len(messages) == 2
    assert any("guardian" in m for m in messages)
    assert any("east" in m for m in messages)


# ---------------------------------------------------------------------------
# World event log unit tests
# ---------------------------------------------------------------------------


def test_world_event_log_populated_on_npc_defeat() -> None:
    """Defeating an NPC writes a npc_defeated entry to world_event_log_json in scenario_vars."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=5)
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None, "world_event_log_json should be set after defeat"
    log = json.loads(raw)
    assert isinstance(log, list) and len(log) >= 1
    entry = next((e for e in log if e["event_type"] == "npc_defeated"), None)
    assert entry is not None
    assert entry["target_id"] == "troll"
    assert entry["step"] == 5


def test_world_event_log_populated_on_route_unlock() -> None:
    """Defeating a guardian that triggers defeat_unlock_effects writes route_unlocked to log."""
    ws = _make_gated_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("hero", "guardian")], ws, step_index=2)
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None
    log = json.loads(raw)
    event_types = {e["event_type"] for e in log}
    assert "npc_defeated" in event_types
    assert "route_unlocked" in event_types

    route_entry = next(e for e in log if e["event_type"] == "route_unlocked")
    assert route_entry["direction"] == "east"
    assert route_entry["destination_room_id"] == "vault"


def test_world_event_log_contains_npc_alert_for_partial_damage() -> None:
    """npc_alert log entry is written when the NPC survives the attack."""
    ws = _make_combat_world(npc_health=20)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=0)
    ws.apply_delta(dict(results[0].world_delta))
    snap = ws.get_snapshot()

    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None
    log = json.loads(raw)
    assert any(e["event_type"] == "npc_alert" for e in log)


def test_world_event_log_persists_across_save_load(tmp_path: Path) -> None:
    """world_event_log_json survives a save/load cycle intact."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=1)
    ws.apply_delta(dict(results[0].world_delta))

    dest = str(tmp_path / "log_snap.json")
    save_result = save_world_snapshot(dest, ws, run_id="r-log", scenario_id="sc1")
    assert save_result.accepted

    load_result = load_world_snapshot(dest)
    assert load_result.accepted
    snap = load_result.world_state.get_snapshot()  # type: ignore[union-attr]

    raw = snap["scenario_vars"].get("world_event_log_json")
    assert raw is not None
    log = json.loads(raw)
    assert any(e["event_type"] == "npc_defeated" for e in log)


def test_world_event_log_persists_across_slot_save_load(tmp_path: Path) -> None:
    """world_event_log_json survives a slot save/load cycle."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    results = processor.process_actions([_attack_request("fighter", "troll")], ws, step_index=3)
    ws.apply_delta(dict(results[0].world_delta))

    save_world_slot("log-slot", tmp_path, ws, run_id="r-sl", scenario_id="sc1")
    loaded = load_world_slot("log-slot", tmp_path)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    raw = snap["scenario_vars"].get("world_event_log_json")
    log = json.loads(raw)
    assert any(e["event_type"] == "npc_defeated" for e in log)


def test_update_world_event_log_caps_at_max_entries() -> None:
    """When the log exceeds MAX_ENTRIES, oldest entries are dropped."""
    from world.state.basic_action_processor import (
        _update_world_event_log,
        _WORLD_EVENT_LOG_KEY,
        _WORLD_EVENT_LOG_MAX_ENTRIES,
    )
    from core.event_logger import EventRecord, normalize_payload

    # Pre-fill log to max capacity
    prefilled = [{"event_type": "npc_defeated", "step": i, "target_id": f"npc_{i}"} for i in range(_WORLD_EVENT_LOG_MAX_ENTRIES)]
    existing_json = json.dumps(prefilled, sort_keys=True, separators=(",", ":"))
    scenario_vars = {_WORLD_EVENT_LOG_KEY: existing_json}
    delta: dict = {}

    new_event = EventRecord(
        step_index=99,
        event_type="npc_defeated",
        actor_id="hero",
        payload=normalize_payload({"target_id": "boss", "room_id": "arena"}),
    )
    _update_world_event_log(scenario_vars, delta, [new_event])

    result = json.loads(delta[_WORLD_EVENT_LOG_KEY])
    assert len(result) == _WORLD_EVENT_LOG_MAX_ENTRIES
    # Newest entry is last
    assert result[-1]["target_id"] == "boss"
    # Oldest (npc_0) was dropped
    assert result[0]["target_id"] == "npc_1"


def test_format_world_event_log_messages_with_history() -> None:
    """_format_world_event_log_messages returns [History] lines for known event types."""
    from evaluation.benchmark_runner.runner import _format_world_event_log_messages

    log = [
        {"event_type": "npc_defeated", "step": 2, "target_id": "goblin"},
        {"event_type": "route_unlocked", "step": 2, "direction": "north", "destination_room_id": "treasure"},
    ]
    scenario_vars = {"world_event_log_json": json.dumps(log, sort_keys=True, separators=(",", ":"))}
    msgs = _format_world_event_log_messages(scenario_vars)
    assert len(msgs) >= 3  # header + 2 entries
    assert any("[History]" in m for m in msgs)
    assert any("goblin" in m for m in msgs)
    assert any("north" in m for m in msgs)


def test_format_world_event_log_messages_empty_log() -> None:
    """_format_world_event_log_messages returns empty tuple for empty log."""
    from evaluation.benchmark_runner.runner import _format_world_event_log_messages

    assert _format_world_event_log_messages({"world_event_log_json": "[]"}) == ()
    assert _format_world_event_log_messages({}) == ()
    assert _format_world_event_log_messages(None) == ()
