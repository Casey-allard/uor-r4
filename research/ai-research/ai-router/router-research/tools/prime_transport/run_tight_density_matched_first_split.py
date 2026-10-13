#!/usr/bin/env python3
"""Tight density-matched comparison for first-splitting vs arrangement statistics."""

from __future__ import annotations

import csv
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
OUT_DIR = ROOT / "results" / "prime_transport_recursive_system"
H_MAX = 60
MATCHED_ROWS = (
    ("quadruplet", (0, 2, 6, 8)),
    ("double_twins_p17", (0, 2, 34, 36)),
    ("quad_alt_0618", (0, 2, 6, 18)),
    ("quad_alt_0826", (0, 2, 8, 26)),
    ("quad_alt_1836", (0, 2, 18, 36)),
    ("quad_alt_3854", (0, 2, 38, 54)),
)
PARENT_W = 2310
CHILD_W = 30030
NEW_PRIME = 13


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


def rolling_codes(bits: np.ndarray, H_max: int):
    length = bits.shape[0]
    repeats = ((H_max + 1) // length) + 2
    extended = np.tile(bits, repeats)
    current = bits.astype(np.uint64)
    yield 1, current.copy(), extended[1 : 1 + length].astype(np.uint8)
    for horizon in range(2, H_max + 1):
        next_bit = extended[horizon - 1 : horizon - 1 + length].astype(np.uint64)
        current = (current << np.uint64(1)) | next_bit
        future_bit = extended[horizon : horizon + length].astype(np.uint8)
        yield horizon, current.copy(), future_bit


def rolling_code_at(bits: np.ndarray, horizon: int) -> np.ndarray:
    for current_h, codes, _ in rolling_codes(bits, horizon):
        if current_h == horizon:
            return codes
    raise RuntimeError("horizon not produced")


def unique_pair_count(base: np.ndarray, codes: np.ndarray) -> int:
    structured = np.empty(base.shape[0], dtype=[("b", np.int16), ("code", np.uint64)])
    structured["b"] = base
    structured["code"] = codes
    return int(np.unique(structured).shape[0])


def internal_split_active(child_codes: np.ndarray, child_next_bit: np.ndarray, parent_length: int, ratio: int) -> bool:
    code_matrix = child_codes.reshape(ratio, parent_length)
    next_matrix = child_next_bit.reshape(ratio, parent_length)
    split_mask = np.zeros(parent_length, dtype=bool)
    for left in range(ratio):
        for right in range(left + 1, ratio):
            split_mask |= (code_matrix[left] == code_matrix[right]) & (next_matrix[left] != next_matrix[right])
    return bool(split_mask.any())


def forbidden_residues(offsets: tuple[int, ...], prime: int) -> list[int]:
    inv6 = pow(6, -1, prime)
    return sorted({int(((-5 - offset) * inv6) % prime) for offset in offsets})


def gap_stats(forbidden: list[int], prime: int) -> tuple[str, int, int, int]:
    doubled = forbidden + [forbidden[0] + prime]
    gaps = [right - left for left, right in zip(doubled, doubled[1:])]
    gap_max = max(gaps)
    gap_span = prime - gap_max
    return " ".join(str(g) for g in sorted(gaps)), min(gaps), gap_max, gap_span


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


def spin_word(code: np.uint64, horizon: int) -> str:
    return format(int(code), f"0{horizon}b")


def analyze_tuplet(name: str, offsets: tuple[int, ...]) -> tuple[dict[str, object], list[dict[str, object]]]:
    parent_bits = admissible_bits(PARENT_W, offsets)
    child_bits = admissible_bits(CHILD_W, offsets)
    parent_length = PARENT_W // 6
    child_length = CHILD_W // 6
    ratio = child_length // parent_length
    parent_base = np.arange(parent_length, dtype=np.int64) % 35
    child_base = np.arange(child_length, dtype=np.int64) % 35

    first_internal = None
    first_visible = None
    for (h, parent_codes, _), (_, child_codes, child_next) in zip(
        rolling_codes(parent_bits, H_MAX),
        rolling_codes(child_bits, H_MAX),
        strict=True,
    ):
        internal = internal_split_active(child_codes, child_next, parent_length, ratio)
        visible = unique_pair_count(child_base, child_codes) > unique_pair_count(parent_base, parent_codes)
        if internal and first_internal is None:
            first_internal = h
        if visible and first_visible is None:
            first_visible = h
            break

    if first_visible is None:
        raise RuntimeError(f"no visible threshold found for {name}")

    pre_H = first_visible - 1
    parent_codes_pre = rolling_code_at(parent_bits, pre_H)
    child_codes_visible = rolling_code_at(child_bits, first_visible)

    parent_labels = np.empty(parent_length, dtype=[("b", np.int16), ("code", np.uint64)])
    parent_labels["b"] = parent_base
    parent_labels["code"] = parent_codes_pre
    unique_parent, parent_class_ids = np.unique(parent_labels, return_inverse=True)
    class_sizes = np.bincount(parent_class_ids)

    child_parent_class_ids = parent_class_ids[np.arange(child_length, dtype=np.int64) % parent_length]
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
    split_sizes = class_sizes[split_mask]
    split_multiplicity = child_class_multiplicity[split_mask]

    forbidden = forbidden_residues(offsets, NEW_PRIME)
    gap_multiset, gap_min, gap_max, gap_span = gap_stats(forbidden, NEW_PRIME)
    parent_density = float(parent_bits.mean())

    row = {
        "tuplet_name": name,
        "offsets": "[" + ",".join(str(x) for x in offsets) + "]",
        "parent_W": PARENT_W,
        "child_W": CHILD_W,
        "new_prime": NEW_PRIME,
        "parent_admissible_density": parent_density,
        "first_internal_split_H": first_internal,
        "first_visible_split_H": first_visible,
        "lag_visible_minus_internal": first_visible - first_internal,
        "forbidden_residues": " ".join(str(x) for x in forbidden),
        "gap_multiset": gap_multiset,
        "gap_min": gap_min,
        "gap_max": gap_max,
        "gap_span": gap_span,
        "num_parent_classes_pre_threshold": int(unique_parent.shape[0]),
        "num_first_splitting_classes": int(split_mask.sum()),
        "fraction_first_splitting_classes": float(split_mask.mean()),
        "fraction_parent_mass_in_first_splitting_classes": float(split_sizes.sum() / parent_length),
        "max_child_classes_from_first_split": int(split_multiplicity.max()),
        "total_extra_child_classes_from_first_split": int(np.maximum(split_multiplicity - 1, 0).sum()),
    }

    class_rows: list[dict[str, object]] = []
    for class_id in np.flatnonzero(split_mask):
        class_rows.append(
            {
                "tuplet_name": name,
                "offsets": row["offsets"],
                "visible_H_first": first_visible,
                "pre_threshold_H": pre_H,
                "parent_class_b": int(unique_parent["b"][class_id]),
                "parent_class_spin": spin_word(unique_parent["code"][class_id], pre_H),
                "parent_class_frequency": int(class_sizes[class_id]),
                "child_future_classes_produced": int(child_class_multiplicity[class_id]),
                "extra_child_classes_produced": int(child_class_multiplicity[class_id] - 1),
            }
        )
    return row, class_rows


def main() -> None:
    summary_rows: list[dict[str, object]] = []
    class_rows: list[dict[str, object]] = []
    for name, offsets in MATCHED_ROWS:
        summary, classes = analyze_tuplet(name, offsets)
        summary_rows.append(summary)
        class_rows.extend(classes)

    y = np.asarray([float(row["first_visible_split_H"]) for row in summary_rows], dtype=np.float64)
    predictors = {
        "num_first_splitting_classes": np.asarray([float(row["num_first_splitting_classes"]) for row in summary_rows], dtype=np.float64),
        "total_extra_child_classes_from_first_split": np.asarray([float(row["total_extra_child_classes_from_first_split"]) for row in summary_rows], dtype=np.float64),
        "fraction_first_splitting_classes": np.asarray([float(row["fraction_first_splitting_classes"]) for row in summary_rows], dtype=np.float64),
        "gap_max": np.asarray([float(row["gap_max"]) for row in summary_rows], dtype=np.float64),
        "gap_span": np.asarray([float(row["gap_span"]) for row in summary_rows], dtype=np.float64),
        "new_prime": np.asarray([float(row["new_prime"]) for row in summary_rows], dtype=np.float64),
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

    summary_path = OUT_DIR / "visible_threshold_tight_density_matched_rows.csv"
    classes_path = OUT_DIR / "visible_threshold_tight_density_matched_first_split_classes.csv"
    scores_path = OUT_DIR / "visible_threshold_tight_density_matched_scores.csv"
    write_csv(summary_path, summary_rows)
    write_csv(classes_path, class_rows)
    write_csv(scores_path, score_rows)

    best = score_rows[0]
    print(f"best_tight_matched_predictor,{best['predictor']}")
    print(f"best_tight_matched_spearman,{float(best['spearman_visible_threshold']):.12f}")
    print(f"summary_csv,{summary_path}")
    print(f"class_csv,{classes_path}")
    print(f"scores_csv,{scores_path}")


if __name__ == "__main__":
    main()
