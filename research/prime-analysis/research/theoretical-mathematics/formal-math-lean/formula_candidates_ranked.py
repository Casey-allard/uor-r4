#!/usr/bin/env python3
"""Rank compact candidate formulas across families and N ranges.

Goals:
- sparse symbolic models for holdout accuracy
- dimensionless invariant discovery with stability metrics
- cross-family consistency checks for transfer robustness
"""

from __future__ import annotations

import argparse
import itertools
import json
import math
import os
import statistics
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import ExperimentConfig, run_experiment, sieve_primes
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


def parse_moduli_sets(text: str) -> List[List[int]]:
    sets = []
    for chunk in text.split(";"):
        chunk = chunk.strip()
        if not chunk:
            continue
        sets.append([int(x.strip()) for x in chunk.split(",") if x.strip()])
    return sets


def collect_rows(n_values: Sequence[int], moduli_sets: Sequence[Sequence[int]], radius_power: float) -> List[Dict[str, object]]:
    rows: List[Dict[str, object]] = []
    for mset in moduli_sets:
        label = "-".join(str(x) for x in mset)
        for n in n_values:
            rep = run_experiment(ExperimentConfig(n_max=n, moduli=list(mset), radius_power=radius_power, bins=36))
            is_prime = sieve_primes(n)
            primes = [x for x in range(2, n + 1) if is_prime[x]]
            comps = [x for x in range(4, n + 1) if not is_prime[x]]
            k = min(len(primes), len(comps))
            psub = primes[:k]
            csub = comps[:k]

            wob_p = phase_wobble_metrics(psub, mset, radius_power)
            wob_c = phase_wobble_metrics(csub, mset, radius_power)
            h_p = harmonic_moments(psub, mset, radius_power, max_l=6)
            h_c = harmonic_moments(csub, mset, radius_power, max_l=6)
            hold = holdout_accuracy(n, is_prime, mset, radius_power)

            row = {
                "family": label,
                "n_max": n,
                "holdout": hold["holdout_accuracy"],
                "gap": hold["generalization_gap"],
                "sep": rep["metrics"]["separation_ratio"],
                "ent": rep["metrics"]["entropy_delta"],
                "wob": wob_p["phase_jerk_std"] - wob_c["phase_jerk_std"],
                "h1": h_p["h1"] - h_c["h1"],
                "h2": h_p["h2"] - h_c["h2"],
                "h3": h_p["h3"] - h_c["h3"],
                "h4": h_p["h4"] - h_c["h4"],
            }
            row["inv1"] = row["h3"] / (abs(row["ent"]) + 1e-12)
            row["inv2"] = row["wob"] / (abs(row["ent"]) + 1e-12)
            row["inv3"] = row["h1"] / (abs(row["wob"]) + 1e-12)
            row["inv4"] = row["sep"] / (abs(row["ent"]) + 1e-12)
            row["inv5"] = (row["h1"] + row["h3"]) / (2.0 * abs(row["ent"]) + 1e-12)
            rows.append(row)
    return rows


def evaluate_sparse_models(rows: Sequence[Dict[str, object]], feature_names: Sequence[str], max_terms: int = 3) -> List[Dict[str, object]]:
    families = sorted({str(r["family"]) for r in rows})
    ranked = []

    for k in range(1, max_terms + 1):
        for combo in itertools.combinations(feature_names, k):
            combo = list(combo)
            # global fit
            x = [[1.0] + [float(r[c]) for c in combo] for r in rows]
            y = [float(r["holdout"]) for r in rows]
            beta = linear_regression(x, y)
            pred = [predict(beta, xx) for xx in x]
            r2_global = r2_score(y, pred)

            # leave-one-family-out consistency
            r2_transfers = []
            for fam in families:
                train = [r for r in rows if str(r["family"]) != fam]
                test = [r for r in rows if str(r["family"]) == fam]
                if len(train) < len(combo) + 2 or len(test) < 2:
                    continue
                xtr = [[1.0] + [float(r[c]) for c in combo] for r in train]
                ytr = [float(r["holdout"]) for r in train]
                b = linear_regression(xtr, ytr)
                xte = [[1.0] + [float(r[c]) for c in combo] for r in test]
                yte = [float(r["holdout"]) for r in test]
                yhat = [predict(b, xx) for xx in xte]
                r2_transfers.append(r2_score(yte, yhat))

            if not r2_transfers:
                continue

            transfer_mean = statistics.fmean(r2_transfers)
            transfer_min = min(r2_transfers)
            complexity_penalty = 0.02 * len(combo)
            score = transfer_mean + 0.25 * transfer_min + 0.25 * r2_global - complexity_penalty

            ranked.append(
                {
                    "features": combo,
                    "global_r2": r2_global,
                    "transfer_r2_mean": transfer_mean,
                    "transfer_r2_min": transfer_min,
                    "score": score,
                    "coefficients": beta,
                }
            )

    ranked.sort(key=lambda r: r["score"], reverse=True)
    return ranked


def evaluate_invariants(rows: Sequence[Dict[str, object]], names: Sequence[str]) -> List[Dict[str, object]]:
    families = sorted({str(r["family"]) for r in rows})
    out = []
    for name in names:
        values = [float(r[name]) for r in rows]
        mean_all = statistics.fmean(values)
        std_all = statistics.pstdev(values)
        cv = std_all / (abs(mean_all) + 1e-12)
        signs = [1 if v > 0 else (-1 if v < 0 else 0) for v in values]
        sign_consistency = sum(1 for s in signs if s == signs[0]) / max(1, len(signs))

        family_means = []
        for fam in families:
            vv = [float(r[name]) for r in rows if str(r["family"]) == fam]
            family_means.append(statistics.fmean(vv))
        family_disp = statistics.pstdev(family_means) / (abs(statistics.fmean(family_means)) + 1e-12)

        stability = (1.0 / (1.0 + cv)) * (1.0 / (1.0 + family_disp)) * sign_consistency
        out.append(
            {
                "name": name,
                "mean": mean_all,
                "cv": cv,
                "family_dispersion": family_disp,
                "sign_consistency": sign_consistency,
                "stability_score": stability,
            }
        )
    out.sort(key=lambda r: r["stability_score"], reverse=True)
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description="Rank formula candidates across families and N")
    parser.add_argument("--n-values", type=str, default="20000,30000,40000,60000,80000,100000")
    parser.add_argument("--moduli-sets", type=str, default="5,7,11,13,17;5,7,11,13,19")
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--max-terms", type=int, default=3)
    parser.add_argument("--output", type=str, default="research/output/formula_candidates_ranked.json")
    args = parser.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    moduli_sets = parse_moduli_sets(args.moduli_sets)

    rows = collect_rows(n_values, moduli_sets, args.radius_power)

    base_features = ["sep", "ent", "wob", "h1", "h2", "h3", "h4", "inv1", "inv2", "inv3", "inv4", "inv5"]
    sparse = evaluate_sparse_models(rows, base_features, max_terms=max(1, args.max_terms))
    invariants = evaluate_invariants(rows, ["ent", "wob", "h1", "h2", "h3", "h4", "inv1", "inv2", "inv3", "inv4", "inv5"])

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_values": n_values,
            "moduli_sets": moduli_sets,
            "radius_power": args.radius_power,
            "max_terms": args.max_terms,
        },
        "series": rows,
        "top_sparse_models": sparse[:20],
        "top_invariants": invariants[:20],
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Ranked Formula Candidates\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")

        f.write("## Top Sparse Models (Cross-Family)\n\n")
        for i, m in enumerate(report["top_sparse_models"][:8], start=1):
            feats = m["features"]
            coefs = m["coefficients"]
            terms = [f"{coefs[0]:+.6f}"]
            for c, name in zip(coefs[1:], feats):
                terms.append(f"{c:+.6f}*{name}")
            f.write(
                f"{i}. holdout ~= {' '.join(terms)}\n"
                f"   score={m['score']:.4f}, global_r2={m['global_r2']:.4f}, transfer_mean={m['transfer_r2_mean']:.4f}, transfer_min={m['transfer_r2_min']:.4f}\n\n"
            )

        f.write("## Top Dimensionless Invariants\n\n")
        for i, inv in enumerate(report["top_invariants"][:8], start=1):
            f.write(
                f"{i}. {inv['name']}: mean={inv['mean']:+.6f}, cv={inv['cv']:.4f}, family_disp={inv['family_dispersion']:.4f}, "
                f"sign_consistency={inv['sign_consistency']:.3f}, stability={inv['stability_score']:.4f}\n"
            )

        f.write("\n## Overnight Validation Plan Inputs\n\n")
        f.write("- Validate top 3 sparse models across larger N and additional modulus families.\n")
        f.write("- Track only top 3 invariants with strongest stability scores.\n")
        f.write("- Require transfer_min R^2 to remain positive across families before promoting conjectures.\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if sparse:
        print("best_model_features:", sparse[0]["features"])
        print("best_model_score:", round(sparse[0]["score"], 6))
    if invariants:
        print("best_invariant:", invariants[0]["name"], round(invariants[0]["stability_score"], 6))


if __name__ == "__main__":
    main()
