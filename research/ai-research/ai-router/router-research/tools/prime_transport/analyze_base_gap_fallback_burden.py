#!/usr/bin/env python3
"""Profile promoted/fallback burden for the guarded base_gap routing policy."""

from __future__ import annotations

import csv
from collections import Counter
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from base_gap_routing_adapter import adapter_output, initialize_adapter, initialize_entry
from run_mock_router_trace_eval import mean


ROOT = Path(__file__).resolve().parents[2]
OUT_CSV = ROOT / "results" / "prime_transport_recursive_system" / "prime_transport_base_gap_fallback_burden.csv"


def write_rows(path: Path, rows: list[dict[str, object]]) -> None:
    if not rows:
        raise ValueError(f"no rows to write: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def classify_fallback(top5_route_share: float, promotion_route_fraction: float) -> str:
    if top5_route_share >= 0.5 and promotion_route_fraction <= 0.2:
        return "A_concentrated"
    if top5_route_share <= 0.12 and promotion_route_fraction >= 0.2:
        return "C_diffuse"
    return "B_moderately_structured"


def main() -> None:
    rows_out: list[dict[str, object]] = []
    traces = build_bounded_traces()

    for trace in traces:
        adapter = initialize_adapter()
        promoted_route_steps: Counter[str] = Counter()
        promoted_gap_steps: Counter[int] = Counter()
        promoted_base_steps: Counter[int] = Counter()

        for j in range(trace.trace_length):
            route_key = f"base_gap:r={trace.r_depth}:b={j % 35}:gap={trace.gaps[j]}"
            state = initialize_entry(
                b=j % 35,
                phi=trace.phases[j],
                r=trace.r_depth,
                next_return_gap=trace.gaps[j],
            )
            result = adapter_output(
                adapter,
                state,
                unresolved_fraction=trace.unresolved_by_route[route_key],
                spin_H=trace.spins[j],
            )
            if result["promoted"]:
                promoted_route_steps[route_key] += 1
                promoted_gap_steps[trace.gaps[j]] += 1
                promoted_base_steps[j % 35] += 1

        promoted_steps = sum(promoted_route_steps.values())
        promoted_routes = len(promoted_route_steps)
        top5_route_share = (
            sum(count for _, count in promoted_route_steps.most_common(5)) / promoted_steps if promoted_steps else 0.0
        )
        top1_gap_share = (
            promoted_gap_steps.most_common(1)[0][1] / promoted_steps if promoted_steps else 0.0
        )
        top5_gap_share = (
            sum(count for _, count in promoted_gap_steps.most_common(5)) / promoted_steps if promoted_steps else 0.0
        )
        top1_base_share = (
            promoted_base_steps.most_common(1)[0][1] / promoted_steps if promoted_steps else 0.0
        )
        top5_base_share = (
            sum(count for _, count in promoted_base_steps.most_common(5)) / promoted_steps if promoted_steps else 0.0
        )
        promotion_route_fraction = promoted_routes / len(adapter.route_policy_cache) if adapter.route_policy_cache else 0.0

        rows_out.append(
            {
                "trace_source": trace.trace_source,
                "tuplet_name": trace.tuplet_name,
                "offsets": trace.offsets,
                "parent_W": trace.parent_W,
                "child_W": trace.child_W,
                "visible_H_first": trace.visible_H_first,
                "trace_length": trace.trace_length,
                "r_depth": trace.r_depth,
                "promoted_steps": promoted_steps,
                "promoted_routes": promoted_routes,
                "promotion_step_fraction": promoted_steps / trace.trace_length,
                "promotion_route_fraction": promotion_route_fraction,
                "top1_route_share": (
                    promoted_route_steps.most_common(1)[0][1] / promoted_steps if promoted_steps else 0.0
                ),
                "top5_route_share": top5_route_share,
                "top1_gap_share": top1_gap_share,
                "top5_gap_share": top5_gap_share,
                "top1_base_share": top1_base_share,
                "top5_base_share": top5_base_share,
                "fallback_burden_class": classify_fallback(top5_route_share, promotion_route_fraction),
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
        "promoted_steps": sum(int(row["promoted_steps"]) for row in rows_out),
        "promoted_routes": sum(int(row["promoted_routes"]) for row in rows_out),
        "promotion_step_fraction": mean([float(row["promotion_step_fraction"]) for row in rows_out]),
        "promotion_route_fraction": mean([float(row["promotion_route_fraction"]) for row in rows_out]),
        "top1_route_share": mean([float(row["top1_route_share"]) for row in rows_out]),
        "top5_route_share": mean([float(row["top5_route_share"]) for row in rows_out]),
        "top1_gap_share": mean([float(row["top1_gap_share"]) for row in rows_out]),
        "top5_gap_share": mean([float(row["top5_gap_share"]) for row in rows_out]),
        "top1_base_share": mean([float(row["top1_base_share"]) for row in rows_out]),
        "top5_base_share": mean([float(row["top5_base_share"]) for row in rows_out]),
        "fallback_burden_class": classify_fallback(
            mean([float(row["top5_route_share"]) for row in rows_out]),
            mean([float(row["promotion_route_fraction"]) for row in rows_out]),
        ),
    }
    rows_out.append(overall)

    write_rows(OUT_CSV, rows_out)
    print(f"fallback_burden_csv,{OUT_CSV}")


if __name__ == "__main__":
    main()
