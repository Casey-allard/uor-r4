#!/usr/bin/env python3
"""Offline routing-loop evaluation for the base_gap prime-transport prototype."""

from __future__ import annotations

import ast
import csv
from collections import Counter
from pathlib import Path

from mock_router_module import initialize_state, should_promote
from run_mock_router_grouping_eval import candidate_key
from run_mock_router_trace_eval import (
    TRACE_SPECS,
    admissible_bits,
    class_unresolved_fraction,
    future_words,
    mean,
    next_return_gaps,
    phase_tuples,
    read_rows,
)


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_base_gap_routing_loop_eval.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    rows_out: list[dict[str, object]] = []

    for trace_spec in TRACE_SPECS:
        for row in read_rows(trace_spec):
            offsets = tuple(ast.literal_eval(row["offsets"]))
            parent_W = int(row["parent_W"])
            child_W = int(row["child_W"])
            visible_H = int(row["first_visible_split_H"])
            pre_H = visible_H - 1

            bits = admissible_bits(parent_W, offsets)
            length = len(bits)
            phases, layer_primes = phase_tuples(length)
            gaps = next_return_gaps(bits)
            spins = future_words(bits, pre_H)
            r_depth = len(layer_primes)

            base_gap_keys: list[str] = []
            full_route_keys: list[str] = []
            position_rows: list[tuple[int, tuple[int, ...], int, str]] = []
            for j in range(length):
                b = j % 35
                phi = phases[j]
                gap = gaps[j]
                literal_key = f"min:b={b}:phi={phi}:r={r_depth}:gap={gap}"
                base_gap_key = candidate_key(
                    "base_gap",
                    b=b,
                    phi=phi,
                    r=r_depth,
                    next_return_gap=gap,
                    literal_key=literal_key,
                )
                base_gap_keys.append(base_gap_key)
                full_route_keys.append(f"full:b={b}:spin={spins[j]}")
                position_rows.append((b, phi, gap, base_gap_key))

            unresolved_by_route = class_unresolved_fraction(base_gap_keys, full_route_keys)

            route_table: dict[str, bool] = {}
            route_counts: Counter[str] = Counter()
            promoted_route_counts: Counter[str] = Counter()
            cache_hits = 0
            fallback_steps = 0
            effective_resolution: list[float] = []

            for (b, phi, gap, route_key), spin_key in zip(position_rows, full_route_keys):
                if route_key in route_table:
                    cache_hits += 1
                else:
                    unresolved_fraction = unresolved_by_route[route_key]
                    route_table[route_key] = should_promote(
                        initialize_state(mode="R_min", b=b, phi=phi, r=r_depth, next_return_gap=gap),
                        unresolved_fraction=unresolved_fraction,
                    )
                route_counts[route_key] += 1
                if route_table[route_key]:
                    fallback_steps += 1
                    promoted_route_counts[route_key] += 1
                    effective_resolution.append(1.0)
                else:
                    effective_resolution.append(1.0 - unresolved_by_route[route_key])

            promoted_routes = [route for route, promoted in route_table.items() if promoted]
            nonpromoted_routes = [route for route, promoted in route_table.items() if not promoted]

            rows_out.append(
                {
                    "trace_source": trace_spec.name,
                    "tuplet_name": row["tuplet_name"],
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "visible_H_first": visible_H,
                    "trace_length": length,
                    "r_depth": r_depth,
                    "unique_base_gap_routes": len(route_table),
                    "route_reuse_fraction": cache_hits / length,
                    "promotion_route_fraction": len(promoted_routes) / len(route_table) if route_table else 0.0,
                    "promotion_step_fraction": fallback_steps / length,
                    "fallback_burden_steps": fallback_steps,
                    "mean_promoted_route_class_size": mean([float(route_counts[key]) for key in promoted_routes]),
                    "mean_nonpromoted_route_class_size": mean([float(route_counts[key]) for key in nonpromoted_routes]),
                    "effective_resolved_fraction": mean(effective_resolution),
                    "mean_unresolved_among_nonpromoted_routes": mean(
                        [unresolved_by_route[key] for key in nonpromoted_routes]
                    ),
                    "max_unresolved_among_nonpromoted_routes": max(
                        [unresolved_by_route[key] for key in nonpromoted_routes],
                        default=0.0,
                    ),
                    "route_decision_instability": 0.0,
                }
            )

    overall_subset = rows_out[:]
    rows_out.append(
        {
            "trace_source": "ALL",
            "tuplet_name": "ALL",
            "offsets": "ALL",
            "parent_W": "",
            "child_W": "",
            "visible_H_first": "",
            "trace_length": sum(int(row["trace_length"]) for row in overall_subset),
            "r_depth": "",
            "unique_base_gap_routes": mean([float(row["unique_base_gap_routes"]) for row in overall_subset]),
            "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in overall_subset]),
            "promotion_route_fraction": mean([float(row["promotion_route_fraction"]) for row in overall_subset]),
            "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in overall_subset]),
            "fallback_burden_steps": mean([float(row["fallback_burden_steps"]) for row in overall_subset]),
            "mean_promoted_route_class_size": mean([float(row["mean_promoted_route_class_size"]) for row in overall_subset]),
            "mean_nonpromoted_route_class_size": mean([float(row["mean_nonpromoted_route_class_size"]) for row in overall_subset]),
            "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in overall_subset]),
            "mean_unresolved_among_nonpromoted_routes": mean(
                [float(row["mean_unresolved_among_nonpromoted_routes"]) for row in overall_subset]
            ),
            "max_unresolved_among_nonpromoted_routes": max(
                float(row["max_unresolved_among_nonpromoted_routes"]) for row in overall_subset
            ),
            "route_decision_instability": 0.0,
        }
    )

    write_rows(OUT_CSV, rows_out)
    print(f"routing_loop_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
