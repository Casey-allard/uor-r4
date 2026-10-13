#!/usr/bin/env python3
"""INC-0166 Part B: K=100 boundary audit probe.

Records shell and sector occupancy distributions to diagnose the
reproducible K≈100 dip in BASE effective-bucket ratios.

NO MSE. Cosine used only as progress checkpoint variable.
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
    arr = np.sort(np.asarray(counts, dtype=np.float64))
    n = len(arr)
    if n == 0 or arr.sum() == 0:
        return 0.0
    index = np.arange(1, n + 1)
    return float((2 * np.sum(index * arr) - (n + 1) * np.sum(arr))
                  / (n * np.sum(arr)))


def effective_count(counts):
    arr = np.asarray(counts, dtype=np.float64)
    total = arr.sum()
    if total == 0:
        return 0.0
    p = arr / total
    p = p[p > 0]
    H = -np.sum(p * np.log(p))
    return float(np.exp(H))


def entropy(counts):
    arr = np.asarray(counts, dtype=np.float64)
    total = arr.sum()
    if total == 0:
        return 0.0
    p = arr / total
    p = p[p > 0]
    return float(-np.sum(p * np.log(p)))


def cosine_similarity(a, b):
    na = np.linalg.norm(a)
    nb = np.linalg.norm(b)
    if na == 0 or nb == 0:
        return 0.0
    return float(np.dot(a, b) / (na * nb))


def main():
    ap = argparse.ArgumentParser(
        description="INC-0166: K=100 boundary audit probe")
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
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
        K = int(route_args.K)
        print(f"\n[{route_id}] K={K}  "
              f"sector_mode={route_args.sector_mode}  "
              f"input_transform={input_transform}")

        state = audit._prepared_route_state(route_args)
        v_tr = state["v_tr"]
        y_tr = state["y_tr"]
        d = int(state["d"])
        dy = int(state["dy"])
        z_tr_eval = state["z_tr_eval"]

        shell_tr = state["shell_tr_eval"]
        sector_tr = state["sector_tr_eval"]

        keys_tr = [
            hr.make_bucket_key(int(shell_tr[i]), int(sector_tr[i]))
            for i in range(v_tr.shape[0])
        ]

        # --- Static routing distributions (before training) ---
        shell_counter = Counter(int(s) for s in shell_tr)
        sector_counter = Counter(int(s) for s in sector_tr)
        bucket_counter = Counter(keys_tr)

        n_active_shells = len(shell_counter)
        n_active_sectors = len(sector_counter)
        n_active_buckets = len(bucket_counter)

        shell_counts = np.array(list(shell_counter.values()), dtype=np.float64)
        sector_counts = np.array(list(sector_counter.values()), dtype=np.float64)
        bucket_counts = np.array(list(bucket_counter.values()), dtype=np.float64)

        shell_entropy = entropy(shell_counts)
        sector_entropy = entropy(sector_counts)
        bucket_entropy = entropy(bucket_counts)

        shell_eff = effective_count(shell_counts)
        sector_eff = effective_count(sector_counts)
        bucket_eff = effective_count(bucket_counts)

        bucket_gini = gini_coefficient(bucket_counts)

        # Top-half concentration
        sorted_bc = np.sort(bucket_counts)[::-1]
        total_mass = sorted_bc.sum()
        half_n = max(1, len(sorted_bc) // 2)
        top_half_conc = float(sorted_bc[:half_n].sum() / total_mass) if total_mass > 0 else 0.0

        # Average mass per shell / sector
        avg_mass_per_shell = float(np.mean(shell_counts))
        avg_mass_per_sector = float(np.mean(sector_counts))

        # Shell distribution (sorted by shell id)
        shell_dist = {str(k): int(v) for k, v in sorted(shell_counter.items())}
        sector_dist = {str(k): int(v) for k, v in sorted(sector_counter.items())}

        # Shell-level Gini
        shell_gini = gini_coefficient(shell_counts)
        sector_gini = gini_coefficient(sector_counts)

        # --- Training to get visit-weighted distributions ---
        buckets = hr.init_buckets(keys_tr, dy=dy, d=d,
                                  seed=route_args.seed + 101)
        n_train = v_tr.shape[0]
        epochs = int(route_args.epochs)
        np.random.seed(route_args.seed + 200)

        visit_counter = Counter()
        step_cosines = []

        for _epoch in range(epochs):
            order = np.random.permutation(n_train)
            for jj in order:
                key = keys_tr[jj]
                z1 = z_tr_eval[jj]
                yhat, sj, _ = hr.predict_from_bucket(buckets, key, z1,
                                                      d=d, dy=dy)
                cos_sim = cosine_similarity(yhat, y_tr[jj])
                step_cosines.append(cos_sim)

                hr.ema_update(
                    hr.get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                    z=z1, y=y_tr[jj],
                    eta_p=float(route_args.eta_p),
                    eta_m=float(route_args.eta_m),
                )
                visit_counter[key] += 1

        # Post-training distribution
        visit_counts = np.array(list(visit_counter.values()), dtype=np.float64)
        train_unique_buckets = len(visit_counter)
        train_eff = effective_count(visit_counts)
        train_gini = gini_coefficient(visit_counts)

        # Visit-weighted shell/sector
        train_shell_counter = Counter()
        train_sector_counter = Counter()
        for (sh, se), cnt in visit_counter.items():
            train_shell_counter[sh] += cnt
            train_sector_counter[se] += cnt

        train_shell_counts = np.array(list(train_shell_counter.values()), dtype=np.float64)
        train_sector_counts = np.array(list(train_sector_counter.values()), dtype=np.float64)
        train_shell_entropy = entropy(train_shell_counts)
        train_sector_entropy = entropy(train_sector_counts)
        train_shell_eff = effective_count(train_shell_counts)
        train_sector_eff = effective_count(train_sector_counts)

        w = min(500, len(step_cosines))
        final_cos = float(np.mean(step_cosines[-w:]))

        print(f"  STATIC: shells={n_active_shells}  sectors={n_active_sectors}  "
              f"buckets={n_active_buckets}  eff={bucket_eff:.1f}  gini={bucket_gini:.4f}")
        print(f"  STATIC: shell_ent={shell_entropy:.4f}  sect_ent={sector_entropy:.4f}  "
              f"top_half={top_half_conc:.4f}")
        print(f"  TRAIN:  unique={train_unique_buckets}  eff={train_eff:.1f}  "
              f"gini={train_gini:.4f}  cos={final_cos:.4f}")
        print(f"  TRAIN:  shell_ent={train_shell_entropy:.4f}  "
              f"sect_ent={train_sector_entropy:.4f}")

        results.append({
            "route_id": route_id,
            "K": K,
            "sector_mode": route_args.sector_mode,
            "input_transform": input_transform,
            "seed": int(args.seed),
            "static": {
                "n_active_shells": n_active_shells,
                "n_active_sectors": n_active_sectors,
                "n_active_buckets": n_active_buckets,
                "shell_entropy": round(shell_entropy, 6),
                "sector_entropy": round(sector_entropy, 6),
                "bucket_entropy": round(bucket_entropy, 6),
                "shell_eff": round(shell_eff, 4),
                "sector_eff": round(sector_eff, 4),
                "bucket_eff": round(bucket_eff, 4),
                "bucket_gini": round(bucket_gini, 6),
                "top_half_concentration": round(top_half_conc, 6),
                "shell_gini": round(shell_gini, 6),
                "sector_gini": round(sector_gini, 6),
                "avg_mass_per_shell": round(avg_mass_per_shell, 2),
                "avg_mass_per_sector": round(avg_mass_per_sector, 2),
                "shell_distribution": shell_dist,
                "sector_distribution": sector_dist,
            },
            "training": {
                "unique_buckets": train_unique_buckets,
                "effective_bucket_count": round(train_eff, 4),
                "gini": round(train_gini, 6),
                "final_window_cosine": round(final_cos, 6),
                "shell_entropy": round(train_shell_entropy, 6),
                "sector_entropy": round(train_sector_entropy, 6),
                "shell_eff": round(train_shell_eff, 4),
                "sector_eff": round(train_sector_eff, 4),
            },
        })

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump({"results": results, "config": args.config,
                    "seed": args.seed}, f, indent=2)
    print(f"\nWrote {args.output}")


if __name__ == "__main__":
    main()
