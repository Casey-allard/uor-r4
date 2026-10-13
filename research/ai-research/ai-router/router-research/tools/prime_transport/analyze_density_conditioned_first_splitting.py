#!/usr/bin/env python3
"""Analyze first-splitting statistics after coarse conditioning on parent density."""

from __future__ import annotations

import csv
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
EVENT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_first_splitting_event_stats.csv"
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"


def density_band(density: float) -> str:
    if density <= 0.08:
        return "low_density_le_0p08"
    if density <= 0.18:
        return "mid_density_0p08_to_0p18"
    return "high_density_gt_0p18"


def rankdata(values: np.ndarray) -> np.ndarray:
    order = np.argsort(values, kind="mergesort")
    ranks = np.empty(values.shape[0], dtype=np.float64)
    idx = 0
    while idx < values.shape[0]:
        start = idx
        value = values[order[idx]]
        while idx + 1 < values.shape[0] and values[order[idx + 1]] == value:
            idx += 1
        end = idx
        avg_rank = 0.5 * (start + end) + 1.0
        for pos in range(start, end + 1):
            ranks[order[pos]] = avg_rank
        idx += 1
    return ranks


def corrcoef(x: np.ndarray, y: np.ndarray) -> float:
    x_center = x - x.mean()
    y_center = y - y.mean()
    denom = np.linalg.norm(x_center) * np.linalg.norm(y_center)
    if denom <= 1e-12:
        return float("nan")
    return float(np.dot(x_center, y_center) / denom)


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    with EVENT_CSV.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))

    band_to_rows: dict[str, list[dict[str, object]]] = {}
    conditioned_rows: list[dict[str, object]] = []

    for row in rows:
        density = float(row["parent_admissible_density"])
        band = density_band(density)
        band_to_rows.setdefault(band, []).append(row)

    band_summary: dict[str, dict[str, float]] = {}
    for band, band_rows in band_to_rows.items():
        band_summary[band] = {
            "mean_density": float(np.mean([float(r["parent_admissible_density"]) for r in band_rows])),
            "mean_visible_H": float(np.mean([float(r["visible_H_first"]) for r in band_rows])),
            "count": float(len(band_rows)),
        }

    for band, band_rows in band_to_rows.items():
        mean_visible = band_summary[band]["mean_visible_H"]
        mean_density = band_summary[band]["mean_density"]
        mean_num_split = float(np.mean([float(r["num_first_splitting_classes"]) for r in band_rows]))
        mean_total_extra = float(np.mean([float(r["total_extra_child_classes_from_first_split"]) for r in band_rows]))
        mean_fraction_split = float(np.mean([float(r["fraction_first_splitting_classes"]) for r in band_rows]))
        mean_gap_max = float(np.mean([float(r["gap_max"]) for r in band_rows]))
        mean_prime = float(np.mean([float(r["new_prime"]) for r in band_rows]))

        for row in band_rows:
            current = dict(row)
            current["density_band"] = band
            current["band_row_count"] = int(band_summary[band]["count"])
            current["band_mean_density"] = mean_density
            current["band_mean_visible_H"] = mean_visible
            current["visible_H_residual_within_band"] = float(row["visible_H_first"]) - mean_visible
            current["num_first_splitting_classes_residual"] = float(row["num_first_splitting_classes"]) - mean_num_split
            current["total_extra_child_classes_residual"] = float(row["total_extra_child_classes_from_first_split"]) - mean_total_extra
            current["fraction_first_splitting_classes_residual"] = float(row["fraction_first_splitting_classes"]) - mean_fraction_split
            current["gap_max_residual"] = float(row["gap_max"]) - mean_gap_max
            current["new_prime_residual"] = float(row["new_prime"]) - mean_prime
            conditioned_rows.append(current)

    y = np.asarray([float(row["visible_H_residual_within_band"]) for row in conditioned_rows], dtype=np.float64)
    predictors = {
        "num_first_splitting_classes_residual": np.asarray(
            [float(row["num_first_splitting_classes_residual"]) for row in conditioned_rows],
            dtype=np.float64,
        ),
        "total_extra_child_classes_residual": np.asarray(
            [float(row["total_extra_child_classes_residual"]) for row in conditioned_rows],
            dtype=np.float64,
        ),
        "fraction_first_splitting_classes_residual": np.asarray(
            [float(row["fraction_first_splitting_classes_residual"]) for row in conditioned_rows],
            dtype=np.float64,
        ),
        "gap_max_residual": np.asarray(
            [float(row["gap_max_residual"]) for row in conditioned_rows],
            dtype=np.float64,
        ),
        "new_prime_residual": np.asarray(
            [float(row["new_prime_residual"]) for row in conditioned_rows],
            dtype=np.float64,
        ),
    }

    score_rows: list[dict[str, object]] = []
    for name, values in predictors.items():
        score_rows.append(
            {
                "predictor": name,
                "pearson_visible_H_residual": corrcoef(values, y),
                "spearman_visible_H_residual": corrcoef(rankdata(values), rankdata(y)),
            }
        )
    score_rows.sort(key=lambda row: abs(float(row["spearman_visible_H_residual"])), reverse=True)

    band_rows_out: list[dict[str, object]] = []
    for band, band_rows in band_to_rows.items():
        visible = np.asarray([float(r["visible_H_first"]) for r in band_rows], dtype=np.float64)
        num_split = np.asarray([float(r["num_first_splitting_classes"]) for r in band_rows], dtype=np.float64)
        total_extra = np.asarray([float(r["total_extra_child_classes_from_first_split"]) for r in band_rows], dtype=np.float64)
        frac_split = np.asarray([float(r["fraction_first_splitting_classes"]) for r in band_rows], dtype=np.float64)
        band_rows_out.append(
            {
                "density_band": band,
                "row_count": len(band_rows),
                "mean_parent_density": band_summary[band]["mean_density"],
                "mean_visible_H": band_summary[band]["mean_visible_H"],
                "spearman_visible_vs_num_first_splitting_classes": corrcoef(rankdata(num_split), rankdata(visible)),
                "spearman_visible_vs_total_extra_child_classes": corrcoef(rankdata(total_extra), rankdata(visible)),
                "spearman_visible_vs_fraction_first_splitting_classes": corrcoef(rankdata(frac_split), rankdata(visible)),
            }
        )

    conditioned_path = OUT_DIR / "visible_threshold_density_conditioned_rows.csv"
    band_path = OUT_DIR / "visible_threshold_density_conditioned_band_scores.csv"
    score_path = OUT_DIR / "visible_threshold_density_conditioned_residual_scores.csv"
    write_csv(conditioned_path, conditioned_rows)
    write_csv(band_path, band_rows_out)
    write_csv(score_path, score_rows)

    best = score_rows[0]
    print(f"best_density_conditioned_predictor,{best['predictor']}")
    print(f"best_density_conditioned_spearman,{float(best['spearman_visible_H_residual']):.12f}")
    print(f"conditioned_rows_csv,{conditioned_path}")
    print(f"band_scores_csv,{band_path}")
    print(f"residual_scores_csv,{score_path}")


if __name__ == "__main__":
    main()
