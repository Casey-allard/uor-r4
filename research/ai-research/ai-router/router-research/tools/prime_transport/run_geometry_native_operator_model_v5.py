#!/usr/bin/env python3
"""Runner for the fifth explicit lawful torus-fiber evolution operator."""

from __future__ import annotations

from geometry_native_operator_model_v5 import OUTPUT_PATH_OPERATOR_V5, summarize_operator_v5, write_operator_summary_v5


def main() -> None:
    rows = summarize_operator_v5(depth=8)
    write_operator_summary_v5(rows, OUTPUT_PATH_OPERATOR_V5)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V5}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
