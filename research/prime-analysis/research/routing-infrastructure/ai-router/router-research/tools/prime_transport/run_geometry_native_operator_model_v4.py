#!/usr/bin/env python3
"""Runner for the fourth explicit lawful torus-fiber evolution operator."""

from __future__ import annotations

from geometry_native_operator_model_v4 import OUTPUT_PATH_OPERATOR_V4, summarize_operator_v4, write_operator_summary_v4


def main() -> None:
    rows = summarize_operator_v4(depth=8)
    write_operator_summary_v4(rows, OUTPUT_PATH_OPERATOR_V4)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V4}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
