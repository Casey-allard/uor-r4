#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np


TARGET_FAMILIES = {
    "fibonacci_block_growth_base",
    "fibonacci_mirrored_growth",
    "balanced_binary_partitions",
    "random_baseline",
}


def divisor_count(x: int) -> int:
    c = 0
    r = int(math.isqrt(x))
    for d in range(1, r + 1):
        if x % d == 0:
            c += 1
            if d * d != x:
                c += 1
    return c


def is_prime(x: int) -> bool:
    if x < 2:
        return False
    if x == 2:
        return True
    if x % 2 == 0:
        return False
    r = int(math.isqrt(x))
    for d in range(3, r + 1, 2):
        if x % d == 0:
            return False
    return True


def phi_split(size: int) -> Tuple[int, int]:
    phi = (1.0 + math.sqrt(5.0)) / 2.0
    left = max(1, min(size - 1, round(size / phi)))
    right = size - left
    return left, right


def block_ranges(n: int, family: str) -> List[Tuple[int, int]]:
    # Top-level partition proxy for block-local statistics.
    if family == "balanced_binary_partitions":
        left = n // 2
        return [(1, left), (left + 1, n)]
    if family in {"fibonacci_block_growth_base", "fibonacci_mirrored_growth"}:
        left, right = phi_split(n)
        if family == "fibonacci_mirrored_growth":
            return [(right + 1, n), (1, right)]
        return [(1, left), (left + 1, n)]
    # random baseline: neutral split for comparability.
    left = n // 2
    return [(1, left), (left + 1, n)]


def recursive_levels(n: int, family: str) -> List[List[int]]:
    # Approximate recursive level blocks for a finite shadow summary.
    blocks = [list(range(1, n + 1))]
    levels = [blocks[0]]
    max_depth = int(math.log2(max(2, n))) + 1
    current = [list(range(1, n + 1))]
    for _ in range(max_depth):
        nxt = []
        for blk in current:
            if len(blk) <= 2:
                nxt.append(blk)
                continue
            m = len(blk)
            if family == "balanced_binary_partitions":
                left = m // 2
            else:
                left, _ = phi_split(m)
            a = blk[:left]
            b = blk[left:]
            if family == "fibonacci_mirrored_growth":
                nxt.extend([b, a])
            else:
                nxt.extend([a, b])
        current = nxt
        levels.extend(current)
        if all(len(x) <= 2 for x in current):
            break
    return levels


def corr(x: np.ndarray, y: np.ndarray) -> float:
    if x.std() < 1e-12 or y.std() < 1e-12:
        return 0.0
    return float(np.corrcoef(x, y)[0, 1])


@dataclass
class Row:
    family: str
    n: int
    seed: int
    delay: float
    force: float
    order_flag: float
    asymmetry_flag: float
    depth_est: float
    block_ratio: float
    d_mean: float
    d_std: float
    d_norm_mean: float
    prime_frac: float
    composite_frac: float
    block_var_mean: float
    cross_block_contrast: float
    level_shadow_sum: float
    order_divisor_alignment: float
    delta_delay_vs_random: float
    delta_force_vs_random: float


def load_rows(path: Path) -> List[dict]:
    out = []
    with path.open() as f:
        reader = csv.DictReader(f)
        for r in reader:
            if r["family"] in TARGET_FAMILIES:
                out.append(r)
    return out


def featureize(rows: List[dict]) -> List[Row]:
    by_key_delay: Dict[Tuple[int, int], float] = {}
    by_key_force: Dict[Tuple[int, int], float] = {}
    for r in rows:
        if r["family"] == "random_baseline":
            key = (int(r["n"]), int(r["seed"]))
            by_key_delay[key] = float(r["delay_score"])
            by_key_force[key] = float(r["force_index"])

    output: List[Row] = []
    for r in rows:
        n = int(r["n"])
        seed = int(r["seed"])
        fam = r["family"]
        delay = float(r["delay_score"])
        force = float(r["force_index"])

        divisors = np.array([divisor_count(i) for i in range(1, n + 1)], dtype=float)
        d_mean = float(divisors.mean())
        d_std = float(divisors.std())
        d_norm_mean = d_mean / max(1e-9, math.log(n + 1.0))
        prime_frac = float(sum(1 for i in range(1, n + 1) if is_prime(i)) / n)
        composite_frac = float(sum(1 for i in range(1, n + 1) if i > 1 and not is_prime(i)) / n)

        blocks = block_ranges(n, fam)
        block_vars = []
        block_means = []
        for lo, hi in blocks:
            if lo > hi:
                continue
            arr = np.array([divisor_count(i) for i in range(lo, hi + 1)], dtype=float)
            block_vars.append(float(arr.var()))
            block_means.append(float(arr.mean()))
        block_var_mean = float(np.mean(block_vars)) if block_vars else 0.0
        cross_block_contrast = abs(block_means[0] - block_means[1]) if len(block_means) >= 2 else 0.0

        levels = recursive_levels(n, fam)
        level_shadow = 0.0
        for idx, blk in enumerate(levels, start=1):
            if len(blk) <= 1:
                continue
            arr = np.array([divisor_count(i) for i in blk], dtype=float)
            level_shadow += float(arr.var()) / idx
        level_shadow_sum = level_shadow

        pos = np.array([(i + 1) / n for i in range(n)], dtype=float)
        if fam == "fibonacci_mirrored_growth":
            pos = 1.0 - pos
        order_divisor_alignment = corr(pos, divisors)

        order_flag = 1.0 if fam == "fibonacci_mirrored_growth" else 0.0
        asymmetry_flag = 1.0 if fam in {"fibonacci_block_growth_base", "fibonacci_mirrored_growth"} else 0.0
        depth_est = float(int(math.log2(max(2, n))))
        if fam == "balanced_binary_partitions":
            left = n // 2
            right = n - left
        else:
            left, right = phi_split(n)
        block_ratio = max(left, right) / max(1, min(left, right))

        key = (n, seed)
        base_delay = by_key_delay.get(key, delay)
        base_force = by_key_force.get(key, force)

        output.append(
            Row(
                family=fam,
                n=n,
                seed=seed,
                delay=delay,
                force=force,
                order_flag=order_flag,
                asymmetry_flag=asymmetry_flag,
                depth_est=depth_est,
                block_ratio=block_ratio,
                d_mean=d_mean,
                d_std=d_std,
                d_norm_mean=d_norm_mean,
                prime_frac=prime_frac,
                composite_frac=composite_frac,
                block_var_mean=block_var_mean,
                cross_block_contrast=cross_block_contrast,
                level_shadow_sum=level_shadow_sum,
                order_divisor_alignment=order_divisor_alignment,
                delta_delay_vs_random=delay - base_delay,
                delta_force_vs_random=force - base_force,
            )
        )
    return output


def design_matrix(rows: List[Row], feature_names: List[str]) -> np.ndarray:
    x = np.ones((len(rows), 1 + len(feature_names)))
    for j, name in enumerate(feature_names, start=1):
        x[:, j] = np.array([getattr(r, name) for r in rows], dtype=float)
    return x


def y_vector(rows: List[Row], target: str) -> np.ndarray:
    return np.array([getattr(r, target) for r in rows], dtype=float)


def fit_ols(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    beta, *_ = np.linalg.lstsq(x, y, rcond=None)
    return beta


def predict(x: np.ndarray, beta: np.ndarray) -> np.ndarray:
    return x @ beta


def r2(y: np.ndarray, yhat: np.ndarray) -> float:
    ss_res = float(np.sum((y - yhat) ** 2))
    ss_tot = float(np.sum((y - y.mean()) ** 2))
    return 1.0 - ss_res / ss_tot if ss_tot > 1e-12 else 0.0


def rmse(y: np.ndarray, yhat: np.ndarray) -> float:
    return float(np.sqrt(np.mean((y - yhat) ** 2)))


def seed_folds(rows: List[Row], k: int = 5) -> Dict[int, List[int]]:
    unique = sorted(set(r.seed for r in rows))
    fold_map = {s: idx % k for idx, s in enumerate(unique)}
    out = defaultdict(list)
    for i, r in enumerate(rows):
        out[fold_map[r.seed]].append(i)
    return out


def cross_val_scores(rows: List[Row], feature_names: List[str], target: str) -> Tuple[float, float]:
    x = design_matrix(rows, feature_names)
    y = y_vector(rows, target)
    folds = seed_folds(rows, 5)
    preds = np.zeros_like(y)
    for _, test_idx in folds.items():
        test_mask = np.zeros(len(rows), dtype=bool)
        test_mask[np.array(test_idx, dtype=int)] = True
        train_mask = ~test_mask
        beta = fit_ols(x[train_mask], y[train_mask])
        preds[test_mask] = predict(x[test_mask], beta)
    return r2(y, preds), rmse(y, preds)


def model_comparison(rows: List[Row]) -> List[dict]:
    baseline = ["n", "depth_est", "block_ratio", "order_flag", "asymmetry_flag"]
    divisor = [
        "d_mean",
        "d_std",
        "d_norm_mean",
        "prime_frac",
        "composite_frac",
        "block_var_mean",
        "cross_block_contrast",
        "level_shadow_sum",
        "order_divisor_alignment",
    ]
    specs = [
        ("baseline_struct_only", baseline),
        ("divisor_observables_only", divisor),
        ("baseline_plus_divisor", baseline + divisor),
    ]
    targets = ["delay", "force", "delta_delay_vs_random", "delta_force_vs_random"]
    out = []
    for target in targets:
        for name, feats in specs:
            x = design_matrix(rows, feats)
            y = y_vector(rows, target)
            beta = fit_ols(x, y)
            train_pred = predict(x, beta)
            train_r2 = r2(y, train_pred)
            train_rmse = rmse(y, train_pred)
            cv_r2, cv_rmse = cross_val_scores(rows, feats, target)
            out.append(
                {
                    "target": target,
                    "model": name,
                    "n_features": len(feats),
                    "train_r2": train_r2,
                    "train_rmse": train_rmse,
                    "cv_r2": cv_r2,
                    "cv_rmse": cv_rmse,
                }
            )
    return out


def divisor_correlations(rows: List[Row]) -> List[dict]:
    feat_names = [
        "d_mean",
        "d_std",
        "d_norm_mean",
        "prime_frac",
        "composite_frac",
        "block_var_mean",
        "cross_block_contrast",
        "level_shadow_sum",
        "order_divisor_alignment",
    ]
    targets = ["delay", "force", "delta_delay_vs_random", "delta_force_vs_random"]
    out = []
    for feat in feat_names:
        x = np.array([getattr(r, feat) for r in rows], dtype=float)
        for t in targets:
            y = np.array([getattr(r, t) for r in rows], dtype=float)
            out.append({"feature": feat, "target": t, "pearson_r": corr(x, y)})
    return out


def mirrored_gap_table(rows: List[Row]) -> List[dict]:
    by_family = defaultdict(dict)
    for r in rows:
        by_family[r.family][(r.n, r.seed)] = r
    base = by_family["fibonacci_block_growth_base"]
    mirr = by_family["fibonacci_mirrored_growth"]
    by_n = defaultdict(list)
    for k in sorted(set(base.keys()) & set(mirr.keys())):
        b = base[k]
        m = mirr[k]
        by_n[k[0]].append(m.delay - b.delay)
    out = []
    for n, vals in sorted(by_n.items()):
        out.append(
            {
                "n": n,
                "mean_mirrored_minus_base_delay": float(np.mean(vals)),
                "std_mirrored_minus_base_delay": float(np.std(vals)),
            }
        )
    return out


def write_csv(path: Path, rows: List[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not rows:
        return
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)


def verdict(model_rows: List[dict]) -> Tuple[str, str]:
    # Use CV R^2 lift of baseline+divisor over baseline-only.
    by_target = defaultdict(dict)
    for r in model_rows:
        by_target[r["target"]][r["model"]] = r
    lifts = {}
    for t in by_target:
        base = by_target[t]["baseline_struct_only"]["cv_r2"]
        plus = by_target[t]["baseline_plus_divisor"]["cv_r2"]
        lifts[t] = plus - base
    useful = sum(1 for v in lifts.values() if v > 0.05)
    weak = sum(1 for v in lifts.values() if 0.01 < v <= 0.05)
    if useful >= 2:
        return "KEEP", "Divisor observables add nontrivial predictive value beyond structural baselines on multiple targets."
    if useful >= 1 or weak >= 2:
        return "REFINE", "Divisor observables show partial/weak incremental value, but evidence is not strong or uniform."
    return "KILL", "Divisor observables add little to no explanatory value beyond simple structural baselines."


def write_report(
    path: Path,
    model_rows: List[dict],
    corr_rows: List[dict],
    gap_rows: List[dict],
    v: Tuple[str, str],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("# Ramsey Nonlocal Shadow Test v1")
    lines.append("")
    lines.append("## Setup")
    lines.append("- Reused existing ablation instance table (no new simulation campaign).")
    lines.append("- Families: base recursive, mirrored recursive, random baseline, balanced binary control.")
    lines.append("- Goal: test divisor-count observables as candidate arithmetic shadows, not as full mechanism.")
    lines.append("")
    lines.append("## Model comparison (CV R^2)")
    targets = sorted(set(r["target"] for r in model_rows))
    for t in targets:
        subset = [r for r in model_rows if r["target"] == t]
        subset.sort(key=lambda r: r["model"])
        lines.append(f"- Target `{t}`:")
        for r in subset:
            lines.append(
                f"  - {r['model']}: cv_r2={r['cv_r2']:.4f}, cv_rmse={r['cv_rmse']:.4f}"
            )
    lines.append("")
    lines.append("## Top divisor-feature correlations (absolute)")
    corr_sorted = sorted(corr_rows, key=lambda r: abs(r["pearson_r"]), reverse=True)[:8]
    for r in corr_sorted:
        lines.append(
            f"- {r['feature']} vs {r['target']}: r={r['pearson_r']:.4f}"
        )
    lines.append("")
    lines.append("## Mirrored > base behavior by n")
    for r in gap_rows:
        lines.append(
            f"- n={r['n']}: mean(mirrored-base delay)={r['mean_mirrored_minus_base_delay']:.4f}"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("- Divisor-count observables are treated as candidate shadows only.")
    lines.append("- Added value is judged by incremental CV performance over size/order/depth/asymmetry baselines.")
    lines.append("")
    lines.append("## Verdict")
    lines.append(f"- Decision: **{v[0]}**")
    lines.append(f"- Rationale: {v[1]}")
    lines.append("")
    lines.append("## Limits")
    lines.append("- This is finite small-n exploratory modeling, not causal proof.")
    lines.append("- Features are hand-crafted proxies and may miss deeper coupling structure.")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Divisor-shadow exploratory test")
    parser.add_argument(
        "--input",
        default="results/ablation/ramsey_recursive_ablation_v1/tables/instances.csv",
    )
    parser.add_argument(
        "--outdir",
        default="results/shadow/ramsey_nonlocal_shadow_test_v1",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    outdir = Path(args.outdir)
    rows_raw = load_rows(input_path)
    rows = featureize(rows_raw)
    model_rows = model_comparison(rows)
    corr_rows = divisor_correlations(rows)
    gap_rows = mirrored_gap_table(rows)
    v = verdict(model_rows)

    feature_table = [r.__dict__ for r in rows]
    write_csv(outdir / "tables" / "feature_table.csv", feature_table)
    write_csv(outdir / "tables" / "model_comparison.csv", model_rows)
    write_csv(outdir / "tables" / "divisor_correlations.csv", corr_rows)
    write_csv(outdir / "tables" / "mirrored_gap_by_n.csv", gap_rows)
    write_report(outdir / "report.md", model_rows, corr_rows, gap_rows, v)

    (outdir / "raw").mkdir(parents=True, exist_ok=True)
    (outdir / "raw" / "run.json").write_text(
        json.dumps(
            {
                "task_id": "ramsey_nonlocal_shadow_test_v1",
                "input": str(input_path),
                "rows_used": len(rows),
                "families": sorted(set(r.family for r in rows)),
                "verdict": v[0],
                "rationale": v[1],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(f"Wrote: {outdir / 'tables' / 'feature_table.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'model_comparison.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'divisor_correlations.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'mirrored_gap_by_n.csv'}")
    print(f"Wrote: {outdir / 'report.md'}")
    print(f"Wrote: {outdir / 'raw' / 'run.json'}")
    print(f"Verdict: {v[0]}")


if __name__ == "__main__":
    main()
