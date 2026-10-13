#!/usr/bin/env python3
"""Spectral-Event Correlation Probe — Stage 6 bridge experiment.

Measures the correlation between spectral smoothness (Stage 5) and
event-gate quiescence (Stage 6) at the per-sample level.

Mathematical object: H^4 Poincaré graph Laplacian eigenvector basis.
Hypothesis: spectrally smooth regions (low Dirichlet energy on the
local error surface) correspond to regions where the event gate is
quiescent (low error → gate inactive), linking Stage 5 structure
to Stage 6 sparse-event behaviour.

Outputs per-route:
  - Pearson & Spearman correlation between per-sample local spectral
    roughness and event-gate activation value
  - Comparison of correlation strength across graph modes
    (poincare_4d vs ambient_euclidean)
"""
import argparse
import json
import os
import sys
from typing import Any, Dict, Iterable, List

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from tasks import router_proxy_eval as task
from tools import spectral_route_audit as audit


def per_sample_local_roughness(
    laplacian: np.ndarray,
    signal: np.ndarray,
    neighbourhood_k: int = 12,
) -> np.ndarray:
    """Compute per-sample local Dirichlet roughness.

    For each sample i, roughness_i = (L @ signal)[i] * signal[i],
    which is the per-sample contribution to the quadratic form f^T L f.
    This measures how much sample i disagrees with its graph neighbours.
    """
    arr = np.asarray(signal, dtype=np.float64)
    if arr.ndim == 1:
        arr = arr[:, None]
    # Per-sample Dirichlet contribution: diag of (signal^T @ L @ signal)
    # For column vector: roughness_i = signal_i * (L @ signal)_i
    Lf = laplacian @ arr  # (n, cols)
    # Sum over columns to get scalar roughness per sample
    roughness = np.sum(arr * Lf, axis=1)
    return roughness.astype(np.float64)


def per_sample_lowfreq_residual(
    evecs: np.ndarray,
    signal: np.ndarray,
    modes: int = 8,
) -> np.ndarray:
    """Per-sample high-frequency residual energy.

    Project signal onto low-frequency modes, compute per-sample residual
    magnitude. High residual = locally rough = bad candidate for quiescence.
    """
    arr = np.asarray(signal, dtype=np.float64)
    if arr.ndim == 1:
        arr = arr[:, None]
    centered = arr - np.mean(arr, axis=0, keepdims=True)
    use = min(max(int(modes), 0), max(evecs.shape[1] - 1, 0))
    if use == 0:
        return np.sqrt(np.sum(centered ** 2, axis=1))
    basis = evecs[:, 1:1 + use]  # skip DC mode (index 0)
    proj = basis @ (basis.T @ centered)  # low-frequency projection
    residual = centered - proj
    return np.sqrt(np.sum(residual ** 2, axis=1)).astype(np.float64)


def per_sample_event_gate(
    yhat: np.ndarray,
    y: np.ndarray,
    mode: str = "soft_error",
    threshold: float = 0.0,
    tau: float = 0.02,
) -> np.ndarray:
    """Compute per-sample event gate value.

    Returns array of gate values in [0, 1] for each eval sample.
    Higher = more active (should update), lower = quiescent.
    """
    n = yhat.shape[0]
    gates = np.zeros(n, dtype=np.float64)
    for i in range(n):
        err = yhat[i] - y[i]
        error_mag = float(np.sqrt(np.mean(err * err)))
        gates[i] = task.event_gate_value_from_error(
            error_mag=error_mag,
            mode=mode,
            threshold=threshold,
            tau=tau,
        )
    return gates


def _spearman_r(x: np.ndarray, y: np.ndarray) -> float:
    """Spearman rank correlation without scipy dependency."""
    n = len(x)
    if n < 3:
        return 0.0
    rx = np.empty(n, dtype=np.float64)
    ry = np.empty(n, dtype=np.float64)
    rx[np.argsort(x)] = np.arange(n, dtype=np.float64)
    ry[np.argsort(y)] = np.arange(n, dtype=np.float64)
    rx -= np.mean(rx)
    ry -= np.mean(ry)
    denom = np.sqrt(np.dot(rx, rx) * np.dot(ry, ry))
    if denom < 1e-12:
        return 0.0
    return float(np.dot(rx, ry) / denom)


def _pearson_r(x: np.ndarray, y: np.ndarray) -> float:
    n = len(x)
    if n < 3:
        return 0.0
    xc = x - np.mean(x)
    yc = y - np.mean(y)
    denom = np.sqrt(np.dot(xc, xc) * np.dot(yc, yc))
    if denom < 1e-12:
        return 0.0
    return float(np.dot(xc, yc) / denom)


def correlation_metrics(
    roughness: np.ndarray,
    highfreq_residual: np.ndarray,
    gate_values: np.ndarray,
    error_magnitudes: np.ndarray,
) -> Dict[str, float]:
    """Compute correlation metrics between spectral measures and event gate."""
    return {
        # Roughness vs gate: positive = rough regions have higher gate (more active)
        "roughness_vs_gate_pearson": _pearson_r(roughness, gate_values),
        "roughness_vs_gate_spearman": _spearman_r(roughness, gate_values),
        # High-freq residual vs gate: positive = spectrally rough regions more active
        "highfreq_vs_gate_pearson": _pearson_r(highfreq_residual, gate_values),
        "highfreq_vs_gate_spearman": _spearman_r(highfreq_residual, gate_values),
        # Roughness vs error: positive = rough regions have higher error (validates link)
        "roughness_vs_error_pearson": _pearson_r(roughness, error_magnitudes),
        "roughness_vs_error_spearman": _spearman_r(roughness, error_magnitudes),
        # Gate statistics
        "gate_mean": float(np.mean(gate_values)),
        "gate_active_frac": float(np.mean(gate_values >= 0.5)),
        "gate_std": float(np.std(gate_values)),
        # Roughness statistics
        "roughness_mean": float(np.mean(roughness)),
        "roughness_std": float(np.std(roughness)),
        # High-freq residual statistics
        "highfreq_residual_mean": float(np.mean(highfreq_residual)),
        "highfreq_residual_std": float(np.std(highfreq_residual)),
        # Error statistics
        "error_mean": float(np.mean(error_magnitudes)),
        "error_std": float(np.std(error_magnitudes)),
    }


def parse_route_ids(raw: str, available: Iterable[str]) -> List[str]:
    return audit.parse_route_ids(raw, available)


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Spectral-event correlation probe (Stage 6 bridge)"
    )
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--max-points", type=int, default=384)
    ap.add_argument("--knn-k", type=int, default=12)
    ap.add_argument("--lowfreq-modes", type=int, default=8)
    ap.add_argument("--graph-mode", type=str, default="poincare_4d",
                     choices=["ambient_euclidean", "hopf_coords", "poincare_4d"])
    # Event gate parameters
    ap.add_argument("--event-gate-mode", type=str, default="soft_error",
                     choices=["soft_error", "hard_error"])
    ap.add_argument("--event-gate-threshold", type=float, default=0.0)
    ap.add_argument("--event-gate-tau", type=float, default=0.02)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    cfg = audit.load_config(args.config)
    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {route["route_id"]: route.get("args", {}) for route in routes}
    route_ids = parse_route_ids(args.routes, route_map.keys())

    results = []
    for route_id in route_ids:
        if route_id not in route_map:
            raise SystemExit(f"route_id {route_id!r} not found in config")
        route_args = audit.build_args(common_args, route_map[route_id], args.seed)

        # 1. Get task eval snapshot (trained model + per-sample predictions)
        snap = audit.task_eval_snapshot(route_args, max_points=args.max_points)

        # 2. Spectral decomposition on the routing graph
        decomp = audit.spectral_decomposition(
            snap["route_z"], knn_k=args.knn_k,
            graph_mode=args.graph_mode, v_ev=snap["v_ev"], dims=snap["dims"],
        )

        # 3. Per-sample error magnitudes
        residual = snap["yhat_eval"] - snap["y_eval"]
        error_magnitudes = np.sqrt(np.mean(residual ** 2, axis=1))

        # 4. Per-sample event gate values
        gate_values = per_sample_event_gate(
            yhat=snap["yhat_eval"],
            y=snap["y_eval"],
            mode=args.event_gate_mode,
            threshold=args.event_gate_threshold,
            tau=args.event_gate_tau,
        )

        # 5. Per-sample spectral roughness (local Dirichlet contribution)
        # Use error_indicator as the signal — this is exactly what
        # Stage 5 proved is spectrally smooth on poincare_4d
        pred_idx = np.argmax(snap["yhat_eval"], axis=1)
        true_idx = np.argmax(snap["y_eval"], axis=1)
        error_indicator = (pred_idx != true_idx).astype(np.float64)

        true_score = snap["yhat_eval"][np.arange(snap["yhat_eval"].shape[0]), true_idx]
        other_scores = snap["yhat_eval"].copy()
        other_scores[np.arange(snap["yhat_eval"].shape[0]), true_idx] = -np.inf
        best_other = np.max(other_scores, axis=1)
        true_margin = true_score - best_other

        roughness_error = per_sample_local_roughness(
            decomp["laplacian"], error_indicator
        )
        roughness_margin = per_sample_local_roughness(
            decomp["laplacian"], true_margin
        )

        # 6. Per-sample high-frequency residual
        highfreq_error = per_sample_lowfreq_residual(
            decomp["evecs"], error_indicator, modes=args.lowfreq_modes
        )
        highfreq_margin = per_sample_lowfreq_residual(
            decomp["evecs"], true_margin, modes=args.lowfreq_modes
        )

        # 7. Aggregate spectral metrics (for comparison with Stage 5)
        spectral_metrics = audit.spectral_metrics_from_decomposition(
            decomp,
            route_z=snap["route_z"],
            shell=snap["shell"],
            sector=snap["sector"],
            lowfreq_modes=args.lowfreq_modes,
        )

        # 8. Correlation analysis
        corr_error = correlation_metrics(
            roughness=roughness_error,
            highfreq_residual=highfreq_error,
            gate_values=gate_values,
            error_magnitudes=error_magnitudes,
        )
        corr_margin = correlation_metrics(
            roughness=roughness_margin,
            highfreq_residual=highfreq_margin,
            gate_values=gate_values,
            error_magnitudes=error_magnitudes,
        )

        results.append({
            "route_id": route_id,
            "args": {
                "sector_mode": route_args.sector_mode,
                "phase4_dims": route_args.phase4_dims,
                "field4_dims": route_args.field4_dims,
                "phase_field_lambda": float(route_args.phase_field_lambda),
                "phase_transport_lambda": float(route_args.phase_transport_lambda),
            },
            "correlation_error_signal": corr_error,
            "correlation_margin_signal": corr_margin,
            "spectral_metrics": spectral_metrics,
        })

    payload = {
        "config": args.config,
        "seed": int(args.seed),
        "max_points": int(args.max_points),
        "knn_k": int(args.knn_k),
        "lowfreq_modes": int(args.lowfreq_modes),
        "graph_mode": args.graph_mode,
        "event_gate_mode": args.event_gate_mode,
        "event_gate_threshold": args.event_gate_threshold,
        "event_gate_tau": args.event_gate_tau,
        "results": results,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, sort_keys=True)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
