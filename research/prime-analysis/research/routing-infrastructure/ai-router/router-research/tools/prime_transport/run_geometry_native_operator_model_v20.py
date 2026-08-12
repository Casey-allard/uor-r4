#!/usr/bin/env python3
"""Runner for the T_c rebuild from spin_H_core_v6."""

from __future__ import annotations

from geometry_native_operator_model_v20 import (
    OUTPUT_PATH_TC_REBUILD_V1,
    summarize_operator_from_spinH_core_v6,
    write_Tc_rebuild_v1,
)


def main() -> None:
    rows = summarize_operator_from_spinH_core_v6(depth=8)
    write_Tc_rebuild_v1(rows, OUTPUT_PATH_TC_REBUILD_V1)
    print(f"wrote {OUTPUT_PATH_TC_REBUILD_V1}")
    for row in rows:
        print(f"{row['metric']}={row['value']}  ({row['note']})")


if __name__ == "__main__":
    main()
