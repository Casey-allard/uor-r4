"""Run metadata state for simulation control."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class RunStatus(str, Enum):
    INITIALIZED = "initialized"
    RUNNING = "running"
    TERMINATED = "terminated"


@dataclass(slots=True)
class RunState:
    """Controller-owned run metadata.

    This intentionally stores only run lifecycle metadata and avoids world logic.
    """

    run_id: str
    seed: int
    max_steps: int
    step_index: int = 0
    status: RunStatus = RunStatus.INITIALIZED
