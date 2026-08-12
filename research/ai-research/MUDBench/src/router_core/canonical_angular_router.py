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

from router_core._shared_ai_router_math import allocate_triplet_bins_budget as _shared_allocate_triplet_bins_budget
from router_core._shared_ai_router_math import assign_sector_hopf_base_scalar as _shared_assign_sector_hopf_base_scalar
from router_core._shared_ai_router_math import assign_sector_hopf_transport_scalar as _shared_assign_sector_hopf_transport_scalar
from router_core._shared_ai_router_math import hopf_coordinate_components_scalar as _shared_hopf_coordinate_components_scalar
from router_core._shared_ai_router_math import hopf_phase_transport_components_scalar as _shared_hopf_phase_transport_components_scalar
from router_core._shared_ai_router_math import normalize_4d_coordinate as _shared_normalize_4d_coordinate

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
    return _shared_normalize_4d_coordinate(routing_coordinate)


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
    return _shared_allocate_triplet_bins_budget(
        total_cap,
        min_first=min_first,
        min_second=min_second,
        min_third=min_third,
    )


def _wrap_to_pi(theta: float) -> float:
    return (theta + math.pi) % (2.0 * math.pi) - math.pi


def _hopf_coordinate_components(
    normalized_coordinate: Sequence[float],
) -> dict[str, float]:
    return _shared_hopf_coordinate_components_scalar(normalized_coordinate)


def _hopf_phase_transport_components(
    normalized_coordinate: Sequence[float],
    *,
    phase_transport_lambda: float,
) -> dict[str, float]:
    return _shared_hopf_phase_transport_components_scalar(
        normalized_coordinate,
        phase_transport_lambda=phase_transport_lambda,
    )


def _assign_sector_angular_hopf_base(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
) -> dict[str, object]:
    return _shared_assign_sector_hopf_base_scalar(
        normalized_coordinate,
        K=K,
    )


def _assign_sector_angular_hopf_trans(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
    phase_transport_lambda: float,
    hopf_chi_bins: int,
) -> dict[str, object]:
    return _shared_assign_sector_hopf_transport_scalar(
        normalized_coordinate,
        K=K,
        phase_transport_lambda=phase_transport_lambda,
        hopf_chi_bins=hopf_chi_bins,
    )


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
