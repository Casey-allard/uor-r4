#!/usr/bin/env python3
"""Runner for the canonical spin_H core state audit."""

from __future__ import annotations

from geometry_native_spinH_core_v1 import (
    OUTPUT_PATH_SPINH_CORE_V1,
    summarize_spinH_core_v1,
    write_spinH_core_v1,
)


def main() -> None:
    rows = summarize_spinH_core_v1(depth=8)
    write_spinH_core_v1(rows, OUTPUT_PATH_SPINH_CORE_V1)
    print(f"wrote {OUTPUT_PATH_SPINH_CORE_V1}")
    for row in rows:
        if row["scope"] in {"summary", "projection", "representation", "comparison_vs_extended_v2"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
