#!/usr/bin/env python3
"""Runner for framework enforcement step 3."""

from __future__ import annotations

from geometry_native_sequence_model_lock1 import TaskConfigLock1
from geometry_native_sequence_model_lock3 import (
    OUTPUT_PATH_LOCK3,
    run_framework_enforcement_step3,
    write_framework_enforcement_step3,
)


def main() -> None:
    rows = run_framework_enforcement_step3(TaskConfigLock1(), seed=241)
    write_framework_enforcement_step3(rows, OUTPUT_PATH_LOCK3)
    print(f"wrote {OUTPUT_PATH_LOCK3}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
