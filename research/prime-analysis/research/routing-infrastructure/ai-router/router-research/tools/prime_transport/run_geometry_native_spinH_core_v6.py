#!/usr/bin/env python3
"""Runner for the coupled-family holonomy residue audit (spin_H_core_v6)."""

from __future__ import annotations

from geometry_native_spinH_core_v6 import (
    OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1,
    summarize_spinH_core_v6,
    write_spinH_core_v6,
)


def main() -> None:
    rows = summarize_spinH_core_v6(depth=8)
    write_spinH_core_v6(rows, OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1)
    print(f"wrote {OUTPUT_PATH_COUPLED_FAMILY_HOLONOMY_V1}")
    for row in rows:
        if row["scope"] in {"summary", "composition", "comparison_vs_v5", "primitive"}:
            print(
                f"{row['scope']}:{row['metric']} "
                f"count={row['count']} total={row['total']} "
                f"fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
