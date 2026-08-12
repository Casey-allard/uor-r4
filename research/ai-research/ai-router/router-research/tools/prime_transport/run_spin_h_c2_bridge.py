#!/usr/bin/env python3
"""Compare finite-horizon spin recovery against grouped-packet baselines.

This is a bounded research script. It treats the recursive dynamical system as
the primary object and the empirical `C^2` backbone as a downstream quotient
candidate.
"""

from __future__ import annotations

import argparse
import csv
from collections import defaultdict
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
DATA_DIR = ROOT / "results" / "prime_transport_grouped_packets"
OUT_DIR = ROOT / "results" / "prime_transport_spin_bridge"
PACKET_SUMMARY = DATA_DIR / "grouped_packet_family_recovery_summary.csv"
DEFAULT_WHEELS = (30030, 510510)
DEFAULT_HORIZONS = (8, 16, 32, 56)


def load_dataset(path: Path) -> dict[str, np.ndarray]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        raise ValueError(f"dataset is empty: {path}")
    return {
        "W": np.asarray([int(row["W"]) for row in rows], dtype=np.int64),
        "j": np.asarray([int(row["j"]) for row in rows], dtype=np.int64),
        "b": np.asarray([int(row["b_mod_35"]) for row in rows], dtype=np.int64),
        "admissible": np.asarray([int(row["admissible_bit"]) for row in rows], dtype=np.int8),
        "z": np.column_stack(
            (
                np.asarray([float(row["z1_real"]) for row in rows], dtype=np.float64)
                + 1j * np.asarray([float(row["z1_imag"]) for row in rows], dtype=np.float64),
                np.asarray([float(row["z2_real"]) for row in rows], dtype=np.float64)
                + 1j * np.asarray([float(row["z2_imag"]) for row in rows], dtype=np.float64),
            )
        ),
    }


def relative_error(z_true: np.ndarray, z_hat: np.ndarray) -> float:
    denom = np.linalg.norm(z_true)
    if denom <= 1e-12:
        return 0.0
    return float(np.linalg.norm(z_true - z_hat) / denom)


def build_spin_words(admissible: np.ndarray, horizon: int) -> list[str]:
    length = admissible.shape[0]
    doubled = np.concatenate((admissible, admissible[: horizon - 1]))
    return [
        "".join("1" if bit else "0" for bit in doubled[idx : idx + horizon])
        for idx in range(length)
    ]


def make_keys(kind: str, b: np.ndarray, spin_words: list[str]) -> list[str]:
    if kind == "spin_only":
        return spin_words
    if kind == "base_spin":
        return [f"{int(base)}|{word}" for base, word in zip(b, spin_words, strict=True)]
    raise ValueError(f"unknown kind: {kind}")


def fit_keyed_mean(keys: list[str], z: np.ndarray, mask: np.ndarray) -> tuple[dict[str, np.ndarray], np.ndarray]:
    sums: dict[str, np.ndarray] = {}
    counts: dict[str, int] = {}
    for key, value, keep in zip(keys, z, mask, strict=True):
        if not bool(keep):
            continue
        if key not in sums:
            sums[key] = np.zeros(2, dtype=np.complex128)
            counts[key] = 0
        sums[key] += value
        counts[key] += 1
    lookup = {key: sums[key] / counts[key] for key in sums}
    global_mean = z[mask].mean(axis=0) if np.any(mask) else np.zeros(2, dtype=np.complex128)
    return lookup, global_mean


def predict_keyed_mean(keys: list[str], lookup: dict[str, np.ndarray], default: np.ndarray) -> np.ndarray:
    return np.asarray([lookup.get(key, default) for key in keys], dtype=np.complex128)


def evaluate_within_scale(data: dict[str, np.ndarray], horizon: int, family: str) -> dict[str, object]:
    spin_words = build_spin_words(data["admissible"], horizon)
    keys = make_keys(family, data["b"], spin_words)
    test_mask = (data["j"] % 5) == 0
    train_mask = ~test_mask
    lookup, default = fit_keyed_mean(keys, data["z"], train_mask)
    z_hat_train = predict_keyed_mean(keys, lookup, default)[train_mask]
    z_hat_test = predict_keyed_mean(keys, lookup, default)[test_mask]
    return {
        "evaluation": "within_scale",
        "family": family,
        "horizon": horizon,
        "W_train": int(data["W"][0]),
        "W_test": int(data["W"][0]),
        "train_error": relative_error(data["z"][train_mask], z_hat_train),
        "test_error": relative_error(data["z"][test_mask], z_hat_test),
        "num_states_train": len(set(key for key, keep in zip(keys, train_mask, strict=True) if keep)),
        "num_states_test": len(set(key for key, keep in zip(keys, test_mask, strict=True) if keep)),
    }


def evaluate_cross_scale(
    train_data: dict[str, np.ndarray],
    test_data: dict[str, np.ndarray],
    horizon: int,
    family: str,
) -> dict[str, object]:
    train_keys = make_keys(family, train_data["b"], build_spin_words(train_data["admissible"], horizon))
    test_keys = make_keys(family, test_data["b"], build_spin_words(test_data["admissible"], horizon))
    train_mask = (train_data["j"] % 5) != 0
    lookup, default = fit_keyed_mean(train_keys, train_data["z"], train_mask)
    z_hat_train = predict_keyed_mean(train_keys, lookup, default)[train_mask]
    z_hat_test = predict_keyed_mean(test_keys, lookup, default)
    return {
        "evaluation": "cross_scale",
        "family": family,
        "horizon": horizon,
        "W_train": int(train_data["W"][0]),
        "W_test": int(test_data["W"][0]),
        "train_error": relative_error(train_data["z"][train_mask], z_hat_train),
        "test_error": relative_error(test_data["z"], z_hat_test),
        "num_states_train": len(set(key for key, keep in zip(train_keys, train_mask, strict=True) if keep)),
        "num_states_test": len(set(test_keys)),
    }


def load_packet_baseline(path: Path) -> dict[str, float]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        raise ValueError(f"packet summary is empty: {path}")
    best_within = min(float(row["best_within_scale_test_error"]) for row in rows)
    best_cross = min(float(row["cross_scale_transfer_test_error_30030_to_510510"]) for row in rows)
    return {
        "best_packet_within_scale_test_error": best_within,
        "best_packet_cross_scale_test_error": best_cross,
    }


def summarize(rows: list[dict[str, object]]) -> list[dict[str, object]]:
    grouped: dict[tuple[str, str], list[dict[str, object]]] = defaultdict(list)
    for row in rows:
        grouped[(str(row["evaluation"]), str(row["family"]))].append(row)
    summary_rows: list[dict[str, object]] = []
    for (evaluation, family), items in sorted(grouped.items()):
        best = min(items, key=lambda item: float(item["test_error"]))
        summary_rows.append(
            {
                "evaluation": evaluation,
                "family": family,
                "best_horizon": int(best["horizon"]),
                "best_test_error": float(best["test_error"]),
                "avg_test_error": float(np.mean([float(item["test_error"]) for item in items])),
                "best_train_error": float(best["train_error"]),
                "avg_num_states_train": float(np.mean([float(item["num_states_train"]) for item in items])),
            }
        )
    return summary_rows


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheel", dest="wheels", type=int, action="append", default=None)
    parser.add_argument("--horizon", dest="horizons", type=int, action="append", default=None)
    args = parser.parse_args()

    wheels = tuple(args.wheels or DEFAULT_WHEELS)
    horizons = tuple(args.horizons or DEFAULT_HORIZONS)
    datasets = {
        wheel: load_dataset(DATA_DIR / f"prime_transport_layer_packet_dataset_W{wheel}.csv")
        for wheel in wheels
    }

    detail_rows: list[dict[str, object]] = []
    for wheel in wheels:
        for horizon in horizons:
            for family in ("spin_only", "base_spin"):
                detail_rows.append(evaluate_within_scale(datasets[wheel], horizon, family))

    if 30030 in datasets and 510510 in datasets:
        for horizon in horizons:
            for family in ("spin_only", "base_spin"):
                detail_rows.append(evaluate_cross_scale(datasets[30030], datasets[510510], horizon, family))

    summary_rows = summarize(detail_rows)
    packet = load_packet_baseline(PACKET_SUMMARY)
    comparison_rows = []
    for row in summary_rows:
        comparison_rows.append(
            {
                **row,
                "best_packet_within_scale_test_error": packet["best_packet_within_scale_test_error"],
                "best_packet_cross_scale_test_error": packet["best_packet_cross_scale_test_error"],
            }
        )

    detail_path = OUT_DIR / "spin_h_c2_bridge_detail.csv"
    summary_path = OUT_DIR / "spin_h_c2_bridge_summary.csv"
    write_csv(detail_path, detail_rows)
    write_csv(summary_path, comparison_rows)

    best_within = min(
        (row for row in comparison_rows if row["evaluation"] == "within_scale"),
        key=lambda item: float(item["best_test_error"]),
    )
    best_cross = min(
        (row for row in comparison_rows if row["evaluation"] == "cross_scale"),
        key=lambda item: float(item["best_test_error"]),
    )
    print(f"best_within_family,{best_within['family']}")
    print(f"best_within_horizon,{best_within['best_horizon']}")
    print(f"best_within_error,{float(best_within['best_test_error']):.12f}")
    print(f"best_cross_family,{best_cross['family']}")
    print(f"best_cross_horizon,{best_cross['best_horizon']}")
    print(f"best_cross_error,{float(best_cross['best_test_error']):.12f}")
    print(f"summary_csv,{summary_path}")


if __name__ == "__main__":
    main()
