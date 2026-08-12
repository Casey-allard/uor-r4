#!/usr/bin/env python3
"""Create a theorem-facing scaffold for the O5 M_ref transfer lemma."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _get_a3_constants(a3: Dict[str, Any]) -> Dict[str, float]:
    if "h_transfer_envelope" in a3:
        h = a3["h_transfer_envelope"]
        return {
            "A_H": float(h["A_H"]),
            "C_H": float(h["C_H_from_density_transfer"]),
        }
    if "uplift_constants" in a3:
        u = a3["uplift_constants"]
        return {
            "A_H": float(u["A_H_from_normalized_sum"]),
            "C_H": float(u["C_H_from_normalized_sum"]),
        }
    raise KeyError("unrecognized A3 artifact schema")


def _check_grid_hold(a4: Dict[str, Any], a3c: Dict[str, float]) -> Dict[str, Any]:
    uc = a4["uniform_constants"]
    a_ref = float(uc["a_ref"])
    b_ref = float(uc["b_ref"])
    c0_ref = float(uc["C0_ref"])
    a_h = float(a3c["A_H"])
    c_h = float(a3c["C_H"])

    rows: List[Dict[str, float]] = []
    ratio_max = 0.0
    gap_min = float("inf")
    holds = True
    for row in a4.get("per_n", []):
        lhs = max(float(b["max_abs_e_scaled"]) for b in row.get("per_base", []))
        n_max = int(row["n_max"])
        x = max(3.0, float(n_max))
        lx = __import__("math").log(x)
        rhs = c0_ref + abs(b_ref) + abs(a_ref) * c_h * (lx ** a_h)
        ratio = lhs / rhs if rhs > 0 else float("inf")
        gap = rhs - lhs
        ratio_max = max(ratio_max, ratio)
        gap_min = min(gap_min, gap)
        if ratio > 1.0:
            holds = False
        rows.append(
            {
                "n_max": n_max,
                "lhs": lhs,
                "rhs_mref_transfer": rhs,
                "gap_rhs_minus_lhs": gap,
                "ratio_lhs_over_rhs": ratio,
            }
        )
    return {
        "holds_on_grid": holds,
        "ratio_max": ratio_max,
        "gap_min": gap_min,
        "rows": rows,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O5 M_ref transfer lemma scaffold")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--a1", default="research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json")
    ap.add_argument(
        "--a3",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument("--a4", default="research/output/a4_uniform_assumption_check_refresh_2026-02-17.json")
    ap.add_argument("--target-log-exponent", type=float, default=2.0)
    ap.add_argument("--output", default="research/output/o5_mref_transfer_lemma_scaffold_2026-02-17.json")
    args = ap.parse_args()

    manifest = _read_json(args.canonical_manifest)
    canonical = manifest.get("canonical", {})
    a1 = _read_json(args.a1 or canonical.get("a1", ""))
    a3 = _read_json(args.a3 or canonical.get("a3", ""))
    a4 = _read_json(args.a4 or canonical.get("a4", ""))

    a3c = _get_a3_constants(a3)
    uc = a4["uniform_constants"]
    a_ref = float(uc["a_ref"])
    b_ref = float(uc["b_ref"])
    c0_ref = float(uc["C0_ref"])
    m_ref = int(a4["config"]["m_ref"])
    a_h = float(a3c["A_H"])
    c_h = float(a3c["C_H"])
    target = float(args.target_log_exponent)

    grid = _check_grid_hold(a4, a3c)
    c0_a1 = float(a1["decomposition_constants"]["C0_uplifted"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a1": args.a1,
            "a3": args.a3,
            "a4": args.a4,
        },
        "lemma_o5_mref_transfer": {
            "name": "M_ref Transfer Lemma",
            "quantified_statement": (
                "There exist x0>=3 and constants C0_ref, a_ref, b_ref, C_H, A_H such that "
                "for all x>=x0 and W in {30,210,2310,30030}, "
                "|E(x)|/sqrt(x) <= C0_ref + |b_ref| + |a_ref| C_H (log x)^A_H."
            ),
            "fixed_regime": {
                "m_ref": m_ref,
                "delta_term": "Delta_M = 0 at M=M_ref",
            },
            "constant_pack": {
                "a_ref": a_ref,
                "b_ref": b_ref,
                "C0_ref": c0_ref,
                "C0_a1_uplifted": c0_a1,
                "C_H": c_h,
                "A_H": a_h,
            },
            "rh_endpoint_target": {
                "form": "|psi(x)-x| <= C sqrt(x) (log x)^2",
                "target_exponent": target,
                "meets_exponent_class_with_current_AH": bool(a_h <= target),
                "exponent_gap_AH_minus_target": a_h - target,
            },
        },
        "discharge_map": [
            {
                "dependency": "A1_ref (O1)",
                "required": "Asymptotic proof of reference residual bound at M_ref.",
                "current_status": "grid_validated_only",
                "evidence": "research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json",
            },
            {
                "dependency": "A3_ref (O3)",
                "required": "Asymptotic bridge-growth theorem at M_ref with explicit constants.",
                "current_status": "grid_validated_only",
                "evidence": "research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
            },
            {
                "dependency": "Base uniformity (O4)",
                "required": "Uniform-in-W theorem constants for all x>=x0.",
                "current_status": "grid_validated_only",
                "evidence": "research/output/a4_uniform_assumption_check_refresh_2026-02-17.json",
            },
        ],
        "grid_check_mref_transfer": grid,
        "next_forcing_steps": [
            "Convert A1 reference residual from sampled-sup constant to asymptotic theorem.",
            "Convert A3 m_ref bridge envelope from calibrated holdout fit to asymptotic theorem.",
            "Write final O5 implication theorem that cites a standard RH-equivalent endpoint criterion.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O5 Mref Transfer Lemma Scaffold",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Quantified Statement",
        "",
        payload["lemma_o5_mref_transfer"]["quantified_statement"],
        "",
        "## Fixed Regime",
        "",
        f"- `m_ref = {m_ref}`",
        "- `Delta_M = 0` at `M=M_ref`.",
        "",
        "## Current Constants",
        "",
        f"- `a_ref = {a_ref}`",
        f"- `b_ref = {b_ref}`",
        f"- `C0_ref = {c0_ref}`",
        f"- `C_H = {c_h}`",
        f"- `A_H = {a_h}`",
        "",
        "## RH Endpoint Class Check",
        "",
        f"- target: `|psi(x)-x| <= C sqrt(x) (log x)^{target}`",
        f"- meets exponent class now: `{a_h <= target}`",
        f"- exponent gap `A_H-target`: `{a_h - target}`",
        "",
        "## Dependency Discharge Map",
        "",
        "| dependency | status | required |",
        "|---|---|---|",
    ]
    for row in payload["discharge_map"]:
        lines.append(f"| {row['dependency']} | {row['current_status']} | {row['required']} |")

    lines += [
        "",
        "## Grid Check (Mref Transfer RHS)",
        "",
        f"- holds on grid: `{grid['holds_on_grid']}`",
        f"- ratio max: `{grid['ratio_max']}`",
        f"- gap min: `{grid['gap_min']}`",
        "",
        "## Forced Next Steps",
        "",
    ]
    for step in payload["next_forcing_steps"]:
        lines.append(f"- {step}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
