#!/usr/bin/env python3
"""Model tau(M) tails with closed-form candidates for asymptotic uplift."""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import List, Sequence

import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


def tail_fp(zeros_ref: Sequence[float], m: int, kernel: str, scale: float) -> float:
    t = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        t += w * (g / den)
    return t


def fit_linear(x: Sequence[float], y: Sequence[float]):
    n = len(x)
    if n < 2:
        return 0.0, 0.0
    sx = sum(x)
    sy = sum(y)
    sxx = sum(v * v for v in x)
    sxy = sum(a * b for a, b in zip(x, y))
    det = n * sxx - sx * sx
    if abs(det) < 1e-15:
        return 0.0, 0.0
    a = (n * sxy - sx * sy) / det
    b = (sy * sxx - sx * sxy) / det
    return a, b


def corr(x: Sequence[float], y: Sequence[float]) -> float:
    if len(x) != len(y) or not x:
        return 0.0
    mx = sum(x) / len(x)
    my = sum(y) / len(y)
    vx = sum((u - mx) ** 2 for u in x)
    vy = sum((v - my) ** 2 for v in y)
    if vx <= 1e-15 or vy <= 1e-15:
        return 0.0
    cov = sum((u - mx) * (v - my) for u, v in zip(x, y))
    return cov / math.sqrt(vx * vy)


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 tau(M) model probe")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--ref-zero-limit", type=int, default=512)
    ap.add_argument("--m-grid", type=str, default="64,96,128,160,192,224,256,320,384,448")
    ap.add_argument("--zero-kernel", type=str, default="gaussian", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=100.0)
    ap.add_argument("--output", type=str, default="research/output/a2_tau_model_probe.json")
    args = ap.parse_args()

    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.ref_zero_limit > len(zeros_all):
        raise ValueError("ref-zero-limit exceeds available zeros")
    zeros_ref = zeros_all[: args.ref_zero_limit]
    m_grid = [int(x.strip()) for x in args.m_grid.split(",") if x.strip()]
    m_grid = [m for m in m_grid if 0 < m < args.ref_zero_limit]
    if len(m_grid) < 3:
        raise ValueError("need at least 3 M points")

    rows = []
    for m in m_grid:
        tau = tail_fp(zeros_ref, m, args.zero_kernel, args.kernel_scale)
        gamma_m = zeros_ref[m - 1]
        rows.append({"M": m, "gamma_M": gamma_m, "tau_fp": tau})

    gam = [r["gamma_M"] for r in rows]
    tau = [max(1e-300, r["tau_fp"]) for r in rows]
    log_tau = [math.log(t) for t in tau]

    # Model 1 (gaussian-tail-like): log tau = c0 + c1 * gamma^2
    x1 = [g * g for g in gam]
    c1, c0 = fit_linear(x1, log_tau)
    pred1 = [math.exp(c0 + c1 * xx) for xx in x1]
    rel1 = [abs(p - t) / max(1e-300, t) for p, t in zip(pred1, tau)]
    m1 = {
        "name": "exp_quad",
        "form": "tau(M) ~= exp(c0 + c1*gamma_M^2)",
        "c0": c0,
        "c1": c1,
        "corr_log": corr(x1, log_tau),
        "max_rel_err": max(rel1),
        "mean_rel_err": sum(rel1) / len(rel1),
    }

    # Model 2 (subexp): log tau = d0 + d1 * gamma
    x2 = gam
    d1, d0 = fit_linear(x2, log_tau)
    pred2 = [math.exp(d0 + d1 * xx) for xx in x2]
    rel2 = [abs(p - t) / max(1e-300, t) for p, t in zip(pred2, tau)]
    m2 = {
        "name": "exp_lin",
        "form": "tau(M) ~= exp(d0 + d1*gamma_M)",
        "d0": d0,
        "d1": d1,
        "corr_log": corr(x2, log_tau),
        "max_rel_err": max(rel2),
        "mean_rel_err": sum(rel2) / len(rel2),
    }

    best = m1 if m1["mean_rel_err"] <= m2["mean_rel_err"] else m2
    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "ref_zero_limit": args.ref_zero_limit,
            "m_grid": m_grid,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
        },
        "rows": rows,
        "models": [m1, m2],
        "selected_model": best,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# A2 Tau Model Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Kernel: {args.zero_kernel}, scale={args.kernel_scale}\n\n")
        for m in report["models"]:
            f.write(
                f"- {m['name']}: corr_log={m['corr_log']:.6f} "
                f"mean_rel_err={m['mean_rel_err']:.6f} max_rel_err={m['max_rel_err']:.6f}\n"
            )
        f.write("\nSelected model:\n\n")
        f.write(f"- {best['name']}: {best['form']}\n")
        for k, v in best.items():
            if k in {"name", "form"}:
                continue
            f.write(f"- {k}: {v}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()

