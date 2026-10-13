"""Action validation boundary for agent submissions."""

from __future__ import annotations

from dataclasses import dataclass

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation


@dataclass(frozen=True, slots=True)
class ActionValidationResult:
    """Validation result for a submitted action."""

    accepted: bool
    reason: str | None = None


def validate_action_submission(
    submission: ActionSubmission,
    observation: Observation,
) -> ActionValidationResult:
    """Validate an action against observation-provided action space."""
    if submission.action not in observation.action_space:
        return ActionValidationResult(accepted=False, reason="action_not_in_action_space")
    return ActionValidationResult(accepted=True)
