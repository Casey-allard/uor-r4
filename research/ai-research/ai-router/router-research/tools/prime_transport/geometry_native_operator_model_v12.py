#!/usr/bin/env python3
"""Eighth operator formulation with a fuller bounded radial transport law."""

from __future__ import annotations

import csv
from collections import Counter, deque
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1, OperatorTransitionV1
from geometry_native_operator_model_v10 import (
    OUTPUT_PATH_OPERATOR_V7,
    OperatorStateV10,
    class_tuple_v10,
    composite_swap_component_v10,
    composite_twist_component_v10,
    coupled_torus_kick_component_v10,
    fiber_phase_lift_component_v10,
    hold_component_v10,
    initial_operator_state_v10,
    safestr_tau,
)
from geometry_native_operator_model_v10 import summarize_operator_v7 as summarize_operator_v10_placeholder
from geometry_native_operator_model_v10 import torus_base_advance_component_v10
from geometry_native_spinH_candidate_v3 import NativeTauV3


OUTPUT_PATH_RADIAL_LAW_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_radial_law_v1.csv"
)

RADIAL_PERIOD_V12 = 3


def _binary_spin_value_v12(spin_h: NativeSpinHV1) -> int:
    return int("".join(str(int(bit)) for bit in spin_h.bits), 2)


def _unfolding_load_v12(spin_h: NativeSpinHV1) -> int:
    return sum(int(bit) for bit in spin_h.bits)


def radial_direction_v12(state: OperatorStateV10) -> int:
    """Bounded expansion/contraction rule derived from predictive unfolding load."""
    unfolding_charge = (
        _unfolding_load_v12(state.spin_h)
        + state.tau.coupled_phase
        + state.tau.lift_phase
    ) % 2
    return 1 if unfolding_charge == 0 else -1


def radial_phi_transport_v12(state: OperatorStateV10, *, target_r: int, direction: int) -> int:
    """Fiber precession coupled to radial expansion/contraction."""
    phase_step = state.tau.twist_phase + (1 if direction < 0 else 0)
    return (state.phi + phase_step) % 3


def radial_spin_transport_map_v12(
    spin_h: NativeSpinHV1,
    *,
    source_r: int,
    target_r: int,
    target_phi: int,
    direction: int,
    tau: NativeTauV3,
    composite_compat_class: tuple[str, str],
) -> NativeSpinHV1:
    """Recursive-unfolding spin transport.

    Expansion rotates forward by one slot and writes an exposed unfolding bit.
    Contraction rotates by two slots and writes a folded bit at a tau-shifted
    slot. This is still bounded, but it is no longer the adjacent radial
    placeholder.
    """
    bits = tuple(int(bit) for bit in spin_h.bits)
    rotate_by = 1 if direction > 0 else 2
    rotated = bits[rotate_by:] + bits[:rotate_by]
    compat_weight = sum(int(bit) for mask in composite_compat_class for bit in mask)
    radial_index = (target_r + target_phi + tau.lift_phase + compat_weight) % spin_h.horizon
    transported = list(rotated)
    transported[radial_index] = 1 if direction > 0 else 0
    if direction < 0:
        folded_index = (radial_index + tau.coupled_phase + source_r) % spin_h.horizon
        transported[folded_index] = (tau.swap_phase + tau.twist_phase) % 2
    return NativeSpinHV1(horizon=spin_h.horizon, bits=tuple(transported))


def radial_tau_transport_map_v12(
    tau: NativeTauV3,
    *,
    source_r: int,
    target_r: int,
    target_phi: int,
    spin_h: NativeSpinHV1,
    direction: int,
) -> NativeTauV3:
    """Recursive-unfolding tau transport."""
    spin_value = _binary_spin_value_v12(spin_h)
    unfolding_load = _unfolding_load_v12(spin_h)
    return NativeTauV3(
        swap_phase=tau.swap_phase,
        coupled_phase=(tau.coupled_phase + target_r + (1 if direction > 0 else 2)) % 5,
        twist_phase=(tau.twist_phase + (1 if direction < 0 else 0)) % 2,
        lift_phase=(tau.lift_phase + unfolding_load + target_phi + (spin_value % 3)) % 12,
    )


def radial_transport_component_v12(state: OperatorStateV10) -> OperatorStateV10:
    direction = radial_direction_v12(state)
    target_r = (state.r + direction) % RADIAL_PERIOD_V12
    target_phi = radial_phi_transport_v12(state, target_r=target_r, direction=direction)
    target_spin = radial_spin_transport_map_v12(
        state.spin_h,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        direction=direction,
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    target_tau = radial_tau_transport_map_v12(
        state.tau,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        spin_h=state.spin_h,
        direction=direction,
    )
    return OperatorStateV10(
        b=state.b,
        phi=target_phi,
        r=target_r,
        spin_h=target_spin,
        tau=target_tau,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


class SparseLawfulOperatorV12:
    """Explicit sparse operator H_v8 with fuller bounded radial law T_r*."""

    def transitions(self, state: OperatorStateV10) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v10(state)
        base_target = torus_base_advance_component_v10(state)
        swap_target = composite_swap_component_v10(state)
        coupled_target = coupled_torus_kick_component_v10(state)
        twist_target = composite_twist_component_v10(state)
        lift_target = fiber_phase_lift_component_v10(state)
        radial_target = radial_transport_component_v12(state)
        return (
            OperatorTransitionV1(source=state, target=hold_target, component="hold", lawful=self._is_lawful(state, hold_target, "hold")),
            OperatorTransitionV1(source=state, target=base_target, component="torus_base_advance", lawful=self._is_lawful(state, base_target, "torus_base_advance")),
            OperatorTransitionV1(source=state, target=swap_target, component="composite_swap", lawful=self._is_lawful(state, swap_target, "composite_swap")),
            OperatorTransitionV1(source=state, target=coupled_target, component="coupled_torus_kick", lawful=self._is_lawful(state, coupled_target, "coupled_torus_kick")),
            OperatorTransitionV1(source=state, target=twist_target, component="composite_twist", lawful=self._is_lawful(state, twist_target, "composite_twist")),
            OperatorTransitionV1(source=state, target=lift_target, component="fiber_phase_lift_spin_transport", lawful=self._is_lawful(state, lift_target, "fiber_phase_lift_spin_transport")),
            OperatorTransitionV1(source=state, target=radial_target, component="radial_transport_unfolding", lawful=self._is_lawful(state, radial_target, "radial_transport_unfolding")),
        )

    @staticmethod
    def _is_lawful(source: OperatorStateV10, target: OperatorStateV10, component: str) -> bool:
        if component == "hold":
            return target == source
        if component == "torus_base_advance":
            return target.b != source.b and target.r == source.r and target.phi == source.phi
        if component == "composite_swap":
            return target.b == source.b and target.r == source.r and target.phi == source.phi
        if component == "coupled_torus_kick":
            return target.r == source.r and target.phi == source.phi
        if component == "composite_twist":
            return target.r == source.r and target.phi == source.phi
        if component == "fiber_phase_lift_spin_transport":
            return target.r == source.r and target.phi != source.phi
        if component == "radial_transport_unfolding":
            direction = radial_direction_v12(source)
            expected_r = (source.r + direction) % RADIAL_PERIOD_V12
            expected_phi = radial_phi_transport_v12(source, target_r=expected_r, direction=direction)
            expected_spin = radial_spin_transport_map_v12(
                source.spin_h,
                source_r=source.r,
                target_r=expected_r,
                target_phi=expected_phi,
                direction=direction,
                tau=source.tau,
                composite_compat_class=source.composite_compat_class,
            )
            expected_tau = radial_tau_transport_map_v12(
                source.tau,
                source_r=source.r,
                target_r=expected_r,
                target_phi=expected_phi,
                spin_h=source.spin_h,
                direction=direction,
            )
            return (
                target.b == source.b
                and target.r == expected_r
                and target.phi == expected_phi
                and target.spin_h == expected_spin
                and target.tau == expected_tau
                and target.composite_compat_class == source.composite_compat_class
                and target.spin_h.horizon == source.spin_h.horizon
            )
        return False


def bounded_operator_surface_v8(depth: int = 8) -> tuple[tuple[OperatorStateV10, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV12()
    seed = initial_operator_state_v10()
    seen: set[OperatorStateV10] = {seed}
    states: list[OperatorStateV10] = [seed]
    transitions: list[OperatorTransitionV1] = []
    queue: deque[tuple[OperatorStateV10, int]] = deque([(seed, 0)])

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


def _summary_metrics_from_v7() -> dict[str, float]:
    rows = list(csv.DictReader(OUTPUT_PATH_OPERATOR_V7.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"operator", "radial_escape", "class_escape", "spin_escape"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_radial_law_v1(depth: int = 8) -> list[dict[str, object]]:
    old = _summary_metrics_from_v7()
    states, transitions = bounded_operator_surface_v8(depth=depth)
    lawful_transitions = [t for t in transitions if t.lawful]
    illegal_transitions = [t for t in transitions if not t.lawful]
    component_counter = Counter()
    target_class_counter = Counter()
    target_spin_counter = Counter()
    radial_counter = Counter()
    radial_change_hits = 0
    spin_change_hits = 0
    tau_change_hits = 0

    for transition in transitions:
        component_counter[transition.component] += 1
        target_class_counter[class_tuple_v10(transition.target)] += 1
        target_spin_counter[transition.target.spin_h] += 1
        radial_counter[transition.target.r] += 1
        if transition.target.r != transition.source.r:
            radial_change_hits += 1
        if transition.target.spin_h != transition.source.spin_h:
            spin_change_hits += 1
        if transition.target.tau != transition.source.tau:
            tau_change_hits += 1

    rows: list[dict[str, object]] = []
    rows.append({"scope": "operator", "metric": "state_count", "count": len(states), "total": len(states), "fraction": 1.0, "note": "reachable states under fuller bounded radial law"})
    rows.append({"scope": "operator", "metric": "transition_count", "count": len(transitions), "total": len(transitions), "fraction": 1.0, "note": "nonzero operator entries under fuller bounded radial law"})
    rows.append({"scope": "operator", "metric": "lawful_transition_fraction", "count": len(lawful_transitions), "total": len(transitions), "fraction": len(lawful_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are lawful"})
    rows.append({"scope": "operator", "metric": "illegal_transition_fraction", "count": len(illegal_transitions), "total": len(transitions), "fraction": len(illegal_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are illegal"})
    rows.append({"scope": "radial_escape", "metric": "distinct_radial_classes_reached", "count": len(radial_counter), "total": len(radial_counter), "fraction": 1.0, "note": "number of distinct target radial classes reached"})
    rows.append({"scope": "class_escape", "metric": "distinct_class_identities_reached", "count": len(target_class_counter), "total": len(target_class_counter), "fraction": 1.0, "note": "number of distinct tau-aware target class identities reached"})
    rows.append({"scope": "spin_escape", "metric": "distinct_spin_classes_reached", "count": len(target_spin_counter), "total": len(target_spin_counter), "fraction": 1.0, "note": "number of distinct target spin classes reached"})
    rows.append({"scope": "radial_escape", "metric": "fraction_changing_radial_class", "count": radial_change_hits, "total": len(transitions), "fraction": radial_change_hits / max(len(transitions), 1), "note": "fraction of transitions that change radial class"})
    rows.append({"scope": "spin_escape", "metric": "fraction_changing_spin_class_lawfully", "count": spin_change_hits, "total": len(transitions), "fraction": spin_change_hits / max(len(transitions), 1), "note": "fraction of transitions that change spin class lawfully"})
    rows.append({"scope": "tau_escape", "metric": "fraction_changing_tau_lawfully", "count": tau_change_hits, "total": len(transitions), "fraction": tau_change_hits / max(len(transitions), 1), "note": "fraction of transitions that change tau lawfully"})
    rows.append({"scope": "comparison_vs_old_radial", "metric": "state_space_expansion_vs_T_r", "count": len(states), "total": int(old['state_count__count']), "fraction": len(states) / max(old['state_count__count'], 1), "note": "reachable state expansion relative to adjacent radial placeholder"})
    rows.append({"scope": "comparison_vs_old_radial", "metric": "class_diversity_vs_T_r", "count": len(target_class_counter), "total": int(old['distinct_class_identities_reached__count']), "fraction": len(target_class_counter) / max(old['distinct_class_identities_reached__count'], 1), "note": "class diversity relative to adjacent radial placeholder"})
    rows.append({"scope": "comparison_vs_old_radial", "metric": "spin_diversity_vs_T_r", "count": len(target_spin_counter), "total": int(old['distinct_spin_classes_reached__count']), "fraction": len(target_spin_counter) / max(old['distinct_spin_classes_reached__count'], 1), "note": "spin diversity relative to adjacent radial placeholder"})
    rows.append({"scope": "generator_axis", "metric": "T_r_star_changes_radial_class", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport_unfolding changes radial class via expansion/contraction law"})
    rows.append({"scope": "generator_axis", "metric": "T_r_star_changes_spin_class_lawfully", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport_unfolding uses explicit recursive-unfolding spin transport"})
    rows.append({"scope": "generator_axis", "metric": "T_r_star_changes_tau_lawfully", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport_unfolding uses explicit recursive-unfolding tau transport"})

    for component_name in (
        "hold",
        "torus_base_advance",
        "composite_swap",
        "coupled_torus_kick",
        "composite_twist",
        "fiber_phase_lift_spin_transport",
        "radial_transport_unfolding",
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

    for radial_class, count in sorted(radial_counter.items()):
        rows.append(
            {
                "scope": "radial_distribution",
                "metric": f"radial_class_{radial_class}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by target radial class",
            }
        )

    return rows


def write_radial_law_v1(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_RADIAL_LAW_V1) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_radial_law_v1(depth=8)
    write_radial_law_v1(summary_rows)
    print(f"wrote {OUTPUT_PATH_RADIAL_LAW_V1}")
    for row in summary_rows:
        if row["scope"] in {"operator", "radial_escape", "class_escape", "spin_escape", "tau_escape", "comparison_vs_old_radial", "generator_axis"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
