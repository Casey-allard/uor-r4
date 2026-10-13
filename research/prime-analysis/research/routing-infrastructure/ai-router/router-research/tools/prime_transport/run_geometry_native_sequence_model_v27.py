#!/usr/bin/env python3
"""Runner for transfer robustness of the learned regional field."""

from __future__ import annotations

import csv
from pathlib import Path

from geometry_native_sequence_model_v27 import TaskConfigV27, run_bounded_sequence_comparison_v27


OUTPUT_PATH = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_geometry_native_sequence_model_v27.csv"
)


def main() -> None:
    metrics = run_bounded_sequence_comparison_v27(TaskConfigV27(), seed=167)
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "model",
                "test_loss",
                "test_accuracy",
                "transfer_query_accuracy",
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
            f"transfer_query_accuracy={metric.transfer_query_accuracy:.4f} "
            f"test_loss={metric.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
