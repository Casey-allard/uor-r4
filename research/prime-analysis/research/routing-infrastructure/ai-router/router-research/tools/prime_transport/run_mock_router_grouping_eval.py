#!/usr/bin/env python3
"""Bounded comparison of coarser exact-layer grouping keys for R_min."""

from __future__ import annotations

import ast
import csv
from pathlib import Path

from mock_router_module import initialize_state, score_or_route, should_promote
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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_mock_router_grouping_eval.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def fiber_zero_count(phi: tuple[int, ...]) -> int:
    return sum(1 for value in phi if value == 0)


def candidate_key(name: str, *, b: int, phi: tuple[int, ...], r: int, next_return_gap: int, literal_key: str) -> str:
    if name == "literal_rmin":
        return literal_key
    if name == "base_gap":
        return f"base_gap:r={r}:b={b}:gap={next_return_gap}"
    if name == "phi_gap":
        return f"phi_gap:r={r}:phi={phi}:gap={next_return_gap}"
    if name == "base_zero_gap":
        return f"base_zero_gap:r={r}:b={b}:zero={fiber_zero_count(phi)}:gap={next_return_gap}"
    if name == "gap_only":
        return f"gap_only:r={r}:gap={next_return_gap}"
    raise ValueError(f"unknown candidate: {name}")


def main() -> None:
    candidates = (
        "literal_rmin",
        "base_gap",
        "phi_gap",
        "base_zero_gap",
        "gap_only",
    )
    rows_out: list[dict[str, object]] = []

    for trace_spec in TRACE_SPECS:
        for row in read_rows(trace_spec):
            offsets = tuple(ast.literal_eval(row["offsets"]))
            parent_W = int(row["parent_W"])
            visible_H = int(row["first_visible_split_H"])
            pre_H = visible_H - 1

            bits = admissible_bits(parent_W, offsets)
            length = len(bits)
            phases, layer_primes = phase_tuples(length)
            gaps = next_return_gaps(bits)
            spins = future_words(bits, pre_H)
            r_depth = len(layer_primes)

            literal_route_keys: list[str] = []
            full_route_keys: list[str] = []
            positions: list[tuple[int, tuple[int, ...], int, str]] = []

            for j in range(length):
                base = j % 35
                minimal_state = initialize_state(mode="R_min", b=base, phi=phases[j], r=r_depth, next_return_gap=gaps[j])
                full_state = initialize_state(mode="R_full", b=base, r=r_depth, spin_H=spins[j])
                literal_key = str(score_or_route(minimal_state)["route_key"])
                literal_route_keys.append(literal_key)
                full_route_keys.append(str(score_or_route(full_state)["route_key"]))
                positions.append((base, phases[j], gaps[j], literal_key))

            for candidate in candidates:
                route_keys = [
                    candidate_key(
                        candidate,
                        b=base,
                        phi=phi,
                        r=r_depth,
                        next_return_gap=gap,
                        literal_key=literal_key,
                    )
                    for base, phi, gap, literal_key in positions
                ]
                unresolved_by_route = class_unresolved_fraction(route_keys, full_route_keys)
                promotion_flags = [
                    int(
                        should_promote(
                            initialize_state(mode="R_min", b=base, phi=phi, r=r_depth, next_return_gap=gap),
                            unresolved_fraction=unresolved_by_route[route_key],
                        )
                    )
                    for (base, phi, gap, _), route_key in zip(positions, route_keys)
                ]

                unique_routes = len(set(route_keys))
                rows_out.append(
                    {
                        "trace_source": trace_spec.name,
                        "tuplet_name": row["tuplet_name"],
                        "offsets": row["offsets"],
                        "parent_W": parent_W,
                        "child_W": int(row["child_W"]),
                        "visible_H_first": visible_H,
                        "trace_length": length,
                        "r_depth": r_depth,
                        "candidate_key": candidate,
                        "unique_route_keys": unique_routes,
                        "route_reuse_fraction": 1.0 - (unique_routes / length),
                        "mean_route_class_size": length / unique_routes,
                        "max_unresolved_fraction_within_route": max(unresolved_by_route.values()),
                        "mean_unresolved_fraction_within_route": mean(list(unresolved_by_route.values())),
                        "promotion_fraction": mean([float(v) for v in promotion_flags]),
                        "sufficient_without_promotion_fraction": 1.0 - mean([float(v) for v in promotion_flags]),
                        "full_unique_route_keys": len(set(full_route_keys)),
                    }
                )

    overall_rows: list[dict[str, object]] = []
    for candidate in candidates:
        subset = [row for row in rows_out if row["candidate_key"] == candidate]
        overall_rows.append(
            {
                "trace_source": "ALL",
                "tuplet_name": "ALL",
                "offsets": "ALL",
                "parent_W": "",
                "child_W": "",
                "visible_H_first": "",
                "trace_length": sum(int(row["trace_length"]) for row in subset),
                "r_depth": "",
                "candidate_key": candidate,
                "unique_route_keys": mean([float(row["unique_route_keys"]) for row in subset]),
                "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in subset]),
                "mean_route_class_size": mean([float(row["mean_route_class_size"]) for row in subset]),
                "max_unresolved_fraction_within_route": max(float(row["max_unresolved_fraction_within_route"]) for row in subset),
                "mean_unresolved_fraction_within_route": mean([float(row["mean_unresolved_fraction_within_route"]) for row in subset]),
                "promotion_fraction": mean([float(row["promotion_fraction"]) for row in subset]),
                "sufficient_without_promotion_fraction": mean([float(row["sufficient_without_promotion_fraction"]) for row in subset]),
                "full_unique_route_keys": mean([float(row["full_unique_route_keys"]) for row in subset]),
            }
        )

    write_rows(OUT_CSV, rows_out + overall_rows)
    print(f"grouping_eval_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
