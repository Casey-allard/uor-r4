"""Strict single-shot local wrapper example for MUDBench."""

from __future__ import annotations

import json
import sys
from typing import Any, Mapping, Sequence


_REQUIRED_FIELDS = (
    "run_id",
    "step",
    "location",
    "description",
    "exits",
    "entities",
    "inventory",
    "health",
    "messages",
    "action_space",
    "remaining_steps",
)
_SUPPORTED_PROTOCOL_VERSION = "1.0"


def _read_observation() -> dict[str, Any]:
    raw_line = sys.stdin.readline()
    if not raw_line:
        raise ValueError("missing_observation_payload")
    payload = json.loads(raw_line)
    if not isinstance(payload, dict):
        raise ValueError("observation_payload_must_be_object")

    for field_name in _REQUIRED_FIELDS:
        if field_name not in payload:
            raise ValueError(f"missing_observation_field:{field_name}")

    protocol_version = payload.get("protocol_version", _SUPPORTED_PROTOCOL_VERSION)
    if protocol_version != _SUPPORTED_PROTOCOL_VERSION:
        raise ValueError(
            f"unsupported_protocol_version:{protocol_version}:supported={_SUPPORTED_PROTOCOL_VERSION}"
        )

    if not isinstance(payload["action_space"], Sequence) or isinstance(payload["action_space"], (str, bytes)):
        raise ValueError("action_space_must_be_sequence")
    if not all(isinstance(action, str) for action in payload["action_space"]):
        raise ValueError("action_space_must_contain_strings")
    return payload


def _select_action(observation: Mapping[str, Any]) -> str:
    action_space = tuple(observation["action_space"])
    if "look" in action_space:
        return "look"
    if "wait" in action_space:
        return "wait"
    if action_space:
        return action_space[0]
    raise ValueError("empty_action_space")


def main() -> int:
    try:
        observation = _read_observation()
        action = _select_action(observation)
        sys.stdout.write(json.dumps({"action": action}, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        sys.stdout.write("\n")
        sys.stdout.flush()
        return 0
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        print(f"strict_wrapper_example error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
