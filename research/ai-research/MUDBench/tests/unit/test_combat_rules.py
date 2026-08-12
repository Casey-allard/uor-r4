from __future__ import annotations

from core.action_processor import ActionRequest, normalize_arguments
from world.combat.combat_rules import DeterministicCombatRules
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-a": {
                    "title": "Room A",
                    "description": "Arena A.",
                    "exits": {"east": "room-b"},
                    "entities": [],
                },
                "room-b": {
                    "title": "Room B",
                    "description": "Arena B.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
            }
        }
    )


def _world_with_combatants(
    *,
    attacker_room: str = "room-a",
    target_room: str = "room-a",
    target_health: int | None = 30,
) -> DeterministicWorldStateManager:
    world = bootstrap_world_state_manager(_room_graph(), seed=17)
    DeterministicSpawnManager(seed=0).place_actors(
        world,
        (
            SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id=attacker_room),
            SpawnRequest(
                actor_id="npc-1",
                actor_type="npc",
                preferred_room_id=target_room,
                health=target_health,
            ),
        ),
    )
    return DeterministicWorldStateManager.from_json(world.to_json())


def _apply_resolution_delta(world: DeterministicWorldStateManager, result) -> None:
    if result.world_delta is not None:
        world.apply_delta(result.world_delta)


def test_attack_reduces_health_by_fixed_damage() -> None:
    world = _world_with_combatants(target_health=30)
    rules = DeterministicCombatRules(damage=10)
    before = world.to_json()

    result = rules.apply_attack(world, attacker_id="agent-1", target_id="npc-1")

    assert result.success is True
    assert result.damage == 10
    assert result.resulting_health == 20
    assert result.world_delta is not None
    assert world.to_json() == before

    _apply_resolution_delta(world, result)
    snapshot = world.get_snapshot()
    assert snapshot["entities"]["npc-1"]["health"] == 20


def test_attack_clamps_health_to_zero() -> None:
    world = _world_with_combatants(target_health=7)
    rules = DeterministicCombatRules(damage=10)
    before = world.to_json()

    result = rules.apply_attack(world, attacker_id="agent-1", target_id="npc-1")

    assert result.success is True
    assert result.resulting_health == 0
    assert result.world_delta is not None
    assert world.to_json() == before
    _apply_resolution_delta(world, result)
    assert world.get_snapshot()["entities"]["npc-1"]["health"] == 0


def test_attack_rejects_target_in_different_room_without_mutation() -> None:
    world = _world_with_combatants(attacker_room="room-a", target_room="room-b", target_health=30)
    rules = DeterministicCombatRules(damage=10)
    before = world.to_json()

    result = rules.apply_attack(world, attacker_id="agent-1", target_id="npc-1")

    assert result.success is False
    assert result.reason == "target_not_in_same_room"
    assert world.to_json() == before


def test_attack_rejects_invalid_target_health_without_mutation() -> None:
    world = _world_with_combatants(target_health=None)
    rules = DeterministicCombatRules(damage=10)
    before = world.to_json()

    result = rules.apply_attack(world, attacker_id="agent-1", target_id="npc-1")

    assert result.success is False
    assert result.reason == "target_health_invalid"
    assert world.to_json() == before


def test_action_processor_integrates_attack_resolution() -> None:
    world = _world_with_combatants(target_health=30)
    processor = BasicDeterministicActionProcessor()

    result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="attack",
                arguments=normalize_arguments({"target_id": "npc-1"}),
            ),
        ),
        world,
        step_index=3,
    )[0]

    payload = dict(result.events[0].payload)
    assert result.accepted is True
    assert result.events[0].event_type == "action_attack"
    assert payload["target_id"] == "npc-1"
    assert payload["damage"] == 10
    assert payload["resulting_health"] == 20
    assert result.world_delta
    world.apply_delta(dict(result.world_delta))
    assert world.get_snapshot()["entities"]["npc-1"]["health"] == 20


def test_action_processor_rejects_attack_with_missing_target_argument() -> None:
    world = _world_with_combatants(target_health=30)
    processor = BasicDeterministicActionProcessor()
    before = world.to_json()

    result = processor.process_actions(
        (ActionRequest(actor_id="agent-1", action_type="attack"),),
        world,
        step_index=2,
    )[0]

    assert result.accepted is False
    assert result.events[0].event_type == "action_rejected"
    assert dict(result.events[0].payload)["reason"] == "attack_requires_target_id"
    assert world.to_json() == before


def test_attack_resolution_is_deterministic_for_identical_inputs() -> None:
    first_world = _world_with_combatants(target_health=30)
    second_world = _world_with_combatants(target_health=30)
    rules = DeterministicCombatRules(damage=10)

    first = rules.apply_attack(first_world, attacker_id="agent-1", target_id="npc-1")
    second = rules.apply_attack(second_world, attacker_id="agent-1", target_id="npc-1")
    _apply_resolution_delta(first_world, first)
    _apply_resolution_delta(second_world, second)

    assert first == second
    assert first_world.to_json() == second_world.to_json()
