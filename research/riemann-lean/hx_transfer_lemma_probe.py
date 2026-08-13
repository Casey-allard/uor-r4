#!/usr/bin/env python3
"""Transfer-lemma probe for frozen H(x) v2 bridge."""

from __future__ import annotations

import argparse
import bisect
import concurrent.futures
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

import hx_bridge_probe as hx
import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


WEIGHTS = (-1.0, -1.0, 0.0, -1.0)


def _stream_base_worker(payload):
    base, n_max, zeros, u_mode, zero_kernel, kernel_scale, chunk_n = payload
    seq_n, g_vals = hx.stream_weighted_events(
        n_max=n_max,
        base=base,
        zeros=zeros,
        u_mode=u_mode,
        zero_kernel=zero_kernel,
        kernel_scale=kernel_scale,
        weights=WEIGHTS,
        chunk_n=chunk_n,
    )
    return base, seq_n, g_vals


def manifold_points_for_indices(
    indices: Sequence[int],
    zeros: Sequence[float],
    u_mode: str,
    zero_kernel: str,
    kernel_scale: float,
) -> List[Tuple[float, float, float, float]]:
    np = s4.np
    if np is not None and zeros:
        u = np.asarray([math.log(n) if u_mode == "log" else float(n) for n in indices], dtype=np.float64)
        g_all = np.asarray(zeros, dtype=np.float64)
        w_all = np.asarray([s4.zero_weight(float(g), zero_kernel, kernel_scale) for g in g_all], dtype=np.float64)
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

    us = [math.log(n) if u_mode == "log" else float(n) for n in indices]
    zero_terms = [
        (g, s4.zero_weight(g, zero_kernel, kernel_scale), 1.0 / (0.25 + g * g))
        for g in zeros
    ]
    pts = []
    for u in us:
        fr = fi = gr = gi = 0.0
        for g, w, den_inv in zero_terms:
            c = math.cos(g * u)
            s = math.sin(g * u)
            ar = (0.5 * c + g * s) * den_inv
            ai = (0.5 * s - g * c) * den_inv
            fr += w * ar
            fi += w * ai
            gr += w * (-g * ai)
            gi += w * (g * ar)
        pts.append((fr, fi, gr, gi))
    return pts


def candidate_series_from_positions(
    points_all: Sequence[Tuple[float, float, float, float]],
    pos_idx: Sequence[int],
) -> Dict[str, List[float]]:
    if len(pos_idx) < 16:
        return {k: [] for k in s4.ALL_CHANNELS}
    theta = [math.atan2(points_all[i][1], points_all[i][0]) for i in pos_idx]
    phi2 = [math.atan2(points_all[i][3], points_all[i][2]) for i in pos_idx]
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
    radius = [
        math.sqrt(
            points_all[i][0] * points_all[i][0]
            + points_all[i][1] * points_all[i][1]
            + points_all[i][2] * points_all[i][2]
            + points_all[i][3] * points_all[i][3]
        )
        for i in pos_idx
    ]
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


def h_scaled_sparse(seq_n: Sequence[int], g_vals: Sequence[float], n_max: int, x_step: int) -> Dict[str, Sequence[float]]:
    xs = list(range(max(2, x_step), n_max + 1, x_step))
    hs = []
    i = 0
    acc = 0.0
    for x in xs:
        while i < len(seq_n) and seq_n[i] <= x:
            acc += g_vals[i]
            i += 1
        hs.append(acc / math.sqrt(x))
    return {"x": xs, "h_scaled": hs}


def fit_affine_global(h_all: Sequence[float], e_all: Sequence[float]):
    a, b, rmse = hx.fit_linear(h_all, e_all)
    return a, b, rmse


def residual_stats(e: Sequence[float], fit: Sequence[float]) -> Dict[str, float]:
    r = [a - b for a, b in zip(e, fit)]
    if not r:
        return {"rmse": 0.0, "max_abs": 0.0, "mean_abs": 0.0}
    rmse = math.sqrt(sum(v * v for v in r) / len(r))
    ma = sum(abs(v) for v in r) / len(r)
    mx = max(abs(v) for v in r)
    return {"rmse": rmse, "mean_abs": ma, "max_abs": mx}


def main() -> None:
    ap = argparse.ArgumentParser(description="Transfer lemma probe for H(x) v2")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-values", type=str, default="120000,200000,300000")
    ap.add_argument("--x-step", type=int, default=1000)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--streaming", action="store_true", help="compute per-base events via chunked streaming")
    ap.add_argument("--chunk-n", type=int, default=20000, help="streaming chunk size in n")
    ap.add_argument("--jobs", type=int, default=1, help="parallel base workers in streaming mode")
    ap.add_argument("--output", type=str, default="research/output/hx_transfer_lemma_probe.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    n_values = [int(x.strip()) for x in args.n_values.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    rows = []
    h_pool = []
    e_pool = []
    n_values = sorted(n_values)
    n_max_global = n_values[-1]
    xs_by_n = {n: list(range(max(2, args.x_step), n + 1, args.x_step)) for n in n_values}
    all_xs = sorted({x for xs in xs_by_n.values() for x in xs})
    psi_at = {}
    for x, v in zip(all_xs, hx.psi_exact_samples(all_xs)):
        psi_at[x] = v

    base_series = {}
    if args.streaming:
        payloads = [
            (b, n_max_global, zeros, args.u_mode, args.zero_kernel, args.kernel_scale, args.chunk_n)
            for b in bases
        ]
        if args.jobs > 1 and len(payloads) > 1:
            try:
                with concurrent.futures.ProcessPoolExecutor(max_workers=args.jobs) as ex:
                    for b, seq_n, g_vals in ex.map(_stream_base_worker, payloads):
                        base_series[b] = (seq_n, g_vals)
            except (PermissionError, OSError):
                for p in payloads:
                    b, seq_n, g_vals = _stream_base_worker(p)
                    base_series[b] = (seq_n, g_vals)
        else:
            for p in payloads:
                b, seq_n, g_vals = _stream_base_worker(p)
                base_series[b] = (seq_n, g_vals)
    else:
        # Superset for all target bases: coprimes to 30.
        sup_n = [n for n in range(2, n_max_global + 1) if s4.gcd(n, 30) == 1]
        pts_sup = manifold_points_for_indices(
            sup_n,
            zeros,
            args.u_mode,
            args.zero_kernel,
            args.kernel_scale,
        )
        base_pos = {}
        base_n = {}
        for b in bases:
            pos = [i for i, n in enumerate(sup_n) if s4.gcd(n, b) == 1]
            base_pos[b] = pos
            base_n[b] = [sup_n[i] for i in pos]
            ch = candidate_series_from_positions(pts_sup, pos)
            m = min(len(ch[k]) for k in ["Y11s", "Y20", "su2_trace", "phase_vel"])
            seq_n = base_n[b][:m]
            g_vals = [
                WEIGHTS[0] * ch["Y11s"][i]
                + WEIGHTS[1] * ch["Y20"][i]
                + WEIGHTS[2] * ch["su2_trace"][i]
                + WEIGHTS[3] * ch["phase_vel"][i]
                for i in range(m)
            ]
            base_series[b] = (seq_n, g_vals)

    for n_max in n_values:
        for b in bases:
            seq_n_all, g_vals_all = base_series[b]
            cut = bisect.bisect_right(seq_n_all, n_max)
            seq_n = seq_n_all[:cut]
            g_vals = g_vals_all[:cut]
            hser = h_scaled_sparse(seq_n, g_vals, n_max, args.x_step)
            es = [(psi_at[x] - x) / math.sqrt(x) for x in hser["x"]]
            s = {"x": hser["x"], "h_scaled": hser["h_scaled"], "e_scaled": es}
            rows.append({"n_max": n_max, "base": b, **s})
            h_pool.extend(s["h_scaled"])
            e_pool.extend(s["e_scaled"])

    a, b, rmse_pool = fit_affine_global(h_pool, e_pool)
    c_pool = hx.corr([a * x + b for x in h_pool], e_pool)

    per_row = []
    max_abs_global = 0.0
    for row in rows:
        fit = [a * v + b for v in row["h_scaled"]]
        st = residual_stats(row["e_scaled"], fit)
        max_abs_global = max(max_abs_global, st["max_abs"])
        per_row.append(
            {
                "n_max": row["n_max"],
                "base": row["base"],
                "corr_h_vs_e": hx.corr(row["h_scaled"], row["e_scaled"]),
                "corr_fit_vs_e": hx.corr(fit, row["e_scaled"]),
                **st,
            }
        )

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_values": sorted(n_values),
            "x_step": args.x_step,
            "weights": WEIGHTS,
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
            "max_zeta_zeros": len(zeros),
            "streaming": bool(args.streaming),
            "chunk_n": args.chunk_n if args.streaming else 0,
            "jobs": args.jobs if args.streaming else 1,
        },
        "global_fit": {
            "slope": a,
            "intercept": b,
            "pool_rmse": rmse_pool,
            "pool_corr_fit_vs_e": c_pool,
            "global_max_abs_residual": max_abs_global,
        },
        "rows": per_row,
        "lemma_candidate": {
            "statement": "For tested ranges, E_s(x) = a*H_s(x) + b + R(x) with |R(x)| <= C_emp.",
            "a": a,
            "b": b,
            "C_emp": max_abs_global,
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# H(x) Transfer Lemma Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        gf = report["global_fit"]
        f.write(f"- global slope a: {gf['slope']:.6f}\n")
        f.write(f"- global intercept b: {gf['intercept']:.6f}\n")
        f.write(f"- pooled RMSE: {gf['pool_rmse']:.6f}\n")
        f.write(f"- pooled corr(fit,E): {gf['pool_corr_fit_vs_e']:.6f}\n")
        f.write(f"- empirical residual envelope C_emp: {gf['global_max_abs_residual']:.6f}\n\n")
        f.write("## Per Base / Scale Residuals\n\n")
        for r in sorted(report["rows"], key=lambda z: (z["n_max"], z["base"])):
            f.write(
                f"- n_max={r['n_max']} base={r['base']} corr={r['corr_h_vs_e']:.6f} "
                f"corr_fit={r['corr_fit_vs_e']:.6f} rmse={r['rmse']:.6f} max_abs={r['max_abs']:.6f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
