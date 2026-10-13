#!/usr/bin/env python3
"""Runner for the T_c rebuild from spin_H_core_v5."""

from __future__ import annotations

from geometry_native_operator_model_v19 import (
    OUTPUT_PATH_OPERATOR_FROM_CORE_V5,
    summarize_operator_from_spinH_core_v5,
    write_operator_from_spinH_core_v5,
)


def main() -> None:
    rows = summarize_operator_from_spinH_core_v5(depth=8)
    write_operator_from_spinH_core_v5(rows, OUTPUT_PATH_OPERATOR_FROM_CORE_V5)
    print(f"wrote {OUTPUT_PATH_OPERATOR_FROM_CORE_V5}")
    for row in rows:
        if row["scope"] in {"operator", "core_dependency", "comparison_vs_v16", "class_escape", "spin_escape", "radial_escape"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
