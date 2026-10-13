"""Immutable canonical run-manifest schema."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from evaluation.benchmark_runner.run_config import BenchmarkRunConfig

_MAX_SEED = 2_147_483_647
_SEED_SOURCES = ("seed_override", "run_seed", "scenario_seed", "derived")


@dataclass(frozen=True, slots=True)
class RunManifest:
    """Canonical immutable manifest for a benchmark run configuration."""

    run_id: str
    benchmark_id: str
    benchmark_version: str
    scenario_id: str
    scenario_version: str
    scoring_version: str
    effective_seed: int
    seed_source: str
    actor_ids: tuple[str, ...]
    max_steps: int
    config_json: str

    def __post_init__(self) -> None:
        for field_name in (
            "run_id",
            "benchmark_id",
            "benchmark_version",
            "scenario_id",
            "scenario_version",
            "scoring_version",
        ):
            value = getattr(self, field_name)
            if not isinstance(value, str) or not value:
                raise ValueError(f"{field_name} must be a non-empty string")

        if not isinstance(self.effective_seed, int) or isinstance(self.effective_seed, bool):
            raise ValueError("effective_seed must be an integer")
        if self.effective_seed < 0 or self.effective_seed > _MAX_SEED:
            raise ValueError(f"effective_seed must be within [0, {_MAX_SEED}]")

        if not isinstance(self.seed_source, str) or self.seed_source not in _SEED_SOURCES:
            raise ValueError(f"seed_source must be one of: {','.join(_SEED_SOURCES)}")

        if isinstance(self.actor_ids, (str, bytes)) or not isinstance(self.actor_ids, Sequence):
            raise ValueError("actor_ids must be a sequence of non-empty strings")
        if len(self.actor_ids) == 0:
            raise ValueError("actor_ids must contain at least one actor")
        if tuple(sorted(self.actor_ids)) != self.actor_ids:
            raise ValueError("actor_ids must be sorted")
        seen_actor_ids: set[str] = set()
        for actor_id in self.actor_ids:
            if not isinstance(actor_id, str) or not actor_id:
                raise ValueError("actor_ids must contain non-empty strings")
            if actor_id in seen_actor_ids:
                raise ValueError(f"duplicate actor_id in actor_ids: {actor_id}")
            seen_actor_ids.add(actor_id)

        if not isinstance(self.max_steps, int) or isinstance(self.max_steps, bool) or self.max_steps <= 0:
            raise ValueError("max_steps must be a positive integer")

        if not isinstance(self.config_json, str) or not self.config_json:
            raise ValueError("config_json must be a non-empty string")
        try:
            parsed = json.loads(self.config_json)
        except json.JSONDecodeError as exc:
            raise ValueError("config_json must be valid JSON") from exc
        if not isinstance(parsed, Mapping):
            raise ValueError("config_json must encode an object")
        canonical = _to_canonical_json(parsed)
        if canonical != self.config_json:
            raise ValueError("config_json must be canonical JSON")

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "RunManifest":
        if not isinstance(payload, Mapping):
            raise ValueError("payload must be a mapping")

        config_raw = payload.get("config")
        if not isinstance(config_raw, Mapping):
            raise ValueError("config must be a mapping")
        actor_ids = _read_required_actor_ids(payload, "actor_ids")
        return cls(
            run_id=_read_required_string(payload, "run_id"),
            benchmark_id=_read_required_string(payload, "benchmark_id"),
            benchmark_version=_read_required_string(payload, "benchmark_version"),
            scenario_id=_read_required_string(payload, "scenario_id"),
            scenario_version=_read_required_string(payload, "scenario_version"),
            scoring_version=_read_required_string(payload, "scoring_version"),
            effective_seed=_read_required_int(payload, "effective_seed"),
            seed_source=_read_required_string(payload, "seed_source"),
            actor_ids=tuple(sorted(actor_ids)),
            max_steps=_read_required_positive_int(payload, "max_steps"),
            config_json=_to_canonical_json(config_raw),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "run_id": self.run_id,
            "benchmark_id": self.benchmark_id,
            "benchmark_version": self.benchmark_version,
            "scenario_id": self.scenario_id,
            "scenario_version": self.scenario_version,
            "scoring_version": self.scoring_version,
            "effective_seed": self.effective_seed,
            "seed_source": self.seed_source,
            "actor_ids": list(self.actor_ids),
            "max_steps": self.max_steps,
            "config": json.loads(self.config_json),
        }

    def to_canonical_json(self) -> str:
        return _to_canonical_json(self.to_dict())


def build_run_manifest(
    *,
    run_config: BenchmarkRunConfig | Mapping[str, Any],
    scenario_id: str,
    scenario_version: str,
    benchmark_version: str,
    scoring_version: str,
    max_steps: int,
) -> RunManifest:
    """Build canonical run manifest from run configuration."""
    if isinstance(run_config, BenchmarkRunConfig):
        resolved_config = run_config
    elif isinstance(run_config, Mapping):
        resolved_config = BenchmarkRunConfig.from_mapping(run_config)
    else:
        raise ValueError("run_config must be BenchmarkRunConfig or mapping")

    if not isinstance(scenario_id, str) or not scenario_id:
        raise ValueError("scenario_id must be a non-empty string")
    if not isinstance(scenario_version, str) or not scenario_version:
        raise ValueError("scenario_version must be a non-empty string")
    if not isinstance(benchmark_version, str) or not benchmark_version:
        raise ValueError("benchmark_version must be a non-empty string")
    if not isinstance(scoring_version, str) or not scoring_version:
        raise ValueError("scoring_version must be a non-empty string")
    if not isinstance(max_steps, int) or isinstance(max_steps, bool) or max_steps <= 0:
        raise ValueError("max_steps must be a positive integer")

    config_payload = resolved_config.to_dict()
    return RunManifest(
        run_id=resolved_config.run_id,
        benchmark_id=resolved_config.benchmark_id,
        benchmark_version=benchmark_version,
        scenario_id=scenario_id,
        scenario_version=scenario_version,
        scoring_version=scoring_version,
        effective_seed=resolved_config.effective_seed,
        seed_source=resolved_config.seed_source,
        actor_ids=resolved_config.actor_ids,
        max_steps=max_steps,
        config_json=_to_canonical_json(config_payload),
    )


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = payload.get(field_name)
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be an integer")
    return value


def _read_required_positive_int(payload: Mapping[str, Any], field_name: str) -> int:
    value = _read_required_int(payload, field_name)
    if value <= 0:
        raise ValueError(f"{field_name} must be a positive integer")
    return value


def _read_required_actor_ids(payload: Mapping[str, Any], field_name: str) -> tuple[str, ...]:
    value = payload.get(field_name)
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of non-empty strings")
    actor_ids: list[str] = []
    for actor_id in value:
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError(f"{field_name} must contain non-empty strings")
        actor_ids.append(actor_id)
    if len(actor_ids) == 0:
        raise ValueError(f"{field_name} must contain at least one actor")
    return tuple(actor_ids)


def _to_canonical_json(value: Mapping[str, Any]) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
