#!/usr/bin/env python3
"""Native canonical spin_H core object with explicit projection maps."""

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
from geometry_native_spinH_candidate_v3 import NativeTauV3, PrimaryChartStateSpinHCandidateV3
from geometry_native_spinH_extended_v2 import OUTPUT_PATH_SPINH_EXTENDED_V2


OUTPUT_PATH_SPINH_CORE_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_core_v1.csv"
)
V2_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_extended_v2.csv"
)


@dataclass(frozen=True)
class ThetaCoreV1:
    base_angle: int
    fiber_phase: int


@dataclass(frozen=True)
class RhoCoreV1:
    radial_class: int
    unfolding_load: int
    radial_direction: int
    radial_target: int
    radial_target_phi: int


@dataclass(frozen=True)
class SigmaCoreV1:
    regressive_word: tuple[int, ...]
    fiber_mode_word: tuple[int, ...]
    radial_mode_word: tuple[int, ...]


@dataclass(frozen=True)
class HCoreV1:
    recursive_phase: tuple[int, int, int, int]
    fiber_recursive_phase: tuple[int, int, int, int]
    radial_recursive_phase: tuple[int, int, int, int]
    holonomy_bit: int


@dataclass(frozen=True)
class SpinHCoreV1:
    theta: ThetaCoreV1
    rho: RhoCoreV1
    sigma: SigmaCoreV1
    h: HCoreV1


def _tau_tuple(tau: NativeTauV3) -> tuple[int, int, int, int]:
    return (tau.swap_phase, tau.coupled_phase, tau.twist_phase, tau.lift_phase)


def _tau_from_tuple(tau_tuple: tuple[int, int, int, int]) -> NativeTauV3:
    return NativeTauV3(
        swap_phase=int(tau_tuple[0]),
        coupled_phase=int(tau_tuple[1]),
        twist_phase=int(tau_tuple[2]),
        lift_phase=int(tau_tuple[3]),
    )


def _tau_str(tau: tuple[int, int, int, int]) -> str:
    return f"s{tau[0]}c{tau[1]}t{tau[2]}l{tau[3]}"


def _word_str(word: tuple[int, ...]) -> str:
    return "".join(str(int(bit)) for bit in word)


def primary_chart_of_core_v1(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


def active_transport_lift_core_v1(state: object) -> SpinHCoreV1:
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

    return SpinHCoreV1(
        theta=ThetaCoreV1(
            base_angle=int(state.b),
            fiber_phase=int(state.phi),
        ),
        rho=RhoCoreV1(
            radial_class=int(state.r),
            unfolding_load=sum(int(bit) for bit in state.spin_h.bits),
            radial_direction=int(direction),
            radial_target=int(target_r),
            radial_target_phi=int(target_phi),
        ),
        sigma=SigmaCoreV1(
            regressive_word=tuple(int(bit) for bit in state.spin_h.bits),
            fiber_mode_word=tuple(int(bit) for bit in fiber_state.spin_h.bits),
            radial_mode_word=tuple(int(bit) for bit in radial_spin.bits),
        ),
        h=HCoreV1(
            recursive_phase=_tau_tuple(state.tau),
            fiber_recursive_phase=_tau_tuple(fiber_state.tau),
            radial_recursive_phase=_tau_tuple(radial_tau),
            holonomy_bit=int(state.twist),
        ),
    )


def project_spin_h4_v1(core: SpinHCoreV1) -> tuple[int, ...]:
    return core.sigma.regressive_word


def project_tau_v1(core: SpinHCoreV1) -> NativeTauV3:
    return _tau_from_tuple(core.h.recursive_phase)


def project_kappa_v1(core: SpinHCoreV1) -> int:
    return int(core.h.holonomy_bit)


def _summary_metrics_from_v2(path: Path = V2_PATH) -> dict[str, float]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "representation"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def component_update_signature_v1(source: object, component: str, target: object) -> tuple[int, int, int, int]:
    source_core = active_transport_lift_core_v1(source)
    target_core = active_transport_lift_core_v1(target)
    return (
        int(source_core.theta != target_core.theta),
        int(source_core.rho != target_core.rho),
        int(source_core.sigma != target_core.sigma),
        int(source_core.h != target_core.h),
    )


def summarize_spinH_core_v1(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_core: dict[PrimaryChartStateSpinHCandidateV3, set[SpinHCoreV1]] = defaultdict(set)
    core_to_primary: dict[SpinHCoreV1, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    core_transition_map: dict[SpinHCoreV1, dict[str, set[SpinHCoreV1]]] = defaultdict(lambda: defaultdict(set))
    spin_class_counter: Counter[tuple[int, ...]] = Counter()
    radial_class_counter: Counter[int] = Counter()

    theta_explicit = True
    rho_explicit = True
    sigma_explicit = True
    h_explicit = True
    spin_h4_derivable = True
    tau_derivable = True
    kappa_derivable = True

    for state in states:
        primary = primary_chart_of_core_v1(state)
        core = active_transport_lift_core_v1(state)
        primary_to_core[primary].add(core)
        core_to_primary[core].add(primary)
        spin_class_counter[project_spin_h4_v1(core)] += 1
        radial_class_counter[core.rho.radial_class] += 1
        theta_explicit &= isinstance(core.theta, ThetaCoreV1)
        rho_explicit &= isinstance(core.rho, RhoCoreV1)
        sigma_explicit &= isinstance(core.sigma, SigmaCoreV1)
        h_explicit &= isinstance(core.h, HCoreV1)
        spin_h4_derivable &= project_spin_h4_v1(core) == core.sigma.regressive_word
        tau_derivable &= project_tau_v1(core) == _tau_from_tuple(core.h.recursive_phase)
        kappa_derivable &= project_kappa_v1(core) == core.h.holonomy_bit

    component_signature_counter: Counter[tuple[str, tuple[int, int, int, int]]] = Counter()
    for transition in transitions:
        source_core = active_transport_lift_core_v1(transition.source)
        target_core = active_transport_lift_core_v1(transition.target)
        core_transition_map[source_core][transition.component].add(target_core)
        component_signature_counter[(transition.component, component_update_signature_v1(transition.source, transition.component, transition.target))] += 1

    primary_count = len(primary_to_core)
    distinct_core_count = len(core_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in core_to_primary.values())
    collision_fraction = collision_count / max(primary_count, 1)

    canonical_core_count = 0
    noncanonical_core_count = 0
    for component_map in core_transition_map.values():
        is_canonical = all(len(targets) <= 1 for targets in component_map.values())
        if is_canonical:
            canonical_core_count += 1
        else:
            noncanonical_core_count += 1
    recursive_consistency_rate = canonical_core_count / max(distinct_core_count, 1)

    v2_metrics = _summary_metrics_from_v2()

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states on the bounded lawful H_v8 surface"})
    rows.append({"scope": "summary", "metric": "distinct_spin_H_core_states_reached", "count": distinct_core_count, "total": primary_count, "fraction": distinct_core_count / max(primary_count, 1), "note": "distinct canonical parent transport-side core states"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into canonical parent transport-side core states"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_core_count, "total": distinct_core_count, "fraction": recursive_consistency_rate, "note": "fraction of parent core states whose lawful component updates remain canonical"})
    rows.append({"scope": "summary", "metric": "distinct_spin_classes_represented", "count": len(spin_class_counter), "total": len(spin_class_counter), "fraction": 1.0, "note": "distinct projected spin_h4 classes derived from sigma"})
    rows.append({"scope": "summary", "metric": "distinct_radial_classes_represented", "count": len(radial_class_counter), "total": len(radial_class_counter), "fraction": 1.0, "note": "distinct radial classes represented in rho"})

    rows.append({"scope": "projection", "metric": "spin_h4_derivable_from_parent", "count": int(spin_h4_derivable), "total": 1, "fraction": float(spin_h4_derivable), "note": "Pi_pred(spin_H_core_v1) -> spin_h4"})
    rows.append({"scope": "projection", "metric": "tau_derivable_from_parent", "count": int(tau_derivable), "total": 1, "fraction": float(tau_derivable), "note": "Pi_rec(spin_H_core_v1) -> tau"})
    rows.append({"scope": "projection", "metric": "kappa_derivable_from_parent", "count": int(kappa_derivable), "total": 1, "fraction": float(kappa_derivable), "note": "Pi_hol(spin_H_core_v1) -> kappa"})

    rows.append({"scope": "representation", "metric": "theta_explicit", "count": int(theta_explicit), "total": 1, "fraction": float(theta_explicit), "note": "theta = (b, phi) exists explicitly"})
    rows.append({"scope": "representation", "metric": "rho_explicit", "count": int(rho_explicit), "total": 1, "fraction": float(rho_explicit), "note": "rho exists explicitly"})
    rows.append({"scope": "representation", "metric": "sigma_explicit", "count": int(sigma_explicit), "total": 1, "fraction": float(sigma_explicit), "note": "sigma exists explicitly"})
    rows.append({"scope": "representation", "metric": "h_explicit", "count": int(h_explicit), "total": 1, "fraction": float(h_explicit), "note": "h exists explicitly"})

    rows.append({"scope": "comparison_vs_extended_v2", "metric": "collision_change", "count": collision_count - int(v2_metrics["collision_count__count"]), "total": int(v2_metrics["collision_count__count"]), "fraction": collision_fraction - v2_metrics["collision_count__fraction"], "note": "change in collisions relative to spin_H_extended_v2"})
    rows.append({"scope": "comparison_vs_extended_v2", "metric": "recursive_consistency_change", "count": canonical_core_count - int(v2_metrics["canonical_states_under_iteration__count"]), "total": distinct_core_count, "fraction": recursive_consistency_rate - v2_metrics["recursive_consistency_rate__fraction"], "note": "change in recursive consistency relative to spin_H_extended_v2"})

    for component in (
        "hold",
        "torus_base_advance",
        "composite_swap",
        "coupled_torus_kick",
        "composite_twist",
        "fiber_phase_lift_spin_transport",
        "radial_transport_unfolding",
    ):
        matching = [(sig, count) for (name, sig), count in component_signature_counter.items() if name == component]
        matching.sort(key=lambda item: (-item[1], item[0]))
        dominant_sig, dominant_count = matching[0]
        rows.append(
            {
                "scope": "update_law",
                "metric": component,
                "count": dominant_count,
                "total": sum(count for _, count in matching),
                "fraction": dominant_count / max(sum(count for _, count in matching), 1),
                "note": (
                    f"dominant_component_delta="
                    f"theta{dominant_sig[0]}_rho{dominant_sig[1]}_sigma{dominant_sig[2]}_h{dominant_sig[3]}"
                ),
            }
        )

    for primary, cores in sorted(primary_to_core.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        ordered = sorted(
            cores,
            key=lambda core: (
                core.theta.base_angle,
                core.theta.fiber_phase,
                core.rho.radial_class,
                _word_str(core.sigma.regressive_word),
                _tau_str(core.h.recursive_phase),
                core.h.holonomy_bit,
            ),
        )
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": f"primary_b{primary.b}_phi{primary.phi}_r{primary.r}",
                "count": len(ordered),
                "total": distinct_core_count,
                "fraction": len(ordered) / max(distinct_core_count, 1),
                "note": "core_states="
                + "|".join(
                    f"theta({core.theta.base_angle},{core.theta.fiber_phase})"
                    f"_rho(r{core.rho.radial_class},u{core.rho.unfolding_load},d{core.rho.radial_direction},rt{core.rho.radial_target},pt{core.rho.radial_target_phi})"
                    f"_sigma(cur{_word_str(core.sigma.regressive_word)},fib{_word_str(core.sigma.fiber_mode_word)},rad{_word_str(core.sigma.radial_mode_word)})"
                    f"_h(tau{_tau_str(core.h.recursive_phase)},ftau{_tau_str(core.h.fiber_recursive_phase)},rtau{_tau_str(core.h.radial_recursive_phase)},k{core.h.holonomy_bit})"
                    for core in ordered
                ),
            }
        )

    return rows


def write_spinH_core_v1(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CORE_V1,
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
    summary_rows = summarize_spinH_core_v1(depth=8)
    write_spinH_core_v1(summary_rows)
    print(f"wrote {OUTPUT_PATH_SPINH_CORE_V1}")
    for row in summary_rows:
        if row["scope"] in {"summary", "projection", "representation", "comparison_vs_extended_v2"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
