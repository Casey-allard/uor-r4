#!/usr/bin/env python3
"""Assemble E2 component closures into a theorem-facing E2-GLUE closure artifact."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 E2-GLUE asymptotic closure artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--diag-closure", default="research/output/o3_e2_diag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--offdiag-closure", default="research/output/o3_e2_offdiag_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--rem-closure", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    ap.add_argument("--o3-e2-target-pack", default="research/output/o3_e2_theorem_target_pack_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o3_e2_glue_asymptotic_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    d = _read_json(args.diag_closure)
    o = _read_json(args.offdiag_closure)
    r = _read_json(args.rem_closure)
    t = _read_json(args.o3_e2_target_pack)["o3_e2_target_pack"]

    a_diag = float(d["derived_diag_bound"]["A_diag"])
    c_diag = float(d["derived_diag_bound"]["C_diag"])
    a_off = float(o["derived_offdiag_bound"]["A_offdiag"])
    c_off = float(o["derived_offdiag_bound"]["C_offdiag"])
    a_rem = float(r["asymptotic_closure"]["A_rem"])
    c_rem = float(r["asymptotic_closure"]["C_rem_uniform"])
    a_e2 = max(a_diag, a_off, a_rem)
    c_e2 = c_diag + c_off + c_rem

    a_e2_max = float(t["e2_exponent_targets"]["A_E2_max_for_target_AH"])
    c_e2_min = float(t["e2_constant_target"]["C_E2_min_from_budget_chain"])

    status_closed = (
        d.get("status") == "conditional_asymptotic_closed"
        and o.get("status") == "conditional_asymptotic_closed"
        and r.get("status") == "conditional_asymptotic_closed"
    )
    target_compatible = a_e2 <= a_e2_max

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "diag_closure": args.diag_closure,
            "offdiag_closure": args.offdiag_closure,
            "rem_closure": args.rem_closure,
            "o3_e2_target_pack": args.o3_e2_target_pack,
        },
        "lemma_id": "E2-GLUE",
        "status": "conditional_asymptotic_closed" if status_closed else "open",
        "assembly": {
            "A_E2": a_e2,
            "C_E2": c_e2,
            "components": {
                "diag": {"A": a_diag, "C": c_diag},
                "offdiag": {"A": a_off, "C": c_off},
                "remainder": {"A": a_rem, "C": c_rem},
            },
            "formula": "A_E2=max(A_diag,A_offdiag,A_rem), C_E2=C_diag+C_offdiag+C_rem",
        },
        "target_check": {
            "A_E2_max_target": a_e2_max,
            "A_E2_target_compatible": target_compatible,
            "C_E2_min_chain_compatibility": c_e2_min,
            "C_E2_minus_min_target": c_e2 - c_e2_min,
        },
        "interpretation": (
            "E2 assembly is now explicit in theorem-facing form. Remaining work is to replace conditional envelopes "
            "for DIAG/OFFDIAG/REM by unconditional proofs under the fixed O2/O3/O4 assumption framework."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Glue Asymptotic Closure",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Status",
        "",
        f"- `{payload['status']}`",
        "",
        "## Assembly",
        "",
        f"- `A_E2 = {a_e2}`",
        f"- `C_E2 = {c_e2}`",
        f"- components: diag({a_diag}, {c_diag}), offdiag({a_off}, {c_off}), rem({a_rem}, {c_rem})",
        "",
        "## Target Check",
        "",
        f"- `A_E2_max_target = {a_e2_max}`",
        f"- exponent compatible: `{target_compatible}`",
        f"- `C_E2_min_chain_compatibility = {c_e2_min}`",
        f"- `C_E2 - C_E2_min = {c_e2 - c_e2_min}`",
        "",
        payload["interpretation"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
