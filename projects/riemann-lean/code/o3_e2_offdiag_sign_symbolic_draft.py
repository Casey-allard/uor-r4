#!/usr/bin/env python3
"""Draft symbolic sign-sensitive offdiag inequality for O3 E2 proof."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build E2-OFFDIAG-SIGN symbolic draft")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--o3-e2-full", default="research/output/o3_e2_full_proof_draft_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_offdiag_sign_symbolic_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    s = _read_json(args.o3_sign)["theorem_assumption_candidates"]
    full = _read_json(args.o3_e2_full)["theorem_goal"]

    eps = float(s["eps_sign"])
    neg = float(s["neg_over_abs_cap"])
    pos_floor = float(s["pos_over_abs_floor"])

    # Conservative symbolic coefficients for a sign-sensitive offdiag majorant.
    # Offdiag is split into positive and negative signed masses:
    # Offdiag = Offdiag_pos - Offdiag_neg.
    # With quantified ratios we keep a non-absolute structure.
    k_pos = eps
    k_neg = neg
    k_abs = eps + neg

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_sign": args.o3_sign,
            "o3_e2_full": args.o3_e2_full,
        },
        "lemma_id": "E2-OFFDIAG-SIGN",
        "target_theorem_goal": full["statement"],
        "symbolic_draft": {
            "assumptions": {
                "eps_sign": eps,
                "neg_over_abs_cap": neg,
                "pos_over_abs_floor": pos_floor,
            },
            "proposed_inequality": (
                "|Offdiag(x;W)| <= k_pos * Offdiag_pos(x;W) + "
                "k_neg * Offdiag_neg(x;W) <= k_abs * Offdiag_abs(x;W)"
            ),
            "coefficients": {
                "k_pos": k_pos,
                "k_neg": k_neg,
                "k_abs": k_abs,
            },
            "interpretation": (
                "This is a theorem-draft placeholder preserving sign structure by tracking "
                "positive/negative offdiag channels before fallback to absolute mass."
            ),
        },
        "remaining_to_formalize": [
            "Provide explicit definitions of Offdiag_pos, Offdiag_neg, Offdiag_abs in the E2 decomposition.",
            "Prove asymptotic bounds for these components uniformly in W and x>=x0.",
            "Derive an explicit offdiag exponent A_offdiag compatible with A_E2 target.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Offdiag Sign Symbolic Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Lemma Target",
        "",
        f"- `{payload['lemma_id']}`",
        f"- {payload['target_theorem_goal']}",
        "",
        "## Assumptions",
        "",
        f"- `eps_sign = {eps}`",
        f"- `neg_over_abs_cap = {neg}`",
        f"- `pos_over_abs_floor = {pos_floor}`",
        "",
        "## Proposed Symbolic Inequality",
        "",
        payload["symbolic_draft"]["proposed_inequality"],
        "",
        "## Coefficients",
        "",
        f"- `k_pos = {k_pos}`",
        f"- `k_neg = {k_neg}`",
        f"- `k_abs = {k_abs}`",
        "",
        payload["symbolic_draft"]["interpretation"],
        "",
        "## Remaining To Formalize",
        "",
    ]
    for x in payload["remaining_to_formalize"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
