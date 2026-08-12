#!/usr/bin/env python3
"""
hyperbolic_router_so8.py

Hyperbolic discrete routing + local memory + growth splitting
with optional learned chart applied in tangent space.

Chart options:
  - Rotation R in SO(d) (optional)
  - Scaling:
      * global diagonal: D = diag(exp(s)), s in R^d
      * radial-bin diagonal: D(r) = diag(exp(S[bin(r)])), S in R^{B x d}

Routing options:
  - sector_mode = "kmeans" : spherical k-means centers on normalized directions (default)
  - sector_mode = "phase2" : polar phase bins in a fixed 2D plane of charted tangent z
  - sector_mode = "phase4d_adaptive" : time-expanding anisotropic 4D phase routing with phi/pi widening
  - sector_mode = "phase4d_hopf" : Hopf-aware shell-capacity pilot derived from H4 diagnostics
  - sector_mode = "phase4d_hopf_base" : coarse routing on the Hopf base (chi, delta) with fiber phase alpha excluded from the address
  - sector_mode = "phase4d_hopf_transport" : Hopf-base routing plus geometry-induced fiber transport on alpha
  - sector_mode = "phase4d_hopf_transport_complex" : Hopf-base routing plus complex-field-coupled fiber transport
  - sector_mode = "phase4d_hopf_product_phase" : explicit product H^4 x H^4 phase coupling with routing and field factors kept separate
  - sector_mode = "phase4d_hopf_iso" : Hopf-aware routing from a rotation-only near-isometric chart coordinate
  - sector_mode = "phase4d_hopf_ball" : Hopf-aware routing with shells anchored to original-ball geodesic radius
  - sector_mode = "phase4d_hopf_base_ball" : Hopf-base coarse routing with shells anchored to original-ball geodesic radius
  - sector_mode = "phase4d_hopf_fib" : Hopf-aware routing with Fibonacci-constrained shell/chi/phase lattice
  - sector_mode = "phase4d_hopf_fib_rung" : Hopf-aware routing with recurrence-constrained phi^2 rung forcing
  - sector_mode = "phase4d_hopf_fib_band" : Hopf-aware routing with shared phi^2 rung-band states
  - sector_mode = "phase4d_hopf_fib_band_iso" : banded Hopf widening on a rotation-only near-isometric route coordinate
  - sector_mode = "phase4d_hopf_fib_band_bound" : banded Hopf widening on a bounded-isometry blended route coordinate
  - sector_mode = "phase4d_hopf_blend" : Hopf-aware routing with blended shell-capacity correction from gap/chi/shell pressure
  - sector_mode = "phase4d_complex_local" : phase4d_adaptive coarse routing plus local complex zoom

Time-pressure (optional):
  During TRAIN EMA updates only, shells can be computed with a time-dependent radial expansion:
      r_eff = r * exp(time_pressure_lambda * tau), tau in [0,1] over training progress.
  This introduces a monotone “pressure” to push representation outward over time.

Pipeline:
    v = log_0(x)          (tangent at origin)
    z = chart(v)          (global or radial chart)
    route(z) -> (shell, sector) buckets

IMPORTANT UPGRADE (kept):
    If a chart is learned AND sector_mode="kmeans", recompute sector centers C in charted space:
        U_chart = normalize(z_train)
        C = spherical_kmeans(U_chart, K)
    Then routing/memory/growth all use this updated C.

Chart optimization objective (TRAIN):
    label_SSE(chart; C0) + alpha_overload * sum_b p_b^2 + beta_bucketcount * |B|
    + radial_l2 * ||S||^2   (only for radial-bin scaling)

Diagnostics printed:
- test_unseen_rate (w.r.t. TRAIN keys under tau=1.0 eval routing)
- train/test label SSE (and per-sample) under the *final* routing
- before/after growth test MSE, pmax, entropy H, bucket count

Examples:

  # baseline
  python hyperbolic_router_so8.py --mode anis --learn_so8 0 --learn_scale 0

  # global scale only
  python hyperbolic_router_so8.py --mode anis --learn_so8 0 --learn_scale 1 --scale_mode global

  # radial scale only (pure geometry)
  python hyperbolic_router_so8.py --mode anis --learn_so8 0 --learn_scale 1 --scale_mode radial --radial_bins 10

  # polar phase routing (no kmeans sectors)
  python hyperbolic_router_so8.py --mode anis --learn_so8 0 --learn_scale 1 --scale_mode radial \
      --sector_mode phase2 --phase_dims 0,1

  # add time-pressure during training updates
  python hyperbolic_router_so8.py --mode anis --learn_so8 0 --learn_scale 1 --scale_mode radial \
      --time_pressure_lambda 0.8

  # rotation only (if you insist on suffering)
  python hyperbolic_router_so8.py --mode anis --learn_so8 1 --learn_scale 0
"""

import argparse
import hashlib
import json
import os
import subprocess
import time
import numpy as np
from dataclasses import dataclass
from typing import Dict, Tuple, List, Iterable, Optional


PHI = (1.0 + np.sqrt(5.0)) / 2.0
LOG_PHI = float(np.log(PHI))
GOLDEN_ANGLE = 2.0 * np.pi * (1.0 - 1.0 / PHI)
HOPF_CHI_BINS = 12


# ----------------------------
# Utilities
# ----------------------------

def set_seed(seed: int):
    np.random.seed(seed)

def safe_norm(x: np.ndarray, axis=None, keepdims=False, eps: float = 1e-12):
    n = np.linalg.norm(x, axis=axis, keepdims=keepdims)
    return np.maximum(n, eps)

def entropy_from_counts(counts: np.ndarray) -> float:
    p = counts / np.sum(counts)
    p = p[p > 0]
    return float(-np.sum(p * np.log(p + 1e-12)))


def histogram_entropy(values: np.ndarray, bins: int, value_range: Tuple[float, float]) -> float:
    hist, _ = np.histogram(values, bins=max(2, int(bins)), range=value_range)
    if np.sum(hist) <= 0:
        return 0.0
    return entropy_from_counts(hist)


def fibonacci_values_upto(max_value: int) -> List[int]:
    max_value = max(1, int(max_value))
    vals = [1, 2]
    while vals[-1] < max_value:
        vals.append(vals[-1] + vals[-2])
    return [v for v in vals if v <= max_value]


def fibonacci_ceil_indices(values: np.ndarray, max_value: int) -> Tuple[np.ndarray, np.ndarray]:
    max_value = max(1, int(max_value))
    fib_vals = np.asarray(fibonacci_values_upto(max_value), dtype=np.int64)
    arr = np.asarray(values, dtype=np.float64)
    flat = np.maximum(1, np.rint(arr).astype(np.int64)).reshape(-1)
    idx = np.searchsorted(fib_vals, flat, side="left")
    idx = np.clip(idx, 0, len(fib_vals) - 1)
    return idx.reshape(arr.shape), fib_vals


def fibonacci_ceil_array(values: np.ndarray, max_value: int) -> np.ndarray:
    idx, fib_vals = fibonacci_ceil_indices(values, max_value=max_value)
    return fib_vals[idx]


def allocate_fibonacci_pair_bins(total_cap: np.ndarray, min_bins: int, ratio_scale: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
    total_cap = np.maximum(np.asarray(total_cap, dtype=np.int64), 1)
    ratio_scale = np.maximum(np.asarray(ratio_scale, dtype=np.float64), 1e-9)
    pair_min = max(1, int(min_bins))
    max_cap = int(np.max(total_cap))
    fib_vals = [f for f in fibonacci_values_upto(max_cap) if f >= pair_min]
    if not fib_vals:
        return allocate_pair_bins(total_cap, min_bins=min_bins, ratio_scale=ratio_scale)

    k1 = np.empty(total_cap.shape, dtype=np.int64)
    k2 = np.empty(total_cap.shape, dtype=np.int64)
    for idx, (cap_i, ratio_i) in enumerate(zip(total_cap.tolist(), ratio_scale.tolist())):
        best_prod = -1
        best_ratio_err = float("inf")
        best_pair = (pair_min, pair_min)
        for a in fib_vals:
            if a > cap_i:
                break
            for b in fib_vals:
                prod = a * b
                if prod > cap_i:
                    break
                ratio_err = abs(np.log(max(float(a), 1e-9) / max(float(b), 1e-9)) - np.log(ratio_i))
                if prod > best_prod or (prod == best_prod and ratio_err < best_ratio_err):
                    best_prod = prod
                    best_ratio_err = ratio_err
                    best_pair = (a, b)
        k1[idx] = best_pair[0]
        k2[idx] = best_pair[1]
    return k1.astype(np.int64), k2.astype(np.int64)


def allocate_fibonacci_chi_bins(
    total_cap: np.ndarray,
    min_pair_bins: int,
    chi_u: np.ndarray,
    div_score: np.ndarray,
) -> np.ndarray:
    total_cap = np.maximum(np.asarray(total_cap, dtype=np.int64), 1)
    chi_u = np.asarray(chi_u, dtype=np.float64)
    div_score = np.clip(np.asarray(div_score, dtype=np.float64), 0.0, 1.0)
    min_pair_product = max(1, int(min_pair_bins) * int(min_pair_bins))
    max_cap = int(np.max(total_cap))
    fib_vals = fibonacci_values_upto(max_cap)
    out = np.ones(total_cap.shape, dtype=np.int64)

    chi_pressure = (4.0 * chi_u * (1.0 - chi_u)) * (0.5 + 0.5 * div_score)
    for idx, cap_i in enumerate(total_cap.tolist()):
        max_chi = max(1, int(cap_i // max(min_pair_product, 1)))
        candidates = [f for f in fib_vals if f <= max_chi]
        if not candidates:
            out[idx] = 1
            continue
        rung = int(np.rint(float(chi_pressure[idx]) * float(len(candidates) - 1)))
        rung = int(np.clip(rung, 0, len(candidates) - 1))
        out[idx] = int(candidates[rung])
    return out.astype(np.int64)


def allocate_phi2_rung_bins(
    total_cap: np.ndarray,
    min_pair_bins: int,
    chi_u: np.ndarray,
    rho1: np.ndarray,
    rho2: np.ndarray,
    div_score: np.ndarray,
    max_total: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    total_cap = np.maximum(np.asarray(total_cap, dtype=np.int64), 1)
    chi_u = np.asarray(chi_u, dtype=np.float64)
    rho1 = np.maximum(np.asarray(rho1, dtype=np.float64), 1e-9)
    rho2 = np.maximum(np.asarray(rho2, dtype=np.float64), 1e-9)
    div_score = np.clip(np.asarray(div_score, dtype=np.float64), 0.0, 1.0)

    cap_idx, fib_vals = fibonacci_ceil_indices(total_cap, max_value=max(int(max_total), int(np.max(total_cap))))
    pair_floor_idx = int(np.searchsorted(fib_vals, max(1, int(min_pair_bins)), side="left"))
    pair_floor_idx = int(np.clip(pair_floor_idx, 0, len(fib_vals) - 1))
    pair_floor_val = int(fib_vals[pair_floor_idx])
    max_chi_allowed = max(1, int(max_total) // max(pair_floor_val * pair_floor_val, 1))
    chi_cap_idx = int(np.searchsorted(fib_vals, max_chi_allowed, side="right") - 1)
    chi_cap_idx = int(np.clip(chi_cap_idx, 0, len(fib_vals) - 1))

    chi_pressure = (4.0 * chi_u * (1.0 - chi_u)) * (0.5 + 0.5 * div_score)
    log_ratio = 0.5 * np.log(rho1 / rho2)

    flat_cap_idx = cap_idx.reshape(-1)
    flat_chi_pressure = chi_pressure.reshape(-1)
    flat_log_ratio = log_ratio.reshape(-1)

    kchi = np.ones(flat_cap_idx.shape, dtype=np.int64)
    k1 = np.empty(flat_cap_idx.shape, dtype=np.int64)
    k2 = np.empty(flat_cap_idx.shape, dtype=np.int64)
    forced_total = np.empty(flat_cap_idx.shape, dtype=np.int64)

    for i, cap_rung in enumerate(flat_cap_idx.tolist()):
        cap_rung = int(np.clip(cap_rung, 0, len(fib_vals) - 1))
        chi_active = bool(flat_chi_pressure[i] >= (1.0 / PHI) and cap_rung >= (pair_floor_idx + 2))
        chi_floor_idx = 0
        if chi_active:
            chi_floor_idx = min(max(1, cap_rung - 4), chi_cap_idx)
        chi_idx = chi_floor_idx

        major_idx = max(pair_floor_idx, cap_rung - 2)
        minor_idx = max(pair_floor_idx, cap_rung - 3)

        keep_scores = [
            1.0 + float(flat_chi_pressure[i]),
            1.0 + max(float(abs(flat_log_ratio[i])) / max(LOG_PHI, 1e-9), 0.0),
            1.0,
        ]
        lower_idx = [chi_floor_idx, pair_floor_idx, pair_floor_idx]
        rung_idx = [chi_idx, major_idx, minor_idx]

        while int(fib_vals[rung_idx[0]]) * int(fib_vals[rung_idx[1]]) * int(fib_vals[rung_idx[2]]) > int(max_total):
            candidates = []
            for axis in range(3):
                if rung_idx[axis] > lower_idx[axis]:
                    candidates.append((keep_scores[axis], -int(fib_vals[rung_idx[axis]]), axis))
            if not candidates:
                break
            _, _, axis = min(candidates)
            rung_idx[axis] -= 1

        chi_idx, major_idx, minor_idx = rung_idx
        kchi_i = int(fib_vals[chi_idx])
        major_i = int(fib_vals[major_idx])
        minor_i = int(fib_vals[minor_idx])
        if flat_log_ratio[i] >= 0.0:
            k1_i, k2_i = major_i, minor_i
        else:
            k1_i, k2_i = minor_i, major_i

        kchi[i] = kchi_i
        k1[i] = k1_i
        k2[i] = k2_i
        forced_total[i] = int(kchi_i * k1_i * k2_i)

    out_shape = total_cap.shape
    return (
        kchi.reshape(out_shape).astype(np.int64),
        k1.reshape(out_shape).astype(np.int64),
        k2.reshape(out_shape).astype(np.int64),
        forced_total.reshape(out_shape).astype(np.int64),
    )


def fibonacci_prev_at_least(value: int, min_value: int) -> int:
    value = max(1, int(value))
    min_value = max(1, int(min_value))
    fib_vals = fibonacci_values_upto(max(value, min_value))
    idx = int(np.searchsorted(fib_vals, value, side="left"))
    if idx >= len(fib_vals) or fib_vals[idx] != value:
        idx = max(0, idx - 1)
    while idx > 0 and fib_vals[idx - 1] >= min_value:
        idx -= 1
        return int(fib_vals[idx])
    return int(max(value, min_value))


def phi2_state_total(state: Tuple[int, int, int]) -> int:
    return int(state[0] * state[1] * state[2])


def representative_phi2_rung_state(
    mask: np.ndarray,
    fib_total: np.ndarray,
    chi_u: np.ndarray,
    rho1: np.ndarray,
    rho2: np.ndarray,
    div_score: np.ndarray,
    min_pair_bins: int,
    max_total: int,
) -> Optional[Tuple[int, int, int]]:
    mask = np.asarray(mask, dtype=bool)
    if not np.any(mask):
        return None
    rep_total = float(np.median(fib_total[mask]))
    rep_total = int(fibonacci_ceil_array(np.asarray([rep_total]), max_value=max_total)[0])
    rep_chi_u = float(np.median(chi_u[mask]))
    rep_rho1 = float(np.median(rho1[mask]))
    rep_rho2 = float(np.median(rho2[mask]))
    rep_div = float(np.median(div_score[mask]))
    kchi, k1, k2, _ = allocate_phi2_rung_bins(
        total_cap=np.asarray([rep_total], dtype=np.int64),
        min_pair_bins=min_pair_bins,
        chi_u=np.asarray([rep_chi_u], dtype=np.float64),
        rho1=np.asarray([rep_rho1], dtype=np.float64),
        rho2=np.asarray([rep_rho2], dtype=np.float64),
        div_score=np.asarray([rep_div], dtype=np.float64),
        max_total=max_total,
    )
    major = int(max(k1[0], k2[0]))
    minor = int(min(k1[0], k2[0]))
    return (int(kchi[0]), major, minor)


def step_down_phi2_state(
    state: Tuple[int, int, int],
    base_state: Tuple[int, int, int],
    min_pair_bins: int,
    max_total: int,
) -> Tuple[int, int, int]:
    pair_floor = int(fibonacci_ceil_array(np.asarray([max(1, int(min_pair_bins))]), max_value=max_total)[0])
    base_total = phi2_state_total(base_state)
    cur_total = phi2_state_total(state)
    candidates: List[Tuple[int, int, int, int]] = []
    for axis in range(3):
        floor = 1 if axis == 0 else pair_floor
        prev = fibonacci_prev_at_least(state[axis], min_value=floor)
        if prev >= state[axis]:
            continue
        cand = list(state)
        cand[axis] = prev
        prod = int(cand[0] * cand[1] * cand[2])
        if prod > int(max_total) or prod < base_total or prod >= cur_total:
            continue
        candidates.append((prod, int(cand[0]), int(cand[1]), int(cand[2])))
    if not candidates:
        return base_state
    candidates.sort(key=lambda x: (x[0], x[1], x[2], x[3]), reverse=True)
    best = candidates[0]
    return (int(best[1]), int(best[2]), int(best[3]))


def derive_phi2_band_states(
    comp: Dict[str, np.ndarray],
    min_pair_bins: int,
    max_total: int,
) -> Tuple[np.ndarray, Tuple[int, int, int], Tuple[int, int, int], Tuple[int, int, int], np.ndarray]:
    chi_pressure = (4.0 * comp["chi_u"] * (1.0 - comp["chi_u"])) * (0.5 + 0.5 * comp["div_score"])
    band_lo = 1.0 / (PHI * PHI)
    band_hi = 1.0 / PHI
    band = np.zeros(chi_pressure.shape, dtype=np.int64)
    band = np.where(chi_pressure >= band_lo, 1, band)
    band = np.where(chi_pressure >= band_hi, 2, band)

    fib_total = fibonacci_ceil_array(comp["hopf_shell_capacity"], max_value=max_total)
    hopf_major = np.maximum(comp["hopf_k1"], comp["hopf_k2"])
    hopf_minor = np.minimum(comp["hopf_k1"], comp["hopf_k2"])
    base_state = (
        1,
        int(np.rint(np.median(hopf_major))),
        int(np.rint(np.median(hopf_minor))),
    )

    full_state = representative_phi2_rung_state(
        mask=(band == 2),
        fib_total=fib_total,
        chi_u=comp["chi_u"],
        rho1=comp["rho1"],
        rho2=comp["rho2"],
        div_score=comp["div_score"],
        min_pair_bins=min_pair_bins,
        max_total=max_total,
    )
    if full_state is None:
        full_state = representative_phi2_rung_state(
            mask=(band >= 1),
            fib_total=fib_total,
            chi_u=comp["chi_u"],
            rho1=comp["rho1"],
            rho2=comp["rho2"],
            div_score=comp["div_score"],
            min_pair_bins=min_pair_bins,
            max_total=max_total,
        )
    if full_state is None:
        full_state = base_state

    moderate_state = representative_phi2_rung_state(
        mask=(band == 1),
        fib_total=fib_total,
        chi_u=comp["chi_u"],
        rho1=comp["rho1"],
        rho2=comp["rho2"],
        div_score=comp["div_score"],
        min_pair_bins=min_pair_bins,
        max_total=max_total,
    )
    if moderate_state is None:
        moderate_state = step_down_phi2_state(
            full_state,
            base_state=base_state,
            min_pair_bins=min_pair_bins,
            max_total=max_total,
        )

    if phi2_state_total(moderate_state) <= phi2_state_total(base_state) and phi2_state_total(full_state) > phi2_state_total(base_state):
        moderate_state = step_down_phi2_state(
            full_state,
            base_state=base_state,
            min_pair_bins=min_pair_bins,
            max_total=max_total,
        )
    if phi2_state_total(full_state) < phi2_state_total(moderate_state):
        moderate_state, full_state = full_state, moderate_state

    return band.astype(np.int64), base_state, moderate_state, full_state, fib_total.astype(np.int64)


def load_pressure(ids: np.ndarray) -> Tuple[np.ndarray, int]:
    ids = np.asarray(ids, dtype=np.int64).reshape(-1)
    if ids.size == 0:
        return np.zeros(0, dtype=np.float64), 0
    _, inv, counts = np.unique(ids, return_inverse=True, return_counts=True)
    bins_used = int(len(counts))
    if bins_used <= 1:
        return np.zeros(ids.shape[0], dtype=np.float64), bins_used
    loads = counts[inv].astype(np.float64) / float(ids.size)
    uniform = 1.0 / float(bins_used)
    pressure = np.maximum(loads - uniform, 0.0) / max(1.0 - uniform, 1e-9)
    return pressure.astype(np.float64), bins_used


def allocate_pair_bins(total_cap: np.ndarray, min_bins: int, ratio_scale: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
    total_cap = np.maximum(np.asarray(total_cap, dtype=np.int64), 1)
    ratio_scale = np.maximum(np.asarray(ratio_scale, dtype=np.float64), 1e-9)
    pair_min = np.minimum(total_cap, max(1, int(min_bins))).astype(np.int64)

    base = np.sqrt(total_cap.astype(np.float64))
    k1 = np.rint(base * ratio_scale).astype(np.int64)
    k1 = np.maximum(k1, pair_min)
    k1 = np.minimum(k1, total_cap)

    k2 = np.rint(total_cap / np.maximum(k1, 1)).astype(np.int64)
    k2 = np.maximum(k2, pair_min)
    k2 = np.minimum(k2, total_cap)

    prod = k1 * k2
    over = prod > total_cap
    if np.any(over):
        k2[over] = np.maximum(
            pair_min[over],
            np.floor(total_cap[over].astype(np.float64) / np.maximum(k1[over], 1)).astype(np.int64),
        )
        k2[over] = np.minimum(k2[over], total_cap[over])

    prod = k1 * k2
    over = prod > total_cap
    if np.any(over):
        k1[over] = np.maximum(
            pair_min[over],
            np.floor(total_cap[over].astype(np.float64) / np.maximum(k2[over], 1)).astype(np.int64),
        )
        k1[over] = np.minimum(k1[over], total_cap[over])

    return k1.astype(np.int64), k2.astype(np.int64)


def allocate_triplet_bins_budget(
    total_cap: int,
    min_first: int = 2,
    min_second: int = 1,
    min_third: int = 1,
) -> Tuple[int, int, int]:
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
    best_score = None
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

def cayley_from_skew(S: np.ndarray) -> np.ndarray:
    d = S.shape[0]
    I = np.eye(d, dtype=np.float64)
    A = I + S
    B = I - S
    Q = np.linalg.solve(A, B)
    return Q

def random_skew(d: int, scale: float) -> np.ndarray:
    A = np.random.randn(d, d).astype(np.float64)
    S = A - A.T
    S *= (scale / safe_norm(S))
    return S

def ensure_det_plus_one(R: np.ndarray) -> np.ndarray:
    if np.linalg.det(R) < 0:
        R = R.copy()
        R[:, 0] *= -1.0
    return R

def clip_scales(s: np.ndarray, clip: float) -> np.ndarray:
    if clip <= 0:
        return s
    return np.clip(s, -clip, clip)

def normalize_rows(X: np.ndarray) -> np.ndarray:
    return X / safe_norm(X, axis=1, keepdims=True)

def parse_phase_dims(s: str) -> Tuple[int, int]:
    parts = s.split(",")
    if len(parts) != 2:
        raise ValueError("--phase_dims must be like '0,1'")
    return int(parts[0].strip()), int(parts[1].strip())

def parse_quad_dims(s: str, flag_name: str) -> Tuple[int, int, int, int]:
    parts = s.split(",")
    if len(parts) != 4:
        raise ValueError(f"{flag_name} must be like '0,1,2,3'")
    return tuple(int(p.strip()) for p in parts)  # type: ignore[return-value]

def parse_pair_dims(s: str, flag_name: str) -> Tuple[int, int]:
    parts = s.split(",")
    if len(parts) != 2:
        raise ValueError(f"{flag_name} must be like '0,1'")
    return int(parts[0].strip()), int(parts[1].strip())

def wrap_to_pi(theta: np.ndarray) -> np.ndarray:
    # map to (-pi, pi]
    return (theta + np.pi) % (2.0 * np.pi) - np.pi

def ensure_dims_in_range(dims: Iterable[int], d: int, flag_name: str):
    for idx in dims:
        if idx < 0 or idx >= d:
            raise ValueError(f"{flag_name} index {idx} out of range for d={d}")


def ensure_distinct_dims(dims: Iterable[int], flag_name: str):
    dims_list = list(int(idx) for idx in dims)
    if len(set(dims_list)) != len(dims_list):
        raise ValueError(f"{flag_name} must contain distinct indices")


def ensure_disjoint_dims(dims_a: Iterable[int], flag_a: str, dims_b: Iterable[int], flag_b: str):
    overlap = sorted(set(int(idx) for idx in dims_a).intersection(int(idx) for idx in dims_b))
    if overlap:
        raise ValueError(f"{flag_a} and {flag_b} must be disjoint; overlap={overlap}")

def stable_hash(payload: Dict) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(blob).hexdigest()

def maybe_git_info() -> Dict[str, str]:
    info = {"branch": "", "commit": ""}
    try:
        info["branch"] = subprocess.check_output(
            ["git", "branch", "--show-current"], stderr=subprocess.DEVNULL
        ).decode("utf-8").strip()
    except Exception:
        pass
    try:
        info["commit"] = subprocess.check_output(
            ["git", "rev-parse", "HEAD"], stderr=subprocess.DEVNULL
        ).decode("utf-8").strip()
    except Exception:
        pass
    return info

def ensure_dir(path: str):
    os.makedirs(path, exist_ok=True)


# ----------------------------
# Poincaré ball maps (curvature c=1)
# ----------------------------

def artanh(x: np.ndarray) -> np.ndarray:
    return 0.5 * (np.log1p(x) - np.log1p(-x))

def exp_map0(v: np.ndarray) -> np.ndarray:
    """exp_0(v) = tanh(||v||) * v/||v||"""
    n = safe_norm(v, axis=-1, keepdims=True)
    return np.tanh(n) * (v / n)

def log_map0(x: np.ndarray) -> np.ndarray:
    """log_0(x) = artanh(||x||) * x/||x||"""
    nx = safe_norm(x, axis=-1, keepdims=True)
    return artanh(nx) * (x / nx)


def poincare_radius(x: np.ndarray) -> np.ndarray:
    nx = np.clip(safe_norm(x, axis=-1, keepdims=False), 0.0, 1.0 - 1e-12)
    return 2.0 * artanh(nx)


def poincare_distance(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    diff2 = np.sum((x - y) ** 2, axis=-1)
    nx2 = np.clip(np.sum(x * x, axis=-1), 0.0, 1.0 - 1e-12)
    ny2 = np.clip(np.sum(y * y, axis=-1), 0.0, 1.0 - 1e-12)
    denom = np.maximum((1.0 - nx2) * (1.0 - ny2), 1e-12)
    arg = 1.0 + (2.0 * diff2 / denom)
    return np.arccosh(np.maximum(arg, 1.0))


def sample_pair_indices(n: int, max_pairs: int, seed: int) -> Tuple[np.ndarray, np.ndarray]:
    n = int(n)
    max_pairs = max(0, int(max_pairs))
    if n < 2 or max_pairs <= 0:
        empty = np.zeros((0,), dtype=np.int64)
        return empty, empty
    total_pairs = n * (n - 1) // 2
    if total_pairs <= max_pairs:
        iu = np.triu_indices(n, k=1)
        return iu[0].astype(np.int64), iu[1].astype(np.int64)
    rs = np.random.RandomState(int(seed))
    i = rs.randint(0, n, size=max_pairs, dtype=np.int64)
    j = rs.randint(0, n - 1, size=max_pairs, dtype=np.int64)
    j += (j >= i).astype(np.int64)
    return i, j


def safe_corrcoef_1d(a: np.ndarray, b: np.ndarray) -> float:
    a = np.asarray(a, dtype=np.float64).reshape(-1)
    b = np.asarray(b, dtype=np.float64).reshape(-1)
    if a.size != b.size or a.size < 2:
        return 0.0
    a_std = float(np.std(a))
    b_std = float(np.std(b))
    if a_std < 1e-12 and b_std < 1e-12:
        return 1.0 if np.allclose(a, b) else 0.0
    if a_std < 1e-12 or b_std < 1e-12:
        return 0.0
    corr = float(np.corrcoef(a, b)[0, 1])
    if not np.isfinite(corr):
        return 0.0
    return corr


def poincare_alignment_diagnostics(
    v: np.ndarray,
    z: np.ndarray,
    max_pairs: int = 512,
    seed: int = 0,
) -> Dict[str, float]:
    if v.shape != z.shape:
        raise ValueError(f"Shape mismatch for Poincare alignment: {v.shape} vs {z.shape}")
    x_v = exp_map0(v)
    x_z = exp_map0(z)

    d_v = poincare_radius(x_v)
    d_z = poincare_radius(x_z)
    radial_abs = np.abs(d_z - d_v)
    radial_den = np.maximum(d_v, 1e-9)

    pair_i, pair_j = sample_pair_indices(v.shape[0], max_pairs=max_pairs, seed=seed)
    if pair_i.size > 0:
        pair_v = poincare_distance(x_v[pair_i], x_v[pair_j])
        pair_z = poincare_distance(x_z[pair_i], x_z[pair_j])
        pair_abs = np.abs(pair_z - pair_v)
        pair_den = np.maximum(pair_v, 1e-9)
        pair_mae = float(np.mean(pair_abs))
        pair_rel = float(np.mean(pair_abs / pair_den))
        pair_corr = safe_corrcoef_1d(pair_v, pair_z)
    else:
        pair_mae = 0.0
        pair_rel = 0.0
        pair_corr = 0.0

    return {
        "poincare_alignment_pairs_used": int(pair_i.size),
        "poincare_alignment_radial_mae": float(np.mean(radial_abs)),
        "poincare_alignment_radial_rel_mean": float(np.mean(radial_abs / radial_den)),
        "poincare_alignment_radial_corr": safe_corrcoef_1d(d_v, d_z),
        "poincare_alignment_pair_mae": pair_mae,
        "poincare_alignment_pair_rel_mean": pair_rel,
        "poincare_alignment_pair_corr": pair_corr,
    }


def h4_mass_antiderivative(geo_r: np.ndarray) -> np.ndarray:
    geo_r = np.asarray(geo_r, dtype=np.float64)
    return (np.cosh(geo_r) ** 3) / 3.0 - np.cosh(geo_r)


def h4_cumulative_mass(geo_r: np.ndarray) -> np.ndarray:
    geo_r = np.asarray(geo_r, dtype=np.float64)
    return h4_mass_antiderivative(geo_r) - h4_mass_antiderivative(np.asarray(0.0, dtype=np.float64))


def inverse_h4_cumulative_mass(target_mass: np.ndarray) -> np.ndarray:
    target = np.maximum(np.asarray(target_mass, dtype=np.float64), 0.0)
    if target.size == 0:
        return target
    lo = np.zeros_like(target)
    hi = np.ones_like(target)
    flat_hi = hi.reshape(-1)
    flat_target = target.reshape(-1)
    for idx, tgt in enumerate(flat_target):
        while float(h4_cumulative_mass(np.asarray(flat_hi[idx], dtype=np.float64))) < float(tgt):
            flat_hi[idx] *= 2.0
            if flat_hi[idx] > 1e6:
                break
    hi = flat_hi.reshape(target.shape)
    for _ in range(48):
        mid = 0.5 * (lo + hi)
        mid_mass = h4_cumulative_mass(mid)
        lo = np.where(mid_mass < target, mid, lo)
        hi = np.where(mid_mass < target, hi, mid)
    return 0.5 * (lo + hi)


def h4_mass_step(delta_r: float) -> float:
    delta_r_safe = max(float(delta_r), 1e-9)
    return float(h4_cumulative_mass(np.asarray(2.0 * delta_r_safe, dtype=np.float64)))


def shell_boundary_tangent(shell_idx: np.ndarray, delta_r: float, shell_mode: str) -> np.ndarray:
    idx = np.asarray(shell_idx, dtype=np.float64)
    delta_r_safe = max(float(delta_r), 1e-9)
    if shell_mode == "linear":
        return idx * delta_r_safe
    if shell_mode in ("phi_log", "phi_phase"):
        return delta_r_safe * (np.power(PHI, idx) - 1.0)
    if shell_mode == "h4_mass":
        mass_step = h4_mass_step(delta_r_safe)
        geo_boundary = inverse_h4_cumulative_mass(idx * mass_step)
        return 0.5 * geo_boundary
    if shell_mode == "h4_mass_phi":
        mass_step = h4_mass_step(delta_r_safe)
        target_mass = mass_step * (np.power(PHI, idx) - 1.0)
        geo_boundary = inverse_h4_cumulative_mass(target_mass)
        return 0.5 * geo_boundary
    raise ValueError(f"unsupported shell_mode={shell_mode!r}")


def h4_shell_mass(lower_geo: np.ndarray, upper_geo: np.ndarray) -> np.ndarray:
    lower = np.asarray(lower_geo, dtype=np.float64)
    upper = np.asarray(upper_geo, dtype=np.float64)
    return h4_mass_antiderivative(upper) - h4_mass_antiderivative(lower)


def shell_measure_diagnostics(
    shell: np.ndarray,
    delta_r: float,
    shell_mode: str,
) -> Dict[str, float]:
    shell = np.asarray(shell, dtype=np.int64).reshape(-1)
    if shell.size == 0:
        return {
            "shell_mass_error_l1": 0.0,
            "shell_mass_error_max": 0.0,
            "shell_mass_kl": 0.0,
            "shell_mass_corr": 0.0,
            "shell_mass_shells_used": 0,
        }

    max_shell = int(np.max(shell))
    shell_ids = np.arange(max_shell + 1, dtype=np.int64)
    counts = np.bincount(shell, minlength=max_shell + 1).astype(np.float64)
    obs = counts / max(np.sum(counts), 1.0)

    lower_tan = shell_boundary_tangent(shell_ids, delta_r=delta_r, shell_mode=shell_mode)
    upper_tan = shell_boundary_tangent(shell_ids + 1, delta_r=delta_r, shell_mode=shell_mode)
    expected_mass = h4_shell_mass(2.0 * lower_tan, 2.0 * upper_tan)
    expected = expected_mass / max(float(np.sum(expected_mass)), 1e-12)

    return {
        "shell_mass_error_l1": float(np.sum(np.abs(obs - expected))),
        "shell_mass_error_max": float(np.max(np.abs(obs - expected))),
        "shell_mass_kl": float(np.sum(obs * np.log((obs + 1e-12) / (expected + 1e-12)))),
        "shell_mass_corr": safe_corrcoef_1d(obs, expected),
        "shell_mass_shells_used": int(len(shell_ids)),
    }


def hopf_coordinate_components(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
) -> Dict[str, np.ndarray]:
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    rho1 = np.sqrt(a * a + b * b)
    rho2 = np.sqrt(c * c + d * d)
    denom = np.maximum(np.sqrt(rho1 * rho1 + rho2 * rho2), 1e-12)
    cos_chi = rho1 / denom
    sin_chi = rho2 / denom
    chi_u = np.clip(sin_chi * sin_chi, 0.0, 1.0 - 1e-12)
    chi = np.arcsin(np.clip(sin_chi, 0.0, 1.0))
    theta1 = wrap_to_pi(np.arctan2(b, a))
    theta2 = wrap_to_pi(np.arctan2(d, c))
    delta = wrap_to_pi(theta1 - theta2)
    alpha = wrap_to_pi(0.5 * (theta1 + theta2))
    return {
        "rho1": rho1,
        "rho2": rho2,
        "chi": chi,
        "chi_u": chi_u,
        "theta1": theta1,
        "theta2": theta2,
        "delta": delta,
        "alpha": alpha,
    }


def hopf_phase_transport_components(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    phase_transport_lambda: float,
) -> Dict[str, np.ndarray]:
    comp = hopf_coordinate_components(z, dim_i=dim_i, dim_j=dim_j, dim_k=dim_k, dim_l=dim_l)
    chi = comp["chi"]
    delta = comp["delta"]
    alpha = comp["alpha"]
    connection_weight = 0.5 * float(phase_transport_lambda) * np.cos(2.0 * chi)
    phase_shift = wrap_to_pi(connection_weight * delta)
    transported_alpha = wrap_to_pi(alpha + phase_shift)
    return {
        **comp,
        "transport_connection_weight": connection_weight,
        "transport_phase_shift": phase_shift,
        "transported_alpha": transported_alpha,
    }


def hopf_phase_transport_diagnostics(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    phase_transport_lambda: float,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "phase_transport_coherence": 0.0,
            "phase_transport_shift_abs_mean": 0.0,
            "phase_transport_shift_abs_max": 0.0,
            "phase_transport_connection_abs_mean": 0.0,
        }
    comp = hopf_phase_transport_components(
        z,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        phase_transport_lambda=phase_transport_lambda,
    )
    phase_shift = wrap_to_pi(comp["transported_alpha"] - comp["alpha"])
    return {
        "phase_transport_coherence": float(np.mean(np.cos(phase_shift))),
        "phase_transport_shift_abs_mean": float(np.mean(np.abs(phase_shift))),
        "phase_transport_shift_abs_max": float(np.max(np.abs(phase_shift))),
        "phase_transport_connection_abs_mean": float(np.mean(np.abs(comp["transport_connection_weight"]))),
    }


def hopf_phase_transport_complex_components(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
) -> Dict[str, np.ndarray]:
    comp = hopf_phase_transport_components(
        z,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        phase_transport_lambda=phase_transport_lambda,
    )
    field_phase = wrap_to_pi(np.arctan2(z[:, complex_dim_j], z[:, complex_dim_i]))
    field_phase_shift = wrap_to_pi(float(phase_field_lambda) * field_phase)
    transported_alpha = wrap_to_pi(comp["transported_alpha"] + field_phase_shift)
    return {
        **comp,
        "field_phase": field_phase,
        "field_phase_shift": field_phase_shift,
        "transported_alpha": transported_alpha,
    }


def hopf_phase_transport_complex_diagnostics(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "phase_transport_coherence": 0.0,
            "phase_transport_shift_abs_mean": 0.0,
            "phase_transport_shift_abs_max": 0.0,
            "phase_transport_connection_abs_mean": 0.0,
            "phase_transport_field_shift_abs_mean": 0.0,
            "phase_transport_field_weight_abs_mean": 0.0,
        }
    comp = hopf_phase_transport_complex_components(
        z,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        complex_dim_i=complex_dim_i,
        complex_dim_j=complex_dim_j,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
    )
    phase_shift = wrap_to_pi(comp["transported_alpha"] - comp["alpha"])
    return {
        "phase_transport_coherence": float(np.mean(np.cos(phase_shift))),
        "phase_transport_shift_abs_mean": float(np.mean(np.abs(phase_shift))),
        "phase_transport_shift_abs_max": float(np.max(np.abs(phase_shift))),
        "phase_transport_connection_abs_mean": float(np.mean(np.abs(comp["transport_connection_weight"]))),
        "phase_transport_field_shift_abs_mean": float(np.mean(np.abs(comp["field_phase_shift"]))),
        "phase_transport_field_weight_abs_mean": float(np.mean(np.abs(comp["field_phase"]))),
    }


def hopf_product_phase_components(
    z: np.ndarray,
    route_dim_i: int,
    route_dim_j: int,
    route_dim_k: int,
    route_dim_l: int,
    field_dim_i: int,
    field_dim_j: int,
    field_dim_k: int,
    field_dim_l: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
) -> Dict[str, np.ndarray]:
    route_comp = hopf_phase_transport_components(
        z,
        dim_i=route_dim_i,
        dim_j=route_dim_j,
        dim_k=route_dim_k,
        dim_l=route_dim_l,
        phase_transport_lambda=phase_transport_lambda,
    )
    field_comp = hopf_coordinate_components(
        z,
        dim_i=field_dim_i,
        dim_j=field_dim_j,
        dim_k=field_dim_k,
        dim_l=field_dim_l,
    )
    field_coupling_weight = np.sin(2.0 * route_comp["chi"]) * np.sin(2.0 * field_comp["chi"]) * field_comp["alpha"]
    field_phase_shift = wrap_to_pi(float(phase_field_lambda) * field_coupling_weight)
    transported_alpha = wrap_to_pi(route_comp["transported_alpha"] + field_phase_shift)
    return {
        **route_comp,
        "field_chi": field_comp["chi"],
        "field_delta": field_comp["delta"],
        "field_alpha": field_comp["alpha"],
        "field_coupling_weight": field_coupling_weight,
        "field_phase_shift": field_phase_shift,
        "transported_alpha": transported_alpha,
    }


def hopf_product_phase_diagnostics(
    z: np.ndarray,
    route_dim_i: int,
    route_dim_j: int,
    route_dim_k: int,
    route_dim_l: int,
    field_dim_i: int,
    field_dim_j: int,
    field_dim_k: int,
    field_dim_l: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "phase_transport_coherence": 0.0,
            "phase_transport_shift_abs_mean": 0.0,
            "phase_transport_shift_abs_max": 0.0,
            "phase_transport_connection_abs_mean": 0.0,
            "phase_transport_field_shift_abs_mean": 0.0,
            "phase_transport_field_weight_abs_mean": 0.0,
        }
    comp = hopf_product_phase_components(
        z,
        route_dim_i=route_dim_i,
        route_dim_j=route_dim_j,
        route_dim_k=route_dim_k,
        route_dim_l=route_dim_l,
        field_dim_i=field_dim_i,
        field_dim_j=field_dim_j,
        field_dim_k=field_dim_k,
        field_dim_l=field_dim_l,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
    )
    phase_shift = wrap_to_pi(comp["transported_alpha"] - comp["alpha"])
    return {
        "phase_transport_coherence": float(np.mean(np.cos(phase_shift))),
        "phase_transport_shift_abs_mean": float(np.mean(np.abs(phase_shift))),
        "phase_transport_shift_abs_max": float(np.max(np.abs(phase_shift))),
        "phase_transport_connection_abs_mean": float(np.mean(np.abs(comp["transport_connection_weight"]))),
        "phase_transport_field_shift_abs_mean": float(np.mean(np.abs(comp["field_phase_shift"]))),
        "phase_transport_field_weight_abs_mean": float(np.mean(np.abs(comp["field_coupling_weight"]))),
    }


def product_phase_shell_controller(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    adaptive_shell_growth: float,
    adaptive_shell_balance: float,
    adaptive_converge_lambda: float,
    adaptive_converge_target: float,
    adaptive_converge_hysteresis: float,
    adaptive_converge_mode: str,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
) -> Dict[str, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        shell_phase_coupling=0.0,
    )
    mode = str(product_shell_control_mode or "continuous")
    min_bins = max(1, int(adaptive_min_pair_bins))
    k_floor = max(1, min(int(K), min_bins * min_bins))
    denom = max(float(int(K) - k_floor), 1.0)
    capacity_pressure = np.clip((comp["hopf_shell_capacity"] - float(k_floor)) / denom, 0.0, 1.0)
    chi_pressure = (4.0 * comp["chi_u"] * (1.0 - comp["chi_u"])) * (0.5 + 0.5 * comp["div_score"])
    local_concentration = np.clip(comp["shell_ratio"] / PHI, 0.0, 1.0)
    gate_score = chi_pressure * (0.5 + 0.5 * capacity_pressure) * (0.5 + 0.5 * local_concentration)
    gate_score = np.clip(gate_score, 0.0, 1.0)
    gate_threshold = max(0.0, float(product_shell_gate_threshold))
    shell_multiplier = comp["shell_multiplier"].astype(np.float64)

    if mode == "continuous":
        active_mask = np.ones(gate_score.shape, dtype=bool)
        band_state = np.full(gate_score.shape, 2, dtype=np.int64)
        controlled_multiplier = shell_multiplier
    elif mode == "gated":
        active_mask = gate_score >= gate_threshold
        band_state = np.where(active_mask, 2, 0).astype(np.int64)
        controlled_multiplier = np.where(active_mask, shell_multiplier, 1.0)
    elif mode == "banded":
        band_lo = max(0.0, gate_threshold / PHI) if gate_threshold > 0.0 else (1.0 / (PHI * PHI))
        band_hi = gate_threshold if gate_threshold > 0.0 else (1.0 / PHI)
        band_state = np.zeros(gate_score.shape, dtype=np.int64)
        band_state = np.where(gate_score >= band_lo, 1, band_state)
        band_state = np.where(gate_score >= band_hi, 2, band_state)
        active_mask = band_state > 0
        controlled_multiplier = np.ones_like(shell_multiplier)
        controlled_multiplier = np.where(band_state == 1, np.sqrt(np.maximum(shell_multiplier, 1e-12)), controlled_multiplier)
        controlled_multiplier = np.where(band_state == 2, shell_multiplier, controlled_multiplier)
    else:
        raise ValueError(f"unsupported product_shell_control_mode={mode!r}")

    return {
        "shell_multiplier": controlled_multiplier.astype(np.float64),
        "gate_score": gate_score.astype(np.float64),
        "active_mask": active_mask.astype(bool),
        "band_state": band_state.astype(np.int64),
        "chi_pressure": chi_pressure.astype(np.float64),
        "capacity_pressure": capacity_pressure.astype(np.float64),
        "local_concentration": local_concentration.astype(np.float64),
    }


def product_phase_shell_diagnostics(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    adaptive_shell_growth: float,
    adaptive_shell_balance: float,
    adaptive_converge_lambda: float,
    adaptive_converge_target: float,
    adaptive_converge_hysteresis: float,
    adaptive_converge_mode: str,
    product_shell_control_mode: str,
    product_shell_gate_threshold: float,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "product_shell_gate_score_mean": 0.0,
            "product_shell_gate_score_max": 0.0,
            "product_shell_active_frac": 0.0,
            "product_shell_state_mean": 0.0,
            "product_shell_states_used": 0.0,
            "product_shell_chi_pressure_mean": 0.0,
            "product_shell_capacity_pressure_mean": 0.0,
            "product_shell_local_concentration_mean": 0.0,
            "product_shell_multiplier_mean": 1.0,
            "product_shell_multiplier_max": 1.0,
        }
    ctrl = product_phase_shell_controller(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        product_shell_control_mode=product_shell_control_mode,
        product_shell_gate_threshold=product_shell_gate_threshold,
    )
    state_counts = np.unique(ctrl["band_state"]).size
    return {
        "product_shell_gate_score_mean": float(np.mean(ctrl["gate_score"])),
        "product_shell_gate_score_max": float(np.max(ctrl["gate_score"])),
        "product_shell_active_frac": float(np.mean(ctrl["active_mask"].astype(np.float64))),
        "product_shell_state_mean": float(np.mean(ctrl["band_state"])),
        "product_shell_states_used": float(state_counts),
        "product_shell_chi_pressure_mean": float(np.mean(ctrl["chi_pressure"])),
        "product_shell_capacity_pressure_mean": float(np.mean(ctrl["capacity_pressure"])),
        "product_shell_local_concentration_mean": float(np.mean(ctrl["local_concentration"])),
        "product_shell_multiplier_mean": float(np.mean(ctrl["shell_multiplier"])),
        "product_shell_multiplier_max": float(np.max(ctrl["shell_multiplier"])),
    }


def hopf_base_measure_diagnostics(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    chi_bins: int = HOPF_CHI_BINS,
    delta_bins: int = 12,
    alpha_bins: int = 12,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "hopf_base_mass_error": 0.0,
            "hopf_delta_mass_error": 0.0,
            "hopf_alpha_entropy": 0.0,
        }

    comp = hopf_coordinate_components(z, dim_i=dim_i, dim_j=dim_j, dim_k=dim_k, dim_l=dim_l)
    chi_bins = max(2, int(chi_bins))
    delta_bins = max(4, int(delta_bins))
    alpha_bins = max(4, int(alpha_bins))

    chi_ids = np.minimum((comp["chi_u"] * chi_bins).astype(np.int64), chi_bins - 1)
    chi_counts = np.bincount(chi_ids, minlength=chi_bins).astype(np.float64)
    chi_obs = chi_counts / max(np.sum(chi_counts), 1.0)
    chi_exp = np.full((chi_bins,), 1.0 / float(chi_bins), dtype=np.float64)
    chi_mass_error = float(np.sum(np.abs(chi_obs - chi_exp)))

    delta_hist, _ = np.histogram(comp["delta"], bins=delta_bins, range=(-np.pi, np.pi))
    delta_obs = delta_hist.astype(np.float64) / max(np.sum(delta_hist), 1.0)
    delta_exp = np.full((delta_bins,), 1.0 / float(delta_bins), dtype=np.float64)
    delta_mass_error = float(np.sum(np.abs(delta_obs - delta_exp)))

    alpha_hist, _ = np.histogram(comp["alpha"], bins=alpha_bins, range=(-np.pi, np.pi))
    alpha_entropy = entropy_from_counts(alpha_hist) if np.sum(alpha_hist) > 0 else 0.0

    return {
        "hopf_base_mass_error": float(np.mean([chi_mass_error, delta_mass_error])),
        "hopf_delta_mass_error": delta_mass_error,
        "hopf_alpha_entropy": float(alpha_entropy),
    }


def hopf_angular_measure_diagnostics(
    z: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    chi_bins: int = HOPF_CHI_BINS,
    theta_bins: int = 12,
) -> Dict[str, float]:
    if z.shape[0] == 0:
        return {
            "hopf_angular_mass_error": 0.0,
            "hopf_chi_mass_error": 0.0,
            "hopf_theta1_mass_error": 0.0,
            "hopf_theta2_mass_error": 0.0,
            "hopf_theta1_entropy": 0.0,
            "hopf_theta2_entropy": 0.0,
        }

    comp = hopf_coordinate_components(z, dim_i=dim_i, dim_j=dim_j, dim_k=dim_k, dim_l=dim_l)
    chi_bins = max(2, int(chi_bins))
    theta_bins = max(4, int(theta_bins))

    chi_ids = np.minimum((comp["chi_u"] * chi_bins).astype(np.int64), chi_bins - 1)
    chi_counts = np.bincount(chi_ids, minlength=chi_bins).astype(np.float64)
    chi_obs = chi_counts / max(np.sum(chi_counts), 1.0)
    chi_exp = np.full((chi_bins,), 1.0 / float(chi_bins), dtype=np.float64)
    chi_mass_error = float(np.sum(np.abs(chi_obs - chi_exp)))

    theta1_hist, _ = np.histogram(comp["theta1"], bins=theta_bins, range=(-np.pi, np.pi))
    theta2_hist, _ = np.histogram(comp["theta2"], bins=theta_bins, range=(-np.pi, np.pi))
    theta1_obs = theta1_hist.astype(np.float64) / max(np.sum(theta1_hist), 1.0)
    theta2_obs = theta2_hist.astype(np.float64) / max(np.sum(theta2_hist), 1.0)
    theta_exp = np.full((theta_bins,), 1.0 / float(theta_bins), dtype=np.float64)

    theta1_mass_error = float(np.sum(np.abs(theta1_obs - theta_exp)))
    theta2_mass_error = float(np.sum(np.abs(theta2_obs - theta_exp)))
    theta1_entropy = entropy_from_counts(theta1_hist) if np.sum(theta1_hist) > 0 else 0.0
    theta2_entropy = entropy_from_counts(theta2_hist) if np.sum(theta2_hist) > 0 else 0.0

    return {
        "hopf_angular_mass_error": float(np.mean([chi_mass_error, theta1_mass_error, theta2_mass_error])),
        "hopf_chi_mass_error": chi_mass_error,
        "hopf_theta1_mass_error": theta1_mass_error,
        "hopf_theta2_mass_error": theta2_mass_error,
        "hopf_theta1_entropy": float(theta1_entropy),
        "hopf_theta2_entropy": float(theta2_entropy),
    }


def hopf_sector_routing_diagnostics(
    z: np.ndarray,
    sector: np.ndarray,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    alpha_bins: int = 12,
) -> Dict[str, float]:
    sector = np.asarray(sector, dtype=np.int64).reshape(-1)
    if z.shape[0] == 0 or sector.size == 0:
        return {
            "hopf_sector_groups_used": 0,
            "hopf_sector_chi_std_mean": 0.0,
            "hopf_sector_delta_cvar_mean": 0.0,
            "hopf_sector_alpha_entropy_mean": 0.0,
            "hopf_sector_alpha_entropy_gap": 0.0,
        }

    comp = hopf_coordinate_components(z, dim_i=dim_i, dim_j=dim_j, dim_k=dim_k, dim_l=dim_l)
    alpha_bins = max(4, int(alpha_bins))
    global_alpha_hist, _ = np.histogram(comp["alpha"], bins=alpha_bins, range=(-np.pi, np.pi))
    global_alpha_entropy = entropy_from_counts(global_alpha_hist) if np.sum(global_alpha_hist) > 0 else 0.0

    weights = []
    chi_std = []
    delta_cvar = []
    alpha_entropy = []
    for sid in np.unique(sector):
        mask = sector == sid
        count = int(np.sum(mask))
        if count <= 0:
            continue
        weights.append(float(count))
        chi_std.append(float(np.std(comp["chi_u"][mask])))
        delta_vec = np.exp(1j * comp["delta"][mask])
        delta_cvar.append(float(1.0 - np.abs(np.mean(delta_vec))))
        alpha_hist, _ = np.histogram(comp["alpha"][mask], bins=alpha_bins, range=(-np.pi, np.pi))
        alpha_entropy.append(entropy_from_counts(alpha_hist) if np.sum(alpha_hist) > 0 else 0.0)

    if not weights:
        return {
            "hopf_sector_groups_used": 0,
            "hopf_sector_chi_std_mean": 0.0,
            "hopf_sector_delta_cvar_mean": 0.0,
            "hopf_sector_alpha_entropy_mean": 0.0,
            "hopf_sector_alpha_entropy_gap": 0.0,
        }

    w = np.asarray(weights, dtype=np.float64)
    w = w / np.sum(w)
    alpha_entropy_mean = float(np.sum(w * np.asarray(alpha_entropy, dtype=np.float64)))
    return {
        "hopf_sector_groups_used": int(len(weights)),
        "hopf_sector_chi_std_mean": float(np.sum(w * np.asarray(chi_std, dtype=np.float64))),
        "hopf_sector_delta_cvar_mean": float(np.sum(w * np.asarray(delta_cvar, dtype=np.float64))),
        "hopf_sector_alpha_entropy_mean": alpha_entropy_mean,
        "hopf_sector_alpha_entropy_gap": float(global_alpha_entropy - alpha_entropy_mean),
    }


def route_entropy_radius_diagnostics(shell: np.ndarray, sector: np.ndarray) -> Dict[str, float]:
    shell = np.asarray(shell, dtype=np.int64).reshape(-1)
    sector = np.asarray(sector, dtype=np.int64).reshape(-1)
    if shell.size == 0:
        return {
            "route_entropy_radius_corr": 0.0,
            "route_entropy_radius_slope": 0.0,
            "route_entropy_shells_used": 0,
        }
    shell_ids = np.unique(shell)
    entropies = []
    for sid in shell_ids:
        counts = np.unique(sector[shell == sid], return_counts=True)[1]
        entropies.append(entropy_from_counts(counts) if len(counts) else 0.0)
    ent = np.asarray(entropies, dtype=np.float64)
    if shell_ids.size < 2:
        slope = 0.0
    else:
        slope = float(np.polyfit(shell_ids.astype(np.float64), ent, deg=1)[0])
    return {
        "route_entropy_radius_corr": safe_corrcoef_1d(shell_ids.astype(np.float64), ent),
        "route_entropy_radius_slope": slope,
        "route_entropy_shells_used": int(shell_ids.size),
    }


def geodesic_neighborhood_diagnostics(
    v: np.ndarray,
    z: np.ndarray,
    max_points: int = 256,
    k: int = 8,
    seed: int = 0,
) -> Dict[str, float]:
    if v.shape != z.shape:
        raise ValueError(f"Shape mismatch for neighborhood diagnostics: {v.shape} vs {z.shape}")
    n = int(v.shape[0])
    if n < 2:
        return {
            "geodesic_knn_overlap_k": float(k),
            "geodesic_knn_overlap_mean": 0.0,
            "geodesic_knn_jaccard_mean": 0.0,
            "geodesic_knn_points_used": n,
        }
    rs = np.random.RandomState(int(seed))
    take = min(n, max(2, int(max_points)))
    idx = rs.choice(n, size=take, replace=False)
    x_v = exp_map0(v[idx])
    x_z = exp_map0(z[idx])
    dist_v = poincare_distance(x_v[:, None, :], x_v[None, :, :])
    dist_z = poincare_distance(x_z[:, None, :], x_z[None, :, :])
    np.fill_diagonal(dist_v, np.inf)
    np.fill_diagonal(dist_z, np.inf)
    k_eff = min(max(1, int(k)), take - 1)
    nn_v = np.argsort(dist_v, axis=1)[:, :k_eff]
    nn_z = np.argsort(dist_z, axis=1)[:, :k_eff]

    overlap = []
    jaccard = []
    for a, b in zip(nn_v, nn_z):
        sa = set(int(x) for x in a.tolist())
        sb = set(int(x) for x in b.tolist())
        inter = len(sa & sb)
        union = len(sa | sb)
        overlap.append(float(inter) / float(k_eff))
        jaccard.append(float(inter) / float(max(union, 1)))
    return {
        "geodesic_knn_overlap_k": float(k_eff),
        "geodesic_knn_overlap_mean": float(np.mean(overlap)),
        "geodesic_knn_jaccard_mean": float(np.mean(jaccard)),
        "geodesic_knn_points_used": int(take),
    }


# ----------------------------
# Synthetic hierarchical data generator
# ----------------------------

@dataclass
class DataBatch:
    x_ball: np.ndarray  # (N, d)
    y: np.ndarray       # (N, dy)
    depth: np.ndarray   # (N,)
    branch: np.ndarray  # (N,)
    v_tan: np.ndarray   # (N, d)

def build_tree(branching: int, depth: int):
    nodes = []
    top_branch = []
    node_depth = []

    nodes.append((0, -1, 0))
    top_branch.append(-1)
    node_depth.append(0)

    next_id = 1
    frontier = [0]
    for dd in range(1, depth + 1):
        new_frontier = []
        for parent in frontier:
            for b in range(branching):
                nid = next_id
                next_id += 1
                nodes.append((nid, parent, dd))
                tb = b if dd == 1 else top_branch[parent]
                top_branch.append(tb)
                node_depth.append(dd)
                new_frontier.append(nid)
        frontier = new_frontier

    return nodes, top_branch, node_depth

def make_hyperbolic_tree_dataset(
    N: int,
    d: int,
    dy: int,
    branching: int,
    max_depth: int,
    depth_radius: float,
    noise: float,
    seed: int,
    mode: str,
    anis_scale: float,
) -> DataBatch:
    set_seed(seed)
    _, top_branch, node_depth = build_tree(branching, max_depth)
    n_nodes = len(node_depth)

    dirs = np.random.randn(n_nodes, d).astype(np.float64)
    dirs /= safe_norm(dirs, axis=1, keepdims=True)

    radii = np.array([node_depth[i] * depth_radius for i in range(n_nodes)], dtype=np.float64)
    v_node = dirs * radii[:, None]
    y_node = np.random.randn(n_nodes, dy).astype(np.float64)

    node_ids = np.random.randint(0, n_nodes, size=N, dtype=np.int64)
    v = v_node[node_ids].copy()
    v += noise * np.random.randn(N, d).astype(np.float64)

    if mode in ("mix", "anis_mix"):
        frac = 0.35
        m = int(frac * N)
        idx = np.random.choice(N, size=m, replace=False)

        n_flat = 12
        centers = 0.8 * np.random.randn(n_flat, d).astype(np.float64)
        centers /= safe_norm(centers, axis=1, keepdims=True)
        centers *= 0.6
        cid = np.random.randint(0, n_flat, size=m)

        v[idx] = centers[cid] + 0.25 * np.random.randn(m, d).astype(np.float64)

        y = y_node[node_ids].copy()
        y_flat = np.random.randn(n_flat, dy).astype(np.float64)
        y[idx] = y_flat[cid]

        depth_arr = np.array([node_depth[i] for i in node_ids], dtype=np.int64)
        depth_arr[idx] = 0

        branch_arr = np.array([top_branch[i] if top_branch[i] >= 0 else 0 for i in node_ids], dtype=np.int64)
        branch_arr[idx] = -1
    else:
        y = y_node[node_ids].copy()
        depth_arr = np.array([node_depth[i] for i in node_ids], dtype=np.int64)
        branch_arr = np.array([top_branch[i] if top_branch[i] >= 0 else 0 for i in node_ids], dtype=np.int64)

    if mode in ("anis", "anis_mix"):
        scales = np.ones(d, dtype=np.float64)
        perm = np.random.permutation(d)
        hi = perm[: d // 2]
        lo = perm[d // 2 :]
        scales[hi] *= anis_scale
        scales[lo] *= (1.0 / max(anis_scale, 1e-6))
        v *= scales[None, :]

    x_ball = exp_map0(v)
    v_tan = log_map0(x_ball)
    return DataBatch(x_ball=x_ball, y=y, depth=depth_arr, branch=branch_arr, v_tan=v_tan)


# ----------------------------
# Spherical k-means
# ----------------------------

def spherical_kmeans(U: np.ndarray, K: int, iters: int = 25, seed: int = 0) -> np.ndarray:
    set_seed(seed)
    N, d = U.shape
    idx = np.random.choice(N, size=K, replace=False)
    C = U[idx].copy()
    C /= safe_norm(C, axis=1, keepdims=True)

    for _ in range(iters):
        sims = U @ C.T
        a = np.argmax(sims, axis=1)
        for k in range(K):
            mask = (a == k)
            if not np.any(mask):
                C[k] = U[np.random.randint(0, N)]
            else:
                m = np.mean(U[mask], axis=0)
                C[k] = m / safe_norm(m)
    return C

def assign_sectors_kmeans(U: np.ndarray, C: np.ndarray) -> np.ndarray:
    sims = U @ C.T
    return np.argmax(sims, axis=1).astype(np.int64)

def assign_sectors_phase2(z: np.ndarray, K: int, dim_i: int, dim_j: int) -> np.ndarray:
    a = z[:, dim_i]
    b = z[:, dim_j]
    theta = np.arctan2(b, a)
    theta = wrap_to_pi(theta)
    u = (theta + np.pi) / (2.0 * np.pi)  # in [0,1)
    s = (u * K).astype(np.int64)
    s = np.clip(s, 0, K - 1)
    return s

def assign_sectors_phase4d(
    z: np.ndarray, K: int, dim_i: int, dim_j: int, dim_k: int, dim_l: int
) -> np.ndarray:
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    th1 = wrap_to_pi(np.arctan2(b, a))
    th2 = wrap_to_pi(np.arctan2(d, c))
    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)

    k1 = max(1, int(np.floor(np.sqrt(K))))
    k2 = max(1, int(np.ceil(float(K) / float(k1))))
    b1 = np.clip((u1 * k1).astype(np.int64), 0, k1 - 1)
    b2 = np.clip((u2 * k2).astype(np.int64), 0, k2 - 1)
    s = (b1 * k2 + b2) % K
    return s.astype(np.int64)

def assign_sectors_phase4d_adaptive(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    k1 = comp["k1"]
    k2 = comp["k2"]
    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))

    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)

    b1 = np.minimum((u1 * k1).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2).astype(np.int64), np.maximum(k2 - 1, 0))
    sector = (b1 * k2 + b2) % max(int(K), 1)
    return sector.astype(np.int64), k1.astype(np.int64), k2.astype(np.int64)


def assign_sectors_phase4d_hopf(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    k1 = comp["hopf_k1"]
    k2 = comp["hopf_k2"]
    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))

    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    b1 = np.minimum((u1 * k1).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2).astype(np.int64), np.maximum(k2 - 1, 0))
    sector = b1 * k2 + b2
    return sector.astype(np.int64), k1.astype(np.int64), k2.astype(np.int64)


def assign_sectors_phase4d_hopf_base(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    comp = hopf_coordinate_components(z, dim_i=dim_i, dim_j=dim_j, dim_k=dim_k, dim_l=dim_l)
    kchi = max(1, int(np.floor(np.sqrt(max(int(K), 1)))))
    kdelta = max(1, int(np.ceil(float(max(int(K), 1)) / float(kchi))))

    u_chi = comp["chi_u"]
    u_delta = (comp["delta"] + np.pi) / (2.0 * np.pi)

    bchi = np.minimum((u_chi * float(kchi)).astype(np.int64), max(kchi - 1, 0))
    bdelta = np.minimum((u_delta * float(kdelta)).astype(np.int64), max(kdelta - 1, 0))

    sector = (bchi * kdelta + bdelta) % max(int(K), 1)
    return (
        sector.astype(np.int64),
        np.full(z.shape[0], kchi, dtype=np.int64),
        np.full(z.shape[0], kdelta, dtype=np.int64),
    )


def assign_sectors_phase4d_hopf_transport(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    phase_transport_lambda: float,
    hopf_chi_bins: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = hopf_phase_transport_components(
        z,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        phase_transport_lambda=phase_transport_lambda,
    )
    kchi, kdelta, kalpha = allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )

    u_chi = comp["chi_u"]
    u_delta = (comp["delta"] + np.pi) / (2.0 * np.pi)
    u_alpha = (comp["transported_alpha"] + np.pi) / (2.0 * np.pi)

    bchi = np.minimum((u_chi * float(kchi)).astype(np.int64), max(kchi - 1, 0))
    bdelta = np.minimum((u_delta * float(kdelta)).astype(np.int64), max(kdelta - 1, 0))
    balpha = np.minimum((u_alpha * float(kalpha)).astype(np.int64), max(kalpha - 1, 0))

    local_span = max(kdelta * kalpha, 1)
    sector = bchi * local_span + bdelta * kalpha + balpha
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        np.full(z.shape[0], kchi, dtype=np.int64),
        np.full(z.shape[0], kdelta, dtype=np.int64),
        np.full(z.shape[0], kalpha, dtype=np.int64),
    )


def assign_sectors_phase4d_hopf_transport_complex(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
    hopf_chi_bins: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = hopf_phase_transport_complex_components(
        z,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        complex_dim_i=complex_dim_i,
        complex_dim_j=complex_dim_j,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
    )
    kchi, kdelta, kalpha = allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )

    u_chi = comp["chi_u"]
    u_delta = (comp["delta"] + np.pi) / (2.0 * np.pi)
    u_alpha = (comp["transported_alpha"] + np.pi) / (2.0 * np.pi)

    bchi = np.minimum((u_chi * float(kchi)).astype(np.int64), max(kchi - 1, 0))
    bdelta = np.minimum((u_delta * float(kdelta)).astype(np.int64), max(kdelta - 1, 0))
    balpha = np.minimum((u_alpha * float(kalpha)).astype(np.int64), max(kalpha - 1, 0))

    local_span = max(kdelta * kalpha, 1)
    sector = bchi * local_span + bdelta * kalpha + balpha
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        np.full(z.shape[0], kchi, dtype=np.int64),
        np.full(z.shape[0], kdelta, dtype=np.int64),
        np.full(z.shape[0], kalpha, dtype=np.int64),
    )


def assign_sectors_phase4d_hopf_product_phase(
    z: np.ndarray,
    K: int,
    route_dim_i: int,
    route_dim_j: int,
    route_dim_k: int,
    route_dim_l: int,
    field_dim_i: int,
    field_dim_j: int,
    field_dim_k: int,
    field_dim_l: int,
    phase_transport_lambda: float,
    phase_field_lambda: float,
    hopf_chi_bins: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = hopf_product_phase_components(
        z,
        route_dim_i=route_dim_i,
        route_dim_j=route_dim_j,
        route_dim_k=route_dim_k,
        route_dim_l=route_dim_l,
        field_dim_i=field_dim_i,
        field_dim_j=field_dim_j,
        field_dim_k=field_dim_k,
        field_dim_l=field_dim_l,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
    )
    kchi, kdelta, kalpha = allocate_triplet_bins_budget(
        K,
        min_first=max(2, int(hopf_chi_bins)),
        min_second=2,
        min_third=2,
    )

    u_chi = comp["chi_u"]
    u_delta = (comp["delta"] + np.pi) / (2.0 * np.pi)
    u_alpha = (comp["transported_alpha"] + np.pi) / (2.0 * np.pi)

    bchi = np.minimum((u_chi * float(kchi)).astype(np.int64), max(kchi - 1, 0))
    bdelta = np.minimum((u_delta * float(kdelta)).astype(np.int64), max(kdelta - 1, 0))
    balpha = np.minimum((u_alpha * float(kalpha)).astype(np.int64), max(kalpha - 1, 0))

    local_span = max(kdelta * kalpha, 1)
    sector = bchi * local_span + bdelta * kalpha + balpha
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        np.full(z.shape[0], kchi, dtype=np.int64),
        np.full(z.shape[0], kdelta, dtype=np.int64),
        np.full(z.shape[0], kalpha, dtype=np.int64),
    )


def assign_sectors_phase4d_hopf_chi(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    hopf_chi_bins: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    k1 = comp["hopf_k1"]
    k2 = comp["hopf_k2"]
    kchi = max(1, int(hopf_chi_bins))
    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))

    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    u_chi = comp["chi_u"]

    bchi = np.minimum((u_chi * kchi).astype(np.int64), np.maximum(kchi - 1, 0))
    b1 = np.minimum((u1 * k1).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2).astype(np.int64), np.maximum(k2 - 1, 0))

    local_span = np.maximum(k1 * k2, 1)
    sector = bchi * local_span + (b1 * k2 + b2)
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return sector.astype(np.int64), kchi * np.ones_like(k1, dtype=np.int64), k1.astype(np.int64), k2.astype(np.int64)


def assign_sectors_phase4d_hopf_fib(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]

    fib_total = fibonacci_ceil_array(comp["hopf_shell_capacity"], max_value=K)
    kchi = allocate_fibonacci_chi_bins(
        fib_total,
        min_pair_bins=adaptive_min_pair_bins,
        chi_u=comp["chi_u"],
        div_score=comp["div_score"],
    )
    pair_cap = np.maximum(fib_total // np.maximum(kchi, 1), max(1, int(adaptive_min_pair_bins) * int(adaptive_min_pair_bins)))
    hopf_ratio = np.sqrt(
        np.maximum(comp["rho1"], 1e-9) / np.maximum(comp["rho2"], 1e-9)
    )
    k1, k2 = allocate_fibonacci_pair_bins(pair_cap, min_bins=adaptive_min_pair_bins, ratio_scale=hopf_ratio)

    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))
    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    u_chi = comp["chi_u"]

    bchi = np.minimum((u_chi * kchi.astype(np.float64)).astype(np.int64), np.maximum(kchi - 1, 0))
    b1 = np.minimum((u1 * k1.astype(np.float64)).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2.astype(np.float64)).astype(np.int64), np.maximum(k2 - 1, 0))

    local_span = np.maximum(k1 * k2, 1)
    sector = bchi * local_span + (b1 * k2 + b2)
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return sector.astype(np.int64), kchi.astype(np.int64), k1.astype(np.int64), k2.astype(np.int64), fib_total.astype(np.int64)


def assign_sectors_phase4d_hopf_fib_rung(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    fib_rung_gate_threshold: float = 0.0,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]

    fib_total = fibonacci_ceil_array(comp["hopf_shell_capacity"], max_value=K)
    kchi, k1, k2, forced_total = allocate_phi2_rung_bins(
        total_cap=fib_total,
        min_pair_bins=adaptive_min_pair_bins,
        chi_u=comp["chi_u"],
        rho1=comp["rho1"],
        rho2=comp["rho2"],
        div_score=comp["div_score"],
        max_total=K,
    )
    gate_threshold = max(0.0, float(fib_rung_gate_threshold))
    if gate_threshold > 0.0:
        chi_pressure = (4.0 * comp["chi_u"] * (1.0 - comp["chi_u"])) * (0.5 + 0.5 * comp["div_score"])
        gate_mask = chi_pressure >= gate_threshold
        if np.any(~gate_mask):
            kchi = np.where(gate_mask, kchi, 1)
            k1 = np.where(gate_mask, k1, comp["hopf_k1"])
            k2 = np.where(gate_mask, k2, comp["hopf_k2"])
            forced_total = np.where(gate_mask, forced_total, comp["hopf_k1"] * comp["hopf_k2"])

    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))
    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    u_chi = comp["chi_u"]

    bchi = np.minimum((u_chi * kchi.astype(np.float64)).astype(np.int64), np.maximum(kchi - 1, 0))
    b1 = np.minimum((u1 * k1.astype(np.float64)).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2.astype(np.float64)).astype(np.int64), np.maximum(k2 - 1, 0))

    local_span = np.maximum(k1 * k2, 1)
    sector = bchi * local_span + (b1 * k2 + b2)
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        kchi.astype(np.int64),
        k1.astype(np.int64),
        k2.astype(np.int64),
        fib_total.astype(np.int64),
        forced_total.astype(np.int64),
    )


def assign_sectors_phase4d_hopf_fib_band(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
    )
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]

    band, base_state, moderate_state, full_state, fib_total = derive_phi2_band_states(
        comp=comp,
        min_pair_bins=adaptive_min_pair_bins,
        max_total=K,
    )
    state_kchi = np.asarray([base_state[0], moderate_state[0], full_state[0]], dtype=np.int64)
    state_major = np.asarray([base_state[1], moderate_state[1], full_state[1]], dtype=np.int64)
    state_minor = np.asarray([base_state[2], moderate_state[2], full_state[2]], dtype=np.int64)

    kchi = state_kchi[band]
    major = state_major[band]
    minor = state_minor[band]
    dominant_first = comp["rho1"] >= comp["rho2"]
    k1 = np.where(dominant_first, major, minor).astype(np.int64)
    k2 = np.where(dominant_first, minor, major).astype(np.int64)
    state_total = (kchi * major * minor).astype(np.int64)

    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))
    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    u_chi = comp["chi_u"]

    bchi = np.minimum((u_chi * kchi.astype(np.float64)).astype(np.int64), np.maximum(kchi - 1, 0))
    b1 = np.minimum((u1 * k1.astype(np.float64)).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2.astype(np.float64)).astype(np.int64), np.maximum(k2 - 1, 0))

    local_span = np.maximum(k1 * k2, 1)
    sector = bchi * local_span + (b1 * k2 + b2)
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        kchi.astype(np.int64),
        k1.astype(np.int64),
        k2.astype(np.int64),
        fib_total.astype(np.int64),
        state_total.astype(np.int64),
        band.astype(np.int64),
    )


def phase4d_hopf_blend_components(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    shell_mode: str,
    shell_phase_coupling: float,
    hopf_chi_bins: int,
    hopf_blend_lambda: float,
    hopf_blend_chi_weight: float,
    hopf_blend_shell_weight: float,
) -> Dict[str, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=0.0,
        adaptive_shell_balance=0.0,
        adaptive_converge_lambda=0.0,
        adaptive_converge_target=1.0,
        adaptive_converge_hysteresis=0.1,
        adaptive_converge_mode="fixed",
        shell_phase_coupling=shell_phase_coupling,
    )
    chi_bins = max(1, int(hopf_chi_bins))
    chi_ids = np.minimum(
        (comp["chi_u"] * chi_bins).astype(np.int64),
        max(chi_bins - 1, 0),
    )
    chi_pressure, _ = load_pressure(chi_ids)
    shell_ids, _, _ = shell_metric_components(
        comp["r"],
        delta_r,
        shell_mode,
        comp["shell_phase_bias"],
    )
    shell_pressure, _ = load_pressure(shell_ids)

    gap_sum = (comp["hopf_k1_gap"] + comp["hopf_k2_gap"]).astype(np.float64)
    gap_norm = gap_sum / max(float(np.max(gap_sum)), 1e-9)

    chi_measure = 4.0 * comp["chi_u"] * (1.0 - comp["chi_u"])
    chi_signal = chi_pressure * (0.5 + 0.5 * chi_measure)
    denom = 1.0 + max(0.0, float(hopf_blend_chi_weight)) + max(0.0, float(hopf_blend_shell_weight))
    blend_score = comp["div_score"] * (
        gap_norm
        + max(0.0, float(hopf_blend_chi_weight)) * chi_signal
        + max(0.0, float(hopf_blend_shell_weight)) * shell_pressure
    ) / denom
    blend_score = np.clip(blend_score, 0.0, 1.0)

    base_total = (comp["hopf_k1"] * comp["hopf_k2"]).astype(np.int64)
    blend_cap = comp["hopf_shell_capacity"] + float(hopf_blend_lambda) * (float(K) - comp["hopf_shell_capacity"]) * blend_score
    blend_total = np.rint(blend_cap).astype(np.int64)
    blend_total = np.clip(blend_total, base_total, int(K))

    pair_min = max(2, int(adaptive_min_pair_bins) - 1)
    max_kchi_possible = max(1, int(K) // max(1, pair_min * pair_min))
    kchi_target = 1.0 + float(hopf_blend_lambda) * (1.0 + max(0.0, float(hopf_blend_chi_weight)) * chi_signal) * (0.5 + 0.5 * comp["div_score"])
    kchi = np.rint(kchi_target).astype(np.int64)
    kchi = np.clip(kchi, 1, max_kchi_possible)

    min_total = kchi * pair_min * pair_min
    blend_total = np.maximum(blend_total, min_total)
    blend_total = np.clip(blend_total, 1, int(K))

    pair_cap = np.maximum(
        blend_total // np.maximum(kchi, 1),
        pair_min * pair_min,
    )
    pair_cap = np.clip(pair_cap, pair_min * pair_min, int(K))
    hopf_ratio = np.sqrt(np.maximum(comp["rho1"], 1e-9) / np.maximum(comp["rho2"], 1e-9))
    k1, k2 = allocate_pair_bins(pair_cap, min_bins=pair_min, ratio_scale=hopf_ratio)

    return {
        "comp": comp,
        "chi_ids": chi_ids.astype(np.int64),
        "shell_ids": shell_ids.astype(np.int64),
        "chi_pressure": chi_pressure.astype(np.float64),
        "shell_pressure": shell_pressure.astype(np.float64),
        "gap_norm": gap_norm.astype(np.float64),
        "chi_signal": chi_signal.astype(np.float64),
        "blend_score": blend_score.astype(np.float64),
        "blend_total": blend_total.astype(np.int64),
        "kchi": kchi.astype(np.int64),
        "k1": k1.astype(np.int64),
        "k2": k2.astype(np.int64),
    }


def assign_sectors_phase4d_hopf_blend(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    shell_mode: str,
    shell_phase_coupling: float,
    hopf_chi_bins: int,
    hopf_blend_lambda: float,
    hopf_blend_chi_weight: float,
    hopf_blend_shell_weight: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    dbg = phase4d_hopf_blend_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        hopf_chi_bins=hopf_chi_bins,
        hopf_blend_lambda=hopf_blend_lambda,
        hopf_blend_chi_weight=hopf_blend_chi_weight,
        hopf_blend_shell_weight=hopf_blend_shell_weight,
    )
    comp = dbg["comp"]
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]
    theta_shift = comp["theta_shift"]
    th1 = wrap_to_pi(np.arctan2(b, a) + theta_shift)
    th2 = wrap_to_pi(np.arctan2(d, c) - (theta_shift / PHI))
    u1 = (th1 + np.pi) / (2.0 * np.pi)
    u2 = (th2 + np.pi) / (2.0 * np.pi)
    u_chi = comp["chi_u"]
    kchi = dbg["kchi"]
    k1 = dbg["k1"]
    k2 = dbg["k2"]

    bchi = np.minimum((u_chi * kchi.astype(np.float64)).astype(np.int64), np.maximum(kchi - 1, 0))
    b1 = np.minimum((u1 * k1.astype(np.float64)).astype(np.int64), np.maximum(k1 - 1, 0))
    b2 = np.minimum((u2 * k2.astype(np.float64)).astype(np.int64), np.maximum(k2 - 1, 0))

    local_span = np.maximum(k1 * k2, 1)
    sector = bchi * local_span + (b1 * k2 + b2)
    sector = np.minimum(sector, max(int(K) - 1, 0))
    return (
        sector.astype(np.int64),
        kchi.astype(np.int64),
        k1.astype(np.int64),
        k2.astype(np.int64),
        dbg["blend_total"].astype(np.int64),
        dbg["blend_score"].astype(np.float64),
        dbg["chi_pressure"].astype(np.float64),
        dbg["shell_pressure"].astype(np.float64),
    )


def local_complex_bin_shape(local_k: int) -> Tuple[int, int]:
    local_k = max(1, int(local_k))
    k_angle = max(1, int(np.floor(np.sqrt(local_k))))
    k_rad = max(1, int(np.ceil(float(local_k) / float(k_angle))))
    return k_angle, k_rad

def phase4d_adaptive_components(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    delta_r: float,
    tau: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    adaptive_shell_growth: float,
    adaptive_shell_balance: float,
    adaptive_converge_lambda: float,
    adaptive_converge_target: float,
    adaptive_converge_hysteresis: float,
    adaptive_converge_mode: str,
    shell_phase_coupling: float = 0.0,
) -> Dict[str, np.ndarray]:
    a = z[:, dim_i]
    b = z[:, dim_j]
    c = z[:, dim_k]
    d = z[:, dim_l]

    theta1 = wrap_to_pi(np.arctan2(b, a))
    theta2 = wrap_to_pi(np.arctan2(d, c))
    phase_gap = wrap_to_pi(theta1 - theta2)
    rho1 = np.sqrt(a * a + b * b)
    rho2 = np.sqrt(c * c + d * d)
    rho_sum = np.maximum(rho1 + rho2, 1e-12)
    balance = (rho1 - rho2) / rho_sum
    chi = np.arctan2(rho2, np.maximum(rho1, 1e-12))
    chi_entropy = histogram_entropy(chi, bins=HOPF_CHI_BINS, value_range=(0.0, 0.5 * np.pi))

    r = safe_norm(z, axis=1, keepdims=False)
    alpha_core = max(float(delta_r), 1e-9)
    r_alpha = np.arcsinh(r / alpha_core)
    r_alpha_hat = r_alpha / np.maximum(r_alpha + 1.0, 1e-12)
    r_hat = np.clip(r / np.maximum(r + max(float(delta_r), 1e-9), 1e-12), 0.0, 1.0)
    tau_clip = float(np.clip(tau, 0.0, 1.0))

    div_score = 1.0 - np.exp(-max(0.0, float(adaptive_time_growth)) * np.pi * tau_clip * r_hat)

    min_bins = max(1, int(adaptive_min_pair_bins))
    k_floor = max(1, min(K, min_bins * min_bins))
    if K <= k_floor:
        k_eff = np.full(z.shape[0], max(1, int(K)), dtype=np.int64)
    else:
        k_eff = np.rint(k_floor + (float(K - k_floor) * div_score)).astype(np.int64)
        k_eff = np.clip(k_eff, k_floor, K)

    phi_scale = np.power(PHI, float(adaptive_balance) * balance)
    k1, k2 = allocate_pair_bins(k_eff, min_bins=min_bins, ratio_scale=phi_scale)

    hopf_growth_arg = max(0.0, float(adaptive_time_growth)) * np.pi * tau_clip * r_alpha_hat
    hopf_growth_raw = np.sinh(hopf_growth_arg)
    hopf_growth_cap = max(np.sinh(max(0.0, float(adaptive_time_growth)) * np.pi), 1e-9)
    hopf_capacity_frac = np.power(hopf_growth_raw / hopf_growth_cap, 3.0)
    hopf_shell_capacity = k_floor + (float(K - k_floor) * hopf_capacity_frac)
    hopf_shell_capacity = np.clip(hopf_shell_capacity, k_floor, K)
    hopf_k_eff = np.rint(hopf_shell_capacity).astype(np.int64)

    hopf_cos = rho1 / np.maximum(np.sqrt(rho1 * rho1 + rho2 * rho2), 1e-12)
    hopf_sin = rho2 / np.maximum(np.sqrt(rho1 * rho1 + rho2 * rho2), 1e-12)
    chi_u = hopf_sin * hopf_sin
    hopf_ratio = np.sqrt(np.maximum(hopf_cos, 1e-9) / np.maximum(hopf_sin, 1e-9))
    hopf_k1, hopf_k2 = allocate_pair_bins(hopf_k_eff, min_bins=min_bins, ratio_scale=hopf_ratio)
    hopf_k1_gap = np.abs(hopf_k1 - k1)
    hopf_k2_gap = np.abs(hopf_k2 - k2)

    theta_shift = float(adaptive_angle_growth) * GOLDEN_ANGLE * tau_clip * r_hat * balance
    phase_pressure = div_score * np.abs(balance) * np.sin(phase_gap + theta_shift)
    shell_phase_bias = float(shell_phase_coupling) * phase_pressure

    shell_drive = div_score * (1.0 + float(adaptive_shell_balance) * np.abs(balance))
    shell_expand = float(adaptive_shell_growth) * shell_drive
    converge_target = max(1e-9, float(adaptive_converge_target))
    converge_hysteresis = max(0.0, float(adaptive_converge_hysteresis))
    shell_target_band = converge_target + converge_hysteresis
    shell_overflow = np.maximum(shell_expand - shell_target_band, 0.0)
    shell_ratio = shell_expand / max(shell_target_band, 1e-9)
    shell_ratio_pressure = np.maximum(shell_ratio - 1.0, 0.0)
    shell_ladder_steps = np.zeros(shell_expand.shape, dtype=np.int64)
    if adaptive_converge_mode == "fixed":
        shell_converge = float(adaptive_converge_lambda) * shell_overflow
    elif adaptive_converge_mode == "phi_ratio":
        shell_converge = (
            float(adaptive_converge_lambda)
            * shell_target_band
            * shell_ratio_pressure
            / PHI
        )
    elif adaptive_converge_mode == "phi_ladder":
        shell_ladder_steps = np.ceil(shell_overflow / max(LOG_PHI, 1e-9)).astype(np.int64)
        shell_ladder_steps = np.where(shell_overflow > 1e-12, np.maximum(shell_ladder_steps, 1), 0)
        shell_converge = float(adaptive_converge_lambda) * shell_ladder_steps.astype(np.float64) * LOG_PHI
    else:
        raise ValueError(f"unsupported adaptive_converge_mode={adaptive_converge_mode!r}")
    shell_multiplier = np.exp(shell_expand - shell_converge)

    return {
        "rho1": rho1,
        "rho2": rho2,
        "chi": chi,
        "chi_u": chi_u,
        "chi_entropy": np.full(chi.shape, chi_entropy, dtype=np.float64),
        "theta1": theta1,
        "theta2": theta2,
        "phase_gap": phase_gap,
        "phase_pressure": phase_pressure,
        "shell_phase_bias": shell_phase_bias,
        "balance": balance,
        "r": r,
        "r_alpha": r_alpha,
        "r_alpha_hat": r_alpha_hat,
        "r_hat": r_hat,
        "div_score": div_score,
        "k1": k1.astype(np.int64),
        "k2": k2.astype(np.int64),
        "hopf_shell_capacity": hopf_shell_capacity,
        "hopf_k1": hopf_k1.astype(np.int64),
        "hopf_k2": hopf_k2.astype(np.int64),
        "hopf_k1_gap": hopf_k1_gap.astype(np.int64),
        "hopf_k2_gap": hopf_k2_gap.astype(np.int64),
        "theta_shift": theta_shift,
        "shell_drive": shell_drive,
        "shell_expand": shell_expand,
        "shell_target_band": np.full(shell_expand.shape, shell_target_band, dtype=np.float64),
        "shell_overflow": shell_overflow,
        "shell_ratio": shell_ratio,
        "shell_ratio_pressure": shell_ratio_pressure,
        "shell_ladder_steps": shell_ladder_steps,
        "shell_converge": shell_converge,
        "shell_multiplier": shell_multiplier,
    }


def shell_metric_components(
    r_eff: np.ndarray,
    delta_r: float,
    shell_mode: str,
    shell_phase_bias: Optional[np.ndarray] = None,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    delta_r_safe = max(float(delta_r), 1e-9)
    r_eff_clip = np.maximum(r_eff.astype(np.float64), 0.0)
    if shell_mode == "linear":
        shell_cont = r_eff_clip / delta_r_safe
    elif shell_mode == "phi_log":
        shell_cont = np.log1p(r_eff_clip / delta_r_safe) / max(LOG_PHI, 1e-9)
    elif shell_mode == "phi_phase":
        phase_bias = 0.0 if shell_phase_bias is None else np.asarray(shell_phase_bias, dtype=np.float64)
        shell_cont = np.log1p(r_eff_clip / delta_r_safe) / max(LOG_PHI, 1e-9)
        shell_cont = np.maximum(shell_cont + phase_bias, 0.0)
    elif shell_mode == "h4_mass":
        mass_step = h4_mass_step(delta_r_safe)
        shell_cont = h4_cumulative_mass(2.0 * r_eff_clip) / max(mass_step, 1e-12)
    elif shell_mode == "h4_mass_phi":
        mass_step = h4_mass_step(delta_r_safe)
        shell_cont = np.log1p(h4_cumulative_mass(2.0 * r_eff_clip) / max(mass_step, 1e-12)) / max(LOG_PHI, 1e-9)
    else:
        raise ValueError(f"unsupported shell_mode={shell_mode!r}")
    shell = np.floor(shell_cont + 1e-12).astype(np.int64)
    shell_frac = np.clip(shell_cont - shell.astype(np.float64), 0.0, 1.0 - 1e-12)
    return shell, shell_frac, shell_cont

def assign_sectors_complex2(z: np.ndarray, K: int, dim_i: int, dim_j: int) -> np.ndarray:
    a = z[:, dim_i]
    b = z[:, dim_j]
    theta = wrap_to_pi(np.arctan2(b, a))
    r = np.sqrt(a * a + b * b)
    rmax = float(np.max(r)) if len(r) else 1.0
    rmax = max(rmax, 1e-9)
    ur = np.clip(r / rmax, 0.0, 1.0 - 1e-12)
    ua = (theta + np.pi) / (2.0 * np.pi)

    k_angle = max(1, int(np.floor(np.sqrt(K))))
    k_rad = max(1, int(np.ceil(float(K) / float(k_angle))))
    ba = np.clip((ua * k_angle).astype(np.int64), 0, k_angle - 1)
    br = np.clip((ur * k_rad).astype(np.int64), 0, k_rad - 1)
    s = (ba * k_rad + br) % K
    return s.astype(np.int64)


def phase4d_complex_local_components(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    delta_r: float,
    tau: float,
    time_pressure_lambda: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    adaptive_shell_growth: float,
    adaptive_shell_balance: float,
    adaptive_converge_lambda: float,
    adaptive_converge_target: float,
    adaptive_converge_hysteresis: float,
    adaptive_converge_mode: str,
    shell_mode: str,
    hybrid_local_k: int,
    hybrid_complex_roots: int,
    hybrid_local_min_k: int,
    hybrid_local_target: float,
    hybrid_local_hysteresis: float,
    hybrid_local_converge_lambda: float,
    shell_phase_coupling: float = 0.0,
) -> Dict[str, np.ndarray]:
    comp = phase4d_adaptive_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        shell_phase_coupling=shell_phase_coupling,
    )
    coarse_sector, _, _ = assign_sectors_phase4d_adaptive(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        delta_r=delta_r,
        tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
    )

    r_eff = comp["r"]
    if time_pressure_lambda != 0.0:
        r_eff = r_eff * np.exp(float(time_pressure_lambda) * float(tau))
    r_eff = r_eff * comp["shell_multiplier"]
    shell_phase_bias = comp["shell_phase_bias"] if shell_mode == "phi_phase" else None
    shell, shell_frac, _ = shell_metric_components(
        r_eff,
        delta_r=delta_r,
        shell_mode=shell_mode,
        shell_phase_bias=shell_phase_bias,
    )

    a = z[:, complex_dim_i]
    b = z[:, complex_dim_j]
    rho_local = np.sqrt(a * a + b * b)
    rho_local_hat = np.clip(rho_local / np.maximum(rho_local + max(float(delta_r), 1e-9), 1e-12), 0.0, 1.0)

    local_drive = comp["div_score"] * rho_local_hat
    local_target_band = max(1e-9, float(hybrid_local_target) + max(0.0, float(hybrid_local_hysteresis)))
    local_overflow = np.maximum(local_drive - local_target_band, 0.0)
    local_ratio = local_drive / local_target_band
    local_ratio_pressure = np.maximum(local_ratio - 1.0, 0.0)
    # Local zoom should respond to relative pressure, not absolute shell-scale magnitude.
    local_activation = np.clip(local_ratio_pressure / PHI, 0.0, 1.0)
    local_converge = float(hybrid_local_converge_lambda) * comp["shell_ratio_pressure"] / PHI
    local_activation = np.clip(local_activation - local_converge, 0.0, 1.0)

    local_k = max(1, int(hybrid_local_k))
    local_min_k = max(1, min(int(hybrid_local_min_k), local_k))
    local_k_eff = local_min_k + np.rint((local_k - local_min_k) * local_activation).astype(np.int64)
    local_k_eff = np.clip(local_k_eff, local_min_k, local_k)

    theta = wrap_to_pi(np.arctan2(b, a))
    roots = max(1, int(hybrid_complex_roots))
    root_step = (2.0 * np.pi) / float(roots)
    theta = wrap_to_pi(theta + ((coarse_sector % roots).astype(np.float64) * root_step))

    base_k_angle, base_k_rad = local_complex_bin_shape(local_k)
    ua = (theta + np.pi) / (2.0 * np.pi)
    ur = shell_frac
    ba = np.clip((ua * base_k_angle).astype(np.int64), 0, base_k_angle - 1)
    br = np.clip((ur * base_k_rad).astype(np.int64), 0, base_k_rad - 1)
    local_unit = ((ba * base_k_rad) + br).astype(np.float64) / float(local_k)
    local_sector = np.minimum((local_unit * local_k_eff.astype(np.float64)).astype(np.int64), np.maximum(local_k_eff - 1, 0))
    sector = coarse_sector.astype(np.int64) * local_k + local_sector.astype(np.int64)

    return {
        "coarse_sector": coarse_sector.astype(np.int64),
        "local_sector": local_sector.astype(np.int64),
        "sector": sector.astype(np.int64),
        "shell": shell.astype(np.int64),
        "shell_frac": shell_frac,
        "rho_local": rho_local,
        "rho_local_hat": rho_local_hat,
        "local_drive": local_drive,
        "local_target_band": np.full(local_drive.shape, local_target_band, dtype=np.float64),
        "local_overflow": local_overflow,
        "local_ratio": local_ratio,
        "local_ratio_pressure": local_ratio_pressure,
        "local_converge": local_converge,
        "local_activation": local_activation,
        "local_k_eff": local_k_eff.astype(np.int64),
        "comp": comp,
    }


def assign_sectors_phase4d_complex_local(
    z: np.ndarray,
    K: int,
    dim_i: int,
    dim_j: int,
    dim_k: int,
    dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    delta_r: float,
    tau: float,
    time_pressure_lambda: float,
    adaptive_min_pair_bins: int,
    adaptive_time_growth: float,
    adaptive_balance: float,
    adaptive_angle_growth: float,
    adaptive_shell_growth: float,
    adaptive_shell_balance: float,
    adaptive_converge_lambda: float,
    adaptive_converge_target: float,
    adaptive_converge_hysteresis: float,
    adaptive_converge_mode: str,
    shell_mode: str,
    shell_phase_coupling: float,
    hopf_chi_bins: int,
    hybrid_local_k: int,
    hybrid_complex_roots: int,
    hybrid_local_min_k: int,
    hybrid_local_target: float,
    hybrid_local_hysteresis: float,
    hybrid_local_converge_lambda: float,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray, Dict[str, np.ndarray]]:
    hybrid = phase4d_complex_local_components(
        z=z,
        K=K,
        dim_i=dim_i,
        dim_j=dim_j,
        dim_k=dim_k,
        dim_l=dim_l,
        complex_dim_i=complex_dim_i,
        complex_dim_j=complex_dim_j,
        delta_r=delta_r,
        tau=tau,
        time_pressure_lambda=time_pressure_lambda,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        hybrid_local_k=hybrid_local_k,
        hybrid_complex_roots=hybrid_complex_roots,
        hybrid_local_min_k=hybrid_local_min_k,
        hybrid_local_target=hybrid_local_target,
        hybrid_local_hysteresis=hybrid_local_hysteresis,
        hybrid_local_converge_lambda=hybrid_local_converge_lambda,
    )
    return hybrid["shell"], hybrid["sector"], hybrid["local_sector"], hybrid


# ----------------------------
# Router + memory
# ----------------------------

@dataclass
class Slot:
    proto: np.ndarray  # (d,)
    mem: np.ndarray    # (dy,)

@dataclass
class Bucket:
    slots: List[Slot]


# ----------------------------
# Chart representation (global or radial-bin)
# ----------------------------

@dataclass
class Chart:
    R: np.ndarray                          # (d,d)
    s_global: Optional[np.ndarray] = None  # (d,) if global
    S_radial: Optional[np.ndarray] = None  # (B,d) if radial
    scale_mode: str = "global"             # "global" or "radial"
    radial_rmax: float = 0.0               # >0 if radial
    radial_bins: int = 0                   # B if radial

def _radial_bins_index(r: np.ndarray, B: int, rmax: float) -> np.ndarray:
    if rmax <= 0:
        return np.zeros_like(r, dtype=np.int64)
    u = np.clip(r / rmax, 0.0, 1.0 - 1e-12)
    b = (u * B).astype(np.int64)
    b = np.clip(b, 0, B - 1)
    return b

def apply_chart(v: np.ndarray, chart: Chart) -> np.ndarray:
    """
    v: (N,d)
    returns z: (N,d)
    """
    w = (v @ chart.R.T).astype(np.float64)

    if chart.s_global is None and chart.S_radial is None:
        return w

    if chart.scale_mode == "global":
        s = chart.s_global
        if s is None:
            return w
        return (w * np.exp(s)[None, :]).astype(np.float64)

    S = chart.S_radial
    if S is None or chart.radial_bins <= 0:
        return w
    r = safe_norm(w, axis=1, keepdims=False)
    b = _radial_bins_index(r, chart.radial_bins, chart.radial_rmax)
    scales = np.exp(S[b])  # (N,d)
    return (w * scales).astype(np.float64)


def apply_chart_isometric(v: np.ndarray, chart: Chart) -> np.ndarray:
    """
    Rotation-only routing coordinate.

    This preserves the learned SO(d) orientation while intentionally ignoring
    learned scale so shell and sector routing can share a more isometric chart.
    """
    return (v @ chart.R.T).astype(np.float64)


def apply_chart_route_blend(v: np.ndarray, chart: Chart, route_scale_lambda: float) -> np.ndarray:
    """
    Blend between the rotation-only route coordinate and the learned scaled chart.

    route_scale_lambda=0.0 -> pure isometry
    route_scale_lambda=1.0 -> full learned chart scale
    """
    lam = float(np.clip(route_scale_lambda, 0.0, 1.0))
    w = apply_chart_isometric(v, chart)
    if lam <= 0.0:
        return w

    if chart.s_global is None and chart.S_radial is None:
        return w

    if chart.scale_mode == "global":
        s = chart.s_global
        if s is None:
            return w
        return (w * np.exp(lam * s)[None, :]).astype(np.float64)

    S = chart.S_radial
    if S is None or chart.radial_bins <= 0:
        return w
    r = safe_norm(w, axis=1, keepdims=False)
    b = _radial_bins_index(r, chart.radial_bins, chart.radial_rmax)
    scales = np.exp(lam * S[b])
    return (w * scales).astype(np.float64)


def route_coordinate(
    v: np.ndarray,
    chart: Chart,
    sector_mode: str,
    route_scale_lambda: float,
) -> np.ndarray:
    z = apply_chart(v, chart)
    if sector_mode in ("phase4d_hopf_iso", "phase4d_hopf_fib_band_iso"):
        return apply_chart_isometric(v, chart)
    if sector_mode == "phase4d_hopf_fib_band_bound":
        return apply_chart_route_blend(v, chart, route_scale_lambda=route_scale_lambda)
    return z

def route_addresses(
    v: np.ndarray,
    delta_r: float,
    C: Optional[np.ndarray],
    chart: Chart,
    sector_mode: str,
    phase_dim_i: int,
    phase_dim_j: int,
    phase4_dim_i: int,
    phase4_dim_j: int,
    phase4_dim_k: int,
    phase4_dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    K: int,
    field4_dim_i: int = 1,
    field4_dim_j: int = 3,
    field4_dim_k: int = 5,
    field4_dim_l: int = 7,
    time_pressure_lambda: float = 0.0,
    tau: float = 1.0,
    adaptive_min_pair_bins: int = 2,
    adaptive_time_growth: float = 1.0,
    adaptive_balance: float = 1.0,
    adaptive_angle_growth: float = 0.35,
    adaptive_shell_growth: float = 0.0,
    adaptive_shell_balance: float = 0.0,
    adaptive_converge_lambda: float = 0.0,
    adaptive_converge_target: float = 1.0,
    adaptive_converge_hysteresis: float = 0.1,
    adaptive_converge_mode: str = "fixed",
    fib_rung_gate_threshold: float = 0.0,
    route_scale_lambda: float = 1.0,
    memory_coord_mode: str = "route_chart",
    shell_mode: str = "linear",
    shell_phase_coupling: float = 0.0,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
    hopf_chi_bins: int = 2,
    hopf_blend_lambda: float = 0.8,
    hopf_blend_chi_weight: float = 1.0,
    hopf_blend_shell_weight: float = 0.5,
    phase_transport_lambda: float = 1.0,
    phase_field_lambda: float = 0.0,
    hybrid_local_k: int = 4,
    hybrid_complex_roots: int = 4,
    hybrid_local_min_k: int = 1,
    hybrid_local_target: float = 0.60,
    hybrid_local_hysteresis: float = 0.05,
    hybrid_local_converge_lambda: float = 1.0,
    shell_pressure_w: float = 0.0,
):
    """
    v: (N,d)
    Returns: shell, sector, U (unit dir), z
    - shell uses r_eff = r * exp(lambda * tau)
    - sector uses either kmeans(U,C) or phase2(z)
    - shell_pressure_w: blend weight in [0,1] between chart radius and geodesic radius
      (0 = pure chart, 1 = pure geodesic; only active for phase4d_hopf_base)
    """
    z = apply_chart(v, chart)
    route_z = route_coordinate(v, chart, sector_mode=sector_mode, route_scale_lambda=route_scale_lambda)
    if memory_coord_mode not in ("route_chart", "full_chart"):
        raise ValueError("memory_coord_mode must be one of {'route_chart','full_chart'}")
    memory_z = z if memory_coord_mode == "full_chart" else route_z
    memory_u = memory_z / safe_norm(memory_z, axis=1, keepdims=True)

    r = safe_norm(route_z, axis=1, keepdims=False)
    shell_r_base = r
    if sector_mode in ("phase4d_hopf_ball", "phase4d_hopf_base_ball"):
        shell_r_base = poincare_radius(exp_map0(v))
    # Bounded blend: gently pull shell boundaries toward geodesic-correct mass
    # without the hard substitution that collapsed shells in INC-0136.
    if shell_pressure_w > 0.0 and sector_mode == "phase4d_hopf_base":
        w = float(np.clip(shell_pressure_w, 0.0, 1.0))
        r_geodesic = poincare_radius(exp_map0(v))
        shell_r_base = (1.0 - w) * r + w * r_geodesic

    if time_pressure_lambda != 0.0:
        r_eff = shell_r_base * np.exp(float(time_pressure_lambda) * float(tau))
    else:
        r_eff = shell_r_base

    if sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        comp = phase4d_adaptive_components(
            z=route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
            adaptive_shell_growth=adaptive_shell_growth,
            adaptive_shell_balance=adaptive_shell_balance,
            adaptive_converge_lambda=adaptive_converge_lambda,
            adaptive_converge_target=adaptive_converge_target,
            adaptive_converge_hysteresis=adaptive_converge_hysteresis,
            adaptive_converge_mode=adaptive_converge_mode,
            shell_phase_coupling=shell_phase_coupling,
        )
        if sector_mode == "phase4d_hopf_product_phase":
            shell_ctrl = product_phase_shell_controller(
                z=route_z,
                K=K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=delta_r,
                tau=tau,
                adaptive_min_pair_bins=adaptive_min_pair_bins,
                adaptive_time_growth=adaptive_time_growth,
                adaptive_balance=adaptive_balance,
                adaptive_angle_growth=adaptive_angle_growth,
                adaptive_shell_growth=adaptive_shell_growth,
                adaptive_shell_balance=adaptive_shell_balance,
                adaptive_converge_lambda=adaptive_converge_lambda,
                adaptive_converge_target=adaptive_converge_target,
                adaptive_converge_hysteresis=adaptive_converge_hysteresis,
                adaptive_converge_mode=adaptive_converge_mode,
                product_shell_control_mode=product_shell_control_mode,
                product_shell_gate_threshold=product_shell_gate_threshold,
            )
            comp = {**comp, "shell_multiplier": shell_ctrl["shell_multiplier"]}
        r_eff = r_eff * comp["shell_multiplier"]

    shell_phase_bias = None
    if sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local") and shell_mode == "phi_phase":
        shell_phase_bias = comp["shell_phase_bias"]
    shell, _, _ = shell_metric_components(
        r_eff,
        delta_r=delta_r,
        shell_mode=shell_mode,
        shell_phase_bias=shell_phase_bias,
    )

    if sector_mode == "phase2":
        sector = assign_sectors_phase2(route_z, K=K, dim_i=phase_dim_i, dim_j=phase_dim_j)
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d":
        sector = assign_sectors_phase4d(
            route_z, K=K, dim_i=phase4_dim_i, dim_j=phase4_dim_j, dim_k=phase4_dim_k, dim_l=phase4_dim_l
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_adaptive":
        sector, _, _ = assign_sectors_phase4d_adaptive(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode in ("phase4d_hopf", "phase4d_hopf_iso", "phase4d_hopf_ball"):
        sector, _, _ = assign_sectors_phase4d_hopf(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode in ("phase4d_hopf_base", "phase4d_hopf_base_ball"):
        sector, _, _ = assign_sectors_phase4d_hopf_base(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_transport":
        sector, _, _, _ = assign_sectors_phase4d_hopf_transport(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            phase_transport_lambda=phase_transport_lambda,
            hopf_chi_bins=hopf_chi_bins,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_transport_complex":
        sector, _, _, _ = assign_sectors_phase4d_hopf_transport_complex(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            phase_transport_lambda=phase_transport_lambda,
            phase_field_lambda=phase_field_lambda,
            hopf_chi_bins=hopf_chi_bins,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_product_phase":
        sector, _, _, _ = assign_sectors_phase4d_hopf_product_phase(
            route_z,
            K=K,
            route_dim_i=phase4_dim_i,
            route_dim_j=phase4_dim_j,
            route_dim_k=phase4_dim_k,
            route_dim_l=phase4_dim_l,
            field_dim_i=field4_dim_i,
            field_dim_j=field4_dim_j,
            field_dim_k=field4_dim_k,
            field_dim_l=field4_dim_l,
            phase_transport_lambda=phase_transport_lambda,
            phase_field_lambda=phase_field_lambda,
            hopf_chi_bins=hopf_chi_bins,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_chi":
        sector, _, _, _ = assign_sectors_phase4d_hopf_chi(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
            hopf_chi_bins=hopf_chi_bins,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_fib":
        sector, _, _, _, _ = assign_sectors_phase4d_hopf_fib(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_fib_rung":
        sector, _, _, _, _, _ = assign_sectors_phase4d_hopf_fib_rung(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
            fib_rung_gate_threshold=fib_rung_gate_threshold,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode in ("phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound"):
        sector, _, _, _, _, _, _ = assign_sectors_phase4d_hopf_fib_band(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_hopf_blend":
        sector, _, _, _, _, _, _, _ = assign_sectors_phase4d_hopf_blend(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=delta_r,
            tau=tau,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
            shell_mode=shell_mode,
            shell_phase_coupling=shell_phase_coupling,
            hopf_chi_bins=hopf_chi_bins,
            hopf_blend_lambda=hopf_blend_lambda,
            hopf_blend_chi_weight=hopf_blend_chi_weight,
            hopf_blend_shell_weight=hopf_blend_shell_weight,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "phase4d_complex_local":
        shell, sector, _, _ = assign_sectors_phase4d_complex_local(
            route_z,
            K=K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            delta_r=delta_r,
            tau=tau,
            time_pressure_lambda=time_pressure_lambda,
            adaptive_min_pair_bins=adaptive_min_pair_bins,
            adaptive_time_growth=adaptive_time_growth,
            adaptive_balance=adaptive_balance,
            adaptive_angle_growth=adaptive_angle_growth,
            adaptive_shell_growth=adaptive_shell_growth,
            adaptive_shell_balance=adaptive_shell_balance,
            adaptive_converge_lambda=adaptive_converge_lambda,
            adaptive_converge_target=adaptive_converge_target,
            adaptive_converge_hysteresis=adaptive_converge_hysteresis,
            adaptive_converge_mode=adaptive_converge_mode,
            shell_mode=shell_mode,
            shell_phase_coupling=shell_phase_coupling,
            hopf_chi_bins=hopf_chi_bins,
            hybrid_local_k=hybrid_local_k,
            hybrid_complex_roots=hybrid_complex_roots,
            hybrid_local_min_k=hybrid_local_min_k,
            hybrid_local_target=hybrid_local_target,
            hybrid_local_hysteresis=hybrid_local_hysteresis,
            hybrid_local_converge_lambda=hybrid_local_converge_lambda,
        )
        return shell, sector, memory_u, memory_z
    if sector_mode == "complex2":
        sector = assign_sectors_complex2(route_z, K=K, dim_i=complex_dim_i, dim_j=complex_dim_j)
        U = route_z / safe_norm(route_z, axis=1, keepdims=True)
        return shell, sector, U, route_z

    # kmeans default
    U = memory_u
    if C is None:
        raise ValueError("C cannot be None when sector_mode='kmeans'")
    sector = assign_sectors_kmeans(U, C)
    return shell, sector, U, memory_z

def route_one(
    v1: np.ndarray,
    delta_r: float,
    C: Optional[np.ndarray],
    chart: Chart,
    sector_mode: str,
    phase_dim_i: int,
    phase_dim_j: int,
    phase4_dim_i: int,
    phase4_dim_j: int,
    phase4_dim_k: int,
    phase4_dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    K: int,
    field4_dim_i: int = 1,
    field4_dim_j: int = 3,
    field4_dim_k: int = 5,
    field4_dim_l: int = 7,
    time_pressure_lambda: float = 0.0,
    tau: float = 1.0,
    adaptive_min_pair_bins: int = 2,
    adaptive_time_growth: float = 1.0,
    adaptive_balance: float = 1.0,
    adaptive_angle_growth: float = 0.35,
    adaptive_shell_growth: float = 0.0,
    adaptive_shell_balance: float = 0.0,
    adaptive_converge_lambda: float = 0.0,
    adaptive_converge_target: float = 1.0,
    adaptive_converge_hysteresis: float = 0.1,
    adaptive_converge_mode: str = "fixed",
    fib_rung_gate_threshold: float = 0.0,
    route_scale_lambda: float = 1.0,
    memory_coord_mode: str = "route_chart",
    shell_mode: str = "linear",
    shell_phase_coupling: float = 0.0,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
    hopf_chi_bins: int = 2,
    hopf_blend_lambda: float = 0.8,
    hopf_blend_chi_weight: float = 1.0,
    hopf_blend_shell_weight: float = 0.5,
    phase_transport_lambda: float = 1.0,
    phase_field_lambda: float = 0.0,
    hybrid_local_k: int = 4,
    hybrid_complex_roots: int = 4,
    hybrid_local_min_k: int = 1,
    hybrid_local_target: float = 0.60,
    hybrid_local_hysteresis: float = 0.05,
    hybrid_local_converge_lambda: float = 1.0,
) -> Tuple[Tuple[int, int], np.ndarray]:
    vv = v1.reshape(1, -1)
    shell, sector, _, z = route_addresses(
        vv, delta_r=delta_r, C=C, chart=chart,
        sector_mode=sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        K=K,
        time_pressure_lambda=time_pressure_lambda, tau=tau,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        fib_rung_gate_threshold=fib_rung_gate_threshold,
        route_scale_lambda=route_scale_lambda,
        memory_coord_mode=memory_coord_mode,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        product_shell_control_mode=product_shell_control_mode,
        product_shell_gate_threshold=product_shell_gate_threshold,
        hopf_chi_bins=hopf_chi_bins,
        hopf_blend_lambda=hopf_blend_lambda,
        hopf_blend_chi_weight=hopf_blend_chi_weight,
        hopf_blend_shell_weight=hopf_blend_shell_weight,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
        hybrid_local_k=hybrid_local_k,
        hybrid_complex_roots=hybrid_complex_roots,
        hybrid_local_min_k=hybrid_local_min_k,
        hybrid_local_target=hybrid_local_target,
        hybrid_local_hysteresis=hybrid_local_hysteresis,
        hybrid_local_converge_lambda=hybrid_local_converge_lambda,
    )
    key = (int(shell[0]), int(sector[0]))
    return key, z[0]

def make_bucket_key(shell: int, sector: int) -> Tuple[int, int]:
    return (int(shell), int(sector))

def make_default_bucket(d: int, dy: int) -> Bucket:
    proto = np.zeros(d, dtype=np.float64)
    mem = np.zeros(dy, dtype=np.float64)
    return Bucket(slots=[Slot(proto=proto, mem=mem)])

def get_bucket(buckets: Dict[Tuple[int, int], Bucket], key: Tuple[int, int], d: int, dy: int) -> Bucket:
    b = buckets.get(key, None)
    if b is None:
        b = make_default_bucket(d, dy)
        buckets[key] = b
    return b

def init_buckets(keys: Iterable[Tuple[int, int]], dy: int, d: int, seed: int) -> Dict[Tuple[int, int], Bucket]:
    set_seed(seed)
    buckets: Dict[Tuple[int, int], Bucket] = {}
    for k in keys:
        if k not in buckets:
            proto = 0.01 * np.random.randn(d).astype(np.float64)
            mem = np.zeros(dy, dtype=np.float64)
            buckets[k] = Bucket(slots=[Slot(proto=proto, mem=mem)])
    return buckets

def predict_from_bucket(
    buckets: Dict[Tuple[int, int], Bucket],
    key: Tuple[int, int],
    z: np.ndarray,
    d: int,
    dy: int
):
    b = get_bucket(buckets, key, d=d, dy=dy)
    best_j = 0
    best_d2 = float("inf")
    for j, s in enumerate(b.slots):
        d2 = float(np.sum((z - s.proto) ** 2))
        if d2 < best_d2:
            best_d2 = d2
            best_j = j
    return b.slots[best_j].mem, best_j, best_d2

def ema_update(slot: Slot, z: np.ndarray, y: np.ndarray, eta_p: float, eta_m: float):
    slot.proto = (1.0 - eta_p) * slot.proto + eta_p * z
    slot.mem = (1.0 - eta_m) * slot.mem + eta_m * y

def local_loss(yhat: np.ndarray, y: np.ndarray) -> float:
    e = yhat - y
    return float(np.mean(e * e))


# ----------------------------
# Growth (loss-based splitting)
# ----------------------------

def kmeans2_multi_init(X: np.ndarray, n_init: int = 4, iters: int = 10):
    N, d = X.shape
    best_inertia = float("inf")
    best_c = None
    best_l = None
    for _ in range(n_init):
        idx = np.random.choice(N, size=2, replace=False)
        C = X[idx].copy()
        for _ in range(iters):
            d0 = np.sum((X - C[0]) ** 2, axis=1)
            d1 = np.sum((X - C[1]) ** 2, axis=1)
            L = (d1 < d0).astype(np.int64)
            if np.any(L == 0):
                C[0] = np.mean(X[L == 0], axis=0)
            if np.any(L == 1):
                C[1] = np.mean(X[L == 1], axis=0)
        d0 = np.sum((X - C[0]) ** 2, axis=1)
        d1 = np.sum((X - C[1]) ** 2, axis=1)
        inertia = float(np.sum(np.minimum(d0, d1)))
        if inertia < best_inertia:
            best_inertia = inertia
            best_c = C.copy()
            best_l = L.copy()
    return best_c, best_l

def loss_based_splitting(
    buckets: Dict[Tuple[int, int], Bucket],
    keys: List[Tuple[int, int]],
    z: np.ndarray,
    y: np.ndarray,
    max_slots_per_bucket: int,
    extra_budget: int,
    eta_p: float,
    eta_m: float,
    n_split_rounds: int,
    min_gain: float,
    seed: int,
    d: int,
    dy: int
):
    set_seed(seed)
    N = z.shape[0]

    def compute_assignments():
        bucket_slot_to_indices: Dict[Tuple[int, int, int], List[int]] = {}
        losses = np.zeros(N, dtype=np.float64)

        for i in range(N):
            k = keys[i]
            yhat, sj, _ = predict_from_bucket(buckets, k, z[i], d=d, dy=dy)
            losses[i] = local_loss(yhat, y[i])
            ks = (k[0], k[1], int(sj))
            bucket_slot_to_indices.setdefault(ks, []).append(i)

        return bucket_slot_to_indices, losses

    created = 0
    accepted = 0

    for _round in range(n_split_rounds):
        if created >= extra_budget:
            break

        bucket_slot_to_idx, losses = compute_assignments()

        cand = []
        for ks, idxs in bucket_slot_to_idx.items():
            shell, sector, sj = ks
            bkey = (shell, sector)
            b = get_bucket(buckets, bkey, d=d, dy=dy)
            if len(b.slots) >= max_slots_per_bucket:
                continue
            if len(idxs) < 16:
                continue
            m = float(np.mean(losses[idxs]))
            cand.append((m, bkey, int(sj), idxs))

        if not cand:
            break

        cand.sort(reverse=True, key=lambda t: t[0])

        did_split = False
        for _, bkey, sj, idxs in cand[:10]:
            if created >= extra_budget:
                break

            X = z[idxs]
            Y = y[idxs]
            n = len(idxs)

            perm = np.random.permutation(n)
            n_train = int(0.7 * n)
            tr = perm[:n_train]
            te = perm[n_train:]

            b = get_bucket(buckets, bkey, d=d, dy=dy)
            cur_slot = b.slots[sj]
            yhat_cur = np.tile(cur_slot.mem[None, :], (len(te), 1))
            cur_loss = float(np.mean((yhat_cur - Y[te]) ** 2))

            C2, L2 = kmeans2_multi_init(X[tr], n_init=4, iters=8)
            group0 = (L2 == 0)
            group1 = (L2 == 1)
            if not np.any(group0) or not np.any(group1):
                continue

            mem0 = np.mean(Y[tr][group0], axis=0)
            mem1 = np.mean(Y[tr][group1], axis=0)

            d0 = np.sum((X[te] - C2[0]) ** 2, axis=1)
            d1 = np.sum((X[te] - C2[1]) ** 2, axis=1)
            use1 = (d1 < d0)

            yhat_split = np.empty_like(Y[te])
            yhat_split[~use1] = mem0
            yhat_split[use1] = mem1

            split_loss = float(np.mean((yhat_split - Y[te]) ** 2))
            gain = cur_loss - split_loss

            if gain > min_gain:
                cur_slot.proto = (1.0 - eta_p) * cur_slot.proto + eta_p * C2[0]
                cur_slot.mem = (1.0 - eta_m) * cur_slot.mem + eta_m * mem0
                new_slot = Slot(proto=C2[1].copy(), mem=mem1.copy())
                b.slots.append(new_slot)

                created += 1
                accepted += 1
                did_split = True
                break

        if not did_split:
            break

    return created, accepted


# ----------------------------
# Chart optimization: learn R (SO(d)) and/or scaling (global or radial bins)
# ----------------------------

@dataclass
class ChartOptResult:
    chart: Chart
    loss_hist: List[float]
    s_global: Optional[np.ndarray]
    S_radial: Optional[np.ndarray]

def label_coherence_sse(
    v: np.ndarray,
    y: np.ndarray,
    delta_r: float,
    C: Optional[np.ndarray],
    chart: Chart,
    sector_mode: str,
    phase_dim_i: int,
    phase_dim_j: int,
    phase4_dim_i: int,
    phase4_dim_j: int,
    phase4_dim_k: int,
    phase4_dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    K: int,
    field4_dim_i: int = 1,
    field4_dim_j: int = 3,
    field4_dim_k: int = 5,
    field4_dim_l: int = 7,
    adaptive_min_pair_bins: int = 2,
    adaptive_time_growth: float = 1.0,
    adaptive_balance: float = 1.0,
    adaptive_angle_growth: float = 0.35,
    adaptive_shell_growth: float = 0.0,
    adaptive_shell_balance: float = 0.0,
    adaptive_converge_lambda: float = 0.0,
    adaptive_converge_target: float = 1.0,
    adaptive_converge_hysteresis: float = 0.1,
    adaptive_converge_mode: str = "fixed",
    fib_rung_gate_threshold: float = 0.0,
    route_scale_lambda: float = 1.0,
    memory_coord_mode: str = "route_chart",
    shell_mode: str = "linear",
    shell_phase_coupling: float = 0.0,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
    hopf_chi_bins: int = 2,
    hopf_blend_lambda: float = 0.8,
    hopf_blend_chi_weight: float = 1.0,
    hopf_blend_shell_weight: float = 0.5,
    phase_transport_lambda: float = 1.0,
    phase_field_lambda: float = 0.0,
    hybrid_local_k: int = 4,
    hybrid_complex_roots: int = 4,
    hybrid_local_min_k: int = 1,
    hybrid_local_target: float = 0.60,
    hybrid_local_hysteresis: float = 0.05,
    hybrid_local_converge_lambda: float = 1.0,
) -> float:
    shell, sector, _, _ = route_addresses(
        v, delta_r=delta_r, C=C, chart=chart,
        sector_mode=sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=K,
        time_pressure_lambda=0.0, tau=1.0,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        fib_rung_gate_threshold=fib_rung_gate_threshold,
        route_scale_lambda=route_scale_lambda,
        memory_coord_mode=memory_coord_mode,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        product_shell_control_mode=product_shell_control_mode,
        product_shell_gate_threshold=product_shell_gate_threshold,
        hopf_chi_bins=hopf_chi_bins,
        hopf_blend_lambda=hopf_blend_lambda,
        hopf_blend_chi_weight=hopf_blend_chi_weight,
        hopf_blend_shell_weight=hopf_blend_shell_weight,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
        hybrid_local_k=hybrid_local_k,
        hybrid_complex_roots=hybrid_complex_roots,
        hybrid_local_min_k=hybrid_local_min_k,
        hybrid_local_target=hybrid_local_target,
        hybrid_local_hysteresis=hybrid_local_hysteresis,
        hybrid_local_converge_lambda=hybrid_local_converge_lambda,
    )
    key_hash = shell.astype(np.int64) * 1000003 + sector.astype(np.int64)
    uniq, inv, counts = np.unique(key_hash, return_inverse=True, return_counts=True)
    B = len(uniq)
    dy = y.shape[1]
    sums = np.zeros((B, dy), dtype=np.float64)
    np.add.at(sums, inv, y)
    means = sums / counts[:, None]
    dif = y - means[inv]
    return float(np.sum(dif * dif))

def chart_objective(
    v: np.ndarray,
    y: np.ndarray,
    delta_r: float,
    C: Optional[np.ndarray],
    chart: Chart,
    sector_mode: str,
    phase_dim_i: int,
    phase_dim_j: int,
    phase4_dim_i: int,
    phase4_dim_j: int,
    phase4_dim_k: int,
    phase4_dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    K: int,
    alpha_overload: float,
    beta_bucketcount: float,
    radial_l2: float,
    field4_dim_i: int = 1,
    field4_dim_j: int = 3,
    field4_dim_k: int = 5,
    field4_dim_l: int = 7,
    adaptive_min_pair_bins: int = 2,
    adaptive_time_growth: float = 1.0,
    adaptive_balance: float = 1.0,
    adaptive_angle_growth: float = 0.35,
    adaptive_shell_growth: float = 0.0,
    adaptive_shell_balance: float = 0.0,
    adaptive_converge_lambda: float = 0.0,
    adaptive_converge_target: float = 1.0,
    adaptive_converge_hysteresis: float = 0.1,
    adaptive_converge_mode: str = "fixed",
    fib_rung_gate_threshold: float = 0.0,
    route_scale_lambda: float = 1.0,
    memory_coord_mode: str = "route_chart",
    shell_mode: str = "linear",
    shell_phase_coupling: float = 0.0,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
    hopf_chi_bins: int = 2,
    hopf_blend_lambda: float = 0.8,
    hopf_blend_chi_weight: float = 1.0,
    hopf_blend_shell_weight: float = 0.5,
    phase_transport_lambda: float = 1.0,
    phase_field_lambda: float = 0.0,
    hybrid_local_k: int = 4,
    hybrid_complex_roots: int = 4,
    hybrid_local_min_k: int = 1,
    hybrid_local_target: float = 0.60,
    hybrid_local_hysteresis: float = 0.05,
    hybrid_local_converge_lambda: float = 1.0,
) -> float:
    shell, sector, _, _ = route_addresses(
        v, delta_r=delta_r, C=C, chart=chart,
        sector_mode=sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=K,
        time_pressure_lambda=0.0, tau=1.0,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        fib_rung_gate_threshold=fib_rung_gate_threshold,
        route_scale_lambda=route_scale_lambda,
        memory_coord_mode=memory_coord_mode,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        product_shell_control_mode=product_shell_control_mode,
        product_shell_gate_threshold=product_shell_gate_threshold,
        hopf_chi_bins=hopf_chi_bins,
        hopf_blend_lambda=hopf_blend_lambda,
        hopf_blend_chi_weight=hopf_blend_chi_weight,
        hopf_blend_shell_weight=hopf_blend_shell_weight,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
        hybrid_local_k=hybrid_local_k,
        hybrid_complex_roots=hybrid_complex_roots,
        hybrid_local_min_k=hybrid_local_min_k,
        hybrid_local_target=hybrid_local_target,
        hybrid_local_hysteresis=hybrid_local_hysteresis,
        hybrid_local_converge_lambda=hybrid_local_converge_lambda,
    )
    key_hash = shell.astype(np.int64) * 1000003 + sector.astype(np.int64)
    uniq, inv, counts = np.unique(key_hash, return_inverse=True, return_counts=True)

    B = len(uniq)
    dy = y.shape[1]
    sums = np.zeros((B, dy), dtype=np.float64)
    np.add.at(sums, inv, y)
    means = sums / counts[:, None]
    dif = y - means[inv]
    sse = float(np.sum(dif * dif))

    p = counts / np.sum(counts)
    over = float(np.sum(p * p))

    reg = 0.0
    if chart.scale_mode == "radial" and chart.S_radial is not None and radial_l2 > 0:
        reg = float(radial_l2 * np.sum(chart.S_radial * chart.S_radial))
    elif chart.scale_mode == "global" and chart.s_global is not None and radial_l2 > 0:
        reg = float(radial_l2 * np.sum(chart.s_global * chart.s_global))

    return sse + alpha_overload * over + beta_bucketcount * float(B) + reg

def _make_chart(
    d: int,
    learn_so8: int,
    learn_scale: int,
    scale_mode: str,
    R: np.ndarray,
    s_global: Optional[np.ndarray],
    S_radial: Optional[np.ndarray],
    radial_bins: int,
    radial_rmax: float
) -> Chart:
    R_use = R if learn_so8 == 1 else np.eye(d, dtype=np.float64)

    if learn_scale == 0:
        return Chart(R=R_use, s_global=None, S_radial=None, scale_mode="global")

    if scale_mode == "global":
        assert s_global is not None
        return Chart(R=R_use, s_global=s_global, S_radial=None, scale_mode="global")
    else:
        assert S_radial is not None
        return Chart(
            R=R_use, s_global=None, S_radial=S_radial, scale_mode="radial",
            radial_bins=radial_bins, radial_rmax=radial_rmax
        )

def optimize_chart(
    v_train: np.ndarray,
    y_train: np.ndarray,
    delta_r: float,
    C0: Optional[np.ndarray],
    learn_so8: int,
    learn_scale: int,
    scale_mode: str,
    radial_bins: int,
    radial_rmax: float,
    radial_update_frac: float,
    radial_l2: float,
    iters: int,
    so8_step: float,
    so8_candidates: int,
    scale_step: float,
    scale_candidates: int,
    scale_clip: float,
    alpha_overload: float,
    beta_bucketcount: float,
    sector_mode: str,
    phase_dim_i: int,
    phase_dim_j: int,
    phase4_dim_i: int,
    phase4_dim_j: int,
    phase4_dim_k: int,
    phase4_dim_l: int,
    complex_dim_i: int,
    complex_dim_j: int,
    K: int,
    seed: int,
    early_stop_patience: int,
    early_stop_min_delta: float,
    field4_dim_i: int = 1,
    field4_dim_j: int = 3,
    field4_dim_k: int = 5,
    field4_dim_l: int = 7,
    adaptive_min_pair_bins: int = 2,
    adaptive_time_growth: float = 1.0,
    adaptive_balance: float = 1.0,
    adaptive_angle_growth: float = 0.35,
    adaptive_shell_growth: float = 0.0,
    adaptive_shell_balance: float = 0.0,
    adaptive_converge_lambda: float = 0.0,
    adaptive_converge_target: float = 1.0,
    adaptive_converge_hysteresis: float = 0.1,
    adaptive_converge_mode: str = "fixed",
    fib_rung_gate_threshold: float = 0.0,
    route_scale_lambda: float = 1.0,
    memory_coord_mode: str = "route_chart",
    shell_mode: str = "linear",
    shell_phase_coupling: float = 0.0,
    product_shell_control_mode: str = "continuous",
    product_shell_gate_threshold: float = 0.0,
    hopf_chi_bins: int = 2,
    hopf_blend_lambda: float = 0.8,
    hopf_blend_chi_weight: float = 1.0,
    hopf_blend_shell_weight: float = 0.5,
    phase_transport_lambda: float = 1.0,
    phase_field_lambda: float = 0.0,
    hybrid_local_k: int = 4,
    hybrid_complex_roots: int = 4,
    hybrid_local_min_k: int = 1,
    hybrid_local_target: float = 0.60,
    hybrid_local_hysteresis: float = 0.05,
    hybrid_local_converge_lambda: float = 1.0,
) -> ChartOptResult:
    set_seed(seed)
    d = v_train.shape[1]

    R = ensure_det_plus_one(np.eye(d, dtype=np.float64))

    s_global = np.zeros(d, dtype=np.float64)
    S_radial = None
    if learn_scale == 1 and scale_mode == "radial":
        if radial_bins <= 0:
            raise ValueError("radial_bins must be > 0 when scale_mode=radial")
        S_radial = np.zeros((radial_bins, d), dtype=np.float64)

    # auto radial rmax if needed
    if learn_scale == 1 and scale_mode == "radial":
        if radial_rmax <= 0:
            radial_rmax = float(np.quantile(safe_norm(v_train, axis=1, keepdims=False), 0.995))
            radial_rmax = max(radial_rmax, 1e-6)

    chart = _make_chart(
        d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
        R=R,
        s_global=(s_global if (learn_scale == 1 and scale_mode == "global") else None),
        S_radial=(S_radial if (learn_scale == 1 and scale_mode == "radial") else None),
        radial_bins=radial_bins,
        radial_rmax=radial_rmax
    )

    L = chart_objective(
        v_train, y_train, delta_r, C0, chart,
        sector_mode=sector_mode, phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=K,
        alpha_overload=alpha_overload, beta_bucketcount=beta_bucketcount, radial_l2=radial_l2,
        adaptive_min_pair_bins=adaptive_min_pair_bins,
        adaptive_time_growth=adaptive_time_growth,
        adaptive_balance=adaptive_balance,
        adaptive_angle_growth=adaptive_angle_growth,
        adaptive_shell_growth=adaptive_shell_growth,
        adaptive_shell_balance=adaptive_shell_balance,
        adaptive_converge_lambda=adaptive_converge_lambda,
        adaptive_converge_target=adaptive_converge_target,
        adaptive_converge_hysteresis=adaptive_converge_hysteresis,
        adaptive_converge_mode=adaptive_converge_mode,
        fib_rung_gate_threshold=fib_rung_gate_threshold,
        route_scale_lambda=route_scale_lambda,
        memory_coord_mode=memory_coord_mode,
        shell_mode=shell_mode,
        shell_phase_coupling=shell_phase_coupling,
        product_shell_control_mode=product_shell_control_mode,
        product_shell_gate_threshold=product_shell_gate_threshold,
        hopf_chi_bins=hopf_chi_bins,
        hopf_blend_lambda=hopf_blend_lambda,
        hopf_blend_chi_weight=hopf_blend_chi_weight,
        hopf_blend_shell_weight=hopf_blend_shell_weight,
        phase_transport_lambda=phase_transport_lambda,
        phase_field_lambda=phase_field_lambda,
        hybrid_local_k=hybrid_local_k,
        hybrid_complex_roots=hybrid_complex_roots,
        hybrid_local_min_k=hybrid_local_min_k,
        hybrid_local_target=hybrid_local_target,
        hybrid_local_hysteresis=hybrid_local_hysteresis,
        hybrid_local_converge_lambda=hybrid_local_converge_lambda,
    )
    hist = [L]

    if learn_so8 == 0 and learn_scale == 0:
        return ChartOptResult(chart=chart, loss_hist=hist, s_global=None, S_radial=None)

    best_seen = L
    no_improve = 0

    for _ in range(iters):
        best_L = L
        best_R = R
        best_sg = s_global
        best_Sr = S_radial

        # Rotation proposals
        if learn_so8 == 1:
            for _c in range(so8_candidates):
                S = random_skew(d, scale=so8_step)
                Q = cayley_from_skew(S)
                R_try = ensure_det_plus_one(Q @ R)

                chart_try = _make_chart(
                    d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
                    R=R_try,
                    s_global=(best_sg if (learn_scale == 1 and scale_mode == "global") else None),
                    S_radial=(best_Sr if (learn_scale == 1 and scale_mode == "radial") else None),
                    radial_bins=radial_bins,
                    radial_rmax=radial_rmax
                )
                L_try = chart_objective(
                    v_train, y_train, delta_r, C0, chart_try,
                    sector_mode=sector_mode, phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
                    phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
                    phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
                    field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
                    field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
                    K=K,
                    alpha_overload=alpha_overload, beta_bucketcount=beta_bucketcount, radial_l2=radial_l2,
                    adaptive_min_pair_bins=adaptive_min_pair_bins,
                    adaptive_time_growth=adaptive_time_growth,
                    adaptive_balance=adaptive_balance,
                    adaptive_angle_growth=adaptive_angle_growth,
                    adaptive_shell_growth=adaptive_shell_growth,
                    adaptive_shell_balance=adaptive_shell_balance,
                    adaptive_converge_lambda=adaptive_converge_lambda,
                    adaptive_converge_target=adaptive_converge_target,
                    adaptive_converge_hysteresis=adaptive_converge_hysteresis,
                    adaptive_converge_mode=adaptive_converge_mode,
                    fib_rung_gate_threshold=fib_rung_gate_threshold,
                    route_scale_lambda=route_scale_lambda,
                    memory_coord_mode=memory_coord_mode,
                    shell_mode=shell_mode,
                    shell_phase_coupling=shell_phase_coupling,
                    product_shell_control_mode=product_shell_control_mode,
                    product_shell_gate_threshold=product_shell_gate_threshold,
                    hopf_chi_bins=hopf_chi_bins,
                    hopf_blend_lambda=hopf_blend_lambda,
                    hopf_blend_chi_weight=hopf_blend_chi_weight,
                    hopf_blend_shell_weight=hopf_blend_shell_weight,
                    phase_transport_lambda=phase_transport_lambda,
                    phase_field_lambda=phase_field_lambda,
                    hybrid_local_k=hybrid_local_k,
                    hybrid_complex_roots=hybrid_complex_roots,
                    hybrid_local_min_k=hybrid_local_min_k,
                    hybrid_local_target=hybrid_local_target,
                    hybrid_local_hysteresis=hybrid_local_hysteresis,
                    hybrid_local_converge_lambda=hybrid_local_converge_lambda,
                )
                if L_try < best_L:
                    best_L = L_try
                    best_R = R_try

        # Scale proposals
        if learn_scale == 1:
            for _c in range(scale_candidates):
                if scale_mode == "global":
                    s_try = best_sg.copy()
                    mask = (np.random.rand(d) < 0.5)
                    if not np.any(mask):
                        mask[np.random.randint(0, d)] = True
                    s_try[mask] += scale_step * np.random.randn(np.sum(mask)).astype(np.float64)
                    s_try = clip_scales(s_try, scale_clip)

                    chart_try = _make_chart(
                        d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
                        R=(best_R if learn_so8 == 1 else R),
                        s_global=s_try,
                        S_radial=None,
                        radial_bins=0,
                        radial_rmax=0.0
                    )
                else:
                    assert best_Sr is not None
                    Sr_try = best_Sr.copy()
                    B = Sr_try.shape[0]
                    frac = float(np.clip(radial_update_frac, 1.0 / B, 1.0))
                    nb = max(1, int(round(frac * B)))
                    bins = np.random.choice(B, size=nb, replace=False)

                    for b in bins:
                        mask = (np.random.rand(d) < 0.5)
                        if not np.any(mask):
                            mask[np.random.randint(0, d)] = True
                        Sr_try[b, mask] += scale_step * np.random.randn(np.sum(mask)).astype(np.float64)

                    Sr_try = clip_scales(Sr_try, scale_clip)

                    chart_try = _make_chart(
                        d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
                        R=(best_R if learn_so8 == 1 else R),
                        s_global=None,
                        S_radial=Sr_try,
                        radial_bins=radial_bins,
                        radial_rmax=radial_rmax
                    )

                L_try = chart_objective(
                    v_train, y_train, delta_r, C0, chart_try,
                    sector_mode=sector_mode, phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
                    phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
                    phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
                    K=K,
                    alpha_overload=alpha_overload, beta_bucketcount=beta_bucketcount, radial_l2=radial_l2,
                    adaptive_min_pair_bins=adaptive_min_pair_bins,
                    adaptive_time_growth=adaptive_time_growth,
                    adaptive_balance=adaptive_balance,
                    adaptive_angle_growth=adaptive_angle_growth,
                    adaptive_shell_growth=adaptive_shell_growth,
                    adaptive_shell_balance=adaptive_shell_balance,
                    adaptive_converge_lambda=adaptive_converge_lambda,
                    adaptive_converge_target=adaptive_converge_target,
                    adaptive_converge_hysteresis=adaptive_converge_hysteresis,
                    adaptive_converge_mode=adaptive_converge_mode,
                    fib_rung_gate_threshold=fib_rung_gate_threshold,
                    route_scale_lambda=route_scale_lambda,
                    memory_coord_mode=memory_coord_mode,
                    shell_mode=shell_mode,
                    shell_phase_coupling=shell_phase_coupling,
                    product_shell_control_mode=product_shell_control_mode,
                    product_shell_gate_threshold=product_shell_gate_threshold,
                    hopf_chi_bins=hopf_chi_bins,
                    hopf_blend_lambda=hopf_blend_lambda,
                    hopf_blend_chi_weight=hopf_blend_chi_weight,
                    hopf_blend_shell_weight=hopf_blend_shell_weight,
                    phase_transport_lambda=phase_transport_lambda,
                    phase_field_lambda=phase_field_lambda,
                    hybrid_local_k=hybrid_local_k,
                    hybrid_complex_roots=hybrid_complex_roots,
                    hybrid_local_min_k=hybrid_local_min_k,
                    hybrid_local_target=hybrid_local_target,
                    hybrid_local_hysteresis=hybrid_local_hysteresis,
                    hybrid_local_converge_lambda=hybrid_local_converge_lambda,
                )
                if L_try < best_L:
                    best_L = L_try
                    if scale_mode == "global":
                        best_sg = chart_try.s_global
                        best_Sr = None
                    else:
                        best_Sr = chart_try.S_radial

        if best_L < L:
            R = best_R
            s_global = best_sg
            S_radial = best_Sr
            chart = _make_chart(
                d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
                R=R,
                s_global=(s_global if (learn_scale == 1 and scale_mode == "global") else None),
                S_radial=(S_radial if (learn_scale == 1 and scale_mode == "radial") else None),
                radial_bins=radial_bins,
                radial_rmax=radial_rmax
            )
            L = best_L

        hist.append(L)
        if L < (best_seen - float(early_stop_min_delta)):
            best_seen = L
            no_improve = 0
        else:
            no_improve += 1
            if early_stop_patience > 0 and no_improve >= early_stop_patience:
                break

    chart = _make_chart(
        d=d, learn_so8=learn_so8, learn_scale=learn_scale, scale_mode=scale_mode,
        R=R,
        s_global=(s_global if (learn_scale == 1 and scale_mode == "global") else None),
        S_radial=(S_radial if (learn_scale == 1 and scale_mode == "radial") else None),
        radial_bins=radial_bins,
        radial_rmax=radial_rmax
    )

    return ChartOptResult(chart=chart, loss_hist=hist, s_global=chart.s_global, S_radial=chart.S_radial)


# ----------------------------
# Metrics + diagnostics
# ----------------------------

def unseen_key_rate(keys_te: List[Tuple[int, int]], train_key_set: set) -> float:
    if not keys_te:
        return 0.0
    unseen = sum(1 for k in keys_te if k not in train_key_set)
    return float(unseen) / float(len(keys_te))

def eval_metrics(
    buckets: Dict[Tuple[int, int], Bucket],
    keys: List[Tuple[int, int]],
    z: np.ndarray,
    y: np.ndarray
):
    Nloc = z.shape[0]
    losses = np.zeros(Nloc, dtype=np.float64)

    sh = np.array([k[0] for k in keys], dtype=np.int64)
    se = np.array([k[1] for k in keys], dtype=np.int64)
    key_hash = sh * 1000003 + se
    _, counts = np.unique(key_hash, return_counts=True)

    pmax = float(np.max(counts) / np.sum(counts)) if len(counts) else 0.0
    H = entropy_from_counts(counts) if len(counts) else 0.0

    d = z.shape[1]
    dy = next(iter(buckets.values())).slots[0].mem.shape[0] if buckets else y.shape[1]

    for i in range(Nloc):
        yhat, _, _ = predict_from_bucket(buckets, keys[i], z[i], d=d, dy=dy)
        losses[i] = local_loss(yhat, y[i])

    mse = float(np.mean(losses))
    return mse, pmax, H, int(len(counts))

def apply_fast_dev_overrides(args: argparse.Namespace):
    if int(args.fast_dev) != 1:
        return
    args.N = min(int(args.N), 2500)
    args.epochs = min(int(args.epochs), 1)
    args.chart_iters = min(int(args.chart_iters), 120)
    args.kmeans_iters = min(int(args.kmeans_iters), 12)
    args.so8_candidates = min(int(args.so8_candidates), 8)
    args.scale_candidates = min(int(args.scale_candidates), 8)
    args.split_rounds = min(int(args.split_rounds), 40)
    args.extra_budget = min(int(args.extra_budget), 32)

def chart_cache_payload(args: argparse.Namespace) -> Dict:
    keys = [
        "mode", "seed", "N", "d", "dy", "branching", "depth", "depth_radius", "noise", "anis_scale",
        "K", "delta_r", "kmeans_iters", "learn_so8", "learn_scale", "scale_mode", "radial_bins",
        "radial_rmax", "radial_update_frac", "radial_l2", "chart_iters", "chart_alpha", "chart_beta",
        "so8_step", "so8_candidates", "scale_step", "scale_candidates", "scale_clip",
        "sector_mode", "phase_dims", "phase4_dims", "field4_dims", "complex_dims", "route_scale_lambda", "memory_coord_mode", "shell_mode", "shell_phase_coupling", "product_shell_control_mode", "product_shell_gate_threshold", "hopf_chi_bins", "hopf_blend_lambda", "hopf_blend_chi_weight", "hopf_blend_shell_weight", "phase_transport_lambda", "phase_field_lambda", "recluster_after_chart", "fast_dev",
        "hybrid_local_k", "hybrid_complex_roots", "hybrid_local_min_k", "hybrid_local_target",
        "hybrid_local_hysteresis", "hybrid_local_converge_lambda",
        "adaptive_min_pair_bins", "adaptive_time_growth", "adaptive_balance", "adaptive_angle_growth",
        "adaptive_shell_growth", "adaptive_shell_balance", "adaptive_converge_lambda",
        "adaptive_converge_target", "adaptive_converge_hysteresis", "adaptive_converge_mode",
    ]
    return {"cache_version": "chart-v11", "args": {k: getattr(args, k) for k in keys}}

def route_cache_payload(args: argparse.Namespace, chart_fp: str, c_fp: str) -> Dict:
    keys = [
        "seed", "delta_r", "sector_mode", "phase_dims", "phase4_dims", "field4_dims", "complex_dims", "route_scale_lambda", "memory_coord_mode", "shell_mode", "shell_phase_coupling", "product_shell_control_mode", "product_shell_gate_threshold", "hopf_chi_bins", "hopf_blend_lambda", "hopf_blend_chi_weight", "hopf_blend_shell_weight", "phase_transport_lambda", "phase_field_lambda", "K",
        "hybrid_local_k", "hybrid_complex_roots", "hybrid_local_min_k", "hybrid_local_target",
        "hybrid_local_hysteresis", "hybrid_local_converge_lambda",
        "time_pressure_lambda", "recluster_after_chart",
        "adaptive_min_pair_bins", "adaptive_time_growth", "adaptive_balance", "adaptive_angle_growth",
        "adaptive_shell_growth", "adaptive_shell_balance", "adaptive_converge_lambda",
        "adaptive_converge_target", "adaptive_converge_hysteresis", "adaptive_converge_mode",
    ]
    return {
        "cache_version": "routes-v11",
        "args": {k: getattr(args, k) for k in keys},
        "chart_fp": chart_fp,
        "c_fp": c_fp,
    }

def chart_fingerprint(chart: Chart) -> str:
    payload = {
        "R": chart.R.tolist(),
        "s_global": (chart.s_global.tolist() if chart.s_global is not None else None),
        "S_radial": (chart.S_radial.tolist() if chart.S_radial is not None else None),
        "scale_mode": chart.scale_mode,
        "radial_rmax": chart.radial_rmax,
        "radial_bins": chart.radial_bins,
    }
    return stable_hash(payload)


# ----------------------------
# Full experiment runner
# ----------------------------

def run_full(args: argparse.Namespace):
    t_total_start = time.perf_counter()
    timings = {
        "dataset": 0.0,
        "chart_opt": 0.0,
        "routing_eval": 0.0,
        "training_route": 0.0,
        "training_update": 0.0,
        "training_ema": 0.0,
        "growth": 0.0,
        "total": 0.0,
    }
    notes: List[str] = []
    artifacts: Dict[str, str] = {}

    apply_fast_dev_overrides(args)

    d = int(args.d)
    dy = int(args.dy)
    phase_dim_i, phase_dim_j = parse_pair_dims(args.phase_dims, "--phase_dims")
    phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l = parse_quad_dims(args.phase4_dims, "--phase4_dims")
    field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l = parse_quad_dims(args.field4_dims, "--field4_dims")
    complex_dim_i, complex_dim_j = parse_pair_dims(args.complex_dims, "--complex_dims")
    ensure_dims_in_range([phase_dim_i, phase_dim_j], d, "--phase_dims")
    ensure_dims_in_range([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], d, "--phase4_dims")
    ensure_dims_in_range([field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l], d, "--field4_dims")
    ensure_dims_in_range([complex_dim_i, complex_dim_j], d, "--complex_dims")
    ensure_distinct_dims([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], "--phase4_dims")
    ensure_distinct_dims([field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l], "--field4_dims")
    if args.sector_mode == "phase4d_hopf_product_phase":
        ensure_disjoint_dims(
            [phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l],
            "--phase4_dims",
            [field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l],
            "--field4_dims",
        )

    if int(args.cache_chart) == 1 or int(args.cache_routes) == 1:
        ensure_dir(args.cache_dir)

    t0 = time.perf_counter()
    data = make_hyperbolic_tree_dataset(
        N=args.N, d=d, dy=dy,
        branching=args.branching, max_depth=args.depth,
        depth_radius=args.depth_radius, noise=args.noise,
        seed=args.seed, mode=args.mode, anis_scale=args.anis_scale,
    )
    timings["dataset"] = time.perf_counter() - t0

    N = data.v_tan.shape[0]
    n_train = int(0.7 * N)
    idx = np.random.permutation(N)
    tr = idx[:n_train]
    te = idx[n_train:]

    v_tr = data.v_tan[tr]
    y_tr = data.y[tr]
    v_te = data.v_tan[te]
    y_te = data.y[te]

    C0 = None
    if args.sector_mode == "kmeans":
        U0 = normalize_rows(v_tr)
        C0 = spherical_kmeans(U0, K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

    chart = Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)

    radial_rmax = float(args.radial_rmax)
    if learned_chart and args.learn_scale == 1 and args.scale_mode == "radial":
        if radial_rmax <= 0:
            radial_rmax = float(np.quantile(safe_norm(v_tr, axis=1, keepdims=False), 0.995))
            radial_rmax = max(radial_rmax, 1e-6)

    opt_res: Optional[ChartOptResult] = None
    chart_cache_hit = False
    chart_cache_file = ""
    if learned_chart and int(args.cache_chart) == 1:
        c_key = stable_hash(chart_cache_payload(args))
        chart_cache_file = os.path.join(args.cache_dir, f"chart_{c_key}.npz")
        artifacts["chart_cache_file"] = chart_cache_file
        if os.path.exists(chart_cache_file):
            cc = np.load(chart_cache_file, allow_pickle=True)
            s_global = cc["s_global"] if bool(cc["has_s_global"]) else None
            S_radial = cc["S_radial"] if bool(cc["has_S_radial"]) else None
            chart = Chart(
                R=cc["R"],
                s_global=s_global,
                S_radial=S_radial,
                scale_mode=str(cc["scale_mode"]),
                radial_rmax=float(cc["radial_rmax"]),
                radial_bins=int(cc["radial_bins"]),
            )
            chart_cache_hit = True
            notes.append("chart cache hit")

    if learned_chart and not chart_cache_hit:
        t_chart = time.perf_counter()
        opt_res = optimize_chart(
            v_train=v_tr,
            y_train=y_tr,
            delta_r=args.delta_r,
            C0=C0,
            learn_so8=args.learn_so8,
            learn_scale=args.learn_scale,
            scale_mode=args.scale_mode,
            radial_bins=args.radial_bins,
            radial_rmax=radial_rmax,
            radial_update_frac=args.radial_update_frac,
            radial_l2=args.radial_l2,
            iters=args.chart_iters,
            so8_step=args.so8_step,
            so8_candidates=args.so8_candidates,
            scale_step=args.scale_step,
            scale_candidates=args.scale_candidates,
            scale_clip=args.scale_clip,
            alpha_overload=args.chart_alpha,
            beta_bucketcount=args.chart_beta,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i,
            phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i,
            phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k,
            phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            field4_dim_i=field4_dim_i,
            field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k,
            field4_dim_l=field4_dim_l,
            K=args.K,
            seed=args.seed + 23,
            early_stop_patience=args.early_stop_patience,
            early_stop_min_delta=args.early_stop_min_delta,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        timings["chart_opt"] = time.perf_counter() - t_chart
        chart = opt_res.chart
        if int(args.cache_chart) == 1:
            if chart_cache_file == "":
                c_key = stable_hash(chart_cache_payload(args))
                chart_cache_file = os.path.join(args.cache_dir, f"chart_{c_key}.npz")
                artifacts["chart_cache_file"] = chart_cache_file
            np.savez_compressed(
                chart_cache_file,
                R=chart.R,
                has_s_global=(chart.s_global is not None),
                s_global=(chart.s_global if chart.s_global is not None else np.zeros((0,), dtype=np.float64)),
                has_S_radial=(chart.S_radial is not None),
                S_radial=(chart.S_radial if chart.S_radial is not None else np.zeros((0, 0), dtype=np.float64)),
                scale_mode=np.array(chart.scale_mode),
                radial_rmax=np.array(chart.radial_rmax),
                radial_bins=np.array(chart.radial_bins),
            )

    reclustered = False
    C_used = C0
    if args.sector_mode == "kmeans" and learned_chart and args.recluster_after_chart == 1:
        z_tr0 = apply_chart(v_tr, chart)
        U_tr_chart = normalize_rows(z_tr0)
        C_used = spherical_kmeans(U_tr_chart, K=args.K, iters=args.kmeans_iters, seed=args.seed + 111)
        reclustered = True

    t_route = time.perf_counter()
    route_cache_file = ""
    route_cache_hit = False
    if int(args.cache_routes) == 1:
        chart_fp = chart_fingerprint(chart)
        c_fp = "none"
        if C_used is not None:
            c_fp = stable_hash({"C": C_used.tolist()})
        r_key = stable_hash(route_cache_payload(args, chart_fp=chart_fp, c_fp=c_fp))
        route_cache_file = os.path.join(args.cache_dir, f"routes_{r_key}.npz")
        artifacts["route_cache_file"] = route_cache_file
        if os.path.exists(route_cache_file):
            rr = np.load(route_cache_file)
            shell_tr_eval = rr["shell_tr_eval"]
            sector_tr_eval = rr["sector_tr_eval"]
            z_tr_eval = rr["z_tr_eval"]
            shell_te = rr["shell_te"]
            sector_te = rr["sector_te"]
            z_te = rr["z_te"]
            route_cache_hit = True
            notes.append("route cache hit")
    if not route_cache_hit:
        shell_tr_eval, sector_tr_eval, _, z_tr_eval = route_addresses(
            v_tr, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            K=args.K,
            time_pressure_lambda=0.0, tau=1.0,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        shell_te, sector_te, _, z_te = route_addresses(
            v_te, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            K=args.K,
            time_pressure_lambda=0.0, tau=1.0,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        if int(args.cache_routes) == 1:
            if route_cache_file == "":
                chart_fp = chart_fingerprint(chart)
                c_fp = stable_hash({"C": (C_used.tolist() if C_used is not None else None)})
                r_key = stable_hash(route_cache_payload(args, chart_fp=chart_fp, c_fp=c_fp))
                route_cache_file = os.path.join(args.cache_dir, f"routes_{r_key}.npz")
                artifacts["route_cache_file"] = route_cache_file
            np.savez_compressed(
                route_cache_file,
                shell_tr_eval=shell_tr_eval,
                sector_tr_eval=sector_tr_eval,
                z_tr_eval=z_tr_eval,
                shell_te=shell_te,
                sector_te=sector_te,
                z_te=z_te,
            )
    timings["routing_eval"] = time.perf_counter() - t_route

    keys_tr_eval = [make_bucket_key(shell_tr_eval[i], sector_tr_eval[i]) for i in range(len(tr))]
    keys_te = [make_bucket_key(shell_te[i], sector_te[i]) for i in range(len(te))]
    train_key_set = set(keys_tr_eval)
    unseen_rate = unseen_key_rate(keys_te, train_key_set)
    shell_vals, shell_counts = np.unique(shell_te, return_counts=True)
    sector_vals, sector_counts = np.unique(sector_te, return_counts=True)
    shell_pmax = float(np.max(shell_counts) / np.sum(shell_counts)) if len(shell_counts) else 0.0
    sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
    shell_entropy = entropy_from_counts(shell_counts) if len(shell_counts) else 0.0
    sector_entropy = entropy_from_counts(sector_counts) if len(sector_counts) else 0.0
    route_z_te = route_coordinate(v_te, chart, sector_mode=args.sector_mode, route_scale_lambda=args.route_scale_lambda)
    alignment = poincare_alignment_diagnostics(
        v_te,
        route_z_te,
        max_pairs=512,
        seed=args.seed + 811,
    )
    poincare_alignment_pairs_used = int(alignment["poincare_alignment_pairs_used"])
    poincare_alignment_radial_mae = float(alignment["poincare_alignment_radial_mae"])
    poincare_alignment_radial_rel_mean = float(alignment["poincare_alignment_radial_rel_mean"])
    poincare_alignment_radial_corr = float(alignment["poincare_alignment_radial_corr"])
    poincare_alignment_pair_mae = float(alignment["poincare_alignment_pair_mae"])
    poincare_alignment_pair_rel_mean = float(alignment["poincare_alignment_pair_rel_mean"])
    poincare_alignment_pair_corr = float(alignment["poincare_alignment_pair_corr"])
    adaptive_k1_mean = 0.0
    adaptive_k2_mean = 0.0
    adaptive_k1_max = 0
    adaptive_k2_max = 0
    adaptive_shell_drive_mean = 0.0
    adaptive_shell_drive_max = 0.0
    adaptive_shell_expand_mean = 0.0
    adaptive_shell_expand_max = 0.0
    adaptive_shell_target_band_mean = 0.0
    adaptive_shell_target_band_max = 0.0
    adaptive_shell_overflow_mean = 0.0
    adaptive_shell_overflow_max = 0.0
    adaptive_shell_ratio_mean = 0.0
    adaptive_shell_ratio_max = 0.0
    adaptive_shell_ratio_pressure_mean = 0.0
    adaptive_shell_ratio_pressure_max = 0.0
    adaptive_shell_ladder_steps_mean = 0.0
    adaptive_shell_ladder_steps_max = 0
    adaptive_shell_converge_mean = 0.0
    adaptive_shell_converge_max = 0.0
    adaptive_shell_mult_mean = 1.0
    adaptive_shell_mult_max = 1.0
    adaptive_chi_mean = 0.0
    adaptive_chi_entropy = 0.0
    adaptive_r_alpha_mean = 0.0
    adaptive_r_alpha_max = 0.0
    adaptive_hopf_shell_capacity_mean = 0.0
    adaptive_hopf_shell_capacity_max = 0.0
    adaptive_hopf_k1_mean = 0.0
    adaptive_hopf_k2_mean = 0.0
    adaptive_hopf_k1_max = 0
    adaptive_hopf_k2_max = 0
    adaptive_hopf_k1_gap_mean = 0.0
    adaptive_hopf_k2_gap_mean = 0.0
    adaptive_chi_bins_used = 0
    adaptive_chi_bin_pmax = 0.0
    adaptive_chi_bin_entropy = 0.0
    adaptive_fib_total_mean = 0.0
    adaptive_fib_total_max = 0
    adaptive_fib_kchi_mean = 0.0
    adaptive_fib_kchi_max = 0
    adaptive_fib_forced_total_mean = 0.0
    adaptive_fib_forced_total_max = 0
    adaptive_fib_band_mean = 0.0
    adaptive_fib_band_max = 0
    adaptive_fib_band_entropy = 0.0
    adaptive_fib_band_states_used = 0
    adaptive_blend_total_mean = 0.0
    adaptive_blend_total_max = 0
    adaptive_blend_score_mean = 0.0
    adaptive_blend_score_max = 0.0
    adaptive_blend_chi_pressure_mean = 0.0
    adaptive_blend_chi_pressure_max = 0.0
    adaptive_blend_shell_pressure_mean = 0.0
    adaptive_blend_shell_pressure_max = 0.0
    phase_transport_coherence = 0.0
    phase_transport_shift_abs_mean = 0.0
    phase_transport_shift_abs_max = 0.0
    phase_transport_connection_abs_mean = 0.0
    phase_transport_field_shift_abs_mean = 0.0
    phase_transport_field_weight_abs_mean = 0.0
    phase_transport_alpha_bins = 0.0
    hopf_sector_groups_used = 0
    hopf_sector_chi_std_mean = 0.0
    hopf_sector_delta_cvar_mean = 0.0
    hopf_sector_alpha_entropy_mean = 0.0
    hopf_sector_alpha_entropy_gap = 0.0
    hybrid_coarse_eval_sectors = 0
    hybrid_local_eval_sectors = 0
    hybrid_local_pmax = 0.0
    hybrid_local_entropy = 0.0
    hybrid_local_drive_mean = 0.0
    hybrid_local_drive_max = 0.0
    hybrid_local_target_band_mean = 0.0
    hybrid_local_target_band_max = 0.0
    hybrid_local_ratio_mean = 0.0
    hybrid_local_ratio_max = 0.0
    hybrid_local_ratio_pressure_mean = 0.0
    hybrid_local_ratio_pressure_max = 0.0
    hybrid_local_converge_mean = 0.0
    hybrid_local_converge_max = 0.0
    hybrid_local_activation_mean = 0.0
    hybrid_local_activation_max = 0.0
    hybrid_local_k_eff_mean = 0.0
    hybrid_local_k_eff_max = 0
    diag_z_te = route_z_te
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        comp = phase4d_adaptive_components(
            z=diag_z_te,
            K=args.K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=args.delta_r,
            tau=1.0,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
        )
        k1_dbg = comp["k1"]
        k2_dbg = comp["k2"]
        chi_ids = np.minimum((comp["chi_u"] * max(1, int(args.hopf_chi_bins))).astype(np.int64), max(max(1, int(args.hopf_chi_bins)) - 1, 0))
        if args.sector_mode in ("phase4d_hopf_base", "phase4d_hopf_base_ball"):
            _, kchi_dbg, kdelta_dbg = assign_sectors_phase4d_hopf_base(
                z=diag_z_te,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
            )
            k1_dbg = kchi_dbg
            k2_dbg = kdelta_dbg
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode in ("phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase"):
            if args.sector_mode == "phase4d_hopf_transport_complex":
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = assign_sectors_phase4d_hopf_transport_complex(
                    z=diag_z_te,
                    K=args.K,
                    dim_i=phase4_dim_i,
                    dim_j=phase4_dim_j,
                    dim_k=phase4_dim_k,
                    dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    phase_transport_lambda=args.phase_transport_lambda,
                    phase_field_lambda=args.phase_field_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            elif args.sector_mode == "phase4d_hopf_product_phase":
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = assign_sectors_phase4d_hopf_product_phase(
                    z=diag_z_te,
                    K=args.K,
                    route_dim_i=phase4_dim_i,
                    route_dim_j=phase4_dim_j,
                    route_dim_k=phase4_dim_k,
                    route_dim_l=phase4_dim_l,
                    field_dim_i=field4_dim_i,
                    field_dim_j=field4_dim_j,
                    field_dim_k=field4_dim_k,
                    field_dim_l=field4_dim_l,
                    phase_transport_lambda=args.phase_transport_lambda,
                    phase_field_lambda=args.phase_field_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            else:
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = assign_sectors_phase4d_hopf_transport(
                    z=diag_z_te,
                    K=args.K,
                    dim_i=phase4_dim_i,
                    dim_j=phase4_dim_j,
                    dim_k=phase4_dim_k,
                    dim_l=phase4_dim_l,
                    phase_transport_lambda=args.phase_transport_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            k1_dbg = kchi_dbg
            k2_dbg = kdelta_dbg
            phase_transport_alpha_bins = float(np.mean(kalpha_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_fib":
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg = assign_sectors_phase4d_hopf_fib(
                z=diag_z_te,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(kchi_dbg * k1_dbg * k2_dbg))
            adaptive_fib_forced_total_max = int(np.max(kchi_dbg * k1_dbg * k2_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_fib_rung":
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg, fib_forced_total_dbg = assign_sectors_phase4d_hopf_fib_rung(
                z=diag_z_te,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
                fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(fib_forced_total_dbg))
            adaptive_fib_forced_total_max = int(np.max(fib_forced_total_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_fib_band":
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg, fib_forced_total_dbg, fib_band_dbg = assign_sectors_phase4d_hopf_fib_band(
                z=z_te,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(fib_forced_total_dbg))
            adaptive_fib_forced_total_max = int(np.max(fib_forced_total_dbg))
            adaptive_fib_band_mean = float(np.mean(fib_band_dbg))
            adaptive_fib_band_max = int(np.max(fib_band_dbg))
            band_counts = np.unique(fib_band_dbg, return_counts=True)[1]
            adaptive_fib_band_entropy = entropy_from_counts(band_counts) if len(band_counts) else 0.0
            adaptive_fib_band_states_used = int(len(band_counts))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_blend":
            _, kchi_dbg, k1_dbg, k2_dbg, blend_total_dbg, blend_score_dbg, chi_pressure_dbg, shell_pressure_dbg = assign_sectors_phase4d_hopf_blend(
                z=diag_z_te,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
                shell_mode=args.shell_mode,
                shell_phase_coupling=args.shell_phase_coupling,
                hopf_chi_bins=args.hopf_chi_bins,
                hopf_blend_lambda=args.hopf_blend_lambda,
                hopf_blend_chi_weight=args.hopf_blend_chi_weight,
                hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            )
            adaptive_blend_total_mean = float(np.mean(blend_total_dbg))
            adaptive_blend_total_max = int(np.max(blend_total_dbg))
            adaptive_blend_score_mean = float(np.mean(blend_score_dbg))
            adaptive_blend_score_max = float(np.max(blend_score_dbg))
            adaptive_blend_chi_pressure_mean = float(np.mean(chi_pressure_dbg))
            adaptive_blend_chi_pressure_max = float(np.max(chi_pressure_dbg))
            adaptive_blend_shell_pressure_mean = float(np.mean(shell_pressure_dbg))
            adaptive_blend_shell_pressure_max = float(np.max(shell_pressure_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        adaptive_k1_mean = float(np.mean(k1_dbg))
        adaptive_k2_mean = float(np.mean(k2_dbg))
        adaptive_k1_max = int(np.max(k1_dbg))
        adaptive_k2_max = int(np.max(k2_dbg))
        adaptive_shell_drive_mean = float(np.mean(comp["shell_drive"]))
        adaptive_shell_drive_max = float(np.max(comp["shell_drive"]))
        adaptive_shell_expand_mean = float(np.mean(comp["shell_expand"]))
        adaptive_shell_expand_max = float(np.max(comp["shell_expand"]))
        adaptive_shell_target_band_mean = float(np.mean(comp["shell_target_band"]))
        adaptive_shell_target_band_max = float(np.max(comp["shell_target_band"]))
        adaptive_shell_overflow_mean = float(np.mean(comp["shell_overflow"]))
        adaptive_shell_overflow_max = float(np.max(comp["shell_overflow"]))
        adaptive_shell_ratio_mean = float(np.mean(comp["shell_ratio"]))
        adaptive_shell_ratio_max = float(np.max(comp["shell_ratio"]))
        adaptive_shell_ratio_pressure_mean = float(np.mean(comp["shell_ratio_pressure"]))
        adaptive_shell_ratio_pressure_max = float(np.max(comp["shell_ratio_pressure"]))
        adaptive_shell_ladder_steps_mean = float(np.mean(comp["shell_ladder_steps"]))
        adaptive_shell_ladder_steps_max = int(np.max(comp["shell_ladder_steps"]))
        adaptive_shell_converge_mean = float(np.mean(comp["shell_converge"]))
        adaptive_shell_converge_max = float(np.max(comp["shell_converge"]))
        adaptive_shell_mult_mean = float(np.mean(comp["shell_multiplier"]))
        adaptive_shell_mult_max = float(np.max(comp["shell_multiplier"]))
        adaptive_chi_mean = float(np.mean(comp["chi"]))
        adaptive_chi_entropy = float(comp["chi_entropy"][0]) if len(comp["chi_entropy"]) else 0.0
        adaptive_r_alpha_mean = float(np.mean(comp["r_alpha"]))
        adaptive_r_alpha_max = float(np.max(comp["r_alpha"]))
        adaptive_hopf_shell_capacity_mean = float(np.mean(comp["hopf_shell_capacity"]))
        adaptive_hopf_shell_capacity_max = float(np.max(comp["hopf_shell_capacity"]))
        adaptive_hopf_k1_mean = float(np.mean(comp["hopf_k1"]))
        adaptive_hopf_k2_mean = float(np.mean(comp["hopf_k2"]))
        adaptive_hopf_k1_max = int(np.max(comp["hopf_k1"]))
        adaptive_hopf_k2_max = int(np.max(comp["hopf_k2"]))
        adaptive_hopf_k1_gap_mean = float(np.mean(comp["hopf_k1_gap"]))
        adaptive_hopf_k2_gap_mean = float(np.mean(comp["hopf_k2_gap"]))
        chi_counts = np.unique(chi_ids, return_counts=True)[1]
        adaptive_chi_bins_used = int(len(chi_counts))
        adaptive_chi_bin_pmax = float(np.max(chi_counts) / np.sum(chi_counts)) if len(chi_counts) else 0.0
        adaptive_chi_bin_entropy = entropy_from_counts(chi_counts) if len(chi_counts) else 0.0
    if args.sector_mode == "phase4d_complex_local":
        _, _, local_sector_te, hybrid_dbg = assign_sectors_phase4d_complex_local(
            z=diag_z_te,
            K=args.K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            delta_r=args.delta_r,
            tau=1.0,
            time_pressure_lambda=0.0,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        coarse_sector_te = hybrid_dbg["coarse_sector"]
        coarse_counts = np.unique(coarse_sector_te, return_counts=True)[1]
        local_counts = np.unique(local_sector_te, return_counts=True)[1]
        hybrid_coarse_eval_sectors = int(len(coarse_counts))
        hybrid_local_eval_sectors = int(len(local_counts))
        hybrid_local_pmax = float(np.max(local_counts) / np.sum(local_counts)) if len(local_counts) else 0.0
        hybrid_local_entropy = entropy_from_counts(local_counts) if len(local_counts) else 0.0
        hybrid_local_drive_mean = float(np.mean(hybrid_dbg["local_drive"]))
        hybrid_local_drive_max = float(np.max(hybrid_dbg["local_drive"]))
        hybrid_local_target_band_mean = float(np.mean(hybrid_dbg["local_target_band"]))
        hybrid_local_target_band_max = float(np.max(hybrid_dbg["local_target_band"]))
        hybrid_local_ratio_mean = float(np.mean(hybrid_dbg["local_ratio"]))
        hybrid_local_ratio_max = float(np.max(hybrid_dbg["local_ratio"]))
        hybrid_local_ratio_pressure_mean = float(np.mean(hybrid_dbg["local_ratio_pressure"]))
        hybrid_local_ratio_pressure_max = float(np.max(hybrid_dbg["local_ratio_pressure"]))
        hybrid_local_converge_mean = float(np.mean(hybrid_dbg["local_converge"]))
        hybrid_local_converge_max = float(np.max(hybrid_dbg["local_converge"]))
        hybrid_local_activation_mean = float(np.mean(hybrid_dbg["local_activation"]))
        hybrid_local_activation_max = float(np.max(hybrid_dbg["local_activation"]))
        hybrid_local_k_eff_mean = float(np.mean(hybrid_dbg["local_k_eff"]))
        hybrid_local_k_eff_max = int(np.max(hybrid_dbg["local_k_eff"]))

    hopf_sector = hopf_sector_routing_diagnostics(
        route_z_te,
        sector_te,
        dim_i=phase4_dim_i,
        dim_j=phase4_dim_j,
        dim_k=phase4_dim_k,
        dim_l=phase4_dim_l,
        alpha_bins=12,
    )
    hopf_sector_groups_used = int(hopf_sector["hopf_sector_groups_used"])
    hopf_sector_chi_std_mean = float(hopf_sector["hopf_sector_chi_std_mean"])
    hopf_sector_delta_cvar_mean = float(hopf_sector["hopf_sector_delta_cvar_mean"])
    hopf_sector_alpha_entropy_mean = float(hopf_sector["hopf_sector_alpha_entropy_mean"])
    hopf_sector_alpha_entropy_gap = float(hopf_sector["hopf_sector_alpha_entropy_gap"])

    train_label_sse = label_coherence_sse(
        v_tr, y_tr, delta_r=args.delta_r, C=C_used, chart=chart,
        sector_mode=args.sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=args.K,
        adaptive_min_pair_bins=args.adaptive_min_pair_bins,
        adaptive_time_growth=args.adaptive_time_growth,
        adaptive_balance=args.adaptive_balance,
        adaptive_angle_growth=args.adaptive_angle_growth,
        adaptive_shell_growth=args.adaptive_shell_growth,
        adaptive_shell_balance=args.adaptive_shell_balance,
        adaptive_converge_lambda=args.adaptive_converge_lambda,
        adaptive_converge_target=args.adaptive_converge_target,
        adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
        adaptive_converge_mode=args.adaptive_converge_mode,
        fib_rung_gate_threshold=args.fib_rung_gate_threshold,
        route_scale_lambda=args.route_scale_lambda,
        memory_coord_mode=args.memory_coord_mode,
        shell_mode=args.shell_mode,
        shell_phase_coupling=args.shell_phase_coupling,
        hopf_chi_bins=args.hopf_chi_bins,
        hopf_blend_lambda=args.hopf_blend_lambda,
        hopf_blend_chi_weight=args.hopf_blend_chi_weight,
        hopf_blend_shell_weight=args.hopf_blend_shell_weight,
        phase_transport_lambda=args.phase_transport_lambda,
        phase_field_lambda=args.phase_field_lambda,
        hybrid_local_k=args.hybrid_local_k,
        hybrid_complex_roots=args.hybrid_complex_roots,
        hybrid_local_min_k=args.hybrid_local_min_k,
        hybrid_local_target=args.hybrid_local_target,
        hybrid_local_hysteresis=args.hybrid_local_hysteresis,
        hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
    )
    test_label_sse = label_coherence_sse(
        v_te, y_te, delta_r=args.delta_r, C=C_used, chart=chart,
        sector_mode=args.sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=args.K,
        adaptive_min_pair_bins=args.adaptive_min_pair_bins,
        adaptive_time_growth=args.adaptive_time_growth,
        adaptive_balance=args.adaptive_balance,
        adaptive_angle_growth=args.adaptive_angle_growth,
        adaptive_shell_growth=args.adaptive_shell_growth,
        adaptive_shell_balance=args.adaptive_shell_balance,
        adaptive_converge_lambda=args.adaptive_converge_lambda,
        adaptive_converge_target=args.adaptive_converge_target,
        adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
        adaptive_converge_mode=args.adaptive_converge_mode,
        fib_rung_gate_threshold=args.fib_rung_gate_threshold,
        route_scale_lambda=args.route_scale_lambda,
        memory_coord_mode=args.memory_coord_mode,
        shell_mode=args.shell_mode,
        shell_phase_coupling=args.shell_phase_coupling,
        hopf_chi_bins=args.hopf_chi_bins,
        hopf_blend_lambda=args.hopf_blend_lambda,
        hopf_blend_chi_weight=args.hopf_blend_chi_weight,
        hopf_blend_shell_weight=args.hopf_blend_shell_weight,
        phase_transport_lambda=args.phase_transport_lambda,
        phase_field_lambda=args.phase_field_lambda,
        hybrid_local_k=args.hybrid_local_k,
        hybrid_complex_roots=args.hybrid_complex_roots,
        hybrid_local_min_k=args.hybrid_local_min_k,
        hybrid_local_target=args.hybrid_local_target,
        hybrid_local_hysteresis=args.hybrid_local_hysteresis,
        hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
    )
    train_label_sse_per = train_label_sse / max(1, len(tr))
    test_label_sse_per = test_label_sse / max(1, len(te))

    buckets = init_buckets(keys_tr_eval, dy=dy, d=d, seed=args.seed + 101)

    t_train = time.perf_counter()
    total_steps = args.epochs * len(tr)
    step_ctr = 0
    for _epoch in range(args.epochs):
        order = np.random.permutation(len(tr))
        for jj in order:
            if total_steps <= 1:
                tau = 1.0
            else:
                tau = float(step_ctr) / float(total_steps - 1)
            t_train_route = time.perf_counter()
            if args.train_route_mode == "final_static":
                key = keys_tr_eval[jj]
                z1 = z_tr_eval[jj]
            else:
                key, z1 = route_one(
                    v_tr[jj],
                    delta_r=args.delta_r,
                    C=C_used,
                    chart=chart,
                    sector_mode=args.sector_mode,
                    phase_dim_i=phase_dim_i,
                    phase_dim_j=phase_dim_j,
                    phase4_dim_i=phase4_dim_i,
                    phase4_dim_j=phase4_dim_j,
                    phase4_dim_k=phase4_dim_k,
                    phase4_dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    field4_dim_i=field4_dim_i,
                    field4_dim_j=field4_dim_j,
                    field4_dim_k=field4_dim_k,
                    field4_dim_l=field4_dim_l,
                    K=args.K,
                    time_pressure_lambda=float(args.time_pressure_lambda),
                    tau=tau,
                    adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                    adaptive_time_growth=args.adaptive_time_growth,
                    adaptive_balance=args.adaptive_balance,
                    adaptive_angle_growth=args.adaptive_angle_growth,
                    adaptive_shell_growth=args.adaptive_shell_growth,
                    adaptive_shell_balance=args.adaptive_shell_balance,
                    adaptive_converge_lambda=args.adaptive_converge_lambda,
                    adaptive_converge_target=args.adaptive_converge_target,
                    adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
                    adaptive_converge_mode=args.adaptive_converge_mode,
                    fib_rung_gate_threshold=args.fib_rung_gate_threshold,
                    route_scale_lambda=args.route_scale_lambda,
                    memory_coord_mode=args.memory_coord_mode,
                    shell_mode=args.shell_mode,
                    shell_phase_coupling=args.shell_phase_coupling,
                    product_shell_control_mode=args.product_shell_control_mode,
                    product_shell_gate_threshold=args.product_shell_gate_threshold,
                    hopf_chi_bins=args.hopf_chi_bins,
                    hopf_blend_lambda=args.hopf_blend_lambda,
                    hopf_blend_chi_weight=args.hopf_blend_chi_weight,
                    hopf_blend_shell_weight=args.hopf_blend_shell_weight,
                    phase_transport_lambda=args.phase_transport_lambda,
                    phase_field_lambda=args.phase_field_lambda,
                    hybrid_local_k=args.hybrid_local_k,
                    hybrid_complex_roots=args.hybrid_complex_roots,
                    hybrid_local_min_k=args.hybrid_local_min_k,
                    hybrid_local_target=args.hybrid_local_target,
                    hybrid_local_hysteresis=args.hybrid_local_hysteresis,
                    hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
                )
            timings["training_route"] += time.perf_counter() - t_train_route
            t_train_update = time.perf_counter()
            _, sj, _ = predict_from_bucket(buckets, key, z1, d=d, dy=dy)
            ema_update(
                get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                z=z1,
                y=y_tr[jj],
                eta_p=args.eta_p,
                eta_m=args.eta_m
            )
            timings["training_update"] += time.perf_counter() - t_train_update
            step_ctr += 1
    timings["training_ema"] = time.perf_counter() - t_train

    mse0, pmax0, H0, nb0 = eval_metrics(buckets, keys_te, z_te, y_te)

    t_growth = time.perf_counter()
    created, accepted = loss_based_splitting(
        buckets=buckets,
        keys=keys_tr_eval,
        z=z_tr_eval,
        y=y_tr,
        max_slots_per_bucket=args.max_slots_per_bucket,
        extra_budget=args.extra_budget,
        eta_p=args.eta_p,
        eta_m=args.eta_m,
        n_split_rounds=args.split_rounds,
        min_gain=args.min_split_gain,
        seed=args.seed + 303,
        d=d,
        dy=dy
    )
    timings["growth"] = time.perf_counter() - t_growth

    mse1, pmax1, H1, nb1 = eval_metrics(buckets, keys_te, z_te, y_te)
    slots_used = int(sum(len(b.slots) for b in buckets.values()))
    timings["total"] = time.perf_counter() - t_total_start

    print("=== RUN SUMMARY ===")
    print(f"mode={args.mode} seed={args.seed} learn_so8={args.learn_so8} learn_scale={args.learn_scale}")
    print(f"sector_mode={args.sector_mode} phase_dims={args.phase_dims} phase4_dims={args.phase4_dims} field4_dims={args.field4_dims} complex_dims={args.complex_dims}")
    print(
        f"time_pressure_lambda={args.time_pressure_lambda} "
        f"train_route_mode={args.train_route_mode} fast_dev={args.fast_dev}"
    )
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        print(
            "adaptive_phase4d="
            f"min_pair_bins={args.adaptive_min_pair_bins} "
            f"time_growth={args.adaptive_time_growth} "
            f"balance={args.adaptive_balance} "
            f"angle_growth={args.adaptive_angle_growth} "
            f"shell_growth={args.adaptive_shell_growth} "
            f"shell_balance={args.adaptive_shell_balance} "
            f"converge_lambda={args.adaptive_converge_lambda} "
            f"converge_target={args.adaptive_converge_target} "
            f"converge_hysteresis={args.adaptive_converge_hysteresis} "
            f"converge_mode={args.adaptive_converge_mode} "
            f"memory_coord_mode={args.memory_coord_mode} "
            f"shell_mode={args.shell_mode}"
        )
    if args.sector_mode == "phase4d_hopf_blend":
        print(
            "hopf_blend="
            f"lambda={args.hopf_blend_lambda} "
            f"chi_weight={args.hopf_blend_chi_weight} "
            f"shell_weight={args.hopf_blend_shell_weight} "
            f"chi_bins={args.hopf_chi_bins}"
        )
    if args.sector_mode in ("phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase"):
        if args.sector_mode == "phase4d_hopf_transport_complex":
            phase_transport = hopf_phase_transport_complex_diagnostics(
                route_z_te,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                complex_dim_i=complex_dim_i,
                complex_dim_j=complex_dim_j,
                phase_transport_lambda=args.phase_transport_lambda,
                phase_field_lambda=args.phase_field_lambda,
            )
            phase_transport_field_shift_abs_mean = float(phase_transport["phase_transport_field_shift_abs_mean"])
            phase_transport_field_weight_abs_mean = float(phase_transport["phase_transport_field_weight_abs_mean"])
        elif args.sector_mode == "phase4d_hopf_product_phase":
            phase_transport = hopf_product_phase_diagnostics(
                route_z_te,
                route_dim_i=phase4_dim_i,
                route_dim_j=phase4_dim_j,
                route_dim_k=phase4_dim_k,
                route_dim_l=phase4_dim_l,
                field_dim_i=field4_dim_i,
                field_dim_j=field4_dim_j,
                field_dim_k=field4_dim_k,
                field_dim_l=field4_dim_l,
                phase_transport_lambda=args.phase_transport_lambda,
                phase_field_lambda=args.phase_field_lambda,
            )
            phase_transport_field_shift_abs_mean = float(phase_transport["phase_transport_field_shift_abs_mean"])
            phase_transport_field_weight_abs_mean = float(phase_transport["phase_transport_field_weight_abs_mean"])
        else:
            phase_transport = hopf_phase_transport_diagnostics(
                route_z_te,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                phase_transport_lambda=args.phase_transport_lambda,
            )
        phase_transport_coherence = float(phase_transport["phase_transport_coherence"])
        phase_transport_shift_abs_mean = float(phase_transport["phase_transport_shift_abs_mean"])
        phase_transport_shift_abs_max = float(phase_transport["phase_transport_shift_abs_max"])
        phase_transport_connection_abs_mean = float(phase_transport["phase_transport_connection_abs_mean"])
        print(
            "phase_transport="
            f"lambda={args.phase_transport_lambda:.6f} "
            f"field_lambda={args.phase_field_lambda:.6f} "
            f"coherence={phase_transport_coherence:.6f} "
            f"shift_abs_mean={phase_transport_shift_abs_mean:.6f} "
            f"shift_abs_max={phase_transport_shift_abs_max:.6f} "
            f"conn_abs_mean={phase_transport_connection_abs_mean:.6f} "
            f"field_shift_abs_mean={phase_transport_field_shift_abs_mean:.6f} "
            f"field_weight_abs_mean={phase_transport_field_weight_abs_mean:.6f} "
            f"alpha_bins={phase_transport_alpha_bins:.6f}"
        )
    if args.sector_mode == "phase4d_complex_local":
        print(
            "hybrid_complex_local="
            f"local_k={args.hybrid_local_k} "
            f"local_min_k={args.hybrid_local_min_k} "
            f"local_target={args.hybrid_local_target} "
            f"local_hysteresis={args.hybrid_local_hysteresis} "
            f"local_converge_lambda={args.hybrid_local_converge_lambda} "
            f"roots={args.hybrid_complex_roots}"
        )
    if args.learn_scale == 1:
        print(f"scale_mode={args.scale_mode}", end="")
        if args.scale_mode == "radial":
            print(f" radial_bins={args.radial_bins} radial_rmax={chart.radial_rmax:.6f} radial_update_frac={args.radial_update_frac} radial_l2={args.radial_l2}")
        else:
            print("")
    print(f"K={args.K} delta_r={args.delta_r} shell_mode={args.shell_mode} extra_budget={args.extra_budget} max_slots_per_bucket={args.max_slots_per_bucket}")
    print(f"epochs={args.epochs} eta_p={args.eta_p} eta_m={args.eta_m}")
    print(f"recluster_after_chart={args.recluster_after_chart} reclustered={reclustered} chart_cache_hit={chart_cache_hit} route_cache_hit={route_cache_hit}")
    print(f"timings_sec={timings}")

    if opt_res is not None:
        print(f"CHART: iters={args.chart_iters} alpha={args.chart_alpha} beta={args.chart_beta}")
        print(f"CHART steps: so8_step={args.so8_step} so8_cand={args.so8_candidates} scale_step={args.scale_step} scale_cand={args.scale_candidates} scale_clip={args.scale_clip}")
        print(f"CHART loss(C0): start={opt_res.loss_hist[0]:.6f} end={opt_res.loss_hist[-1]:.6f} best={min(opt_res.loss_hist):.6f}")
        if args.learn_scale == 1 and args.scale_mode == "global" and opt_res.s_global is not None:
            s = opt_res.s_global
            s_min, s_max = float(np.min(s)), float(np.max(s))
            Ddiag = np.exp(s)
            dmin, dmax = float(np.min(Ddiag)), float(np.max(Ddiag))
            print(f"CHART scales(global): log_s in [{s_min:.3f},{s_max:.3f}]  diag(D) in [{dmin:.3f},{dmax:.3f}]")
        if args.learn_scale == 1 and args.scale_mode == "radial" and opt_res.S_radial is not None:
            S = opt_res.S_radial
            s_min, s_max = float(np.min(S)), float(np.max(S))
            Dmin, Dmax = float(np.min(np.exp(S))), float(np.max(np.exp(S)))
            print(f"CHART scales(radial): log_S in [{s_min:.3f},{s_max:.3f}]  diag(D) in [{Dmin:.3f},{Dmax:.3f}] bins={S.shape[0]}")

    print("--- routing diagnostics (eval keys) ---")
    print(f"test_unseen_rate={unseen_rate:.6f}")
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        print(
            "adaptive_shell="
            f"drive_mean={adaptive_shell_drive_mean:.6f} "
            f"drive_max={adaptive_shell_drive_max:.6f} "
            f"expand_mean={adaptive_shell_expand_mean:.6f} "
            f"expand_max={adaptive_shell_expand_max:.6f} "
            f"target_band_mean={adaptive_shell_target_band_mean:.6f} "
            f"target_band_max={adaptive_shell_target_band_max:.6f} "
            f"overflow_mean={adaptive_shell_overflow_mean:.6f} "
            f"overflow_max={adaptive_shell_overflow_max:.6f} "
            f"ratio_mean={adaptive_shell_ratio_mean:.6f} "
            f"ratio_max={adaptive_shell_ratio_max:.6f} "
            f"ratio_pressure_mean={adaptive_shell_ratio_pressure_mean:.6f} "
            f"ratio_pressure_max={adaptive_shell_ratio_pressure_max:.6f} "
            f"ladder_steps_mean={adaptive_shell_ladder_steps_mean:.6f} "
            f"ladder_steps_max={adaptive_shell_ladder_steps_max} "
            f"converge_mean={adaptive_shell_converge_mean:.6f} "
            f"converge_max={adaptive_shell_converge_max:.6f} "
            f"mult_mean={adaptive_shell_mult_mean:.6f} "
            f"mult_max={adaptive_shell_mult_max:.6f}"
        )
        print(
            "adaptive_hopf="
            f"chi_mean={adaptive_chi_mean:.6f} "
            f"chi_entropy={adaptive_chi_entropy:.6f} "
            f"r_alpha_mean={adaptive_r_alpha_mean:.6f} "
            f"r_alpha_max={adaptive_r_alpha_max:.6f} "
            f"shell_capacity_mean={adaptive_hopf_shell_capacity_mean:.6f} "
            f"shell_capacity_max={adaptive_hopf_shell_capacity_max:.6f} "
            f"hopf_k1_mean={adaptive_hopf_k1_mean:.6f} "
            f"hopf_k2_mean={adaptive_hopf_k2_mean:.6f} "
            f"hopf_k1_max={adaptive_hopf_k1_max} "
            f"hopf_k2_max={adaptive_hopf_k2_max} "
            f"hopf_k1_gap_mean={adaptive_hopf_k1_gap_mean:.6f} "
            f"hopf_k2_gap_mean={adaptive_hopf_k2_gap_mean:.6f} "
            f"chi_bins_used={adaptive_chi_bins_used} "
            f"chi_bin_pmax={adaptive_chi_bin_pmax:.6f} "
            f"chi_bin_entropy={adaptive_chi_bin_entropy:.6f} "
            f"fib_total_mean={adaptive_fib_total_mean:.6f} "
            f"fib_total_max={adaptive_fib_total_max} "
            f"fib_kchi_mean={adaptive_fib_kchi_mean:.6f} "
            f"fib_kchi_max={adaptive_fib_kchi_max} "
            f"fib_forced_total_mean={adaptive_fib_forced_total_mean:.6f} "
            f"fib_forced_total_max={adaptive_fib_forced_total_max} "
            f"fib_band_mean={adaptive_fib_band_mean:.6f} "
            f"fib_band_max={adaptive_fib_band_max} "
            f"fib_band_entropy={adaptive_fib_band_entropy:.6f} "
            f"fib_band_states_used={adaptive_fib_band_states_used} "
            f"blend_total_mean={adaptive_blend_total_mean:.6f} "
            f"blend_total_max={adaptive_blend_total_max} "
            f"blend_score_mean={adaptive_blend_score_mean:.6f} "
            f"blend_score_max={adaptive_blend_score_max:.6f} "
            f"blend_chi_pressure_mean={adaptive_blend_chi_pressure_mean:.6f} "
            f"blend_chi_pressure_max={adaptive_blend_chi_pressure_max:.6f} "
            f"blend_shell_pressure_mean={adaptive_blend_shell_pressure_mean:.6f} "
            f"blend_shell_pressure_max={adaptive_blend_shell_pressure_max:.6f}"
        )
        print(
            "hopf_sector="
            f"groups={hopf_sector_groups_used} "
            f"chi_std_mean={hopf_sector_chi_std_mean:.6f} "
            f"delta_cvar_mean={hopf_sector_delta_cvar_mean:.6f} "
            f"alpha_entropy_mean={hopf_sector_alpha_entropy_mean:.6f} "
            f"alpha_entropy_gap={hopf_sector_alpha_entropy_gap:.6f}"
        )
    if args.sector_mode == "phase4d_complex_local":
        print(
            "hybrid_local="
            f"coarse_eval_sectors={hybrid_coarse_eval_sectors} "
            f"local_eval_sectors={hybrid_local_eval_sectors} "
            f"local_pmax={hybrid_local_pmax:.6f} "
            f"local_entropy={hybrid_local_entropy:.6f} "
            f"drive_mean={hybrid_local_drive_mean:.6f} "
            f"drive_max={hybrid_local_drive_max:.6f} "
            f"target_band_mean={hybrid_local_target_band_mean:.6f} "
            f"target_band_max={hybrid_local_target_band_max:.6f} "
            f"ratio_mean={hybrid_local_ratio_mean:.6f} "
            f"ratio_max={hybrid_local_ratio_max:.6f} "
            f"ratio_pressure_mean={hybrid_local_ratio_pressure_mean:.6f} "
            f"ratio_pressure_max={hybrid_local_ratio_pressure_max:.6f} "
            f"converge_mean={hybrid_local_converge_mean:.6f} "
            f"converge_max={hybrid_local_converge_max:.6f} "
            f"activation_mean={hybrid_local_activation_mean:.6f} "
            f"activation_max={hybrid_local_activation_max:.6f} "
            f"k_eff_mean={hybrid_local_k_eff_mean:.6f} "
            f"k_eff_max={hybrid_local_k_eff_max}"
        )
    print(f"train_label_sse={train_label_sse:.6f} train_label_sse_per={train_label_sse_per:.6f}")
    print(f"test_label_sse={test_label_sse:.6f} test_label_sse_per={test_label_sse_per:.6f}")
    print("--- before growth ---")
    print(f"test_mse={mse0:.6f} pmax={pmax0:.6f} H={H0:.6f} buckets={nb0}")
    print("--- after growth ---")
    print(f"test_mse={mse1:.6f} pmax={pmax1:.6f} H={H1:.6f} buckets={nb1}")
    print(
        "poincare_alignment="
        f"pairs={poincare_alignment_pairs_used} "
        f"radial_mae={poincare_alignment_radial_mae:.6f} "
        f"radial_rel={poincare_alignment_radial_rel_mean:.6f} "
        f"radial_corr={poincare_alignment_radial_corr:.6f} "
        f"pair_mae={poincare_alignment_pair_mae:.6f} "
        f"pair_rel={poincare_alignment_pair_rel_mean:.6f} "
        f"pair_corr={poincare_alignment_pair_corr:.6f}"
    )
    print("--- growth stats ---")
    print(f"slots_used={slots_used} new_slots={created} accepted_splits={accepted} n_buckets_total={len(buckets)}")

    summary = {
        "schema_version": "1.0",
        "parsed": True,
        "args": {k: v for k, v in vars(args).items()},
        "metrics": {
            "test_mse_before": float(mse0),
            "test_mse_after": float(mse1),
            "train_label_sse_per": float(train_label_sse_per),
            "test_label_sse_per": float(test_label_sse_per),
            "buckets": int(nb1),
            "slots_used": int(slots_used),
            "test_unseen_rate": float(unseen_rate),
            "pmax_before": float(pmax0),
            "pmax_after": float(pmax1),
            "entropy_before": float(H0),
            "entropy_after": float(H1),
            "eval_shells": int(len(shell_vals)),
            "eval_sectors": int(len(sector_vals)),
            "shell_pmax": float(shell_pmax),
            "sector_pmax": float(sector_pmax),
            "shell_entropy": float(shell_entropy),
            "sector_entropy": float(sector_entropy),
            "poincare_alignment_pairs_used": int(poincare_alignment_pairs_used),
            "poincare_alignment_radial_mae": float(poincare_alignment_radial_mae),
            "poincare_alignment_radial_rel_mean": float(poincare_alignment_radial_rel_mean),
            "poincare_alignment_radial_corr": float(poincare_alignment_radial_corr),
            "poincare_alignment_pair_mae": float(poincare_alignment_pair_mae),
            "poincare_alignment_pair_rel_mean": float(poincare_alignment_pair_rel_mean),
            "poincare_alignment_pair_corr": float(poincare_alignment_pair_corr),
            "adaptive_k1_mean": float(adaptive_k1_mean),
            "adaptive_k2_mean": float(adaptive_k2_mean),
            "adaptive_k1_max": int(adaptive_k1_max),
            "adaptive_k2_max": int(adaptive_k2_max),
            "adaptive_shell_drive_mean": float(adaptive_shell_drive_mean),
            "adaptive_shell_drive_max": float(adaptive_shell_drive_max),
            "adaptive_shell_expand_mean": float(adaptive_shell_expand_mean),
            "adaptive_shell_expand_max": float(adaptive_shell_expand_max),
            "adaptive_shell_target_band_mean": float(adaptive_shell_target_band_mean),
            "adaptive_shell_target_band_max": float(adaptive_shell_target_band_max),
            "adaptive_shell_overflow_mean": float(adaptive_shell_overflow_mean),
            "adaptive_shell_overflow_max": float(adaptive_shell_overflow_max),
            "adaptive_shell_ratio_mean": float(adaptive_shell_ratio_mean),
            "adaptive_shell_ratio_max": float(adaptive_shell_ratio_max),
            "adaptive_shell_ratio_pressure_mean": float(adaptive_shell_ratio_pressure_mean),
            "adaptive_shell_ratio_pressure_max": float(adaptive_shell_ratio_pressure_max),
            "adaptive_shell_ladder_steps_mean": float(adaptive_shell_ladder_steps_mean),
            "adaptive_shell_ladder_steps_max": int(adaptive_shell_ladder_steps_max),
            "adaptive_shell_converge_mean": float(adaptive_shell_converge_mean),
            "adaptive_shell_converge_max": float(adaptive_shell_converge_max),
            "adaptive_shell_mult_mean": float(adaptive_shell_mult_mean),
            "adaptive_shell_mult_max": float(adaptive_shell_mult_max),
            "adaptive_chi_mean": float(adaptive_chi_mean),
            "adaptive_chi_entropy": float(adaptive_chi_entropy),
            "adaptive_r_alpha_mean": float(adaptive_r_alpha_mean),
            "adaptive_r_alpha_max": float(adaptive_r_alpha_max),
            "adaptive_hopf_shell_capacity_mean": float(adaptive_hopf_shell_capacity_mean),
            "adaptive_hopf_shell_capacity_max": float(adaptive_hopf_shell_capacity_max),
            "adaptive_hopf_k1_mean": float(adaptive_hopf_k1_mean),
            "adaptive_hopf_k2_mean": float(adaptive_hopf_k2_mean),
            "adaptive_hopf_k1_max": int(adaptive_hopf_k1_max),
            "adaptive_hopf_k2_max": int(adaptive_hopf_k2_max),
            "adaptive_hopf_k1_gap_mean": float(adaptive_hopf_k1_gap_mean),
            "adaptive_hopf_k2_gap_mean": float(adaptive_hopf_k2_gap_mean),
            "adaptive_chi_bins_used": int(adaptive_chi_bins_used),
            "adaptive_chi_bin_pmax": float(adaptive_chi_bin_pmax),
            "adaptive_chi_bin_entropy": float(adaptive_chi_bin_entropy),
            "adaptive_fib_total_mean": float(adaptive_fib_total_mean),
            "adaptive_fib_total_max": int(adaptive_fib_total_max),
            "adaptive_fib_kchi_mean": float(adaptive_fib_kchi_mean),
            "adaptive_fib_kchi_max": int(adaptive_fib_kchi_max),
            "adaptive_fib_forced_total_mean": float(adaptive_fib_forced_total_mean),
            "adaptive_fib_forced_total_max": int(adaptive_fib_forced_total_max),
            "adaptive_fib_band_mean": float(adaptive_fib_band_mean),
            "adaptive_fib_band_max": int(adaptive_fib_band_max),
            "adaptive_fib_band_entropy": float(adaptive_fib_band_entropy),
            "adaptive_fib_band_states_used": int(adaptive_fib_band_states_used),
            "adaptive_blend_total_mean": float(adaptive_blend_total_mean),
            "adaptive_blend_total_max": int(adaptive_blend_total_max),
            "adaptive_blend_score_mean": float(adaptive_blend_score_mean),
            "adaptive_blend_score_max": float(adaptive_blend_score_max),
            "adaptive_blend_chi_pressure_mean": float(adaptive_blend_chi_pressure_mean),
            "adaptive_blend_chi_pressure_max": float(adaptive_blend_chi_pressure_max),
            "adaptive_blend_shell_pressure_mean": float(adaptive_blend_shell_pressure_mean),
            "adaptive_blend_shell_pressure_max": float(adaptive_blend_shell_pressure_max),
            "phase_transport_coherence": float(phase_transport_coherence),
            "phase_transport_shift_abs_mean": float(phase_transport_shift_abs_mean),
            "phase_transport_shift_abs_max": float(phase_transport_shift_abs_max),
            "phase_transport_connection_abs_mean": float(phase_transport_connection_abs_mean),
            "phase_transport_field_shift_abs_mean": float(phase_transport_field_shift_abs_mean),
            "phase_transport_field_weight_abs_mean": float(phase_transport_field_weight_abs_mean),
            "phase_transport_alpha_bins": float(phase_transport_alpha_bins),
            "hopf_sector_groups_used": int(hopf_sector_groups_used),
            "hopf_sector_chi_std_mean": float(hopf_sector_chi_std_mean),
            "hopf_sector_delta_cvar_mean": float(hopf_sector_delta_cvar_mean),
            "hopf_sector_alpha_entropy_mean": float(hopf_sector_alpha_entropy_mean),
            "hopf_sector_alpha_entropy_gap": float(hopf_sector_alpha_entropy_gap),
            "hybrid_coarse_eval_sectors": int(hybrid_coarse_eval_sectors),
            "hybrid_local_eval_sectors": int(hybrid_local_eval_sectors),
            "hybrid_local_pmax": float(hybrid_local_pmax),
            "hybrid_local_entropy": float(hybrid_local_entropy),
            "hybrid_local_drive_mean": float(hybrid_local_drive_mean),
            "hybrid_local_drive_max": float(hybrid_local_drive_max),
            "hybrid_local_target_band_mean": float(hybrid_local_target_band_mean),
            "hybrid_local_target_band_max": float(hybrid_local_target_band_max),
            "hybrid_local_ratio_mean": float(hybrid_local_ratio_mean),
            "hybrid_local_ratio_max": float(hybrid_local_ratio_max),
            "hybrid_local_ratio_pressure_mean": float(hybrid_local_ratio_pressure_mean),
            "hybrid_local_ratio_pressure_max": float(hybrid_local_ratio_pressure_max),
            "hybrid_local_converge_mean": float(hybrid_local_converge_mean),
            "hybrid_local_converge_max": float(hybrid_local_converge_max),
            "hybrid_local_activation_mean": float(hybrid_local_activation_mean),
            "hybrid_local_activation_max": float(hybrid_local_activation_max),
            "hybrid_local_k_eff_mean": float(hybrid_local_k_eff_mean),
            "hybrid_local_k_eff_max": int(hybrid_local_k_eff_max),
            "new_slots": int(created),
            "accepted_splits": int(accepted),
            "n_buckets_total": int(len(buckets)),
        },
        "timings_sec": {k: float(v) for k, v in timings.items()},
        "artifacts": {
            "cache_dir": str(args.cache_dir),
            "chart_cache_file": chart_cache_file,
            "route_cache_file": route_cache_file,
            "run_tag": str(args.run_tag),
        },
        "git": maybe_git_info(),
        "notes": notes,
    }
    print("__JSON_SUMMARY__ " + json.dumps(summary, sort_keys=True))


# ----------------------------
# CLI
# ----------------------------

def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", type=str, default="base",
                    choices=["base", "mix", "anis", "anis_mix"])
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--N", type=int, default=9000)
    ap.add_argument("--d", type=int, default=8)
    ap.add_argument("--dy", type=int, default=32)
    ap.add_argument("--branching", type=int, default=3)
    ap.add_argument("--depth", type=int, default=5)
    ap.add_argument("--depth_radius", type=float, default=0.95)
    ap.add_argument("--noise", type=float, default=0.18)
    ap.add_argument("--anis_scale", type=float, default=2.5)

    ap.add_argument("--K", type=int, default=16)
    ap.add_argument("--delta_r", type=float, default=2.0)
    ap.add_argument("--kmeans_iters", type=int, default=25)

    ap.add_argument("--epochs", type=int, default=2)
    ap.add_argument("--eta_p", type=float, default=0.04)
    ap.add_argument("--eta_m", type=float, default=0.08)
    ap.add_argument("--extra_budget", type=int, default=64)
    ap.add_argument("--max_slots_per_bucket", type=int, default=4)
    ap.add_argument("--split_rounds", type=int, default=128)
    ap.add_argument("--min_split_gain", type=float, default=1e-4)

    # Routing mode: sectors
    ap.add_argument("--sector_mode", type=str, default="kmeans", choices=["kmeans", "phase2", "phase4d", "phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local", "complex2"],
                    help="how to compute sector index")
    ap.add_argument("--phase_dims", type=str, default="0,1",
                    help="two dims i,j used for phase2 sectoring, e.g. '0,1'")
    ap.add_argument("--phase4_dims", type=str, default="0,1,2,3",
                    help="four dims i,j,k,l used for phase4d sectoring, e.g. '0,1,2,3'")
    ap.add_argument("--field4_dims", type=str, default="1,3,5,7",
                    help="four dims i,j,k,l used for the coupled field factor in phase4d_hopf_product_phase")
    ap.add_argument("--complex_dims", type=str, default="0,1",
                    help="two dims i,j used for complex2 sectoring")
    ap.add_argument("--hybrid_local_k", type=int, default=4,
                    help="local complex refinement bins per coarse sector for phase4d_complex_local")
    ap.add_argument("--hybrid_complex_roots", type=int, default=4,
                    help="discrete complex root count for local zoom rotation; 4 means i^n")
    ap.add_argument("--hybrid_local_min_k", type=int, default=1,
                    help="minimum local complex bins retained after local convergence")
    ap.add_argument("--hybrid_local_target", type=float, default=0.60,
                    help="local complex drive threshold before hybrid zoom opens")
    ap.add_argument("--hybrid_local_hysteresis", type=float, default=0.05,
                    help="deadband around hybrid_local_target for local zoom activation")
    ap.add_argument("--hybrid_local_converge_lambda", type=float, default=1.0,
                    help="local convergence pressure suppressing hybrid zoom when shell pressure is already high")
    ap.add_argument("--adaptive_min_pair_bins", type=int, default=2,
                    help="minimum per-pair angular bins for phase4d_adaptive")
    ap.add_argument("--adaptive_time_growth", type=float, default=1.0,
                    help="time/radius widening strength for phase4d_adaptive")
    ap.add_argument("--adaptive_balance", type=float, default=1.0,
                    help="phi-based anisotropy strength across the two phase pairs")
    ap.add_argument("--adaptive_angle_growth", type=float, default=0.35,
                    help="golden-angle phase offset strength for phase4d_adaptive")
    ap.add_argument("--adaptive_shell_growth", type=float, default=0.0,
                    help="divergence-aware shell expansion strength for phase4d_adaptive")
    ap.add_argument("--adaptive_shell_balance", type=float, default=0.0,
                    help="extra shell repulsion from phase-pair imbalance in phase4d_adaptive")
    ap.add_argument("--adaptive_converge_lambda", type=float, default=0.0,
                    help="explicit convergence pressure opposing shell divergence in phase4d_adaptive")
    ap.add_argument("--adaptive_converge_target", type=float, default=1.0,
                    help="target shell-expansion logit before convergence engages in phase4d_adaptive")
    ap.add_argument("--adaptive_converge_hysteresis", type=float, default=0.1,
                    help="deadband around converge_target before convergence engages in phase4d_adaptive")
    ap.add_argument("--adaptive_converge_mode", type=str, default="fixed",
                    choices=["fixed", "phi_ratio", "phi_ladder"],
                    help="shell convergence controller for phase4d_adaptive")
    ap.add_argument("--shell_mode", type=str, default="linear", choices=["linear", "phi_log", "phi_phase", "h4_mass", "h4_mass_phi"],
                    help="shell metric: linear, phi-spaced log, or phase-coupled phi shells")
    ap.add_argument("--shell_phase_coupling", type=float, default=0.0,
                    help="signed phase-pressure shift applied to phi-based shells when shell_mode=phi_phase")
    ap.add_argument("--product_shell_control_mode", type=str, default="continuous",
                    choices=["continuous", "gated", "banded"],
                    help="shell-capacity controller for phase4d_hopf_product_phase")
    ap.add_argument("--product_shell_gate_threshold", type=float, default=0.0,
                    help="activation threshold for sparse/banded product shell control")
    ap.add_argument("--fib_rung_gate_threshold", type=float, default=0.0,
                    help="gate threshold for phase4d_hopf_fib_rung; 0 preserves global ungated rung forcing")
    ap.add_argument("--route_scale_lambda", type=float, default=1.0,
                    help="blend factor for bounded-isometry route modes; 0=isometric, 1=full learned scale")
    ap.add_argument("--memory_coord_mode", type=str, default="route_chart", choices=["route_chart", "full_chart"],
                    help="coordinate used for bucket memory and prototype updates")
    ap.add_argument("--hopf_chi_bins", type=int, default=2,
                    help="explicit measure-aware chi bins for phase4d_hopf_chi")
    ap.add_argument("--hopf_blend_lambda", type=float, default=0.8,
                    help="blend strength from pure Hopf shell capacity toward K in phase4d_hopf_blend")
    ap.add_argument("--hopf_blend_chi_weight", type=float, default=1.0,
                    help="relative weight of chi-load pressure in phase4d_hopf_blend")
    ap.add_argument("--hopf_blend_shell_weight", type=float, default=0.5,
                    help="relative weight of shell-load pressure in phase4d_hopf_blend")
    ap.add_argument("--phase_transport_lambda", type=float, default=1.0,
                    help="strength of geometry-induced fiber transport alpha -> alpha + 0.5*lambda*cos(2chi)*delta for phase4d_hopf_transport")
    ap.add_argument("--phase_field_lambda", type=float, default=0.0,
                    help="strength of field-coupled phase transport for phase4d_hopf_transport_complex and phase4d_hopf_product_phase")

    # Time pressure
    ap.add_argument("--time_pressure_lambda", type=float, default=0.0,
                    help="if >0, apply r_eff = r*exp(lambda*tau) during TRAIN routing for shells")
    ap.add_argument("--train_route_mode", type=str, default="dynamic",
                    choices=["dynamic", "final_static"],
                    help="training-time route assignment: reroute every step or reuse final tau=1 eval routes")

    # Chart learning
    ap.add_argument("--learn_so8", type=int, default=0, choices=[0, 1], help="learn rotation R in SO(d)")
    ap.add_argument("--learn_scale", type=int, default=0, choices=[0, 1], help="learn scaling (global or radial)")

    ap.add_argument("--scale_mode", type=str, default="global", choices=["global", "radial"],
                    help="scaling parameterization when learn_scale=1")
    ap.add_argument("--radial_bins", type=int, default=10, help="number of radial bins for scale_mode=radial")
    ap.add_argument("--radial_rmax", type=float, default=0.0,
                    help="max radius for radial binning; <=0 means auto (0.995 quantile on TRAIN)")
    ap.add_argument("--radial_update_frac", type=float, default=0.25,
                    help="fraction of bins to perturb per scale proposal (radial mode)")
    ap.add_argument("--radial_l2", type=float, default=0.0,
                    help="L2 regularization weight on scales (global or radial).")

    ap.add_argument("--chart_iters", type=int, default=400, help="chart optimization iterations")
    ap.add_argument("--chart_alpha", type=float, default=0.01, help="overload penalty weight")
    ap.add_argument("--chart_beta", type=float, default=0.0, help="bucket-count penalty weight")

    ap.add_argument("--so8_step", type=float, default=0.10)
    ap.add_argument("--so8_candidates", type=int, default=30)

    ap.add_argument("--scale_step", type=float, default=0.08)
    ap.add_argument("--scale_candidates", type=int, default=20)
    ap.add_argument("--scale_clip", type=float, default=2.0, help="clip log-scales into [-clip,clip]")

    # Upgrade control
    ap.add_argument("--recluster_after_chart", type=int, default=1, choices=[0, 1],
                    help="if chart learned and sector_mode=kmeans, recompute sector centers C in charted space")
    ap.add_argument("--fast_dev", type=int, default=0, choices=[0, 1],
                    help="if 1, cap major runtime knobs for rapid iteration")
    ap.add_argument("--early_stop_patience", type=int, default=0,
                    help="chart optimizer early-stop patience; 0 disables early stop")
    ap.add_argument("--early_stop_min_delta", type=float, default=0.0,
                    help="minimum chart loss improvement to reset early-stop patience")
    ap.add_argument("--cache_dir", type=str, default="results/cache",
                    help="cache directory for chart and route artifacts")
    ap.add_argument("--cache_chart", type=int, default=1, choices=[0, 1],
                    help="if 1, cache/load learned chart parameters")
    ap.add_argument("--cache_routes", type=int, default=1, choices=[0, 1],
                    help="if 1, cache/load eval routing outputs")
    ap.add_argument("--run_tag", type=str, default="",
                    help="optional run label included in summary artifacts")

    return ap.parse_args()

if __name__ == "__main__":
    args = parse_args()
    run_full(args)
