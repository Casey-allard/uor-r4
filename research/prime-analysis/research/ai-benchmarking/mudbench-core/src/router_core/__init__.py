"""Vendored router cores for prompt-layer routing.

Derived from the pure mathematical helpers in:
`~/ai-router/router-research/hyperbolic_router_so8.py`
"""

from router_core.canonical_angular_router import (
    CANONICAL_ANGULAR_COORDINATE_LABELS,
    CANONICAL_ANGULAR_HOPF_CHI_BINS,
    CANONICAL_ANGULAR_PHASE_TRANSPORT_LAMBDA,
    CANONICAL_ANGULAR_ROUTER_K,
    DEFAULT_CANONICAL_ANGULAR_VARIANT,
    SUPPORTED_CANONICAL_ANGULAR_VARIANTS,
    build_canonical_angular_prompt_plan,
    normalize_4d_coordinate,
)
from router_core.prompt_router import (
    LOG_PHI,
    PHI,
    SUPPORTED_LEGACY_ROUTER_VARIANTS,
    SUPPORTED_ROUTER_VARIANTS,
    build_legacy_router_backed_prompt_plan,
    build_router_backed_prompt_plan,
    fibonacci_values_upto,
)

__all__ = [
    "CANONICAL_ANGULAR_COORDINATE_LABELS",
    "CANONICAL_ANGULAR_HOPF_CHI_BINS",
    "CANONICAL_ANGULAR_PHASE_TRANSPORT_LAMBDA",
    "CANONICAL_ANGULAR_ROUTER_K",
    "DEFAULT_CANONICAL_ANGULAR_VARIANT",
    "LOG_PHI",
    "PHI",
    "SUPPORTED_CANONICAL_ANGULAR_VARIANTS",
    "SUPPORTED_LEGACY_ROUTER_VARIANTS",
    "SUPPORTED_ROUTER_VARIANTS",
    "build_canonical_angular_prompt_plan",
    "build_legacy_router_backed_prompt_plan",
    "build_router_backed_prompt_plan",
    "fibonacci_values_upto",
    "normalize_4d_coordinate",
]
