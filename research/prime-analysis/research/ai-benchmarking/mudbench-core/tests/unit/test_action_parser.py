from __future__ import annotations

import pytest

from agents.gateway.action_parser import (
    ActionCommandParseResult,
    ModelActionOutputParseResult,
    parse_action_command,
    parse_model_action_output,
)
from core.action_processor import ActionRequest, normalize_arguments


@pytest.mark.parametrize(
    ("raw_action", "expected_request"),
    (
        ("wait", ActionRequest(actor_id="agent-1", action_type="wait", arguments=())),
        ("look", ActionRequest(actor_id="agent-1", action_type="look", arguments=())),
        (
            "move north",
            ActionRequest(
                actor_id="agent-1",
                action_type="move",
                arguments=normalize_arguments({"direction": "north"}),
            ),
        ),
        (
            "take key-1",
            ActionRequest(
                actor_id="agent-1",
                action_type="take",
                arguments=normalize_arguments({"item_id": "key-1"}),
            ),
        ),
        (
            "drop key-1",
            ActionRequest(
                actor_id="agent-1",
                action_type="drop",
                arguments=normalize_arguments({"item_id": "key-1"}),
            ),
        ),
        (
            "use potion-1",
            ActionRequest(
                actor_id="agent-1",
                action_type="use",
                arguments=normalize_arguments({"item_id": "potion-1"}),
            ),
        ),
        (
            "attack npc-1",
            ActionRequest(
                actor_id="agent-1",
                action_type="attack",
                arguments=normalize_arguments({"target_id": "npc-1"}),
            ),
        ),
        (
            "give token-1 trader-1",
            ActionRequest(
                actor_id="agent-1",
                action_type="give",
                arguments=normalize_arguments({"item_id": "token-1", "target_id": "trader-1"}),
            ),
        ),
    ),
)
def test_parse_action_command_maps_valid_commands(
    raw_action: str,
    expected_request: ActionRequest,
) -> None:
    result = parse_action_command(actor_id="agent-1", action=raw_action)

    assert result == ActionCommandParseResult(accepted=True, action_request=expected_request)


def test_parse_action_command_trims_outer_whitespace_only() -> None:
    result = parse_action_command(actor_id="agent-1", action="  move north  ")

    assert result == ActionCommandParseResult(
        accepted=True,
        action_request=ActionRequest(
            actor_id="agent-1",
            action_type="move",
            arguments=normalize_arguments({"direction": "north"}),
        ),
    )


@pytest.mark.parametrize(
    ("raw_action", "expected_reason"),
    (
        ("", "empty_command"),
        ("   ", "empty_command"),
        ("MOVE north", "non_canonical_case"),
        ("move  north", "non_canonical_spacing"),
        ("move\tnorth", "non_canonical_spacing"),
        ("dance", "unsupported_action_verb"),
        ("wait now", "unexpected_argument"),
        ("look around", "unexpected_argument"),
        ("move", "missing_argument"),
        ("take", "missing_argument"),
        ("give", "missing_argument"),
        ("give token-1", "missing_argument"),
        ("drop item one", "too_many_arguments"),
        ("attack npc one", "too_many_arguments"),
        ("give token-1 trader-1 extra", "too_many_arguments"),
    ),
)
def test_parse_action_command_rejects_invalid_with_explicit_reason(
    raw_action: str,
    expected_reason: str,
) -> None:
    result = parse_action_command(actor_id="agent-1", action=raw_action)

    assert result.accepted is False
    assert result.action_request is None
    assert result.reason == expected_reason


def test_parse_action_command_is_deterministic_for_repeated_input() -> None:
    first = parse_action_command(actor_id="agent-1", action="use potion-1")
    second = parse_action_command(actor_id="agent-1", action="use potion-1")
    third = parse_action_command(actor_id="agent-1", action="use potion-1")

    assert first == second == third


def test_parse_action_command_rejects_invalid_actor_id() -> None:
    with pytest.raises(ValueError, match="actor_id"):
        parse_action_command(actor_id="", action="wait")


def test_parse_action_command_rejects_non_string_action() -> None:
    with pytest.raises(ValueError, match="action must be a string"):
        parse_action_command(actor_id="agent-1", action=object())  # type: ignore[arg-type]


def test_parse_model_action_output_accepts_valid_json_action_payload() -> None:
    result = parse_model_action_output(
        raw_output='{"action":"move north"}',
        action_space=("move north", "wait"),
    )

    assert result.accepted is True
    assert result.reason is None
    assert result.action_submission is not None
    assert result.action_submission.action == "move north"


def test_parse_model_action_output_rejects_invalid_json_with_explicit_reason() -> None:
    result = parse_model_action_output(
        raw_output="not-json",
        action_space=("move north", "wait"),
    )

    assert result == ModelActionOutputParseResult(
        accepted=False,
        reason="invalid_json",
    )


def test_parse_model_action_output_rejects_unexpected_extra_fields() -> None:
    result = parse_model_action_output(
        raw_output='{"action":"wait","reasoning":"because"}',
        action_space=("move north", "wait"),
    )

    assert result == ModelActionOutputParseResult(
        accepted=False,
        reason="unexpected_output_field",
    )
