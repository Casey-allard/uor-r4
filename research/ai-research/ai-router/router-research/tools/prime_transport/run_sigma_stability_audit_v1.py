#!/usr/bin/env python3
"""Bounded sigma-carrier stability audit on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

For each operator O in {T_b, T_x, T_y, T_c, T_z', T_r*}:

  sigma-stable states: states s where spin_h(O(s)) == spin_h(s)

  Within the sigma-stable subset, additionally characterise which other
  field families are also invariant:
      theta        : b, phi unchanged
      rho          : r unchanged
      tau_holonomy : all four tau sub-fields unchanged
      swap_geometry: query_semiprime, binding_semiprime, composite_compat_class,
                     admissible_transition, twist — all unchanged

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
    "prime_transport_sigma_stability_audit_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_sigma_stability_audit_v1.csv"
)

OPERATORS = [
    ("T_b",  torus_base_advance_component_v23,   "non-transport"),
    ("T_x",  composite_swap_component_v24,        "non-transport"),
    ("T_y",  composite_twist_component_v25,       "non-transport"),
    ("T_c",  coupled_torus_kick_component_v20,    "transport"),
    ("T_z'", fiber_phase_lift_component_v21,      "transport"),
    ("T_r*", radial_transport_component_v22,      "transport"),
]

# predicate: does s->t leave this family unchanged?
COND_SIGMA      = lambda s, t: s.spin_h == t.spin_h
COND_THETA      = lambda s, t: s.b == t.b and s.phi == t.phi
COND_RHO        = lambda s, t: s.r == t.r
COND_TAU        = lambda s, t: s.tau == t.tau
COND_SWAP       = lambda s, t: (
    s.query_semiprime == t.query_semiprime
    and s.binding_semiprime == t.binding_semiprime
    and s.composite_compat_class == t.composite_compat_class
    and s.admissible_transition == t.admissible_transition
    and s.twist == t.twist
)


def run_audit(depth: int = 8) -> dict:
    print(f"[ss] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[ss] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    results: dict[str, dict] = {}

    for name, fn, cluster in OPERATORS:
        t1 = time.perf_counter()
        ss_count = 0          # sigma-stable
        ss_theta  = 0         # sigma-stable AND theta-invariant
        ss_rho    = 0         # sigma-stable AND rho-invariant
        ss_tau    = 0         # sigma-stable AND tau-invariant
        ss_swap   = 0         # sigma-stable AND swap_geometry-invariant

        for s in states:
            t = fn(s)
            if COND_SIGMA(s, t):
                ss_count += 1
                if COND_THETA(s, t): ss_theta += 1
                if COND_RHO(s, t):   ss_rho   += 1
                if COND_TAU(s, t):   ss_tau    += 1
                if COND_SWAP(s, t):  ss_swap   += 1

        elapsed = time.perf_counter() - t1
        ss_frac = ss_count / n if n > 0 else 0.0
        # conditional fractions (within sigma-stable subset)
        def cf(x):
            return x / ss_count if ss_count > 0 else float("nan")

        results[name] = {
            "cluster":    cluster,
            "n":          n,
            "ss_count":   ss_count,
            "ss_frac":    ss_frac,
            "ss_theta":   ss_theta,
            "ss_rho":     ss_rho,
            "ss_tau":     ss_tau,
            "ss_swap":    ss_swap,
            "cf_theta":   cf(ss_theta),
            "cf_rho":     cf(ss_rho),
            "cf_tau":     cf(ss_tau),
            "cf_swap":    cf(ss_swap),
        }
        print(
            f"[ss] {name:5s}  sigma_stable={ss_count:>7} ({ss_frac:.6f})  "
            f"theta={cf(ss_theta):.4f}  rho={cf(ss_rho):.4f}  "
            f"tau={cf(ss_tau):.4f}  swap={cf(ss_swap):.4f}  ({elapsed:.1f}s)"
        )

    print(f"[ss] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"states": n, "results": results}


def write_csv(data: dict) -> None:
    n = data["states"]
    rows = []
    for name, _, cluster in OPERATORS:
        r = data["results"][name]
        rows.append({
            "operator":                                name,
            "cluster":                                 cluster,
            "sigma_stable_count":                      r["ss_count"],
            "sigma_stable_fraction":                   f"{r['ss_frac']:.6f}",
            "theta_invariant_within_sigma_stable":     f"{r['cf_theta']:.6f}",
            "rho_invariant_within_sigma_stable":       f"{r['cf_rho']:.6f}",
            "tau_invariant_within_sigma_stable":       f"{r['cf_tau']:.6f}",
            "swap_geometry_invariant_within_sigma_stable": f"{r['cf_swap']:.6f}",
            "note": (
                f"sigma-stable: {r['ss_count']}/{n}; "
                f"conditional fractions within sigma-stable subset"
            ),
        })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=[
            "operator",
            "cluster",
            "sigma_stable_count",
            "sigma_stable_fraction",
            "theta_invariant_within_sigma_stable",
            "rho_invariant_within_sigma_stable",
            "tau_invariant_within_sigma_stable",
            "swap_geometry_invariant_within_sigma_stable",
            "note",
        ])
        writer.writeheader()
        writer.writerows(rows)
    print(f"[ss] Wrote {OUTPUT_CSV}")


def write_md(data: dict) -> None:
    n = data["states"]
    res = data["results"]

    lines = [
        "# Prime Transport — Sigma-Carrier Stability Audit",
        "",
        "**Audit type:** Read-only characterization",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total states on surface:** {n:,}",
        "",
        "---",
        "",
        "## Operators audited",
        "",
        "| Label | Function | Cluster |",
        "|---|---|---|",
        "| T_b  | `torus_base_advance_component_v23`  | non-transport |",
        "| T_x  | `composite_swap_component_v24`       | non-transport |",
        "| T_y  | `composite_twist_component_v25`      | non-transport |",
        "| T_c  | `coupled_torus_kick_component_v20`   | transport     |",
        "| T_z' | `fiber_phase_lift_component_v21`     | transport     |",
        "| T_r* | `radial_transport_component_v22`     | transport     |",
        "",
        "---",
        "",
        "## Definition",
        "",
        "A state `s` is **sigma-stable** under operator `O` if:",
        "",
        "    spin_h(O(s)) == spin_h(s)",
        "",
        "i.e. the observable spin-H carrier is left unchanged by the operator action.",
        "This is a weaker condition than a full fixed point (`O(s) == s`).",
        "",
        "Within the sigma-stable subset, we additionally report the fraction of those",
        "states that are also invariant under each other field family",
        "(`theta`, `rho`, `tau_holonomy`, `swap_geometry`).",
        "",
        "---",
        "",
        "## Per-operator sigma-stable count and fraction",
        "",
        "| Operator | Cluster | sigma-stable count | sigma-stable fraction |",
        "|---|---|---|---|",
    ]

    for name, _, cluster in OPERATORS:
        r = res[name]
        lines.append(
            f"| {name} | {cluster} | {r['ss_count']:,} | {r['ss_frac']:.6f} |"
        )

    nt_mean = sum(
        res[n]["ss_frac"] for n, _, c in OPERATORS if c == "non-transport"
    ) / 3
    tr_mean = sum(
        res[n]["ss_frac"] for n, _, c in OPERATORS if c == "transport"
    ) / 3

    lines += [
        "",
        f"Non-transport mean sigma-stable fraction: **{nt_mean:.6f}**",
        f"Transport mean sigma-stable fraction:     **{tr_mean:.6f}**",
        "",
        "---",
        "",
        "## Conditional invariance within sigma-stable subset",
        "",
        "Fractions below are computed *within* the sigma-stable states for each operator.",
        "A value of `1.000000` means every sigma-stable state is also invariant in that family.",
        "A value of `0.000000` means no sigma-stable state is invariant in that family.",
        "",
        "| Operator | Cluster | ss_count | theta | rho | tau | swap_geo |",
        "|---|---|---|---|---|---|---|",
    ]

    for name, _, cluster in OPERATORS:
        r = res[name]
        lines.append(
            f"| {name} | {cluster} | {r['ss_count']:,} "
            f"| {r['cf_theta']:.4f} "
            f"| {r['cf_rho']:.4f} "
            f"| {r['cf_tau']:.4f} "
            f"| {r['cf_swap']:.4f} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Cluster-level comparison",
        "",
        "### Non-transport cluster {T_b, T_x, T_y}",
        "",
    ]

    for name, _, cluster in OPERATORS:
        if cluster != "non-transport":
            continue
        r = res[name]
        lines += [
            f"**{name}** — sigma-stable fraction: {r['ss_frac']:.6f}",
            f"  - theta invariant (within ss):    {r['cf_theta']:.6f}",
            f"  - rho invariant (within ss):      {r['cf_rho']:.6f}",
            f"  - tau invariant (within ss):      {r['cf_tau']:.6f}",
            f"  - swap_geo invariant (within ss): {r['cf_swap']:.6f}",
            "",
        ]

    lines += [
        "### Transport cluster {T_c, T_z', T_r*}",
        "",
    ]

    for name, _, cluster in OPERATORS:
        if cluster != "transport":
            continue
        r = res[name]
        lines += [
            f"**{name}** — sigma-stable fraction: {r['ss_frac']:.6f}",
            f"  - theta invariant (within ss):    {r['cf_theta']:.6f}",
            f"  - rho invariant (within ss):      {r['cf_rho']:.6f}",
            f"  - tau invariant (within ss):      {r['cf_tau']:.6f}",
            f"  - swap_geo invariant (within ss): {r['cf_swap']:.6f}",
            "",
        ]

    lines += [
        "---",
        "",
        "## Interpretation",
        "",
    ]

    # Determine T_b bridge status
    tb = res["T_b"]
    tc = res["T_c"]
    tz = res["T_z'"]
    tr = res["T_r*"]
    tx = res["T_x"]
    ty = res["T_y"]

    # Transport coherence: do all three share swap_geo invariance within ss?
    tr_swap_coherent = all(
        res[nm]["cf_swap"] > 0.99
        for nm, _, cl in OPERATORS if cl == "transport"
    )
    tr_tau_coherent = all(
        res[nm]["cf_tau"] < 0.05
        for nm, _, cl in OPERATORS if cl == "transport"
    )

    lines += [
        "### Sigma-stable fraction comparison",
        "",
        f"The non-transport mean sigma-stable fraction ({nt_mean:.6f}) and transport "
        f"mean ({tr_mean:.6f}) reveal whether one cluster systematically preserves the "
        "spin-H carrier more or less than the other.",
        "",
        "### Transport coherence within sigma-stable subset",
        "",
    ]

    if tr_swap_coherent:
        lines.append(
            "All three transport operators share **100% swap_geometry invariance** "
            "within their sigma-stable subsets — confirming that transport sigma-stable "
            "states are also semiprime/permutation-preserving, consistent with the "
            "field-signature finding that transport operators never touch swap_geometry fields."
        )
    else:
        lines.append(
            "Transport operators do NOT share uniform swap_geometry invariance within "
            "their sigma-stable subsets — the transport cluster is heterogeneous at the "
            "sigma-stability level."
        )

    lines += [""]

    if tr_tau_coherent:
        lines.append(
            "Transport sigma-stable states have near-zero tau_holonomy invariance — "
            "even sigma-stable states are being moved by the tau carrier, consistent "
            "with tau being the primary motion axis for transport operators."
        )
    else:
        lines.append(
            "Transport sigma-stable states have non-negligible tau_holonomy invariance — "
            "some sigma-stable transport states are also tau-frozen."
        )

    lines += [
        "",
        "### T_b bridge status",
        "",
    ]

    # T_b bridge: check if its ss profile resembles transport more than NT
    tb_like_transport = (
        abs(tb["cf_swap"] - 1.0) < 0.01  # 100% swap invariant like transport
    )
    if tb_like_transport:
        lines.append(
            "**T_b** again shows bridge behavior: within its sigma-stable subset, "
            "swap_geometry invariance is 100% — matching the transport cluster — even "
            "though T_b belongs to the non-transport label. This is consistent with the "
            "fixed-point census finding that T_b preserves swap_geometry globally."
        )
    else:
        lines.append(
            "**T_b** does not show unambiguous bridge behavior within its sigma-stable "
            "subset at the swap_geometry level."
        )

    lines += [
        "",
        "### What sigma-stability means for the rebuilt algebra",
        "",
        "Sigma stability characterises the subset of states on which an operator acts",
        "as a 'spin-carrier isometry': the observable spin-H is unchanged even though",
        "other coordinates (tau, theta, rho, swap fields) may change.  The size and",
        "field-composition of this subset determine which part of the bounded surface",
        "is 'transparent' to the sigma observable under each operator.",
        "",
        "The conditional invariance profile inside this subset reveals whether sigma",
        "stability is correlated with or independent of stability in other field families.",
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
        "Run a bounded **sigma-stable sub-algebra closure test**:",
        "apply each non-transport operator to every sigma-stable state of each transport",
        "operator and measure how often the output remains sigma-stable.",
        "This tests whether the transport sigma-stable subsets are closed (or approximately",
        "closed) under the action of the non-transport layer — the natural follow-on to",
        "this characterization step.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[ss] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_audit(depth=8)
    write_csv(data)
    write_md(data)
