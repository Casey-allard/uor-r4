#!/usr/bin/env python3
"""Guarded research-side head-to-head: prime transport vs angular Hopf."""

from __future__ import annotations

import csv
from pathlib import Path

from angular_hopf_benchmark_adapter import angular_route_key_for_trace_entry
from base_gap_benchmark_wrapper import ROOT, build_bounded_traces
from run_guarded_benchmark_harness_eval import (
    evaluate_base_gap_policy,
    evaluate_gap_only_policy,
    full_route_keys_for_trace,
)
from run_mock_router_trace_eval import class_unresolved_fraction, mean


OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_vs_angular_hopf_head_to_head.csv"
THRESHOLD_SUMMARY = ROOT / "results" / "prime_transport_recursive_system" / "threshold_law_summary.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def _add_fallback_metric(row: dict[str, object]) -> dict[str, object]:
    out = dict(row)
    out["fallback_step_fraction"] = float(row["promotion_step_fraction"])
    return out


def evaluate_angular_hopf_policy(trace) -> dict[str, object]:
    route_keys: list[str] = []
    cache_hits = 0
    seen_keys: set[str] = set()

    for j in range(trace.trace_length):
        route_key = angular_route_key_for_trace_entry(
            r_depth=trace.r_depth,
            base_phase=j % 35,
            phase_tuple=trace.phases[j],
            layer_primes=trace.layer_primes,
            next_return_gap=trace.gaps[j],
        )
        if route_key in seen_keys:
            cache_hits += 1
        seen_keys.add(route_key)
        route_keys.append(route_key)

    full_keys = full_route_keys_for_trace(trace)
    unresolved_by_route = class_unresolved_fraction(route_keys, full_keys)

    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "r_depth": trace.r_depth,
        "policy_name": "angular_hopf",
        "unique_route_keys": len(seen_keys),
        "unique_emitted_route_keys": len(seen_keys),
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": 0.0,
        "promotion_step_fraction": 0.0,
        "fallback_step_fraction": 0.0,
        "effective_resolved_fraction": 1.0 - mean(list(unresolved_by_route.values())),
        "mean_unresolved_among_nonpromoted_routes": mean(list(unresolved_by_route.values())),
        "route_decision_instability": 0.0,
    }


def summarize_policy(rows: list[dict[str, object]], policy_name: str) -> dict[str, object]:
    subset = [row for row in rows if row["policy_name"] == policy_name]
    return {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in subset),
        "r_depth": "",
        "policy_name": policy_name,
        "unique_route_keys": mean([float(row["unique_route_keys"]) for row in subset]),
        "unique_emitted_route_keys": mean([float(row["unique_emitted_route_keys"]) for row in subset]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in subset]),
        "promotion_route_fraction": mean([float(row["promotion_route_fraction"]) for row in subset]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in subset]),
        "fallback_step_fraction": mean([float(row["fallback_step_fraction"]) for row in subset]),
        "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in subset]),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [float(row["mean_unresolved_among_nonpromoted_routes"]) for row in subset]
        ),
        "route_decision_instability": 0.0,
    }


def main() -> None:
    traces = build_bounded_traces(source_paths=(THRESHOLD_SUMMARY,))
    rows: list[dict[str, object]] = []
    for trace in traces:
        rows.append(evaluate_angular_hopf_policy(trace))
        rows.append(_add_fallback_metric(evaluate_gap_only_policy(trace)))
        rows.append(_add_fallback_metric(evaluate_base_gap_policy(trace)))

    rows.append(summarize_policy(rows, "angular_hopf"))
    rows.append(summarize_policy(rows, "gap_only"))
    rows.append(summarize_policy(rows, "base_gap"))
    write_rows(OUT_CSV, rows)
    print(f"prime_transport_vs_angular_hopf_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
