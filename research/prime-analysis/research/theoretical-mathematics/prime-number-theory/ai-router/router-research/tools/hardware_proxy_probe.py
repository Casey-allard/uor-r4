#!/usr/bin/env python3
"""INC-0165: Hardware proxy probe.

Extends the INC-0164 training loop with three hardware proxy models:
  Model A: cumulative effective-bucket cost (sum of per-step eff counts)
  Model B: cache-line grouping (G=1,2,4 buckets per line, distinct line touches)
  Model C: LRU cache reuse (capacities 8,16,32; miss counts)

NO MSE. Cosine similarity used ONLY as matched-progress checkpoint variable.
"""
import argparse
import json
import os
import sys
from collections import Counter, OrderedDict

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


def cosine_similarity(a, b):
    na = np.linalg.norm(a)
    nb = np.linalg.norm(b)
    if na == 0 or nb == 0:
        return 0.0
    return float(np.dot(a, b) / (na * nb))


def eval_cosine(buckets, keys, z, y, d, dy):
    n = z.shape[0]
    sims = np.zeros(n, dtype=np.float64)
    for i in range(n):
        yhat, _, _ = hr.predict_from_bucket(buckets, keys[i], z[i], d=d, dy=dy)
        sims[i] = cosine_similarity(yhat, y[i])
    return float(np.mean(sims))


class LRUCache:
    """Simple LRU cache tracking misses."""
    def __init__(self, capacity):
        self.capacity = capacity
        self.cache = OrderedDict()
        self.misses = 0
        self.accesses = 0

    def access(self, key):
        self.accesses += 1
        if key in self.cache:
            self.cache.move_to_end(key)
        else:
            self.misses += 1
            if len(self.cache) >= self.capacity:
                self.cache.popitem(last=False)
            self.cache[key] = True


def cache_line_key(bucket_key, granularity):
    """Map a bucket key to a cache-line key at given granularity."""
    # bucket_key is (shell, sector). Group by sector // granularity
    shell, sector = bucket_key
    return (shell, sector // granularity)


def main():
    ap = argparse.ArgumentParser(
        description="INC-0165: hardware proxy probe")
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--window", type=int, default=500)
    ap.add_argument("--sample-interval", type=int, default=100)
    ap.add_argument("--eval-interval", type=int, default=500)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    with open(args.config, "r", encoding="utf-8") as f:
        cfg = json.load(f)

    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {r["route_id"]: r.get("args", {}) for r in routes}
    route_ids = audit.parse_route_ids(args.routes, route_map.keys())

    CACHE_LINE_GRANULARITIES = [1, 2, 4]
    LRU_CAPACITIES = [8, 16, 32]

    results = []
    for route_id in route_ids:
        route_args = audit.build_args(common_args, route_map[route_id],
                                      args.seed)
        input_transform = getattr(route_args, "input_transform", "none")
        print(f"\n[{route_id}] K={route_args.K} "
              f"sector_mode={route_args.sector_mode} "
              f"input_transform={input_transform}")

        state = audit._prepared_route_state(route_args)
        v_tr = state["v_tr"]
        y_tr = state["y_tr"]
        d = int(state["d"])
        dy = int(state["dy"])
        z_tr_eval = state["z_tr_eval"]

        keys_tr = [
            hr.make_bucket_key(int(state["shell_tr_eval"][i]),
                               int(state["sector_tr_eval"][i]))
            for i in range(v_tr.shape[0])
        ]

        v_ev = state["v_ev"]
        y_ev = state["y_ev"]
        z_ev = state.get("z_ev", state.get("z_tr_eval", None))
        keys_ev = [
            hr.make_bucket_key(int(state["shell_ev"][i]),
                               int(state["sector_ev"][i]))
            for i in range(v_ev.shape[0])
        ]

        buckets = hr.init_buckets(keys_tr, dy=dy, d=d,
                                  seed=route_args.seed + 101)

        n_train = v_tr.shape[0]
        epochs = int(route_args.epochs)
        total_steps = epochs * n_train
        bucket_size_bytes = (d + dy) * 8  # float64

        np.random.seed(route_args.seed + 200)

        # Telemetry
        seen_keys = set()
        visit_counter = Counter()
        step_cosines = []

        # Model A: cumulative effective-bucket cost
        cumul_eff_cost = 0.0

        # Model B: cache-line touch sets per granularity
        cl_touched = {g: set() for g in CACHE_LINE_GRANULARITIES}

        # Model C: LRU caches
        lru_caches = {c: LRUCache(c) for c in LRU_CAPACITIES}

        # Progress curve with hardware metrics
        progress_curve = []
        step_ctr = 0

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

                # Record access
                seen_keys.add(key)
                visit_counter[key] += 1
                step_ctr += 1

                # Model A: add current effective count
                cumul_counts = np.array(list(visit_counter.values()),
                                       dtype=np.float64)
                cur_eff = effective_count(cumul_counts)
                cumul_eff_cost += cur_eff

                # Model B: cache-line touches
                for g in CACHE_LINE_GRANULARITIES:
                    cl_touched[g].add(cache_line_key(key, g))

                # Model C: LRU access
                for c in LRU_CAPACITIES:
                    lru_caches[c].access(key)

                # Sample progress curve
                if step_ctr % args.sample_interval == 0 or step_ctr == total_steps:
                    w = min(args.window, len(step_cosines))
                    window_cos = float(np.mean(step_cosines[-w:]))
                    cumul_unique = len(seen_keys)
                    cumul_gini = gini_coefficient(cumul_counts)

                    pt = {
                        "step": step_ctr,
                        "window_cosine": round(window_cos, 6),
                        "cumul_unique_buckets": cumul_unique,
                        "cumul_effective_buckets": round(cur_eff, 4),
                        "cumul_gini": round(cumul_gini, 6),
                        # Model A
                        "cumul_eff_cost": round(cumul_eff_cost, 2),
                        # Model B
                        "cache_line_touches": {
                            str(g): len(cl_touched[g])
                            for g in CACHE_LINE_GRANULARITIES
                        },
                        # Model C
                        "lru_misses": {
                            str(c): lru_caches[c].misses
                            for c in LRU_CAPACITIES
                        },
                        "lru_miss_ratio": {
                            str(c): round(lru_caches[c].misses
                                          / max(lru_caches[c].accesses, 1), 6)
                            for c in LRU_CAPACITIES
                        },
                        # Bytes moved (Model C, LRU-16 as default)
                        "bytes_moved_lru16": lru_caches[16].misses * bucket_size_bytes,
                    }
                    progress_curve.append(pt)

        # Final summary
        visit_counts = np.array(list(visit_counter.values()), dtype=np.float64)
        unique_buckets = len(visit_counter)
        final_eff = effective_count(visit_counts)
        final_gini = gini_coefficient(visit_counts)

        w = min(args.window, len(step_cosines))
        final_window_cos = float(np.mean(step_cosines[-w:]))
        final_eval_cos = eval_cosine(buckets, keys_ev, z_ev, y_ev, d, dy)

        print(f"  TRAINING: unique={unique_buckets}  eff={final_eff:.1f}  "
              f"gini={final_gini:.4f}  cos={final_window_cos:.4f}")
        print(f"  EVAL:     cos={final_eval_cos:.4f}")
        print(f"  HW: eff_cost={cumul_eff_cost:.0f}  "
              f"cl_G1={len(cl_touched[1])}  "
              f"lru16_misses={lru_caches[16].misses}  "
              f"lru16_ratio={lru_caches[16].misses/max(lru_caches[16].accesses,1):.4f}")

        results.append({
            "route_id": route_id,
            "args": {
                "K": int(route_args.K),
                "sector_mode": route_args.sector_mode,
                "input_transform": input_transform,
                "seed": int(args.seed),
            },
            "summary": {
                "total_steps": total_steps,
                "unique_buckets": unique_buckets,
                "effective_bucket_count": round(final_eff, 4),
                "training_gini": round(final_gini, 6),
                "final_window_cosine": round(final_window_cos, 6),
                "final_eval_cosine": round(final_eval_cos, 6),
                "bucket_size_bytes": bucket_size_bytes,
                # Model A final
                "cumul_eff_cost": round(cumul_eff_cost, 2),
                # Model B final
                "cache_line_touches": {
                    str(g): len(cl_touched[g])
                    for g in CACHE_LINE_GRANULARITIES
                },
                # Model C final
                "lru_misses": {
                    str(c): lru_caches[c].misses
                    for c in LRU_CAPACITIES
                },
                "lru_miss_ratio": {
                    str(c): round(lru_caches[c].misses
                                  / max(lru_caches[c].accesses, 1), 6)
                    for c in LRU_CAPACITIES
                },
                "bytes_moved_lru16": lru_caches[16].misses * bucket_size_bytes,
            },
            "progress_curve": progress_curve,
        })

    payload = {
        "experiment": "inc0165_hardware_proxy",
        "config": args.config,
        "seed": int(args.seed),
        "results": results,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    print(f"\nWrote {args.output}")

    if len(results) >= 2:
        print("\n=== COMPARISON ===")
        for i in range(0, len(results) - 1, 2):
            r0 = results[i]
            r1 = results[i + 1]
            s0 = r0["summary"]
            s1 = r1["summary"]
            print(f"\n  {r0['route_id']} vs {r1['route_id']}:")
            eff_ratio = s1["effective_bucket_count"] / max(s0["effective_bucket_count"], 1)
            cost_ratio = s1["cumul_eff_cost"] / max(s0["cumul_eff_cost"], 1)
            lru16_ratio = s1["lru_misses"]["16"] / max(s0["lru_misses"]["16"], 1)
            print(f"    eff_bucket_ratio:  {eff_ratio:.4f}")
            print(f"    eff_cost_ratio:    {cost_ratio:.4f}")
            print(f"    lru16_miss_ratio:  {lru16_ratio:.4f}")


if __name__ == "__main__":
    main()
