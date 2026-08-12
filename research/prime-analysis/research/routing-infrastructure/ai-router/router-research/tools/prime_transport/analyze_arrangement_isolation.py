#!/usr/bin/env python3
"""Isolate arrangement effects with prime and forbidden-count held fixed."""

from __future__ import annotations

import csv
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
ROWS_CSV = ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_predictor_rows.csv"
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"
FOCUS_PRIMES = (13, 17)
FIXED_FORBIDDEN = 4


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
    with ROWS_CSV.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))

    exact_focus = [
        row for row in rows
        if int(row["new_prime"]) in FOCUS_PRIMES
        and int(row["local_forbidden_count"]) == FIXED_FORBIDDEN
        and int(row["visible_found_exact_within_range"]) == 1
    ]
    exact_focus.sort(key=lambda row: (int(row["new_prime"]), row["tuplet_name"]))

    family_means: dict[int, float] = {}
    for prime in FOCUS_PRIMES:
        family_rows = [row for row in exact_focus if int(row["new_prime"]) == prime]
        family_means[prime] = float(np.mean([float(row["first_visible_split_H"]) for row in family_rows]))

    focus_rows: list[dict[str, object]] = []
    for row in exact_focus:
        prime = int(row["new_prime"])
        visible = float(row["first_visible_split_H"])
        focus_rows.append(
            {
                "family_prime": prime,
                "tuplet_name": row["tuplet_name"],
                "offsets": row["offsets"],
                "local_forbidden_count": int(row["local_forbidden_count"]),
                "forbidden_residues": row["forbidden_residues"],
                "gap_multiset": row["gap_multiset"],
                "gap_min": float(row["gap_min"]),
                "gap_max": float(row["gap_max"]),
                "gap_second_max": float(sorted([int(x) for x in row["gap_multiset"].split()])[-2]),
                "gap_span": float(row["gap_span"]),
                "gap_num_distinct": float(row["gap_num_distinct"]),
                "gap_entropy": float(row["gap_entropy"]),
                "parent_admissible_density": float(row["parent_admissible_density"]),
                "first_visible_split_H": visible,
                "family_mean_visible_H": family_means[prime],
                "visible_H_residual_from_family_mean": visible - family_means[prime],
            }
        )

    y = np.asarray([row["visible_H_residual_from_family_mean"] for row in focus_rows], dtype=np.float64)
    predictors = {
        "gap_max": np.asarray([row["gap_max"] for row in focus_rows], dtype=np.float64),
        "gap_second_max": np.asarray([row["gap_second_max"] for row in focus_rows], dtype=np.float64),
        "gap_span": np.asarray([row["gap_span"] for row in focus_rows], dtype=np.float64),
        "gap_num_distinct": np.asarray([row["gap_num_distinct"] for row in focus_rows], dtype=np.float64),
        "gap_entropy": np.asarray([row["gap_entropy"] for row in focus_rows], dtype=np.float64),
        "parent_admissible_density": np.asarray([row["parent_admissible_density"] for row in focus_rows], dtype=np.float64),
    }
    score_rows = []
    for name, values in predictors.items():
        score_rows.append(
            {
                "predictor": name,
                "pearson_visible_H_residual": corrcoef(values, y),
                "spearman_visible_H_residual": corrcoef(rankdata(values), rankdata(y)),
            }
        )
    score_rows.sort(key=lambda row: abs(float(row["spearman_visible_H_residual"])), reverse=True)

    focus_path = OUT_DIR / "visible_threshold_arrangement_isolation_rows.csv"
    score_path = OUT_DIR / "visible_threshold_arrangement_isolation_scores.csv"
    write_csv(focus_path, focus_rows)
    write_csv(score_path, score_rows)
    best = score_rows[0]
    print(f"best_isolated_predictor,{best['predictor']}")
    print(f"best_isolated_spearman,{float(best['spearman_visible_H_residual']):.12f}")
    print(f"focus_csv,{focus_path}")
    print(f"score_csv,{score_path}")


if __name__ == "__main__":
    main()
