#!/usr/bin/env python3
"""Build an explicit O3 sign-sensitive constant replacement pack from cached artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 sign-sensitive constant replacement pack")
    ap.add_argument(
        "--sign-sensitive",
        default="research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_sf5_cached.json",
    )
    ap.add_argument(
        "--primary",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json",
    )
    ap.add_argument(
        "--output",
        default="research/output/a3_sign_sensitive_constant_replacement_pack_2026-02-17.json",
    )
    args = ap.parse_args()

    ss = read_json(args.sign_sensitive)
    prim = read_json(args.primary)

    cfg = ss["config"]
    chain = ss["chain_checks"]
    tail = ss["tail_calibration"]
    nc = ss["normalized_constants"]
    tail_mode = str(tail.get("mode", cfg.get("tail_mode", "empirical")))

    primary_c_eta = float(prim["eta_envelope"]["C_eta_uplifted"])
    primary_a_eta = float(prim["eta_envelope"]["A_eta"])
    primary_a_h = float(prim["h_transfer_envelope"]["A_H"])
    primary_c_h = float(prim["h_transfer_envelope"]["C_H_from_density_transfer"])

    replacement = {
        "A_eta": float(cfg["a_eta"]),
        "C_eta_replacement": float(nc["C_ss_uplifted"]),
        "C_eta_primary": primary_c_eta,
        "replacement_over_primary_ratio": float(nc["C_ss_uplifted"] / max(1e-30, primary_c_eta)),
        "valid_ratio_replacement": float(nc["valid_ratio_ss_over_uplifted"]),
        "deterministic_chain_holds": bool(chain["eta_pos_le_eta_ss_holds"]),
        "max_chain_gap_eta_pos_minus_eta_ss": float(chain["max_gap_eta_pos_minus_eta_ss"]),
        "k_tail_used": float(tail["k_tail_used"]),
        "tail_mode": tail_mode,
    }

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {"sign_sensitive": args.sign_sensitive, "primary": args.primary},
        "o3_primary_constants": {
            "A_eta": primary_a_eta,
            "C_eta": primary_c_eta,
            "A_H": primary_a_h,
            "C_H": primary_c_h,
        },
        "o3_sign_sensitive_replacement": replacement,
        "status": {
            "replacement_is_valid_on_cached_grid": bool(
                replacement["deterministic_chain_holds"]
                and replacement["valid_ratio_replacement"] <= 1.0 + 1e-12
            ),
            "replacement_is_tighter_than_primary": bool(
                replacement["replacement_over_primary_ratio"] <= 1.0
            ),
            "replacement_is_deterministic_tail": bool(tail_mode == "deterministic"),
        },
        "interpretation": (
            "Sign-sensitive deterministic chain yields a held-out-valid eta envelope, "
            "but at a substantially larger constant than the primary calibrated branch."
        ),
        "analytic_gap_remaining": [
            "Replace empirical lag-tail calibration k_tail_used by theorem-side explicit lag-sum constant.",
            "Convert finite-grid replacement constants into asymptotic x>=x0 statement.",
        ],
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    r = payload["o3_sign_sensitive_replacement"]
    st = payload["status"]
    lines = [
        "# A3 Sign-Sensitive Constant Replacement Pack",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## O3 Primary vs Deterministic Replacement",
        "",
        f"- Primary `A_eta={primary_a_eta}`, `C_eta={primary_c_eta:.12g}`, `A_H={primary_a_h}`, `C_H={primary_c_h:.12g}`",
        f"- Replacement `A_eta={r['A_eta']}`, `C_eta={r['C_eta_replacement']:.12g}`",
        f"- replacement/primary ratio: `{r['replacement_over_primary_ratio']:.12g}`",
        "",
        "## Deterministic Chain Check",
        "",
        f"- `eta_pos <= eta_ss` holds: `{r['deterministic_chain_holds']}`",
        f"- max gap `eta_pos-eta_ss`: `{r['max_chain_gap_eta_pos_minus_eta_ss']:.12g}`",
        f"- held-out ratio over replacement `C_eta`: `{r['valid_ratio_replacement']:.12g}`",
        f"- lag-tail constant used: `{r['k_tail_used']:.12g}`",
        f"- tail mode: `{r['tail_mode']}`",
        "",
        "## Status",
        "",
        f"- replacement valid on cached grid: `{st['replacement_is_valid_on_cached_grid']}`",
        f"- replacement tighter than primary: `{st['replacement_is_tighter_than_primary']}`",
        f"- deterministic tail mode: `{st['replacement_is_deterministic_tail']}`",
        "",
        "## Remaining Analytic Gap",
        "",
    ]
    for item in payload["analytic_gap_remaining"]:
        lines.append(f"- {item}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
