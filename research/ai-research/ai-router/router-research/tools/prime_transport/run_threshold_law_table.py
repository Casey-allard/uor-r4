#!/usr/bin/env python3
"""Build a compact threshold-law table for exact recursive admissibility lifts.

This script stays entirely inside the exact recursive-system layer. It measures:
- first internal split horizon
- first visible split horizon
- lag = visible - internal

for successive wheel lifts and established tuplet patterns.
"""

from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path

import numpy as np


OUT_DIR = Path(__file__).resolve().parents[2] / "results" / "prime_transport_recursive_system"
H_MAX = 60
LIFTS = (
    (210, 2310, 11),
    (2310, 30030, 13),
    (30030, 510510, 17),
    (510510, 9699690, 19),
)
TUPLETS = (
    ("twins", (0, 2)),
    ("triplet", (0, 2, 6)),
    ("quadruplet", (0, 2, 6, 8)),
    ("double_twins_p13", (0, 2, 26, 28)),
    ("double_twins_p17", (0, 2, 34, 36)),
    ("double_twins_p19", (0, 2, 38, 40)),
)


@dataclass(frozen=True)
class WheelState:
    W: int
    L: int
    base: np.ndarray


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
    mask = np.ones(L, dtype=np.uint8)
    j = np.arange(L, dtype=np.int64)
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


def unique_pair_count(base: np.ndarray, codes: np.ndarray) -> int:
    structured = np.empty(base.shape[0], dtype=[("b", np.int16), ("code", np.uint64)])
    structured["b"] = base
    structured["code"] = codes
    return int(np.unique(structured).shape[0])


def internal_split_active(child_codes: np.ndarray, child_next_bit: np.ndarray, parent_length: int, ratio: int) -> tuple[bool, int]:
    code_matrix = child_codes.reshape(ratio, parent_length)
    next_matrix = child_next_bit.reshape(ratio, parent_length)
    split_mask = np.zeros(parent_length, dtype=bool)
    for left in range(ratio):
        for right in range(left + 1, ratio):
            split_mask |= (code_matrix[left] == code_matrix[right]) & (next_matrix[left] != next_matrix[right])
    split_positions = int(split_mask.sum())
    return bool(split_positions > 0), split_positions


def analyze_lift_and_tuplet(
    parent_W: int,
    child_W: int,
    new_prime: int,
    tuplet_name: str,
    offsets: tuple[int, ...],
    H_max: int,
    bit_cache: dict[tuple[int, tuple[int, ...]], np.ndarray],
) -> tuple[list[dict[str, object]], dict[str, object]]:
    parent_length = parent_W // 6
    child_length = child_W // 6
    ratio = child_length // parent_length
    if ratio != new_prime:
        raise ValueError("lift ratio does not match declared prime")

    parent_state = WheelState(parent_W, parent_length, np.arange(parent_length, dtype=np.int64) % 35)
    child_state = WheelState(child_W, child_length, np.arange(child_length, dtype=np.int64) % 35)
    parent_bits = bit_cache[(parent_W, offsets)]
    child_bits = bit_cache[(child_W, offsets)]

    parent_iter = rolling_codes(parent_bits, H_max)
    child_iter = rolling_codes(child_bits, H_max)
    detail_rows: list[dict[str, object]] = []
    first_internal = None
    first_visible = None

    for (H_p, parent_codes, _), (H_c, child_codes, child_next) in zip(parent_iter, child_iter, strict=True):
        if H_p != H_c:
            raise RuntimeError("parent/child horizon mismatch")
        horizon = H_p
        parent_predictive = unique_pair_count(parent_state.base, parent_codes)
        child_predictive = unique_pair_count(child_state.base, child_codes)
        visible_active = child_predictive > parent_predictive
        internal_active, split_parent_positions = internal_split_active(
            child_codes=child_codes,
            child_next_bit=child_next,
            parent_length=parent_length,
            ratio=ratio,
        )
        if internal_active and first_internal is None:
            first_internal = horizon
        if visible_active and first_visible is None:
            first_visible = horizon
        detail_rows.append(
            {
                "tuplet_name": tuplet_name,
                "offsets": " ".join(str(x) for x in offsets),
                "parent_W": parent_W,
                "child_W": child_W,
                "new_prime": new_prime,
                "H": horizon,
                "parent_predictive_state_count": parent_predictive,
                "child_predictive_state_count": child_predictive,
                "predictive_state_delta": child_predictive - parent_predictive,
                "internal_split_active": int(internal_active),
                "visible_split_active": int(visible_active),
                "internal_split_parent_positions": split_parent_positions,
            }
        )
        if first_internal is not None and first_visible is not None:
            break

    summary_row = {
        "tuplet_name": tuplet_name,
        "offsets": "[" + ",".join(str(x) for x in offsets) + "]",
        "tuplet_size": len(offsets),
        "parent_W": parent_W,
        "child_W": child_W,
        "new_prime": new_prime,
        "max_H_checked": H_max,
        "first_internal_split_H": first_internal if first_internal is not None else "",
        "first_visible_split_H": first_visible if first_visible is not None else "",
        "lag_visible_minus_internal": (
            first_visible - first_internal
            if first_internal is not None and first_visible is not None
            else ""
        ),
        "internal_found_exact_within_range": int(first_internal is not None),
        "visible_found_exact_within_range": int(first_visible is not None),
        "visible_status": "exact" if first_visible is not None else f"not_found_le_{H_max}",
    }
    return detail_rows, summary_row


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    bit_cache = {
        (W, offsets): admissible_bits(W, offsets)
        for W in sorted({value for lift in LIFTS for value in lift[:2]})
        for _, offsets in TUPLETS
    }
    detail_rows: list[dict[str, object]] = []
    summary_rows: list[dict[str, object]] = []
    for parent_W, child_W, new_prime in LIFTS:
        for tuplet_name, offsets in TUPLETS:
            detail, summary = analyze_lift_and_tuplet(
                parent_W=parent_W,
                child_W=child_W,
                new_prime=new_prime,
                tuplet_name=tuplet_name,
                offsets=offsets,
                H_max=H_MAX,
                bit_cache=bit_cache,
            )
            detail_rows.extend(detail)
            summary_rows.append(summary)

    detail_path = OUT_DIR / "threshold_law_detail.csv"
    summary_path = OUT_DIR / "threshold_law_summary.csv"
    write_csv(detail_path, detail_rows)
    write_csv(summary_path, summary_rows)
    for row in summary_rows:
        print(
            ",".join(
                [
                    f"tuplet={row['tuplet_name']}",
                    f"lift={row['parent_W']}->{row['child_W']}",
                    f"p={row['new_prime']}",
                    f"internal={row['first_internal_split_H']}",
                    f"visible={row['first_visible_split_H']}",
                    f"lag={row['lag_visible_minus_internal']}",
                    f"visible_status={row['visible_status']}",
                ]
            )
        )
    print(f"summary_csv,{summary_path}")


if __name__ == "__main__":
    main()
