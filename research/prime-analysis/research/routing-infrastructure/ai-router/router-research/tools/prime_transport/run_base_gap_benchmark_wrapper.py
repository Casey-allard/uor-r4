#!/usr/bin/env python3
"""Tiny driver for the research-only base_gap benchmark wrapper."""

from __future__ import annotations

import csv
from pathlib import Path

from base_gap_benchmark_wrapper import ROOT, build_bounded_traces, run_trace, summarize_rows


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_base_gap_benchmark_wrapper.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    traces = build_bounded_traces()
    rows = [run_trace(trace) for trace in traces]
    rows.append(summarize_rows(rows))
    write_rows(OUT_CSV, rows)
    print(f"benchmark_wrapper_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
