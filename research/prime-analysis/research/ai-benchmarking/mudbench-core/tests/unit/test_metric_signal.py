from __future__ import annotations

import pytest

from evaluation.metrics.metric_signal import (
    MetricSignal,
    MetricSignalParseResult,
    parse_metric_signal,
)


def _valid_signal_payload() -> dict[str, object]:
    return {
        "run_id": "run-1",
        "step": 2,
        "actor_id": "agent-a",
        "metric_name": "exploration.coverage",
        "value": 1.25,
        "attributes": {"zone": "north", "attempt": 3, "success": True},
    }


def test_parse_metric_signal_accepts_valid_payload_and_roundtrips_deterministically() -> None:
    payload = _valid_signal_payload()
    result = parse_metric_signal(payload)

    assert result.accepted is True
    assert result.reason is None
    assert result.signal == MetricSignal(
        run_id="run-1",
        step=2,
        actor_id="agent-a",
        metric_name="exploration.coverage",
        value=1.25,
        attributes=(("attempt", 3), ("success", True), ("zone", "north")),
    )
    assert result.signal is not None
    assert result.signal.to_dict() == {
        "run_id": "run-1",
        "step": 2,
        "actor_id": "agent-a",
        "metric_name": "exploration.coverage",
        "value": 1.25,
        "attributes": {"attempt": 3, "success": True, "zone": "north"},
    }

    reparsed = parse_metric_signal(result.signal.to_dict())
    assert reparsed == result


def test_parse_metric_signal_rejects_non_mapping_payload() -> None:
    result = parse_metric_signal(["not", "a", "mapping"])
    assert result == MetricSignalParseResult(accepted=False, reason="payload_not_mapping")


def test_parse_metric_signal_rejects_missing_required_field() -> None:
    payload = _valid_signal_payload()
    payload.pop("metric_name")

    result = parse_metric_signal(payload)
    assert result.accepted is False
    assert result.signal is None
    assert result.reason == "missing_required_fields:metric_name"


@pytest.mark.parametrize("bad_step", (-1, 1.5, True, "1"))
def test_parse_metric_signal_rejects_invalid_step(bad_step: object) -> None:
    payload = _valid_signal_payload()
    payload["step"] = bad_step

    result = parse_metric_signal(payload)
    assert result.accepted is False
    assert result.reason == "step_must_be_non_negative_integer"


@pytest.mark.parametrize("bad_value", (True, "3", float("nan"), float("inf")))
def test_parse_metric_signal_rejects_invalid_value(bad_value: object) -> None:
    payload = _valid_signal_payload()
    payload["value"] = bad_value

    result = parse_metric_signal(payload)
    assert result.accepted is False
    if isinstance(bad_value, float):
        assert result.reason == "value_must_be_finite"
    else:
        assert result.reason == "value_must_be_numeric"


def test_parse_metric_signal_rejects_invalid_attributes() -> None:
    payload = _valid_signal_payload()
    payload["attributes"] = {"ok": 1, 2: "bad-key"}

    result = parse_metric_signal(payload)
    assert result.accepted is False
    assert result.reason == "attribute_key_must_be_non_empty_string"


def test_metric_signal_to_canonical_json_is_stable_for_attribute_order() -> None:
    first = MetricSignal(
        run_id="run-9",
        step=0,
        actor_id="agent-z",
        metric_name="efficiency.steps",
        value=8,
        attributes=(("z", "last"), ("a", "first")),
    )
    second = MetricSignal(
        run_id="run-9",
        step=0,
        actor_id="agent-z",
        metric_name="efficiency.steps",
        value=8,
        attributes=(("a", "first"), ("z", "last")),
    )

    assert first.attributes == second.attributes == (("a", "first"), ("z", "last"))
    assert first.to_canonical_json() == second.to_canonical_json()


def test_metric_signal_from_dict_raises_explicit_reason() -> None:
    payload = _valid_signal_payload()
    payload["actor_id"] = ""

    with pytest.raises(ValueError, match="actor_id_must_be_non_empty_string"):
        MetricSignal.from_dict(payload)
