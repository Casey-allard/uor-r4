from __future__ import annotations

import pytest

from replay.logging.replay_log_format import (
    MAX_SEED,
    REPLAY_LOG_SCHEMA_VERSION,
    ReplayLogEnvelope,
    ReplayLogEnvelopeParseResult,
    parse_replay_log_envelope,
)


def _valid_envelope_payload() -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-forest",
        "initial_seed": 41,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": 12,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "priority": 1,
            "phase": "phase4",
            "recorded": True,
        },
    }


def test_parse_replay_log_envelope_accepts_valid_payload_and_roundtrips() -> None:
    payload = _valid_envelope_payload()
    result = parse_replay_log_envelope(payload)

    assert result.accepted is True
    assert result.reason is None
    assert result.envelope == ReplayLogEnvelope(
        schema_version=REPLAY_LOG_SCHEMA_VERSION,
        run_id="run-1",
        benchmark_id="benchmark-phase4",
        scenario_id="scenario-forest",
        initial_seed=41,
        seed_source="run_seed",
        actor_ids=("agent-a", "agent-b"),
        max_steps=12,
        run_metadata=(
            ("benchmark_version", "0.1"),
            ("phase", "phase4"),
            ("priority", 1),
            ("recorded", True),
            ("scenario_version", "1.0"),
            ("scoring_version", "phase3-v1"),
        ),
    )
    assert result.envelope is not None
    assert result.envelope.to_dict() == {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-forest",
        "initial_seed": 41,
        "seed_source": "run_seed",
        "actor_ids": ["agent-a", "agent-b"],
        "max_steps": 12,
        "run_metadata": {
            "benchmark_version": "0.1",
            "phase": "phase4",
            "priority": 1,
            "recorded": True,
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
        },
    }

    reparsed = parse_replay_log_envelope(result.envelope.to_dict())
    assert reparsed == result


def test_parse_replay_log_envelope_rejects_non_mapping_payload() -> None:
    result = parse_replay_log_envelope(["not", "a", "mapping"])
    assert result == ReplayLogEnvelopeParseResult(accepted=False, reason="payload_not_mapping")


def test_parse_replay_log_envelope_rejects_missing_required_field() -> None:
    payload = _valid_envelope_payload()
    payload.pop("seed_source")

    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.envelope is None
    assert result.reason == "missing_required_fields:seed_source"


def test_parse_replay_log_envelope_rejects_invalid_seed_values() -> None:
    payload = _valid_envelope_payload()
    payload["initial_seed"] = "41"
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "initial_seed_must_be_integer"

    payload = _valid_envelope_payload()
    payload["initial_seed"] = MAX_SEED + 1
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == f"initial_seed_must_be_within_bounds:0..{MAX_SEED}"


def test_parse_replay_log_envelope_rejects_invalid_seed_source_and_actor_ids() -> None:
    payload = _valid_envelope_payload()
    payload["seed_source"] = "manual"
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "seed_source_must_be_one_of:seed_override,run_seed,scenario_seed,derived"

    payload = _valid_envelope_payload()
    payload["actor_ids"] = ["agent-a", "agent-a"]
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "actor_ids_must_be_non_empty_unique_string_sequence"


def test_parse_replay_log_envelope_rejects_invalid_run_metadata() -> None:
    payload = _valid_envelope_payload()
    payload["run_metadata"] = {"ok": 1, "nested": {"bad": "value"}}
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "run_metadata_must_be_mapping_with_scalar_values"


def test_parse_replay_log_envelope_rejects_missing_version_provenance() -> None:
    payload = _valid_envelope_payload()
    payload["run_metadata"] = {"phase": "phase4"}
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "run_metadata_missing_required_fields:benchmark_version,scenario_version,scoring_version"


def test_parse_replay_log_envelope_rejects_non_string_version_provenance() -> None:
    payload = _valid_envelope_payload()
    payload["run_metadata"]["scoring_version"] = 1  # type: ignore[index]
    result = parse_replay_log_envelope(payload)
    assert result.accepted is False
    assert result.reason == "run_metadata_scoring_version_must_be_non_empty_string"


def test_replay_log_envelope_to_canonical_json_is_stable_for_key_order() -> None:
    first = ReplayLogEnvelope(
        schema_version=REPLAY_LOG_SCHEMA_VERSION,
        run_id="run-9",
        benchmark_id="benchmark-phase4",
        scenario_id="scenario-9",
        initial_seed=7,
        seed_source="derived",
        actor_ids=("agent-z", "agent-a"),
        max_steps=5,
        run_metadata=(
            ("z", "last"),
            ("a", "first"),
            ("benchmark_version", "0.1"),
            ("scenario_version", "1.0"),
            ("scoring_version", "phase3-v1"),
        ),
    )
    second = ReplayLogEnvelope(
        schema_version=REPLAY_LOG_SCHEMA_VERSION,
        run_id="run-9",
        benchmark_id="benchmark-phase4",
        scenario_id="scenario-9",
        initial_seed=7,
        seed_source="derived",
        actor_ids=("agent-a", "agent-z"),
        max_steps=5,
        run_metadata=(
            ("a", "first"),
            ("benchmark_version", "0.1"),
            ("scenario_version", "1.0"),
            ("scoring_version", "phase3-v1"),
            ("z", "last"),
        ),
    )

    assert first.actor_ids == second.actor_ids == ("agent-a", "agent-z")
    assert first.run_metadata == second.run_metadata == (
        ("a", "first"),
        ("benchmark_version", "0.1"),
        ("scenario_version", "1.0"),
        ("scoring_version", "phase3-v1"),
        ("z", "last"),
    )
    assert first.to_canonical_json() == second.to_canonical_json()


def test_replay_log_envelope_from_mapping_raises_explicit_reason() -> None:
    payload = _valid_envelope_payload()
    payload["schema_version"] = "2.0"

    with pytest.raises(ValueError, match="unsupported_schema_version:2.0"):
        ReplayLogEnvelope.from_mapping(payload)
