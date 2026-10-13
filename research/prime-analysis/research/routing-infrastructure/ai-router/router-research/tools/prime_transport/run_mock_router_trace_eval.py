#!/usr/bin/env python3
"""Trace-level offline evaluation of the prime-transport mock router module."""

from __future__ import annotations

import ast
import csv
from collections import Counter, defaultdict
from pathlib import Path

from mock_router_module import initialize_state, promote_state, score_or_route, should_promote, update_state


ROOT = Path(__file__).resolve().parents[2]
TRACE_SPECS = (
    ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_tight_density_matched_rows.csv",
    ROOT / "results" / "prime_transport_recursive_system" / "visible_threshold_second_tight_density_matched_rows.csv",
)
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_mock_router_trace_eval_summary.csv"


def read_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def odd_prime_factors(W: int) -> list[int]:
    remaining = W
    factors: list[int] = []
    for prime in (2, 3):
        while remaining % prime == 0:
            remaining //= prime
    divisor = 5
    while divisor * divisor <= remaining:
        if remaining % divisor == 0:
            factors.append(divisor)
            while remaining % divisor == 0:
                remaining //= divisor
        divisor += 2
    if remaining > 1:
        factors.append(remaining)
    return factors


def admissible_bits(W: int, offsets: tuple[int, ...]) -> list[int]:
    length = W // 6
    bits = [1] * length
    for prime in odd_prime_factors(W):
        inv6 = pow(6, -1, prime)
        forbidden = {int(((-5 - offset) * inv6) % prime) for offset in offsets}
        for j in range(length):
            if j % prime in forbidden:
                bits[j] = 0
    return bits


def factor_prime_layers(value: int) -> list[int]:
    remaining = int(value)
    factors: list[int] = []
    divisor = 2
    while divisor * divisor <= remaining:
        while remaining % divisor == 0:
            factors.append(divisor)
            remaining //= divisor
        divisor += 1 if divisor == 2 else 2
    if remaining > 1:
        factors.append(remaining)
    return factors


def phase_tuples(length: int) -> tuple[list[tuple[int, ...]], tuple[int, ...]]:
    fiber_mod = length // 35
    layer_primes = tuple(factor_prime_layers(fiber_mod))
    tuples: list[tuple[int, ...]] = []
    for j in range(length):
        t = j // 35
        tuples.append(tuple(int(t % prime) for prime in layer_primes))
    return tuples, layer_primes


def next_return_gaps(bits: list[int]) -> list[int]:
    length = len(bits)
    gaps: list[int] = []
    for j in range(length):
        gap = 0
        while gap < length and bits[(j + gap) % length] == 0:
            gap += 1
        gaps.append(gap)
    return gaps


def future_words(bits: list[int], horizon: int) -> list[tuple[int, ...]]:
    length = len(bits)
    out: list[tuple[int, ...]] = []
    for j in range(length):
        out.append(tuple(int(bits[(j + step) % length]) for step in range(horizon)))
    return out


def class_unresolved_fraction(route_keys: list[str], target_keys: list[str]) -> dict[str, float]:
    grouped: dict[str, Counter[str]] = defaultdict(Counter)
    totals: Counter[str] = Counter()
    for route_key, target_key in zip(route_keys, target_keys):
        grouped[route_key][target_key] += 1
        totals[route_key] += 1
    out: dict[str, float] = {}
    for route_key, counts in grouped.items():
        total = totals[route_key]
        out[route_key] = 1.0 - (max(counts.values()) / total)
    return out


def mean(values: list[float]) -> float:
    return sum(values) / len(values) if values else 0.0


def main() -> None:
    summary_rows: list[dict[str, object]] = []

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
            rstatic_matches = 0
            rmin_matches = 0
            rfull_matches = 0

            rmin_route_keys: list[str] = []
            rfull_route_keys: list[str] = []
            rmin_promotion_flags: list[int] = []

            # First pass: exact route keys at each position.
            for j in range(length):
                base = j % 35
                static_state = initialize_state(mode="R_static", b=base, phi=phases[j], r=r_depth)
                minimal_state = initialize_state(mode="R_min", b=base, phi=phases[j], r=r_depth, next_return_gap=gaps[j])
                full_state = initialize_state(mode="R_full", b=base, r=r_depth, spin_H=spins[j])
                rmin_route_keys.append(str(score_or_route(minimal_state)["route_key"]))
                rfull_route_keys.append(str(score_or_route(full_state)["route_key"]))

                next_j = (j + 1) % length
                expected_static = initialize_state(mode="R_static", b=next_j % 35, phi=phases[next_j], r=r_depth)
                expected_min = initialize_state(mode="R_min", b=next_j % 35, phi=phases[next_j], r=r_depth, next_return_gap=gaps[next_j])
                expected_full = initialize_state(mode="R_full", b=next_j % 35, r=r_depth, spin_H=spins[next_j])

                updated_static = update_state(static_state, fiber_moduli=layer_primes)
                updated_min = update_state(minimal_state, fiber_moduli=layer_primes, next_return_gap=gaps[next_j])
                updated_full = update_state(full_state, next_bit=bits[(j + pre_H) % length] if pre_H > 0 else bits[next_j])

                if updated_static == expected_static:
                    rstatic_matches += 1
                if updated_min == expected_min:
                    rmin_matches += 1
                if updated_full == expected_full:
                    rfull_matches += 1

            unresolved_by_route = class_unresolved_fraction(rmin_route_keys, rfull_route_keys)

            for j in range(length):
                base = j % 35
                minimal_state = initialize_state(mode="R_min", b=base, phi=phases[j], r=r_depth, next_return_gap=gaps[j])
                route_key = str(score_or_route(minimal_state)["route_key"])
                unresolved_fraction = unresolved_by_route[route_key]
                promote = should_promote(minimal_state, unresolved_fraction=unresolved_fraction)
                if promote:
                    _ = promote_state(minimal_state, spin_H=spins[j])
                rmin_promotion_flags.append(int(promote))

            summary_rows.append(
                {
                    "trace_source": trace_spec.name,
                    "tuplet_name": row["tuplet_name"],
                    "offsets": row["offsets"],
                    "parent_W": parent_W,
                    "child_W": child_W,
                    "visible_H_first": visible_H,
                    "trace_length": length,
                    "r_depth": r_depth,
                    "rstatic_transition_match_fraction": rstatic_matches / length,
                    "rmin_transition_match_fraction": rmin_matches / length,
                    "rfull_transition_match_fraction": rfull_matches / length,
                    "rmin_unique_route_keys": len(set(rmin_route_keys)),
                    "rfull_unique_route_keys": len(set(rfull_route_keys)),
                    "rmin_promotion_fraction": mean([float(v) for v in rmin_promotion_flags]),
                    "rmin_sufficient_without_promotion_fraction": 1.0 - mean([float(v) for v in rmin_promotion_flags]),
                    "max_unresolved_fraction_within_rmin_route": max(unresolved_by_route.values()),
                    "mean_unresolved_fraction_within_rmin_route": mean(list(unresolved_by_route.values())),
                }
            )

    overall = {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in summary_rows),
        "r_depth": "",
        "rstatic_transition_match_fraction": mean([float(row["rstatic_transition_match_fraction"]) for row in summary_rows]),
        "rmin_transition_match_fraction": mean([float(row["rmin_transition_match_fraction"]) for row in summary_rows]),
        "rfull_transition_match_fraction": mean([float(row["rfull_transition_match_fraction"]) for row in summary_rows]),
        "rmin_unique_route_keys": mean([float(row["rmin_unique_route_keys"]) for row in summary_rows]),
        "rfull_unique_route_keys": mean([float(row["rfull_unique_route_keys"]) for row in summary_rows]),
        "rmin_promotion_fraction": mean([float(row["rmin_promotion_fraction"]) for row in summary_rows]),
        "rmin_sufficient_without_promotion_fraction": mean([float(row["rmin_sufficient_without_promotion_fraction"]) for row in summary_rows]),
        "max_unresolved_fraction_within_rmin_route": max(float(row["max_unresolved_fraction_within_rmin_route"]) for row in summary_rows),
        "mean_unresolved_fraction_within_rmin_route": mean([float(row["mean_unresolved_fraction_within_rmin_route"]) for row in summary_rows]),
    }
    summary_rows.append(overall)

    write_rows(OUT_CSV, summary_rows)
    print(f"summary_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
