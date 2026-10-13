from __future__ import annotations

from replay.integrity.replay_checksum import compute_replay_artifact_checksum
from replay.integrity.replay_verifier import verify_replay_reconstruction
from replay.logging.replay_artifact import ReplayArtifact, emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.reconstruction.reconstructor import reconstruct_replay_state
from replay.telemetry.collector import collect_replay_telemetry
from replay.telemetry.metadata_index import (
    REQUIRED_ARTIFACT_REF_KEYS,
    RUN_METADATA_INDEX_SCHEMA_VERSION,
    build_run_metadata_index,
    parse_run_metadata_index,
)


def _envelope_payload() -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "phase4-gate-contract-run",
        "benchmark_id": "phase4-benchmark",
        "scenario_id": "phase4-gate-contract-scenario",
        "initial_seed": 17,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": 4,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
            "mode": "contract",
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
        "run_id": "phase4-gate-contract-run",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload or {},
        "metadata": {},
    }


def _artifact() -> ReplayArtifact:
    emit_result = emit_replay_artifact(
        envelope=_envelope_payload(),
        events=(
            _event_payload(
                step_index=0,
                event_type="action_move",
                actor_id="agent-a",
                payload={"source_room_id": "room-start", "destination_room_id": "room-mid"},
            ),
            _event_payload(
                step_index=1,
                event_type="action_take",
                actor_id="agent-a",
                payload={"item_id": "item-key"},
            ),
            _event_payload(
                step_index=1,
                event_type="action_attack",
                actor_id="agent-a",
                payload={"target_id": "agent-b", "resulting_health": 7},
            ),
            _event_payload(
                step_index=2,
                event_type="action_rejected",
                actor_id="agent-b",
                payload={"action_type": "move", "reason": "blocked_exit"},
            ),
        ),
    )
    assert emit_result.accepted is True
    assert emit_result.artifact is not None
    return emit_result.artifact


def _build_gate_bundle() -> dict[str, object]:
    artifact = _artifact()

    checksum_result = compute_replay_artifact_checksum(artifact)
    assert checksum_result.accepted is True
    assert checksum_result.checksum is not None
    checksum = checksum_result.checksum

    reconstructed_result = reconstruct_replay_state(artifact)
    assert reconstructed_result.accepted is True
    assert reconstructed_result.reconstructed_state is not None
    reconstructed = reconstructed_result.reconstructed_state

    verification_result = verify_replay_reconstruction(
        artifact=artifact,
        expected_terminal_state=reconstructed,
    )
    assert verification_result.accepted is True
    assert verification_result.matched is True

    telemetry_result = collect_replay_telemetry(artifact)
    assert telemetry_result.accepted is True
    assert telemetry_result.snapshot is not None
    telemetry = telemetry_result.snapshot

    index_result = build_run_metadata_index(
        replay_artifact=artifact,
        telemetry_snapshot=telemetry,
        reconstructed_state=reconstructed,
        replay_checksum=checksum,
    )
    assert index_result.accepted is True
    assert index_result.entry is not None
    index_entry = index_result.entry

    return {
        "artifact": artifact,
        "checksum": checksum,
        "reconstructed": reconstructed,
        "verification": verification_result,
        "telemetry": telemetry,
        "index_entry": index_entry,
    }


def test_phase4_gate_contract_cross_artifact_identity_and_refs() -> None:
    bundle = _build_gate_bundle()
    artifact = bundle["artifact"]
    checksum = bundle["checksum"]
    reconstructed = bundle["reconstructed"]
    verification = bundle["verification"]
    telemetry = bundle["telemetry"]
    index_entry = bundle["index_entry"]

    assert isinstance(artifact, ReplayArtifact)
    assert verification.matched is True
    assert verification.mismatches == ()
    assert telemetry.run_id == reconstructed.run_id == index_entry.run_id == artifact.envelope.run_id
    assert (
        telemetry.initial_seed
        == reconstructed.initial_seed
        == index_entry.initial_seed
        == artifact.envelope.initial_seed
    )
    assert telemetry.max_steps == reconstructed.max_steps == index_entry.max_steps == artifact.envelope.max_steps
    assert telemetry.event_count == reconstructed.event_count == index_entry.replay_event_count == len(
        artifact.events
    )
    run_metadata = dict(artifact.envelope.run_metadata)
    assert run_metadata["benchmark_version"] == index_entry.benchmark_version == "0.1"
    assert run_metadata["scenario_version"] == index_entry.scenario_version == "1.0"
    assert run_metadata["scoring_version"] == index_entry.scoring_version == "phase3-v1"
    assert index_entry.schema_version == RUN_METADATA_INDEX_SCHEMA_VERSION
    assert tuple(key for key, _ in index_entry.artifact_refs) == REQUIRED_ARTIFACT_REF_KEYS
    assert dict(index_entry.artifact_refs)["replay_checksum"] == f"sha256:{checksum.digest_hex}"

    parsed = parse_run_metadata_index(index_entry.to_dict())
    assert parsed.accepted is True
    assert parsed.entry == index_entry


def test_phase4_gate_contract_outputs_are_deterministic_across_reruns() -> None:
    first = _build_gate_bundle()
    second = _build_gate_bundle()

    assert first["checksum"].digest_hex == second["checksum"].digest_hex
    assert first["telemetry"].to_canonical_json() == second["telemetry"].to_canonical_json()
    assert first["reconstructed"].to_canonical_json() == second["reconstructed"].to_canonical_json()
    assert first["index_entry"].to_canonical_json() == second["index_entry"].to_canonical_json()
