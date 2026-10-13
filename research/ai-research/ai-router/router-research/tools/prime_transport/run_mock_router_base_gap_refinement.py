#!/usr/bin/env python3
"""Bounded evaluation of one minimal refinement on top of base_gap routing."""

from __future__ import annotations

import ast
import csv
from pathlib import Path

from mock_router_module import initialize_state, promote_state, score_or_route, should_promote
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
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_mock_router_base_gap_refinement.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def phi_head(phi: tuple[int, ...]) -> int | None:
    return int(phi[0]) if phi else None


def route_key_for_mode(
    mode: str,
    *,
    b: int,
    phi: tuple[int, ...],
    r: int,
    next_return_gap: int,
    literal_key: str,
    full_key: str,
) -> str:
    if mode == "literal_rmin":
        return literal_key
    if mode == "base_gap":
        return candidate_key("base_gap", b=b, phi=phi, r=r, next_return_gap=next_return_gap, literal_key=literal_key)
    if mode == "base_gap_phi0":
        return f"base_gap_phi0:r={r}:b={b}:gap={next_return_gap}:phi0={phi_head(phi)}"
    if mode == "gap_only":
        return candidate_key("gap_only", b=b, phi=phi, r=r, next_return_gap=next_return_gap, literal_key=literal_key)
    if mode == "full_spin_baseline":
        return full_key
    raise ValueError(f"unknown mode: {mode}")


def evaluate_mode(
    mode: str,
    *,
    positions: list[tuple[int, tuple[int, ...], int, str]],
    full_route_keys: list[str],
    spins: list[tuple[int, ...]],
    r_depth: int,
) -> dict[str, float]:
    route_keys = [
        route_key_for_mode(
            mode,
            b=base,
            phi=phi,
            r=r_depth,
            next_return_gap=gap,
            literal_key=literal_key,
            full_key=full_key,
        )
        for (base, phi, gap, literal_key), full_key in zip(positions, full_route_keys)
    ]

    if mode == "full_spin_baseline":
        unresolved_by_route = {key: 0.0 for key in route_keys}
    else:
        unresolved_by_route = class_unresolved_fraction(route_keys, full_route_keys)

    promotion_flags: list[int] = []
    final_route_keys: list[str] = []
    promoted_phi0_fraction = 0.0

    if mode != "full_spin_baseline":
        promoted_positions = [
            (base, phi, gap, literal_key, route_key, full_key)
            for (base, phi, gap, literal_key), route_key, full_key in zip(positions, route_keys, full_route_keys)
            if should_promote(
                initialize_state(mode="R_min", b=base, phi=phi, r=r_depth, next_return_gap=gap),
                unresolved_fraction=unresolved_by_route[route_key],
            )
        ]
        if promoted_positions:
            promoted_phi0_fraction = len({phi_head(phi) for _, phi, _, _, _, _ in promoted_positions}) / len(promoted_positions)

    for (base, phi, gap, literal_key), route_key, spin in zip(positions, route_keys, spins):
        if mode == "full_spin_baseline":
            promote = False
        else:
            state = initialize_state(mode="R_min", b=base, phi=phi, r=r_depth, next_return_gap=gap)
            promote = should_promote(state, unresolved_fraction=unresolved_by_route[route_key])
        promotion_flags.append(int(promote))
        if promote:
            promoted = promote_state(
                initialize_state(mode="R_min", b=base, phi=phi, r=r_depth, next_return_gap=gap),
                spin_H=spin,
            )
            final_route_keys.append(str(score_or_route(promoted)["route_key"]))
        else:
            final_route_keys.append(route_key)

    final_unresolved = class_unresolved_fraction(final_route_keys, full_route_keys)
    unique_route_keys = len(set(route_keys))
    final_unique_route_keys = len(set(final_route_keys))
    unpromoted_unresolved = [
        unresolved_by_route[route_key]
        for route_key, promoted in zip(route_keys, promotion_flags)
        if not promoted
    ]

    return {
        "unique_route_keys": float(unique_route_keys),
        "route_reuse_fraction": 1.0 - (unique_route_keys / len(route_keys)),
        "mean_route_class_size": len(route_keys) / unique_route_keys,
        "pre_promotion_mean_unresolved_fraction": mean(list(unresolved_by_route.values())),
        "pre_promotion_max_unresolved_fraction": max(unresolved_by_route.values()),
        "promotion_fraction": mean([float(v) for v in promotion_flags]),
        "post_promotion_unique_route_keys": float(final_unique_route_keys),
        "post_promotion_route_reuse_fraction": 1.0 - (final_unique_route_keys / len(final_route_keys)),
        "post_promotion_mean_unresolved_fraction": mean(list(final_unresolved.values())),
        "post_promotion_max_unresolved_fraction": max(final_unresolved.values()),
        "post_promotion_resolution_fraction": 1.0 - mean(list(final_unresolved.values())),
        "max_unresolved_among_unpromoted": max(unpromoted_unresolved) if unpromoted_unresolved else 0.0,
        "promoted_phi0_fraction": promoted_phi0_fraction,
    }


def main() -> None:
    modes = ("literal_rmin", "base_gap", "base_gap_phi0", "gap_only", "full_spin_baseline")
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

            positions: list[tuple[int, tuple[int, ...], int, str]] = []
            full_route_keys: list[str] = []
            for j in range(length):
                base = j % 35
                minimal_state = initialize_state(mode="R_min", b=base, phi=phases[j], r=r_depth, next_return_gap=gaps[j])
                full_state = initialize_state(mode="R_full", b=base, r=r_depth, spin_H=spins[j])
                positions.append((base, phases[j], gaps[j], str(score_or_route(minimal_state)["route_key"])))
                full_route_keys.append(str(score_or_route(full_state)["route_key"]))

            for mode in modes:
                metrics = evaluate_mode(
                    mode,
                    positions=positions,
                    full_route_keys=full_route_keys,
                    spins=spins,
                    r_depth=r_depth,
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
                        "prototype_mode": mode,
                        **metrics,
                    }
                )

    overall_rows: list[dict[str, object]] = []
    for mode in modes:
        subset = [row for row in rows_out if row["prototype_mode"] == mode]
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
                "prototype_mode": mode,
                "unique_route_keys": mean([float(row["unique_route_keys"]) for row in subset]),
                "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in subset]),
                "mean_route_class_size": mean([float(row["mean_route_class_size"]) for row in subset]),
                "pre_promotion_mean_unresolved_fraction": mean([float(row["pre_promotion_mean_unresolved_fraction"]) for row in subset]),
                "pre_promotion_max_unresolved_fraction": max(float(row["pre_promotion_max_unresolved_fraction"]) for row in subset),
                "promotion_fraction": mean([float(row["promotion_fraction"]) for row in subset]),
                "post_promotion_unique_route_keys": mean([float(row["post_promotion_unique_route_keys"]) for row in subset]),
                "post_promotion_route_reuse_fraction": mean([float(row["post_promotion_route_reuse_fraction"]) for row in subset]),
                "post_promotion_mean_unresolved_fraction": mean([float(row["post_promotion_mean_unresolved_fraction"]) for row in subset]),
                "post_promotion_max_unresolved_fraction": max(float(row["post_promotion_max_unresolved_fraction"]) for row in subset),
                "post_promotion_resolution_fraction": mean([float(row["post_promotion_resolution_fraction"]) for row in subset]),
                "max_unresolved_among_unpromoted": max(float(row["max_unresolved_among_unpromoted"]) for row in subset),
                "promoted_phi0_fraction": mean([float(row["promoted_phi0_fraction"]) for row in subset]),
            }
        )

    write_rows(OUT_CSV, rows_out + overall_rows)
    print(f"base_gap_refinement_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
