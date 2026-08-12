#!/usr/bin/env python3
"""Runner for the second explicit lawful torus-fiber evolution operator."""

from __future__ import annotations

from geometry_native_operator_model_v2 import OUTPUT_PATH_OPERATOR_V2, summarize_operator_v2, write_operator_summary_v2


def main() -> None:
    rows = summarize_operator_v2(depth=8)
    write_operator_summary_v2(rows, OUTPUT_PATH_OPERATOR_V2)
    print(f"wrote {OUTPUT_PATH_OPERATOR_V2}")
    for row in rows:
        print(
            f"{row['scope']}:{row['metric']} count={row['count']} "
            f"total={row['total']} fraction={float(row['fraction']):.4f}"
        )


if __name__ == "__main__":
    main()
