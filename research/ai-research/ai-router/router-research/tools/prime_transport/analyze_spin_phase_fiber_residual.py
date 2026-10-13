#!/usr/bin/env python3
"""Analyze whether a small exact dynamical object can approximate spin classes."""

from __future__ import annotations

import ast
import csv
from collections import Counter, defaultdict
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


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def cyclic_suffix(bits: np.ndarray, length: int) -> list[tuple[int, ...]]:
    L = bits.shape[0]
    out: list[tuple[int, ...]] = []
    for idx in range(L):
        values = [int(bits[(idx - length + 1 + shift) % L]) for shift in range(length)]
        out.append(tuple(values))
    return out


def transition_triples(bits: np.ndarray) -> list[tuple[int, int, int]]:
    L = bits.shape[0]
    out: list[tuple[int, int, int]] = []
    for idx in range(L):
        out.append((int(bits[(idx - 1) % L]), int(bits[idx]), int(bits[(idx + 1) % L])))
    return out


def next_gap(bits: np.ndarray) -> list[int]:
    L = bits.shape[0]
    out: list[int] = []
    for idx in range(L):
        gap = 0
        while gap < L and bits[(idx + gap) % L] == 0:
            gap += 1
        out.append(int(gap))
    return out


def prev_gap(bits: np.ndarray) -> list[int]:
    L = bits.shape[0]
    out: list[int] = []
    for idx in range(L):
        gap = 0
        while gap < L and bits[(idx - gap) % L] == 0:
            gap += 1
        out.append(int(gap))
    return out


def pair_prev_next_gap(bits: np.ndarray) -> list[tuple[int, int]]:
    prev = prev_gap(bits)
    nxt = next_gap(bits)
    return list(zip(prev, nxt))


def phi_tuples_for_length(length: int) -> list[tuple[int, ...]]:
    fiber_mod = length // 35
    layer_primes = factor_prime_layers(fiber_mod)
    j = np.arange(length, dtype=np.int64)
    t_index = j // 35
    return [tuple(int(t % prime) for prime in layer_primes) for t in t_index.tolist()]


def purity_metrics(base_labels: list[tuple[object, ...]], target_labels: list[tuple[object, ...]]) -> dict[str, float]:
    grouped: dict[tuple[object, ...], set[tuple[object, ...]]] = defaultdict(set)
    counts: Counter[tuple[object, ...]] = Counter()
    for label, target in zip(base_labels, target_labels):
        grouped[label].add(target)
        counts[label] += 1

    unresolved_labels = [label for label, values in grouped.items() if len(values) > 1]
    unresolved_positions = sum(counts[label] for label in unresolved_labels)
    total_positions = len(base_labels)

    weighted_multiplicity = 0.0
    for label, values in grouped.items():
        weighted_multiplicity += counts[label] * len(values)
    weighted_multiplicity /= total_positions

    return {
        "unresolved_label_fraction": len(unresolved_labels) / max(1, len(grouped)),
        "unresolved_position_fraction": unresolved_positions / max(1, total_positions),
        "exact_purity_fraction": 1.0 - (unresolved_positions / max(1, total_positions)),
        "weighted_target_multiplicity": weighted_multiplicity,
        "label_count": float(len(grouped)),
    }


def main() -> None:
    matched_rows: list[dict[str, str]] = []
    for path in MATCHED_CSVS:
        with path.open("r", encoding="utf-8", newline="") as handle:
            matched_rows.extend(csv.DictReader(handle))

    residual_rows: list[dict[str, object]] = []
    summary_rows: list[dict[str, object]] = []
    bits_cache: dict[tuple[int, tuple[int, ...]], np.ndarray] = {}

    for row in matched_rows:
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
        b_values = (np.arange(parent_length, dtype=np.int64) % 35).tolist()

        spin_pre = rolling_code_at(parent_bits, pre_H)
        child_visible = rolling_code_at(child_bits, visible_H)

        parent_labels = np.empty(parent_length, dtype=[("b", np.int16), ("code", np.uint64)])
        parent_labels["b"] = np.asarray(b_values, dtype=np.int16)
        parent_labels["code"] = spin_pre
        unique_parent, parent_class_ids = np.unique(parent_labels, return_inverse=True)

        child_parent_ids = parent_class_ids[np.arange(child_length, dtype=np.int64) % parent_length]
        child_b = np.arange(child_length, dtype=np.int64) % 35
        child_pairs = np.empty(
            child_length,
            dtype=[("parent_id", np.int32), ("b", np.int16), ("code", np.uint64)],
        )
        child_pairs["parent_id"] = child_parent_ids.astype(np.int32)
        child_pairs["b"] = child_b.astype(np.int16)
        child_pairs["code"] = child_visible
        unique_child_pairs = np.unique(child_pairs)
        child_multiplicity = np.bincount(unique_child_pairs["parent_id"], minlength=unique_parent.shape[0])
        split_mask = child_multiplicity > 1
        split_positions = np.flatnonzero(split_mask[parent_class_ids]).tolist()

        split_target_labels = [(b_values[idx], int(spin_pre[idx])) for idx in split_positions]
        b_only_labels = [(b_values[idx],) for idx in split_positions]
        b_only = purity_metrics(b_only_labels, split_target_labels)

        candidate_values = {
            "recent_suffix_3": cyclic_suffix(parent_bits, 3),
            "recent_suffix_5": cyclic_suffix(parent_bits, 5),
            "local_transition_3": transition_triples(parent_bits),
            "next_return_gap": next_gap(parent_bits),
            "prev_next_gap_pair": pair_prev_next_gap(parent_bits),
        }

        summary_rows.append(
            {
                "tuplet_name": row["tuplet_name"],
                "offsets": row["offsets"],
                "parent_W": parent_W,
                "child_W": child_W,
                "visible_H_first": visible_H,
                "split_position_count": len(split_positions),
                "target_predictive_state_count": len(set(split_target_labels)),
                "b_only_exact_purity_fraction": b_only["exact_purity_fraction"],
                "b_only_unresolved_position_fraction": b_only["unresolved_position_fraction"],
                "b_only_weighted_target_multiplicity": b_only["weighted_target_multiplicity"],
                "b_only_label_count": b_only["label_count"],
            }
        )

        for candidate_name, values in candidate_values.items():
            split_candidate_labels = [(b_values[idx], values[idx]) for idx in split_positions]
            metrics = purity_metrics(split_candidate_labels, split_target_labels)
            residual_rows.append(
                {
                    "tuplet_name": row["tuplet_name"],
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "visible_H_first": visible_H,
                    "candidate": candidate_name,
                    "split_position_count": len(split_positions),
                    "target_predictive_state_count": len(set(split_target_labels)),
                    "b_only_exact_purity_fraction": b_only["exact_purity_fraction"],
                    "b_only_unresolved_position_fraction": b_only["unresolved_position_fraction"],
                    "candidate_exact_purity_fraction": metrics["exact_purity_fraction"],
                    "candidate_unresolved_position_fraction": metrics["unresolved_position_fraction"],
                    "purity_gain_over_b_only": metrics["exact_purity_fraction"] - b_only["exact_purity_fraction"],
                    "candidate_weighted_target_multiplicity": metrics["weighted_target_multiplicity"],
                    "candidate_label_count": metrics["label_count"],
                    "candidate_to_target_label_ratio": float(metrics["label_count"] / max(1, len(set(split_target_labels)))),
                }
            )

    candidate_groups: dict[str, list[dict[str, object]]] = defaultdict(list)
    for row in residual_rows:
        candidate_groups[str(row["candidate"])].append(row)

    score_rows: list[dict[str, object]] = []
    for candidate_name, rows in candidate_groups.items():
        gains = np.asarray([float(row["purity_gain_over_b_only"]) for row in rows], dtype=np.float64)
        visible = np.asarray([float(row["visible_H_first"]) for row in rows], dtype=np.float64)
        candidate_purity = np.asarray([float(row["candidate_exact_purity_fraction"]) for row in rows], dtype=np.float64)
        label_ratio = np.asarray([float(row["candidate_to_target_label_ratio"]) for row in rows], dtype=np.float64)
        score_rows.append(
            {
                "candidate": candidate_name,
                "mean_purity_gain_over_b_only": float(np.mean(gains)),
                "min_purity_gain_over_b_only": float(np.min(gains)),
                "max_purity_gain_over_b_only": float(np.max(gains)),
                "mean_candidate_exact_purity_fraction": float(np.mean(candidate_purity)),
                "mean_candidate_to_target_label_ratio": float(np.mean(label_ratio)),
                "spearman_visible_threshold": corrcoef(rankdata(candidate_purity), rankdata(visible)),
            }
        )
    score_rows.sort(
        key=lambda row: (
            float(row["mean_purity_gain_over_b_only"]),
            float(row["mean_candidate_exact_purity_fraction"]),
            -float(row["mean_candidate_to_target_label_ratio"]),
        ),
        reverse=True,
    )

    residual_path = OUT_DIR / "visible_threshold_spin_minus_phase_fiber_residuals.csv"
    summary_path = OUT_DIR / "visible_threshold_spin_minus_phase_fiber_summary.csv"
    score_path = OUT_DIR / "visible_threshold_spin_minus_phase_fiber_scores.csv"
    write_csv(residual_path, residual_rows)
    write_csv(summary_path, summary_rows)
    write_csv(score_path, score_rows)

    best = score_rows[0]
    print(f"best_candidate,{best['candidate']}")
    print(f"best_mean_purity_gain_over_b_only,{float(best['mean_purity_gain_over_b_only']):.12f}")
    print(f"best_mean_candidate_exact_purity,{float(best['mean_candidate_exact_purity_fraction']):.12f}")
    print(f"residual_csv,{residual_path}")
    print(f"summary_csv,{summary_path}")
    print(f"score_csv,{score_path}")


if __name__ == "__main__":
    main()
