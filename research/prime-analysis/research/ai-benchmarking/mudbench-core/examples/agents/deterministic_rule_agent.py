"""Deterministic single-shot rule-based local agent example for MUDBench."""

from __future__ import annotations

import json
import sys
from typing import Any


def _read_observation() -> dict[str, Any]:
    raw_line = sys.stdin.readline()
    if not raw_line:
        raise ValueError("missing_observation_payload")
    payload = json.loads(raw_line)
    if not isinstance(payload, dict):
        raise ValueError("observation_payload_must_be_object")
    return payload


def _select_action(action_space: tuple[str, ...]) -> str:
    for candidate in action_space:
        if candidate.startswith("take "):
            return candidate
    for candidate in action_space:
        if candidate.startswith("move "):
            return candidate
    for candidate in action_space:
        if candidate.startswith("attack "):
            return candidate
    if "look" in action_space:
        return "look"
    if "wait" in action_space:
        return "wait"
    raise ValueError("empty_action_space")


def main() -> int:
    try:
        observation = _read_observation()
        raw_action_space = observation.get("action_space", ())
        if not isinstance(raw_action_space, list) and not isinstance(raw_action_space, tuple):
            raise ValueError("action_space_must_be_sequence")
        action_space = tuple(str(action) for action in raw_action_space)
        action = _select_action(action_space)
        sys.stdout.write(json.dumps({"action": action}, sort_keys=True, separators=(",", ":"), ensure_ascii=True))
        sys.stdout.write("\n")
        sys.stdout.flush()
        return 0
    except (ValueError, TypeError, json.JSONDecodeError) as exc:
        print(f"deterministic_rule_agent error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
