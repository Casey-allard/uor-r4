#!/usr/bin/env python3
"""Quantize sign-cancellation constants from deterministic lagbound profiles for O3."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _safe_list(x: Any) -> List[float]:
    if isinstance(x, list):
        return [float(v) for v in x]
    return []


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract O3 sign-cancellation constants")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument(
        "--lagbound",
        default="research/output/a3_offdiag_sign_sensitive_lagbound_2026-02-17_deterministic_k1_sf4.json",
    )
    ap.add_argument("--safety", type=float, default=2.0)
    ap.add_argument(
        "--output",
        default="research/output/o3_sign_cancellation_quantization_2026-02-17.json",
    )
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    d = _read_json(args.lagbound)
    profiles = d.get("low_lag_profiles", {})
    safety = float(args.safety)

    eps_signed_max = 0.0
    neg_over_abs_max = 0.0
    pos_over_abs_min = 1.0
    per_base: Dict[str, Dict[str, float]] = {}

    for base, prof in profiles.items():
        signed_over_abs = _safe_list(prof.get("signed_over_abs"))
        pos_over_abs = _safe_list(prof.get("pos_over_abs"))
        neg_mass = _safe_list(prof.get("neg_mass"))
        abs_mass = _safe_list(prof.get("abs_mass"))

        eps_b = max((1.0 - v) for v in signed_over_abs) if signed_over_abs else 0.0
        pos_min_b = min(pos_over_abs) if pos_over_abs else 1.0
        neg_ratio_b = 0.0
        if neg_mass and abs_mass and len(neg_mass) == len(abs_mass):
            neg_ratio_b = max((n / a) if a > 0 else 0.0 for n, a in zip(neg_mass, abs_mass))

        eps_signed_max = max(eps_signed_max, eps_b)
        neg_over_abs_max = max(neg_over_abs_max, neg_ratio_b)
        pos_over_abs_min = min(pos_over_abs_min, pos_min_b)

        per_base[str(base)] = {
            "eps_signed_max": eps_b,
            "neg_over_abs_max": neg_ratio_b,
            "pos_over_abs_min": pos_min_b,
        }

    # Conservative theorem-assumption candidates.
    eps_theorem = safety * eps_signed_max
    neg_ratio_theorem = safety * neg_over_abs_max
    pos_floor_theorem = max(0.0, 1.0 - eps_theorem)

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "lagbound": args.lagbound,
            "safety": safety,
        },
        "empirical_extrema": {
            "eps_signed_max": eps_signed_max,
            "neg_over_abs_max": neg_over_abs_max,
            "pos_over_abs_min": pos_over_abs_min,
        },
        "theorem_assumption_candidates": {
            "eps_sign": eps_theorem,
            "neg_over_abs_cap": neg_ratio_theorem,
            "pos_over_abs_floor": pos_floor_theorem,
            "assumption_template": (
                "For all large x and all W in wheel family, low-lag signed/abs cancellation defect "
                "is <= eps_sign and neg/abs ratio is <= neg_over_abs_cap."
            ),
        },
        "per_base_summary": per_base,
        "interpretation": (
            "These constants convert deterministic lag-profile evidence into explicit "
            "sign-cancellation assumption targets for O3 asymptotic proofs."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 Sign-Cancellation Quantization",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Empirical Extrema",
        "",
        f"- `eps_signed_max = {eps_signed_max}`",
        f"- `neg_over_abs_max = {neg_over_abs_max}`",
        f"- `pos_over_abs_min = {pos_over_abs_min}`",
        "",
        "## Theorem Assumption Candidates",
        "",
        f"- `eps_sign = {eps_theorem}`",
        f"- `neg_over_abs_cap = {neg_ratio_theorem}`",
        f"- `pos_over_abs_floor = {pos_floor_theorem}`",
        "",
        payload["interpretation"],
        "",
    ]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
