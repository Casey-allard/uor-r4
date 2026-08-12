#!/usr/bin/env python3
"""Runner for the adaptive-horizon refined lift map."""

from __future__ import annotations

from geometry_native_lift_map_v3 import OUTPUT_PATH_LIFT_MAP_V3, summarize_lift_map_v3, write_lift_map_v3


def main() -> None:
    rows = summarize_lift_map_v3()
    write_lift_map_v3(rows, OUTPUT_PATH_LIFT_MAP_V3)
    print(f"wrote {OUTPUT_PATH_LIFT_MAP_V3}")
    for row in rows:
        if row["scope"] in {"summary", "comparison", "adaptive_horizon"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
