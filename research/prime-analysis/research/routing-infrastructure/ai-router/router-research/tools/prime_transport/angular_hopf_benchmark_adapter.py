#!/usr/bin/env python3
"""Research-only adapter for canonical angular Hopf routing on prime traces."""

from __future__ import annotations

import math
import sys
from pathlib import Path
from typing import Sequence


ROOT = Path(__file__).resolve().parents[2]
AI_RESEARCH_ROOT = ROOT.parents[1]
MUDBENCH_SRC = AI_RESEARCH_ROOT / "MUDBench" / "src"
if str(MUDBENCH_SRC) not in sys.path:
    sys.path.insert(0, str(MUDBENCH_SRC))

from router_core.canonical_angular_router import (  # noqa: E402
    DEFAULT_CANONICAL_ANGULAR_VARIANT,
    build_canonical_angular_prompt_plan,
)


_NEUTRAL_BUNDLE_SCORES = {
    "action_local": 1,
    "local_objective": 1,
    "temporal_world": 1,
    "shared_state": 1,
}


def _angle_pair(index: int, modulus: int) -> tuple[float, float]:
    safe_modulus = max(1, int(modulus))
    theta = (2.0 * math.pi * float(index % safe_modulus)) / float(safe_modulus)
    return math.cos(theta), math.sin(theta)


def routing_coordinate_from_trace_entry(
    *,
    base_phase: int,
    phase_tuple: Sequence[int],
    layer_primes: Sequence[int],
    next_return_gap: int,
) -> tuple[float, float, float, float]:
    """Project one exact trace position into the canonical 4D angular contract.

    The first planar pair is the exact base phase on the 35-cycle.
    The second planar pair uses the first fiber phase when a fiber exists;
    otherwise it falls back to the bounded next-return-gap cycle so the
    canonical angular router still receives two deterministic angular pairs.
    """

    first_pair = _angle_pair(base_phase, 35)
    if layer_primes:
        second_pair = _angle_pair(int(phase_tuple[0]), int(layer_primes[0]))
    else:
        second_pair = _angle_pair(int(next_return_gap), max(2, int(next_return_gap) + 1))
    return first_pair + second_pair


def build_angular_route_plan_for_trace_entry(
    *,
    base_phase: int,
    phase_tuple: Sequence[int],
    layer_primes: Sequence[int],
    next_return_gap: int,
    router_variant: str = DEFAULT_CANONICAL_ANGULAR_VARIANT,
) -> dict[str, object]:
    routing_coordinate = routing_coordinate_from_trace_entry(
        base_phase=base_phase,
        phase_tuple=phase_tuple,
        layer_primes=layer_primes,
        next_return_gap=next_return_gap,
    )
    return build_canonical_angular_prompt_plan(
        routing_coordinate=routing_coordinate,
        bundle_scores=_NEUTRAL_BUNDLE_SCORES,
        persistence_score=1,
        multi_agent_score=1,
        router_variant=router_variant,
    )


def angular_route_key_for_trace_entry(
    *,
    r_depth: int,
    base_phase: int,
    phase_tuple: Sequence[int],
    layer_primes: Sequence[int],
    next_return_gap: int,
    router_variant: str = DEFAULT_CANONICAL_ANGULAR_VARIANT,
) -> str:
    plan = build_angular_route_plan_for_trace_entry(
        base_phase=base_phase,
        phase_tuple=phase_tuple,
        layer_primes=layer_primes,
        next_return_gap=next_return_gap,
        router_variant=router_variant,
    )
    return (
        f"{router_variant}:r={int(r_depth)}:"
        f"sector={int(plan['sector_id'])}"
    )
