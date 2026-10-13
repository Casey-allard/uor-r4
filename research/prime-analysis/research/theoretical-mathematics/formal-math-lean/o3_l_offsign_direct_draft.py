#!/usr/bin/env python3
"""Create direct theorem-draft artifact for O3 lemma L-OFFSIGN."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 direct lemma draft L-OFFSIGN")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-direct-pack", default="research/output/o3_direct_envelope_derivation_pack_2026-02-17.json")
    ap.add_argument("--offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_l_offsign_direct_draft_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    pack = _read_json(args.o3_direct_pack)["o3_direct_envelope_derivation_pack"]
    off = _read_json(args.offdiag_closure)
    caps = pack["frozen_sign_caps"]
    coeff = off["derived_offdiag_bound"]["coefficients"]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_direct_pack": args.o3_direct_pack,
            "offdiag_closure": args.offdiag_closure,
        },
        "lemma_id": "L-OFFSIGN",
        "status": "direct_theorem_draft_started",
        "target_statement": (
            "For all x>=x0 and W in wheel family, |Offdiag| <= k_abs*Offdiag_abs "
            "with k_abs derived from explicit sign-cancellation caps."
        ),
        "frozen_caps": caps,
        "frozen_coefficients": coeff,
        "derivation_outline": [
            "Split offdiag into positive and negative channels.",
            "Apply caps eps_sign and neg_over_abs_cap to signed channel masses.",
            "Derive explicit k_abs multiplier with no empirical re-fit.",
            "Compose with L-OFFABS to produce final offdiag majorant.",
        ],
        "replacement_goal": "Replace U-OFFDIAG interface argument by direct sign-sensitive lemma.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 L-OFFSIGN Direct Draft",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['status']}`",
        f"- `k_abs = {coeff['k_abs']}`",
        "",
        "## Derivation Outline",
        "",
    ]
    for x in payload["derivation_outline"]:
        lines.append(f"- {x}")
    lines += ["", payload["replacement_goal"], ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
