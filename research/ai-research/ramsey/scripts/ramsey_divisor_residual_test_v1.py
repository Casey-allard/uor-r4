#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np


BASELINE_FEATURES = [
    "n",
    "block_ratio",
    "order_flag",
    "depth_est",
    "asymmetry_flag",
]

DIVISOR_FEATURES = [
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

TARGETS = [
    "delay",
    "force",
    "delta_delay_vs_random",
    "delta_force_vs_random",
]


def load_rows(path: Path) -> List[dict]:
    rows = []
    with path.open() as f:
        reader = csv.DictReader(f)
        for r in reader:
            rows.append(r)
    return rows


def to_float_matrix(rows: List[dict], features: List[str]) -> np.ndarray:
    x = np.ones((len(rows), 1 + len(features)))
    for j, feat in enumerate(features, start=1):
        x[:, j] = np.array([float(r[feat]) for r in rows], dtype=float)
    return x


def to_target(rows: List[dict], target: str) -> np.ndarray:
    return np.array([float(r[target]) for r in rows], dtype=float)


def fit_ols(x: np.ndarray, y: np.ndarray) -> np.ndarray:
    beta, *_ = np.linalg.lstsq(x, y, rcond=None)
    return beta


def pred(x: np.ndarray, beta: np.ndarray) -> np.ndarray:
    return x @ beta


def r2(y: np.ndarray, yhat: np.ndarray) -> float:
    ss_res = float(np.sum((y - yhat) ** 2))
    ss_tot = float(np.sum((y - y.mean()) ** 2))
    return 1.0 - ss_res / ss_tot if ss_tot > 1e-12 else 0.0


def rmse(y: np.ndarray, yhat: np.ndarray) -> float:
    return float(np.sqrt(np.mean((y - yhat) ** 2)))


def seed_folds(rows: List[dict], k: int = 5) -> Dict[int, np.ndarray]:
    seeds = sorted(set(int(r["seed"]) for r in rows))
    fold_for_seed = {s: idx % k for idx, s in enumerate(seeds)}
    folds: Dict[int, List[int]] = defaultdict(list)
    for i, r in enumerate(rows):
        folds[fold_for_seed[int(r["seed"])]] .append(i)
    return {k: np.array(v, dtype=int) for k, v in folds.items()}


def two_stage_oof(
    rows: List[dict],
    target: str,
    baseline_feats: List[str],
    divisor_feats: List[str],
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    y = to_target(rows, target)
    xb = to_float_matrix(rows, baseline_feats)
    xd = to_float_matrix(rows, divisor_feats)

    oof_baseline = np.zeros_like(y)
    oof_combined = np.zeros_like(y)
    oof_residual = np.zeros_like(y)

    folds = seed_folds(rows, 5)
    all_idx = np.arange(len(rows))
    for _, test_idx in folds.items():
        test_mask = np.zeros(len(rows), dtype=bool)
        test_mask[test_idx] = True
        train_idx = all_idx[~test_mask]

        b_beta = fit_ols(xb[train_idx], y[train_idx])
        base_train_pred = pred(xb[train_idx], b_beta)
        base_test_pred = pred(xb[test_idx], b_beta)
        oof_baseline[test_idx] = base_test_pred

        train_resid = y[train_idx] - base_train_pred
        d_beta = fit_ols(xd[train_idx], train_resid)
        resid_test_pred = pred(xd[test_idx], d_beta)
        oof_residual[test_idx] = resid_test_pred
        oof_combined[test_idx] = base_test_pred + resid_test_pred

    return oof_baseline, oof_residual, oof_combined


def two_stage_metrics(rows: List[dict], target: str) -> Tuple[dict, np.ndarray]:
    y = to_target(rows, target)
    oof_base, oof_resid_pred, oof_combined = two_stage_oof(
        rows, target, BASELINE_FEATURES, DIVISOR_FEATURES
    )

    baseline_r2 = r2(y, oof_base)
    combined_r2 = r2(y, oof_combined)
    baseline_rmse = rmse(y, oof_base)
    combined_rmse = rmse(y, oof_combined)
    residual_true = y - oof_base
    residual_r2 = r2(residual_true, oof_resid_pred)

    return (
        {
            "target": target,
            "baseline_cv_r2": baseline_r2,
            "baseline_cv_rmse": baseline_rmse,
            "combined_cv_r2": combined_r2,
            "combined_cv_rmse": combined_rmse,
            "incremental_cv_r2_lift": combined_r2 - baseline_r2,
            "residual_explained_r2": residual_r2,
        },
        residual_true,
    )


def single_feature_residual_tests(rows: List[dict], target: str) -> List[dict]:
    y = to_target(rows, target)
    xb = to_float_matrix(rows, BASELINE_FEATURES)
    folds = seed_folds(rows, 5)
    all_idx = np.arange(len(rows))

    out = []
    for feat in DIVISOR_FEATURES:
        xd = to_float_matrix(rows, [feat])
        oof_base = np.zeros_like(y)
        oof_comb = np.zeros_like(y)
        oof_resid_pred = np.zeros_like(y)
        for _, test_idx in folds.items():
            test_mask = np.zeros(len(rows), dtype=bool)
            test_mask[test_idx] = True
            train_idx = all_idx[~test_mask]
            b_beta = fit_ols(xb[train_idx], y[train_idx])
            base_train = pred(xb[train_idx], b_beta)
            base_test = pred(xb[test_idx], b_beta)
            train_resid = y[train_idx] - base_train
            d_beta = fit_ols(xd[train_idx], train_resid)
            resid_test = pred(xd[test_idx], d_beta)
            oof_base[test_idx] = base_test
            oof_resid_pred[test_idx] = resid_test
            oof_comb[test_idx] = base_test + resid_test

        baseline_r2 = r2(y, oof_base)
        combined_r2 = r2(y, oof_comb)
        resid_true = y - oof_base
        resid_r2 = r2(resid_true, oof_resid_pred)
        out.append(
            {
                "target": target,
                "feature": feat,
                "incremental_cv_r2_lift": combined_r2 - baseline_r2,
                "residual_explained_r2": resid_r2,
            }
        )
    out.sort(key=lambda r: r["incremental_cv_r2_lift"], reverse=True)
    return out


def build_residual_rows(rows: List[dict], target: str, residual: np.ndarray) -> List[dict]:
    out = []
    for i, r in enumerate(rows):
        row = dict(r)
        row["target"] = target
        row["residual"] = float(residual[i])
        out.append(row)
    return out


def mirrored_gap_analysis(rows: List[dict]) -> List[dict]:
    # Use delay and force residuals from baseline-only models.
    out = []
    for target in ["delay", "force"]:
        y = to_target(rows, target)
        xb = to_float_matrix(rows, BASELINE_FEATURES)
        oof_base = np.zeros_like(y)
        folds = seed_folds(rows, 5)
        all_idx = np.arange(len(rows))
        for _, test_idx in folds.items():
            test_mask = np.zeros(len(rows), dtype=bool)
            test_mask[test_idx] = True
            train_idx = all_idx[~test_mask]
            b_beta = fit_ols(xb[train_idx], y[train_idx])
            oof_base[test_idx] = pred(xb[test_idx], b_beta)
        resid = y - oof_base

        by_key = {}
        for i, r in enumerate(rows):
            key = (int(r["n"]), int(r["seed"]), r["family"])
            by_key[key] = (r, float(resid[i]))

        gap_rows = []
        for n in sorted(set(int(r["n"]) for r in rows)):
            for seed in sorted(set(int(r["seed"]) for r in rows)):
                kb = (n, seed, "fibonacci_block_growth_base")
                km = (n, seed, "fibonacci_mirrored_growth")
                if kb not in by_key or km not in by_key:
                    continue
                rb, eb = by_key[kb]
                rm, em = by_key[km]
                gap = em - eb
                # divisor-gap features (mirrored - base)
                d = {}
                for feat in DIVISOR_FEATURES:
                    d[feat] = float(rm[feat]) - float(rb[feat])
                d["n"] = n
                d["seed"] = seed
                d["residual_gap"] = gap
                gap_rows.append(d)

        gx = np.ones((len(gap_rows), 1 + len(DIVISOR_FEATURES)))
        for j, feat in enumerate(DIVISOR_FEATURES, start=1):
            gx[:, j] = np.array([gr[feat] for gr in gap_rows], dtype=float)
        gy = np.array([gr["residual_gap"] for gr in gap_rows], dtype=float)

        # Seed-based OOF on gap table.
        unique_seeds = sorted(set(gr["seed"] for gr in gap_rows))
        fold_for_seed = {s: idx % 5 for idx, s in enumerate(unique_seeds)}
        folds_gap: Dict[int, List[int]] = defaultdict(list)
        for i, gr in enumerate(gap_rows):
            folds_gap[fold_for_seed[gr["seed"]]].append(i)

        preds = np.zeros_like(gy)
        all_idx_gap = np.arange(len(gap_rows))
        for _, test_idx_list in folds_gap.items():
            test_idx = np.array(test_idx_list, dtype=int)
            test_mask = np.zeros(len(gap_rows), dtype=bool)
            test_mask[test_idx] = True
            train_idx = all_idx_gap[~test_mask]
            beta = fit_ols(gx[train_idx], gy[train_idx])
            preds[test_idx] = pred(gx[test_idx], beta)

        cv_r2 = r2(gy, preds)
        cv_rmse = rmse(gy, preds)

        # strongest individual divisor feature on residual gap
        feat_scores = []
        for feat in DIVISOR_FEATURES:
            fx = np.ones((len(gap_rows), 2))
            fx[:, 1] = np.array([gr[feat] for gr in gap_rows], dtype=float)
            fpreds = np.zeros_like(gy)
            for _, test_idx_list in folds_gap.items():
                test_idx = np.array(test_idx_list, dtype=int)
                test_mask = np.zeros(len(gap_rows), dtype=bool)
                test_mask[test_idx] = True
                train_idx = all_idx_gap[~test_mask]
                beta = fit_ols(fx[train_idx], gy[train_idx])
                fpreds[test_idx] = pred(fx[test_idx], beta)
            feat_scores.append((feat, r2(gy, fpreds)))
        feat_scores.sort(key=lambda x: x[1], reverse=True)
        best_feat, best_r2 = feat_scores[0]

        out.append(
            {
                "target": target,
                "gap_rows": len(gap_rows),
                "group_divisor_gap_cv_r2": cv_r2,
                "group_divisor_gap_cv_rmse": cv_rmse,
                "best_single_gap_feature": best_feat,
                "best_single_gap_feature_cv_r2": best_r2,
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


def decide(summary_rows: List[dict]) -> Tuple[str, str]:
    lifts = [r["incremental_cv_r2_lift"] for r in summary_rows]
    resid_r2 = [r["residual_explained_r2"] for r in summary_rows]
    strong = sum(1 for x in lifts if x > 0.05)
    moderate = sum(1 for x in lifts if 0.015 < x <= 0.05)
    resid_strong = sum(1 for x in resid_r2 if x > 0.12)
    if strong >= 2 and resid_strong >= 2:
        return "KEEP", "Divisor features explain meaningful residual structure across multiple targets."
    if strong >= 1 or moderate >= 2:
        return "REFINE", "Divisor features explain partial residual structure, but not consistently strong across all targets."
    return "KILL", "Residual explanatory value from divisor features is negligible after baseline controls."


def write_report(
    path: Path,
    summary_rows: List[dict],
    top_feat_rows: List[dict],
    mirrored_rows: List[dict],
    verdict: Tuple[str, str],
) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("# Ramsey Divisor Residual Test v1")
    lines.append("")
    lines.append("## Setup")
    lines.append("- Baseline model fit first on structural predictors only.")
    lines.append("- Divisor observables tested on baseline residuals (individually and grouped).")
    lines.append("- Reused existing feature table; no new simulation campaign.")
    lines.append("")
    lines.append("## Residual Increment Results")
    for r in summary_rows:
        lines.append(
            f"- {r['target']}: baseline_cv_r2={r['baseline_cv_r2']:.4f}, combined_cv_r2={r['combined_cv_r2']:.4f}, "
            f"lift={r['incremental_cv_r2_lift']:.4f}, residual_explained_r2={r['residual_explained_r2']:.4f}"
        )
    lines.append("")
    lines.append("## Strongest Individual Residual-Side Divisor Features")
    for t in TARGETS:
        rows_t = [r for r in top_feat_rows if r["target"] == t][:3]
        lines.append(f"- {t}:")
        for r in rows_t:
            lines.append(
                f"  - {r['feature']}: lift={r['incremental_cv_r2_lift']:.4f}, residual_explained_r2={r['residual_explained_r2']:.4f}"
            )
    lines.append("")
    lines.append("## Mirrored-vs-Base Residual Gap")
    for r in mirrored_rows:
        lines.append(
            f"- {r['target']}: group_gap_cv_r2={r['group_divisor_gap_cv_r2']:.4f}, "
            f"best_single={r['best_single_gap_feature']} ({r['best_single_gap_feature_cv_r2']:.4f})"
        )
    lines.append("")
    lines.append("## Verdict")
    lines.append(f"- Decision: **{verdict[0]}**")
    lines.append(f"- Rationale: {verdict[1]}")
    lines.append("")
    lines.append("## Limits")
    lines.append("- Residual analysis is still correlational and finite small-n.")
    lines.append("- Divisor features may remain partially entangled with unmodeled interactions.")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Residual test for divisor-shadow value")
    parser.add_argument(
        "--input",
        default="results/shadow/ramsey_nonlocal_shadow_test_v1/tables/feature_table.csv",
    )
    parser.add_argument(
        "--outdir",
        default="results/residual/ramsey_divisor_residual_test_v1",
    )
    args = parser.parse_args()

    rows = load_rows(Path(args.input))
    outdir = Path(args.outdir)

    summary_rows = []
    residual_rows_all = []
    top_feat_rows = []
    for target in TARGETS:
        summary, resid = two_stage_metrics(rows, target)
        summary_rows.append(summary)
        residual_rows_all.extend(build_residual_rows(rows, target, resid))
        top_feat_rows.extend(single_feature_residual_tests(rows, target))

    mirrored_rows = mirrored_gap_analysis(rows)
    v = decide(summary_rows)

    write_csv(outdir / "tables" / "residual_increment_summary.csv", summary_rows)
    write_csv(outdir / "tables" / "residual_rows.csv", residual_rows_all)
    write_csv(outdir / "tables" / "single_feature_residual_tests.csv", top_feat_rows)
    write_csv(outdir / "tables" / "mirrored_residual_gap_tests.csv", mirrored_rows)
    write_report(
        outdir / "report.md",
        summary_rows,
        top_feat_rows,
        mirrored_rows,
        v,
    )

    (outdir / "raw").mkdir(parents=True, exist_ok=True)
    (outdir / "raw" / "run.json").write_text(
        json.dumps(
            {
                "task_id": "ramsey_divisor_residual_test_v1",
                "input_rows": len(rows),
                "input_path": args.input,
                "verdict": v[0],
                "rationale": v[1],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(f"Wrote: {outdir / 'tables' / 'residual_increment_summary.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'single_feature_residual_tests.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'mirrored_residual_gap_tests.csv'}")
    print(f"Wrote: {outdir / 'report.md'}")
    print(f"Wrote: {outdir / 'raw' / 'run.json'}")
    print(f"Verdict: {v[0]}")


if __name__ == "__main__":
    main()
