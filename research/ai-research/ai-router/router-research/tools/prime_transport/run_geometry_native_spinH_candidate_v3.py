#!/usr/bin/env python3
"""Runner for the augmented spin_H candidate with native tau."""

from __future__ import annotations

from geometry_native_spinH_candidate_v3 import (
    OUTPUT_PATH_SPINH_CANDIDATE_V3,
    summarize_spinH_candidate_v3,
    write_spinH_candidate_v3,
)


def main() -> None:
    rows = summarize_spinH_candidate_v3()
    write_spinH_candidate_v3(rows, OUTPUT_PATH_SPINH_CANDIDATE_V3)
    print(f"wrote {OUTPUT_PATH_SPINH_CANDIDATE_V3}")
    for row in rows:
        if row["scope"] in {"summary", "comparison", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )


if __name__ == "__main__":
    main()
