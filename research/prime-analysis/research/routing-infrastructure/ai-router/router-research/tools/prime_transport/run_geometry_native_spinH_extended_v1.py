#!/usr/bin/env python3
"""Runner for the coherent extended transport-side state audit."""

from __future__ import annotations

from geometry_native_spinH_extended_v1 import (
    OUTPUT_PATH_SPINH_EXTENDED_V1,
    summarize_spinH_extended_v1,
    write_spinH_extended_v1,
)


def main() -> None:
    rows = summarize_spinH_extended_v1(depth=8)
    write_spinH_extended_v1(rows, OUTPUT_PATH_SPINH_EXTENDED_V1)
    print(f"wrote {OUTPUT_PATH_SPINH_EXTENDED_V1}")
    for row in rows:
        if row["scope"] in {"summary", "comparison_vs_v3", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
