#!/usr/bin/env python3
"""Refined primary-to-transport lift with explicit radial preservation."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from geometry_native_lift_map_v1 import (
    LIFT_HORIZON_V1,
    PrimaryChartStateV1,
    _phi_str,
    _spin_str,
    _truncate_spin_h_v1,
    build_lift_observations_v1,
)
from geometry_native_operator_model_v1 import NativeSpinHV1


OUTPUT_PATH_LIFT_MAP_V2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_lift_map_v2.csv"
)


@dataclass(frozen=True)
class TransportStateLHPlusV2:
    b: int
    r: int
    spin_h: NativeSpinHV1


@dataclass(frozen=True)
class LiftObservationV2:
    trace_source: str
    parent_W: int
    child_W: int
    j: int
    primary_state: PrimaryChartStateV1
    transport_state: TransportStateLHPlusV2
    full_spin_length: int


def build_lift_observations_v2(horizon: int = LIFT_HORIZON_V1) -> tuple[list[LiftObservationV2], int]:
    traces = build_bounded_traces(limit_per_source=None)
    observations: list[LiftObservationV2] = []
    skipped_short_spin = 0
    for trace in traces:
        for j in range(trace.trace_length):
            spin_H = tuple(int(bit) for bit in trace.spins[j])
            if len(spin_H) < horizon:
                skipped_short_spin += 1
                continue
            primary_state = PrimaryChartStateV1(
                b=int(j % 35),
                phi=tuple(int(v) for v in trace.phases[j]),
                r=int(trace.r_depth),
            )
            transport_state = TransportStateLHPlusV2(
                b=primary_state.b,
                r=primary_state.r,
                spin_h=_truncate_spin_h_v1(spin_H, horizon=horizon),
            )
            observations.append(
                LiftObservationV2(
                    trace_source=trace.trace_source,
                    parent_W=int(trace.parent_W),
                    child_W=int(trace.child_W),
                    j=int(j),
                    primary_state=primary_state,
                    transport_state=transport_state,
                    full_spin_length=len(spin_H),
                )
            )
    return observations, skipped_short_spin


def _majority_transport_v2(states: Counter[TransportStateLHPlusV2]) -> TransportStateLHPlusV2:
    return max(
        states,
        key=lambda state: (
            states[state],
            -state.b,
            -state.r,
            _spin_str(state.spin_h),
        ),
    )


def summarize_lift_map_v2(horizon: int = LIFT_HORIZON_V1) -> list[dict[str, object]]:
    observations, skipped_short_spin = build_lift_observations_v2(horizon=horizon)
    v1_observations, _ = build_lift_observations_v1(horizon=horizon)
    primary_to_transport: dict[PrimaryChartStateV1, Counter[TransportStateLHPlusV2]] = defaultdict(Counter)
    transport_preimage: dict[TransportStateLHPlusV2, list[PrimaryChartStateV1]] = defaultdict(list)
    full_spin_lengths: Counter[int] = Counter()

    for obs in observations:
        primary_to_transport[obs.primary_state][obs.transport_state] += 1
        full_spin_lengths[obs.full_spin_length] += 1

    representative_map: dict[PrimaryChartStateV1, TransportStateLHPlusV2] = {}
    ambiguous_primary_count = 0
    phi_discriminates = False
    r_discriminates = False

    by_b_r: dict[tuple[int, int], set[tuple[tuple[int, ...], TransportStateLHPlusV2]]] = defaultdict(set)
    by_b_phi: dict[tuple[int, tuple[int, ...]], set[tuple[int, TransportStateLHPlusV2]]] = defaultdict(set)

    for primary_state, transport_counter in primary_to_transport.items():
        representative = _majority_transport_v2(transport_counter)
        representative_map[primary_state] = representative
        transport_preimage[representative].append(primary_state)
        if len(transport_counter) > 1:
            ambiguous_primary_count += 1
        by_b_r[(primary_state.b, primary_state.r)].add((primary_state.phi, representative))
        by_b_phi[(primary_state.b, primary_state.phi)].add((primary_state.r, representative))

    for values in by_b_r.values():
        phi_to_transport = defaultdict(set)
        for phi, transport in values:
            phi_to_transport[phi].add(transport)
        if len(phi_to_transport) > 1:
            transports = {next(iter(v)) for v in phi_to_transport.values() if len(v) == 1}
            if len(transports) > 1:
                phi_discriminates = True
                break

    for values in by_b_phi.values():
        r_to_transport = defaultdict(set)
        for r, transport in values:
            r_to_transport[r].add(transport)
        if len(r_to_transport) > 1:
            transports = {next(iter(v)) for v in r_to_transport.values() if len(v) == 1}
            if len(transports) > 1:
                r_discriminates = True
                break

    primary_count = len(representative_map)
    distinct_transport_count = len(transport_preimage)
    collision_count = sum(max(len(preimage) - 1, 0) for preimage in transport_preimage.values())
    collision_fraction = collision_count / max(primary_count, 1)

    v1_primary_to_transport: dict[PrimaryChartStateV1, Counter[tuple[int, str]]] = defaultdict(Counter)
    for obs in v1_observations:
        v1_primary_to_transport[obs.primary_state][(obs.transport_state.b, _spin_str(obs.transport_state.spin_h))] += 1
    v1_ambiguous = sum(1 for counter in v1_primary_to_transport.values() if len(counter) > 1)
    v1_repr: dict[PrimaryChartStateV1, tuple[int, str]] = {
        primary: max(counter, key=lambda item: (counter[item], -item[0], item[1]))
        for primary, counter in v1_primary_to_transport.items()
    }
    v1_preimage: dict[tuple[int, str], list[PrimaryChartStateV1]] = defaultdict(list)
    for primary, transport in v1_repr.items():
        v1_preimage[transport].append(primary)
    v1_collision_count = sum(max(len(preimage) - 1, 0) for preimage in v1_preimage.values())
    v1_collision_fraction = v1_collision_count / max(len(v1_repr), 1)

    rows: list[dict[str, object]] = []
    rows.append(
        {
            "scope": "summary",
            "metric": "primary_states_examined",
            "count": primary_count,
            "total": primary_count,
            "fraction": 1.0,
            "note": "distinct observed primary chart states (b,phi,r) on bounded exact surface",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "distinct_transport_states_reached",
            "count": distinct_transport_count,
            "total": primary_count,
            "fraction": distinct_transport_count / max(primary_count, 1),
            "note": "distinct refined transport states (b,r,spin_h)",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "collision_count",
            "count": collision_count,
            "total": primary_count,
            "fraction": collision_fraction,
            "note": "many-to-one collisions under representative L_h^+",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "ambiguous_primary_states",
            "count": ambiguous_primary_count,
            "total": primary_count,
            "fraction": ambiguous_primary_count / max(primary_count, 1),
            "note": "primary states observed with more than one exact truncated refined transport state",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "skipped_short_spin_observations",
            "count": skipped_short_spin,
            "total": len(observations) + skipped_short_spin,
            "fraction": skipped_short_spin / max(len(observations) + skipped_short_spin, 1),
            "note": "observations skipped because exact future word shorter than truncation horizon",
        }
    )
    rows.append(
        {
            "scope": "comparison_vs_v1",
            "metric": "collision_reduction_vs_v1",
            "count": v1_collision_count - collision_count,
            "total": v1_collision_count,
            "fraction": v1_collision_fraction - collision_fraction,
            "note": "absolute collision reduction and collision-fraction reduction relative to lift v1",
        }
    )
    rows.append(
        {
            "scope": "comparison_vs_v1",
            "metric": "ambiguity_reduction_vs_v1",
            "count": v1_ambiguous - ambiguous_primary_count,
            "total": v1_ambiguous,
            "fraction": (v1_ambiguous / max(len(v1_repr), 1)) - (ambiguous_primary_count / max(primary_count, 1)),
            "note": "absolute ambiguity reduction and ambiguity-fraction reduction relative to lift v1",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "phi_represented_in_transport_identity",
            "count": int(phi_discriminates),
            "total": 1,
            "fraction": float(phi_discriminates),
            "note": "whether changes in phi can change refined transport identity at fixed (b,r)",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "r_represented_in_transport_identity",
            "count": int(r_discriminates),
            "total": 1,
            "fraction": float(r_discriminates),
            "note": "whether changes in r can change refined transport identity at fixed (b,phi)",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "predictive_unfolding_explicit",
            "count": 1,
            "total": 1,
            "fraction": 1.0,
            "note": "refined transport identity carries explicit truncated future admissibility word spin_h",
        }
    )

    for full_length, count in sorted(full_spin_lengths.items()):
        rows.append(
            {
                "scope": "source_spin_length",
                "metric": f"full_spin_length_{full_length}",
                "count": count,
                "total": len(observations),
                "fraction": count / max(len(observations), 1),
                "note": "distribution of exact source spin_H lengths before truncation",
            }
        )

    for primary_state, representative in sorted(
        representative_map.items(),
        key=lambda item: (item[0].r, item[0].b, item[0].phi),
    ):
        observed_counter = primary_to_transport[primary_state]
        majority_count = observed_counter[representative]
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": (
                    f"primary_b{primary_state.b}_phi{_phi_str(primary_state.phi)}_r{primary_state.r}"
                    f"__to__transport_b{representative.b}_r{representative.r}_spin{_spin_str(representative.spin_h)}"
                ),
                "count": majority_count,
                "total": sum(observed_counter.values()),
                "fraction": majority_count / max(sum(observed_counter.values()), 1),
                "note": (
                    f"representative L_h^+ image; variants={len(observed_counter)}; "
                    f"observed_transport_words={[ _spin_str(state.spin_h) for state in sorted(observed_counter, key=lambda s: (s.r, _spin_str(s.spin_h))) ]}"
                ),
            }
        )

    for transport_state, preimage in sorted(
        transport_preimage.items(),
        key=lambda item: (item[0].r, item[0].b, _spin_str(item[0].spin_h)),
    ):
        rows.append(
            {
                "scope": "transport_distribution",
                "metric": f"transport_b{transport_state.b}_r{transport_state.r}_spin{_spin_str(transport_state.spin_h)}",
                "count": len(preimage),
                "total": primary_count,
                "fraction": len(preimage) / max(primary_count, 1),
                "note": "number of primary states mapping to this refined transport state under representative L_h^+",
            }
        )

    return rows


def write_lift_map_v2(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_LIFT_MAP_V2) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["scope", "metric", "count", "total", "fraction", "note"],
        )
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_lift_map_v2(horizon=LIFT_HORIZON_V1)
    write_lift_map_v2(summary_rows, OUTPUT_PATH_LIFT_MAP_V2)
    print(f"wrote {OUTPUT_PATH_LIFT_MAP_V2}")
    for row in summary_rows:
        if row["scope"] in {"summary", "comparison_vs_v1"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
