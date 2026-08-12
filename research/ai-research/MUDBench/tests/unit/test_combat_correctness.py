"""Shared-world same-tick multi-actor combat correctness.

Verifies that the incremental delta fix in SimulationController.step() ensures
correct, deterministic NPC health/state and one-time defeat side-effects when
multiple actors attack the same NPC in one tick.

Key invariants tested:
- Two simultaneous lethal attacks → defeat applied once, second attack rejected
- NPC health never goes below zero or gets double-subtracted
- npc_defeated event emitted exactly once
- npc_alert event emitted exactly once (not duplicated by the second attacker)
- defeated.{npc_id} scenario var set exactly once (True)
- NPC removed from room exactly once
- Defeat side-effects (route unlock) applied once only
- Partial damage + lethal damage in the same tick → cumulative result
- Save/load/reconnect preserves the corrected post-combat state
- Deterministic ordering: same inputs produce same outputs
- Single-actor attack still works (no regression)
"""

from __future__ import annotations

import json

from core.action_processor import ActionRequest
from core.simulation_controller import SimulationController
from core.event_logger import EventLogger
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


class _CapturingLogger(EventLogger):
    """Minimal event logger that captures all emitted records."""

    def __init__(self) -> None:
        self._records: list = []

    def log(self, record) -> None:  # type: ignore[override]
        self._records.append(record)

    def reset(self) -> None:
        self._records.clear()

    def records(self) -> tuple:
        return tuple(self._records)


def _make_world(
    *,
    npc_health: int = 20,
    npc_tags: list[str] | None = None,
    extra_scenario_vars: dict | None = None,
) -> DeterministicWorldStateManager:
    """Build a minimal combat world with two players and one NPC in room-a."""
    svars: dict = {}
    if extra_scenario_vars:
        svars.update(extra_scenario_vars)
    return DeterministicWorldStateManager.from_dict(
        {
            "tick": 0,
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a",
                    "entity_type": "player",
                    "location": "room-a",
                    "health": 100,
                    "inventory": [],
                    "tags": [],
                },
                "actor-b": {
                    "entity_id": "actor-b",
                    "entity_type": "player",
                    "location": "room-a",
                    "health": 100,
                    "inventory": [],
                    "tags": [],
                },
                "npc-x": {
                    "entity_id": "npc-x",
                    "entity_type": "npc",
                    "location": "room-a",
                    "health": npc_health,
                    "inventory": [],
                    "tags": npc_tags or [],
                },
            },
            "rooms": {
                "room-a": {
                    "room_id": "room-a",
                    "description": "Arena",
                    "exits": {"east": "room-b"},
                    "entities_present": ["actor-a", "actor-b", "npc-x"],
                },
                "room-b": {
                    "room_id": "room-b",
                    "description": "Exit",
                    "exits": {"west": "room-a"},
                    "entities_present": [],
                },
            },
            "scenario_vars": svars,
        }
    )


def _make_controller(
    world: DeterministicWorldStateManager,
) -> tuple[SimulationController, _CapturingLogger]:
    logger = _CapturingLogger()
    ctrl = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=logger,
        seed=42,
        max_steps=20,
    )
    ctrl.initialize()
    return ctrl, logger


def _attack(actor_id: str, target_id: str = "npc-x") -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="attack",
        arguments=(("target_id", target_id),),
    )


def _event_types(outcome) -> list[str]:
    return [e.event_type for e in outcome.emitted_events]


def _defeat_unlock_scenario_vars(
    npc_id: str = "npc-x",
    source_room: str = "room-a",
    direction: str = "north",
    dest_room: str = "room-b",
    effect_id: str = "gate-open",
) -> dict:
    """Build scenario_vars with a defeat_unlock_effects_json dict (keyed by npc_id)."""
    effect_map = {
        npc_id: {
            "destination_room_id": dest_room,
            "direction": direction,
            "effect_id": effect_id,
            "source_room_id": source_room,
        }
    }
    return {"defeat_unlock_effects_json": json.dumps(effect_map, sort_keys=True)}


# ---------------------------------------------------------------------------
# Tests: simultaneous lethal attacks
# ---------------------------------------------------------------------------


def test_two_lethal_attacks_only_one_defeat() -> None:
    """Both actors attack a 10-health NPC → only the first attacker defeats it."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    types = _event_types(outcome)

    assert types.count("npc_defeated") == 1
    assert "action_rejected" in types


def test_two_lethal_attacks_npc_health_not_negative() -> None:
    """NPC health after simultaneous lethal attacks stays at 0 (not negative)."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    npc_entity = snap["entities"].get("npc-x", {})
    # NPC is either removed or has health exactly 0
    health = npc_entity.get("health", 0)
    assert health == 0


def test_two_lethal_attacks_npc_removed_from_room_once() -> None:
    """NPC appears in room-a exactly zero times after defeat (no ghost)."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    entities_present = snap["rooms"]["room-a"].get("entities_present", [])
    assert entities_present.count("npc-x") == 0


def test_two_lethal_attacks_defeated_var_set_once() -> None:
    """defeated.npc-x scenario var is True (and not duplicated/double-set)."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("defeated.npc-x") is True


def test_two_lethal_attacks_second_attack_rejected() -> None:
    """The second attacker's action_rejected is specifically target_already_defeated."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    rejected = [e for e in outcome.emitted_events if e.event_type == "action_rejected"]
    assert len(rejected) == 1
    payload = dict(rejected[0].payload) if rejected[0].payload else {}
    assert payload.get("reason") == "target_already_defeated"


# ---------------------------------------------------------------------------
# Tests: npc_alert deduplication
# ---------------------------------------------------------------------------


def test_two_non_lethal_attacks_npc_alert_emitted_once() -> None:
    """Two non-lethal attacks → npc_alert fires once, NPC becomes hostile once."""
    world = _make_world(npc_health=30)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    assert _event_types(outcome).count("npc_alert") == 1


def test_two_non_lethal_attacks_npc_becomes_hostile_once() -> None:
    """'hostile' tag on NPC appears exactly once after two non-lethal attacks."""
    world = _make_world(npc_health=30)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    tags = snap["entities"]["npc-x"].get("tags", [])
    assert tags.count("hostile") == 1


def test_already_hostile_npc_no_additional_alert() -> None:
    """Attacking an already-hostile NPC doesn't emit npc_alert again."""
    world = _make_world(npc_health=30, npc_tags=["hostile"])
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    assert "npc_alert" not in _event_types(outcome)


# ---------------------------------------------------------------------------
# Tests: partial + lethal in the same tick
# ---------------------------------------------------------------------------


def test_partial_then_lethal_same_tick_npc_defeated() -> None:
    """20-health NPC, actor-a does 10 damage, actor-b does 10 damage → npc_defeated once."""
    world = _make_world(npc_health=20)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    assert _event_types(outcome).count("npc_defeated") == 1


def test_partial_then_lethal_same_tick_health_exactly_zero() -> None:
    """Cumulative damage from two attacks reduces 20-health NPC to exactly 0."""
    world = _make_world(npc_health=20)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    health = snap["entities"].get("npc-x", {}).get("health", 0)
    assert health == 0


def test_partial_then_lethal_same_tick_npc_alert_once() -> None:
    """20-health NPC: first hit alerts, second hit defeats → npc_alert exactly once."""
    world = _make_world(npc_health=20)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    types = _event_types(outcome)
    assert types.count("npc_alert") == 1
    assert types.count("npc_defeated") == 1


# ---------------------------------------------------------------------------
# Tests: defeat side-effects applied once only
# ---------------------------------------------------------------------------


def test_defeat_unlock_applied_once() -> None:
    """route_unlocked event emitted exactly once when two actors kill an NPC with unlock effect."""
    svars = _defeat_unlock_scenario_vars()
    world = _make_world(npc_health=10, extra_scenario_vars=svars)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a"), _attack("actor-b")])
    assert _event_types(outcome).count("route_unlocked") == 1


def test_defeat_unlock_var_set_exactly_once() -> None:
    """unlock.<effect_id> scenario var is True exactly once (not duplicated)."""
    svars = _defeat_unlock_scenario_vars()
    world = _make_world(npc_health=10, extra_scenario_vars=svars)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("unlock.gate-open") is True


def test_defeat_unlock_route_added_to_room() -> None:
    """After defeat unlock, the unlocked direction is added to room-a's exits."""
    svars = _defeat_unlock_scenario_vars(
        npc_id="npc-x",
        source_room="room-a",
        direction="north",
        dest_room="room-b",
    )
    world = _make_world(npc_health=10, extra_scenario_vars=svars)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    snap = world.get_snapshot()
    exits = snap["rooms"]["room-a"].get("exits", {})
    assert exits.get("north") == "room-b"


# ---------------------------------------------------------------------------
# Tests: save/load preserves post-combat state
# ---------------------------------------------------------------------------


def test_save_load_preserves_npc_defeated_state() -> None:
    """After save/load, defeated NPC state is consistent with pre-save combat result."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()
    assert snap2["scenario_vars"].get("defeated.npc-x") is True
    assert "npc-x" not in snap2["rooms"]["room-a"].get("entities_present", [])


def test_save_load_preserves_unlock_state() -> None:
    """After save/load, route unlock state matches pre-save result."""
    svars = _defeat_unlock_scenario_vars()
    world = _make_world(npc_health=10, extra_scenario_vars=svars)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a"), _attack("actor-b")])
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()
    assert snap2["scenario_vars"].get("unlock.gate-open") is True
    assert snap2["rooms"]["room-a"]["exits"].get("north") == "room-b"


# ---------------------------------------------------------------------------
# Tests: deterministic ordering
# ---------------------------------------------------------------------------


def test_combat_ordering_deterministic() -> None:
    """Same inputs on two separate world instances produce identical post-combat state."""
    world1 = _make_world(npc_health=20)
    world2 = _make_world(npc_health=20)
    ctrl1, _ = _make_controller(world1)
    ctrl2, _ = _make_controller(world2)

    outcome1 = ctrl1.step([_attack("actor-a"), _attack("actor-b")])
    outcome2 = ctrl2.step([_attack("actor-a"), _attack("actor-b")])

    assert _event_types(outcome1) == _event_types(outcome2)
    assert world1.get_snapshot()["scenario_vars"] == world2.get_snapshot()["scenario_vars"]


def test_combat_ordering_action_order_independent() -> None:
    """Swapping the action list order doesn't produce different state (first by actor_id sort)."""
    world1 = _make_world(npc_health=10)
    world2 = _make_world(npc_health=10)
    ctrl1, _ = _make_controller(world1)
    ctrl2, _ = _make_controller(world2)

    # actor-a < actor-b lexicographically — both orderings produce same result
    ctrl1.step([_attack("actor-a"), _attack("actor-b")])
    ctrl2.step([_attack("actor-b"), _attack("actor-a")])

    snap1 = world1.get_snapshot()
    snap2 = world2.get_snapshot()
    assert snap1["scenario_vars"].get("defeated.npc-x") is True
    assert snap2["scenario_vars"].get("defeated.npc-x") is True
    assert snap1["rooms"]["room-a"].get("entities_present") == snap2["rooms"]["room-a"].get(
        "entities_present"
    )


# ---------------------------------------------------------------------------
# Tests: no regression — single-actor attack
# ---------------------------------------------------------------------------


def test_single_actor_attack_still_works() -> None:
    """Solo actor attack on full-health NPC: alert event fired, health reduced."""
    world = _make_world(npc_health=30)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a")])
    types = _event_types(outcome)

    assert "action_attack" in types
    assert "npc_alert" in types
    snap = world.get_snapshot()
    npc_health = snap["entities"]["npc-x"]["health"]
    assert 0 < npc_health < 30


def test_single_actor_lethal_attack_defeats_npc() -> None:
    """Solo actor lethal attack emits npc_defeated exactly once."""
    world = _make_world(npc_health=10)
    ctrl, _ = _make_controller(world)

    outcome = ctrl.step([_attack("actor-a")])
    types = _event_types(outcome)

    assert types.count("npc_defeated") == 1
    assert "action_rejected" not in types


def test_single_actor_multi_tick_reduces_health() -> None:
    """Solo actor can reduce NPC health over multiple ticks without weirdness."""
    world = _make_world(npc_health=30)
    ctrl, _ = _make_controller(world)

    ctrl.step([_attack("actor-a")])
    health_after_tick1 = world.get_snapshot()["entities"]["npc-x"]["health"]

    ctrl.step([_attack("actor-a")])
    health_after_tick2 = world.get_snapshot()["entities"]["npc-x"]["health"]

    assert 0 < health_after_tick1 < 30
    assert health_after_tick2 < health_after_tick1
