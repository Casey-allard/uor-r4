"""Deterministic benchmark run lifecycle state machine."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class BenchmarkLifecycleStatus(str, Enum):
    """Explicit benchmark lifecycle states."""

    INITIALIZED = "initialized"
    RUNNING = "running"
    FINALIZED = "finalized"


@dataclass(frozen=True, slots=True)
class BenchmarkLifecycleState:
    """Immutable lifecycle state snapshot."""

    run_id: str
    scenario_id: str
    seed: int
    max_steps: int
    step_index: int = 0
    status: BenchmarkLifecycleStatus = BenchmarkLifecycleStatus.INITIALIZED

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.scenario_id, str) or not self.scenario_id:
            raise ValueError("scenario_id must be a non-empty string")
        if not isinstance(self.seed, int) or isinstance(self.seed, bool):
            raise ValueError("seed must be an integer")
        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")
        if not isinstance(self.step_index, int) or isinstance(self.step_index, bool) or self.step_index < 0:
            raise ValueError("step_index must be a non-negative integer")


class BenchmarkRunLifecycle:
    """Deterministic lifecycle transition controller."""

    def __init__(self, *, run_id: str, scenario_id: str, seed: int, max_steps: int) -> None:
        self._state = BenchmarkLifecycleState(
            run_id=run_id,
            scenario_id=scenario_id,
            seed=seed,
            max_steps=max_steps,
        )

    @property
    def state(self) -> BenchmarkLifecycleState:
        return self._state

    def start(self) -> BenchmarkLifecycleState:
        if self._state.status is not BenchmarkLifecycleStatus.INITIALIZED:
            raise RuntimeError("lifecycle.start() requires initialized state")
        self._state = BenchmarkLifecycleState(
            run_id=self._state.run_id,
            scenario_id=self._state.scenario_id,
            seed=self._state.seed,
            max_steps=self._state.max_steps,
            step_index=0,
            status=BenchmarkLifecycleStatus.RUNNING,
        )
        return self._state

    def advance_step(self) -> BenchmarkLifecycleState:
        if self._state.status is not BenchmarkLifecycleStatus.RUNNING:
            raise RuntimeError("lifecycle.advance_step() requires running state")
        if self._state.step_index >= self._state.max_steps:
            raise RuntimeError("lifecycle.advance_step() cannot exceed max_steps")

        next_step = self._state.step_index + 1
        next_status = (
            BenchmarkLifecycleStatus.FINALIZED
            if next_step >= self._state.max_steps
            else BenchmarkLifecycleStatus.RUNNING
        )
        self._state = BenchmarkLifecycleState(
            run_id=self._state.run_id,
            scenario_id=self._state.scenario_id,
            seed=self._state.seed,
            max_steps=self._state.max_steps,
            step_index=next_step,
            status=next_status,
        )
        return self._state

    def finalize(self) -> BenchmarkLifecycleState:
        if self._state.status is BenchmarkLifecycleStatus.FINALIZED:
            return self._state
        if self._state.status is not BenchmarkLifecycleStatus.RUNNING:
            raise RuntimeError("lifecycle.finalize() requires running or finalized state")
        self._state = BenchmarkLifecycleState(
            run_id=self._state.run_id,
            scenario_id=self._state.scenario_id,
            seed=self._state.seed,
            max_steps=self._state.max_steps,
            step_index=self._state.step_index,
            status=BenchmarkLifecycleStatus.FINALIZED,
        )
        return self._state

