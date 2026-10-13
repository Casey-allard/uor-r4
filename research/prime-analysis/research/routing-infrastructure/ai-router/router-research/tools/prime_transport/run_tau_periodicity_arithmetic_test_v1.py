#!/usr/bin/env python3
"""Bounded tau-periodicity arithmetic-factor test.

Read-only analysis.  No files are modified.  No operators are rebuilt.
No operator orbits are recomputed — uses prime_transport_tau_periodicity_classes_v1.csv directly.

Canonical arithmetic moduli sourced from geometry_native_spinH_candidate_v3.py:
  SWAP_PHASE_MODULUS_V3   = 2
  COUPLED_PHASE_MODULUS_V3 = 5
  TWIST_PHASE_MODULUS_V3  = 2
  LIFT_PHASE_MODULUS_V3   = 12
  LCM of all four          = 60
  Product                  = 240

Relations tested for each observed cycle length L:
  1. divides_60   : L | 60           (L divides the LCM of all tau sub-moduli)
  2. divides_120  : L | 120          (L divides 2 × LCM)
  3. divides_240  : L | 240          (L divides tau state-space cardinality)
  4. multiple_60  : 60 | L           (L is a multiple of the full tau LCM)
  5. mult_2       : 2 | L            (L divisible by swap/twist sub-modulus)
  6. mult_5       : 5 | L            (L divisible by coupled sub-modulus)
  7. mult_12      : 12 | L           (L divisible by lift sub-modulus)

Null baseline: for each operator, draw 100,000 random integers uniformly from
[1, max_observed_cycle_length] and measure the same alignment rates.
"""

from __future__ import annotations

import csv
import math
import random
from collections import defaultdict
from pathlib import Path

INPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_periodicity_classes_v1.csv"
)
OUTPUT_MD = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research/"
    "prime_transport_tau_periodicity_arithmetic_test_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_tau_periodicity_arithmetic_test_v1.csv"
)

OPERATORS_ORDER = [
    ("T_b",  "non-transport"),
    ("T_x",  "non-transport"),
    ("T_y",  "non-transport"),
    ("T_c",  "transport"),
    ("T_z'", "transport"),
    ("T_r*", "transport"),
]

# ──────────────────────────────────────────────────────────────────────────────
# Canonical moduli — sourced from geometry_native_spinH_candidate_v3.py
# ──────────────────────────────────────────────────────────────────────────────
SWAP_MOD   = 2
COUPLED_MOD = 5
TWIST_MOD  = 2
LIFT_MOD   = 12
TAU_LCM    = math.lcm(SWAP_MOD, COUPLED_MOD, TWIST_MOD, LIFT_MOD)   # = 60
TAU_PROD   = SWAP_MOD * COUPLED_MOD * TWIST_MOD * LIFT_MOD            # = 240

CANONICAL_MODULI = {
    "swap":     SWAP_MOD,
    "coupled":  COUPLED_MOD,
    "twist":    TWIST_MOD,
    "lift":     LIFT_MOD,
    "lcm(all)": TAU_LCM,
    "prod(all)": TAU_PROD,
}

NULL_DRAWS   = 100_000
RANDOM_SEED  = 77

RELATIONS: list[tuple[str, callable]] = [
    ("divides_60",  lambda L: TAU_LCM  % L == 0 if L > 0 else False),
    ("divides_120", lambda L: 120      % L == 0 if L > 0 else False),
    ("divides_240", lambda L: TAU_PROD % L == 0 if L > 0 else False),
    ("multiple_60", lambda L: L % TAU_LCM  == 0),
    ("mult_2",      lambda L: L % SWAP_MOD  == 0),
    ("mult_5",      lambda L: L % COUPLED_MOD == 0),
    ("mult_12",     lambda L: L % LIFT_MOD   == 0),
]


# ──────────────────────────────────────────────────────────────────────────────
# Load prior periodicity CSV
# ──────────────────────────────────────────────────────────────────────────────
def load_periodicity() -> dict[str, dict[int, int]]:
    """Returns {operator: {cycle_length: count}} (excludes 'unresolved' rows)."""
    data: dict[str, dict[int, int]] = {nm: {} for nm, _ in OPERATORS_ORDER}
    with INPUT_CSV.open(encoding="utf-8") as fh:
        reader = csv.DictReader(fh)
        for row in reader:
            op = row["operator"]
            if row["cycle_length"] == "unresolved":
                continue
            cl = int(row["cycle_length"])
            cnt = int(row["count"])
            data.setdefault(op, {})[cl] = cnt
    return data


# ──────────────────────────────────────────────────────────────────────────────
# Alignment computation
# ──────────────────────────────────────────────────────────────────────────────
def alignment(lengths_with_counts: dict[int, int]) -> dict[str, float]:
    """Weighted fraction of (cycle_length, count) pairs matching each relation."""
    total = sum(lengths_with_counts.values())
    if total == 0:
        return {rel_name: 0.0 for rel_name, _ in RELATIONS}
    result = {}
    for rel_name, predicate in RELATIONS:
        matched = sum(cnt for L, cnt in lengths_with_counts.items() if predicate(L))
        result[rel_name] = matched / total
    return result


def null_alignment(max_L: int, n_draws: int, rng: random.Random) -> dict[str, float]:
    """Alignment rate for n_draws uniform integers in [1, max_L]."""
    counts: dict[int, int] = defaultdict(int)
    for _ in range(n_draws):
        counts[rng.randint(1, max_L)] += 1
    return alignment(counts)


# ──────────────────────────────────────────────────────────────────────────────
# Main
# ──────────────────────────────────────────────────────────────────────────────
def run() -> dict:
    rng = random.Random(RANDOM_SEED)
    per_op_data = load_periodicity()

    results = {}
    for op_name, cluster in OPERATORS_ORDER:
        cc = per_op_data.get(op_name, {})
        if not cc:
            results[op_name] = {"cluster": cluster, "rows": []}
            continue

        max_L    = max(cc.keys())
        obs_aln  = alignment(cc)
        null_aln = null_alignment(max_L, NULL_DRAWS, rng)

        rows = []
        for rel_name, predicate in RELATIONS:
            obs_rate  = obs_aln[rel_name]
            null_rate = null_aln[rel_name]
            enrich    = obs_rate / null_rate if null_rate > 1e-9 else float("inf")

            # Which cycle lengths match this relation?
            matching = {L: cnt for L, cnt in cc.items() if predicate(L)}
            matched_frac = sum(matching.values()) / sum(cc.values()) if cc else 0.0

            rows.append({
                "relation":        rel_name,
                "obs_rate":        obs_rate,
                "null_rate":       null_rate,
                "enrichment_ratio": enrich,
                "matching_lengths": sorted(matching.keys()),
                "matched_count":   sum(matching.values()),
                "total_count":     sum(cc.values()),
            })
            print(
                f"  {op_name:5s} {rel_name:12s}  obs={obs_rate:.4f}  "
                f"null={null_rate:.4f}  enrich={enrich:.2f}"
            )

        results[op_name] = {"cluster": cluster, "rows": rows, "cc": cc, "max_L": max_L}

    return results


# ──────────────────────────────────────────────────────────────────────────────
# Write CSV
# ──────────────────────────────────────────────────────────────────────────────
def write_csv(results: dict) -> None:
    fieldnames = [
        "operator", "cluster",
        "cycle_length", "frequency",
        "canonical_modulus", "relation_type", "matches_canonical",
        "null_expected_rate", "observed_rate", "enrichment_ratio", "note",
    ]
    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)

    rows = []
    for op_name, cluster in OPERATORS_ORDER:
        r = results.get(op_name, {})
        cc = r.get("cc", {})
        total = sum(cc.values())
        rel_rows = r.get("rows", [])
        rel_map = {row["relation"]: row for row in rel_rows}

        for rel_name, predicate in RELATIONS:
            row_r = rel_map.get(rel_name, {})
            obs_rate  = row_r.get("obs_rate", float("nan"))
            null_rate = row_r.get("null_rate", float("nan"))
            enrich    = row_r.get("enrichment_ratio", float("nan"))

            # per-cycle-length rows for matching lengths
            matching = {L: cnt for L, cnt in cc.items() if predicate(L)} if cc else {}
            non_matching = {L: cnt for L, cnt in cc.items() if not predicate(L)} if cc else {}

            def fmt_e(v):
                if isinstance(v, float) and v != v:
                    return "nan"
                if v == float("inf"):
                    return "inf"
                return f"{v:.6f}"

            # One aggregated row per (operator, relation)
            rows.append({
                "operator":          op_name,
                "cluster":           cluster,
                "cycle_length":      "ALL",
                "frequency":         total,
                "canonical_modulus": f"LCM={TAU_LCM};PROD={TAU_PROD};sub=[{SWAP_MOD},{COUPLED_MOD},{TWIST_MOD},{LIFT_MOD}]",
                "relation_type":     rel_name,
                "matches_canonical": sum(matching.values()),
                "null_expected_rate": fmt_e(null_rate),
                "observed_rate":     fmt_e(obs_rate),
                "enrichment_ratio":  fmt_e(enrich),
                "note": (
                    f"null_draws={NULL_DRAWS}; rng_seed={RANDOM_SEED}; "
                    f"matching_lengths={sorted(matching.keys())[:10]}"
                ),
            })

    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)
    print(f"Wrote {OUTPUT_CSV}")


# ──────────────────────────────────────────────────────────────────────────────
# Write MD
# ──────────────────────────────────────────────────────────────────────────────
def write_md(results: dict) -> None:
    def fmt(v, prec=4):
        if isinstance(v, float) and v != v:
            return "N/A"
        if v == float("inf"):
            return "∞"
        if isinstance(v, float):
            return f"{v:.{prec}f}"
        return str(v)

    lines = [
        "# Prime Transport — Tau-Periodicity Arithmetic-Factor Test",
        "",
        "**Audit type:** Read-only arithmetic alignment test (no orbit recomputation)",
        "**Input:** `prime_transport_tau_periodicity_classes_v1.csv`",
        "**Canonical moduli source:** `geometry_native_spinH_candidate_v3.py`",
        "",
        "---",
        "",
        "## Canonical arithmetic moduli used",
        "",
        "Sourced directly from `geometry_native_spinH_candidate_v3.py`:",
        "",
        "| Constant | Value | Role |",
        "|---|---|---|",
        f"| `SWAP_PHASE_MODULUS_V3`   | {SWAP_MOD}  | tau.swap_phase period |",
        f"| `COUPLED_PHASE_MODULUS_V3`| {COUPLED_MOD}  | tau.coupled_phase period |",
        f"| `TWIST_PHASE_MODULUS_V3`  | {TWIST_MOD}  | tau.twist_phase period |",
        f"| `LIFT_PHASE_MODULUS_V3`   | {LIFT_MOD} | tau.lift_phase period |",
        f"| LCM of all four           | **{TAU_LCM}** | maximum fixed-step tau period |",
        f"| Product of all four       | **{TAU_PROD}** | tau state-space cardinality |",
        "",
        "Note: `coupled_phase` step size depends on `(query_semiprime × binding_semiprime) % 5`",
        "and `lift_phase` step size depends on `phi` and `spin_h` bits — so the actual orbit",
        "period can exceed 60 for state-dependent stepping.",
        "",
        "---",
        "",
        "## Relations tested",
        "",
        "| Relation | Test | Interpretation |",
        "|---|---|---|",
        "| `divides_60`  | L \\| 60   | L is a divisor of the tau LCM |",
        "| `divides_120` | L \\| 120  | L is a divisor of 2 × tau LCM |",
        "| `divides_240` | L \\| 240  | L is a divisor of tau state-space size |",
        "| `multiple_60` | 60 \\| L   | L is a multiple of the tau LCM |",
        "| `mult_2`      | 2 \\| L    | L divisible by swap/twist sub-modulus |",
        "| `mult_5`      | 5 \\| L    | L divisible by coupled sub-modulus |",
        "| `mult_12`     | 12 \\| L   | L divisible by lift sub-modulus |",
        "",
        "---",
        "",
        "## Null baseline",
        "",
        f"For each operator, {NULL_DRAWS:,} random integers were drawn uniformly from",
        "`[1, max_observed_cycle_length]` and the same alignment fractions were computed.",
        "Enrichment ratio = observed_rate / null_rate.",
        "Enrichment ≥ 2.0 is noted as substantial; ≥ 5.0 is noted as strong.",
        "",
        "---",
        "",
        "## Per-operator results",
        "",
    ]

    for op_name, cluster in OPERATORS_ORDER:
        r = results.get(op_name, {})
        cc = r.get("cc", {})
        max_L = r.get("max_L", "?")
        rows = r.get("rows", [])
        total = sum(cc.values())

        lines += [
            f"### {op_name} ({cluster})",
            "",
            f"Observed cycle lengths: {len(cc)} distinct | max = {max_L} | total sample = {total}",
            "",
            "| Relation | obs rate | null rate | enrichment | assessment |",
            "|---|---|---|---|---|",
        ]
        for row in rows:
            er = row["enrichment_ratio"]
            if er == float("inf"):
                assessment = "null impossible; perfect alignment"
            elif er >= 5.0:
                assessment = "**strong enrichment**"
            elif er >= 2.0:
                assessment = "substantial enrichment"
            elif er >= 1.2:
                assessment = "moderate enrichment"
            elif er >= 0.8:
                assessment = "near null"
            else:
                assessment = "depleted"
            lines.append(
                f"| {row['relation']:12s} "
                f"| {fmt(row['obs_rate'])} "
                f"| {fmt(row['null_rate'])} "
                f"| {fmt(er)} "
                f"| {assessment} |"
            )

        # Top observed cycle lengths and whether they match lcm/divisor structure
        top5 = sorted(cc.items(), key=lambda x: -x[1])[:5]
        div60_top5 = [(L, TAU_LCM % L == 0 if L > 0 else False) for L, _ in top5]
        lines += [
            "",
            "Top-5 cycle lengths and LCM-60 divisor status:",
            "",
            "| Cycle length | count | divides_60? |",
            "|---|---|---|",
        ]
        for L, cnt in top5:
            divides = "yes" if TAU_LCM % L == 0 and L > 0 else "no"
            lines.append(f"| {L} | {cnt} | {divides} |")
        lines.append("")

    # Cluster-level summary
    nt_rows = {op: results[op]["rows"] for op, cl in OPERATORS_ORDER if cl == "non-transport" and "rows" in results.get(op, {})}
    tr_rows = {op: results[op]["rows"] for op, cl in OPERATORS_ORDER if cl == "transport" and "rows" in results.get(op, {})}

    import statistics as _stats

    lines += [
        "---",
        "",
        "## Cluster-level enrichment summary",
        "",
        "Mean enrichment ratio across operators per cluster:",
        "",
        "| Relation | non-transport mean enrich | transport mean enrich |",
        "|---|---|---|",
    ]

    for rel_name, _ in RELATIONS:
        def mean_enrich(op_dict):
            vals = []
            for op, rowlist in op_dict.items():
                for row in rowlist:
                    if row["relation"] == rel_name:
                        er = row["enrichment_ratio"]
                        if er != float("inf") and er == er:  # not inf or nan
                            vals.append(er)
            return _stats.mean(vals) if vals else float("nan")

        nt_e = mean_enrich(nt_rows)
        tr_e = mean_enrich(tr_rows)
        lines.append(
            f"| {rel_name:12s} "
            f"| {fmt(nt_e)} "
            f"| {fmt(tr_e)} |"
        )

    lines += [
        "",
        "---",
        "",
        "## Interpretation",
        "",
    ]

    # Collect notable findings
    notable = []
    for op_name, cluster in OPERATORS_ORDER:
        r = results.get(op_name, {})
        for row in r.get("rows", []):
            er = row["enrichment_ratio"]
            if er >= 5.0 and er != float("inf"):
                notable.append((op_name, cluster, row["relation"], er, row["obs_rate"]))

    if notable:
        lines += [
            "### Operators with strong enrichment (ratio ≥ 5×)",
            "",
        ]
        for op_name, cluster, rel, er, obs in notable:
            lines.append(
                f"- **{op_name}** ({cluster}): `{rel}` — enrichment {fmt(er)}×, "
                f"observed rate {fmt(obs)}"
            )
        lines.append("")

    lines += [
        "### Non-transport cluster",
        "",
        "The non-transport operators (T_b, T_x, T_y) show strong `divides_60` enrichment "
        "if their dominant periods (6, 12, 20, 30, 60) are divisors of the tau LCM (60). "
        "This would confirm that their tau-cycle structure is governed directly by the "
        "fixed-step tau sub-moduli defined in the repo.",
        "",
        "### Transport cluster",
        "",
        "The transport operators (T_c, T_z', T_r*) have more dispersed periodicity structures. "
        "State-dependent coupled_phase and lift_phase steps allow orbit periods to exceed 60. "
        "Enrichment for `mult_2`, `mult_5`, or `mult_12` in the transport cluster would indicate "
        "that the tau sub-moduli still impose partial constraints even when the step is variable.",
        "",
        "### Arithmetic evidence strength",
        "",
    ]

    # Assess overall evidence
    all_div60 = []
    for op_name, cluster in OPERATORS_ORDER:
        r = results.get(op_name, {})
        for row in r.get("rows", []):
            if row["relation"] == "divides_60":
                all_div60.append((op_name, cluster, row["enrichment_ratio"], row["obs_rate"]))

    nt_div60 = [(op, er, obs) for op, cl, er, obs in all_div60 if cl == "non-transport"]
    tr_div60 = [(op, er, obs) for op, cl, er, obs in all_div60 if cl == "transport"]

    nt_div60_mean_enrich = _stats.mean(er for _, er, _ in nt_div60) if nt_div60 else float("nan")
    tr_div60_mean_enrich = _stats.mean(er for _, er, _ in tr_div60 if er != float("inf")) if tr_div60 else float("nan")

    lines += [
        f"Non-transport `divides_60` mean enrichment: **{fmt(nt_div60_mean_enrich)}×**",
        f"Transport `divides_60` mean enrichment:     **{fmt(tr_div60_mean_enrich)}×**",
        "",
    ]

    if nt_div60_mean_enrich > 3.0:
        lines.append(
            "The arithmetic evidence for non-transport tau cycles aligning with the "
            "LCM-60 structure of the tau sub-moduli is **strong** — the enrichment "
            "ratio is well above the null baseline."
        )
    elif nt_div60_mean_enrich > 1.5:
        lines.append(
            "The arithmetic evidence for non-transport tau cycle alignment with "
            "LCM-60 is **moderate** — above null but not conclusive."
        )
    else:
        lines.append(
            "The arithmetic evidence for tau cycle alignment with LCM-60 is **weak** — "
            "the enrichment ratio is close to the null baseline."
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
        "| Is arithmetic evidence strong, weak, or inconclusive? | See enrichment ratios above |",
        "",
        "---",
        "",
        "## Recommended next step",
        "",
        "Run a bounded **tau sub-field periodicity decomposition**: for each operator, "
        "separately track the period of each tau sub-field (swap_phase, coupled_phase, "
        "twist_phase, lift_phase) in the sampled orbits, and report the per-sub-field "
        "period distribution.  This would reveal whether the observed cycle-length "
        "arithmetic structure arises from one dominant sub-field or from the interaction "
        "of all four tau components.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {OUTPUT_MD}")


if __name__ == "__main__":
    print("Loading prior periodicity data ...")
    results = run()
    write_csv(results)
    write_md(results)
    print("Done.")
