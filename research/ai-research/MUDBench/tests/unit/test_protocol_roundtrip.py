from __future__ import annotations

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps, json_loads_object


def test_canonical_json_dumps_is_deterministic() -> None:
    payload = {"b": 2, "a": 1}
    assert canonical_json_dumps(payload) == '{"a":1,"b":2}'


def test_observation_json_roundtrip() -> None:
    observation = Observation.from_dict(
        {
            "run_id": "abc123",
            "step": 3,
            "location": "forest_path",
            "description": "A path.",
            "exits": ["north", "south"],
            "entities": [{"type": "item", "name": "coin"}],
            "inventory": ["coin"],
            "health": 100,
            "messages": ["hello"],
            "action_space": ["wait"],
            "remaining_steps": 10,
        }
    )

    encoded = canonical_json_dumps(observation.to_dict())
    decoded = json_loads_object(encoded)

    assert Observation.from_dict(decoded) == observation


def test_action_json_roundtrip() -> None:
    action = ActionSubmission(action="wait")
    encoded = canonical_json_dumps(action.to_dict())
    decoded = json_loads_object(encoded)
    assert ActionSubmission.from_dict(decoded) == action
