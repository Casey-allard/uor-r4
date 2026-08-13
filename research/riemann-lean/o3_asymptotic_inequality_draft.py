#!/usr/bin/env python3
"""Draft explicit O3 asymptotic inequality conditions and exponent-gap analysis."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 asymptotic inequality draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-scaffold", default="research/output/o3_ref_bridge_asymptotic_scaffold_2026-02-17.json")
    ap.add_argument("--o3-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--a-h-target", type=float, default=2.0)
    ap.add_argument("--output", default="research/output/o3_asymptotic_inequality_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    s = _read_json(args.o3_scaffold)
    b = _read_json(args.o3_budget)
    q = _read_json(args.o3_sign)

    cc = s["lemma_o3_a3_ref"]["current_constant_candidates"]
    a_eta = float(cc["A_eta"])
    a_h_emp = float(cc["A_H"])
    c_h_budget = float(b["o3_theorem_budget"]["C_H_theorem_budget"])
    c_eta_budget = float(b["o3_theorem_budget"]["C_eta_theorem_budget"])
    eps_sign = float(q["theorem_assumption_candidates"]["eps_sign"])
    neg_cap = float(q["theorem_assumption_candidates"]["neg_over_abs_cap"])

    # Naive chain: if eta <= C_eta log^A_eta and E2/x <= C_E2 log^A_E2,
    # then |H| <= sqrt((1+eta)E2/x) gives A_H <= (A_eta + A_E2)/2.
    # Required A_E2 bounds:
    a_e2_needed_for_target = 2.0 * float(args.a_h_target) - a_eta
    a_e2_needed_for_emp = 2.0 * a_h_emp - a_eta

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_scaffold": args.o3_scaffold,
            "o3_budget": args.o3_budget,
            "o3_sign": args.o3_sign,
            "a_h_target": float(args.a_h_target),
        },
        "o3_asymptotic_draft": {
            "inequality_template": (
                "Assume eta_+(x;W) <= C_eta (log x)^A_eta and E2(x;W)/x <= C_E2 (log x)^A_E2 "
                "for all x>=x0 and W in wheel family; then "
                "|H_W(x)| <= sqrt((1+eta_+(x;W))E2(x;W)/x)."
            ),
            "current_constants": {
                "A_eta": a_eta,
                "C_eta_budget": c_eta_budget,
                "A_H_empirical": a_h_emp,
                "C_H_budget": c_h_budget,
                "eps_sign": eps_sign,
                "neg_over_abs_cap": neg_cap,
            },
            "exponent_gap_analysis": {
                "a_e2_needed_for_target_AH": a_e2_needed_for_target,
                "a_e2_needed_for_empirical_AH": a_e2_needed_for_emp,
                "interpretation": (
                    "Under naive product-exponent propagation, achieving low A_H requires "
                    "strong control on E2/x exponent. This indicates proof must exploit "
                    "structured cancellation/sign control beyond loose worst-case mixing."
                ),
            },
            "proof_obligations_for_next_step": [
                "Prove explicit sign-sensitive offdiag inequality using quantified eps_sign and neg_over_abs_cap assumptions.",
                "Derive asymptotic E2/x bound with explicit exponent A_E2 and constants.",
                "Show combined chain yields an A_H class compatible with O5 endpoint target.",
            ],
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    gap = payload["o3_asymptotic_draft"]["exponent_gap_analysis"]
    lines = [
        "# O3 Asymptotic Inequality Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Core Template",
        "",
        payload["o3_asymptotic_draft"]["inequality_template"],
        "",
        "## Current Constants",
        "",
        f"- `A_eta = {a_eta}`",
        f"- `C_eta_budget = {c_eta_budget}`",
        f"- `A_H_empirical = {a_h_emp}`",
        f"- `C_H_budget = {c_h_budget}`",
        f"- `eps_sign = {eps_sign}`",
        f"- `neg_over_abs_cap = {neg_cap}`",
        "",
        "## Exponent Gap Analysis",
        "",
        f"- `A_E2 needed for target A_H={args.a_h_target}: {gap['a_e2_needed_for_target_AH']}`",
        f"- `A_E2 needed for empirical A_H={a_h_emp}: {gap['a_e2_needed_for_empirical_AH']}`",
        f"- {gap['interpretation']}",
        "",
        "## Forced O3 Next Obligations",
        "",
    ]
    for x in payload["o3_asymptotic_draft"]["proof_obligations_for_next_step"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
