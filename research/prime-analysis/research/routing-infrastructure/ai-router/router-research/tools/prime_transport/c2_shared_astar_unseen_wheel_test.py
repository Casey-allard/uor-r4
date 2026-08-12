#!/usr/bin/env python3
from __future__ import annotations

import csv
import math
from dataclasses import dataclass
from pathlib import Path

import numpy as np


TRAIN_WHEELS = (2310, 30030, 510510)
TEST_WHEEL = 9699690
OFFSETS = (0, 2, 6, 8)
MODE_RANGE = range(-3, 4)
CHUNK_SIZE = 200_000
OUT_DIR = Path("/Users/adminamn/AI-Research")


@dataclass
class ScaleModel:
    W: int
    L: int
    fiber_mod: int
    admissible_count: int
    mean: np.ndarray
    right_vectors: np.ndarray
    singular_values: np.ndarray
    mode_labels: list[tuple[int, int]]
    A: np.ndarray
    rel_error: float


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
    admissible_count = 0

    for start, end in chunk_bounds(L):
        j = np.arange(start, end, dtype=np.int64)
        X = build_chunk(j, fiber_mod, modes)
        sums += X.sum(axis=0)
        gram += X.conj().T @ X
        admissible_count += int(admissible_mask(j, W).sum())

    mean = sums / L
    centered_gram = gram - L * np.outer(mean.conj(), mean)
    centered_gram = 0.5 * (centered_gram + centered_gram.conj().T)
    return L, fiber_mod, admissible_count, mean, centered_gram


def canonical_top_components(
    centered_gram: np.ndarray,
    modes: list[tuple[int, int]],
    top_k: int = 2,
) -> tuple[np.ndarray, np.ndarray]:
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


def projected_rows(
    W: int,
    mean: np.ndarray,
    right_vectors: np.ndarray,
    modes: list[tuple[int, int]],
):
    L = W // 6
    fiber_mod = L // 35
    for start, end in chunk_bounds(L):
        j = np.arange(start, end, dtype=np.int64)
        X = build_chunk(j, fiber_mod, modes)
        yield (X - mean) @ right_vectors


def fit_linear_transport(
    W: int,
    mean: np.ndarray,
    right_vectors: np.ndarray,
    modes: list[tuple[int, int]],
) -> tuple[np.ndarray, float]:
    gram = np.zeros((2, 2), dtype=np.complex128)
    cross = np.zeros((2, 2), dtype=np.complex128)
    prev_last = None
    residual_sq = 0.0
    target_sq = 0.0

    cached_chunks = []
    for Z in projected_rows(W, mean, right_vectors, modes):
        cached_chunks.append(Z)
        if prev_last is not None:
            now = prev_last[np.newaxis, :]
            nxt = Z[:1, :]
            gram += now.conj().T @ now
            cross += now.conj().T @ nxt
        if Z.shape[0] >= 2:
            now = Z[:-1, :]
            nxt = Z[1:, :]
            gram += now.conj().T @ now
            cross += now.conj().T @ nxt
        prev_last = Z[-1, :]

    B = np.linalg.solve(gram, cross)

    prev_last = None
    for Z in cached_chunks:
        if prev_last is not None:
            now = prev_last[np.newaxis, :]
            nxt = Z[:1, :]
            pred = now @ B
            residual_sq += float(np.linalg.norm(nxt - pred) ** 2)
            target_sq += float(np.linalg.norm(nxt) ** 2)
        if Z.shape[0] >= 2:
            now = Z[:-1, :]
            nxt = Z[1:, :]
            pred = now @ B
            residual_sq += float(np.linalg.norm(nxt - pred) ** 2)
            target_sq += float(np.linalg.norm(nxt) ** 2)
        prev_last = Z[-1, :]

    rel_error = math.sqrt(residual_sq / target_sq)
    return B.T, rel_error


def accumulate_shared_normals(models: list[ScaleModel], modes: list[tuple[int, int]]) -> np.ndarray:
    gram = np.zeros((2, 2), dtype=np.complex128)
    cross = np.zeros((2, 2), dtype=np.complex128)

    for model in models:
        prev_last = None
        for Z in projected_rows(model.W, model.mean, model.right_vectors, modes):
            if prev_last is not None:
                now = prev_last[np.newaxis, :]
                nxt = Z[:1, :]
                gram += now.conj().T @ now
                cross += now.conj().T @ nxt
            if Z.shape[0] >= 2:
                now = Z[:-1, :]
                nxt = Z[1:, :]
                gram += now.conj().T @ now
                cross += now.conj().T @ nxt
            prev_last = Z[-1, :]

    B = np.linalg.solve(gram, cross)
    return B.T


def transport_error_for_matrix(
    model: ScaleModel,
    A: np.ndarray,
    modes: list[tuple[int, int]],
) -> float:
    residual_sq = 0.0
    target_sq = 0.0
    B = A.T
    prev_last = None

    for Z in projected_rows(model.W, model.mean, model.right_vectors, modes):
        if prev_last is not None:
            now = prev_last[np.newaxis, :]
            nxt = Z[:1, :]
            pred = now @ B
            residual_sq += float(np.linalg.norm(nxt - pred) ** 2)
            target_sq += float(np.linalg.norm(nxt) ** 2)
        if Z.shape[0] >= 2:
            now = Z[:-1, :]
            nxt = Z[1:, :]
            pred = now @ B
            residual_sq += float(np.linalg.norm(nxt - pred) ** 2)
            target_sq += float(np.linalg.norm(nxt) ** 2)
        prev_last = Z[-1, :]

    return math.sqrt(residual_sq / target_sq)


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


def analyze_scale(W: int, modes: list[tuple[int, int]]) -> ScaleModel:
    L, fiber_mod, admissible_count, mean, centered_gram = compute_mean_and_gram(W, modes)
    singular_values, right_vectors = canonical_top_components(centered_gram, modes, top_k=2)
    A, rel_error = fit_linear_transport(W, mean, right_vectors, modes)
    return ScaleModel(
        W=W,
        L=L,
        fiber_mod=fiber_mod,
        admissible_count=admissible_count,
        mean=mean,
        right_vectors=right_vectors,
        singular_values=singular_values,
        mode_labels=modes,
        A=A,
        rel_error=rel_error,
    )


def write_csv(
    train_models: list[ScaleModel],
    test_model: ScaleModel,
    A_star: np.ndarray,
    err_star: float,
    err_ratio: float,
    aligned_distance: float,
):
    summary_path = OUT_DIR / "c2_shared_astar_unseen_9699690_summary.csv"
    eigen_path = OUT_DIR / "c2_shared_astar_unseen_9699690_eigenvalues.csv"

    with summary_path.open("w", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(
            [
                "role",
                "W",
                "L",
                "fiber_mod",
                "admissible_count",
                "scale_specific_rel_error",
                "shared_Astar_rel_error",
                "error_ratio_shared_over_scale_specific",
                "A11_real",
                "A11_imag",
                "A12_real",
                "A12_imag",
                "A21_real",
                "A21_imag",
                "A22_real",
                "A22_imag",
                "conjugation_aligned_distance_to_test",
            ]
        )
        for model in train_models:
            writer.writerow(
                [
                    "train",
                    model.W,
                    model.L,
                    model.fiber_mod,
                    model.admissible_count,
                    model.rel_error,
                    "",
                    "",
                    model.A[0, 0].real,
                    model.A[0, 0].imag,
                    model.A[0, 1].real,
                    model.A[0, 1].imag,
                    model.A[1, 0].real,
                    model.A[1, 0].imag,
                    model.A[1, 1].real,
                    model.A[1, 1].imag,
                    conjugation_aligned_distance(test_model.A, model.A),
                ]
            )
        writer.writerow(
            [
                "test_scale_specific",
                test_model.W,
                test_model.L,
                test_model.fiber_mod,
                test_model.admissible_count,
                test_model.rel_error,
                err_star,
                err_ratio,
                test_model.A[0, 0].real,
                test_model.A[0, 0].imag,
                test_model.A[0, 1].real,
                test_model.A[0, 1].imag,
                test_model.A[1, 0].real,
                test_model.A[1, 0].imag,
                test_model.A[1, 1].real,
                test_model.A[1, 1].imag,
                0.0,
            ]
        )
        writer.writerow(
            [
                "shared_A_star",
                "",
                "",
                "",
                "",
                "",
                err_star,
                err_ratio,
                A_star[0, 0].real,
                A_star[0, 0].imag,
                A_star[0, 1].real,
                A_star[0, 1].imag,
                A_star[1, 0].real,
                A_star[1, 0].imag,
                A_star[1, 1].real,
                A_star[1, 1].imag,
                aligned_distance,
            ]
        )

    with eigen_path.open("w", newline="") as fh:
        writer = csv.writer(fh)
        writer.writerow(["matrix_name", "W", "eig_index", "eig_real", "eig_imag", "eig_abs", "eig_angle"])
        matrices = [
            ("A_star_shared", "", A_star),
            ("A_test_scale_specific", test_model.W, test_model.A),
        ]
        for name, W, matrix in matrices:
            eigvals = np.linalg.eigvals(matrix)
            order = np.argsort(-np.abs(eigvals))
            for idx, eig in enumerate(eigvals[order], start=1):
                writer.writerow(
                    [
                        name,
                        W,
                        idx,
                        eig.real,
                        eig.imag,
                        abs(eig),
                        np.angle(eig),
                    ]
                )


def write_note(
    train_models: list[ScaleModel],
    test_model: ScaleModel,
    A_star: np.ndarray,
    err_star: float,
    err_ratio: float,
    aligned_distance: float,
):
    note_path = OUT_DIR / "c2_shared_astar_unseen_9699690_note.md"
    test_eigs = np.linalg.eigvals(test_model.A)
    star_eigs = np.linalg.eigvals(A_star)
    conclusion = (
        "A_* generalizes acceptably at the unseen scale."
        if err_ratio <= 1.10
        else "A_* does not cleanly generalize to the unseen scale."
    )
    note_path.write_text(
        "\n".join(
            [
                "# Shared C^2 transport law on unseen W=9699690",
                "",
                "Setup:",
                "- train wheels: 2310, 30030, 510510",
                "- test wheel: 9699690",
                "- exact torus modes with kb,kf in [-3,3], excluding (0,0)",
                "- deterministic gauge fixing applied to the top-2 complex SVD basis",
                "",
                "Test comparison:",
                f"- err_test = {test_model.rel_error:.12f}",
                f"- err_star = {err_star:.12f}",
                f"- err_star / err_test = {err_ratio:.12f}",
                "",
                "Eigenvalues:",
                f"- A_test: {test_eigs[0]}, {test_eigs[1]}",
                f"- A_*: {star_eigs[0]}, {star_eigs[1]}",
                "",
                f"- conjugation-aligned distance(A_*, A_test) = {aligned_distance:.12f}",
                "",
                f"Conclusion: {conclusion}",
            ]
        )
        + "\n"
    )


def main():
    modes = mode_list()
    train_models = [analyze_scale(W, modes) for W in TRAIN_WHEELS]
    test_model = analyze_scale(TEST_WHEEL, modes)
    A_star = accumulate_shared_normals(train_models, modes)
    err_star = transport_error_for_matrix(test_model, A_star, modes)
    err_ratio = err_star / test_model.rel_error
    aligned_distance = conjugation_aligned_distance(test_model.A, A_star)

    write_csv(train_models, test_model, A_star, err_star, err_ratio, aligned_distance)
    write_note(train_models, test_model, A_star, err_star, err_ratio, aligned_distance)

    print("Test wheel:", test_model.W)
    print("Scale-specific error:", f"{test_model.rel_error:.12f}")
    print("Shared A_* error:", f"{err_star:.12f}")
    print("Error ratio:", f"{err_ratio:.12f}")
    print("A_test eigenvalues:", np.linalg.eigvals(test_model.A))
    print("A_* eigenvalues:", np.linalg.eigvals(A_star))
    print("Conjugation-aligned distance:", f"{aligned_distance:.12f}")


if __name__ == "__main__":
    main()
