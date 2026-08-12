#!/usr/bin/env python3
"""Runner for the seventh inner-representation rebuild."""

from __future__ import annotations

import csv
from pathlib import Path

from geometry_native_sequence_model_r7 import TaskConfigR7, run_bounded_sequence_comparison_r7


OUTPUT_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v7.csv"
)


def main() -> None:
    metrics, compat_stats = run_bounded_sequence_comparison_r7(TaskConfigR7(), seed=241)
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "model",
                "test_loss",
                "test_accuracy",
                "query_accuracy",
                "param_count",
                "effective_state_size",
                "train_seconds",
                "eval_seconds",
            ],
        )
        writer.writeheader()
        for metric in metrics:
            writer.writerow(metric.__dict__)

    print(f"wrote {OUTPUT_PATH}")
    for metric in metrics:
        print(
            f"{metric.model}: test_accuracy={metric.test_accuracy:.4f} "
            f"query_accuracy={metric.query_accuracy:.4f} "
            f"test_loss={metric.test_loss:.4f}"
        )
    print(
        "r7 compatibility stats: "
        f"selected_radial_mismatch_fraction={compat_stats.selected_radial_mismatch_fraction:.4f} "
        f"selected_spin_mismatch_fraction={compat_stats.selected_spin_mismatch_fraction:.4f} "
        f"selected_compat_mismatch_fraction={compat_stats.selected_compat_mismatch_fraction:.4f} "
        f"avg_scored_candidates_per_side={compat_stats.avg_scored_candidates_per_side:.4f} "
        f"compared_incompatible_fraction={compat_stats.compared_incompatible_fraction:.4f} "
        f"tiers=({compat_stats.tier_usage_fraction_t0:.4f},"
        f"{compat_stats.tier_usage_fraction_t1:.4f},"
        f"{compat_stats.tier_usage_fraction_t2:.4f},"
        f"{compat_stats.tier_usage_fraction_t3:.4f},"
        f"{compat_stats.tier_usage_fraction_t4:.4f})"
    )


if __name__ == "__main__":
    main()
