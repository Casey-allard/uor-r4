#!/usr/bin/env python3
"""Assess whether sign-cancellation can materially close O3 constant/exponent gaps."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _vals(x: Any) -> List[float]:
    if isinstance(x, list):
        return [float(v) for v in x]
    return []


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit sign-cancellation impact on O3 closure")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument(
        "--lagbound",
        default="research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_deterministic_k1_sf4.json",
    )
    ap.add_argument(
        "--o3-budget",
        default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json",
    )
    ap.add_argument(
        "--o3-draft",
        default="research/output/o3_asymptotic_inequality_draft_2026-02-17.json",
    )
    ap.add_argument("--output", default="research/output/o3_sign_cancellation_impact_audit_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    lag = _read_json(args.lagbound)
    b = _read_json(args.o3_budget)
    d = _read_json(args.o3_draft)

    ratios: List[float] = []
    for _, prof in lag.get("low_lag_profiles", {}).items():
        ratios.extend(_vals(prof.get("signed_over_abs")))

    if ratios:
        ratio_min = min(ratios)
        ratio_max = max(ratios)
        mean_ratio = sum(ratios) / len(ratios)
    else:
        ratio_min = 0.0
        ratio_max = 0.0
        mean_ratio = 0.0

    ceta_budget = float(b["o3_theorem_budget"]["C_eta_theorem_budget"])
    # If one could replace absolute by signed mass perfectly, best-case reduction
    # is roughly by factor ratio_min (close to 1 here).
    ceta_best_sign_only = ceta_budget * ratio_min
    rel_gain = (ceta_budget - ceta_best_sign_only) / ceta_budget if ceta_budget > 0 else 0.0

    ae2_target = float(d["o3_asymptotic_draft"]["exponent_gap_analysis"]["a_e2_needed_for_target_AH"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "lagbound": args.lagbound,
            "o3_budget": args.o3_budget,
            "o3_draft": args.o3_draft,
        },
        "sign_ratio_summary": {
            "signed_over_abs_min": ratio_min,
            "signed_over_abs_mean": mean_ratio,
            "signed_over_abs_max": ratio_max,
        },
        "impact_estimate": {
            "C_eta_budget": ceta_budget,
            "C_eta_best_sign_only": ceta_best_sign_only,
            "relative_gain_from_sign_only": rel_gain,
        },
        "conclusion": {
            "sign_only_material_for_o3_closure": bool(rel_gain >= 0.1),
            "primary_next_focus": "E2/x asymptotic exponent and constant control",
            "reason": (
                "Observed signed/abs ratios are too close to 1 for sign-only tightening "
                "to materially shrink theorem-side O3 constants."
            ),
            "a_e2_needed_for_target_AH": ae2_target,
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 Sign-Cancellation Impact Audit",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Signed/Abs Ratio Summary",
        "",
        f"- `min = {ratio_min}`",
        f"- `mean = {mean_ratio}`",
        f"- `max = {ratio_max}`",
        "",
        "## Sign-Only Tightening Estimate",
        "",
        f"- `C_eta_budget = {ceta_budget}`",
        f"- `C_eta_best_sign_only = {ceta_best_sign_only}`",
        f"- `relative_gain = {rel_gain}`",
        "",
        "## Conclusion",
        "",
        f"- sign-only materially closes O3: `{payload['conclusion']['sign_only_material_for_o3_closure']}`",
        f"- primary next focus: `{payload['conclusion']['primary_next_focus']}`",
        f"- target condition from O3 draft: `A_E2 <= {ae2_target}` for `A_H<=2` under naive chain.",
        "",
        payload["conclusion"]["reason"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
