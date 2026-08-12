#!/usr/bin/env python3
"""Guarded non-runtime adapter for the base_gap prime-transport routing policy."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Iterable

from mock_router_module import (
    DEFAULT_PROMOTION_THRESHOLD,
    FullState,
    MinimalState,
    RouterState,
    initialize_state,
    promote_state,
    score_or_route,
    should_promote,
    update_state,
)


@dataclass
class BaseGapRoutingAdapter:
    """Research-side adapter for the current base_gap routing policy."""

    promotion_threshold: float = DEFAULT_PROMOTION_THRESHOLD
    route_policy_cache: dict[str, bool] = field(default_factory=dict)


def initialize_adapter(*, promotion_threshold: float = DEFAULT_PROMOTION_THRESHOLD) -> BaseGapRoutingAdapter:
    return BaseGapRoutingAdapter(promotion_threshold=float(promotion_threshold))


def initialize_entry(
    *,
    b: int,
    phi: Iterable[int],
    r: int,
    next_return_gap: int,
) -> MinimalState:
    state = initialize_state(mode="R_min", b=b, phi=phi, r=r, next_return_gap=next_return_gap)
    if not isinstance(state, MinimalState):
        raise TypeError("initialize_entry must produce MinimalState")
    return state


def step_entry(
    state: RouterState,
    *,
    fiber_moduli: Iterable[int] = (),
    next_return_gap: int | None = None,
    next_bit: int | None = None,
) -> RouterState:
    return update_state(
        state,
        fiber_moduli=fiber_moduli,
        next_return_gap=next_return_gap,
        next_bit=next_bit,
    )


def base_gap_route_key(state: MinimalState) -> str:
    return f"base_gap:r={state.r}:b={state.b}:gap={state.next_return_gap}"


def route_decision(
    adapter: BaseGapRoutingAdapter,
    state: MinimalState,
    *,
    unresolved_fraction: float,
) -> dict[str, object]:
    route_key = base_gap_route_key(state)
    was_cached = route_key in adapter.route_policy_cache
    if not was_cached:
        adapter.route_policy_cache[route_key] = should_promote(
            state,
            unresolved_fraction=float(unresolved_fraction),
            threshold=adapter.promotion_threshold,
        )
    promote = adapter.route_policy_cache[route_key]
    return {
        "route_key": route_key,
        "promote": promote,
        "cached_decision": was_cached,
        "score_hint": 0.75 if not promote else 1.0,
    }


def promotion_fallback(state: MinimalState, *, spin_H: Iterable[int]) -> FullState:
    promoted = promote_state(state, spin_H=spin_H)
    if not isinstance(promoted, FullState):
        raise TypeError("promotion_fallback must produce FullState")
    return promoted


def adapter_output(
    adapter: BaseGapRoutingAdapter,
    state: RouterState,
    *,
    unresolved_fraction: float = 0.0,
    spin_H: Iterable[int] | None = None,
) -> dict[str, object]:
    if isinstance(state, FullState):
        return {
            "route_mode": state.mode,
            "route_key": score_or_route(state)["route_key"],
            "promoted": False,
            "state": state,
        }

    decision = route_decision(adapter, state, unresolved_fraction=unresolved_fraction)
    if decision["promote"]:
        if spin_H is None:
            raise ValueError("spin_H is required when promotion is triggered")
        promoted_state = promotion_fallback(state, spin_H=spin_H)
        return {
            "route_mode": promoted_state.mode,
            "route_key": score_or_route(promoted_state)["route_key"],
            "promoted": True,
            "state": promoted_state,
        }

    return {
        "route_mode": state.mode,
        "route_key": decision["route_key"],
        "promoted": False,
        "state": state,
    }
