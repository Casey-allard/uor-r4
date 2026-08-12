#!/usr/bin/env python3
"""Runner for framework enforcement step 1."""

from __future__ import annotations

from geometry_native_sequence_model_lock1 import (
    OUTPUT_PATH_LOCK1,
    TaskConfigLock1,
    run_framework_enforcement_step1,
    write_framework_enforcement_step1,
)


def main() -> None:
    rows = run_framework_enforcement_step1(TaskConfigLock1(), seed=241)
    write_framework_enforcement_step1(rows, OUTPUT_PATH_LOCK1)
    print(f"wrote {OUTPUT_PATH_LOCK1}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
