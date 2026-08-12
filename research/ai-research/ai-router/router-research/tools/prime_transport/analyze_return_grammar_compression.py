#!/usr/bin/env python3
"""Bounded return-grammar compression test for the spin residual."""

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
    length = W // 6
    j = np.arange(length, dtype=np.int64)
    mask = np.ones(length, dtype=np.uint8)
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


def purity_against_target(labels: list[tuple[object, ...]], target: list[tuple[object, ...]]) -> dict[str, float]:
    grouped: dict[tuple[object, ...], Counter[tuple[object, ...]]] = defaultdict(Counter)
    counts: Counter[tuple[object, ...]] = Counter()
    for label, tgt in zip(labels, target):
        grouped[label][tgt] += 1
        counts[label] += 1

    resolved_positions = 0
    weighted_multiplicity = 0.0
    for label, tgt_counts in grouped.items():
        resolved_positions += max(tgt_counts.values())
        weighted_multiplicity += counts[label] * len(tgt_counts)
    total = len(labels)
    weighted_multiplicity /= max(1, total)
    return {
        "purity_fraction": resolved_positions / max(1, total),
        "weighted_target_multiplicity": weighted_multiplicity,
        "label_count": float(len(grouped)),
    }


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def next_gap_and_anchor(bits: np.ndarray) -> tuple[list[int], list[int]]:
    length = bits.shape[0]
    gaps: list[int] = []
    anchors: list[int] = []
    for idx in range(length):
        gap = 0
        while gap < length and bits[(idx + gap) % length] == 0:
            gap += 1
        gaps.append(int(gap))
        anchors.append(int((idx + gap) % length))
    return gaps, anchors


def prev_gap(bits: np.ndarray) -> list[int]:
    length = bits.shape[0]
    gaps: list[int] = []
    for idx in range(length):
        gap = 0
        while gap < length and bits[(idx - gap) % length] == 0:
            gap += 1
        gaps.append(int(gap))
    return gaps


def next_admissible_gap_word(bits: np.ndarray) -> list[int]:
    length = bits.shape[0]
    admissible_positions = np.flatnonzero(bits).tolist()
    if not admissible_positions:
        return [length] * length
    gap_at_anchor = [0] * length
    for idx, pos in enumerate(admissible_positions):
        nxt = admissible_positions[(idx + 1) % len(admissible_positions)]
        gap = (nxt - pos) % length
        if gap == 0:
            gap = length
        gap_at_anchor[pos] = int(gap)
    return gap_at_anchor


def local_transition_3(bits: np.ndarray) -> list[tuple[int, int, int]]:
    length = bits.shape[0]
    return [
        (int(bits[(idx - 1) % length]), int(bits[idx]), int(bits[(idx + 1) % length]))
        for idx in range(length)
    ]


def main() -> None:
    matched_rows: list[dict[str, str]] = []
    for path in MATCHED_CSVS:
        with path.open("r", encoding="utf-8", newline="") as handle:
            matched_rows.extend(csv.DictReader(handle))

    detail_rows: list[dict[str, object]] = []
    score_rows: list[dict[str, object]] = []
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

        parent_labels = np.empty(parent_length, dtype=[("b", np.int16), ("code", np.uint64)])
        parent_labels["b"] = np.asarray(b_values, dtype=np.int16)
        parent_labels["code"] = spin_pre
        unique_parent, parent_class_ids = np.unique(parent_labels, return_inverse=True)

        child_visible = rolling_code_at(child_bits, visible_H)
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
        split_positions = np.flatnonzero((child_multiplicity > 1)[parent_class_ids]).tolist()

        target_membership = [int(idx in set(split_positions)) for idx in range(parent_length)]
        split_target = [(b_values[idx], int(spin_pre[idx])) for idx in split_positions]

        next_gap, anchors = next_gap_and_anchor(parent_bits)
        prev = prev_gap(parent_bits)
        anchor_gap = next_admissible_gap_word(parent_bits)
        next2 = [anchor_gap[anchor] for anchor in anchors]
        next3 = [anchor_gap[(anchors[idx] + next2[idx]) % parent_length] if parent_bits[anchors[idx]] else next2[idx] for idx in range(parent_length)]
        transition3 = local_transition_3(parent_bits)

        candidate_values = {
            "prev_next_gap_pair": [(prev[idx], next_gap[idx]) for idx in range(parent_length)],
            "next_return_gap": next_gap,
            "return_gap_seq_2": [(next_gap[idx], next2[idx]) for idx in range(parent_length)],
            "return_gap_seq_3": [(prev[idx], next_gap[idx], next2[idx]) for idx in range(parent_length)],
            "transition_memory_3": transition3,
            "branch_identity_min": [(next_gap[idx], next2[idx], int(parent_bits[(anchors[idx] + next2[idx]) % parent_length])) for idx in range(parent_length)],
        }

        b_only_membership = purity_against_target([(b,) for b in b_values], [(m,) for m in target_membership])
        b_only_split = purity_against_target([(b_values[idx],) for idx in split_positions], split_target)

        spin_full_membership = purity_against_target([(b_values[idx], int(spin_pre[idx])) for idx in range(parent_length)], [(m,) for m in target_membership])
        spin_full_split = purity_against_target(split_target, split_target)

        for candidate_name, values in candidate_values.items():
            membership_labels = [(b_values[idx], values[idx]) for idx in range(parent_length)]
            split_labels = [(b_values[idx], values[idx]) for idx in split_positions]
            membership_metrics = purity_against_target(membership_labels, [(m,) for m in target_membership])
            split_metrics = purity_against_target(split_labels, split_target)

            membership_gain = membership_metrics["purity_fraction"] - b_only_membership["purity_fraction"]
            split_gain = split_metrics["purity_fraction"] - b_only_split["purity_fraction"]
            max_membership_gain = spin_full_membership["purity_fraction"] - b_only_membership["purity_fraction"]
            max_split_gain = spin_full_split["purity_fraction"] - b_only_split["purity_fraction"]

            detail_rows.append(
                {
                    "tuplet_name": row["tuplet_name"],
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "visible_H_first": visible_H,
                    "candidate": candidate_name,
                    "membership_purity_fraction": membership_metrics["purity_fraction"],
                    "membership_gain_over_b_only": membership_gain,
                    "membership_capture_fraction_of_spin": membership_gain / max_membership_gain if max_membership_gain > 1e-12 else float("nan"),
                    "split_partition_purity_fraction": split_metrics["purity_fraction"],
                    "split_partition_gain_over_b_only": split_gain,
                    "split_partition_capture_fraction_of_spin": split_gain / max_split_gain if max_split_gain > 1e-12 else float("nan"),
                    "candidate_split_label_ratio_to_spin": split_metrics["label_count"] / max(1.0, spin_full_split["label_count"]),
                }
            )

    grouped: dict[str, list[dict[str, object]]] = defaultdict(list)
    for row in detail_rows:
        grouped[str(row["candidate"])].append(row)

    for candidate_name, rows in grouped.items():
        mem_capture = np.asarray([float(r["membership_capture_fraction_of_spin"]) for r in rows], dtype=np.float64)
        split_capture = np.asarray([float(r["split_partition_capture_fraction_of_spin"]) for r in rows], dtype=np.float64)
        label_ratio = np.asarray([float(r["candidate_split_label_ratio_to_spin"]) for r in rows], dtype=np.float64)
        score_rows.append(
            {
                "candidate": candidate_name,
                "mean_membership_capture_fraction_of_spin": float(np.nanmean(mem_capture)),
                "mean_split_partition_capture_fraction_of_spin": float(np.nanmean(split_capture)),
                "mean_combined_capture_fraction_of_spin": float(np.nanmean((mem_capture + split_capture) / 2.0)),
                "mean_split_label_ratio_to_spin": float(np.mean(label_ratio)),
            }
        )

    score_rows.sort(
        key=lambda row: (
            float(row["mean_combined_capture_fraction_of_spin"]),
            -float(row["mean_split_label_ratio_to_spin"]),
        ),
        reverse=True,
    )

    detail_path = OUT_DIR / "visible_threshold_return_grammar_comparison.csv"
    score_path = OUT_DIR / "visible_threshold_return_grammar_scores.csv"
    write_csv(detail_path, detail_rows)
    write_csv(score_path, score_rows)

    best = score_rows[0]
    print(f"best_candidate,{best['candidate']}")
    print(f"best_mean_combined_capture_fraction_of_spin,{float(best['mean_combined_capture_fraction_of_spin']):.12f}")
    print(f"best_mean_split_label_ratio_to_spin,{float(best['mean_split_label_ratio_to_spin']):.12f}")
    print(f"detail_csv,{detail_path}")
    print(f"score_csv,{score_path}")


if __name__ == "__main__":
    main()
