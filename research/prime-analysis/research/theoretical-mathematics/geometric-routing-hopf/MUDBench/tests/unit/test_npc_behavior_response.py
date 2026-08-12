"""Unit tests for persistent NPC behavior-response mechanics.

Covers:
- hostile NPC deals counter-damage to actors in same room each tick
- non-hostile NPC does not deal counter-damage
- actor not in same room as hostile NPC is unaffected
- damage accumulates correctly across multiple ticks
- actor with None health treated as 100 HP (default)
- actor already at 0 HP is not further harmed
- npc_counter_attack event emitted with correct payload
- delta persists through world state apply_delta
- hostile NPC bootstrapped with tags persists through snapshot roundtrip
- counter-attack mechanic correct with multiple NPCs in same room
- counter-attack state survives save/load
"""

from __future__ import annotations

import json
from pathlib import Path

from world.state.state_types import WorldEntityState, WorldRoomState, WorldStateSnapshot
from world.state.world_state import DeterministicWorldStateManager
from world.state.world_persistence import save_world_snapshot, load_world_snapshot
from world.state.basic_action_processor import (
    process_npc_tick,
    _NPC_COUNTER_DAMAGE,
    _DEFAULT_ACTOR_HEALTH,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_combat_world(
    *,
    actor_ids: tuple[str, ...] = ("hero",),
    actor_location: str = "arena",
    npc_id: str = "patrol",
    npc_location: str = "arena",
    npc_tags: tuple[str, ...] = ("hostile",),
    npc_health: int = 15,
    actor_health: int | None = None,
) -> DeterministicWorldStateManager:
    entities = [
        WorldEntityState(
            entity_id=actor_id,
            entity_type="agent",
            location=actor_location,
            health=actor_health,
            inventory=(),
            tags=(),
        )
        for actor_id in actor_ids
    ]
    entities.append(
        WorldEntityState(
            entity_id=npc_id,
            entity_type="npc",
            location=npc_location,
            health=npc_health,
            inventory=(),
            tags=npc_tags,
        )
    )
    snapshot = WorldStateSnapshot(
        tick=0,
        entities=tuple(entities),
        rooms=(
            WorldRoomState(
                room_id="arena",
                description="Arena.",
                exits=(("south", "safe"),),
                entities_present=tuple(sorted(list(actor_ids) + ([npc_id] if npc_location == "arena" else []))),
            ),
            WorldRoomState(
                room_id="safe",
                description="Safe room.",
                exits=(("north", "arena"),),
                entities_present=tuple(sorted([npc_id] if npc_location == "safe" else [])),
            ),
        ),
        scenario_vars=(),
    )
    return DeterministicWorldStateManager(initial_state=snapshot)


# ---------------------------------------------------------------------------
# Basic counter-damage
# ---------------------------------------------------------------------------


def test_hostile_npc_deals_damage_to_actor_in_same_room() -> None:
    """Hostile NPC in same room deals _NPC_COUNTER_DAMAGE to actor each tick."""
    ws = _make_combat_world()
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["hero"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE


def test_non_hostile_npc_does_not_deal_damage() -> None:
    """NPC without 'hostile' tag does not deal counter-damage."""
    ws = _make_combat_world(npc_tags=())
    result = process_npc_tick(ws, step_index=0)
    assert result is None


def test_actor_in_different_room_not_hit() -> None:
    """Actor in a different room from hostile NPC is not damaged."""
    ws = _make_combat_world(actor_location="safe", npc_location="arena")
    result = process_npc_tick(ws, step_index=0)
    assert result is None


def test_actor_health_none_treated_as_default() -> None:
    """Actor with health=None is treated as _DEFAULT_ACTOR_HEALTH for damage calculation."""
    ws = _make_combat_world(actor_health=None)
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))
    snap = ws.get_snapshot()
    assert snap["entities"]["hero"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE


def test_actor_at_zero_hp_not_further_damaged() -> None:
    """Actor already at 0 HP is not further damaged by NPC counter-attack."""
    ws = _make_combat_world(actor_health=0)
    result = process_npc_tick(ws, step_index=0)
    assert result is None


def test_damage_accumulates_across_multiple_ticks() -> None:
    """Calling process_npc_tick N times reduces actor health by N*_NPC_COUNTER_DAMAGE."""
    ws = _make_combat_world()
    for i in range(3):
        result = process_npc_tick(ws, step_index=i)
        assert result is not None
        ws.apply_delta(dict(result.world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["hero"]["health"] == _DEFAULT_ACTOR_HEALTH - 3 * _NPC_COUNTER_DAMAGE


def test_health_clamps_at_zero() -> None:
    """Actor health cannot go below 0 even with many ticks of damage."""
    ws = _make_combat_world(actor_health=3)  # less than _NPC_COUNTER_DAMAGE
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))
    snap = ws.get_snapshot()
    assert snap["entities"]["hero"]["health"] == 0


# ---------------------------------------------------------------------------
# Event payload
# ---------------------------------------------------------------------------


def test_npc_counter_attack_event_emitted() -> None:
    """npc_counter_attack event is emitted for each actor hit."""
    ws = _make_combat_world()
    result = process_npc_tick(ws, step_index=5)
    assert result is not None
    event_types = [e.event_type for e in result.events]
    assert "npc_counter_attack" in event_types


def test_npc_counter_attack_event_payload_correct() -> None:
    """npc_counter_attack payload includes npc_id, target_id, damage, resulting_health."""
    ws = _make_combat_world()
    result = process_npc_tick(ws, step_index=2)
    assert result is not None
    events = [e for e in result.events if e.event_type == "npc_counter_attack"]
    assert len(events) == 1
    payload = dict(events[0].payload)
    assert payload["npc_id"] == "patrol"
    assert payload["target_id"] == "hero"
    assert payload["damage"] == _NPC_COUNTER_DAMAGE
    assert payload["resulting_health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE


# ---------------------------------------------------------------------------
# Multiple actors and NPCs
# ---------------------------------------------------------------------------


def test_multiple_actors_both_take_damage() -> None:
    """All actors in same room as hostile NPC take damage."""
    ws = _make_combat_world(actor_ids=("actor-a", "actor-b"))
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))
    snap = ws.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE
    assert snap["entities"]["actor-b"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE


def test_only_actor_in_npc_room_takes_damage() -> None:
    """Only actor-a in the NPC room takes damage; actor-b in safe room is unaffected."""
    entities = [
        WorldEntityState(
            entity_id="actor-a",
            entity_type="agent",
            location="arena",
            health=None,
            inventory=(), tags=(),
        ),
        WorldEntityState(
            entity_id="actor-b",
            entity_type="agent",
            location="safe",
            health=None,
            inventory=(), tags=(),
        ),
        WorldEntityState(
            entity_id="patrol",
            entity_type="npc",
            location="arena",
            health=15,
            inventory=(), tags=("hostile",),
        ),
    ]
    snapshot = WorldStateSnapshot(
        tick=0,
        entities=tuple(entities),
        rooms=(
            WorldRoomState(room_id="arena", description="", exits=(("south", "safe"),),
                           entities_present=("actor-a", "patrol")),
            WorldRoomState(room_id="safe", description="", exits=(("north", "arena"),),
                           entities_present=("actor-b",)),
        ),
        scenario_vars=(),
    )
    ws = DeterministicWorldStateManager(initial_state=snapshot)
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))

    snap = ws.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE
    assert snap["entities"]["actor-b"].get("health") is None  # untouched


# ---------------------------------------------------------------------------
# NPC tags persistence
# ---------------------------------------------------------------------------


def test_hostile_tag_in_npc_bootstrap_persists_in_snapshot() -> None:
    """NPC bootstrapped with hostile tag retains it after from_dict roundtrip."""
    ws = _make_combat_world(npc_tags=("hostile",))
    snap = ws.get_snapshot()
    assert "hostile" in snap["entities"]["patrol"]["tags"]

    ws2 = DeterministicWorldStateManager.from_dict(snap)
    snap2 = ws2.get_snapshot()
    assert "hostile" in snap2["entities"]["patrol"]["tags"]


# ---------------------------------------------------------------------------
# Save / load
# ---------------------------------------------------------------------------


def test_counter_damage_persists_through_save_load(tmp_path: Path) -> None:
    """Actor HP reduced by counter-attack survives save/load."""
    ws = _make_combat_world()
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))

    dest = str(tmp_path / "snap.json")
    save_world_snapshot(dest, ws, run_id="r-ca", scenario_id="sc1")
    loaded = load_world_snapshot(dest)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]
    assert snap["entities"]["hero"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE


def test_hostile_npc_tag_and_counter_damage_state_inspectable_in_json(tmp_path: Path) -> None:
    """Saved snapshot JSON shows reduced actor HP and hostile NPC tag."""
    ws = _make_combat_world()
    result = process_npc_tick(ws, step_index=0)
    assert result is not None
    ws.apply_delta(dict(result.world_delta))

    dest = tmp_path / "inspect.json"
    save_world_snapshot(str(dest), ws, run_id="r-inspect", scenario_id="sc1")
    payload = json.loads(dest.read_text(encoding="utf-8"))

    assert payload["world_state"]["entities"]["hero"]["health"] == _DEFAULT_ACTOR_HEALTH - _NPC_COUNTER_DAMAGE
    assert "hostile" in payload["world_state"]["entities"]["patrol"]["tags"]
