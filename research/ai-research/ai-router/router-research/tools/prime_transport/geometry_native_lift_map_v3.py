#!/usr/bin/env python3
"""Adaptive-horizon primary-to-transport lift with explicit radial preservation."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from geometry_native_lift_map_v1 import LIFT_HORIZON_V1, PrimaryChartStateV1, _phi_str, _spin_str
from geometry_native_operator_model_v1 import NativeSpinHV1


OUTPUT_PATH_LIFT_MAP_V3 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_lift_map_v3.csv"
)

ADAPTIVE_H_MIN_V3 = LIFT_HORIZON_V1
ADAPTIVE_H_CAP_V3 = 12


@dataclass(frozen=True)
class TransportStateAdaptiveV3:
    b: int
    r: int
    spin_h: NativeSpinHV1


@dataclass(frozen=True)
class LiftObservationV3:
    trace_source: str
    parent_W: int
    child_W: int
    j: int
    primary_state: PrimaryChartStateV1
    transport_state: TransportStateAdaptiveV3
    full_spin_length: int
    realized_horizon: int


def _truncate_spin_v3(spin_H: tuple[int, ...], horizon: int) -> NativeSpinHV1:
    return NativeSpinHV1(horizon=horizon, bits=tuple(int(bit) for bit in spin_H[:horizon]))


def _majority_transport_v3(states: Counter[TransportStateAdaptiveV3]) -> TransportStateAdaptiveV3:
    return max(
        states,
        key=lambda state: (
            states[state],
            -state.b,
            -state.r,
            -state.spin_h.horizon,
            _spin_str(state.spin_h),
        ),
    )


def _summary_metrics_from_csv(path: Path) -> dict[str, float]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "comparison_vs_v1"}:
            out[row["metric"]] = float(row["count"]) if row["metric"].endswith("_count") or row["metric"].endswith("_states") else float(row["fraction"])
            # also keep raw count name explicitly
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def build_lift_observations_v3(
    h_min: int = ADAPTIVE_H_MIN_V3,
    h_cap: int = ADAPTIVE_H_CAP_V3,
) -> tuple[list[LiftObservationV3], int]:
    traces = build_bounded_traces(limit_per_source=None)
    raw_records: list[tuple[str, int, int, int, PrimaryChartStateV1, tuple[int, ...]]] = []
    skipped_short_spin = 0
    prefix_to_primary_by_h: dict[int, dict[tuple[int, int, tuple[int, ...]], set[PrimaryChartStateV1]]] = defaultdict(lambda: defaultdict(set))

    for trace in traces:
        for j in range(trace.trace_length):
            full_spin = tuple(int(bit) for bit in trace.spins[j])
            if len(full_spin) < h_min:
                skipped_short_spin += 1
                continue
            primary = PrimaryChartStateV1(
                b=int(j % 35),
                phi=tuple(int(v) for v in trace.phases[j]),
                r=int(trace.r_depth),
            )
            raw_records.append((trace.trace_source, int(trace.parent_W), int(trace.child_W), int(j), primary, full_spin))
            max_h = min(h_cap, len(full_spin))
            for horizon in range(h_min, max_h + 1):
                prefix_to_primary_by_h[horizon][(primary.b, primary.r, full_spin[:horizon])].add(primary)

    observations: list[LiftObservationV3] = []
    for trace_source, parent_W, child_W, j, primary, full_spin in raw_records:
        max_h = min(h_cap, len(full_spin))
        chosen_h = max_h
        for horizon in range(h_min, max_h + 1):
            prefix_key = (primary.b, primary.r, full_spin[:horizon])
            primary_set = prefix_to_primary_by_h[horizon][prefix_key]
            if len(primary_set) == 1:
                chosen_h = horizon
                break
        observations.append(
            LiftObservationV3(
                trace_source=trace_source,
                parent_W=parent_W,
                child_W=child_W,
                j=j,
                primary_state=primary,
                transport_state=TransportStateAdaptiveV3(
                    b=primary.b,
                    r=primary.r,
                    spin_h=_truncate_spin_v3(full_spin, chosen_h),
                ),
                full_spin_length=len(full_spin),
                realized_horizon=chosen_h,
            )
        )
    return observations, skipped_short_spin


def summarize_lift_map_v3(
    h_min: int = ADAPTIVE_H_MIN_V3,
    h_cap: int = ADAPTIVE_H_CAP_V3,
) -> list[dict[str, object]]:
    observations, skipped_short_spin = build_lift_observations_v3(h_min=h_min, h_cap=h_cap)
    primary_to_transport: dict[PrimaryChartStateV1, Counter[TransportStateAdaptiveV3]] = defaultdict(Counter)
    transport_preimage: dict[TransportStateAdaptiveV3, list[PrimaryChartStateV1]] = defaultdict(list)
    full_spin_lengths: Counter[int] = Counter()
    horizon_counter: Counter[int] = Counter()

    for obs in observations:
        primary_to_transport[obs.primary_state][obs.transport_state] += 1
        full_spin_lengths[obs.full_spin_length] += 1
        horizon_counter[obs.realized_horizon] += 1

    representative_map: dict[PrimaryChartStateV1, TransportStateAdaptiveV3] = {}
    ambiguous_primary_count = 0
    phi_discriminates = False
    r_discriminates = False

    by_b_r: dict[tuple[int, int], set[tuple[tuple[int, ...], TransportStateAdaptiveV3]]] = defaultdict(set)
    by_b_phi: dict[tuple[int, tuple[int, ...]], set[tuple[int, TransportStateAdaptiveV3]]] = defaultdict(set)

    for primary_state, transport_counter in primary_to_transport.items():
        representative = _majority_transport_v3(transport_counter)
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

    v1_metrics = _summary_metrics_from_csv(
        Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v1.csv")
    )
    v2_metrics = _summary_metrics_from_csv(
        Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v2.csv")
    )

    horizon_values = [obs.realized_horizon for obs in observations]
    horizon_min = min(horizon_values) if horizon_values else 0
    horizon_mean = (sum(horizon_values) / len(horizon_values)) if horizon_values else 0.0
    horizon_max = max(horizon_values) if horizon_values else 0

    rows: list[dict[str, object]] = []
    rows.append(
        {"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct observed primary chart states (b,phi,r) on bounded exact surface"}
    )
    rows.append(
        {"scope": "summary", "metric": "distinct_transport_states_reached", "count": distinct_transport_count, "total": primary_count, "fraction": distinct_transport_count / max(primary_count, 1), "note": "distinct adaptive transport states (b,r,spin_h)"}
    )
    rows.append(
        {"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions under representative adaptive L_h^+"}
    )
    rows.append(
        {"scope": "summary", "metric": "ambiguous_primary_states", "count": ambiguous_primary_count, "total": primary_count, "fraction": ambiguous_primary_count / max(primary_count, 1), "note": "primary states observed with more than one exact truncated adaptive transport state"}
    )
    rows.append(
        {"scope": "summary", "metric": "skipped_short_spin_observations", "count": skipped_short_spin, "total": len(observations) + skipped_short_spin, "fraction": skipped_short_spin / max(len(observations) + skipped_short_spin, 1), "note": "observations skipped because exact future word shorter than minimum truncation horizon"}
    )
    rows.append(
        {"scope": "comparison", "metric": "collision_reduction_vs_v1", "count": int(v1_metrics["collision_count__count"] - collision_count), "total": int(v1_metrics["collision_count__count"]), "fraction": v1_metrics["collision_count__fraction"] - collision_fraction, "note": "absolute collision reduction and collision-fraction reduction relative to lift v1"}
    )
    rows.append(
        {"scope": "comparison", "metric": "collision_reduction_vs_v2", "count": int(v2_metrics["collision_count__count"] - collision_count), "total": int(v2_metrics["collision_count__count"]), "fraction": v2_metrics["collision_count__fraction"] - collision_fraction, "note": "absolute collision reduction and collision-fraction reduction relative to lift v2"}
    )
    rows.append(
        {"scope": "comparison", "metric": "ambiguity_reduction_vs_v1", "count": int(v1_metrics["ambiguous_primary_states__count"] - ambiguous_primary_count), "total": int(v1_metrics["ambiguous_primary_states__count"]), "fraction": v1_metrics["ambiguous_primary_states__fraction"] - (ambiguous_primary_count / max(primary_count, 1)), "note": "absolute ambiguity reduction and ambiguity-fraction reduction relative to lift v1"}
    )
    rows.append(
        {"scope": "comparison", "metric": "ambiguity_reduction_vs_v2", "count": int(v2_metrics["ambiguous_primary_states__count"] - ambiguous_primary_count), "total": int(v2_metrics["ambiguous_primary_states__count"]), "fraction": v2_metrics["ambiguous_primary_states__fraction"] - (ambiguous_primary_count / max(primary_count, 1)), "note": "absolute ambiguity reduction and ambiguity-fraction reduction relative to lift v2"}
    )
    rows.append(
        {"scope": "representation", "metric": "phi_represented_in_transport_identity", "count": int(phi_discriminates), "total": 1, "fraction": float(phi_discriminates), "note": "whether changes in phi can change adaptive transport identity at fixed (b,r)"}
    )
    rows.append(
        {"scope": "representation", "metric": "r_represented_in_transport_identity", "count": int(r_discriminates), "total": 1, "fraction": float(r_discriminates), "note": "whether changes in r can change adaptive transport identity at fixed (b,phi)"}
    )
    rows.append(
        {"scope": "representation", "metric": "predictive_unfolding_explicit", "count": 1, "total": 1, "fraction": 1.0, "note": "adaptive transport identity carries explicit truncated future admissibility word spin_h"}
    )
    rows.append(
        {"scope": "adaptive_horizon", "metric": "horizon_min", "count": horizon_min, "total": h_cap, "fraction": horizon_min / max(h_cap, 1), "note": "minimum realized adaptive horizon"}
    )
    rows.append(
        {"scope": "adaptive_horizon", "metric": "horizon_mean_x1000", "count": int(round(horizon_mean * 1000)), "total": 1000, "fraction": horizon_mean, "note": "mean realized adaptive horizon"}
    )
    rows.append(
        {"scope": "adaptive_horizon", "metric": "horizon_max", "count": horizon_max, "total": h_cap, "fraction": horizon_max / max(h_cap, 1), "note": "maximum realized adaptive horizon"}
    )

    for horizon, count in sorted(horizon_counter.items()):
        rows.append(
            {"scope": "adaptive_horizon_distribution", "metric": f"horizon_{horizon}", "count": count, "total": len(observations), "fraction": count / max(len(observations), 1), "note": "distribution of realized adaptive horizons"}
        )

    for full_length, count in sorted(full_spin_lengths.items()):
        rows.append(
            {"scope": "source_spin_length", "metric": f"full_spin_length_{full_length}", "count": count, "total": len(observations), "fraction": count / max(len(observations), 1), "note": "distribution of exact source spin_H lengths before adaptive truncation"}
        )

    for primary_state, representative in sorted(representative_map.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        observed_counter = primary_to_transport[primary_state]
        majority_count = observed_counter[representative]
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": (
                    f"primary_b{primary_state.b}_phi{_phi_str(primary_state.phi)}_r{primary_state.r}"
                    f"__to__transport_b{representative.b}_r{representative.r}_h{representative.spin_h.horizon}_spin{_spin_str(representative.spin_h)}"
                ),
                "count": majority_count,
                "total": sum(observed_counter.values()),
                "fraction": majority_count / max(sum(observed_counter.values()), 1),
                "note": (
                    f"representative adaptive lift image; variants={len(observed_counter)}; "
                    f"observed_horizons={[state.spin_h.horizon for state in sorted(observed_counter, key=lambda s: (s.spin_h.horizon, _spin_str(s.spin_h)))]}"
                ),
            }
        )

    for transport_state, preimage in sorted(transport_preimage.items(), key=lambda item: (item[0].r, item[0].b, item[0].spin_h.horizon, _spin_str(item[0].spin_h))):
        rows.append(
            {
                "scope": "transport_distribution",
                "metric": f"transport_b{transport_state.b}_r{transport_state.r}_h{transport_state.spin_h.horizon}_spin{_spin_str(transport_state.spin_h)}",
                "count": len(preimage),
                "total": primary_count,
                "fraction": len(preimage) / max(primary_count, 1),
                "note": "number of primary states mapping to this adaptive transport state under representative lift",
            }
        )

    return rows


def write_lift_map_v3(rows: list[dict[str, object]], output_path: Path = OUTPUT_PATH_LIFT_MAP_V3) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_lift_map_v3()
    write_lift_map_v3(summary_rows, OUTPUT_PATH_LIFT_MAP_V3)
    print(f"wrote {OUTPUT_PATH_LIFT_MAP_V3}")
    for row in summary_rows:
        if row["scope"] in {"summary", "comparison", "adaptive_horizon"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
