#!/usr/bin/env python3
"""Runner for learned weighting over lawful H_v6 operator entries."""

from __future__ import annotations

from geometry_native_operator_model_v8 import (
    OUTPUT_PATH_OPERATOR_WEIGHTING_V2,
    TaskConfigV8,
    run_operator_weighting_v2,
    write_operator_weighting_v2,
)


def main() -> None:
    metrics = run_operator_weighting_v2(TaskConfigV8(), seed=241)
    write_operator_weighting_v2(metrics, OUTPUT_PATH_OPERATOR_WEIGHTING_V2)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WEIGHTING_V2}")
    for row in metrics:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
