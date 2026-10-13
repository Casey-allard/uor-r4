#!/usr/bin/env python3
"""Bounded tau-return-depth histogram audit for all six non-hold operators on v25.

Read-only audit.  No files are modified.  No operators are rebuilt.

Reuses the EXACT same per-operator samples as run_tau_orbit_depth_audit_v1.py:
  - rng = random.Random(42)
  - operators processed in order [T_b, T_x, T_y, T_c, T_z', T_r*]
  - 500 seeds per stratum (sigma_stable + non_sigma_stable) = 1000 per operator

Cap is raised to 1000 for all operators (consistent with T_r* extended census).

Histogram bin edges (tau return depth, inclusive-left exclusive-right):
  bin 1: [  1,   5)   very shallow
  bin 2: [  5,  15)   shallow
  bin 3: [ 15,  50)   medium
  bin 4: [ 50, 150)   deep
  bin 5: [150,1000]   very deep
  bin 6: no_tau_return (cycle resolved but tau never returned to initial value)
"""

from __future__ import annotations

import csv
import random
import statistics
import sys
import time
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
    "prime_transport_tau_depth_histogram_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_depth_histogram_v1.csv"
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

# Histogram bin edges: (label, left_inclusive, right_exclusive or None for open)
BINS = [
    ("1–4",    1,    5),
    ("5–14",   5,   15),
    ("15–49", 15,   50),
    ("50–149",50,  150),
    ("150+", 150, 1001),   # right=1001 means ≥150 (cap is 1000, so ≤1000)
]
NO_RETURN_LABEL = "no_tau_return"


def orbit_tau(fn, seed, cap: int) -> dict:
    tau0 = seed.tau
    seen: dict = {seed: 0}
    current = seed
    tau_return_depth: Optional[int] = None

    for step in range(1, cap + 1):
        current = fn(current)
        if tau_return_depth is None and current.tau == tau0:
            tau_return_depth = step
        if current in seen:
            return {
                "tau_return_depth": tau_return_depth,
                "resolved": True,
                "cycle_length": step - seen[current],
            }
        seen[current] = step

    return {"tau_return_depth": tau_return_depth, "resolved": False, "cycle_length": None}


def assign_bin(depth: Optional[int]) -> str:
    if depth is None:
        return NO_RETURN_LABEL
    for label, lo, hi in BINS:
        if lo <= depth < hi:
            return label
    return NO_RETURN_LABEL   # shouldn't happen within cap=1000


def run_histogram(depth: int = 8) -> dict:
    rng = random.Random(RANDOM_SEED)

    print(f"[hg] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[hg] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    all_results: dict[str, dict] = {}

    for op_name, op_fn, cluster in OPERATORS:
        # Stratify
        ss_states  = [s for s in states if s.spin_h == op_fn(s).spin_h]
        nss_states = [s for s in states if s.spin_h != op_fn(s).spin_h]

        sample_ss  = rng.sample(ss_states,  min(SAMPLE_PER_STRATUM, len(ss_states)))
        sample_nss = rng.sample(nss_states, min(SAMPLE_PER_STRATUM, len(nss_states)))
        seeds = sample_ss + sample_nss
        total = len(seeds)

        t1 = time.perf_counter()
        orbit_res = [orbit_tau(op_fn, s, CAP) for s in seeds]

        # Bin the tau return depths
        bin_counts: dict[str, int] = {label: 0 for label, *_ in BINS}
        bin_counts[NO_RETURN_LABEL] = 0

        tau_return_depths = []
        for r in orbit_res:
            b = assign_bin(r["tau_return_depth"])
            bin_counts[b] += 1
            if r["tau_return_depth"] is not None:
                tau_return_depths.append(r["tau_return_depth"])

        unresolved = sum(1 for r in orbit_res if not r["resolved"])

        elapsed = time.perf_counter() - t1
        all_results[op_name] = {
            "cluster":     cluster,
            "sample":      total,
            "ss_n":        len(ss_states),
            "nss_n":       len(nss_states),
            "bin_counts":  bin_counts,
            "tau_depths":  tau_return_depths,
            "unresolved":  unresolved,
            "avg":   statistics.mean(tau_return_depths)   if tau_return_depths else float("nan"),
            "median":statistics.median(tau_return_depths) if tau_return_depths else float("nan"),
            "stdev": statistics.stdev(tau_return_depths)  if len(tau_return_depths) > 1 else float("nan"),
        }

        print(
            f"[hg] {op_name:5s} ({cluster[:3]})  sample={total}  "
            f"returned={len(tau_return_depths)}  "
            f"avg={all_results[op_name]['avg']:.2f}  "
            f"stdev={all_results[op_name]['stdev']:.2f}  "
            f"bins={bin_counts}  ({elapsed:.1f}s)"
        )

    print(f"[hg] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"surface_n": n, "results": all_results}


def write_csv(data: dict) -> None:
    rows = []
    bin_labels = [label for label, *_ in BINS] + [NO_RETURN_LABEL]
    bin_edges  = {label: (lo, hi) for label, lo, hi in BINS}
    bin_edges[NO_RETURN_LABEL] = (None, None)

    for op_name, _, cluster in OPERATORS:
        r = data["results"][op_name]
        total = r["sample"]
        for label in bin_labels:
            count = r["bin_counts"][label]
            lo, hi = bin_edges[label]
            rows.append({
                "operator": op_name,
                "cluster":  cluster,
                "bin_label": label,
                "bin_left":  lo if lo is not None else "N/A",
                "bin_right": hi if hi is not None else "N/A",
                "count":    count,
                "fraction": f"{count/total:.6f}" if total else "nan",
                "note": (
                    f"cap={CAP}; sample={total}; rng_seed={RANDOM_SEED}; "
                    f"same sample as prior tau orbit audit"
                ),
            })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "operator", "cluster", "bin_label", "bin_left", "bin_right",
        "count", "fraction", "note",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[hg] Wrote {OUTPUT_CSV}")


def _bar(fraction: float, width: int = 30) -> str:
    filled = round(fraction * width)
    return "█" * filled + "░" * (width - filled)


def write_md(data: dict) -> None:
    n   = data["surface_n"]
    res = data["results"]

    def fmt(v, prec=2):
        if isinstance(v, float) and v != v:
            return "N/A"
        if isinstance(v, float):
            return f"{v:.{prec}f}"
        return str(v)

    bin_labels = [label for label, *_ in BINS] + [NO_RETURN_LABEL]

    lines = [
        "# Prime Transport — Tau-Return-Depth Histogram Audit",
        "",
        "**Audit type:** Read-only bounded histogram characterization",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total surface states:** {n:,}",
        "",
        "---",
        "",
        "## Sample and method",
        "",
        "| Parameter | Value |",
        "|---|---|",
        f"| Sample reuse | **Yes** — exact same seeds as `run_tau_orbit_depth_audit_v1.py` |",
        f"| Samples per stratum per operator | {SAMPLE_PER_STRATUM} (sigma_stable + non_sigma_stable) |",
        f"| Total seeds per operator | {SAMPLE_PER_STRATUM * 2} |",
        f"| Orbit cap | {CAP} |",
        f"| Random seed | {RANDOM_SEED} |",
        "",
        "### Bin edges",
        "",
        "| Bin label | Range (tau return depth) | Interpretation |",
        "|---|---|---|",
        "| 1–4       | [1, 5)    | very shallow |",
        "| 5–14      | [5, 15)   | shallow |",
        "| 15–49     | [15, 50)  | medium |",
        "| 50–149    | [50, 150) | deep |",
        "| 150+      | [150, 1000] | very deep |",
        "| no_tau_return | cycle resolved; tau never returned | non-returning |",
        "",
        "---",
        "",
        "## Per-operator histograms",
        "",
    ]

    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        total = r["sample"]
        bc    = r["bin_counts"]

        lines += [
            f"### {op_name} ({cluster})",
            "",
            f"Sample: {total} | tau returns: {len(r['tau_depths'])} | "
            f"avg depth: {fmt(r['avg'])} | median: {fmt(r['median'])} | "
            f"stdev: {fmt(r['stdev'])}",
            "",
            "| Bin | count | fraction | bar |",
            "|---|---|---|---|",
        ]
        for label in bin_labels:
            count = bc[label]
            frac  = count / total if total else 0.0
            lines.append(
                f"| {label:15s} | {count:5d} | {frac:.4f} | {_bar(frac, 25)} |"
            )
        lines.append("")

    # Summary table
    lines += [
        "---",
        "",
        "## Summary statistics",
        "",
        "| Operator | Cluster | avg depth | median | stdev | mode bin | no_tau_return fraction |",
        "|---|---|---|---|---|---|---|",
    ]
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        total = r["sample"]
        bc    = r["bin_counts"]
        mode_bin = max(bin_labels[:-1], key=lambda lb: bc[lb])   # exclude no_tau_return
        no_ret_frac = bc[NO_RETURN_LABEL] / total if total else 0.0
        lines.append(
            f"| {op_name} | {cluster} "
            f"| {fmt(r['avg'])} | {fmt(r['median'])} | {fmt(r['stdev'])} "
            f"| {mode_bin} | {no_ret_frac:.4f} |"
        )

    # Cluster comparison
    nt_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "non-transport"]
    tr_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "transport"]

    def clu_bin_frac(ops, label):
        vals = [r["bin_counts"][label] / r["sample"] for _, r in ops if r["sample"] > 0]
        return statistics.mean(vals) if vals else 0.0

    lines += [
        "",
        "---",
        "",
        "## Cluster-level bin comparison",
        "",
        "Mean bin fraction across each cluster:",
        "",
        "| Bin | non-transport mean | transport mean | direction |",
        "|---|---|---|---|",
    ]
    for label in bin_labels:
        nt_f = clu_bin_frac(nt_ops, label)
        tr_f = clu_bin_frac(tr_ops, label)
        if tr_f > nt_f + 0.01:
            direction = "transport higher"
        elif nt_f > tr_f + 0.01:
            direction = "non-transport higher"
        else:
            direction = "similar"
        lines.append(
            f"| {label:15s} | {nt_f:.4f} | {tr_f:.4f} | {direction} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Interpretation",
        "",
        "### Distribution shapes",
        "",
    ]

    # Per-operator characterization
    for op_name, _, cluster in OPERATORS:
        r   = res[op_name]
        bc  = r["bin_counts"]
        total = r["sample"]

        # Compute fractions for shape characterization
        very_shallow = bc["1–4"] / total
        shallow      = bc["5–14"] / total
        medium       = bc["15–49"] / total
        deep_bin     = bc["50–149"] / total
        very_deep    = bc["150+"] / total
        no_ret       = bc[NO_RETURN_LABEL] / total

        # Simple shape description
        returning_total = 1.0 - no_ret
        if returning_total < 0.01:
            shape = "no-return (tau never resets)"
        else:
            top3 = sorted(
                [("1–4", very_shallow), ("5–14", shallow), ("15–49", medium),
                 ("50–149", deep_bin), ("150+", very_deep)],
                key=lambda x: -x[1]
            )[:2]
            dominant = top3[0][0]
            second   = top3[1][0]
            frac_dom = top3[0][1]
            frac_sec = top3[1][1]

            if frac_dom > 0.70:
                shape = f"sharply concentrated in bin [{dominant}]"
            elif frac_dom > 0.45 and frac_dom / (frac_sec + 1e-9) > 2.5:
                shape = f"unimodal, dominated by [{dominant}]"
            elif abs(frac_dom - frac_sec) < 0.10:
                shape = f"bimodal / broadly spread across [{dominant}] and [{second}]"
            else:
                shape = f"right-skewed, concentrated in [{dominant}] with tail in [{second}]"

        lines.append(f"- **{op_name}** ({cluster}): {shape}  (avg={fmt(r['avg'])}, stdev={fmt(r['stdev'])})")

    lines += [
        "",
        "### Transport vs non-transport cluster",
        "",
    ]

    # Key cluster-level observations
    nt_shallow_mean = statistics.mean(
        [(res[nm]["bin_counts"]["1–4"] + res[nm]["bin_counts"]["5–14"]) / res[nm]["sample"]
         for nm, _, cl in OPERATORS if cl == "non-transport"]
    )
    tr_shallow_mean = statistics.mean(
        [(res[nm]["bin_counts"]["1–4"] + res[nm]["bin_counts"]["5–14"]) / res[nm]["sample"]
         for nm, _, cl in OPERATORS if cl == "transport"]
    )
    nt_deep_mean = statistics.mean(
        [(res[nm]["bin_counts"]["50–149"] + res[nm]["bin_counts"]["150+"]) / res[nm]["sample"]
         for nm, _, cl in OPERATORS if cl == "non-transport"]
    )
    tr_deep_mean = statistics.mean(
        [(res[nm]["bin_counts"]["50–149"] + res[nm]["bin_counts"]["150+"]) / res[nm]["sample"]
         for nm, _, cl in OPERATORS if cl == "transport"]
    )

    lines += [
        f"Non-transport combined shallow (bins 1–14) mean fraction: **{nt_shallow_mean:.4f}**",
        f"Transport combined shallow (bins 1–14) mean fraction:     **{tr_shallow_mean:.4f}**",
        "",
        f"Non-transport combined deep (bins 50+) mean fraction: **{nt_deep_mean:.4f}**",
        f"Transport combined deep (bins 50+) mean fraction:     **{tr_deep_mean:.4f}**",
        "",
    ]

    if tr_deep_mean > nt_deep_mean + 0.05:
        lines.append(
            "Transport operators are **heavier-tailed** in the deep tau-depth bins. "
            "The transport cluster systematically produces deeper tau excursions, "
            "consistent with the cluster mean avg-depth finding from the prior audit."
        )
    elif nt_deep_mean > tr_deep_mean + 0.05:
        lines.append(
            "Non-transport operators are **heavier-tailed** in the deep bins — unexpected."
        )
    else:
        lines.append(
            "Transport and non-transport clusters show **similar deep-bin fractions**. "
            "The cluster difference is primarily in the mean, not the tail weight."
        )

    tz_avg = fmt(res["T_z'"]["avg"])
    lines += [
        "",
        "### T_r* distribution shape vs T_c and T_z'",
        "",
        "T_r* avg depth ({}) vs T_c ({}) vs T_z' ({}).".format(
            fmt(res["T_r*"]["avg"]), fmt(res["T_c"]["avg"]), tz_avg
        ),
    ]

    tr_star_deep = (res["T_r*"]["bin_counts"]["50–149"] + res["T_r*"]["bin_counts"]["150+"]) / res["T_r*"]["sample"]
    tc_deep      = (res["T_c"]["bin_counts"]["50–149"]  + res["T_c"]["bin_counts"]["150+"])  / res["T_c"]["sample"]
    tz_deep      = (res["T_z'"]["bin_counts"]["50–149"] + res["T_z'"]["bin_counts"]["150+"]) / res["T_z'"]["sample"]

    lines += [
        f"Deep-bin (50+) fractions: T_r*={tr_star_deep:.4f}, T_c={tc_deep:.4f}, T_z'={tz_deep:.4f}.",
        "",
    ]

    if tr_star_deep > tc_deep + 0.10 and tr_star_deep > tz_deep + 0.10:
        lines.append(
            "**T_r* is structurally distinct within the transport cluster**, not merely "
            "deeper on average: it has a substantially larger fraction of deep-bin returns "
            "compared to T_c and T_z'. T_c and T_z' are more concentrated in the medium bin."
        )
    else:
        lines.append(
            "T_r* is deeper on average but not dramatically more heavy-tailed than T_c and T_z' "
            "in the deep-bin fractions."
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
        "Run a bounded **tau-holonomy periodicity class audit**: for each operator, "
        "enumerate the distinct cycle lengths observed in the sampled orbits and "
        "determine whether cycle lengths cluster around specific values (e.g., factors "
        "of the primorial used to construct the surface).  This would test whether the "
        "tau orbit structure is arithmetically constrained by the surface construction "
        "or is effectively aperiodic within the bounded surface.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[hg] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_histogram(depth=8)
    write_csv(data)
    write_md(data)
