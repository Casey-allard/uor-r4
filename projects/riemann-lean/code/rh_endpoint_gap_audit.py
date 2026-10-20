#!/usr/bin/env python3
"""Math-first audit: can current bridge chain reach RH-equivalent endpoint class?"""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Audit endpoint gap vs RH-equivalent class")
    ap.add_argument(
        "--canonical-manifest",
        default="research/output/proof_canonical_manifest.json",
    )
    ap.add_argument(
        "--target-log-exponent",
        type=float,
        default=2.0,
        help="Target exponent in |psi(x)-x| <= C sqrt(x) (log x)^A_target.",
    )
    ap.add_argument(
        "--output",
        default="research/output/rh_endpoint_gap_audit_2026-02-17.json",
    )
    args = ap.parse_args()

    man = read_json(args.canonical_manifest)
    canon = man["canonical"]
    pack = read_json(canon["theorem_pack"])
    a2 = read_json(canon["a2"])
    a3 = read_json(canon["a3"])

    beta = float(pack["constants"]["beta"])
    a_h = float(pack["constants"]["A_H"])
    implied_a = max(beta, a_h)
    target = float(args.target_log_exponent)

    # Required inequalities for a direct RH-equivalent style endpoint:
    # max(beta, A_H) <= target.
    beta_gap = beta - target
    ah_gap = a_h - target
    implied_gap = implied_a - target

    blockers = []
    if beta_gap > 0:
        blockers.append("A2 exponent beta exceeds target endpoint class")
    if ah_gap > 0:
        blockers.append("A3 exponent A_H exceeds target endpoint class")

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "theorem_pack": canon["theorem_pack"],
            "a2_artifact": canon["a2"],
            "a3_artifact": canon["a3"],
        },
        "endpoint_target": {
            "form": "|psi(x)-x| <= C sqrt(x) (log x)^A_target",
            "A_target": target,
        },
        "current_implied_endpoint": {
            "beta_from_a2": beta,
            "A_H_from_a3": a_h,
            "A_implied_max": implied_a,
        },
        "gap_to_target": {
            "beta_minus_target": beta_gap,
            "A_H_minus_target": ah_gap,
            "A_implied_minus_target": implied_gap,
        },
        "direct_reachability": {
            "reachable_with_current_exponents": bool(implied_a <= target),
            "blockers": blockers,
        },
        "math_conclusion": (
            "Current chain cannot reach the RH-equivalent log^2 endpoint class directly "
            "unless both beta and A_H are reduced to <= A_target by theorem proof, "
            "or the endpoint implication is sharpened by a different final transfer argument."
        ),
        "next_math_focus": [
            "Derive A2 truncation theorem with beta <= 2 (or show beta does not enter final endpoint at full strength).",
            "Derive A3 bridge-growth theorem with A_H <= 2.",
            "Rework final transfer inequality to avoid max(beta, A_H) bottleneck if possible.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# RH Endpoint Gap Audit",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Target Endpoint Class",
        "",
        f"- `{payload['endpoint_target']['form']}`",
        f"- `A_target = {target}`",
        "",
        "## Current Implied Exponents",
        "",
        f"- `beta (A2) = {beta}`",
        f"- `A_H (A3) = {a_h}`",
        f"- `A_implied = max(beta, A_H) = {implied_a}`",
        "",
        "## Reachability Check",
        "",
        f"- reachable now: `{payload['direct_reachability']['reachable_with_current_exponents']}`",
        f"- `beta-target = {beta_gap}`",
        f"- `A_H-target = {ah_gap}`",
        f"- `A_implied-target = {implied_gap}`",
        "",
        "## Proof-Critical Blockers",
        "",
    ]
    if blockers:
        for b in blockers:
            lines.append(f"- {b}")
    else:
        lines.append("- none")
    lines += [
        "",
        "## Conclusion",
        "",
        payload["math_conclusion"],
        "",
        "## Next Math Focus",
        "",
    ]
    for s in payload["next_math_focus"]:
        lines.append(f"- {s}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
