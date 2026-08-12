"""World state manager interface for simulation control."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, Mapping

WorldSnapshot = Mapping[str, Any]
WorldDelta = Mapping[str, Any]


class WorldStateManager(ABC):
    """Boundary for world state access and mutation.

    Implementations must apply deltas deterministically and return stable snapshots
    for identical state.
    """

    @abstractmethod
    def get_snapshot(self) -> WorldSnapshot:
        """Return a read-only snapshot of the current world state."""

    @abstractmethod
    def apply_delta(self, delta: WorldDelta) -> None:
        """Apply a deterministic state delta to world state."""

    @abstractmethod
    def reset(self) -> None:
        """Reset world state to its initial deterministic baseline."""
