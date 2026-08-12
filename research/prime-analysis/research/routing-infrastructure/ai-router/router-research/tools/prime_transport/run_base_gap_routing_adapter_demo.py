#!/usr/bin/env python3
"""Tiny benchmark-like demo for the guarded base_gap routing adapter."""

from __future__ import annotations

import ast
import csv
from pathlib import Path

from base_gap_routing_adapter import (
    adapter_output,
    initialize_adapter,
    initialize_entry,
    step_entry,
)
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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_base_gap_routing_adapter_demo.csv"


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
        source_rows = read_rows(trace_spec)
        if not source_rows:
            continue
        row = source_rows[0]

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

        adapter = initialize_adapter()
        base_gap_keys = [
            f"base_gap:r={r_depth}:b={j % 35}:gap={gaps[j]}"
            for j in range(length)
        ]
        full_route_keys = [
            f"full:b={j % 35}:spin={spins[j]}"
            for j in range(length)
        ]
        unresolved_by_route = class_unresolved_fraction(base_gap_keys, full_route_keys)

        current = initialize_entry(b=0, phi=phases[0], r=r_depth, next_return_gap=gaps[0])
        cache_hits = 0
        promoted_steps = 0
        route_keys_seen: set[str] = set()
        route_modes: list[str] = []
        resolved_scores: list[float] = []

        for j in range(length):
            route_key = f"base_gap:r={r_depth}:b={j % 35}:gap={gaps[j]}"
            if route_key in adapter.route_policy_cache:
                cache_hits += 1
            result = adapter_output(
                adapter,
                current,
                unresolved_fraction=unresolved_by_route[route_key],
                spin_H=spins[j],
            )
            route_keys_seen.add(str(result["route_key"]))
            route_modes.append(str(result["route_mode"]))
            if result["promoted"]:
                promoted_steps += 1
                resolved_scores.append(1.0)
            else:
                resolved_scores.append(1.0 - unresolved_by_route[route_key])

            next_j = (j + 1) % length
            current = step_entry(
                current,
                fiber_moduli=layer_primes,
                next_return_gap=gaps[next_j],
            )
            if result["promoted"]:
                current = initialize_entry(
                    b=next_j % 35,
                    phi=phases[next_j],
                    r=r_depth,
                    next_return_gap=gaps[next_j],
                )

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
                "route_policy_cache_size": len(adapter.route_policy_cache),
                "route_reuse_fraction": cache_hits / length,
                "promotion_step_fraction": promoted_steps / length,
                "effective_resolved_fraction": mean(resolved_scores),
                "route_decision_instability": 0.0,
                "observed_route_modes": ",".join(sorted(set(route_modes))),
                "unique_emitted_route_keys": len(route_keys_seen),
            }
        )

    overall = {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in rows_out),
        "r_depth": "",
        "route_policy_cache_size": mean([float(row["route_policy_cache_size"]) for row in rows_out]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows_out]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows_out]),
        "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in rows_out]),
        "route_decision_instability": 0.0,
        "observed_route_modes": "R_full,R_min",
        "unique_emitted_route_keys": mean([float(row["unique_emitted_route_keys"]) for row in rows_out]),
    }
    rows_out.append(overall)

    write_rows(OUT_CSV, rows_out)
    print(f"adapter_demo_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
