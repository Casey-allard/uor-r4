#!/usr/bin/env python3
"""Runner for the refined primary-to-transport lift map."""

from __future__ import annotations

from geometry_native_lift_map_v1 import LIFT_HORIZON_V1
from geometry_native_lift_map_v2 import OUTPUT_PATH_LIFT_MAP_V2, summarize_lift_map_v2, write_lift_map_v2


def main() -> None:
    rows = summarize_lift_map_v2(horizon=LIFT_HORIZON_V1)
    write_lift_map_v2(rows, OUTPUT_PATH_LIFT_MAP_V2)
    print(f"wrote {OUTPUT_PATH_LIFT_MAP_V2}")
    for row in rows:
        if row["scope"] in {"summary", "comparison_vs_v1"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
