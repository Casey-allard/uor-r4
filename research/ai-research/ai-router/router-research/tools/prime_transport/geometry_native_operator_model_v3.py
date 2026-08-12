#!/usr/bin/env python3
"""Third explicit lawful evolution operator with one added coupled generator."""

from __future__ import annotations

import csv
from collections import Counter, deque
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v1 import (
    BASE_PERIOD_V1,
    OperatorStateV1,
    OperatorTransitionV1,
    class_tuple_v1,
    hold_component_v1,
    initial_operator_state_v1,
    torus_base_advance_component_v1,
)
from geometry_native_operator_model_v2 import (
    OUTPUT_PATH_OPERATOR_V2,
    SparseLawfulOperatorV2,
    bounded_operator_surface_v2,
    composite_swap_component_v2,
    summarize_operator_v2,
)


OUTPUT_PATH_OPERATOR_V3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_formulation_v3.csv"
)


def _interaction_step_v3(query_semiprime: int, binding_semiprime: int) -> int:
    gcd_term = __import__("math").gcd(query_semiprime, binding_semiprime)
    return int((query_semiprime + (2 * binding_semiprime) + gcd_term) % BASE_PERIOD_V1)


def coupled_torus_kick_component_v3(state: OperatorStateV1) -> OperatorStateV1:
    interaction_step = _interaction_step_v3(state.query_semiprime, state.binding_semiprime)
    return OperatorStateV1(
        b=(state.b + interaction_step) % BASE_PERIOD_V1,
        phi=state.phi,
        r=state.r,
        spin_h=state.spin_h,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
    )


class SparseLawfulOperatorV3:
    """Explicit sparse operator H_v3 = I + T_b + T_x + T_c."""

    def transitions(self, state: OperatorStateV1) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v1(state)
        base_target = torus_base_advance_component_v1(state)
        swap_target = composite_swap_component_v2(state)
        coupled_target = coupled_torus_kick_component_v3(state)
        return (
            OperatorTransitionV1(
                source=state,
                target=hold_target,
                component="hold",
                lawful=self._is_lawful(state, hold_target),
            ),
            OperatorTransitionV1(
                source=state,
                target=base_target,
                component="torus_base_advance",
                lawful=self._is_lawful(state, base_target),
            ),
            OperatorTransitionV1(
                source=state,
                target=swap_target,
                component="composite_swap",
                lawful=self._is_lawful(state, swap_target),
            ),
            OperatorTransitionV1(
                source=state,
                target=coupled_target,
                component="coupled_torus_kick",
                lawful=self._is_lawful(state, coupled_target),
            ),
        )

    @staticmethod
    def _is_lawful(source: OperatorStateV1, target: OperatorStateV1) -> bool:
        return class_tuple_v1(source) == class_tuple_v1(target)


def bounded_operator_surface_v3(depth: int = 8) -> tuple[tuple[OperatorStateV1, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV3()
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


def summarize_operator_v3(depth: int = 8) -> list[dict[str, object]]:
    v2_rows = summarize_operator_v2(depth=depth)
    v2_state_count = next(int(row["count"]) for row in v2_rows if row["scope"] == "operator" and row["metric"] == "state_count")
    v2_transition_count = next(int(row["count"]) for row in v2_rows if row["scope"] == "operator" and row["metric"] == "transition_count")

    states, transitions = bounded_operator_surface_v3(depth=depth)
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
    rows.append(
        {
            "scope": "orbit_change",
            "metric": "state_count_vs_v2",
            "count": len(states),
            "total": v2_state_count,
            "fraction": len(states) / max(v2_state_count, 1),
            "note": "reachable state expansion relative to H_v2",
        }
    )
    rows.append(
        {
            "scope": "orbit_change",
            "metric": "transition_count_vs_v2",
            "count": len(transitions),
            "total": v2_transition_count,
            "fraction": len(transitions) / max(v2_transition_count, 1),
            "note": "nonzero transition expansion relative to H_v2",
        }
    )

    for state, count in sorted(nonzero_per_state.items(), key=lambda item: (item[0].b, item[0].phi, item[0].r, item[0].query_semiprime, item[0].binding_semiprime)):
        rows.append(
            {
                "scope": "sparsity",
                "metric": f"nonzero_from_state_b{state.b}_phi{state.phi}_r{state.r}_q{state.query_semiprime}_b{state.binding_semiprime}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "nonzero transitions emitted by operator from one structured state",
            }
        )

    for component_name in ("hold", "torus_base_advance", "composite_swap", "coupled_torus_kick"):
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


def write_operator_summary_v3(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_OPERATOR_V3) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_operator_v3(depth=8)
    write_operator_summary_v3(summary_rows)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V3}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
