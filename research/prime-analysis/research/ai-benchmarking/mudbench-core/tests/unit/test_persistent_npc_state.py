"""Unit tests for persistent NPC state mechanics.

Covers:
- partial NPC health persists after attack (entity.health updated in world delta)
- hostile tag set on first attack (entity.tags includes 'hostile')
- hostile tag idempotent (not added twice on second attack)
- hostile tag absent when NPC is defeated
- npc_alert event emitted on first attack only
- npc_alert persisted in world event log
- npc_alert in event log survives save/load
- multiple actors see same NPC state
"""

from __future__ import annotations

import json
from pathlib import Path

from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager
from world.state.world_persistence import save_world_snapshot, load_world_snapshot
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from core.action_processor import ActionRequest


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_GUARDIAN_HEALTH = 30  # 3 hits at default damage=10 to defeat


def _make_combat_world(
    *,
    actor_ids: tuple[str, ...] = ("hero",),
    npc_health: int = _GUARDIAN_HEALTH,
) -> DeterministicWorldStateManager:
    entities = [
        WorldEntityState(
            entity_id=actor_id,
            entity_type="player",
            location="arena",
            health=100,
            inventory=(),
            tags=(),
        )
        for actor_id in actor_ids
    ]
    entities.append(
        WorldEntityState(
            entity_id="guard",
            entity_type="npc",
            location="arena",
            health=npc_health,
            inventory=(),
            tags=(),
        )
    )
    snapshot = WorldStateSnapshot(
        tick=0,
        entities=tuple(entities),
        rooms=(
            WorldRoomState(
                room_id="arena",
                description="Combat arena.",
                exits=(("south", "exit"),),
                entities_present=tuple(sorted(list(actor_ids) + ["guard"])),
            ),
            WorldRoomState(
                room_id="exit",
                description="Exit.",
                exits=(("north", "arena"),),
                entities_present=(),
            ),
        ),
        scenario_vars=(),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


def _attack(actor_id: str, target_id: str) -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="attack",
        arguments=(("target_id", target_id),),
    )


# ---------------------------------------------------------------------------
# Partial health persistence
# ---------------------------------------------------------------------------


def test_partial_health_persists_after_first_attack() -> None:
    """NPC health decreases and is stored in entity after one attack."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    assert r[0].accepted
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["guard"]["health"] == _GUARDIAN_HEALTH - 10


def test_health_decrements_correctly_across_multiple_attacks() -> None:
    """NPC health decrements by damage per attack until zero."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    for i in range(2):
        r = processor.process_actions([_attack("hero", "guard")], ws, step_index=i)
        ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["guard"]["health"] == _GUARDIAN_HEALTH - 20


def test_defeated_npc_health_is_zero() -> None:
    """After defeat (health reaches 0), health is 0 in entity state."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["guard"]["health"] == 0


# ---------------------------------------------------------------------------
# Hostile tag
# ---------------------------------------------------------------------------


def test_hostile_tag_added_on_first_attack_surviving_npc() -> None:
    """'hostile' tag is added to entity tags after first attack when NPC survives."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert "hostile" in snap["entities"]["guard"]["tags"]


def test_hostile_tag_idempotent_on_second_attack() -> None:
    """'hostile' appears exactly once in tags after multiple attacks."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    for i in range(2):
        r = processor.process_actions([_attack("hero", "guard")], ws, step_index=i)
        ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["guard"]["tags"].count("hostile") == 1


def test_hostile_tag_not_set_when_npc_defeated_in_one_hit() -> None:
    """When NPC dies in one hit, hostile tag is not set (no surviving attack)."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    assert "hostile" not in snap["entities"]["guard"].get("tags", [])


def test_hostile_tag_persists_through_snapshot_roundtrip() -> None:
    """Hostile tag in entity.tags survives to_dict()/from_dict() roundtrip."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap1 = ws.get_snapshot()
    # Roundtrip through from_dict
    ws2 = DeterministicWorldStateManager.from_dict(snap1)
    snap2 = ws2.get_snapshot()
    assert "hostile" in snap2["entities"]["guard"]["tags"]


# ---------------------------------------------------------------------------
# npc_alert event
# ---------------------------------------------------------------------------


def test_npc_alert_event_emitted_on_first_attack() -> None:
    """npc_alert event is emitted when NPC first becomes hostile."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    event_types = [e.event_type for e in r[0].events]
    assert "npc_alert" in event_types


def test_npc_alert_event_not_emitted_on_second_attack() -> None:
    """npc_alert is NOT emitted a second time when NPC is already hostile."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r1 = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r1[0].world_delta))

    r2 = processor.process_actions([_attack("hero", "guard")], ws, step_index=1)
    event_types = [e.event_type for e in r2[0].events]
    assert "npc_alert" not in event_types


def test_npc_alert_not_emitted_on_defeating_attack() -> None:
    """When a single attack defeats the NPC, npc_alert is not emitted."""
    ws = _make_combat_world(npc_health=10)
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    event_types = [e.event_type for e in r[0].events]
    assert "npc_alert" not in event_types
    assert "npc_defeated" in event_types


def test_npc_alert_payload_contains_remaining_health() -> None:
    """npc_alert event payload includes remaining_health."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    alert_events = [e for e in r[0].events if e.event_type == "npc_alert"]
    assert len(alert_events) == 1
    payload = dict(alert_events[0].payload)
    assert payload["remaining_health"] == _GUARDIAN_HEALTH - 10
    assert payload["target_id"] == "guard"


# ---------------------------------------------------------------------------
# World event log
# ---------------------------------------------------------------------------


def test_npc_alert_logged_in_world_event_log() -> None:
    """npc_alert is persisted to world_event_log_json in scenario_vars."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    raw_log = snap["scenario_vars"].get("world_event_log_json")
    assert raw_log is not None
    log = json.loads(raw_log)
    event_types = [e["event_type"] for e in log]
    assert "npc_alert" in event_types


def test_npc_alert_log_survives_save_load(tmp_path: Path) -> None:
    """npc_alert in world event log persists through save/load."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    dest = str(tmp_path / "alert_snap.json")
    save_world_snapshot(dest, ws, run_id="r-alert", scenario_id="sc1")
    loaded = load_world_snapshot(dest)
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    raw_log = snap["scenario_vars"].get("world_event_log_json")
    assert raw_log is not None
    log = json.loads(raw_log)
    assert any(e["event_type"] == "npc_alert" for e in log)


# ---------------------------------------------------------------------------
# Multi-actor state consistency
# ---------------------------------------------------------------------------


def test_multiple_actors_see_same_npc_hostile_state() -> None:
    """After actor-a attacks, both actors share the same NPC state in one world."""
    ws = _make_combat_world(actor_ids=("actor-a", "actor-b"))
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("actor-a", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    snap = ws.get_snapshot()
    # Single shared snapshot — both actors read from the same entity state
    assert "hostile" in snap["entities"]["guard"]["tags"]
    assert snap["entities"]["guard"]["health"] == _GUARDIAN_HEALTH - 10


def test_hostile_state_persists_across_save_load(tmp_path: Path) -> None:
    """Hostile tag and partial health both survive save/load roundtrip."""
    ws = _make_combat_world()
    processor = BasicDeterministicActionProcessor()

    r = processor.process_actions([_attack("hero", "guard")], ws, step_index=0)
    ws.apply_delta(dict(r[0].world_delta))

    dest = str(tmp_path / "hostile_snap.json")
    save_world_snapshot(dest, ws, run_id="r-hostile", scenario_id="sc1")
    loaded = load_world_snapshot(dest)
    assert loaded.accepted

    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert "hostile" in snap["entities"]["guard"]["tags"]
    assert snap["entities"]["guard"]["health"] == _GUARDIAN_HEALTH - 10
