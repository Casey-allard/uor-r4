from __future__ import annotations

from core.event_logger import EventLogger, EventRecord, normalize_payload


class InMemoryEventLogger(EventLogger):
    def __init__(self) -> None:
        self._events: list[EventRecord] = []

    def log(self, event: EventRecord) -> None:
        self._events.append(event)

    def records(self) -> tuple[EventRecord, ...]:
        return tuple(self._events)

    def reset(self) -> None:
        self._events.clear()


def test_event_logger_preserves_insertion_order() -> None:
    logger = InMemoryEventLogger()
    first = EventRecord(step_index=0, event_type="first", payload=normalize_payload({"x": 1}))
    second = EventRecord(step_index=0, event_type="second", payload=normalize_payload({"x": 2}))

    logger.log(first)
    logger.log(second)

    assert logger.records() == (first, second)


def test_event_logger_reset_clears_records() -> None:
    logger = InMemoryEventLogger()
    logger.log(EventRecord(step_index=0, event_type="first"))
    assert logger.records()
    logger.reset()
    assert logger.records() == ()
