#!/usr/bin/env python3
"""Runner for the sixth inner-representation rebuild."""

from __future__ import annotations

import csv
from pathlib import Path

from geometry_native_sequence_model_r6 import TaskConfigR6, run_bounded_sequence_comparison_r6


OUTPUT_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_inner_representation_rebuild_v6.csv"
)


def main() -> None:
    metrics = run_bounded_sequence_comparison_r6(TaskConfigR6(), seed=241)
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


if __name__ == "__main__":
    main()
