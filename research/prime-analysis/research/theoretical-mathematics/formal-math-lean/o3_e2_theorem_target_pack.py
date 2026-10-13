#!/usr/bin/env python3
"""Build explicit theorem-target constraints for O3 E2/x asymptotic closure."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Create O3 E2/x theorem target pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-budget", default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json")
    ap.add_argument("--o3-draft", default="research/output/o3_asymptotic_inequality_draft_2026-02-17.json")
    ap.add_argument("--a-h-target", type=float, default=2.0)
    ap.add_argument("--output", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    b = _read_json(args.o3_budget)
    d = _read_json(args.o3_draft)

    ob = b["o3_theorem_budget"]
    c_eta = float(ob["C_eta_theorem_budget"])
    a_eta = float(ob["A_eta"])
    c_h = float(ob["C_H_theorem_budget"])
    a_h_emp = float(ob["A_H"])
    a_h_target = float(args.a_h_target)

    # Naive exponent condition from |H| <= sqrt((1+eta)E2/x):
    # A_H <= (A_eta + A_E2)/2  => A_E2 <= 2*A_H - A_eta.
    a_e2_max_for_target = 2.0 * a_h_target - a_eta
    a_e2_max_for_emp = 2.0 * a_h_emp - a_eta

    # Naive constant condition:
    # C_H^2 <= (1 + C_eta) * C_E2  => C_E2 >= C_H^2 / (1 + C_eta).
    c_e2_min_from_budget = (c_h * c_h) / max(1e-30, (1.0 + c_eta))

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_budget": args.o3_budget,
            "o3_draft": args.o3_draft,
            "a_h_target": a_h_target,
        },
        "o3_e2_target_pack": {
            "naive_chain_template": "|H| <= sqrt((1+eta_+) E2/x)",
            "known_budget_constants": {
                "A_eta": a_eta,
                "C_eta_budget": c_eta,
                "A_H_empirical": a_h_emp,
                "C_H_budget": c_h,
            },
            "e2_exponent_targets": {
                "A_E2_max_for_target_AH": a_e2_max_for_target,
                "A_E2_max_for_empirical_AH": a_e2_max_for_emp,
            },
            "e2_constant_target": {
                "C_E2_min_from_budget_chain": c_e2_min_from_budget,
            },
            "interpretation": (
                "Primary O3 theorem objective: prove an asymptotic E2/x bound with "
                "A_E2 <= A_E2_max_for_target_AH (i.e., <=0 for A_H<=2 under naive chain), "
                "plus compatible constants, while leveraging sign-sensitive structure."
            ),
            "next_math_action": [
                "Draft theorem-side E2/x asymptotic lemma with explicit A_E2, C_E2 symbols.",
                "Connect sign-sensitive offdiag inequalities to E2/x exponent suppression.",
                "Feed proved E2 targets back into O3 closure and then O5 final theorem."
            ],
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    t = payload["o3_e2_target_pack"]
    lines = [
        "# O3 E2 Theorem Target Pack",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Known Budget Constants",
        "",
        f"- `A_eta = {t['known_budget_constants']['A_eta']}`",
        f"- `C_eta_budget = {t['known_budget_constants']['C_eta_budget']}`",
        f"- `A_H_empirical = {t['known_budget_constants']['A_H_empirical']}`",
        f"- `C_H_budget = {t['known_budget_constants']['C_H_budget']}`",
        "",
        "## E2 Exponent Targets",
        "",
        f"- `A_E2_max_for_target_AH = {t['e2_exponent_targets']['A_E2_max_for_target_AH']}`",
        f"- `A_E2_max_for_empirical_AH = {t['e2_exponent_targets']['A_E2_max_for_empirical_AH']}`",
        "",
        "## E2 Constant Target",
        "",
        f"- `C_E2_min_from_budget_chain = {t['e2_constant_target']['C_E2_min_from_budget_chain']}`",
        "",
        t["interpretation"],
        "",
        "## Next Math Action",
        "",
    ]
    for x in t["next_math_action"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
