"""Unit tests for persistent NPC calm/de-escalation mechanic.

task_id: persistent_shared_npc_resolution_and_deescalation_v1

Tests cover:
- _find_calm_npc_effect helper
- _build_calm_npc_effects_scenario_delta helper
- calm effect triggered by ``use ward-sigil`` in vault-entrance
- hostile tag removed from NPC after calm
- ward-sigil entity deleted from world
- idempotency (second use has no calm effect)
- NPC-not-hostile or NPC-already-defeated guard
- npc_calmed event payload
- calmed state persists through save/load
- world event log receives npc_calmed entry
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,
    _find_calm_npc_effect,  # type: ignore[attr-defined]
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_calm_world(
    *,
    patrol_tags: list[str] | None = None,
    patrol_health: int | None = 15,
    ward_sigil_location: str = "camp",  # "camp" means in room, actor must take first
    actor_has_ward_sigil: bool = False,  # put ward-sigil directly in actor inventory
) -> Any:
    """Build a minimal world suitable for calm mechanic tests."""
    from world.state.world_state import DeterministicWorldStateManager

    rooms = {
        "camp": {
            "title": "Camp",
            "description": "Camp.",
            "exits": {"east": "vault-entrance"},
            "entities": [],
        },
        "vault-entrance": {
            "title": "Vault Entrance",
            "description": "Guarded entrance.",
            "exits": {"west": "camp"},
            "entities": [],
        },
    }
    patrol_entity: dict[str, Any] = {
        "entity_id": "patrol",
        "entity_type": "npc",
        "location": "vault-entrance",
        "health": patrol_health,
        "inventory": (),
        "tags": tuple(patrol_tags if patrol_tags is not None else ["hostile"]),
    }
    actor_inventory: tuple[str, ...] = ("ward-sigil",) if actor_has_ward_sigil else ()
    actor_entity: dict[str, Any] = {
        "entity_id": "hero",
        "entity_type": "agent",
        "location": "vault-entrance",  # start in same room as patrol
        "health": 100,
        "inventory": actor_inventory,
        "tags": (),
    }
    ward_sigil_entity: dict[str, Any] = {
        "entity_id": "ward-sigil",
        "entity_type": "consumable",
        "location": None if actor_has_ward_sigil else ward_sigil_location,
        "health": None,
        "inventory": (),
        "tags": (),
    }

    calm_effects = [
        {
            "effect_id": "patrol-calmed",
            "item_id": "ward-sigil",
            "source_room_id": "vault-entrance",
            "target_npc_id": "patrol",
        }
    ]

    ws = DeterministicWorldStateManager.from_dict(
        {
            "entities": {
                "hero": actor_entity,
                "patrol": patrol_entity,
                "ward-sigil": ward_sigil_entity,
            },
            "rooms": rooms,
            "scenario_vars": {
                "calm_npc_effects_json": json.dumps(calm_effects),
            },
        }
    )
    return ws


def _use_ward_sigil(ws: Any, *, step_index: int = 0) -> Any:
    from core.action_processor import ActionRequest
    proc = BasicDeterministicActionProcessor()
    results = proc.process_actions(
        [ActionRequest(actor_id="hero", action_type="use", arguments=(("item_id", "ward-sigil"),))],
        world_state=ws,
        step_index=step_index,
    )
    for r in results:
        if r.accepted and r.world_delta:
            ws.apply_delta(dict(r.world_delta))
    return results


# ---------------------------------------------------------------------------
# _find_calm_npc_effect helper
# ---------------------------------------------------------------------------

def test_find_calm_npc_effect_returns_entry_for_matching_item_and_room() -> None:
    """_find_calm_npc_effect finds entry when item+room match."""
    effects = [{"effect_id": "e1", "item_id": "ward-sigil", "source_room_id": "r1", "target_npc_id": "p1"}]
    svars = {"calm_npc_effects_json": json.dumps(effects)}
    result = _find_calm_npc_effect(svars, item_id="ward-sigil", room_id="r1")
    assert result is not None
    assert result["target_npc_id"] == "p1"
    assert result["effect_id"] == "e1"


def test_find_calm_npc_effect_returns_none_for_wrong_room() -> None:
    effects = [{"effect_id": "e1", "item_id": "ward-sigil", "source_room_id": "r1", "target_npc_id": "p1"}]
    svars = {"calm_npc_effects_json": json.dumps(effects)}
    assert _find_calm_npc_effect(svars, item_id="ward-sigil", room_id="r2") is None


def test_find_calm_npc_effect_returns_none_for_wrong_item() -> None:
    effects = [{"effect_id": "e1", "item_id": "ward-sigil", "source_room_id": "r1", "target_npc_id": "p1"}]
    svars = {"calm_npc_effects_json": json.dumps(effects)}
    assert _find_calm_npc_effect(svars, item_id="other-item", room_id="r1") is None


def test_find_calm_npc_effect_returns_none_when_key_absent() -> None:
    assert _find_calm_npc_effect({}, item_id="ward-sigil", room_id="r1") is None


def test_find_calm_npc_effect_returns_none_for_none_room() -> None:
    effects = [{"effect_id": "e1", "item_id": "ward-sigil", "source_room_id": "r1", "target_npc_id": "p1"}]
    svars = {"calm_npc_effects_json": json.dumps(effects)}
    assert _find_calm_npc_effect(svars, item_id="ward-sigil", room_id=None) is None


# ---------------------------------------------------------------------------
# _build_calm_npc_effects_scenario_delta helper
# ---------------------------------------------------------------------------

def test_build_calm_npc_effects_delta_produces_json_list() -> None:
    from evaluation.benchmark_runner.runner import _build_calm_npc_effects_scenario_delta  # type: ignore
    world_config = {
        "calm_npc_effects": [
            {"effect_id": "patrol-calmed", "item_id": "ward-sigil", "source_room_id": "r1", "target_npc_id": "patrol"}
        ]
    }
    result = _build_calm_npc_effects_scenario_delta(world_config)
    assert "calm_npc_effects_json" in result
    parsed = json.loads(result["calm_npc_effects_json"])
    assert isinstance(parsed, list) and len(parsed) == 1
    assert parsed[0]["target_npc_id"] == "patrol"


def test_build_calm_npc_effects_delta_empty_when_no_effects() -> None:
    from evaluation.benchmark_runner.runner import _build_calm_npc_effects_scenario_delta  # type: ignore
    assert _build_calm_npc_effects_scenario_delta({}) == {}
    assert _build_calm_npc_effects_scenario_delta(None) == {}  # type: ignore[arg-type]


# ---------------------------------------------------------------------------
# Core calm mechanic
# ---------------------------------------------------------------------------

def test_use_ward_sigil_removes_hostile_tag_from_patrol() -> None:
    """Using ward-sigil in vault-entrance removes 'hostile' from patrol tags."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)
    snap = ws.get_snapshot()
    assert "hostile" not in snap["entities"]["patrol"]["tags"]


def test_use_ward_sigil_deletes_ward_sigil_entity() -> None:
    """Ward-sigil entity is removed from world after calm."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)
    snap = ws.get_snapshot()
    assert snap["entities"].get("ward-sigil") is None


def test_use_ward_sigil_removes_from_actor_inventory() -> None:
    """Ward-sigil is no longer in actor inventory after calm."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)
    snap = ws.get_snapshot()
    assert "ward-sigil" not in snap["entities"]["hero"]["inventory"]


def test_npc_calmed_event_emitted() -> None:
    """npc_calmed event is in results after successful calm."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    results = _use_ward_sigil(ws)
    all_events = [e for r in results for e in r.events]
    calm_events = [e for e in all_events if e.event_type == "npc_calmed"]
    assert len(calm_events) == 1


def test_npc_calmed_event_payload() -> None:
    """npc_calmed event carries correct target_id, item_id, and room_id."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    results = _use_ward_sigil(ws)
    all_events = [e for r in results for e in r.events]
    calm_event = next(e for e in all_events if e.event_type == "npc_calmed")
    payload = dict(calm_event.payload)
    assert payload["target_id"] == "patrol"
    assert payload["item_id"] == "ward-sigil"
    assert payload["room_id"] == "vault-entrance"


def test_npc_calmed_persists_to_world_event_log() -> None:
    """npc_calmed event is recorded in world_event_log_json after calm."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)
    snap = ws.get_snapshot()
    raw_log = snap["scenario_vars"].get("world_event_log_json")
    assert isinstance(raw_log, str)
    log = json.loads(raw_log)
    calm_entries = [e for e in log if e.get("event_type") == "npc_calmed"]
    assert len(calm_entries) == 1
    assert calm_entries[0]["target_id"] == "patrol"


def test_calm_sets_calmed_idempotency_marker() -> None:
    """calmed.<effect_id> is set in scenario_vars after calm."""
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)
    snap = ws.get_snapshot()
    assert snap["scenario_vars"].get("calmed.patrol-calmed") is True


def test_calm_idempotent_second_use_has_no_calm_effect() -> None:
    """If effect already triggered, re-using another ward-sigil does not emit npc_calmed again."""
    from core.action_processor import ActionRequest

    # Give hero two ward-sigils and a second consumable entity in world
    from world.state.world_state import DeterministicWorldStateManager
    ws = DeterministicWorldStateManager.from_dict(
        {
            "entities": {
                "hero": {
                    "entity_id": "hero",
                    "entity_type": "agent",
                    "location": "vault-entrance",
                    "health": 100,
                    "inventory": ("ward-sigil2",),
                    "tags": (),
                },
                "patrol": {
                    "entity_id": "patrol",
                    "entity_type": "npc",
                    "location": "vault-entrance",
                    "health": 15,
                    "inventory": (),
                    "tags": [],
                },
                "ward-sigil2": {
                    "entity_id": "ward-sigil2",
                    "entity_type": "consumable",
                    "location": None,
                    "health": None,
                    "inventory": (),
                    "tags": (),
                },
            },
            "rooms": {
                "vault-entrance": {"title": "VE", "description": "", "exits": {}, "entities": []},
            },
            "scenario_vars": {
                "calm_npc_effects_json": json.dumps([
                    {"effect_id": "patrol-calmed", "item_id": "ward-sigil2", "source_room_id": "vault-entrance", "target_npc_id": "patrol"}
                ]),
                "calmed.patrol-calmed": True,  # already triggered
            },
        }
    )
    proc = BasicDeterministicActionProcessor()
    results = proc.process_actions(
        [ActionRequest(actor_id="hero", action_type="use", arguments=(("item_id", "ward-sigil2"),))],
        world_state=ws,
        step_index=0,
    )
    all_events = [e for r in results for e in r.events]
    calm_events = [e for e in all_events if e.event_type == "npc_calmed"]
    # idempotent: no second npc_calmed event
    assert len(calm_events) == 0


def test_calm_has_no_effect_if_npc_not_hostile() -> None:
    """Using ward-sigil when patrol is not hostile emits no npc_calmed event."""
    ws = _make_calm_world(actor_has_ward_sigil=True, patrol_tags=[])
    results = _use_ward_sigil(ws)
    all_events = [e for r in results for e in r.events]
    assert not any(e.event_type == "npc_calmed" for e in all_events)


def test_calm_has_no_effect_if_npc_already_defeated() -> None:
    """Using ward-sigil when patrol is defeated (health=0) emits no npc_calmed event."""
    ws = _make_calm_world(actor_has_ward_sigil=True, patrol_health=0)
    results = _use_ward_sigil(ws)
    all_events = [e for r in results for e in r.events]
    assert not any(e.event_type == "npc_calmed" for e in all_events)


def test_calm_survives_save_load_roundtrip(tmp_path: Path) -> None:
    """NPC calmed state (tags, idempotency marker, event log) persist through save/load."""
    from world.state.world_persistence import save_world_slot, load_world_slot

    save_dir = str(tmp_path / "saves")
    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)

    save_world_slot(
        "calm-test", save_dir, ws,
        run_id="calm-r1", scenario_id="test-calm",
    )
    loaded = load_world_slot("calm-test", save_dir)
    assert loaded.accepted
    snap = loaded.world_state.get_snapshot()  # type: ignore[union-attr]

    assert "hostile" not in snap["entities"]["patrol"]["tags"]
    assert snap["scenario_vars"].get("calmed.patrol-calmed") is True
    raw_log = snap["scenario_vars"].get("world_event_log_json")
    log = json.loads(raw_log)
    assert any(e.get("event_type") == "npc_calmed" for e in log)


def test_non_hostile_npc_does_not_counter_attack_after_calm() -> None:
    """Calmed NPC no longer deals counter-attack damage."""
    from world.state.basic_action_processor import process_npc_tick

    ws = _make_calm_world(actor_has_ward_sigil=True)
    _use_ward_sigil(ws)  # calm patrol

    result = process_npc_tick(ws, step_index=1)
    assert result is None  # no hostile NPCs → no counter-attacks
