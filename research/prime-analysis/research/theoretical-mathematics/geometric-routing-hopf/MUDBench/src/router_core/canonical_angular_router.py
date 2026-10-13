"""Canonical single-shell angular router core for prompt planning.

This is a bounded pure-Python extraction of the frozen preprint-era mechanism:

- normalize the routing coordinate onto one unit shell
- route through Hopf-base angular sectors
- keep BASE as the reference control
- keep TRANS as the canonical phase-transport regime

The caller is responsible for supplying one deterministic 4D routing coordinate
derived from MUDBench prompt/world state. This module intentionally does not
import the older shell-heavy routing pipeline.
"""

from __future__ import annotations

import math
from typing import Mapping
from typing import Sequence

SUPPORTED_CANONICAL_ANGULAR_VARIANTS = (
    "angular-hopf-base",
    "angular-hopf-trans",
)
DEFAULT_CANONICAL_ANGULAR_VARIANT = "angular-hopf-trans"
CANONICAL_ANGULAR_ROUTER_K = 8
CANONICAL_ANGULAR_HOPF_CHI_BINS = 2
CANONICAL_ANGULAR_PHASE_TRANSPORT_LAMBDA = 1.0
CANONICAL_ANGULAR_COORDINATE_LABELS = (
    "action_local",
    "local_objective",
    "temporal_world",
    "shared_state",
)
_BASE_BUNDLE_ORDER = CANONICAL_ANGULAR_COORDINATE_LABELS


def normalize_4d_coordinate(routing_coordinate: Sequence[float]) -> tuple[float, float, float, float]:
    """Normalize one deterministic 4D routing coordinate onto the unit shell."""
    if isinstance(routing_coordinate, (str, bytes)) or not isinstance(routing_coordinate, Sequence):
        raise ValueError("routing_coordinate must be a sequence of four numbers")
    if len(routing_coordinate) != 4:
        raise ValueError("routing_coordinate must contain exactly four values")
    normalized_values: list[float] = []
    for value in routing_coordinate:
        if not isinstance(value, (int, float)):
            raise ValueError("routing_coordinate must contain only numbers")
        normalized_values.append(float(value))
    norm = math.sqrt(sum(value * value for value in normalized_values))
    if norm <= 1e-12:
        norm = 1.0
    return tuple(value / norm for value in normalized_values)  # type: ignore[return-value]


def build_canonical_angular_prompt_plan(
    *,
    routing_coordinate: Sequence[float],
    bundle_scores: Mapping[str, int],
    persistence_score: int,
    multi_agent_score: int,
    router_variant: str = DEFAULT_CANONICAL_ANGULAR_VARIANT,
) -> dict[str, object]:
    """Build one deterministic canonical angular prompt plan."""
    normalized_router_variant = _normalize_canonical_variant(router_variant)
    normalized_coordinate = normalize_4d_coordinate(routing_coordinate)
    normalized_bundle_scores = {
        bundle_name: max(1, int(bundle_scores.get(bundle_name, 1)))
        for bundle_name in _BASE_BUNDLE_ORDER
    }
    persistence_score = max(1, int(persistence_score))
    multi_agent_score = max(1, int(multi_agent_score))

    if normalized_router_variant == "angular-hopf-base":
        sector_info = _assign_sector_angular_hopf_base(
            normalized_coordinate,
            K=CANONICAL_ANGULAR_ROUTER_K,
        )
    else:
        sector_info = _assign_sector_angular_hopf_trans(
            normalized_coordinate,
            K=CANONICAL_ANGULAR_ROUTER_K,
            phase_transport_lambda=CANONICAL_ANGULAR_PHASE_TRANSPORT_LAMBDA,
            hopf_chi_bins=CANONICAL_ANGULAR_HOPF_CHI_BINS,
        )

    ordered_bundles = sorted(
        _BASE_BUNDLE_ORDER,
        key=lambda bundle_name: (
            -normalized_bundle_scores[bundle_name],
            _bundle_tiebreak_rank(
                bundle_name,
                router_variant=normalized_router_variant,
                sector_info=sector_info,
            ),
            _BASE_BUNDLE_ORDER.index(bundle_name),
        ),
    )
    shared_state_layers = _expand_shared_state_layers(
        router_variant=normalized_router_variant,
        sector_info=sector_info,
        persistence_score=persistence_score,
        multi_agent_score=multi_agent_score,
    )
    prompt_section_order = _expand_prompt_section_order(
        ordered_bundles,
        shared_state_layers=shared_state_layers,
    )
    compressed_sections = _build_compressed_sections(
        ordered_bundles,
        shared_state_layers=shared_state_layers,
        router_variant=normalized_router_variant,
    )
    dominant_bundle = ordered_bundles[0]
    dominant_section = prompt_section_order[0]

    plan: dict[str, object] = {
        "engine": "angular-canonical",
        "router_family": "canonical_single_shell_angular_hopf_router",
        "router_variant": normalized_router_variant,
        "single_shell": True,
        "shell_id": 0,
        "router_k": CANONICAL_ANGULAR_ROUTER_K,
        "coordinate_adapter_contract": "mudbench_prompt_state_v1",
        "coordinate_labels": list(CANONICAL_ANGULAR_COORDINATE_LABELS),
        "raw_coordinate": [round(float(value), 6) for value in routing_coordinate],
        "normalized_coordinate": [round(value, 6) for value in normalized_coordinate],
        "bundle_scores": dict(normalized_bundle_scores),
        "dominant_bundle": dominant_bundle,
        "secondary_bundles": list(ordered_bundles[1:]),
        "prompt_section_order": prompt_section_order,
        "compressed_sections": compressed_sections,
        "reasoning_pressure_tag": _reasoning_pressure_tag(
            dominant_section,
            router_variant=normalized_router_variant,
        ),
        "hopf_coordinates": _rounded_hopf_coordinates(sector_info["coordinates"]),
        "sector_id": sector_info["sector_id"],
        "sector_bins": sector_info["sector_bins"],
    }
    if normalized_router_variant == "angular-hopf-trans":
        plan["phase_transport_lambda"] = CANONICAL_ANGULAR_PHASE_TRANSPORT_LAMBDA
        plan["hopf_chi_bins"] = CANONICAL_ANGULAR_HOPF_CHI_BINS
    return plan


def _normalize_canonical_variant(router_variant: str) -> str:
    if not isinstance(router_variant, str) or router_variant not in SUPPORTED_CANONICAL_ANGULAR_VARIANTS:
        raise ValueError(
            "router_variant must be one of: "
            + ", ".join(SUPPORTED_CANONICAL_ANGULAR_VARIANTS)
        )
    return router_variant


def _allocate_triplet_bins_budget(
    total_cap: int,
    *,
    min_first: int = 2,
    min_second: int = 1,
    min_third: int = 1,
) -> tuple[int, int, int]:
    total_cap = max(1, int(total_cap))
    min_first = max(1, int(min_first))
    min_second = max(1, int(min_second))
    min_third = max(1, int(min_third))
    if min_first * min_second * min_third > total_cap:
        min_third = 1
    if min_first * min_second * min_third > total_cap:
        min_second = 1
    if min_first * min_second * min_third > total_cap:
        min_first = 1
    best = (1, total_cap, 1)
    best_score: tuple[int, int, int, int, int] | None = None
    for k_first in range(min_first, total_cap + 1):
        for k_second in range(min_second, total_cap + 1):
            max_third = total_cap // max(k_first * k_second, 1)
            if max_third < min_third:
                break
            for k_third in range(min_third, max_third + 1):
                product = k_first * k_second * k_third
                favor_base = 1 if k_second >= k_third else 0
                spread = (
                    abs(k_first - k_second)
                    + abs(k_second - k_third)
                    + abs(k_first - k_third)
                )
                score = (product, favor_base, -spread, k_second, -k_third)
                if best_score is None or score > best_score:
                    best_score = score
                    best = (k_first, k_second, k_third)
    return best


def _wrap_to_pi(theta: float) -> float:
    return (theta + math.pi) % (2.0 * math.pi) - math.pi


def _hopf_coordinate_components(
    normalized_coordinate: Sequence[float],
) -> dict[str, float]:
    a, b, c, d = (float(value) for value in normalized_coordinate)
    rho1 = math.sqrt((a * a) + (b * b))
    rho2 = math.sqrt((c * c) + (d * d))
    denom = max(math.sqrt((rho1 * rho1) + (rho2 * rho2)), 1e-12)
    cos_chi = rho1 / denom
    sin_chi = rho2 / denom
    chi_u = min(max(sin_chi * sin_chi, 0.0), 1.0 - 1e-12)
    chi = math.asin(min(max(sin_chi, 0.0), 1.0))
    theta1 = _wrap_to_pi(math.atan2(b, a))
    theta2 = _wrap_to_pi(math.atan2(d, c))
    delta = _wrap_to_pi(theta1 - theta2)
    alpha = _wrap_to_pi(0.5 * (theta1 + theta2))
    return {
        "rho1": rho1,
        "rho2": rho2,
        "chi": chi,
        "chi_u": chi_u,
        "theta1": theta1,
        "theta2": theta2,
        "delta": delta,
        "alpha": alpha,
        "cos_chi": cos_chi,
        "sin_chi": sin_chi,
    }


def _hopf_phase_transport_components(
    normalized_coordinate: Sequence[float],
    *,
    phase_transport_lambda: float,
) -> dict[str, float]:
    components = _hopf_coordinate_components(normalized_coordinate)
    chi = components["chi"]
    delta = components["delta"]
    alpha = components["alpha"]
    connection_weight = 0.5 * float(phase_transport_lambda) * math.cos(2.0 * chi)
    phase_shift = _wrap_to_pi(connection_weight * delta)
    transported_alpha = _wrap_to_pi(alpha + phase_shift)
    return {
        **components,
        "transport_connection_weight": connection_weight,
        "transport_phase_shift": phase_shift,
        "transported_alpha": transported_alpha,
    }


def _assign_sector_angular_hopf_base(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
) -> dict[str, object]:
    components = _hopf_coordinate_components(normalized_coordinate)
    kchi = max(1, int(math.floor(math.sqrt(max(int(K), 1)))))
    kdelta = max(1, int(math.ceil(float(max(int(K), 1)) / float(kchi))))
    u_chi = components["chi_u"]
    u_delta = (components["delta"] + math.pi) / (2.0 * math.pi)
    chi_bin = min(int(u_chi * float(kchi)), max(kchi - 1, 0))
    delta_bin = min(int(u_delta * float(kdelta)), max(kdelta - 1, 0))
    sector_id = (chi_bin * kdelta + delta_bin) % max(int(K), 1)
    return {
        "coordinates": components,
        "sector_id": int(sector_id),
        "sector_bins": {
            "chi_bins": kchi,
            "delta_bins": kdelta,
            "chi_bin": chi_bin,
            "delta_bin": delta_bin,
        },
    }


def _assign_sector_angular_hopf_trans(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
    phase_transport_lambda: float,
    hopf_chi_bins: int,
) -> dict[str, object]:
    components = _hopf_phase_transport_components(
        normalized_coordinate,
        phase_transport_lambda=phase_transport_lambda,
    )
    kchi, kdelta, kalpha = _allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )
    u_chi = components["chi_u"]
    u_delta = (components["delta"] + math.pi) / (2.0 * math.pi)
    u_alpha = (components["transported_alpha"] + math.pi) / (2.0 * math.pi)
    chi_bin = min(int(u_chi * float(kchi)), max(kchi - 1, 0))
    delta_bin = min(int(u_delta * float(kdelta)), max(kdelta - 1, 0))
    alpha_bin = min(int(u_alpha * float(kalpha)), max(kalpha - 1, 0))
    local_span = max(kdelta * kalpha, 1)
    sector_id = min((chi_bin * local_span) + (delta_bin * kalpha) + alpha_bin, max(int(K) - 1, 0))
    return {
        "coordinates": components,
        "sector_id": int(sector_id),
        "sector_bins": {
            "chi_bins": kchi,
            "delta_bins": kdelta,
            "alpha_bins": kalpha,
            "chi_bin": chi_bin,
            "delta_bin": delta_bin,
            "alpha_bin": alpha_bin,
        },
    }


def _bundle_tiebreak_rank(
    bundle_name: str,
    *,
    router_variant: str,
    sector_info: Mapping[str, object],
) -> int:
    if router_variant != "angular-hopf-trans":
        return _BASE_BUNDLE_ORDER.index(bundle_name)
    sector_bins = sector_info.get("sector_bins", {})
    if not isinstance(sector_bins, Mapping):
        return _BASE_BUNDLE_ORDER.index(bundle_name)
    alpha_bin = int(sector_bins.get("alpha_bin", 0))
    if alpha_bin % 2 == 1:
        rotated_order = (
            "action_local",
            "local_objective",
            "shared_state",
            "temporal_world",
        )
        return rotated_order.index(bundle_name)
    return _BASE_BUNDLE_ORDER.index(bundle_name)


def _expand_shared_state_layers(
    *,
    router_variant: str,
    sector_info: Mapping[str, object],
    persistence_score: int,
    multi_agent_score: int,
) -> tuple[str, str]:
    if router_variant == "angular-hopf-trans":
        sector_bins = sector_info.get("sector_bins", {})
        alpha_bin = int(sector_bins.get("alpha_bin", 0)) if isinstance(sector_bins, Mapping) else 0
        if alpha_bin % 2 == 1:
            return ("multi_agent", "persistence")
    if multi_agent_score > persistence_score:
        return ("multi_agent", "persistence")
    return ("persistence", "multi_agent")


def _expand_prompt_section_order(
    ordered_bundles: Sequence[str],
    *,
    shared_state_layers: Sequence[str],
) -> list[str]:
    section_order: list[str] = []
    for bundle_name in ordered_bundles:
        if bundle_name == "action_local":
            section_order.append("immediate_action")
        elif bundle_name == "local_objective":
            section_order.append("local_objective")
        elif bundle_name == "temporal_world":
            section_order.append("temporal_world")
        elif bundle_name == "shared_state":
            section_order.extend(shared_state_layers)
        else:
            raise ValueError(f"unsupported angular bundle: {bundle_name}")
    return section_order


def _build_compressed_sections(
    ordered_bundles: Sequence[str],
    *,
    shared_state_layers: Sequence[str],
    router_variant: str,
) -> list[str]:
    uncompressed_bundle_count = 2 if router_variant == "angular-hopf-base" else 3
    compressed: list[str] = []
    for bundle_name in ordered_bundles[uncompressed_bundle_count:]:
        if bundle_name == "shared_state":
            compressed.extend(shared_state_layers)
        elif bundle_name == "action_local":
            compressed.append("immediate_action")
        else:
            compressed.append(bundle_name)
    return compressed


def _rounded_hopf_coordinates(coordinates: Mapping[str, float]) -> dict[str, float]:
    rounded_keys = (
        "chi",
        "chi_u",
        "delta",
        "alpha",
        "transported_alpha",
        "transport_phase_shift",
        "transport_connection_weight",
    )
    return {
        key: round(float(coordinates[key]), 6)
        for key in rounded_keys
        if key in coordinates
    }


def _reasoning_pressure_tag(dominant_section: str, *, router_variant: str) -> str:
    phase_prefix = {
        "angular-hopf-base": "angular_hopf_base",
        "angular-hopf-trans": "angular_hopf_trans",
    }[router_variant]
    loop_suffix = {
        "immediate_action": "execution_pressure",
        "local_objective": "objective_pressure",
        "temporal_world": "temporal_pressure",
        "multi_agent": "coordination_pressure",
        "persistence": "state_carryover_pressure",
    }[dominant_section]
    return f"{phase_prefix}:{loop_suffix}"
