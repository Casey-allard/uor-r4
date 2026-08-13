#!/usr/bin/env python3
"""Build a theorem-side O3 constant budget from sign-sensitive deterministic chain."""

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
    ap = argparse.ArgumentParser(description="Create O3 sign-sensitive theorem budget")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument(
        "--a3-primary",
        default="research/output/a3_offdiag_dynamic_majorant_2026-02-17_m512_stability_eta4sf3_globallog.json",
    )
    ap.add_argument(
        "--sign-replacement",
        default="research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17_deterministic_k1.json",
    )
    ap.add_argument(
        "--output",
        default="research/output/o3_sign_sensitive_theorem_budget_2026-02-17.json",
    )
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    a3 = _read_json(args.a3_primary)
    rep = _read_json(args.sign_replacement)

    eta_primary = float(a3["eta_envelope"]["C_eta_uplifted"])
    ah = float(a3["h_transfer_envelope"]["A_H"])
    ch_primary = float(a3["h_transfer_envelope"]["C_H_from_density_transfer"])

    r = float(rep["o3_sign_sensitive_replacement"]["replacement_over_primary_ratio"])
    ceta_theorem = eta_primary * r
    # Conservative constant uplift for H-envelope when eta majorant is replaced.
    ch_theorem = ch_primary * math.sqrt(max(1.0, r))

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "a3_primary": args.a3_primary,
            "sign_replacement": args.sign_replacement,
        },
        "o3_theorem_budget": {
            "interpretation": "Conservative theorem-side constant budget using deterministic sign-sensitive eta replacement.",
            "A_eta": float(a3["eta_envelope"]["A_eta"]),
            "C_eta_primary": eta_primary,
            "C_eta_theorem_budget": ceta_theorem,
            "replacement_ratio": r,
            "A_H": ah,
            "C_H_primary": ch_primary,
            "C_H_theorem_budget": ch_theorem,
            "k_tail_mode": "deterministic",
        },
        "status": {
            "deterministic_chain_holds_on_cached_grid": bool(
                rep["o3_sign_sensitive_replacement"]["deterministic_chain_holds"]
            ),
            "note": "This provides theorem-side constant budgeting; asymptotic proof still required.",
        },
        "remaining_o3_blockers": [
            "Prove sign-sensitive offdiag inequality with explicit asymptotic constants (no grid calibration).",
            "Prove eta_+(x;W) bound for all x>=x0, uniformly in W.",
            "Prove E2/x asymptotic bound and combine with theorem-side constants.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    b = payload["o3_theorem_budget"]
    lines = [
        "# O3 Sign-Sensitive Theorem Budget",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Budget Construction",
        "",
        f"- `C_eta_primary = {b['C_eta_primary']}`",
        f"- `replacement_ratio = {b['replacement_ratio']}`",
        f"- `C_eta_theorem_budget = {b['C_eta_theorem_budget']}`",
        f"- `A_H = {b['A_H']}`",
        f"- `C_H_primary = {b['C_H_primary']}`",
        f"- `C_H_theorem_budget = {b['C_H_theorem_budget']}`",
        "",
        "## Status",
        "",
        f"- deterministic chain holds on cached grid: `{payload['status']['deterministic_chain_holds_on_cached_grid']}`",
        f"- {payload['status']['note']}",
        "",
        "## Remaining O3 Blockers",
        "",
    ]
    for x in payload["remaining_o3_blockers"]:
        lines.append(f"- {x}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
