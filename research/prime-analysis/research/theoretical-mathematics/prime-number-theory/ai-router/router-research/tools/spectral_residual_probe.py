#!/usr/bin/env python3
import argparse
import json
import os
import sys
from typing import Any, Dict, Iterable, List

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from tools import spectral_route_audit as audit


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def signal_metrics(prefix: str, laplacian: np.ndarray, evecs: np.ndarray, signal: np.ndarray, lowfreq_modes: int) -> Dict[str, Any]:
    arr = np.asarray(signal, dtype=np.float64)
    if arr.ndim == 1:
        arr = arr[:, None]
    return {
        f"{prefix}_lowfreq_energy": audit.low_frequency_matrix_energy(evecs, arr, modes=lowfreq_modes),
        f"{prefix}_dirichlet_energy": audit.normalized_dirichlet_energy(laplacian, arr),
        f"{prefix}_mean": float(np.mean(arr)),
        f"{prefix}_std": float(np.std(arr)),
    }


def task_residual_metrics(laplacian: np.ndarray, evecs: np.ndarray, y_eval: np.ndarray, yhat_eval: np.ndarray, lowfreq_modes: int) -> Dict[str, Any]:
    residual = yhat_eval - y_eval
    residual_l2 = np.sqrt(np.sum(residual * residual, axis=1))
    true_idx = np.argmax(y_eval, axis=1)
    pred_idx = np.argmax(yhat_eval, axis=1)
    error_indicator = (pred_idx != true_idx).astype(np.float64)

    true_score = yhat_eval[np.arange(yhat_eval.shape[0]), true_idx]
    other_scores = yhat_eval.copy()
    other_scores[np.arange(yhat_eval.shape[0]), true_idx] = -np.inf
    best_other = np.max(other_scores, axis=1)
    true_margin = true_score - best_other

    metrics: Dict[str, Any] = {
        "task_error_rate": float(np.mean(error_indicator)),
        "task_residual_l2_mean": float(np.mean(residual_l2)),
        "task_true_score_mean": float(np.mean(true_score)),
        "task_true_margin_mean": float(np.mean(true_margin)),
    }
    metrics.update(signal_metrics("residual_vector", laplacian, evecs, residual, lowfreq_modes))
    metrics.update(signal_metrics("residual_l2", laplacian, evecs, residual_l2, lowfreq_modes))
    metrics.update(signal_metrics("error_indicator", laplacian, evecs, error_indicator, lowfreq_modes))
    metrics.update(signal_metrics("true_score", laplacian, evecs, true_score, lowfreq_modes))
    metrics.update(signal_metrics("true_margin", laplacian, evecs, true_margin, lowfreq_modes))
    return metrics


def parse_route_ids(raw: str, available: Iterable[str]) -> List[str]:
    return audit.parse_route_ids(raw, available)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--max-points", type=int, default=384)
    ap.add_argument("--knn-k", type=int, default=12)
    ap.add_argument("--lowfreq-modes", type=int, default=8)
    ap.add_argument("--graph-mode", type=str, default="ambient_euclidean",
                     choices=["ambient_euclidean", "hopf_coords", "poincare_4d"])
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    cfg = load_config(args.config)
    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {route["route_id"]: route.get("args", {}) for route in routes}
    route_ids = parse_route_ids(args.routes, route_map.keys())

    results = []
    for route_id in route_ids:
        if route_id not in route_map:
            raise SystemExit(f"route_id {route_id!r} not found in config")
        route_args = audit.build_args(common_args, route_map[route_id], args.seed)
        snap = audit.task_eval_snapshot(route_args, max_points=args.max_points)
        decomp = audit.spectral_decomposition(
            snap["route_z"], knn_k=args.knn_k,
            graph_mode=args.graph_mode, v_ev=snap["v_ev"], dims=snap["dims"],
        )
        metrics = audit.spectral_metrics_from_decomposition(
            decomp,
            route_z=snap["route_z"],
            shell=snap["shell"],
            sector=snap["sector"],
            lowfreq_modes=args.lowfreq_modes,
        )
        metrics.update(
            task_residual_metrics(
                decomp["laplacian"],
                decomp["evecs"],
                snap["y_eval"],
                snap["yhat_eval"],
                lowfreq_modes=args.lowfreq_modes,
            )
        )
        results.append(
            {
                "route_id": route_id,
                "args": {
                    "sector_mode": route_args.sector_mode,
                    "phase4_dims": route_args.phase4_dims,
                    "field4_dims": route_args.field4_dims,
                    "phase_field_lambda": float(route_args.phase_field_lambda),
                    "phase_transport_lambda": float(route_args.phase_transport_lambda),
                },
                "metrics": metrics,
            }
        )

    payload = {
        "config": args.config,
        "seed": int(args.seed),
        "max_points": int(args.max_points),
        "knn_k": int(args.knn_k),
        "lowfreq_modes": int(args.lowfreq_modes),
        "graph_mode": args.graph_mode,
        "results": results,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, sort_keys=True)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
