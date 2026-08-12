"""Canonical seed-input normalization and deterministic seed derivation helpers."""

from __future__ import annotations

import hashlib
import json
from typing import Any, Mapping

MAX_SEED = 2_147_483_647


def canonicalize_seed_input(payload: Mapping[str, Any]) -> str:
    """Return canonical JSON used as stable seed-derivation input."""
    if not isinstance(payload, Mapping):
        raise ValueError("payload must be a mapping")
    normalized = _normalize_json_value(payload, field_name="payload")
    if not isinstance(normalized, Mapping):
        raise ValueError("payload must normalize to a mapping")
    return json.dumps(normalized, sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def derive_seed(payload: Mapping[str, Any], *, max_seed: int = MAX_SEED) -> int:
    """Derive deterministic seed from canonicalized payload."""
    canonical = canonicalize_seed_input(payload)
    return derive_seed_from_canonical(canonical, max_seed=max_seed)


def derive_seed_from_canonical(canonical_payload: str, *, max_seed: int = MAX_SEED) -> int:
    """Derive deterministic seed from canonical payload string."""
    if not isinstance(canonical_payload, str) or not canonical_payload:
        raise ValueError("canonical_payload must be a non-empty string")
    _validate_max_seed(max_seed)
    digest = hashlib.sha256(canonical_payload.encode("ascii")).digest()
    return int.from_bytes(digest[:8], byteorder="big", signed=False) % (max_seed + 1)


def normalize_optional_seed(value: int | None, *, field_name: str, max_seed: int = MAX_SEED) -> int | None:
    """Normalize optional explicit seed and validate deterministic integer bounds."""
    if value is None:
        return None
    return normalize_seed(value, field_name=field_name, max_seed=max_seed)


def normalize_seed(value: int, *, field_name: str, max_seed: int = MAX_SEED) -> int:
    """Validate explicit seed and return it unchanged."""
    _validate_max_seed(max_seed)
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be an integer")
    if value < 0 or value > max_seed:
        raise ValueError(f"{field_name} must be within [0, {max_seed}]")
    return value


def _validate_max_seed(max_seed: int) -> None:
    if not isinstance(max_seed, int) or isinstance(max_seed, bool) or max_seed <= 0:
        raise ValueError("max_seed must be a positive integer")


def _normalize_json_value(value: Any, *, field_name: str) -> Any:
    if isinstance(value, Mapping):
        normalized_items: list[tuple[str, Any]] = []
        for raw_key, raw_value in value.items():
            if not isinstance(raw_key, str) or not raw_key:
                raise ValueError(f"{field_name} contains a non-string or empty key")
            normalized_items.append(
                (raw_key, _normalize_json_value(raw_value, field_name=f"{field_name}.{raw_key}"))
            )
        return {
            key: normalized_value
            for key, normalized_value in sorted(normalized_items, key=lambda item: item[0])
        }

    if isinstance(value, list):
        return [_normalize_json_value(item, field_name=field_name) for item in value]

    if isinstance(value, tuple):
        return [_normalize_json_value(item, field_name=field_name) for item in value]

    if value is None or isinstance(value, (str, int, float, bool)):
        return value

    raise ValueError(f"{field_name} contains unsupported value type: {type(value).__name__}")
