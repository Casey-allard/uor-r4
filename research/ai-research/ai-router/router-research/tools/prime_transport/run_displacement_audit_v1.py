#!/usr/bin/env python3
"""Bounded displacement and divergence audit on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

For each operator O in {T_b, T_x, T_y, T_c, T_z', T_r*}:
  - Per state s: displacement(s, O) = number of the 10 top-level fields
    that differ between s and O(s).
    Fields: b, phi, r, composite_compat_class, query_semiprime,
            binding_semiprime, admissible_transition, twist, spin_h, tau.
  - Stats: avg, median, max, quartiles over all surface states.

For each unordered pair {O_i, O_j}:
  - image_i = set of O_i(s) for s in surface
  - image_j = set of O_j(s) for s in surface
  - overlap_fraction = |image_i ∩ image_j| / N
  - divergence = 1 - overlap_fraction
  - distinct_output_count reported per operator.

Surface: geometry_native_operator_model_v25.bounded_operator_surface_v25, depth=8.
"""

from __future__ import annotations

import csv
import statistics
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
    "prime_transport_displacement_audit_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_displacement_audit_v1.csv"
)

OPERATORS = [
    ("T_b",  torus_base_advance_component_v23,   "non-transport"),
    ("T_x",  composite_swap_component_v24,        "non-transport"),
    ("T_y",  composite_twist_component_v25,       "non-transport"),
    ("T_c",  coupled_torus_kick_component_v20,    "transport"),
    ("T_z'", fiber_phase_lift_component_v21,      "transport"),
    ("T_r*", radial_transport_component_v22,      "transport"),
]

MAX_FIELDS = 10  # b, phi, r, composite_compat_class, query_semiprime,
                 # binding_semiprime, admissible_transition, twist, spin_h, tau


def _displacement(s, t) -> int:
    """Count top-level fields that differ between source s and target t."""
    count = 0
    if s.b != t.b: count += 1
    if s.phi != t.phi: count += 1
    if s.r != t.r: count += 1
    if s.composite_compat_class != t.composite_compat_class: count += 1
    if s.query_semiprime != t.query_semiprime: count += 1
    if s.binding_semiprime != t.binding_semiprime: count += 1
    if s.admissible_transition != t.admissible_transition: count += 1
    if s.twist != t.twist: count += 1
    if s.spin_h != t.spin_h: count += 1
    if s.tau != t.tau: count += 1
    return count


def _quartiles(data: list[int]) -> dict:
    n = len(data)
    s = sorted(data)
    return {
        "q1":  s[n // 4],
        "q2":  s[n // 2],
        "q3":  s[3 * n // 4],
        "p10": s[n // 10],
        "p90": s[9 * n // 10],
    }


def _hist(data: list[int], bins: int = 11) -> dict[int, int]:
    counts: dict[int, int] = {}
    for v in data:
        counts[v] = counts.get(v, 0) + 1
    return dict(sorted(counts.items()))


def run_audit(depth: int = 8) -> dict:
    print(f"[disp] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[disp] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    # Single pass: compute all operator outputs and displacements
    print("[disp] Computing operator images and displacements ...")
    t1 = time.perf_counter()

    op_images: dict[str, set] = {name: set() for name, _, _ in OPERATORS}
    op_disps:  dict[str, list[int]] = {name: [] for name, _, _ in OPERATORS}

    for s in states:
        for name, fn, _ in OPERATORS:
            t = fn(s)
            op_images[name].add(t)
            op_disps[name].append(_displacement(s, t))

    print(f"[disp] Images and displacements done ({time.perf_counter()-t1:.1f}s)")

    # Per-operator stats
    op_stats: dict[str, dict] = {}
    for name, _, cluster in OPERATORS:
        d = op_disps[name]
        q = _quartiles(d)
        h = _hist(d)
        op_stats[name] = {
            "cluster": cluster,
            "n": n,
            "avg": statistics.mean(d),
            "median": statistics.median(d),
            "max": max(d),
            "min": min(d),
            "stdev": statistics.stdev(d),
            "q1": q["q1"], "q2": q["q2"], "q3": q["q3"],
            "p10": q["p10"], "p90": q["p90"],
            "hist": h,
            "distinct_output_count": len(op_images[name]),
            "distinct_fraction": len(op_images[name]) / n,
        }
        print(
            f"[disp] {name:5s}  avg={op_stats[name]['avg']:.4f}  "
            f"median={op_stats[name]['median']:.1f}  max={op_stats[name]['max']}  "
            f"distinct={op_stats[name]['distinct_output_count']:,}  "
            f"hist={h}"
        )

    # Pairwise image overlaps
    print("[disp] Computing pairwise image overlaps ...")
    t2 = time.perf_counter()
    names = [name for name, _, _ in OPERATORS]
    pair_stats: list[dict] = []
    for i, ni in enumerate(names):
        for j, nj in enumerate(names):
            if j <= i:
                continue
            overlap = len(op_images[ni] & op_images[nj])
            union   = len(op_images[ni] | op_images[nj])
            jaccard = overlap / union if union > 0 else 0.0
            overlap_frac = overlap / n
            divergence = 1.0 - overlap_frac
            pair_stats.append({
                "op_i": ni, "op_j": nj,
                "image_i_size": len(op_images[ni]),
                "image_j_size": len(op_images[nj]),
                "overlap": overlap,
                "overlap_fraction": overlap_frac,
                "jaccard": jaccard,
                "divergence": divergence,
                "union": union,
            })
            print(
                f"[disp] {ni:5s} × {nj:5s}  "
                f"overlap={overlap:,}  frac={overlap_frac:.4f}  "
                f"jaccard={jaccard:.4f}  div={divergence:.4f}"
            )

    print(f"[disp] Pairwise done ({time.perf_counter()-t2:.1f}s)")
    print(f"[disp] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"states": n, "op_stats": op_stats, "pair_stats": pair_stats}


def write_csv(result: dict) -> None:
    n = result["states"]
    rows = []

    # Per-operator rows
    for name, _, cluster in OPERATORS:
        st = result["op_stats"][name]
        rows.append({
            "row_type": "operator",
            "operator": name,
            "cluster": cluster,
            "avg_displacement": f"{st['avg']:.6f}",
            "median_displacement": f"{st['median']:.1f}",
            "max_displacement": st["max"],
            "min_displacement": st["min"],
            "stdev_displacement": f"{st['stdev']:.6f}",
            "q1_displacement": st["q1"],
            "q3_displacement": st["q3"],
            "p10_displacement": st["p10"],
            "p90_displacement": st["p90"],
            "distinct_output_count": st["distinct_output_count"],
            "distinct_output_fraction": f"{st['distinct_fraction']:.6f}",
            "displacement_hist": str(st["hist"]),
            "operator_left": "",
            "operator_right": "",
            "overlap_fraction": "",
            "divergence": "",
            "jaccard": "",
            "overlap_count": "",
        })

    # Pairwise rows
    for p in result["pair_stats"]:
        rows.append({
            "row_type": "pair",
            "operator": "",
            "cluster": "",
            "avg_displacement": "",
            "median_displacement": "",
            "max_displacement": "",
            "min_displacement": "",
            "stdev_displacement": "",
            "q1_displacement": "",
            "q3_displacement": "",
            "p10_displacement": "",
            "p90_displacement": "",
            "distinct_output_count": p["image_i_size"],
            "distinct_output_fraction": "",
            "displacement_hist": "",
            "operator_left": p["op_i"],
            "operator_right": p["op_j"],
            "overlap_fraction": f"{p['overlap_fraction']:.6f}",
            "divergence": f"{p['divergence']:.6f}",
            "jaccard": f"{p['jaccard']:.6f}",
            "overlap_count": p["overlap"],
        })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "row_type", "operator", "cluster",
        "avg_displacement", "median_displacement", "max_displacement",
        "min_displacement", "stdev_displacement",
        "q1_displacement", "q3_displacement", "p10_displacement", "p90_displacement",
        "distinct_output_count", "distinct_output_fraction", "displacement_hist",
        "operator_left", "operator_right",
        "overlap_fraction", "divergence", "jaccard", "overlap_count",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[disp] Wrote {OUTPUT_CSV}")


def write_md(result: dict) -> None:
    n = result["states"]
    ops = result["op_stats"]
    pairs = result["pair_stats"]

    # Cluster averages
    nt_names = ["T_b", "T_x", "T_y"]
    tr_names = ["T_c", "T_z'", "T_r*"]
    nt_avg = statistics.mean(ops[k]["avg"] for k in nt_names)
    tr_avg = statistics.mean(ops[k]["avg"] for k in tr_names)
    nt_med = statistics.median(ops[k]["median"] for k in nt_names)
    tr_med = statistics.median(ops[k]["median"] for k in tr_names)
    nt_max = max(ops[k]["max"] for k in nt_names)
    tr_max = max(ops[k]["max"] for k in tr_names)
    nt_distinct_avg = statistics.mean(ops[k]["distinct_fraction"] for k in nt_names)
    tr_distinct_avg = statistics.mean(ops[k]["distinct_fraction"] for k in tr_names)

    # Pairwise cluster summaries
    nt_pairs = [p for p in pairs if p["op_i"] in nt_names and p["op_j"] in nt_names]
    tr_pairs = [p for p in pairs if p["op_i"] in tr_names and p["op_j"] in tr_names]
    cross_pairs = [p for p in pairs if
                   (p["op_i"] in nt_names) != (p["op_j"] in nt_names)]

    nt_ol_avg = statistics.mean(p["overlap_fraction"] for p in nt_pairs)
    tr_ol_avg = statistics.mean(p["overlap_fraction"] for p in tr_pairs)
    cross_ol_avg = statistics.mean(p["overlap_fraction"] for p in cross_pairs)
    nt_div_avg = statistics.mean(p["divergence"] for p in nt_pairs)
    tr_div_avg = statistics.mean(p["divergence"] for p in tr_pairs)

    transport_higher_disp = tr_avg > nt_avg
    transport_lower_overlap = tr_ol_avg < nt_ol_avg

    lines = [
        "# Prime Transport Displacement Audit v1",
        "",
        "## Purpose",
        "",
        "Measure per-operator state displacement and pairwise image-set divergence",
        "on the bounded v25 surface.  Determine whether transport operators produce",
        "systematically larger displacement and lower image-set overlap than",
        "non-transport operators.",
        "",
        "This is a read-only audit.  No files were modified.  No operators were rebuilt.",
        "",
        "---",
        "",
        "## Surface and operators",
        "",
        "| Item | Value |",
        "|------|-------|",
        "| Surface | `geometry_native_operator_model_v25.py`, depth=8 |",
        f"| State count (N) | {n:,} |",
        "| Displacement metric | count of top-level fields changed (max=10): "
        "`b`, `phi`, `r`, `composite_compat_class`, `query_semiprime`, "
        "`binding_semiprime`, `admissible_transition`, `twist`, `spin_h`, `tau` |",
        "",
        "---",
        "",
        "## Per-operator displacement summary",
        "",
        "| Operator | Cluster | Avg disp | Median | Max | Stdev | Q1 | Q3 | P10 | P90 | Distinct outputs |",
        "|----------|---------|----------|--------|-----|-------|----|----|-----|-----|-----------------|",
    ]

    for name, _, cluster in OPERATORS:
        st = ops[name]
        lines.append(
            f"| `{name}` | {cluster} "
            f"| {st['avg']:.4f} | {st['median']:.1f} | {st['max']} "
            f"| {st['stdev']:.4f} | {st['q1']} | {st['q3']} "
            f"| {st['p10']} | {st['p90']} "
            f"| {st['distinct_output_count']:,} ({100*st['distinct_fraction']:.2f}%) |"
        )

    lines += [
        "",
        "### Displacement histograms (displacement value → state count)",
        "",
    ]
    for name, _, cluster in OPERATORS:
        st = ops[name]
        hist_str = "  ".join(f"{k}:{v:,}" for k, v in st["hist"].items())
        lines.append(f"- **`{name}`** ({cluster}): {hist_str}")

    lines += [
        "",
        "---",
        "",
        "## Transport vs non-transport displacement comparison",
        "",
        "| Statistic | Non-transport mean | Transport mean | Transport higher? |",
        "|-----------|-------------------|----------------|-------------------|",
        f"| Avg displacement | {nt_avg:.4f} | {tr_avg:.4f} "
        f"| {'**yes**' if transport_higher_disp else '**no**'} |",
        f"| Median displacement | {nt_med:.1f} | {tr_med:.1f} "
        f"| {'**yes**' if tr_med > nt_med else '**no**'} |",
        f"| Max displacement | {nt_max} | {tr_max} "
        f"| {'**yes**' if tr_max > nt_max else '**no**'} |",
        f"| Distinct output fraction | {nt_distinct_avg:.4f} | {tr_distinct_avg:.4f} "
        f"| {'**yes**' if tr_distinct_avg > nt_distinct_avg else 'no (transport lower)'} |",
        "",
        "---",
        "",
        "## Pairwise image-set overlap and divergence",
        "",
        "**Overlap fraction** = |image_i ∩ image_j| / N.  "
        "**Divergence** = 1 − overlap_fraction.  "
        "**Jaccard** = |image_i ∩ image_j| / |image_i ∪ image_j|.",
        "",
        "### Non-transport intra-cluster pairs",
        "",
        "| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |",
        "|------|-------------|-------------|---------|--------------|------------|---------|",
    ]
    for p in nt_pairs:
        lines.append(
            f"| `{{{p['op_i']}, {p['op_j']}}}` "
            f"| {p['image_i_size']:,} | {p['image_j_size']:,} "
            f"| {p['overlap']:,} | {p['overlap_fraction']:.4f} "
            f"| {p['divergence']:.4f} | {p['jaccard']:.4f} |"
        )

    lines += [
        f"| **Non-transport mean** | — | — | — | **{nt_ol_avg:.4f}** | **{nt_div_avg:.4f}** | — |",
        "",
        "### Transport intra-cluster pairs",
        "",
        "| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |",
        "|------|-------------|-------------|---------|--------------|------------|---------|",
    ]
    for p in tr_pairs:
        lines.append(
            f"| `{{{p['op_i']}, {p['op_j']}}}` "
            f"| {p['image_i_size']:,} | {p['image_j_size']:,} "
            f"| {p['overlap']:,} | {p['overlap_fraction']:.4f} "
            f"| {p['divergence']:.4f} | {p['jaccard']:.4f} |"
        )
    lines += [
        f"| **Transport mean** | — | — | — | **{tr_ol_avg:.4f}** | **{tr_div_avg:.4f}** | — |",
        "",
        "### Cross-cluster pairs",
        "",
        "| Pair | Image i size | Image j size | Overlap | Overlap frac | Divergence | Jaccard |",
        "|------|-------------|-------------|---------|--------------|------------|---------|",
    ]
    for p in cross_pairs:
        lines.append(
            f"| `{{{p['op_i']}, {p['op_j']}}}` "
            f"| {p['image_i_size']:,} | {p['image_j_size']:,} "
            f"| {p['overlap']:,} | {p['overlap_fraction']:.4f} "
            f"| {p['divergence']:.4f} | {p['jaccard']:.4f} |"
        )
    lines += [
        f"| **Cross-cluster mean** | — | — | — | **{cross_ol_avg:.4f}** | **{1-cross_ol_avg:.4f}** | — |",
        "",
        "---",
        "",
        "## Conclusion: 'transport = fast displacement' hypothesis",
        "",
    ]

    if transport_higher_disp and transport_lower_overlap:
        hyp_verdict = (
            "**SUPPORTED.** Transport operators produce higher average displacement "
            f"({tr_avg:.4f} vs {nt_avg:.4f}) and lower intra-cluster image-set overlap "
            f"({tr_ol_avg:.4f} vs {nt_ol_avg:.4f}) compared to non-transport operators."
        )
    elif transport_higher_disp and not transport_lower_overlap:
        hyp_verdict = (
            "**PARTIALLY SUPPORTED.** Transport operators produce higher average displacement "
            f"({tr_avg:.4f} vs {nt_avg:.4f}) but do NOT show lower intra-cluster overlap "
            f"(transport overlap {tr_ol_avg:.4f}, non-transport {nt_ol_avg:.4f})."
        )
    elif not transport_higher_disp and transport_lower_overlap:
        hyp_verdict = (
            "**PARTIALLY SUPPORTED.** Transport operators produce lower intra-cluster overlap "
            f"({tr_ol_avg:.4f} vs {nt_ol_avg:.4f}) but do NOT show higher average displacement "
            f"(transport {tr_avg:.4f}, non-transport {nt_avg:.4f})."
        )
    else:
        hyp_verdict = (
            "**NOT SUPPORTED.** Transport operators show neither higher displacement "
            f"({tr_avg:.4f} vs {nt_avg:.4f}) nor lower intra-cluster overlap "
            f"({tr_ol_avg:.4f} vs {nt_ol_avg:.4f})."
        )

    lines += [
        hyp_verdict,
        "",
        "### Supporting detail",
        "",
        f"- Non-transport cluster mean displacement: **{nt_avg:.4f}** fields/state",
        f"- Transport cluster mean displacement: **{tr_avg:.4f}** fields/state",
        f"- Ratio (transport / non-transport): **{tr_avg/nt_avg:.2f}×**",
        f"- Non-transport intra-cluster mean overlap fraction: **{nt_ol_avg:.4f}**",
        f"- Transport intra-cluster mean overlap fraction: **{tr_ol_avg:.4f}**",
        f"- Cross-cluster mean overlap fraction: **{cross_ol_avg:.4f}**",
        "",
        "The displacement metric counts how many of the 10 top-level state fields",
        "differ between input and output for each operator application.  It is an",
        "integer in [0, 10]; a value of 0 means the operator acted as the identity",
        "on that state.",
        "",
        "---",
        "",
        "## Honesty section",
        "",
        "| Question | Answer |",
        "|----------|--------|",
        "| Were any files modified? | **no** — read-only audit on the accepted v25 surface |",
        "| Were any operators rebuilt? | **no** |",
        "| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |",
        "",
        "---",
        "",
        "## Next step",
        "",
        "Run a bounded fixed-point census on the v25 surface: for each operator,",
        "count and characterise the states `s` where `O(s) == s` (displacement = 0).",
        "Fixed points reveal the invariant sub-spaces of each operator and may",
        "identify structurally degenerate states that do not participate in transport.",
        "",
        "Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[disp] Wrote {OUTPUT_MD}")


def main(depth: int = 8) -> None:
    result = run_audit(depth=depth)
    write_csv(result)
    write_md(result)


if __name__ == "__main__":
    main()
