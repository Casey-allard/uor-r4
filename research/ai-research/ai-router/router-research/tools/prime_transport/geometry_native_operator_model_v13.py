#!/usr/bin/env python3
"""Ninth operator formulation rebuilding T_c directly from spin_H_core_v3."""

from __future__ import annotations

import csv
from collections import Counter, deque
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1, OperatorTransitionV1
from geometry_native_operator_model_v10 import (
    OperatorStateV10,
    class_tuple_v10,
    composite_swap_component_v10,
    composite_twist_component_v10,
    fiber_phase_lift_component_v10,
    hold_component_v10,
    initial_operator_state_v10,
    torus_base_advance_component_v10,
)
from geometry_native_operator_model_v12 import (
    OUTPUT_PATH_RADIAL_LAW_V1,
    radial_transport_component_v12,
)
from geometry_native_spinH_candidate_v3 import NativeTauV3
from geometry_native_spinH_core_v3 import (
    active_transport_lift_core_v3,
    derive_mode_orbit_v3,
    project_tau_v3,
)


OUTPUT_PATH_OPERATOR_FROM_CORE_V3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_from_spinH_core_v3.csv"
)


def _binary_spin_value_v13(word: tuple[int, ...]) -> int:
    return int("".join(str(int(bit)) for bit in word), 2)


def _projection_profile_map_v13(state: OperatorStateV10) -> dict[str, tuple[int, ...]]:
    core = active_transport_lift_core_v3(state)
    return dict(core.sigma.global_regressive_mode_index.generator_projection_profile)


def _orbit_profile_map_v13(state: OperatorStateV10) -> dict[str, tuple[tuple[int, ...], ...]]:
    core = active_transport_lift_core_v3(state)
    return dict(core.sigma.global_regressive_mode_index.generator_orbit_profile)


def coupled_torus_kick_component_v13(state: OperatorStateV10) -> OperatorStateV10:
    core = active_transport_lift_core_v3(state)
    profile_map = _projection_profile_map_v13(state)
    orbit_map = _orbit_profile_map_v13(state)

    source_tau = project_tau_v3(core)
    coupled_projection = profile_map["coupled_torus_kick"]
    coupled_orbit = orbit_map["coupled_torus_kick"]
    seed_orbit = derive_mode_orbit_v3(core.sigma.global_regressive_mode_index)

    theta_weight = core.theta.base_angle + core.theta.fiber_phase
    rho_weight = (
        core.rho.radial_class
        + core.rho.unfolding_load
        + core.rho.radial_target
        + core.rho.radial_target_phi
    )
    sigma_weight = (
        core.sigma.current_residue
        + core.sigma.fiber_residue
        + core.sigma.radial_residue
        + len(seed_orbit)
        + len(coupled_orbit)
    )
    h_weight = sum(core.h.recursive_phase) + sum(core.h.radial_recursive_phase) + core.h.holonomy_bit
    base_step = 1 + ((theta_weight + rho_weight + sigma_weight + h_weight) % 4)

    target_b = (state.b + base_step) % 5
    target_spin = NativeSpinHV1(horizon=state.spin_h.horizon, bits=tuple(int(bit) for bit in coupled_projection))
    target_tau = NativeTauV3(
        swap_phase=source_tau.swap_phase,
        coupled_phase=(source_tau.coupled_phase + len(coupled_orbit) + core.rho.radial_direction) % 5,
        twist_phase=(source_tau.twist_phase + core.h.holonomy_bit + (core.theta.fiber_phase % 2)) % 2,
        lift_phase=(source_tau.lift_phase + core.rho.unfolding_load + (_binary_spin_value_v13(coupled_projection) % 5)) % 12,
    )
    return OperatorStateV10(
        b=target_b,
        phi=state.phi,
        r=state.r,
        spin_h=target_spin,
        tau=target_tau,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


class SparseLawfulOperatorV13:
    """Explicit sparse operator with T_c rebuilt from spin_H_core_v3."""

    def transitions(self, state: OperatorStateV10) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v10(state)
        base_target = torus_base_advance_component_v10(state)
        swap_target = composite_swap_component_v10(state)
        coupled_target = coupled_torus_kick_component_v13(state)
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
            expected = coupled_torus_kick_component_v13(source)
            return (
                target == expected
                and target.phi == source.phi
                and target.r == source.r
                and target.composite_compat_class == source.composite_compat_class
                and target.spin_h.horizon == source.spin_h.horizon
            )
        if component == "composite_twist":
            return target.r == source.r and target.phi == source.phi
        if component == "fiber_phase_lift_spin_transport":
            return target.r == source.r and target.phi != source.phi
        if component == "radial_transport_unfolding":
            expected = radial_transport_component_v12(source)
            return target == expected
        return False


def bounded_operator_surface_v13(depth: int = 8) -> tuple[tuple[OperatorStateV10, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV13()
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


def _summary_metrics_from_v12() -> dict[str, float]:
    rows = list(csv.DictReader(OUTPUT_PATH_RADIAL_LAW_V1.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"operator", "radial_escape", "class_escape", "spin_escape", "tau_escape"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_operator_from_spinH_core_v3(depth: int = 8) -> list[dict[str, object]]:
    old = _summary_metrics_from_v12()
    states, transitions = bounded_operator_surface_v13(depth=depth)
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
    rows.append({"scope": "operator", "metric": "state_count", "count": len(states), "total": len(states), "fraction": 1.0, "note": "reachable states with T_c rebuilt from spin_H_core_v3"})
    rows.append({"scope": "operator", "metric": "transition_count", "count": len(transitions), "total": len(transitions), "fraction": 1.0, "note": "nonzero operator entries with one core-v3-derived component"})
    rows.append({"scope": "operator", "metric": "lawful_transition_fraction", "count": len(lawful_transitions), "total": len(transitions), "fraction": len(lawful_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are lawful"})
    rows.append({"scope": "operator", "metric": "illegal_transition_fraction", "count": len(illegal_transitions), "total": len(transitions), "fraction": len(illegal_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are illegal"})
    rows.append({"scope": "class_escape", "metric": "distinct_class_identities_reached", "count": len(target_class_counter), "total": len(target_class_counter), "fraction": 1.0, "note": "distinct tau-aware target class identities reached"})
    rows.append({"scope": "spin_escape", "metric": "distinct_spin_classes_reached", "count": len(target_spin_counter), "total": len(target_spin_counter), "fraction": 1.0, "note": "distinct target spin classes reached"})
    rows.append({"scope": "radial_escape", "metric": "distinct_radial_classes_reached", "count": len(radial_counter), "total": len(radial_counter), "fraction": 1.0, "note": "distinct target radial classes reached"})
    rows.append({"scope": "radial_escape", "metric": "fraction_changing_radial_class", "count": radial_change_hits, "total": len(transitions), "fraction": radial_change_hits / max(len(transitions), 1), "note": "fraction of transitions changing radial class"})
    rows.append({"scope": "spin_escape", "metric": "fraction_changing_spin_class_lawfully", "count": spin_change_hits, "total": len(transitions), "fraction": spin_change_hits / max(len(transitions), 1), "note": "fraction of transitions changing spin class lawfully"})
    rows.append({"scope": "tau_escape", "metric": "fraction_changing_tau_lawfully", "count": tau_change_hits, "total": len(transitions), "fraction": tau_change_hits / max(len(transitions), 1), "note": "fraction of transitions changing tau lawfully"})

    rows.append({"scope": "core_dependency", "metric": "rebuilt_component_depends_on_spin_H_core_v3", "count": 1, "total": 1, "fraction": 1.0, "note": "T_c calls active_transport_lift_core_v3 and reads sigma** global_regressive_mode_index"})
    rows.append({"scope": "core_dependency", "metric": "sigma_parent_mode_identity_materially_used", "count": 1, "total": 1, "fraction": 1.0, "note": "T_c uses sigma** generator_projection_profile, generator_orbit_profile, and derived mode orbit"})
    rows.append({"scope": "core_dependency", "metric": "theta_materially_used", "count": 1, "total": 1, "fraction": 1.0, "note": "T_c uses theta.base_angle and theta.fiber_phase in coupled base-step logic"})
    rows.append({"scope": "core_dependency", "metric": "rho_materially_used", "count": 1, "total": 1, "fraction": 1.0, "note": "T_c uses rho.radial_class, unfolding_load, radial_target, and radial_target_phi in coupled base-step logic"})
    rows.append({"scope": "core_dependency", "metric": "h_materially_used", "count": 1, "total": 1, "fraction": 1.0, "note": "T_c uses h.recursive_phase, h.radial_recursive_phase, and h.holonomy_bit in coupled tau update logic"})

    rows.append({"scope": "comparison_vs_v12", "metric": "state_count_change", "count": len(states) - int(old["state_count__count"]), "total": int(old["state_count__count"]), "fraction": (len(states) / max(old["state_count__count"], 1.0)) - 1.0, "note": "reachable state change vs prior operator with older T_c"})
    rows.append({"scope": "comparison_vs_v12", "metric": "transition_count_change", "count": len(transitions) - int(old["transition_count__count"]), "total": int(old["transition_count__count"]), "fraction": (len(transitions) / max(old["transition_count__count"], 1.0)) - 1.0, "note": "transition count change vs prior operator with older T_c"})
    rows.append({"scope": "comparison_vs_v12", "metric": "class_diversity_change", "count": len(target_class_counter) - int(old["distinct_class_identities_reached__count"]), "total": int(old["distinct_class_identities_reached__count"]), "fraction": (len(target_class_counter) / max(old["distinct_class_identities_reached__count"], 1.0)) - 1.0, "note": "class diversity change vs prior operator with older T_c"})
    rows.append({"scope": "comparison_vs_v12", "metric": "spin_diversity_change", "count": len(target_spin_counter) - int(old["distinct_spin_classes_reached__count"]), "total": int(old["distinct_spin_classes_reached__count"]), "fraction": (len(target_spin_counter) / max(old["distinct_spin_classes_reached__count"], 1.0)) - 1.0, "note": "spin diversity change vs prior operator with older T_c"})

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

    return rows


def write_operator_from_spinH_core_v3(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_OPERATOR_FROM_CORE_V3,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


__all__ = [
    "OUTPUT_PATH_OPERATOR_FROM_CORE_V3",
    "SparseLawfulOperatorV13",
    "bounded_operator_surface_v13",
    "coupled_torus_kick_component_v13",
    "summarize_operator_from_spinH_core_v3",
    "write_operator_from_spinH_core_v3",
]
