from __future__ import annotations

from core.action_processor import ActionProcessingResult, ActionProcessor, ActionRequest
from core.event_logger import EventLogger, EventRecord, normalize_payload
from core.run_state import RunStatus
from core.simulation_controller import SimulationController
from core.world_state_manager import WorldStateManager


class InMemoryWorldState(WorldStateManager):
    def __init__(self) -> None:
        self._state: dict[str, object] = {"updates": 0}

    def get_snapshot(self) -> dict[str, object]:
        return dict(self._state)

    def apply_delta(self, delta: dict[str, object]) -> None:
        self._state.update(delta)

    def reset(self) -> None:
        self._state = {"updates": 0}


class InMemoryEventLogger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


class DeterministicActionProcessor(ActionProcessor):
    def validate_action(self, action: ActionRequest) -> bool:
        return bool(action.actor_id and action.action_type)

    def process_actions(
        self,
        actions: tuple[ActionRequest, ...],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        results: list[ActionProcessingResult] = []
        for index, action in enumerate(actions):
            event = EventRecord(
                step_index=step_index,
                event_type="processed",
                actor_id=action.actor_id,
                payload=normalize_payload({"index": index, "type": action.action_type}),
            )
            results.append(
                ActionProcessingResult(
                    accepted=self.validate_action(action),
                    events=(event,),
                    world_delta=(("updates", step_index + index + 1),),
                )
            )
        return tuple(results)


class RejectedDeltaActionProcessor(ActionProcessor):
    def validate_action(self, action: ActionRequest) -> bool:
        del action
        return False

    def process_actions(
        self,
        actions: tuple[ActionRequest, ...],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        del world_state
        results: list[ActionProcessingResult] = []
        for action in actions:
            results.append(
                ActionProcessingResult(
                    accepted=False,
                    events=(
                        EventRecord(
                            step_index=step_index,
                            event_type="action_rejected",
                            actor_id=action.actor_id,
                            payload=normalize_payload({"reason": "invalid_action"}),
                        ),
                    ),
                    world_delta=(("updates", 999),),
                )
            )
        return tuple(results)


def build_controller(seed: int = 11, max_steps: int = 2) -> tuple[SimulationController, InMemoryEventLogger]:
    event_logger = InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=InMemoryWorldState(),
        action_processor=DeterministicActionProcessor(),
        event_logger=event_logger,
        seed=seed,
        max_steps=max_steps,
    )
    return controller, event_logger


def test_simulation_controller_lifecycle() -> None:
    controller, logger = build_controller()
    run_state = controller.initialize()

    assert run_state.status is RunStatus.RUNNING
    assert logger.records()[0].event_type == "run_initialized"

    outcome_one = controller.step((ActionRequest(actor_id="a", action_type="move"),))
    assert outcome_one.processed_actions == 1
    assert outcome_one.status is RunStatus.RUNNING
    assert outcome_one.emitted_events[-1].event_type == "step_completed"
    assert dict(outcome_one.emitted_events[-1].payload) == {
        "domain_event_count": 1,
        "processed_actions": 1,
    }

    outcome_two = controller.step((ActionRequest(actor_id="b", action_type="inspect"),))
    assert outcome_two.processed_actions == 1
    assert outcome_two.status is RunStatus.TERMINATED
    assert outcome_two.emitted_events[-1].event_type == "step_completed"

    outcome_three = controller.step(())
    assert outcome_three.processed_actions == 0
    assert outcome_three.status is RunStatus.TERMINATED
    assert tuple(event.event_type for event in logger.records()) == (
        "run_initialized",
        "processed",
        "step_completed",
        "processed",
        "step_completed",
    )


def test_simulation_controller_requires_initialize_before_step() -> None:
    controller, _ = build_controller()
    try:
        controller.step(())
    except RuntimeError as exc:  # explicit behavior check
        assert "initialized" in str(exc)
    else:
        raise AssertionError("step() should require initialize()")


def test_simulation_controller_seeded_determinism() -> None:
    actions = (
        ActionRequest(actor_id="a", action_type="move"),
        ActionRequest(actor_id="b", action_type="inspect"),
    )

    first_controller, first_logger = build_controller(seed=7, max_steps=1)
    second_controller, second_logger = build_controller(seed=7, max_steps=1)

    first_controller.initialize()
    second_controller.initialize()

    first_outcome = first_controller.step(actions)
    second_outcome = second_controller.step(actions)

    assert first_outcome == second_outcome
    assert first_logger.records() == second_logger.records()


def test_simulation_controller_does_not_apply_world_delta_for_rejected_results() -> None:
    world = InMemoryWorldState()
    logger = InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=world,
        action_processor=RejectedDeltaActionProcessor(),
        event_logger=logger,
        seed=17,
        max_steps=1,
    )
    controller.initialize()
    assert world.get_snapshot()["updates"] == 0

    outcome = controller.step((ActionRequest(actor_id="agent-a", action_type="move"),))

    assert outcome.processed_actions == 1
    assert world.get_snapshot()["updates"] == 0
    assert tuple(event.event_type for event in outcome.emitted_events) == (
        "action_rejected",
        "step_completed",
    )
    assert dict(outcome.emitted_events[-1].payload) == {
        "domain_event_count": 1,
        "processed_actions": 1,
    }


def test_simulation_controller_applies_world_delta_for_accepted_results() -> None:
    world = InMemoryWorldState()
    logger = InMemoryEventLogger()
    controller = SimulationController(
        world_state_manager=world,
        action_processor=DeterministicActionProcessor(),
        event_logger=logger,
        seed=19,
        max_steps=1,
    )
    controller.initialize()

    controller.step((ActionRequest(actor_id="agent-a", action_type="move"),))

    assert world.get_snapshot()["updates"] == 1


def test_simulation_controller_emits_step_completed_for_zero_action_step() -> None:
    controller, logger = build_controller(max_steps=1)
    controller.initialize()

    outcome = controller.step(())

    assert outcome.processed_actions == 0
    assert tuple(event.event_type for event in outcome.emitted_events) == ("step_completed",)
    assert dict(outcome.emitted_events[0].payload) == {
        "domain_event_count": 0,
        "processed_actions": 0,
    }
    assert logger.records()[-1] == outcome.emitted_events[0]
