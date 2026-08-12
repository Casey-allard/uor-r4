from __future__ import annotations

from core.action_processor import ActionProcessingResult, ActionProcessor, ActionRequest
from core.event_logger import EventLogger, EventRecord, normalize_payload
from core.simulation_controller import SimulationController
from core.world_state_manager import WorldStateManager


class InMemoryWorldState(WorldStateManager):
    def __init__(self) -> None:
        self._state: dict[str, object] = {"counter": 0}

    def get_snapshot(self) -> dict[str, object]:
        return dict(self._state)

    def apply_delta(self, delta: dict[str, object]) -> None:
        self._state.update(delta)

    def reset(self) -> None:
        self._state = {"counter": 0}


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
        return True

    def process_actions(
        self,
        actions: tuple[ActionRequest, ...],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        results: list[ActionProcessingResult] = []
        for idx, action in enumerate(actions):
            results.append(
                ActionProcessingResult(
                    accepted=True,
                    events=(
                        EventRecord(
                            step_index=step_index,
                            event_type="deterministic_processed",
                            actor_id=action.actor_id,
                            payload=normalize_payload({"idx": idx, "action": action.action_type}),
                        ),
                    ),
                    world_delta=(("counter", step_index + idx + 1),),
                )
            )
        return tuple(results)


def run_once(seed: int) -> tuple[tuple[EventRecord, ...], tuple[dict[str, object], ...]]:
    event_logger = InMemoryEventLogger()
    world_state = InMemoryWorldState()
    controller = SimulationController(
        world_state_manager=world_state,
        action_processor=DeterministicActionProcessor(),
        event_logger=event_logger,
        seed=seed,
        max_steps=2,
    )
    actions = (
        ActionRequest(actor_id="agent-1", action_type="wait"),
        ActionRequest(actor_id="agent-2", action_type="inspect"),
    )

    controller.initialize()
    snapshots: list[dict[str, object]] = []
    for _ in range(2):
        controller.step(actions)
        snapshots.append(world_state.get_snapshot())

    return event_logger.records(), tuple(snapshots)


def test_determinism_scaffold_same_seed_same_outputs() -> None:
    first_events, first_snapshots = run_once(seed=42)
    second_events, second_snapshots = run_once(seed=42)

    assert first_events == second_events
    assert first_snapshots == second_snapshots
