"""Shared-world same-tick NPC counter-attack correctness.

Verifies that ``process_npc_tick`` produces correct, deterministic retaliation
damage under shared-world conditions including multiple hostile NPCs, multiple
actors, defeated NPCs, room-boundary guards, and save/load persistence.

Key invariants tested:
- Two hostile NPCs in the same room both damage the same actor (damage stacks)
- Actor health floors at 0 (never goes negative)
- Second NPC skips an actor already reduced to 0 hp
- Player defeating an NPC this tick → NPC does NOT counter-attack same tick
- Non-hostile NPC → no counter-attack (result is None)
- NPC in a different room → does not hit actors in other rooms
- Player non-lethal attack + NPC retaliation in the same tick → correct ordering
- Two actors in same room with one hostile NPC → both actors take damage
- Save/load/reconnect preserves post-counter-attack health state
- Deterministic: same inputs produce identical outputs on separate world instances
- Multi-tick damage accumulates correctly across successive ticks
"""

from __future__ import annotations

from core.action_processor import ActionRequest
from core.event_logger import EventLogger
from core.simulation_controller import SimulationController
from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,
    _NPC_COUNTER_DAMAGE,
    process_npc_tick,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_DAMAGE = _NPC_COUNTER_DAMAGE


class _CapturingLogger(EventLogger):
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
    actors: list[tuple[str, int, str]] | None = None,
    npcs: list[tuple[str, int, list[str], str]] | None = None,
    extra_rooms: list[str] | None = None,
) -> DeterministicWorldStateManager:
    """Build a minimal world.

    actors: list of (entity_id, health, room_id)
    npcs:   list of (entity_id, health, tags, room_id)
    """
    actors = actors or [("actor-a", 100, "room-a")]
    npcs = npcs or [("npc-x", 20, ["hostile"], "room-a")]

    entities: dict = {}
    ep: dict[str, list[str]] = {}

    for aid, hp, loc in actors:
        entities[aid] = {
            "entity_id": aid,
            "entity_type": "player",
            "location": loc,
            "health": hp,
            "inventory": [],
            "tags": [],
        }
        ep.setdefault(loc, []).append(aid)

    for nid, hp, tags, loc in npcs:
        entities[nid] = {
            "entity_id": nid,
            "entity_type": "npc",
            "location": loc,
            "health": hp,
            "inventory": [],
            "tags": list(tags),
        }
        ep.setdefault(loc, []).append(nid)

    all_rooms = set(ep.keys()) | {"room-a", "room-b"}
    if extra_rooms:
        all_rooms.update(extra_rooms)

    return DeterministicWorldStateManager.from_dict(
        {
            "tick": 0,
            "entities": entities,
            "rooms": {
                r: {
                    "room_id": r,
                    "description": r,
                    "exits": {},
                    "entities_present": ep.get(r, []),
                }
                for r in sorted(all_rooms)
            },
            "scenario_vars": {},
        }
    )


def _apply_npc_tick(world: DeterministicWorldStateManager, step_index: int = 0):
    result = process_npc_tick(world, step_index=step_index)
    if result is not None and result.world_delta:
        world.apply_delta(dict(result.world_delta))
    return result


def _make_ctrl(world: DeterministicWorldStateManager) -> SimulationController:
    ctrl = SimulationController(
        world_state_manager=world,
        action_processor=BasicDeterministicActionProcessor(),
        event_logger=_CapturingLogger(),
        seed=42,
        max_steps=20,
    )
    ctrl.initialize()
    return ctrl


def _attack(actor_id: str, target_id: str = "npc-x") -> ActionRequest:
    return ActionRequest(
        actor_id=actor_id,
        action_type="attack",
        arguments=(("target_id", target_id),),
    )


# ---------------------------------------------------------------------------
# Tests: two hostile NPCs damage the same actor
# ---------------------------------------------------------------------------


def test_two_hostile_npcs_both_damage_actor() -> None:
    """Two hostile NPCs in the same room each deal one hit of damage to the actor."""
    world = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    _apply_npc_tick(world)

    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 100 - 2 * _DAMAGE


def test_two_hostile_npcs_emit_two_counter_attack_events() -> None:
    """Each hostile NPC emits one npc_counter_attack event per target."""
    world = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    result = _apply_npc_tick(world)

    assert result is not None
    events = [e for e in result.events if e.event_type == "npc_counter_attack"]
    assert len(events) == 2
    npc_ids = {e.actor_id for e in events}
    assert npc_ids == {"npc-1", "npc-2"}


def test_two_hostile_npcs_damage_stacks_not_overwrites() -> None:
    """Damage from two hostile NPCs accumulates; later delta doesn't reset first hit."""
    world = _make_world(
        actors=[("actor-a", 100, "room-a")],
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ],
    )
    _apply_npc_tick(world)
    snap = world.get_snapshot()
    # Must be 100 - 2*DAMAGE, NOT 100 - DAMAGE (stale overwrite would give wrong value)
    assert snap["entities"]["actor-a"]["health"] == 100 - 2 * _DAMAGE


# ---------------------------------------------------------------------------
# Tests: health floor at 0
# ---------------------------------------------------------------------------


def test_actor_health_floors_at_zero() -> None:
    """Actor hit by two NPCs when health equals one hit of damage → health is 0, not negative."""
    world = _make_world(
        actors=[("actor-a", _DAMAGE, "room-a")],
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ],
    )
    _apply_npc_tick(world)
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 0


def test_second_npc_skips_already_zero_health_actor() -> None:
    """Second NPC does not emit a counter-attack against an actor already at 0 hp this tick."""
    world = _make_world(
        actors=[("actor-a", _DAMAGE, "room-a")],
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ],
    )
    result = _apply_npc_tick(world)

    assert result is not None
    # Only ONE event: npc-1 kills actor-a; npc-2 skips
    events = [e for e in result.events if e.event_type == "npc_counter_attack"]
    assert len(events) == 1


# ---------------------------------------------------------------------------
# Tests: defeated NPC does not counter-attack
# ---------------------------------------------------------------------------


def test_npc_defeated_this_tick_does_not_counter_attack() -> None:
    """When a player defeats an NPC in the same tick, that NPC does NOT counter-attack."""
    world = _make_world(npcs=[("npc-x", 10, ["hostile"], "room-a")])
    ctrl = _make_ctrl(world)

    outcome = ctrl.step([_attack("actor-a")])
    assert "npc_defeated" in [e.event_type for e in outcome.emitted_events]

    result = process_npc_tick(world, step_index=0)
    assert result is None


def test_npc_at_zero_health_does_not_counter_attack() -> None:
    """NPC with health=0 in the world snapshot does not counter-attack."""
    world = _make_world(npcs=[("npc-x", 0, ["hostile"], "room-a")])
    result = process_npc_tick(world, step_index=0)
    assert result is None


# ---------------------------------------------------------------------------
# Tests: non-hostile NPC does not retaliate
# ---------------------------------------------------------------------------


def test_non_hostile_npc_no_counter_attack() -> None:
    """NPC without 'hostile' tag never counter-attacks."""
    world = _make_world(npcs=[("npc-x", 20, [], "room-a")])
    result = process_npc_tick(world, step_index=0)
    assert result is None


def test_calmed_npc_no_counter_attack() -> None:
    """NPC with 'calmed' tag but not 'hostile' does not counter-attack."""
    world = _make_world(npcs=[("npc-x", 20, ["calmed"], "room-a")])
    result = process_npc_tick(world, step_index=0)
    assert result is None


# ---------------------------------------------------------------------------
# Tests: room boundary — NPC only hits actors in same room
# ---------------------------------------------------------------------------


def test_npc_in_different_room_does_not_hit_actor() -> None:
    """Hostile NPC in room-b does not damage actor in room-a."""
    world = _make_world(
        actors=[("actor-a", 100, "room-a")],
        npcs=[("npc-x", 20, ["hostile"], "room-b")],
    )
    result = process_npc_tick(world, step_index=0)
    assert result is None


def test_npc_only_hits_colocated_actors() -> None:
    """Hostile NPC in room-a damages actor in room-a but not actor in room-b."""
    world = _make_world(
        actors=[("actor-a", 100, "room-a"), ("actor-b", 100, "room-b")],
        npcs=[("npc-x", 20, ["hostile"], "room-a")],
    )
    _apply_npc_tick(world)
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 100 - _DAMAGE
    assert snap["entities"]["actor-b"]["health"] == 100


# ---------------------------------------------------------------------------
# Tests: player attack + NPC counter-attack in same tick
# ---------------------------------------------------------------------------


def test_player_non_lethal_attack_then_npc_counter_correct_ordering() -> None:
    """Player attack reduces NPC health; NPC then counter-attacks with correct actor health."""
    world = _make_world(npcs=[("npc-x", 30, ["hostile"], "room-a")])
    ctrl = _make_ctrl(world)

    ctrl.step([_attack("actor-a")])
    snap_post_player = world.get_snapshot()
    assert snap_post_player["entities"]["npc-x"]["health"] < 30

    result = _apply_npc_tick(world)
    snap_post_npc = world.get_snapshot()

    assert result is not None
    assert snap_post_npc["entities"]["actor-a"]["health"] == 100 - _DAMAGE


def test_player_attack_npc_counter_attack_events_emitted() -> None:
    """NPC counter-attack event is emitted when hostile NPC retaliates after player attack."""
    world = _make_world(npcs=[("npc-x", 30, ["hostile"], "room-a")])
    ctrl = _make_ctrl(world)

    ctrl.step([_attack("actor-a")])
    result = _apply_npc_tick(world)

    assert result is not None
    events = [e for e in result.events if e.event_type == "npc_counter_attack"]
    assert len(events) == 1
    payload = dict(events[0].payload)
    assert payload["target_id"] == "actor-a"
    assert payload["damage"] == _DAMAGE


# ---------------------------------------------------------------------------
# Tests: one hostile NPC, two actors in same room
# ---------------------------------------------------------------------------


def test_one_npc_damages_all_actors_in_room() -> None:
    """Single hostile NPC deals damage to every alive actor in the same room."""
    world = _make_world(
        actors=[("actor-a", 100, "room-a"), ("actor-b", 100, "room-a")],
        npcs=[("npc-x", 20, ["hostile"], "room-a")],
    )
    result = _apply_npc_tick(world)
    snap = world.get_snapshot()

    assert snap["entities"]["actor-a"]["health"] == 100 - _DAMAGE
    assert snap["entities"]["actor-b"]["health"] == 100 - _DAMAGE
    assert result is not None
    events = [e for e in result.events if e.event_type == "npc_counter_attack"]
    assert len(events) == 2


def test_one_npc_two_actors_counter_attack_targets_are_correct() -> None:
    """npc_counter_attack events correctly name each targeted actor."""
    world = _make_world(
        actors=[("actor-a", 100, "room-a"), ("actor-b", 100, "room-a")],
        npcs=[("npc-x", 20, ["hostile"], "room-a")],
    )
    result = _apply_npc_tick(world)

    assert result is not None
    targeted = {dict(e.payload)["target_id"] for e in result.events if e.event_type == "npc_counter_attack"}
    assert targeted == {"actor-a", "actor-b"}


# ---------------------------------------------------------------------------
# Tests: save/load preserves post-counter-attack state
# ---------------------------------------------------------------------------


def test_save_load_preserves_actor_health_after_counter_attack() -> None:
    """Actor health from NPC counter-attack is correctly preserved after save/load."""
    world = _make_world()
    _apply_npc_tick(world)
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()

    assert snap2["entities"]["actor-a"]["health"] == 100 - _DAMAGE


def test_save_load_two_npc_damage_stacks_preserved() -> None:
    """Stacked damage from two NPCs survives save/load round-trip."""
    world = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    _apply_npc_tick(world)
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()

    assert snap2["entities"]["actor-a"]["health"] == 100 - 2 * _DAMAGE


# ---------------------------------------------------------------------------
# Tests: deterministic ordering
# ---------------------------------------------------------------------------


def test_counter_attack_deterministic_same_inputs() -> None:
    """Identical world instances produce the same counter-attack results."""
    world1 = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    world2 = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    _apply_npc_tick(world1)
    _apply_npc_tick(world2)

    assert world1.get_snapshot()["entities"]["actor-a"]["health"] == world2.get_snapshot()["entities"]["actor-a"]["health"]


def test_counter_attack_event_order_deterministic() -> None:
    """Event ordering from process_npc_tick is consistent across separate calls."""
    world1 = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    world2 = _make_world(
        npcs=[
            ("npc-1", 20, ["hostile"], "room-a"),
            ("npc-2", 20, ["hostile"], "room-a"),
        ]
    )
    result1 = process_npc_tick(world1, step_index=0)
    result2 = process_npc_tick(world2, step_index=0)

    assert result1 is not None and result2 is not None
    actor_ids_1 = [dict(e.payload)["npc_id"] for e in result1.events if e.event_type == "npc_counter_attack"]
    actor_ids_2 = [dict(e.payload)["npc_id"] for e in result2.events if e.event_type == "npc_counter_attack"]
    assert actor_ids_1 == actor_ids_2


# ---------------------------------------------------------------------------
# Tests: multi-tick damage accumulation
# ---------------------------------------------------------------------------


def test_multi_tick_damage_accumulates_correctly() -> None:
    """Calling process_npc_tick N times correctly reduces actor health by N * damage."""
    world = _make_world(npcs=[("npc-x", 20, ["hostile"], "room-a")])

    for i in range(3):
        _apply_npc_tick(world, step_index=i)

    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 100 - 3 * _DAMAGE


def test_multi_tick_health_never_negative() -> None:
    """Repeated NPC ticks do not push actor health below 0."""
    ticks_to_kill = (100 + _DAMAGE - 1) // _DAMAGE + 5  # several ticks past death
    world = _make_world(npcs=[("npc-x", 20, ["hostile"], "room-a")])

    for i in range(ticks_to_kill):
        _apply_npc_tick(world, step_index=i)

    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 0


# ---------------------------------------------------------------------------
# Tests: no regression — no hostile NPC returns None
# ---------------------------------------------------------------------------


def test_no_hostile_npc_returns_none() -> None:
    """process_npc_tick returns None when there are no hostile NPCs."""
    world = _make_world(npcs=[("npc-x", 20, [], "room-a")])
    assert process_npc_tick(world, step_index=0) is None


def test_no_actors_returns_none() -> None:
    """process_npc_tick returns None when there are no alive actors."""
    # Actor with 0 health is excluded from room_to_actors
    world = _make_world(
        actors=[("actor-a", 0, "room-a")],
        npcs=[("npc-x", 20, ["hostile"], "room-a")],
    )
    result = process_npc_tick(world, step_index=0)
    assert result is None
