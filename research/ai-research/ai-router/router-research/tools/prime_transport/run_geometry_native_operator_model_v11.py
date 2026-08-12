#!/usr/bin/env python3
"""Runner for lawful H_v7 operator weighting with tau-aware transport state."""

from __future__ import annotations

from geometry_native_operator_model_v11 import (
    OUTPUT_PATH_OPERATOR_WEIGHTING_V3,
    TaskConfigV8,
    run_operator_weighting_v3,
    write_operator_weighting_v3,
)


def main() -> None:
    metrics = run_operator_weighting_v3(TaskConfigV8(), seed=241)
    write_operator_weighting_v3(metrics, OUTPUT_PATH_OPERATOR_WEIGHTING_V3)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WEIGHTING_V3}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
