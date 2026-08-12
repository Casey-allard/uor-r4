#!/usr/bin/env python3
"""Lie-phase + spherical-harmonic probe for prime sequences."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence
from itertools import combinations

from prime_geometry_loop import load_zeta_zeros_file, zeta_permutation_control


CHANNELS = [
    "Y11c",
    "Y11s",
    "Y20",
    "Y22c",
    "Y22s",
    "transport",
    "curvature",
    "su2_trace",
    "su2_comm",
]


def sieve_primes(n_max: int) -> List[int]:
    is_prime = [False, False] + [True] * (n_max - 1)
    for p in range(2, int(n_max**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n_max + 1 : p] = [False] * (((n_max - p * p) // p) + 1)
    return [i for i in range(2, n_max + 1) if is_prime[i]]


def gcd(a: int, b: int) -> int:
    while b:
        a, b = b, a % b
    return a


def totatives(base: int) -> List[int]:
    return [r for r in range(1, base) if gcd(r, base) == 1]


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2.0 * math.pi
    while d < -math.pi:
        d += 2.0 * math.pi
    return d


def harmonics_from_angles(theta: float, chi: float) -> Dict[str, float]:
    # Real spherical harmonics low orders (scaled conventional forms).
    sin_chi = math.sin(chi)
    cos_chi = math.cos(chi)
    return {
        "Y11c": math.sqrt(3.0 / (4.0 * math.pi)) * sin_chi * math.cos(theta),
        "Y11s": math.sqrt(3.0 / (4.0 * math.pi)) * sin_chi * math.sin(theta),
        "Y20": math.sqrt(5.0 / (16.0 * math.pi)) * (3.0 * cos_chi * cos_chi - 1.0),
        "Y22c": math.sqrt(15.0 / (16.0 * math.pi)) * sin_chi * sin_chi * math.cos(2.0 * theta),
        "Y22s": math.sqrt(15.0 / (16.0 * math.pi)) * sin_chi * sin_chi * math.sin(2.0 * theta),
    }


def su2_step(theta: float, chi: float, alpha: float):
    # U = cos(a/2) I + i sin(a/2) (n . sigma), n from local phase direction.
    nx = math.sin(chi) * math.cos(theta)
    ny = math.sin(chi) * math.sin(theta)
    nz = math.cos(chi)
    c = math.cos(0.5 * alpha)
    s = math.sin(0.5 * alpha)
    return (
        (c + 1j * s * nz, 1j * s * (nx - 1j * ny)),
        (1j * s * (nx + 1j * ny), c - 1j * s * nz),
    )


def mmul(a, b):
    return (
        (a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]),
        (a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]),
    )


def msub(a, b):
    return (
        (a[0][0] - b[0][0], a[0][1] - b[0][1]),
        (a[1][0] - b[1][0], a[1][1] - b[1][1]),
    )


def fnorm(a) -> float:
    return math.sqrt(
        (a[0][0].real * a[0][0].real + a[0][0].imag * a[0][0].imag)
        + (a[0][1].real * a[0][1].real + a[0][1].imag * a[0][1].imag)
        + (a[1][0].real * a[1][0].real + a[1][0].imag * a[1][0].imag)
        + (a[1][1].real * a[1][1].real + a[1][1].imag * a[1][1].imag)
    )


def wheel_lie_series(primes: Sequence[int], base: int) -> Dict[str, List[float]]:
    rs = totatives(base)
    idx = {r: i for i, r in enumerate(rs)}
    phi = len(rs)
    seq = [p for p in primes if gcd(p, base) == 1]
    if len(seq) < 16:
        return {k: [] for k in CHANNELS}

    theta = []
    chi = []
    for i, p in enumerate(seq):
        r = p % base
        t = 2.0 * math.pi * (idx[r] / phi)
        gap = 0 if i == 0 else (seq[i] - seq[i - 1]) % base
        # Colatitude tied to local modular gap phase in [0, pi].
        c = math.pi * (gap / base)
        theta.append(t)
        chi.append(c)

    h = {k: [] for k in ["Y11c", "Y11s", "Y20", "Y22c", "Y22s"]}
    for t, c in zip(theta, chi):
        vals = harmonics_from_angles(t, c)
        for k, v in vals.items():
            h[k].append(v)

    transport = [unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
    curvature = [transport[i + 1] - transport[i] for i in range(len(transport) - 1)]
    h["transport"] = transport
    h["curvature"] = curvature
    su2 = [su2_step(theta[i], chi[i], transport[i]) for i in range(len(transport))]
    h["su2_trace"] = [float((u[0][0] + u[1][1]).real) for u in su2]
    su2_comm = []
    for i in range(len(su2) - 1):
        uv = mmul(su2[i], su2[i + 1])
        vu = mmul(su2[i + 1], su2[i])
        su2_comm.append(fnorm(msub(uv, vu)))
    h["su2_comm"] = su2_comm
    # Square-regime ids used for stability penalties.
    rid_point = [int(math.isqrt(p)) for p in seq]
    h["_regime_ids"] = {
        "Y11c": rid_point[: len(h["Y11c"])],
        "Y11s": rid_point[: len(h["Y11s"])],
        "Y20": rid_point[: len(h["Y20"])],
        "Y22c": rid_point[: len(h["Y22c"])],
        "Y22s": rid_point[: len(h["Y22s"])],
        "transport": rid_point[1 : 1 + len(h["transport"])],
        "curvature": rid_point[2 : 2 + len(h["curvature"])],
        "su2_trace": rid_point[1 : 1 + len(h["su2_trace"])],
        "su2_comm": rid_point[2 : 2 + len(h["su2_comm"])],
    }
    return h


def wheel_lie_series_gauge_v2(primes: Sequence[int], base: int, gauge_beta: float, gauge_base: int) -> Dict[str, List[float]]:
    """Gauge-aware variant: transport includes coupled residue + gap phase connection."""
    rs = totatives(base)
    idx = {r: i for i, r in enumerate(rs)}
    phi = len(rs)
    seq = [p for p in primes if gcd(p, base) == 1]
    if len(seq) < 16:
        return {k: [] for k in CHANNELS}

    theta = []
    chi = []
    for i, p in enumerate(seq):
        r = p % base
        t = 2.0 * math.pi * (idx[r] / phi)
        gap = 0 if i == 0 else (seq[i] - seq[i - 1]) % base
        c = math.pi * (gap / base)
        theta.append(t)
        chi.append(c)

    h = {k: [] for k in ["Y11c", "Y11s", "Y20", "Y22c", "Y22s"]}
    for t, c in zip(theta, chi):
        vals = harmonics_from_angles(t, c)
        for k, v in vals.items():
            h[k].append(v)

    theta_step = [unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
    chi_step = [unwrap_delta(chi[i], chi[i + 1]) for i in range(len(chi) - 1)]
    beta = gauge_beta if base == gauge_base else 0.0
    transport = [theta_step[i] + beta * chi_step[i] for i in range(min(len(theta_step), len(chi_step)))]
    curvature = [transport[i + 1] - transport[i] for i in range(len(transport) - 1)]
    h["transport"] = transport
    h["curvature"] = curvature

    su2 = [su2_step(theta[i], chi[i], transport[i]) for i in range(min(len(transport), len(theta), len(chi)))]
    h["su2_trace"] = [float((u[0][0] + u[1][1]).real) for u in su2]
    su2_comm = []
    for i in range(len(su2) - 1):
        uv = mmul(su2[i], su2[i + 1])
        vu = mmul(su2[i + 1], su2[i])
        su2_comm.append(fnorm(msub(uv, vu)))
    h["su2_comm"] = su2_comm
    rid_point = [int(math.isqrt(p)) for p in seq]
    h["_regime_ids"] = {
        "Y11c": rid_point[: len(h["Y11c"])],
        "Y11s": rid_point[: len(h["Y11s"])],
        "Y20": rid_point[: len(h["Y20"])],
        "Y22c": rid_point[: len(h["Y22c"])],
        "Y22s": rid_point[: len(h["Y22s"])],
        "transport": rid_point[1 : 1 + len(h["transport"])],
        "curvature": rid_point[2 : 2 + len(h["curvature"])],
        "su2_trace": rid_point[1 : 1 + len(h["su2_trace"])],
        "su2_comm": rid_point[2 : 2 + len(h["su2_comm"])],
    }
    return h


def regime_penalty(series: Sequence[float], regime_ids: Sequence[int], min_count: int = 8) -> float:
    if not series or not regime_ids or len(series) != len(regime_ids):
        return 1.0
    groups: Dict[int, List[float]] = {}
    for x, rid in zip(series, regime_ids):
        groups.setdefault(rid, []).append(float(x))
    means = []
    for vals in groups.values():
        if len(vals) >= min_count:
            means.append(sum(abs(v) for v in vals) / len(vals))
    if len(means) < 2:
        return 1.0
    m = sum(means) / len(means)
    var = sum((a - m) ** 2 for a in means) / len(means)
    rel = var / max(1e-12, m * m)
    return 1.0 / (1.0 + rel)


def regime_whiten(series: Sequence[float], regime_ids: Sequence[int], min_count: int = 8) -> List[float]:
    if not series or not regime_ids or len(series) != len(regime_ids):
        return list(series)
    groups: Dict[int, List[int]] = {}
    for i, rid in enumerate(regime_ids):
        groups.setdefault(rid, []).append(i)

    out = list(series)
    for idxs in groups.values():
        if len(idxs) < min_count:
            continue
        vals = [series[i] for i in idxs]
        mu = sum(vals) / len(vals)
        var = sum((v - mu) ** 2 for v in vals) / len(vals)
        sd = math.sqrt(var)
        if sd < 1e-12:
            continue
        for i in idxs:
            out[i] = (series[i] - mu) / sd
    return out


def compress_series(series: Sequence[float], max_points: int) -> List[float]:
    if max_points <= 0 or len(series) <= max_points:
        return list(series)
    n = len(series)
    out = []
    for i in range(max_points):
        lo = int(i * n / max_points)
        hi = int((i + 1) * n / max_points)
        if hi <= lo:
            hi = min(n, lo + 1)
        seg = series[lo:hi]
        out.append(sum(seg) / len(seg))
    return out


def quantile(values: Sequence[float], q: float) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    i = int(max(0, min(len(v) - 1, round(q * (len(v) - 1)))))
    return v[i]


def winsorize(values: Sequence[float], q_low: float = 0.1, q_high: float = 0.9) -> List[float]:
    if not values:
        return []
    lo = quantile(values, q_low)
    hi = quantile(values, q_high)
    return [min(hi, max(lo, x)) for x in values]


def median(values: Sequence[float]) -> float:
    if not values:
        return 0.0
    v = sorted(values)
    n = len(v)
    m = n // 2
    return v[m] if n % 2 == 1 else 0.5 * (v[m - 1] + v[m])


def clip(x: float, lo: float = -25.0, hi: float = 25.0) -> float:
    return min(hi, max(lo, x))


def key_hash(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_sig(zeros: Sequence[float]) -> str:
    text = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


def load_cache(path: str) -> Dict[str, object]:
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def save_cache(path: str, data: Dict[str, object]) -> None:
    if not path:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(data, f)


def score_base(
    base: int,
    n_values: Sequence[int],
    primes_by_n: Dict[int, Sequence[int]],
    zeros: Sequence[float],
    perm_trials: int,
    cache: Dict[str, object],
    zsig: str,
    gauge_beta: float,
    gauge_base: int,
    regime_gamma: float,
    regime_whiten_flag: bool,
    regime_min_count: int,
    max_control_points: int,
    control_max_zeros: int,
) -> Dict[str, object]:
    rows = []
    for i, n in enumerate(n_values):
        k = key_hash(
            {
                "v": "lie_harm_v3",
                "base": base,
                "n": n,
                "perm_trials": perm_trials,
                "zsig": zsig,
                "gauge_beta": gauge_beta,
                "gauge_base": gauge_base,
                "regime_gamma": regime_gamma,
                "regime_whiten_flag": regime_whiten_flag,
                "regime_min_count": regime_min_count,
                "max_control_points": max_control_points,
                "control_max_zeros": control_max_zeros,
            }
        )
        if k in cache:
            rows.append(cache[k])
            continue
        s = wheel_lie_series_gauge_v2(primes_by_n[n], base, gauge_beta, gauge_base)
        regime_ids = s.get("_regime_ids", {})
        ch = {}
        for c in CHANNELS:
            series = (
                regime_whiten(s[c], regime_ids.get(c, []), regime_min_count)
                if regime_whiten_flag
                else s[c]
            )
            control_series = compress_series(series, max_control_points)
            control_zeros = zeros[:control_max_zeros] if control_max_zeros > 0 else zeros
            ctrl = zeta_permutation_control(
                control_series,
                perm_trials,
                20260701 + base + i * 11 + len(c),
                zeros_imag=control_zeros,
            )
            raw_p = ctrl.get("p_value_ge", 1.0)
            raw_z = clip(ctrl.get("z_score", 0.0))
            raw_eff = 1.0 - 2.0 * raw_p
            rp = regime_penalty(series, regime_ids.get(c, []), regime_min_count)
            penalty = (1.0 - regime_gamma) + regime_gamma * rp
            adj_p = min(1.0, raw_p + (1.0 - penalty) * 0.5)
            ch[c] = {
                "p": adj_p,
                "z": raw_z * penalty,
                "eff": raw_eff * penalty,
                "regime_penalty": penalty,
            }
        row = {"n_max": n, "channels": ch}
        cache[k] = row
        rows.append(row)

    p_raw = [sum(r["channels"][c]["p"] for c in CHANNELS) / len(CHANNELS) for r in rows]
    z_raw = [sum(r["channels"][c]["z"] for c in CHANNELS) / len(CHANNELS) for r in rows]
    e_raw = [sum(r["channels"][c]["eff"] for c in CHANNELS) / len(CHANNELS) for r in rows]

    p_w = winsorize(p_raw)
    z_w = winsorize(z_raw)
    e_w = winsorize(e_raw)
    med_p = median(p_w)
    med_z = median(z_w)
    med_e = median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good >= 0.5 and e_var <= 0.25

    mean_p = sum(p_raw) / len(p_raw)
    mean_z = sum(z_raw) / len(z_raw)
    mean_e = sum(e_raw) / len(e_raw)
    return {
        "wheel_base": base,
        "channels": CHANNELS,
        "per_n": rows,
        "conjecture": conjecture_from_rows(rows, CHANNELS),
    }


def conjecture_from_rows(rows: Sequence[Dict[str, object]], subset: Sequence[str]) -> Dict[str, float]:
    p_raw = [sum(r["channels"][c]["p"] for c in subset) / len(subset) for r in rows]
    z_raw = [sum(r["channels"][c]["z"] for c in subset) / len(subset) for r in rows]
    e_raw = [sum(r["channels"][c]["eff"] for c in subset) / len(subset) for r in rows]

    p_w = winsorize(p_raw)
    z_w = winsorize(z_raw)
    e_w = winsorize(e_raw)
    med_p = median(p_w)
    med_z = median(z_w)
    med_e = median(e_w)
    z_var = sum((z - med_z) ** 2 for z in z_w) / max(1, len(z_w))
    e_var = sum((e - med_e) ** 2 for e in e_w) / max(1, len(e_w))
    signs = [1 if z > 0 else (-1 if z < 0 else 0) for z in z_w]
    sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))
    p_good = sum(1 for p in p_w if p <= 0.2) / max(1, len(p_w))
    gate = sign_consistency >= 0.66 and z_var <= 25.0 and p_good >= 0.5 and e_var <= 0.25

    mean_p = sum(p_raw) / len(p_raw)
    mean_z = sum(z_raw) / len(z_raw)
    mean_e = sum(e_raw) / len(e_raw)
    return {
        "guardrail_mean_p": mean_p,
        "guardrail_mean_z": mean_z,
        "guardrail_mean_effect": mean_e,
        "guardrail_median_p_winsor": med_p,
        "guardrail_median_z_winsor": med_z,
        "guardrail_median_effect_winsor": med_e,
        "guardrail_z_var_winsor": z_var,
        "guardrail_effect_var_winsor": e_var,
        "guardrail_sign_consistency": sign_consistency,
        "guardrail_p_good_fraction": p_good,
        "stability_gate_pass": gate,
        "support_score": (1.0 - mean_p) * max(0.0, mean_z),
        "support_score_robust": (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var)),
        "support_score_rank": (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var)),
    }


def ablate_channels(rows: Sequence[Dict[str, object]], min_size: int = 3) -> List[Dict[str, object]]:
    out = []
    for k in range(min_size, len(CHANNELS) + 1):
        for sub in combinations(CHANNELS, k):
            c = conjecture_from_rows(rows, sub)
            score = c["support_score_rank"] + 0.2 * c["support_score_robust"] + (0.5 if c["stability_gate_pass"] else 0.0)
            out.append(
                {
                    "channels": list(sub),
                    "subset_size": k,
                    "rank_score": score,
                    "conjecture": c,
                }
            )
    out.sort(key=lambda r: r["rank_score"], reverse=True)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Lie-phase harmonic probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="100000,150000,200000")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--zeta-perm-trials", type=int, default=10)
    ap.add_argument("--gauge-beta", type=float, default=0.0)
    ap.add_argument("--gauge-base", type=int, default=210)
    ap.add_argument("--regime-gamma", type=float, default=0.0, help="0 disables square-regime penalty; 1 full penalty.")
    ap.add_argument("--regime-whiten", action="store_true", help="Whiten channels within square regimes before controls.")
    ap.add_argument("--regime-min-count", type=int, default=8)
    ap.add_argument("--max-control-points", type=int, default=4096, help="Downsample series for spectral controls.")
    ap.add_argument("--control-max-zeros", type=int, default=128, help="Limit zeta zeros used in controls.")
    ap.add_argument("--cache-file", type=str, default="research/output/cache/lie_harmonic_cache.json")
    ap.add_argument("--ablate-base", type=int, default=0, help="Run channel subset scoring for this base.")
    ap.add_argument("--ablate-min-size", type=int, default=3)
    ap.add_argument("--output", type=str, default="research/output/lie_phase_harmonic_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = zeros_sig(zeros)
    primes_by_n = {n: sieve_primes(n) for n in n_values}
    cache = load_cache(args.cache_file)

    rows = []
    for base in bases:
        rows.append(
            score_base(
                base,
                n_values,
                primes_by_n,
                zeros,
                args.zeta_perm_trials,
                cache,
                zsig,
                args.gauge_beta,
                args.gauge_base,
                args.regime_gamma,
                args.regime_whiten,
                args.regime_min_count,
                args.max_control_points,
                args.control_max_zeros,
            )
        )
    save_cache(args.cache_file, cache)

    rows.sort(key=lambda r: (r["conjecture"]["stability_gate_pass"], r["conjecture"]["support_score_robust"]), reverse=True)
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "zeta_perm_trials": args.zeta_perm_trials,
            "max_zeta_zeros": len(zeros),
            "gauge_beta": args.gauge_beta,
            "gauge_base": args.gauge_base,
            "regime_gamma": args.regime_gamma,
            "regime_whiten": args.regime_whiten,
            "regime_min_count": args.regime_min_count,
            "max_control_points": args.max_control_points,
            "control_max_zeros": args.control_max_zeros,
        },
        "leaderboard": rows,
    }
    if args.ablate_base > 0:
        target = next((r for r in rows if r["wheel_base"] == args.ablate_base), None)
        if target:
            report["ablation"] = {
                "base": args.ablate_base,
                "min_subset_size": args.ablate_min_size,
                "top_subsets": ablate_channels(target["per_n"], args.ablate_min_size)[:20],
            }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lie-Phase Harmonic Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for i, r in enumerate(rows, 1):
            c = r["conjecture"]
            f.write(
                f"{i}. base={r['wheel_base']} gate={c['stability_gate_pass']} "
                f"robust={c['support_score_robust']:.6f} rank={c['support_score_rank']:.6f} "
                f"mean_p={c['guardrail_mean_p']:.6f} mean_z={c['guardrail_mean_z']:.6f}\n"
            )
        if "ablation" in report:
            f.write("\n## Base Ablation\n\n")
            for i, row in enumerate(report["ablation"]["top_subsets"], 1):
                c = row["conjecture"]
                f.write(
                    f"{i}. channels={','.join(row['channels'])} gate={c['stability_gate_pass']} "
                    f"robust={c['support_score_robust']:.6f} rank={c['support_score_rank']:.6f}\n"
                )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if rows:
        top = rows[0]
        print("top_base:", top["wheel_base"], "robust:", round(top["conjecture"]["support_score_robust"], 6))


if __name__ == "__main__":
    main()
