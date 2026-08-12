#!/usr/bin/env python3
"""Runner for learned weighting over lawful H_v5 operator support only."""

from __future__ import annotations

from geometry_native_operator_model_v6 import OUTPUT_PATH_OPERATOR_WEIGHTING_V1, TaskConfigV6, run_operator_weighting_v1, write_operator_weighting_v1


def main() -> None:
    rows = run_operator_weighting_v1(TaskConfigV6(), seed=241)
    write_operator_weighting_v1(rows, OUTPUT_PATH_OPERATOR_WEIGHTING_V1)
    print(f"wrote {OUTPUT_PATH_OPERATOR_WEIGHTING_V1}")
    for row in rows:
        print(
            f"{row.model}: test_accuracy={row.test_accuracy:.4f} "
            f"query_accuracy={row.query_accuracy:.4f} "
            f"test_loss={row.test_loss:.4f}"
        )


if __name__ == "__main__":
    main()
