#!/usr/bin/env python3
"""Runner for lawful operator weighting with active tau-augmented transport state."""

from __future__ import annotations

from geometry_native_operator_model_v9 import (
    OUTPUT_PATH_OPERATOR_WITH_TAU_V1,
    TaskConfigV8,
    run_operator_with_tau_v1,
    write_operator_with_tau_v1,
)


def main() -> None:
    metrics = run_operator_with_tau_v1(TaskConfigV8(), seed=241)
    write_operator_with_tau_v1(metrics, OUTPUT_PATH_OPERATOR_WITH_TAU_V1)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WITH_TAU_V1}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
