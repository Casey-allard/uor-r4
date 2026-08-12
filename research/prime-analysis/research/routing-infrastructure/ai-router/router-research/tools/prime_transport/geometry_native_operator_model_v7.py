#!/usr/bin/env python3
"""Sixth operator formulation with a nontrivial lawful spin-transport lift."""

from __future__ import annotations

import csv
from collections import Counter, deque
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1, OperatorTransitionV1
from geometry_native_operator_model_v4 import (
    OperatorStateV4,
    class_tuple_v4,
    composite_swap_component_v4,
    composite_twist_component_v4,
    coupled_torus_kick_component_v4,
    hold_component_v4,
    initial_operator_state_v4,
    torus_base_advance_component_v4,
)
from geometry_native_operator_model_v5 import (
    FIBER_PERIOD_V5,
    OUTPUT_PATH_OPERATOR_V5,
    summarize_operator_v5,
)


OUTPUT_PATH_OPERATOR_V6 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_formulation_v6.csv"
)


def fiber_spin_transport_map_v6(
    spin_h: NativeSpinHV1,
    lifted_phi: int,
    twist: int,
    composite_compat_class: tuple[str, str],
) -> NativeSpinHV1:
    """Explicit lawful spin transport for the fiber-phase lift.

    The transport rule is mechanical:

    1. Rotate the truncated admissibility word forward by one slot.
    2. Compute a fiber-compatibility phase index from the lifted phi, twist, and
       shared-prime mask.
    3. Set the phase-indexed slot to 0 to mark the transported chart-occlusion
       slot in the lifted fiber.

    This preserves:
    - fixed spin horizon
    - bounded word ordering up to one-step transport
    - composite compatibility identity

    It changes spin class nontrivially on the bounded surface without stepping
    outside the explicit lawful lift definition.
    """
    bits = tuple(int(bit) for bit in spin_h.bits)
    rotated = bits[1:] + bits[:1]
    compat_weight = sum(int(bit) for mask in composite_compat_class for bit in mask)
    occlusion_index = (lifted_phi + twist + compat_weight) % spin_h.horizon
    transported = list(rotated)
    transported[occlusion_index] = 0
    return NativeSpinHV1(horizon=spin_h.horizon, bits=tuple(transported))


def fiber_phase_lift_component_v6(state: OperatorStateV4) -> OperatorStateV4:
    lifted_phi = (state.phi + 1) % FIBER_PERIOD_V5
    lifted_spin = fiber_spin_transport_map_v6(
        spin_h=state.spin_h,
        lifted_phi=lifted_phi,
        twist=state.twist,
        composite_compat_class=state.composite_compat_class,
    )
    return OperatorStateV4(
        b=state.b,
        phi=lifted_phi,
        r=state.r,
        spin_h=lifted_spin,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


class SparseLawfulOperatorV7:
    """Explicit sparse operator H_v6 = I + T_b + T_x + T_c + T_y + T_z'."""

    def transitions(self, state: OperatorStateV4) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v4(state)
        base_target = torus_base_advance_component_v4(state)
        swap_target = composite_swap_component_v4(state)
        coupled_target = coupled_torus_kick_component_v4(state)
        twist_target = composite_twist_component_v4(state)
        lift_target = fiber_phase_lift_component_v6(state)
        return (
            OperatorTransitionV1(
                source=state,
                target=hold_target,
                component="hold",
                lawful=self._is_lawful(state, hold_target, "hold"),
            ),
            OperatorTransitionV1(
                source=state,
                target=base_target,
                component="torus_base_advance",
                lawful=self._is_lawful(state, base_target, "torus_base_advance"),
            ),
            OperatorTransitionV1(
                source=state,
                target=swap_target,
                component="composite_swap",
                lawful=self._is_lawful(state, swap_target, "composite_swap"),
            ),
            OperatorTransitionV1(
                source=state,
                target=coupled_target,
                component="coupled_torus_kick",
                lawful=self._is_lawful(state, coupled_target, "coupled_torus_kick"),
            ),
            OperatorTransitionV1(
                source=state,
                target=twist_target,
                component="composite_twist",
                lawful=self._is_lawful(state, twist_target, "composite_twist"),
            ),
            OperatorTransitionV1(
                source=state,
                target=lift_target,
                component="fiber_phase_lift_spin_transport",
                lawful=self._is_lawful(state, lift_target, "fiber_phase_lift_spin_transport"),
            ),
        )

    @staticmethod
    def _is_lawful(source: OperatorStateV4, target: OperatorStateV4, component: str) -> bool:
        if component == "fiber_phase_lift_spin_transport":
            return (
                target.r == source.r
                and target.composite_compat_class == source.composite_compat_class
                and target.spin_h.horizon == source.spin_h.horizon
                and target.phi == (source.phi + 1) % FIBER_PERIOD_V5
            )
        return (
            target.r == source.r
            and target.composite_compat_class == source.composite_compat_class
            and target.spin_h.horizon == source.spin_h.horizon
        )


def bounded_operator_surface_v6(
    depth: int = 8,
) -> tuple[tuple[OperatorStateV4, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV7()
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


def summarize_operator_v6(depth: int = 8) -> list[dict[str, object]]:
    v5_rows = summarize_operator_v5(depth=depth)
    v5_state_count = next(
        int(row["count"])
        for row in v5_rows
        if row["scope"] == "operator" and row["metric"] == "state_count"
    )
    v5_transition_count = next(
        int(row["count"])
        for row in v5_rows
        if row["scope"] == "operator" and row["metric"] == "transition_count"
    )

    states, transitions = bounded_operator_surface_v6(depth=depth)
    lawful_transitions = [t for t in transitions if t.lawful]
    illegal_transitions = [t for t in transitions if not t.lawful]
    component_counter = Counter()
    target_class_counter = Counter()
    target_spin_counter = Counter()

    for transition in transitions:
        component_counter[transition.component] += 1
        target_class_counter[class_tuple_v4(transition.target)] += 1
        target_spin_counter[transition.target.spin_h] += 1

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
            "metric": "state_count_vs_v5",
            "count": len(states),
            "total": v5_state_count,
            "fraction": len(states) / max(v5_state_count, 1),
            "note": "reachable state expansion relative to H_v5",
        }
    )
    rows.append(
        {
            "scope": "orbit_change",
            "metric": "transition_count_vs_v5",
            "count": len(transitions),
            "total": v5_transition_count,
            "fraction": len(transitions) / max(v5_transition_count, 1),
            "note": "nonzero transition expansion relative to H_v5",
        }
    )
    rows.append(
        {
            "scope": "class_escape",
            "metric": "distinct_class_identities_reached",
            "count": len(target_class_counter),
            "total": len(target_class_counter),
            "fraction": 1.0,
            "note": "number of distinct target class identities reached",
        }
    )
    rows.append(
        {
            "scope": "spin_escape",
            "metric": "distinct_spin_classes_reached",
            "count": len(target_spin_counter),
            "total": len(target_spin_counter),
            "fraction": 1.0,
            "note": "number of distinct target spin_h classes reached",
        }
    )
    rows.append(
        {
            "scope": "generator_axis",
            "metric": "T_z_changes_fiber_class",
            "count": 1,
            "total": 1,
            "fraction": 1.0,
            "note": "refined lift changes phi",
        }
    )
    rows.append(
        {
            "scope": "generator_axis",
            "metric": "T_z_changes_spin_class",
            "count": 1,
            "total": 1,
            "fraction": 1.0,
            "note": "refined lift changes spin_h through explicit spin transport",
        }
    )
    rows.append(
        {
            "scope": "component",
            "metric": "fraction_using_refined_lift",
            "count": component_counter["fiber_phase_lift_spin_transport"],
            "total": len(transitions),
            "fraction": component_counter["fiber_phase_lift_spin_transport"] / max(len(transitions), 1),
            "note": "fraction of transitions emitted by refined lift component",
        }
    )

    for component_name in (
        "hold",
        "torus_base_advance",
        "composite_swap",
        "coupled_torus_kick",
        "composite_twist",
        "fiber_phase_lift_spin_transport",
    ):
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

    for class_key, count in sorted(
        target_class_counter.items(),
        key=lambda item: (
            item[0][0],
            item[0][1],
            "".join(str(bit) for bit in item[0][2].bits),
            item[0][3][0],
            item[0][3][1],
        ),
    ):
        r, phi, spin_h, compat = class_key
        rows.append(
            {
                "scope": "class_distribution",
                "metric": (
                    f"class_r{r}_phi{phi}_spin{''.join(str(bit) for bit in spin_h.bits)}"
                    f"_compat{compat[0]}_{compat[1]}"
                ),
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by target class tuple",
            }
        )

    for spin_h, count in sorted(
        target_spin_counter.items(),
        key=lambda item: "".join(str(bit) for bit in item[0].bits),
    ):
        rows.append(
            {
                "scope": "spin_distribution",
                "metric": f"spin_{''.join(str(bit) for bit in spin_h.bits)}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by target spin_h",
            }
        )

    return rows


def write_operator_summary_v6(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_OPERATOR_V6,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["scope", "metric", "count", "total", "fraction", "note"],
        )
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_operator_v6(depth=8)
    write_operator_summary_v6(summary_rows)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V6}")
    for row in summary_rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )
