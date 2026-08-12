#!/usr/bin/env python3
"""Bucket semantic coherence probe for INC-0157B.

For each route, computes:
- per-bucket label distribution
- per-bucket label entropy (Shannon)
- per-bucket label purity (max label fraction)
- bucket-label mutual information I(bucket; label)
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

from tools import spectral_route_audit as audit


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def parse_route_ids(raw: str, available: Iterable[str]) -> List[str]:
    return audit.parse_route_ids(raw, available)


def bucket_coherence_metrics(sector: np.ndarray, y_onehot: np.ndarray) -> Dict[str, Any]:
    """Compute bucket semantic coherence metrics.

    Args:
        sector: (N,) int array of bucket assignments
        y_onehot: (N, C) one-hot label matrix
    """
    labels = np.argmax(y_onehot, axis=1)
    n_classes = y_onehot.shape[1]
    unique_buckets = np.unique(sector)
    n_buckets = len(unique_buckets)
    n_total = len(labels)

    per_bucket_entropy = []
    per_bucket_purity = []
    per_bucket_size = []

    for b in unique_buckets:
        mask = sector == b
        count = int(np.sum(mask))
        if count == 0:
            continue
        per_bucket_size.append(count)
        bucket_labels = labels[mask]
        # label distribution within this bucket
        counts = np.bincount(bucket_labels, minlength=n_classes).astype(np.float64)
        probs = counts / counts.sum()
        # purity = max label fraction
        purity = float(np.max(probs))
        per_bucket_purity.append(purity)
        # Shannon entropy
        nonzero = probs[probs > 0]
        entropy = float(-np.sum(nonzero * np.log2(nonzero)))
        per_bucket_entropy.append(entropy)

    # mutual information I(bucket; label) = H(label) - H(label|bucket)
    # H(label)
    label_counts = np.bincount(labels, minlength=n_classes).astype(np.float64)
    label_probs = label_counts / label_counts.sum()
    label_nonzero = label_probs[label_probs > 0]
    h_label = float(-np.sum(label_nonzero * np.log2(label_nonzero)))

    # H(label|bucket) = sum_b P(bucket=b) * H(label|bucket=b)
    h_label_given_bucket = 0.0
    for i, b in enumerate(unique_buckets):
        mask = sector == b
        count = int(np.sum(mask))
        if count == 0:
            continue
        p_bucket = count / n_total
        h_label_given_bucket += p_bucket * per_bucket_entropy[i]

    mi = h_label - h_label_given_bucket

    return {
        "n_buckets_active": n_buckets,
        "n_classes": n_classes,
        "n_samples": n_total,
        "bucket_purity_mean": float(np.mean(per_bucket_purity)) if per_bucket_purity else 0.0,
        "bucket_purity_std": float(np.std(per_bucket_purity)) if per_bucket_purity else 0.0,
        "bucket_purity_min": float(np.min(per_bucket_purity)) if per_bucket_purity else 0.0,
        "bucket_purity_max": float(np.max(per_bucket_purity)) if per_bucket_purity else 0.0,
        "bucket_entropy_mean": float(np.mean(per_bucket_entropy)) if per_bucket_entropy else 0.0,
        "bucket_entropy_std": float(np.std(per_bucket_entropy)) if per_bucket_entropy else 0.0,
        "bucket_label_mi": mi,
        "h_label": h_label,
        "h_label_given_bucket": h_label_given_bucket,
        "bucket_size_mean": float(np.mean(per_bucket_size)) if per_bucket_size else 0.0,
        "bucket_size_std": float(np.std(per_bucket_size)) if per_bucket_size else 0.0,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--max-points", type=int, default=0,
                    help="0 = use all eval points (no subsampling)")
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
        snap = audit.route_eval_snapshot(route_args, max_points=args.max_points)
        metrics = bucket_coherence_metrics(snap["sector"], snap["y_eval"])
        # Confirm input_transform is applied
        metrics["input_transform"] = getattr(route_args, "input_transform", "none")
        results.append(
            {
                "route_id": route_id,
                "args": {
                    "K": int(route_args.K),
                    "sector_mode": route_args.sector_mode,
                    "input_transform": getattr(route_args, "input_transform", "none"),
                },
                "metrics": metrics,
            }
        )

    payload = {
        "config": args.config,
        "seed": int(args.seed),
        "max_points": int(args.max_points),
        "results": results,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, sort_keys=True)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
