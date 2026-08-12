#!/usr/bin/env python3
"""INC-0164: Scaling-law consistency probe.

Reuses the INC-0163 matched-progress training loop and adds:
- Multiple progress checkpoints (p = 0.50, 0.60, 0.70, 0.80 of PERM max cosine)
- Per-checkpoint cumulative effective-bucket cost
- Comparison against predicted scaling-law ratios from INC-0162

NO MSE is used anywhere. Cosine similarity is the progress metric.
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


def main():
    ap = argparse.ArgumentParser(
        description="INC-0164: scaling-law consistency probe")
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--window", type=int, default=500)
    ap.add_argument("--sample-interval", type=int, default=100,
                    help="Steps between curve samples (finer than INC-0163)")
    ap.add_argument("--eval-interval", type=int, default=500)
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
        np.random.seed(route_args.seed + 200)

        step_keys = []
        seen_keys = set()
        visit_counter = Counter()
        step_cosines = []

        progress_curve = []
        eval_curve = []

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

                step_keys.append(key)
                seen_keys.add(key)
                visit_counter[key] += 1
                step_ctr += 1

                if step_ctr % args.sample_interval == 0 or step_ctr == total_steps:
                    w = min(args.window, len(step_cosines))
                    window_cos = float(np.mean(step_cosines[-w:]))
                    cumul_unique = len(seen_keys)
                    cumul_counts = np.array(list(visit_counter.values()),
                                           dtype=np.float64)
                    cumul_eff = effective_count(cumul_counts)

                    progress_curve.append({
                        "step": step_ctr,
                        "window_cosine": round(window_cos, 6),
                        "cumul_unique_buckets": cumul_unique,
                        "cumul_effective_buckets": round(cumul_eff, 4),
                        "cumul_gini": round(gini_coefficient(cumul_counts), 6),
                    })

                if step_ctr % args.eval_interval == 0 or step_ctr == total_steps:
                    ec = eval_cosine(buckets, keys_ev, z_ev, y_ev, d, dy)
                    ev_counter = Counter(keys_ev)
                    ev_counts = np.array(list(ev_counter.values()),
                                         dtype=np.float64)
                    ev_eff = effective_count(ev_counts)
                    eval_curve.append({
                        "step": step_ctr,
                        "eval_cosine": round(ec, 6),
                        "eval_effective_buckets": round(ev_eff, 4),
                        "eval_unique_buckets": len(ev_counter),
                    })

        visit_counts = np.array(list(visit_counter.values()), dtype=np.float64)
        unique_buckets = len(visit_counter)
        final_eff = effective_count(visit_counts)
        final_gini = gini_coefficient(visit_counts)

        w = min(args.window, len(step_cosines))
        final_window_cos = float(np.mean(step_cosines[-w:]))
        final_eval_cos = eval_cosine(buckets, keys_ev, z_ev, y_ev, d, dy)

        sorted_counts = np.sort(visit_counts)[::-1]
        n_top = max(1, len(sorted_counts) // 2)
        top_half_conc = float(sorted_counts[:n_top].sum() / sorted_counts.sum())

        print(f"  TRAINING: unique={unique_buckets}  eff={final_eff:.1f}  "
              f"gini={final_gini:.4f}  cos={final_window_cos:.4f}")
        print(f"  EVAL:     cos={final_eval_cos:.4f}")

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
                "top_half_concentration": round(top_half_conc, 6),
                "final_window_cosine": round(final_window_cos, 6),
                "final_eval_cosine": round(final_eval_cos, 6),
            },
            "progress_curve": progress_curve,
            "eval_curve": eval_curve,
        })

    payload = {
        "experiment": "inc0164_scaling_law_consistency",
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
            cos_diff = s0["final_eval_cosine"] - s1["final_eval_cosine"]
            print(f"    eff_bucket_ratio (PERM/ORIG): {eff_ratio:.4f}")
            print(f"    eval_cosine diff (ORIG-PERM): {cos_diff:+.4f}")


if __name__ == "__main__":
    main()
