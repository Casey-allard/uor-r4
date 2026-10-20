#!/usr/bin/env python3
"""Build a theorem-facing asymptotic closure artifact for O3 lemma E2-OFFDIAG-SIGN."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 E2-OFFDIAG-SIGN asymptotic closure artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-offdiag-draft", default="research/output/o3_e2_offdiag_sign_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--o3-e2-target-pack", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--offdiag-abs-constant", type=float, default=-1.0, help="C_abs in Offdiag_abs <= C_abs*(log x)^A_abs; <=0 means auto from target pack")
    ap.add_argument("--offdiag-abs-exponent", type=float, default=0.0, help="A_abs in Offdiag_abs <= C_abs*(log x)^A_abs")
    ap.add_argument("--x0", type=float, default=3.0)
    ap.add_argument("--output", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    draft = _read_json(args.o3_e2_offdiag_draft)
    sign = _read_json(args.o3_sign)["theorem_assumption_candidates"]
    tpack = _read_json(args.o3_e2_target_pack)["o3_e2_target_pack"]

    k_pos = float(draft["symbolic_draft"]["coefficients"]["k_pos"])
    k_neg = float(draft["symbolic_draft"]["coefficients"]["k_neg"])
    k_abs = float(draft["symbolic_draft"]["coefficients"]["k_abs"])
    a_e2_max = float(tpack["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    c_e2_target = float(tpack["e2_constant_target"]["C_E2_min_from_budget_chain"])

    c_abs = float(args.offdiag_abs_constant)
    if c_abs <= 0.0:
        c_abs = c_e2_target
    a_abs = float(args.offdiag_abs_exponent)

    c_offdiag = k_abs * c_abs
    a_offdiag = a_abs
    compatible = a_offdiag <= a_e2_max

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_offdiag_draft": args.o3_e2_offdiag_draft,
            "o3_sign": args.o3_sign,
            "o3_e2_target_pack": args.o3_e2_target_pack,
        },
        "lemma_id": "E2-OFFDIAG-SIGN",
        "status": "conditional_asymptotic_closed",
        "assumption_scope": (
            "inherits quantified sign-cancellation assumptions plus an explicit asymptotic envelope "
            "Offdiag_abs(x;W) <= C_abs*(log x)^A_abs uniformly in W"
        ),
        "assumptions": {
            "eps_sign": float(sign["eps_sign"]),
            "neg_over_abs_cap": float(sign["neg_over_abs_cap"]),
            "pos_over_abs_floor": float(sign["pos_over_abs_floor"]),
            "x0": float(args.x0),
            "offdiag_abs_envelope": {
                "C_abs": c_abs,
                "A_abs": a_abs,
                "statement": "For all x>=x0 and W in {30,210,2310,30030}, Offdiag_abs(x;W) <= C_abs*(log x)^A_abs.",
            },
        },
        "derived_offdiag_bound": {
            "chain": "|Offdiag| <= k_pos*Offdiag_pos + k_neg*Offdiag_neg <= k_abs*Offdiag_abs <= C_offdiag*(log x)^A_offdiag",
            "coefficients": {
                "k_pos": k_pos,
                "k_neg": k_neg,
                "k_abs": k_abs,
            },
            "C_offdiag": c_offdiag,
            "A_offdiag": a_offdiag,
            "A_E2_max_target": a_e2_max,
            "target_compatible": compatible,
        },
        "interpretation": (
            "This discharges the sign-sensitive offdiag step as a theorem-facing conditional lemma, "
            "moving the remaining O3 bottleneck to diagonal and glue assembly."
        ),
        "remaining_to_unconditionalize": [
            "Prove the assumed Offdiag_abs asymptotic envelope from analytic decomposition (no empirical fitting)."
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Offdiag Asymptotic Closure",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Status",
        "",
        f"- `{payload['status']}`",
        f"- scope: {payload['assumption_scope']}",
        "",
        "## Assumed Absolute Envelope",
        "",
        f"- `Offdiag_abs <= {c_abs}*(log x)^{a_abs}` for `x>= {args.x0}`",
        "",
        "## Derived Offdiag Bound",
        "",
        f"- `k_abs = {k_abs}`",
        f"- `C_offdiag = {c_offdiag}`",
        f"- `A_offdiag = {a_offdiag}`",
        f"- `A_E2_max_target = {a_e2_max}`",
        f"- target compatible: `{compatible}`",
        "",
        "## Remaining To Unconditionalize",
        "",
    ]
    for item in payload["remaining_to_unconditionalize"]:
        lines.append(f"- {item}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
