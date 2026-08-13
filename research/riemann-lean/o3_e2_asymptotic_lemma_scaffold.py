#!/usr/bin/env python3
"""Create theorem-facing scaffold for O3 E2/x asymptotic lemma."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 E2/x asymptotic lemma scaffold")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--e2-target-pack", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--o3-sign", default="research/output/o3_sign_cancellation_quantization_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_asymptotic_lemma_scaffold_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    t = _read_json(args.e2_target_pack)
    q = _read_json(args.o3_sign)

    pack = t["o3_e2_target_pack"]
    ae2_max = float(pack["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    ce2_min = float(pack["e2_constant_target"]["C_E2_min_from_budget_chain"])
    eps = float(q["theorem_assumption_candidates"]["eps_sign"])
    neg = float(q["theorem_assumption_candidates"]["neg_over_abs_cap"])

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "e2_target_pack": args.e2_target_pack,
            "o3_sign_quantization": args.o3_sign,
        },
        "lemma_o3_e2_asymptotic": {
            "name": "E2/x Asymptotic Control Lemma (O3 core)",
            "quantified_statement": (
                "There exist x0>=3, C_E2>=0, A_E2<=A_E2_max such that for all x>=x0 and "
                "W in {30,210,2310,30030}, E2(x;W)/x <= C_E2 (log x)^A_E2."
            ),
            "target_thresholds": {
                "A_E2_max": ae2_max,
                "C_E2_min_chain_compatibility": ce2_min,
                "eps_sign_assumption": eps,
                "neg_over_abs_cap_assumption": neg,
            },
            "proof_obligation_slices": [
                "Split E2 into diagonal and offdiag components with explicit remainder control.",
                "Use quantified sign assumptions to bound offdiag without worst-case absolute loss.",
                "Derive final asymptotic exponent A_E2 and constant C_E2, uniform in wheel family.",
            ],
            "closure_link": (
                "Combined with eta_+ bound and |H|<=sqrt((1+eta_+)E2/x), this discharges O3 bridge-growth theorem target."
            ),
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Asymptotic Lemma Scaffold",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Quantified Statement",
        "",
        payload["lemma_o3_e2_asymptotic"]["quantified_statement"],
        "",
        "## Target Thresholds",
        "",
        f"- `A_E2_max = {ae2_max}`",
        f"- `C_E2_min_chain_compatibility = {ce2_min}`",
        f"- `eps_sign_assumption = {eps}`",
        f"- `neg_over_abs_cap_assumption = {neg}`",
        "",
        "## Proof Obligation Slices",
        "",
    ]
    for x in payload["lemma_o3_e2_asymptotic"]["proof_obligation_slices"]:
        lines.append(f"- {x}")
    lines += [
        "",
        payload["lemma_o3_e2_asymptotic"]["closure_link"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
