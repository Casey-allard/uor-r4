#!/usr/bin/env python3
"""INC-0159: Routing sparsity probe.

Tests whether geometry-native routing produces a sparser representation
than permuted routing — i.e., whether the label signal is more concentrated
in fewer buckets and fewer spectral modes.

Mathematical object: bucket semantic coherence → routing sparsity.

Observables (NOT MSE — INC-0155 proved MSE invalid):
- Bucket purity / entropy distribution (INC-0157/0158)
- label_indicator_lowfreq_max spectral signal (INC-0156/0158)
- Purity-weighted bucket concentration: what fraction of samples
  live in high-purity buckets?
- Information density: MI / n_active_buckets
- Bucket Gini coefficient: how uneven is sample distribution across buckets?

K=75 is the geometric growth boundary (INC-0146: K=75 resolves Hopf fiber
bin dilution, K=50 does not).
"""
import argparse
import json
import os
import sys

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr
from tools import spectral_route_audit as audit
from tools.bucket_coherence_probe import bucket_coherence_metrics
from tools.spectral_signal_probe import label_signal_metrics


def compute_per_bucket_purity(keys, y_onehot):
    """Compute label purity for each combined (shell, sector) bucket key."""
    n_classes = y_onehot.shape[1]
    labels = np.argmax(y_onehot, axis=1)
    buckets = {}
    for i, key in enumerate(keys):
        buckets.setdefault(key, []).append(i)
    purity_map = {}
    for key, indices in buckets.items():
        bucket_labels = labels[indices]
        counts = np.bincount(bucket_labels, minlength=n_classes)
        purity_map[key] = float(np.max(counts)) / float(np.sum(counts))
    return purity_map


def bucket_sparsity_metrics(keys, y_onehot, purity_thresholds):
    """Compute routing sparsity metrics from bucket assignments.

    Measures how concentrated the label signal is across buckets:
    - What fraction of buckets exceed each purity threshold?
    - What fraction of SAMPLES live in those high-purity buckets?
    - Gini coefficient of bucket size distribution
    - Information density: MI per bucket
    """
    n_classes = y_onehot.shape[1]
    labels = np.argmax(y_onehot, axis=1)
    n_total = len(labels)

    # Bucket sizes and purities
    buckets = {}
    for i, key in enumerate(keys):
        buckets.setdefault(key, []).append(i)

    n_buckets = len(buckets)
    sizes = []
    purities = []
    entropies = []

    for key, indices in buckets.items():
        count = len(indices)
        sizes.append(count)
        bucket_labels = labels[indices]
        counts = np.bincount(bucket_labels, minlength=n_classes).astype(np.float64)
        probs = counts / counts.sum()
        purities.append(float(np.max(probs)))
        nonzero = probs[probs > 0]
        entropies.append(float(-np.sum(nonzero * np.log2(nonzero))))

    sizes = np.array(sizes, dtype=np.float64)
    purities = np.array(purities)
    entropies = np.array(entropies)

    # Gini coefficient of bucket size distribution
    sorted_sizes = np.sort(sizes)
    n = len(sorted_sizes)
    if n == 0 or sorted_sizes.sum() == 0:
        gini = 0.0
    else:
        index = np.arange(1, n + 1)
        gini = float((2 * np.sum(index * sorted_sizes) - (n + 1) * np.sum(sorted_sizes))
                      / (n * np.sum(sorted_sizes)))

    # Coverage at each purity threshold
    threshold_stats = {}
    for t in purity_thresholds:
        mask = purities >= t
        n_above = int(np.sum(mask))
        samples_above = int(sizes[mask].sum()) if n_above > 0 else 0
        threshold_stats[f"t{t:.2f}"] = {
            "n_buckets_above": n_above,
            "frac_buckets_above": n_above / max(n_buckets, 1),
            "n_samples_in_pure_buckets": samples_above,
            "frac_samples_in_pure_buckets": samples_above / max(n_total, 1),
        }

    return {
        "n_buckets": n_buckets,
        "n_samples": n_total,
        "n_classes": n_classes,
        "purity_mean": float(np.mean(purities)),
        "purity_std": float(np.std(purities)),
        "purity_max": float(np.max(purities)),
        "purity_min": float(np.min(purities)),
        "entropy_mean": float(np.mean(entropies)),
        "entropy_std": float(np.std(entropies)),
        "size_mean": float(np.mean(sizes)),
        "size_std": float(np.std(sizes)),
        "size_gini": gini,
        "thresholds": threshold_stats,
    }


def main():
    ap = argparse.ArgumentParser(
        description="INC-0159: routing sparsity probe")
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--purity-thresholds", type=str, default="0.10,0.15,0.20,0.25,0.30,0.50",
                    help="Comma-separated purity thresholds for concentration analysis.")
    ap.add_argument("--max-points", type=int, default=384,
                    help="Max eval points for spectral graph (matches INC-0156/0158)")
    ap.add_argument("--knn-k", type=int, default=12)
    ap.add_argument("--lowfreq-modes", type=int, default=8)
    ap.add_argument("--graph-mode", type=str, default="poincare_4d",
                    choices=["ambient_euclidean", "hopf_coords", "poincare_4d"])
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    with open(args.config, "r", encoding="utf-8") as f:
        cfg = json.load(f)

    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {r["route_id"]: r.get("args", {}) for r in routes}
    route_ids = audit.parse_route_ids(args.routes, route_map.keys())
    thresholds = [float(t) for t in args.purity_thresholds.split(",")]

    results = []
    for route_id in route_ids:
        route_args = audit.build_args(common_args, route_map[route_id],
                                      args.seed)
        input_transform = getattr(route_args, "input_transform", "none")
        print(f"\n[{route_id}] K={route_args.K} "
              f"sector_mode={route_args.sector_mode} "
              f"input_transform={input_transform}")

        # Compute routing state (no training)
        state = audit._prepared_route_state(route_args)
        n_eval = state["y_ev"].shape[0]

        # Bucket keys for eval data
        keys_ev = [
            hr.make_bucket_key(int(state["shell_ev"][i]),
                               int(state["sector_ev"][i]))
            for i in range(n_eval)
        ]

        # Bucket coherence on eval data (sector-based)
        coherence = bucket_coherence_metrics(
            state["sector_ev"].astype(np.int64), state["y_ev"])

        # Bucket sparsity metrics on eval data (using combined shell+sector keys)
        sparsity = bucket_sparsity_metrics(keys_ev, state["y_ev"], thresholds)

        # Spectral signal
        idx_spectral = audit._sample_indices(
            state["route_z_ev"].shape[0], max_points=args.max_points,
            seed=route_args.seed)
        route_z_sub = state["route_z_ev"][idx_spectral]
        y_ev_sub = state["y_ev"][idx_spectral]
        v_ev_sub = state["v_ev"][idx_spectral]

        decomp = audit.spectral_decomposition(
            route_z_sub, knn_k=args.knn_k,
            graph_mode=args.graph_mode, v_ev=v_ev_sub,
            dims=state["dims"])
        spectral = label_signal_metrics(
            decomp["laplacian"], decomp["evecs"], y_ev_sub,
            lowfreq_modes=args.lowfreq_modes)

        # Spectral graph metrics
        graph_metrics = audit.spectral_metrics_from_decomposition(
            decomp, route_z=route_z_sub,
            shell=state["shell_ev"][idx_spectral],
            sector=state["sector_ev"][idx_spectral],
            lowfreq_modes=args.lowfreq_modes)

        # Information density
        mi = coherence["bucket_label_mi"]
        n_bk = coherence["n_buckets_active"]
        info_density = mi / max(n_bk, 1)

        # Print summary
        print(f"  buckets={n_bk}  purity={sparsity['purity_mean']:.4f}  "
              f"entropy={sparsity['entropy_mean']:.4f}  "
              f"lowfreq_max={spectral['label_indicator_lowfreq_max']:.6f}  "
              f"MI={mi:.4f}  info_density={info_density:.6f}  "
              f"gini={sparsity['size_gini']:.4f}")
        for tkey, tval in sparsity["thresholds"].items():
            print(f"    {tkey}: {tval['n_buckets_above']}/{n_bk} buckets "
                  f"({tval['frac_buckets_above']:.3f}), "
                  f"samples_covered={tval['frac_samples_in_pure_buckets']:.4f}")

        results.append({
            "route_id": route_id,
            "args": {
                "K": int(route_args.K),
                "sector_mode": route_args.sector_mode,
                "input_transform": input_transform,
                "seed": int(args.seed),
            },
            "coherence": coherence,
            "sparsity": sparsity,
            "spectral": spectral,
            "graph_metrics": {
                "spectral_lambda2": graph_metrics["spectral_lambda2"],
                "spectral_gap_23": graph_metrics["spectral_gap_23"],
                "spectral_lowfreq_mass_ratio": graph_metrics["spectral_lowfreq_mass_ratio"],
                "shell_lowfreq_energy": graph_metrics["shell_lowfreq_energy"],
                "sector_lowfreq_energy": graph_metrics["sector_lowfreq_energy"],
            },
            "efficiency": {
                "bucket_label_mi": mi,
                "n_active_buckets": n_bk,
                "info_density_per_bucket": info_density,
            },
        })

    payload = {
        "experiment": "inc0159_routing_sparsity",
        "config": args.config,
        "seed": int(args.seed),
        "purity_thresholds": thresholds,
        "graph_mode": args.graph_mode,
        "max_points": args.max_points,
        "knn_k": args.knn_k,
        "lowfreq_modes": args.lowfreq_modes,
        "n_routes": len(route_ids),
        "results": results,
    }
    os.makedirs(os.path.dirname(os.path.abspath(args.output)), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)
    print(f"\nWrote {args.output}")


def compute_per_bucket_purity(keys, y_onehot):
    """Compute label purity for each combined (shell, sector) bucket key.

    Returns dict mapping bucket_key -> purity (float in [0, 1]).
    """
    n_classes = y_onehot.shape[1]
    labels = np.argmax(y_onehot, axis=1)
    buckets = {}
    for i, key in enumerate(keys):
        buckets.setdefault(key, []).append(i)
    purity_map = {}
    for key, indices in buckets.items():
        bucket_labels = labels[indices]
        counts = np.bincount(bucket_labels, minlength=n_classes)
        purity_map[key] = float(np.max(counts)) / float(np.sum(counts))
    return purity_map




if __name__ == "__main__":
    main()


if __name__ == "__main__":
    main()
