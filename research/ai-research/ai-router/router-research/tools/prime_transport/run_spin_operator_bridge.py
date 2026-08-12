#!/usr/bin/env python3
"""Operator-level bridge test from finite-horizon spin dynamics to shared C^2 transport.

This experiment treats finite-horizon spin as a predictive dynamical state and
asks whether a low-dimensional quotient of the spin transition operator yields a
transport law close to the empirical shared `C^2` backbone `A_*`.
"""

from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path

import numpy as np


ROOT = Path(__file__).resolve().parents[2]
DATA_DIR = ROOT / "results" / "prime_transport_grouped_packets"
ASTAR_SUMMARY = ROOT / "results" / "prime_transport_c2_backbone" / "c2_shared_astar_unseen_9699690_summary.csv"
OUT_DIR = ROOT / "results" / "prime_transport_spin_operator_bridge"
DEFAULT_WHEELS = (30030, 510510)
DEFAULT_HORIZONS = (8, 16, 32, 55, 56)


def load_dataset_columns(path: Path) -> tuple[np.ndarray, np.ndarray]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    if not rows:
        raise ValueError(f"dataset is empty: {path}")
    bits = np.asarray([int(row["admissible_bit"]) for row in rows], dtype=np.int8)
    base = np.asarray([int(row["b_mod_35"]) for row in rows], dtype=np.int64)
    return bits, base


def load_astar(path: Path) -> np.ndarray:
    with path.open("r", encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle))
    for row in rows:
        if row["role"] == "shared_A_star":
            return np.array(
                [
                    [float(row["A11_real"]) + 1j * float(row["A11_imag"]), float(row["A12_real"]) + 1j * float(row["A12_imag"])],
                    [float(row["A21_real"]) + 1j * float(row["A21_imag"]), float(row["A22_real"]) + 1j * float(row["A22_imag"])],
                ],
                dtype=np.complex128,
            )
    raise ValueError(f"shared_A_star row not found: {path}")


def build_spin_words(admissible: np.ndarray, horizon: int) -> list[str]:
    length = admissible.shape[0]
    doubled = np.concatenate((admissible, admissible[: horizon - 1]))
    return [
        "".join("1" if bit else "0" for bit in doubled[idx : idx + horizon])
        for idx in range(length)
    ]


def build_transition(words: list[str]) -> tuple[np.ndarray, np.ndarray, list[str]]:
    states = sorted(set(words))
    index = {state: idx for idx, state in enumerate(states)}
    state_seq = np.asarray([index[word] for word in words], dtype=np.int64)
    counts = np.zeros((len(states), len(states)), dtype=np.float64)
    for pos, src in enumerate(state_seq):
        dst = int(state_seq[(pos + 1) % state_seq.shape[0]])
        counts[src, dst] += 1.0
    row_sums = counts.sum(axis=1, keepdims=True)
    T = np.divide(
        counts.astype(np.complex128),
        row_sums.astype(np.complex128),
        out=np.zeros(counts.shape, dtype=np.complex128),
        where=row_sums > 0,
    )
    return T, state_seq, states


def nondeterministic_state_count(words: list[str]) -> int:
    next_sets: dict[str, set[str]] = {}
    length = len(words)
    for idx, word in enumerate(words):
        nxt = words[(idx + 1) % length]
        next_sets.setdefault(word, set()).add(nxt)
    return sum(1 for targets in next_sets.values() if len(targets) > 1)


def choose_modes(T: np.ndarray, num_modes: int = 2) -> tuple[np.ndarray, np.ndarray]:
    eigvals, eigvecs = np.linalg.eig(T)
    order = np.argsort(-np.abs(eigvals))
    chosen_vals = []
    chosen_vecs = []
    for idx in order:
        eig = eigvals[idx]
        vec = eigvecs[:, idx].astype(np.complex128, copy=True)
        if np.linalg.norm(vec) <= 1e-12:
            continue
        vec /= np.linalg.norm(vec)
        if np.std(vec) <= 1e-9 and abs(eig - 1.0) <= 1e-9:
            continue
        chosen_vals.append(eig)
        chosen_vecs.append(vec)
        if len(chosen_vals) == num_modes:
            break
    if len(chosen_vals) != num_modes:
        raise RuntimeError("could not extract two nontrivial spin modes")
    V = np.column_stack(chosen_vecs)
    q, _ = np.linalg.qr(V)
    for col in range(q.shape[1]):
        dom = int(np.argmax(np.abs(q[:, col])))
        phase = np.angle(q[dom, col])
        q[:, col] *= np.exp(-1j * phase)
        if q[dom, col].real < 0:
            q[:, col] *= -1
    return np.asarray(chosen_vals, dtype=np.complex128), q


def fit_transport(Y: np.ndarray) -> tuple[np.ndarray, float]:
    now = Y[:-1, :]
    nxt = Y[1:, :]
    B, *_ = np.linalg.lstsq(now, nxt, rcond=None)
    pred = now @ B
    denom = np.linalg.norm(nxt)
    err = 0.0 if denom <= 1e-12 else float(np.linalg.norm(nxt - pred) / denom)
    return B.T, err


def relative_matrix_distance(A_ref: np.ndarray, A_other: np.ndarray) -> float:
    return float(np.linalg.norm(A_ref - A_other) / np.linalg.norm(A_ref))


def unitary_from_angles(theta: float, alpha: float, beta: float) -> np.ndarray:
    a = np.exp(1j * alpha) * math.cos(theta)
    b = np.exp(1j * beta) * math.sin(theta)
    return np.array([[a, b], [-np.conj(b), np.conj(a)]], dtype=np.complex128)


def conjugation_aligned_distance(A_ref: np.ndarray, A_other: np.ndarray) -> float:
    best = math.inf
    best_params = (0.0, 0.0, 0.0)
    theta_lo, theta_hi = 0.0, math.pi / 2.0
    alpha_lo, alpha_hi = 0.0, 2.0 * math.pi
    beta_lo, beta_hi = 0.0, 2.0 * math.pi
    for steps in (17, 17, 17):
        theta_grid = np.linspace(theta_lo, theta_hi, steps)
        alpha_grid = np.linspace(alpha_lo, alpha_hi, steps, endpoint=False)
        beta_grid = np.linspace(beta_lo, beta_hi, steps, endpoint=False)
        for theta in theta_grid:
            for alpha in alpha_grid:
                for beta in beta_grid:
                    U = unitary_from_angles(theta, alpha, beta)
                    aligned = U @ A_other @ U.conj().T
                    dist = np.linalg.norm(A_ref - aligned) / np.linalg.norm(A_ref)
                    if dist < best:
                        best = float(dist)
                        best_params = (theta, alpha, beta)
        theta0, alpha0, beta0 = best_params
        theta_span = (theta_hi - theta_lo) / max(steps - 1, 1)
        alpha_span = (alpha_hi - alpha_lo) / steps
        beta_span = (beta_hi - beta_lo) / steps
        theta_lo, theta_hi = max(0.0, theta0 - theta_span), min(math.pi / 2.0, theta0 + theta_span)
        alpha_lo, alpha_hi = alpha0 - alpha_span, alpha0 + alpha_span
        beta_lo, beta_hi = beta0 - beta_span, beta0 + beta_span
    return best


def eigenvalue_distance(A_ref: np.ndarray, A_other: np.ndarray) -> float:
    ref = np.linalg.eigvals(A_ref)
    other = np.linalg.eigvals(A_other)
    ref = ref[np.argsort(-np.abs(ref))]
    other = other[np.argsort(-np.abs(other))]
    return float(np.linalg.norm(ref - other) / max(np.linalg.norm(ref), 1e-12))


def evaluate_operator_bridge(W: int, horizon: int, A_star: np.ndarray) -> dict[str, object]:
    bits, base = load_dataset_columns(DATA_DIR / f"prime_transport_layer_packet_dataset_W{W}.csv")
    words = build_spin_words(bits, horizon)
    predictive_labels = [f"{int(b)}|{word}" for b, word in zip(base, words, strict=True)]
    T, state_seq, states = build_transition(predictive_labels)
    chosen_vals, state_modes = choose_modes(T, num_modes=2)
    Y = state_modes[state_seq, :]
    A_spin, fit_error = fit_transport(Y)
    return {
        "W": W,
        "horizon": horizon,
        "raw_spin_nondeterministic_state_count": nondeterministic_state_count(words),
        "num_states": len(states),
        "mode1_real": float(chosen_vals[0].real),
        "mode1_imag": float(chosen_vals[0].imag),
        "mode1_abs": float(abs(chosen_vals[0])),
        "mode2_real": float(chosen_vals[1].real),
        "mode2_imag": float(chosen_vals[1].imag),
        "mode2_abs": float(abs(chosen_vals[1])),
        "spin_embedding_fit_error": fit_error,
        "A11_real": float(A_spin[0, 0].real),
        "A11_imag": float(A_spin[0, 0].imag),
        "A12_real": float(A_spin[0, 1].real),
        "A12_imag": float(A_spin[0, 1].imag),
        "A21_real": float(A_spin[1, 0].real),
        "A21_imag": float(A_spin[1, 0].imag),
        "A22_real": float(A_spin[1, 1].real),
        "A22_imag": float(A_spin[1, 1].imag),
        "relative_matrix_distance_to_Astar": relative_matrix_distance(A_star, A_spin),
        "conjugation_aligned_distance_to_Astar": conjugation_aligned_distance(A_star, A_spin),
        "eigenvalue_distance_to_Astar": eigenvalue_distance(A_star, A_spin),
    }


def write_csv(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def summarize(rows: list[dict[str, object]]) -> list[dict[str, object]]:
    summary = []
    for W in sorted({int(row["W"]) for row in rows}):
        items = [row for row in rows if int(row["W"]) == W]
        best = min(items, key=lambda row: float(row["conjugation_aligned_distance_to_Astar"]))
        summary.append(
            {
                "W": W,
                "best_horizon_by_conjugation_distance": int(best["horizon"]),
                "best_num_states": int(best["num_states"]),
                "best_spin_embedding_fit_error": float(best["spin_embedding_fit_error"]),
                "best_relative_matrix_distance_to_Astar": float(best["relative_matrix_distance_to_Astar"]),
                "best_conjugation_aligned_distance_to_Astar": float(best["conjugation_aligned_distance_to_Astar"]),
                "best_eigenvalue_distance_to_Astar": float(best["eigenvalue_distance_to_Astar"]),
            }
        )
    overall = min(rows, key=lambda row: float(row["conjugation_aligned_distance_to_Astar"]))
    summary.append(
        {
            "W": "overall_best",
            "best_horizon_by_conjugation_distance": int(overall["horizon"]),
            "best_num_states": int(overall["num_states"]),
            "best_spin_embedding_fit_error": float(overall["spin_embedding_fit_error"]),
            "best_relative_matrix_distance_to_Astar": float(overall["relative_matrix_distance_to_Astar"]),
            "best_conjugation_aligned_distance_to_Astar": float(overall["conjugation_aligned_distance_to_Astar"]),
            "best_eigenvalue_distance_to_Astar": float(overall["eigenvalue_distance_to_Astar"]),
        }
    )
    return summary


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--wheel", dest="wheels", type=int, action="append", default=None)
    parser.add_argument("--horizon", dest="horizons", type=int, action="append", default=None)
    args = parser.parse_args()

    wheels = tuple(args.wheels or DEFAULT_WHEELS)
    horizons = tuple(args.horizons or DEFAULT_HORIZONS)
    A_star = load_astar(ASTAR_SUMMARY)
    detail_rows = [evaluate_operator_bridge(W, H, A_star) for W in wheels for H in horizons]
    summary_rows = summarize(detail_rows)
    detail_path = OUT_DIR / "spin_operator_bridge_detail.csv"
    summary_path = OUT_DIR / "spin_operator_bridge_summary.csv"
    write_csv(detail_path, detail_rows)
    write_csv(summary_path, summary_rows)
    best = min(detail_rows, key=lambda row: float(row["conjugation_aligned_distance_to_Astar"]))
    print(f"best_W,{best['W']}")
    print(f"best_horizon,{best['horizon']}")
    print(f"best_conjugation_distance,{float(best['conjugation_aligned_distance_to_Astar']):.12f}")
    print(f"best_relative_matrix_distance,{float(best['relative_matrix_distance_to_Astar']):.12f}")
    print(f"best_eigenvalue_distance,{float(best['eigenvalue_distance_to_Astar']):.12f}")
    print(f"summary_csv,{summary_path}")


if __name__ == "__main__":
    main()
