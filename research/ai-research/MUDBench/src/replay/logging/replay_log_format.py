"""Canonical immutable replay log envelope schema and parser."""

from __future__ import annotations

import json
import math
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

REPLAY_LOG_SCHEMA_VERSION = "1.0"
MAX_SEED = 2_147_483_647
SEED_SOURCES = ("seed_override", "run_seed", "scenario_seed", "derived")
REQUIRED_REPLAY_ENVELOPE_FIELDS = (
    "schema_version",
    "run_id",
    "benchmark_id",
    "scenario_id",
    "initial_seed",
    "seed_source",
    "actor_ids",
    "max_steps",
)
REQUIRED_RUN_METADATA_VERSION_FIELDS = (
    "benchmark_version",
    "scenario_version",
    "scoring_version",
)

ReplayEnvelopeScalar = str | int | float | bool | None
_INVALID_VALUE = object()


@dataclass(frozen=True, slots=True)
class ReplayLogEnvelope:
    """Immutable canonical replay log envelope for one run."""

    schema_version: str
    run_id: str
    benchmark_id: str
    scenario_id: str
    initial_seed: int
    seed_source: str
    actor_ids: tuple[str, ...]
    max_steps: int
    run_metadata: tuple[tuple[str, ReplayEnvelopeScalar], ...] = ()

    def __post_init__(self) -> None:
        if not isinstance(self.schema_version, str) or not self.schema_version:
            raise ValueError("schema_version must be a non-empty string")
        if self.schema_version != REPLAY_LOG_SCHEMA_VERSION:
            raise ValueError(f"unsupported schema_version: {self.schema_version}")

        for field_name in ("run_id", "benchmark_id", "scenario_id"):
            value = getattr(self, field_name)
            if not isinstance(value, str) or not value:
                raise ValueError(f"{field_name} must be a non-empty string")

        if not isinstance(self.initial_seed, int) or isinstance(self.initial_seed, bool):
            raise ValueError("initial_seed must be an integer")
        if self.initial_seed < 0 or self.initial_seed > MAX_SEED:
            raise ValueError(f"initial_seed must be within [0, {MAX_SEED}]")

        if not isinstance(self.seed_source, str) or self.seed_source not in SEED_SOURCES:
            raise ValueError(f"seed_source must be one of: {','.join(SEED_SOURCES)}")

        normalized_actor_ids = _normalize_actor_ids(self.actor_ids)
        object.__setattr__(self, "actor_ids", normalized_actor_ids)

        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")

        normalized_metadata = _normalize_key_value_pairs(self.run_metadata, field_name="run_metadata")
        _validate_required_run_metadata_versions(normalized_metadata)
        object.__setattr__(self, "run_metadata", normalized_metadata)

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "ReplayLogEnvelope":
        result = parse_replay_log_envelope(payload)
        if not result.accepted or result.envelope is None:
            raise ValueError(f"Invalid replay log envelope payload: {result.reason}")
        return result.envelope

    def to_dict(self) -> dict[str, Any]:
        return {
            "schema_version": self.schema_version,
            "run_id": self.run_id,
            "benchmark_id": self.benchmark_id,
            "scenario_id": self.scenario_id,
            "initial_seed": self.initial_seed,
            "seed_source": self.seed_source,
            "actor_ids": list(self.actor_ids),
            "max_steps": self.max_steps,
            "run_metadata": {key: value for key, value in self.run_metadata},
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


@dataclass(frozen=True, slots=True)
class ReplayLogEnvelopeParseResult:
    """Explicit accept/reject parse result for replay-log envelope payloads."""

    accepted: bool
    envelope: ReplayLogEnvelope | None = None
    reason: str | None = None


def parse_replay_log_envelope(payload: object) -> ReplayLogEnvelopeParseResult:
    if not isinstance(payload, Mapping):
        return _rejected_result("payload_not_mapping")

    missing_fields = [field for field in REQUIRED_REPLAY_ENVELOPE_FIELDS if field not in payload]
    if missing_fields:
        return _rejected_result(f"missing_required_fields:{','.join(missing_fields)}")

    schema_version = payload.get("schema_version")
    if not isinstance(schema_version, str) or not schema_version:
        return _rejected_result("schema_version_must_be_non_empty_string")
    if schema_version != REPLAY_LOG_SCHEMA_VERSION:
        return _rejected_result(f"unsupported_schema_version:{schema_version}")

    run_id = payload.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        return _rejected_result("run_id_must_be_non_empty_string")

    benchmark_id = payload.get("benchmark_id")
    if not isinstance(benchmark_id, str) or not benchmark_id:
        return _rejected_result("benchmark_id_must_be_non_empty_string")

    scenario_id = payload.get("scenario_id")
    if not isinstance(scenario_id, str) or not scenario_id:
        return _rejected_result("scenario_id_must_be_non_empty_string")

    initial_seed = payload.get("initial_seed")
    if not isinstance(initial_seed, int) or isinstance(initial_seed, bool):
        return _rejected_result("initial_seed_must_be_integer")
    if initial_seed < 0 or initial_seed > MAX_SEED:
        return _rejected_result(f"initial_seed_must_be_within_bounds:0..{MAX_SEED}")

    seed_source = payload.get("seed_source")
    if not isinstance(seed_source, str) or seed_source not in SEED_SOURCES:
        return _rejected_result(f"seed_source_must_be_one_of:{','.join(SEED_SOURCES)}")

    actor_ids = _normalize_actor_ids_for_parse(payload.get("actor_ids"))
    if actor_ids is _INVALID_VALUE:
        return _rejected_result("actor_ids_must_be_non_empty_unique_string_sequence")

    max_steps = payload.get("max_steps")
    if not isinstance(max_steps, int) or isinstance(max_steps, bool) or max_steps <= 0:
        return _rejected_result("max_steps_must_be_positive_integer")

    metadata_raw = payload.get("run_metadata", {})
    normalized_metadata = _normalize_mapping(metadata_raw)
    if normalized_metadata is _INVALID_VALUE:
        return _rejected_result("run_metadata_must_be_mapping_with_scalar_values")
    validation_reason = _validate_required_run_metadata_versions_for_parse(normalized_metadata)
    if validation_reason is not None:
        return _rejected_result(validation_reason)

    return _accepted_result(
        ReplayLogEnvelope(
            schema_version=schema_version,
            run_id=run_id,
            benchmark_id=benchmark_id,
            scenario_id=scenario_id,
            initial_seed=initial_seed,
            seed_source=seed_source,
            actor_ids=actor_ids,  # type: ignore[arg-type]
            max_steps=max_steps,
            run_metadata=normalized_metadata,  # type: ignore[arg-type]
        )
    )


def _normalize_actor_ids(actor_ids: Sequence[str]) -> tuple[str, ...]:
    if isinstance(actor_ids, (str, bytes)) or not isinstance(actor_ids, Sequence):
        raise ValueError("actor_ids must be a sequence of non-empty strings")
    normalized: list[str] = []
    seen: set[str] = set()
    for actor_id in actor_ids:
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError("actor_ids must contain non-empty strings")
        if actor_id in seen:
            raise ValueError(f"duplicate actor_id: {actor_id}")
        seen.add(actor_id)
        normalized.append(actor_id)
    if not normalized:
        raise ValueError("actor_ids must contain at least one actor")
    return tuple(sorted(normalized))


def _normalize_actor_ids_for_parse(value: object) -> tuple[str, ...] | object:
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        return _INVALID_VALUE
    actor_ids: list[str] = []
    seen: set[str] = set()
    for actor_id in value:
        if not isinstance(actor_id, str) or not actor_id:
            return _INVALID_VALUE
        if actor_id in seen:
            return _INVALID_VALUE
        seen.add(actor_id)
        actor_ids.append(actor_id)
    if not actor_ids:
        return _INVALID_VALUE
    actor_ids.sort()
    return tuple(actor_ids)


def _normalize_mapping(value: object) -> tuple[tuple[str, ReplayEnvelopeScalar], ...] | object:
    if not isinstance(value, Mapping):
        return _INVALID_VALUE
    keys = tuple(value.keys())
    for key in keys:
        if not isinstance(key, str) or not key:
            return _INVALID_VALUE

    normalized: list[tuple[str, ReplayEnvelopeScalar]] = []
    for key in sorted(keys):
        normalized_value = _normalize_scalar(value[key])
        if normalized_value is _INVALID_VALUE:
            return _INVALID_VALUE
        normalized.append((key, normalized_value))
    return tuple(normalized)


def _validate_required_run_metadata_versions(
    run_metadata: tuple[tuple[str, ReplayEnvelopeScalar], ...]
) -> None:
    metadata = {key: value for key, value in run_metadata}
    missing = [field for field in REQUIRED_RUN_METADATA_VERSION_FIELDS if field not in metadata]
    if missing:
        raise ValueError(f"run_metadata missing required fields: {','.join(missing)}")
    for field in REQUIRED_RUN_METADATA_VERSION_FIELDS:
        value = metadata[field]
        if not isinstance(value, str) or not value:
            raise ValueError(f"run_metadata.{field} must be a non-empty string")


def _validate_required_run_metadata_versions_for_parse(
    run_metadata: tuple[tuple[str, ReplayEnvelopeScalar], ...]
) -> str | None:
    metadata = {key: value for key, value in run_metadata}
    missing = [field for field in REQUIRED_RUN_METADATA_VERSION_FIELDS if field not in metadata]
    if missing:
        return f"run_metadata_missing_required_fields:{','.join(missing)}"
    for field in REQUIRED_RUN_METADATA_VERSION_FIELDS:
        value = metadata[field]
        if not isinstance(value, str) or not value:
            return f"run_metadata_{field}_must_be_non_empty_string"
    return None


def _normalize_key_value_pairs(
    pairs: Sequence[tuple[str, ReplayEnvelopeScalar]],
    *,
    field_name: str,
) -> tuple[tuple[str, ReplayEnvelopeScalar], ...]:
    if isinstance(pairs, (str, bytes)) or not isinstance(pairs, Sequence):
        raise ValueError(f"{field_name} must be a sequence of key/value pairs")

    normalized_pairs: list[tuple[str, ReplayEnvelopeScalar]] = []
    seen_keys: set[str] = set()
    for pair in pairs:
        if not isinstance(pair, tuple) or len(pair) != 2:
            raise ValueError(f"{field_name} entries must be key/value tuples")
        key, raw_value = pair
        if not isinstance(key, str) or not key:
            raise ValueError(f"{field_name} keys must be non-empty strings")
        if key in seen_keys:
            raise ValueError(f"duplicate key in {field_name}: {key}")
        seen_keys.add(key)
        normalized_value = _normalize_scalar(raw_value)
        if normalized_value is _INVALID_VALUE:
            raise ValueError(f"{field_name} values must be JSON scalar values")
        normalized_pairs.append((key, normalized_value))

    normalized_pairs.sort(key=lambda item: item[0])
    return tuple(normalized_pairs)


def _normalize_scalar(value: Any) -> ReplayEnvelopeScalar | object:
    if value is None:
        return None
    if isinstance(value, bool):
        return value
    if isinstance(value, str):
        return value
    if isinstance(value, int):
        return value
    if isinstance(value, float):
        if not math.isfinite(value):
            return _INVALID_VALUE
        return value
    return _INVALID_VALUE


def _accepted_result(envelope: ReplayLogEnvelope) -> ReplayLogEnvelopeParseResult:
    return ReplayLogEnvelopeParseResult(accepted=True, envelope=envelope)


def _rejected_result(reason: str) -> ReplayLogEnvelopeParseResult:
    return ReplayLogEnvelopeParseResult(accepted=False, reason=reason)
