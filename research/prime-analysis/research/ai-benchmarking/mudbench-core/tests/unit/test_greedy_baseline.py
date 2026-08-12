from __future__ import annotations

import pytest

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from examples.agents.baselines.greedy_agent import GreedyBaselineAgent


@pytest.fixture
def greedy_observation() -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-2",
            "step": 1,
            "location": "camp",
            "description": "A camp.",
            "exits": ["east"],
            "entities": [],
            "inventory": [],
            "health": 90,
            "messages": [],
            "action_space": ["wait", "move east", "take coin", "attack goblin", "inspect sign"],
            "remaining_steps": 20,
        }
    )


def test_greedy_baseline_prefers_highest_priority_verb(greedy_observation: Observation) -> None:
    agent = GreedyBaselineAgent(seed=0)
    decision = agent.decide(greedy_observation)

    assert decision.action == "attack goblin"


def test_greedy_baseline_tie_break_is_seed_deterministic() -> None:
    observation = Observation.from_dict(
        {
            "run_id": "run-3",
            "step": 2,
            "location": "plaza",
            "description": "A plaza.",
            "exits": ["west"],
            "entities": [],
            "inventory": [],
            "health": 80,
            "messages": [],
            "action_space": ["move north", "move south", "move east"],
            "remaining_steps": 30,
        }
    )

    first = GreedyBaselineAgent(seed=19)
    second = GreedyBaselineAgent(seed=19)

    first_actions = [first.decide(observation).action for _ in range(12)]
    second_actions = [second.decide(observation).action for _ in range(12)]

    assert first_actions == second_actions


def test_greedy_baseline_only_emits_valid_actions(greedy_observation: Observation) -> None:
    agent = GreedyBaselineAgent(seed=4)
    decision = agent.decide(greedy_observation)

    assert decision.action in greedy_observation.action_space


def test_greedy_baseline_returns_action_submission(greedy_observation: Observation) -> None:
    agent = GreedyBaselineAgent(seed=2)
    decision = agent.decide(greedy_observation)

    assert isinstance(decision, ActionSubmission)


def test_greedy_baseline_rejects_empty_action_space(greedy_observation: Observation) -> None:
    agent = GreedyBaselineAgent(seed=2)
    empty_observation = Observation.from_dict({**greedy_observation.to_dict(), "action_space": []})

    with pytest.raises(ValueError, match="non-empty action_space"):
        agent.decide(empty_observation)
