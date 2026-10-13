#!/usr/bin/env python3
"""Bounded tau sub-field periodicity decomposition on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

Reuses the EXACT same per-operator samples as run_tau_orbit_depth_audit_v1.py
(rng = random.Random(42), same operator order, 500 per stratum = 1000 per operator).

For each sampled orbit, determines:
  - period of each tau sub-field independently:
      swap_phase   (mod 2 in theory, but state-dependent step context)
      coupled_phase (mod 5, variable step)
      twist_phase   (mod 2)
      lift_phase    (mod 12, variable step)
  - full tau state return depth (= LCM or other combination)
  - whether full tau period == LCM(sub-periods) [pure synchronization]
  - whether full tau period == max(sub-periods) [dominated by one field]
  - which sub-field most often sets the bottleneck

Canonical moduli from geometry_native_spinH_candidate_v3.py:
    SWAP_PHASE_MODULUS   = 2
    COUPLED_PHASE_MODULUS = 5
    TWIST_PHASE_MODULUS  = 2
    LIFT_PHASE_MODULUS   = 12
    TAU_LCM              = 60
"""

from __future__ import annotations

import csv
import math
import random
import statistics
import sys
import time
from collections import Counter, defaultdict
from functools import reduce
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
    "prime_transport_tau_subfield_periodicity_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_subfield_periodicity_v1.csv"
)

OPERATORS = [
    ("T_b",  torus_base_advance_component_v23,   "non-transport"),
    ("T_x",  composite_swap_component_v24,        "non-transport"),
    ("T_y",  composite_twist_component_v25,       "non-transport"),
    ("T_c",  coupled_torus_kick_component_v20,    "transport"),
    ("T_z'", fiber_phase_lift_component_v21,      "transport"),
    ("T_r*", radial_transport_component_v22,      "transport"),
]

SUBFIELDS   = ["swap", "coupled", "twist", "lift"]
SAMPLE_PER_STRATUM = 500
CAP         = 1000
RANDOM_SEED = 42


def _lcm(a: int, b: int) -> int:
    return a * b // math.gcd(a, b)

def _lcm_list(vals: list[int]) -> int:
    return reduce(_lcm, vals, 1)


def orbit_subfields(fn, seed, cap: int) -> dict:
    """Walk orbit, tracking per-sub-field first return and full tau return."""
    tau0 = seed.tau
    sf0  = {
        "swap":    tau0.swap_phase,
        "coupled": tau0.coupled_phase,
        "twist":   tau0.twist_phase,
        "lift":    tau0.lift_phase,
    }
    seen: dict = {seed: 0}
    current = seed

    sf_return: dict[str, Optional[int]] = {f: None for f in SUBFIELDS}
    tau_return: Optional[int]           = None
    cycle_length: Optional[int]         = None

    for step in range(1, cap + 1):
        current = fn(current)
        tau_curr = current.tau

        # Sub-field first returns
        if sf_return["swap"]    is None and tau_curr.swap_phase    == sf0["swap"]:
            sf_return["swap"]    = step
        if sf_return["coupled"] is None and tau_curr.coupled_phase == sf0["coupled"]:
            sf_return["coupled"] = step
        if sf_return["twist"]   is None and tau_curr.twist_phase   == sf0["twist"]:
            sf_return["twist"]   = step
        if sf_return["lift"]    is None and tau_curr.lift_phase    == sf0["lift"]:
            sf_return["lift"]    = step

        # Full tau return (all sub-fields simultaneously)
        if tau_return is None and tau_curr == tau0:
            tau_return = step

        # State cycle detection
        if current in seen:
            cycle_length = step - seen[current]
            break
        seen[current] = step

    # Synchronization analysis
    known_sf = {f: p for f, p in sf_return.items() if p is not None}
    lcm_known = _lcm_list(list(known_sf.values())) if known_sf else None
    max_known = max(known_sf.values()) if known_sf else None

    # Bottleneck: sub-field whose period == max(sub-periods)
    if known_sf and max_known is not None:
        bottlenecks = [f for f, p in known_sf.items() if p == max_known]
    else:
        bottlenecks = []

    # Synchronization type
    sync_type = "unknown"
    if tau_return is not None and known_sf:
        if len(known_sf) == 4:
            if tau_return == lcm_known:
                sync_type = "pure_lcm"
            elif tau_return == max_known:
                sync_type = "single_dominant"
            elif tau_return % (lcm_known or 1) == 0:
                sync_type = "lcm_multiple"
            else:
                sync_type = "complex"
        else:
            sync_type = "incomplete"   # some sub-fields didn't return within cap
    elif tau_return is None:
        sync_type = "tau_unresolved"

    return {
        "sf_return":     sf_return,
        "tau_return":    tau_return,
        "cycle_length":  cycle_length,
        "resolved":      cycle_length is not None,
        "lcm_known":     lcm_known,
        "max_known":     max_known,
        "bottlenecks":   bottlenecks,
        "sync_type":     sync_type,
    }


def run_decomposition(depth: int = 8) -> dict:
    rng = random.Random(RANDOM_SEED)

    print(f"[sf] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    n = len(states)
    print(f"[sf] Surface ready: {n} states ({time.perf_counter()-t0:.1f}s)")

    all_results: dict[str, dict] = {}

    for op_name, op_fn, cluster in OPERATORS:
        ss_states  = [s for s in states if s.spin_h == op_fn(s).spin_h]
        nss_states = [s for s in states if s.spin_h != op_fn(s).spin_h]

        sample_ss  = rng.sample(ss_states,  min(SAMPLE_PER_STRATUM, len(ss_states)))
        sample_nss = rng.sample(nss_states, min(SAMPLE_PER_STRATUM, len(nss_states)))
        seeds = sample_ss + sample_nss
        total = len(seeds)

        t1 = time.perf_counter()
        orbits = [orbit_subfields(op_fn, s, CAP) for s in seeds]
        elapsed = time.perf_counter() - t1

        # Per-sub-field period distributions
        sf_counters: dict[str, Counter] = {f: Counter() for f in SUBFIELDS}
        sf_none: dict[str, int]         = {f: 0 for f in SUBFIELDS}

        sync_counter   = Counter()
        bottleneck_ctr = Counter()

        for o in orbits:
            for f in SUBFIELDS:
                p = o["sf_return"][f]
                if p is not None:
                    sf_counters[f][p] += 1
                else:
                    sf_none[f] += 1
            sync_counter[o["sync_type"]] += 1
            for b in o["bottlenecks"]:
                bottleneck_ctr[b] += 1

        # Dominant sub-field period for each field
        sf_dom = {f: sf_counters[f].most_common(1)[0][0]
                  if sf_counters[f] else None
                  for f in SUBFIELDS}
        sf_dom_frac = {f: sf_counters[f].most_common(1)[0][1] / total
                       if sf_counters[f] else 0.0
                       for f in SUBFIELDS}

        all_results[op_name] = {
            "cluster":       cluster,
            "sample":        total,
            "orbits":        orbits,
            "sf_counters":   sf_counters,
            "sf_none":       sf_none,
            "sf_dom":        sf_dom,
            "sf_dom_frac":   sf_dom_frac,
            "sync_counter":  sync_counter,
            "bottleneck_ctr": bottleneck_ctr,
        }

        top_bot = bottleneck_ctr.most_common(2)
        top_sync = sync_counter.most_common(3)
        print(
            f"[sf] {op_name:5s}  "
            f"swap_dom={sf_dom['swap']}  coupled_dom={sf_dom['coupled']}  "
            f"twist_dom={sf_dom['twist']}  lift_dom={sf_dom['lift']}  "
            f"top_bottleneck={top_bot}  "
            f"sync={top_sync}  ({elapsed:.1f}s)"
        )

    print(f"[sf] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"surface_n": n, "results": all_results}


def write_csv(data: dict) -> None:
    rows = []
    for op_name, _, cluster in OPERATORS:
        r = data["results"][op_name]
        total = r["sample"]
        sf_c  = r["sf_counters"]
        sf_no = r["sf_none"]
        bt    = r["bottleneck_ctr"]
        sc    = r["sync_counter"]

        for f in SUBFIELDS:
            counter   = sf_c[f]
            none_cnt  = sf_no[f]
            total_obs = sum(counter.values())

            # Top observed periods
            for period, cnt in counter.most_common():
                # Does this sub-field's period == full tau period in those orbits?
                # Proxy: is this period the LCM driver (bottleneck) for this field?
                drives = bt.get(f, 0) / total if total else 0.0
                rows.append({
                    "operator":            op_name,
                    "subfield":            f,
                    "observed_period":     period,
                    "count":               cnt,
                    "fraction":            f"{cnt/total:.6f}",
                    "drives_full_tau_period": f"{drives:.4f}",
                    "synchronization_role": f"bottleneck_fraction={drives:.4f}",
                    "cluster":             cluster,
                    "note": (
                        f"cap={CAP}; sample={total}; rng_seed={RANDOM_SEED}; "
                        f"subfield_unresolved_count={none_cnt}; "
                        f"dominant_sync_type={sc.most_common(1)[0][0] if sc else 'none'}"
                    ),
                })
            # Row for unresolved
            if none_cnt > 0:
                rows.append({
                    "operator":            op_name,
                    "subfield":            f,
                    "observed_period":     "unresolved",
                    "count":               none_cnt,
                    "fraction":            f"{none_cnt/total:.6f}",
                    "drives_full_tau_period": "N/A",
                    "synchronization_role": "sub-field did not return within cap",
                    "cluster":             cluster,
                    "note": f"cap={CAP}; sub-field period > cap",
                })

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    fieldnames = [
        "operator", "cluster", "subfield", "observed_period",
        "count", "fraction", "drives_full_tau_period",
        "synchronization_role", "note",
    ]
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"[sf] Wrote {OUTPUT_CSV}")


def write_md(data: dict) -> None:
    res = data["results"]

    def fmt(v, prec=3):
        if v is None:
            return "N/A"
        if isinstance(v, float) and v != v:
            return "N/A"
        if isinstance(v, float):
            return f"{v:.{prec}f}"
        return str(v)

    lines = [
        "# Prime Transport — Tau Sub-Field Periodicity Decomposition",
        "",
        "**Audit type:** Read-only bounded sub-field periodicity decomposition",
        "**Surface:** `geometry_native_operator_model_v25`, `bounded_operator_surface_v25(depth=8)`",
        f"**Total surface states:** {data['surface_n']:,}",
        "",
        "---",
        "",
        "## Sample and method",
        "",
        "| Parameter | Value |",
        "|---|---|",
        f"| Sample reuse | **Yes** — exact same seeds as prior tau orbit audits |",
        f"| Samples per stratum per operator | {SAMPLE_PER_STRATUM} |",
        f"| Total seeds per operator | {SAMPLE_PER_STRATUM * 2} |",
        f"| Orbit cap | {CAP} |",
        f"| Random seed | {RANDOM_SEED} |",
        "",
        "For each sampled seed, the orbit is iterated to cap=1000 and the",
        "first-return step for each of the four tau sub-fields is recorded independently.",
        "The full tau return step (all sub-fields simultaneously) is also recorded.",
        "",
        "**Synchronization type classification per orbit:**",
        "- `pure_lcm`: full tau period == LCM of all four sub-field periods",
        "- `single_dominant`: full tau period == max of sub-field periods (one bottleneck)",
        "- `lcm_multiple`: full tau period is a multiple of LCM(sub-periods) but not equal",
        "- `complex`: full tau period doesn't fit the above patterns",
        "- `incomplete`: at least one sub-field did not return within cap",
        "- `tau_unresolved`: tau itself did not return within cap",
        "",
        "---",
        "",
        "## Per-operator sub-field period summary",
        "",
    ]

    for op_name, _, cluster in OPERATORS:
        r   = res[op_name]
        total = r["sample"]
        sf_c  = r["sf_counters"]
        sf_no = r["sf_none"]
        bt    = r["bottleneck_ctr"]
        sc    = r["sync_counter"]

        lines += [
            f"### {op_name} ({cluster})",
            "",
            f"Sample: {total}",
            "",
            "**Per-sub-field dominant period and top-3 distribution:**",
            "",
            "| Sub-field | canonical modulus | top-1 period | top-1 fraction | top-2 | top-3 | unresolved |",
            "|---|---|---|---|---|---|---|",
        ]
        canon = {"swap": 2, "coupled": 5, "twist": 2, "lift": 12}
        for f in SUBFIELDS:
            top3 = sf_c[f].most_common(3)
            t1  = f"{top3[0][0]} ({top3[0][1]/total:.3f})" if len(top3) >= 1 else "—"
            t2  = f"{top3[1][0]} ({top3[1][1]/total:.3f})" if len(top3) >= 2 else "—"
            t3  = f"{top3[2][0]} ({top3[2][1]/total:.3f})" if len(top3) >= 3 else "—"
            dom_p = top3[0][0] if top3 else None
            dom_f = top3[0][1]/total if top3 else 0.0
            lines.append(
                f"| {f:10s} | {canon[f]} "
                f"| {dom_p} | {dom_f:.4f} "
                f"| {t2} | {t3} "
                f"| {sf_no[f]/total:.4f} |"
            )

        # Synchronization breakdown
        lines += [
            "",
            "**Synchronization type distribution:**",
            "",
            "| Sync type | count | fraction |",
            "|---|---|---|",
        ]
        for stype, cnt in sc.most_common():
            lines.append(f"| {stype} | {cnt} | {cnt/total:.4f} |")

        # Bottleneck distribution
        lines += [
            "",
            "**Bottleneck sub-field frequency** (sub-field whose period equals max sub-period):",
            "",
            "| Sub-field | bottleneck count | fraction |",
            "|---|---|---|",
        ]
        for f in SUBFIELDS:
            cnt = bt.get(f, 0)
            lines.append(f"| {f} | {cnt} | {cnt/total:.4f} |")

        lines.append("")

    # Summary table across operators
    lines += [
        "---",
        "",
        "## Cross-operator summary",
        "",
        "| Operator | Cluster | primary bottleneck | top sync type | pure_lcm fraction | single_dominant fraction |",
        "|---|---|---|---|---|---|",
    ]
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        total = r["sample"]
        bt  = r["bottleneck_ctr"]
        sc  = r["sync_counter"]
        top_bot = bt.most_common(1)
        top_syn = sc.most_common(1)
        primary_bot = top_bot[0][0] if top_bot else "none"
        top_syn_label = top_syn[0][0] if top_syn else "none"
        pure_lcm_frac = sc.get("pure_lcm", 0) / total
        single_dom_frac = sc.get("single_dominant", 0) / total
        lines.append(
            f"| {op_name} | {cluster} | {primary_bot} "
            f"| {top_syn_label} "
            f"| {pure_lcm_frac:.4f} "
            f"| {single_dom_frac:.4f} |"
        )

    # Cluster-level means
    nt_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "non-transport"]
    tr_ops = [(nm, res[nm]) for nm, _, cl in OPERATORS if cl == "transport"]

    def clu_mean(ops, fn):
        vals = [fn(r) for _, r in ops]
        return statistics.mean(vals) if vals else float("nan")

    nt_pure_lcm = clu_mean(nt_ops, lambda r: r["sync_counter"].get("pure_lcm", 0) / r["sample"])
    tr_pure_lcm = clu_mean(tr_ops, lambda r: r["sync_counter"].get("pure_lcm", 0) / r["sample"])
    nt_single_dom = clu_mean(nt_ops, lambda r: r["sync_counter"].get("single_dominant", 0) / r["sample"])
    tr_single_dom = clu_mean(tr_ops, lambda r: r["sync_counter"].get("single_dominant", 0) / r["sample"])

    lines += [
        "",
        "| Metric | non-transport mean | transport mean |",
        "|---|---|---|",
        f"| pure_lcm fraction     | {nt_pure_lcm:.4f} | {tr_pure_lcm:.4f} |",
        f"| single_dominant fraction | {nt_single_dom:.4f} | {tr_single_dom:.4f} |",
        "",
        "---",
        "",
        "## Interpretation",
        "",
        "### Which sub-field is the primary bottleneck?",
        "",
    ]

    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        total = r["sample"]
        bt  = r["bottleneck_ctr"]
        top_bot = bt.most_common(2)
        if top_bot:
            primary = top_bot[0][0]
            primary_frac = top_bot[0][1] / total
            secondary = top_bot[1][0] if len(top_bot) > 1 else "none"
            secondary_frac = top_bot[1][1] / total if len(top_bot) > 1 else 0.0
            lines.append(
                f"- **{op_name}** ({cluster}): primary bottleneck = `{primary}` "
                f"({primary_frac:.3f} of orbits); secondary = `{secondary}` ({secondary_frac:.3f})"
            )

    lines += [
        "",
        "### LCM-60 arithmetic alignment: which sub-field drives it?",
        "",
        "The prior audit showed strong `divides_60` enrichment across operators.",
        "The tau LCM of 60 = lcm(2, 5, 2, 12).",
        "If `lift_phase` (mod 12) is the dominant bottleneck, then the 60 alignment",
        "is primarily driven by the lift sub-field.",
        "If `coupled_phase` (mod 5) is the bottleneck, it is driven by the coupled sub-field.",
        "Pure-LCM synchronization means the 60 arises from combining multiple sub-fields.",
        "",
    ]

    # Determine LCM driver
    lift_dominant_ops = []
    coupled_dominant_ops = []
    lcm_sync_ops = []
    for op_name, _, cluster in OPERATORS:
        r = res[op_name]
        sc = r["sync_counter"]
        bt = r["bottleneck_ctr"]
        total = r["sample"]
        top_bot = bt.most_common(1)
        if top_bot and top_bot[0][0] == "lift" and top_bot[0][1] / total > 0.30:
            lift_dominant_ops.append(op_name)
        if top_bot and top_bot[0][0] == "coupled" and top_bot[0][1] / total > 0.30:
            coupled_dominant_ops.append(op_name)
        if sc.get("pure_lcm", 0) / total > 0.30:
            lcm_sync_ops.append(op_name)

    if lift_dominant_ops:
        lines.append(
            f"Operators where `lift_phase` is primary bottleneck (>30% of orbits): "
            f"**{', '.join(lift_dominant_ops)}**. "
            "The LCM-60 alignment for these operators is primarily driven by the lift sub-field."
        )
    if coupled_dominant_ops:
        lines.append(
            f"Operators where `coupled_phase` is primary bottleneck (>30%): "
            f"**{', '.join(coupled_dominant_ops)}**."
        )
    if lcm_sync_ops:
        lines.append(
            f"Operators where `pure_lcm` synchronization is dominant (>30%): "
            f"**{', '.join(lcm_sync_ops)}**. "
            "The 60-alignment in these operators arises from composite sub-field synchronization."
        )

    lines += [
        "",
        "### Transport vs non-transport cluster comparison",
        "",
        f"Non-transport pure_lcm mean fraction: {nt_pure_lcm:.4f}",
        f"Transport pure_lcm mean fraction:     {tr_pure_lcm:.4f}",
        f"Non-transport single_dominant mean fraction: {nt_single_dom:.4f}",
        f"Transport single_dominant mean fraction:     {tr_single_dom:.4f}",
        "",
    ]

    if nt_pure_lcm > tr_pure_lcm + 0.05:
        lines.append(
            "**Non-transport operators are more often pure-LCM synchronized** — their tau "
            "periods arise from combining all four sub-field periods. Transport operators "
            "show more complex or incomplete synchronization."
        )
    elif tr_pure_lcm > nt_pure_lcm + 0.05:
        lines.append(
            "**Transport operators are more often pure-LCM synchronized** — despite their "
            "higher tau orbit dispersion, their tau period structure is more compositionally "
            "regular at the sub-field level."
        )
    else:
        lines.append(
            "Transport and non-transport operators show similar pure-LCM synchronization rates.",
        )

    lines += [
        "",
        "### What this means for the rebuilt algebra",
        "",
        "The sub-field decomposition reveals which component of the tau holonomy structure",
        "acts as the period-setting bottleneck for each operator. The `lift_phase` sub-field",
        "(mod 12) has the widest modulus and the most variable step size, making it the",
        "natural candidate for the dominant period driver across all operators.",
        "The `coupled_phase` sub-field (mod 5, variable step from semiprime interaction)",
        "can introduce non-trivial period contributions when its step size is coprime to",
        "other sub-field periods.",
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
        "| Arithmetic evidence strength after decomposition | see sync type fractions above |",
        "",
        "---",
        "",
        "## Recommended next step",
        "",
        "Run a bounded **coupled_phase step-size distribution audit**: "
        "for each operator, compute the distribution of the effective coupled_phase step "
        "sizes observed across sampled states (`(query_semiprime × binding_semiprime) % 5`), "
        "and determine whether step-size concentration or variation explains the different "
        "coupled_phase period structures observed between transport and non-transport operators.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[sf] Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    data = run_decomposition(depth=8)
    write_csv(data)
    write_md(data)
