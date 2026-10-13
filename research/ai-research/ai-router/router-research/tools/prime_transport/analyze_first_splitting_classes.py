#!/usr/bin/env python3
"""Analyze first splitting predictive classes at the visible threshold."""

from __future__ import annotations

import ast
import csv
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
SUMMARY_CSV = ROOT / "results" / "prime_transport_recursive_system" / "threshold_law_summary.csv"
PREDICTOR_ROWS_CSV = ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_predictor_rows.csv"
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"
DETAIL_MIN_VISIBLE_H = 13


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


def rolling_code_at(bits: np.ndarray, horizon: int) -> np.ndarray:
    if horizon < 1:
        raise ValueError("horizon must be >= 1")
    length = bits.shape[0]
    repeats = ((horizon + 1) // length) + 2
    extended = np.tile(bits, repeats)
    current = bits.astype(np.uint64)
    if horizon == 1:
        return current.copy()
    for step in range(2, horizon + 1):
        next_bit = extended[step - 1 : step - 1 + length].astype(np.uint64)
        current = (current << np.uint64(1)) | next_bit
    return current.copy()


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


def spin_word(code: np.uint64, horizon: int) -> str:
    return format(int(code), f"0{horizon}b")


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    with SUMMARY_CSV.open("r", encoding="utf-8", newline="") as handle:
        summary_rows = list(csv.DictReader(handle))
    with PREDICTOR_ROWS_CSV.open("r", encoding="utf-8", newline="") as handle:
        predictor_rows = {
            (
                row["tuplet_name"],
                int(row["parent_W"]),
                int(row["child_W"]),
            ): row
            for row in csv.DictReader(handle)
        }

    exact_rows = [
        row
        for row in summary_rows
        if row["visible_found_exact_within_range"] == "1" and row["first_visible_split_H"] and int(row["first_visible_split_H"]) > 1
    ]
    exact_rows.sort(
        key=lambda row: (
            int(row["first_visible_split_H"]),
            int(row["child_W"]),
            row["tuplet_name"],
        ),
        reverse=True,
    )

    bits_cache: dict[tuple[int, tuple[int, ...]], np.ndarray] = {}
    class_rows: list[dict[str, object]] = []
    event_rows: list[dict[str, object]] = []

    for row in exact_rows:
        tuplet_name = row["tuplet_name"]
        offsets = tuple(ast.literal_eval(row["offsets"]))
        parent_W = int(row["parent_W"])
        child_W = int(row["child_W"])
        new_prime = int(row["new_prime"])
        visible_H = int(row["first_visible_split_H"])
        pre_H = visible_H - 1
        predictor = predictor_rows[(tuplet_name, parent_W, child_W)]

        parent_key = (parent_W, offsets)
        child_key = (child_W, offsets)
        if parent_key not in bits_cache:
            bits_cache[parent_key] = admissible_bits(parent_W, offsets)
        if child_key not in bits_cache:
            bits_cache[child_key] = admissible_bits(child_W, offsets)

        parent_bits = bits_cache[parent_key]
        child_bits = bits_cache[child_key]
        parent_length = parent_W // 6
        child_length = child_W // 6
        ratio = child_length // parent_length

        parent_base = np.arange(parent_length, dtype=np.int64) % 35
        child_base = np.arange(child_length, dtype=np.int64) % 35

        parent_codes_pre = rolling_code_at(parent_bits, pre_H)
        child_codes_visible = rolling_code_at(child_bits, visible_H)

        parent_labels = np.empty(parent_length, dtype=[("b", np.int16), ("code", np.uint64)])
        parent_labels["b"] = parent_base
        parent_labels["code"] = parent_codes_pre
        unique_parent, parent_class_ids = np.unique(parent_labels, return_inverse=True)
        class_sizes = np.bincount(parent_class_ids)

        child_parent_class_ids = parent_class_ids[np.arange(child_length, dtype=np.int64) % parent_length]
        child_labels_visible = np.empty(child_length, dtype=[("b", np.int16), ("code", np.uint64)])
        child_labels_visible["b"] = child_base
        child_labels_visible["code"] = child_codes_visible

        child_pairs = np.empty(
            child_length,
            dtype=[("parent_id", np.int32), ("b", np.int16), ("code", np.uint64)],
        )
        child_pairs["parent_id"] = child_parent_class_ids.astype(np.int32)
        child_pairs["b"] = child_labels_visible["b"]
        child_pairs["code"] = child_labels_visible["code"]
        unique_child_pairs = np.unique(child_pairs)
        child_class_multiplicity = np.bincount(unique_child_pairs["parent_id"], minlength=unique_parent.shape[0])
        split_mask = child_class_multiplicity > 1

        num_parent_classes = int(unique_parent.shape[0])
        num_first_splitting_classes = int(split_mask.sum())
        split_sizes = class_sizes[split_mask]
        split_multiplicity = child_class_multiplicity[split_mask]
        split_mass = int(split_sizes.sum())
        total_extra_child_classes = int(np.maximum(split_multiplicity - 1, 0).sum()) if split_multiplicity.size else 0

        event_rows.append(
            {
                "tuplet_name": tuplet_name,
                "offsets": row["offsets"],
                "parent_W": parent_W,
                "child_W": child_W,
                "new_prime": new_prime,
                "visible_H_first": visible_H,
                "pre_threshold_H": pre_H,
                "num_parent_classes_pre_threshold": num_parent_classes,
                "num_first_splitting_classes": num_first_splitting_classes,
                "fraction_first_splitting_classes": (
                    num_first_splitting_classes / num_parent_classes if num_parent_classes > 0 else float("nan")
                ),
                "parent_mass_in_first_splitting_classes": split_mass,
                "fraction_parent_mass_in_first_splitting_classes": (
                    split_mass / parent_length if parent_length > 0 else float("nan")
                ),
                "max_first_splitting_class_size": int(split_sizes.max()) if split_sizes.size else 0,
                "mean_first_splitting_class_size": float(split_sizes.mean()) if split_sizes.size else 0.0,
                "max_child_classes_from_first_split": int(split_multiplicity.max()) if split_multiplicity.size else 0,
                "mean_child_classes_from_first_split": float(split_multiplicity.mean()) if split_multiplicity.size else 0.0,
                "total_extra_child_classes_from_first_split": total_extra_child_classes,
                "num_unsplit_parent_classes_pre_threshold": int(num_parent_classes - num_first_splitting_classes),
                "fraction_unsplit_parent_classes_pre_threshold": (
                    (num_parent_classes - num_first_splitting_classes) / num_parent_classes if num_parent_classes > 0 else float("nan")
                ),
                "parent_admissible_density": float(predictor["parent_admissible_density"]),
                "local_allowed_count": float(predictor["local_allowed_count"]),
                "gap_max": float(predictor["gap_max"]),
                "gap_entropy": float(predictor["gap_entropy"]),
                "predictive_state_delta_at_threshold": total_extra_child_classes,
            }
        )

        if visible_H < DETAIL_MIN_VISIBLE_H:
            continue

        for class_id in np.flatnonzero(split_mask):
            class_rows.append(
                {
                    "tuplet_name": tuplet_name,
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "new_prime": new_prime,
                    "visible_H_first": visible_H,
                    "pre_threshold_H": pre_H,
                    "parent_class_b": int(unique_parent["b"][class_id]),
                    "parent_class_spin": spin_word(unique_parent["code"][class_id], pre_H),
                    "parent_class_frequency": int(class_sizes[class_id]),
                    "parent_class_mass_fraction": float(class_sizes[class_id] / parent_length),
                    "child_future_classes_produced": int(child_class_multiplicity[class_id]),
                    "extra_child_classes_produced": int(child_class_multiplicity[class_id] - 1),
                    "visible_after_additional_horizon_step": 1,
                }
            )

    y = np.asarray([float(row["visible_H_first"]) for row in event_rows], dtype=np.float64)
    predictors = {
        "new_prime": np.asarray([float(row["new_prime"]) for row in event_rows], dtype=np.float64),
        "parent_admissible_density": np.asarray([float(row["parent_admissible_density"]) for row in event_rows], dtype=np.float64),
        "local_allowed_count": np.asarray([float(row["local_allowed_count"]) for row in event_rows], dtype=np.float64),
        "gap_max": np.asarray([float(row["gap_max"]) for row in event_rows], dtype=np.float64),
        "gap_entropy": np.asarray([float(row["gap_entropy"]) for row in event_rows], dtype=np.float64),
        "num_first_splitting_classes": np.asarray([float(row["num_first_splitting_classes"]) for row in event_rows], dtype=np.float64),
        "fraction_first_splitting_classes": np.asarray([float(row["fraction_first_splitting_classes"]) for row in event_rows], dtype=np.float64),
        "fraction_parent_mass_in_first_splitting_classes": np.asarray([float(row["fraction_parent_mass_in_first_splitting_classes"]) for row in event_rows], dtype=np.float64),
        "max_first_splitting_class_size": np.asarray([float(row["max_first_splitting_class_size"]) for row in event_rows], dtype=np.float64),
        "mean_first_splitting_class_size": np.asarray([float(row["mean_first_splitting_class_size"]) for row in event_rows], dtype=np.float64),
        "max_child_classes_from_first_split": np.asarray([float(row["max_child_classes_from_first_split"]) for row in event_rows], dtype=np.float64),
        "mean_child_classes_from_first_split": np.asarray([float(row["mean_child_classes_from_first_split"]) for row in event_rows], dtype=np.float64),
        "total_extra_child_classes_from_first_split": np.asarray([float(row["total_extra_child_classes_from_first_split"]) for row in event_rows], dtype=np.float64),
        "fraction_unsplit_parent_classes_pre_threshold": np.asarray([float(row["fraction_unsplit_parent_classes_pre_threshold"]) for row in event_rows], dtype=np.float64),
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
    score_rows.sort(key=lambda current: abs(float(current["spearman_visible_threshold"])), reverse=True)

    class_path = OUT_DIR / "visible_threshold_first_splitting_classes.csv"
    event_path = OUT_DIR / "visible_threshold_first_splitting_event_stats.csv"
    score_path = OUT_DIR / "visible_threshold_first_splitting_scores.csv"
    write_csv(class_path, class_rows)
    write_csv(event_path, event_rows)
    write_csv(score_path, score_rows)

    best = score_rows[0]
    print(f"best_first_split_predictor,{best['predictor']}")
    print(f"best_first_split_spearman,{float(best['spearman_visible_threshold']):.12f}")
    print(f"class_csv,{class_path}")
    print(f"event_csv,{event_path}")
    print(f"score_csv,{score_path}")


if __name__ == "__main__":
    main()
