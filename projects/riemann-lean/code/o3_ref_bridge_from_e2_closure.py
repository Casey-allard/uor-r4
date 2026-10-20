#!/usr/bin/env python3
"""Derive O3 bridge-growth constants from eta budget and closed E2 assembly."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 derived bridge constants from E2 closure")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--x0", type=float, default=3.0)
    ap.add_argument("--output", default="research/output/o3_ref_bridge_from_e2_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    b = _read_json(args.o3_budget)["o3_theorem_budget"]
    e2 = _read_json(args.o3_e2_full)

    glue = e2.get("explicit_assumptions", {}).get("glue_closure", {})
    if not (glue.get("present") and glue.get("status") == "conditional_asymptotic_closed"):
        raise RuntimeError("o3_e2_full does not contain a closed glue_closure; cannot derive O3 bridge constants")

    a_eta = float(b["A_eta"])
    c_eta = float(b["C_eta_theorem_budget"])
    a_e2 = float(glue["A_E2_if_closed"])
    c_e2 = float(glue["C_E2_if_closed"])

    # For x>=x0>=3, log(x)>=1 so: 1 + C_eta (log x)^A_eta <= (1+C_eta) (log x)^A_eta.
    a_h = 0.5 * (a_eta + a_e2)
    c_h = math.sqrt((1.0 + c_eta) * c_e2)

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_budget": args.o3_budget,
            "o3_e2_full": args.o3_e2_full,
        },
        "derived_o3_bridge": {
            "status": "conditional_derived_from_eta_and_e2",
            "x0": float(args.x0),
            "identity_used": "|H| <= sqrt((1+eta_+) E2/x)",
            "majorization_used": "1 + C_eta (log x)^A_eta <= (1+C_eta)(log x)^A_eta (for x>=x0, log x>=1)",
            "input_constants": {
                "A_eta": a_eta,
                "C_eta": c_eta,
                "A_E2": a_e2,
                "C_E2": c_e2,
            },
            "derived_constants": {
                "A_H_derived": a_h,
                "C_H_derived": c_h,
            },
            "derived_statement": (
                "For all x>=x0 and W in {30,210,2310,30030}, "
                "|H_W^(M_ref)(x)| <= C_H_derived * (log x)^A_H_derived."
            ),
        },
        "interpretation": (
            "This replaces placeholder H-growth constants by constants derived from the closed E2 chain "
            "and theorem-side eta budget. Remaining work is unconditionalization of component envelopes."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 Derived Bridge From E2 Closure",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Inputs",
        "",
        f"- `A_eta = {a_eta}`",
        f"- `C_eta = {c_eta}`",
        f"- `A_E2 = {a_e2}`",
        f"- `C_E2 = {c_e2}`",
        "",
        "## Derived O3 Bridge Constants",
        "",
        f"- `A_H_derived = {a_h}`",
        f"- `C_H_derived = {c_h}`",
        "",
        payload["derived_o3_bridge"]["derived_statement"],
        "",
        payload["interpretation"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
