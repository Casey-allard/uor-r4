"""Deterministic gateway action submission->validation->parse pipeline."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Sequence

from agents.gateway.action_parser import ActionCommandParseResult, parse_action_command
from agents.gateway.action_validator import ActionValidationResult, validate_action_submission
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from core.action_processor import ActionRequest


@dataclass(frozen=True, slots=True)
class ActionPipelineEntry:
    """Batch item for deterministic action pipeline processing."""

    actor_id: str
    submission: ActionSubmission
    observation: Observation

    def __post_init__(self) -> None:
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")


@dataclass(frozen=True, slots=True)
class ActionPipelineResult:
    """Deterministic gateway pipeline outcome."""

    actor_id: str
    accepted: bool
    action_request: ActionRequest | None = None
    reason: str | None = None
    rejected_stage: str | None = None
    validation_result: ActionValidationResult | None = None
    parse_result: ActionCommandParseResult | None = None


def run_action_pipeline(
    *,
    actor_id: str,
    submission: ActionSubmission,
    observation: Observation,
) -> ActionPipelineResult:
    """Process one submission through validation then parsing."""
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")

    validation_result = validate_action_submission(submission, observation)
    if not validation_result.accepted:
        if validation_result.reason is None:
            raise RuntimeError("validation rejection must include a reason")
        return ActionPipelineResult(
            actor_id=actor_id,
            accepted=False,
            reason=validation_result.reason,
            rejected_stage="validation",
            validation_result=validation_result,
        )

    parse_result = parse_action_command(actor_id=actor_id, action=submission.action)
    if not parse_result.accepted:
        if parse_result.reason is None:
            raise RuntimeError("parse rejection must include a reason")
        return ActionPipelineResult(
            actor_id=actor_id,
            accepted=False,
            reason=parse_result.reason,
            rejected_stage="parse",
            validation_result=validation_result,
            parse_result=parse_result,
        )

    if parse_result.action_request is None:
        raise RuntimeError("accepted parse result must include action_request")
    return ActionPipelineResult(
        actor_id=actor_id,
        accepted=True,
        action_request=parse_result.action_request,
        validation_result=validation_result,
        parse_result=parse_result,
    )


def run_action_pipeline_batch(entries: Sequence[ActionPipelineEntry]) -> tuple[ActionPipelineResult, ...]:
    """Process a batch of entries in deterministic order."""
    ordered_entries = tuple(sorted(entries, key=_entry_sort_key))
    return tuple(
        run_action_pipeline(
            actor_id=entry.actor_id,
            submission=entry.submission,
            observation=entry.observation,
        )
        for entry in ordered_entries
    )


def _entry_sort_key(entry: ActionPipelineEntry) -> tuple[str, str, int, str]:
    return (
        entry.actor_id,
        entry.observation.run_id,
        entry.observation.step,
        entry.submission.action,
    )

