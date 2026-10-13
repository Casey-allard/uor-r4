from __future__ import annotations

import pytest

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from examples.agents.baselines.random_agent import RandomBaselineAgent


@pytest.fixture
def sample_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-1",
            "step": 3,
            "location": "forest_path",
            "description": "A forest path.",
            "exits": ["north", "south"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": ["move north", "move south", "wait"],
            "remaining_steps": 10,
        }
    )


def test_random_baseline_is_deterministic_with_same_seed(sample_observation: Observation) -> None:
    first = RandomBaselineAgent(seed=7)
    second = RandomBaselineAgent(seed=7)

    first_actions = [first.decide(sample_observation).action for _ in range(20)]
    second_actions = [second.decide(sample_observation).action for _ in range(20)]

    assert first_actions == second_actions


def test_random_baseline_only_emits_valid_actions(sample_observation: Observation) -> None:
    agent = RandomBaselineAgent(seed=11)

    for _ in range(25):
        action = agent.decide(sample_observation)
        assert action.action in sample_observation.action_space


def test_random_baseline_returns_action_submission(sample_observation: Observation) -> None:
    agent = RandomBaselineAgent(seed=3)
    decision = agent.decide(sample_observation)

    assert isinstance(decision, ActionSubmission)


def test_random_baseline_rejects_empty_action_space(sample_observation: Observation) -> None:
    agent = RandomBaselineAgent(seed=5)
    empty_observation = Observation.from_dict({**sample_observation.to_dict(), "action_space": []})

    with pytest.raises(ValueError, match="non-empty action_space"):
        agent.decide(empty_observation)
