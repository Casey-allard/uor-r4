#!/usr/bin/env python3
"""Probe candidate formulas from prime-geometry experiments.

Outputs:
- explicit linear discriminant score S(n)=w·v(n)+b
- fitted asymptotic laws y(N)=L+C*N^{-alpha}
- simple predictive meta-formula for holdout accuracy from geometric features
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import ExperimentConfig, embed_4d, mean_vec, run_experiment, sieve_primes
from stage2_prime_program import harmonic_moments, holdout_accuracy, phase_wobble_metrics


def mat_solve(a: List[List[float]], b: List[float]) -> List[float]:
    n = len(a)
    aug = [row[:] + [b[i]] for i, row in enumerate(a)]
    for i in range(n):
        piv = max(range(i, n), key=lambda r: abs(aug[r][i]))
        aug[i], aug[piv] = aug[piv], aug[i]
        den = aug[i][i]
        if abs(den) < 1e-12:
            den = 1e-12
        for j in range(i, n + 1):
            aug[i][j] /= den
        for r in range(n):
            if r == i:
                continue
            fac = aug[r][i]
            for j in range(i, n + 1):
                aug[r][j] -= fac * aug[i][j]
    return [aug[i][n] for i in range(n)]


def linear_regression(x_rows: List[List[float]], y: List[float], ridge: float = 1e-8) -> List[float]:
    p = len(x_rows[0])
    xtx = [[0.0] * p for _ in range(p)]
    xty = [0.0] * p
    for x, yy in zip(x_rows, y):
        for i in range(p):
            xty[i] += x[i] * yy
            for j in range(p):
                xtx[i][j] += x[i] * x[j]
    for i in range(p):
        xtx[i][i] += ridge
    return mat_solve(xtx, xty)


def predict(beta: Sequence[float], x: Sequence[float]) -> float:
    return sum(beta[i] * x[i] for i in range(len(beta)))


def r2_score(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    mu = sum(y_true) / max(1, len(y_true))
    ss_tot = sum((v - mu) ** 2 for v in y_true)
    ss_res = sum((a - b) ** 2 for a, b in zip(y_true, y_pred))
    if ss_tot < 1e-12:
        return 0.0
    return 1.0 - ss_res / ss_tot


def fit_asymptotic(n_vals: Sequence[int], y_vals: Sequence[float]) -> Dict[str, float]:
    best = None
    for step in range(4, 61):
        alpha = step / 20.0  # 0.2 .. 3.0
        x = [n ** (-alpha) for n in n_vals]
        s1 = len(x)
        sx = sum(x)
        sxx = sum(v * v for v in x)
        sy = sum(y_vals)
        sxy = sum(xx * yy for xx, yy in zip(x, y_vals))
        det = s1 * sxx - sx * sx
        if abs(det) < 1e-12:
            continue
        l = (sy * sxx - sx * sxy) / det
        c = (s1 * sxy - sx * sy) / det
        pred = [l + c * xx for xx in x]
        mse = sum((a - b) ** 2 for a, b in zip(y_vals, pred)) / len(y_vals)
        if best is None or mse < best["mse"]:
            best = {"alpha": alpha, "L": l, "C": c, "mse": mse, "r2": r2_score(y_vals, pred)}
    return best if best else {"alpha": 1.0, "L": 0.0, "C": 0.0, "mse": 0.0, "r2": 0.0}


def discriminant_formula(n_max: int, moduli: Sequence[int], radius_power: float) -> Dict[str, object]:
    is_prime = sieve_primes(n_max)
    split = n_max // 2
    train_primes = [n for n in range(2, split + 1) if is_prime[n]]
    train_comp = [n for n in range(4, split + 1) if not is_prime[n]]
    k = min(len(train_primes), len(train_comp))

    pv = [embed_4d(n, moduli, radius_power) for n in train_primes[:k]]
    cv = [embed_4d(n, moduli, radius_power) for n in train_comp[:k]]
    cp = mean_vec(pv)
    cc = mean_vec(cv)

    # score(v) = ||v-cc||^2 - ||v-cp||^2 = w·v + b
    w = [2.0 * (cp[i] - cc[i]) for i in range(4)]
    b = sum(cc[i] * cc[i] - cp[i] * cp[i] for i in range(4))

    return {
        "split": split,
        "coefficients_w": w,
        "bias_b": b,
        "formula": "S(n)=w1*x+w2*y+w3*z+w4*t+b, prime-like if S(n)>0",
        "embedding": "(x,y,z,t)=embed_4d(n,moduli,radius_power)",
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Probe candidate formulas for prime-geometry patterns")
    parser.add_argument("--n-values", type=str, default="20000,30000,40000,60000,80000,100000,120000")
    parser.add_argument("--moduli", type=str, default="5,7,11,13,19")
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--output", type=str, default="research/output/formula_probe_main.json")
    args = parser.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    moduli = [int(x) for x in args.moduli.split(",") if x.strip()]

    rows = []
    for n in n_values:
        cfg = ExperimentConfig(n_max=n, moduli=moduli, radius_power=args.radius_power, bins=36, control_trials=0)
        rep = run_experiment(cfg)
        is_prime = sieve_primes(n)
        primes = [x for x in range(2, n + 1) if is_prime[x]]
        comps = [x for x in range(4, n + 1) if not is_prime[x]]
        k = min(len(primes), len(comps))
        psub = primes[:k]
        csub = comps[:k]

        wob_p = phase_wobble_metrics(psub, moduli, args.radius_power)
        wob_c = phase_wobble_metrics(csub, moduli, args.radius_power)
        h_p = harmonic_moments(psub, moduli, args.radius_power, max_l=6)
        h_c = harmonic_moments(csub, moduli, args.radius_power, max_l=6)
        hold = holdout_accuracy(n, is_prime, moduli, args.radius_power)

        row = {
            "n_max": n,
            "separation_ratio": rep["metrics"]["separation_ratio"],
            "entropy_delta": rep["metrics"]["entropy_delta"],
            "holdout_accuracy": hold["holdout_accuracy"],
            "generalization_gap": hold["generalization_gap"],
            "phase_jerk_delta": wob_p["phase_jerk_std"] - wob_c["phase_jerk_std"],
            "delta_h1": h_p["h1"] - h_c["h1"],
            "delta_h2": h_p["h2"] - h_c["h2"],
            "delta_h3": h_p["h3"] - h_c["h3"],
            "delta_h4": h_p["h4"] - h_c["h4"],
        }
        rows.append(row)

    rows.sort(key=lambda r: r["n_max"])

    # Meta-formula for holdout accuracy
    x = []
    y = []
    for r in rows:
        x.append([1.0, r["separation_ratio"], r["entropy_delta"], r["phase_jerk_delta"], r["delta_h1"], r["delta_h3"]])
        y.append(r["holdout_accuracy"])
    beta = linear_regression(x, y)
    yhat = [predict(beta, xx) for xx in x]

    # Asymptotic fits
    ns = [r["n_max"] for r in rows]
    fits = {}
    for key in ["separation_ratio", "entropy_delta", "phase_jerk_delta", "delta_h1", "delta_h3", "holdout_accuracy"]:
        fits[key] = fit_asymptotic(ns, [r[key] for r in rows])

    disc = discriminant_formula(max(n_values), moduli, args.radius_power)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {"n_values": n_values, "moduli": moduli, "radius_power": args.radius_power},
        "series": rows,
        "meta_formula": {
            "features": ["1", "separation_ratio", "entropy_delta", "phase_jerk_delta", "delta_h1", "delta_h3"],
            "coefficients": beta,
            "r2": r2_score(y, yhat),
            "formula": "holdout ~= b0 + b1*sep + b2*entropy_delta + b3*phase_jerk_delta + b4*delta_h1 + b5*delta_h3",
        },
        "asymptotic_fits": fits,
        "discriminant_formula": disc,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Formula Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write("## Candidate Meta-Formula\n\n")
        b = report["meta_formula"]["coefficients"]
        f.write(
            f"holdout ~= {b[0]:+.6f} {b[1]:+.6f}*sep {b[2]:+.6f}*entropy_delta "
            f"{b[3]:+.6f}*phase_jerk_delta {b[4]:+.6f}*delta_h1 {b[5]:+.6f}*delta_h3\n\n"
        )
        f.write(f"R^2={report['meta_formula']['r2']:.6f}\n\n")

        f.write("## Asymptotic Fits\n\n")
        for k, v in report["asymptotic_fits"].items():
            f.write(
                f"- {k}(N) ~= {v['L']:+.6f} {v['C']:+.6f}*N^(-{v['alpha']:.2f}) "
                f"(R^2={v['r2']:.4f})\n"
            )

        f.write("\n## Discriminant Score\n\n")
        w = report["discriminant_formula"]["coefficients_w"]
        bb = report["discriminant_formula"]["bias_b"]
        f.write(f"S(n) = {w[0]:+.6f}*x + {w[1]:+.6f}*y + {w[2]:+.6f}*z + {w[3]:+.6f}*t + {bb:+.6f}\n")
        f.write("Prime-like if S(n) > 0, where (x,y,z,t)=embed_4d(n,moduli,radius_power).\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("meta_formula:")
    print(report["meta_formula"]["formula"])
    print("coefficients:", [round(v, 6) for v in beta])


if __name__ == "__main__":
    main()
