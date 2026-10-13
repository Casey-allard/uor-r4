#!/usr/bin/env python3
"""Runner for the explicit primary-to-transport lift map."""

from __future__ import annotations

from geometry_native_lift_map_v1 import LIFT_HORIZON_V1, OUTPUT_PATH_LIFT_MAP_V1, summarize_lift_map_v1, write_lift_map_v1


def main() -> None:
    rows = summarize_lift_map_v1(horizon=LIFT_HORIZON_V1)
    write_lift_map_v1(rows, OUTPUT_PATH_LIFT_MAP_V1)
    print(f"wrote {OUTPUT_PATH_LIFT_MAP_V1}")
    for row in rows:
        if row["scope"] == "summary":
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
