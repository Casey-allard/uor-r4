#!/usr/bin/env python3
"""Tail-bound style probe for kernelized H(x) truncation.

Compares H_M to H_Mref and reports empirical error envelopes by base.
Also reports analytic tail sums for F and F' coordinates.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

import hx_truncation_probe as hxt
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


def analytic_tail_sums(zeros: Sequence[float], m: int, kernel: str, scale: float) -> Dict[str, float]:
    # Bounds:
    # |ΔF| <= Σ_{j>m} K(gamma_j)/|1/2+i gamma_j|
    # |ΔF'| <= Σ_{j>m} K(gamma_j)*gamma_j/|1/2+i gamma_j|
    t0 = 0.0
    t1 = 0.0
    for g in zeros[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t0 += w / den
        t1 += w * (g / den)
    return {"tail_F_bound": t0, "tail_Fp_bound": t1}


def max_abs_diff(a: Sequence[float], b: Sequence[float]) -> float:
    return max(abs(x - y) for x, y in zip(a, b)) if a and len(a) == len(b) else 0.0


def l2_diff(a: Sequence[float], b: Sequence[float]) -> float:
    if not a or len(a) != len(b):
        return 0.0
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)) / len(a))


def candidate_series_from_sequence(points_seq: Sequence[Tuple[float, float, float, float]]) -> Dict[str, List[float]]:
    if len(points_seq) < 16:
        return {k: [] for k in s4.ALL_CHANNELS}
    theta = [math.atan2(p[1], p[0]) for p in points_seq]
    phi2 = [math.atan2(p[3], p[2]) for p in points_seq]
    chi = [0.5 * (v + math.pi) for v in phi2]

    y11c = [math.sqrt(3.0 / (4.0 * math.pi)) * math.sin(c) * math.cos(th) for th, c in zip(theta, chi)]
    y11s = [math.sqrt(3.0 / (4.0 * math.pi)) * math.sin(c) * math.sin(th) for th, c in zip(theta, chi)]
    y20 = [math.sqrt(5.0 / (16.0 * math.pi)) * (3.0 * (math.cos(c) ** 2) - 1.0) for c in chi]
    transport = [s4.unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
    curvature = [transport[i + 1] - transport[i] for i in range(len(transport) - 1)]

    su2 = [s4.su2_step(theta[i], chi[i], transport[i]) for i in range(min(len(transport), len(theta), len(chi)))]
    su2_trace = [float((u[0][0] + u[1][1]).real) for u in su2]
    su2_comm = []
    for i in range(len(su2) - 1):
        uv = s4.mmul(su2[i], su2[i + 1])
        vu = s4.mmul(su2[i + 1], su2[i])
        su2_comm.append(s4.fnorm(s4.msub(uv, vu)))

    radius = [math.sqrt(p[0] * p[0] + p[1] * p[1] + p[2] * p[2] + p[3] * p[3]) for p in points_seq]
    phase_vel = [s4.unwrap_delta(theta[i], theta[i + 1]) for i in range(len(theta) - 1)]
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


def h_scaled_from_sequence(points_seq, seq_n: Sequence[int], n_max: int, x_step: int) -> Dict[str, Sequence[float]]:
    ch = candidate_series_from_sequence(points_seq)
    subset = ["Y11s", "Y20", "su2_trace", "phase_vel"]
    m = min(len(ch[k]) for k in subset)
    idx = list(seq_n[:m])
    g_by_n = [0.0] * (n_max + 1)
    for j, n in enumerate(idx):
        g_by_n[n] = (
            hxt.WEIGHTS[0] * ch["Y11s"][j]
            + hxt.WEIGHTS[1] * ch["Y20"][j]
            + hxt.WEIGHTS[2] * ch["su2_trace"][j]
            + hxt.WEIGHTS[3] * ch["phase_vel"][j]
        )
    h = [0.0] * (n_max + 1)
    for n in range(2, n_max + 1):
        h[n] = h[n - 1] + g_by_n[n]
    xs = list(range(max(2, x_step), n_max + 1, x_step))
    hs = [h[x] / math.sqrt(x) for x in xs]
    return {"x": xs, "h_scaled": hs}


def manifold_points_at_limits_for_indices(
    indices: Sequence[int],
    zeros_ref: Sequence[float],
    limits: Sequence[int],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
) -> Dict[int, List[Tuple[float, float, float, float]]]:
    checkpoints = sorted(set(limits))
    if not checkpoints:
        return {}
    us = [math.log(n) if u_mode == "log" else float(n) for n in indices]
    fr = [0.0] * len(indices)
    fi = [0.0] * len(indices)
    gr = [0.0] * len(indices)
    gi = [0.0] * len(indices)
    out: Dict[int, List[Tuple[float, float, float, float]]] = {}
    ck_ptr = 0
    ck_target = checkpoints[ck_ptr]
    for j, g in enumerate(zeros_ref, start=1):
        w = s4.zero_weight(g, zero_kernel, kernel_scale)
        den = 0.25 + g * g
        for i, u in enumerate(us):
            c = math.cos(g * u)
            s = math.sin(g * u)
            ar = (0.5 * c + g * s) / den
            ai = (0.5 * s - g * c) / den
            fr[i] += w * ar
            fi[i] += w * ai
            gr[i] += w * (-g * ai)
            gi[i] += w * (g * ar)
        if j == ck_target:
            out[j] = [(fr[i], fi[i], gr[i], gi[i]) for i in range(len(indices))]
            ck_ptr += 1
            if ck_ptr >= len(checkpoints):
                break
            ck_target = checkpoints[ck_ptr]
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Kernelized H(x) tail bound probe")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-max", type=int, default=300000)
    ap.add_argument("--x-step", type=int, default=1000)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--zero-limits", type=str, default="32,64,96,128,192,256")
    ap.add_argument("--ref-zero-limit", type=int, default=512)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--output", type=str, default="research/output/hx_tail_bound_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    limits = sorted(set(int(x.strip()) for x in args.zero_limits.split(",") if x.strip()))
    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.ref_zero_limit > len(zeros_all):
        raise ValueError("ref-zero-limit exceeds available zeros")
    limits = [m for m in limits if 0 < m < args.ref_zero_limit]
    if not limits:
        raise ValueError("no valid zero limits below ref-zero-limit")

    # Base-30 coprimes form a superset for 210/2310/30030; compute only on this superset.
    sup_n = [n for n in range(2, args.n_max + 1) if s4.gcd(n, 30) == 1]
    base_pos = {}
    for b in bases:
        base_pos[b] = [i for i, n in enumerate(sup_n) if s4.gcd(n, b) == 1]

    # Precompute manifold incrementally once per M on superset indices.
    all_limits = sorted(set(limits + [args.ref_zero_limit]))
    points_by_m = manifold_points_at_limits_for_indices(
        sup_n,
        zeros_all[: args.ref_zero_limit],
        all_limits,
        args.u_mode,
        args.zero_kernel,
        args.kernel_scale,
    )

    rows = []
    for b in bases:
        ref_points = [points_by_m[args.ref_zero_limit][i] for i in base_pos[b]]
        ref_n = [sup_n[i] for i in base_pos[b]]
        ref = h_scaled_from_sequence(ref_points, ref_n, args.n_max, args.x_step)
        base_rows = []
        for m in limits:
            cur_points = [points_by_m[m][i] for i in base_pos[b]]
            cur_n = ref_n
            cur = h_scaled_from_sequence(cur_points, cur_n, args.n_max, args.x_step)
            base_rows.append(
                {
                    "M": m,
                    "max_abs_diff_vs_ref": max_abs_diff(cur["h_scaled"], ref["h_scaled"]),
                    "l2_diff_vs_ref": l2_diff(cur["h_scaled"], ref["h_scaled"]),
                    **analytic_tail_sums(zeros_all[: args.ref_zero_limit], m, args.zero_kernel, args.kernel_scale),
                }
            )
        rows.append({"base": b, "rows": base_rows})

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_max": args.n_max,
            "x_step": args.x_step,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "zero_limits": limits,
            "ref_zero_limit": args.ref_zero_limit,
        },
        "per_base": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# H(x) Tail Bound Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(
            f"Kernel: {args.zero_kernel}, scale={args.kernel_scale}, ref_zero_limit={args.ref_zero_limit}, n_max={args.n_max}\n\n"
        )
        for br in rows:
            f.write(f"## base={br['base']}\n\n")
            for r in br["rows"]:
                f.write(
                    f"- M={r['M']} max_abs_diff={r['max_abs_diff_vs_ref']:.6f} "
                    f"l2_diff={r['l2_diff_vs_ref']:.6f} "
                    f"tail_F_bound={r['tail_F_bound']:.6f} tail_Fp_bound={r['tail_Fp_bound']:.6f}\n"
                )
            f.write("\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
