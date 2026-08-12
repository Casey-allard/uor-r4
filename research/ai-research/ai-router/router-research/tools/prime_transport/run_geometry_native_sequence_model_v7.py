#!/usr/bin/env python3
"""Run the reduced-schema-alignment transfer comparison."""

from __future__ import annotations

import csv
from pathlib import Path

from geometry_native_sequence_model_v7 import TaskConfigV7, run_bounded_sequence_comparison_v7


ROOT = Path(__file__).resolve().parents[2]
RESULTS_DIR = ROOT / "results" / "prime_transport_recursive_system"
OUTPUT_CSV = RESULTS_DIR / "prime_transport_geometry_native_sequence_model_v7.csv"


def main() -> None:
    RESULTS_DIR.mkdir(parents=True, exist_ok=True)
    metrics = run_bounded_sequence_comparison_v7(TaskConfigV7(), seed=89)

    with OUTPUT_CSV.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(
            [
                "model",
                "train_loss",
                "test_loss",
                "test_accuracy",
                "transfer_query_accuracy",
                "param_count",
                "effective_state_size",
                "train_seconds",
                "eval_seconds",
            ]
        )
        for row in metrics:
            writer.writerow(
                [
                    row.model,
                    f"{row.train_loss:.12f}",
                    f"{row.test_loss:.12f}",
                    f"{row.test_accuracy:.12f}",
                    f"{row.transfer_query_accuracy:.12f}",
                    row.param_count,
                    row.effective_state_size,
                    f"{row.train_seconds:.12f}",
                    f"{row.eval_seconds:.12f}",
                ]
            )

    print(f"wrote {OUTPUT_CSV}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"transfer_query_accuracy={row.transfer_query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
