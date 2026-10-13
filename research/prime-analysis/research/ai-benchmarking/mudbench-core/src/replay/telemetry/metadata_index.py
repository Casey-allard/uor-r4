"""Deterministic run metadata indexing over replay/run artifacts."""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from replay.integrity.replay_checksum import ReplayArtifactChecksum
from replay.logging.replay_artifact import (
    ReplayArtifact,
    ReplayArtifactEmitResult,
    parse_replay_artifact,
)
from replay.reconstruction.reconstructor import (
    ReconstructedEntityState,
    ReconstructedReplayState,
    RejectedActionTrace,
)
from replay.telemetry.collector import ActorTelemetryMetrics, ReplayTelemetrySnapshot

RUN_METADATA_INDEX_SCHEMA_VERSION = "1.0"
REQUIRED_RUN_METADATA_INDEX_FIELDS = (
    "schema_version",
    "run_id",
    "benchmark_id",
    "benchmark_version",
    "scenario_id",
    "scenario_version",
    "scoring_version",
    "initial_seed",
    "seed_source",
    "max_steps",
    "actor_ids",
    "replay_event_count",
    "terminal_step",
    "artifact_refs",
)
REQUIRED_ARTIFACT_REF_KEYS = (
    "reconstructed_state",
    "replay_artifact",
    "replay_checksum",
    "telemetry_snapshot",
)
_SHA256_REF_RE = re.compile(r"^sha256:[0-9a-f]{64}$")

_REQUIRED_TELEMETRY_FIELDS = (
    "run_id",
    "initial_seed",
    "seed_source",
    "max_steps",
    "event_count",
    "step_event_counts",
    "event_type_counts",
    "category_counts",
    "unknown_event_types",
    "actor_metrics",
)
_REQUIRED_RECONSTRUCTED_FIELDS = (
    "run_id",
    "initial_seed",
    "seed_source",
    "max_steps",
    "terminal_step",
    "event_count",
    "applied_steps",
    "entities",
    "event_type_counts",
    "rejected_actions",
)


@dataclass(frozen=True, slots=True)
class RunMetadataIndexEntry:
    """Immutable versioned metadata index entry for one benchmark run."""

    schema_version: str
    run_id: str
    benchmark_id: str
    benchmark_version: str
    scenario_id: str
    scenario_version: str
    scoring_version: str
    initial_seed: int
    seed_source: str
    max_steps: int
    actor_ids: tuple[str, ...]
    replay_event_count: int
    terminal_step: int | None
    artifact_refs: tuple[tuple[str, str], ...]

    def __post_init__(self) -> None:
        if self.schema_version != RUN_METADATA_INDEX_SCHEMA_VERSION:
            raise ValueError(f"schema_version must be {RUN_METADATA_INDEX_SCHEMA_VERSION}")
        for field_name in (
            "run_id",
            "benchmark_id",
            "benchmark_version",
            "scenario_id",
            "scenario_version",
            "scoring_version",
            "seed_source",
        ):
            value = getattr(self, field_name)
            if not isinstance(value, str) or not value:
                raise ValueError(f"{field_name} must be a non-empty string")
        if not isinstance(self.initial_seed, int) or isinstance(self.initial_seed, bool) or self.initial_seed < 0:
            raise ValueError("initial_seed must be a non-negative integer")
        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")
        if (
            not isinstance(self.replay_event_count, int)
            or isinstance(self.replay_event_count, bool)
            or self.replay_event_count < 0
        ):
            raise ValueError("replay_event_count must be a non-negative integer")
        if self.terminal_step is not None:
            if (
                not isinstance(self.terminal_step, int)
                or isinstance(self.terminal_step, bool)
                or self.terminal_step < 0
            ):
                raise ValueError("terminal_step must be None or a non-negative integer")
            if self.terminal_step > self.max_steps:
                raise ValueError("terminal_step must be less than or equal to max_steps")

        object.__setattr__(self, "actor_ids", _normalize_actor_ids(self.actor_ids))
        object.__setattr__(self, "artifact_refs", _normalize_artifact_refs(self.artifact_refs))

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "RunMetadataIndexEntry":
        result = parse_run_metadata_index(payload)
        if not result.accepted or result.entry is None:
            raise ValueError(f"Invalid run metadata index payload: {result.reason}")
        return result.entry

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "run_id": self.run_id,
            "benchmark_id": self.benchmark_id,
            "benchmark_version": self.benchmark_version,
            "scenario_id": self.scenario_id,
            "scenario_version": self.scenario_version,
            "scoring_version": self.scoring_version,
            "initial_seed": self.initial_seed,
            "seed_source": self.seed_source,
            "max_steps": self.max_steps,
            "actor_ids": list(self.actor_ids),
            "replay_event_count": self.replay_event_count,
            "terminal_step": self.terminal_step,
            "artifact_refs": {key: value for key, value in self.artifact_refs},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class RunMetadataIndexParseResult:
    """Explicit accept/reject parse result for metadata index records."""

    accepted: bool
    entry: RunMetadataIndexEntry | None = None
    reason: str | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.entry is None:
                raise ValueError("accepted parse result must include entry")
            if self.reason is not None:
                raise ValueError("accepted parse result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected parse result must include reason")
        if self.entry is not None:
            raise ValueError("rejected parse result must not include entry")


@dataclass(frozen=True, slots=True)
class RunMetadataIndexBuildResult:
    """Explicit accept/reject build result for metadata indexing."""

    accepted: bool
    entry: RunMetadataIndexEntry | None = None
    reason: str | None = None
    artifact_result: ReplayArtifactEmitResult | None = None

    def __post_init__(self) -> None:
        if self.accepted:
            if self.entry is None:
                raise ValueError("accepted build result must include entry")
            if self.reason is not None:
                raise ValueError("accepted build result must not include reason")
            return
        if self.reason is None:
            raise ValueError("rejected build result must include reason")
        if self.entry is not None:
            raise ValueError("rejected build result must not include entry")


def build_run_metadata_index(
    *,
    replay_artifact: object,
    telemetry_snapshot: object,
    reconstructed_state: object,
    replay_checksum: object,
) -> RunMetadataIndexBuildResult:
    """Build deterministic versioned run metadata index entry."""
    artifact, artifact_result, reason = _resolve_replay_artifact(replay_artifact)
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    if artifact is None:
        raise RuntimeError("internal error resolving replay artifact")

    telemetry, reason = _resolve_telemetry_snapshot(telemetry_snapshot)
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    if telemetry is None:
        raise RuntimeError("internal error resolving telemetry snapshot")

    reconstructed, reason = _resolve_reconstructed_state(reconstructed_state)
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    if reconstructed is None:
        raise RuntimeError("internal error resolving reconstructed state")

    checksum, reason = _resolve_replay_checksum(replay_checksum)
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    if checksum is None:
        raise RuntimeError("internal error resolving replay checksum")

    envelope = artifact.envelope
    if telemetry.run_id != envelope.run_id:
        return _rejected_build_result(
            reason="run_id_mismatch:telemetry_vs_artifact",
            artifact_result=artifact_result,
        )
    if reconstructed.run_id != envelope.run_id:
        return _rejected_build_result(
            reason="run_id_mismatch:reconstructed_vs_artifact",
            artifact_result=artifact_result,
        )

    if telemetry.initial_seed != envelope.initial_seed:
        return _rejected_build_result(
            reason="initial_seed_mismatch:telemetry_vs_artifact",
            artifact_result=artifact_result,
        )
    if reconstructed.initial_seed != envelope.initial_seed:
        return _rejected_build_result(
            reason="initial_seed_mismatch:reconstructed_vs_artifact",
            artifact_result=artifact_result,
        )

    if telemetry.seed_source != envelope.seed_source:
        return _rejected_build_result(
            reason="seed_source_mismatch:telemetry_vs_artifact",
            artifact_result=artifact_result,
        )
    if reconstructed.seed_source != envelope.seed_source:
        return _rejected_build_result(
            reason="seed_source_mismatch:reconstructed_vs_artifact",
            artifact_result=artifact_result,
        )

    if telemetry.max_steps != envelope.max_steps:
        return _rejected_build_result(
            reason="max_steps_mismatch:telemetry_vs_artifact",
            artifact_result=artifact_result,
        )
    if reconstructed.max_steps != envelope.max_steps:
        return _rejected_build_result(
            reason="max_steps_mismatch:reconstructed_vs_artifact",
            artifact_result=artifact_result,
        )

    run_metadata = {key: value for key, value in envelope.run_metadata}
    benchmark_version, reason = _read_required_version_from_run_metadata(
        run_metadata,
        "benchmark_version",
    )
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    scenario_version, reason = _read_required_version_from_run_metadata(
        run_metadata,
        "scenario_version",
    )
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)
    scoring_version, reason = _read_required_version_from_run_metadata(
        run_metadata,
        "scoring_version",
    )
    if reason is not None:
        return _rejected_build_result(reason=reason, artifact_result=artifact_result)

    replay_event_count = len(artifact.events)
    if telemetry.event_count != replay_event_count:
        return _rejected_build_result(
            reason="event_count_mismatch:telemetry_vs_artifact",
            artifact_result=artifact_result,
        )
    if reconstructed.event_count != replay_event_count:
        return _rejected_build_result(
            reason="event_count_mismatch:reconstructed_vs_artifact",
            artifact_result=artifact_result,
        )

    envelope_actor_ids = set(envelope.actor_ids)
    telemetry_actor_ids = {metric.actor_id for metric in telemetry.actor_metrics}
    if not envelope_actor_ids.issubset(telemetry_actor_ids):
        return _rejected_build_result(
            reason="actor_ids_mismatch:telemetry_missing_envelope_actor",
            artifact_result=artifact_result,
        )
    reconstructed_actor_ids = {entity.entity_id for entity in reconstructed.entities}
    if not envelope_actor_ids.issubset(reconstructed_actor_ids):
        return _rejected_build_result(
            reason="actor_ids_mismatch:reconstructed_missing_envelope_actor",
            artifact_result=artifact_result,
        )

    replay_bytes = artifact.to_canonical_bytes()
    replay_digest = hashlib.sha256(replay_bytes).hexdigest()
    if checksum.digest_hex != replay_digest:
        return _rejected_build_result(
            reason="replay_checksum_mismatch:digest",
            artifact_result=artifact_result,
        )
    if checksum.canonical_size_bytes != len(replay_bytes):
        return _rejected_build_result(
            reason="replay_checksum_mismatch:size",
            artifact_result=artifact_result,
        )

    telemetry_digest = hashlib.sha256(telemetry.to_canonical_json().encode("utf-8")).hexdigest()
    reconstructed_digest = hashlib.sha256(
        reconstructed.to_canonical_json().encode("utf-8")
    ).hexdigest()

    entry = RunMetadataIndexEntry(
        schema_version=RUN_METADATA_INDEX_SCHEMA_VERSION,
        run_id=envelope.run_id,
        benchmark_id=envelope.benchmark_id,
        benchmark_version=benchmark_version,
        scenario_id=envelope.scenario_id,
        scenario_version=scenario_version,
        scoring_version=scoring_version,
        initial_seed=envelope.initial_seed,
        seed_source=envelope.seed_source,
        max_steps=envelope.max_steps,
        actor_ids=envelope.actor_ids,
        replay_event_count=replay_event_count,
        terminal_step=reconstructed.terminal_step,
        artifact_refs=(
            ("replay_artifact", f"sha256:{replay_digest}"),
            ("replay_checksum", f"sha256:{checksum.digest_hex}"),
            ("telemetry_snapshot", f"sha256:{telemetry_digest}"),
            ("reconstructed_state", f"sha256:{reconstructed_digest}"),
        ),
    )
    return _accepted_build_result(entry=entry, artifact_result=artifact_result)


def parse_run_metadata_index(payload: object) -> RunMetadataIndexParseResult:
    """Parse one index row payload with explicit acceptance semantics."""
    if not isinstance(payload, Mapping):
        return _rejected_parse_result(reason="payload_not_mapping")

    missing_fields = [field for field in REQUIRED_RUN_METADATA_INDEX_FIELDS if field not in payload]
    if missing_fields:
        return _rejected_parse_result(f"missing_required_fields:{','.join(missing_fields)}")

    run_id, reason = _read_required_non_empty_string(payload, "run_id")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    benchmark_id, reason = _read_required_non_empty_string(payload, "benchmark_id")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    benchmark_version, reason = _read_required_non_empty_string(payload, "benchmark_version")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    scenario_id, reason = _read_required_non_empty_string(payload, "scenario_id")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    scenario_version, reason = _read_required_non_empty_string(payload, "scenario_version")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    scoring_version, reason = _read_required_non_empty_string(payload, "scoring_version")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    seed_source, reason = _read_required_non_empty_string(payload, "seed_source")
    if reason is not None:
        return _rejected_parse_result(reason=reason)

    schema_version = payload.get("schema_version")
    if not isinstance(schema_version, str) or not schema_version:
        return _rejected_parse_result(reason="schema_version_must_be_non_empty_string")

    initial_seed, reason = _read_required_non_negative_int(payload, "initial_seed")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    max_steps, reason = _read_required_positive_int(payload, "max_steps")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    replay_event_count, reason = _read_required_non_negative_int(payload, "replay_event_count")
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    terminal_step, reason = _read_optional_non_negative_int(payload, "terminal_step")
    if reason is not None:
        return _rejected_parse_result(reason=reason)

    actor_ids, reason = _parse_actor_ids(payload.get("actor_ids"))
    if reason is not None:
        return _rejected_parse_result(reason=reason)
    artifact_refs, reason = _parse_artifact_refs(payload.get("artifact_refs"))
    if reason is not None:
        return _rejected_parse_result(reason=reason)

    try:
        entry = RunMetadataIndexEntry(
            schema_version=schema_version,
            run_id=run_id,
            benchmark_id=benchmark_id,
            benchmark_version=benchmark_version,
            scenario_id=scenario_id,
            scenario_version=scenario_version,
            scoring_version=scoring_version,
            initial_seed=initial_seed,
            seed_source=seed_source,
            max_steps=max_steps,
            actor_ids=actor_ids,
            replay_event_count=replay_event_count,
            terminal_step=terminal_step,
            artifact_refs=artifact_refs,
        )
    except ValueError as exc:
        return _rejected_parse_result(reason=str(exc))
    return _accepted_parse_result(entry=entry)


def _resolve_replay_artifact(
    value: object,
) -> tuple[ReplayArtifact | None, ReplayArtifactEmitResult | None, str | None]:
    if isinstance(value, ReplayArtifact):
        return value, None, None
    if isinstance(value, Mapping):
        parse_result = parse_replay_artifact(value)
        if not parse_result.accepted or parse_result.artifact is None:
            parse_reason = parse_result.reason or "unknown_parse_rejection"
            return None, parse_result, f"invalid_artifact:{parse_reason}"
        return parse_result.artifact, parse_result, None
    return None, None, "replay_artifact_must_be_replay_artifact_or_mapping"


def _resolve_telemetry_snapshot(value: object) -> tuple[ReplayTelemetrySnapshot | None, str | None]:
    if isinstance(value, ReplayTelemetrySnapshot):
        return value, None
    if isinstance(value, Mapping):
        return _parse_telemetry_snapshot_mapping(value)
    return None, "telemetry_snapshot_must_be_replay_telemetry_snapshot_or_mapping"


def _resolve_reconstructed_state(value: object) -> tuple[ReconstructedReplayState | None, str | None]:
    if isinstance(value, ReconstructedReplayState):
        return value, None
    if isinstance(value, Mapping):
        return _parse_reconstructed_state_mapping(value)
    return None, "reconstructed_state_must_be_reconstructed_replay_state_or_mapping"


def _resolve_replay_checksum(value: object) -> tuple[ReplayArtifactChecksum | None, str | None]:
    if isinstance(value, ReplayArtifactChecksum):
        return value, None
    if isinstance(value, Mapping):
        return _parse_replay_checksum_mapping(value)
    return None, "replay_checksum_must_be_replay_artifact_checksum_or_mapping"


def _parse_telemetry_snapshot_mapping(
    payload: Mapping[str, Any],
) -> tuple[ReplayTelemetrySnapshot | None, str | None]:
    missing_fields = [field for field in _REQUIRED_TELEMETRY_FIELDS if field not in payload]
    if missing_fields:
        return None, f"invalid_telemetry_snapshot:missing_required_fields:{','.join(missing_fields)}"

    run_id, reason = _read_required_non_empty_string(payload, "run_id")
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    initial_seed, reason = _read_required_non_negative_int(payload, "initial_seed")
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    seed_source, reason = _read_required_non_empty_string(payload, "seed_source")
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    max_steps, reason = _read_required_positive_int(payload, "max_steps")
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    event_count, reason = _read_required_non_negative_int(payload, "event_count")
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"

    step_event_counts, reason = _parse_step_event_counts(payload.get("step_event_counts"))
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    event_type_counts, reason = _parse_event_type_counts(payload.get("event_type_counts"))
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    category_counts, reason = _parse_category_counts(payload.get("category_counts"))
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    unknown_event_types, reason = _parse_unknown_event_types(payload.get("unknown_event_types"))
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"
    actor_metrics, reason = _parse_actor_metrics(payload.get("actor_metrics"))
    if reason is not None:
        return None, f"invalid_telemetry_snapshot:{reason}"

    try:
        snapshot = ReplayTelemetrySnapshot(
            run_id=run_id,
            initial_seed=initial_seed,
            seed_source=seed_source,
            max_steps=max_steps,
            event_count=event_count,
            step_event_counts=step_event_counts,
            event_type_counts=event_type_counts,
            category_counts=category_counts,
            unknown_event_types=unknown_event_types,
            actor_metrics=actor_metrics,
        )
    except ValueError as exc:
        return None, f"invalid_telemetry_snapshot:{exc}"
    return snapshot, None


def _parse_reconstructed_state_mapping(
    payload: Mapping[str, Any],
) -> tuple[ReconstructedReplayState | None, str | None]:
    missing_fields = [field for field in _REQUIRED_RECONSTRUCTED_FIELDS if field not in payload]
    if missing_fields:
        return None, f"invalid_reconstructed_state:missing_required_fields:{','.join(missing_fields)}"

    run_id, reason = _read_required_non_empty_string(payload, "run_id")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    initial_seed, reason = _read_required_non_negative_int(payload, "initial_seed")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    seed_source, reason = _read_required_non_empty_string(payload, "seed_source")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    max_steps, reason = _read_required_positive_int(payload, "max_steps")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    terminal_step, reason = _read_optional_non_negative_int(payload, "terminal_step")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    event_count, reason = _read_required_non_negative_int(payload, "event_count")
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"

    applied_steps, reason = _parse_applied_steps(payload.get("applied_steps"))
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    entities, reason = _parse_reconstructed_entities(payload.get("entities"))
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    event_type_counts, reason = _parse_event_type_counts(payload.get("event_type_counts"))
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    rejected_actions, reason = _parse_rejected_actions(payload.get("rejected_actions"))
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"
    canonical_state_json, reason = _parse_optional_canonical_state(payload.get("canonical_state"))
    if reason is not None:
        return None, f"invalid_reconstructed_state:{reason}"

    try:
        reconstructed = ReconstructedReplayState(
            run_id=run_id,
            initial_seed=initial_seed,
            seed_source=seed_source,
            max_steps=max_steps,
            terminal_step=terminal_step,
            event_count=event_count,
            applied_steps=applied_steps,
            entities=entities,
            event_type_counts=event_type_counts,
            rejected_actions=rejected_actions,
            canonical_state_json=canonical_state_json,
        )
    except ValueError as exc:
        return None, f"invalid_reconstructed_state:{exc}"
    return reconstructed, None


def _parse_replay_checksum_mapping(
    payload: Mapping[str, Any],
) -> tuple[ReplayArtifactChecksum | None, str | None]:
    missing_fields = [
        field
        for field in ("algorithm", "digest_hex", "canonical_size_bytes")
        if field not in payload
    ]
    if missing_fields:
        return None, f"invalid_replay_checksum:missing_required_fields:{','.join(missing_fields)}"
    try:
        checksum = ReplayArtifactChecksum(
            algorithm=payload.get("algorithm"),
            digest_hex=payload.get("digest_hex"),
            canonical_size_bytes=payload.get("canonical_size_bytes"),
        )
    except ValueError as exc:
        return None, f"invalid_replay_checksum:{exc}"
    return checksum, None


def _parse_optional_canonical_state(value: object) -> tuple[str | None, str | None]:
    if value is None:
        return None, None
    if not isinstance(value, Mapping):
        return None, "canonical_state_must_be_mapping"
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True), None


def _parse_actor_ids(value: object) -> tuple[tuple[str, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "actor_ids_must_be_sequence_of_non_empty_strings"
    actor_ids: list[str] = []
    seen: set[str] = set()
    for index, actor_id in enumerate(value):
        if not isinstance(actor_id, str) or not actor_id:
            return (), f"actor_ids_item_invalid_at_index:{index}"
        if actor_id in seen:
            return (), f"actor_ids_duplicate:{actor_id}"
        seen.add(actor_id)
        actor_ids.append(actor_id)
    if not actor_ids:
        return (), "actor_ids_must_contain_at_least_one_actor"
    return tuple(sorted(actor_ids)), None


def _parse_artifact_refs(value: object) -> tuple[tuple[tuple[str, str], ...], str | None]:
    if not isinstance(value, Mapping):
        return (), "artifact_refs_must_be_mapping"

    parsed: list[tuple[str, str]] = []
    seen: set[str] = set()
    for key, ref_value in value.items():
        if not isinstance(key, str) or not key:
            return (), "artifact_refs_key_must_be_non_empty_string"
        if key in seen:
            return (), f"artifact_refs_duplicate_key:{key}"
        seen.add(key)
        if not isinstance(ref_value, str) or not _SHA256_REF_RE.fullmatch(ref_value):
            return (), f"artifact_ref_invalid:{key}"
        parsed.append((key, ref_value))

    parsed.sort(key=lambda item: item[0])
    parsed_keys = tuple(key for key, _ in parsed)
    if parsed_keys != REQUIRED_ARTIFACT_REF_KEYS:
        return (), (
            "artifact_refs_must_include_exact_keys:"
            + ",".join(REQUIRED_ARTIFACT_REF_KEYS)
        )
    return tuple(parsed), None


def _parse_step_event_counts(value: object) -> tuple[tuple[tuple[int, int], ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "step_event_counts_must_be_sequence"
    parsed: list[tuple[int, int]] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"step_event_count_must_be_mapping_at_index:{index}"
        step_index, reason = _read_required_non_negative_int(item, "step_index")
        if reason is not None:
            return (), f"step_event_count_invalid_at_index:{index}:{reason}"
        count, reason = _read_required_non_negative_int(item, "count")
        if reason is not None:
            return (), f"step_event_count_invalid_at_index:{index}:{reason}"
        parsed.append((step_index, count))
    return tuple(parsed), None


def _parse_event_type_counts(value: object) -> tuple[tuple[tuple[str, int], ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "event_type_counts_must_be_sequence"
    parsed: list[tuple[str, int]] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"event_type_count_must_be_mapping_at_index:{index}"
        event_type, reason = _read_required_non_empty_string(item, "event_type")
        if reason is not None:
            return (), f"event_type_count_invalid_at_index:{index}:{reason}"
        count, reason = _read_required_non_negative_int(item, "count")
        if reason is not None:
            return (), f"event_type_count_invalid_at_index:{index}:{reason}"
        parsed.append((event_type, count))
    return tuple(parsed), None


def _parse_category_counts(value: object) -> tuple[tuple[tuple[str, int], ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "category_counts_must_be_sequence"
    parsed: list[tuple[str, int]] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"category_count_must_be_mapping_at_index:{index}"
        category, reason = _read_required_non_empty_string(item, "category")
        if reason is not None:
            return (), f"category_count_invalid_at_index:{index}:{reason}"
        count, reason = _read_required_non_negative_int(item, "count")
        if reason is not None:
            return (), f"category_count_invalid_at_index:{index}:{reason}"
        parsed.append((category, count))
    return tuple(parsed), None


def _parse_unknown_event_types(value: object) -> tuple[tuple[str, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "unknown_event_types_must_be_sequence"
    parsed: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item:
            return (), f"unknown_event_type_invalid_at_index:{index}"
        parsed.append(item)
    return tuple(parsed), None


def _parse_actor_metrics(value: object) -> tuple[tuple[ActorTelemetryMetrics, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "actor_metrics_must_be_sequence"
    parsed: list[ActorTelemetryMetrics] = []
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"actor_metric_must_be_mapping_at_index:{index}"
        actor_id, reason = _read_required_non_empty_string(item, "actor_id")
        if reason is not None:
            return (), f"actor_metric_invalid_at_index:{index}:{reason}"
        event_count, reason = _read_required_non_negative_int(item, "event_count")
        if reason is not None:
            return (), f"actor_metric_invalid_at_index:{index}:{reason}"
        action_count, reason = _read_required_non_negative_int(item, "action_count")
        if reason is not None:
            return (), f"actor_metric_invalid_at_index:{index}:{reason}"
        rejected_count, reason = _read_required_non_negative_int(item, "rejected_count")
        if reason is not None:
            return (), f"actor_metric_invalid_at_index:{index}:{reason}"
        unknown_event_count, reason = _read_required_non_negative_int(item, "unknown_event_count")
        if reason is not None:
            return (), f"actor_metric_invalid_at_index:{index}:{reason}"
        try:
            parsed.append(
                ActorTelemetryMetrics(
                    actor_id=actor_id,
                    event_count=event_count,
                    action_count=action_count,
                    rejected_count=rejected_count,
                    unknown_event_count=unknown_event_count,
                )
            )
        except ValueError as exc:
            return (), f"actor_metric_invalid_at_index:{index}:{exc}"
    return tuple(parsed), None


def _parse_applied_steps(value: object) -> tuple[tuple[int, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "applied_steps_must_be_sequence"
    parsed: list[int] = []
    for index, item in enumerate(value):
        if not isinstance(item, int) or isinstance(item, bool) or item < 0:
            return (), f"applied_steps_invalid_at_index:{index}"
        parsed.append(item)
    return tuple(parsed), None


def _parse_reconstructed_entities(
    value: object,
) -> tuple[tuple[ReconstructedEntityState, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "entities_must_be_sequence"
    parsed: list[ReconstructedEntityState] = []
    required_fields = ("entity_id", "location", "health", "inventory", "last_event_type")
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"entity_must_be_mapping_at_index:{index}"
        missing_fields = [field for field in required_fields if field not in item]
        if missing_fields:
            return (), f"entity_missing_required_fields_at_index:{index}:{','.join(missing_fields)}"
        entity_id, reason = _read_required_non_empty_string(item, "entity_id")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"
        location, reason = _read_optional_non_empty_string(item, "location")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"
        health, reason = _read_optional_int(item, "health")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"
        inventory, reason = _parse_inventory(item.get("inventory"))
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"
        last_event_type, reason = _read_optional_non_empty_string(item, "last_event_type")
        if reason is not None:
            return (), f"entity_invalid_at_index:{index}:{reason}"
        try:
            parsed.append(
                ReconstructedEntityState(
                    entity_id=entity_id,
                    location=location,
                    health=health,
                    inventory=inventory,
                    last_event_type=last_event_type,
                )
            )
        except ValueError as exc:
            return (), f"entity_invalid_at_index:{index}:{exc}"
    return tuple(parsed), None


def _parse_rejected_actions(
    value: object,
) -> tuple[tuple[RejectedActionTrace, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "rejected_actions_must_be_sequence"
    parsed: list[RejectedActionTrace] = []
    required_fields = ("step_index", "actor_id", "action_type", "reason")
    for index, item in enumerate(value):
        if not isinstance(item, Mapping):
            return (), f"rejected_action_must_be_mapping_at_index:{index}"
        missing_fields = [field for field in required_fields if field not in item]
        if missing_fields:
            return (), f"rejected_action_missing_required_fields_at_index:{index}:{','.join(missing_fields)}"
        step_index, reason = _read_required_non_negative_int(item, "step_index")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        actor_id, reason = _read_optional_non_empty_string(item, "actor_id")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        action_type, reason = _read_required_non_empty_string(item, "action_type")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        reason_value, reason = _read_required_non_empty_string(item, "reason")
        if reason is not None:
            return (), f"rejected_action_invalid_at_index:{index}:{reason}"
        try:
            parsed.append(
                RejectedActionTrace(
                    step_index=step_index,
                    actor_id=actor_id,
                    action_type=action_type,
                    reason=reason_value,
                )
            )
        except ValueError as exc:
            return (), f"rejected_action_invalid_at_index:{index}:{exc}"
    return tuple(parsed), None


def _parse_inventory(value: object) -> tuple[tuple[str, ...], str | None]:
    if not _is_non_string_sequence(value):
        return (), "inventory_must_be_sequence"
    parsed: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item:
            return (), f"inventory_invalid_at_index:{index}"
        parsed.append(item)
    return tuple(parsed), None


def _normalize_actor_ids(actor_ids: Sequence[str]) -> tuple[str, ...]:
    if isinstance(actor_ids, (str, bytes)) or not isinstance(actor_ids, Sequence):
        raise ValueError("actor_ids must be a sequence of non-empty strings")
    seen: set[str] = set()
    parsed: list[str] = []
    for actor_id in actor_ids:
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError("actor_ids must contain non-empty strings")
        if actor_id in seen:
            raise ValueError(f"duplicate actor_id: {actor_id}")
        seen.add(actor_id)
        parsed.append(actor_id)
    if not parsed:
        raise ValueError("actor_ids must contain at least one actor")
    return tuple(sorted(parsed))


def _normalize_artifact_refs(refs: Sequence[tuple[str, str]]) -> tuple[tuple[str, str], ...]:
    if isinstance(refs, (str, bytes)) or not isinstance(refs, Sequence):
        raise ValueError("artifact_refs must be a sequence of key/value tuples")
    parsed: list[tuple[str, str]] = []
    seen: set[str] = set()
    for item in refs:
        if not isinstance(item, tuple) or len(item) != 2:
            raise ValueError("artifact_refs entries must be key/value tuples")
        key, value = item
        if not isinstance(key, str) or not key:
            raise ValueError("artifact_refs keys must be non-empty strings")
        if key in seen:
            raise ValueError(f"duplicate artifact_ref key: {key}")
        seen.add(key)
        if not isinstance(value, str) or not _SHA256_REF_RE.fullmatch(value):
            raise ValueError(f"artifact_ref value must be sha256:<hex64> for key {key}")
        parsed.append((key, value))
    parsed.sort(key=lambda item: item[0])
    keys = tuple(key for key, _ in parsed)
    if keys != REQUIRED_ARTIFACT_REF_KEYS:
        raise ValueError(
            "artifact_refs must include exact keys: " + ",".join(REQUIRED_ARTIFACT_REF_KEYS)
        )
    return tuple(parsed)


def _read_required_version_from_run_metadata(
    run_metadata: Mapping[str, Any],
    field_name: str,
) -> tuple[str, str | None]:
    value = run_metadata.get(field_name)
    if not isinstance(value, str) or not value:
        return "", f"run_metadata_missing_or_invalid:{field_name}"
    return value, None


def _read_required_non_empty_string(payload: Mapping[str, Any], field_name: str) -> tuple[str, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        return "", f"{field_name}_must_be_non_empty_string"
    return value, None


def _read_optional_non_empty_string(payload: Mapping[str, Any], field_name: str) -> tuple[str | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, str) or not value:
        return None, f"{field_name}_must_be_none_or_non_empty_string"
    return value, None


def _read_required_non_negative_int(payload: Mapping[str, Any], field_name: str) -> tuple[int, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        return 0, f"{field_name}_must_be_non_negative_integer"
    return value, None


def _read_required_positive_int(payload: Mapping[str, Any], field_name: str) -> tuple[int, str | None]:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        return 0, f"{field_name}_must_be_positive_integer"
    return value, None


def _read_optional_non_negative_int(payload: Mapping[str, Any], field_name: str) -> tuple[int | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        return None, f"{field_name}_must_be_none_or_non_negative_integer"
    return value, None


def _read_optional_int(payload: Mapping[str, Any], field_name: str) -> tuple[int | None, str | None]:
    value = payload.get(field_name)
    if value is None:
        return None, None
    if not isinstance(value, int) or isinstance(value, bool):
        return None, f"{field_name}_must_be_none_or_integer"
    return value, None


def _is_non_string_sequence(value: object) -> bool:
    return isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray))


def _accepted_parse_result(*, entry: RunMetadataIndexEntry) -> RunMetadataIndexParseResult:
    return RunMetadataIndexParseResult(accepted=True, entry=entry)


def _rejected_parse_result(reason: str) -> RunMetadataIndexParseResult:
    return RunMetadataIndexParseResult(accepted=False, reason=reason)


def _accepted_build_result(
    *,
    entry: RunMetadataIndexEntry,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> RunMetadataIndexBuildResult:
    return RunMetadataIndexBuildResult(
        accepted=True,
        entry=entry,
        artifact_result=artifact_result,
    )


def _rejected_build_result(
    *,
    reason: str,
    artifact_result: ReplayArtifactEmitResult | None = None,
) -> RunMetadataIndexBuildResult:
    return RunMetadataIndexBuildResult(
        accepted=False,
        reason=reason,
        artifact_result=artifact_result,
    )
