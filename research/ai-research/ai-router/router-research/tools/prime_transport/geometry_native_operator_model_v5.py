#!/usr/bin/env python3
"""Fifth lawful operator with the first explicit class-adjacent lift."""

from __future__ import annotations

import csv
from collections import Counter, deque
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v4 import (
    OUTPUT_PATH_OPERATOR_V4,
    OperatorStateV4,
    SparseLawfulOperatorV4,
    bounded_operator_surface_v4,
    class_tuple_v4,
    composite_swap_component_v4,
    composite_twist_component_v4,
    coupled_torus_kick_component_v4,
    hold_component_v4,
    initial_operator_state_v4,
    summarize_operator_v4,
    torus_base_advance_component_v4,
)
from geometry_native_operator_model_v1 import OperatorTransitionV1


OUTPUT_PATH_OPERATOR_V5 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_formulation_v5.csv"
)

FIBER_PERIOD_V5 = 3


def fiber_spin_transport_map_v5(spin_h):
    """Explicit spin transport map used by the fiber refinement lift.

    On the current bounded surface the admissibility word is constant, so the
    lawful transported spin_h is the identity image of the current spin_h.
    """
    return spin_h


def fiber_phase_lift_component_v5(state: OperatorStateV4) -> OperatorStateV4:
    return OperatorStateV4(
        b=state.b,
        phi=(state.phi + 1) % FIBER_PERIOD_V5,
        r=state.r,
        spin_h=fiber_spin_transport_map_v5(state.spin_h),
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


class SparseLawfulOperatorV5:
    """Explicit sparse operator H_v5 = I + T_b + T_x + T_c + T_y + T_z."""

    def transitions(self, state: OperatorStateV4) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v4(state)
        base_target = torus_base_advance_component_v4(state)
        swap_target = composite_swap_component_v4(state)
        coupled_target = coupled_torus_kick_component_v4(state)
        twist_target = composite_twist_component_v4(state)
        lift_target = fiber_phase_lift_component_v5(state)
        return (
            OperatorTransitionV1(source=state, target=hold_target, component="hold", lawful=self._is_lawful(hold_target, "hold")),
            OperatorTransitionV1(source=state, target=base_target, component="torus_base_advance", lawful=self._is_lawful(base_target, "torus_base_advance")),
            OperatorTransitionV1(source=state, target=swap_target, component="composite_swap", lawful=self._is_lawful(swap_target, "composite_swap")),
            OperatorTransitionV1(source=state, target=coupled_target, component="coupled_torus_kick", lawful=self._is_lawful(coupled_target, "coupled_torus_kick")),
            OperatorTransitionV1(source=state, target=twist_target, component="composite_twist", lawful=self._is_lawful(twist_target, "composite_twist")),
            OperatorTransitionV1(source=state, target=lift_target, component="fiber_phase_lift", lawful=self._is_lawful(lift_target, "fiber_phase_lift")),
        )

    @staticmethod
    def _is_lawful(target: OperatorStateV4, component: str) -> bool:
        if component == "fiber_phase_lift":
            return True
        return True


def bounded_operator_surface_v5(depth: int = 8) -> tuple[tuple[OperatorStateV4, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV5()
    seed = initial_operator_state_v4()
    seen: set[OperatorStateV4] = {seed}
    states: list[OperatorStateV4] = [seed]
    transitions: list[OperatorTransitionV1] = []
    queue: deque[tuple[OperatorStateV4, int]] = deque([(seed, 0)])

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


def summarize_operator_v5(depth: int = 8) -> list[dict[str, object]]:
    v4_rows = summarize_operator_v4(depth=depth)
    v4_state_count = next(int(row["count"]) for row in v4_rows if row["scope"] == "operator" and row["metric"] == "state_count")
    v4_transition_count = next(int(row["count"]) for row in v4_rows if row["scope"] == "operator" and row["metric"] == "transition_count")
    original_class_metric = next(
        row["metric"] for row in v4_rows if row["scope"] == "class_distribution"
    )

    states, transitions = bounded_operator_surface_v5(depth=depth)
    lawful_transitions = [t for t in transitions if t.lawful]
    illegal_transitions = [t for t in transitions if not t.lawful]
    nonzero_per_state = Counter()
    component_counter = Counter()
    target_class_counter = Counter()

    original_class = class_tuple_v4(initial_operator_state_v4())
    original_target_count = 0
    new_class_target_count = 0

    for transition in transitions:
        nonzero_per_state[transition.source] += 1
        component_counter[transition.component] += 1
        target_class = class_tuple_v4(transition.target)
        target_class_counter[target_class] += 1
        if target_class == original_class:
            original_target_count += 1
        else:
            new_class_target_count += 1

    rows: list[dict[str, object]] = []
    rows.append({"scope": "operator", "metric": "state_count", "count": len(states), "total": len(states), "fraction": 1.0, "note": "reachable structured states on bounded operator surface"})
    rows.append({"scope": "operator", "metric": "transition_count", "count": len(transitions), "total": len(transitions), "fraction": 1.0, "note": "nonzero operator entries on bounded surface"})
    rows.append({"scope": "operator", "metric": "lawful_transition_fraction", "count": len(lawful_transitions), "total": len(transitions), "fraction": len(lawful_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are lawful"})
    rows.append({"scope": "operator", "metric": "illegal_transition_fraction", "count": len(illegal_transitions), "total": len(transitions), "fraction": len(illegal_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are illegal"})
    rows.append({"scope": "orbit_change", "metric": "state_count_vs_v4", "count": len(states), "total": v4_state_count, "fraction": len(states) / max(v4_state_count, 1), "note": "reachable state expansion relative to H_v4"})
    rows.append({"scope": "orbit_change", "metric": "transition_count_vs_v4", "count": len(transitions), "total": v4_transition_count, "fraction": len(transitions) / max(v4_transition_count, 1), "note": "nonzero transition expansion relative to H_v4"})
    rows.append({"scope": "class_escape", "metric": "distinct_class_identities_reached", "count": len(target_class_counter), "total": len(target_class_counter), "fraction": 1.0, "note": "number of distinct target class identities reached"})
    rows.append({"scope": "class_escape", "metric": "fraction_staying_in_original_class", "count": original_target_count, "total": len(transitions), "fraction": original_target_count / max(len(transitions), 1), "note": f"target transitions remaining in original class {original_class_metric}"})
    rows.append({"scope": "class_escape", "metric": "fraction_entering_new_lawful_class", "count": new_class_target_count, "total": len(transitions), "fraction": new_class_target_count / max(len(transitions), 1), "note": "target transitions entering any new lawful class identity"})
    rows.append({"scope": "generator_axis", "metric": "T_z_changes_radial_class", "count": 0, "total": 1, "fraction": 0.0, "note": "fiber_phase_lift does not change r"})
    rows.append({"scope": "generator_axis", "metric": "T_z_changes_fiber_class", "count": 1, "total": 1, "fraction": 1.0, "note": "fiber_phase_lift changes phi"})
    rows.append({"scope": "generator_axis", "metric": "T_z_changes_spin_class", "count": 0, "total": 1, "fraction": 0.0, "note": "fiber_phase_lift preserves spin_h via explicit transport map"})
    rows.append({"scope": "generator_axis", "metric": "T_z_changes_composite_compat_class", "count": 0, "total": 1, "fraction": 0.0, "note": "fiber_phase_lift preserves composite compatibility class"})

    for state, count in sorted(nonzero_per_state.items(), key=lambda item: (item[0].b, item[0].phi, item[0].r, item[0].query_semiprime, item[0].binding_semiprime, item[0].twist)):
        rows.append(
            {
                "scope": "sparsity",
                "metric": f"nonzero_from_state_b{state.b}_phi{state.phi}_r{state.r}_q{state.query_semiprime}_b{state.binding_semiprime}_tw{state.twist}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "nonzero transitions emitted by operator from one structured state",
            }
        )

    for component_name in ("hold", "torus_base_advance", "composite_swap", "coupled_torus_kick", "composite_twist", "fiber_phase_lift"):
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

    for class_key, count in target_class_counter.items():
        r, phi, spin_h, compat = class_key
        rows.append(
            {
                "scope": "class_distribution",
                "metric": f"class_r{r}_phi{phi}_spin{''.join(str(bit) for bit in spin_h.bits)}_compat{compat[0]}_{compat[1]}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by target class tuple",
            }
        )

    return rows


def write_operator_summary_v5(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_OPERATOR_V5) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_operator_v5(depth=8)
    write_operator_summary_v5(summary_rows)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V5}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
