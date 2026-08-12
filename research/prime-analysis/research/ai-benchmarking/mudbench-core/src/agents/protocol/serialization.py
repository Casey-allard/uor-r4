"""Deterministic serialization helpers for agent protocol payloads."""

from __future__ import annotations

import json
from typing import Any, Mapping


def canonical_json_dumps(payload: Mapping[str, Any]) -> str:
    """Serialize a payload with stable key ordering."""
    return json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def json_loads_object(raw_payload: str) -> dict[str, Any]:
    """Parse and validate that a payload is a JSON object."""
    parsed = json.loads(raw_payload)
    if not isinstance(parsed, dict):
        raise ValueError("Expected JSON object payload")
    return parsed
