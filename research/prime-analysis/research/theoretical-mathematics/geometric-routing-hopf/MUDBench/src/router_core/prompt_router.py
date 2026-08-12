"""Legacy proxy router-backed prompt planning core.

This vendors only a tiny mathematical subset adapted from
`~/ai-router/router-research/hyperbolic_router_so8.py`:

- `PHI`
- `LOG_PHI`
- `fibonacci_values_upto`
- `allocate_pair_bins` logic, reduced to scalar prompt-planning use

This module intentionally remains a legacy prompt-layer surrogate. It does not
implement the canonical preprint-era single-shell angular router.
"""

from __future__ import annotations

import math
from typing import Mapping

PHI = (1.0 + math.sqrt(5.0)) / 2.0
LOG_PHI = math.log(PHI)
_DEFAULT_LAYER_ORDER = (
    "immediate_action",
    "local_objective",
    "temporal_world",
    "multi_agent",
    "persistence",
)
SUPPORTED_LEGACY_ROUTER_VARIANTS = (
    "legacy-phase4d_hopf_base",
    "legacy-phase4d_hopf_transport",
    "legacy-phase4d_hopf_product_phase",
)
_LEGACY_ROUTER_VARIANT_ALIASES = {
    "phase4d_hopf_base": "legacy-phase4d_hopf_base",
    "phase4d_hopf_transport": "legacy-phase4d_hopf_transport",
    "phase4d_hopf_product_phase": "legacy-phase4d_hopf_product_phase",
}


def fibonacci_values_upto(max_value: int) -> list[int]:
    max_value = max(1, int(max_value))
    vals = [1, 2]
    while vals[-1] < max_value:
        vals.append(vals[-1] + vals[-2])
    return [value for value in vals if value <= max_value]


def _fibonacci_ceil(value: int, *, max_value: int) -> int:
    for fib in fibonacci_values_upto(max_value):
        if fib >= value:
            return fib
    return fibonacci_values_upto(max_value)[-1]


def _allocate_pair_bins(total_cap: int, *, min_bins: int, ratio_scale: float) -> tuple[int, int]:
    total_cap = max(1, int(total_cap))
    ratio_scale = max(float(ratio_scale), 1e-9)
    pair_min = min(total_cap, max(1, int(min_bins)))

    base = math.sqrt(float(total_cap))
    k1 = max(pair_min, min(total_cap, int(round(base * ratio_scale))))
    k2 = max(pair_min, min(total_cap, int(round(float(total_cap) / max(k1, 1)))))

    if k1 * k2 > total_cap:
        k2 = max(pair_min, min(total_cap, int(math.floor(float(total_cap) / max(k1, 1)))))
    if k1 * k2 > total_cap:
        k1 = max(pair_min, min(total_cap, int(math.floor(float(total_cap) / max(k2, 1)))))
    return k1, k2


def build_legacy_router_backed_prompt_plan(
    loop_scores: Mapping[str, int],
    *,
    router_variant: str,
    layer_order: tuple[str, ...] = _DEFAULT_LAYER_ORDER,
) -> dict[str, object]:
    """Build a deterministic prompt routing plan using a tiny vendored router subset."""
    normalized_router_variant = _normalize_router_variant(router_variant)
    normalized_scores = {
        layer_name: max(1, int(loop_scores.get(layer_name, 1)))
        for layer_name in layer_order
    }
    variant_scored_layers = _apply_router_variant_bias(
        normalized_scores,
        router_variant=normalized_router_variant,
        layer_order=layer_order,
    )
    ordered_layers = sorted(
        layer_order,
        key=lambda layer_name: (-variant_scored_layers[layer_name], layer_order.index(layer_name)),
    )

    max_score = max(variant_scored_layers.values())
    quantized_scores = {
        layer_name: _fibonacci_ceil(
            max(1, int(round(variant_scored_layers[layer_name] / 10.0))),
            max_value=max(5, int(round(max_score / 10.0))),
        )
        for layer_name in layer_order
    }
    dominant_loop = ordered_layers[0]
    secondary_loops = list(ordered_layers[1:])
    secondary_ratio = (
        variant_scored_layers[dominant_loop]
        / max(variant_scored_layers[secondary_loops[0]], 1)
    )
    primary_band, secondary_band = _allocate_pair_bins(
        max(sum(quantized_scores.values()), len(layer_order)),
        min_bins=1,
        ratio_scale=secondary_ratio * _router_variant_ratio_scale(normalized_router_variant),
    )
    compressed_sections = [
        layer_name
        for layer_name in ordered_layers[1:]
        if quantized_scores[layer_name]
        <= _router_variant_compression_threshold(
            normalized_router_variant,
            secondary_band=secondary_band,
        )
    ]
    emphasis_bands = {
        layer_name: (
            "primary"
            if quantized_scores[layer_name] >= primary_band
            else "secondary"
            if quantized_scores[layer_name] > secondary_band
            else "compressed"
        )
        for layer_name in ordered_layers
    }

    return {
        "engine": "legacy-router-backed",
        "router_family": "legacy_phi_fibonacci_prompt_router",
        "router_variant": normalized_router_variant,
        "dominant_loop": dominant_loop,
        "secondary_loops": secondary_loops,
        "prompt_section_order": ordered_layers,
        "compressed_sections": compressed_sections,
        "reasoning_pressure_tag": _reasoning_pressure_tag(
            dominant_loop,
            router_variant=normalized_router_variant,
        ),
        "quantized_loop_scores": quantized_scores,
        "variant_scored_layers": variant_scored_layers,
        "emphasis_bands": emphasis_bands,
        "pair_band_budget": {
            "primary_band": primary_band,
            "secondary_band": secondary_band,
        },
    }


def _normalize_router_variant(router_variant: str) -> str:
    if not isinstance(router_variant, str):
        raise ValueError(
            "router_variant must be one of: "
            + ", ".join(SUPPORTED_LEGACY_ROUTER_VARIANTS)
        )
    normalized_router_variant = _LEGACY_ROUTER_VARIANT_ALIASES.get(router_variant, router_variant)
    if normalized_router_variant not in SUPPORTED_LEGACY_ROUTER_VARIANTS:
        raise ValueError(
            "router_variant must be one of: "
            + ", ".join(SUPPORTED_LEGACY_ROUTER_VARIANTS)
        )
    return normalized_router_variant


def _apply_router_variant_bias(
    normalized_scores: Mapping[str, int],
    *,
    router_variant: str,
    layer_order: tuple[str, ...],
) -> dict[str, int]:
    variant_bias_by_layer = {
        "legacy-phase4d_hopf_base": {
            "immediate_action": 12,
            "local_objective": 8,
        },
        "legacy-phase4d_hopf_transport": {
            "temporal_world": 14,
            "multi_agent": 10,
            "persistence": 6,
        },
        "legacy-phase4d_hopf_product_phase": {
            "local_objective": 12,
            "temporal_world": 10,
            "multi_agent": 8,
            "persistence": 10,
        },
    }[router_variant]
    return {
        layer_name: normalized_scores[layer_name] + variant_bias_by_layer.get(layer_name, 0)
        for layer_name in layer_order
    }


def _router_variant_ratio_scale(router_variant: str) -> float:
    return {
        "legacy-phase4d_hopf_base": 1.0,
        "legacy-phase4d_hopf_transport": 1.15,
        "legacy-phase4d_hopf_product_phase": 1.25,
    }[router_variant]


def _router_variant_compression_threshold(
    router_variant: str,
    *,
    secondary_band: int,
) -> int:
    if router_variant == "legacy-phase4d_hopf_product_phase":
        return max(1, secondary_band - 1)
    return secondary_band


def _reasoning_pressure_tag(dominant_loop: str, *, router_variant: str) -> str:
    phase_prefix = {
        "legacy-phase4d_hopf_base": "legacy_hopf_base",
        "legacy-phase4d_hopf_transport": "legacy_hopf_transport",
        "legacy-phase4d_hopf_product_phase": "legacy_hopf_product_phase",
    }[router_variant]
    loop_suffix = {
        "immediate_action": "router_execution_pressure",
        "local_objective": "router_objective_pressure",
        "temporal_world": "router_temporal_pressure",
        "multi_agent": "router_coordination_pressure",
        "persistence": "router_state_carryover_pressure",
    }[dominant_loop]
    return f"{phase_prefix}:{loop_suffix}"


SUPPORTED_ROUTER_VARIANTS = SUPPORTED_LEGACY_ROUTER_VARIANTS
build_router_backed_prompt_plan = build_legacy_router_backed_prompt_plan
