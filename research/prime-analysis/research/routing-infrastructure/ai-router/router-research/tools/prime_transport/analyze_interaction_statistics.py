#!/usr/bin/env python3
"""Analyze parent-grammar / new-stencil interaction statistics."""

from __future__ import annotations

import ast
import csv
from collections import Counter
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
ROWS_CSV = ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_predictor_rows.csv"
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"


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


def admissible_bits(W: int, offsets: tuple[int, ...]) -> np.ndarray:
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
    return mask


def forbidden_residues(offsets: tuple[int, ...], prime: int) -> list[int]:
    inv6 = pow(6, -1, prime)
    return sorted(
        {
            int(((-5 - offset) * inv6) % prime)
            for offset in offsets
        }
    )


def return_gap_counter(bits: np.ndarray) -> Counter[int]:
    positions = np.flatnonzero(bits > 0)
    length = bits.shape[0]
    if positions.size == 0:
        return Counter()
    gaps = []
    for left, right in zip(positions, positions[1:]):
        gaps.append(int(right - left))
    gaps.append(int((positions[0] + length) - positions[-1]))
    return Counter(gaps)


def circular_distance_to_set(value: int, residues: list[int], prime: int) -> int:
    return min(
        min((value - residue) % prime, (residue - value) % prime)
        for residue in residues
    )


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

    bit_cache: dict[tuple[int, tuple[int, ...]], np.ndarray] = {}
    interaction_rows: list[dict[str, object]] = []

    for row in rows:
        offsets = tuple(ast.literal_eval(row["offsets"]))
        parent_W = int(row["parent_W"])
        prime = int(row["new_prime"])
        cache_key = (parent_W, offsets)
        if cache_key not in bit_cache:
            bit_cache[cache_key] = admissible_bits(parent_W, offsets)
        gap_counts = return_gap_counter(bit_cache[cache_key])
        total = float(sum(gap_counts.values()))
        residues = forbidden_residues(offsets, prime)

        hit_mass = 0.0
        distance_mass = 0.0
        largest_gap = max(gap_counts, key=lambda g: (g, gap_counts[g]))
        most_common_gap, most_common_count = max(gap_counts.items(), key=lambda item: (item[1], item[0]))
        for gap, count in gap_counts.items():
            residue = gap % prime
            dist = circular_distance_to_set(residue, residues, prime)
            if residue in residues:
                hit_mass += count
            distance_mass += count * dist

        weighted_hit_rate = hit_mass / total if total > 0 else float("nan")
        weighted_mean_distance = distance_mass / total if total > 0 else float("nan")
        largest_gap_residue = largest_gap % prime
        largest_gap_distance = circular_distance_to_set(largest_gap_residue, residues, prime)
        most_common_gap_residue = most_common_gap % prime
        most_common_gap_distance = circular_distance_to_set(most_common_gap_residue, residues, prime)

        interaction_rows.append(
            {
                "tuplet_name": row["tuplet_name"],
                "offsets": row["offsets"],
                "parent_W": parent_W,
                "child_W": int(row["child_W"]),
                "new_prime": prime,
                "forbidden_residues": " ".join(str(x) for x in residues),
                "num_parent_return_gaps": len(gap_counts),
                "largest_parent_gap": largest_gap,
                "largest_parent_gap_residue_mod_p": largest_gap_residue,
                "largest_gap_forbidden_distance": largest_gap_distance,
                "most_common_parent_gap": most_common_gap,
                "most_common_parent_gap_count": most_common_count,
                "most_common_gap_residue_mod_p": most_common_gap_residue,
                "most_common_gap_forbidden_distance": most_common_gap_distance,
                "weighted_gap_forbidden_hit_rate": weighted_hit_rate,
                "weighted_gap_mean_forbidden_distance": weighted_mean_distance,
                "parent_admissible_density": float(row["parent_admissible_density"]),
                "gap_max": float(row["gap_max"]),
                "gap_entropy": float(row["gap_entropy"]),
                "local_allowed_count": float(row["local_allowed_count"]),
                "first_visible_split_H": row["first_visible_split_H"],
                "first_visible_split_lower_bound": int(row["first_visible_split_lower_bound"]),
                "visible_found_exact_within_range": int(row["visible_found_exact_within_range"]),
                "visible_status": row["visible_status"],
            }
        )

    exact_rows = [row for row in interaction_rows if row["visible_found_exact_within_range"] == 1]
    y = np.asarray([float(row["first_visible_split_H"]) for row in exact_rows], dtype=np.float64)
    predictors = {
        "new_prime": np.asarray([float(row["new_prime"]) for row in exact_rows], dtype=np.float64),
        "parent_admissible_density": np.asarray([float(row["parent_admissible_density"]) for row in exact_rows], dtype=np.float64),
        "local_allowed_count": np.asarray([float(row["local_allowed_count"]) for row in exact_rows], dtype=np.float64),
        "gap_max": np.asarray([float(row["gap_max"]) for row in exact_rows], dtype=np.float64),
        "gap_entropy": np.asarray([float(row["gap_entropy"]) for row in exact_rows], dtype=np.float64),
        "weighted_gap_forbidden_hit_rate": np.asarray([float(row["weighted_gap_forbidden_hit_rate"]) for row in exact_rows], dtype=np.float64),
        "weighted_gap_mean_forbidden_distance": np.asarray([float(row["weighted_gap_mean_forbidden_distance"]) for row in exact_rows], dtype=np.float64),
        "largest_gap_forbidden_distance": np.asarray([float(row["largest_gap_forbidden_distance"]) for row in exact_rows], dtype=np.float64),
        "most_common_gap_forbidden_distance": np.asarray([float(row["most_common_gap_forbidden_distance"]) for row in exact_rows], dtype=np.float64),
    }

    score_rows = []
    for name, values in predictors.items():
        score_rows.append(
            {
                "predictor": name,
                "pearson_visible_threshold": corrcoef(values, y),
                "spearman_visible_threshold": corrcoef(rankdata(values), rankdata(y)),
            }
        )
    score_rows.sort(key=lambda row: abs(float(row["spearman_visible_threshold"])), reverse=True)

    rows_path = OUT_DIR / "visible_threshold_interaction_stats.csv"
    scores_path = OUT_DIR / "visible_threshold_interaction_scores.csv"
    write_csv(rows_path, interaction_rows)
    write_csv(scores_path, score_rows)
    best = score_rows[0]
    print(f"best_interaction_predictor,{best['predictor']}")
    print(f"best_interaction_spearman,{float(best['spearman_visible_threshold']):.12f}")
    print(f"rows_csv,{rows_path}")
    print(f"scores_csv,{scores_path}")


if __name__ == "__main__":
    main()
