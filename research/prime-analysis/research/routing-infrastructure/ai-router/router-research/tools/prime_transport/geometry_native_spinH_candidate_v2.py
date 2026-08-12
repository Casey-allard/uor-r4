#!/usr/bin/env python3
"""Recursive-stability audit for spin_H_candidate used as active transport lift."""

from __future__ import annotations

import csv
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path

from geometry_native_operator_model_v1 import NativeSpinHV1
from geometry_native_operator_model_v7 import bounded_operator_surface_v6
from geometry_native_spinH_candidate_v1 import _phi_str, _spin_str


OUTPUT_PATH_SPINH_CANDIDATE_V2 = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/prime_transport_spinH_candidate_v2.csv"
)


@dataclass(frozen=True)
class PrimaryChartStateSpinHCandidateV2:
    b: int
    phi: int
    r: int


@dataclass(frozen=True)
class SpinHCandidateV2:
    angular_fiber: tuple[int, ...]
    radial_depth: int
    predictive_word: NativeSpinHV1


@dataclass(frozen=True)
class TransportStateSpinHCandidateV2:
    b: int
    spin_H_candidate: SpinHCandidateV2


def active_transport_lift_v2(state: object) -> TransportStateSpinHCandidateV2:
    """Install spin_H_candidate_v1 as the active transport identity."""
    phi_tuple = (int(state.phi),)
    predictive_word = NativeSpinHV1(
        horizon=int(state.spin_h.horizon),
        bits=tuple(int(bit) for bit in state.spin_h.bits),
    )
    return TransportStateSpinHCandidateV2(
        b=int(state.b),
        spin_H_candidate=SpinHCandidateV2(
            angular_fiber=phi_tuple,
            radial_depth=int(state.r),
            predictive_word=predictive_word,
        ),
    )


def primary_chart_of_v2(state: object) -> PrimaryChartStateSpinHCandidateV2:
    return PrimaryChartStateSpinHCandidateV2(
        b=int(state.b),
        phi=int(state.phi),
        r=int(state.r),
    )


def summarize_spinH_candidate_v2(depth: int = 8) -> list[dict[str, object]]:
    states, transitions = bounded_operator_surface_v6(depth=depth)

    primary_to_transport: dict[
        PrimaryChartStateSpinHCandidateV2, set[TransportStateSpinHCandidateV2]
    ] = defaultdict(set)
    transport_to_primary: dict[
        TransportStateSpinHCandidateV2, set[PrimaryChartStateSpinHCandidateV2]
    ] = defaultdict(set)
    spin_counter: Counter[NativeSpinHV1] = Counter()
    class_counter: Counter[tuple[int, int, str]] = Counter()
    transport_counter: Counter[TransportStateSpinHCandidateV2] = Counter()

    angular_preserved = True
    radial_preserved = True
    predictive_present = True

    for state in states:
        primary = primary_chart_of_v2(state)
        transport = active_transport_lift_v2(state)
        primary_to_transport[primary].add(transport)
        transport_to_primary[transport].add(primary)
        spin_counter[transport.spin_H_candidate.predictive_word] += 1
        class_counter[
            (
                transport.b,
                transport.spin_H_candidate.radial_depth,
                _spin_str(transport.spin_H_candidate.predictive_word),
            )
        ] += 1
        transport_counter[transport] += 1
        angular_preserved &= transport.b == primary.b and transport.spin_H_candidate.angular_fiber == (primary.phi,)
        radial_preserved &= transport.spin_H_candidate.radial_depth == primary.r
        predictive_present &= transport.spin_H_candidate.predictive_word.horizon > 0

    primary_count = len(primary_to_transport)
    distinct_transport_count = len(transport_to_primary)
    collision_count = sum(max(len(preimages) - 1, 0) for preimages in transport_to_primary.values())
    ambiguity_count = sum(1 for images in primary_to_transport.values() if len(images) > 1)
    canonical_count = primary_count - ambiguity_count
    recursive_consistency_rate = canonical_count / max(primary_count, 1)
    branching_count = ambiguity_count

    rows: list[dict[str, object]] = []
    rows.append(
        {
            "scope": "summary",
            "metric": "primary_states_examined",
            "count": primary_count,
            "total": primary_count,
            "fraction": 1.0,
            "note": "distinct primary chart states (b,phi,r) on the bounded lawful operator surface",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "distinct_transport_identities_reached",
            "count": distinct_transport_count,
            "total": primary_count,
            "fraction": distinct_transport_count / max(primary_count, 1),
            "note": "distinct active transport identities under spin_H_candidate_v1 lift",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "collision_count",
            "count": collision_count,
            "total": primary_count,
            "fraction": collision_count / max(primary_count, 1),
            "note": "many-to-one collisions from primary chart states into active transport identities",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "ambiguity_count",
            "count": ambiguity_count,
            "total": primary_count,
            "fraction": ambiguity_count / max(primary_count, 1),
            "note": "primary chart states with more than one transport identity under repeated lawful iteration",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "recursive_consistency_rate",
            "count": canonical_count,
            "total": primary_count,
            "fraction": recursive_consistency_rate,
            "note": "fraction of primary chart states whose transport identity remains canonical under bounded lawful iteration",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "canonical_states_under_iteration",
            "count": canonical_count,
            "total": primary_count,
            "fraction": canonical_count / max(primary_count, 1),
            "note": "primary states that retain a single transport identity under repeated lawful transport",
        }
    )
    rows.append(
        {
            "scope": "summary",
            "metric": "noncanonical_branching_states",
            "count": branching_count,
            "total": primary_count,
            "fraction": branching_count / max(primary_count, 1),
            "note": "primary states whose transport identity branches under repeated lawful transport",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "angular_identity_preserved",
            "count": int(angular_preserved),
            "total": 1,
            "fraction": float(angular_preserved),
            "note": "active transport lift preserves b exactly and carries phi explicitly",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "radial_identity_preserved",
            "count": int(radial_preserved),
            "total": 1,
            "fraction": float(radial_preserved),
            "note": "active transport lift preserves r exactly",
        }
    )
    rows.append(
        {
            "scope": "representation",
            "metric": "predictive_structure_preserved",
            "count": int(predictive_present),
            "total": 1,
            "fraction": float(predictive_present),
            "note": "active transport lift carries predictive spin_h4 structure explicitly",
        }
    )
    recursively_closed_enough = ambiguity_count == 0
    rows.append(
        {
            "scope": "representation",
            "metric": "recursively_closed_enough_for_operator_derivation",
            "count": int(recursively_closed_enough),
            "total": 1,
            "fraction": float(recursively_closed_enough),
            "note": "strict closure test: every primary state must keep a single transport identity under lawful iteration",
        }
    )

    for primary, images in sorted(primary_to_transport.items(), key=lambda item: (item[0].r, item[0].b, item[0].phi)):
        ordered_images = sorted(
            images,
            key=lambda t: (
                t.b,
                t.spin_H_candidate.radial_depth,
                t.spin_H_candidate.angular_fiber,
                _spin_str(t.spin_H_candidate.predictive_word),
            ),
        )
        rows.append(
            {
                "scope": "primary_distribution",
                "metric": f"primary_b{primary.b}_phi{primary.phi}_r{primary.r}",
                "count": len(ordered_images),
                "total": distinct_transport_count,
                "fraction": len(ordered_images) / max(distinct_transport_count, 1),
                "note": "transport_images="
                + "|".join(
                    f"b{transport.b}_phi{_phi_str(transport.spin_H_candidate.angular_fiber)}_r{transport.spin_H_candidate.radial_depth}_spin{_spin_str(transport.spin_H_candidate.predictive_word)}"
                    for transport in ordered_images
                ),
            }
        )

    for transport, count in sorted(
        transport_counter.items(),
        key=lambda item: (
            item[0].b,
            item[0].spin_H_candidate.radial_depth,
            item[0].spin_H_candidate.angular_fiber,
            _spin_str(item[0].spin_H_candidate.predictive_word),
        ),
    ):
        rows.append(
            {
                "scope": "transport_distribution",
                "metric": f"transport_b{transport.b}_phi{_phi_str(transport.spin_H_candidate.angular_fiber)}_r{transport.spin_H_candidate.radial_depth}_spin{_spin_str(transport.spin_H_candidate.predictive_word)}",
                "count": count,
                "total": len(states),
                "fraction": count / max(len(states), 1),
                "note": "reachable operator states carrying this active transport identity",
            }
        )

    for spin, count in sorted(spin_counter.items(), key=lambda item: (item[0].horizon, _spin_str(item[0]))):
        rows.append(
            {
                "scope": "spin_distribution",
                "metric": f"spin_{_spin_str(spin)}",
                "count": count,
                "total": len(states),
                "fraction": count / max(len(states), 1),
                "note": "distribution of predictive spin component among reachable states",
            }
        )

    for class_key, count in sorted(class_counter.items(), key=lambda item: (item[0][1], item[0][0], item[0][2])):
        b, r, spin_word = class_key
        rows.append(
            {
                "scope": "class_distribution",
                "metric": f"class_b{b}_r{r}_spin{spin_word}",
                "count": count,
                "total": len(states),
                "fraction": count / max(len(states), 1),
                "note": "distribution of active transport identities by transport-side class coordinates",
            }
        )

    return rows


def write_spinH_candidate_v2(
    rows: list[dict[str, object]],
    output_path: Path = OUTPUT_PATH_SPINH_CANDIDATE_V2,
) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    with output_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=["scope", "metric", "count", "total", "fraction", "note"],
        )
        writer.writeheader()
        writer.writerows(rows)


if __name__ == "__main__":
    summary_rows = summarize_spinH_candidate_v2()
    write_spinH_candidate_v2(summary_rows, OUTPUT_PATH_SPINH_CANDIDATE_V2)
    print(f"wrote {OUTPUT_PATH_SPINH_CANDIDATE_V2}")
    for row in summary_rows:
        if row["scope"] in {"summary", "representation"}:
            print(
                f"{row['scope']}:{row['metric']} count={row['count']} "
                f"total={row['total']} fraction={float(row['fraction']):.4f}"
            )
