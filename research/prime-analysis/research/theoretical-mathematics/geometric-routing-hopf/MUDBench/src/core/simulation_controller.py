"""Deterministic simulation controller skeleton."""

from __future__ import annotations

import random
from dataclasses import dataclass
from typing import Sequence

from .action_processor import ActionProcessor, ActionRequest
from .event_logger import EventLogger, EventRecord, normalize_payload
from .run_state import RunState, RunStatus
from .world_state_manager import WorldStateManager


@dataclass(frozen=True, slots=True)
class StepOutcome:
    """Result envelope for a single simulation step."""

    step_index: int
    processed_actions: int
    emitted_events: tuple[EventRecord, ...]
    status: RunStatus


class SimulationController:
    """Coordinates deterministic step progression between core interfaces."""

    def __init__(
        self,
        *,
        world_state_manager: WorldStateManager,
        action_processor: ActionProcessor,
        event_logger: EventLogger,
        seed: int,
        max_steps: int,
        run_id: str = "run-0",
    ) -> None:
        if max_steps <= 0:
            raise ValueError("max_steps must be greater than zero")

        self._world_state_manager = world_state_manager
        self._action_processor = action_processor
        self._event_logger = event_logger
        self._rng = random.Random(seed)
        self.run_state = RunState(run_id=run_id, seed=seed, max_steps=max_steps)

    def initialize(self) -> RunState:
        """Prepare interfaces and mark run as active."""
        if self.run_state.status != RunStatus.INITIALIZED:
            raise RuntimeError("SimulationController is already initialized")

        self._world_state_manager.reset()
        self._event_logger.reset()
        self._event_logger.log(
            EventRecord(
                step_index=self.run_state.step_index,
                event_type="run_initialized",
                payload=normalize_payload(
                    {"max_steps": self.run_state.max_steps, "seed": self.run_state.seed}
                ),
            )
        )
        self.run_state.status = RunStatus.RUNNING
        return self.run_state

    def step(self, actions: Sequence[ActionRequest]) -> StepOutcome:
        """Execute a single deterministic step over ordered actions."""
        if self.run_state.status == RunStatus.INITIALIZED:
            raise RuntimeError("SimulationController must be initialized before stepping")

        if self.run_state.status == RunStatus.TERMINATED:
            return StepOutcome(
                step_index=self.run_state.step_index,
                processed_actions=0,
                emitted_events=(),
                status=self.run_state.status,
            )

        # Sort deterministically by actor_id (primary key matches process_actions ordering).
        # Process one action at a time, applying each delta to world state before the next
        # action is evaluated.  This prevents lost-update races when multiple actors modify
        # the same room in the same tick (e.g. both moving out of the same source room).
        ordered_actions = tuple(sorted(actions, key=lambda r: str(r.actor_id)))

        results = []
        emitted_events: list[EventRecord] = []
        for action in ordered_actions:
            single_results = self._action_processor.process_actions(
                (action,),
                self._world_state_manager,
                step_index=self.run_state.step_index,
            )
            for result in single_results:
                results.append(result)
                emitted_events.extend(result.events)
                if result.accepted and result.world_delta:
                    self._world_state_manager.apply_delta(dict(result.world_delta))

        # Runtime contract: every executed step emits a deterministic step marker so replay can
        # reconstruct true step progression even when no domain events were produced.
        emitted_events.append(
            EventRecord(
                step_index=self.run_state.step_index,
                event_type="step_completed",
                payload=normalize_payload(
                    {
                        "domain_event_count": len(emitted_events),
                        "processed_actions": len(results),
                    }
                ),
            )
        )

        for event in emitted_events:
            self._event_logger.log(event)

        completed_step = self.run_state.step_index
        self.run_state.step_index += 1
        if self.run_state.step_index >= self.run_state.max_steps:
            self.terminate()

        return StepOutcome(
            step_index=completed_step,
            processed_actions=len(results),
            emitted_events=tuple(emitted_events),
            status=self.run_state.status,
        )

    def terminate(self) -> RunState:
        """Mark run lifecycle as terminated."""
        self.run_state.status = RunStatus.TERMINATED
        return self.run_state

    @property
    def random_seed(self) -> int:
        """Expose configured deterministic seed."""
        return self.run_state.seed

    @property
    def random_source(self) -> random.Random:
        """Expose deterministic random source for controlled extension points."""
        return self._rng
