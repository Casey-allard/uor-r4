#!/usr/bin/env python3
"""Augmented spin_H candidate with native recursive transport-phase state tau."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict, deque
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1
from geometry_native_operator_model_v3 import _interaction_step_v3
from geometry_native_operator_model_v4 import initial_operator_state_v4
from geometry_native_operator_model_v7 import SparseLawfulOperatorV7
from geometry_native_spinH_candidate_v1 import _phi_str, _spin_str


OUTPUT_PATH_SPINH_CANDIDATE_V3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_candidate_v3.csv"
)

V2_SUMMARY_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_candidate_v2.csv"
)

SWAP_PHASE_MODULUS_V3 = 2
COUPLED_PHASE_MODULUS_V3 = 5
TWIST_PHASE_MODULUS_V3 = 2
LIFT_PHASE_MODULUS_V3 = 12


@dataclass(frozen=True)
class NativeTauV3:
    swap_phase: int
    coupled_phase: int
    twist_phase: int
    lift_phase: int


@dataclass(frozen=True)
class PrimaryChartStateSpinHCandidateV3:
    b: int
    phi: int
    r: int


@dataclass(frozen=True)
class SpinHCandidateV3:
    angular_fiber: tuple[int, ...]
    radial_depth: int
    predictive_word: NativeSpinHV1
    tau: NativeTauV3


@dataclass(frozen=True)
class TransportStateSpinHCandidateV3:
    b: int
    spin_H_candidate: SpinHCandidateV3


@dataclass(frozen=True)
class AugmentedOperatorStateV3:
    operator_state: object
    tau: NativeTauV3


def _tau_str(tau: NativeTauV3) -> str:
    return f"s{tau.swap_phase}c{tau.coupled_phase}t{tau.twist_phase}l{tau.lift_phase}"


def initial_tau_v3() -> NativeTauV3:
    return NativeTauV3(swap_phase=0, coupled_phase=0, twist_phase=0, lift_phase=0)


def _binary_spin_value_v3(spin_h: NativeSpinHV1) -> int:
    return int("".join(str(int(bit)) for bit in spin_h.bits), 2)


def swap_phase_v3(tau: NativeTauV3) -> NativeTauV3:
    return NativeTauV3(
        swap_phase=(tau.swap_phase + 1) % SWAP_PHASE_MODULUS_V3,
        coupled_phase=tau.coupled_phase,
        twist_phase=tau.twist_phase,
        lift_phase=tau.lift_phase,
    )


def coupled_phase_v3(tau: NativeTauV3, query_semiprime: int, binding_semiprime: int) -> NativeTauV3:
    interaction_step = _interaction_step_v3(query_semiprime, binding_semiprime)
    return NativeTauV3(
        swap_phase=tau.swap_phase,
        coupled_phase=(tau.coupled_phase + interaction_step) % COUPLED_PHASE_MODULUS_V3,
        twist_phase=tau.twist_phase,
        lift_phase=tau.lift_phase,
    )


def twist_phase_v3(tau: NativeTauV3) -> NativeTauV3:
    return NativeTauV3(
        swap_phase=tau.swap_phase,
        coupled_phase=tau.coupled_phase,
        twist_phase=(tau.twist_phase + 1) % TWIST_PHASE_MODULUS_V3,
        lift_phase=tau.lift_phase,
    )


def lift_phase_v3(tau: NativeTauV3, phi: int, spin_h: NativeSpinHV1) -> NativeTauV3:
    phase_step = 1 + int(phi) + (_binary_spin_value_v3(spin_h) % LIFT_PHASE_MODULUS_V3)
    return NativeTauV3(
        swap_phase=tau.swap_phase,
        coupled_phase=tau.coupled_phase,
        twist_phase=tau.twist_phase,
        lift_phase=(tau.lift_phase + phase_step) % LIFT_PHASE_MODULUS_V3,
    )


def update_tau_v3(source_state: object, component: str, tau: NativeTauV3) -> NativeTauV3:
    if component in {"hold", "torus_base_advance"}:
        return tau
    if component == "composite_swap":
        return swap_phase_v3(tau)
    if component == "coupled_torus_kick":
        return coupled_phase_v3(tau, int(source_state.query_semiprime), int(source_state.binding_semiprime))
    if component == "composite_twist":
        return twist_phase_v3(tau)
    if component == "fiber_phase_lift_spin_transport":
        return lift_phase_v3(tau, int(source_state.phi), source_state.spin_h)
    raise ValueError(f"unknown operator component {component!r}")


def active_transport_lift_v3(state: object, tau: NativeTauV3) -> TransportStateSpinHCandidateV3:
    phi_tuple = (int(state.phi),)
    predictive_word = NativeSpinHV1(
        horizon=int(state.spin_h.horizon),
        bits=tuple(int(bit) for bit in state.spin_h.bits),
    )
    return TransportStateSpinHCandidateV3(
        b=int(state.b),
        spin_H_candidate=SpinHCandidateV3(
            angular_fiber=phi_tuple,
            radial_depth=int(state.r),
            predictive_word=predictive_word,
            tau=tau,
        ),
    )


def primary_chart_of_v3(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


def bounded_augmented_surface_v3(depth: int = 8) -> tuple[tuple[AugmentedOperatorStateV3, ...], tuple[tuple[AugmentedOperatorStateV3, str, AugmentedOperatorStateV3], ...]]:
    operator = SparseLawfulOperatorV7()
    seed = AugmentedOperatorStateV3(operator_state=initial_operator_state_v4(), tau=initial_tau_v3())
    seen: set[AugmentedOperatorStateV3] = {seed}
    states: list[AugmentedOperatorStateV3] = [seed]
    transitions: list[tuple[AugmentedOperatorStateV3, str, AugmentedOperatorStateV3]] = []
    queue: deque[tuple[AugmentedOperatorStateV3, int]] = deque([(seed, 0)])

    while queue:
        augmented_state, dist = queue.popleft()
        for transition in operator.transitions(augmented_state.operator_state):
            target_tau = update_tau_v3(augmented_state.operator_state, transition.component, augmented_state.tau)
            target_augmented = AugmentedOperatorStateV3(operator_state=transition.target, tau=target_tau)
            transitions.append((augmented_state, transition.component, target_augmented))
            if dist >= depth:
                continue
            if target_augmented not in seen:
                seen.add(target_augmented)
                states.append(target_augmented)
                queue.append((target_augmented, dist + 1))

    return tuple(states), tuple(transitions)


def _summary_metrics_from_v2(path: Path = V2_SUMMARY_PATH) -> dict[str, float]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "representation"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_spinH_candidate_v3(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_augmented_surface_v3(depth=depth)

    primary_to_transport: dict[PrimaryChartStateSpinHCandidateV3, set[TransportStateSpinHCandidateV3]] = defaultdict(set)
    transport_to_primary: dict[TransportStateSpinHCandidateV3, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    transport_counter: Counter[TransportStateSpinHCandidateV3] = Counter()
    transport_transition_map: dict[TransportStateSpinHCandidateV3, dict[str, set[TransportStateSpinHCandidateV3]]] = defaultdict(lambda: defaultdict(set))

    angular_preserved = True
    radial_preserved = True
    predictive_present = True

    for augmented_state in states:
        primary = primary_chart_of_v3(augmented_state.operator_state)
        transport = active_transport_lift_v3(augmented_state.operator_state, augmented_state.tau)
        primary_to_transport[primary].add(transport)
        transport_to_primary[transport].add(primary)
        transport_counter[transport] += 1
        angular_preserved &= transport.b == primary.b and transport.spin_H_candidate.angular_fiber == (primary.phi,)
        radial_preserved &= transport.spin_H_candidate.radial_depth == primary.r
        predictive_present &= transport.spin_H_candidate.predictive_word.horizon > 0

    for source_augmented, component, target_augmented in transitions:
        source_transport = active_transport_lift_v3(source_augmented.operator_state, source_augmented.tau)
        target_transport = active_transport_lift_v3(target_augmented.operator_state, target_augmented.tau)
        transport_transition_map[source_transport][component].add(target_transport)

    primary_count = len(primary_to_transport)
    distinct_transport_count = len(transport_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in transport_to_primary.values())
    collision_fraction = collision_count / max(primary_count, 1)
    ambiguity_count = sum(1 for images in primary_to_transport.values() if len(images) > 1)
    ambiguity_fraction = ambiguity_count / max(primary_count, 1)

    canonical_transport_count = 0
    noncanonical_transport_count = 0
    for component_map in transport_transition_map.values():
        is_canonical = all(len(targets) <= 1 for targets in component_map.values())
        if is_canonical:
            canonical_transport_count += 1
        else:
            noncanonical_transport_count += 1
    recursive_consistency_rate = canonical_transport_count / max(distinct_transport_count, 1)

    v2_metrics = _summary_metrics_from_v2()
    recursive_consistency_improvement = recursive_consistency_rate - v2_metrics["recursive_consistency_rate__fraction"]
    branching_reduction = int(v2_metrics["noncanonical_branching_states__count"]) - noncanonical_transport_count
    collision_change = collision_count - int(v2_metrics["collision_count__count"])
    recursive_closure_improves_materially = recursive_consistency_improvement > 0.25

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states (b,phi,r) on the bounded lawful operator surface"})
    rows.append({"scope": "summary", "metric": "distinct_transport_identities_reached", "count": distinct_transport_count, "total": primary_count, "fraction": distinct_transport_count / max(primary_count, 1), "note": "distinct augmented transport identities under spin_H_candidate_v2 = (phi,r,spin_h4,tau)"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into augmented transport identities"})
    rows.append({"scope": "summary", "metric": "ambiguity_count", "count": ambiguity_count, "total": primary_count, "fraction": ambiguity_fraction, "note": "primary chart states with more than one augmented transport identity under repeated lawful iteration"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_transport_count, "total": distinct_transport_count, "fraction": recursive_consistency_rate, "note": "fraction of augmented transport identities whose lawful component updates are canonical"})
    rows.append({"scope": "summary", "metric": "canonical_states_under_iteration", "count": canonical_transport_count, "total": distinct_transport_count, "fraction": canonical_transport_count / max(distinct_transport_count, 1), "note": "augmented transport identities with unique lawful successor identity for every component"})
    rows.append({"scope": "summary", "metric": "noncanonical_branching_states", "count": noncanonical_transport_count, "total": distinct_transport_count, "fraction": noncanonical_transport_count / max(distinct_transport_count, 1), "note": "augmented transport identities whose lawful successors still branch for some component"})

    rows.append({"scope": "comparison", "metric": "recursive_consistency_improvement_vs_v2", "count": canonical_transport_count - int(v2_metrics["recursive_consistency_rate__count"]), "total": distinct_transport_count, "fraction": recursive_consistency_improvement, "note": "change in recursive consistency rate relative to spin_H candidate v2"})
    rows.append({"scope": "comparison", "metric": "branching_reduction_vs_v2", "count": branching_reduction, "total": int(v2_metrics["noncanonical_branching_states__count"]), "fraction": branching_reduction / max(int(v2_metrics["noncanonical_branching_states__count"]), 1), "note": "reduction in noncanonical branching states relative to spin_H candidate v2"})
    rows.append({"scope": "comparison", "metric": "collision_change_vs_v2", "count": collision_change, "total": int(v2_metrics["collision_count__count"]), "fraction": collision_fraction - v2_metrics["collision_count__fraction"], "note": "change in primary-state collisions relative to spin_H candidate v2"})

    rows.append({"scope": "representation", "metric": "angular_identity_preserved", "count": int(angular_preserved), "total": 1, "fraction": float(angular_preserved), "note": "augmented transport lift preserves b exactly and carries phi explicitly"})
    rows.append({"scope": "representation", "metric": "radial_identity_preserved", "count": int(radial_preserved), "total": 1, "fraction": float(radial_preserved), "note": "augmented transport lift preserves r exactly"})
    rows.append({"scope": "representation", "metric": "predictive_structure_preserved", "count": int(predictive_present), "total": 1, "fraction": float(predictive_present), "note": "augmented transport lift carries predictive spin_h4 structure explicitly"})
    rows.append({"scope": "representation", "metric": "recursive_closure_improves_materially", "count": int(recursive_closure_improves_materially), "total": 1, "fraction": float(recursive_closure_improves_materially), "note": "material improvement threshold set at recursive consistency improvement > 0.25"})

    for primary, images in sorted(primary_to_transport.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        ordered_images = sorted(
            images,
            key=lambda t: (
                t.b,
                t.spin_H_candidate.radial_depth,
                t.spin_H_candidate.angular_fiber,
                _spin_str(t.spin_H_candidate.predictive_word),
                _tau_str(t.spin_H_candidate.tau),
            ),
        )
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": f"primary_b{primary.b}_phi{primary.phi}_r{primary.r}",
                "count": len(ordered_images),
                "total": distinct_transport_count,
                "fraction": len(ordered_images) / max(distinct_transport_count, 1),
                "note": "transport_images="
                + "|".join(
                    f"b{transport.b}_phi{_phi_str(transport.spin_H_candidate.angular_fiber)}_r{transport.spin_H_candidate.radial_depth}_spin{_spin_str(transport.spin_H_candidate.predictive_word)}_tau{_tau_str(transport.spin_H_candidate.tau)}"
                    for transport in ordered_images
                ),
            }
        )

    for transport, count in sorted(
        transport_counter.items(),
        key=lambda item: (
            item[0].b,
            item[0].spin_H_candidate.radial_depth,
            item[0].spin_H_candidate.angular_fiber,
            _spin_str(item[0].spin_H_candidate.predictive_word),
            _tau_str(item[0].spin_H_candidate.tau),
        ),
    ):
        rows.append(
            {
                "scope": "transport_distribution",
                "metric": (
                    f"transport_b{transport.b}_phi{_phi_str(transport.spin_H_candidate.angular_fiber)}"
                    f"_r{transport.spin_H_candidate.radial_depth}_spin{_spin_str(transport.spin_H_candidate.predictive_word)}"
                    f"_tau{_tau_str(transport.spin_H_candidate.tau)}"
                ),
                "count": count,
                "total": len(states),
                "fraction": count / max(len(states), 1),
                "note": "reachable augmented operator states carrying this transport identity",
            }
        )

    return rows


def write_spinH_candidate_v3(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CANDIDATE_V3,
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
    summary_rows = summarize_spinH_candidate_v3()
    write_spinH_candidate_v3(summary_rows, OUTPUT_PATH_SPINH_CANDIDATE_V3)
    print(f"wrote {OUTPUT_PATH_SPINH_CANDIDATE_V3}")
    for row in summary_rows:
        if row["scope"] in {"summary", "comparison", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
