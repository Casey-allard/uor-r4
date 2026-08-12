"""Deterministic scenario loading and initialization adapters."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from scenarios.scenario_definition import ScenarioDefinition


@dataclass(frozen=True, slots=True)
class ScenarioLoadResult:
    """Explicit accept/reject result for scenario loading."""

    accepted: bool
    scenario: ScenarioDefinition | None = None
    reason: str | None = None


@dataclass(frozen=True, slots=True)
class ScenarioInitialization:
    """Deterministic benchmark-runner initialization payload."""

    scenario_id: str
    run_seed: int
    start_room_id: str
    max_steps: int
    version: str
    scenario_vars: tuple[tuple[str, Any], ...]
    objectives: tuple[tuple[str, tuple[tuple[str, Any], ...]], ...]

    def to_dict(self) -> dict[str, Any]:
        return {
            "scenario_id": self.scenario_id,
            "run_seed": self.run_seed,
            "start_room_id": self.start_room_id,
            "max_steps": self.max_steps,
            "version": self.version,
            "scenario_vars": {key: value for key, value in self.scenario_vars},
            "objectives": [
                {
                    "objective_id": objective_id,
                    "fields": {field_key: field_value for field_key, field_value in fields},
                }
                for objective_id, fields in self.objectives
            ],
        }

    def to_canonical_json(self) -> str:
        return json.dumps(self.to_dict(), sort_keys=True, separators=(",", ":"), ensure_ascii=True)


def load_scenario_definition(payload: ScenarioDefinition | Mapping[str, Any] | str) -> ScenarioLoadResult:
    """Load a scenario definition with explicit accept/reject semantics."""
    if isinstance(payload, ScenarioDefinition):
        return ScenarioLoadResult(accepted=True, scenario=payload)

    raw_payload: object = payload
    if isinstance(payload, str):
        try:
            raw_payload = json.loads(payload)
        except json.JSONDecodeError:
            return ScenarioLoadResult(accepted=False, reason="invalid_json")

    if not isinstance(raw_payload, Mapping):
        return ScenarioLoadResult(accepted=False, reason="payload_not_mapping")

    try:
        scenario = ScenarioDefinition.from_dict(raw_payload)
    except ValueError as exc:
        return ScenarioLoadResult(accepted=False, reason=str(exc))
    return ScenarioLoadResult(accepted=True, scenario=scenario)


def load_scenario_batch(
    payloads: Sequence[ScenarioDefinition | Mapping[str, Any] | str],
) -> tuple[ScenarioLoadResult, ...]:
    """Load a sequence of scenario payloads in deterministic input order."""
    if isinstance(payloads, (str, bytes)) or not isinstance(payloads, Sequence):
        raise ValueError("payloads must be a sequence")
    return tuple(load_scenario_definition(payload) for payload in payloads)


def build_scenario_initialization(
    scenario: ScenarioDefinition,
    *,
    seed_override: int | None = None,
) -> ScenarioInitialization:
    """Build deterministic initialization artifact for benchmark runner startup."""
    if not isinstance(scenario, ScenarioDefinition):
        raise ValueError("scenario must be a ScenarioDefinition")
    if seed_override is not None and (not isinstance(seed_override, int) or isinstance(seed_override, bool)):
        raise ValueError("seed_override must be None or an integer")

    run_seed = scenario.seed if seed_override is None else seed_override
    objectives = tuple(
        (
            objective.objective_id,
            (
                ("objective_type", objective.objective_type),
                ("required_count", objective.required_count),
                ("target_id", objective.target_id),
                ("metadata", {key: value for key, value in objective.metadata}),
            ),
        )
        for objective in scenario.objectives
    )
    return ScenarioInitialization(
        scenario_id=scenario.scenario_id,
        run_seed=run_seed,
        start_room_id=scenario.start_room_id,
        max_steps=scenario.max_steps,
        version=scenario.version,
        scenario_vars=scenario.scenario_vars,
        objectives=objectives,
    )

