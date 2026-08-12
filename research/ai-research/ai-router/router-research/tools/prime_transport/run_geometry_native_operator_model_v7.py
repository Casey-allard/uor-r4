#!/usr/bin/env python3
"""Runner for the sixth operator formulation with refined lawful spin lift."""

from __future__ import annotations

from geometry_native_operator_model_v7 import (
    OUTPUT_PATH_OPERATOR_V6,
    summarize_operator_v6,
    write_operator_summary_v6,
)


def main() -> None:
    rows = summarize_operator_v6(depth=8)
    write_operator_summary_v6(rows, OUTPUT_PATH_OPERATOR_V6)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V6}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
