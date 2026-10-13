#!/usr/bin/env python3
"""Analyze coarse predictor candidates for visible split thresholds.

This script uses the exact threshold table as the core dataset and stays fully
inside the exact recursive-system layer.
"""

from __future__ import annotations

import ast
import csv
import math
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
THRESHOLD_SUMMARY = ROOT / "results" / "prime_transport_recursive_system" / "threshold_law_summary.csv"
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"
OFFSETS_BY_NAME = {
    "twins": (0, 2),
    "triplet": (0, 2, 6),
    "quadruplet": (0, 2, 6, 8),
}


def odd_prime_factors(W: int) -> list[int]:
    remaining = W
    factors: list[int] = []
    for prime in (2, 3):
        while remaining % prime == 0:
            remaining //= prime
    divisor = 5
    while divisor * divisor <= remaining:
        if remaining % divisor == 0:
            factors.append(divisor)
            while remaining % divisor == 0:
                remaining //= divisor
        divisor += 2
    if remaining > 1:
        factors.append(remaining)
    return factors


def admissible_density(W: int, offsets: tuple[int, ...]) -> float:
    L = W // 6
    j = np.arange(L, dtype=np.int64)
    mask = np.ones(L, dtype=np.uint8)
    for prime in odd_prime_factors(W):
        inv6 = pow(6, -1, prime)
        forbidden = {
            int(((-5 - offset) * inv6) % prime)
            for offset in offsets
        }
        allowed = np.ones(prime, dtype=np.uint8)
        for residue in forbidden:
            allowed[residue] = 0
        mask &= allowed[j % prime]
    return float(mask.mean())


def nu_p(offsets: tuple[int, ...], prime: int) -> int:
    inv6 = pow(6, -1, prime)
    forbidden = {
        int(((-5 - offset) * inv6) % prime)
        for offset in offsets
    }
    return len(forbidden)


def forbidden_residues(offsets: tuple[int, ...], prime: int) -> list[int]:
    inv6 = pow(6, -1, prime)
    return sorted(
        {
            int(((-5 - offset) * inv6) % prime)
            for offset in offsets
        }
    )


def circular_gaps(sorted_residues: list[int], prime: int) -> list[int]:
    if not sorted_residues:
        return []
    gaps = []
    for left, right in zip(sorted_residues, sorted_residues[1:]):
        gaps.append(int(right - left))
    gaps.append(int((sorted_residues[0] + prime) - sorted_residues[-1]))
    return gaps


def gap_entropy(gaps: list[int]) -> float:
    total = float(sum(gaps))
    if total <= 0.0:
        return 0.0
    probs = np.asarray(gaps, dtype=np.float64) / total
    probs = probs[probs > 0.0]
    return float(-(probs * np.log(probs)).sum())


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
    if x.size < 2:
        return float("nan")
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


def monotone_score(rows: list[dict[str, object]], group_key: str, order_key: str, target_key: str) -> tuple[int, int]:
    ok = 0
    total = 0
    groups: dict[object, list[dict[str, object]]] = {}
    for row in rows:
        groups.setdefault(row[group_key], []).append(row)
    for items in groups.values():
        items.sort(key=lambda row: float(row[order_key]))
        for left, right in zip(items, items[1:]):
            total += 1
            if float(left[target_key]) <= float(right[target_key]):
                ok += 1
    return ok, total


def main() -> None:
    with THRESHOLD_SUMMARY.open("r", encoding="utf-8", newline="") as handle:
        raw_rows = list(csv.DictReader(handle))

    density_cache: dict[tuple[int, tuple[int, ...]], float] = {}
    analysis_rows: list[dict[str, object]] = []
    for row in raw_rows:
        offsets = tuple(ast.literal_eval(row["offsets"]))
        parent_W = int(row["parent_W"])
        child_W = int(row["child_W"])
        prime = int(row["new_prime"])
        tuplet_size = int(row["tuplet_size"])
        visible_exact = row["visible_found_exact_within_range"] == "1"
        visible_value = float(row["first_visible_split_H"]) if visible_exact else float("nan")
        density_key = (parent_W, offsets)
        if density_key not in density_cache:
            density_cache[density_key] = admissible_density(parent_W, offsets)
        residues = forbidden_residues(offsets, prime)
        local_forbidden = len(residues)
        local_allowed = prime - local_forbidden
        gaps = circular_gaps(residues, prime)
        min_gap = min(gaps)
        max_gap = max(gaps)
        span = prime - max_gap
        analysis_rows.append(
            {
                "tuplet_name": row["tuplet_name"],
                "offsets": row["offsets"],
                "parent_W": parent_W,
                "child_W": child_W,
                "new_prime": prime,
                "tuplet_size": tuplet_size,
                "parent_phase_length": parent_W // 6,
                "parent_admissible_density": density_cache[density_key],
                "local_forbidden_count": local_forbidden,
                "local_allowed_count": local_allowed,
                "local_allowed_fraction": local_allowed / prime,
                "forbidden_residues": " ".join(str(x) for x in residues),
                "gap_multiset": " ".join(str(x) for x in sorted(gaps)),
                "gap_min": min_gap,
                "gap_max": max_gap,
                "gap_span": span,
                "gap_max_over_prime": max_gap / prime,
                "gap_span_over_prime": span / prime,
                "gap_num_distinct": len(set(gaps)),
                "gap_entropy": gap_entropy(gaps),
                "new_prime_times_tuplet_size": prime * tuplet_size,
                "new_prime_times_forbidden_count": prime * local_forbidden,
                "first_internal_split_H": int(row["first_internal_split_H"]),
                "first_visible_split_H": row["first_visible_split_H"],
                "first_visible_split_lower_bound": (
                    int(row["max_H_checked"]) + 1 if not visible_exact else int(row["first_visible_split_H"])
                ),
                "visible_found_exact_within_range": int(row["visible_found_exact_within_range"]),
                "visible_status": row["visible_status"],
            }
        )

    exact_rows = [row for row in analysis_rows if row["visible_found_exact_within_range"] == 1]
    y = np.asarray([float(row["first_visible_split_H"]) for row in exact_rows], dtype=np.float64)
    predictors = {
        "new_prime": np.asarray([float(row["new_prime"]) for row in exact_rows], dtype=np.float64),
        "tuplet_size": np.asarray([float(row["tuplet_size"]) for row in exact_rows], dtype=np.float64),
        "local_forbidden_count": np.asarray([float(row["local_forbidden_count"]) for row in exact_rows], dtype=np.float64),
        "local_allowed_count": np.asarray([float(row["local_allowed_count"]) for row in exact_rows], dtype=np.float64),
        "local_allowed_fraction": np.asarray([float(row["local_allowed_fraction"]) for row in exact_rows], dtype=np.float64),
        "gap_min": np.asarray([float(row["gap_min"]) for row in exact_rows], dtype=np.float64),
        "gap_max": np.asarray([float(row["gap_max"]) for row in exact_rows], dtype=np.float64),
        "gap_span": np.asarray([float(row["gap_span"]) for row in exact_rows], dtype=np.float64),
        "gap_max_over_prime": np.asarray([float(row["gap_max_over_prime"]) for row in exact_rows], dtype=np.float64),
        "gap_span_over_prime": np.asarray([float(row["gap_span_over_prime"]) for row in exact_rows], dtype=np.float64),
        "gap_num_distinct": np.asarray([float(row["gap_num_distinct"]) for row in exact_rows], dtype=np.float64),
        "gap_entropy": np.asarray([float(row["gap_entropy"]) for row in exact_rows], dtype=np.float64),
        "parent_phase_length": np.asarray([float(row["parent_phase_length"]) for row in exact_rows], dtype=np.float64),
        "parent_admissible_density": np.asarray([float(row["parent_admissible_density"]) for row in exact_rows], dtype=np.float64),
        "new_prime_times_tuplet_size": np.asarray([float(row["new_prime_times_tuplet_size"]) for row in exact_rows], dtype=np.float64),
        "new_prime_times_forbidden_count": np.asarray([float(row["new_prime_times_forbidden_count"]) for row in exact_rows], dtype=np.float64),
    }
    score_rows: list[dict[str, object]] = []
    for name, values in predictors.items():
        score_rows.append(
            {
                "predictor": name,
                "pearson_visible_threshold": corrcoef(values, y),
                "spearman_visible_threshold": corrcoef(rankdata(values), rankdata(y)),
            }
        )
    score_rows.sort(key=lambda row: abs(float(row["spearman_visible_threshold"])), reverse=True)

    monotone_rows = []
    by_tuplet_ok, by_tuplet_total = monotone_score(exact_rows, "tuplet_name", "new_prime", "first_visible_split_H")
    monotone_rows.append(
        {
            "relation": "within_tuplet_visible_non_decreasing_in_new_prime",
            "ok_pairs": by_tuplet_ok,
            "total_pairs": by_tuplet_total,
            "fraction_ok": by_tuplet_ok / by_tuplet_total if by_tuplet_total else float("nan"),
        }
    )
    by_prime_ok, by_prime_total = monotone_score(exact_rows, "new_prime", "tuplet_size", "first_visible_split_H")
    monotone_rows.append(
        {
            "relation": "within_new_prime_visible_non_decreasing_in_tuplet_size",
            "ok_pairs": by_prime_ok,
            "total_pairs": by_prime_total,
            "fraction_ok": by_prime_ok / by_prime_total if by_prime_total else float("nan"),
        }
    )

    rows_path = OUT_DIR / "visible_threshold_predictor_rows.csv"
    arrangement_path = OUT_DIR / "visible_threshold_arrangement_stats.csv"
    scores_path = OUT_DIR / "visible_threshold_predictor_scores.csv"
    monotone_path = OUT_DIR / "visible_threshold_monotonicity.csv"
    write_csv(rows_path, analysis_rows)
    arrangement_rows = [
        {
            "tuplet_name": row["tuplet_name"],
            "offsets": row["offsets"],
            "parent_W": row["parent_W"],
            "child_W": row["child_W"],
            "new_prime": row["new_prime"],
            "forbidden_residues": row["forbidden_residues"],
            "gap_multiset": row["gap_multiset"],
            "gap_min": row["gap_min"],
            "gap_max": row["gap_max"],
            "gap_span": row["gap_span"],
            "gap_max_over_prime": row["gap_max_over_prime"],
            "gap_span_over_prime": row["gap_span_over_prime"],
            "gap_num_distinct": row["gap_num_distinct"],
            "gap_entropy": row["gap_entropy"],
            "first_visible_split_H": row["first_visible_split_H"],
            "first_visible_split_lower_bound": row["first_visible_split_lower_bound"],
            "visible_status": row["visible_status"],
        }
        for row in analysis_rows
    ]
    write_csv(arrangement_path, arrangement_rows)
    write_csv(scores_path, score_rows)
    write_csv(monotone_path, monotone_rows)

    top = score_rows[0]
    print(f"top_predictor,{top['predictor']}")
    print(f"top_spearman,{float(top['spearman_visible_threshold']):.12f}")
    print(f"rows_csv,{rows_path}")
    print(f"scores_csv,{scores_path}")


if __name__ == "__main__":
    main()
