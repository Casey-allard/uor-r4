#!/usr/bin/env python3
"""Runner for the radial rebuild from spin_H_core_v3."""

from __future__ import annotations

from geometry_native_operator_model_v15 import (
    OUTPUT_PATH_OPERATOR_FROM_CORE_V3_RADIAL,
    summarize_operator_from_spinH_core_v3_radial,
    write_operator_from_spinH_core_v3_radial,
)


def main() -> None:
    rows = summarize_operator_from_spinH_core_v3_radial(depth=8)
    write_operator_from_spinH_core_v3_radial(rows, OUTPUT_PATH_OPERATOR_FROM_CORE_V3_RADIAL)
    print(f"wrote {OUTPUT_PATH_OPERATOR_FROM_CORE_V3_RADIAL}")
    for row in rows:
        if row["scope"] in {"operator", "core_dependency", "comparison_vs_v14", "class_escape", "spin_escape", "radial_escape"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
