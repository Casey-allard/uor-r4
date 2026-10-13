#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np


BASELINE_FEATURES = ["n", "block_ratio", "order_flag", "depth_est", "asymmetry_flag"]
DIV_FEATS = ["cross_block_contrast", "composite_frac", "d_std"]
TARGETS = ["delay", "force", "delta_delay_vs_random", "delta_force_vs_random"]


def load_rows(path: Path) -> List[dict]:
    with path.open() as f:
        return list(csv.DictReader(f))


def to_target(rows: List[dict], name: str) -> np.ndarray:
    return np.array([float(r[name]) for r in rows], dtype=float)


def make_matrix(rows: List[dict], features: List[str]) -> np.ndarray:
    x = np.ones((len(rows), 1 + len(features)))
    for j, feat in enumerate(features, start=1):
        x[:, j] = np.array([float(r[feat]) for r in rows], dtype=float)
    return x


def fit_ols(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    b, *_ = np.linalg.lstsq(x, y, rcond=None)
    return b


def predict(x: np.ndarray, b: np.ndarray) -> np.ndarray:
    return x @ b


def r2(y: np.ndarray, yhat: np.ndarray) -> float:
    ss_res = float(np.sum((y - yhat) ** 2))
    ss_tot = float(np.sum((y - y.mean()) ** 2))
    return 1.0 - ss_res / ss_tot if ss_tot > 1e-12 else 0.0


def rmse(y: np.ndarray, yhat: np.ndarray) -> float:
    return float(np.sqrt(np.mean((y - yhat) ** 2)))


def seed_folds(rows: List[dict], k: int = 5) -> Dict[int, np.ndarray]:
    seeds = sorted(set(int(r["seed"]) for r in rows))
    seed_to_fold = {s: i % k for i, s in enumerate(seeds)}
    out = defaultdict(list)
    for i, r in enumerate(rows):
        out[seed_to_fold[int(r["seed"])]] .append(i)
    return {f: np.array(idx, dtype=int) for f, idx in out.items()}


def make_divisor_design(rows: List[dict], nonlinear: bool) -> np.ndarray:
    # Requested bounded feature set: 3 divisor features + pairwise interactions.
    base = np.array([[float(r[f]) for f in DIV_FEATS] for r in rows], dtype=float)
    cols = [base[:, 0], base[:, 1], base[:, 2]]
    if nonlinear:
        c1, c2, c3 = cols
        cols.extend([c1 * c2, c1 * c3, c2 * c3])
        # optional single baseline interaction term (with n), justified as scale coupling.
        n_scaled = np.array([float(r["n"]) for r in rows], dtype=float)
        n_scaled = (n_scaled - n_scaled.mean()) / (n_scaled.std() + 1e-12)
        cols.extend([n_scaled * c1, n_scaled * c2, n_scaled * c3])
    x = np.ones((len(rows), 1 + len(cols)))
    for j, c in enumerate(cols, start=1):
        x[:, j] = c
    return x


def baseline_oof(rows: List[dict], target: str) -> np.ndarray:
    y = to_target(rows, target)
    xb = make_matrix(rows, BASELINE_FEATURES)
    folds = seed_folds(rows, 5)
    preds = np.zeros_like(y)
    all_idx = np.arange(len(rows))
    for _, test_idx in folds.items():
        mask = np.zeros(len(rows), dtype=bool)
        mask[test_idx] = True
        train_idx = all_idx[~mask]
        b = fit_ols(xb[train_idx], y[train_idx])
        preds[test_idx] = predict(xb[test_idx], b)
    return preds


def residual_model_oof(rows: List[dict], residual: np.ndarray, nonlinear: bool) -> np.ndarray:
    xd = make_divisor_design(rows, nonlinear=nonlinear)
    folds = seed_folds(rows, 5)
    preds = np.zeros_like(residual)
    all_idx = np.arange(len(rows))
    for _, test_idx in folds.items():
        mask = np.zeros(len(rows), dtype=bool)
        mask[test_idx] = True
        train_idx = all_idx[~mask]
        b = fit_ols(xd[train_idx], residual[train_idx])
        preds[test_idx] = predict(xd[test_idx], b)
    return preds


def residual_comparison(rows: List[dict]) -> List[dict]:
    out = []
    for t in TARGETS:
        y = to_target(rows, t)
        y_base = baseline_oof(rows, t)
        resid_true = y - y_base

        resid_lin = residual_model_oof(rows, resid_true, nonlinear=False)
        resid_nl = residual_model_oof(rows, resid_true, nonlinear=True)

        y_lin = y_base + resid_lin
        y_nl = y_base + resid_nl

        out.append(
            {
                "target": t,
                "baseline_cv_r2": r2(y, y_base),
                "linear_combined_cv_r2": r2(y, y_lin),
                "nonlinear_combined_cv_r2": r2(y, y_nl),
                "linear_lift": r2(y, y_lin) - r2(y, y_base),
                "nonlinear_lift": r2(y, y_nl) - r2(y, y_base),
                "nonlinear_minus_linear_lift": r2(y, y_nl) - r2(y, y_lin),
                "linear_residual_r2": r2(resid_true, resid_lin),
                "nonlinear_residual_r2": r2(resid_true, resid_nl),
                "nonlinear_minus_linear_residual_r2": r2(resid_true, resid_nl) - r2(resid_true, resid_lin),
                "linear_combined_rmse": rmse(y, y_lin),
                "nonlinear_combined_rmse": rmse(y, y_nl),
            }
        )
    return out


def mirrored_gap_rows(rows: List[dict], target: str) -> Tuple[List[dict], np.ndarray]:
    y = to_target(rows, target)
    y_base = baseline_oof(rows, target)
    resid = y - y_base

    by_key: Dict[Tuple[int, int, str], Tuple[dict, float]] = {}
    for i, r in enumerate(rows):
        by_key[(int(r["n"]), int(r["seed"]), r["family"])] = (r, float(resid[i]))

    gaps = []
    for n in sorted(set(int(r["n"]) for r in rows)):
        for seed in sorted(set(int(r["seed"]) for r in rows)):
            kb = (n, seed, "fibonacci_block_growth_base")
            km = (n, seed, "fibonacci_mirrored_growth")
            if kb not in by_key or km not in by_key:
                continue
            rb, eb = by_key[kb]
            rm, em = by_key[km]
            row = {
                "n": n,
                "seed": seed,
                "residual_gap": em - eb,
                "cross_block_contrast": float(rm["cross_block_contrast"]) - float(rb["cross_block_contrast"]),
                "composite_frac": float(rm["composite_frac"]) - float(rb["composite_frac"]),
                "d_std": float(rm["d_std"]) - float(rb["d_std"]),
            }
            gaps.append(row)
    y_gap = np.array([g["residual_gap"] for g in gaps], dtype=float)
    return gaps, y_gap


def gap_design(gaps: List[dict], nonlinear: bool) -> np.ndarray:
    rows = [{k: g[k] for k in ["cross_block_contrast", "composite_frac", "d_std", "n"]} for g in gaps]
    # reuse divisor design builder by emulating row schema
    fake = []
    for r in rows:
        fake.append(
            {
                "cross_block_contrast": str(r["cross_block_contrast"]),
                "composite_frac": str(r["composite_frac"]),
                "d_std": str(r["d_std"]),
                "n": str(r["n"]),
            }
        )
    return make_divisor_design(fake, nonlinear=nonlinear)


def gap_cv(gaps: List[dict], y_gap: np.ndarray, nonlinear: bool) -> np.ndarray:
    x = gap_design(gaps, nonlinear=nonlinear)
    seeds = sorted(set(int(g["seed"]) for g in gaps))
    seed_to_fold = {s: i % 5 for i, s in enumerate(seeds)}
    fold_map = defaultdict(list)
    for i, g in enumerate(gaps):
        fold_map[seed_to_fold[int(g["seed"])]] .append(i)
    preds = np.zeros_like(y_gap)
    all_idx = np.arange(len(gaps))
    for _, test in fold_map.items():
        test_idx = np.array(test, dtype=int)
        mask = np.zeros(len(gaps), dtype=bool)
        mask[test_idx] = True
        train_idx = all_idx[~mask]
        b = fit_ols(x[train_idx], y_gap[train_idx])
        preds[test_idx] = predict(x[test_idx], b)
    return preds


def mirrored_gap_comparison(rows: List[dict]) -> List[dict]:
    out = []
    for t in ["delay", "force"]:
        gaps, y_gap = mirrored_gap_rows(rows, t)
        lin = gap_cv(gaps, y_gap, nonlinear=False)
        nl = gap_cv(gaps, y_gap, nonlinear=True)
        out.append(
            {
                "target": t,
                "gap_rows": len(gaps),
                "linear_gap_cv_r2": r2(y_gap, lin),
                "nonlinear_gap_cv_r2": r2(y_gap, nl),
                "nonlinear_minus_linear_gap_r2": r2(y_gap, nl) - r2(y_gap, lin),
                "linear_gap_rmse": rmse(y_gap, lin),
                "nonlinear_gap_rmse": rmse(y_gap, nl),
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


def decide(resid_rows: List[dict], gap_rows: List[dict]) -> Tuple[str, str]:
    # Material change requires meaningful improvement in mirrored-gap or residual targets.
    resid_gain = max(r["nonlinear_minus_linear_residual_r2"] for r in resid_rows)
    gap_gain = max(r["nonlinear_minus_linear_gap_r2"] for r in gap_rows)
    # KEEP if substantial new value (>0.03) including mirrored-gap gain.
    if gap_gain > 0.03 and resid_gain > 0.03:
        return "KEEP", "Limited nonlinear interactions reveal meaningful new residual and mirrored-gap signal."
    # REFINE for small but real gains.
    if gap_gain > 0.01 or resid_gain > 0.015:
        return "REFINE", "Nonlinear terms add small/partial residual nuance but do not materially change overall conclusion."
    return "KILL", "Nonlinear check confirms no meaningful mirrored-gap residual explanation from divisor features."


def write_report(
    path: Path,
    resid_rows: List[dict],
    gap_rows: List[dict],
    verdict: Tuple[str, str],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("# Ramsey Divisor Nonlinear Residual Check v1")
    lines.append("")
    lines.append("## Setup")
    lines.append("- Reused existing feature table; no new simulations.")
    lines.append("- Divisor candidates: cross_block_contrast, composite_frac, d_std.")
    lines.append("- Nonlinear model: pairwise interactions + optional n interactions only.")
    lines.append("")
    lines.append("## Residual Targets: Linear vs Nonlinear")
    for r in resid_rows:
        lines.append(
            f"- {r['target']}: linear_resid_r2={r['linear_residual_r2']:.4f}, "
            f"nonlinear_resid_r2={r['nonlinear_residual_r2']:.4f}, "
            f"delta={r['nonlinear_minus_linear_residual_r2']:.4f}"
        )
    lines.append("")
    lines.append("## Mirrored-vs-Base Residual Gap: Linear vs Nonlinear")
    for r in gap_rows:
        lines.append(
            f"- {r['target']}: linear_gap_r2={r['linear_gap_cv_r2']:.4f}, "
            f"nonlinear_gap_r2={r['nonlinear_gap_cv_r2']:.4f}, "
            f"delta={r['nonlinear_minus_linear_gap_r2']:.4f}"
        )
    lines.append("")
    lines.append("## Verdict")
    lines.append(f"- Decision: **{verdict[0]}**")
    lines.append(f"- Rationale: {verdict[1]}")
    lines.append("")
    lines.append("## Limits")
    lines.append("- Minimal nonlinear check only; still finite and correlational.")
    lines.append("- No claim that divisor counts are the mechanism.")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Minimal nonlinear divisor residual check")
    parser.add_argument(
        "--input",
        default="results/shadow/ramsey_nonlocal_shadow_test_v1/tables/feature_table.csv",
    )
    parser.add_argument(
        "--outdir",
        default="results/nonlinear/ramsey_divisor_nonlinear_residual_check_v1",
    )
    args = parser.parse_args()

    rows = load_rows(Path(args.input))
    outdir = Path(args.outdir)

    resid_rows = residual_comparison(rows)
    gap_rows = mirrored_gap_comparison(rows)
    verdict = decide(resid_rows, gap_rows)

    write_csv(outdir / "tables" / "nonlinear_residual_comparison.csv", resid_rows)
    write_csv(outdir / "tables" / "nonlinear_mirrored_gap_comparison.csv", gap_rows)
    write_report(outdir / "report.md", resid_rows, gap_rows, verdict)

    (outdir / "raw").mkdir(parents=True, exist_ok=True)
    (outdir / "raw" / "run.json").write_text(
        json.dumps(
            {
                "task_id": "ramsey_divisor_nonlinear_residual_check_v1",
                "input_rows": len(rows),
                "verdict": verdict[0],
                "rationale": verdict[1],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(f"Wrote: {outdir / 'tables' / 'nonlinear_residual_comparison.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'nonlinear_mirrored_gap_comparison.csv'}")
    print(f"Wrote: {outdir / 'report.md'}")
    print(f"Wrote: {outdir / 'raw' / 'run.json'}")
    print(f"Verdict: {verdict[0]}")


if __name__ == "__main__":
    main()
