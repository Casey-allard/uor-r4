#!/usr/bin/env python3
"""INC-0160: Training routing cost probe.

Measures routed computation cost during the EMA training loop:
how many unique memory regions (bucket keys) does training access,
and how concentrated is that access?

This is a COST measurement, not a quality measurement.
MSE is NOT used (INC-0155: MSE is routing-agnostic).

Metrics:
- unique_buckets: distinct bucket keys accessed during training
- effective_bucket_count: exp(H(p)) = perplexity of bucket visit distribution
- training_gini: Gini coefficient of per-bucket visit counts
- per_bucket_density: N / unique_buckets
- top_half_concentration: fraction of samples in top-50% most visited buckets
- cumulative unique bucket curves (sampled at intervals)
- per-window Gini curves
"""
import argparse
import json
import os
import sys
from collections import Counter

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr
from tools import spectral_route_audit as audit


def gini_coefficient(counts):
    """Gini coefficient of an array of counts."""
    arr = np.sort(np.asarray(counts, dtype=np.float64))
    n = len(arr)
    if n == 0 or arr.sum() == 0:
        return 0.0
    index = np.arange(1, n + 1)
    return float((2 * np.sum(index * arr) - (n + 1) * np.sum(arr))
                  / (n * np.sum(arr)))


def effective_count(counts):
    """Perplexity: exp(H(p)) where p = normalized distribution."""
    arr = np.asarray(counts, dtype=np.float64)
    total = arr.sum()
    if total == 0:
        return 0.0
    p = arr / total
    p = p[p > 0]
    H = -np.sum(p * np.log(p))
    return float(np.exp(H))


def main():
    ap = argparse.ArgumentParser(
        description="INC-0160: training routing cost probe")
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--window", type=int, default=500,
                    help="Sliding window size for per-window metrics")
    ap.add_argument("--sample-interval", type=int, default=250,
                    help="Steps between curve samples")
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    with open(args.config, "r", encoding="utf-8") as f:
        cfg = json.load(f)

    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {r["route_id"]: r.get("args", {}) for r in routes}
    route_ids = audit.parse_route_ids(args.routes, route_map.keys())

    results = []
    for route_id in route_ids:
        route_args = audit.build_args(common_args, route_map[route_id],
                                      args.seed)
        input_transform = getattr(route_args, "input_transform", "none")
        print(f"\n[{route_id}] K={route_args.K} "
              f"sector_mode={route_args.sector_mode} "
              f"input_transform={input_transform}")

        # Set up routing state (same as INC-0159)
        state = audit._prepared_route_state(route_args)
        v_tr = state["v_tr"]
        y_tr = state["y_tr"]
        d = int(state["d"])
        dy = int(state["dy"])
        z_tr_eval = state["z_tr_eval"]

        # Training bucket keys (pre-computed, train_route_mode=final_static)
        keys_tr = [
            hr.make_bucket_key(int(state["shell_tr_eval"][i]),
                               int(state["sector_tr_eval"][i]))
            for i in range(v_tr.shape[0])
        ]

        # Initialize buckets
        buckets = hr.init_buckets(keys_tr, dy=dy, d=d,
                                  seed=route_args.seed + 101)

        # Run training loop, recording bucket access per step
        n_train = v_tr.shape[0]
        epochs = int(route_args.epochs)
        total_steps = epochs * n_train
        np.random.seed(route_args.seed + 200)

        # Telemetry arrays
        step_keys = []  # bucket key per step
        cumul_unique_curve = []  # (step, cumul_unique_buckets)
        window_gini_curve = []  # (step, window_gini)

        seen_keys = set()
        visit_counter = Counter()
        step_ctr = 0

        for _epoch in range(epochs):
            order = np.random.permutation(n_train)
            for jj in order:
                key = keys_tr[jj]
                z1 = z_tr_eval[jj]

                # EMA update (same as router_proxy_eval.py)
                _, sj, _ = hr.predict_from_bucket(buckets, key, z1,
                                                  d=d, dy=dy)
                hr.ema_update(
                    hr.get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                    z=z1,
                    y=y_tr[jj],
                    eta_p=float(route_args.eta_p),
                    eta_m=float(route_args.eta_m),
                )

                # Record access
                step_keys.append(key)
                seen_keys.add(key)
                visit_counter[key] += 1
                step_ctr += 1

                # Sample curves at intervals
                if step_ctr % args.sample_interval == 0 or step_ctr == total_steps:
                    cumul_unique_curve.append({
                        "step": step_ctr,
                        "cumul_unique_buckets": len(seen_keys),
                    })

                    # Window Gini
                    if step_ctr >= args.window:
                        window_keys = step_keys[-args.window:]
                        window_counts = list(Counter(window_keys).values())
                        wg = gini_coefficient(window_counts)
                    else:
                        window_counts = list(Counter(step_keys).values())
                        wg = gini_coefficient(window_counts)
                    window_gini_curve.append({
                        "step": step_ctr,
                        "window_gini": round(wg, 6),
                    })

        # Final routing cost metrics
        visit_counts = np.array(list(visit_counter.values()), dtype=np.float64)
        unique_buckets = len(visit_counter)
        eff_count = effective_count(visit_counts)
        training_gini = gini_coefficient(visit_counts)
        density = total_steps / max(unique_buckets, 1)

        # Top-half concentration
        sorted_counts = np.sort(visit_counts)[::-1]
        n_top = max(1, len(sorted_counts) // 2)
        top_half_conc = float(sorted_counts[:n_top].sum() / sorted_counts.sum())

        # Bucket coverage (how many of K possible buckets used)
        coverage = unique_buckets / max(int(route_args.K), 1)

        # Eval-data routing sparsity (for cross-check with INC-0159)
        keys_ev = [
            hr.make_bucket_key(int(state["shell_ev"][i]),
                               int(state["sector_ev"][i]))
            for i in range(state["v_ev"].shape[0])
        ]
        eval_counter = Counter(keys_ev)
        eval_unique = len(eval_counter)
        eval_counts = np.array(list(eval_counter.values()), dtype=np.float64)
        eval_gini = gini_coefficient(eval_counts)
        eval_eff = effective_count(eval_counts)

        print(f"  TRAINING: unique_buckets={unique_buckets}  "
              f"effective={eff_count:.1f}  gini={training_gini:.4f}  "
              f"density={density:.1f}  top_half_conc={top_half_conc:.4f}  "
              f"coverage={coverage:.4f}")
        print(f"  EVAL:     unique_buckets={eval_unique}  "
              f"effective={eval_eff:.1f}  gini={eval_gini:.4f}")

        results.append({
            "route_id": route_id,
            "args": {
                "K": int(route_args.K),
                "sector_mode": route_args.sector_mode,
                "input_transform": input_transform,
                "seed": int(args.seed),
            },
            "training_cost": {
                "total_steps": total_steps,
                "unique_buckets": unique_buckets,
                "effective_bucket_count": round(eff_count, 4),
                "training_gini": round(training_gini, 6),
                "per_bucket_density": round(density, 4),
                "top_half_concentration": round(top_half_conc, 6),
                "bucket_coverage": round(coverage, 6),
            },
            "eval_routing": {
                "unique_buckets": eval_unique,
                "effective_bucket_count": round(eval_eff, 4),
                "eval_gini": round(eval_gini, 6),
            },
            "curves": {
                "cumul_unique": cumul_unique_curve,
                "window_gini": window_gini_curve,
            },
        })

    payload = {
        "experiment": "inc0160_training_routing_cost",
        "config": args.config,
        "seed": int(args.seed),
        "results": results,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    print(f"\nWrote {args.output}")

    # Print summary comparison
    if len(results) >= 2:
        print("\n=== COMPARISON ===")
        for i in range(0, len(results) - 1, 2):
            r0 = results[i]
            r1 = results[i + 1]
            t0 = r0["training_cost"]
            t1 = r1["training_cost"]
            print(f"\n  {r0['route_id']} vs {r1['route_id']}:")
            ub_ratio = t1["unique_buckets"] / max(t0["unique_buckets"], 1)
            eff_ratio = t1["effective_bucket_count"] / max(t0["effective_bucket_count"], 1)
            gini_ratio = t0["training_gini"] / max(t1["training_gini"], 0.001)
            density_ratio = t0["per_bucket_density"] / max(t1["per_bucket_density"], 1)
            print(f"    unique_bucket_ratio (PERM/ORIG): {ub_ratio:.4f}")
            print(f"    effective_bucket_ratio (PERM/ORIG): {eff_ratio:.4f}")
            print(f"    gini_ratio (ORIG/PERM): {gini_ratio:.4f}")
            print(f"    density_ratio (ORIG/PERM): {density_ratio:.4f}")


if __name__ == "__main__":
    main()
