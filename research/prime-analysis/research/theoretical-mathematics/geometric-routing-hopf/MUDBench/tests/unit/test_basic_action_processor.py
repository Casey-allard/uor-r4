from __future__ import annotations

from core.action_processor import ActionRequest, normalize_arguments
from core.event_logger import EventLogger, EventRecord
from core.run_state import RunStatus
from core.simulation_controller import SimulationController
from world.rooms.room_graph import DeterministicRoomGraph
from world.state.basic_action_processor import BasicDeterministicActionProcessor
from world.state.spawn_manager import DeterministicSpawnManager, SpawnRequest
from world.state.world_bootstrap import bootstrap_world_state_manager
from world.state.world_state import DeterministicWorldStateManager


class InMemoryEventLogger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def _room_graph() -> DeterministicRoomGraph:
    return DeterministicRoomGraph.from_dict(
        {
            "rooms": {
                "room-a": {
                    "title": "Room A",
                    "description": "Start room.",
                    "exits": {"east": "room-b"},
                    "entities": [],
                },
                "room-b": {
                    "title": "Room B",
                    "description": "Destination room.",
                    "exits": {"west": "room-a"},
                    "entities": [],
                },
            }
        }
    )


def _world_with_actors(
    requests: tuple[SpawnRequest, ...],
) -> DeterministicWorldStateManager:
    world = bootstrap_world_state_manager(_room_graph(), seed=5)
    DeterministicSpawnManager(seed=0).place_actors(world, requests)
    return DeterministicWorldStateManager.from_json(world.to_json())


def _add_entity(
    world: DeterministicWorldStateManager,
    *,
    entity_id: str,
    entity_type: str,
    room_id: str,
) -> None:
    snapshot = world.get_snapshot()
    room_payload = snapshot["rooms"][room_id]
    entities_present = sorted(set(room_payload.get("entities_present", [])) | {entity_id})
    world.apply_delta(
        {
            "entities": {
                entity_id: {
                    "entity_id": entity_id,
                    "entity_type": entity_type,
                    "location": room_id,
                }
            },
            "rooms": {
                room_id: {
                    **room_payload,
                    "entities_present": entities_present,
                }
            },
        }
    )


def test_validate_action_supports_only_wait_look_move() -> None:
    processor = BasicDeterministicActionProcessor()

    assert processor.validate_action(ActionRequest(actor_id="agent-1", action_type="wait"))
    assert processor.validate_action(ActionRequest(actor_id="agent-1", action_type="look"))
    assert processor.validate_action(
        ActionRequest(
            actor_id="agent-1",
            action_type="move",
            arguments=normalize_arguments({"direction": "east"}),
        )
    )

    assert not processor.validate_action(ActionRequest(actor_id="agent-1", action_type="attack"))
    assert not processor.validate_action(ActionRequest(actor_id="agent-1", action_type="move"))
    assert not processor.validate_action(
        ActionRequest(
            actor_id="agent-1",
            action_type="wait",
            arguments=normalize_arguments({"extra": "value"}),
        )
    )


def test_process_actions_uses_deterministic_actor_order() -> None:
    world = _world_with_actors(
        (
            SpawnRequest(actor_id="agent-b", actor_type="agent", preferred_room_id="room-a"),
            SpawnRequest(actor_id="agent-a", actor_type="agent", preferred_room_id="room-a"),
        )
    )
    processor = BasicDeterministicActionProcessor()
    actions = (
        ActionRequest(actor_id="agent-b", action_type="wait"),
        ActionRequest(actor_id="agent-a", action_type="wait"),
    )

    results = processor.process_actions(actions, world, step_index=0)

    assert tuple(result.events[0].actor_id for result in results) == ("agent-a", "agent-b")
    assert tuple(result.events[0].event_type for result in results) == ("action_wait", "action_wait")


def test_move_emits_event_and_declarative_delta_without_direct_mutation() -> None:
    world = _world_with_actors(
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),)
    )
    processor = BasicDeterministicActionProcessor()
    before = world.to_json()

    result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "east"}),
            ),
        ),
        world,
        step_index=1,
    )[0]

    payload = dict(result.events[0].payload)
    assert result.accepted is True
    assert result.events[0].event_type == "action_move"
    assert payload["direction"] == "east"
    assert payload["source_room_id"] == "room-a"
    assert payload["destination_room_id"] == "room-b"
    assert world.to_json() == before
    assert result.world_delta

    world.apply_delta(dict(result.world_delta))
    assert world.get_snapshot()["entities"]["agent-1"]["location"] == "room-b"


def test_invalid_move_is_rejected_and_is_no_op() -> None:
    world = _world_with_actors(
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),)
    )
    processor = BasicDeterministicActionProcessor()
    before = world.to_json()

    result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "north"}),
            ),
        ),
        world,
        step_index=2,
    )[0]

    assert result.accepted is False
    assert result.events[0].event_type == "action_rejected"
    assert dict(result.events[0].payload)["reason"] == "exit_not_found"
    assert result.world_delta == ()
    assert world.to_json() == before


def test_look_returns_room_context_without_state_mutation() -> None:
    world = _world_with_actors(
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),)
    )
    processor = BasicDeterministicActionProcessor()
    before = world.to_json()

    result = processor.process_actions(
        (ActionRequest(actor_id="agent-1", action_type="look"),),
        world,
        step_index=3,
    )[0]

    payload = dict(result.events[0].payload)
    assert result.accepted is True
    assert result.events[0].event_type == "action_look"
    assert payload["location"] == "room-a"
    assert payload["visible_exits"] == ("east",)
    assert dict(result.world_delta)["scenario_vars"]["reveal.agent-1.room-a"] == 1
    assert world.to_json() == before


def test_controller_integrates_with_basic_action_processor() -> None:
    world = _world_with_actors(
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),)
    )
    logger = InMemoryEventLogger()
    processor = BasicDeterministicActionProcessor()
    controller = SimulationController(
        world_state_manager=world,
        action_processor=processor,
        event_logger=logger,
        seed=13,
        max_steps=2,
    )
    controller.initialize()

    first = controller.step(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "east"}),
            ),
        )
    )
    second = controller.step(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "north"}),
            ),
        )
    )

    assert first.status is RunStatus.RUNNING
    assert second.status is RunStatus.TERMINATED
    assert world.get_snapshot()["entities"]["agent-1"]["location"] == "room-b"
    assert logger.records()[0].event_type == "run_initialized"
    assert logger.records()[1].event_type == "action_move"
    assert logger.records()[2].event_type == "step_completed"
    assert logger.records()[3].event_type == "action_rejected"
    assert logger.records()[4].event_type == "step_completed"


def test_give_action_emits_social_dependency_event_and_reward_flow() -> None:
    world = _world_with_actors(
        (SpawnRequest(actor_id="agent-1", actor_type="agent", preferred_room_id="room-a"),)
    )
    _add_entity(world, entity_id="trade-token", entity_type="item", room_id="room-a")
    _add_entity(world, entity_id="trader", entity_type="npc", room_id="room-a")
    world.apply_delta(
        {
            "scenario_vars": {
                "trade_effects_json": (
                    '{"trade-token|trader":{"effect_id":"trade-gate",'
                    '"reward_item_id":"artifact","reward_entity_type":"item"}}'
                )
            }
        }
    )
    processor = BasicDeterministicActionProcessor()
    take_result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="take",
                arguments=normalize_arguments({"item_id": "trade-token"}),
            ),
        ),
        world,
        step_index=0,
    )[0]
    assert take_result.accepted is True
    world.apply_delta(dict(take_result.world_delta))
    give_result = processor.process_actions(
        (
            ActionRequest(
                actor_id="agent-1",
                action_type="give",
                arguments=normalize_arguments({"item_id": "trade-token", "target_id": "trader"}),
            ),
        ),
        world,
        step_index=1,
    )[0]

    assert give_result.accepted is True
    assert [event.event_type for event in give_result.events] == ["action_give", "dependency_unlocked"]
    assert dict(give_result.events[0].payload)["target_id"] == "trader"
    world.apply_delta(dict(give_result.world_delta))
    assert world.get_snapshot()["entities"]["artifact"]["location"] == "room-a"
