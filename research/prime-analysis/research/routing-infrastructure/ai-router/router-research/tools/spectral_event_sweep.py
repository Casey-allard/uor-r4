#!/usr/bin/env python3
"""Multi-seed sweep orchestrator for spectral-event correlation probe.

Runs spectral_event_correlation_probe.py across seeds specified in config,
aggregates correlation metrics.
"""
import argparse
import json
import os
import subprocess
import sys
from typing import Any, Dict, List

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--python_bin", type=str, default=sys.executable)
    args = ap.parse_args()

    cfg = load_config(args.config)
    seeds = cfg.get("seeds", [0])
    graph_mode = cfg.get("graph_mode", "poincare_4d")
    experiment_id = cfg.get("experiment_id", "spectral_event_sweep")
    event_gate_mode = cfg.get("event_gate_mode", "soft_error")
    event_gate_threshold = cfg.get("event_gate_threshold", 0.0)
    event_gate_tau = cfg.get("event_gate_tau", 0.02)
    max_points = cfg.get("max_points", 384)
    knn_k = cfg.get("knn_k", 12)
    lowfreq_modes = cfg.get("lowfreq_modes", 8)

    probe_script = os.path.join(ROOT, "tools", "spectral_event_correlation_probe.py")
    output_dir = os.path.join(ROOT, "results", "analysis")
    os.makedirs(output_dir, exist_ok=True)

    per_seed_results: List[Dict[str, Any]] = []
    for seed in seeds:
        out_path = os.path.join(output_dir, f"{experiment_id}_seed{seed}.json")
        cmd = [
            args.python_bin, probe_script,
            "--config", args.config,
            "--seed", str(seed),
            "--max-points", str(max_points),
            "--knn-k", str(knn_k),
            "--lowfreq-modes", str(lowfreq_modes),
            "--graph-mode", graph_mode,
            "--event-gate-mode", event_gate_mode,
            "--event-gate-threshold", str(event_gate_threshold),
            "--event-gate-tau", str(event_gate_tau),
            "--output", out_path,
        ]
        print(f"[seed {seed}] running: {' '.join(cmd)}")
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=ROOT)
        if result.returncode != 0:
            print(f"[seed {seed}] FAILED: {result.stderr}")
            sys.exit(1)

        with open(out_path, "r") as f:
            seed_data = json.load(f)
        per_seed_results.append(seed_data)
        print(f"[seed {seed}] done")

    # Aggregate: compute mean/std of correlation metrics across seeds
    aggregate = _aggregate_seeds(per_seed_results)
    aggregate["experiment_id"] = experiment_id
    aggregate["seeds"] = seeds
    aggregate["graph_mode"] = graph_mode
    aggregate["event_gate_mode"] = event_gate_mode
    aggregate["event_gate_threshold"] = event_gate_threshold
    aggregate["event_gate_tau"] = event_gate_tau

    agg_path = os.path.join(output_dir, f"{experiment_id}.json")
    with open(agg_path, "w") as f:
        json.dump(aggregate, f, indent=2, sort_keys=True)
    print(f"\nAggregate written to {agg_path}")
    print(json.dumps(aggregate, indent=2, sort_keys=True))


def _aggregate_seeds(per_seed: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Aggregate correlation metrics across seeds."""
    import numpy as np

    if not per_seed:
        return {}

    # Collect per-route, per-seed correlation values
    route_ids = [r["route_id"] for r in per_seed[0].get("results", [])]
    aggregate_routes = []

    for ri, route_id in enumerate(route_ids):
        error_corrs: Dict[str, List[float]] = {}
        margin_corrs: Dict[str, List[float]] = {}

        for seed_data in per_seed:
            results = seed_data.get("results", [])
            if ri >= len(results):
                continue
            route_result = results[ri]
            for key, val in route_result.get("correlation_error_signal", {}).items():
                error_corrs.setdefault(key, []).append(float(val))
            for key, val in route_result.get("correlation_margin_signal", {}).items():
                margin_corrs.setdefault(key, []).append(float(val))

        error_agg = {
            k: {"mean": float(np.mean(v)), "std": float(np.std(v)), "values": v}
            for k, v in error_corrs.items()
        }
        margin_agg = {
            k: {"mean": float(np.mean(v)), "std": float(np.std(v)), "values": v}
            for k, v in margin_corrs.items()
        }
        aggregate_routes.append({
            "route_id": route_id,
            "correlation_error_signal": error_agg,
            "correlation_margin_signal": margin_agg,
        })

    return {"routes": aggregate_routes}


if __name__ == "__main__":
    main()
