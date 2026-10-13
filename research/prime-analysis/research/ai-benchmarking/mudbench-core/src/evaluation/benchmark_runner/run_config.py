"""Benchmark run configuration schema and deterministic seed policy."""

from __future__ import annotations

import hashlib
import json
from dataclasses import dataclass, field
from typing import Any, Mapping, Sequence

_MIN_SEED = 0
_MAX_SEED = 2_147_483_647
_MIN_MAX_STEPS_OVERRIDE = 1


@dataclass(frozen=True, slots=True)
class BenchmarkRunConfig:
    """Immutable benchmark run configuration with deterministic effective-seed resolution."""

    run_id: str
    benchmark_id: str
    scenario: Mapping[str, Any] | str
    actor_ids: Sequence[str]
    run_seed: int | None = None
    seed_override: int | None = None
    max_steps_override: int | None = None
    _scenario_canonical_json: str = field(init=False, repr=False, compare=False)
    _scenario_seed: int | None = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        if not isinstance(self.run_id, str) or not self.run_id:
            raise ValueError("run_id must be a non-empty string")
        if not isinstance(self.benchmark_id, str) or not self.benchmark_id:
            raise ValueError("benchmark_id must be a non-empty string")

        canonical_actor_ids = _normalize_actor_ids(self.actor_ids)
        object.__setattr__(self, "actor_ids", canonical_actor_ids)

        scenario_payload = _normalize_scenario_payload(self.scenario)
        object.__setattr__(
            self,
            "_scenario_canonical_json",
            json.dumps(scenario_payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True),
        )
        object.__setattr__(self, "_scenario_seed", _read_optional_seed(scenario_payload, "seed"))

        _validate_optional_seed(self.run_seed, "run_seed")
        _validate_optional_seed(self.seed_override, "seed_override")

        _validate_optional_positive_int(self.max_steps_override, "max_steps_override")

    @classmethod
    def from_mapping(cls, payload: Mapping[str, Any]) -> "BenchmarkRunConfig":
        if not isinstance(payload, Mapping):
            raise ValueError("payload must be a mapping")
        return cls(
            run_id=_read_required_string(payload, "run_id"),
            benchmark_id=_read_required_string(payload, "benchmark_id"),
            scenario=payload.get("scenario"),
            actor_ids=_read_required_actor_ids(payload, "actor_ids"),
            run_seed=_read_optional_int(payload, "run_seed"),
            seed_override=_read_optional_int(payload, "seed_override"),
            max_steps_override=_read_optional_int(payload, "max_steps_override"),
        )

    @property
    def scenario_seed(self) -> int | None:
        return self._scenario_seed

    @property
    def seed_source(self) -> str:
        source, _ = self._resolved_seed_precedence()
        return source

    @property
    def effective_seed(self) -> int:
        _, seed = self._resolved_seed_precedence()
        return seed

    def _resolved_seed_precedence(self) -> tuple[str, int]:
        return _resolve_seed_precedence(
            seed_override=self.seed_override,
            run_seed=self.run_seed,
            scenario_seed=self._scenario_seed,
            derived_seed=_derive_seed(
                {
                    "benchmark_id": self.benchmark_id,
                    "run_id": self.run_id,
                    "actor_ids": self.actor_ids,
                    "scenario": self._scenario_canonical_json,
                }
            ),
        )

    def to_dict(self) -> dict[str, Any]:
        scenario_payload = json.loads(self._scenario_canonical_json)
        return {
            "run_id": self.run_id,
            "benchmark_id": self.benchmark_id,
            "scenario": scenario_payload,
            "actor_ids": list(self.actor_ids),
            "run_seed": self.run_seed,
            "seed_override": self.seed_override,
            "max_steps_override": self.max_steps_override,
            "effective_seed": self.effective_seed,
            "seed_source": self.seed_source,
        }


def _normalize_actor_ids(actor_ids: Sequence[str]) -> tuple[str, ...]:
    if isinstance(actor_ids, (str, bytes)) or not isinstance(actor_ids, Sequence):
        raise ValueError("actor_ids must be a sequence of strings")
    if len(actor_ids) == 0:
        raise ValueError("actor_ids must contain at least one actor")

    seen: set[str] = set()
    normalized: list[str] = []
    for actor_id in actor_ids:
        if not isinstance(actor_id, str) or not actor_id:
            raise ValueError("actor_ids must contain non-empty strings")
        if actor_id in seen:
            raise ValueError(f"duplicate actor_id in actor_ids: {actor_id}")
        seen.add(actor_id)
        normalized.append(actor_id)
    return tuple(sorted(normalized))


def _normalize_scenario_payload(scenario: Mapping[str, Any] | str) -> Mapping[str, Any]:
    if isinstance(scenario, str):
        try:
            parsed = json.loads(scenario)
        except json.JSONDecodeError as exc:
            raise ValueError("scenario JSON string is invalid") from exc
        if not isinstance(parsed, Mapping):
            raise ValueError("scenario JSON must decode to an object")
        scenario_payload: Mapping[str, Any] = parsed
    elif isinstance(scenario, Mapping):
        scenario_payload = scenario
    else:
        raise ValueError("scenario must be a mapping or JSON string")

    if len(scenario_payload) == 0:
        raise ValueError("scenario must not be empty")

    normalized = _normalize_json_value(scenario_payload, field_name="scenario")
    if not isinstance(normalized, Mapping):
        raise ValueError("scenario must normalize to a mapping")
    return normalized


def _normalize_json_value(value: Any, *, field_name: str) -> Any:
    if isinstance(value, Mapping):
        normalized_items: list[tuple[str, Any]] = []
        for raw_key, raw_value in value.items():
            if not isinstance(raw_key, str) or not raw_key:
                raise ValueError(f"{field_name} contains a non-string or empty key")
            normalized_items.append(
                (raw_key, _normalize_json_value(raw_value, field_name=f"{field_name}.{raw_key}"))
            )
        return {key: normalized_value for key, normalized_value in sorted(normalized_items, key=lambda kv: kv[0])}

    if isinstance(value, list):
        return [_normalize_json_value(item, field_name=field_name) for item in value]

    if isinstance(value, tuple):
        return [_normalize_json_value(item, field_name=field_name) for item in value]

    if value is None or isinstance(value, (str, int, float, bool)):
        return value

    raise ValueError(f"{field_name} contains unsupported value type: {type(value).__name__}")


def _derive_seed(payload: Mapping[str, Any]) -> int:
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=True)
    digest = hashlib.sha256(canonical.encode("ascii")).digest()
    return int.from_bytes(digest[:8], byteorder="big", signed=False) % (_MAX_SEED + 1)


def _resolve_seed_precedence(
    *,
    seed_override: int | None,
    run_seed: int | None,
    scenario_seed: int | None,
    derived_seed: int,
) -> tuple[str, int]:
    if seed_override is not None:
        return "seed_override", seed_override
    if run_seed is not None:
        return "run_seed", run_seed
    if scenario_seed is not None:
        return "scenario_seed", scenario_seed
    return "derived", derived_seed


def _read_required_string(payload: Mapping[str, Any], field_name: str) -> str:
    value = payload.get(field_name)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{field_name} must be a non-empty string")
    return value


def _read_required_actor_ids(payload: Mapping[str, Any], field_name: str) -> Sequence[str]:
    value = payload.get(field_name)
    if isinstance(value, (str, bytes)) or not isinstance(value, Sequence):
        raise ValueError(f"{field_name} must be a sequence of strings")
    return value


def _read_optional_int(payload: Mapping[str, Any], field_name: str) -> int | None:
    value = payload.get(field_name)
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be None or an integer")
    return value


def _read_optional_seed(payload: Mapping[str, Any], field_name: str) -> int | None:
    value = payload.get(field_name)
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be an integer when provided")
    if value < _MIN_SEED or value > _MAX_SEED:
        raise ValueError(f"{field_name} must be within [{_MIN_SEED}, {_MAX_SEED}]")
    return value


def _validate_optional_seed(value: int | None, field_name: str) -> None:
    if value is None:
        return
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be None or an integer")
    if value < _MIN_SEED or value > _MAX_SEED:
        raise ValueError(f"{field_name} must be within [{_MIN_SEED}, {_MAX_SEED}]")


def _validate_optional_positive_int(value: int | None, field_name: str) -> None:
    if value is None:
        return
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{field_name} must be None or a positive integer")
    if value < _MIN_MAX_STEPS_OVERRIDE:
        raise ValueError(f"{field_name} must be None or a positive integer")
