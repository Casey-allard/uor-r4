#!/usr/bin/env python3
"""Canonical spin_H core v2 with regressive sigma mode replacing raw word shells."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
import sys

from geometry_native_operator_model_v10 import fiber_phase_lift_component_v10
from geometry_native_operator_model_v12 import (
    bounded_operator_surface_v8,
    radial_direction_v12,
    radial_phi_transport_v12,
    radial_spin_transport_map_v12,
    radial_tau_transport_map_v12,
)
from geometry_native_spinH_candidate_v3 import NativeTauV3, PrimaryChartStateSpinHCandidateV3
from geometry_native_spinH_core_v1 import (
    HCoreV1,
    RhoCoreV1,
    ThetaCoreV1,
    _summary_metrics_from_v2,
    _tau_from_tuple,
    _tau_str,
    _tau_tuple,
)


OUTPUT_PATH_SPINH_CORE_V2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_core_v2.csv"
)
V1_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_core_v1.csv"
)


@dataclass(frozen=True)
class SigmaCoreV2:
    mode_orbit: tuple[tuple[int, ...], ...]
    current_residue: int
    fiber_residue: int
    radial_residue: int


@dataclass(frozen=True)
class SpinHCoreV2:
    theta: ThetaCoreV1
    rho: RhoCoreV1
    sigma: SigmaCoreV2
    h: HCoreV1


def _word_str(word: tuple[int, ...]) -> str:
    return "".join(str(int(bit)) for bit in word)


def _tau_from_state_tuple(tau_tuple: tuple[int, int, int, int]) -> NativeTauV3:
    return _tau_from_tuple(tau_tuple)


def primary_chart_of_core_v2(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


def _canonical_mode_orbit_v2(
    current_word: tuple[int, ...],
    fiber_word: tuple[int, ...],
    radial_word: tuple[int, ...],
) -> SigmaCoreV2:
    orbit = tuple(sorted({tuple(current_word), tuple(fiber_word), tuple(radial_word)}))
    index = {word: idx for idx, word in enumerate(orbit)}
    return SigmaCoreV2(
        mode_orbit=orbit,
        current_residue=index[tuple(current_word)],
        fiber_residue=index[tuple(fiber_word)],
        radial_residue=index[tuple(radial_word)],
    )


def active_transport_lift_core_v2(state: object) -> SpinHCoreV2:
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

    current_word = tuple(int(bit) for bit in state.spin_h.bits)
    fiber_word = tuple(int(bit) for bit in fiber_state.spin_h.bits)
    radial_word = tuple(int(bit) for bit in radial_spin.bits)

    return SpinHCoreV2(
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
        sigma=_canonical_mode_orbit_v2(
            current_word=current_word,
            fiber_word=fiber_word,
            radial_word=radial_word,
        ),
        h=HCoreV1(
            recursive_phase=_tau_tuple(state.tau),
            fiber_recursive_phase=_tau_tuple(fiber_state.tau),
            radial_recursive_phase=_tau_tuple(radial_tau),
            holonomy_bit=int(state.twist),
        ),
    )


def project_spin_h4_v2(core: SpinHCoreV2) -> tuple[int, ...]:
    return core.sigma.mode_orbit[core.sigma.current_residue]


def project_tau_v2(core: SpinHCoreV2) -> NativeTauV3:
    return _tau_from_state_tuple(core.h.recursive_phase)


def project_kappa_v2(core: SpinHCoreV2) -> int:
    return int(core.h.holonomy_bit)


def component_update_signature_v2(source: object, target: object) -> tuple[int, int, int, int]:
    source_core = active_transport_lift_core_v2(source)
    target_core = active_transport_lift_core_v2(target)
    return (
        int(source_core.theta != target_core.theta),
        int(source_core.rho != target_core.rho),
        int(source_core.sigma != target_core.sigma),
        int(source_core.h != target_core.h),
    )


def _summary_metrics_from_v1(path: Path = V1_PATH) -> dict[str, float]:
    csv.field_size_limit(min(sys.maxsize, 10**9))
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "projection", "representation"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_spinH_core_v2(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_core: dict[PrimaryChartStateSpinHCandidateV3, set[SpinHCoreV2]] = defaultdict(set)
    core_to_primary: dict[SpinHCoreV2, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    core_transition_map: dict[SpinHCoreV2, dict[str, set[SpinHCoreV2]]] = defaultdict(lambda: defaultdict(set))
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
        primary = primary_chart_of_core_v2(state)
        core = active_transport_lift_core_v2(state)
        primary_to_core[primary].add(core)
        core_to_primary[core].add(primary)
        spin_class_counter[project_spin_h4_v2(core)] += 1
        radial_class_counter[core.rho.radial_class] += 1
        theta_explicit &= isinstance(core.theta, ThetaCoreV1)
        rho_explicit &= isinstance(core.rho, RhoCoreV1)
        sigma_explicit &= isinstance(core.sigma, SigmaCoreV2)
        h_explicit &= isinstance(core.h, HCoreV1)
        spin_h4_derivable &= project_spin_h4_v2(core) in core.sigma.mode_orbit
        tau_derivable &= project_tau_v2(core) == _tau_from_state_tuple(core.h.recursive_phase)
        kappa_derivable &= project_kappa_v2(core) == core.h.holonomy_bit

    component_signature_counter: Counter[tuple[str, tuple[int, int, int, int]]] = Counter()
    for transition in transitions:
        source_core = active_transport_lift_core_v2(transition.source)
        target_core = active_transport_lift_core_v2(transition.target)
        core_transition_map[source_core][transition.component].add(target_core)
        component_signature_counter[(transition.component, component_update_signature_v2(transition.source, transition.target))] += 1

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

    v1_metrics = _summary_metrics_from_v1()

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states on the bounded lawful H_v8 surface"})
    rows.append({"scope": "summary", "metric": "distinct_spin_H_core_v2_states_reached", "count": distinct_core_count, "total": primary_count, "fraction": distinct_core_count / max(primary_count, 1), "note": "distinct canonical parent transport-side core states with sigma*"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into canonical parent transport-side core states"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_core_count, "total": distinct_core_count, "fraction": recursive_consistency_rate, "note": "fraction of parent core states whose lawful component updates remain canonical"})
    rows.append({"scope": "summary", "metric": "distinct_spin_classes_represented", "count": len(spin_class_counter), "total": len(spin_class_counter), "fraction": 1.0, "note": "distinct projected spin_h4 classes derived from sigma*"})
    rows.append({"scope": "summary", "metric": "distinct_radial_classes_represented", "count": len(radial_class_counter), "total": len(radial_class_counter), "fraction": 1.0, "note": "distinct radial classes represented in rho"})

    rows.append({"scope": "projection", "metric": "spin_h4_derivable_from_parent", "count": int(spin_h4_derivable), "total": 1, "fraction": float(spin_h4_derivable), "note": "Pi_pred(spin_H_core_v2) -> spin_h4 via sigma*.mode_orbit[current_residue]"})
    rows.append({"scope": "projection", "metric": "tau_derivable_from_parent", "count": int(tau_derivable), "total": 1, "fraction": float(tau_derivable), "note": "Pi_rec(spin_H_core_v2) -> tau"})
    rows.append({"scope": "projection", "metric": "kappa_derivable_from_parent", "count": int(kappa_derivable), "total": 1, "fraction": float(kappa_derivable), "note": "Pi_hol(spin_H_core_v2) -> kappa"})

    rows.append({"scope": "representation", "metric": "theta_explicit", "count": int(theta_explicit), "total": 1, "fraction": float(theta_explicit), "note": "theta = (b, phi) exists explicitly"})
    rows.append({"scope": "representation", "metric": "rho_explicit", "count": int(rho_explicit), "total": 1, "fraction": float(rho_explicit), "note": "rho exists explicitly"})
    rows.append({"scope": "representation", "metric": "sigma_star_explicit", "count": int(sigma_explicit), "total": 1, "fraction": float(sigma_explicit), "note": "sigma* exists explicitly as canonical orbit-plus-residue mode"})
    rows.append({"scope": "representation", "metric": "h_explicit", "count": int(h_explicit), "total": 1, "fraction": float(h_explicit), "note": "h exists explicitly"})

    rows.append({"scope": "comparison_vs_v1", "metric": "collision_change", "count": collision_count - int(v1_metrics["collision_count__count"]), "total": int(v1_metrics["collision_count__count"]), "fraction": collision_fraction - v1_metrics["collision_count__fraction"], "note": "change in collisions relative to spin_H_core_v1"})
    rows.append({"scope": "comparison_vs_v1", "metric": "recursive_consistency_change", "count": canonical_core_count - int(v1_metrics["recursive_consistency_rate__count"]), "total": distinct_core_count, "fraction": recursive_consistency_rate - v1_metrics["recursive_consistency_rate__fraction"], "note": "change in recursive consistency relative to spin_H_core_v1"})
    rows.append({"scope": "comparison_vs_v1", "metric": "distinct_state_count_change", "count": distinct_core_count - int(v1_metrics["distinct_spin_H_core_states_reached__count"]), "total": int(v1_metrics["distinct_spin_H_core_states_reached__count"]), "fraction": (distinct_core_count / max(v1_metrics["distinct_spin_H_core_states_reached__count"], 1.0)) - 1.0, "note": "change in distinct parent core states relative to spin_H_core_v1"})

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
                tuple(_word_str(word) for word in core.sigma.mode_orbit),
                core.sigma.current_residue,
                core.sigma.fiber_residue,
                core.sigma.radial_residue,
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
                    f"_sigma(orbit[{','.join(_word_str(word) for word in core.sigma.mode_orbit)}],c{core.sigma.current_residue},f{core.sigma.fiber_residue},r{core.sigma.radial_residue})"
                    f"_h(tau{_tau_str(core.h.recursive_phase)},ftau{_tau_str(core.h.fiber_recursive_phase)},rtau{_tau_str(core.h.radial_recursive_phase)},k{core.h.holonomy_bit})"
                    for core in ordered
                ),
            }
        )

    return rows


def write_spinH_core_v2(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CORE_V2,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=("scope", "metric", "count", "total", "fraction", "note"),
        )
        writer.writeheader()
        writer.writerows(rows)


__all__ = [
    "OUTPUT_PATH_SPINH_CORE_V2",
    "SigmaCoreV2",
    "SpinHCoreV2",
    "active_transport_lift_core_v2",
    "primary_chart_of_core_v2",
    "project_kappa_v2",
    "project_spin_h4_v2",
    "project_tau_v2",
    "summarize_spinH_core_v2",
    "write_spinH_core_v2",
]
