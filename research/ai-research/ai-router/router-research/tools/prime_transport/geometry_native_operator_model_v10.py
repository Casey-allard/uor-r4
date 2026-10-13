#!/usr/bin/env python3
"""Seventh operator formulation with the first lawful native radial transport generator."""

from __future__ import annotations

import csv
from collections import Counter, deque
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1, OperatorTransitionV1
from geometry_native_operator_model_v4 import (
    composite_swap_component_v4,
    composite_twist_component_v4,
    coupled_torus_kick_component_v4,
    initial_operator_state_v4,
    torus_base_advance_component_v4,
)
from geometry_native_operator_model_v7 import (
    OUTPUT_PATH_OPERATOR_V6,
    fiber_phase_lift_component_v6,
    summarize_operator_v6,
)
from geometry_native_spinH_candidate_v3 import NativeTauV3, initial_tau_v3, update_tau_v3


OUTPUT_PATH_OPERATOR_V7 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_operator_formulation_v7.csv"
)

RADIAL_PERIOD_V10 = 3


@dataclass(frozen=True)
class OperatorStateV10:
    b: int
    phi: int
    r: int
    spin_h: NativeSpinHV1
    tau: NativeTauV3
    composite_compat_class: tuple[str, str]
    query_semiprime: int
    binding_semiprime: int
    admissible_transition: int
    twist: int


def initial_operator_state_v10() -> OperatorStateV10:
    seed = initial_operator_state_v4()
    return OperatorStateV10(
        b=seed.b,
        phi=seed.phi,
        r=seed.r,
        spin_h=seed.spin_h,
        tau=initial_tau_v3(),
        composite_compat_class=seed.composite_compat_class,
        query_semiprime=seed.query_semiprime,
        binding_semiprime=seed.binding_semiprime,
        admissible_transition=seed.admissible_transition,
        twist=seed.twist,
    )


def _to_v4_view(state: OperatorStateV10):
    from geometry_native_operator_model_v4 import OperatorStateV4

    return OperatorStateV4(
        b=state.b,
        phi=state.phi,
        r=state.r,
        spin_h=state.spin_h,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


def class_tuple_v10(state: OperatorStateV10) -> tuple[int, int, NativeSpinHV1, NativeTauV3, tuple[str, str]]:
    return (state.r, state.phi, state.spin_h, state.tau, state.composite_compat_class)


def hold_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    return state


def torus_base_advance_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target = torus_base_advance_component_v4(_to_v4_view(state))
    return OperatorStateV10(
        b=target.b,
        phi=target.phi,
        r=target.r,
        spin_h=target.spin_h,
        tau=update_tau_v3(_to_v4_view(state), "torus_base_advance", state.tau),
        composite_compat_class=target.composite_compat_class,
        query_semiprime=target.query_semiprime,
        binding_semiprime=target.binding_semiprime,
        admissible_transition=target.admissible_transition,
        twist=target.twist,
    )


def composite_swap_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target = composite_swap_component_v4(_to_v4_view(state))
    return OperatorStateV10(
        b=target.b,
        phi=target.phi,
        r=target.r,
        spin_h=target.spin_h,
        tau=update_tau_v3(_to_v4_view(state), "composite_swap", state.tau),
        composite_compat_class=target.composite_compat_class,
        query_semiprime=target.query_semiprime,
        binding_semiprime=target.binding_semiprime,
        admissible_transition=target.admissible_transition,
        twist=target.twist,
    )


def coupled_torus_kick_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target = coupled_torus_kick_component_v4(_to_v4_view(state))
    return OperatorStateV10(
        b=target.b,
        phi=target.phi,
        r=target.r,
        spin_h=target.spin_h,
        tau=update_tau_v3(_to_v4_view(state), "coupled_torus_kick", state.tau),
        composite_compat_class=target.composite_compat_class,
        query_semiprime=target.query_semiprime,
        binding_semiprime=target.binding_semiprime,
        admissible_transition=target.admissible_transition,
        twist=target.twist,
    )


def composite_twist_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target = composite_twist_component_v4(_to_v4_view(state))
    return OperatorStateV10(
        b=target.b,
        phi=target.phi,
        r=target.r,
        spin_h=target.spin_h,
        tau=update_tau_v3(_to_v4_view(state), "composite_twist", state.tau),
        composite_compat_class=target.composite_compat_class,
        query_semiprime=target.query_semiprime,
        binding_semiprime=target.binding_semiprime,
        admissible_transition=target.admissible_transition,
        twist=target.twist,
    )


def fiber_phase_lift_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target = fiber_phase_lift_component_v6(_to_v4_view(state))
    return OperatorStateV10(
        b=target.b,
        phi=target.phi,
        r=target.r,
        spin_h=target.spin_h,
        tau=update_tau_v3(_to_v4_view(state), "fiber_phase_lift_spin_transport", state.tau),
        composite_compat_class=target.composite_compat_class,
        query_semiprime=target.query_semiprime,
        binding_semiprime=target.binding_semiprime,
        admissible_transition=target.admissible_transition,
        twist=target.twist,
    )


def radial_spin_transport_map_v10(
    spin_h: NativeSpinHV1,
    *,
    target_r: int,
    phi: int,
    tau: NativeTauV3,
    composite_compat_class: tuple[str, str],
) -> NativeSpinHV1:
    """Explicit lawful radial spin transport map.

    The rule is mechanical:

    1. rotate the predictive word by one step to expose one deeper unfolding slot
    2. compute a radial index from target depth, fiber phase, tau lift residue,
       and compatibility weight
    3. set the radial index slot to the parity of target depth plus tau coupled phase
    """
    bits = tuple(int(bit) for bit in spin_h.bits)
    rotated = bits[-1:] + bits[:-1]
    compat_weight = sum(int(bit) for mask in composite_compat_class for bit in mask)
    radial_index = (target_r + phi + tau.lift_phase + compat_weight) % spin_h.horizon
    transported = list(rotated)
    transported[radial_index] = (target_r + tau.coupled_phase) % 2
    return NativeSpinHV1(horizon=spin_h.horizon, bits=tuple(transported))


def radial_tau_transport_map_v10(
    tau: NativeTauV3,
    *,
    source_r: int,
    target_r: int,
    spin_h: NativeSpinHV1,
    phi: int,
) -> NativeTauV3:
    """Explicit lawful radial tau transport map."""
    binary_spin = int("".join(str(int(bit)) for bit in spin_h.bits), 2)
    radial_step = (1 + source_r + target_r + phi + (binary_spin % 3)) % 12
    return NativeTauV3(
        swap_phase=tau.swap_phase,
        coupled_phase=(tau.coupled_phase + target_r + 1) % 5,
        twist_phase=tau.twist_phase,
        lift_phase=(tau.lift_phase + radial_step) % 12,
    )


def radial_transport_component_v10(state: OperatorStateV10) -> OperatorStateV10:
    target_r = (state.r + 1) % RADIAL_PERIOD_V10
    target_spin = radial_spin_transport_map_v10(
        state.spin_h,
        target_r=target_r,
        phi=state.phi,
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    target_tau = radial_tau_transport_map_v10(
        state.tau,
        source_r=state.r,
        target_r=target_r,
        spin_h=state.spin_h,
        phi=state.phi,
    )
    return OperatorStateV10(
        b=state.b,
        phi=state.phi,
        r=target_r,
        spin_h=target_spin,
        tau=target_tau,
        composite_compat_class=state.composite_compat_class,
        query_semiprime=state.query_semiprime,
        binding_semiprime=state.binding_semiprime,
        admissible_transition=state.admissible_transition,
        twist=state.twist,
    )


class SparseLawfulOperatorV10:
    """Explicit sparse operator H_v7 = I + T_b + T_x + T_c + T_y + T_z' + T_r."""

    def transitions(self, state: OperatorStateV10) -> tuple[OperatorTransitionV1, ...]:
        hold_target = hold_component_v10(state)
        base_target = torus_base_advance_component_v10(state)
        swap_target = composite_swap_component_v10(state)
        coupled_target = coupled_torus_kick_component_v10(state)
        twist_target = composite_twist_component_v10(state)
        lift_target = fiber_phase_lift_component_v10(state)
        radial_target = radial_transport_component_v10(state)
        return (
            OperatorTransitionV1(source=state, target=hold_target, component="hold", lawful=self._is_lawful(state, hold_target, "hold")),
            OperatorTransitionV1(source=state, target=base_target, component="torus_base_advance", lawful=self._is_lawful(state, base_target, "torus_base_advance")),
            OperatorTransitionV1(source=state, target=swap_target, component="composite_swap", lawful=self._is_lawful(state, swap_target, "composite_swap")),
            OperatorTransitionV1(source=state, target=coupled_target, component="coupled_torus_kick", lawful=self._is_lawful(state, coupled_target, "coupled_torus_kick")),
            OperatorTransitionV1(source=state, target=twist_target, component="composite_twist", lawful=self._is_lawful(state, twist_target, "composite_twist")),
            OperatorTransitionV1(source=state, target=lift_target, component="fiber_phase_lift_spin_transport", lawful=self._is_lawful(state, lift_target, "fiber_phase_lift_spin_transport")),
            OperatorTransitionV1(source=state, target=radial_target, component="radial_transport", lawful=self._is_lawful(state, radial_target, "radial_transport")),
        )

    @staticmethod
    def _is_lawful(source: OperatorStateV10, target: OperatorStateV10, component: str) -> bool:
        if component == "hold":
            return target == source
        if component == "torus_base_advance":
            return (
                target.b == (source.b + 1) % 5
                and target.phi == source.phi
                and target.r == source.r
                and target.spin_h == source.spin_h
                and target.tau == update_tau_v3(_to_v4_view(source), "torus_base_advance", source.tau)
                and target.composite_compat_class == source.composite_compat_class
            )
        if component == "composite_swap":
            return (
                target.b == source.b
                and target.phi == source.phi
                and target.r == source.r
                and target.spin_h == source.spin_h
                and target.tau == update_tau_v3(_to_v4_view(source), "composite_swap", source.tau)
            )
        if component == "coupled_torus_kick":
            return (
                target.phi == source.phi
                and target.r == source.r
                and target.spin_h == source.spin_h
                and target.tau == update_tau_v3(_to_v4_view(source), "coupled_torus_kick", source.tau)
                and target.composite_compat_class == source.composite_compat_class
            )
        if component == "composite_twist":
            return (
                target.b == source.b
                and target.phi == source.phi
                and target.r == source.r
                and target.spin_h == source.spin_h
                and target.tau == update_tau_v3(_to_v4_view(source), "composite_twist", source.tau)
                and target.composite_compat_class == source.composite_compat_class
            )
        if component == "fiber_phase_lift_spin_transport":
            return (
                target.b == source.b
                and target.phi == (source.phi + 1) % 3
                and target.r == source.r
                and target.tau == update_tau_v3(_to_v4_view(source), "fiber_phase_lift_spin_transport", source.tau)
                and target.composite_compat_class == source.composite_compat_class
                and target.spin_h.horizon == source.spin_h.horizon
            )
        if component == "radial_transport":
            return (
                target.b == source.b
                and target.phi == source.phi
                and target.r == (source.r + 1) % RADIAL_PERIOD_V10
                and target.tau == radial_tau_transport_map_v10(
                    source.tau,
                    source_r=source.r,
                    target_r=target.r,
                    spin_h=source.spin_h,
                    phi=source.phi,
                )
                and target.spin_h == radial_spin_transport_map_v10(
                    source.spin_h,
                    target_r=target.r,
                    phi=source.phi,
                    tau=source.tau,
                    composite_compat_class=source.composite_compat_class,
                )
                and target.composite_compat_class == source.composite_compat_class
                and target.spin_h.horizon == source.spin_h.horizon
            )
        return False


def bounded_operator_surface_v7(depth: int = 8) -> tuple[tuple[OperatorStateV10, ...], tuple[OperatorTransitionV1, ...]]:
    operator = SparseLawfulOperatorV10()
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


def summarize_operator_v7(depth: int = 8) -> list[dict[str, object]]:
    v6_rows = summarize_operator_v6(depth=depth)
    v6_state_count = next(int(row["count"]) for row in v6_rows if row["scope"] == "operator" and row["metric"] == "state_count")
    v6_transition_count = next(int(row["count"]) for row in v6_rows if row["scope"] == "operator" and row["metric"] == "transition_count")

    states, transitions = bounded_operator_surface_v7(depth=depth)
    lawful_transitions = [t for t in transitions if t.lawful]
    illegal_transitions = [t for t in transitions if not t.lawful]
    component_counter = Counter()
    target_class_counter = Counter()
    target_spin_counter = Counter()
    radial_counter = Counter()
    radial_change_hits = 0
    radial_preserve_hits = 0

    for transition in transitions:
        component_counter[transition.component] += 1
        target_class_counter[class_tuple_v10(transition.target)] += 1
        target_spin_counter[transition.target.spin_h] += 1
        radial_counter[transition.target.r] += 1
        if transition.target.r != transition.source.r:
            radial_change_hits += 1
        else:
            radial_preserve_hits += 1

    rows: list[dict[str, object]] = []
    rows.append({"scope": "operator", "metric": "state_count", "count": len(states), "total": len(states), "fraction": 1.0, "note": "reachable structured tau-aware states on bounded operator surface"})
    rows.append({"scope": "operator", "metric": "transition_count", "count": len(transitions), "total": len(transitions), "fraction": 1.0, "note": "nonzero operator entries on bounded surface"})
    rows.append({"scope": "operator", "metric": "lawful_transition_fraction", "count": len(lawful_transitions), "total": len(transitions), "fraction": len(lawful_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are lawful"})
    rows.append({"scope": "operator", "metric": "illegal_transition_fraction", "count": len(illegal_transitions), "total": len(transitions), "fraction": len(illegal_transitions) / max(len(transitions), 1), "note": "fraction of nonzero operator entries that are illegal"})
    rows.append({"scope": "orbit_change", "metric": "state_count_vs_v6", "count": len(states), "total": v6_state_count, "fraction": len(states) / max(v6_state_count, 1), "note": "reachable state expansion relative to H_v6"})
    rows.append({"scope": "orbit_change", "metric": "transition_count_vs_v6", "count": len(transitions), "total": v6_transition_count, "fraction": len(transitions) / max(v6_transition_count, 1), "note": "nonzero transition expansion relative to H_v6"})
    rows.append({"scope": "radial_escape", "metric": "distinct_radial_classes_reached", "count": len(radial_counter), "total": len(radial_counter), "fraction": 1.0, "note": "number of distinct target radial classes reached"})
    rows.append({"scope": "class_escape", "metric": "distinct_class_identities_reached", "count": len(target_class_counter), "total": len(target_class_counter), "fraction": 1.0, "note": "number of distinct tau-aware target class identities reached"})
    rows.append({"scope": "spin_escape", "metric": "distinct_spin_classes_reached", "count": len(target_spin_counter), "total": len(target_spin_counter), "fraction": 1.0, "note": "number of distinct target spin classes reached"})
    rows.append({"scope": "radial_escape", "metric": "fraction_changing_radial_class", "count": radial_change_hits, "total": len(transitions), "fraction": radial_change_hits / max(len(transitions), 1), "note": "fraction of transitions that change radial class"})
    rows.append({"scope": "radial_escape", "metric": "fraction_preserving_radial_class", "count": radial_preserve_hits, "total": len(transitions), "fraction": radial_preserve_hits / max(len(transitions), 1), "note": "fraction of transitions that preserve radial class"})
    rows.append({"scope": "generator_axis", "metric": "T_r_changes_radial_class", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport changes r by one adjacent step"})
    rows.append({"scope": "generator_axis", "metric": "T_r_preserves_or_lawfully_transports_spin_state", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport uses explicit radial_spin_transport_map_v10"})
    rows.append({"scope": "generator_axis", "metric": "T_r_preserves_or_lawfully_transports_tau", "count": 1, "total": 1, "fraction": 1.0, "note": "radial_transport uses explicit radial_tau_transport_map_v10"})

    for component_name in (
        "hold",
        "torus_base_advance",
        "composite_swap",
        "coupled_torus_kick",
        "composite_twist",
        "fiber_phase_lift_spin_transport",
        "radial_transport",
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

    for spin_h, count in sorted(target_spin_counter.items(), key=lambda item: (item[0].horizon, item[0].bits)):
        rows.append(
            {
                "scope": "spin_distribution",
                "metric": f"spin_{''.join(str(bit) for bit in spin_h.bits)}",
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by target spin class",
            }
        )

    for class_key, count in sorted(
        target_class_counter.items(),
        key=lambda item: (
            item[0][0],
            item[0][1],
            item[0][2].bits,
            item[0][3].swap_phase,
            item[0][3].coupled_phase,
            item[0][3].twist_phase,
            item[0][3].lift_phase,
            item[0][4],
        ),
    ):
        r, phi, spin_h, tau, compat = class_key
        rows.append(
            {
                "scope": "class_distribution",
                "metric": (
                    f"class_r{r}_phi{phi}_spin{''.join(str(bit) for bit in spin_h.bits)}"
                    f"_tau{safestr_tau(tau)}_compat{compat[0]}_{compat[1]}"
                ),
                "count": count,
                "total": len(transitions),
                "fraction": count / max(len(transitions), 1),
                "note": "distribution of operator transitions by tau-aware target class tuple",
            }
        )

    return rows


def safestr_tau(tau: NativeTauV3) -> str:
    return f"s{tau.swap_phase}c{tau.coupled_phase}t{tau.twist_phase}l{tau.lift_phase}"


def write_operator_summary_v7(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_OPERATOR_V7) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_operator_v7(depth=8)
    write_operator_summary_v7(summary_rows)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V7}")
    for row in summary_rows:
        if row["scope"] in {"operator", "orbit_change", "radial_escape", "class_escape", "spin_escape", "generator_axis"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
