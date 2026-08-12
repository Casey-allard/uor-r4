#!/usr/bin/env python3
"""Structured spin_H candidate that is fuller than prefix-only truncation."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from base_gap_benchmark_wrapper import build_bounded_traces
from geometry_native_lift_map_v1 import LIFT_HORIZON_V1, PrimaryChartStateV1, _phi_str, _spin_str, _truncate_spin_h_v1
from geometry_native_operator_model_v1 import NativeSpinHV1


OUTPUT_PATH_SPINH_CANDIDATE_V1 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_candidate_v1.csv"
)


@dataclass(frozen=True)
class SpinHCandidateV1:
    angular_fiber: tuple[int, ...]
    radial_depth: int
    predictive_word: NativeSpinHV1


@dataclass(frozen=True)
class TransportStateSpinHCandidateV1:
    b: int
    spin_H_candidate: SpinHCandidateV1


@dataclass(frozen=True)
class LiftObservationSpinHCandidateV1:
    trace_source: str
    parent_W: int
    child_W: int
    j: int
    primary_state: PrimaryChartStateV1
    transport_state: TransportStateSpinHCandidateV1
    full_spin_length: int


def _summary_metrics_from_csv(path: Path) -> dict[str, float]:
    rows = list(csv.DictReader(path.open("r", encoding="utf-8")))
    out: dict[str, float] = {}
    for row in rows:
        if row["scope"] in {"summary", "comparison", "comparison_vs_v1"}:
            out[f"{row['metric']}__count"] = float(row["count"])
            out[f"{row['metric']}__fraction"] = float(row["fraction"])
    return out


def build_spinH_candidate_observations_v1(horizon: int = LIFT_HORIZON_V1) -> tuple[list[LiftObservationSpinHCandidateV1], int]:
    traces = build_bounded_traces(limit_per_source=None)
    observations: list[LiftObservationSpinHCandidateV1] = []
    skipped_short_spin = 0
    for trace in traces:
        for j in range(trace.trace_length):
            full_spin = tuple(int(bit) for bit in trace.spins[j])
            if len(full_spin) < horizon:
                skipped_short_spin += 1
                continue
            primary = PrimaryChartStateV1(
                b=int(j % 35),
                phi=tuple(int(v) for v in trace.phases[j]),
                r=int(trace.r_depth),
            )
            candidate = SpinHCandidateV1(
                angular_fiber=primary.phi,
                radial_depth=primary.r,
                predictive_word=_truncate_spin_h_v1(full_spin, horizon=horizon),
            )
            observations.append(
                LiftObservationSpinHCandidateV1(
                    trace_source=trace.trace_source,
                    parent_W=int(trace.parent_W),
                    child_W=int(trace.child_W),
                    j=int(j),
                    primary_state=primary,
                    transport_state=TransportStateSpinHCandidateV1(b=primary.b, spin_H_candidate=candidate),
                    full_spin_length=len(full_spin),
                )
            )
    return observations, skipped_short_spin


def _majority_transport_v1(
    states: Counter[TransportStateSpinHCandidateV1],
) -> TransportStateSpinHCandidateV1:
    return max(
        states,
        key=lambda state: (
            states[state],
            -state.b,
            _phi_str(state.spin_H_candidate.angular_fiber),
            -state.spin_H_candidate.radial_depth,
            _spin_str(state.spin_H_candidate.predictive_word),
        ),
    )


def summarize_spinH_candidate_v1(horizon: int = LIFT_HORIZON_V1) -> list[dict[str, object]]:
    observations, skipped_short_spin = build_spinH_candidate_observations_v1(horizon=horizon)
    primary_to_transport: dict[PrimaryChartStateV1, Counter[TransportStateSpinHCandidateV1]] = defaultdict(Counter)
    transport_preimage: dict[TransportStateSpinHCandidateV1, list[PrimaryChartStateV1]] = defaultdict(list)
    full_spin_lengths: Counter[int] = Counter()

    for obs in observations:
        primary_to_transport[obs.primary_state][obs.transport_state] += 1
        full_spin_lengths[obs.full_spin_length] += 1

    representative_map: dict[PrimaryChartStateV1, TransportStateSpinHCandidateV1] = {}
    ambiguous_primary_count = 0
    phi_discriminates = False
    r_discriminates = False

    by_b_r: dict[tuple[int, int], set[tuple[tuple[int, ...], TransportStateSpinHCandidateV1]]] = defaultdict(set)
    by_b_phi: dict[tuple[int, tuple[int, ...]], set[tuple[int, TransportStateSpinHCandidateV1]]] = defaultdict(set)

    for primary_state, transport_counter in primary_to_transport.items():
        representative = _majority_transport_v1(transport_counter)
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
    v3_metrics = _summary_metrics_from_csv(
        Path("/Users/adminamn/AI-Research/ai-router/router-research/results/prime_transport_recursive_system/prime_transport_lift_map_v3.csv")
    )

    rows: list[dict[str, object]] = []
    rows.append(
        {"scope": "summary", "metric": "primary_states_examined", "count": primary_count, "total": primary_count, "fraction": 1.0, "note": "distinct observed primary chart states (b,phi,r) on bounded exact surface"}
    )
    rows.append(
        {"scope": "summary", "metric": "distinct_transport_states_reached", "count": distinct_transport_count, "total": primary_count, "fraction": distinct_transport_count / max(primary_count, 1), "note": "distinct transport states for structured spin_H candidate"}
    )
    rows.append(
        {"scope": "summary", "metric": "collision_count", "count": collision_count, "total": primary_count, "fraction": collision_fraction, "note": "many-to-one collisions under representative structured spin_H candidate"}
    )
    rows.append(
        {"scope": "summary", "metric": "ambiguous_primary_states", "count": ambiguous_primary_count, "total": primary_count, "fraction": ambiguous_primary_count / max(primary_count, 1), "note": "primary states observed with more than one structured spin_H candidate state"}
    )
    rows.append(
        {"scope": "summary", "metric": "skipped_short_spin_observations", "count": skipped_short_spin, "total": len(observations) + skipped_short_spin, "fraction": skipped_short_spin / max(len(observations) + skipped_short_spin, 1), "note": "observations skipped because exact future word shorter than candidate predictive horizon"}
    )
    for label, metrics in (("v1", v1_metrics), ("v2", v2_metrics), ("v3", v3_metrics)):
        rows.append(
            {"scope": "comparison", "metric": f"collision_reduction_vs_{label}", "count": int(metrics["collision_count__count"] - collision_count), "total": int(metrics["collision_count__count"]), "fraction": metrics["collision_count__fraction"] - collision_fraction, "note": f"absolute collision reduction and collision-fraction reduction relative to lift {label}"}
        )
        rows.append(
            {"scope": "comparison", "metric": f"ambiguity_reduction_vs_{label}", "count": int(metrics["ambiguous_primary_states__count"] - ambiguous_primary_count), "total": int(metrics["ambiguous_primary_states__count"]), "fraction": metrics["ambiguous_primary_states__fraction"] - (ambiguous_primary_count / max(primary_count, 1)), "note": f"absolute ambiguity reduction and ambiguity-fraction reduction relative to lift {label}"}
        )
    rows.append(
        {"scope": "representation", "metric": "phi_represented_in_transport_identity", "count": int(phi_discriminates), "total": 1, "fraction": float(phi_discriminates), "note": "whether angular fiber component phi changes candidate transport identity at fixed (b,r)"}
    )
    rows.append(
        {"scope": "representation", "metric": "r_represented_in_transport_identity", "count": int(r_discriminates), "total": 1, "fraction": float(r_discriminates), "note": "whether radial depth r changes candidate transport identity at fixed (b,phi)"}
    )
    rows.append(
        {"scope": "representation", "metric": "predictive_unfolding_explicit", "count": 1, "total": 1, "fraction": 1.0, "note": "candidate transport identity carries explicit predictive admissibility component"}
    )
    rows.append(
        {"scope": "representation", "metric": "angular_identity_explicit", "count": 1, "total": 1, "fraction": 1.0, "note": "candidate transport chart preserves base angle b explicitly and carries phi as angular-fiber subcomponent"}
    )

    for full_length, count in sorted(full_spin_lengths.items()):
        rows.append(
            {"scope": "source_spin_length", "metric": f"full_spin_length_{full_length}", "count": count, "total": len(observations), "fraction": count / max(len(observations), 1), "note": "distribution of exact source spin_H lengths before structured candidate construction"}
        )

    for primary_state, representative in sorted(representative_map.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        observed_counter = primary_to_transport[primary_state]
        majority_count = observed_counter[representative]
        candidate = representative.spin_H_candidate
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": (
                    f"primary_b{primary_state.b}_phi{_phi_str(primary_state.phi)}_r{primary_state.r}"
                    f"__to__transport_b{representative.b}_phi{_phi_str(candidate.angular_fiber)}_r{candidate.radial_depth}_spin{_spin_str(candidate.predictive_word)}"
                ),
                "count": majority_count,
                "total": sum(observed_counter.values()),
                "fraction": majority_count / max(sum(observed_counter.values()), 1),
                "note": (
                    f"representative structured candidate image; variants={len(observed_counter)}; "
                    f"observed_transport_words={[ _spin_str(state.spin_H_candidate.predictive_word) for state in sorted(observed_counter, key=lambda s: (_phi_str(s.spin_H_candidate.angular_fiber), s.spin_H_candidate.radial_depth, _spin_str(s.spin_H_candidate.predictive_word))) ]}"
                ),
            }
        )

    for transport_state, preimage in sorted(
        transport_preimage.items(),
        key=lambda item: (
            item[0].b,
            item[0].spin_H_candidate.radial_depth,
            item[0].spin_H_candidate.angular_fiber,
            _spin_str(item[0].spin_H_candidate.predictive_word),
        ),
    ):
        candidate = transport_state.spin_H_candidate
        rows.append(
            {
                "scope": "transport_distribution",
                "metric": f"transport_b{transport_state.b}_phi{_phi_str(candidate.angular_fiber)}_r{candidate.radial_depth}_spin{_spin_str(candidate.predictive_word)}",
                "count": len(preimage),
                "total": primary_count,
                "fraction": len(preimage) / max(primary_count, 1),
                "note": "number of primary states mapping to this structured spin_H candidate state under representative lift",
            }
        )

    return rows


def write_spinH_candidate_v1(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CANDIDATE_V1,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=["scope", "metric", "count", "total", "fraction", "note"])
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_spinH_candidate_v1(horizon=LIFT_HORIZON_V1)
    write_spinH_candidate_v1(summary_rows, OUTPUT_PATH_SPINH_CANDIDATE_V1)
    print(f"wrote {OUTPUT_PATH_SPINH_CANDIDATE_V1}")
    for row in summary_rows:
        if row["scope"] in {"summary", "comparison", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
