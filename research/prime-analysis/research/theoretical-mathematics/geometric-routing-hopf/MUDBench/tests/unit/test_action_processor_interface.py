from __future__ import annotations

from core.action_processor import ActionProcessingResult, ActionProcessor, ActionRequest, normalize_arguments
from core.event_logger import EventRecord, normalize_payload
from core.world_state_manager import WorldStateManager


class InMemoryWorldState(WorldStateManager):
    def __init__(self) -> None:
        self._state: dict[str, object] = {}

    def get_snapshot(self) -> dict[str, object]:
        return dict(self._state)

    def apply_delta(self, delta: dict[str, object]) -> None:
        self._state.update(delta)

    def reset(self) -> None:
        self._state.clear()


class EchoActionProcessor(ActionProcessor):
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
        for action in actions:
            accepted = self.validate_action(action)
            events = (
                EventRecord(
                    step_index=step_index,
                    event_type="action_processed",
                    actor_id=action.actor_id,
                    payload=normalize_payload({"action_type": action.action_type, "accepted": accepted}),
                ),
            )
            results.append(ActionProcessingResult(accepted=accepted, events=events))
        return tuple(results)


def test_normalize_arguments_is_sorted() -> None:
    assert normalize_arguments({"b": 2, "a": 1}) == (("a", 1), ("b", 2))


def test_action_processor_contract_preserves_order() -> None:
    processor = EchoActionProcessor()
    world_state = InMemoryWorldState()
    actions = (
        ActionRequest(actor_id="alpha", action_type="move"),
        ActionRequest(actor_id="beta", action_type="inspect"),
    )

    results = processor.process_actions(actions, world_state, step_index=0)

    assert len(results) == 2
    assert results[0].events[0].actor_id == "alpha"
    assert results[1].events[0].actor_id == "beta"
