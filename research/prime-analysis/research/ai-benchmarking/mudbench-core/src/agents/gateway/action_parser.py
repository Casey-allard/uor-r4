"""Deterministic action-command parser for gateway submissions."""

from __future__ import annotations

from dataclasses import dataclass
import json
from typing import Sequence

from agents.protocol.action import ActionSubmission
from agents.protocol.observation import require_supported_protocol_version
from core.action_processor import ActionRequest, normalize_arguments

_NO_ARGUMENT_VERBS = frozenset({"wait", "look", "defend"})
_VERB_TO_ARGUMENT_KEY = {
    "move": "direction",
    "take": "item_id",
    "drop": "item_id",
    "use": "item_id",
    "attack": "target_id",
    "give": "item_id",
    "talk": "target_id",
}
_SUPPORTED_VERBS = _NO_ARGUMENT_VERBS | frozenset(_VERB_TO_ARGUMENT_KEY.keys())


@dataclass(frozen=True, slots=True)
class ActionCommandParseResult:
    """Deterministic command-parse outcome."""

    accepted: bool
    action_request: ActionRequest | None = None
    reason: str | None = None


@dataclass(frozen=True, slots=True)
class ModelActionOutputParseResult:
    """Deterministic model-output parse outcome for LLM gameplay helpers."""

    accepted: bool
    action_submission: ActionSubmission | None = None
    reason: str | None = None


def parse_action_command(*, actor_id: str, action: str) -> ActionCommandParseResult:
    """Parse a strict canonical action command into an ActionRequest."""
    if not isinstance(actor_id, str) or not actor_id:
        raise ValueError("actor_id must be a non-empty string")
    if not isinstance(action, str):
        raise ValueError("action must be a string")

    command = action.strip()
    if not command:
        return _rejected_result("empty_command")

    spacing_reason = _validate_spacing(command)
    if spacing_reason is not None:
        return _rejected_result(spacing_reason)

    if command != command.lower():
        return _rejected_result("non_canonical_case")

    parts = command.split(" ")
    verb = parts[0]
    if verb not in _SUPPORTED_VERBS:
        return _rejected_result("unsupported_action_verb")

    if verb in _NO_ARGUMENT_VERBS:
        if len(parts) != 1:
            return _rejected_result("unexpected_argument")
        return _accepted_result(
            ActionRequest(actor_id=actor_id, action_type=verb, arguments=())
        )

    if verb == "give":
        if len(parts) < 3:
            return _rejected_result("missing_argument")
        if len(parts) > 3:
            return _rejected_result("too_many_arguments")
        return _accepted_result(
            ActionRequest(
                actor_id=actor_id,
                action_type=verb,
                arguments=normalize_arguments({"item_id": parts[1], "target_id": parts[2]}),
            )
        )

    if len(parts) == 1:
        return _rejected_result("missing_argument")
    if len(parts) > 2:
        return _rejected_result("too_many_arguments")

    argument_value = parts[1]
    argument_key = _VERB_TO_ARGUMENT_KEY[verb]
    return _accepted_result(
        ActionRequest(
            actor_id=actor_id,
            action_type=verb,
            arguments=normalize_arguments({argument_key: argument_value}),
        )
    )


def _validate_spacing(command: str) -> str | None:
    if "  " in command:
        return "non_canonical_spacing"

    for char in command:
        if char.isspace() and char != " ":
            return "non_canonical_spacing"
    return None


def _accepted_result(action_request: ActionRequest) -> ActionCommandParseResult:
    return ActionCommandParseResult(accepted=True, action_request=action_request)


def _rejected_result(reason: str) -> ActionCommandParseResult:
    return ActionCommandParseResult(accepted=False, reason=reason)


def parse_model_action_output(
    *,
    raw_output: str,
    action_space: Sequence[str],
) -> ModelActionOutputParseResult:
    """Parse strict model JSON output against the canonical action contract."""
    if not isinstance(raw_output, str):
        raise ValueError("raw_output must be a string")
    if isinstance(action_space, (str, bytes)) or not isinstance(action_space, Sequence):
        raise ValueError("action_space must be a sequence of strings")

    normalized_action_space: list[str] = []
    for action in action_space:
        if not isinstance(action, str):
            raise ValueError("action_space must contain only strings")
        normalized_action_space.append(action)

    try:
        payload = json.loads(raw_output)
    except json.JSONDecodeError:
        return ModelActionOutputParseResult(accepted=False, reason="invalid_json")

    if not isinstance(payload, dict):
        return ModelActionOutputParseResult(accepted=False, reason="model_output_must_be_object")

    allowed_fields = {"action", "protocol_version"}
    unexpected_fields = sorted(field for field in payload if field not in allowed_fields)
    if unexpected_fields:
        return ModelActionOutputParseResult(
            accepted=False,
            reason="unexpected_output_field",
        )

    if "protocol_version" in payload:
        try:
            require_supported_protocol_version(payload.get("protocol_version"))
        except ValueError:
            return ModelActionOutputParseResult(
                accepted=False,
                reason="unsupported_protocol_version",
            )

    if "action" not in payload:
        return ModelActionOutputParseResult(accepted=False, reason="missing_action")

    raw_action = payload.get("action")
    if not isinstance(raw_action, str):
        return ModelActionOutputParseResult(accepted=False, reason="action_must_be_string")

    normalized_action = raw_action.strip()
    if not normalized_action:
        return ModelActionOutputParseResult(accepted=False, reason="empty_action")
    if normalized_action not in normalized_action_space:
        return ModelActionOutputParseResult(
            accepted=False,
            reason="action_not_in_action_space",
        )

    return ModelActionOutputParseResult(
        accepted=True,
        action_submission=ActionSubmission(action=normalized_action),
    )
