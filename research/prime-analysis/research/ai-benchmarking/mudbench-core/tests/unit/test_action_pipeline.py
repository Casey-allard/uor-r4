from __future__ import annotations

import pytest

from agents.gateway.action_pipeline import (
    ActionPipelineEntry,
    run_action_pipeline,
    run_action_pipeline_batch,
)
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from core.action_processor import ActionRequest, normalize_arguments


def _observation(*, action_space: list[str], step: int = 0) -> Observation:
    return Observation.from_dict(
        {
            "run_id": "run-1",
            "step": step,
            "location": "room-a",
            "description": "A room.",
            "exits": ["north"],
            "entities": [],
            "inventory": [],
            "health": 100,
            "messages": [],
            "action_space": action_space,
            "remaining_steps": 20,
        }
    )


def test_run_action_pipeline_accepts_valid_submission_and_parse() -> None:
    result = run_action_pipeline(
        actor_id="agent-1",
        submission=ActionSubmission(action="move north"),
        observation=_observation(action_space=["move north", "wait"]),
    )

    assert result.accepted is True
    assert result.reason is None
    assert result.rejected_stage is None
    assert result.action_request == ActionRequest(
        actor_id="agent-1",
        action_type="move",
        arguments=normalize_arguments({"direction": "north"}),
    )
    assert result.validation_result is not None
    assert result.validation_result.accepted is True
    assert result.parse_result is not None
    assert result.parse_result.accepted is True


def test_run_action_pipeline_rejects_on_validation_stage() -> None:
    result = run_action_pipeline(
        actor_id="agent-1",
        submission=ActionSubmission(action="move north"),
        observation=_observation(action_space=["wait"]),
    )

    assert result.accepted is False
    assert result.rejected_stage == "validation"
    assert result.reason == "action_not_in_action_space"
    assert result.action_request is None
    assert result.validation_result is not None
    assert result.validation_result.accepted is False
    assert result.parse_result is None


def test_run_action_pipeline_rejects_on_parse_stage_with_verbatim_reason() -> None:
    result = run_action_pipeline(
        actor_id="agent-1",
        submission=ActionSubmission(action="MOVE north"),
        observation=_observation(action_space=["MOVE north"]),
    )

    assert result.accepted is False
    assert result.rejected_stage == "parse"
    assert result.reason == "non_canonical_case"
    assert result.action_request is None
    assert result.validation_result is not None
    assert result.validation_result.accepted is True
    assert result.parse_result is not None
    assert result.parse_result.accepted is False
    assert result.parse_result.reason == "non_canonical_case"


def test_run_action_pipeline_batch_is_deterministically_ordered() -> None:
    entry_b = ActionPipelineEntry(
        actor_id="agent-b",
        submission=ActionSubmission(action="wait"),
        observation=_observation(action_space=["wait"], step=1),
    )
    entry_a = ActionPipelineEntry(
        actor_id="agent-a",
        submission=ActionSubmission(action="wait"),
        observation=_observation(action_space=["wait"], step=1),
    )

    first_results = run_action_pipeline_batch((entry_b, entry_a))
    second_results = run_action_pipeline_batch((entry_a, entry_b))

    assert tuple(result.actor_id for result in first_results) == ("agent-a", "agent-b")
    assert first_results == second_results


def test_run_action_pipeline_is_deterministic_for_identical_input() -> None:
    submission = ActionSubmission(action="use potion-1")
    observation = _observation(action_space=["use potion-1"])

    first = run_action_pipeline(
        actor_id="agent-1",
        submission=submission,
        observation=observation,
    )
    second = run_action_pipeline(
        actor_id="agent-1",
        submission=submission,
        observation=observation,
    )

    assert first == second


def test_run_action_pipeline_rejects_empty_actor_id() -> None:
    with pytest.raises(ValueError, match="actor_id"):
        run_action_pipeline(
            actor_id="",
            submission=ActionSubmission(action="wait"),
            observation=_observation(action_space=["wait"]),
        )

