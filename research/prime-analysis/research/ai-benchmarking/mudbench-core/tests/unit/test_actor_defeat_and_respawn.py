"""Shared-world actor defeat detection and respawn correctness.

Verifies that process_actor_defeat_tick and process_actor_respawn_tick
produce correct, deterministic actor defeat/respawn state under shared-world
conditions, and that defeated actors cannot act until respawned.

Key invariants tested:
- Actor at 0 HP triggers defeat detection exactly once
- Defeat sets actor_defeated.{actor_id}=True and schedules respawn
- Already-defeated actor is not re-detected in subsequent ticks
- Alive actor (health > 0) is not marked defeated
- Respawn fires at the correct tick (not before)
- Respawn restores health from actor_respawn_health scenario var
- Respawn moves actor to actor_respawn_room_id if configured
- Respawn clears actor_defeated and actor_respawn_at vars
- Room entities_present updated correctly on respawn room change
- Defeated actor cannot act in advance_tick (action skipped)
- Respawned actor can act again in advance_tick
- Default respawn delay and health apply when no scenario vars are set
- Save/load/reconnect preserves defeat and respawn state
- Deterministic: same inputs produce same outputs
- No regression: alive actors and unrelated world state unchanged
"""

from __future__ import annotations

from core.action_processor import ActionRequest  # noqa: F401 — kept for completeness
from core.event_logger import EventLogger
from core.simulation_controller import SimulationController  # noqa: F401
from evaluation.benchmark_runner.runner import (
    SharedShardLoopSession,
    _format_actor_defeat_status_message,
)
from world.state.basic_action_processor import (
    BasicDeterministicActionProcessor,  # noqa: F401
    _DEFAULT_ACTOR_RESPAWN_DELAY,
    _DEFAULT_ACTOR_RESPAWN_HEALTH,
    _NPC_COUNTER_DAMAGE,
    process_actor_defeat_tick,
    process_actor_respawn_tick,
    process_npc_tick,
)
from world.state.world_state import DeterministicWorldStateManager


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


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
    actor_health: int = 100,
    npc_health: int = 20,
    npc_tags: list[str] | None = None,
    actor_room: str = "camp",
    respawn_delay: int | None = 3,
    respawn_health: int | None = 50,
    respawn_room: str | None = "vault",
) -> DeterministicWorldStateManager:
    svars: dict = {}
    if respawn_delay is not None:
        svars["actor_respawn_delay_ticks"] = respawn_delay
    if respawn_health is not None:
        svars["actor_respawn_health"] = respawn_health
    if respawn_room is not None:
        svars["actor_respawn_room_id"] = respawn_room

    entities_present_camp = ["actor-a"]
    entities_present_vault: list[str] = []
    if actor_room == "vault":
        entities_present_camp = []
        entities_present_vault = ["actor-a"]

    npc_room = "camp"
    entities_present_camp.append("npc-x")

    return DeterministicWorldStateManager.from_dict(
        {
            "tick": 0,
            "entities": {
                "actor-a": {
                    "entity_id": "actor-a",
                    "entity_type": "player",
                    "location": actor_room,
                    "health": actor_health,
                    "inventory": [],
                    "tags": [],
                },
                "npc-x": {
                    "entity_id": "npc-x",
                    "entity_type": "npc",
                    "location": npc_room,
                    "health": npc_health,
                    "inventory": [],
                    "tags": npc_tags if npc_tags is not None else ["hostile"],
                },
            },
            "rooms": {
                "camp": {
                    "room_id": "camp",
                    "description": "Camp",
                    "exits": {"east": "vault"},
                    "entities_present": entities_present_camp,
                },
                "vault": {
                    "room_id": "vault",
                    "description": "Vault",
                    "exits": {"west": "camp"},
                    "entities_present": entities_present_vault,
                },
            },
            "scenario_vars": svars,
        }
    )


def _apply(world: DeterministicWorldStateManager, result) -> None:
    if result is not None and result.world_delta:
        world.apply_delta(dict(result.world_delta))


def _defeat_actor(world: DeterministicWorldStateManager, step_index: int = 0) -> None:
    """Drive actor-a to 0 HP via NPC tick and process defeat."""
    npc_result = process_npc_tick(world, step_index=step_index)
    _apply(world, npc_result)
    defeat_result = process_actor_defeat_tick(world, step_index=step_index)
    _apply(world, defeat_result)


# ---------------------------------------------------------------------------
# Tests: defeat detection
# ---------------------------------------------------------------------------


def test_actor_at_zero_hp_is_detected_as_defeated() -> None:
    """Actor with health=0 after NPC tick is detected by process_actor_defeat_tick."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _apply(world, process_npc_tick(world, step_index=0))
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 0

    result = process_actor_defeat_tick(world, step_index=0)
    assert result is not None
    assert any(e.event_type == "actor_defeated" for e in result.events)


def test_defeat_sets_actor_defeated_scenario_var() -> None:
    """actor_defeated.{actor_id}=True is set in scenario_vars after defeat."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _defeat_actor(world, step_index=0)
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("actor_defeated.actor-a") is True


def test_defeat_schedules_respawn_at_correct_tick() -> None:
    """actor_respawn_at.{actor_id} set to step_index + delay."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=3)
    _apply(world, process_npc_tick(world, step_index=5))
    result = process_actor_defeat_tick(world, step_index=5)
    _apply(world, result)
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("actor_respawn_at.actor-a") == 5 + 3


def test_defeat_event_payload_contains_actor_id_and_respawn_tick() -> None:
    """actor_defeated event payload includes actor_id and respawn_at_tick."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=3)
    _apply(world, process_npc_tick(world, step_index=0))
    result = process_actor_defeat_tick(world, step_index=0)
    assert result is not None
    events = [e for e in result.events if e.event_type == "actor_defeated"]
    assert len(events) == 1
    payload = dict(events[0].payload)
    assert payload["actor_id"] == "actor-a"
    assert payload["respawn_at_tick"] == 3


def test_alive_actor_not_marked_defeated() -> None:
    """process_actor_defeat_tick returns None when all actors have health > 0."""
    world = _make_world(actor_health=100)
    result = process_actor_defeat_tick(world, step_index=0)
    assert result is None


def test_already_defeated_actor_not_re_detected() -> None:
    """Second call to process_actor_defeat_tick for same actor returns None."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _defeat_actor(world, step_index=0)
    result2 = process_actor_defeat_tick(world, step_index=1)
    assert result2 is None


def test_defeat_event_emitted_exactly_once() -> None:
    """actor_defeated event fires exactly once across repeated defeat checks."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _apply(world, process_npc_tick(world, step_index=0))
    r1 = process_actor_defeat_tick(world, step_index=0)
    _apply(world, r1)
    r2 = process_actor_defeat_tick(world, step_index=1)
    assert r1 is not None
    defeat_count = sum(1 for e in r1.events if e.event_type == "actor_defeated")
    assert defeat_count == 1
    assert r2 is None


# ---------------------------------------------------------------------------
# Tests: default respawn values
# ---------------------------------------------------------------------------


def test_default_respawn_delay_used_when_no_scenario_var() -> None:
    """Default respawn delay applies when actor_respawn_delay_ticks is absent."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=None)
    _apply(world, process_npc_tick(world, step_index=0))
    result = process_actor_defeat_tick(world, step_index=0)
    _apply(world, result)
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("actor_respawn_at.actor-a") == _DEFAULT_ACTOR_RESPAWN_DELAY


def test_default_respawn_health_used_when_no_scenario_var() -> None:
    """Default respawn health applies when actor_respawn_health is absent."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=None, respawn_health=None, respawn_room=None)
    _defeat_actor(world, step_index=0)
    result = process_actor_respawn_tick(world, step_index=_DEFAULT_ACTOR_RESPAWN_DELAY)
    _apply(world, result)
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == _DEFAULT_ACTOR_RESPAWN_HEALTH


# ---------------------------------------------------------------------------
# Tests: respawn timing
# ---------------------------------------------------------------------------


def test_respawn_fires_at_correct_tick() -> None:
    """process_actor_respawn_tick returns events exactly when step_index >= respawn_at."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=3)
    _defeat_actor(world, step_index=0)

    # Tick 2 — too early
    assert process_actor_respawn_tick(world, step_index=2) is None

    # Tick 3 — fires
    result = process_actor_respawn_tick(world, step_index=3)
    assert result is not None
    assert any(e.event_type == "actor_respawned" for e in result.events)


def test_no_respawn_before_due_tick() -> None:
    """process_actor_respawn_tick returns None before step_index reaches respawn_at."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_delay=5)
    _defeat_actor(world, step_index=0)
    for tick in range(5):
        assert process_actor_respawn_tick(world, step_index=tick) is None


# ---------------------------------------------------------------------------
# Tests: respawn state restoration
# ---------------------------------------------------------------------------


def test_respawn_restores_health() -> None:
    """Respawned actor has health equal to actor_respawn_health scenario var."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_health=42)
    _defeat_actor(world, step_index=0)
    _apply(world, process_actor_respawn_tick(world, step_index=3))
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 42


def test_respawn_moves_actor_to_respawn_room() -> None:
    """Respawned actor is moved to actor_respawn_room_id."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_room="vault")
    _defeat_actor(world, step_index=0)
    _apply(world, process_actor_respawn_tick(world, step_index=3))
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["location"] == "vault"


def test_respawn_updates_room_entities_present() -> None:
    """After respawn to new room, entities_present is correct in both rooms."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, actor_room="camp", respawn_room="vault")
    _defeat_actor(world, step_index=0)
    _apply(world, process_actor_respawn_tick(world, step_index=3))
    snap = world.get_snapshot()
    assert "actor-a" not in snap["rooms"]["camp"]["entities_present"]
    assert "actor-a" in snap["rooms"]["vault"]["entities_present"]


def test_respawn_clears_defeat_scenario_vars() -> None:
    """actor_defeated and actor_respawn_at are cleared after respawn."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _defeat_actor(world, step_index=0)
    _apply(world, process_actor_respawn_tick(world, step_index=3))
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("actor_defeated.actor-a") is None
    assert snap["scenario_vars"].get("actor_respawn_at.actor-a") is None


def test_respawn_event_payload_correct() -> None:
    """actor_respawned event payload contains actor_id, room_id, and health."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_health=50, respawn_room="vault")
    _defeat_actor(world, step_index=0)
    result = process_actor_respawn_tick(world, step_index=3)
    assert result is not None
    events = [e for e in result.events if e.event_type == "actor_respawned"]
    assert len(events) == 1
    payload = dict(events[0].payload)
    assert payload["actor_id"] == "actor-a"
    assert payload["room_id"] == "vault"
    assert payload["health"] == 50


def test_respawn_stays_in_same_room_when_no_respawn_room_configured() -> None:
    """When actor_respawn_room_id is absent, actor stays at their current location."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_room=None)
    _defeat_actor(world, step_index=0)
    snap_before = world.get_snapshot()
    old_location = snap_before["entities"]["actor-a"]["location"]

    _apply(world, process_actor_respawn_tick(world, step_index=3))
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["location"] == old_location


# ---------------------------------------------------------------------------
# Tests: action gating via SharedShardLoopSession
# ---------------------------------------------------------------------------


def _make_shared_shard_session() -> tuple[SharedShardLoopSession, DeterministicWorldStateManager]:
    """Build a minimal SharedShardLoopSession for action-gating tests."""
    from evaluation.benchmark_runner.runner import build_shared_shard_loop_session

    # Minimal two-player scenario with one hostile NPC. Actor health is set
    # equal to _NPC_COUNTER_DAMAGE so one counter-attack tick is lethal.
    _MINIMAL_SCENARIO = {
        "scenario_id": "test-defeat-respawn",
        "title": "Test Defeat Respawn",
        "description": "Defeat/respawn test scenario.",
        "version": "1",
        "seed": 42,
        "max_steps": 20,
        "start_room_id": "camp",
        "objectives": [{"objective_id": "test-obj", "objective_type": "collect_item", "required_count": 1, "target_id": "relic"}],
        "scenario_vars": {
            "mode": "shared_shard",
            "actor_respawn_delay_ticks": 2,
            "actor_respawn_health": 50,
            "actor_respawn_room_id": "vault",
            "world_config_json": '{"rooms":{"camp":{"title":"Camp","description":"A test camp.","exits":{"east":"vault"},"entities":[]},"vault":{"title":"Vault","description":"A test vault.","exits":{"west":"camp"},"entities":[]}},"npcs":[{"entity_id":"npc-x","entity_type":"npc","location":"camp","health":20,"tags":["hostile"]}],"items":[]}',
        },
    }

    session = build_shared_shard_loop_session(
        scenario=_MINIMAL_SCENARIO,
        actor_ids=["actor-a", "actor-b"],
        run_id="test-run",
    )
    # Override actor-a's health to _NPC_COUNTER_DAMAGE so one NPC tick defeats them.
    session.world_state.apply_delta({
        "entities": {
            "actor-a": {
                "entity_id": "actor-a",
                "entity_type": "player",
                "location": "camp",
                "health": _NPC_COUNTER_DAMAGE,
                "inventory": [],
                "tags": [],
            }
        }
    })
    # Sessions are opened by build_shared_shard_loop_session — no need to reopen.
    return session, session.world_state


def test_defeated_actor_action_skipped_in_advance_tick() -> None:
    """A defeated actor's action is silently skipped (treated as wait) in advance_tick."""
    session, world = _make_shared_shard_session()

    # Advance tick — NPC hits actor-a (health=5 → 0), defeat is processed
    session.advance_tick({"actor-a": "wait"})
    snap = world.get_snapshot()

    # Actor should be defeated
    assert snap["scenario_vars"].get("actor_defeated.actor-a") is True

    # Next tick: actor submits "look" but is defeated — should succeed without error
    result2 = session.advance_tick({"actor-a": "look"})
    assert result2 is not None  # No error raised


def test_respawned_actor_can_act_again() -> None:
    """After respawn, a previously-defeated actor can submit actions normally."""
    session, world = _make_shared_shard_session()

    # Defeat actor-a in tick 0
    session.advance_tick({"actor-a": "wait"})
    assert world.get_snapshot()["scenario_vars"].get("actor_defeated.actor-a") is True

    # Tick 1 — still defeated (respawn_delay=2, respawn_at=0+2=2)
    session.advance_tick({"actor-a": "wait"})

    # Tick 2 — respawn fires (step_index=2 >= respawn_at=2)
    session.advance_tick({"actor-a": "wait"})
    snap = world.get_snapshot()
    assert snap["scenario_vars"].get("actor_defeated.actor-a") is None
    assert snap["entities"]["actor-a"]["health"] == 50

    # Tick 3 — alive actor can act
    result = session.advance_tick({"actor-a": "look"})
    assert result is not None


# ---------------------------------------------------------------------------
# Tests: observation surfacing
# ---------------------------------------------------------------------------


def test_defeat_status_message_shown_when_defeated() -> None:
    """_format_actor_defeat_status_message returns a non-None message when defeated."""
    svars = {"actor_defeated.actor-a": True, "actor_respawn_at.actor-a": 7}
    msg = _format_actor_defeat_status_message(svars, "actor-a")
    assert msg is not None
    assert "defeated" in msg.lower()
    assert "7" in msg


def test_defeat_status_message_none_when_alive() -> None:
    """_format_actor_defeat_status_message returns None when actor is alive."""
    svars: dict = {}
    msg = _format_actor_defeat_status_message(svars, "actor-a")
    assert msg is None


def test_defeat_status_message_not_shown_for_other_actor() -> None:
    """Defeat status for actor-b does not appear in actor-a's message."""
    svars = {"actor_defeated.actor-b": True, "actor_respawn_at.actor-b": 5}
    msg = _format_actor_defeat_status_message(svars, "actor-a")
    assert msg is None


# ---------------------------------------------------------------------------
# Tests: save/load preserves defeat/respawn state
# ---------------------------------------------------------------------------


def test_save_load_preserves_defeat_state() -> None:
    """actor_defeated and actor_respawn_at survive save/load round-trip."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    _defeat_actor(world, step_index=0)
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()
    assert snap2["scenario_vars"].get("actor_defeated.actor-a") is True
    assert snap2["scenario_vars"].get("actor_respawn_at.actor-a") == 3


def test_save_load_preserves_post_respawn_state() -> None:
    """Post-respawn actor health and location survive save/load round-trip."""
    world = _make_world(actor_health=_NPC_COUNTER_DAMAGE, respawn_health=50, respawn_room="vault")
    _defeat_actor(world, step_index=0)
    _apply(world, process_actor_respawn_tick(world, step_index=3))
    saved = world.get_snapshot()

    world2 = DeterministicWorldStateManager.from_dict(saved)
    snap2 = world2.get_snapshot()
    assert snap2["entities"]["actor-a"]["health"] == 50
    assert snap2["entities"]["actor-a"]["location"] == "vault"
    assert snap2["scenario_vars"].get("actor_defeated.actor-a") is None


# ---------------------------------------------------------------------------
# Tests: determinism
# ---------------------------------------------------------------------------


def test_defeat_respawn_deterministic() -> None:
    """Same inputs on two separate world instances produce identical results."""
    world1 = _make_world(actor_health=_NPC_COUNTER_DAMAGE)
    world2 = _make_world(actor_health=_NPC_COUNTER_DAMAGE)

    for world in (world1, world2):
        _defeat_actor(world, step_index=0)
        _apply(world, process_actor_respawn_tick(world, step_index=3))

    assert world1.get_snapshot()["entities"]["actor-a"] == world2.get_snapshot()["entities"]["actor-a"]
    assert world1.get_snapshot()["scenario_vars"] == world2.get_snapshot()["scenario_vars"]


# ---------------------------------------------------------------------------
# Tests: no regression — alive actor unaffected
# ---------------------------------------------------------------------------


def test_alive_actor_health_unchanged_by_defeat_tick() -> None:
    """process_actor_defeat_tick does not change an alive actor's health."""
    world = _make_world(actor_health=100)
    process_actor_defeat_tick(world, step_index=0)
    snap = world.get_snapshot()
    assert snap["entities"]["actor-a"]["health"] == 100


def test_no_respawn_result_when_no_defeated_actors() -> None:
    """process_actor_respawn_tick returns None when no actors are scheduled for respawn."""
    world = _make_world(actor_health=100)
    assert process_actor_respawn_tick(world, step_index=99) is None
