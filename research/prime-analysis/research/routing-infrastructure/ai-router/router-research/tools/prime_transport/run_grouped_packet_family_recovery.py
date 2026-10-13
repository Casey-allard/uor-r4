#!/usr/bin/env python3
"""Run second-stage grouped-packet recovery experiments against the `C^2` target."""

from __future__ import annotations

import argparse
import csv
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np


RESULTS_DIR = Path(__file__).resolve().parents[2] / "results" / "prime_transport_grouped_packets"
DEFAULT_WHEELS = (30030, 510510)


@dataclass(frozen=True)
class FamilySpec:
    name: str
    encoder: str
    composition: str


FAMILIES = (
    FamilySpec("baseline_phase_conjugate_sum", "symmetric2", "uniform_sum_norm"),
    FamilySpec("single_mode_sum", "single1", "uniform_sum_norm"),
    FamilySpec("symmetric_sum", "symmetric2", "uniform_sum_norm"),
    FamilySpec("fourier4_sum", "fourier4", "uniform_sum_norm"),
    FamilySpec("symmetric_concat_linear", "symmetric2", "concat_linear"),
    FamilySpec("fourier4_concat_linear", "fourier4", "concat_linear"),
    FamilySpec("fourier4_adjacent_interactions", "fourier4", "adjacent_interactions"),
)


def complex_encoder(phi: np.ndarray, encoder_name: str) -> np.ndarray:
    phase = np.asarray(phi, dtype=np.float64)
    if encoder_name == "single1":
        return np.exp(1j * phase)[:, None]
    if encoder_name == "symmetric2":
        return np.column_stack((np.exp(1j * phase), np.exp(-1j * phase))) / math.sqrt(2.0)
    if encoder_name == "fourier4":
        return np.column_stack(
            (
                np.exp(1j * phase),
                np.exp(-1j * phase),
                np.exp(2j * phase),
                np.exp(-2j * phase),
            )
        ) / 2.0
    raise ValueError(f"unsupported encoder: {encoder_name}")


def load_dataset(path: Path) -> dict[str, np.ndarray]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        rows = list(reader)
    if not rows:
        raise ValueError(f"dataset is empty: {path}")
    phase_keys = sorted(
        (
            key
            for key in rows[0].keys()
            if key and key.startswith("phi_") and key.endswith("_angle")
        ),
        key=lambda key: int(key.split("_")[1]),
    )
    phase_data: list[np.ndarray] = []
    for key in phase_keys:
        values = [row[key] for row in rows]
        if all(str(value).strip() == "" for value in values):
            continue
        if any(str(value).strip() == "" for value in values):
            raise ValueError(f"partial blanks found in phase column {key}")
        phase_data.append(np.asarray([float(value) for value in values], dtype=np.float64))
    z = np.column_stack(
        (
            np.asarray([float(row["z1_real"]) for row in rows], dtype=np.float64)
            + 1j * np.asarray([float(row["z1_imag"]) for row in rows], dtype=np.float64),
            np.asarray([float(row["z2_real"]) for row in rows], dtype=np.float64)
            + 1j * np.asarray([float(row["z2_imag"]) for row in rows], dtype=np.float64),
        )
    )
    j = np.asarray([int(row["j"]) for row in rows], dtype=np.int64)
    return {"j": j, "z": z, "phis": phase_data}


def max_depth(datasets: dict[int, dict[str, np.ndarray]]) -> int:
    return max(len(dataset["phis"]) for dataset in datasets.values())


def build_features(dataset: dict[str, np.ndarray], family: FamilySpec, *, target_depth: int) -> np.ndarray:
    encoded_layers = [complex_encoder(phi, family.encoder) for phi in dataset["phis"]]
    packet_dim = encoded_layers[0].shape[1]
    zero_packet = np.zeros((dataset["z"].shape[0], packet_dim), dtype=np.complex128)
    padded_layers = encoded_layers + [zero_packet] * (target_depth - len(encoded_layers))

    if family.composition == "uniform_sum_norm":
        mixed = np.zeros_like(padded_layers[0])
        active_count = max(len(encoded_layers), 1)
        for packet in encoded_layers:
            mixed += packet
        mixed /= float(active_count)
        norms = np.linalg.norm(mixed, axis=1, keepdims=True)
        norms = np.where(norms <= 1e-12, 1.0, norms)
        return mixed / norms

    if family.composition == "concat_linear":
        return np.hstack(padded_layers)

    if family.composition == "adjacent_interactions":
        features = list(padded_layers)
        for left, right in zip(padded_layers[:-1], padded_layers[1:], strict=True):
            features.append(left * np.conj(right))
        return np.hstack(features)

    raise ValueError(f"unsupported composition: {family.composition}")


def augment_bias(X: np.ndarray) -> np.ndarray:
    return np.hstack((X, np.ones((X.shape[0], 1), dtype=np.complex128)))


def fit_linear_map(X: np.ndarray, z: np.ndarray) -> np.ndarray:
    X_aug = augment_bias(X)
    return np.linalg.lstsq(X_aug, z, rcond=None)[0]


def predict_linear_map(X: np.ndarray, B: np.ndarray) -> np.ndarray:
    return augment_bias(X) @ B


def relative_error(z_true: np.ndarray, z_pred: np.ndarray) -> float:
    denom = np.linalg.norm(z_true)
    if denom <= 1e-12:
        return 0.0
    return float(np.linalg.norm(z_true - z_pred) / denom)


def split_masks(j: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    test_mask = (j % 5) == 0
    train_mask = ~test_mask
    return train_mask, test_mask


def run_family(
    family: FamilySpec,
    datasets: dict[int, dict[str, np.ndarray]],
    *,
    target_depth: int,
) -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    features_by_wheel = {
        wheel: build_features(dataset, family, target_depth=target_depth)
        for wheel, dataset in datasets.items()
    }

    for wheel in sorted(datasets):
        dataset = datasets[wheel]
        X = features_by_wheel[wheel]
        z = dataset["z"]
        train_mask, test_mask = split_masks(dataset["j"])
        B = fit_linear_map(X[train_mask], z[train_mask])
        z_train_pred = predict_linear_map(X[train_mask], B)
        z_test_pred = predict_linear_map(X[test_mask], B)
        train_err = relative_error(z[train_mask], z_train_pred)
        test_err = relative_error(z[test_mask], z_test_pred)
        rows.append(
            {
                "family": family.name,
                "encoder": family.encoder,
                "composition": family.composition,
                "train_wheel": wheel,
                "eval_wheel": wheel,
                "evaluation_kind": "within_scale",
                "train_error": train_err,
                "test_error": test_err,
                "generalization_gap": test_err - train_err,
                "feature_dim": X.shape[1],
            }
        )

    if 30030 in datasets and 510510 in datasets:
        source = datasets[30030]
        X_source = features_by_wheel[30030]
        train_mask, test_mask = split_masks(source["j"])
        B = fit_linear_map(X_source[train_mask], source["z"][train_mask])
        target = datasets[510510]
        X_target = features_by_wheel[510510]
        target_train_mask, target_test_mask = split_masks(target["j"])
        transfer_train_err = relative_error(
            target["z"][target_train_mask],
            predict_linear_map(X_target[target_train_mask], B),
        )
        transfer_test_err = relative_error(
            target["z"][target_test_mask],
            predict_linear_map(X_target[target_test_mask], B),
        )
        rows.append(
            {
                "family": family.name,
                "encoder": family.encoder,
                "composition": family.composition,
                "train_wheel": 30030,
                "eval_wheel": 510510,
                "evaluation_kind": "cross_scale_transfer",
                "train_error": transfer_train_err,
                "test_error": transfer_test_err,
                "generalization_gap": transfer_test_err - transfer_train_err,
                "feature_dim": X_target.shape[1],
            }
        )

    return rows


def summary_rows(detail_rows: list[dict[str, object]]) -> list[dict[str, object]]:
    families = sorted({row["family"] for row in detail_rows})
    out: list[dict[str, object]] = []
    for family in families:
        within = [row for row in detail_rows if row["family"] == family and row["evaluation_kind"] == "within_scale"]
        transfer = [row for row in detail_rows if row["family"] == family and row["evaluation_kind"] == "cross_scale_transfer"]
        best_within = min(float(row["test_error"]) for row in within)
        avg_within = sum(float(row["test_error"]) for row in within) / len(within)
        transfer_test = float(transfer[0]["test_error"]) if transfer else math.nan
        transfer_train = float(transfer[0]["train_error"]) if transfer else math.nan
        out.append(
            {
                "family": family,
                "encoder": within[0]["encoder"],
                "composition": within[0]["composition"],
                "best_within_scale_test_error": best_within,
                "avg_within_scale_test_error": avg_within,
                "cross_scale_transfer_test_error_30030_to_510510": transfer_test,
                "cross_scale_transfer_train_error_on_510510": transfer_train,
                "feature_dim": int(within[0]["feature_dim"]),
            }
        )
    out.sort(key=lambda row: (row["best_within_scale_test_error"], row["cross_scale_transfer_test_error_30030_to_510510"]))
    return out


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError("no rows to write")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--dataset",
        dest="datasets",
        type=Path,
        action="append",
        help="Dataset CSV path. Repeat for multiple scales.",
    )
    args = parser.parse_args()

    dataset_paths = args.datasets or [
        RESULTS_DIR / "prime_transport_layer_packet_dataset_W30030.csv",
        RESULTS_DIR / "prime_transport_layer_packet_dataset_W510510.csv",
    ]

    datasets: dict[int, dict[str, np.ndarray]] = {}
    for path in dataset_paths:
        dataset = load_dataset(path)
        wheel = int(path.stem.split("_W")[-1])
        datasets[wheel] = dataset

    target_depth = max_depth(datasets)
    detail_rows: list[dict[str, object]] = []
    for family in FAMILIES:
        detail_rows.extend(run_family(family, datasets, target_depth=target_depth))

    summary = summary_rows(detail_rows)
    write_csv(RESULTS_DIR / "grouped_packet_family_recovery_detail.csv", detail_rows)
    write_csv(RESULTS_DIR / "grouped_packet_family_recovery_summary.csv", summary)

    best = summary[0]
    print(
        "best_family,"
        f"{best['family']},"
        f"{best['best_within_scale_test_error']:.12f},"
        f"{best['cross_scale_transfer_test_error_30030_to_510510']:.12f}"
    )


if __name__ == "__main__":
    main()
