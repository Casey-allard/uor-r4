#!/usr/bin/env python3
"""Explicit formula probe for Chebyshev psi(x).

Compares exact psi(x) from von Mangoldt sums with truncated zeta-zero
explicit-formula reconstruction, then analyzes residuals.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

from prime_geometry_loop import ExperimentConfig, run_experiment, sieve_primes
from stage2_prime_program import harmonic_moments, phase_wobble_metrics


def load_zeros(path: str, limit: int) -> List[float]:
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    if isinstance(data, dict):
        if "zeros" in data:
            z = [float(v) for v in data["zeros"]]
        elif "positive_zeros" in data:
            z = [float(v) for v in data["positive_zeros"]]
        else:
            z = []
    elif isinstance(data, list):
        z = [float(v) for v in data]
    else:
        z = []
    z = sorted(v for v in z if v > 0)
    return z[:limit] if limit > 0 else z


def list_primes(n: int) -> List[int]:
    is_prime = [False, False] + [True] * (n - 1)
    for p in range(2, int(n**0.5) + 1):
        if is_prime[p]:
            is_prime[p * p : n + 1 : p] = [False] * (((n - p * p) // p) + 1)
    return [i for i in range(2, n + 1) if is_prime[i]]


def psi_exact_table(n_max: int) -> List[float]:
    primes = list_primes(n_max)
    psi = [0.0] * (n_max + 1)
    for p in primes:
        lp = math.log(p)
        pk = p
        while pk <= n_max:
            psi[pk] += lp
            if pk > n_max // p:
                break
            pk *= p
    # prefix sums
    for x in range(1, n_max + 1):
        psi[x] += psi[x - 1]
    return psi


def psi_estimate(x: float, zeros: Sequence[float]) -> float:
    if x <= 1.0:
        return 0.0
    lx = math.log(x)
    rootx = math.sqrt(x)

    total = 0.0
    for g in zeros:
        den = 0.25 + g * g
        # 2*Re(x^rho/rho), rho=1/2+ig
        term = 2.0 * rootx * ((0.5 * math.cos(g * lx) + g * math.sin(g * lx)) / den)
        total += term

    # simplified explicit formula for psi(x)
    return x - total - math.log(2.0 * math.pi)


def corr(a: Sequence[float], b: Sequence[float]) -> float:
    if len(a) != len(b) or not a:
        return 0.0
    ma = sum(a) / len(a)
    mb = sum(b) / len(b)
    va = sum((x - ma) ** 2 for x in a)
    vb = sum((x - mb) ** 2 for x in b)
    if va < 1e-12 or vb < 1e-12:
        return 0.0
    cov = sum((x - ma) * (y - mb) for x, y in zip(a, b))
    return cov / math.sqrt(va * vb)


def invariants_for_n(n: int, moduli: Sequence[int], radius_power: float) -> Dict[str, float]:
    rep = run_experiment(
        ExperimentConfig(
            n_max=n,
            moduli=list(moduli),
            radius_power=radius_power,
            bins=36,
            control_trials=0,
        )
    )
    is_prime = sieve_primes(n)
    primes = [x for x in range(2, n + 1) if is_prime[x]]
    comps = [x for x in range(4, n + 1) if not is_prime[x]]
    k = min(len(primes), len(comps))
    psub = primes[:k]
    csub = comps[:k]
    wob_p = phase_wobble_metrics(psub, moduli, radius_power)
    wob_c = phase_wobble_metrics(csub, moduli, radius_power)
    h_p = harmonic_moments(psub, moduli, radius_power, max_l=6)
    h_c = harmonic_moments(csub, moduli, radius_power, max_l=6)
    ent = rep["metrics"]["entropy_delta"]
    wob = wob_p["phase_jerk_std"] - wob_c["phase_jerk_std"]
    h1 = h_p["h1"] - h_c["h1"]
    h3 = h_p["h3"] - h_c["h3"]
    return {
        "ent": ent,
        "wob": wob,
        "h1": h1,
        "h3": h3,
        "inv2": wob / (abs(ent) + 1e-12),
    }


def write_svg(xs: Sequence[int], exact: Sequence[float], est: Sequence[float], out_path: str) -> None:
    if not xs:
        return
    w, h = 980, 500
    pad = 55
    x0, x1 = xs[0], xs[-1]
    y0 = min(min(exact), min(est))
    y1 = max(max(exact), max(est))

    def px(x: float) -> float:
        return pad + (x - x0) / max(1e-12, (x1 - x0)) * (w - 2 * pad)

    def py(y: float) -> float:
        return h - pad - (y - y0) / max(1e-12, (y1 - y0)) * (h - 2 * pad)

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n')
        f.write(f'<rect x="0" y="0" width="{w}" height="{h}" fill="#fafaf8"/>\n')
        f.write('<text x="20" y="28" font-size="18" fill="#111827">psi(x): exact vs explicit-formula estimate</text>\n')
        f.write(f'<line x1="{pad}" y1="{h-pad}" x2="{w-pad}" y2="{h-pad}" stroke="#9ca3af"/>\n')

        p1 = " ".join(f"{px(x):.2f},{py(y):.2f}" for x, y in zip(xs, exact))
        p2 = " ".join(f"{px(x):.2f},{py(y):.2f}" for x, y in zip(xs, est))
        f.write(f'<polyline fill="none" stroke="#1f77b4" stroke-width="2" points="{p1}"/>\n')
        f.write(f'<polyline fill="none" stroke="#d97706" stroke-width="2" points="{p2}"/>\n')
        f.write('<text x="20" y="50" font-size="12" fill="#1f77b4">exact psi(x)</text>\n')
        f.write('<text x="20" y="66" font-size="12" fill="#d97706">truncated explicit formula</text>\n')
        f.write("</svg>\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="Explicit-formula reconstruction probe")
    parser.add_argument("--zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    parser.add_argument("--zero-limit", type=int, default=128)
    parser.add_argument("--x-max", type=int, default=100000)
    parser.add_argument("--x-step", type=int, default=500)
    parser.add_argument("--link-n-values", type=str, default="20000,30000,40000,60000,80000,100000")
    parser.add_argument("--moduli", type=str, default="5,7,11,13,19")
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--output", type=str, default="research/output/explicit_formula_probe.json")
    args = parser.parse_args()

    zeros = load_zeros(args.zeros_file, args.zero_limit)
    psi_tab = psi_exact_table(args.x_max)

    xs = list(range(max(2, args.x_step), args.x_max + 1, args.x_step))
    exact = [psi_tab[x] for x in xs]
    est = [psi_estimate(float(x), zeros) for x in xs]
    residual = [a - b for a, b in zip(exact, est)]

    rmse = math.sqrt(sum(r * r for r in residual) / max(1, len(residual)))
    mae = sum(abs(r) for r in residual) / max(1, len(residual))
    corr_val = corr(exact, est)

    # Normalize residual by sqrt(x), an explicit-formula natural scale.
    scaled = [residual[i] / math.sqrt(xs[i]) for i in range(len(xs))]
    scaled_mean = sum(scaled) / max(1, len(scaled))
    scaled_std = math.sqrt(sum((v - scaled_mean) ** 2 for v in scaled) / max(1, len(scaled)))

    # Residual-vs-invariant linkage across N windows.
    n_values = [int(v) for v in args.link_n_values.split(",") if v.strip()]
    moduli = [int(v) for v in args.moduli.split(",") if v.strip()]
    link_rows = []
    for n in n_values:
        if n > args.x_max:
            continue
        win = [residual[i] / math.sqrt(xs[i]) for i in range(len(xs)) if xs[i] <= n]
        if len(win) < 4:
            continue
        res_std = math.sqrt(sum((v - (sum(win) / len(win))) ** 2 for v in win) / len(win))
        inv = invariants_for_n(n, moduli, args.radius_power)
        row = {"n_max": n, "residual_std_scaled": res_std, **inv}
        link_rows.append(row)

    link_corr = {}
    if len(link_rows) >= 3:
        y = [r["residual_std_scaled"] for r in link_rows]
        for k in ["ent", "wob", "h1", "h3", "inv2"]:
            link_corr[k] = corr(y, [r[k] for r in link_rows])

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeros_file": args.zeros_file,
            "zero_limit": len(zeros),
            "x_max": args.x_max,
            "x_step": args.x_step,
        },
        "fit": {
            "rmse": rmse,
            "mae": mae,
            "corr_exact_vs_estimate": corr_val,
            "scaled_residual_mean": scaled_mean,
            "scaled_residual_std": scaled_std,
        },
        "residual_invariant_link": {
            "n_values": [r["n_max"] for r in link_rows],
            "rows": link_rows,
            "corr_residual_std_with_feature": link_corr,
        },
        "series": {
            "x": xs,
            "psi_exact": exact,
            "psi_estimate": est,
            "residual": residual,
            "residual_over_sqrtx": scaled,
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    svg = args.output.replace(".json", ".svg")
    write_svg(xs, exact, est, svg)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Explicit Formula Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- zeros used: {len(zeros)}\n")
        f.write(f"- RMSE: {rmse:.6f}\n")
        f.write(f"- MAE: {mae:.6f}\n")
        f.write(f"- corr(exact, estimate): {corr_val:.6f}\n")
        f.write(f"- mean(residual/sqrt(x)): {scaled_mean:.6f}\n")
        f.write(f"- std(residual/sqrt(x)): {scaled_std:.6f}\n")
        if link_rows:
            f.write("\n## Residual-Invariant Link\n\n")
            for k, v in link_corr.items():
                f.write(f"- corr(residual_std_scaled, {k}) = {v:+.6f}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print(f"wrote: {svg}")
    print(f"rmse: {rmse:.6f}")
    print(f"corr: {corr_val:.6f}")


if __name__ == "__main__":
    main()
