from __future__ import annotations

from agents.gateway.action_validator import validate_action_submission
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation


def _sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "abc123",
            "step": 0,
            "location": "forest_path",
            "description": "A path.",
            "exits": ["north"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["move north", "wait"],
            "remaining_steps": 100,
        }
    )


def test_action_validator_accepts_action_in_action_space() -> None:
    result = validate_action_submission(ActionSubmission(action="wait"), _sample_observation())
    assert result.accepted is True
    assert result.reason is None


def test_action_validator_rejects_action_outside_action_space() -> None:
    result = validate_action_submission(ActionSubmission(action="attack goblin"), _sample_observation())
    assert result.accepted is False
    assert result.reason == "action_not_in_action_space"
