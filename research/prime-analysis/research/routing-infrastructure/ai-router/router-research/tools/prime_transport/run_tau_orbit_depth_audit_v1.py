#!/usr/bin/env python3
"""Bounded stratified tau-holonomy orbit depth audit on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

For each operator O in {T_b, T_x, T_y, T_c, T_z', T_r*}:

  Stratified sample:
    - sigma_stable stratum : states s where spin_h(O(s)) == spin_h(s)
    - non_sigma_stable stratum : complement

  For each sampled seed state s, iterate s -> O(s) -> O²(s) -> ...
  and track the tau sub-state (NativeTauV3) until:
    (a) tau returns to tau(s)         -> record tau_return_depth
    (b) full state cycle detected     -> record cycle_length, stop
    (c) iteration cap reached         -> unresolved

  Metrics reported:
    - tau_return_count / fraction
    - avg / median tau_return_depth
    - avg / max cycle_length
    - unresolved_at_cap fraction

Sampling parameters:
    SAMPLE_PER_STRATUM : 500 (up to 500 from each stratum per operator)
    CAP                : 200 iterations per seed

Surface: geometry_native_operator_model_v25.bounded_operator_surface_v25, depth=8.
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
    "prime_transport_tau_orbit_depth_audit_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_orbit_depth_audit_v1.csv"
)

OPERATORS = [
    ("T_b",  torus_base_advance_component_v23,   "non-transport"),
    ("T_x",  composite_swap_component_v24,        "non-transport"),
    ("T_y",  composite_twist_component_v25,       "non-transport"),
    ("T_c",  coupled_torus_kick_component_v20,    "transport"),
    ("T_z'", fiber_phase_lift_component_v21,      "transport"),
    ("T_r*", radial_transport_component_v22,      "transport"),
]

SAMPLE_PER_STRATUM = 500   # up to 500 seeds per stratum per operator
CAP = 200                   # max orbit steps per seed
RANDOM_SEED = 42


def orbit_tau(fn, seed, cap: int) -> dict:
    """Walk the orbit of seed under fn, tracking tau.

    Returns dict with keys:
      tau_return_depth : int or None (step at which tau first returns to seed.tau)
      cycle_length     : int or None (state cycle length if detected before cap)
      resolved         : bool (False if cap reached without cycle)
      steps            : int  (total steps taken)
    """
    tau0 = seed.tau
    seen: dict = {seed: 0}   # state -> step
    current = seed
    tau_return_depth: Optional[int] = None

    for step in range(1, cap + 1):
        current = fn(current)

        # Tau return check (only record first occurrence)
        if tau_return_depth is None and current.tau == tau0:
            tau_return_depth = step

        # Full state cycle detection
        if current in seen:
            cycle_length = step - seen[current]
            return {
                "tau_return_depth": tau_return_depth,
                "cycle_length": cycle_length,
                "resolved": True,
                "steps": step,
            }
        seen[current] = step

    # Cap reached without cycle
    return {
        "tau_return_depth": tau_return_depth,
        "cycle_length": None,
        "resolved": False,
        "steps": cap,
    }


def run_audit(depth: int = 8) -> dict:
    rng = random.Random(RANDOM_SEED)

    print(f"[to] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[to] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    all_results: dict[str, dict] = {}

    for op_name, op_fn, cluster in OPERATORS:
        print(f"[to] Operator {op_name} ...")
        t1 = time.perf_counter()

        # Stratify: sigma-stable vs non-sigma-stable
        ss_states = []
        nss_states = []
        for s in states:
            ts = op_fn(s)
            if s.spin_h == ts.spin_h:
                ss_states.append(s)
            else:
                nss_states.append(s)

        # Sample up to SAMPLE_PER_STRATUM from each stratum
        sample_ss  = rng.sample(ss_states,  min(SAMPLE_PER_STRATUM, len(ss_states)))
        sample_nss = rng.sample(nss_states, min(SAMPLE_PER_STRATUM, len(nss_states)))

        print(
            f"[to]   strata: ss={len(ss_states)} -> sample {len(sample_ss)}; "
            f"nss={len(nss_states)} -> sample {len(sample_nss)}"
        )

        # Run orbits for each stratum
        stratum_data = {}
        for stratum_name, sample in [("sigma_stable", sample_ss), ("non_sigma_stable", sample_nss)]:
            orbit_results = [orbit_tau(op_fn, s, CAP) for s in sample]

            tau_returns   = [r["tau_return_depth"] for r in orbit_results if r["tau_return_depth"] is not None]
            cycle_lengths = [r["cycle_length"]     for r in orbit_results if r["cycle_length"] is not None]
            unresolved    = sum(1 for r in orbit_results if not r["resolved"])

            total = len(sample)
            stratum_data[stratum_name] = {
                "sample_count":      total,
                "tau_return_count":  len(tau_returns),
                "tau_return_fraction": len(tau_returns) / total if total else float("nan"),
                "avg_tau_return_depth": statistics.mean(tau_returns) if tau_returns else float("nan"),
                "median_tau_return_depth": statistics.median(tau_returns) if tau_returns else float("nan"),
                "avg_cycle_length":  statistics.mean(cycle_lengths) if cycle_lengths else float("nan"),
                "max_cycle_length":  max(cycle_lengths) if cycle_lengths else float("nan"),
                "unresolved_count":  unresolved,
                "unresolved_fraction": unresolved / total if total else float("nan"),
            }

        # Also pool both strata
        all_sample = sample_ss + sample_nss
        orbit_all = [orbit_tau(op_fn, s, CAP) for s in all_sample]
        tau_returns_all   = [r["tau_return_depth"] for r in orbit_all if r["tau_return_depth"] is not None]
        cycle_lengths_all = [r["cycle_length"]     for r in orbit_all if r["cycle_length"] is not None]
        unresolved_all    = sum(1 for r in orbit_all if not r["resolved"])
        total_all = len(all_sample)

        all_results[op_name] = {
            "cluster":               cluster,
            "ss_total":              len(ss_states),
            "nss_total":             len(nss_states),
            "stratum":               stratum_data,
            "pooled": {
                "sample_count":          total_all,
                "tau_return_count":      len(tau_returns_all),
                "tau_return_fraction":   len(tau_returns_all) / total_all if total_all else float("nan"),
                "avg_tau_return_depth":  statistics.mean(tau_returns_all) if tau_returns_all else float("nan"),
                "median_tau_return_depth": statistics.median(tau_returns_all) if tau_returns_all else float("nan"),
                "avg_cycle_length":      statistics.mean(cycle_lengths_all) if cycle_lengths_all else float("nan"),
                "max_cycle_length":      max(cycle_lengths_all) if cycle_lengths_all else float("nan"),
                "unresolved_count":      unresolved_all,
                "unresolved_fraction":   unresolved_all / total_all if total_all else float("nan"),
            },
        }

        elapsed = time.perf_counter() - t1
        p = all_results[op_name]["pooled"]
        print(
            f"[to]   {op_name} pooled: sample={total_all}  "
            f"tau_ret={p['tau_return_count']}({p['tau_return_fraction']:.4f})  "
            f"avg_ret_depth={p['avg_tau_return_depth']:.2f}  "
            f"avg_cycle={p['avg_cycle_length']:.2f}  "
            f"unresolved={p['unresolved_fraction']:.4f}  ({elapsed:.1f}s)"
        )

    print(f"[to] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"surface_n": n, "results": all_results}


def write_csv(data: dict) -> None:
    rows = []
    for op_name, _, cluster in OPERATORS:
        r = data["results"][op_name]
        # One row per (operator, stratum) + one pooled row
        for stratum_name in ("sigma_stable", "non_sigma_stable", "pooled"):
            d = r["stratum"][stratum_name] if stratum_name != "pooled" else r["pooled"]

            def fmt(v):
                if isinstance(v, float) and v != v:  # nan
                    return "nan"
                if isinstance(v, float):
                    return f"{v:.6f}"
                return str(v)

            rows.append({
                "operator":                 op_name,
                "cluster":                  cluster,
                "stratum":                  stratum_name,
                "sample_count":             d["sample_count"],
                "tau_return_count":         d["tau_return_count"],
                "tau_return_fraction":      fmt(d["tau_return_fraction"]),
                "avg_tau_return_depth":     fmt(d["avg_tau_return_depth"]),
                "median_tau_return_depth":  fmt(d["median_tau_return_depth"]),
                "avg_cycle_length":         fmt(d["avg_cycle_length"]),
                "max_cycle_length":         fmt(d["max_cycle_length"]),
                "unresolved_at_cap_fraction": fmt(d["unresolved_fraction"]),
                "note": (
                    f"cap={CAP}; sample_per_stratum={SAMPLE_PER_STRATUM}; "
                    f"rng_seed={RANDOM_SEED}"
                ),
            })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "operator", "cluster", "stratum", "sample_count",
        "tau_return_count", "tau_return_fraction",
        "avg_tau_return_depth", "median_tau_return_depth",
        "avg_cycle_length", "max_cycle_length",
        "unresolved_at_cap_fraction", "note",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[to] Wrote {OUTPUT_CSV}")


def write_md(data: dict) -> None:
    n = data["surface_n"]
    res = data["results"]

    def fmt(v, prec=4):
        if isinstance(v, float) and v != v:
            return "N/A"
        if isinstance(v, float):
            return f"{v:.{prec}f}"
        return str(v)

    lines = [
        "# Prime Transport — Tau-Holonomy Orbit Depth Audit",
        "",
        "**Audit type:** Read-only bounded stratified orbit audit",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total surface states:** {n:,}",
        "",
        "---",
        "",
        "## Sampling and bounding parameters",
        "",
        f"| Parameter | Value |",
        "|---|---|",
        f"| Samples per stratum per operator | {SAMPLE_PER_STRATUM} |",
        f"| Strata | sigma-stable, non-sigma-stable (under each operator) |",
        f"| Maximum orbit iterations (cap) | {CAP} |",
        f"| Random seed | {RANDOM_SEED} |",
        f"| Tau-return criterion | tau(Oᵏ(s)) == tau(s) for smallest k ≥ 1 |",
        f"| Cycle-detection criterion | full state Oᵏ(s) ∈ previously seen states |",
        "",
        "---",
        "",
        "## Stratum sizes per operator",
        "",
        "| Operator | Cluster | sigma-stable count | non-sigma-stable count |",
        "|---|---|---|---|",
    ]
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        lines.append(
            f"| {op_name} | {cluster} | {r['ss_total']:,} | {r['nss_total']:,} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Per-operator pooled results (both strata combined)",
        "",
        "| Operator | Cluster | sample | τ-return count | τ-return fraction | avg return depth | median return depth | avg cycle | max cycle | unresolved |",
        "|---|---|---|---|---|---|---|---|---|---|",
    ]
    for op_name, _, cluster in OPERATORS:
        p = res[op_name]["pooled"]
        lines.append(
            f"| {op_name} | {cluster} "
            f"| {p['sample_count']} "
            f"| {p['tau_return_count']} "
            f"| {fmt(p['tau_return_fraction'])} "
            f"| {fmt(p['avg_tau_return_depth'])} "
            f"| {fmt(p['median_tau_return_depth'])} "
            f"| {fmt(p['avg_cycle_length'])} "
            f"| {fmt(p['max_cycle_length'])} "
            f"| {fmt(p['unresolved_fraction'])} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Per-stratum detail",
        "",
    ]
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        lines += [
            f"### {op_name} ({cluster})",
            "",
            "| Stratum | sample | τ-return fraction | avg depth | median depth | avg cycle | unresolved |",
            "|---|---|---|---|---|---|---|",
        ]
        for stratum_name in ("sigma_stable", "non_sigma_stable"):
            d = r["stratum"][stratum_name]
            lines.append(
                f"| {stratum_name} "
                f"| {d['sample_count']} "
                f"| {fmt(d['tau_return_fraction'])} "
                f"| {fmt(d['avg_tau_return_depth'])} "
                f"| {fmt(d['median_tau_return_depth'])} "
                f"| {fmt(d['avg_cycle_length'])} "
                f"| {fmt(d['unresolved_fraction'])} |"
            )
        lines.append("")

    # Cluster comparison
    nt_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "non-transport"]
    tr_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "transport"]

    def cluster_mean(ops, key):
        vals = [r["pooled"][key] for _, r in ops
                if not (isinstance(r["pooled"][key], float) and r["pooled"][key] != r["pooled"][key])]
        return statistics.mean(vals) if vals else float("nan")

    nt_avg_depth   = cluster_mean(nt_ops, "avg_tau_return_depth")
    tr_avg_depth   = cluster_mean(tr_ops, "avg_tau_return_depth")
    nt_ret_frac    = cluster_mean(nt_ops, "tau_return_fraction")
    tr_ret_frac    = cluster_mean(tr_ops, "tau_return_fraction")
    nt_cycle       = cluster_mean(nt_ops, "avg_cycle_length")
    tr_cycle       = cluster_mean(tr_ops, "avg_cycle_length")
    nt_unresolved  = cluster_mean(nt_ops, "unresolved_fraction")
    tr_unresolved  = cluster_mean(tr_ops, "unresolved_fraction")

    lines += [
        "---",
        "",
        "## Cluster-level comparison",
        "",
        "| Metric | non-transport {T_b, T_x, T_y} | transport {T_c, T_z', T_r*} |",
        "|---|---|---|",
        f"| mean tau-return fraction  | {fmt(nt_ret_frac)}   | {fmt(tr_ret_frac)} |",
        f"| mean avg return depth     | {fmt(nt_avg_depth)}  | {fmt(tr_avg_depth)} |",
        f"| mean avg cycle length     | {fmt(nt_cycle)}      | {fmt(tr_cycle)} |",
        f"| mean unresolved fraction  | {fmt(nt_unresolved)} | {fmt(tr_unresolved)} |",
        "",
    ]

    lines += [
        "---",
        "",
        "## Interpretation",
        "",
        "### Tau-return structure",
        "",
    ]

    if abs(tr_avg_depth - nt_avg_depth) > 1.0:
        if tr_avg_depth > nt_avg_depth:
            lines.append(
                f"Transport operators have a **longer mean tau return depth** "
                f"({fmt(tr_avg_depth)}) than non-transport operators ({fmt(nt_avg_depth)}). "
                "This is consistent with the hypothesis that transport operators drive "
                "deeper tau excursions before the holonomy coordinate returns."
            )
        else:
            lines.append(
                f"Transport operators have a **shorter mean tau return depth** "
                f"({fmt(tr_avg_depth)}) than non-transport operators ({fmt(nt_avg_depth)}). "
                "Transport operators cycle back to the initial tau configuration faster "
                "than non-transport operators."
            )
    else:
        lines.append(
            f"Transport and non-transport operators have **similar mean tau return depths** "
            f"({fmt(tr_avg_depth)} vs {fmt(nt_avg_depth)}). "
            "Tau return timing does not differentiate the two clusters strongly."
        )

    lines += [""]

    if abs(tr_ret_frac - nt_ret_frac) > 0.05:
        if tr_ret_frac < nt_ret_frac:
            lines.append(
                f"Transport operators have a **lower tau-return rate within cap** "
                f"({fmt(tr_ret_frac)}) compared to non-transport ({fmt(nt_ret_frac)}). "
                "Transport tau excursions are harder to close within the cap, suggesting "
                "longer or more complex tau orbits."
            )
        else:
            lines.append(
                f"Transport operators have a **higher tau-return rate within cap** "
                f"({fmt(tr_ret_frac)}) compared to non-transport ({fmt(nt_ret_frac)})."
            )
    else:
        lines.append(
            f"Tau-return rates are similar across clusters "
            f"(transport {fmt(tr_ret_frac)}, non-transport {fmt(nt_ret_frac)})."
        )

    lines += [
        "",
        "### Tau orbit and the two-cluster picture",
        "",
        "Given that tau_holonomy was identified as the primary motion axis for both "
        "clusters in the field-signature audit, the tau orbit depth reveals whether the "
        "**rate** and **depth** of tau motion differs between clusters or whether the "
        "distinction lies purely in *which other fields* accompany the tau motion.",
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
        "Run a bounded **tau-holonomy cycle-length census on the full surface**: "
        "for each operator, exhaustively compute the tau-return depth for every state "
        "on the bounded v25 surface (not just a sample).  This would confirm whether "
        "the sampled tau orbit distributions are representative and whether any operator "
        "has states with structurally anomalous tau return depths that the sample "
        "did not capture.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[to] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_audit(depth=8)
    write_csv(data)
    write_md(data)
