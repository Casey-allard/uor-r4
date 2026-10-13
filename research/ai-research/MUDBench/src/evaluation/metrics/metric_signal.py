"""Deterministic metric-signal contract for benchmark evaluation."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

REQUIRED_METRIC_SIGNAL_FIELDS = (
    "run_id",
    "step",
    "actor_id",
    "metric_name",
    "value",
)

MetricSignalScalar = str | int | float | bool | None
_INVALID_VALUE = object()


@dataclass(frozen=True, slots=True)
class MetricSignal:
    """Canonical immutable evaluation metric signal."""

    run_id: str
    step: int
    actor_id: str
    metric_name: str
    value: int | float
    attributes: tuple[tuple[str, MetricSignalScalar], ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.step, int) or isinstance(self.step, bool) or self.step < 0:
            raise ValueError("step must be a non-negative integer")
        if not isinstance(self.actor_id, str) or not self.actor_id:
            raise ValueError("actor_id must be a non-empty string")
        if not isinstance(self.metric_name, str) or not self.metric_name:
            raise ValueError("metric_name must be a non-empty string")
        if not isinstance(self.value, (int, float)) or isinstance(self.value, bool):
            raise ValueError("value must be an int or float")
        if isinstance(self.value, float) and not math.isfinite(self.value):
            raise ValueError("value must be finite")

        if isinstance(self.attributes, (str, bytes)) or not isinstance(self.attributes, Sequence):
            raise ValueError("attributes must be a sequence of key/value pairs")

        normalized_attributes: list[tuple[str, MetricSignalScalar]] = []
        seen_keys: set[str] = set()
        for attribute in self.attributes:
            if not isinstance(attribute, tuple) or len(attribute) != 2:
                raise ValueError("attributes entries must be key/value tuples")
            raw_key, raw_value = attribute
            if not isinstance(raw_key, str) or not raw_key:
                raise ValueError("attribute keys must be non-empty strings")
            if raw_key in seen_keys:
                raise ValueError(f"duplicate attribute key: {raw_key}")
            seen_keys.add(raw_key)

            normalized_value = _normalize_attribute_value(raw_value)
            if normalized_value is _INVALID_VALUE:
                raise ValueError("attribute values must be JSON scalar values")
            normalized_attributes.append((raw_key, normalized_value))

        normalized_attributes.sort(key=lambda item: item[0])
        object.__setattr__(self, "attributes", tuple(normalized_attributes))

    @classmethod
    def from_dict(cls, payload: Mapping[str, Any]) -> "MetricSignal":
        result = parse_metric_signal(payload)
        if not result.accepted or result.signal is None:
            raise ValueError(f"Invalid metric signal payload: {result.reason}")
        return result.signal

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "step": self.step,
            "actor_id": self.actor_id,
            "metric_name": self.metric_name,
            "value": self.value,
            "attributes": {key: value for key, value in self.attributes},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class MetricSignalParseResult:
    """Explicit accept/reject parse result for metric-signal payloads."""

    accepted: bool
    signal: MetricSignal | None = None
    reason: str | None = None


def parse_metric_signal(payload: object) -> MetricSignalParseResult:
    """Parse a metric-signal payload using explicit accept/reject semantics."""
    if not isinstance(payload, Mapping):
        return _rejected_result("payload_not_mapping")

    missing_fields = [field for field in REQUIRED_METRIC_SIGNAL_FIELDS if field not in payload]
    if missing_fields:
        return _rejected_result(f"missing_required_fields:{','.join(missing_fields)}")

    run_id = payload.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        return _rejected_result("run_id_must_be_non_empty_string")

    step = payload.get("step")
    if not isinstance(step, int) or isinstance(step, bool) or step < 0:
        return _rejected_result("step_must_be_non_negative_integer")

    actor_id = payload.get("actor_id")
    if not isinstance(actor_id, str) or not actor_id:
        return _rejected_result("actor_id_must_be_non_empty_string")

    metric_name = payload.get("metric_name")
    if not isinstance(metric_name, str) or not metric_name:
        return _rejected_result("metric_name_must_be_non_empty_string")

    value = payload.get("value")
    if not isinstance(value, (int, float)) or isinstance(value, bool):
        return _rejected_result("value_must_be_numeric")
    if isinstance(value, float) and not math.isfinite(value):
        return _rejected_result("value_must_be_finite")

    raw_attributes = payload.get("attributes", {})
    if not isinstance(raw_attributes, Mapping):
        return _rejected_result("attributes_must_be_mapping")

    attribute_keys = tuple(raw_attributes.keys())
    for key in attribute_keys:
        if not isinstance(key, str) or not key:
            return _rejected_result("attribute_key_must_be_non_empty_string")

    normalized_attributes: list[tuple[str, MetricSignalScalar]] = []
    for key in sorted(attribute_keys):
        normalized_value = _normalize_attribute_value(raw_attributes[key])
        if normalized_value is _INVALID_VALUE:
            return _rejected_result("attribute_value_must_be_scalar")
        normalized_attributes.append((key, normalized_value))

    return _accepted_result(
        MetricSignal(
            run_id=run_id,
            step=step,
            actor_id=actor_id,
            metric_name=metric_name,
            value=value,
            attributes=tuple(normalized_attributes),
        )
    )


def _normalize_attribute_value(raw_value: Any) -> MetricSignalScalar | object:
    if raw_value is None:
        return None
    if isinstance(raw_value, bool):
        return raw_value
    if isinstance(raw_value, str):
        return raw_value
    if isinstance(raw_value, int):
        return raw_value
    if isinstance(raw_value, float):
        if not math.isfinite(raw_value):
            return _INVALID_VALUE
        return raw_value
    return _INVALID_VALUE


def _accepted_result(signal: MetricSignal) -> MetricSignalParseResult:
    return MetricSignalParseResult(accepted=True, signal=signal)


def _rejected_result(reason: str) -> MetricSignalParseResult:
    return MetricSignalParseResult(accepted=False, reason=reason)
