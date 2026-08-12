#!/usr/bin/env python3
"""Minimal offline/mock router module for prime-transport routing states."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Iterable


BASE_PHASE_MODULUS = 35
DEFAULT_PROMOTION_THRESHOLD = 0.30


@dataclass(frozen=True)
class StaticState:
    b: int
    phi: tuple[int, ...]
    r: int
    mode: str = "R_static"


@dataclass(frozen=True)
class MinimalState:
    b: int
    phi: tuple[int, ...]
    r: int
    next_return_gap: int
    mode: str = "R_min"


@dataclass(frozen=True)
class FullState:
    b: int
    spin_H: tuple[int, ...]
    mode: str = "R_full"


RouterState = StaticState | MinimalState | FullState


def _step_phi(phi: tuple[int, ...], fiber_moduli: tuple[int, ...]) -> tuple[int, ...]:
    if not phi:
        return ()
    if not fiber_moduli or len(phi) != len(fiber_moduli):
        return phi
    stepped = []
    for value, modulus in zip(phi, fiber_moduli):
        if modulus <= 0:
            stepped.append(value)
            continue
        stepped.append((value + 1) % modulus)
    return tuple(stepped)


def initialize_state(
    *,
    mode: str = "R_min",
    b: int,
    phi: Iterable[int] = (),
    r: int,
    next_return_gap: int | None = None,
    spin_H: Iterable[int] | None = None,
) -> RouterState:
    phi_tuple = tuple(int(v) for v in phi)
    base_phase = int(b) % BASE_PHASE_MODULUS
    if mode == "R_static":
        return StaticState(b=base_phase, phi=phi_tuple, r=int(r))
    if mode == "R_min":
        if next_return_gap is None:
            raise ValueError("next_return_gap is required for R_min")
        return MinimalState(
            b=base_phase,
            phi=phi_tuple,
            r=int(r),
            next_return_gap=max(0, int(next_return_gap)),
        )
    if mode == "R_full":
        if spin_H is None:
            raise ValueError("spin_H is required for R_full")
        return FullState(
            b=base_phase,
            spin_H=tuple(int(v) for v in spin_H),
        )
    raise ValueError(f"unknown mode: {mode}")


def update_state(
    state: RouterState,
    *,
    fiber_moduli: Iterable[int] = (),
    next_bit: int | None = None,
    next_return_gap: int | None = None,
) -> RouterState:
    fiber_moduli_tuple = tuple(int(v) for v in fiber_moduli)
    if isinstance(state, StaticState):
        next_b = (state.b + 1) % BASE_PHASE_MODULUS
        return StaticState(
            b=next_b,
            phi=_step_phi(state.phi, fiber_moduli_tuple) if next_b == 0 else state.phi,
            r=state.r,
        )
    if isinstance(state, MinimalState):
        next_b = (state.b + 1) % BASE_PHASE_MODULUS
        return MinimalState(
            b=next_b,
            phi=_step_phi(state.phi, fiber_moduli_tuple) if next_b == 0 else state.phi,
            r=state.r,
            next_return_gap=max(0, int(next_return_gap)) if next_return_gap is not None else max(0, state.next_return_gap - 1),
        )
    if next_bit is None:
        raise ValueError("next_bit is required to update R_full")
    shifted = state.spin_H[1:] + (int(next_bit),) if state.spin_H else (int(next_bit),)
    return FullState(
        b=(state.b + 1) % BASE_PHASE_MODULUS,
        spin_H=shifted,
    )


def score_or_route(state: RouterState) -> dict[str, object]:
    if isinstance(state, StaticState):
        return {
            "route_mode": state.mode,
            "route_key": f"static:b={state.b}:phi={state.phi}:r={state.r}",
            "score_hint": 0.25,
        }
    if isinstance(state, MinimalState):
        return {
            "route_mode": state.mode,
            "route_key": f"min:b={state.b}:phi={state.phi}:r={state.r}:gap={state.next_return_gap}",
            "score_hint": 0.75,
        }
    return {
        "route_mode": state.mode,
        "route_key": f"full:b={state.b}:spin={state.spin_H}",
        "score_hint": 1.0,
    }


def should_promote(
    state: RouterState,
    *,
    unresolved_fraction: float,
    threshold: float = DEFAULT_PROMOTION_THRESHOLD,
) -> bool:
    if not isinstance(state, MinimalState):
        return False
    return float(unresolved_fraction) > float(threshold)


def promote_state(
    state: RouterState,
    *,
    spin_H: Iterable[int],
) -> RouterState:
    if not isinstance(state, MinimalState):
        raise ValueError("promotion currently only defined from R_min to R_full")
    return FullState(
        b=state.b,
        spin_H=tuple(int(v) for v in spin_H),
    )
