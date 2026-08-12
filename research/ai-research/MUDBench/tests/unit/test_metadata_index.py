from __future__ import annotations

from replay.integrity.replay_checksum import compute_replay_artifact_checksum
from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.reconstruction.reconstructor import reconstruct_replay_state
from replay.telemetry.collector import collect_replay_telemetry
from replay.telemetry.metadata_index import (
    REQUIRED_ARTIFACT_REF_KEYS,
    RUN_METADATA_INDEX_SCHEMA_VERSION,
    RunMetadataIndexBuildResult,
    RunMetadataIndexParseResult,
    build_run_metadata_index,
    parse_run_metadata_index,
)


def _envelope_payload(*, max_steps: int = 4) -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "run-index-1",
        "benchmark_id": "benchmark-phase4",
        "scenario_id": "scenario-index",
        "initial_seed": 13,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": max_steps,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
        },
    }


def _event_payload(
    *,
    step_index: int,
    event_type: str,
    actor_id: str | None,
    payload: dict[str, object] | None = None,
) -> dict[str, object]:
    return {
        "run_id": "run-index-1",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload or {},
        "metadata": {},
    }


def _artifact(events: tuple[dict[str, object], ...], *, max_steps: int = 4) -> ReplayArtifact:
    emit_result = emit_replay_artifact(envelope=_envelope_payload(max_steps=max_steps), events=events)
    assert emit_result.accepted is True
    assert emit_result.artifact is not None
    return emit_result.artifact


def _components() -> tuple[ReplayArtifact, object, object, object]:
    artifact = _artifact(
        (
            _event_payload(step_index=0, event_type="action_move", actor_id="agent-a"),
            _event_payload(step_index=1, event_type="action_attack", actor_id="agent-a"),
            _event_payload(step_index=1, event_type="action_rejected", actor_id="agent-b"),
        )
    )
    telemetry_result = collect_replay_telemetry(artifact)
    assert telemetry_result.accepted is True
    assert telemetry_result.snapshot is not None
    reconstructed_result = reconstruct_replay_state(artifact)
    assert reconstructed_result.accepted is True
    assert reconstructed_result.reconstructed_state is not None
    checksum_result = compute_replay_artifact_checksum(artifact)
    assert checksum_result.accepted is True
    assert checksum_result.checksum is not None
    return (
        artifact,
        telemetry_result.snapshot,
        reconstructed_result.reconstructed_state,
        checksum_result.checksum,
    )


def test_build_run_metadata_index_is_deterministic_versioned_and_roundtrippable() -> None:
    artifact, telemetry_snapshot, reconstructed_state, checksum = _components()

    first = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_snapshot,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum,
    )
    second = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_snapshot,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum,
    )

    assert first.accepted is True
    assert second.accepted is True
    assert first.entry == second.entry
    assert first.entry is not None
    assert first.entry.schema_version == RUN_METADATA_INDEX_SCHEMA_VERSION
    assert first.entry.benchmark_version == "0.1"
    assert first.entry.scenario_version == "1.0"
    assert first.entry.scoring_version == "phase3-v1"
    assert tuple(key for key, _ in first.entry.artifact_refs) == REQUIRED_ARTIFACT_REF_KEYS

    parsed = parse_run_metadata_index(first.entry.to_dict())
    assert parsed.accepted is True
    assert parsed.entry == first.entry
    assert parsed.entry is not None
    assert parsed.entry.to_canonical_json() == first.entry.to_canonical_json()


def test_build_run_metadata_index_supports_mapping_inputs_with_parity() -> None:
    artifact, telemetry_snapshot, reconstructed_state, checksum = _components()

    typed_result = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_snapshot,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum,
    )
    mapping_result = build_run_metadata_index(
        replay_artifact=artifact.to_dict(),
        telemetry_snapshot=telemetry_snapshot.to_dict(),
        reconstructed_state=reconstructed_state.to_dict(),
        replay_checksum=checksum.to_dict(),
    )

    assert typed_result.accepted is True
    assert mapping_result.accepted is True
    assert typed_result.entry == mapping_result.entry


def test_build_run_metadata_index_rejects_checksum_mismatch() -> None:
    artifact, telemetry_snapshot, reconstructed_state, checksum = _components()
    checksum_payload = checksum.to_dict()
    checksum_payload["digest_hex"] = "0" * 64

    result = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_snapshot,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum_payload,
    )

    assert result == RunMetadataIndexBuildResult(
        accepted=False,
        reason="replay_checksum_mismatch:digest",
    )


def test_build_run_metadata_index_rejects_run_id_mismatch_between_telemetry_and_artifact() -> None:
    artifact, telemetry_snapshot, reconstructed_state, checksum = _components()
    telemetry_payload = telemetry_snapshot.to_dict()
    telemetry_payload["run_id"] = "run-other"

    result = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_payload,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum,
    )

    assert result.accepted is False
    assert result.reason == "run_id_mismatch:telemetry_vs_artifact"


def test_parse_run_metadata_index_rejects_partial_records_explicitly() -> None:
    artifact, telemetry_snapshot, reconstructed_state, checksum = _components()
    build_result = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry_snapshot,
        reconstructed_state=reconstructed_state,
        replay_checksum=checksum,
    )
    assert build_result.entry is not None
    payload = build_result.entry.to_dict()
    payload.pop("scenario_id")

    result = parse_run_metadata_index(payload)
    assert result == RunMetadataIndexParseResult(
        accepted=False,
        reason="missing_required_fields:scenario_id",
    )

    payload = build_result.entry.to_dict()
    payload.pop("scoring_version")
    result = parse_run_metadata_index(payload)
    assert result == RunMetadataIndexParseResult(
        accepted=False,
        reason="missing_required_fields:scoring_version",
    )

    payload = build_result.entry.to_dict()
    payload["artifact_refs"].pop("replay_checksum")
    result = parse_run_metadata_index(payload)
    assert result.accepted is False
    assert (
        result.reason
        == "artifact_refs_must_include_exact_keys:reconstructed_state,replay_artifact,replay_checksum,telemetry_snapshot"
    )


def test_parse_run_metadata_index_rejects_non_mapping_payload() -> None:
    result = parse_run_metadata_index(["not", "a", "mapping"])
    assert result == RunMetadataIndexParseResult(accepted=False, reason="payload_not_mapping")
