#!/usr/bin/env python3
"""Larger research-side benchmark using the existing guarded wrapper boundary."""

from __future__ import annotations

import csv
from pathlib import Path

from base_gap_benchmark_wrapper import ROOT, build_bounded_traces
from run_guarded_benchmark_harness_eval import (
    evaluate_base_gap_policy,
    evaluate_gap_only_policy,
    evaluate_partition_control,
    static_route_keys_for_trace,
    summarize_policy,
)


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_larger_research_side_benchmark.csv"
THRESHOLD_SUMMARY = ROOT / "results" / "prime_transport_recursive_system" / "threshold_law_summary.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    traces = build_bounded_traces(source_paths=(THRESHOLD_SUMMARY,))
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(
            evaluate_partition_control(
                trace,
                policy_name="static_only",
                route_keys=static_route_keys_for_trace(trace),
            )
        )
        rows.append(evaluate_gap_only_policy(trace))
        rows.append(evaluate_base_gap_policy(trace))

    rows.append(summarize_policy(rows, "static_only"))
    rows.append(summarize_policy(rows, "gap_only"))
    rows.append(summarize_policy(rows, "base_gap"))
    write_rows(OUT_CSV, rows)
    print(f"larger_benchmark_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
