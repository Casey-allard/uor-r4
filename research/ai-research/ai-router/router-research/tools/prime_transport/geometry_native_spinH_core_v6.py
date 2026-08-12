#!/usr/bin/env python3
"""Canonical spin_H core v6 with coupled-family holonomy residue primitive.

Extends v5 by replacing the stub _component_role_order_v5 (which always
returned (0, 1, 2)) with the coupled_holonomy_residue_v6 primitive, applied
inside sigma_family_holonomy_law_v6.  All seven R_G maps are uplifted to call
the v6 law.  No new state fields, no h dependency, no architecture changes.
"""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from functools import lru_cache
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
    _tau_from_tuple,
    _tau_str,
    _tau_tuple,
)
from geometry_native_spinH_core_v5 import (
    COMPONENTS_V5 as COMPONENTS_V6,
    COMPOSITION_PAIRS_V5 as COMPOSITION_PAIRS_V6,
    HOLONOMY_ACTIVE_COMPONENTS_V5 as HOLONOMY_ACTIVE_COMPONENTS_V6,
    HOLONOMY_CHARGE_V5 as HOLONOMY_CHARGE_V6,
    OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1,
    SigmaDiagnosticsV5 as SigmaDiagnosticsV6,
    SigmaDirectV5,
    SpinHCoreV5 as SpinHCoreV6,
    _binary_value_v5 as _binary_value_v6,
    _canonical_orbit_v5 as _canonical_orbit_v6,
    _holonomy_bridge_word_v5 as _holonomy_bridge_word_v6,
    _local_sigma_words_v5 as _local_sigma_words_v6,
    _mix_words_v5 as _mix_words_v6,
    _rotate_word_v5 as _rotate_word_v6,
    _seed_orbit_from_sigma_v5 as _seed_orbit_from_sigma_v6,
    _tau_from_state_tuple,
    _word_str,
    _xor_words_v5 as _xor_words_v6,
    active_transport_lift_core_v5,
    component_update_signature_v5,
    direct_sigma_from_words_v5 as direct_sigma_from_words_v6,
    primary_chart_of_core_v5 as primary_chart_of_core_v6,
    project_kappa_v5 as project_kappa_v6,
    project_spin_h4_v5 as project_spin_h4_v6,
    project_tau_v5 as project_tau_v6,
)


OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_coupled_family_holonomy_v1.csv"
)


# ---------------------------------------------------------------------------
# Coupled-family holonomy residue — the single new primitive
# ---------------------------------------------------------------------------


def coupled_holonomy_residue_v6(
    source_sigma: SigmaDirectV5,
    proposed_sigma: SigmaDirectV5,
) -> int:
    """Cyclic coupling signature of the proposed sigma transition.

    Measures how each proposed mode slot diverges from the corresponding source
    slot in the cyclic order (current←radial, fiber←current, radial←fiber).
    Sigma-local: all inputs are fields of SigmaDirectV5.  No h, no branches.
    """
    def _xor_sum(a: tuple[int, ...], b: tuple[int, ...]) -> int:
        return sum((int(x) + int(y)) % 2 for x, y in zip(a, b))

    cyclic_divergence = (
        _xor_sum(proposed_sigma.current_mode, source_sigma.radial_mode)
        + _xor_sum(proposed_sigma.fiber_mode, source_sigma.current_mode)
        + _xor_sum(proposed_sigma.radial_mode, source_sigma.fiber_mode)
    )
    return (
        cyclic_divergence
        + source_sigma.regressive_phase
        + source_sigma.family_holonomy_class
    ) % 3


# ---------------------------------------------------------------------------
# Family holonomy law v6 — replaces the (0,1,2) stub
# ---------------------------------------------------------------------------


def sigma_family_holonomy_law_v6(
    source_sigma: SigmaDirectV5,
    component: str,
    proposed_sigma: SigmaDirectV5,
) -> SigmaDirectV5:
    """Family-level sigma holonomy law with coupled-holonomy residue role dispatch."""
    next_holonomy_class = (
        source_sigma.family_holonomy_class + HOLONOMY_CHARGE_V6[component]
    ) % 6

    if component not in HOLONOMY_ACTIVE_COMPONENTS_V6:
        return SigmaDirectV5(
            current_mode=proposed_sigma.current_mode,
            fiber_mode=proposed_sigma.fiber_mode,
            radial_mode=proposed_sigma.radial_mode,
            regressive_phase=(
                proposed_sigma.regressive_phase + sum(proposed_sigma.current_mode)
            ) % 12,
            family_holonomy_class=next_holonomy_class,
        )

    orbit = tuple(sorted({
        source_sigma.current_mode,
        source_sigma.fiber_mode,
        source_sigma.radial_mode,
    }))
    orbit_len = len(orbit)

    bridge_word = _rotate_word_v6(
        _holonomy_bridge_word_v6(source_sigma),
        next_holonomy_class,
    )

    # Coupled-holonomy residue replaces the (0,1,2) stub.
    role_shift = coupled_holonomy_residue_v6(source_sigma, proposed_sigma)
    current_idx = role_shift % orbit_len
    fiber_idx = (role_shift + 1) % orbit_len
    radial_idx = (role_shift + 2) % orbit_len

    return SigmaDirectV5(
        current_mode=orbit[current_idx],
        fiber_mode=orbit[fiber_idx],
        radial_mode=orbit[radial_idx],
        regressive_phase=(
            source_sigma.regressive_phase
            + HOLONOMY_CHARGE_V6[component]
            + sum(bridge_word)
        ) % 12,
        family_holonomy_class=next_holonomy_class,
    )


# ---------------------------------------------------------------------------
# All seven R_G maps — identical proposals to v5, uplifted to v6 law
# ---------------------------------------------------------------------------


def R_I_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    proposed = SigmaDirectV5(
        current_mode=sigma.current_mode,
        fiber_mode=sigma.fiber_mode,
        radial_mode=sigma.radial_mode,
        regressive_phase=sigma.regressive_phase,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "hold", proposed)


def R_Tb_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    proposed = SigmaDirectV5(
        current_mode=_rotate_word_v6(sigma.current_mode, 1),
        fiber_mode=_rotate_word_v6(sigma.fiber_mode, 1),
        radial_mode=_rotate_word_v6(sigma.radial_mode, 1),
        regressive_phase=(sigma.regressive_phase + 1) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "torus_base_advance", proposed)


def R_Tx_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    proposed = SigmaDirectV5(
        current_mode=sigma.fiber_mode,
        fiber_mode=sigma.current_mode,
        radial_mode=_xor_words_v6(sigma.radial_mode, sigma.current_mode),
        regressive_phase=(sigma.regressive_phase + 5) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "composite_swap", proposed)


def R_Tc_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    new_current = _xor_words_v6(sigma.current_mode, sigma.radial_mode)
    new_fiber = _mix_words_v6(sigma.fiber_mode, sigma.current_mode, sigma.regressive_phase + 1)
    new_radial = _rotate_word_v6(
        _mix_words_v6(sigma.radial_mode, sigma.fiber_mode, sigma.regressive_phase),
        1 + (sigma.regressive_phase % 2),
    )
    proposed = SigmaDirectV5(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(
            sigma.regressive_phase
            + sum(new_current)
            + sum(new_fiber)
            + sum(new_radial)
        ) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "coupled_torus_kick", proposed)


def R_Ty_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    proposed = SigmaDirectV5(
        current_mode=_rotate_word_v6(sigma.current_mode[::-1], 1),
        fiber_mode=_rotate_word_v6(sigma.fiber_mode[::-1], 1),
        radial_mode=_rotate_word_v6(sigma.radial_mode[::-1], 1),
        regressive_phase=(sigma.regressive_phase + 6) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "composite_twist", proposed)


def R_Tz_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    new_current = sigma.fiber_mode
    new_fiber = _rotate_word_v6(sigma.fiber_mode, 1)
    new_radial = _mix_words_v6(sigma.radial_mode, sigma.current_mode, sigma.regressive_phase + 2)
    proposed = SigmaDirectV5(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(sigma.regressive_phase + sum(new_current) + 1) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "fiber_phase_lift_spin_transport", proposed)


def R_Tr_v6(sigma: SigmaDirectV5) -> SigmaDirectV5:
    new_current = sigma.radial_mode
    new_fiber = _mix_words_v6(sigma.fiber_mode, sigma.radial_mode, sigma.regressive_phase + 3)
    new_radial = _rotate_word_v6(sigma.radial_mode, 1)
    proposed = SigmaDirectV5(
        current_mode=new_current,
        fiber_mode=new_fiber,
        radial_mode=new_radial,
        regressive_phase=(sigma.regressive_phase + sum(new_radial) + 2) % 12,
        family_holonomy_class=sigma.family_holonomy_class,
    )
    return sigma_family_holonomy_law_v6(sigma, "radial_transport_unfolding", proposed)


def sigma_update_v6(sigma: SigmaDirectV5, component: str) -> SigmaDirectV5:
    if component == "hold":
        return R_I_v6(sigma)
    if component == "torus_base_advance":
        return R_Tb_v6(sigma)
    if component == "composite_swap":
        return R_Tx_v6(sigma)
    if component == "coupled_torus_kick":
        return R_Tc_v6(sigma)
    if component == "composite_twist":
        return R_Ty_v6(sigma)
    if component == "fiber_phase_lift_spin_transport":
        return R_Tz_v6(sigma)
    if component == "radial_transport_unfolding":
        return R_Tr_v6(sigma)
    raise ValueError(f"unknown component {component!r}")


# ---------------------------------------------------------------------------
# Diagnostics (same structure as v5, using v6 sigma_update)
# ---------------------------------------------------------------------------


def derive_mode_orbit_v6(sigma: SigmaDirectV5) -> tuple[tuple[int, ...], ...]:
    return _canonical_orbit_v6(sigma.current_mode, sigma.fiber_mode, sigma.radial_mode)


def derive_sigma_diagnostics_v6(sigma: SigmaDirectV5) -> SigmaDiagnosticsV6:
    projection_profile: list[tuple[str, tuple[int, ...]]] = []
    orbit_profile: list[tuple[str, tuple[tuple[int, ...], ...]]] = []
    composition_profile: list[tuple[str, tuple[tuple[str, tuple[int, ...]], ...]]] = []

    for first in COMPONENTS_V6:
        first_sigma = sigma_update_v6(sigma, first)
        projection_profile.append((first, first_sigma.current_mode))
        orbit_profile.append((first, derive_mode_orbit_v6(first_sigma)))

        second_rows: list[tuple[str, tuple[int, ...]]] = []
        for second in COMPONENTS_V6:
            second_rows.append((second, sigma_update_v6(first_sigma, second).current_mode))
        composition_profile.append((first, tuple(second_rows)))

    return SigmaDiagnosticsV6(
        seed_orbit=derive_mode_orbit_v6(sigma),
        generator_projection_profile=tuple(projection_profile),
        generator_orbit_profile=tuple(orbit_profile),
        generator_composition_profile=tuple(composition_profile),
    )


# ---------------------------------------------------------------------------
# Active transport core v6 — uses v6 sigma update
# ---------------------------------------------------------------------------


@lru_cache(maxsize=None)
def active_transport_lift_core_v6(state: object) -> SpinHCoreV6:
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

    current_word, fiber_word, radial_word = _local_sigma_words_v6(state)
    sigma_direct = direct_sigma_from_words_v6(
        current_word=current_word,
        fiber_word=fiber_word,
        radial_word=radial_word,
    )

    return SpinHCoreV6(
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


# ---------------------------------------------------------------------------
# Comparison helpers
# ---------------------------------------------------------------------------


def _summary_metrics_from_v5(
    path: Path = OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1,
) -> dict[str, float]:
    csv.field_size_limit(min(sys.maxsize, 10**9))
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def _family_agreement_metrics_from_v5(
    path: Path = OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1,
) -> dict[str, int]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, int] = {}
    for row in rows:
        if row["scope"] == "composition":
            try:
                out[row["metric"]] = int(row["count"])
            except ValueError:
                continue
    return out


def _composition_exact_agreement_count_v6(
    sigma_states: list[SigmaDirectV5],
    first: str,
    second: str,
) -> int:
    exact = 0
    for sigma in sigma_states:
        left = sigma_update_v6(sigma_update_v6(sigma, second), first)
        right = sigma_update_v6(sigma_update_v6(sigma, first), second)
        if left == right:
            exact += 1
    return exact


# ---------------------------------------------------------------------------
# Summarize and write
# ---------------------------------------------------------------------------


def summarize_spinH_core_v6(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v8(depth=depth)

    primary_to_core: dict[PrimaryChartStateSpinHCandidateV3, set[SpinHCoreV6]] = defaultdict(set)
    core_to_primary: dict[SpinHCoreV6, set[PrimaryChartStateSpinHCandidateV3]] = defaultdict(set)
    core_transition_map: dict[SpinHCoreV6, dict[str, set[SpinHCoreV6]]] = defaultdict(
        lambda: defaultdict(set)
    )

    sigma_directly_updated = True
    diagnostics_only = True
    spin_h4_derivable = True
    tau_derivable = True
    kappa_derivable = True

    for state in states:
        primary = primary_chart_of_core_v6(state)
        core = active_transport_lift_core_v6(state)
        primary_to_core[primary].add(core)
        core_to_primary[core].add(primary)

        diagnostics = derive_sigma_diagnostics_v6(core.sigma)
        sigma_directly_updated &= isinstance(sigma_update_v6(core.sigma, "hold"), SigmaDirectV5)
        diagnostics_only &= isinstance(diagnostics, SigmaDiagnosticsV6)
        spin_h4_derivable &= project_spin_h4_v6(core) in diagnostics.seed_orbit
        tau_derivable &= project_tau_v6(core) == _tau_from_state_tuple(core.h.recursive_phase)
        kappa_derivable &= project_kappa_v6(core) == core.h.holonomy_bit

    component_signature_counter: Counter[tuple[str, tuple[int, int, int, int]]] = Counter()
    for transition in transitions:
        source_core = active_transport_lift_core_v6(transition.source)
        target_core = active_transport_lift_core_v6(transition.target)
        core_transition_map[source_core][transition.component].add(target_core)
        component_signature_counter[
            (transition.component, component_update_signature_v5(transition.source, transition.target))
        ] += 1

    primary_count = len(primary_to_core)
    distinct_core_count = len(core_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in core_to_primary.values())

    canonical_core_count = 0
    for component_map in core_transition_map.values():
        if all(len(targets) <= 1 for targets in component_map.values()):
            canonical_core_count += 1
    recursive_consistency_rate = canonical_core_count / max(distinct_core_count, 1)

    ordered_sigma_states = sorted(
        (core.sigma for core in core_to_primary),
        key=lambda sigma: (
            _word_str(sigma.current_mode),
            _word_str(sigma.fiber_mode),
            _word_str(sigma.radial_mode),
            sigma.regressive_phase,
            sigma.family_holonomy_class,
        ),
    )

    family_counts_v6: dict[str, int] = {}
    for first, second, metric_name in COMPOSITION_PAIRS_V6:
        family_counts_v6[metric_name] = _composition_exact_agreement_count_v6(
            ordered_sigma_states,
            first=first,
            second=second,
        )

    v5_summary = _summary_metrics_from_v5()
    v5_family = _family_agreement_metrics_from_v5()

    rows: list[dict[str, object]] = []

    rows.append({
        "scope": "summary",
        "metric": "primary_states_examined",
        "count": primary_count,
        "total": primary_count,
        "fraction": 1.0,
        "note": "distinct primary chart states on the bounded lawful H_v8 surface",
    })
    rows.append({
        "scope": "summary",
        "metric": "distinct_parent_states_reached",
        "count": distinct_core_count,
        "total": primary_count,
        "fraction": distinct_core_count / max(primary_count, 1),
        "note": "distinct canonical parent states with coupled-holonomy residue law",
    })
    rows.append({
        "scope": "summary",
        "metric": "collision_count",
        "count": collision_count,
        "total": primary_count,
        "fraction": collision_count / max(primary_count, 1),
        "note": "many-to-one collisions from primary chart states into v6 parent states",
    })
    rows.append({
        "scope": "summary",
        "metric": "recursive_consistency_rate",
        "count": canonical_core_count,
        "total": distinct_core_count,
        "fraction": recursive_consistency_rate,
        "note": "fraction of parent states whose lawful component updates remain canonical",
    })

    rows.append({
        "scope": "projection",
        "metric": "sigma_updated_directly_by_R_G",
        "count": int(sigma_directly_updated),
        "total": 1,
        "fraction": float(sigma_directly_updated),
        "note": "sigma updated by direct R_G maps with coupled-family holonomy residue law",
    })
    rows.append({
        "scope": "projection",
        "metric": "profile_fields_are_derived_diagnostics_only",
        "count": int(diagnostics_only),
        "total": 1,
        "fraction": float(diagnostics_only),
        "note": "seed_orbit and action/composition profiles remain derived diagnostics only",
    })
    rows.append({
        "scope": "projection",
        "metric": "spin_h4_derivable_from_parent",
        "count": int(spin_h4_derivable),
        "total": 1,
        "fraction": float(spin_h4_derivable),
        "note": "Pi_pred(spin_H_core_v6) -> sigma.current_mode",
    })
    rows.append({
        "scope": "projection",
        "metric": "tau_derivable_from_parent",
        "count": int(tau_derivable),
        "total": 1,
        "fraction": float(tau_derivable),
        "note": "Pi_rec(spin_H_core_v6) -> tau",
    })
    rows.append({
        "scope": "projection",
        "metric": "kappa_derivable_from_parent",
        "count": int(kappa_derivable),
        "total": 1,
        "fraction": float(kappa_derivable),
        "note": "Pi_hol(spin_H_core_v6) -> kappa",
    })

    for _, _, metric_name in COMPOSITION_PAIRS_V6:
        count = family_counts_v6[metric_name]
        rows.append({
            "scope": "composition",
            "metric": metric_name,
            "count": count,
            "total": distinct_core_count,
            "fraction": count / max(distinct_core_count, 1),
            "note": "exact sigma agreement under coupled-family holonomy residue law",
        })
        v5_count = v5_family.get(metric_name, 0)
        rows.append({
            "scope": "comparison_vs_v5",
            "metric": f"{metric_name}_change",
            "count": count - v5_count,
            "total": distinct_core_count,
            "fraction": (count - v5_count) / max(distinct_core_count, 1),
            "note": "exact-agreement change relative to spin_H_core_v5 family holonomy law",
        })

    rows.append({
        "scope": "primitive",
        "metric": "coupled_holonomy_residue_active",
        "count": 1,
        "total": 1,
        "fraction": 1.0,
        "note": "coupled_holonomy_residue_v6 replaces (0,1,2) stub in sigma_family_holonomy_law_v6",
    })
    rows.append({
        "scope": "primitive",
        "metric": "family_uplift_scope",
        "count": 7,
        "total": 7,
        "fraction": 1.0,
        "note": "all seven R_G maps (R_I R_Tb R_Tx R_Tc R_Ty R_Tz R_Tr) call sigma_family_holonomy_law_v6",
    })
    rows.append({
        "scope": "primitive",
        "metric": "new_state_fields_added",
        "count": 0,
        "total": 0,
        "fraction": 0.0,
        "note": "no new fields in SigmaDirectV5; primitive is purely functional",
    })
    rows.append({
        "scope": "primitive",
        "metric": "h_dependency",
        "count": 0,
        "total": 0,
        "fraction": 0.0,
        "note": "coupled_holonomy_residue_v6 uses only source_sigma and proposed_sigma fields",
    })

    for component in COMPONENTS_V6:
        matching = [
            (sig, cnt)
            for (name, sig), cnt in component_signature_counter.items()
            if name == component
        ]
        matching.sort(key=lambda item: (-item[1], item[0]))
        if matching:
            dominant_sig, dominant_count = matching[0]
            rows.append({
                "scope": "update_law",
                "metric": component,
                "count": dominant_count,
                "total": sum(cnt for _, cnt in matching),
                "fraction": dominant_count / max(sum(cnt for _, cnt in matching), 1),
                "note": (
                    f"dominant_component_delta="
                    f"theta{dominant_sig[0]}_rho{dominant_sig[1]}"
                    f"_sigma{dominant_sig[2]}_h{dominant_sig[3]}"
                ),
            })

    return rows


def write_spinH_core_v6(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1,
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
    "OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1",
    "R_I_v6",
    "R_Tb_v6",
    "R_Tx_v6",
    "R_Tc_v6",
    "R_Ty_v6",
    "R_Tz_v6",
    "R_Tr_v6",
    "active_transport_lift_core_v6",
    "coupled_holonomy_residue_v6",
    "derive_mode_orbit_v6",
    "derive_sigma_diagnostics_v6",
    "sigma_family_holonomy_law_v6",
    "sigma_update_v6",
    "summarize_spinH_core_v6",
    "write_spinH_core_v6",
]
