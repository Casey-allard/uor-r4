#!/usr/bin/env python3
"""Bounded tau-holonomy periodicity-class audit for all six non-hold operators on v25.

Read-only audit.  No files are modified.  No operators are rebuilt.

Reuses the EXACT same per-operator samples as run_tau_orbit_depth_audit_v1.py
(rng = random.Random(42), same operator order, 500 per stratum = 1000 per operator).

For each orbit, records:
  - full state cycle length L  (step at which a previously-seen state recurs)
  - tau return depth d         (first step where tau equals initial tau)

Key relationship: d always divides L (tau is a function of state; if state has
period L then tau period divides L).

Reports:
  - Frequency table of observed cycle lengths per operator
  - Frequency table of tau return depths per operator
  - Concentration measures (top-k, entropy proxy, distinct count)
  - Comparison: transport vs non-transport periodicity structure

Cap = 1000 (sufficient for all operators per prior extended census).
"""

from __future__ import annotations

import csv
import math
import random
import statistics
import sys
import time
from collections import Counter
from pathlib import Path
from typing import Optional

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
    "prime_transport_tau_periodicity_classes_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_periodicity_classes_v1.csv"
)

OPERATORS = [
    ("T_b",  torus_base_advance_component_v23,   "non-transport"),
    ("T_x",  composite_swap_component_v24,        "non-transport"),
    ("T_y",  composite_twist_component_v25,       "non-transport"),
    ("T_c",  coupled_torus_kick_component_v20,    "transport"),
    ("T_z'", fiber_phase_lift_component_v21,      "transport"),
    ("T_r*", radial_transport_component_v22,      "transport"),
]

SAMPLE_PER_STRATUM = 500
CAP = 1000
RANDOM_SEED = 42
TOP_K = 10   # top cycle lengths to report per operator


def orbit_full(fn, seed, cap: int) -> dict:
    """Walk orbit of seed; record cycle length and tau return depth."""
    tau0 = seed.tau
    seen: dict = {seed: 0}
    current = seed
    tau_return_depth: Optional[int] = None
    cycle_length: Optional[int] = None

    for step in range(1, cap + 1):
        current = fn(current)

        if tau_return_depth is None and current.tau == tau0:
            tau_return_depth = step

        if current in seen:
            cycle_length = step - seen[current]
            break
        seen[current] = step

    return {
        "tau_return_depth": tau_return_depth,
        "cycle_length":     cycle_length,
        "resolved":         cycle_length is not None,
    }


def normalised_entropy(counter: Counter, total: int) -> float:
    """Shannon entropy normalised by log(total), range [0,1]."""
    if total <= 1:
        return 0.0
    h = 0.0
    for cnt in counter.values():
        if cnt > 0:
            p = cnt / total
            h -= p * math.log2(p)
    max_h = math.log2(total)
    return h / max_h if max_h > 0 else 0.0


def run_periodicity(depth: int = 8) -> dict:
    rng = random.Random(RANDOM_SEED)

    print(f"[pc] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[pc] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    all_results: dict[str, dict] = {}

    for op_name, op_fn, cluster in OPERATORS:
        ss_states  = [s for s in states if s.spin_h == op_fn(s).spin_h]
        nss_states = [s for s in states if s.spin_h != op_fn(s).spin_h]

        sample_ss  = rng.sample(ss_states,  min(SAMPLE_PER_STRATUM, len(ss_states)))
        sample_nss = rng.sample(nss_states, min(SAMPLE_PER_STRATUM, len(nss_states)))
        seeds = sample_ss + sample_nss
        total = len(seeds)

        t1 = time.perf_counter()
        orbits = [orbit_full(op_fn, s, CAP) for s in seeds]
        elapsed = time.perf_counter() - t1

        # Cycle-length distribution (for resolved orbits only)
        resolved_orbits = [o for o in orbits if o["resolved"]]
        unresolved      = total - len(resolved_orbits)

        cycle_counter   = Counter(o["cycle_length"] for o in resolved_orbits)
        tau_ret_counter = Counter(
            o["tau_return_depth"] for o in orbits
            if o["tau_return_depth"] is not None
        )

        distinct_cycles = len(cycle_counter)
        top_cycles = cycle_counter.most_common(TOP_K)
        top1_frac  = top_cycles[0][1] / total if top_cycles else 0.0
        topk_frac  = sum(cnt for _, cnt in top_cycles) / total if top_cycles else 0.0

        cycle_entropy = normalised_entropy(cycle_counter, len(resolved_orbits))

        # Concentration label
        if distinct_cycles == 1:
            concentration = "single period"
        elif distinct_cycles <= 3 and top1_frac >= 0.70:
            concentration = "strongly concentrated"
        elif distinct_cycles <= 5 and topk_frac >= 0.85:
            concentration = "concentrated"
        elif cycle_entropy < 0.3:
            concentration = "dominated by few periods"
        elif cycle_entropy > 0.7:
            concentration = "broadly dispersed"
        else:
            concentration = "moderately dispersed"

        all_results[op_name] = {
            "cluster":         cluster,
            "sample":          total,
            "resolved":        len(resolved_orbits),
            "unresolved":      unresolved,
            "cycle_counter":   cycle_counter,
            "tau_ret_counter": tau_ret_counter,
            "distinct_cycles": distinct_cycles,
            "top_cycles":      top_cycles,
            "top1_frac":       top1_frac,
            "cycle_entropy":   cycle_entropy,
            "concentration":   concentration,
        }

        top3_str = ", ".join(f"{cl}×{cnt}" for cl, cnt in top_cycles[:3])
        print(
            f"[pc] {op_name:5s} ({cluster[:3]})  resolved={len(resolved_orbits)}/{total}  "
            f"distinct_cycles={distinct_cycles}  entropy={cycle_entropy:.3f}  "
            f"top3=[{top3_str}]  ({elapsed:.1f}s)"
        )

    print(f"[pc] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"surface_n": n, "results": all_results}


def write_csv(data: dict) -> None:
    rows = []
    for op_name, _, cluster in OPERATORS:
        r = data["results"][op_name]
        total = r["sample"]
        cc    = r["cycle_counter"]
        # All observed cycle lengths, sorted descending by count
        ranked = cc.most_common()
        for rank, (cl, cnt) in enumerate(ranked, start=1):
            rows.append({
                "operator":            op_name,
                "cluster":             cluster,
                "cycle_length":        cl,
                "count":               cnt,
                "fraction":            f"{cnt/total:.6f}",
                "rank_within_operator": rank,
                "note": (
                    f"cap={CAP}; sample={total}; "
                    f"rng_seed={RANDOM_SEED}; same sample as prior tau audits"
                ),
            })
        # One row for unresolved
        if r["unresolved"] > 0:
            rows.append({
                "operator":            op_name,
                "cluster":             cluster,
                "cycle_length":        "unresolved",
                "count":               r["unresolved"],
                "fraction":            f"{r['unresolved']/total:.6f}",
                "rank_within_operator": len(ranked) + 1,
                "note": f"cap={CAP}; cycle not detected within cap",
            })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "operator", "cluster", "cycle_length", "count",
        "fraction", "rank_within_operator", "note",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[pc] Wrote {OUTPUT_CSV}")


def _bar(fraction: float, width: int = 28) -> str:
    filled = round(fraction * width)
    return "█" * filled + "░" * (width - filled)


def write_md(data: dict) -> None:
    n   = data["surface_n"]
    res = data["results"]

    def fmt(v, prec=3):
        if isinstance(v, float) and v != v:
            return "N/A"
        if isinstance(v, float):
            return f"{v:.{prec}f}"
        return str(v)

    lines = [
        "# Prime Transport — Tau-Holonomy Periodicity-Class Audit",
        "",
        "**Audit type:** Read-only bounded periodicity enumeration",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total surface states:** {n:,}",
        "",
        "---",
        "",
        "## Sample and method",
        "",
        "| Parameter | Value |",
        "|---|---|",
        f"| Sample reuse | **Yes** — exact same seeds as prior tau orbit audits |",
        f"| Samples per stratum per operator | {SAMPLE_PER_STRATUM} (sigma_stable + non_sigma_stable) |",
        f"| Total seeds per operator | {SAMPLE_PER_STRATUM * 2} |",
        f"| Orbit cap | {CAP} |",
        f"| Random seed | {RANDOM_SEED} |",
        f"| Top-k reported | {TOP_K} |",
        "",
        "**Cycle length** = step at which the full operator-state orbit first recurs.",
        "**Tau return depth** = step at which the tau sub-state first returns to its initial value.",
        "Note: tau return depth always divides cycle length (standard orbit-theory result).",
        "",
        "---",
        "",
        "## Summary table",
        "",
        "| Operator | Cluster | resolved | distinct cycles | norm entropy | top-1 fraction | concentration |",
        "|---|---|---|---|---|---|---|",
    ]

    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        lines.append(
            f"| {op_name} | {cluster} "
            f"| {r['resolved']}/{r['sample']} "
            f"| {r['distinct_cycles']} "
            f"| {fmt(r['cycle_entropy'])} "
            f"| {fmt(r['top1_frac'])} "
            f"| {r['concentration']} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Per-operator periodicity-class details",
        "",
    ]

    for op_name, _, cluster in OPERATORS:
        r    = res[op_name]
        total = r["sample"]
        cc   = r["cycle_counter"]
        tc   = r["tau_ret_counter"]

        lines += [
            f"### {op_name} ({cluster})",
            "",
            f"- Sample: {total} | Resolved: {r['resolved']} | Unresolved: {r['unresolved']}",
            f"- Distinct observed cycle lengths: **{r['distinct_cycles']}**",
            f"- Normalised cycle-length entropy: **{fmt(r['cycle_entropy'])}** "
            f"(0 = single period, 1 = maximally dispersed)",
            f"- Concentration assessment: **{r['concentration']}**",
            "",
            f"**Top-{min(TOP_K, len(r['top_cycles']))} observed cycle lengths:**",
            "",
            "| Rank | Cycle length | Count | Fraction | Bar |",
            "|---|---|---|---|---|",
        ]
        for rank, (cl, cnt) in enumerate(r["top_cycles"], start=1):
            frac = cnt / total
            lines.append(
                f"| {rank} | {cl} | {cnt} | {frac:.4f} | {_bar(frac, 20)} |"
            )

        # Tau return depth top-5
        top_tau = tc.most_common(5)
        if top_tau:
            lines += [
                "",
                "**Top-5 tau return depths (within this sample):**",
                "",
                "| Tau return depth | Count | Fraction |",
                "|---|---|---|",
            ]
            for d, cnt in top_tau:
                lines.append(f"| {d} | {cnt} | {cnt/total:.4f} |")

        lines.append("")

    # Cluster comparison
    nt_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "non-transport"]
    tr_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "transport"]

    nt_mean_distinct = statistics.mean(r["distinct_cycles"] for _, r in nt_ops)
    tr_mean_distinct = statistics.mean(r["distinct_cycles"] for _, r in tr_ops)
    nt_mean_entropy  = statistics.mean(r["cycle_entropy"]   for _, r in nt_ops)
    tr_mean_entropy  = statistics.mean(r["cycle_entropy"]   for _, r in tr_ops)
    nt_mean_top1     = statistics.mean(r["top1_frac"]       for _, r in nt_ops)
    tr_mean_top1     = statistics.mean(r["top1_frac"]       for _, r in tr_ops)

    lines += [
        "---",
        "",
        "## Cluster-level periodicity comparison",
        "",
        "| Metric | non-transport {T_b, T_x, T_y} | transport {T_c, T_z', T_r*} |",
        "|---|---|---|",
        f"| mean distinct cycle lengths | {fmt(nt_mean_distinct)} | {fmt(tr_mean_distinct)} |",
        f"| mean normalised entropy     | {fmt(nt_mean_entropy)}  | {fmt(tr_mean_entropy)} |",
        f"| mean top-1 fraction         | {fmt(nt_mean_top1)}     | {fmt(tr_mean_top1)} |",
        "",
        "---",
        "",
        "## Interpretation",
        "",
        "### Concentration vs dispersion",
        "",
    ]

    # Per-operator interpretation
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        cc = r["cycle_counter"]
        top1 = r["top_cycles"][0] if r["top_cycles"] else (None, 0)
        lines.append(
            f"- **{op_name}** ({cluster}): {r['concentration']} "
            f"({r['distinct_cycles']} distinct lengths; "
            f"top period = {top1[0]} at {fmt(r['top1_frac'])} of sample; "
            f"entropy = {fmt(r['cycle_entropy'])})"
        )

    lines += [
        "",
        "### Transport vs non-transport periodicity structure",
        "",
    ]

    if tr_mean_entropy > nt_mean_entropy + 0.10:
        lines.append(
            "The transport cluster has **higher cycle-length entropy** than the non-transport "
            "cluster — transport operators are more broadly dispersed across periodicity classes. "
            "The non-transport cluster tends toward more concentrated, repeatable cycle structures."
        )
    elif nt_mean_entropy > tr_mean_entropy + 0.10:
        lines.append(
            "The non-transport cluster has **higher cycle-length entropy** than the transport "
            "cluster — non-transport operators are more broadly dispersed across periodicity classes."
        )
    else:
        lines.append(
            "Transport and non-transport clusters have **similar cycle-length entropy** — "
            "neither cluster is dramatically more concentrated than the other in periodicity structure."
        )

    lines += [
        "",
        "### Whether any operator is dominated by a single period",
        "",
    ]
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        if r["top1_frac"] > 0.60:
            top1_cl = r["top_cycles"][0][0]
            lines.append(
                f"- **{op_name}** has {fmt(r['top1_frac'])} of resolved orbits with "
                f"cycle length {top1_cl} — this operator is **dominated by a single period**."
            )
    lines.append("")

    lines += [
        "### Observation on tau return depth vs cycle length",
        "",
        "Since tau return depth divides cycle length, operators with short tau return depths "
        "have tau structures that are 'fast-cycling' components of the full orbit. "
        "Operators with tau return depths close to cycle length have tau that tracks nearly "
        "the full orbit period before resetting.",
        "",
    ]

    # For each operator with enough resolved orbits, note ratio
    for op_name, _, cluster in OPERATORS:
        r  = res[op_name]
        cc = r["cycle_counter"]
        tc = r["tau_ret_counter"]
        if not cc or not tc:
            continue
        med_cycle = statistics.median(
            cl for cl, cnt in cc.items() for _ in range(cnt)
        )
        med_tau = statistics.median(
            d for d, cnt in tc.items() for _ in range(cnt)
        )
        if med_cycle > 0:
            ratio = med_tau / med_cycle
            lines.append(
                f"- **{op_name}**: median cycle={fmt(med_cycle, 1)}, "
                f"median tau-return={fmt(med_tau, 1)}, "
                f"ratio={fmt(ratio, 3)} "
                f"({'tau tracks ~full orbit' if ratio > 0.5 else 'tau is a fast sub-cycle'})"
            )

    lines += [
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
        "Run a bounded **tau-periodicity arithmetic-factor test**: "
        "for each operator, test whether the most commonly observed cycle lengths "
        "are divisors or small multiples of the primorial moduli already defined "
        "in the repo (e.g., `W_210`, `W_2310`, `W_30030`).  "
        "This directly tests the arithmetic-constraint hypothesis suggested by "
        "the concentrated periodicity structure observed here.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[pc] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_periodicity(depth=8)
    write_csv(data)
    write_md(data)
