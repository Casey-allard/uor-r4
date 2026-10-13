#!/usr/bin/env python3
"""Runner for the canonical spin_H core v4 audit."""

from __future__ import annotations

from geometry_native_spinH_core_v4 import (
    OUTPUT_PATH_SPINH_CORE_V4,
    summarize_spinH_core_v4,
    write_spinH_core_v4,
)


def main() -> None:
    rows = summarize_spinH_core_v4(depth=8)
    write_spinH_core_v4(rows, OUTPUT_PATH_SPINH_CORE_V4)
    print(f"wrote {OUTPUT_PATH_SPINH_CORE_V4}")
    for row in rows:
        if row["scope"] in {"summary", "projection", "comparison_vs_v3"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
