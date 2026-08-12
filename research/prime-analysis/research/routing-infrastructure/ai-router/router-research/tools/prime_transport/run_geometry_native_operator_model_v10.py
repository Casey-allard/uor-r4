#!/usr/bin/env python3
"""Runner for the first lawful native radial transport operator."""

from __future__ import annotations

from geometry_native_operator_model_v10 import (
    OUTPUT_PATH_OPERATOR_V7,
    summarize_operator_v7,
    write_operator_summary_v7,
)


def main() -> None:
    rows = summarize_operator_v7(depth=8)
    write_operator_summary_v7(rows, OUTPUT_PATH_OPERATOR_V7)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V7}")
    for row in rows:
        if row["scope"] in {"operator", "orbit_change", "radial_escape", "class_escape", "spin_escape", "generator_axis"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
