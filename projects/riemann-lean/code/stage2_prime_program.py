#!/usr/bin/env python3
"""Stage-2 prime geometry research program.

Focus:
- stronger controls (more trials)
- holdout generalization
- convergence vs N
- phase-wobble metrics inspired by spinning-top intuition
- FDR correction across many tested configurations
"""

from __future__ import annotations

import argparse
import json
import math
import os
import random
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import (
    ExperimentConfig,
    embed_4d,
    load_zeta_zeros_file,
    mean_vec,
    parse_floats,
    parse_moduli,
    run_experiment,
    sieve_primes,
)


def predict_by_centroids(v: Sequence[float], c_p: Sequence[float], c_c: Sequence[float]) -> bool:
    d_p = sum((v[i] - c_p[i]) ** 2 for i in range(4))
    d_c = sum((v[i] - c_c[i]) ** 2 for i in range(4))
    return d_p < d_c


def balanced_vectors(
    lo: int,
    hi: int,
    is_prime: Sequence[bool],
    moduli: Sequence[int],
    radius_power: float,
) -> Tuple[List[Tuple[float, float, float, float]], List[Tuple[float, float, float, float]]]:
    primes = [n for n in range(max(2, lo), hi + 1) if is_prime[n]]
    composites = [n for n in range(max(4, lo), hi + 1) if not is_prime[n]]
    pv = [embed_4d(n, moduli, radius_power) for n in primes]
    cv = [embed_4d(n, moduli, radius_power) for n in composites]
    k = min(len(pv), len(cv))
    return pv[:k], cv[:k]


def holdout_accuracy(
    n_max: int,
    is_prime: Sequence[bool],
    moduli: Sequence[int],
    radius_power: float,
) -> Dict[str, float]:
    split = n_max // 2
    train_p, train_c = balanced_vectors(2, split, is_prime, moduli, radius_power)
    test_p, test_c = balanced_vectors(split + 1, n_max, is_prime, moduli, radius_power)

    if not train_p or not train_c or not test_p or not test_c:
        return {
            "train_size": 0,
            "test_size": 0,
            "holdout_accuracy": 0.0,
            "train_accuracy": 0.0,
        }

    c_p = mean_vec(train_p)
    c_c = mean_vec(train_c)

    train_correct = sum(1 for v in train_p if predict_by_centroids(v, c_p, c_c))
    train_correct += sum(1 for v in train_c if not predict_by_centroids(v, c_p, c_c))
    train_acc = train_correct / (len(train_p) + len(train_c))

    test_correct = sum(1 for v in test_p if predict_by_centroids(v, c_p, c_c))
    test_correct += sum(1 for v in test_c if not predict_by_centroids(v, c_p, c_c))
    test_acc = test_correct / (len(test_p) + len(test_c))

    return {
        "train_size": len(train_p),
        "test_size": len(test_p),
        "holdout_accuracy": test_acc,
        "train_accuracy": train_acc,
        "generalization_gap": train_acc - test_acc,
    }


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2 * math.pi
    while d < -math.pi:
        d += 2 * math.pi
    return d


def shannon_entropy(values: Sequence[float], bins: int = 32) -> float:
    if not values:
        return 0.0
    lo, hi = min(values), max(values)
    if hi - lo < 1e-12:
        return 0.0
    counts = [0] * bins
    for v in values:
        idx = int((v - lo) / (hi - lo) * bins)
        idx = max(0, min(bins - 1, idx))
        counts[idx] += 1
    total = sum(counts)
    h = 0.0
    for c in counts:
        if c:
            p = c / total
            h -= p * math.log(p)
    return h / math.log(bins)


def phase_wobble_metrics(nums: Sequence[int], moduli: Sequence[int], radius_power: float) -> Dict[str, float]:
    # Projection used for phase intuition: angle in first complex plane (x,y).
    angles = []
    for n in nums:
        x, y, _, _ = embed_4d(n, moduli, radius_power)
        angles.append(math.atan2(y, x))

    if len(angles) < 4:
        return {
            "phase_velocity_std": 0.0,
            "phase_jerk_std": 0.0,
            "phase_velocity_entropy": 0.0,
        }

    vel = [unwrap_delta(angles[i], angles[i + 1]) for i in range(len(angles) - 1)]
    jerk = [vel[i + 1] - vel[i] for i in range(len(vel) - 1)]

    return {
        "phase_velocity_std": statistics.pstdev(vel),
        "phase_jerk_std": statistics.pstdev(jerk) if jerk else 0.0,
        "phase_velocity_entropy": shannon_entropy(vel, bins=32),
    }


def phase_wobble_metrics_from_angles(angles: Sequence[float]) -> Dict[str, float]:
    if len(angles) < 4:
        return {
            "phase_velocity_std": 0.0,
            "phase_jerk_std": 0.0,
            "phase_velocity_entropy": 0.0,
        }
    vel = [unwrap_delta(angles[i], angles[i + 1]) for i in range(len(angles) - 1)]
    jerk = [vel[i + 1] - vel[i] for i in range(len(vel) - 1)]
    return {
        "phase_velocity_std": statistics.pstdev(vel),
        "phase_jerk_std": statistics.pstdev(jerk) if jerk else 0.0,
        "phase_velocity_entropy": shannon_entropy(vel, bins=32),
    }


def harmonic_moments(nums: Sequence[int], moduli: Sequence[int], radius_power: float, max_l: int = 6) -> Dict[str, float]:
    angles = []
    for n in nums:
        x, y, _, _ = embed_4d(n, moduli, radius_power)
        angles.append(math.atan2(y, x))
    if not angles:
        return {f"h{l}": 0.0 for l in range(1, max_l + 1)}

    out = {}
    n = len(angles)
    for l in range(1, max_l + 1):
        c_re = sum(math.cos(l * a) for a in angles) / n
        c_im = sum(math.sin(l * a) for a in angles) / n
        out[f"h{l}"] = math.sqrt(c_re * c_re + c_im * c_im)
    return out


def harmonic_moments_from_angles(angles: Sequence[float], max_l: int = 6) -> Dict[str, float]:
    if not angles:
        return {f"h{l}": 0.0 for l in range(1, max_l + 1)}
    out = {}
    n = len(angles)
    for l in range(1, max_l + 1):
        c_re = sum(math.cos(l * a) for a in angles) / n
        c_im = sum(math.sin(l * a) for a in angles) / n
        out[f"h{l}"] = math.sqrt(c_re * c_re + c_im * c_im)
    return out


def p_value_ge(observed: float, null_values: Sequence[float]) -> float:
    if not null_values:
        return 1.0
    ge = sum(1 for v in null_values if v >= observed)
    return (ge + 1.0) / (len(null_values) + 1.0)


def precompute_angles(n_max: int, moduli: Sequence[int], radius_power: float) -> List[float]:
    angles = [0.0] * (n_max + 1)
    for n in range(2, n_max + 1):
        x, y, _, _ = embed_4d(n, moduli, radius_power)
        angles[n] = math.atan2(y, x)
    return angles


def sample_cramer_set(n_max: int, k: int, rng: random.Random) -> List[int]:
    candidates = [n for n in range(3, n_max + 1, 2)]
    chosen = [2] if k > 0 and n_max >= 2 else []
    for n in candidates:
        p = min(1.0, 1.0 / max(2.0, math.log(n)))
        if rng.random() < p:
            chosen.append(n)
    if len(chosen) < k:
        pool = [n for n in candidates if n not in set(chosen)]
        rng.shuffle(pool)
        chosen.extend(pool[: max(0, k - len(chosen))])
    rng.shuffle(chosen)
    return sorted(chosen[:k])


def sample_residue_preserving_set(
    k: int,
    prime_nums: Sequence[int],
    composite_nums: Sequence[int],
    residue_mod: int,
    rng: random.Random,
) -> List[int]:
    target_counts: Dict[int, int] = defaultdict(int)
    for n in prime_nums[:k]:
        target_counts[n % residue_mod] += 1

    buckets: Dict[int, List[int]] = defaultdict(list)
    for n in composite_nums:
        buckets[n % residue_mod].append(n)
    for r in buckets:
        rng.shuffle(buckets[r])

    out: List[int] = []
    used = set()
    for r, cnt in target_counts.items():
        take = buckets.get(r, [])[:cnt]
        out.extend(take)
        used.update(take)

    if len(out) < k:
        pool = [n for n in composite_nums if n not in used]
        rng.shuffle(pool)
        out.extend(pool[: k - len(out)])

    return sorted(out[:k])


def null_model_pvalues(
    prime_nums: Sequence[int],
    comp_nums: Sequence[int],
    angle_by_n: Sequence[float],
    observed_phase_jerk_delta: float,
    observed_harmonic_deltas: Dict[str, float],
    trials: int,
    seed: int,
    residue_mod: int,
    max_sample_size: int,
) -> Dict[str, object]:
    if trials <= 0:
        return {}

    rng = random.Random(seed)
    k = min(len(prime_nums), len(comp_nums))
    if max_sample_size > 0:
        k = min(k, max_sample_size)
    prime_sub = sorted(prime_nums[:k])
    comp_sub = sorted(comp_nums[:k])
    comp_pool = list(comp_nums)

    def metrics_for_sets(a_nums: Sequence[int], b_nums: Sequence[int]) -> Tuple[float, Dict[str, float]]:
        a_angles = [angle_by_n[n] for n in a_nums]
        b_angles = [angle_by_n[n] for n in b_nums]
        wob_a = phase_wobble_metrics_from_angles(a_angles)
        wob_b = phase_wobble_metrics_from_angles(b_angles)
        harm_a = harmonic_moments_from_angles(a_angles, max_l=6)
        harm_b = harmonic_moments_from_angles(b_angles, max_l=6)
        return (
            wob_a["phase_jerk_std"] - wob_b["phase_jerk_std"],
            {f"delta_h{l}": harm_a[f"h{l}"] - harm_b[f"h{l}"] for l in range(1, 7)},
        )

    cramer_jerk = []
    residue_jerk = []
    cramer_h = {f"delta_h{l}": [] for l in range(1, 7)}
    residue_h = {f"delta_h{l}": [] for l in range(1, 7)}

    for _ in range(trials):
        # Cramer-style synthetic "primes"
        s = sample_cramer_set(len(angle_by_n) - 1, k, rng)
        c_control = rng.sample(comp_pool, k)
        d_j, d_h = metrics_for_sets(s, c_control)
        cramer_jerk.append(d_j)
        for key in cramer_h:
            cramer_h[key].append(d_h[key])

        # Residue-preserving synthetic set
        rset = sample_residue_preserving_set(k, prime_sub, comp_pool, residue_mod, rng)
        c_control2 = rng.sample(comp_pool, k)
        d_j2, d_h2 = metrics_for_sets(rset, c_control2)
        residue_jerk.append(d_j2)
        for key in residue_h:
            residue_h[key].append(d_h2[key])

    out = {
        "trials": trials,
        "residue_mod": residue_mod,
        "sample_size": k,
        "phase_jerk_delta": {
            "observed": observed_phase_jerk_delta,
            "cramer_p_value_ge": p_value_ge(observed_phase_jerk_delta, cramer_jerk),
            "residue_preserving_p_value_ge": p_value_ge(observed_phase_jerk_delta, residue_jerk),
        },
        "harmonic_deltas": {},
    }
    for key, obs in observed_harmonic_deltas.items():
        out["harmonic_deltas"][key] = {
            "observed": obs,
            "cramer_p_value_ge": p_value_ge(obs, cramer_h[key]),
            "residue_preserving_p_value_ge": p_value_ge(obs, residue_h[key]),
        }
    return out


def bh_fdr(rows: List[Dict[str, object]], key: str, out_key: str) -> None:
    pvals = [(i, float(r.get(key, 1.0))) for i, r in enumerate(rows)]
    pvals.sort(key=lambda x: x[1])
    m = max(1, len(pvals))
    q = [1.0] * m
    for rank, (_, p) in enumerate(pvals, start=1):
        q[rank - 1] = p * m / rank
    for i in range(m - 2, -1, -1):
        q[i] = min(q[i], q[i + 1])
    for (idx, _), qq in zip(pvals, q):
        rows[idx][out_key] = min(1.0, qq)


def write_convergence_svg(rows: List[Dict[str, object]], out_path: str) -> None:
    if not rows:
        return
    # Keep only best config per N by holdout accuracy.
    best_by_n: Dict[int, Dict[str, object]] = {}
    for r in rows:
        n = int(r["n_max"])
        if n not in best_by_n or float(r["holdout_accuracy"]) > float(best_by_n[n]["holdout_accuracy"]):
            best_by_n[n] = r

    ns = sorted(best_by_n)
    acc = [float(best_by_n[n]["holdout_accuracy"]) for n in ns]
    wob = [float(best_by_n[n]["prime_phase_jerk_std"] - best_by_n[n]["comp_phase_jerk_std"]) for n in ns]

    w, h = 920, 460
    pad = 55
    inner_w = w - 2 * pad
    inner_h = h - 2 * pad

    def norm_x(v: float) -> float:
        if len(ns) == 1:
            return pad + inner_w * 0.5
        return pad + (v - ns[0]) / (ns[-1] - ns[0]) * inner_w

    def norm_y(v: float, lo: float, hi: float) -> float:
        if hi - lo < 1e-12:
            return pad + inner_h * 0.5
        return h - pad - (v - lo) / (hi - lo) * inner_h

    acc_lo, acc_hi = min(acc), max(acc)
    wob_lo, wob_hi = min(wob), max(wob)

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n')
        f.write(f'<rect x="0" y="0" width="{w}" height="{h}" fill="#fbfaf7"/>\n')
        f.write('<text x="20" y="28" font-size="18" fill="#111827">Convergence: Holdout Accuracy and Wobble Delta</text>\n')

        for n in ns:
            x = norm_x(n)
            f.write(f'<line x1="{x:.2f}" y1="{h-pad}" x2="{x:.2f}" y2="{h-pad+5}" stroke="#6b7280"/>\n')
            f.write(f'<text x="{x-18:.2f}" y="{h-pad+18}" font-size="10" fill="#6b7280">{n}</text>\n')

        f.write(f'<line x1="{pad}" y1="{h-pad}" x2="{w-pad}" y2="{h-pad}" stroke="#9ca3af"/>\n')

        pts_acc = []
        pts_wob = []
        for n, a, wb in zip(ns, acc, wob):
            x = norm_x(n)
            y_a = norm_y(a, acc_lo, acc_hi)
            y_w = norm_y(wb, wob_lo, wob_hi)
            pts_acc.append((x, y_a))
            pts_wob.append((x, y_w))

        f.write('<polyline fill="none" stroke="#1f77b4" stroke-width="2" points="')
        f.write(" ".join(f"{x:.2f},{y:.2f}" for x, y in pts_acc))
        f.write('"/>\n')
        f.write('<polyline fill="none" stroke="#d97706" stroke-width="2" points="')
        f.write(" ".join(f"{x:.2f},{y:.2f}" for x, y in pts_wob))
        f.write('"/>\n')

        f.write('<text x="20" y="50" font-size="12" fill="#1f77b4">blue: best holdout accuracy by N</text>\n')
        f.write('<text x="20" y="66" font-size="12" fill="#d97706">orange: prime-composite phase jerk std delta</text>\n')
        f.write("</svg>\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="Stage-2 robust prime geometry analysis")
    parser.add_argument("--n-values", type=str, default="30000,60000,100000")
    parser.add_argument("--moduli-sets", type=str, default="5,7,11,13,17;5,7,11,13,19;5,7,11,17,19")
    parser.add_argument("--radius-powers", type=str, default="1.3,1.5")
    parser.add_argument("--control-trials", type=int, default=200)
    parser.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    parser.add_argument("--max-zeta-zeros", type=int, default=64)
    parser.add_argument("--null-trials", type=int, default=40)
    parser.add_argument("--residue-mod", type=int, default=30)
    parser.add_argument("--null-sample-size", type=int, default=3000)
    parser.add_argument("--seed", type=int, default=20260216)
    parser.add_argument("--output", type=str, default="")
    args = parser.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    radius_values = parse_floats(args.radius_powers)
    moduli_sets = [parse_moduli(x) for x in args.moduli_sets.split(";") if x.strip()]
    zeta_zeros = load_zeta_zeros_file(args.zeta_zeros_file) if args.zeta_zeros_file else None

    rows: List[Dict[str, object]] = []

    for n_max in n_values:
        is_prime = sieve_primes(n_max)
        for m_idx, moduli in enumerate(moduli_sets):
            for r_idx, rp in enumerate(radius_values):
                angle_by_n = precompute_angles(n_max, moduli, rp)
                cfg = ExperimentConfig(
                    n_max=n_max,
                    moduli=moduli,
                    radius_power=rp,
                    bins=36,
                    control_trials=max(0, args.control_trials),
                    control_seed=args.seed + n_max + m_idx * 100 + r_idx,
                    zeta_zeros_imag=zeta_zeros,
                    max_zeta_zeros=max(0, args.max_zeta_zeros),
                )
                rep = run_experiment(cfg)
                hold = holdout_accuracy(n_max, is_prime, moduli, rp)

                prime_nums = [x for x in range(2, n_max + 1) if is_prime[x]]
                comp_nums = [x for x in range(4, n_max + 1) if not is_prime[x]]
                k = min(len(prime_nums), len(comp_nums))
                rng = random.Random(args.seed + n_max + m_idx * 1000 + r_idx)
                rng.shuffle(comp_nums)
                prime_sub = prime_nums[:k]
                comp_sub = comp_nums[:k]

                wob_p = phase_wobble_metrics(prime_sub, moduli, rp)
                wob_c = phase_wobble_metrics(comp_sub, moduli, rp)
                harm_p = harmonic_moments(prime_sub, moduli, rp, max_l=6)
                harm_c = harmonic_moments(comp_sub, moduli, rp, max_l=6)

                acc_p = rep.get("controls", {}).get("label_permutation", {}).get("accuracy", {}).get("p_value_ge", 1.0)
                zeta_p = rep.get("controls", {}).get("zeta_permutation", {}).get("p_value_ge", 1.0)

                row = {
                    "n_max": n_max,
                    "moduli_label": "-".join(str(v) for v in moduli),
                    "radius_power": rp,
                    "separation_ratio": rep["metrics"]["separation_ratio"],
                    "train_accuracy": hold.get("train_accuracy", 0.0),
                    "holdout_accuracy": hold.get("holdout_accuracy", 0.0),
                    "generalization_gap": hold.get("generalization_gap", 0.0),
                    "entropy_delta": rep["metrics"]["entropy_delta"],
                    "zeta_alignment_score": rep["metrics"]["zeta_alignment_score"],
                    "accuracy_p_value": acc_p,
                    "zeta_p_value": zeta_p,
                    "prime_phase_velocity_std": wob_p["phase_velocity_std"],
                    "comp_phase_velocity_std": wob_c["phase_velocity_std"],
                    "prime_phase_jerk_std": wob_p["phase_jerk_std"],
                    "comp_phase_jerk_std": wob_c["phase_jerk_std"],
                    "phase_jerk_delta": wob_p["phase_jerk_std"] - wob_c["phase_jerk_std"],
                    "prime_phase_velocity_entropy": wob_p["phase_velocity_entropy"],
                    "comp_phase_velocity_entropy": wob_c["phase_velocity_entropy"],
                }
                for l in range(1, 7):
                    key = f"h{l}"
                    row[f"prime_{key}"] = harm_p[key]
                    row[f"comp_{key}"] = harm_c[key]
                    row[f"delta_{key}"] = harm_p[key] - harm_c[key]

                observed_harmonic_deltas = {f"delta_h{l}": row[f"delta_h{l}"] for l in range(1, 7)}
                nulls = null_model_pvalues(
                    prime_nums=prime_sub,
                    comp_nums=comp_sub,
                    angle_by_n=angle_by_n,
                    observed_phase_jerk_delta=row["phase_jerk_delta"],
                    observed_harmonic_deltas=observed_harmonic_deltas,
                    trials=max(0, args.null_trials),
                    seed=args.seed + n_max + m_idx * 10000 + r_idx,
                    residue_mod=max(2, args.residue_mod),
                    max_sample_size=max(0, args.null_sample_size),
                )
                row["null_models"] = nulls
                row["phase_jerk_cramer_p_value"] = (
                    nulls.get("phase_jerk_delta", {}).get("cramer_p_value_ge", 1.0) if nulls else 1.0
                )
                row["phase_jerk_residue_p_value"] = (
                    nulls.get("phase_jerk_delta", {}).get("residue_preserving_p_value_ge", 1.0) if nulls else 1.0
                )
                row["score"] = (
                    row["holdout_accuracy"]
                    * row["separation_ratio"]
                    * (1.0 - min(1.0, row["accuracy_p_value"]))
                )
                rows.append(row)

    bh_fdr(rows, "accuracy_p_value", "accuracy_q_value")
    bh_fdr(rows, "zeta_p_value", "zeta_q_value")
    bh_fdr(rows, "phase_jerk_cramer_p_value", "phase_jerk_cramer_q_value")
    bh_fdr(rows, "phase_jerk_residue_p_value", "phase_jerk_residue_q_value")

    rows.sort(key=lambda r: float(r["score"]), reverse=True)

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_path = args.output or f"research/output/stage2_report_{ts}.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_values": n_values,
            "moduli_sets": moduli_sets,
            "radius_powers": radius_values,
            "control_trials": args.control_trials,
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": args.max_zeta_zeros,
            "null_trials": args.null_trials,
            "residue_mod": args.residue_mod,
            "null_sample_size": args.null_sample_size,
            "seed": args.seed,
        },
        "top": rows[:12],
        "all": rows,
    }

    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    tsv_path = out_path.replace(".json", ".tsv")
    with open(tsv_path, "w", encoding="utf-8") as f:
        header = [
            "n_max",
            "moduli_label",
            "radius_power",
            "holdout_accuracy",
            "separation_ratio",
            "generalization_gap",
            "accuracy_p_value",
            "accuracy_q_value",
            "zeta_p_value",
            "zeta_q_value",
            "phase_jerk_delta",
            "phase_jerk_cramer_p_value",
            "phase_jerk_cramer_q_value",
            "phase_jerk_residue_p_value",
            "phase_jerk_residue_q_value",
            "score",
        ]
        f.write("\t".join(header) + "\n")
        for r in rows:
            f.write(
                f"{r['n_max']}\t{r['moduli_label']}\t{r['radius_power']}\t"
                f"{r['holdout_accuracy']:.6f}\t{r['separation_ratio']:.6f}\t{r['generalization_gap']:.6f}\t"
                f"{r['accuracy_p_value']:.6f}\t{r['accuracy_q_value']:.6f}\t{r['zeta_p_value']:.6f}\t{r['zeta_q_value']:.6f}\t"
                f"{r['phase_jerk_delta']:.6f}\t"
                f"{r['phase_jerk_cramer_p_value']:.6f}\t{r['phase_jerk_cramer_q_value']:.6f}\t"
                f"{r['phase_jerk_residue_p_value']:.6f}\t{r['phase_jerk_residue_q_value']:.6f}\t"
                f"{r['score']:.6f}\n"
            )

    conv_svg = out_path.replace(".json", "_convergence.svg")
    write_convergence_svg(rows, conv_svg)

    md_path = out_path.replace(".json", "_candidate_conjectures.md")
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# Candidate Conjectures (Stage 2)\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        best = rows[0] if rows else None
        if best:
            f.write("## Strongest Current Configuration\n\n")
            f.write(
                f"- n_max={best['n_max']}, moduli={best['moduli_label']}, radius_power={best['radius_power']}\n"
                f"- holdout_accuracy={best['holdout_accuracy']:.6f}, separation={best['separation_ratio']:.6f}\n"
                f"- accuracy_p={best['accuracy_p_value']:.6f}, accuracy_q={best['accuracy_q_value']:.6f}\n"
                f"- zeta_p={best['zeta_p_value']:.6f}, zeta_q={best['zeta_q_value']:.6f}\n"
                f"- phase_jerk_delta(prime-composite)={best['phase_jerk_delta']:.6f}\n\n"
                f"- phase_jerk_cramer_q={best['phase_jerk_cramer_q_value']:.6f}\n"
                f"- phase_jerk_residue_q={best['phase_jerk_residue_q_value']:.6f}\n\n"
            )

        f.write("## Conjecture Templates to Test\n\n")
        f.write("1. Geometric Separation Conjecture: For selected modular families, holdout accuracy remains >0.70 and stable as N grows.\n")
        f.write("2. Entropy Deficit Conjecture: Prime angular entropy remains systematically below composite entropy by a non-vanishing margin.\n")
        f.write("3. Wobble/Precession Conjecture: Prime phase-jerk statistics differ from composites with consistent sign across N and parameter families.\n")
        f.write("4. Zeta Coupling Conjecture: Zeta-alignment remains significant after FDR correction across tested families.\n")

    print(f"wrote: {out_path}")
    print(f"wrote: {tsv_path}")
    print(f"wrote: {conv_svg}")
    print(f"wrote: {md_path}")
    print("top_results:")
    for r in rows[:8]:
        print(
            f"  n={r['n_max']} moduli={r['moduli_label']} r={r['radius_power']:.2f} "
            f"holdout={r['holdout_accuracy']:.6f} sep={r['separation_ratio']:.6f} "
            f"acc_p={r['accuracy_p_value']:.4f} acc_q={r['accuracy_q_value']:.4f} "
            f"zeta_q={r['zeta_q_value']:.4f} wobble_delta={r['phase_jerk_delta']:.6f} "
            f"wobble_cramer_q={r['phase_jerk_cramer_q_value']:.4f} wobble_residue_q={r['phase_jerk_residue_q_value']:.4f}"
        )


if __name__ == "__main__":
    main()
