"""Shared scalar router-math helpers for downstream adapters.

This module exposes a narrow, stable helper boundary for consumers such as
MUDBench. The formulas are a light scalar extraction of the canonical logic in
``hyperbolic_router_so8.py`` so downstream prompt planners do not need the full
NumPy-heavy router runtime.
"""

from __future__ import annotations

import math
from collections.abc import Sequence


def fibonacci_values_upto(max_value: int) -> list[int]:
    max_value = max(1, int(max_value))
    vals = [1, 2]
    while vals[-1] < max_value:
        vals.append(vals[-1] + vals[-2])
    return [value for value in vals if value <= max_value]


def normalize_4d_coordinate(routing_coordinate: Sequence[float]) -> tuple[float, float, float, float]:
    if isinstance(routing_coordinate, (str, bytes)) or not isinstance(routing_coordinate, Sequence):
        raise ValueError("routing_coordinate must be a sequence of four numbers")
    if len(routing_coordinate) != 4:
        raise ValueError("routing_coordinate must contain exactly four values")
    normalized_values: list[float] = []
    for value in routing_coordinate:
        if not isinstance(value, (int, float)):
            raise ValueError("routing_coordinate must contain only numbers")
        normalized_values.append(float(value))
    norm = math.sqrt(sum(value * value for value in normalized_values))
    if norm <= 1e-12:
        norm = 1.0
    return tuple(value / norm for value in normalized_values)  # type: ignore[return-value]


def allocate_pair_bins_scalar(total_cap: int, *, min_bins: int, ratio_scale: float) -> tuple[int, int]:
    total_cap = max(1, int(total_cap))
    ratio_scale = max(float(ratio_scale), 1e-9)
    pair_min = min(total_cap, max(1, int(min_bins)))

    base = math.sqrt(float(total_cap))
    k1 = max(pair_min, min(total_cap, int(round(base * ratio_scale))))
    k2 = max(pair_min, min(total_cap, int(round(float(total_cap) / max(k1, 1)))))

    if k1 * k2 > total_cap:
        k2 = max(pair_min, min(total_cap, int(math.floor(float(total_cap) / max(k1, 1)))))
    if k1 * k2 > total_cap:
        k1 = max(pair_min, min(total_cap, int(math.floor(float(total_cap) / max(k2, 1)))))
    return k1, k2


def allocate_triplet_bins_budget(
    total_cap: int,
    *,
    min_first: int = 2,
    min_second: int = 1,
    min_third: int = 1,
) -> tuple[int, int, int]:
    total_cap = max(1, int(total_cap))
    min_first = max(1, int(min_first))
    min_second = max(1, int(min_second))
    min_third = max(1, int(min_third))
    if min_first * min_second * min_third > total_cap:
        min_third = 1
    if min_first * min_second * min_third > total_cap:
        min_second = 1
    if min_first * min_second * min_third > total_cap:
        min_first = 1
    best = (1, total_cap, 1)
    best_score: tuple[int, int, int, int, int] | None = None
    for k_first in range(min_first, total_cap + 1):
        for k_second in range(min_second, total_cap + 1):
            max_third = total_cap // max(k_first * k_second, 1)
            if max_third < min_third:
                break
            for k_third in range(min_third, max_third + 1):
                product = k_first * k_second * k_third
                favor_base = 1 if k_second >= k_third else 0
                spread = (
                    abs(k_first - k_second)
                    + abs(k_second - k_third)
                    + abs(k_first - k_third)
                )
                score = (product, favor_base, -spread, k_second, -k_third)
                if best_score is None or score > best_score:
                    best_score = score
                    best = (k_first, k_second, k_third)
    return best


def hopf_coordinate_components_scalar(normalized_coordinate: Sequence[float]) -> dict[str, float]:
    a, b, c, d = (float(value) for value in normalized_coordinate)
    rho1 = math.sqrt((a * a) + (b * b))
    rho2 = math.sqrt((c * c) + (d * d))
    denom = max(math.sqrt((rho1 * rho1) + (rho2 * rho2)), 1e-12)
    cos_chi = rho1 / denom
    sin_chi = rho2 / denom
    return {
        "rho1": rho1,
        "rho2": rho2,
        "chi": math.asin(min(max(sin_chi, 0.0), 1.0)),
        "chi_u": min(max(sin_chi * sin_chi, 0.0), 1.0 - 1e-12),
        "theta1": _wrap_to_pi(math.atan2(b, a)),
        "theta2": _wrap_to_pi(math.atan2(d, c)),
        "delta": _wrap_to_pi(math.atan2(b, a) - math.atan2(d, c)),
        "alpha": _wrap_to_pi(0.5 * (math.atan2(b, a) + math.atan2(d, c))),
        "cos_chi": cos_chi,
        "sin_chi": sin_chi,
    }


def hopf_phase_transport_components_scalar(
    normalized_coordinate: Sequence[float],
    *,
    phase_transport_lambda: float,
) -> dict[str, float]:
    components = hopf_coordinate_components_scalar(normalized_coordinate)
    chi = components["chi"]
    delta = components["delta"]
    alpha = components["alpha"]
    connection_weight = 0.5 * float(phase_transport_lambda) * math.cos(2.0 * chi)
    phase_shift = _wrap_to_pi(connection_weight * delta)
    transported_alpha = _wrap_to_pi(alpha + phase_shift)
    return {
        **components,
        "transport_connection_weight": connection_weight,
        "transport_phase_shift": phase_shift,
        "transported_alpha": transported_alpha,
    }


def assign_sector_hopf_base_scalar(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
) -> dict[str, object]:
    components = hopf_coordinate_components_scalar(normalized_coordinate)
    kchi_value = max(1, int(math.floor(math.sqrt(max(int(K), 1)))))
    kdelta_value = max(1, int(math.ceil(float(max(int(K), 1)) / float(kchi_value))))
    u_delta = (components["delta"] + math.pi) / (2.0 * math.pi)
    chi_bin = min(int(components["chi_u"] * float(kchi_value)), max(kchi_value - 1, 0))
    delta_bin = min(int(u_delta * float(kdelta_value)), max(kdelta_value - 1, 0))
    sector_id = (chi_bin * kdelta_value + delta_bin) % max(int(K), 1)
    return {
        "coordinates": components,
        "sector_id": int(sector_id),
        "sector_bins": {
            "chi_bins": kchi_value,
            "delta_bins": kdelta_value,
            "chi_bin": chi_bin,
            "delta_bin": delta_bin,
        },
    }


def assign_sector_hopf_transport_scalar(
    normalized_coordinate: Sequence[float],
    *,
    K: int,
    phase_transport_lambda: float,
    hopf_chi_bins: int,
) -> dict[str, object]:
    components = hopf_phase_transport_components_scalar(
        normalized_coordinate,
        phase_transport_lambda=phase_transport_lambda,
    )
    kchi_value, kdelta_value, kalpha_value = allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )
    u_delta = (components["delta"] + math.pi) / (2.0 * math.pi)
    u_alpha = (components["transported_alpha"] + math.pi) / (2.0 * math.pi)
    chi_bin = min(int(components["chi_u"] * float(kchi_value)), max(kchi_value - 1, 0))
    delta_bin = min(int(u_delta * float(kdelta_value)), max(kdelta_value - 1, 0))
    alpha_bin = min(int(u_alpha * float(kalpha_value)), max(kalpha_value - 1, 0))
    local_span = max(kdelta_value * kalpha_value, 1)
    sector_id = min((chi_bin * local_span) + (delta_bin * kalpha_value) + alpha_bin, max(int(K) - 1, 0))
    return {
        "coordinates": components,
        "sector_id": int(sector_id),
        "sector_bins": {
            "chi_bins": kchi_value,
            "delta_bins": kdelta_value,
            "alpha_bins": kalpha_value,
            "chi_bin": chi_bin,
            "delta_bin": delta_bin,
            "alpha_bin": alpha_bin,
        },
}


def _wrap_to_pi(theta: float) -> float:
    return (theta + math.pi) % (2.0 * math.pi) - math.pi


__all__ = [
    "allocate_pair_bins_scalar",
    "allocate_triplet_bins_budget",
    "assign_sector_hopf_base_scalar",
    "assign_sector_hopf_transport_scalar",
    "fibonacci_values_upto",
    "hopf_coordinate_components_scalar",
    "hopf_phase_transport_components_scalar",
    "normalize_4d_coordinate",
]
