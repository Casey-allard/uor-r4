"""Action processing interface for simulation steps."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from .event_logger import EventRecord
from .world_state_manager import WorldStateManager


ArgumentItems = tuple[tuple[str, Any], ...]
WorldDeltaItems = tuple[tuple[str, Any], ...]


def normalize_arguments(arguments: Mapping[str, Any] | None = None) -> ArgumentItems:
    """Create an immutable, key-sorted action argument representation."""
    if arguments is None:
        return ()
    return tuple(sorted(arguments.items(), key=lambda item: item[0]))


@dataclass(frozen=True, slots=True)
class ActionRequest:
    """Canonical action envelope for controller/action processor interaction."""

    actor_id: str
    action_type: str
    arguments: ArgumentItems = ()


@dataclass(frozen=True, slots=True)
class ActionProcessingResult:
    """Deterministic action processing result container."""

    accepted: bool
    events: tuple[EventRecord, ...] = ()
    world_delta: WorldDeltaItems = ()


class ActionProcessor(ABC):
    """Boundary for action validation and deterministic processing."""

    @abstractmethod
    def validate_action(self, action: ActionRequest) -> bool:
        """Return True if an action is syntactically valid."""

    @abstractmethod
    def process_actions(
        self,
        actions: Sequence[ActionRequest],
        world_state: WorldStateManager,
        *,
        step_index: int,
    ) -> tuple[ActionProcessingResult, ...]:
        """Process actions in deterministic order for a single step."""
