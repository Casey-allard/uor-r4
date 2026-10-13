#!/usr/bin/env python3
"""Map first-splitting classes into phase-fiber-scale coordinates."""

from __future__ import annotations

import ast
import csv
from collections import Counter
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
MATCHED_CSVS = (
    ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_tight_density_matched_rows.csv",
    ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_second_tight_density_matched_rows.csv",
)
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
        forbidden = {int(((-5 - offset) * inv6) % prime) for offset in offsets}
        allowed = np.ones(prime, dtype=np.uint8)
        for residue in forbidden:
            allowed[residue] = 0
        mask &= allowed[j % prime]
    return mask


def rolling_code_at(bits: np.ndarray, horizon: int) -> np.ndarray:
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


def factor_prime_layers(value: int) -> list[int]:
    remaining = int(value)
    factors: list[int] = []
    divisor = 2
    while divisor * divisor <= remaining:
        while remaining % divisor == 0:
            factors.append(divisor)
            remaining //= divisor
        divisor += 1 if divisor == 2 else 2
    if remaining > 1:
        factors.append(remaining)
    return factors


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


def top_counter_share(values: list[object]) -> tuple[object, float]:
    counts = Counter(values)
    top_value, top_count = max(counts.items(), key=lambda item: (item[1], str(item[0])))
    return top_value, top_count / len(values)


def format_phi_tuple(phi_tuple: tuple[int, ...]) -> str:
    if not phi_tuple:
        return "()"
    return "(" + ",".join(str(v) for v in phi_tuple) + ")"


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    matched_rows: list[dict[str, str]] = []
    for path in MATCHED_CSVS:
        with path.open("r", encoding="utf-8", newline="") as handle:
            matched_rows.extend(csv.DictReader(handle))

    class_rows: list[dict[str, object]] = []
    summary_rows: list[dict[str, object]] = []
    bits_cache: dict[tuple[int, tuple[int, ...]], np.ndarray] = {}

    for row in matched_rows:
        tuplet_name = row["tuplet_name"]
        offsets = tuple(ast.literal_eval(row["offsets"]))
        parent_W = int(row["parent_W"])
        child_W = int(row["child_W"])
        visible_H = int(row["first_visible_split_H"])
        pre_H = visible_H - 1

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
        fiber_mod = parent_length // 35
        layer_primes = factor_prime_layers(fiber_mod)

        j_parent = np.arange(parent_length, dtype=np.int64)
        parent_base = j_parent % 35
        t_index = j_parent // 35
        phi_tuples = [tuple(int(t % prime) for prime in layer_primes) for t in t_index.tolist()]
        boundary_any = [
            int(any((value in {0, 1, prime - 2, prime - 1}) for value, prime in zip(phi_tuple, layer_primes)))
            for phi_tuple in phi_tuples
        ]

        parent_codes_pre = rolling_code_at(parent_bits, pre_H)
        child_codes_visible = rolling_code_at(child_bits, visible_H)

        parent_labels = np.empty(parent_length, dtype=[("b", np.int16), ("code", np.uint64)])
        parent_labels["b"] = parent_base
        parent_labels["code"] = parent_codes_pre
        unique_parent, parent_class_ids = np.unique(parent_labels, return_inverse=True)
        class_sizes = np.bincount(parent_class_ids)

        child_parent_class_ids = parent_class_ids[np.arange(child_length, dtype=np.int64) % parent_length]
        child_base = np.arange(child_length, dtype=np.int64) % 35
        child_labels = np.empty(child_length, dtype=[("b", np.int16), ("code", np.uint64)])
        child_labels["b"] = child_base
        child_labels["code"] = child_codes_visible
        child_pairs = np.empty(
            child_length,
            dtype=[("parent_id", np.int32), ("b", np.int16), ("code", np.uint64)],
        )
        child_pairs["parent_id"] = child_parent_class_ids.astype(np.int32)
        child_pairs["b"] = child_labels["b"]
        child_pairs["code"] = child_labels["code"]
        unique_child_pairs = np.unique(child_pairs)
        child_class_multiplicity = np.bincount(unique_child_pairs["parent_id"], minlength=unique_parent.shape[0])
        split_mask = child_class_multiplicity > 1
        split_class_ids = np.flatnonzero(split_mask)

        split_position_mask = split_mask[parent_class_ids]
        split_indices = np.flatnonzero(split_position_mask)
        split_phi_values = [phi_tuples[idx] for idx in split_indices.tolist()]
        all_phi_values = phi_tuples
        split_b_values = parent_base[split_indices].tolist()

        top_split_phi, top_split_phi_share = top_counter_share(split_phi_values)
        top_split_b, top_split_b_share = top_counter_share(split_b_values)
        split_boundary_fraction = float(np.mean([boundary_any[idx] for idx in split_indices.tolist()])) if split_indices.size else float("nan")
        all_boundary_fraction = float(np.mean(boundary_any))
        split_phi_support_fraction = len(set(split_phi_values)) / len(set(all_phi_values))
        split_base_support_fraction = len(set(split_b_values)) / 35.0

        summary_rows.append(
            {
                "tuplet_name": tuplet_name,
                "offsets": row["offsets"],
                "parent_W": parent_W,
                "child_W": child_W,
                "visible_H_first": visible_H,
                "layer_depth": len(layer_primes),
                "layer_primes": " ".join(str(p) for p in layer_primes),
                "num_first_splitting_classes": int(split_class_ids.size),
                "split_position_mass_fraction": float(split_indices.size / parent_length),
                "split_phi_tuple_support_fraction": float(split_phi_support_fraction),
                "split_base_support_fraction": float(split_base_support_fraction),
                "top_split_phi_tuple": format_phi_tuple(top_split_phi),
                "top_split_phi_tuple_share": float(top_split_phi_share),
                "top_split_base_phase": int(top_split_b),
                "top_split_base_share": float(top_split_b_share),
                "split_boundary_any_fraction": float(split_boundary_fraction),
                "all_boundary_any_fraction": float(all_boundary_fraction),
                "boundary_enrichment_ratio": float(split_boundary_fraction / all_boundary_fraction) if all_boundary_fraction > 0 else float("nan"),
            }
        )

        for class_id in split_class_ids.tolist():
            position_idx = np.flatnonzero(parent_class_ids == class_id)
            class_phi_values = [phi_tuples[idx] for idx in position_idx.tolist()]
            class_boundary_fraction = float(np.mean([boundary_any[idx] for idx in position_idx.tolist()]))
            dominant_phi, dominant_phi_share = top_counter_share(class_phi_values)
            class_rows.append(
                {
                    "tuplet_name": tuplet_name,
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "visible_H_first": visible_H,
                    "layer_depth": len(layer_primes),
                    "layer_primes": " ".join(str(p) for p in layer_primes),
                    "parent_class_b": int(unique_parent["b"][class_id]),
                    "parent_class_spin_code": int(unique_parent["code"][class_id]),
                    "parent_class_frequency": int(class_sizes[class_id]),
                    "child_future_classes_produced": int(child_class_multiplicity[class_id]),
                    "dominant_phi_tuple": format_phi_tuple(dominant_phi),
                    "dominant_phi_tuple_share": float(dominant_phi_share),
                    "phi_tuple_support_size": int(len(set(class_phi_values))),
                    "boundary_any_fraction": float(class_boundary_fraction),
                }
            )

    y = np.asarray([float(row["visible_H_first"]) for row in summary_rows], dtype=np.float64)
    predictors = {
        "top_split_phi_tuple_share": np.asarray([float(row["top_split_phi_tuple_share"]) for row in summary_rows], dtype=np.float64),
        "top_split_base_share": np.asarray([float(row["top_split_base_share"]) for row in summary_rows], dtype=np.float64),
        "split_phi_tuple_support_fraction": np.asarray([float(row["split_phi_tuple_support_fraction"]) for row in summary_rows], dtype=np.float64),
        "split_base_support_fraction": np.asarray([float(row["split_base_support_fraction"]) for row in summary_rows], dtype=np.float64),
        "boundary_enrichment_ratio": np.asarray([float(row["boundary_enrichment_ratio"]) for row in summary_rows], dtype=np.float64),
        "num_first_splitting_classes": np.asarray([float(row["num_first_splitting_classes"]) for row in summary_rows], dtype=np.float64),
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

    class_path = OUT_DIR / "visible_threshold_phase_fiber_mapped_first_split_classes.csv"
    summary_path = OUT_DIR / "visible_threshold_phase_fiber_readability_summary.csv"
    score_path = OUT_DIR / "visible_threshold_phase_fiber_readability_scores.csv"
    write_csv(class_path, class_rows)
    write_csv(summary_path, summary_rows)
    write_csv(score_path, score_rows)

    best = score_rows[0]
    print(f"best_phase_fiber_predictor,{best['predictor']}")
    print(f"best_phase_fiber_spearman,{float(best['spearman_visible_threshold']):.12f}")
    print(f"class_csv,{class_path}")
    print(f"summary_csv,{summary_path}")
    print(f"score_csv,{score_path}")


if __name__ == "__main__":
    main()
