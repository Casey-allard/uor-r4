#!/usr/bin/env python3
"""Candidate A on spinning-top R^4 manifold.

R^4 manifold per integer n:
  (Re F(u_n), Im F(u_n), Re F'(u_n), Im F'(u_n))
where F is zeta-zero trajectory sum.

Evaluates Candidate A channels on each wheel base:
  Y11c, curvature, su2_comm
"""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import math
import os
import sys
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import load_zeta_zeros_file, zeta_permutation_control

VENDOR_PATH = os.path.join(os.path.dirname(__file__), ".vendor")
if os.path.isdir(VENDOR_PATH) and VENDOR_PATH not in sys.path:
    sys.path.insert(0, VENDOR_PATH)

try:
    import numpy as np
except Exception:
    np = None


DEFAULT_CANDIDATE = ("phase_vel", "phase_accel", "radial_drift")
ALL_CHANNELS = (
    "Y11c",
    "Y11s",
    "Y20",
    "transport",
    "curvature",
    "su2_trace",
    "su2_comm",
    "phase_vel",
    "phase_accel",
    "radial_drift",
)


def gcd(a: int, b: int) -> int:
    while b:
        a, b = b, a % b
    return a


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2.0 * math.pi
    while d < -math.pi:
        d += 2.0 * math.pi
    return d


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


def load_cache(path: str) -> Dict[str, object]:
    if not path or not os.path.exists(path):
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            d = json.load(f)
        return d if isinstance(d, dict) else {}
    except Exception:
        return {}


def save_cache(path: str, cache: Dict[str, object]) -> None:
    if not path:
        return
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(cache, f)


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def su2_step(theta: float, chi: float, alpha: float):
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


def zero_weight(gamma: float, kernel: str, scale: float) -> float:
    if kernel == "none" or scale <= 0:
        return 1.0
    x = gamma / scale
    if kernel == "gaussian":
        return math.exp(-(x * x))
    if kernel == "lorentz":
        return 1.0 / (1.0 + x * x)
    return 1.0


def manifold_points(
    n_max: int,
    zeros: Sequence[float],
    u_mode: str,
    zero_kernel: str = "none",
    kernel_scale: float = 0.0,
) -> List[Tuple[float, float, float, float]]:
    if np is not None and zeros:
        indices = np.arange(2, n_max + 1, dtype=np.float64)
        u = np.log(indices) if u_mode == "log" else indices
        g_all = np.asarray(zeros, dtype=np.float64)
        w_all = np.asarray([zero_weight(float(g), zero_kernel, kernel_scale) for g in g_all], dtype=np.float64)
        dinv_all = 1.0 / (0.25 + g_all * g_all)
        fr = np.zeros_like(u)
        fi = np.zeros_like(u)
        gr = np.zeros_like(u)
        gi = np.zeros_like(u)
        chunk = 16
        for lo in range(0, len(g_all), chunk):
            hi = min(len(g_all), lo + chunk)
            g = g_all[lo:hi]
            w = w_all[lo:hi]
            dinv = dinv_all[lo:hi]
            phase = np.outer(u, g)
            c = np.cos(phase)
            s = np.sin(phase)
            core_ar = (0.5 * c + s * g) * dinv
            core_ai = (0.5 * s - c * g) * dinv
            ar = core_ar * w
            ai = core_ai * w
            fr += ar.sum(axis=1)
            fi += ai.sum(axis=1)
            gr += (-(g * ai)).sum(axis=1)
            gi += ((g * ar)).sum(axis=1)
        return [(float(a), float(b), float(c), float(d)) for a, b, c, d in zip(fr, fi, gr, gi)]

    pts = []
    for n in range(2, n_max + 1):
        u = math.log(n) if u_mode == "log" else float(n)
        fr = fi = gr = gi = 0.0
        for g in zeros:
            w = zero_weight(g, zero_kernel, kernel_scale)
            c = math.cos(g * u)
            s = math.sin(g * u)
            den = 0.25 + g * g
            ar = (0.5 * c + g * s) / den
            ai = (0.5 * s - g * c) / den
            fr += w * ar
            fi += w * ai
            gr += w * (-g * ai)
            gi += w * (g * ar)
        pts.append((fr, fi, gr, gi))
    return pts


def candidate_series_for_base(
    points: Sequence[Tuple[float, float, float, float]], seq_idx: Sequence[int]
) -> Dict[str, List[float]]:
    # Keep integers coprime to base for wheel-consistent subsequence.
    if len(seq_idx) < 16:
        return {k: [] for k in ALL_CHANNELS}

    theta = []
    chi = []
    for n in seq_idx:
        x, y, z, t = points[n - 2]
        theta.append(math.atan2(y, x))
        # map phase of F' to [0, pi] as colatitude-like variable
        phi2 = math.atan2(t, z)
        chi.append(0.5 * (phi2 + math.pi))

    y11c = [math.sqrt(3.0 / (4.0 * math.pi)) * math.sin(c) * math.cos(th) for th, c in zip(theta, chi)]
    y11s = [math.sqrt(3.0 / (4.0 * math.pi)) * math.sin(c) * math.sin(th) for th, c in zip(theta, chi)]
    y20 = [math.sqrt(5.0 / (16.0 * math.pi)) * (3.0 * (math.cos(c) ** 2) - 1.0) for c in chi]
    transport = [unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
    curvature = [transport[i + 1] - transport[i] for i in range(len(transport) - 1)]

    su2 = [su2_step(theta[i], chi[i], transport[i]) for i in range(min(len(transport), len(theta), len(chi)))]
    su2_comm = []
    for i in range(len(su2) - 1):
        uv = mmul(su2[i], su2[i + 1])
        vu = mmul(su2[i + 1], su2[i])
        su2_comm.append(fnorm(msub(uv, vu)))
    su2_trace = [float((u[0][0] + u[1][1]).real) for u in su2]

    radius = []
    for n in seq_idx:
        x, y, z, t = points[n - 2]
        radius.append(math.sqrt(x * x + y * y + z * z + t * t))
    phase_vel = [unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
    phase_accel = [phase_vel[i + 1] - phase_vel[i] for i in range(len(phase_vel) - 1)]
    radial_drift = [radius[i + 1] - radius[i] for i in range(len(radius) - 1)]

    m = min(
        len(curvature),
        len(y11c),
        len(y11s),
        len(y20),
        len(transport),
        len(su2_trace),
        len(su2_comm),
        len(phase_vel),
        len(phase_accel),
        len(radial_drift),
    )

    return {
        "Y11c": y11c[:m],
        "Y11s": y11s[:m],
        "Y20": y20[:m],
        "transport": transport[:m],
        "curvature": curvature[:m],
        "su2_trace": su2_trace[:m],
        "su2_comm": su2_comm[:m],
        "phase_vel": phase_vel[:m],
        "phase_accel": phase_accel[:m],
        "radial_drift": radial_drift[:m],
    }


def row_stats(
    channels: Dict[str, List[float]],
    zeros: Sequence[float],
    trials: int,
    seed: int,
    max_points: int,
    max_zeros: int,
):
    stats = {}
    control_zeros = zeros[:max_zeros] if max_zeros > 0 else zeros
    for i, c in enumerate(ALL_CHANNELS):
        s = channels[c]
        s = compress_series(s, max_points)
        ctrl = zeta_permutation_control(s, trials, seed + 17 * i, zeros_imag=control_zeros)
        p = ctrl.get("p_value_ge", 1.0)
        z = ctrl.get("z_score", 0.0)
        e = 1.0 - 2.0 * p
        stats[c] = {"p": p, "z": z, "eff": e}
    return stats


def gate_from_rows(rows: Sequence[Dict[str, object]], subset: Sequence[str]) -> Dict[str, float]:
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
    robust = (1.0 - med_p) * max(0.0, med_z) / (1.0 + math.sqrt(z_var))
    rank = (1.0 - med_p) * max(0.0, med_e) / (1.0 + math.sqrt(e_var))
    return {
        "gate": gate,
        "robust": robust,
        "rank": rank,
        "p_good": p_good,
        "z_var": z_var,
        "e_var": e_var,
        "sign_consistency": sign_consistency,
    }


def find_best_subset(rows_by_base: Dict[int, Sequence[Dict[str, object]]], min_k: int = 3):
    best = None
    for k in range(min_k, len(ALL_CHANNELS) + 1):
        for sub in itertools.combinations(ALL_CHANNELS, k):
            metrics = {b: gate_from_rows(rows, sub) for b, rows in rows_by_base.items()}
            g_all = all(m["gate"] for m in metrics.values())
            transfer = min(m["rank"] for m in metrics.values()) + 0.2 * min(m["robust"] for m in metrics.values()) + (0.5 if g_all else 0.0)
            cand = {"subset": list(sub), "per_base": metrics, "global_pass": g_all, "transfer_score": transfer}
            if best is None or cand["transfer_score"] > best["transfer_score"]:
                best = cand
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="Candidate A on spinning-top R^4 manifold")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="100000,150000,200000")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--zeta-perm-trials", type=int, default=12)
    ap.add_argument("--max-control-points", type=int, default=2048)
    ap.add_argument("--control-max-zeros", type=int, default=64)
    ap.add_argument("--candidate-channels", type=str, default=",".join(DEFAULT_CANDIDATE))
    ap.add_argument("--ablate", action="store_true")
    ap.add_argument("--cache-file", type=str, default="research/output/cache/spinning_top_r4_candidate_a_cache.json")
    ap.add_argument("--output", type=str, default="research/output/spinning_top_r4_candidate_a.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros
    zsig = zeros_sig(zeros)
    cache = load_cache(args.cache_file)

    max_n = max(n_values)
    # Compute manifold once, then slice for each n.
    all_points = manifold_points(max_n, zeros, args.u_mode, args.zero_kernel, args.kernel_scale)
    # Precompute coprime indices once per base.
    base_indices = {b: [n for n in range(2, max_n + 1) if gcd(n, b) == 1] for b in bases}

    subset = [x.strip() for x in args.candidate_channels.split(",") if x.strip()]
    if not subset:
        subset = list(DEFAULT_CANDIDATE)
    for s in subset:
        if s not in ALL_CHANNELS:
            raise ValueError(f"unknown channel: {s}")

    per_base = []
    rows_by_base = {}
    for b in bases:
        rows = []
        for n in n_values:
            key = cache_key(
                {
                    "v": "spin_r4_a_v3",
                    "base": b,
                    "n": n,
                    "u_mode": args.u_mode,
                    "trials": args.zeta_perm_trials,
                    "max_points": args.max_control_points,
                    "max_zeros": args.control_max_zeros,
                    "zsig": zsig,
                    "zero_kernel": args.zero_kernel,
                    "kernel_scale": args.kernel_scale,
                }
            )
            if key in cache:
                rows.append(cache[key])
                continue

            pts = all_points[: n - 1]
            idx = [k for k in base_indices[b] if k <= n]
            ch = candidate_series_for_base(pts, idx)
            stat = row_stats(
                ch,
                zeros,
                args.zeta_perm_trials,
                20260801 + b + n,
                args.max_control_points,
                args.control_max_zeros,
            )
            row = {"n_max": n, "channels": stat}
            cache[key] = row
            rows.append(row)
        metrics = gate_from_rows(rows, subset)
        rows_by_base[b] = rows
        per_base.append({"wheel_base": b, "rows": rows, **metrics})

    per_base.sort(key=lambda r: r["wheel_base"])
    global_pass = all(r["gate"] for r in per_base)
    transfer_score = min(r["rank"] for r in per_base) + 0.2 * min(r["robust"] for r in per_base) + (0.5 if global_pass else 0.0)
    best_subset = find_best_subset(rows_by_base, min_k=3) if args.ablate else None
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": n_values,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "max_zeta_zeros": len(zeros),
            "zeta_perm_trials": args.zeta_perm_trials,
            "max_control_points": args.max_control_points,
            "control_max_zeros": args.control_max_zeros,
            "cache_file": args.cache_file,
        },
        "candidate_channels": list(subset),
        "per_base": per_base,
        "global": {
            "pass": global_pass,
            "transfer_score": transfer_score,
        },
    }
    if best_subset:
        report["ablation_best"] = best_subset

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    save_cache(args.cache_file, cache)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Spinning-Top R4 Candidate A\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Global pass: {global_pass}\n")
        f.write(f"Transfer score: {transfer_score:.6f}\n\n")
        f.write(f"Candidate channels: {','.join(subset)}\n\n")
        for r in per_base:
            f.write(
                f"- base={r['wheel_base']} pass={r['gate']} robust={r['robust']:.6f} "
                f"rank={r['rank']:.6f} p_good={r['p_good']:.3f} z_var={r['z_var']:.6f}\n"
            )
        if best_subset:
            f.write("\n## Best Ablation Subset\n\n")
            f.write(f"- channels: {','.join(best_subset['subset'])}\n")
            f.write(f"- global_pass: {best_subset['global_pass']}\n")
            f.write(f"- transfer_score: {best_subset['transfer_score']:.6f}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("global_pass:", global_pass)
    print("transfer_score:", round(transfer_score, 6))


if __name__ == "__main__":
    main()
