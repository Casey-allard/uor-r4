#!/usr/bin/env python3
"""Extended transport-side state with residual holonomy bit kappa."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v10 import fiber_phase_lift_component_v10
from geometry_native_operator_model_v12 import (
    bounded_operator_surface_v8,
    radial_direction_v12,
    radial_phi_transport_v12,
    radial_spin_transport_map_v12,
    radial_tau_transport_map_v12,
)
from geometry_native_spinH_candidate_v1 import _spin_str
from geometry_native_spinH_candidate_v3 import NativeTauV3, PrimaryChartStateSpinHCandidateV3
from geometry_native_spinH_extended_v1 import (
    AngularIdentityV1,
    ClosureShellV1,
    PredictiveShellV1,
    RadialUnfoldingIdentityV1,
)


OUTPUT_PATH_SPINH_EXTENDED_V2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_extended_v2.csv"
)
V1_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_extended_v1.csv"
)


@dataclass(frozen=True)
class SpinHExtendedV2:
    angular_identity: AngularIdentityV1
    radial_unfolding: RadialUnfoldingIdentityV1
    predictive_shell: PredictiveShellV1
    closure_shell: ClosureShellV1
    kappa: int


@dataclass(frozen=True)
class TransportStateSpinHExtendedV2:
    spin_H_extended: SpinHExtendedV2


def _tau_tuple(tau: NativeTauV3) -> tuple[int, int, int, int]:
    return (tau.swap_phase, tau.coupled_phase, tau.twist_phase, tau.lift_phase)


def _tau_str(tau: tuple[int, int, int, int]) -> str:
    return f"s{tau[0]}c{tau[1]}t{tau[2]}l{tau[3]}"


def _predictive_str(predictive: PredictiveShellV1) -> str:
    return (
        f"cur{''.join(str(bit) for bit in predictive.current_spin)}"
        f"_fib{''.join(str(bit) for bit in predictive.fiber_spin)}"
        f"_rad{''.join(str(bit) for bit in predictive.radial_spin)}"
    )


def primary_chart_of_extended_v2(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


def active_transport_lift_extended_v2(state: object) -> TransportStateSpinHExtendedV2:
    direction = radial_direction_v12(state)
    target_r = (state.r + direction) % 3
    target_phi = radial_phi_transport_v12(state, target_r=target_r, direction=direction)

    fiber_state = fiber_phase_lift_component_v10(state)
    radial_spin = radial_spin_transport_map_v12(
        state.spin_h,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        direction=direction,
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    radial_tau = radial_tau_transport_map_v12(
        state.tau,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        spin_h=state.spin_h,
        direction=direction,
    )

    extended = SpinHExtendedV2(
        angular_identity=AngularIdentityV1(
            base_angle=int(state.b),
            fiber_phase=int(state.phi),
        ),
        radial_unfolding=RadialUnfoldingIdentityV1(
            radial_class=int(state.r),
            unfolding_load=sum(int(bit) for bit in state.spin_h.bits),
            radial_direction=int(direction),
            radial_target=int(target_r),
            radial_target_phi=int(target_phi),
        ),
        predictive_shell=PredictiveShellV1(
            current_spin=tuple(int(bit) for bit in state.spin_h.bits),
            fiber_spin=tuple(int(bit) for bit in fiber_state.spin_h.bits),
            radial_spin=tuple(int(bit) for bit in radial_spin.bits),
        ),
        closure_shell=ClosureShellV1(
            current_tau=_tau_tuple(state.tau),
            fiber_tau=_tau_tuple(fiber_state.tau),
            radial_tau=_tau_tuple(radial_tau),
        ),
        kappa=int(state.twist),
    )
    return TransportStateSpinHExtendedV2(spin_H_extended=extended)


def _summary_metrics_from_v1(path: Path = V1_PATH) -> dict[str, float]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "representation"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_spinH_extended_v2(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_transport: dict[PrimaryChartStateSpinHCandidateV3, set[TransportStateSpinHExtendedV2]] = defaultdict(set)
    transport_to_primary: dict[TransportStateSpinHExtendedV2, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    transport_counter: Counter[TransportStateSpinHExtendedV2] = Counter()
    transport_transition_map: dict[TransportStateSpinHExtendedV2, dict[str, set[TransportStateSpinHExtendedV2]]] = defaultdict(lambda: defaultdict(set))
    spin_repr_counter: Counter[tuple[tuple[int, ...], tuple[int, ...], tuple[int, ...], int]] = Counter()
    radial_repr_counter: Counter[int] = Counter()

    angular_preserved = True
    radial_preserved = True
    predictive_preserved = True
    recursive_preserved = True
    kappa_active = True

    for state in states:
        primary = primary_chart_of_extended_v2(state)
        transport = active_transport_lift_extended_v2(state)
        primary_to_transport[primary].add(transport)
        transport_to_primary[transport].add(primary)
        transport_counter[transport] += 1
        spin_repr_counter[
            (
                transport.spin_H_extended.predictive_shell.current_spin,
                transport.spin_H_extended.predictive_shell.fiber_spin,
                transport.spin_H_extended.predictive_shell.radial_spin,
                transport.spin_H_extended.kappa,
            )
        ] += 1
        radial_repr_counter[transport.spin_H_extended.radial_unfolding.radial_class] += 1
        angular_preserved &= (
            transport.spin_H_extended.angular_identity.base_angle == primary.b
            and transport.spin_H_extended.angular_identity.fiber_phase == primary.phi
        )
        radial_preserved &= transport.spin_H_extended.radial_unfolding.radial_class == primary.r
        predictive_preserved &= len(transport.spin_H_extended.predictive_shell.current_spin) > 0
        recursive_preserved &= len(transport.spin_H_extended.closure_shell.current_tau) == 4
        kappa_active &= transport.spin_H_extended.kappa == int(state.twist)

    for transition in transitions:
        source_transport = active_transport_lift_extended_v2(transition.source)
        target_transport = active_transport_lift_extended_v2(transition.target)
        transport_transition_map[source_transport][transition.component].add(target_transport)

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

    v1_metrics = _summary_metrics_from_v1()
    recursive_consistency_improvement = recursive_consistency_rate - v1_metrics["recursive_consistency_rate__fraction"]
    branching_reduction = int(v1_metrics["noncanonical_branching_states__count"]) - noncanonical_transport_count
    collision_change = collision_count - int(v1_metrics["collision_count__count"])

    previous_two_preimage_pattern_gone = True
    for preimages in transport_to_primary.values():
        if len(preimages) == 2:
            previous_two_preimage_pattern_gone = False
            break

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states on the bounded lawful H_v8 surface"})
    rows.append({"scope": "summary", "metric": "distinct_transport_identities_reached", "count": distinct_transport_count, "total": primary_count, "fraction": distinct_transport_count / max(primary_count, 1), "note": "distinct coherent transport identities under spin_H_extended_v2"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into coherent extended transport identities with kappa"})
    rows.append({"scope": "summary", "metric": "ambiguity_count", "count": ambiguity_count, "total": primary_count, "fraction": ambiguity_fraction, "note": "primary chart states with more than one coherent extended transport identity"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_transport_count, "total": distinct_transport_count, "fraction": recursive_consistency_rate, "note": "fraction of coherent transport identities whose lawful component updates remain canonical"})
    rows.append({"scope": "summary", "metric": "canonical_states_under_iteration", "count": canonical_transport_count, "total": distinct_transport_count, "fraction": canonical_transport_count / max(distinct_transport_count, 1), "note": "coherent transport identities with unique lawful successor identity for every component"})
    rows.append({"scope": "summary", "metric": "noncanonical_branching_states", "count": noncanonical_transport_count, "total": distinct_transport_count, "fraction": noncanonical_transport_count / max(distinct_transport_count, 1), "note": "coherent transport identities whose lawful successors still branch for some component"})
    rows.append({"scope": "summary", "metric": "distinct_spin_classes_represented", "count": len(spin_repr_counter), "total": len(spin_repr_counter), "fraction": 1.0, "note": "number of distinct predictive-plus-kappa spin classes represented"})
    rows.append({"scope": "summary", "metric": "distinct_radial_classes_represented", "count": len(radial_repr_counter), "total": len(radial_repr_counter), "fraction": 1.0, "note": "number of distinct radial classes represented in extended transport identities"})

    rows.append({"scope": "comparison_vs_v1", "metric": "recursive_consistency_improvement", "count": canonical_transport_count - int(v1_metrics["canonical_states_under_iteration__count"]), "total": distinct_transport_count, "fraction": recursive_consistency_improvement, "note": "change in recursive consistency rate relative to spin_H_extended_v1"})
    rows.append({"scope": "comparison_vs_v1", "metric": "branching_reduction", "count": branching_reduction, "total": int(v1_metrics["noncanonical_branching_states__count"]), "fraction": branching_reduction / max(int(v1_metrics["noncanonical_branching_states__count"]), 1), "note": "reduction in noncanonical branching states relative to spin_H_extended_v1"})
    rows.append({"scope": "comparison_vs_v1", "metric": "collision_change", "count": collision_change, "total": int(v1_metrics["collision_count__count"]), "fraction": collision_fraction - v1_metrics["collision_count__fraction"], "note": "change in primary-state collisions relative to spin_H_extended_v1"})

    rows.append({"scope": "representation", "metric": "angular_identity_preserved", "count": int(angular_preserved), "total": 1, "fraction": float(angular_preserved), "note": "extended state preserves angular identity explicitly"})
    rows.append({"scope": "representation", "metric": "radial_identity_preserved", "count": int(radial_preserved), "total": 1, "fraction": float(radial_preserved), "note": "extended state preserves radial class explicitly"})
    rows.append({"scope": "representation", "metric": "predictive_structure_preserved", "count": int(predictive_preserved), "total": 1, "fraction": float(predictive_preserved), "note": "extended state preserves predictive structure explicitly"})
    rows.append({"scope": "representation", "metric": "recursive_closure_preserved", "count": int(recursive_preserved and recursive_consistency_rate == 1.0), "total": 1, "fraction": float(recursive_preserved and recursive_consistency_rate == 1.0), "note": "extended state preserves recursive closure structure explicitly"})
    rows.append({"scope": "representation", "metric": "kappa_active_in_transport_identity", "count": int(kappa_active), "total": 1, "fraction": float(kappa_active), "note": "kappa is carried explicitly and equals the hidden operator-side twist parity"})
    rows.append({"scope": "representation", "metric": "previous_two_preimage_branching_pattern_gone", "count": int(previous_two_preimage_pattern_gone), "total": 1, "fraction": float(previous_two_preimage_pattern_gone), "note": "the prior two-preimage hidden-twist branching pattern is absent"})

    for primary, images in sorted(primary_to_transport.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        ordered_images = sorted(
            images,
            key=lambda t: (
                t.spin_H_extended.angular_identity.base_angle,
                t.spin_H_extended.angular_identity.fiber_phase,
                t.spin_H_extended.radial_unfolding.radial_class,
                _predictive_str(t.spin_H_extended.predictive_shell),
                _tau_str(t.spin_H_extended.closure_shell.current_tau),
                t.spin_H_extended.kappa,
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
                    f"ang({img.spin_H_extended.angular_identity.base_angle},{img.spin_H_extended.angular_identity.fiber_phase})"
                    f"_rad(r{img.spin_H_extended.radial_unfolding.radial_class},u{img.spin_H_extended.radial_unfolding.unfolding_load},d{img.spin_H_extended.radial_unfolding.radial_direction},rt{img.spin_H_extended.radial_unfolding.radial_target},pt{img.spin_H_extended.radial_unfolding.radial_target_phi})"
                    f"_pred({_predictive_str(img.spin_H_extended.predictive_shell)})"
                    f"_clos({_tau_str(img.spin_H_extended.closure_shell.current_tau)}|{_tau_str(img.spin_H_extended.closure_shell.fiber_tau)}|{_tau_str(img.spin_H_extended.closure_shell.radial_tau)})"
                    f"_k{img.spin_H_extended.kappa}"
                    for img in ordered_images
                ),
            }
        )

    return rows


def write_spinH_extended_v2(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_EXTENDED_V2,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_spinH_extended_v2(depth=8)
    write_spinH_extended_v2(summary_rows)
    print(f"wrote {OUTPUT_PATH_SPINH_EXTENDED_V2}")
    for row in summary_rows:
        if row["scope"] in {"summary", "comparison_vs_v1", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
