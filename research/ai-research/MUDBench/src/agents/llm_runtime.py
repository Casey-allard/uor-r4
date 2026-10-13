"""Deterministic benchmark-mode LLM gameplay helpers."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any, Mapping, Sequence

from agents.gateway.action_parser import ModelActionOutputParseResult, parse_model_action_output
from agents.gateway.observation_builder import build_model_facing_observation_payload
from agents.protocol.action import ActionSubmission
from agents.protocol.observation import Observation
from agents.protocol.serialization import canonical_json_dumps
from router_core import (
    DEFAULT_CANONICAL_ANGULAR_VARIANT,
    SUPPORTED_CANONICAL_ANGULAR_VARIANTS,
    SUPPORTED_LEGACY_ROUTER_VARIANTS,
    build_canonical_angular_prompt_plan as build_router_core_canonical_angular_prompt_plan,
    build_legacy_router_backed_prompt_plan as build_router_core_legacy_prompt_plan,
    normalize_4d_coordinate,
)

_BENCHMARK_MODE = "benchmark_single_turn"
_PERSISTENT_SESSION_MODE = "persistent_session"
_PROMPT_ENGINE_BASELINE = "baseline"
_PROMPT_ENGINE_GEOMETRIC_ROUTED = "geometric-routed"
_PROMPT_ENGINE_ANGULAR_CANONICAL = "angular-canonical"
_PROMPT_ENGINE_LEGACY_ROUTER_BACKED = "legacy-router-backed"
_DEFAULT_CANONICAL_ROUTER_VARIANT = DEFAULT_CANONICAL_ANGULAR_VARIANT
_DEFAULT_LEGACY_ROUTER_VARIANT = "legacy-phase4d_hopf_base"
_PROMPT_ENGINE_ALIASES = {
    "router-backed": _PROMPT_ENGINE_LEGACY_ROUTER_BACKED,
}
_LEGACY_ROUTER_VARIANT_ALIASES = {
    "phase4d_hopf_base": "legacy-phase4d_hopf_base",
    "phase4d_hopf_transport": "legacy-phase4d_hopf_transport",
    "phase4d_hopf_product_phase": "legacy-phase4d_hopf_product_phase",
}
_ROUTED_LAYER_ORDER = (
    "immediate_action",
    "local_objective",
    "temporal_world",
    "multi_agent",
    "persistence",
)
_ROUTED_SELECTION_POLICY_BY_ENGINE = {
    _PROMPT_ENGINE_GEOMETRIC_ROUTED: {
        "full_layer_limit": 1,
        "summary_layer_limit": 1,
        "observation_anchor_mode": "reduced_summary",
    },
    _PROMPT_ENGINE_ANGULAR_CANONICAL: {
        "full_layer_limit": 1,
        "summary_layer_limit": 1,
        "observation_anchor_mode": "reduced_summary",
    },
    _PROMPT_ENGINE_LEGACY_ROUTER_BACKED: {
        "full_layer_limit": 1,
        "summary_layer_limit": 1,
        "observation_anchor_mode": "reduced_summary",
    },
}
_SUPPORTED_PROMPT_ENGINES = frozenset(
    {
        _PROMPT_ENGINE_BASELINE,
        _PROMPT_ENGINE_GEOMETRIC_ROUTED,
        _PROMPT_ENGINE_ANGULAR_CANONICAL,
        _PROMPT_ENGINE_LEGACY_ROUTER_BACKED,
    }
)
_EXPECTED_ACTION_OUTPUT_CONTRACT = {
    "type": "json_object",
    "required_fields": ["action"],
    "additional_properties": False,
}
_ROUTED_HAZARD_KEYWORDS = (
    "guard",
    "marauder",
    "raider",
    "storm",
    "hazard",
    "danger",
    "spill",
    "drain",
    "threat",
    "hostile",
)
_ROUTED_GATE_KEYWORDS = (
    "seal",
    "lock",
    "valve",
    "bypass",
    "door",
    "archive",
    "vault",
    "unlock",
    "lift",
    "requires",
    "powered",
    "threshold",
)
_ROUTED_PROGRESS_KEYWORDS = (
    "archive",
    "vault",
    "prism",
    "ledger",
    "artifact",
    "relic",
    "seal",
    "bypass",
    "final",
    "inner",
)
_ROUTED_MISLEADING_KEYWORDS = (
    "shortcut",
    "short route",
    "annex",
    "signal",
    "flare",
    "emergency",
    "side",
    "drain",
    "waste",
    "decoy",
)
_ROUTED_RESOURCE_KEYWORDS = (
    "cell",
    "power",
    "coolant",
    "charge",
    "battery",
    "fuel",
    "key",
    "handle",
)
_ROUTED_CONSUMABLE_RESOURCE_KEYWORDS = (
    "cell",
    "power",
    "coolant",
    "charge",
    "battery",
    "fuel",
)


@dataclass(frozen=True, slots=True)
class BenchmarkLLMTurnResult:
    """Deterministic benchmark-mode LLM turn outcome."""

    model_facing_observation_payload: dict[str, Any]
    prompt: str
    prompt_engine: str
    router_variant: str | None
    routing_plan: dict[str, Any] | None
    initial_parse_result: ModelActionOutputParseResult
    action_submission: ActionSubmission
    repair_prompt: str | None = None
    repaired_parse_result: ModelActionOutputParseResult | None = None
    used_fail_closed_fallback: bool = False
    fail_closed_reason: str | None = None
    runtime_telemetry: dict[str, Any] | None = None


def build_expected_action_output_contract() -> dict[str, Any]:
    """Return the canonical expected action output contract."""
    return {
        "type": _EXPECTED_ACTION_OUTPUT_CONTRACT["type"],
        "required_fields": list(_EXPECTED_ACTION_OUTPUT_CONTRACT["required_fields"]),
        "additional_properties": _EXPECTED_ACTION_OUTPUT_CONTRACT["additional_properties"],
    }


def build_canonical_model_facing_observation_payload(
    observation: Observation,
    *,
    actor_id: str | None = None,
) -> dict[str, Any]:
    """Build the benchmark-mode model-facing observation payload."""
    return build_model_facing_observation_payload(
        observation,
        mode=_BENCHMARK_MODE,
        actor_id=actor_id,
    )


def build_persistent_session_model_facing_observation_payload(
    observation: Observation,
    *,
    actor_id: str | None = None,
) -> dict[str, Any]:
    """Build the persistent-session model-facing observation payload."""
    return build_model_facing_observation_payload(
        observation,
        mode=_PERSISTENT_SESSION_MODE,
        actor_id=actor_id,
    )


def build_benchmark_prompt(
    observation: Observation,
    *,
    actor_id: str | None = None,
    prompt_engine: str = _PROMPT_ENGINE_BASELINE,
    router_variant: str | None = None,
) -> str:
    """Build the canonical benchmark-mode prompt for one observation step."""
    payload = build_canonical_model_facing_observation_payload(
        observation,
        actor_id=actor_id,
    )
    return build_benchmark_prompt_from_payload(
        payload,
        prompt_engine=prompt_engine,
        router_variant=router_variant,
    )


def build_persistent_session_prompt(
    observation: Observation,
    *,
    actor_id: str | None = None,
) -> str:
    """Build the canonical persistent-session prompt for one observation step."""
    payload = build_persistent_session_model_facing_observation_payload(
        observation,
        actor_id=actor_id,
    )
    return build_persistent_session_prompt_from_payload(payload)


def build_benchmark_prompt_from_payload(
    payload: Mapping[str, Any],
    *,
    prompt_engine: str = _PROMPT_ENGINE_BASELINE,
    router_variant: str | None = None,
) -> str:
    """Build the benchmark-mode prompt from a model-facing payload."""
    return _build_prompt_from_payload(
        payload,
        expected_mode=_BENCHMARK_MODE,
        prompt_engine=prompt_engine,
        router_variant=router_variant,
    )


def build_persistent_session_prompt_from_payload(
    payload: Mapping[str, Any],
) -> str:
    """Build the canonical persistent-session prompt from a model-facing payload."""
    return _build_prompt_from_payload(
        payload,
        expected_mode=_PERSISTENT_SESSION_MODE,
        prompt_engine=_PROMPT_ENGINE_BASELINE,
        router_variant=None,
    )


def build_geometric_routed_prompt_plan(
    payload: Mapping[str, Any],
    *,
    expected_mode: str = _BENCHMARK_MODE,
) -> dict[str, Any]:
    """Build the deterministic routed prompt plan over the existing loop layers."""
    if not isinstance(payload, Mapping):
        raise ValueError("payload must be a mapping")
    if payload.get("mode") != expected_mode:
        raise ValueError(f"payload mode must be {expected_mode}")

    observation_payload = payload.get("observation")
    if not isinstance(observation_payload, Mapping):
        raise ValueError("payload must include observation mapping")
    allowed_actions = payload.get("allowed_actions")
    if isinstance(allowed_actions, (str, bytes)) or not isinstance(allowed_actions, Sequence):
        raise ValueError("payload must include allowed_actions sequence")
    normalized_allowed_actions = [action for action in allowed_actions if isinstance(action, str)]
    if len(normalized_allowed_actions) != len(allowed_actions):
        raise ValueError("allowed_actions must contain only strings")
    allowed_targets = payload.get("allowed_targets", {})
    if not isinstance(allowed_targets, Mapping):
        raise ValueError("payload allowed_targets must be a mapping when provided")

    loop_layers = _build_routed_loop_layers(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )
    routing_context = _build_routed_routing_context(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )

    layer_order = _ROUTED_LAYER_ORDER
    scores = _build_loop_layer_scores(loop_layers, routing_context=routing_context)
    ordered_layers = sorted(
        layer_order,
        key=lambda layer_name: (-scores[layer_name], layer_order.index(layer_name)),
    )
    dominant_loop = ordered_layers[0]
    secondary_loops = list(ordered_layers[1:])
    compressed_sections = [
        layer_name
        for layer_name in ordered_layers
        if layer_name != dominant_loop and scores[layer_name] <= 25
    ]
    return {
        "engine": _PROMPT_ENGINE_GEOMETRIC_ROUTED,
        "dominant_loop": dominant_loop,
        "secondary_loops": secondary_loops,
        "prompt_section_order": list(ordered_layers),
        "compressed_sections": compressed_sections,
        "reasoning_pressure_tag": _build_reasoning_pressure_tag(dominant_loop),
        "layer_scores": dict(scores),
        "routing_context": dict(routing_context),
    }


def build_angular_canonical_prompt_plan(
    payload: Mapping[str, Any],
    *,
    expected_mode: str = _BENCHMARK_MODE,
    router_variant: str | None = None,
) -> dict[str, Any]:
    """Build the canonical single-shell angular prompt plan."""
    if not isinstance(payload, Mapping):
        raise ValueError("payload must be a mapping")
    if payload.get("mode") != expected_mode:
        raise ValueError(f"payload mode must be {expected_mode}")

    observation_payload = payload.get("observation")
    if not isinstance(observation_payload, Mapping):
        raise ValueError("payload must include observation mapping")
    allowed_actions = payload.get("allowed_actions")
    if isinstance(allowed_actions, (str, bytes)) or not isinstance(allowed_actions, Sequence):
        raise ValueError("payload must include allowed_actions sequence")
    normalized_allowed_actions = [action for action in allowed_actions if isinstance(action, str)]
    if len(normalized_allowed_actions) != len(allowed_actions):
        raise ValueError("allowed_actions must contain only strings")
    allowed_targets = payload.get("allowed_targets", {})
    if not isinstance(allowed_targets, Mapping):
        raise ValueError("payload allowed_targets must be a mapping when provided")

    loop_layers = _build_routed_loop_layers(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )
    routing_context = _build_routed_routing_context(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )
    layer_scores = _build_loop_layer_scores(loop_layers, routing_context=routing_context)
    routing_coordinate_adapter = _build_canonical_angular_routing_coordinate(
        layer_scores=layer_scores,
        routing_context=routing_context,
    )
    plan = build_router_core_canonical_angular_prompt_plan(
        routing_coordinate=routing_coordinate_adapter["raw_coordinate"],
        bundle_scores=routing_coordinate_adapter["bundle_scores"],
        persistence_score=routing_coordinate_adapter["component_scores"]["persistence"],
        multi_agent_score=routing_coordinate_adapter["component_scores"]["multi_agent"],
        router_variant=_normalize_router_variant(
            _PROMPT_ENGINE_ANGULAR_CANONICAL,
            router_variant,
        ),
    )
    plan["coordinate_adapter_contract"] = routing_coordinate_adapter["adapter_schema"]
    plan["layer_scores"] = dict(layer_scores)
    plan["routing_context"] = dict(routing_context)
    plan["routing_coordinate_adapter"] = routing_coordinate_adapter
    return plan


def build_legacy_router_backed_prompt_plan(
    payload: Mapping[str, Any],
    *,
    expected_mode: str = _BENCHMARK_MODE,
    router_variant: str | None = None,
) -> dict[str, Any]:
    """Build the explicit legacy proxy prompt plan from the vendored router core."""
    if not isinstance(payload, Mapping):
        raise ValueError("payload must be a mapping")
    if payload.get("mode") != expected_mode:
        raise ValueError(f"payload mode must be {expected_mode}")

    observation_payload = payload.get("observation")
    if not isinstance(observation_payload, Mapping):
        raise ValueError("payload must include observation mapping")
    allowed_actions = payload.get("allowed_actions")
    if isinstance(allowed_actions, (str, bytes)) or not isinstance(allowed_actions, Sequence):
        raise ValueError("payload must include allowed_actions sequence")
    normalized_allowed_actions = [action for action in allowed_actions if isinstance(action, str)]
    if len(normalized_allowed_actions) != len(allowed_actions):
        raise ValueError("allowed_actions must contain only strings")
    allowed_targets = payload.get("allowed_targets", {})
    if not isinstance(allowed_targets, Mapping):
        raise ValueError("payload allowed_targets must be a mapping when provided")

    loop_layers = _build_routed_loop_layers(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )
    routing_context = _build_routed_routing_context(
        observation_payload,
        normalized_allowed_actions,
        allowed_targets,
    )
    layer_scores = _build_loop_layer_scores(loop_layers, routing_context=routing_context)
    plan = build_router_core_legacy_prompt_plan(
        layer_scores,
        router_variant=_normalize_router_variant(
            _PROMPT_ENGINE_LEGACY_ROUTER_BACKED,
            router_variant,
        ),
        layer_order=_ROUTED_LAYER_ORDER,
    )
    plan["layer_scores"] = dict(layer_scores)
    plan["routing_context"] = dict(routing_context)
    return plan


build_router_backed_prompt_plan = build_legacy_router_backed_prompt_plan


def _build_prompt_from_payload(
    payload: Mapping[str, Any],
    *,
    expected_mode: str,
    prompt_engine: str,
    router_variant: str | None,
) -> str:
    normalized_prompt_engine = _normalize_prompt_engine(prompt_engine)
    if not isinstance(payload, Mapping):
        raise ValueError("payload must be a mapping")
    if payload.get("mode") != expected_mode:
        raise ValueError(f"payload mode must be {expected_mode}")

    observation_payload = payload.get("observation")
    if not isinstance(observation_payload, Mapping):
        raise ValueError("payload must include observation mapping")
    allowed_actions = payload.get("allowed_actions")
    if isinstance(allowed_actions, (str, bytes)) or not isinstance(allowed_actions, Sequence):
        raise ValueError("payload must include allowed_actions sequence")
    normalized_allowed_actions: list[str] = []
    for action in allowed_actions:
        if not isinstance(action, str):
            raise ValueError("allowed_actions must contain only strings")
        normalized_allowed_actions.append(action)
    allowed_targets = payload.get("allowed_targets", {})
    if not isinstance(allowed_targets, Mapping):
        raise ValueError("payload allowed_targets must be a mapping when provided")
    session_frame = payload.get("session_frame")
    if not isinstance(session_frame, Mapping):
        raise ValueError("payload must include session_frame mapping")
    response_format = payload.get("response_format")
    if not isinstance(response_format, Mapping):
        raise ValueError("payload must include response_format mapping")

    if normalized_prompt_engine == _PROMPT_ENGINE_BASELINE:
        return _build_baseline_prompt_from_payload(
            expected_mode=expected_mode,
            observation_payload=observation_payload,
            session_frame=session_frame,
            response_format=response_format,
            normalized_allowed_actions=normalized_allowed_actions,
            allowed_targets=allowed_targets,
        )
    if normalized_prompt_engine == _PROMPT_ENGINE_GEOMETRIC_ROUTED:
        return _build_geometric_routed_prompt_from_payload(
            payload,
            expected_mode=expected_mode,
            observation_payload=observation_payload,
            session_frame=session_frame,
            response_format=response_format,
            normalized_allowed_actions=normalized_allowed_actions,
            allowed_targets=allowed_targets,
        )
    if normalized_prompt_engine == _PROMPT_ENGINE_ANGULAR_CANONICAL:
        return _build_angular_canonical_prompt_from_payload(
            payload,
            expected_mode=expected_mode,
            router_variant=router_variant,
            observation_payload=observation_payload,
            session_frame=session_frame,
            response_format=response_format,
            normalized_allowed_actions=normalized_allowed_actions,
            allowed_targets=allowed_targets,
        )
    return _build_legacy_router_backed_prompt_from_payload(
        payload,
        expected_mode=expected_mode,
        router_variant=router_variant,
        observation_payload=observation_payload,
        session_frame=session_frame,
        response_format=response_format,
        normalized_allowed_actions=normalized_allowed_actions,
        allowed_targets=allowed_targets,
    )


def _build_baseline_prompt_from_payload(
    *,
    expected_mode: str,
    observation_payload: Mapping[str, Any],
    session_frame: Mapping[str, Any],
    response_format: Mapping[str, Any],
    normalized_allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> str:
    return "\n".join(
        (
            _build_mode_instructions(expected_mode),
            "",
            "Invariant runtime guardrails:",
            "- Return exactly one JSON object and nothing else.",
            '- The JSON object may contain only one field: "action".',
            "- The action value must be a string copied exactly from allowed_actions.",
            "- Do not output prose, markdown, code fences, commentary, or extra keys.",
            "- If an action references a direction, item, or target, use only the allowed_targets listed below.",
            "- If your output is invalid, MUDBench will allow at most one repair attempt and then fail closed.",
            "",
            "Session frame:",
            canonical_json_dumps(dict(session_frame)),
            "",
            "Expected output contract:",
            canonical_json_dumps(dict(response_format)),
            "",
            "Allowed actions:",
            canonical_json_dumps({"allowed_actions": list(normalized_allowed_actions)}),
            "",
            "Allowed targets:",
            canonical_json_dumps({"allowed_targets": dict(allowed_targets)}),
            "",
            "Observation:",
            canonical_json_dumps(observation_payload),
            "",
            'Return JSON now in the exact form {"action":"..."}',
        )
    )


def _build_geometric_routed_prompt_from_payload(
    payload: Mapping[str, Any],
    *,
    expected_mode: str,
    observation_payload: Mapping[str, Any],
    session_frame: Mapping[str, Any],
    response_format: Mapping[str, Any],
    normalized_allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> str:
    routing_plan = build_geometric_routed_prompt_plan(payload, expected_mode=expected_mode)
    loop_layers = _build_routed_loop_layers(
        observation_payload,
        list(normalized_allowed_actions),
        allowed_targets,
    )
    selection_plan = _build_routed_context_selection_plan(
        prompt_engine=_PROMPT_ENGINE_GEOMETRIC_ROUTED,
        routing_plan=routing_plan,
    )
    section_lines = _build_selected_layer_lines(
        loop_layers=loop_layers,
        selection_plan=selection_plan,
        full_label_prefix="Loop layer",
        summary_label_prefix="Loop layer",
    )
    observation_summary = _build_reduced_observation_summary(
        observation_payload=observation_payload,
        allowed_actions=normalized_allowed_actions,
        allowed_targets=allowed_targets,
    )

    return "\n".join(
        (
            _build_mode_instructions(expected_mode),
            "",
            "Invariant runtime guardrails:",
            "- Return exactly one JSON object and nothing else.",
            '- The JSON object may contain only one field: "action".',
            "- The action value must be a string copied exactly from allowed_actions.",
            "- Do not output prose, markdown, code fences, commentary, or extra keys.",
            "- If an action references a direction, item, or target, use only the allowed_targets listed below.",
            "- If your output is invalid, MUDBench will allow at most one repair attempt and then fail closed.",
            "",
            "Routing plan:",
            canonical_json_dumps(routing_plan),
            "",
            "Session frame:",
            canonical_json_dumps(dict(session_frame)),
            "",
            "Expected output contract:",
            canonical_json_dumps(dict(response_format)),
            "",
            "Allowed actions:",
            canonical_json_dumps({"allowed_actions": list(normalized_allowed_actions)}),
            "",
            "Allowed targets:",
            canonical_json_dumps({"allowed_targets": dict(allowed_targets)}),
            "",
            "Routed observation layers:",
            *section_lines,
            "Observation summary:",
            canonical_json_dumps(observation_summary),
            "",
            'Return JSON now in the exact form {"action":"..."}',
        )
    )


def _build_angular_canonical_prompt_from_payload(
    payload: Mapping[str, Any],
    *,
    expected_mode: str,
    router_variant: str | None,
    observation_payload: Mapping[str, Any],
    session_frame: Mapping[str, Any],
    response_format: Mapping[str, Any],
    normalized_allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> str:
    routing_plan = build_angular_canonical_prompt_plan(
        payload,
        expected_mode=expected_mode,
        router_variant=router_variant,
    )
    loop_layers = _build_routed_loop_layers(
        observation_payload,
        list(normalized_allowed_actions),
        allowed_targets,
    )
    selection_plan = _build_routed_context_selection_plan(
        prompt_engine=_PROMPT_ENGINE_ANGULAR_CANONICAL,
        routing_plan=routing_plan,
    )
    section_lines = _build_selected_layer_lines(
        loop_layers=loop_layers,
        selection_plan=selection_plan,
        full_label_prefix="Angular layer",
        summary_label_prefix="Angular layer",
    )
    observation_summary = _build_reduced_observation_summary(
        observation_payload=observation_payload,
        allowed_actions=normalized_allowed_actions,
        allowed_targets=allowed_targets,
    )

    return "\n".join(
        (
            _build_mode_instructions(expected_mode),
            "",
            "Invariant runtime guardrails:",
            "- Return exactly one JSON object and nothing else.",
            '- The JSON object may contain only one field: "action".',
            "- The action value must be a string copied exactly from allowed_actions.",
            "- Do not output prose, markdown, code fences, commentary, or extra keys.",
            "- If an action references a direction, item, or target, use only the allowed_targets listed below.",
            "- If your output is invalid, MUDBench will allow at most one repair attempt and then fail closed.",
            "",
            "Angular canonical plan:",
            canonical_json_dumps(routing_plan),
            "",
            "Session frame:",
            canonical_json_dumps(dict(session_frame)),
            "",
            "Expected output contract:",
            canonical_json_dumps(dict(response_format)),
            "",
            "Allowed actions:",
            canonical_json_dumps({"allowed_actions": list(normalized_allowed_actions)}),
            "",
            "Allowed targets:",
            canonical_json_dumps({"allowed_targets": dict(allowed_targets)}),
            "",
            "Angular routed observation layers:",
            *section_lines,
            "Observation summary:",
            canonical_json_dumps(observation_summary),
            "",
            'Return JSON now in the exact form {"action":"..."}',
        )
    )


def _build_legacy_router_backed_prompt_from_payload(
    payload: Mapping[str, Any],
    *,
    expected_mode: str,
    router_variant: str | None,
    observation_payload: Mapping[str, Any],
    session_frame: Mapping[str, Any],
    response_format: Mapping[str, Any],
    normalized_allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> str:
    routing_plan = build_legacy_router_backed_prompt_plan(
        payload,
        expected_mode=expected_mode,
        router_variant=router_variant,
    )
    loop_layers = _build_routed_loop_layers(
        observation_payload,
        list(normalized_allowed_actions),
        allowed_targets,
    )
    selection_plan = _build_routed_context_selection_plan(
        prompt_engine=_PROMPT_ENGINE_LEGACY_ROUTER_BACKED,
        routing_plan=routing_plan,
    )
    section_lines = _build_selected_layer_lines(
        loop_layers=loop_layers,
        selection_plan=selection_plan,
        full_label_prefix="Legacy router layer",
        summary_label_prefix="Legacy router layer",
    )
    observation_summary = _build_reduced_observation_summary(
        observation_payload=observation_payload,
        allowed_actions=normalized_allowed_actions,
        allowed_targets=allowed_targets,
    )

    return "\n".join(
        (
            _build_mode_instructions(expected_mode),
            "",
            "Invariant runtime guardrails:",
            "- Return exactly one JSON object and nothing else.",
            '- The JSON object may contain only one field: "action".',
            "- The action value must be a string copied exactly from allowed_actions.",
            "- Do not output prose, markdown, code fences, commentary, or extra keys.",
            "- If an action references a direction, item, or target, use only the allowed_targets listed below.",
            "- If your output is invalid, MUDBench will allow at most one repair attempt and then fail closed.",
            "",
            "Legacy router-backed proxy plan:",
            canonical_json_dumps(routing_plan),
            "",
            "Session frame:",
            canonical_json_dumps(dict(session_frame)),
            "",
            "Expected output contract:",
            canonical_json_dumps(dict(response_format)),
            "",
            "Allowed actions:",
            canonical_json_dumps({"allowed_actions": list(normalized_allowed_actions)}),
            "",
            "Allowed targets:",
            canonical_json_dumps({"allowed_targets": dict(allowed_targets)}),
            "",
            "Legacy router-backed observation layers:",
            *section_lines,
            "Observation summary:",
            canonical_json_dumps(observation_summary),
            "",
            'Return JSON now in the exact form {"action":"..."}',
        )
    )


def _build_mode_instructions(mode: str) -> str:
    if mode == _BENCHMARK_MODE:
        return "You are controlling one MUDBench benchmark agent for exactly one isolated benchmark step."
    if mode == _PERSISTENT_SESSION_MODE:
        return (
            "You are controlling one MUDBench agent inside a persistent local-process session."
            " Treat this turn as one bounded session step that may continue in later turns."
        )
    raise ValueError(f"unsupported prompt mode: {mode}")


def _require_loop_layer_mapping(loop_layers: Mapping[str, Any], layer_name: str) -> Mapping[str, Any]:
    layer_payload = loop_layers.get(layer_name)
    if not isinstance(layer_payload, Mapping):
        raise ValueError(f"payload loop layer must be a mapping: {layer_name}")
    return layer_payload


def _build_routed_context_selection_plan(
    *,
    prompt_engine: str,
    routing_plan: Mapping[str, Any],
) -> dict[str, Any]:
    policy = _ROUTED_SELECTION_POLICY_BY_ENGINE[prompt_engine]
    ordered_sections = [
        str(layer_name)
        for layer_name in routing_plan.get("prompt_section_order", ())
        if isinstance(layer_name, str)
    ]
    full_layer_limit = int(policy["full_layer_limit"])
    summary_layer_limit = int(policy["summary_layer_limit"])
    full_sections = ordered_sections[:full_layer_limit]
    summarized_sections = ordered_sections[
        full_layer_limit : full_layer_limit + summary_layer_limit
    ]
    omitted_sections = ordered_sections[full_layer_limit + summary_layer_limit :]
    compressed_sections = [
        str(layer_name)
        for layer_name in routing_plan.get("compressed_sections", ())
        if isinstance(layer_name, str)
    ]
    return {
        "policy_version": "routed_context_selection_v1",
        "full_layer_limit": full_layer_limit,
        "summary_layer_limit": summary_layer_limit,
        "full_sections": full_sections,
        "summarized_sections": summarized_sections,
        "omitted_sections": omitted_sections,
        "compressed_sections": compressed_sections,
        "observation_anchor_mode": str(policy["observation_anchor_mode"]),
    }


def _build_selected_layer_lines(
    *,
    loop_layers: Mapping[str, Any],
    selection_plan: Mapping[str, Any],
    full_label_prefix: str,
    summary_label_prefix: str,
) -> list[str]:
    section_lines: list[str] = []
    for layer_name in selection_plan.get("full_sections", ()):
        if not isinstance(layer_name, str):
            continue
        layer_payload = _require_loop_layer_mapping(loop_layers, layer_name)
        section_lines.extend(
            (
                f"{full_label_prefix}: {layer_name}",
                canonical_json_dumps(dict(layer_payload)),
                "",
            )
        )
    for layer_name in selection_plan.get("summarized_sections", ()):
        if not isinstance(layer_name, str):
            continue
        layer_payload = _require_loop_layer_mapping(loop_layers, layer_name)
        section_lines.extend(
            (
                f"{summary_label_prefix}: {layer_name} (summary)",
                canonical_json_dumps(
                    _build_compact_layer_summary(layer_name, layer_payload)
                ),
                "",
            )
        )
    return section_lines


def _build_compact_layer_summary(
    layer_name: str,
    layer_payload: Mapping[str, Any],
) -> dict[str, Any]:
    if layer_name == "immediate_action":
        exits = layer_payload.get("exits", ())
        visible_entities = layer_payload.get("visible_entities", ())
        return {
            "location": layer_payload.get("location"),
            "exit_count": (
                len(exits)
                if isinstance(exits, Sequence) and not isinstance(exits, (str, bytes))
                else 0
            ),
            "visible_entity_count": (
                len(visible_entities)
                if isinstance(visible_entities, Sequence)
                and not isinstance(visible_entities, (str, bytes))
                else 0
            ),
            "actionable_hazard_pressure": layer_payload.get(
                "actionable_hazard_pressure", 0
            ),
            "hostile_actor_pressure": layer_payload.get("hostile_actor_pressure", 0),
        }
    if layer_name == "local_objective":
        candidate_targets = layer_payload.get("candidate_targets", ())
        inventory = layer_payload.get("inventory", ())
        interaction_actions = layer_payload.get("interaction_actions", ())
        return {
            "candidate_target_count": (
                len(candidate_targets)
                if isinstance(candidate_targets, Sequence)
                and not isinstance(candidate_targets, (str, bytes))
                else 0
            ),
            "inventory_count": (
                len(inventory)
                if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes))
                else 0
            ),
            "interaction_action_count": (
                len(interaction_actions)
                if isinstance(interaction_actions, Sequence)
                and not isinstance(interaction_actions, (str, bytes))
                else 0
            ),
            "dependency_gate_pressure": layer_payload.get("dependency_gate_pressure", 0),
            "location_sensitive_use_pressure": layer_payload.get(
                "location_sensitive_use_pressure", 0
            ),
            "misleading_local_affordance_pressure": layer_payload.get(
                "misleading_local_affordance_pressure", 0
            ),
            "global_progress_pressure": layer_payload.get("global_progress_pressure", 0),
        }
    if layer_name == "temporal_world":
        messages = layer_payload.get("messages", ())
        normalized_messages = (
            [message for message in messages if isinstance(message, str)]
            if isinstance(messages, Sequence) and not isinstance(messages, (str, bytes))
            else []
        )
        return {
            "message_count": len(normalized_messages),
            "message_preview": normalized_messages[:1],
            "step": layer_payload.get("step", 0),
            "remaining_steps": layer_payload.get("remaining_steps", 0),
            "time_pressure": layer_payload.get("time_pressure", 0),
        }
    if layer_name == "multi_agent":
        visible_other_entities = layer_payload.get("visible_other_entities", ())
        normalized_entities = (
            [entity for entity in visible_other_entities if isinstance(entity, str)]
            if isinstance(visible_other_entities, Sequence)
            and not isinstance(visible_other_entities, (str, bytes))
            else []
        )
        return {
            "visible_other_entity_count": layer_payload.get(
                "visible_other_entity_count", 0
            ),
            "entity_preview": normalized_entities[:2],
            "hostile_actor_pressure": layer_payload.get("hostile_actor_pressure", 0),
        }
    if layer_name == "persistence":
        inventory = layer_payload.get("inventory", ())
        normalized_inventory = (
            [item for item in inventory if isinstance(item, str)]
            if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes))
            else []
        )
        return {
            "inventory_count": len(normalized_inventory),
            "inventory_preview": normalized_inventory[:2],
            "health": layer_payload.get("health", 0),
            "step": layer_payload.get("step", 0),
            "remaining_steps": layer_payload.get("remaining_steps", 0),
            "resource_scarcity_pressure": layer_payload.get(
                "resource_scarcity_pressure", 0
            ),
            "consumable_pressure": layer_payload.get("consumable_pressure", 0),
        }
    raise ValueError(f"unsupported loop layer: {layer_name}")


def _build_reduced_observation_summary(
    *,
    observation_payload: Mapping[str, Any],
    allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> dict[str, Any]:
    entities = observation_payload.get("entities", ())
    inventory = observation_payload.get("inventory", ())
    messages = observation_payload.get("messages", ())
    normalized_entities = (
        [
            str(entity.get("name"))
            for entity in entities
            if isinstance(entity, Mapping) and isinstance(entity.get("name"), str)
        ]
        if isinstance(entities, Sequence) and not isinstance(entities, (str, bytes))
        else []
    )
    normalized_inventory = (
        [item for item in inventory if isinstance(item, str)]
        if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes))
        else []
    )
    normalized_messages = (
        [message for message in messages if isinstance(message, str)]
        if isinstance(messages, Sequence) and not isinstance(messages, (str, bytes))
        else []
    )
    return {
        "location": observation_payload.get("location"),
        "step": observation_payload.get("step", 0),
        "remaining_steps": observation_payload.get("remaining_steps", 0),
        "inventory_count": len(normalized_inventory),
        "inventory_preview": normalized_inventory[:2],
        "visible_entity_count": len(normalized_entities),
        "visible_entity_preview": normalized_entities[:2],
        "message_count": len(normalized_messages),
        "message_preview": normalized_messages[:1],
        "allowed_action_count": len(allowed_actions),
        "allowed_target_verbs": sorted(str(key) for key in allowed_targets.keys()),
    }


def _build_routed_loop_layers(
    observation_payload: Mapping[str, Any],
    allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> dict[str, Any]:
    messages = observation_payload.get("messages", ())
    entities = observation_payload.get("entities", ())
    inventory = observation_payload.get("inventory", ())
    exits = observation_payload.get("exits", ())
    visible_entities = [
        str(entity.get("name"))
        for entity in entities
        if isinstance(entity, Mapping) and isinstance(entity.get("name"), str)
    ]
    local_objective_targets = sorted(
        target
        for verb in ("take", "use", "attack", "give")
        for target in _stringified_targets(allowed_targets.get(verb, ()))
    )
    interaction_actions = [
        action
        for action in allowed_actions
        if action.startswith("take ")
        or action.startswith("use ")
        or action.startswith("attack ")
        or action.startswith("give ")
    ]
    multi_agent_entities = sorted(
        str(entity.get("name"))
        for entity in entities
        if isinstance(entity, Mapping)
        and entity.get("type") not in {"item", "consumable", "npc"}
        and isinstance(entity.get("name"), str)
    )
    pressure_features = _build_routed_pressure_features(
        observation_payload,
        allowed_actions,
        allowed_targets,
    )
    return {
        "immediate_action": {
            "location": observation_payload.get("location"),
            "allowed_actions": list(allowed_actions),
            "allowed_targets": dict(allowed_targets),
            "exits": list(exits) if isinstance(exits, Sequence) and not isinstance(exits, (str, bytes)) else [],
            "visible_entities": visible_entities,
            "actionable_hazard_pressure": pressure_features["actionable_hazard_pressure"],
            "hostile_actor_pressure": pressure_features["hostile_actor_pressure"],
        },
        "local_objective": {
            "candidate_targets": local_objective_targets,
            "inventory": list(inventory) if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes)) else [],
            "interaction_actions": interaction_actions,
            "dependency_gate_pressure": pressure_features["dependency_gate_pressure"],
            "location_sensitive_use_pressure": pressure_features["location_sensitive_use_pressure"],
            "misleading_local_affordance_pressure": pressure_features[
                "misleading_local_affordance_pressure"
            ],
            "global_progress_pressure": pressure_features["global_progress_pressure"],
        },
        "temporal_world": {
            "messages": list(messages) if isinstance(messages, Sequence) and not isinstance(messages, (str, bytes)) else [],
            "step": observation_payload.get("step", 0),
            "remaining_steps": observation_payload.get("remaining_steps", 0),
            "time_pressure": pressure_features["time_pressure"],
        },
        "multi_agent": {
            "visible_other_entities": multi_agent_entities,
            "visible_other_entity_count": len(multi_agent_entities),
            "hostile_actor_pressure": pressure_features["hostile_actor_pressure"],
        },
        "persistence": {
            "inventory": list(inventory) if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes)) else [],
            "health": observation_payload.get("health", 0),
            "step": observation_payload.get("step", 0),
            "remaining_steps": observation_payload.get("remaining_steps", 0),
            "resource_scarcity_pressure": pressure_features["resource_scarcity_pressure"],
            "consumable_pressure": pressure_features["consumable_pressure"],
            "global_progress_pressure": pressure_features["global_progress_pressure"],
        },
    }


def _build_routed_routing_context(
    observation_payload: Mapping[str, Any],
    allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> dict[str, Any]:
    inventory = observation_payload.get("inventory", ())
    messages = observation_payload.get("messages", ())
    entities = observation_payload.get("entities", ())
    pressure_features = _build_routed_pressure_features(
        observation_payload,
        allowed_actions,
        allowed_targets,
    )
    return {
        "inventory_count": len(inventory) if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes)) else 0,
        "message_count": len(messages) if isinstance(messages, Sequence) and not isinstance(messages, (str, bytes)) else 0,
        "visible_entity_count": len(entities) if isinstance(entities, Sequence) and not isinstance(entities, (str, bytes)) else 0,
        "visible_other_entity_count": sum(
            1
            for entity in entities
            if isinstance(entity, Mapping)
            and entity.get("type") not in {"item", "consumable", "npc"}
        ) if isinstance(entities, Sequence) and not isinstance(entities, (str, bytes)) else 0,
        "interaction_action_count": sum(
            1
            for action in allowed_actions
            if action.startswith("take ")
            or action.startswith("use ")
            or action.startswith("attack ")
            or action.startswith("give ")
        ),
        "allowed_target_verbs": sorted(str(key) for key in allowed_targets.keys()),
        **pressure_features,
        "pressure_features": pressure_features,
    }


def _build_loop_layer_scores(
    loop_layers: Mapping[str, Any],
    *,
    routing_context: Mapping[str, Any],
) -> dict[str, int]:
    return {
        layer_name: _score_loop_layer(
            layer_name,
            _require_loop_layer_mapping(loop_layers, layer_name),
            routing_context=routing_context,
        )
        for layer_name in _ROUTED_LAYER_ORDER
    }


def _build_canonical_angular_routing_coordinate(
    *,
    layer_scores: Mapping[str, int],
    routing_context: Mapping[str, Any],
) -> dict[str, Any]:
    """Project current prompt/world state into the canonical 4D routing contract.

    MUDBench does not yet have token-embedding routing coordinates. The canonical
    angular path therefore uses one explicit deterministic adapter from the
    existing prompt/world-state loop scores to a normalized 4D coordinate:

    - dim0 `action_local`: immediate-action execution pressure
    - dim1 `local_objective`: objective/resource pressure
    - dim2 `temporal_world`: world-message and time pressure
    - dim3 `shared_state`: persistence plus multi-agent carryover pressure
    """
    component_scores = {
        "immediate_action": max(1, int(layer_scores.get("immediate_action", 1))),
        "local_objective": max(1, int(layer_scores.get("local_objective", 1))),
        "temporal_world": max(1, int(layer_scores.get("temporal_world", 1))),
        "multi_agent": max(1, int(layer_scores.get("multi_agent", 1))),
        "persistence": max(1, int(layer_scores.get("persistence", 1))),
    }
    pressure_features = {
        "actionable_hazard_pressure": int(
            routing_context.get("actionable_hazard_pressure", 0)
        ),
        "hostile_actor_pressure": int(routing_context.get("hostile_actor_pressure", 0)),
        "resource_scarcity_pressure": int(
            routing_context.get("resource_scarcity_pressure", 0)
        ),
        "dependency_gate_pressure": int(
            routing_context.get("dependency_gate_pressure", 0)
        ),
        "misleading_local_affordance_pressure": int(
            routing_context.get("misleading_local_affordance_pressure", 0)
        ),
        "location_sensitive_use_pressure": int(
            routing_context.get("location_sensitive_use_pressure", 0)
        ),
        "global_progress_pressure": int(
            routing_context.get("global_progress_pressure", 0)
        ),
        "time_pressure": int(routing_context.get("time_pressure", 0)),
        "consumable_pressure": int(routing_context.get("consumable_pressure", 0)),
    }
    bundle_feature_totals = {
        "action_local": pressure_features["actionable_hazard_pressure"]
        + pressure_features["hostile_actor_pressure"],
        "local_objective": pressure_features["dependency_gate_pressure"]
        + pressure_features["location_sensitive_use_pressure"]
        + pressure_features["misleading_local_affordance_pressure"]
        + pressure_features["global_progress_pressure"],
        "temporal_world": pressure_features["time_pressure"]
        + int(routing_context.get("message_count", 0)),
        "shared_state": pressure_features["resource_scarcity_pressure"]
        + pressure_features["consumable_pressure"]
        + pressure_features["hostile_actor_pressure"],
    }
    bundle_scores = {
        "action_local": component_scores["immediate_action"]
        + bundle_feature_totals["action_local"],
        "local_objective": component_scores["local_objective"]
        + bundle_feature_totals["local_objective"],
        "temporal_world": component_scores["temporal_world"]
        + bundle_feature_totals["temporal_world"],
        "shared_state": component_scores["persistence"]
        + component_scores["multi_agent"]
        + bundle_feature_totals["shared_state"],
    }
    raw_coordinate = (
        float(bundle_scores["action_local"]),
        float(bundle_scores["local_objective"]),
        float(bundle_scores["temporal_world"]),
        float(bundle_scores["shared_state"]),
    )
    normalized_coordinate = normalize_4d_coordinate(raw_coordinate)
    return {
        "adapter_schema": "mudbench_prompt_state_v2",
        "coordinate_labels": [
            "action_local",
            "local_objective",
            "temporal_world",
            "shared_state",
        ],
        "component_scores": component_scores,
        "pressure_features": pressure_features,
        "bundle_feature_totals": bundle_feature_totals,
        "bundle_scores": bundle_scores,
        "raw_coordinate": raw_coordinate,
        "normalized_coordinate": tuple(round(value, 6) for value in normalized_coordinate),
    }


def _stringified_targets(value: object) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, Sequence) and not isinstance(value, (str, bytes)):
        items: list[str] = []
        for entry in value:
            if isinstance(entry, str):
                items.append(entry)
            elif isinstance(entry, Mapping):
                for mapping_value in entry.values():
                    if isinstance(mapping_value, str):
                        items.append(mapping_value)
        return items
    return []


def _build_routed_pressure_features(
    observation_payload: Mapping[str, Any],
    allowed_actions: Sequence[str],
    allowed_targets: Mapping[str, Any],
) -> dict[str, Any]:
    entities = observation_payload.get("entities", ())
    inventory = observation_payload.get("inventory", ())
    messages = observation_payload.get("messages", ())
    description = str(observation_payload.get("description", ""))
    location = str(observation_payload.get("location", ""))
    normalized_entities = (
        [entity for entity in entities if isinstance(entity, Mapping)]
        if isinstance(entities, Sequence) and not isinstance(entities, (str, bytes))
        else []
    )
    normalized_inventory = (
        [str(item) for item in inventory if isinstance(item, str)]
        if isinstance(inventory, Sequence) and not isinstance(inventory, (str, bytes))
        else []
    )
    normalized_messages = (
        [message for message in messages if isinstance(message, str)]
        if isinstance(messages, Sequence) and not isinstance(messages, (str, bytes))
        else []
    )
    visible_entity_names = [
        str(entity.get("name"))
        for entity in normalized_entities
        if isinstance(entity.get("name"), str)
    ]
    npc_entity_count = sum(
        1 for entity in normalized_entities if entity.get("type") == "npc"
    )
    attack_target_count = len(_stringified_targets(allowed_targets.get("attack", ())))
    use_target_count = len(_stringified_targets(allowed_targets.get("use", ())))
    take_target_count = len(_stringified_targets(allowed_targets.get("take", ())))
    text_blob = " ".join(
        [
            location,
            description,
            *normalized_messages,
            *visible_entity_names,
            *normalized_inventory,
        ]
    ).lower()
    hazard_keyword_hits = _count_keyword_hits(text_blob, _ROUTED_HAZARD_KEYWORDS)
    gate_keyword_hits = _count_keyword_hits(text_blob, _ROUTED_GATE_KEYWORDS)
    progress_keyword_hits = _count_keyword_hits(text_blob, _ROUTED_PROGRESS_KEYWORDS)
    misleading_keyword_hits = _count_keyword_hits(text_blob, _ROUTED_MISLEADING_KEYWORDS)
    resource_keyword_inventory_count = sum(
        1
        for item in normalized_inventory
        if any(keyword in item.lower() for keyword in _ROUTED_RESOURCE_KEYWORDS)
    )
    consumable_pressure = sum(
        1
        for item in normalized_inventory
        if any(
            keyword in item.lower()
            for keyword in _ROUTED_CONSUMABLE_RESOURCE_KEYWORDS
        )
    )
    remaining_steps = int(observation_payload.get("remaining_steps", 0))
    time_pressure = (
        3
        if remaining_steps <= 2
        else 2
        if remaining_steps <= 4
        else 1
        if remaining_steps <= 6
        else 0
    )
    hostile_actor_pressure = attack_target_count + npc_entity_count + min(
        hazard_keyword_hits,
        2,
    )
    actionable_hazard_pressure = (
        hostile_actor_pressure + time_pressure + (1 if attack_target_count > 0 else 0)
    )
    dependency_gate_pressure = (
        gate_keyword_hits
        + (1 if use_target_count > 0 else 0)
        + (1 if resource_keyword_inventory_count > 0 else 0)
    )
    location_sensitive_use_pressure = use_target_count * (1 + min(gate_keyword_hits, 2))
    resource_scarcity_pressure = (
        resource_keyword_inventory_count
        + consumable_pressure
        + (1 if use_target_count > 0 and consumable_pressure > 0 else 0)
        + (
            1
            if resource_keyword_inventory_count > 0 and remaining_steps <= 4
            else 0
        )
    )
    misleading_local_affordance_pressure = misleading_keyword_hits + (
        1 if misleading_keyword_hits > 0 and use_target_count > 0 else 0
    )
    global_progress_pressure = (
        progress_keyword_hits
        + (1 if take_target_count > 0 else 0)
        + (1 if any(token in location.lower() for token in ("archive", "vault", "seal")) else 0)
    )
    active_pressure_tags = [
        tag
        for tag, value in (
            ("hazard_pressure", actionable_hazard_pressure),
            ("hostile_actor_pressure", hostile_actor_pressure),
            ("resource_scarcity_pressure", resource_scarcity_pressure),
            ("dependency_gate_pressure", dependency_gate_pressure),
            (
                "misleading_local_affordance_pressure",
                misleading_local_affordance_pressure,
            ),
            ("location_sensitive_use_pressure", location_sensitive_use_pressure),
            ("global_progress_pressure", global_progress_pressure),
            ("time_pressure", time_pressure),
        )
        if value > 0
    ]
    return {
        "npc_entity_count": npc_entity_count,
        "attack_target_count": attack_target_count,
        "use_target_count": use_target_count,
        "take_target_count": take_target_count,
        "hazard_keyword_hits": hazard_keyword_hits,
        "gate_keyword_hits": gate_keyword_hits,
        "progress_keyword_hits": progress_keyword_hits,
        "misleading_keyword_hits": misleading_keyword_hits,
        "resource_keyword_inventory_count": resource_keyword_inventory_count,
        "consumable_pressure": consumable_pressure,
        "time_pressure": time_pressure,
        "hostile_actor_pressure": hostile_actor_pressure,
        "actionable_hazard_pressure": actionable_hazard_pressure,
        "resource_scarcity_pressure": resource_scarcity_pressure,
        "dependency_gate_pressure": dependency_gate_pressure,
        "misleading_local_affordance_pressure": misleading_local_affordance_pressure,
        "location_sensitive_use_pressure": location_sensitive_use_pressure,
        "global_progress_pressure": global_progress_pressure,
        "active_pressure_tags": active_pressure_tags,
    }


def _count_keyword_hits(text_blob: str, keywords: Sequence[str]) -> int:
    return sum(1 for keyword in keywords if keyword in text_blob)


def _score_loop_layer(
    layer_name: str,
    layer_payload: Mapping[str, Any],
    *,
    routing_context: Mapping[str, Any],
) -> int:
    if layer_name == "immediate_action":
        allowed_actions = layer_payload.get("allowed_actions", ())
        exits = layer_payload.get("exits", ())
        return (
            24
            + (len(allowed_actions) * 4)
            + (len(exits) * 3)
            + (int(routing_context.get("actionable_hazard_pressure", 0)) * 6)
            + (int(routing_context.get("hostile_actor_pressure", 0)) * 4)
        )
    if layer_name == "local_objective":
        candidate_targets = layer_payload.get("candidate_targets", ())
        interaction_actions = layer_payload.get("interaction_actions", ())
        return (
            20
            + (len(candidate_targets) * 5)
            + (len(interaction_actions) * 3)
            + (int(routing_context.get("dependency_gate_pressure", 0)) * 8)
            + (int(routing_context.get("location_sensitive_use_pressure", 0)) * 5)
            + (
                int(routing_context.get("misleading_local_affordance_pressure", 0))
                * 4
            )
            + (int(routing_context.get("global_progress_pressure", 0)) * 3)
        )
    if layer_name == "temporal_world":
        messages = layer_payload.get("messages", ())
        score = 10 + (len(messages) * 10) + (int(routing_context.get("time_pressure", 0)) * 8)
        message_blob = " ".join(
            message for message in messages if isinstance(message, str)
        ).lower()
        if any(token in message_blob for token in ("watch", "patrol", "phase", "consequence", "route")):
            score += 25
        score += int(routing_context.get("actionable_hazard_pressure", 0)) * 2
        return score
    if layer_name == "multi_agent":
        return (
            5
            + (int(routing_context.get("visible_other_entity_count", 0)) * 8)
            + (int(routing_context.get("hostile_actor_pressure", 0)) * 6)
            + (int(routing_context.get("npc_entity_count", 0)) * 4)
        )
    if layer_name == "persistence":
        return (
            12
            + (int(routing_context.get("inventory_count", 0)) * 5)
            + int(layer_payload.get("step", 0))
            + (int(routing_context.get("resource_scarcity_pressure", 0)) * 8)
            + (int(routing_context.get("consumable_pressure", 0)) * 6)
            + (int(routing_context.get("global_progress_pressure", 0)) * 2)
        )
    raise ValueError(f"unsupported loop layer: {layer_name}")


def _build_reasoning_pressure_tag(dominant_loop: str) -> str:
    return {
        "immediate_action": "execution_pressure",
        "local_objective": "objective_pressure",
        "temporal_world": "temporal_pressure",
        "multi_agent": "coordination_pressure",
        "persistence": "state_carryover_pressure",
    }[dominant_loop]


def _normalize_prompt_engine(prompt_engine: str) -> str:
    if not isinstance(prompt_engine, str):
        raise ValueError(
            "prompt_engine must be one of: baseline, geometric-routed, angular-canonical, legacy-router-backed"
        )
    normalized_prompt_engine = _PROMPT_ENGINE_ALIASES.get(prompt_engine, prompt_engine)
    if normalized_prompt_engine not in _SUPPORTED_PROMPT_ENGINES:
        raise ValueError(
            "prompt_engine must be one of: baseline, geometric-routed, angular-canonical, legacy-router-backed"
        )
    return normalized_prompt_engine


def _normalize_router_variant(prompt_engine: str, router_variant: str | None) -> str | None:
    normalized_prompt_engine = _normalize_prompt_engine(prompt_engine)
    if normalized_prompt_engine == _PROMPT_ENGINE_ANGULAR_CANONICAL:
        normalized_router_variant = router_variant or _DEFAULT_CANONICAL_ROUTER_VARIANT
        if normalized_router_variant not in SUPPORTED_CANONICAL_ANGULAR_VARIANTS:
            raise ValueError(
                "router_variant must be one of: "
                + ", ".join(SUPPORTED_CANONICAL_ANGULAR_VARIANTS)
            )
        return normalized_router_variant
    if normalized_prompt_engine == _PROMPT_ENGINE_LEGACY_ROUTER_BACKED:
        if router_variant is None:
            normalized_router_variant = _DEFAULT_LEGACY_ROUTER_VARIANT
        else:
            normalized_router_variant = _LEGACY_ROUTER_VARIANT_ALIASES.get(
                router_variant,
                router_variant,
            )
        if normalized_router_variant not in SUPPORTED_LEGACY_ROUTER_VARIANTS:
            raise ValueError(
                "router_variant must be one of: "
                + ", ".join(SUPPORTED_LEGACY_ROUTER_VARIANTS)
            )
        return normalized_router_variant
    return None


def build_invalid_output_repair_prompt(
    *,
    failure_reason: str,
    action_space: Sequence[str],
    allowed_targets: Mapping[str, Any] | None = None,
    mode: str = _BENCHMARK_MODE,
) -> str:
    """Build a bounded repair prompt for one invalid model output case."""
    if not isinstance(failure_reason, str) or not failure_reason:
        raise ValueError("failure_reason must be a non-empty string")
    if isinstance(action_space, (str, bytes)) or not isinstance(action_space, Sequence):
        raise ValueError("action_space must be a sequence of strings")
    if not isinstance(mode, str) or mode not in {_BENCHMARK_MODE, _PERSISTENT_SESSION_MODE}:
        raise ValueError("mode must be benchmark_single_turn or persistent_session")
    if allowed_targets is not None and not isinstance(allowed_targets, Mapping):
        raise ValueError("allowed_targets must be a mapping when provided")

    normalized_action_space: list[str] = []
    for action in action_space:
        if not isinstance(action, str):
            raise ValueError("action_space must contain only strings")
        normalized_action_space.append(action)

    normalized_allowed_targets = dict(allowed_targets or {})
    return "\n".join(
        (
            "Your previous response was invalid for MUDBench.",
            f"Runtime mode: {mode}",
            "",
            "Failure:",
            f"- {failure_reason}",
            "",
            "Repair guardrails:",
            "- Return JSON only.",
            '- Return exactly one object with exactly one field: "action".',
            "- Do not include explanation, markdown, or extra keys.",
            "- The action must exactly match one item from this action_space:",
            canonical_json_dumps({"action_space": normalized_action_space}),
            "- If the action references a target-bearing argument, it must come from these allowed targets:",
            canonical_json_dumps({"allowed_targets": normalized_allowed_targets}),
        )
    )


def build_fail_closed_action_submission(
    action_space: Sequence[str],
) -> ActionSubmission:
    """Choose one deterministic fail-closed fallback action."""
    if isinstance(action_space, (str, bytes)) or not isinstance(action_space, Sequence):
        raise ValueError("action_space must be a sequence of strings")

    normalized_action_space: list[str] = []
    for action in action_space:
        if not isinstance(action, str):
            raise ValueError("action_space must contain only strings")
        normalized_action_space.append(action)
    if not normalized_action_space:
        raise ValueError("action_space must not be empty")

    for preferred in ("wait", "look"):
        if preferred in normalized_action_space:
            return ActionSubmission(action=preferred)
    return ActionSubmission(action=normalized_action_space[0])


def build_runtime_telemetry(
    *,
    initial_parse_result: ModelActionOutputParseResult,
    repaired_parse_result: ModelActionOutputParseResult | None = None,
    used_fail_closed_fallback: bool = False,
    fail_closed_reason: str | None = None,
    provider_runtime: Mapping[str, Any] | None = None,
) -> dict[str, Any]:
    """Build deterministic runtime telemetry for one benchmark LLM turn."""
    if not isinstance(initial_parse_result, ModelActionOutputParseResult):
        raise ValueError("initial_parse_result must be a ModelActionOutputParseResult")
    if repaired_parse_result is not None and not isinstance(
        repaired_parse_result, ModelActionOutputParseResult
    ):
        raise ValueError("repaired_parse_result must be a ModelActionOutputParseResult when provided")
    if not isinstance(used_fail_closed_fallback, bool):
        raise ValueError("used_fail_closed_fallback must be a boolean")
    if fail_closed_reason is not None and (not isinstance(fail_closed_reason, str) or not fail_closed_reason):
        raise ValueError("fail_closed_reason must be a non-empty string when provided")
    if provider_runtime is not None and not isinstance(provider_runtime, Mapping):
        raise ValueError("provider_runtime must be a mapping when provided")

    repair_used = repaired_parse_result is not None
    if used_fail_closed_fallback:
        final_parse_status = "fail_closed"
        failure_reason = fail_closed_reason
    elif repair_used:
        final_parse_status = "accepted_after_repair"
        failure_reason = initial_parse_result.reason
    else:
        final_parse_status = "accepted_initial"
        failure_reason = None

    telemetry: dict[str, Any] = {
        "repair_used": repair_used,
        "fail_closed_used": used_fail_closed_fallback,
        "final_parse_status": final_parse_status,
        "failure_reason": failure_reason,
    }
    if provider_runtime is not None:
        for key in ("provider_name", "provider_request_count", "provider_latency_ms"):
            if key in provider_runtime:
                telemetry[key] = provider_runtime[key]
    return telemetry


def run_benchmark_llm_turn(
    observation: Observation,
    *,
    model_completion: Callable[[str], str],
    actor_id: str | None = None,
    prompt_engine: str = _PROMPT_ENGINE_BASELINE,
    router_variant: str | None = None,
) -> BenchmarkLLMTurnResult:
    """Execute one benchmark-mode LLM turn with one bounded repair attempt."""
    if not callable(model_completion):
        raise ValueError("model_completion must be callable")
    normalized_prompt_engine = _normalize_prompt_engine(prompt_engine)
    normalized_router_variant = _normalize_router_variant(
        normalized_prompt_engine,
        router_variant,
    )

    payload = build_canonical_model_facing_observation_payload(
        observation,
        actor_id=actor_id,
    )
    routing_plan = None
    if normalized_prompt_engine == _PROMPT_ENGINE_GEOMETRIC_ROUTED:
        routing_plan = build_geometric_routed_prompt_plan(payload, expected_mode=_BENCHMARK_MODE)
    elif normalized_prompt_engine == _PROMPT_ENGINE_ANGULAR_CANONICAL:
        routing_plan = build_angular_canonical_prompt_plan(
            payload,
            expected_mode=_BENCHMARK_MODE,
            router_variant=normalized_router_variant,
        )
    elif normalized_prompt_engine == _PROMPT_ENGINE_LEGACY_ROUTER_BACKED:
        routing_plan = build_legacy_router_backed_prompt_plan(
            payload,
            expected_mode=_BENCHMARK_MODE,
            router_variant=normalized_router_variant,
        )
    prompt = build_benchmark_prompt_from_payload(
        payload,
        prompt_engine=normalized_prompt_engine,
        router_variant=normalized_router_variant,
    )
    initial_raw_output = model_completion(prompt)
    initial_parse_result = parse_model_action_output(
        raw_output=initial_raw_output,
        action_space=observation.action_space,
    )
    if initial_parse_result.accepted:
        if initial_parse_result.action_submission is None:
            raise ValueError("accepted model output must include action_submission")
        return BenchmarkLLMTurnResult(
            model_facing_observation_payload=payload,
            prompt=prompt,
            prompt_engine=normalized_prompt_engine,
            router_variant=(
                normalized_router_variant
                if normalized_prompt_engine
                in {_PROMPT_ENGINE_ANGULAR_CANONICAL, _PROMPT_ENGINE_LEGACY_ROUTER_BACKED}
                else None
            ),
            routing_plan=routing_plan,
            initial_parse_result=initial_parse_result,
            action_submission=initial_parse_result.action_submission,
            runtime_telemetry=build_runtime_telemetry(
                initial_parse_result=initial_parse_result,
            ),
        )

    repair_prompt = build_invalid_output_repair_prompt(
        failure_reason=initial_parse_result.reason or "unknown_model_output_failure",
        action_space=observation.action_space,
        allowed_targets=payload.get("allowed_targets"),
        mode=_BENCHMARK_MODE,
    )
    repaired_raw_output = model_completion(repair_prompt)
    repaired_parse_result = parse_model_action_output(
        raw_output=repaired_raw_output,
        action_space=observation.action_space,
    )
    if repaired_parse_result.accepted:
        if repaired_parse_result.action_submission is None:
            raise ValueError("accepted repaired output must include action_submission")
        return BenchmarkLLMTurnResult(
            model_facing_observation_payload=payload,
            prompt=prompt,
            prompt_engine=normalized_prompt_engine,
            router_variant=(
                normalized_router_variant
                if normalized_prompt_engine
                in {_PROMPT_ENGINE_ANGULAR_CANONICAL, _PROMPT_ENGINE_LEGACY_ROUTER_BACKED}
                else None
            ),
            routing_plan=routing_plan,
            initial_parse_result=initial_parse_result,
            repair_prompt=repair_prompt,
            repaired_parse_result=repaired_parse_result,
            action_submission=repaired_parse_result.action_submission,
            runtime_telemetry=build_runtime_telemetry(
                initial_parse_result=initial_parse_result,
                repaired_parse_result=repaired_parse_result,
            ),
        )

    fail_closed_reason = (
        "model_output_rejected_after_repair:"
        f"{repaired_parse_result.reason or 'unknown_model_output_failure'}"
    )
    return BenchmarkLLMTurnResult(
        model_facing_observation_payload=payload,
        prompt=prompt,
        prompt_engine=normalized_prompt_engine,
        router_variant=(
            normalized_router_variant
            if normalized_prompt_engine
            in {_PROMPT_ENGINE_ANGULAR_CANONICAL, _PROMPT_ENGINE_LEGACY_ROUTER_BACKED}
            else None
        ),
        routing_plan=routing_plan,
        initial_parse_result=initial_parse_result,
        repair_prompt=repair_prompt,
        repaired_parse_result=repaired_parse_result,
        action_submission=build_fail_closed_action_submission(observation.action_space),
        used_fail_closed_fallback=True,
        fail_closed_reason=fail_closed_reason,
        runtime_telemetry=build_runtime_telemetry(
            initial_parse_result=initial_parse_result,
            repaired_parse_result=repaired_parse_result,
            used_fail_closed_fallback=True,
            fail_closed_reason=fail_closed_reason,
        ),
    )


def build_mock_llm_model_completion(
    action_space: Sequence[str],
    *,
    emit_invalid_first: bool = False,
    emit_invalid_always: bool = False,
) -> Callable[[str], str]:
    """Build the deterministic mock LLM completion used by local wrapper paths."""
    if isinstance(action_space, (str, bytes)) or not isinstance(action_space, Sequence):
        raise ValueError("action_space must be a sequence of strings")
    normalized_action_space: list[str] = []
    for action in action_space:
        if not isinstance(action, str):
            raise ValueError("action_space must contain only strings")
        normalized_action_space.append(action)

    attempts = {"count": 0}

    def _completion(prompt: str) -> str:
        attempt = attempts["count"]
        attempts["count"] += 1
        del prompt
        if emit_invalid_always:
            return "I would choose an action now."
        if emit_invalid_first and attempt == 0:
            return "I would choose an action now."
        return canonical_json_dumps(
            {"action": _select_mock_llm_action(normalized_action_space)}
        )

    return _completion


def run_mock_benchmark_llm_turn(
    observation: Observation,
    *,
    actor_id: str | None = None,
    emit_invalid_first: bool = False,
    emit_invalid_always: bool = False,
) -> BenchmarkLLMTurnResult:
    """Execute one benchmark-mode turn using the deterministic local mock LLM policy."""
    return run_benchmark_llm_turn(
        observation,
        model_completion=build_mock_llm_model_completion(
            observation.action_space,
            emit_invalid_first=emit_invalid_first,
            emit_invalid_always=emit_invalid_always,
        ),
        actor_id=actor_id,
    )


def _select_mock_llm_action(action_space: Sequence[str]) -> str:
    for candidate in action_space:
        if candidate.startswith("take "):
            return candidate
    for candidate in action_space:
        if candidate.startswith("move "):
            return candidate
    if "look" in action_space:
        return "look"
    if "wait" in action_space:
        return "wait"
    if not action_space:
        raise ValueError("action_space must not be empty")
    return action_space[0]
