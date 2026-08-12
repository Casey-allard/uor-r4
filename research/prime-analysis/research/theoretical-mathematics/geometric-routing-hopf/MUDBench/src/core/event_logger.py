"""Event logger interface for deterministic simulation traces."""

from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Any, Mapping, Sequence


PayloadItems = tuple[tuple[str, Any], ...]


def normalize_payload(payload: Mapping[str, Any] | None = None) -> PayloadItems:
    """Create an immutable, key-sorted payload representation."""
    if payload is None:
        return ()
    return tuple(sorted(payload.items(), key=lambda item: item[0]))


@dataclass(frozen=True, slots=True)
class EventRecord:
    """A single immutable simulation event."""

    step_index: int
    event_type: str
    actor_id: str | None = None
    payload: PayloadItems = ()


class EventLogger(ABC):
    """Deterministic event log boundary used by the simulation controller."""

    @abstractmethod
    def log(self, event: EventRecord) -> None:
        """Append an event to the event stream."""

    @abstractmethod
    def records(self) -> Sequence[EventRecord]:
        """Return events in insertion order."""

    @abstractmethod
    def reset(self) -> None:
        """Reset the event stream for a new run."""
