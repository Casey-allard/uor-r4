#!/usr/bin/env python3
"""Merged modular+wheel+local-square model with null controls."""

from __future__ import annotations

import argparse
import json
import math
import os
import random
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import (
    embed_4d,
    load_zeta_zeros_file,
    sieve_primes,
    zeta_alignment,
    zeta_permutation_control,
)
from divisor_wheel_probe import divisor_count_sieve


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


def linear_regression(x_rows: Sequence[Sequence[float]], y: Sequence[float], ridge: float = 1e-8) -> List[float]:
    p = len(x_rows[0])
    xtx = [[0.0] * p for _ in range(p)]
    xty = [0.0] * p
    for x, yy in zip(x_rows, y):
        for i in range(p):
            xi = x[i]
            xty[i] += xi * yy
            row_i = xtx[i]
            for j in range(p):
                row_i[j] += xi * x[j]
    for i in range(p):
        xtx[i][i] += ridge
    return mat_solve(xtx, xty)


def predict(beta: Sequence[float], x: Sequence[float]) -> float:
    return sum(beta[i] * x[i] for i in range(len(beta)))


def r2_score(y_true: Sequence[float], y_pred: Sequence[float]) -> float:
    mu = sum(y_true) / max(1, len(y_true))
    ss_tot = sum((y - mu) ** 2 for y in y_true)
    ss_res = sum((a - b) ** 2 for a, b in zip(y_true, y_pred))
    if ss_tot < 1e-12:
        return 0.0
    return 1.0 - ss_res / ss_tot


def nearest_prime_square_meta(n: int, prime_squares: Sequence[int]) -> Tuple[float, float]:
    best_d = None
    for s in prime_squares:
        d = n - s
        if best_d is None or abs(d) < abs(best_d):
            best_d = d
    d = float(best_d or 0.0)
    side = 1.0 if d > 0 else (-1.0 if d < 0 else 0.0)
    logdist = math.log1p(abs(d))
    return side, logdist


def build_dataset(n_max: int, moduli: Sequence[int], radius_power: float, residue_mod: int) -> Dict[str, object]:
    is_prime = sieve_primes(n_max)
    tau = divisor_count_sieve(n_max + 4)
    primes_small = [p for p in range(2, int(math.sqrt(n_max)) + 1) if is_prime[p]]
    prime_squares = [p * p for p in primes_small if p * p <= n_max]
    if len(prime_squares) < 2:
        prime_squares = [4, 9, 25]

    X = []
    y = []
    residues = []
    n_list = []

    for n in range(8, n_max - 4):
        x, yy, z, t = embed_4d(n, moduli, radius_power)
        r = math.sqrt(x * x + yy * yy + z * z + t * t)
        rn = max(1e-12, r)
        m1, m2, m3, m4 = x / rn, yy / rn, z / rn, t / rn

        d1m = math.log(tau[n - 1] + 1.0)
        d1p = math.log(tau[n + 1] + 1.0)
        d1avg = 0.5 * (d1m + d1p)
        d1asym = abs(d1m - d1p)
        d2avg = 0.5 * (math.log(tau[n - 2] + 1.0) + math.log(tau[n + 2] + 1.0))
        d3avg = 0.5 * (math.log(tau[n - 3] + 1.0) + math.log(tau[n + 3] + 1.0))

        lr = math.log1p(r)
        lr2, lr4, lr6 = lr * lr, lr**4, lr**6
        side, logdist = nearest_prime_square_meta(n, prime_squares)

        feat = [
            1.0,
            m1,
            m2,
            m3,
            m4,
            d1m,
            d1p,
            d1avg,
            d1asym,
            d2avg,
            d3avg,
            lr2,
            lr4,
            lr6,
            side,
            logdist,
            side * logdist,
            logdist * d1avg,
            logdist * m1,
            logdist * m3,
        ]

        X.append(feat)
        y.append(1.0 if is_prime[n] else 0.0)
        residues.append(n % residue_mod)
        n_list.append(n)

    return {
        "X": X,
        "y": y,
        "residues": residues,
        "n_list": n_list,
    }


def split_indices(n_list: Sequence[int], split_n: int) -> Tuple[List[int], List[int]]:
    tr, te = [], []
    for i, n in enumerate(n_list):
        if n <= split_n:
            tr.append(i)
        else:
            te.append(i)
    return tr, te


def eval_model(X, y, tr_idx, te_idx) -> Dict[str, float]:
    xtr = [X[i] for i in tr_idx]
    ytr = [y[i] for i in tr_idx]
    xte = [X[i] for i in te_idx]
    yte = [y[i] for i in te_idx]
    beta = linear_regression(xtr, ytr)
    pte = [predict(beta, v) for v in xte]
    ptr = [predict(beta, v) for v in xtr]
    acc = sum(1 for a, p in zip(yte, pte) if (p >= 0.5) == (a >= 0.5)) / max(1, len(yte))
    return {
        "train_r2": r2_score(ytr, ptr),
        "test_r2": r2_score(yte, pte),
        "test_acc": acc,
        "coef_count": len(beta),
        "beta": beta,
    }


def p_value_ge(obs: float, vals: Sequence[float]) -> float:
    if not vals:
        return 1.0
    ge = sum(1 for v in vals if v >= obs)
    return (ge + 1.0) / (len(vals) + 1.0)


def permutation_null(X, y, tr_idx, te_idx, trials: int, seed: int) -> Dict[str, object]:
    if trials <= 0:
        return {}
    rng = random.Random(seed)
    ytr_true = [y[i] for i in tr_idx]
    xtr = [X[i] for i in tr_idx]
    xte = [X[i] for i in te_idx]
    yte = [y[i] for i in te_idx]
    test_r2 = []
    for _ in range(trials):
        ytr = ytr_true[:]
        rng.shuffle(ytr)
        beta = linear_regression(xtr, ytr)
        pte = [predict(beta, v) for v in xte]
        test_r2.append(r2_score(yte, pte))
    return {"trials": trials, "seed": seed, "test_r2": test_r2}


def sample_labels_residue_preserving(idx: Sequence[int], residues: Sequence[int], y_true: Sequence[float], rng: random.Random) -> List[float]:
    by_r = {}
    for i in idx:
        r = residues[i]
        by_r.setdefault(r, []).append(i)

    target = {}
    for i in idx:
        if y_true[i] >= 0.5:
            r = residues[i]
            target[r] = target.get(r, 0) + 1

    y = [0.0 for _ in idx]
    idx_pos = {j: k for k, j in enumerate(idx)}
    for r, pool in by_r.items():
        k = min(len(pool), target.get(r, 0))
        pick = pool[:]
        rng.shuffle(pick)
        for j in pick[:k]:
            y[idx_pos[j]] = 1.0

    return y


def residue_preserving_null(X, y, residues, tr_idx, te_idx, trials: int, seed: int) -> Dict[str, object]:
    if trials <= 0:
        return {}
    rng = random.Random(seed)
    r2_vals = []
    acc_vals = []
    xtr = [X[i] for i in tr_idx]
    xte = [X[i] for i in te_idx]

    for _ in range(trials):
        ytr = sample_labels_residue_preserving(tr_idx, residues, y, rng)
        yte = sample_labels_residue_preserving(te_idx, residues, y, rng)
        beta = linear_regression(xtr, ytr)
        pte = [predict(beta, v) for v in xte]
        r2_vals.append(r2_score(yte, pte))
        acc = sum(1 for a, p in zip(yte, pte) if (p >= 0.5) == (a >= 0.5)) / max(1, len(yte))
        acc_vals.append(acc)

    return {"trials": trials, "seed": seed, "test_r2": r2_vals, "test_acc": acc_vals}


def maybe_subsample_indices(idx: Sequence[int], sample_size: int, rng: random.Random) -> List[int]:
    if sample_size <= 0 or sample_size >= len(idx):
        return list(idx)
    return rng.sample(list(idx), sample_size)


def run_family(
    n_max: int,
    moduli: Sequence[int],
    radius_power: float,
    residue_mod: int,
    null_trials: int,
    seed: int,
    zeta_zeros: Sequence[float],
    max_zeta_zeros: int,
    null_sample_size: int,
    zeta_perm_trials: int,
) -> Dict[str, object]:
    data = build_dataset(n_max, moduli, radius_power, residue_mod=residue_mod)
    X, y, residues, n_list = data["X"], data["y"], data["residues"], data["n_list"]
    split_n = n_max // 2
    tr_idx, te_idx = split_indices(n_list, split_n)
    rng = random.Random(seed + 13)
    tr_null = maybe_subsample_indices(tr_idx, null_sample_size, rng)
    te_null = maybe_subsample_indices(te_idx, null_sample_size, rng)

    obs = eval_model(X, y, tr_idx, te_idx)

    perm = permutation_null(X, y, tr_null, te_null, trials=null_trials, seed=seed)
    resn = residue_preserving_null(X, y, residues, tr_null, te_null, trials=null_trials, seed=seed + 97)

    is_prime = sieve_primes(n_max)
    prime_nums = [n for n in range(2, n_max + 1) if is_prime[n]]
    used_zeros = list(zeta_zeros[:max_zeta_zeros]) if max_zeta_zeros > 0 else list(zeta_zeros)
    prime_norms = []
    for n in prime_nums:
        xx, yy, zz, tt = embed_4d(n, moduli, radius_power)
        prime_norms.append(math.sqrt(xx * xx + yy * yy + zz * zz + tt * tt))
    zeta_obs = zeta_alignment(prime_norms, zeros_imag=used_zeros)
    zeta_ctrl = zeta_permutation_control(
        prime_norms,
        trials=max(0, zeta_perm_trials),
        seed=seed + 211,
        zeros_imag=used_zeros,
    )

    out = {
        "n_max": n_max,
        "moduli": list(moduli),
        "radius_power": radius_power,
        "split_n": split_n,
        "sample_size": len(X),
        "null_sample_train_size": len(tr_null),
        "null_sample_test_size": len(te_null),
        "observed": {k: obs[k] for k in ["train_r2", "test_r2", "test_acc", "coef_count"]},
        "formula": {
            "beta": obs["beta"],
            "rule": "prime-like if beta·x > 0.5 (regression score)",
        },
        "nulls": {
            "label_permutation": {
                "trials": perm.get("trials", 0),
                "p_value_ge_test_r2": p_value_ge(obs["test_r2"], perm.get("test_r2", [])),
                "mean_test_r2": (sum(perm.get("test_r2", [])) / max(1, len(perm.get("test_r2", [])))) if perm else 0.0,
            },
            "residue_preserving": {
                "trials": resn.get("trials", 0),
                "p_value_ge_test_r2": p_value_ge(obs["test_r2"], resn.get("test_r2", [])),
                "p_value_ge_test_acc": p_value_ge(obs["test_acc"], resn.get("test_acc", [])),
                "mean_test_r2": (sum(resn.get("test_r2", [])) / max(1, len(resn.get("test_r2", [])))) if resn else 0.0,
                "mean_test_acc": (sum(resn.get("test_acc", [])) / max(1, len(resn.get("test_acc", [])))) if resn else 0.0,
            },
        },
        "zeta_guardrail": {
            "zeros_used": len(used_zeros),
            "alignment_score": zeta_obs.get("score", 0.0),
            "alignment_p_value_ge": zeta_ctrl.get("p_value_ge", 1.0),
            "alignment_z_score": zeta_ctrl.get("z_score", 0.0),
            "note": "zeta zeros treated as guardrail tunnel",
        },
    }
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description="Merged square+wheel model with null controls")
    parser.add_argument("--n-max", type=int, default=300000)
    parser.add_argument("--moduli-families", type=str, default="5,7,11,13,19;6,30,210,2310,30030")
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--residue-mod", type=int, default=30)
    parser.add_argument("--null-trials", type=int, default=30)
    parser.add_argument("--null-sample-size", type=int, default=30000)
    parser.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    parser.add_argument("--max-zeta-zeros", type=int, default=64)
    parser.add_argument("--zeta-perm-trials", type=int, default=8)
    parser.add_argument("--seed", type=int, default=20260216)
    parser.add_argument("--output", type=str, default="research/output/merged_square_wheel_model.json")
    args = parser.parse_args()

    families = []
    for chunk in args.moduli_families.split(";"):
        chunk = chunk.strip()
        if not chunk:
            continue
        vals = [int(x.strip()) for x in chunk.split(",") if x.strip()]
        vals = [v for v in vals if v > 1]
        if vals:
            families.append(vals)
    zeta_zeros = load_zeta_zeros_file(args.zeta_zeros_file) if args.zeta_zeros_file else []
    results = []
    for i, fam in enumerate(families):
        results.append(
            run_family(
                n_max=args.n_max,
                moduli=fam,
                radius_power=args.radius_power,
                residue_mod=max(2, args.residue_mod),
                null_trials=max(0, args.null_trials),
                seed=args.seed + 1000 * i,
                zeta_zeros=zeta_zeros,
                max_zeta_zeros=max(0, args.max_zeta_zeros),
                null_sample_size=max(0, args.null_sample_size),
                zeta_perm_trials=max(0, args.zeta_perm_trials),
            )
        )

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "radius_power": args.radius_power,
            "residue_mod": args.residue_mod,
            "null_trials": args.null_trials,
            "null_sample_size": args.null_sample_size,
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": args.max_zeta_zeros,
            "zeta_perm_trials": args.zeta_perm_trials,
            "families": families,
        },
        "results": results,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Merged Square-Wheel Model\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for r in results:
            label = "-".join(str(x) for x in r["moduli"])
            obs = r["observed"]
            lp = r["nulls"]["label_permutation"]
            rp = r["nulls"]["residue_preserving"]
            zg = r["zeta_guardrail"]
            f.write(f"## Family {label}\n\n")
            f.write(f"- test_R2: {obs['test_r2']:.6f}\n")
            f.write(f"- test_accuracy: {obs['test_acc']:.6f}\n")
            f.write(f"- permutation p(test_R2): {lp['p_value_ge_test_r2']:.6f}\n")
            f.write(f"- residue-preserving p(test_R2): {rp['p_value_ge_test_r2']:.6f}\n")
            f.write(f"- residue-preserving p(test_acc): {rp['p_value_ge_test_acc']:.6f}\n\n")
            f.write(f"- zeta guardrail score: {zg['alignment_score']:.6f}\n")
            f.write(f"- zeta guardrail p-value: {zg['alignment_p_value_ge']:.6f}\n\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    for r in results:
        label = "-".join(str(x) for x in r["moduli"])
        print(
            "family",
            label,
            "test_r2",
            round(r["observed"]["test_r2"], 6),
            "test_acc",
            round(r["observed"]["test_acc"], 6),
            "perm_p",
            round(r["nulls"]["label_permutation"]["p_value_ge_test_r2"], 6),
            "res_p",
            round(r["nulls"]["residue_preserving"]["p_value_ge_test_r2"], 6),
            "zeta_p",
            round(r["zeta_guardrail"]["alignment_p_value_ge"], 6),
        )


if __name__ == "__main__":
    main()
