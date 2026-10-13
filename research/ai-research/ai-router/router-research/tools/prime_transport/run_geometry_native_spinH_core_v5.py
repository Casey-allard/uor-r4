#!/usr/bin/env python3
"""Runner for the sigma family holonomy v1 audit."""

from __future__ import annotations

from geometry_native_spinH_core_v5 import (
    OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1,
    summarize_spinH_core_v5,
    write_spinH_core_v5,
)


def main() -> None:
    rows = summarize_spinH_core_v5(depth=8)
    write_spinH_core_v5(rows, OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1)
    print(f"wrote {OUTPUT_PATH_SIGMA_FAMILY_HOLONOMY_V1}")
    for row in rows:
        if row["scope"] in {"summary", "composition", "comparison_vs_v4"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
