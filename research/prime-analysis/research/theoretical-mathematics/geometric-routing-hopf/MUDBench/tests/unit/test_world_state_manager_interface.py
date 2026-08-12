from __future__ import annotations

import pytest

from core.world_state_manager import WorldStateManager


class InMemoryWorldState(WorldStateManager):
    def __init__(self) -> None:
        self._state: dict[str, object] = {"tick": 0}

    def get_snapshot(self) -> dict[str, object]:
        return dict(self._state)

    def apply_delta(self, delta: dict[str, object]) -> None:
        self._state.update(delta)

    def reset(self) -> None:
        self._state = {"tick": 0}


def test_world_state_manager_is_abstract() -> None:
    with pytest.raises(TypeError):
        WorldStateManager()  # type: ignore[abstract]


def test_in_memory_world_state_implements_interface() -> None:
    state = InMemoryWorldState()
    state.apply_delta({"tick": 1})
    assert state.get_snapshot()["tick"] == 1
    state.reset()
    assert state.get_snapshot() == {"tick": 0}
