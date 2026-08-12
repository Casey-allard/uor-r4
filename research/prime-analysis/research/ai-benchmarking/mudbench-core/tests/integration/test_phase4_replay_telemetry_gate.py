from __future__ import annotations

import json
import re
from typing import Any

from replay.integrity.replay_checksum import compute_replay_artifact_checksum
from replay.integrity.replay_verifier import verify_replay_reconstruction
from replay.logging.replay_artifact import emit_replay_artifact
from replay.logging.replay_log_format import REPLAY_LOG_SCHEMA_VERSION
from replay.reconstruction.reconstructor import reconstruct_replay_state
from replay.telemetry.collector import collect_replay_telemetry
from replay.telemetry.metadata_index import (
    REQUIRED_ARTIFACT_REF_KEYS,
    RUN_METADATA_INDEX_SCHEMA_VERSION,
    build_run_metadata_index,
)


def _envelope_payload() -> dict[str, object]:
    return {
        "schema_version": REPLAY_LOG_SCHEMA_VERSION,
        "run_id": "phase4-gate-run",
        "benchmark_id": "phase4-benchmark",
        "scenario_id": "phase4-gate-scenario",
        "initial_seed": 91,
        "seed_source": "run_seed",
        "actor_ids": ["agent-b", "agent-a"],
        "max_steps": 3,
        "run_metadata": {
            "benchmark_version": "0.1",
            "scenario_version": "1.0",
            "scoring_version": "phase3-v1",
            "phase": "phase4",
            "mode": "gate",
        },
    }


def _event_payload(
    *,
    step_index: int,
    event_type: str,
    actor_id: str | None,
    payload: dict[str, object],
) -> dict[str, object]:
    return {
        "run_id": "phase4-gate-run",
        "step_index": step_index,
        "event_type": event_type,
        "actor_id": actor_id,
        "payload": payload,
        "metadata": {},
    }


def _build_phase4_gate_artifact() -> tuple[str, dict[str, Any]]:
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
                step_index=0,
                event_type="observation_room",
                actor_id="agent-a",
                payload={"room_id": "room-mid"},
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
                payload={"target_id": "agent-b", "resulting_health": 8},
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
    artifact = emit_result.artifact

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

    payload: dict[str, Any] = {
        "replay": {
            "run_id": artifact.envelope.run_id,
            "benchmark_id": artifact.envelope.benchmark_id,
            "scenario_id": artifact.envelope.scenario_id,
            "initial_seed": artifact.envelope.initial_seed,
            "seed_source": artifact.envelope.seed_source,
            "max_steps": artifact.envelope.max_steps,
            "actor_ids": list(artifact.envelope.actor_ids),
            "benchmark_version": dict(artifact.envelope.run_metadata)["benchmark_version"],
            "scenario_version": dict(artifact.envelope.run_metadata)["scenario_version"],
            "scoring_version": dict(artifact.envelope.run_metadata)["scoring_version"],
        },
        "reconstruction": {
            "accepted": reconstructed_result.accepted,
            "event_count": reconstructed.event_count,
            "terminal_step": reconstructed.terminal_step,
            "verification_accepted": verification_result.accepted,
            "verification_matched": verification_result.matched,
            "verification_mismatches": list(verification_result.mismatches),
        },
        "telemetry": {
            "event_count": telemetry.event_count,
            "step_event_counts": [
                {"step_index": step_index, "count": count}
                for step_index, count in telemetry.step_event_counts
            ],
            "category_counts": [
                {"category": category, "count": count}
                for category, count in telemetry.category_counts
            ],
            "unknown_event_types": list(telemetry.unknown_event_types),
        },
        "index": {
            "schema_version": index_entry.schema_version,
            "run_id": index_entry.run_id,
            "benchmark_version": index_entry.benchmark_version,
            "scenario_id": index_entry.scenario_id,
            "scenario_version": index_entry.scenario_version,
            "scoring_version": index_entry.scoring_version,
            "initial_seed": index_entry.initial_seed,
            "replay_event_count": index_entry.replay_event_count,
            "terminal_step": index_entry.terminal_step,
            "artifact_ref_keys": [key for key, _ in index_entry.artifact_refs],
            "artifact_refs": [
                {"name": name, "ref": ref}
                for name, ref in index_entry.artifact_refs
            ],
        },
        "consistency": {
            "telemetry_matches_reconstructed_event_count": telemetry.event_count
            == reconstructed.event_count,
            "index_matches_replay_identity": index_entry.run_id == artifact.envelope.run_id
            and index_entry.scenario_id == artifact.envelope.scenario_id
            and index_entry.initial_seed == artifact.envelope.initial_seed,
            "index_matches_replay_event_count": index_entry.replay_event_count == len(artifact.events),
            "index_checksum_matches_checksum_artifact_ref": dict(index_entry.artifact_refs)[
                "replay_checksum"
            ]
            == f"sha256:{checksum.digest_hex}",
        },
    }
    gate_artifact = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    return gate_artifact, payload


def test_phase4_replay_telemetry_gate_valid_contract() -> None:
    artifact, payload = _build_phase4_gate_artifact()
    del artifact

    assert payload["replay"] == {
        "run_id": "phase4-gate-run",
        "benchmark_id": "phase4-benchmark",
        "scenario_id": "phase4-gate-scenario",
        "initial_seed": 91,
        "seed_source": "run_seed",
        "max_steps": 3,
        "actor_ids": ["agent-a", "agent-b"],
        "benchmark_version": "0.1",
        "scenario_version": "1.0",
        "scoring_version": "phase3-v1",
    }
    assert payload["reconstruction"] == {
        "accepted": True,
        "event_count": 5,
        "terminal_step": 2,
        "verification_accepted": True,
        "verification_matched": True,
        "verification_mismatches": [],
    }
    assert payload["telemetry"] == {
        "event_count": 5,
        "step_event_counts": [
            {"step_index": 0, "count": 2},
            {"step_index": 1, "count": 2},
            {"step_index": 2, "count": 1},
        ],
        "category_counts": [
            {"category": "action", "count": 3},
            {"category": "rejection", "count": 1},
            {"category": "state_change", "count": 0},
            {"category": "observation", "count": 1},
            {"category": "unknown", "count": 0},
        ],
        "unknown_event_types": [],
    }
    assert payload["index"]["schema_version"] == RUN_METADATA_INDEX_SCHEMA_VERSION
    assert payload["index"]["run_id"] == "phase4-gate-run"
    assert payload["index"]["benchmark_version"] == "0.1"
    assert payload["index"]["scenario_id"] == "phase4-gate-scenario"
    assert payload["index"]["scenario_version"] == "1.0"
    assert payload["index"]["scoring_version"] == "phase3-v1"
    assert payload["index"]["initial_seed"] == 91
    assert payload["index"]["replay_event_count"] == 5
    assert payload["index"]["terminal_step"] == 2
    assert payload["index"]["artifact_ref_keys"] == list(REQUIRED_ARTIFACT_REF_KEYS)
    for item in payload["index"]["artifact_refs"]:
        assert re.fullmatch(r"sha256:[0-9a-f]{64}", item["ref"]) is not None
    assert payload["consistency"] == {
        "telemetry_matches_reconstructed_event_count": True,
        "index_matches_replay_identity": True,
        "index_matches_replay_event_count": True,
        "index_checksum_matches_checksum_artifact_ref": True,
    }


def test_phase4_replay_telemetry_gate_reruns_are_byte_identical() -> None:
    first_artifact, _ = _build_phase4_gate_artifact()
    second_artifact, _ = _build_phase4_gate_artifact()

    assert first_artifact == second_artifact
