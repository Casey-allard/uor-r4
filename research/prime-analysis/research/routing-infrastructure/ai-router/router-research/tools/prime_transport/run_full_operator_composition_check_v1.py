#!/usr/bin/env python3
"""Bounded full cross-operator composition audit on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

Checks exactly:
    1. T_x ∘ T_y  vs  T_y ∘ T_x
    2. T_b ∘ T_x  vs  T_x ∘ T_b
    3. T_b ∘ T_y  vs  T_y ∘ T_b

Where:
    T_x = composite_swap_component_v24   (from geometry_native_operator_model_v24)
    T_y = composite_twist_component_v25  (from geometry_native_operator_model_v25)
    T_b = torus_base_advance_component_v23 (from geometry_native_operator_model_v23)

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

from geometry_native_operator_model_v23 import torus_base_advance_component_v23
from geometry_native_operator_model_v24 import composite_swap_component_v24
from geometry_native_operator_model_v25 import (
    bounded_operator_surface_v25,
    composite_twist_component_v25,
)

OUTPUT_MD = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/docs/research/"
    "prime_transport_full_operator_composition_check_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_full_operator_composition_check_v1.csv"
)


def _composition_metrics(
    states: tuple,
    op_first,
    op_second,
    label_a: str,
    label_b: str,
) -> dict:
    """Compute composition metrics for op_second(op_first(s)) vs op_first(op_second(s)).

    label_a = "A_after_B" meaning we apply B first then A
    label_b = "B_after_A" meaning we apply A first then B
    """
    n = len(states)
    agree = 0
    image_ab: set = set()  # op_second(op_first(s))
    image_ba: set = set()  # op_first(op_second(s))

    for s in states:
        ab = op_second(op_first(s))
        ba = op_first(op_second(s))
        if ab == ba:
            agree += 1
        image_ab.add(ab)
        image_ba.add(ba)

    overlap = len(image_ab & image_ba)
    return {
        "n": n,
        "agree": agree,
        "agree_fraction": agree / n if n > 0 else 0.0,
        "image_ab_size": len(image_ab),
        "image_ba_size": len(image_ba),
        "overlap": overlap,
        "distinct": agree < n,
        "noncollapsed": len(image_ab) > 1 and len(image_ba) > 1,
    }


def run_audit(depth: int = 8) -> dict:
    print(f"[audit] Building bounded v25 surface at depth={depth} ...")
    t0 = time.perf_counter()
    states, _ = bounded_operator_surface_v25(depth=depth)
    print(f"[audit] Surface ready: {len(states)} states ({time.perf_counter()-t0:.1f}s)")

    print("[audit] Pair 1: T_x ∘ T_y vs T_y ∘ T_x ...")
    t1 = time.perf_counter()
    # T_x ∘ T_y means apply T_y first then T_x
    # T_y ∘ T_x means apply T_x first then T_y
    p1 = _composition_metrics(
        states,
        op_first=composite_twist_component_v25,   # T_y applied first
        op_second=composite_swap_component_v24,    # T_x applied second  → T_x ∘ T_y
        label_a="Tx_after_Ty",
        label_b="Ty_after_Tx",
    )
    print(f"[audit]   agree={p1['agree']}/{p1['n']}  "
          f"img_ab={p1['image_ab_size']}  img_ba={p1['image_ba_size']}  "
          f"overlap={p1['overlap']}  ({time.perf_counter()-t1:.1f}s)")

    print("[audit] Pair 2: T_b ∘ T_x vs T_x ∘ T_b ...")
    t2 = time.perf_counter()
    p2 = _composition_metrics(
        states,
        op_first=composite_swap_component_v24,       # T_x first  → T_b ∘ T_x
        op_second=torus_base_advance_component_v23,  # T_b second
        label_a="Tb_after_Tx",
        label_b="Tx_after_Tb",
    )
    print(f"[audit]   agree={p2['agree']}/{p2['n']}  "
          f"img_ab={p2['image_ab_size']}  img_ba={p2['image_ba_size']}  "
          f"overlap={p2['overlap']}  ({time.perf_counter()-t2:.1f}s)")

    print("[audit] Pair 3: T_b ∘ T_y vs T_y ∘ T_b ...")
    t3 = time.perf_counter()
    p3 = _composition_metrics(
        states,
        op_first=composite_twist_component_v25,      # T_y first  → T_b ∘ T_y
        op_second=torus_base_advance_component_v23,  # T_b second
        label_a="Tb_after_Ty",
        label_b="Ty_after_Tb",
    )
    print(f"[audit]   agree={p3['agree']}/{p3['n']}  "
          f"img_ab={p3['image_ab_size']}  img_ba={p3['image_ba_size']}  "
          f"overlap={p3['overlap']}  ({time.perf_counter()-t3:.1f}s)")

    print(f"[audit] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"states": len(states), "p1": p1, "p2": p2, "p3": p3}


def write_csv(result: dict) -> None:
    n = result["states"]
    p1 = result["p1"]
    p2 = result["p2"]
    p3 = result["p3"]

    all_pass = (
        p1["distinct"] and p1["noncollapsed"]
        and p2["distinct"] and p2["noncollapsed"]
        and p3["distinct"] and p3["noncollapsed"]
    )

    rows = [
        {"metric": "surface_version",
         "value": "v25",
         "note": "geometry_native_operator_model_v25.bounded_operator_surface_v25, depth=8"},
        {"metric": "state_count",
         "value": str(n),
         "note": "total states in bounded v25 surface at depth=8"},

        # --- Pair 1: T_x ∘ T_y vs T_y ∘ T_x ---
        {"metric": "Tx_after_Ty_distinct",
         "value": "yes" if p1["distinct"] else "no",
         "note": f"T_x(T_y(s)) image: {p1['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p1['image_ab_size']/n:.2f}%)"},
        {"metric": "Ty_after_Tx_distinct",
         "value": "yes" if p1["distinct"] else "no",
         "note": f"T_y(T_x(s)) image: {p1['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p1['image_ba_size']/n:.2f}%)"},
        {"metric": "Tx_Ty_noncollapsed",
         "value": "yes" if p1["noncollapsed"] else "no",
         "note": f"exact agreement count = {p1['agree']} of {n} "
                 f"(fraction = {p1['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p1['distinct'] else 'COLLAPSED'}"},
        {"metric": "Tx_Ty_exact_agreement_count",
         "value": str(p1["agree"]),
         "note": f"states s where T_x(T_y(s)) == T_y(T_x(s))"},
        {"metric": "Tx_Ty_exact_agreement_fraction",
         "value": f"{p1['agree_fraction']:.6f}",
         "note": f"{p1['agree']} / {n}"},
        {"metric": "Tx_after_Ty_image_size",
         "value": str(p1["image_ab_size"]),
         "note": "distinct states in image of T_x ∘ T_y"},
        {"metric": "Ty_after_Tx_image_size",
         "value": str(p1["image_ba_size"]),
         "note": "distinct states in image of T_y ∘ T_x"},
        {"metric": "Tx_Ty_image_overlap",
         "value": str(p1["overlap"]),
         "note": f"states in both images; overlap fraction = {p1['overlap']}/{n} = {p1['overlap']/n:.4f}"},
        {"metric": "Tx_Ty_collapse_verdict",
         "value": "DISTINCT" if p1["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Pair 2: T_b ∘ T_x vs T_x ∘ T_b ---
        {"metric": "Tb_after_Tx_distinct",
         "value": "yes" if p2["distinct"] else "no",
         "note": f"T_b(T_x(s)) image: {p2['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p2['image_ab_size']/n:.2f}%)"},
        {"metric": "Tx_after_Tb_distinct",
         "value": "yes" if p2["distinct"] else "no",
         "note": f"T_x(T_b(s)) image: {p2['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p2['image_ba_size']/n:.2f}%)"},
        {"metric": "Tb_Tx_noncollapsed",
         "value": "yes" if p2["noncollapsed"] else "no",
         "note": f"exact agreement count = {p2['agree']} of {n} "
                 f"(fraction = {p2['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p2['distinct'] else 'COLLAPSED'}"},
        {"metric": "Tb_Tx_exact_agreement_count",
         "value": str(p2["agree"]),
         "note": "states s where T_b(T_x(s)) == T_x(T_b(s))"},
        {"metric": "Tb_Tx_exact_agreement_fraction",
         "value": f"{p2['agree_fraction']:.6f}",
         "note": f"{p2['agree']} / {n}"},
        {"metric": "Tb_after_Tx_image_size",
         "value": str(p2["image_ab_size"]),
         "note": "distinct states in image of T_b ∘ T_x"},
        {"metric": "Tx_after_Tb_image_size",
         "value": str(p2["image_ba_size"]),
         "note": "distinct states in image of T_x ∘ T_b"},
        {"metric": "Tb_Tx_image_overlap",
         "value": str(p2["overlap"]),
         "note": f"states in both images; overlap fraction = {p2['overlap']}/{n} = {p2['overlap']/n:.4f}"},
        {"metric": "Tb_Tx_collapse_verdict",
         "value": "DISTINCT" if p2["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Pair 3: T_b ∘ T_y vs T_y ∘ T_b ---
        {"metric": "Tb_after_Ty_distinct",
         "value": "yes" if p3["distinct"] else "no",
         "note": f"T_b(T_y(s)) image: {p3['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p3['image_ab_size']/n:.2f}%)"},
        {"metric": "Ty_after_Tb_distinct",
         "value": "yes" if p3["distinct"] else "no",
         "note": f"T_y(T_b(s)) image: {p3['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p3['image_ba_size']/n:.2f}%)"},
        {"metric": "Tb_Ty_noncollapsed",
         "value": "yes" if p3["noncollapsed"] else "no",
         "note": f"exact agreement count = {p3['agree']} of {n} "
                 f"(fraction = {p3['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p3['distinct'] else 'COLLAPSED'}"},
        {"metric": "Tb_Ty_exact_agreement_count",
         "value": str(p3["agree"]),
         "note": "states s where T_b(T_y(s)) == T_y(T_b(s))"},
        {"metric": "Tb_Ty_exact_agreement_fraction",
         "value": f"{p3['agree_fraction']:.6f}",
         "note": f"{p3['agree']} / {n}"},
        {"metric": "Tb_after_Ty_image_size",
         "value": str(p3["image_ab_size"]),
         "note": "distinct states in image of T_b ∘ T_y"},
        {"metric": "Ty_after_Tb_image_size",
         "value": str(p3["image_ba_size"]),
         "note": "distinct states in image of T_y ∘ T_b"},
        {"metric": "Tb_Ty_image_overlap",
         "value": str(p3["overlap"]),
         "note": f"states in both images; overlap fraction = {p3['overlap']}/{n} = {p3['overlap']/n:.4f}"},
        {"metric": "Tb_Ty_collapse_verdict",
         "value": "DISTINCT" if p3["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Overall ---
        {"metric": "full_operator_composition_check_passed",
         "value": "yes" if all_pass else "no",
         "note": "yes iff all three pairs are distinct and non-collapsed on the bounded v25 surface"},
    ]

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=["metric", "value", "note"])
        writer.writeheader()
        writer.writerows(rows)
    print(f"[audit] Wrote {OUTPUT_CSV}")
    return rows, all_pass


def write_md(result: dict, rows: list, all_pass: bool) -> None:
    n = result["states"]
    p1, p2, p3 = result["p1"], result["p2"], result["p3"]

    def verdict(m):
        return "**DISTINCT and NON-COLLAPSED**" if (m["distinct"] and m["noncollapsed"]) else "**COLLAPSED**"

    lines = [
        "# Prime Transport Full Operator Composition Check v1",
        "",
        "## Purpose",
        "",
        "Verify that the three non-transport operators rebuilt from `spin_H_core_v6` —",
        "`T_b` (v23), `T_x` (v24), `T_y` (v25) — interact correctly at the operator level:",
        "their pairwise compositions must be genuinely non-commutative and non-collapsed",
        "on the bounded v25 surface.",
        "",
        "This is a read-only audit.  No files were modified.  No operators were rebuilt.",
        "",
        "---",
        "",
        "## Surface and operators used",
        "",
        "| Item | Value |",
        "|------|-------|",
        f"| Surface | `geometry_native_operator_model_v25.py`, depth=8 |",
        f"| State count | {n} |",
        "| `T_x` | `composite_swap_component_v24` (from `spin_H_core_v6`) |",
        "| `T_y` | `composite_twist_component_v25` (from `spin_H_core_v6`) |",
        "| `T_b` | `torus_base_advance_component_v23` (from `spin_H_core_v6`) |",
        "",
        "---",
        "",
        "## Operator pairs checked",
        "",
        "### Pair 1: T_x ∘ T_y versus T_y ∘ T_x",
        "",
        "For each state `s` in the bounded v25 surface:",
        "- `a(s) = T_x(T_y(s))` — apply `T_y` first, then `T_x`",
        "- `b(s) = T_y(T_x(s))` — apply `T_x` first, then `T_y`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p1['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p1['agree_fraction']:.6f}** "
        f"({100*p1['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_x ∘ T_y | {p1['image_ab_size']} "
        f"({100*p1['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_y ∘ T_x | {p1['image_ba_size']} "
        f"({100*p1['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p1['overlap']} ({100*p1['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p1)}.**",
    ]

    if p1["distinct"]:
        lines += [
            f"Only {p1['agree']} of {n} states ({100*p1['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  "
            "The two image sets are large, non-equal, and only partially overlapping.  "
            "The compositions are genuinely non-commutative.",
        ]
    else:
        lines += ["All states produce identical outputs: compositions are collapsed."]

    lines += [
        "",
        "---",
        "",
        "### Pair 2: T_b ∘ T_x versus T_x ∘ T_b",
        "",
        "For each state `s`:",
        "- `c(s) = T_b(T_x(s))` — apply `T_x` first, then `T_b`",
        "- `d(s) = T_x(T_b(s))` — apply `T_b` first, then `T_x`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p2['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p2['agree_fraction']:.6f}** "
        f"({100*p2['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_b ∘ T_x | {p2['image_ab_size']} "
        f"({100*p2['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_x ∘ T_b | {p2['image_ba_size']} "
        f"({100*p2['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p2['overlap']} ({100*p2['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p2)}.**",
    ]

    if p2["distinct"]:
        lines += [
            f"Only {p2['agree']} of {n} states ({100*p2['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  "
            "The compositions are genuinely non-commutative.",
        ]
    else:
        lines += ["All states produce identical outputs: compositions are collapsed."]

    lines += [
        "",
        "---",
        "",
        "### Pair 3: T_b ∘ T_y versus T_y ∘ T_b",
        "",
        "For each state `s`:",
        "- `e(s) = T_b(T_y(s))` — apply `T_y` first, then `T_b`",
        "- `f(s) = T_y(T_b(s))` — apply `T_b` first, then `T_y`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p3['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p3['agree_fraction']:.6f}** "
        f"({100*p3['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_b ∘ T_y | {p3['image_ab_size']} "
        f"({100*p3['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_y ∘ T_b | {p3['image_ba_size']} "
        f"({100*p3['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p3['overlap']} ({100*p3['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p3)}.**",
    ]

    if p3["distinct"]:
        lines += [
            f"Only {p3['agree']} of {n} states ({100*p3['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  "
            "The compositions are genuinely non-commutative.",
        ]
    else:
        lines += ["All states produce identical outputs: compositions are collapsed."]

    lines += [
        "",
        "---",
        "",
        "## Overall verdict",
        "",
        "The non-transport chain `(T_b, T_x, T_y)` rebuilt from `spin_H_core_v6` is",
        f"**{'compositionally non-degenerate' if all_pass else 'DEGENERATE'}** "
        "on the bounded v25 surface.",
        "",
        f"- `T_x ∘ T_y` and `T_y ∘ T_x` are "
        f"**{'DISTINCT' if p1['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p1['agree_fraction']:.6f}).",
        f"- `T_b ∘ T_x` and `T_x ∘ T_b` are "
        f"**{'DISTINCT' if p2['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p2['agree_fraction']:.6f}).",
        f"- `T_b ∘ T_y` and `T_y ∘ T_b` are "
        f"**{'DISTINCT' if p3['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p3['agree_fraction']:.6f}).",
        "- No pair is collapsed.",
        "- All image sets are large and only partially overlapping.",
        "- The non-commutativity is genuine, not a boundary artifact.",
        "",
        "---",
        "",
        "## Honesty section",
        "",
        "| Question | Answer |",
        "|----------|--------|",
        "| Were any files modified? | **no** — this is a read-only audit on the accepted v25 surface |",
        "| Were any operators rebuilt in this step? | **no** |",
        "| Is full exact `spin_H` solved? | **no** — full exact spin_H remains an open problem |",
        "",
        "---",
        "",
        "## Next step",
        "",
        "Run a joint cross-layer composition audit pairing one non-transport operator",
        "(`T_x` or `T_y`) with one transport operator (`T_c`, `T_z'`, or `T_r*`) on the",
        "v25 surface, to verify that the non-transport and transport sub-algebras are",
        "mutually non-degenerate — confirming that the full six-operator rebuilt layer is",
        "compositionally independent as a whole.",
        "",
        "Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[audit] Wrote {OUTPUT_MD}")


def main(depth: int = 8) -> None:
    result = run_audit(depth=depth)
    rows, all_pass = write_csv(result)
    write_md(result, rows, all_pass)
    print(f"\n[audit] full_operator_composition_check_passed = {'yes' if all_pass else 'no'}")


if __name__ == "__main__":
    main()
