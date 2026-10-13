from __future__ import annotations

import pytest

from world.rooms.room_graph import DeterministicRoomGraph
from world.rooms.room_types import RoomDefinition


def _sample_graph_payload() -> dict[str, object]:
    return {
        "rooms": {
            "room-b": {
                "title": "Room B",
                "description": "The second room.",
                "exits": {"west": "room-a"},
                "entities": ["npc-1"],
            },
            "room-a": {
                "title": "Room A",
                "description": "The first room.",
                "exits": {"east": "room-b"},
                "entities": [],
            },
        }
    }


def test_room_graph_serialization_is_deterministic() -> None:
    payload_one = _sample_graph_payload()
    payload_two = {
        "rooms": {
            "room-a": payload_one["rooms"]["room-a"],  # type: ignore[index]
            "room-b": payload_one["rooms"]["room-b"],  # type: ignore[index]
        }
    }

    graph_one = DeterministicRoomGraph.from_dict(payload_one)
    graph_two = DeterministicRoomGraph.from_dict(payload_two)

    assert graph_one.to_json() == graph_two.to_json()
    assert graph_one.room_ids == ("room-a", "room-b")


def test_room_graph_rejects_duplicate_room_ids() -> None:
    with pytest.raises(ValueError, match="duplicate room_id"):
        DeterministicRoomGraph(
            rooms=(
                RoomDefinition(room_id="room-a", title="A", description="A"),
                RoomDefinition(room_id="room-a", title="B", description="B"),
            )
        )


def test_room_graph_rejects_unknown_exit_destinations() -> None:
    with pytest.raises(ValueError, match="unknown destination"):
        DeterministicRoomGraph(
            rooms=(
                RoomDefinition(
                    room_id="room-a",
                    title="A",
                    description="A",
                    exits=(("north", "room-missing"),),
                ),
            )
        )


def test_room_graph_exit_resolution() -> None:
    graph = DeterministicRoomGraph.from_dict(_sample_graph_payload())

    assert graph.resolve_exit("room-a", "east") == "room-b"
    assert graph.resolve_exit("room-a", "west") is None


def test_room_graph_json_roundtrip() -> None:
    graph = DeterministicRoomGraph.from_dict(_sample_graph_payload())
    restored = DeterministicRoomGraph.from_json(graph.to_json())

    assert restored.to_dict() == graph.to_dict()

