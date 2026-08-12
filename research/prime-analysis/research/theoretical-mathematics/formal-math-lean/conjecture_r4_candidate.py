#!/usr/bin/env python3
"""Conjecture Candidate #1: R4 spinning-top geometry for primes.

Builds top-like observables from prime path in R4 embedding:
- precession rate
- nutation rate
- wobble (phase jerk)
Then checks zeta-guardrail spectral alignment and stability across N.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import (
    dft_powers,
    embed_4d,
    load_zeta_zeros_file,
    parse_moduli,
    zeta_permutation_control,
)

ALL_CHANNELS = [
    "precession",
    "nutation",
    "wobble",
    "wobble_banded",
    "rho_drift",
    "rho_curvature",
    "square_local_wobble",
    "phase_rho_coupling",
]
CACHE_VERSION = "conjecture_r4_v2"

DEFAULT_CHANNELS = [
    "precession",
    "nutation",
    "wobble",
    "wobble_banded",
    "rho_drift",
    "rho_curvature",
]


def sieve_primes(n_max: int) -> List[int]:
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    return [i for i in range(2, n_max + 1) if is_prime[i]]


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2.0 * math.pi
    while d < -math.pi:
        d += 2.0 * math.pi
    return d


def phase_series(primes: Sequence[int], moduli: Sequence[int], radius_power: float) -> Dict[str, List[float]]:
    phi1 = []
    phi2 = []
    rho = []

    for p in primes:
        x, y, z, t = embed_4d(p, moduli, radius_power)
        phi1.append(math.atan2(y, x))
        phi2.append(math.atan2(t, z))
        rho.append(math.sqrt(x * x + y * y + z * z + t * t))

    precession = []
    nutation = []
    for i in range(len(primes) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        precession.append(0.5 * (d1 + d2))
        nutation.append(0.5 * (d1 - d2))

    wobble = [nutation[i + 1] - nutation[i] for i in range(len(nutation) - 1)]
    rho_drift = [rho[i + 1] - rho[i] for i in range(len(rho) - 1)]
    rho_curvature = [rho_drift[i + 1] - rho_drift[i] for i in range(len(rho_drift) - 1)]
    square_local_wobble = []
    for i in range(len(wobble)):
        n_ref = primes[i + 1]
        s = int(math.isqrt(n_ref))
        lower = s * s
        upper = (s + 1) * (s + 1)
        span = max(1, upper - lower)
        u = (n_ref - lower) / span
        boundary = min(u, 1.0 - u)
        gate = math.exp(-((boundary / 0.2) ** 2))
        square_local_wobble.append(wobble[i] * gate)
    phase_rho_coupling = [precession[i] * rho_drift[i] for i in range(min(len(precession), len(rho_drift)))]
    return {
        "phi1": phi1,
        "phi2": phi2,
        "rho": rho,
        "precession": precession,
        "nutation": nutation,
        "wobble": wobble,
        "rho_drift": rho_drift,
        "rho_curvature": rho_curvature,
        "square_local_wobble": square_local_wobble,
        "phase_rho_coupling": phase_rho_coupling,
    }


def phase_series_wheel_base(primes: Sequence[int], wheel_base: int, radius_power: float) -> Dict[str, List[float]]:
    moduli_stub = [wheel_base, wheel_base]
    phi1 = []
    phi2 = []
    rho = []
    for p in primes:
        r = p**radius_power
        theta = (2.0 * math.pi * (p % wheel_base)) / wheel_base
        # Single-base 4D embedding via phase harmonics of the same wheel modulus.
        x = r * math.cos(theta)
        y = r * math.sin(theta)
        z = r * math.cos(3.0 * theta)
        t = r * math.sin(3.0 * theta)
        phi1.append(math.atan2(y, x))
        phi2.append(math.atan2(t, z))
        rho.append(math.sqrt(x * x + y * y + z * z + t * t))

    precession = []
    nutation = []
    for i in range(len(primes) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        precession.append(0.5 * (d1 + d2))
        nutation.append(0.5 * (d1 - d2))

    wobble = [nutation[i + 1] - nutation[i] for i in range(len(nutation) - 1)]
    rho_drift = [rho[i + 1] - rho[i] for i in range(len(rho) - 1)]
    rho_curvature = [rho_drift[i + 1] - rho_drift[i] for i in range(len(rho_drift) - 1)]
    square_local_wobble = []
    for i in range(len(wobble)):
        n_ref = primes[i + 1]
        s = int(math.isqrt(n_ref))
        lower = s * s
        upper = (s + 1) * (s + 1)
        span = max(1, upper - lower)
        u = (n_ref - lower) / span
        boundary = min(u, 1.0 - u)
        gate = math.exp(-((boundary / 0.2) ** 2))
        square_local_wobble.append(wobble[i] * gate)
    phase_rho_coupling = [precession[i] * rho_drift[i] for i in range(min(len(precession), len(rho_drift)))]
    return {
        "phi1": phi1,
        "phi2": phi2,
        "rho": rho,
        "precession": precession,
        "nutation": nutation,
        "wobble": wobble,
        "rho_drift": rho_drift,
        "rho_curvature": rho_curvature,
        "square_local_wobble": square_local_wobble,
        "phase_rho_coupling": phase_rho_coupling,
        "moduli_stub": moduli_stub,
    }


def zeta_spectral_score(series: Sequence[float], zeros: Sequence[float]) -> float:
    if len(series) < 64 or not zeros:
        return 0.0
    m = max(zeros)
    targets = [0.5 + 0.45 * (z / m) * len(series) for z in zeros]
    tp = dft_powers(series, targets)
    base_freqs = [1 + i for i in range(min(64, len(series) // 2))]
    bp = dft_powers(series, base_freqs)
    b = sum(bp) / max(1, len(bp))
    return (sum(tp) / max(1, len(tp))) / max(1e-12, b)


def zeta_spectral_score_banded(
    series: Sequence[float],
    zeros: Sequence[float],
    freq_frac_min: float,
    freq_frac_max: float,
) -> float:
    if len(series) < 64 or not zeros:
        return 0.0
    n = len(series)
    m = max(zeros)
    raw_targets = [0.5 + 0.45 * (z / m) * n for z in zeros]
    lo = max(0.0, freq_frac_min) * n
    hi = min(0.5, max(freq_frac_min, freq_frac_max)) * n
    targets = [f for f in raw_targets if lo <= f <= hi]
    if not targets:
        return 0.0

    tp = dft_powers(series, targets)
    base = [1 + i for i in range(min(64, n // 2))]
    base = [f for f in base if lo <= f <= hi]
    if not base:
        base = [1 + i for i in range(min(64, n // 2))]
    bp = dft_powers(series, base)
    b = sum(bp) / max(1, len(bp))
    return (sum(tp) / max(1, len(tp))) / max(1e-12, b)


def banded_permutation_control(
    series: Sequence[float],
    zeros: Sequence[float],
    trials: int,
    seed: int,
    freq_frac_min: float,
    freq_frac_max: float,
) -> Dict[str, float]:
    if trials <= 0:
        return {"p_value_ge": 1.0, "z_score": 0.0}
    import random

    rng = random.Random(seed)
    obs = zeta_spectral_score_banded(series, zeros, freq_frac_min, freq_frac_max)
    vals = []
    work = list(series)
    for _ in range(trials):
        rng.shuffle(work)
        vals.append(zeta_spectral_score_banded(work, zeros, freq_frac_min, freq_frac_max))
    mean = sum(vals) / max(1, len(vals))
    var = sum((v - mean) ** 2 for v in vals) / max(1, len(vals))
    std = math.sqrt(var)
    ge = sum(1 for v in vals if v >= obs)
    p = (ge + 1.0) / (len(vals) + 1.0)
    z = (obs - mean) / max(1e-12, std)
    vals_sorted = sorted(vals)
    le = sum(1 for v in vals if v <= obs)
    percentile = le / max(1, len(vals))
    signed_effect = 2.0 * percentile - 1.0  # in [-1,1]
    return {
        "p_value_ge": p,
        "z_score": z,
        "observed": obs,
        "null_mean": mean,
        "null_std": std,
        "percentile": percentile,
        "signed_effect": signed_effect,
    }


def fit_limit(ns: Sequence[int], ys: Sequence[float]) -> Dict[str, float]:
    # y(N)=L + C*N^{-a}, coarse grid for a
    best = None
    for k in range(4, 61):
        a = k / 20.0
        xs = [n ** (-a) for n in ns]
        s1 = len(xs)
        sx = sum(xs)
        sxx = sum(x * x for x in xs)
        sy = sum(ys)
        sxy = sum(x * y for x, y in zip(xs, ys))
        det = s1 * sxx - sx * sx
        if abs(det) < 1e-12:
            continue
        L = (sy * sxx - sx * sxy) / det
        C = (s1 * sxy - sx * sy) / det
        pred = [L + C * x for x in xs]
        mse = sum((a1 - a2) ** 2 for a1, a2 in zip(ys, pred)) / len(ys)
        if best is None or mse < best["mse"]:
            best = {"alpha": a, "L": L, "C": C, "mse": mse}
    return best if best else {"alpha": 1.0, "L": 0.0, "C": 0.0, "mse": 0.0}


def median(values: Sequence[float]) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    n = len(v)
    m = n // 2
    if n % 2 == 1:
        return v[m]
    return 0.5 * (v[m - 1] + v[m])


def quantile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    idx = int(max(0, min(len(v) - 1, round(q * (len(v) - 1)))))
    return v[idx]


def winsorize(values: Sequence[float], q_low: float = 0.1, q_high: float = 0.9) -> List[float]:
    if not values:
        return []
    lo = quantile(values, q_low)
    hi = quantile(values, q_high)
    return [min(hi, max(lo, x)) for x in values]


def clip(x: float, lo: float = -25.0, hi: float = 25.0) -> float:
    return min(hi, max(lo, x))


def load_cache(path: str) -> Dict[str, object]:
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def save_cache(path: str, cache: Dict[str, object]) -> None:
    if not path:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(cache, f)


def zeros_signature(zeros: Sequence[float]) -> str:
    text = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def parse_channels(raw: str) -> List[str]:
    if not raw.strip():
        return list(DEFAULT_CHANNELS)
    seen = set()
    out = []
    for token in raw.split(","):
        name = token.strip()
        if not name:
            continue
        if name not in ALL_CHANNELS:
            raise ValueError(f"unknown channel: {name}")
        if name in seen:
            continue
        seen.add(name)
        out.append(name)
    if not out:
        return list(DEFAULT_CHANNELS)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="R4 spinning-top conjecture candidate")
    ap.add_argument("--n-values", type=str, default="80000,100000,120000,150000,200000")
    ap.add_argument("--moduli", type=str, default="5,7,11,13,19")
    ap.add_argument("--radius-power", type=float, default=1.1)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--zeta-perm-trials", type=int, default=20)
    ap.add_argument("--freq-frac-min", type=float, default=0.05)
    ap.add_argument("--freq-frac-max", type=float, default=0.35)
    ap.add_argument("--wheel-base", type=int, default=0, help="Use single wheel modulus with harmonic 4D lift.")
    ap.add_argument("--cache-file", type=str, default="research/output/cache/conjecture_r4_cache.json")
    ap.add_argument(
        "--include-channels",
        type=str,
        default="",
        help="Comma-separated channel subset. Default uses all channels.",
    )
    ap.add_argument("--output", type=str, default="research/output/conjecture_r4_candidate.json")
    args = ap.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    moduli = parse_moduli(args.moduli)
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = zeros_signature(zeros)
    channels = parse_channels(args.include_channels)
    cache = load_cache(args.cache_file)

    rows = []
    for i, n in enumerate(n_values):
        key = cache_key(
            {
                "v": CACHE_VERSION,
                "n": n,
                "moduli": moduli,
                "wheel_base": args.wheel_base,
                "radius_power": args.radius_power,
                "zeta_perm_trials": args.zeta_perm_trials,
                "freq_frac_min": args.freq_frac_min,
                "freq_frac_max": args.freq_frac_max,
                "zsig": zsig,
            }
        )
        if key in cache:
            rows.append(cache[key])
            continue

        primes = sieve_primes(n)
        s = (
            phase_series_wheel_base(primes, args.wheel_base, args.radius_power)
            if args.wheel_base > 1
            else phase_series(primes, moduli, args.radius_power)
        )

        pre = s["precession"]
        nut = s["nutation"]
        wob = s["wobble"]
        rho_drift = s["rho_drift"]
        rho_curvature = s["rho_curvature"]
        square_local_wobble = s["square_local_wobble"]
        phase_rho_coupling = s["phase_rho_coupling"]

        pre_mean = sum(pre) / max(1, len(pre))
        nut_mean = sum(nut) / max(1, len(nut))
        wob_std = math.sqrt(sum((x - (sum(wob) / max(1, len(wob)))) ** 2 for x in wob) / max(1, len(wob))) if wob else 0.0

        pre_zeta = zeta_spectral_score(pre, zeros)
        nut_zeta = zeta_spectral_score(nut, zeros)
        wob_zeta = zeta_spectral_score(wob, zeros)
        rho_drift_zeta = zeta_spectral_score(rho_drift, zeros)
        rho_curv_zeta = zeta_spectral_score(rho_curvature, zeros)
        square_local_wobble_zeta = zeta_spectral_score(square_local_wobble, zeros)
        phase_rho_zeta = zeta_spectral_score(phase_rho_coupling, zeros)
        wob_zeta_banded = zeta_spectral_score_banded(wob, zeros, args.freq_frac_min, args.freq_frac_max)

        pre_perm = zeta_permutation_control(pre, args.zeta_perm_trials, 20260216 + i * 10, zeros_imag=zeros)
        nut_perm = zeta_permutation_control(nut, args.zeta_perm_trials, 20260217 + i * 10, zeros_imag=zeros)
        wob_perm = zeta_permutation_control(wob, args.zeta_perm_trials, 20260218 + i * 10, zeros_imag=zeros)
        rho_drift_perm = zeta_permutation_control(rho_drift, args.zeta_perm_trials, 20260219 + i * 10, zeros_imag=zeros)
        rho_curv_perm = zeta_permutation_control(
            rho_curvature, args.zeta_perm_trials, 20260220 + i * 10, zeros_imag=zeros
        )
        square_local_wobble_perm = zeta_permutation_control(
            square_local_wobble, args.zeta_perm_trials, 202602205 + i * 10, zeros_imag=zeros
        )
        phase_rho_perm = zeta_permutation_control(
            phase_rho_coupling, args.zeta_perm_trials, 20260221 + i * 10, zeros_imag=zeros
        )
        wob_band_perm = banded_permutation_control(
            wob,
            zeros,
            args.zeta_perm_trials,
            20260300 + i * 10,
            args.freq_frac_min,
            args.freq_frac_max,
        )

        row = {
            "n_max": n,
            "prime_count": len(primes),
            "precession_mean": pre_mean,
            "nutation_mean": nut_mean,
            "wobble_std": wob_std,
            "zeta_scores": {
                "precession": pre_zeta,
                "nutation": nut_zeta,
                "wobble": wob_zeta,
                "wobble_banded": wob_zeta_banded,
                "rho_drift": rho_drift_zeta,
                "rho_curvature": rho_curv_zeta,
                "square_local_wobble": square_local_wobble_zeta,
                "phase_rho_coupling": phase_rho_zeta,
            },
            "zeta_p_values": {
                "precession": pre_perm.get("p_value_ge", 1.0),
                "nutation": nut_perm.get("p_value_ge", 1.0),
                "wobble": wob_perm.get("p_value_ge", 1.0),
                "wobble_banded": wob_band_perm.get("p_value_ge", 1.0),
                "rho_drift": rho_drift_perm.get("p_value_ge", 1.0),
                "rho_curvature": rho_curv_perm.get("p_value_ge", 1.0),
                "square_local_wobble": square_local_wobble_perm.get("p_value_ge", 1.0),
                "phase_rho_coupling": phase_rho_perm.get("p_value_ge", 1.0),
            },
            "zeta_effect_scores": {
                "precession": 1.0 - 2.0 * pre_perm.get("p_value_ge", 1.0),
                "nutation": 1.0 - 2.0 * nut_perm.get("p_value_ge", 1.0),
                "wobble": 1.0 - 2.0 * wob_perm.get("p_value_ge", 1.0),
                "wobble_banded": wob_band_perm.get("signed_effect", 0.0),
                "rho_drift": 1.0 - 2.0 * rho_drift_perm.get("p_value_ge", 1.0),
                "rho_curvature": 1.0 - 2.0 * rho_curv_perm.get("p_value_ge", 1.0),
                "square_local_wobble": 1.0 - 2.0 * square_local_wobble_perm.get("p_value_ge", 1.0),
                "phase_rho_coupling": 1.0 - 2.0 * phase_rho_perm.get("p_value_ge", 1.0),
            },
            "zeta_z_scores": {
                "precession": clip(pre_perm.get("z_score", 0.0)),
                "nutation": clip(nut_perm.get("z_score", 0.0)),
                "wobble": clip(wob_perm.get("z_score", 0.0)),
                "wobble_banded": clip(wob_band_perm.get("z_score", 0.0)),
                "rho_drift": clip(rho_drift_perm.get("z_score", 0.0)),
                "rho_curvature": clip(rho_curv_perm.get("z_score", 0.0)),
                "square_local_wobble": clip(square_local_wobble_perm.get("z_score", 0.0)),
                "phase_rho_coupling": clip(phase_rho_perm.get("z_score", 0.0)),
            },
        }
        cache[key] = row
        rows.append(row)

    rows.sort(key=lambda r: r["n_max"])

    ns = [r["n_max"] for r in rows]
    fits = {
        "precession_mean": fit_limit(ns, [r["precession_mean"] for r in rows]),
        "nutation_mean": fit_limit(ns, [r["nutation_mean"] for r in rows]),
        "wobble_std": fit_limit(ns, [r["wobble_std"] for r in rows]),
        "zeta_precession": fit_limit(ns, [r["zeta_scores"]["precession"] for r in rows]),
        "zeta_nutation": fit_limit(ns, [r["zeta_scores"]["nutation"] for r in rows]),
        "zeta_wobble": fit_limit(ns, [r["zeta_scores"]["wobble"] for r in rows]),
        "zeta_wobble_banded": fit_limit(ns, [r["zeta_scores"]["wobble_banded"] for r in rows]),
        "zeta_rho_drift": fit_limit(ns, [r["zeta_scores"]["rho_drift"] for r in rows]),
        "zeta_rho_curvature": fit_limit(ns, [r["zeta_scores"]["rho_curvature"] for r in rows]),
        "zeta_square_local_wobble": fit_limit(ns, [r["zeta_scores"]["square_local_wobble"] for r in rows]),
        "zeta_phase_rho_coupling": fit_limit(ns, [r["zeta_scores"]["phase_rho_coupling"] for r in rows]),
    }

    # Conjecture score emphasizes zeta-guardrail on top-like observables.
    mean_p = sum(sum(r["zeta_p_values"][c] for c in channels) / len(channels) for r in rows) / len(rows)
    mean_z = sum(sum(r["zeta_z_scores"][c] for c in channels) / len(channels) for r in rows) / len(rows)

    mean_effect = sum(
        sum(r["zeta_effect_scores"][c] for c in channels) / len(channels)
        for r in rows
    ) / len(rows)

    per_n_guardrail = [
        {
            "n_max": r["n_max"],
            "p_mean": sum(r["zeta_p_values"][c] for c in channels) / len(channels),
            "z_mean": sum(r["zeta_z_scores"][c] for c in channels) / len(channels),
            "effect_mean": sum(r["zeta_effect_scores"][c] for c in channels) / len(channels),
        }
        for r in rows
    ]

    z_raw = [x["z_mean"] for x in per_n_guardrail]
    p_raw = [x["p_mean"] for x in per_n_guardrail]
    e_raw = [x["effect_mean"] for x in per_n_guardrail]
    z_w = winsorize(z_raw, 0.1, 0.9)
    p_w = winsorize(p_raw, 0.1, 0.9)
    e_w = winsorize(e_raw, 0.1, 0.9)
    med_z = median(z_w)
    med_p = median(p_w)
    med_e = median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good_frac = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    stability_gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good_frac >= 0.5 and e_var <= 0.25

    conjecture = {
        "title": "R4 Spinning-Top Zeta-Guardrail Conjecture (Candidate 1)",
        "statement": (
            "In a zeta-optimized R4 prime embedding, precession/nutation/wobble observables along the prime path "
            "exhibit stable non-random spectral coupling to zeta-zero frequencies across growing N."
        ),
        "guardrail_mean_p": mean_p,
        "guardrail_mean_z": mean_z,
        "guardrail_mean_effect": mean_effect,
        "guardrail_median_p_winsor": med_p,
        "guardrail_median_z_winsor": med_z,
        "guardrail_median_effect_winsor": med_e,
        "guardrail_z_var_winsor": z_var,
        "guardrail_effect_var_winsor": e_var,
        "guardrail_sign_consistency": sign_consistency,
        "guardrail_p_good_fraction": p_good_frac,
        "stability_gate_pass": stability_gate,
        "support_score": (1.0 - mean_p) * max(0.0, mean_z),
        "support_score_robust": (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var)),
        "support_score_rank": (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var)),
    }

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_values": n_values,
            "moduli": moduli,
            "wheel_base": args.wheel_base,
            "radius_power": args.radius_power,
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": len(zeros),
            "zeta_perm_trials": args.zeta_perm_trials,
            "freq_frac_min": args.freq_frac_min,
            "freq_frac_max": args.freq_frac_max,
            "channels": channels,
        },
        "series": rows,
        "per_n_guardrail": per_n_guardrail,
        "fits": fits,
        "conjecture": conjecture,
    }
    save_cache(args.cache_file, cache)

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Conjecture Candidate #1 (R4 Spinning-Top)\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Statement: {conjecture['statement']}\n\n")
        f.write(f"Support score (raw): {conjecture['support_score']:.6f}\n")
        f.write(f"Support score (robust): {conjecture['support_score_robust']:.6f}\n")
        f.write(f"Support score (rank-based): {conjecture['support_score_rank']:.6f}\n")
        f.write(f"Guardrail mean p: {conjecture['guardrail_mean_p']:.6f}\n")
        f.write(f"Guardrail mean z: {conjecture['guardrail_mean_z']:.6f}\n\n")
        f.write(f"Guardrail mean effect: {conjecture['guardrail_mean_effect']:.6f}\n")
        f.write(f"Guardrail median p (winsor): {conjecture['guardrail_median_p_winsor']:.6f}\n")
        f.write(f"Guardrail median z (winsor): {conjecture['guardrail_median_z_winsor']:.6f}\n")
        f.write(f"Guardrail median effect (winsor): {conjecture['guardrail_median_effect_winsor']:.6f}\n")
        f.write(f"Guardrail z variance (winsor): {conjecture['guardrail_z_var_winsor']:.6f}\n")
        f.write(f"Guardrail effect variance (winsor): {conjecture['guardrail_effect_var_winsor']:.6f}\n")
        f.write(f"Sign consistency: {conjecture['guardrail_sign_consistency']:.3f}\n")
        f.write(f"Good-p fraction (<=0.2): {conjecture['guardrail_p_good_fraction']:.3f}\n")
        f.write(f"Stability gate pass: {conjecture['stability_gate_pass']}\n\n")
        f.write("## Per-N summary\n\n")
        for r in rows:
            eff = sum(r["zeta_effect_scores"][c] for c in channels) / len(channels)
            f.write(
                f"- N={r['n_max']} pre_p={r['zeta_p_values']['precession']:.6f} "
                f"nut_p={r['zeta_p_values']['nutation']:.6f} wob_p={r['zeta_p_values']['wobble']:.6f} "
                f"wobB_p={r['zeta_p_values']['wobble_banded']:.6f} "
                f"rhoD_p={r['zeta_p_values']['rho_drift']:.6f} rhoC_p={r['zeta_p_values']['rho_curvature']:.6f} "
                f"sqW_p={r['zeta_p_values']['square_local_wobble']:.6f} "
                f"rhoPhi_p={r['zeta_p_values']['phase_rho_coupling']:.6f} "
                f"pre_z={r['zeta_z_scores']['precession']:.3f} nut_z={r['zeta_z_scores']['nutation']:.3f} "
                f"wob_z={r['zeta_z_scores']['wobble']:.3f} wobB_z={r['zeta_z_scores']['wobble_banded']:.3f} "
                f"rhoD_z={r['zeta_z_scores']['rho_drift']:.3f} rhoC_z={r['zeta_z_scores']['rho_curvature']:.3f} "
                f"sqW_z={r['zeta_z_scores']['square_local_wobble']:.3f} "
                f"rhoPhi_z={r['zeta_z_scores']['phase_rho_coupling']:.3f} eff={eff:.3f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("support_score_raw:", round(conjecture["support_score"], 6))
    print("support_score_robust:", round(conjecture["support_score_robust"], 6))
    print("support_score_rank:", round(conjecture["support_score_rank"], 6))
    print("stability_gate_pass:", conjecture["stability_gate_pass"])


if __name__ == "__main__":
    main()
