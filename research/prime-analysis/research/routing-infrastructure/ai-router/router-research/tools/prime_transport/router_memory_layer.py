#!/usr/bin/env python3
"""Research-only structured read/write memory layer for prime-transport routing."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass, field
from typing import Iterable

from base_gap_routing_adapter import (
    BaseGapRoutingAdapter,
    base_gap_route_key,
    initialize_adapter,
    initialize_entry,
    promotion_fallback,
    route_decision,
    step_entry,
)
from mock_router_module import FullState, MinimalState, RouterState


@dataclass
class MemorySlot:
    """Persistent memory cell attached to one reusable routing partition."""

    route_key: str
    promoted: bool = False
    unresolved_fraction: float = 0.0
    write_count: int = 0
    read_count: int = 0
    next_bit_counts: Counter[int] = field(default_factory=Counter)
    full_key_counts: Counter[str] = field(default_factory=Counter)
    latest_payload: str | None = None
    payload_counts: Counter[str] = field(default_factory=Counter)
    payload_by_full_key: dict[str, str] = field(default_factory=dict)
    payload_counts_by_hint: dict[str, Counter[str]] = field(default_factory=dict)

    def dominant_full_key(self) -> str | None:
        if not self.full_key_counts:
            return None
        return self.full_key_counts.most_common(1)[0][0]

    def dominant_confidence(self) -> float:
        if not self.full_key_counts:
            return 0.0
        total = sum(self.full_key_counts.values())
        if total <= 0:
            return 0.0
        return self.full_key_counts.most_common(1)[0][1] / total


@dataclass
class RouterMemoryLayer:
    """Small stateful memory machine built on top of the guarded routing policy."""

    adapter: BaseGapRoutingAdapter
    slots: dict[str, MemorySlot] = field(default_factory=dict)


def initialize_memory_layer(*, promotion_threshold: float | None = None) -> RouterMemoryLayer:
    if promotion_threshold is None:
        return RouterMemoryLayer(adapter=initialize_adapter())
    return RouterMemoryLayer(adapter=initialize_adapter(promotion_threshold=promotion_threshold))


def initialize_memory_state(
    *,
    b: int,
    phi: Iterable[int],
    r: int,
    next_return_gap: int,
) -> MinimalState:
    return initialize_entry(b=b, phi=phi, r=r, next_return_gap=next_return_gap)


def update_memory_state(
    state: RouterState,
    *,
    fiber_moduli: Iterable[int] = (),
    next_return_gap: int | None = None,
    next_bit: int | None = None,
) -> RouterState:
    return step_entry(
        state,
        fiber_moduli=fiber_moduli,
        next_return_gap=next_return_gap,
        next_bit=next_bit,
    )


def query_memory(
    layer: RouterMemoryLayer,
    state: MinimalState,
    *,
    unresolved_fraction: float,
    full_key: str | None = None,
    refinement_hint: str | None = None,
) -> dict[str, object]:
    route_key = base_gap_route_key(state)
    slot = layer.slots.get(route_key)
    if slot is None:
        return {
            "route_key": route_key,
            "read_hit": False,
            "read_mode": "miss",
            "predicted_key": None,
            "predicted_confidence": 0.0,
            "predicted_promote": False,
        }

    slot.read_count += 1
    promote = bool(route_decision(layer.adapter, state, unresolved_fraction=unresolved_fraction)["promote"])
    if promote and slot.promoted and refinement_hint is not None and refinement_hint in slot.payload_counts_by_hint:
        hint_counts = slot.payload_counts_by_hint[refinement_hint]
        total = sum(hint_counts.values())
        payload, count = hint_counts.most_common(1)[0]
        predicted_key = f"{route_key}:hint={refinement_hint}"
        predicted_payload = payload
        predicted_confidence = count / total if total else 0.0
        return {
            "route_key": route_key,
            "read_hit": True,
            "read_mode": "hint_refined",
            "predicted_key": predicted_key,
            "predicted_payload": predicted_payload,
            "predicted_confidence": predicted_confidence,
            "predicted_promote": False,
        }
    if promote and slot.promoted and full_key is not None and full_key in slot.payload_by_full_key:
        predicted_key = full_key
        predicted_payload = slot.payload_by_full_key[full_key]
        predicted_confidence = 1.0
    else:
        predicted_key = slot.dominant_full_key() if slot.full_key_counts else route_key
        predicted_payload = slot.latest_payload
        if promote and slot.promoted:
            predicted_confidence = 1.0 if predicted_payload is not None else 0.0
        else:
            predicted_confidence = slot.dominant_confidence()
    return {
        "route_key": route_key,
        "read_hit": True,
        "read_mode": "promoted_exact" if promote and slot.promoted else "coarse_summary",
        "predicted_key": predicted_key,
        "predicted_payload": predicted_payload,
        "predicted_confidence": predicted_confidence,
        "predicted_promote": promote,
    }


def write_memory(
    layer: RouterMemoryLayer,
    state: MinimalState,
    *,
    unresolved_fraction: float,
    observed_next_bit: int,
    observed_full_key: str,
    spin_H: Iterable[int],
    observed_payload: str | None = None,
    refinement_hint: str | None = None,
) -> dict[str, object]:
    route_key = base_gap_route_key(state)
    slot = layer.slots.setdefault(route_key, MemorySlot(route_key=route_key))
    decision = route_decision(layer.adapter, state, unresolved_fraction=unresolved_fraction)
    promote = bool(decision["promote"])

    slot.promoted = slot.promoted or promote
    slot.unresolved_fraction = max(slot.unresolved_fraction, float(unresolved_fraction))
    slot.write_count += 1
    slot.next_bit_counts[int(observed_next_bit)] += 1
    slot.full_key_counts[str(observed_full_key)] += 1
    payload = str(observed_payload) if observed_payload is not None else str(observed_full_key)
    slot.latest_payload = payload
    slot.payload_counts[payload] += 1
    slot.payload_by_full_key[str(observed_full_key)] = payload
    if refinement_hint is not None:
        slot.payload_counts_by_hint.setdefault(str(refinement_hint), Counter())[payload] += 1

    if promote:
        promoted_state = promotion_fallback(state, spin_H=spin_H)
        return {
            "route_key": route_key,
            "route_mode": promoted_state.mode,
            "emitted_key": f"full:b={promoted_state.b}:spin={promoted_state.spin_H}",
            "stored_payload": payload,
            "promoted": True,
        }

    return {
        "route_key": route_key,
        "route_mode": state.mode,
        "emitted_key": route_key,
        "stored_payload": payload,
        "promoted": False,
    }
