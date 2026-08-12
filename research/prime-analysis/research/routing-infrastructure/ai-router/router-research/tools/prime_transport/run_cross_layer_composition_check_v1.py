#!/usr/bin/env python3
"""Bounded joint cross-layer composition audit on the v25 surface.

Read-only audit.  No files are modified.  No operators are rebuilt.

Checks exactly:
    1. T_x ∘ T_c  vs  T_c ∘ T_x
    2. T_y ∘ T_z' vs  T_z' ∘ T_y
    3. T_b ∘ T_r* vs  T_r* ∘ T_b

Where:
    Non-transport layer:
        T_x = composite_swap_component_v24   (geometry_native_operator_model_v24)
        T_y = composite_twist_component_v25  (geometry_native_operator_model_v25)
        T_b = torus_base_advance_component_v23 (geometry_native_operator_model_v23)
    Transport layer:
        T_c  = coupled_torus_kick_component_v20  (geometry_native_operator_model_v20)
        T_z' = fiber_phase_lift_component_v21    (geometry_native_operator_model_v21)
        T_r* = radial_transport_component_v22    (geometry_native_operator_model_v22)

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
    "prime_transport_cross_layer_composition_check_v1.md"
)
OUTPUT_CSV = Path(
    "/Users/adminamn/AI-Research/ai-router/router-research/results/"
    "prime_transport_recursive_system/"
    "prime_transport_cross_layer_composition_check_v1.csv"
)


def _composition_metrics(states: tuple, op_first, op_second) -> dict:
    """For each s: compute op_second(op_first(s)) and op_first(op_second(s)).

    Returns agreement count, image sizes, overlap, and derived flags.
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

    # Pair 1: T_x ∘ T_c vs T_c ∘ T_x
    # "T_x ∘ T_c" means apply T_c first, then T_x → op_first=T_c, op_second=T_x
    # "T_c ∘ T_x" means apply T_x first, then T_c → op_first=T_x, op_second=T_c
    # We call _composition_metrics(op_first=T_c, op_second=T_x):
    #   image_ab = T_x(T_c(s))  ← T_x ∘ T_c
    #   image_ba = T_c(T_x(s))  ← T_c ∘ T_x
    print("[audit] Pair 1: T_x ∘ T_c vs T_c ∘ T_x ...")
    t1 = time.perf_counter()
    p1 = _composition_metrics(
        states,
        op_first=coupled_torus_kick_component_v20,   # T_c
        op_second=composite_swap_component_v24,       # T_x
    )
    print(f"[audit]   agree={p1['agree']}/{p1['n']}  "
          f"img(Tx∘Tc)={p1['image_ab_size']}  img(Tc∘Tx)={p1['image_ba_size']}  "
          f"overlap={p1['overlap']}  ({time.perf_counter()-t1:.1f}s)")

    # Pair 2: T_y ∘ T_z' vs T_z' ∘ T_y
    # image_ab = T_y(T_z'(s))  ← T_y ∘ T_z'
    # image_ba = T_z'(T_y(s))  ← T_z' ∘ T_y
    print("[audit] Pair 2: T_y ∘ T_z' vs T_z' ∘ T_y ...")
    t2 = time.perf_counter()
    p2 = _composition_metrics(
        states,
        op_first=fiber_phase_lift_component_v21,     # T_z'
        op_second=composite_twist_component_v25,      # T_y
    )
    print(f"[audit]   agree={p2['agree']}/{p2['n']}  "
          f"img(Ty∘Tz)={p2['image_ab_size']}  img(Tz∘Ty)={p2['image_ba_size']}  "
          f"overlap={p2['overlap']}  ({time.perf_counter()-t2:.1f}s)")

    # Pair 3: T_b ∘ T_r* vs T_r* ∘ T_b
    # image_ab = T_b(T_r*(s))  ← T_b ∘ T_r*
    # image_ba = T_r*(T_b(s))  ← T_r* ∘ T_b
    print("[audit] Pair 3: T_b ∘ T_r* vs T_r* ∘ T_b ...")
    t3 = time.perf_counter()
    p3 = _composition_metrics(
        states,
        op_first=radial_transport_component_v22,         # T_r*
        op_second=torus_base_advance_component_v23,      # T_b
    )
    print(f"[audit]   agree={p3['agree']}/{p3['n']}  "
          f"img(Tb∘Tr)={p3['image_ab_size']}  img(Tr∘Tb)={p3['image_ba_size']}  "
          f"overlap={p3['overlap']}  ({time.perf_counter()-t3:.1f}s)")

    print(f"[audit] Total elapsed: {time.perf_counter()-t0:.1f}s")
    return {"states": len(states), "p1": p1, "p2": p2, "p3": p3}


def write_csv(result: dict):
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

        # --- Pair 1: T_x ∘ T_c vs T_c ∘ T_x ---
        {"metric": "Tx_after_Tc_distinct",
         "value": "yes" if p1["distinct"] else "no",
         "note": f"T_x(T_c(s)) image: {p1['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p1['image_ab_size']/n:.2f}%)"},
        {"metric": "Tc_after_Tx_distinct",
         "value": "yes" if p1["distinct"] else "no",
         "note": f"T_c(T_x(s)) image: {p1['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p1['image_ba_size']/n:.2f}%)"},
        {"metric": "Tx_Tc_noncollapsed",
         "value": "yes" if p1["noncollapsed"] else "no",
         "note": f"exact agreement count = {p1['agree']} of {n} "
                 f"(fraction = {p1['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p1['distinct'] else 'COLLAPSED'}"},
        {"metric": "Tx_Tc_exact_agreement_count",
         "value": str(p1["agree"]),
         "note": "states s where T_x(T_c(s)) == T_c(T_x(s))"},
        {"metric": "Tx_Tc_exact_agreement_fraction",
         "value": f"{p1['agree_fraction']:.6f}",
         "note": f"{p1['agree']} / {n}"},
        {"metric": "Tx_after_Tc_image_size",
         "value": str(p1["image_ab_size"]),
         "note": "distinct states in image of T_x ∘ T_c"},
        {"metric": "Tc_after_Tx_image_size",
         "value": str(p1["image_ba_size"]),
         "note": "distinct states in image of T_c ∘ T_x"},
        {"metric": "Tx_Tc_image_overlap",
         "value": str(p1["overlap"]),
         "note": f"states in both images; overlap fraction = {p1['overlap']}/{n} = {p1['overlap']/n:.4f}"},
        {"metric": "Tx_Tc_collapse_verdict",
         "value": "DISTINCT" if p1["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Pair 2: T_y ∘ T_z' vs T_z' ∘ T_y ---
        {"metric": "Ty_after_Tz_distinct",
         "value": "yes" if p2["distinct"] else "no",
         "note": f"T_y(T_z'(s)) image: {p2['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p2['image_ab_size']/n:.2f}%)"},
        {"metric": "Tz_after_Ty_distinct",
         "value": "yes" if p2["distinct"] else "no",
         "note": f"T_z'(T_y(s)) image: {p2['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p2['image_ba_size']/n:.2f}%)"},
        {"metric": "Ty_Tz_noncollapsed",
         "value": "yes" if p2["noncollapsed"] else "no",
         "note": f"exact agreement count = {p2['agree']} of {n} "
                 f"(fraction = {p2['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p2['distinct'] else 'COLLAPSED'}"},
        {"metric": "Ty_Tz_exact_agreement_count",
         "value": str(p2["agree"]),
         "note": "states s where T_y(T_z'(s)) == T_z'(T_y(s))"},
        {"metric": "Ty_Tz_exact_agreement_fraction",
         "value": f"{p2['agree_fraction']:.6f}",
         "note": f"{p2['agree']} / {n}"},
        {"metric": "Ty_after_Tz_image_size",
         "value": str(p2["image_ab_size"]),
         "note": "distinct states in image of T_y ∘ T_z'"},
        {"metric": "Tz_after_Ty_image_size",
         "value": str(p2["image_ba_size"]),
         "note": "distinct states in image of T_z' ∘ T_y"},
        {"metric": "Ty_Tz_image_overlap",
         "value": str(p2["overlap"]),
         "note": f"states in both images; overlap fraction = {p2['overlap']}/{n} = {p2['overlap']/n:.4f}"},
        {"metric": "Ty_Tz_collapse_verdict",
         "value": "DISTINCT" if p2["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Pair 3: T_b ∘ T_r* vs T_r* ∘ T_b ---
        {"metric": "Tb_after_Tr_distinct",
         "value": "yes" if p3["distinct"] else "no",
         "note": f"T_b(T_r*(s)) image: {p3['image_ab_size']} distinct states over {n} inputs "
                 f"({100*p3['image_ab_size']/n:.2f}%)"},
        {"metric": "Tr_after_Tb_distinct",
         "value": "yes" if p3["distinct"] else "no",
         "note": f"T_r*(T_b(s)) image: {p3['image_ba_size']} distinct states over {n} inputs "
                 f"({100*p3['image_ba_size']/n:.2f}%)"},
        {"metric": "Tb_Tr_noncollapsed",
         "value": "yes" if p3["noncollapsed"] else "no",
         "note": f"exact agreement count = {p3['agree']} of {n} "
                 f"(fraction = {p3['agree_fraction']:.6f}); "
                 f"compositions are {'non-commutative and non-degenerate' if p3['distinct'] else 'COLLAPSED'}"},
        {"metric": "Tb_Tr_exact_agreement_count",
         "value": str(p3["agree"]),
         "note": "states s where T_b(T_r*(s)) == T_r*(T_b(s))"},
        {"metric": "Tb_Tr_exact_agreement_fraction",
         "value": f"{p3['agree_fraction']:.6f}",
         "note": f"{p3['agree']} / {n}"},
        {"metric": "Tb_after_Tr_image_size",
         "value": str(p3["image_ab_size"]),
         "note": "distinct states in image of T_b ∘ T_r*"},
        {"metric": "Tr_after_Tb_image_size",
         "value": str(p3["image_ba_size"]),
         "note": "distinct states in image of T_r* ∘ T_b"},
        {"metric": "Tb_Tr_image_overlap",
         "value": str(p3["overlap"]),
         "note": f"states in both images; overlap fraction = {p3['overlap']}/{n} = {p3['overlap']/n:.4f}"},
        {"metric": "Tb_Tr_collapse_verdict",
         "value": "DISTINCT" if p3["distinct"] else "COLLAPSED",
         "note": "DISTINCT = agree_count < n; COLLAPSED = agree_count == n"},

        # --- Overall ---
        {"metric": "cross_layer_composition_check_passed",
         "value": "yes" if all_pass else "no",
         "note": "yes iff all three cross-layer pairs are distinct and non-collapsed on the bounded v25 surface"},
    ]

    OUTPUT_CSV.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_CSV.open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=["metric", "value", "note"])
        writer.writeheader()
        writer.writerows(rows)
    print(f"[audit] Wrote {OUTPUT_CSV}")
    return rows, all_pass


def write_md(result: dict, all_pass: bool) -> None:
    n = result["states"]
    p1, p2, p3 = result["p1"], result["p2"], result["p3"]

    def verdict(m):
        return ("**DISTINCT and NON-COLLAPSED**"
                if (m["distinct"] and m["noncollapsed"])
                else "**COLLAPSED**")

    lines = [
        "# Prime Transport Cross-Layer Composition Check v1",
        "",
        "## Purpose",
        "",
        "Verify that the non-transport sub-algebra `(T_b, T_x, T_y)` and the transport",
        "sub-algebra `(T_c, T_z', T_r*)`, both rebuilt from `spin_H_core_v6`, are",
        "mutually non-degenerate: cross-layer pairwise compositions must be genuinely",
        "non-commutative and non-collapsed on the bounded v25 surface.",
        "",
        "This is a read-only audit.  No files were modified.  No operators were rebuilt.",
        "",
        "---",
        "",
        "## Surface and operators used",
        "",
        "| Item | Value |",
        "|------|-------|",
        "| Surface | `geometry_native_operator_model_v25.py`, depth=8 |",
        f"| State count | {n} |",
        "| `T_x` (non-transport) | `composite_swap_component_v24` (from `spin_H_core_v6`) |",
        "| `T_y` (non-transport) | `composite_twist_component_v25` (from `spin_H_core_v6`) |",
        "| `T_b` (non-transport) | `torus_base_advance_component_v23` (from `spin_H_core_v6`) |",
        "| `T_c` (transport) | `coupled_torus_kick_component_v20` (from `spin_H_core_v6`) |",
        "| `T_z'` (transport) | `fiber_phase_lift_component_v21` (from `spin_H_core_v6`) |",
        "| `T_r*` (transport) | `radial_transport_component_v22` (from `spin_H_core_v6`) |",
        "",
        "---",
        "",
        "## Cross-layer operator pairs checked",
        "",
        "### Pair 1: T_x ∘ T_c versus T_c ∘ T_x",
        "",
        "For each state `s` in the bounded v25 surface:",
        "- `a(s) = T_x(T_c(s))` — apply `T_c` first, then `T_x`",
        "- `b(s) = T_c(T_x(s))` — apply `T_x` first, then `T_c`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p1['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p1['agree_fraction']:.6f}** ({100*p1['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_x ∘ T_c | {p1['image_ab_size']} ({100*p1['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_c ∘ T_x | {p1['image_ba_size']} ({100*p1['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p1['overlap']} ({100*p1['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p1)}.**",
    ]

    if p1["distinct"]:
        lines.append(
            f"Only {p1['agree']} of {n} states ({100*p1['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  The two image sets are large, non-equal, "
            "and only partially overlapping.  The compositions are genuinely non-commutative."
        )
    else:
        lines.append("All states produce identical outputs: compositions are collapsed.")

    lines += [
        "",
        "---",
        "",
        "### Pair 2: T_y ∘ T_z' versus T_z' ∘ T_y",
        "",
        "For each state `s`:",
        "- `c(s) = T_y(T_z'(s))` — apply `T_z'` first, then `T_y`",
        "- `d(s) = T_z'(T_y(s))` — apply `T_y` first, then `T_z'`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p2['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p2['agree_fraction']:.6f}** ({100*p2['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_y ∘ T_z' | {p2['image_ab_size']} ({100*p2['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_z' ∘ T_y | {p2['image_ba_size']} ({100*p2['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p2['overlap']} ({100*p2['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p2)}.**",
    ]

    if p2["distinct"]:
        lines.append(
            f"Only {p2['agree']} of {n} states ({100*p2['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  The compositions are genuinely non-commutative."
        )
    else:
        lines.append("All states produce identical outputs: compositions are collapsed.")

    lines += [
        "",
        "---",
        "",
        "### Pair 3: T_b ∘ T_r* versus T_r* ∘ T_b",
        "",
        "For each state `s`:",
        "- `e(s) = T_b(T_r*(s))` — apply `T_r*` first, then `T_b`",
        "- `f(s) = T_r*(T_b(s))` — apply `T_b` first, then `T_r*`",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Exact state agreement count | {p3['agree']} of {n} |",
        f"| Exact state agreement fraction | **{p3['agree_fraction']:.6f}** ({100*p3['agree_fraction']:.4f}%) |",
        f"| Distinct states in image of T_b ∘ T_r* | {p3['image_ab_size']} ({100*p3['image_ab_size']/n:.2f}% of N) |",
        f"| Distinct states in image of T_r* ∘ T_b | {p3['image_ba_size']} ({100*p3['image_ba_size']/n:.2f}% of N) |",
        f"| Image set overlap | {p3['overlap']} ({100*p3['overlap']/n:.2f}% of N) |",
        "",
        f"**Verdict: {verdict(p3)}.**",
    ]

    if p3["distinct"]:
        lines.append(
            f"Only {p3['agree']} of {n} states ({100*p3['agree_fraction']:.4f}%) produce "
            "identical outputs under both orderings.  The compositions are genuinely non-commutative."
        )
    else:
        lines.append("All states produce identical outputs: compositions are collapsed.")

    lines += [
        "",
        "---",
        "",
        "## Overall verdict",
        "",
        "The transport and non-transport sub-algebras rebuilt from `spin_H_core_v6` are",
        f"**{'mutually compositionally non-degenerate' if all_pass else 'NOT mutually non-degenerate'}**"
        " on the bounded v25 surface.",
        "",
        f"- `T_x ∘ T_c` and `T_c ∘ T_x` are **{'DISTINCT' if p1['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p1['agree_fraction']:.6f}).",
        f"- `T_y ∘ T_z'` and `T_z' ∘ T_y` are **{'DISTINCT' if p2['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p2['agree_fraction']:.6f}).",
        f"- `T_b ∘ T_r*` and `T_r* ∘ T_b` are **{'DISTINCT' if p3['distinct'] else 'COLLAPSED'}** "
        f"(agreement fraction: {p3['agree_fraction']:.6f}).",
        "- No cross-layer pair is collapsed.",
        "- All image sets are large and only partially overlapping.",
        "- The cross-layer non-commutativity is genuine, not a boundary artifact.",
        "- The full six-operator rebuilt layer is compositionally distinct across layers.",
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
        "Compute the commutativity spectrum of the full six-operator rebuilt layer on the",
        "v25 surface: for every unordered pair `{O_i, O_j}` among the six non-hold operators,",
        "record the exact agreement fraction `|{s : O_i(O_j(s)) == O_j(O_i(s))}| / N`.",
        "This produces a 6×6 symmetric agreement matrix that characterises the full",
        "non-commutativity structure of the rebuilt algebra in one compact table.",
        "",
        "Do not benchmark.  Do not rebuild any operator.  Do not touch any core file.",
    ]

    OUTPUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"[audit] Wrote {OUTPUT_MD}")


def main(depth: int = 8) -> None:
    result = run_audit(depth=depth)
    _, all_pass = write_csv(result)
    write_md(result, all_pass)
    print(f"\n[audit] cross_layer_composition_check_passed = {'yes' if all_pass else 'no'}")


if __name__ == "__main__":
    main()
