#!/usr/bin/env python3
"""Runner for the recursive-stability audit of spin_H_candidate."""

from __future__ import annotations

from geometry_native_spinH_candidate_v2 import (
    OUTPUT_PATH_SPINH_CANDIDATE_V2,
    summarize_spinH_candidate_v2,
    write_spinH_candidate_v2,
)


def main() -> None:
    rows = summarize_spinH_candidate_v2()
    write_spinH_candidate_v2(rows, OUTPUT_PATH_SPINH_CANDIDATE_V2)
    print(f"wrote {OUTPUT_PATH_SPINH_CANDIDATE_V2}")
    for row in rows:
        if row["scope"] in {"summary", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
