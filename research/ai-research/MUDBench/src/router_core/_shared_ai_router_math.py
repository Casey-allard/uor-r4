"""Thin import shim for ai-router scalar math helpers."""

from __future__ import annotations

import sys
from pathlib import Path


_AI_ROUTER_RESEARCH_PATH = Path(__file__).resolve().parents[3] / "ai-router" / "router-research"
if str(_AI_ROUTER_RESEARCH_PATH) not in sys.path:
    sys.path.insert(0, str(_AI_ROUTER_RESEARCH_PATH))

from router_math_contract import allocate_pair_bins_scalar
from router_math_contract import allocate_triplet_bins_budget
from router_math_contract import assign_sector_hopf_base_scalar
from router_math_contract import assign_sector_hopf_transport_scalar
from router_math_contract import fibonacci_values_upto
from router_math_contract import hopf_coordinate_components_scalar
from router_math_contract import hopf_phase_transport_components_scalar
from router_math_contract import normalize_4d_coordinate


__all__ = [
    "allocate_pair_bins_scalar",
    "allocate_triplet_bins_budget",
    "assign_sector_hopf_base_scalar",
    "assign_sector_hopf_transport_scalar",
    "fibonacci_values_upto",
    "hopf_coordinate_components_scalar",
    "hopf_phase_transport_components_scalar",
    "normalize_4d_coordinate",
]
