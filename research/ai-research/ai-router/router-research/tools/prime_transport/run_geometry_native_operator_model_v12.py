#!/usr/bin/env python3
"""Runner for the fuller bounded radial transport law."""

from __future__ import annotations

from geometry_native_operator_model_v12 import (
    OUTPUT_PATH_RADIAL_LAW_V1,
    summarize_radial_law_v1,
    write_radial_law_v1,
)


def main() -> None:
    rows = summarize_radial_law_v1(depth=8)
    write_radial_law_v1(rows, OUTPUT_PATH_RADIAL_LAW_V1)
    print(f"wrote {OUTPUT_PATH_RADIAL_LAW_V1}")
    for row in rows:
        if row["scope"] in {"operator", "radial_escape", "class_escape", "spin_escape", "tau_escape", "comparison_vs_old_radial", "generator_axis"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
