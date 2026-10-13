#!/usr/bin/env python3
"""Build aligned prime-transport datasets for grouped-packet recovery tests.

Source-of-truth choices:
- quotient coordinates `z(j)` are rebuilt from the current shared-`C^2`
  backbone pipeline used for the unseen-wheel transport experiment
- per-layer phases are reconstructed from the finite-depth transport fiber
  index `t = floor(j / 35)` by factoring `fiber_mod = (W / 6) / 35`
"""

from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path

import numpy as np


MODE_RANGE = range(-3, 4)
OFFSETS = (0, 2, 6, 8)
CHUNK_SIZE = 200_000
OUT_DIR = Path(__file__).resolve().parents[2] / "results" / "prime_transport_grouped_packets"
SUPPORTED_WHEELS = (30030, 510510, 9699690)
MAX_LAYER_COLUMNS = 4


def mode_list() -> list[tuple[int, int]]:
    return [
        (kb, kf)
        for kb in MODE_RANGE
        for kf in MODE_RANGE
        if (kb, kf) != (0, 0)
    ]


def chunk_bounds(length: int, chunk_size: int = CHUNK_SIZE):
    for start in range(0, length, chunk_size):
        yield start, min(length, start + chunk_size)


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


def build_chunk(j: np.ndarray, fiber_mod: int, modes: list[tuple[int, int]]) -> np.ndarray:
    b = j % 35
    f = (j // 35) % fiber_mod
    phase = np.empty((j.size, len(modes)), dtype=np.float64)
    for idx, (kb, kf) in enumerate(modes):
        phase[:, idx] = (kb * b / 35.0) + (kf * f / fiber_mod)
    return np.exp(2j * np.pi * phase).astype(np.complex128, copy=False)


def admissible_mask(j: np.ndarray, W: int) -> np.ndarray:
    n = 5 + 6 * j
    mask = np.ones(j.size, dtype=bool)
    for offset in OFFSETS:
        mask &= np.gcd(n + offset, W) == 1
    return mask


def compute_mean_and_gram(W: int, modes: list[tuple[int, int]]):
    L = W // 6
    fiber_mod = L // 35
    sums = np.zeros(len(modes), dtype=np.complex128)
    gram = np.zeros((len(modes), len(modes)), dtype=np.complex128)

    for start, end in chunk_bounds(L):
        j = np.arange(start, end, dtype=np.int64)
        X = build_chunk(j, fiber_mod, modes)
        sums += X.sum(axis=0)
        gram += X.conj().T @ X

    mean = sums / L
    centered_gram = gram - L * np.outer(mean.conj(), mean)
    centered_gram = 0.5 * (centered_gram + centered_gram.conj().T)
    return L, fiber_mod, mean, centered_gram


def canonical_top_components(centered_gram: np.ndarray, top_k: int = 2) -> tuple[np.ndarray, np.ndarray]:
    evals, evecs = np.linalg.eigh(centered_gram)
    singular_values = np.sqrt(np.clip(evals, 0.0, None))
    dominant_mode = np.argmax(np.abs(evecs), axis=0)
    sort_keys = np.lexsort((dominant_mode, -evals))
    sort_idx = sort_keys[::-1]
    chosen = []
    chosen_s = []
    used = set()
    tol = 1e-10

    for idx in sort_idx:
        vec = evecs[:, idx].copy()
        for existing in chosen:
            vec -= existing * np.vdot(existing, vec)
        norm = np.linalg.norm(vec)
        if norm <= tol:
            continue
        vec /= norm
        dom = int(np.argmax(np.abs(vec)))
        if dom in used:
            continue
        phase = np.angle(vec[dom])
        vec *= np.exp(-1j * phase)
        if vec[dom].real < 0:
            vec *= -1
        chosen.append(vec)
        chosen_s.append(singular_values[idx])
        used.add(dom)
        if len(chosen) == top_k:
            break

    if len(chosen) != top_k:
        raise RuntimeError("Could not extract a deterministic top-2 basis.")

    V = np.column_stack(chosen)
    q, _ = np.linalg.qr(V)
    for col in range(q.shape[1]):
        dom = int(np.argmax(np.abs(q[:, col])))
        phase = np.angle(q[dom, col])
        q[:, col] *= np.exp(-1j * phase)
        if q[dom, col].real < 0:
            q[:, col] *= -1
    return np.array(chosen_s, dtype=np.float64), q


def dataset_fieldnames() -> list[str]:
    names = [
        "W",
        "j",
        "n",
        "layer_depth",
        "b_mod_35",
        "b_angle",
        "t_index",
        "fiber_mod",
        "fiber_index",
        "fiber_angle",
        "admissible_bit",
    ]
    for idx in range(1, MAX_LAYER_COLUMNS + 1):
        names.extend(
            (
                f"phi_{idx}_prime",
                f"phi_{idx}_index",
                f"phi_{idx}_angle",
            )
        )
    names.extend(
        (
            "z1_real",
            "z1_imag",
            "z2_real",
            "z2_imag",
            "z_radius",
        )
    )
    return names


def build_row_dict(
    *,
    W: int,
    j_value: int,
    fiber_mod: int,
    layer_primes: list[int],
    z1: complex,
    z2: complex,
    admissible_bit: int,
) -> dict[str, object]:
    t_index = j_value // 35
    fiber_index = t_index % fiber_mod
    row: dict[str, object] = {
        "W": W,
        "j": j_value,
        "n": 5 + 6 * j_value,
        "layer_depth": len(layer_primes),
        "b_mod_35": j_value % 35,
        "b_angle": (2.0 * math.pi * (j_value % 35)) / 35.0,
        "t_index": t_index,
        "fiber_mod": fiber_mod,
        "fiber_index": fiber_index,
        "fiber_angle": (2.0 * math.pi * fiber_index) / fiber_mod,
        "admissible_bit": admissible_bit,
    }
    for idx in range(MAX_LAYER_COLUMNS):
        if idx < len(layer_primes):
            prime = int(layer_primes[idx])
            phase_index = t_index % prime
            row[f"phi_{idx + 1}_prime"] = prime
            row[f"phi_{idx + 1}_index"] = phase_index
            row[f"phi_{idx + 1}_angle"] = (2.0 * math.pi * phase_index) / prime
        else:
            row[f"phi_{idx + 1}_prime"] = ""
            row[f"phi_{idx + 1}_index"] = ""
            row[f"phi_{idx + 1}_angle"] = ""
    row["z1_real"] = float(np.real(z1))
    row["z1_imag"] = float(np.imag(z1))
    row["z2_real"] = float(np.real(z2))
    row["z2_imag"] = float(np.imag(z2))
    row["z_radius"] = float(math.sqrt((abs(z1) ** 2) + (abs(z2) ** 2)))
    return row


def build_dataset_for_wheel(W: int) -> Path:
    modes = mode_list()
    L, fiber_mod, mean, centered_gram = compute_mean_and_gram(W, modes)
    _, right_vectors = canonical_top_components(centered_gram)
    layer_primes = factor_prime_layers(fiber_mod)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    out_path = OUT_DIR / f"prime_transport_layer_packet_dataset_W{W}.csv"
    with out_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=dataset_fieldnames())
        writer.writeheader()

        for start, end in chunk_bounds(L):
            j = np.arange(start, end, dtype=np.int64)
            X = build_chunk(j, fiber_mod, modes)
            Z = (X - mean) @ right_vectors
            admissible = admissible_mask(j, W)
            for local_idx, j_value in enumerate(j.tolist()):
                z1 = complex(Z[local_idx, 0])
                z2 = complex(Z[local_idx, 1])
                row = build_row_dict(
                    W=W,
                    j_value=j_value,
                    fiber_mod=fiber_mod,
                    layer_primes=layer_primes,
                    z1=z1,
                    z2=z2,
                    admissible_bit=int(admissible[local_idx]),
                )
                writer.writerow(row)
    return out_path


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--wheel",
        dest="wheels",
        type=int,
        action="append",
        help="Wheel scale to export. Repeat for multiple scales.",
    )
    args = parser.parse_args()

    wheels = args.wheels or [30030]
    for wheel in wheels:
        if wheel not in SUPPORTED_WHEELS:
            raise ValueError(f"unsupported wheel: {wheel}")
        out_path = build_dataset_for_wheel(wheel)
        print(f"wrote_dataset,{wheel},{out_path}")


if __name__ == "__main__":
    main()
