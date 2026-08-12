#!/usr/bin/env python3
"""First explicit sparse lawful evolution operator over the torus-fiber state space."""

from __future__ import annotations

import csv
from collections import Counter, deque
from dataclasses import dataclass
from pathlib import Path


OUTPUT_PATH_OPERATOR_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_formulation_v1.csv"
)

BASE_PERIOD_V1 = 5
SPIN_H_HORIZON_V1 = 4


@dataclass(frozen=True)
class NativeSpinHV1:
    horizon: int
    bits: tuple[int, ...]


@dataclass(frozen=True)
class OperatorStateV1:
    b: int
    phi: int
    r: int
    spin_h: NativeSpinHV1
    composite_compat_class: tuple[str, str]
    query_semiprime: int
    binding_semiprime: int
    admissible_transition: int


@dataclass(frozen=True)
class OperatorTransitionV1:
    source: OperatorStateV1
    target: OperatorStateV1
    component: str
    lawful: bool


def _compat_mask_v1(semiprime: int, opposite_semiprime: int) -> str:
    return "".join(
        "1" if semiprime % prime == 0 and opposite_semiprime % prime == 0 else "0"
        for prime in (2, 3, 5)
    )


def _composite_compat_class_v1(query_semiprime: int, binding_semiprime: int) -> tuple[str, str]:
    return (
        _compat_mask_v1(query_semiprime, binding_semiprime),
        _compat_mask_v1(binding_semiprime, query_semiprime),
    )


def _native_spin_h_v1(admissible_transition: int) -> NativeSpinHV1:
    return NativeSpinHV1(
        horizon=SPIN_H_HORIZON_V1,
        bits=tuple(int(admissible_transition) for _ in range(SPIN_H_HORIZON_V1)),
    )


def initial_operator_state_v1() -> OperatorStateV1:
    admissible_transition = 1
    query_semiprime = 6
    binding_semiprime = 10
    return OperatorStateV1(
        b=0,
        phi=0,
        r=0,
        spin_h=_native_spin_h_v1(admissible_transition),
        composite_compat_class=_composite_compat_class_v1(query_semiprime, binding_semiprime),
        query_semiprime=query_semiprime,
        binding_semiprime=binding_semiprime,
        admissible_transition=admissible_transition,
    )


def class_tuple_v1(state: OperatorStateV1) -> tuple[int, int, NativeSpinHV1, tuple[str, str]]:
    return (state.r, state.phi, state.spin_h, state.composite_compat_class)


def hold_component_v1(state: OperatorStateV1) -> OperatorStateV1:
    return state


def torus_base_advance_component_v1(state: OperatorStateV1) -> OperatorStateV1:
    return OperatorStateV1(
        b=(state.b + 1) % BASE_PERIOD_V1,
        phi=state.phi,
        r=state.r,
        spin_h=state.spin_h,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
    )


class SparseLawfulOperatorV1:
    """Explicit sparse operator with only lawful transitions in its support."""

    def transitions(self, state: OperatorStateV1) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v1(state)
        rotate_target = torus_base_advance_component_v1(state)
        return (
            OperatorTransitionV1(
                source=state,
                target=hold_target,
                component="hold",
                lawful=self._is_lawful(state, hold_target),
            ),
            OperatorTransitionV1(
                source=state,
                target=rotate_target,
                component="torus_base_advance",
                lawful=self._is_lawful(state, rotate_target),
            ),
        )

    @staticmethod
    def _is_lawful(source: OperatorStateV1, target: OperatorStateV1) -> bool:
        return class_tuple_v1(source) == class_tuple_v1(target)


def bounded_operator_surface_v1(depth: int = 8) -> tuple[tuple[OperatorStateV1, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV1()
    seed = initial_operator_state_v1()
    seen: set[OperatorStateV1] = {seed}
    states: list[OperatorStateV1] = [seed]
    transitions: list[OperatorTransitionV1] = []
    queue: deque[tuple[OperatorStateV1, int]] = deque([(seed, 0)])

    while queue:
        state, dist = queue.popleft()
        outgoing = operator.transitions(state)
        transitions.extend(outgoing)
        if dist >= depth:
            continue
        for transition in outgoing:
            if transition.target not in seen:
                seen.add(transition.target)
                states.append(transition.target)
                queue.append((transition.target, dist + 1))

    return tuple(states), tuple(transitions)


def summarize_operator_v1(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v1(depth=depth)
    lawful_transitions = [t for t in transitions if t.lawful]
    illegal_transitions = [t for t in transitions if not t.lawful]
    nonzero_per_state = Counter()
    component_counter = Counter()
    class_counter = Counter()

    for transition in transitions:
        nonzero_per_state[transition.source] += 1
        component_counter[transition.component] += 1
        class_counter[class_tuple_v1(transition.source)] += 1

    rows: list[dict[str, object]] = []
    rows.append(
        {
            "scope": "operator",
            "metric": "state_count",
            "count": len(states),
            "total": len(states),
            "fraction": 1.0,
            "note": "reachable structured states on bounded operator surface",
        }
    )
    rows.append(
        {
            "scope": "operator",
            "metric": "transition_count",
            "count": len(transitions),
            "total": len(transitions),
            "fraction": 1.0,
            "note": "nonzero operator entries on bounded surface",
        }
    )
    rows.append(
        {
            "scope": "operator",
            "metric": "lawful_transition_fraction",
            "count": len(lawful_transitions),
            "total": len(transitions),
            "fraction": len(lawful_transitions) / max(len(transitions), 1),
            "note": "fraction of nonzero operator entries that are lawful",
        }
    )
    rows.append(
        {
            "scope": "operator",
            "metric": "illegal_transition_fraction",
            "count": len(illegal_transitions),
            "total": len(transitions),
            "fraction": len(illegal_transitions) / max(len(transitions), 1),
            "note": "fraction of nonzero operator entries that are illegal",
        }
    )

    for state, count in sorted(nonzero_per_state.items(), key=lambda item: (item[0].b, item[0].phi, item[0].r)):
        rows.append(
            {
                "scope": "sparsity",
                "metric": f"nonzero_from_state_b{state.b}_phi{state.phi}_r{state.r}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "nonzero transitions emitted by operator from one structured state",
            }
        )

    for component_name in ("hold", "torus_base_advance"):
        rows.append(
            {
                "scope": "component",
                "metric": component_name,
                "count": component_counter[component_name],
                "total": len(transitions),
                "fraction": component_counter[component_name] / max(len(transitions), 1),
                "note": "fraction of operator entries by component",
            }
        )

    for class_key, count in class_counter.items():
        r, phi, spin_h, compat = class_key
        rows.append(
            {
                "scope": "class_distribution",
                "metric": f"class_r{r}_phi{phi}_spin{''.join(str(bit) for bit in spin_h.bits)}_compat{compat[0]}_{compat[1]}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by source class tuple",
            }
        )

    return rows


def write_operator_summary_v1(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_OPERATOR_V1) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_operator_v1(depth=8)
    write_operator_summary_v1(summary_rows)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V1}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
