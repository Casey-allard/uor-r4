#!/usr/bin/env python3
"""Test phase realignment at prime squares and higher-order R interactions.

Hypotheses:
- Model fit improves when segmented by nearest prime-square phase region.
- Interaction terms in R^2/R^4/R^6 improve cross-segment prediction.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import statistics
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import embed_4d, parse_moduli, sieve_primes
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


def linear_regression(x: List[List[float]], y: List[float], ridge: float = 1e-8) -> List[float]:
    p = len(x[0])
    xtx = [[0.0] * p for _ in range(p)]
    xty = [0.0] * p
    for xx, yy in zip(x, y):
        for i in range(p):
            xty[i] += xx[i] * yy
            for j in range(p):
                xtx[i][j] += xx[i] * xx[j]
    for i in range(p):
        xtx[i][i] += ridge
    return mat_solve(xtx, xty)


def predict(beta: Sequence[float], x: Sequence[float]) -> float:
    return sum(beta[i] * x[i] for i in range(len(beta)))


def r2(y: Sequence[float], yhat: Sequence[float]) -> float:
    mu = sum(y) / max(1, len(y))
    ss_t = sum((v - mu) ** 2 for v in y)
    ss_r = sum((a - b) ** 2 for a, b in zip(y, yhat))
    return 1.0 - ss_r / ss_t if ss_t > 1e-12 else 0.0


def nearest_prime_square_phase(n: int, prime_squares: Sequence[int]) -> float:
    # phase in [-1,1], centered at nearest prime square interval midpoint
    # find bounding prime squares ps[k] <= n < ps[k+1]
    if n <= prime_squares[0]:
        return -1.0
    lo = 0
    hi = len(prime_squares) - 1
    while lo + 1 < hi:
        mid = (lo + hi) // 2
        if prime_squares[mid] <= n:
            lo = mid
        else:
            hi = mid
    a = prime_squares[lo]
    b = prime_squares[min(lo + 1, len(prime_squares) - 1)]
    if b <= a:
        return 0.0
    u = (n - a) / (b - a)  # in [0,1]
    return 2.0 * u - 1.0


def local_square_coords(n: int, prime_squares: Sequence[int]) -> Tuple[float, float]:
    # Signed distance to nearest prime square with log compression.
    best = None
    best_d = 10**18
    for s in prime_squares:
        d = n - s
        ad = abs(d)
        if ad < best_d:
            best_d = ad
            best = d
    d = float(best if best is not None else 0.0)
    sgn = 0.0
    if d > 0:
        sgn = 1.0
    elif d < 0:
        sgn = -1.0
    logd = math.log1p(abs(d))
    return sgn, logd


def build_dataset(n_max: int, moduli: Sequence[int], radius_power: float) -> Tuple[List[Dict[str, float]], List[int]]:
    is_prime = sieve_primes(n_max)
    tau = divisor_count_sieve(n_max + 4)
    primes = [p for p in range(2, int(math.sqrt(n_max)) + 1) if is_prime[p]]
    prime_squares = [p * p for p in primes if p * p <= n_max]
    if len(prime_squares) < 2:
        prime_squares = [4, 9, 25]

    rows = []
    for n in range(8, n_max - 4):
        x, y, z, t = embed_4d(n, moduli, radius_power)
        r = math.sqrt(x * x + y * y + z * z + t * t)
        # normalized modular direction
        rn = max(1e-12, r)
        m1, m2, m3, m4 = x / rn, y / rn, z / rn, t / rn

        d1m = math.log(tau[n - 1] + 1.0)
        d1p = math.log(tau[n + 1] + 1.0)
        d2 = 0.5 * (math.log(tau[n - 2] + 1.0) + math.log(tau[n + 2] + 1.0))
        d3 = 0.5 * (math.log(tau[n - 3] + 1.0) + math.log(tau[n + 3] + 1.0))
        phase = nearest_prime_square_phase(n, prime_squares)
        side, logdist = local_square_coords(n, prime_squares)

        row = {
            "y": 1.0 if is_prime[n] else 0.0,
            "m1": m1,
            "m3": m3,
            "d1m": d1m,
            "d1p": d1p,
            "d2": d2,
            "d3": d3,
            "lr": math.log1p(r),
            "phase": phase,
            "phase2": phase * phase,
            "side": side,
            "logdist": logdist,
        }
        rows.append(row)

    return rows, prime_squares


def phase_bin(phase: float, bins: int = 4) -> int:
    p = max(-1.0, min(1.0, phase))
    u = 0.5 * (p + 1.0)
    idx = int(u * bins)
    return max(0, min(bins - 1, idx))


def make_features(
    rows: Sequence[Dict[str, float]],
    with_high_r: bool,
    with_phase: bool,
    with_piecewise: bool,
    with_local_square: bool,
) -> Tuple[List[List[float]], List[float]]:
    x = []
    y = []
    for r in rows:
        feats = [1.0, r["m1"], r["m3"], r["d1m"], r["d1p"], r["d2"], r["d3"]]
        if with_high_r:
            # stabilized higher-order interaction basis
            lr = r["lr"]
            feats += [lr * lr, lr**4, lr**6]
        if with_phase:
            feats += [r["phase"], r["phase2"], r["phase"] * r["d1m"], r["phase"] * r["d1p"]]
        if with_piecewise:
            b = phase_bin(r["phase"], bins=4)
            # one-hot (drop last bin as baseline)
            h = [1.0 if b == i else 0.0 for i in range(3)]
            d1avg = 0.5 * (r["d1m"] + r["d1p"])
            feats += h
            # local interactions to let wheel/divisor effect vary by square-phase regime
            feats += [h[i] * d1avg for i in range(3)]
            feats += [h[i] * r["m1"] for i in range(3)]
        if with_local_square:
            feats += [r["side"], r["logdist"], r["side"] * r["logdist"], r["logdist"] * r["d1m"], r["logdist"] * r["d1p"]]
        x.append(feats)
        y.append(r["y"])
    return x, y


def split_train_test(rows: Sequence[Dict[str, float]], split_n: int) -> Tuple[List[Dict[str, float]], List[Dict[str, float]]]:
    train = []
    test = []
    for i, r in enumerate(rows, start=8):
        # row order corresponds to n increasing from 8
        n = i
        if n <= split_n:
            train.append(r)
        else:
            test.append(r)
    return train, test


def eval_model(train_rows, test_rows, with_high_r, with_phase, with_piecewise, with_local_square) -> Dict[str, float]:
    xtr, ytr = make_features(
        train_rows,
        with_high_r=with_high_r,
        with_phase=with_phase,
        with_piecewise=with_piecewise,
        with_local_square=with_local_square,
    )
    xte, yte = make_features(
        test_rows,
        with_high_r=with_high_r,
        with_phase=with_phase,
        with_piecewise=with_piecewise,
        with_local_square=with_local_square,
    )
    beta = linear_regression(xtr, ytr)
    yhat_tr = [predict(beta, v) for v in xtr]
    yhat_te = [predict(beta, v) for v in xte]
    # threshold 0.5 for classification accuracy
    acc_te = sum(1 for y, p in zip(yte, yhat_te) if (p >= 0.5) == (y >= 0.5)) / max(1, len(yte))
    return {
        "train_r2": r2(ytr, yhat_tr),
        "test_r2": r2(yte, yhat_te),
        "test_acc": acc_te,
        "coef_count": len(beta),
    }


def eval_boundary_band_ensemble(rows: Sequence[Dict[str, float]], split_n: int, bands: Sequence[float]) -> Dict[str, float]:
    # Fit local models by distance-to-nearest-prime-square band on train,
    # then evaluate on test with matching band model.
    train = []
    test = []
    for i, r in enumerate(rows, start=8):
        if i <= split_n:
            train.append(r)
        else:
            test.append(r)

    def band_idx(logdist: float) -> int:
        for i, b in enumerate(bands):
            if logdist <= b:
                return i
        return len(bands)

    train_by = {}
    for r in train:
        k = band_idx(r["logdist"])
        train_by.setdefault(k, []).append(r)

    models = {}
    for k, rr in train_by.items():
        if len(rr) < 200:
            continue
        x, y = make_features(
            rr,
            with_high_r=True,
            with_phase=False,
            with_piecewise=False,
            with_local_square=True,
        )
        models[k] = linear_regression(x, y)

    y_true = []
    y_hat = []
    correct = 0
    total = 0
    for r in test:
        k = band_idx(r["logdist"])
        beta = models.get(k)
        if beta is None:
            # fallback to nearest available band model
            if not models:
                continue
            kk = min(models.keys(), key=lambda t: abs(t - k))
            beta = models[kk]
        x, y = make_features(
            [r],
            with_high_r=True,
            with_phase=False,
            with_piecewise=False,
            with_local_square=True,
        )
        p = predict(beta, x[0])
        yy = y[0]
        y_true.append(yy)
        y_hat.append(p)
        correct += 1 if (p >= 0.5) == (yy >= 0.5) else 0
        total += 1

    return {
        "train_r2": 0.0,
        "test_r2": r2(y_true, y_hat) if y_true else 0.0,
        "test_acc": correct / total if total else 0.0,
        "coef_count": sum(len(v) for v in models.values()),
        "model_count": len(models),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Prime-square phase and R^2/R^4/R^6 interaction probe")
    parser.add_argument("--n-max", type=int, default=300000)
    parser.add_argument("--moduli", type=str, default="5,7,11,13,19")
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--output", type=str, default="research/output/phase_square_interaction_probe.json")
    args = parser.parse_args()

    moduli = parse_moduli(args.moduli)
    rows, prime_squares = build_dataset(args.n_max, moduli, args.radius_power)
    split_n = args.n_max // 2
    train_rows, test_rows = split_train_test(rows, split_n)

    base = eval_model(
        train_rows, test_rows, with_high_r=False, with_phase=False, with_piecewise=False, with_local_square=False
    )
    high_r = eval_model(
        train_rows, test_rows, with_high_r=True, with_phase=False, with_piecewise=False, with_local_square=False
    )
    phase = eval_model(
        train_rows, test_rows, with_high_r=False, with_phase=True, with_piecewise=False, with_local_square=False
    )
    both = eval_model(
        train_rows, test_rows, with_high_r=True, with_phase=True, with_piecewise=False, with_local_square=False
    )
    piecewise = eval_model(
        train_rows, test_rows, with_high_r=True, with_phase=False, with_piecewise=True, with_local_square=False
    )
    piecewise_full = eval_model(
        train_rows, test_rows, with_high_r=True, with_phase=True, with_piecewise=True, with_local_square=False
    )
    local_square = eval_model(
        train_rows, test_rows, with_high_r=True, with_phase=False, with_piecewise=False, with_local_square=True
    )
    band_ensemble = eval_boundary_band_ensemble(rows, split_n=split_n, bands=[1.5, 2.5, 3.5, 4.5, 5.5])

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "moduli": moduli,
            "radius_power": args.radius_power,
            "split_n": split_n,
            "prime_square_count": len(prime_squares),
        },
        "models": {
            "base": base,
            "with_high_r_terms": high_r,
            "with_phase_terms": phase,
            "with_high_r_and_phase": both,
            "with_piecewise_square_phase": piecewise,
            "with_piecewise_and_phase": piecewise_full,
            "with_local_square_coords": local_square,
            "boundary_band_ensemble": band_ensemble,
        },
        "deltas": {
            "test_r2_gain_high_r": high_r["test_r2"] - base["test_r2"],
            "test_r2_gain_phase": phase["test_r2"] - base["test_r2"],
            "test_r2_gain_both": both["test_r2"] - base["test_r2"],
            "test_r2_gain_piecewise": piecewise["test_r2"] - base["test_r2"],
            "test_r2_gain_piecewise_full": piecewise_full["test_r2"] - base["test_r2"],
            "test_r2_gain_local_square": local_square["test_r2"] - base["test_r2"],
            "test_r2_gain_band_ensemble": band_ensemble["test_r2"] - base["test_r2"],
            "test_acc_gain_both": both["test_acc"] - base["test_acc"],
            "test_acc_gain_piecewise": piecewise["test_acc"] - base["test_acc"],
            "test_acc_gain_piecewise_full": piecewise_full["test_acc"] - base["test_acc"],
            "test_acc_gain_local_square": local_square["test_acc"] - base["test_acc"],
            "test_acc_gain_band_ensemble": band_ensemble["test_acc"] - base["test_acc"],
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Phase-Square Interaction Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- n_max={args.n_max}, split={split_n}\n")
        f.write(f"- prime-square boundaries used: {len(prime_squares)}\n\n")
        for k, m in report["models"].items():
            f.write(f"## {k}\n")
            f.write(f"- test_R2: {m['test_r2']:.6f}\n")
            f.write(f"- test_accuracy: {m['test_acc']:.6f}\n")
            f.write(f"- feature_count: {m['coef_count']}\n\n")
        f.write("## Gains vs base\n")
        for k, v in report["deltas"].items():
            f.write(f"- {k}: {v:+.6f}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    for k, m in report["models"].items():
        print(k, "test_r2", round(m["test_r2"], 6), "test_acc", round(m["test_acc"], 6))


if __name__ == "__main__":
    main()
