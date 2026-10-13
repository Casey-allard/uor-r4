"""Deterministic world bootstrap helpers for Phase 1 initialization."""

from __future__ import annotations

from core.world_state_manager import WorldSnapshot
from world.rooms.room_graph import DeterministicRoomGraph

from .state_types import WorldRoomState, WorldStateSnapshot
from .world_state import DeterministicWorldStateManager


def bootstrap_snapshot_from_room_graph(
    room_graph: DeterministicRoomGraph,
    *,
    seed: int,
) -> WorldStateSnapshot:
    """Create a deterministic world-state snapshot from a validated room graph."""
    if not isinstance(seed, int):
        raise ValueError("seed must be an integer")

    rooms = tuple(
        _room_state_from_graph(room_graph, room_id) for room_id in room_graph.room_ids
    )

    return WorldStateSnapshot(
        tick=0,
        entities=(),
        rooms=rooms,
        scenario_vars=(
            ("bootstrap_seed", seed),
            ("bootstrap_source", "room_graph"),
        ),
    )


def bootstrap_world_state_manager(
    room_graph: DeterministicRoomGraph,
    *,
    seed: int,
) -> DeterministicWorldStateManager:
    """Build a deterministic world-state manager initialized from room topology."""
    snapshot = bootstrap_snapshot_from_room_graph(room_graph, seed=seed)
    return DeterministicWorldStateManager(initial_state=snapshot)


def bootstrap_snapshot_dict(
    room_graph: DeterministicRoomGraph,
    *,
    seed: int,
) -> WorldSnapshot:
    """Return a serializable dictionary snapshot produced by bootstrap logic."""
    return bootstrap_snapshot_from_room_graph(room_graph, seed=seed).to_dict()


def _room_state_from_graph(room_graph: DeterministicRoomGraph, room_id: str) -> WorldRoomState:
    room = room_graph.get_room(room_id)
    return WorldRoomState(
        room_id=room.room_id,
        description=room.description,
        exits=tuple(room.exits),
        entities_present=tuple(room.entities),
    )

