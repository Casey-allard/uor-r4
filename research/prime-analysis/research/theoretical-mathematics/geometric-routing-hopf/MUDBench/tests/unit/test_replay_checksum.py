from __future__ import annotations

import re

from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.integrity.replay_checksum import (
    CHECKSUM_ALGORITHM,
    ReplayChecksumComputationResult,
    compute_replay_artifact_checksum,
    verify_replay_artifact_checksum,
)


def _envelope_payload() -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-checksum-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-checksum",
        "initial_seed": 31,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": 6,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
            "priority": 1,
        },
    }


def _event_payload(*, step_index: int, event_type: str = "action_wait") -> dict[str, object]:
    return {
        "run_id": "run-checksum-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": "agent-a",
        "payload": {"step": step_index},
        "metadata": {"ok": True},
    }


def _artifact() -> ReplayArtifact:
    result = emit_replay_artifact(
        envelope=_envelope_payload(),
        events=(
            _event_payload(step_index=0),
            _event_payload(step_index=1, event_type="action_move"),
        ),
    )
    assert result.accepted is True
    assert result.artifact is not None
    return result.artifact


def test_compute_replay_artifact_checksum_is_deterministic_for_same_artifact() -> None:
    artifact = _artifact()

    first = compute_replay_artifact_checksum(artifact)
    second = compute_replay_artifact_checksum(artifact)

    assert first.accepted is True
    assert second.accepted is True
    assert first.checksum == second.checksum
    assert first.checksum is not None
    assert first.checksum.algorithm == CHECKSUM_ALGORITHM
    assert re.fullmatch(r"[0-9a-f]{64}", first.checksum.digest_hex)


def test_compute_replay_artifact_checksum_accepts_mapping_and_matches_typed_artifact() -> None:
    artifact = _artifact()
    typed_result = compute_replay_artifact_checksum(artifact)
    mapping_result = compute_replay_artifact_checksum(artifact.to_dict())

    assert typed_result.accepted is True
    assert mapping_result.accepted is True
    assert typed_result.checksum == mapping_result.checksum
    assert mapping_result.artifact_result is not None


def test_compute_replay_artifact_checksum_changes_when_artifact_content_changes() -> None:
    base = _artifact()
    mutated_payload = base.to_dict()
    mutated_payload["events"][1]["payload"]["step"] = 99  # type: ignore[index]
    mutated = ReplayArtifact.from_mapping(mutated_payload)

    base_result = compute_replay_artifact_checksum(base)
    mutated_result = compute_replay_artifact_checksum(mutated)

    assert base_result.accepted is True
    assert mutated_result.accepted is True
    assert base_result.checksum is not None
    assert mutated_result.checksum is not None
    assert base_result.checksum.digest_hex != mutated_result.checksum.digest_hex


def test_compute_replay_artifact_checksum_rejects_invalid_input_type() -> None:
    result = compute_replay_artifact_checksum(3.14)

    assert result == ReplayChecksumComputationResult(
        accepted=False,
        reason="artifact_must_be_replay_artifact_or_mapping",
    )


def test_compute_replay_artifact_checksum_rejects_invalid_artifact_mapping() -> None:
    payload = {"envelope": _envelope_payload()}
    result = compute_replay_artifact_checksum(payload)

    assert result.accepted is False
    assert result.reason == "invalid_artifact:missing_required_fields:events"
    assert result.artifact_result is not None
    assert result.artifact_result.reason == "missing_required_fields:events"


def test_verify_replay_artifact_checksum_reports_match_true_for_identical_digest() -> None:
    artifact = _artifact()
    computed = compute_replay_artifact_checksum(artifact)
    assert computed.checksum is not None

    verification = verify_replay_artifact_checksum(
        artifact=artifact,
        expected_digest=computed.checksum.digest_hex.upper(),
    )

    assert verification.accepted is True
    assert verification.matched is True
    assert verification.actual_checksum == computed.checksum


def test_verify_replay_artifact_checksum_reports_mismatch_deterministically() -> None:
    base = _artifact()
    modified_payload = base.to_dict()
    modified_payload["events"][0]["payload"]["step"] = 44  # type: ignore[index]
    modified_artifact = ReplayArtifact.from_mapping(modified_payload)

    expected = compute_replay_artifact_checksum(base)
    assert expected.checksum is not None
    verification = verify_replay_artifact_checksum(
        artifact=modified_artifact,
        expected_digest=expected.checksum.digest_hex,
    )

    assert verification.accepted is True
    assert verification.matched is False
    assert verification.actual_checksum is not None
    assert verification.actual_checksum.digest_hex != expected.checksum.digest_hex


def test_verify_replay_artifact_checksum_rejects_invalid_expected_digest_or_artifact() -> None:
    result = verify_replay_artifact_checksum(artifact=_artifact(), expected_digest="not-a-digest")
    assert result.accepted is False
    assert result.reason == "expected_digest_must_be_sha256_hex"

    result = verify_replay_artifact_checksum(
        artifact={"envelope": _envelope_payload()},
        expected_digest="0" * 64,
    )
    assert result.accepted is False
    assert result.reason == "cannot_compute_checksum:invalid_artifact:missing_required_fields:events"
    assert result.computation_result is not None
