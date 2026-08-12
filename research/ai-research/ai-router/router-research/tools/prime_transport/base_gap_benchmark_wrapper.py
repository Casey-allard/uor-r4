#!/usr/bin/env python3
"""Research-only benchmark wrapper for the guarded base_gap routing adapter."""

from __future__ import annotations

import ast
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from base_gap_routing_adapter import adapter_output, initialize_adapter, initialize_entry, step_entry
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


@dataclass(frozen=True)
class BoundedTrace:
    trace_source: str
    tuplet_name: str
    offsets: str
    parent_W: int
    child_W: int
    visible_H_first: int
    trace_length: int
    r_depth: int
    layer_primes: tuple[int, ...]
    phases: list[tuple[int, ...]]
    gaps: list[int]
    spins: list[tuple[int, ...]]
    unresolved_by_route: dict[str, float]


def _row_has_exact_visible(row: dict[str, str]) -> bool:
    if "visible_found_exact_within_range" not in row:
        return True
    return row["visible_found_exact_within_range"] == "1"


def build_bounded_traces(
    limit_per_source: int | None = None,
    source_paths: Iterable[Path] | None = None,
) -> list[BoundedTrace]:
    traces: list[BoundedTrace] = []
    active_sources = tuple(source_paths) if source_paths is not None else TRACE_SPECS
    for trace_spec in active_sources:
        rows = read_rows(trace_spec)
        rows = [row for row in rows if _row_has_exact_visible(row)]
        if limit_per_source is not None:
            rows = rows[:limit_per_source]
        for row in rows:
            offsets = tuple(ast.literal_eval(row["offsets"]))
            parent_W = int(row["parent_W"])
            child_W = int(row["child_W"])
            visible_H = int(row["first_visible_split_H"])
            pre_H = visible_H - 1

            bits = admissible_bits(parent_W, offsets)
            phases, layer_primes = phase_tuples(len(bits))
            gaps = next_return_gaps(bits)
            spins = future_words(bits, pre_H)
            r_depth = len(layer_primes)

            route_keys = [
                f"base_gap:r={r_depth}:b={j % 35}:gap={gaps[j]}"
                for j in range(len(bits))
            ]
            full_keys = [
                f"full:b={j % 35}:spin={spins[j]}"
                for j in range(len(bits))
            ]
            unresolved = class_unresolved_fraction(route_keys, full_keys)

            traces.append(
                BoundedTrace(
                    trace_source=trace_spec.name,
                    tuplet_name=row["tuplet_name"],
                    offsets=row["offsets"],
                    parent_W=parent_W,
                    child_W=child_W,
                    visible_H_first=visible_H,
                    trace_length=len(bits),
                    r_depth=r_depth,
                    layer_primes=layer_primes,
                    phases=phases,
                    gaps=gaps,
                    spins=spins,
                    unresolved_by_route=unresolved,
                )
            )
    return traces


def run_trace(trace: BoundedTrace) -> dict[str, object]:
    adapter = initialize_adapter()
    state = initialize_entry(b=0, phi=trace.phases[0], r=trace.r_depth, next_return_gap=trace.gaps[0])

    cache_hits = 0
    promoted_steps = 0
    emitted_route_keys: set[str] = set()
    route_modes: set[str] = set()
    resolved_scores: list[float] = []
    route_decisions: dict[str, bool] = {}

    for j in range(trace.trace_length):
        route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
        if route_key in adapter.route_policy_cache:
            cache_hits += 1

        result = adapter_output(
            adapter,
            state,
            unresolved_fraction=trace.unresolved_by_route[route_key],
            spin_H=trace.spins[j],
        )
        emitted_route_keys.add(str(result["route_key"]))
        route_modes.add(str(result["route_mode"]))

        if route_key in route_decisions and route_decisions[route_key] != bool(result["promoted"]):
            raise RuntimeError(f"route decision instability detected for {route_key}")
        route_decisions[route_key] = bool(result["promoted"])

        if result["promoted"]:
            promoted_steps += 1
            resolved_scores.append(1.0)
        else:
            resolved_scores.append(1.0 - trace.unresolved_by_route[route_key])

        next_j = (j + 1) % trace.trace_length
        state = step_entry(
            state,
            fiber_moduli=trace.layer_primes,
            next_return_gap=trace.gaps[next_j],
        )
        if result["promoted"]:
            state = initialize_entry(
                b=next_j % 35,
                phi=trace.phases[next_j],
                r=trace.r_depth,
                next_return_gap=trace.gaps[next_j],
            )

    promoted_routes = [key for key, promoted in route_decisions.items() if promoted]
    nonpromoted_routes = [key for key, promoted in route_decisions.items() if not promoted]

    return {
        "trace_source": trace.trace_source,
        "tuplet_name": trace.tuplet_name,
        "offsets": trace.offsets,
        "parent_W": trace.parent_W,
        "child_W": trace.child_W,
        "visible_H_first": trace.visible_H_first,
        "trace_length": trace.trace_length,
        "r_depth": trace.r_depth,
        "route_policy_cache_size": len(adapter.route_policy_cache),
        "route_reuse_fraction": cache_hits / trace.trace_length,
        "promotion_route_fraction": len(promoted_routes) / len(route_decisions) if route_decisions else 0.0,
        "promotion_step_fraction": promoted_steps / trace.trace_length,
        "effective_resolved_fraction": mean(resolved_scores),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [trace.unresolved_by_route[key] for key in nonpromoted_routes]
        ),
        "route_decision_instability": 0.0,
        "observed_route_modes": ",".join(sorted(route_modes)),
        "unique_emitted_route_keys": len(emitted_route_keys),
    }


def summarize_rows(rows: list[dict[str, object]]) -> dict[str, object]:
    return {
        "trace_source": "ALL",
        "tuplet_name": "ALL",
        "offsets": "ALL",
        "parent_W": "",
        "child_W": "",
        "visible_H_first": "",
        "trace_length": sum(int(row["trace_length"]) for row in rows),
        "r_depth": "",
        "route_policy_cache_size": mean([float(row["route_policy_cache_size"]) for row in rows]),
        "route_reuse_fraction": mean([float(row["route_reuse_fraction"]) for row in rows]),
        "promotion_route_fraction": mean([float(row["promotion_route_fraction"]) for row in rows]),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows]),
        "effective_resolved_fraction": mean([float(row["effective_resolved_fraction"]) for row in rows]),
        "mean_unresolved_among_nonpromoted_routes": mean(
            [float(row["mean_unresolved_among_nonpromoted_routes"]) for row in rows]
        ),
        "route_decision_instability": 0.0,
        "observed_route_modes": "R_full,R_min",
        "unique_emitted_route_keys": mean([float(row["unique_emitted_route_keys"]) for row in rows]),
    }
