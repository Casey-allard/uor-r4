from __future__ import annotations

import pytest

from agents.protocol.action import ActionSubmission


def test_action_submission_accepts_string_payload() -> None:
    submission = ActionSubmission.from_dict({"action": "move north"})
    assert submission.action == "move north"


def test_action_submission_normalizes_whitespace() -> None:
    submission = ActionSubmission(action="  move north  ")
    assert submission.action == "move north"


def test_action_submission_rejects_missing_action_field() -> None:
    with pytest.raises(ValueError, match="requires string field 'action'"):
        ActionSubmission.from_dict({})


def test_action_submission_rejects_empty_action() -> None:
    with pytest.raises(ValueError, match="non-empty"):
        ActionSubmission(action="   ")
