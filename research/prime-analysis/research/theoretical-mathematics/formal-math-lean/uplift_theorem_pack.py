#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Optional


def _read_json(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def _read_optional_json(path: Path) -> Optional[Dict[str, Any]]:
    if not path.exists():
        return None
    return _read_json(path)


def _resolve_path(
    cli_value: Optional[str],
    canonical: Optional[Dict[str, Any]],
    canonical_key: str,
    fallback: str,
) -> Path:
    if cli_value:
        return Path(cli_value)
    if canonical is not None:
        v = canonical.get(canonical_key)
        if isinstance(v, str) and v.strip():
            return Path(v)
    return Path(fallback)


def _extract_a3_constants(a3: Dict[str, Any]) -> Dict[str, float]:
    if "uplift_constants" in a3:
        u = a3["uplift_constants"]
        return {
            "C_H": float(u["C_H_from_normalized_sum"]),
            "A_H": float(u["A_H_from_normalized_sum"]),
            "valid_holds": bool(a3["checks"]["valid"]["deterministic_check_holds"])
            and bool(a3["checks"]["valid"]["log_envelope_holds"]),
        }
    if "h_transfer_envelope" in a3:
        h = a3["h_transfer_envelope"]
        return {
            "C_H": float(h["C_H_from_density_transfer"]),
            "A_H": float(h["A_H"]),
            "valid_holds": bool(a3["checks"]["valid"]["holds"]),
        }
    raise KeyError("unrecognized A3 artifact schema")


def _max_lhs_from_per_n(row: Dict[str, Any]) -> float:
    vals = [float(b["max_abs_e_scaled"]) for b in row.get("per_base", [])]
    return max(vals) if vals else float("nan")


def _build_rows(
    per_n: List[Dict[str, Any]],
    c0: float,
    c_delta: float,
    beta: float,
    c_h: float,
    a_h: float,
) -> List[Dict[str, Any]]:
    rows: List[Dict[str, Any]] = []
    for row in per_n:
        n_max = int(row["n_max"])
        x = max(3.0, float(n_max))
        lx = math.log(x)
        rhs = c0 + c_delta * (lx ** beta) + c_h * (lx ** a_h)
        lhs = _max_lhs_from_per_n(row)
        gap = rhs - lhs
        ratio = lhs / rhs if rhs > 0 else float("inf")
        rows.append(
            {
                "n_max": n_max,
                "lhs_max_abs_e_scaled": lhs,
                "rhs_unified_uplift": rhs,
                "gap_rhs_minus_lhs": gap,
                "ratio_lhs_over_rhs": ratio,
            }
        )
    return rows


def _write_md(path: Path, payload: Dict[str, Any]) -> None:
    c = payload["constants"]
    ch = payload["checks"]
    lines = [
        "# Unified Uplift Theorem Pack",
        "",
        f"- Timestamp (UTC): `{payload['timestamp_utc']}`",
        f"- A1 valid-holds: `{ch['a1_valid_holds']}`",
        f"- A2 valid-holds: `{ch['a2_valid_holds']}`",
        f"- A3 valid deterministic+envelope holds: `{ch['a3_valid_holds']}`",
        f"- A4 holds-on-grid: `{ch['a4_holds_on_grid']}`",
        f"- Unified pack holds: `{ch['unified_pack_holds']}`",
        "",
        "## Constants",
        "",
        f"- `C0_uplifted = {c['C0_uplifted']:.15g}`",
        f"- `C_delta_uplifted = {c['C_delta_uplifted']:.15g}`",
        f"- `beta = {c['beta']:.15g}`",
        f"- `C_H_uplifted = {c['C_H_uplifted']:.15g}`",
        f"- `A_H = {c['A_H']:.15g}`",
        "",
        "## Unified Margin Summary",
        "",
        f"- Max ratio `lhs/rhs`: `{ch['unified_ratio_max']:.12g}`",
        f"- Min gap `(rhs-lhs)`: `{ch['unified_gap_min']:.12g}`",
        "",
        "## Per-n Rows",
        "",
        "| n_max | lhs_max | rhs_uplift | rhs-lhs | lhs/rhs |",
        "|---:|---:|---:|---:|---:|",
    ]
    for row in payload["unified_rows"]:
        lines.append(
            "| "
            f"{row['n_max']} | "
            f"{row['lhs_max_abs_e_scaled']:.12g} | "
            f"{row['rhs_unified_uplift']:.12g} | "
            f"{row['gap_rhs_minus_lhs']:.12g} | "
            f"{row['ratio_lhs_over_rhs']:.12g} |"
        )
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Build a unified uplift theorem pack from A1/A2/A3/A4 artifacts."
    )
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--a1", default=None)
    ap.add_argument("--a2", default=None)
    ap.add_argument("--a3", default=None)
    ap.add_argument("--a4", default=None)
    ap.add_argument("--output", default=None)
    args = ap.parse_args()

    manifest = _read_optional_json(Path(args.canonical_manifest))
    canonical = manifest.get("canonical", {}) if isinstance(manifest, dict) else None

    a1_path = _resolve_path(
        args.a1,
        canonical,
        "a1",
        "research/output/a1_smoothing_uplift_pack_refresh_2026-02-17.json",
    )
    a2_path = _resolve_path(
        args.a2,
        canonical,
        "a2",
        "research/output/a2_infinite_tail_uplift_refresh_2026-02-17_sf3p5.json",
    )
    a3_path = _resolve_path(
        args.a3,
        canonical,
        "a3",
        "research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json",
    )
    a4_path = _resolve_path(
        args.a4,
        canonical,
        "a4",
        "research/output/a4_uniform_assumption_check_refresh_2026-02-17.json",
    )
    out_json = _resolve_path(
        args.output,
        canonical,
        "theorem_pack",
        "research/output/uplift_theorem_pack_refresh_2026-02-17.json",
    )
    out_md = out_json.with_suffix(".md")

    a1 = _read_json(a1_path)
    a2 = _read_json(a2_path)
    a3 = _read_json(a3_path)
    a4 = _read_json(a4_path)

    c0 = float(a1["decomposition_constants"]["C0_uplifted"])
    c_delta = float(a2["uplift_constant"]["C_delta_uplifted"])
    beta = float(a2["config"]["beta_fixed"])
    a3c = _extract_a3_constants(a3)
    c_h = float(a3c["C_H"])
    a_h = float(a3c["A_H"])

    rows = _build_rows(
        per_n=a4.get("per_n", []),
        c0=c0,
        c_delta=c_delta,
        beta=beta,
        c_h=c_h,
        a_h=a_h,
    )
    valid_ratios = [float(r["ratio_lhs_over_rhs"]) for r in rows if math.isfinite(float(r["ratio_lhs_over_rhs"]))]
    valid_gaps = [float(r["gap_rhs_minus_lhs"]) for r in rows if math.isfinite(float(r["gap_rhs_minus_lhs"]))]
    ratio_max = max(valid_ratios) if valid_ratios else float("inf")
    gap_min = min(valid_gaps) if valid_gaps else float("-inf")

    a1_valid = bool(a1["checks"]["valid"]["holds"])
    a2_valid = bool(a2["checks"]["valid"]["holds"])
    a3_valid = bool(a3c["valid_holds"])
    a4_holds = bool(a4["theorem_rhs_check"]["holds_on_grid"])
    unified_holds = ratio_max <= 1.0

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "A1": str(a1_path),
            "A2": str(a2_path),
            "A3": str(a3_path),
            "A4": str(a4_path),
            "canonical_manifest": str(Path(args.canonical_manifest)),
        },
        "constants": {
            "C0_uplifted": c0,
            "C_delta_uplifted": c_delta,
            "beta": beta,
            "C_H_uplifted": c_h,
            "A_H": a_h,
        },
        "checks": {
            "a1_valid_holds": a1_valid,
            "a2_valid_holds": a2_valid,
            "a3_valid_holds": a3_valid,
            "a4_holds_on_grid": a4_holds,
            "unified_pack_holds": unified_holds,
            "unified_ratio_max": ratio_max,
            "unified_gap_min": gap_min,
        },
        "unified_rows": rows,
    }

    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    _write_md(out_md, payload)
    print(json.dumps({"json": str(out_json), "md": str(out_md), "holds": unified_holds}, indent=2))


if __name__ == "__main__":
    main()
