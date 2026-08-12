#!/usr/bin/env python3
"""Scaffold for grouped layer-packet experiments against the prime `C^2` backbone.

This file does not claim that packet composition already explains the backbone.
It defines the smallest auditable objects needed to test that question.
"""

from __future__ import annotations

import argparse
import csv
import math
from dataclasses import dataclass
from pathlib import Path
import re

import numpy as np


@dataclass(frozen=True)
class LayerPacketConfig:
    packet_kind: str = "phase_conjugate_pair"
    composition_kind: str = "normalized_weighted_sum"


def phase_conjugate_packet(phi: np.ndarray) -> np.ndarray:
    """Encode one layer phase as a normalized 2-complex-component packet."""
    phase = np.asarray(phi, dtype=np.float64)
    return np.column_stack((np.exp(1j * phase), np.exp(-1j * phase))) / math.sqrt(2.0)


def compose_packets(
    packets: list[np.ndarray],
    *,
    weights: np.ndarray | None = None,
) -> np.ndarray:
    """Compose per-layer packets by normalized weighted addition."""
    if not packets:
        raise ValueError("packets must be non-empty")
    length = packets[0].shape[0]
    for packet in packets:
        if packet.shape != (length, 2):
            raise ValueError("each packet must have shape (N, 2)")
    if weights is None:
        weights = np.ones(len(packets), dtype=np.float64)
    if weights.shape != (len(packets),):
        raise ValueError("weights must have shape (num_layers,)")
    mixed = np.zeros((length, 2), dtype=np.complex128)
    for weight, packet in zip(weights, packets, strict=True):
        mixed += float(weight) * packet
    norms = np.linalg.norm(mixed, axis=1, keepdims=True)
    norms = np.where(norms <= 1e-12, 1.0, norms)
    return mixed / norms


def fit_packet_projection(q_train: np.ndarray, z_train: np.ndarray) -> np.ndarray:
    """Fit a fixed linear map from composed packets into the existing `C^2` chart."""
    return np.linalg.lstsq(q_train, z_train, rcond=None)[0]


def relative_recovery_error(z_true: np.ndarray, z_hat: np.ndarray) -> float:
    denom = np.linalg.norm(z_true)
    if denom <= 1e-12:
        return 0.0
    return float(np.linalg.norm(z_true - z_hat) / denom)


def load_phase_csv(path: Path) -> dict[str, np.ndarray]:
    """Load a scaffold CSV with per-layer angles plus reference `C^2` coordinates."""
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        rows = list(reader)
    if not rows:
        raise ValueError("phase csv is empty")
    phase_keys = sorted(
        (
            key
            for key in rows[0].keys()
            if key and re.fullmatch(r"phi_\d+_angle", key)
        ),
        key=lambda key: int(key.split("_")[1]),
    )
    if not phase_keys:
        phase_keys = sorted(
            (
                key
                for key in rows[0].keys()
                if key and re.fullmatch(r"phi_\d+", key)
            ),
            key=lambda key: int(key.split("_")[1]),
        )
    packets = {
        key: np.asarray(
            [float(row[key]) for row in rows if str(row.get(key, "")).strip() != ""],
            dtype=np.float64,
        )
        for key in phase_keys
    }
    row_count = len(rows)
    for key in phase_keys:
        values = [float(row[key]) for row in rows if str(row.get(key, "")).strip() != ""]
        if len(values) not in (0, row_count):
            raise ValueError(f"phase column {key} contains partial blanks")
        if len(values) == 0:
            packets.pop(key, None)
    z = np.column_stack(
        (
            np.asarray([float(row["z1_real"]) for row in rows], dtype=np.float64)
            + 1j * np.asarray([float(row["z1_imag"]) for row in rows], dtype=np.float64),
            np.asarray([float(row["z2_real"]) for row in rows], dtype=np.float64)
            + 1j * np.asarray([float(row["z2_imag"]) for row in rows], dtype=np.float64),
        )
    )
    packets["z"] = z
    return packets


def run_recovery_demo(path: Path) -> float:
    data = load_phase_csv(path)
    phase_keys = sorted(key for key in data if key.startswith("phi_"))
    packets = [phase_conjugate_packet(data[key]) for key in phase_keys]
    q = compose_packets(packets)
    z = data["z"]
    projection = fit_packet_projection(q, z)
    z_hat = q @ projection
    return relative_recovery_error(z, z_hat)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--phase-csv",
        type=Path,
        default=None,
        help="Optional scaffold dataset with per-layer phases and reference C^2 coordinates.",
    )
    args = parser.parse_args()

    if args.phase_csv is None:
        print("layer_packet_c2_scaffold: define per-layer phases, compose packets, compare to reference C^2.")
        print("expected dataset columns: phi_1_angle, phi_2_angle, ..., z1_real, z1_imag, z2_real, z2_imag")
        return

    error = run_recovery_demo(args.phase_csv)
    print(f"packet_recovery_relative_error,{error:.12f}")


if __name__ == "__main__":
    main()
