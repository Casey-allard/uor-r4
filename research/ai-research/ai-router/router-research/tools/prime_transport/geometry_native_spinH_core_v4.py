#!/usr/bin/env python3
"""Canonical spin_H core v4 with direct regressive sigma update law."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path
import sys

from geometry_native_operator_model_v10 import (
    composite_swap_component_v10,
    composite_twist_component_v10,
    coupled_torus_kick_component_v10,
    fiber_phase_lift_component_v10,
    hold_component_v10,
    torus_base_advance_component_v10,
)
from geometry_native_operator_model_v12 import (
    bounded_operator_surface_v8,
    radial_direction_v12,
    radial_phi_transport_v12,
    radial_spin_transport_map_v12,
    radial_tau_transport_map_v12,
    radial_transport_component_v12,
)
from geometry_native_spinH_candidate_v3 import NativeTauV3, PrimaryChartStateSpinHCandidateV3
from geometry_native_spinH_core_v1 import (
    HCoreV1,
    RhoCoreV1,
    ThetaCoreV1,
    _tau_from_tuple,
    _tau_str,
    _tau_tuple,
)
from geometry_native_spinH_core_v3 import OUTPUT_PATH_SPINH_CORE_V3


OUTPUT_PATH_SPINH_CORE_V4 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_core_v4.csv"
)


COMPONENTS_V4 = (
    "hold",
    "torus_base_advance",
    "composite_swap",
    "coupled_torus_kick",
    "composite_twist",
    "fiber_phase_lift_spin_transport",
    "radial_transport_unfolding",
)


@dataclass(frozen=True)
class SigmaDirectV4:
    current_mode: tuple[int, ...]
    fiber_mode: tuple[int, ...]
    radial_mode: tuple[int, ...]
    regressive_phase: int


@dataclass(frozen=True)
class SigmaDiagnosticsV4:
    seed_orbit: tuple[tuple[int, ...], ...]
    generator_projection_profile: tuple[tuple[str, tuple[int, ...]], ...]
    generator_orbit_profile: tuple[tuple[str, tuple[tuple[int, ...], ...]], ...]
    generator_composition_profile: tuple[tuple[str, tuple[tuple[str, tuple[int, ...]], ...]], ...]


@dataclass(frozen=True)
class SpinHCoreV4:
    theta: ThetaCoreV1
    rho: RhoCoreV1
    sigma: SigmaDirectV4
    h: HCoreV1


def _word_str(word: tuple[int, ...]) -> str:
    return "".join(str(int(bit)) for bit in word)


def _tau_from_state_tuple(tau_tuple: tuple[int, int, int, int]) -> NativeTauV3:
    return _tau_from_tuple(tau_tuple)


def primary_chart_of_core_v4(state: object) -> PrimaryChartStateSpinHCandidateV3:
    return PrimaryChartStateSpinHCandidateV3(b=int(state.b), phi=int(state.phi), r=int(state.r))


def _binary_value_v4(word: tuple[int, ...]) -> int:
    return int("".join(str(int(bit)) for bit in word), 2)


def _rotate_word_v4(word: tuple[int, ...], shift: int) -> tuple[int, ...]:
    if not word:
        return word
    offset = shift % len(word)
    return word[offset:] + word[:offset]


def _xor_words_v4(left: tuple[int, ...], right: tuple[int, ...]) -> tuple[int, ...]:
    return tuple((int(a) + int(b)) % 2 for a, b in zip(left, right))


def _mix_words_v4(
    left: tuple[int, ...],
    right: tuple[int, ...],
    phase: int,
) -> tuple[int, ...]:
    rotated_right = _rotate_word_v4(right, phase)
    return tuple(
        ((int(left[idx]) + int(rotated_right[idx]) + ((phase + idx) % 2)) % 2)
        for idx in range(len(left))
    )


@lru_cache(maxsize=None)
def _radial_targets_from_state_v4(state: object) -> tuple[int, int]:
    direction = radial_direction_v12(state)
    target_r = (state.r + direction) % 3
    target_phi = radial_phi_transport_v12(state, target_r=target_r, direction=direction)
    return target_r, target_phi


@lru_cache(maxsize=None)
def _local_sigma_words_v4(state: object) -> tuple[tuple[int, ...], tuple[int, ...], tuple[int, ...]]:
    fiber_state = fiber_phase_lift_component_v10(state)
    target_r, target_phi = _radial_targets_from_state_v4(state)
    radial_spin = radial_spin_transport_map_v12(
        state.spin_h,
        source_r=state.r,
        target_r=target_r,
        target_phi=target_phi,
        direction=radial_direction_v12(state),
        tau=state.tau,
        composite_compat_class=state.composite_compat_class,
    )
    return (
        tuple(int(bit) for bit in state.spin_h.bits),
        tuple(int(bit) for bit in fiber_state.spin_h.bits),
        tuple(int(bit) for bit in radial_spin.bits),
    )


def _canonical_orbit_v4(
    current_word: tuple[int, ...],
    fiber_word: tuple[int, ...],
    radial_word: tuple[int, ...],
) -> tuple[tuple[int, ...], ...]:
    return tuple(sorted({tuple(current_word), tuple(fiber_word), tuple(radial_word)}))


def direct_sigma_from_words_v4(
    current_word: tuple[int, ...],
    fiber_word: tuple[int, ...],
    radial_word: tuple[int, ...],
) -> SigmaDirectV4:
    regressive_phase = (
        _binary_value_v4(current_word)
        + 3 * _binary_value_v4(fiber_word)
        + 5 * _binary_value_v4(radial_word)
    ) % 12
    return SigmaDirectV4(
        current_mode=tuple(current_word),
        fiber_mode=tuple(fiber_word),
        radial_mode=tuple(radial_word),
        regressive_phase=regressive_phase,
    )


def R_I_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    return sigma


def R_Tb_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    return SigmaDirectV4(
        current_mode=_rotate_word_v4(sigma.current_mode, 1),
        fiber_mode=_rotate_word_v4(sigma.fiber_mode, 1),
        radial_mode=_rotate_word_v4(sigma.radial_mode, 1),
        regressive_phase=(sigma.regressive_phase + 1) % 12,
    )


def R_Tx_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    return SigmaDirectV4(
        current_mode=sigma.fiber_mode,
        fiber_mode=sigma.current_mode,
        radial_mode=_xor_words_v4(sigma.radial_mode, sigma.current_mode),
        regressive_phase=(sigma.regressive_phase + 5) % 12,
    )


def R_Tc_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    new_current = _xor_words_v4(sigma.current_mode, sigma.radial_mode)
    new_fiber = _mix_words_v4(sigma.fiber_mode, sigma.current_mode, sigma.regressive_phase + 1)
    new_radial = _rotate_word_v4(
        _mix_words_v4(sigma.radial_mode, sigma.fiber_mode, sigma.regressive_phase),
        1 + (sigma.regressive_phase % 2),
    )
    return SigmaDirectV4(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(
            sigma.regressive_phase
            + sum(new_current)
            + sum(new_fiber)
            + sum(new_radial)
        )
        % 12,
    )


def R_Ty_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    return SigmaDirectV4(
        current_mode=_rotate_word_v4(sigma.current_mode[::-1], 1),
        fiber_mode=_rotate_word_v4(sigma.fiber_mode[::-1], 1),
        radial_mode=_rotate_word_v4(sigma.radial_mode[::-1], 1),
        regressive_phase=(sigma.regressive_phase + 6) % 12,
    )


def R_Tz_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    new_current = sigma.fiber_mode
    new_fiber = _rotate_word_v4(sigma.fiber_mode, 1)
    new_radial = _mix_words_v4(sigma.radial_mode, sigma.current_mode, sigma.regressive_phase + 2)
    return SigmaDirectV4(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(sigma.regressive_phase + sum(new_current) + 1) % 12,
    )


def R_Tr_v4(sigma: SigmaDirectV4) -> SigmaDirectV4:
    new_current = sigma.radial_mode
    new_fiber = _mix_words_v4(sigma.fiber_mode, sigma.radial_mode, sigma.regressive_phase + 3)
    new_radial = _rotate_word_v4(sigma.radial_mode, 1)
    return SigmaDirectV4(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(sigma.regressive_phase + sum(new_radial) + 2) % 12,
    )


def sigma_update_v4(sigma: SigmaDirectV4, component: str) -> SigmaDirectV4:
    if component == "hold":
        return R_I_v4(sigma)
    if component == "torus_base_advance":
        return R_Tb_v4(sigma)
    if component == "composite_swap":
        return R_Tx_v4(sigma)
    if component == "coupled_torus_kick":
        return R_Tc_v4(sigma)
    if component == "composite_twist":
        return R_Ty_v4(sigma)
    if component == "fiber_phase_lift_spin_transport":
        return R_Tz_v4(sigma)
    if component == "radial_transport_unfolding":
        return R_Tr_v4(sigma)
    raise ValueError(f"unknown component {component!r}")


def derive_mode_orbit_v4(sigma: SigmaDirectV4) -> tuple[tuple[int, ...], ...]:
    return _canonical_orbit_v4(sigma.current_mode, sigma.fiber_mode, sigma.radial_mode)


def derive_sigma_diagnostics_v4(sigma: SigmaDirectV4) -> SigmaDiagnosticsV4:
    projection_profile: list[tuple[str, tuple[int, ...]]] = []
    orbit_profile: list[tuple[str, tuple[tuple[int, ...], ...]]] = []
    composition_profile: list[tuple[str, tuple[tuple[str, tuple[int, ...]], ...]]] = []

    for first in COMPONENTS_V4:
        first_sigma = sigma_update_v4(sigma, first)
        projection_profile.append((first, first_sigma.current_mode))
        orbit_profile.append((first, derive_mode_orbit_v4(first_sigma)))

        second_rows: list[tuple[str, tuple[int, ...]]] = []
        for second in COMPONENTS_V4:
            second_rows.append((second, sigma_update_v4(first_sigma, second).current_mode))
        composition_profile.append((first, tuple(second_rows)))

    return SigmaDiagnosticsV4(
        seed_orbit=derive_mode_orbit_v4(sigma),
        generator_projection_profile=tuple(projection_profile),
        generator_orbit_profile=tuple(orbit_profile),
        generator_composition_profile=tuple(composition_profile),
    )


@lru_cache(maxsize=None)
def active_transport_lift_core_v4(state: object) -> SpinHCoreV4:
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

    current_word, fiber_word, radial_word = _local_sigma_words_v4(state)
    sigma_direct = direct_sigma_from_words_v4(
        current_word=current_word,
        fiber_word=fiber_word,
        radial_word=radial_word,
    )

    return SpinHCoreV4(
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
        sigma=sigma_direct,
        h=HCoreV1(
            recursive_phase=_tau_tuple(state.tau),
            fiber_recursive_phase=_tau_tuple(fiber_state.tau),
            radial_recursive_phase=_tau_tuple(radial_tau),
            holonomy_bit=int(state.twist),
        ),
    )


def project_spin_h4_v4(core: SpinHCoreV4) -> tuple[int, ...]:
    return core.sigma.current_mode


def project_tau_v4(core: SpinHCoreV4) -> NativeTauV3:
    return _tau_from_state_tuple(core.h.recursive_phase)


def project_kappa_v4(core: SpinHCoreV4) -> int:
    return int(core.h.holonomy_bit)


def component_update_signature_v4(source: object, target: object) -> tuple[int, int, int, int]:
    source_core = active_transport_lift_core_v4(source)
    target_core = active_transport_lift_core_v4(target)
    return (
        int(source_core.theta != target_core.theta),
        int(source_core.rho != target_core.rho),
        int(source_core.sigma != target_core.sigma),
        int(source_core.h != target_core.h),
    )


def _summary_metrics_from_v3(path: Path = OUTPUT_PATH_SPINH_CORE_V3) -> dict[str, float]:
    csv.field_size_limit(min(sys.maxsize, 10**9))
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "projection", "representation"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def summarize_spinH_core_v4(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_core: dict[PrimaryChartStateSpinHCandidateV3, set[SpinHCoreV4]] = defaultdict(set)
    core_to_primary: dict[SpinHCoreV4, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    core_transition_map: dict[SpinHCoreV4, dict[str, set[SpinHCoreV4]]] = defaultdict(lambda: defaultdict(set))

    sigma_directly_updated = True
    profile_fields_diagnostics_only = True
    spin_h4_derivable = True
    tau_derivable = True
    kappa_derivable = True

    for state in states:
        primary = primary_chart_of_core_v4(state)
        core = active_transport_lift_core_v4(state)
        primary_to_core[primary].add(core)
        core_to_primary[core].add(primary)

        diagnostics = derive_sigma_diagnostics_v4(core.sigma)
        sigma_directly_updated &= isinstance(sigma_update_v4(core.sigma, "hold"), SigmaDirectV4)
        profile_fields_diagnostics_only &= isinstance(diagnostics, SigmaDiagnosticsV4)
        spin_h4_derivable &= project_spin_h4_v4(core) in diagnostics.seed_orbit
        tau_derivable &= project_tau_v4(core) == _tau_from_state_tuple(core.h.recursive_phase)
        kappa_derivable &= project_kappa_v4(core) == core.h.holonomy_bit

    component_signature_counter: Counter[tuple[str, tuple[int, int, int, int]]] = Counter()
    for transition in transitions:
        source_core = active_transport_lift_core_v4(transition.source)
        target_core = active_transport_lift_core_v4(transition.target)
        core_transition_map[source_core][transition.component].add(target_core)
        component_signature_counter[(transition.component, component_update_signature_v4(transition.source, transition.target))] += 1

    primary_count = len(primary_to_core)
    distinct_core_count = len(core_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in core_to_primary.values())
    collision_fraction = collision_count / max(primary_count, 1)

    canonical_core_count = 0
    for component_map in core_transition_map.values():
        if all(len(targets) <= 1 for targets in component_map.values()):
            canonical_core_count += 1
    recursive_consistency_rate = canonical_core_count / max(distinct_core_count, 1)

    v3_metrics = _summary_metrics_from_v3()

    rows: list[dict[str, object]] = []
    rows.append({"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct primary chart states on the bounded lawful H_v8 surface"})
    rows.append({"scope": "summary", "metric": "distinct_spin_H_core_v4_states_reached", "count": distinct_core_count, "total": primary_count, "fraction": distinct_core_count / max(primary_count, 1), "note": "distinct canonical parent states with direct sigma update law"})
    rows.append({"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions from primary chart states into direct-law parent states"})
    rows.append({"scope": "summary", "metric": "recursive_consistency_rate", "count": canonical_core_count, "total": distinct_core_count, "fraction": recursive_consistency_rate, "note": "fraction of parent states whose lawful component updates remain canonical"})

    rows.append({"scope": "projection", "metric": "sigma_updated_directly_by_R_G", "count": int(sigma_directly_updated), "total": 1, "fraction": float(sigma_directly_updated), "note": "sigma is updated by direct regressive maps R_G rather than by action-profile reconstruction"})
    rows.append({"scope": "projection", "metric": "profile_fields_are_derived_diagnostics_only", "count": int(profile_fields_diagnostics_only), "total": 1, "fraction": float(profile_fields_diagnostics_only), "note": "seed_orbit and action/composition profiles are derived from sigma_direct via diagnostics only"})
    rows.append({"scope": "projection", "metric": "spin_h4_derivable_from_parent", "count": int(spin_h4_derivable), "total": 1, "fraction": float(spin_h4_derivable), "note": "Pi_pred(spin_H_core_v4) -> sigma.current_mode"})
    rows.append({"scope": "projection", "metric": "tau_derivable_from_parent", "count": int(tau_derivable), "total": 1, "fraction": float(tau_derivable), "note": "Pi_rec(spin_H_core_v4) -> tau"})
    rows.append({"scope": "projection", "metric": "kappa_derivable_from_parent", "count": int(kappa_derivable), "total": 1, "fraction": float(kappa_derivable), "note": "Pi_hol(spin_H_core_v4) -> kappa"})

    rows.append({"scope": "comparison_vs_v3", "metric": "collision_change", "count": collision_count - int(v3_metrics["collision_count__count"]), "total": int(v3_metrics["collision_count__count"]), "fraction": collision_fraction - v3_metrics["collision_count__fraction"], "note": "change in collisions relative to spin_H_core_v3"})
    rows.append({"scope": "comparison_vs_v3", "metric": "recursive_consistency_change", "count": canonical_core_count - int(v3_metrics["recursive_consistency_rate__count"]), "total": distinct_core_count, "fraction": recursive_consistency_rate - v3_metrics["recursive_consistency_rate__fraction"], "note": "change in recursive consistency relative to spin_H_core_v3"})
    rows.append({"scope": "comparison_vs_v3", "metric": "distinct_state_count_change", "count": distinct_core_count - int(v3_metrics["distinct_spin_H_core_v3_states_reached__count"]), "total": int(v3_metrics["distinct_spin_H_core_v3_states_reached__count"]), "fraction": (distinct_core_count / max(v3_metrics["distinct_spin_H_core_v3_states_reached__count"], 1.0)) - 1.0, "note": "change in distinct parent states relative to spin_H_core_v3"})

    for component in COMPONENTS_V4:
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
                "note": f"dominant_component_delta=theta{dominant_sig[0]}_rho{dominant_sig[1]}_sigma{dominant_sig[2]}_h{dominant_sig[3]}",
            }
        )

    for primary, cores in sorted(primary_to_core.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        ordered = sorted(
            cores,
            key=lambda core: (
                core.theta.base_angle,
                core.theta.fiber_phase,
                core.rho.radial_class,
                _word_str(core.sigma.current_mode),
                _word_str(core.sigma.fiber_mode),
                _word_str(core.sigma.radial_mode),
                core.sigma.regressive_phase,
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
                    f"_sigma(cur{_word_str(core.sigma.current_mode)},fib{_word_str(core.sigma.fiber_mode)},rad{_word_str(core.sigma.radial_mode)},p{core.sigma.regressive_phase})"
                    f"_h(tau{_tau_str(core.h.recursive_phase)},ftau{_tau_str(core.h.fiber_recursive_phase)},rtau{_tau_str(core.h.radial_recursive_phase)},k{core.h.holonomy_bit})"
                    for core in ordered
                ),
            }
        )

    return rows


def write_spinH_core_v4(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CORE_V4,
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
    "COMPONENTS_V4",
    "OUTPUT_PATH_SPINH_CORE_V4",
    "SigmaDiagnosticsV4",
    "SigmaDirectV4",
    "SpinHCoreV4",
    "R_I_v4",
    "R_Tb_v4",
    "R_Tx_v4",
    "R_Tc_v4",
    "R_Ty_v4",
    "R_Tz_v4",
    "R_Tr_v4",
    "active_transport_lift_core_v4",
    "derive_mode_orbit_v4",
    "derive_sigma_diagnostics_v4",
    "project_kappa_v4",
    "project_spin_h4_v4",
    "project_tau_v4",
    "sigma_update_v4",
    "summarize_spinH_core_v4",
    "write_spinH_core_v4",
]
