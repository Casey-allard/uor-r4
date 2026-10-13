#!/usr/bin/env python3
"""Quantify exact predictive-state refinement and delayed visibility across wheel depth.

This script stays entirely at the exact recursive-system layer:
- admissibility orbit on n(j) = 5 + 6j
- base phase b(j) = j mod 35
- finite-horizon spin_H from the exact future word
- predictive state (b, spin_H)

It does not use any quotient geometry.
"""

from __future__ import annotations

import csv
from dataclasses import dataclass
from pathlib import Path

import numpy as np


OFFSETS = (0, 2, 6, 8)
OUT_DIR = Path(__file__).resolve().parents[2] / "results" / "prime_transport_recursive_system"
H_MAX = 60
PAIRS = (
    (30030, 510510),
    (510510, 9699690),
)


@dataclass(frozen=True)
class WheelState:
    W: int
    L: int
    bits: np.ndarray
    base: np.ndarray


def admissible_bits(W: int) -> np.ndarray:
    L = W // 6
    j = np.arange(L, dtype=np.int64)
    n = 5 + 6 * j
    mask = np.ones(L, dtype=bool)
    for offset in OFFSETS:
        mask &= np.gcd(n + offset, W) == 1
    return mask.astype(np.uint8)


def build_wheel_state(W: int) -> WheelState:
    L = W // 6
    return WheelState(
        W=W,
        L=L,
        bits=admissible_bits(W),
        base=np.arange(L, dtype=np.int64) % 35,
    )


def rolling_codes(bits: np.ndarray, H_max: int) -> dict[int, np.ndarray]:
    length = bits.shape[0]
    extended = np.concatenate((bits, bits[:H_max]))
    codes: dict[int, np.ndarray] = {}
    current = bits.astype(np.uint64)
    codes[1] = current.copy()
    for horizon in range(2, H_max + 1):
        next_bit = extended[horizon - 1 : horizon - 1 + length].astype(np.uint64)
        current = (current << np.uint64(1)) | next_bit
        codes[horizon] = current.copy()
    return codes


def unique_pair_count(base: np.ndarray, codes: np.ndarray) -> int:
    structured = np.empty(base.shape[0], dtype=[("b", np.int16), ("code", np.uint64)])
    structured["b"] = base
    structured["code"] = codes
    return int(np.unique(structured).shape[0])


def split_statistics(parent_codes: np.ndarray, child_codes: np.ndarray, parent_L: int, ratio: int) -> tuple[int, float]:
    if child_codes.shape[0] != parent_L * ratio:
        raise ValueError("child length is not an integer lift of parent length")
    lifted = child_codes.reshape(ratio, parent_L)
    first = lifted[0, :]
    split_mask = np.zeros(parent_L, dtype=bool)
    for row_idx in range(1, ratio):
        split_mask |= lifted[row_idx, :] != first
    split_count = int(split_mask.sum())
    consistency_rate = float(1.0 - (split_count / parent_L))
    return split_count, consistency_rate


def analyze_pair(parent: WheelState, child: WheelState, H_max: int) -> tuple[list[dict[str, object]], dict[str, object]]:
    if child.L % parent.L != 0:
        raise ValueError("child wheel is not a lift of parent wheel")
    ratio = child.L // parent.L
    parent_codes = rolling_codes(parent.bits, H_max)
    child_codes = rolling_codes(child.bits, H_max)
    detail_rows: list[dict[str, object]] = []
    first_internal = None
    first_visible = None

    for horizon in range(1, H_max + 1):
        p_codes = parent_codes[horizon]
        c_codes = child_codes[horizon]
        parent_spin_count = int(np.unique(p_codes).shape[0])
        child_spin_count = int(np.unique(c_codes).shape[0])
        parent_predictive_count = unique_pair_count(parent.base, p_codes)
        child_predictive_count = unique_pair_count(child.base, c_codes)
        split_count, consistency_rate = split_statistics(parent_codes=p_codes, child_codes=c_codes, parent_L=parent.L, ratio=ratio)
        internal_active = split_count > 0
        visible_active = child_predictive_count > parent_predictive_count
        if internal_active and first_internal is None:
            first_internal = horizon
        if visible_active and first_visible is None:
            first_visible = horizon
        detail_rows.append(
            {
                "parent_W": parent.W,
                "child_W": child.W,
                "refinement_ratio": ratio,
                "refinement_prime_guess": ratio,
                "H": horizon,
                "parent_spin_state_count": parent_spin_count,
                "child_spin_state_count": child_spin_count,
                "parent_predictive_state_count": parent_predictive_count,
                "child_predictive_state_count": child_predictive_count,
                "predictive_state_delta": child_predictive_count - parent_predictive_count,
                "split_parent_positions": split_count,
                "consistency_rate_under_projection": consistency_rate,
                "internal_split_active": int(internal_active),
                "visible_split_active": int(visible_active),
            }
        )

    summary_row = {
        "parent_W": parent.W,
        "child_W": child.W,
        "refinement_ratio": ratio,
        "refinement_prime_guess": ratio,
        "first_internal_split_H": first_internal if first_internal is not None else "",
        "first_visible_split_H": first_visible if first_visible is not None else "",
        "visibility_delay": (first_visible - first_internal) if first_internal is not None and first_visible is not None else "",
        "max_H_checked": H_max,
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
    wheel_states = {W: build_wheel_state(W) for pair in PAIRS for W in pair}
    detail_rows: list[dict[str, object]] = []
    summary_rows: list[dict[str, object]] = []
    for parent_W, child_W in PAIRS:
        detail, summary = analyze_pair(wheel_states[parent_W], wheel_states[child_W], H_MAX)
        detail_rows.extend(detail)
        summary_rows.append(summary)
    detail_path = OUT_DIR / "recursive_state_visibility_detail.csv"
    summary_path = OUT_DIR / "recursive_state_visibility_summary.csv"
    write_csv(detail_path, detail_rows)
    write_csv(summary_path, summary_rows)
    for row in summary_rows:
        print(
            ",".join(
                [
                    f"parent_W={row['parent_W']}",
                    f"child_W={row['child_W']}",
                    f"first_internal_split_H={row['first_internal_split_H']}",
                    f"first_visible_split_H={row['first_visible_split_H']}",
                    f"visibility_delay={row['visibility_delay']}",
                ]
            )
        )
    print(f"summary_csv,{summary_path}")


if __name__ == "__main__":
    main()
