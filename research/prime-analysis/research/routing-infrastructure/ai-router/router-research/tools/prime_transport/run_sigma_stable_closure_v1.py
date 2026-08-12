#!/usr/bin/env python3
"""Bounded sigma-stable sub-algebra closure test on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

For each transport operator T_src in {T_c, T_z', T_r*}:
  Compute the sigma-stable subset SS(T_src) = { s : spin_h(T_src(s)) == spin_h(s) }

  For each non-transport operator N_act in {T_b, T_x, T_y}:
    For each s in SS(T_src):
      s' = N_act(s)
      Check whether s' is still sigma-stable under T_src:
        spin_h(T_src(s')) == spin_h(s')
    Report preserved_count / |SS(T_src)|

Surface: geometry_native_operator_model_v25.bounded_operator_surface_v25, depth=8.
"""

from __future__ import annotations

import csv
import sys
import time
from pathlib import Path

_root = Path(__file__).resolve().parent
if str(_root) not in sys.path:
    sys.path.insert(0, str(_root))

from geometry_native_operator_model_v20 import coupled_torus_kick_component_v20
from geometry_native_operator_model_v21 import fiber_phase_lift_component_v21
from geometry_native_operator_model_v22 import radial_transport_component_v22
from geometry_native_operator_model_v23 import torus_base_advance_component_v23
from geometry_native_operator_model_v24 import composite_swap_component_v24
from geometry_native_operator_model_v25 import (
    bounded_operator_surface_v25,
    composite_twist_component_v25,
)

OUTPUT_MD = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research/"
    "prime_transport_sigma_stable_closure_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_sigma_stable_closure_v1.csv"
)

TRANSPORT_OPS = [
    ("T_c",  coupled_torus_kick_component_v20),
    ("T_z'", fiber_phase_lift_component_v21),
    ("T_r*", radial_transport_component_v22),
]

NONTRANSPORT_OPS = [
    ("T_b", torus_base_advance_component_v23),
    ("T_x", composite_swap_component_v24),
    ("T_y", composite_twist_component_v25),
]

SIGMA_EQ = lambda s, t: s.spin_h == t.spin_h


def run_closure(depth: int = 8) -> dict:
    print(f"[cl] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[cl] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    # Step 1: for each transport op, precompute sigma-stable subset
    print(f"[cl] Precomputing sigma-stable subsets for transport operators ...")
    t1 = time.perf_counter()
    ss: dict[str, list] = {}
    for t_name, t_fn in TRANSPORT_OPS:
        stable = []
        for s in states:
            ts = t_fn(s)
            if SIGMA_EQ(s, ts):
                stable.append(s)
        ss[t_name] = stable
        print(f"[cl]   {t_name}: {len(stable)} sigma-stable states ({len(stable)/n:.6f})")
    print(f"[cl] Subset build: {time.perf_counter()-t1:.1f}s")

    # Step 2: for each (T_src, N_act), measure preservation
    results = []
    for t_name, t_fn in TRANSPORT_OPS:
        subset = ss[t_name]
        ss_size = len(subset)
        for n_name, n_fn in NONTRANSPORT_OPS:
            t2 = time.perf_counter()
            preserved = 0
            for s in subset:
                s_prime = n_fn(s)          # apply non-transport
                ts_prime = t_fn(s_prime)   # re-apply transport source to moved state
                if SIGMA_EQ(s_prime, ts_prime):
                    preserved += 1
            frac = preserved / ss_size if ss_size > 0 else float("nan")
            elapsed = time.perf_counter() - t2
            print(
                f"[cl]   {t_name} | {n_name}: "
                f"preserved={preserved}/{ss_size} ({frac:.6f})  ({elapsed:.1f}s)"
            )
            results.append({
                "transport_source":             t_name,
                "nontransport_actor":           n_name,
                "source_sigma_stable_count":    ss_size,
                "preserved_sigma_stable_count": preserved,
                "preserved_sigma_stable_fraction": frac,
            })

    print(f"[cl] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"states": n, "ss_sizes": {k: len(v) for k, v in ss.items()}, "results": results}


def _verdict(frac: float) -> str:
    if frac >= 0.90:
        return "near-closed"
    elif frac >= 0.50:
        return "partially-preserved"
    elif frac >= 0.10:
        return "mostly-disrupted"
    else:
        return "disrupted"


def write_csv(data: dict) -> None:
    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "transport_source",
        "nontransport_actor",
        "source_sigma_stable_count",
        "preserved_sigma_stable_count",
        "preserved_sigma_stable_fraction",
        "closure_verdict",
        "note",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        for row in data["results"]:
            frac = row["preserved_sigma_stable_fraction"]
            writer.writerow({
                **row,
                "preserved_sigma_stable_fraction": f"{frac:.6f}",
                "closure_verdict": _verdict(frac),
                "note": (
                    f"Fraction of {row['transport_source']} sigma-stable states "
                    f"that remain sigma-stable under {row['transport_source']} "
                    f"after {row['nontransport_actor']} is applied"
                ),
            })
    print(f"[cl] Wrote {OUTPUT_CSV}")


def write_md(data: dict) -> None:
    n = data["states"]
    ss_sizes = data["ss_sizes"]
    results = data["results"]

    # Index results for easy lookup
    idx: dict[tuple, dict] = {
        (r["transport_source"], r["nontransport_actor"]): r
        for r in results
    }

    def frac(tsrc, nact) -> float:
        return idx[(tsrc, nact)]["preserved_sigma_stable_fraction"]

    def pct(tsrc, nact) -> str:
        return f"{frac(tsrc, nact)*100:.2f}%"

    lines = [
        "# Prime Transport — Sigma-Stable Sub-Algebra Closure Test",
        "",
        "**Audit type:** Read-only closure characterization",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total states on surface:** {n:,}",
        "",
        "---",
        "",
        "## Setup",
        "",
        "For each transport operator `T_src ∈ {T_c, T_z', T_r*}`:",
        "",
        "1. Compute the **sigma-stable subset** `SS(T_src) = { s : spin_h(T_src(s)) == spin_h(s) }`",
        "2. For each non-transport operator `N_act ∈ {T_b, T_x, T_y}`:",
        "   - Apply `N_act` to each `s ∈ SS(T_src)` to get `s' = N_act(s)`",
        "   - Check whether `s'` is **still sigma-stable under T_src**: `spin_h(T_src(s')) == spin_h(s')`",
        "   - Report `preserved_count / |SS(T_src)|`",
        "",
        "---",
        "",
        "## Sigma-stable subset sizes (from prior audit)",
        "",
        "| Transport operator | sigma-stable count | sigma-stable fraction |",
        "|---|---|---|",
    ]

    for t_name, _ in TRANSPORT_OPS:
        sz = ss_sizes[t_name]
        lines.append(f"| {t_name} | {sz:,} | {sz/n:.6f} |")

    lines += [
        "",
        "---",
        "",
        "## Closure preservation fractions",
        "",
        "Each cell shows the fraction of `SS(T_src)` states that remain sigma-stable",
        "under `T_src` after `N_act` has been applied.",
        "",
        "| Transport source | N_act = T_b | N_act = T_x | N_act = T_y |",
        "|---|---|---|---|",
    ]

    for t_name, _ in TRANSPORT_OPS:
        row_str = f"| {t_name}"
        for n_name, _ in NONTRANSPORT_OPS:
            f = frac(t_name, n_name)
            v = _verdict(f)
            row_str += f" | {f:.6f} ({v})"
        row_str += " |"
        lines.append(row_str)

    # Column means
    for n_name, _ in NONTRANSPORT_OPS:
        col_mean = sum(frac(t, n_name) for t, _ in TRANSPORT_OPS) / 3
        pass  # will use in prose

    lines += [
        "",
        "---",
        "",
        "## Per-source breakdown",
        "",
    ]

    for t_name, _ in TRANSPORT_OPS:
        sz = ss_sizes[t_name]
        lines += [
            f"### T_src = {t_name}  (|SS| = {sz:,})",
            "",
            f"| N_act | preserved | fraction | verdict |",
            "|---|---|---|---|",
        ]
        for n_name, _ in NONTRANSPORT_OPS:
            r = idx[(t_name, n_name)]
            f = r["preserved_sigma_stable_fraction"]
            lines.append(
                f"| {n_name} | {r['preserved_sigma_stable_count']:,} "
                f"| {f:.6f} | {_verdict(f)} |"
            )
        lines.append("")

    lines += [
        "---",
        "",
        "## Interpretation",
        "",
    ]

    # Compute column means per non-transport op
    col_means = {}
    for n_name, _ in NONTRANSPORT_OPS:
        col_means[n_name] = sum(frac(t, n_name) for t, _ in TRANSPORT_OPS) / 3

    best_actor = max(col_means, key=lambda k: col_means[k])
    worst_actor = min(col_means, key=lambda k: col_means[k])

    lines += [
        "### Which non-transport operator best preserves transport sigma-stability?",
        "",
        f"Mean preservation fractions across all three transport sources:",
        "",
        "| N_act | mean preservation fraction |",
        "|---|---|",
    ]
    for n_name, _ in NONTRANSPORT_OPS:
        lines.append(f"| {n_name} | {col_means[n_name]:.6f} |")

    lines += [
        "",
        f"**{best_actor}** has the highest mean preservation ({col_means[best_actor]:.6f}).",
        f"**{worst_actor}** has the lowest mean preservation ({col_means[worst_actor]:.6f}).",
        "",
        "### Bridge interpretation of T_b",
        "",
    ]

    tb_mean = col_means["T_b"]
    tx_mean = col_means["T_x"]
    ty_mean = col_means["T_y"]

    if tb_mean > tx_mean and tb_mean > ty_mean:
        lines += [
            f"**T_b preserves transport sigma-stability better than both T_x and T_y** "
            f"(mean {tb_mean:.6f} vs T_x {tx_mean:.6f} vs T_y {ty_mean:.6f}).",
            "This is consistent with the bridge interpretation established in the fixed-point",
            "census and sigma-stability audit: T_b shares structural properties with the",
            "transport cluster despite being labeled non-transport.",
        ]
    elif tb_mean > tx_mean or tb_mean > ty_mean:
        worse = "T_x" if tb_mean <= tx_mean else "T_y"
        lines += [
            f"**T_b partially supports the bridge interpretation**: it preserves transport",
            f"sigma-stability better than {worse} (mean {tb_mean:.6f}), but not the best overall.",
        ]
    else:
        lines += [
            f"**T_b does not preserve transport sigma-stability better than both T_x and T_y**",
            f"(mean {tb_mean:.6f}). The closure test does not support the bridge interpretation",
            "at this level of analysis.",
        ]

    lines += [
        "",
        "### Coherence of transport sigma-stable family under non-transport action",
        "",
    ]

    # Check if the three transport subsets respond consistently to each NT op
    all_near_closed = all(
        frac(t, n) >= 0.90
        for t, _ in TRANSPORT_OPS
        for n, _ in NONTRANSPORT_OPS
    )
    all_disrupted = all(
        frac(t, n) < 0.20
        for t, _ in TRANSPORT_OPS
        for n, _ in NONTRANSPORT_OPS
    )
    # Check per-actor consistency
    actor_consistent = {}
    for n_name, _ in NONTRANSPORT_OPS:
        fracs = [frac(t, n_name) for t, _ in TRANSPORT_OPS]
        actor_consistent[n_name] = max(fracs) - min(fracs) < 0.05

    if all_near_closed:
        lines.append(
            "All nine (T_src, N_act) pairs show near-closed preservation (≥90%). "
            "The transport sigma-stable subsets behave as a coherent invariant family "
            "under the full non-transport layer."
        )
    elif all_disrupted:
        lines.append(
            "All nine pairs show disruption (<20% preservation). "
            "The transport sigma-stable subsets are not preserved by the non-transport layer."
        )
    else:
        lines.append(
            "Preservation varies by operator pair. See the table above for details. "
            "The transport sigma-stable family is neither globally closed nor globally "
            "disrupted under non-transport action."
        )

    lines += [
        "",
        "Per-actor consistency (max–min range across three transport sources < 0.05):",
        "",
        "| N_act | consistent across sources? | range |",
        "|---|---|---|",
    ]
    for n_name, _ in NONTRANSPORT_OPS:
        fracs_list = [frac(t, n_name) for t, _ in TRANSPORT_OPS]
        rng = max(fracs_list) - min(fracs_list)
        lines.append(
            f"| {n_name} | {'yes' if actor_consistent[n_name] else 'no'} | {rng:.6f} |"
        )

    lines += [
        "",
        "### What this means for the rebuilt algebra",
        "",
        "The closure fraction measures how 'porous' the boundary of the transport",
        "sigma-stable subsets is to non-transport action.  A high preservation rate",
        "indicates that non-transport operators tend to map sigma-stable transport",
        "states to other sigma-stable transport states (approximate invariant subspace).",
        "A low rate indicates that non-transport operators scatter sigma-stable transport",
        "states out of the invariant subspace, breaking the sigma-carrier structure.",
        "",
        "---",
        "",
        "## Honesty section",
        "",
        "| Question | Answer |",
        "|---|---|",
        "| Were any files modified? | **No** |",
        "| Were any operators rebuilt in this step? | **No** |",
        "| Is full exact spin_H solved? | **No** |",
        "",
        "---",
        "",
        "## Recommended next step",
        "",
        "Run a bounded **tau-holonomy orbit depth audit**: for each operator, compute the",
        "length of the orbit under repeated application until the tau state returns to its",
        "initial configuration (or a cycle is detected).  This characterises the",
        "tau-cycle structure of each operator and tests whether transport operators have",
        "systematically longer tau orbits than non-transport operators — the natural",
        "follow-on to the finding that tau motion persists even within sigma-stable states.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[cl] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_closure(depth=8)
    write_csv(data)
    write_md(data)
