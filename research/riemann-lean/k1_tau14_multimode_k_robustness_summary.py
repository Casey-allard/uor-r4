#!/usr/bin/env python3
"""Aggregate tau14 multimode decomposition outputs by fixed K across windows."""

from __future__ import annotations

import argparse
import json
from datetime import date
from pathlib import Path
from typing import Dict, List


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau14 multimode K-robustness summary")
    ap.add_argument("--files", type=str, required=True, help="Comma-separated W56b JSON files")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_tau14_multimode_k_robustness_summary_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    paths = [Path(p.strip()) for p in args.files.split(",") if p.strip()]
    if not paths:
        raise ValueError("files must be non-empty")

    by_window: Dict[str, Dict[int, Dict[str, float]]] = {}
    a0 = 0.98
    for p in paths:
        d = json.loads(p.read_text(encoding="utf-8"))
        xmax = int(d.get("meta", {}).get("xmax", 0))
        label = f"x{xmax}"
        a0 = float(d.get("meta", {}).get("abs_cos_min", a0))
        rows: Dict[int, Dict[str, float]] = {}
        for r in d.get("rows", []):
            k = int(r["extra_mode_count"])
            rows[k] = {
                "rr": float(r["rr_cofinal_grid"]),
                "delta": float(r["delta_rr_cofinal_grid"]),
                "dtri": float(r["delta_tri_cofinal_grid"]),
                "rmse": float(r["rmse_lattice"]),
            }
        by_window[label] = rows

    windows = sorted(by_window.keys())
    ks = sorted({k for rows in by_window.values() for k in rows.keys()})
    rows_out: List[Dict] = []
    robust_ks: List[int] = []

    for k in ks:
        rr_vals: List[float] = []
        dtri_vals: List[float] = []
        rmse_vals: List[float] = []
        full = True
        for w in windows:
            row = by_window[w].get(k)
            if row is None:
                full = False
                break
            rr_vals.append(float(row["rr"]))
            dtri_vals.append(float(row["dtri"]))
            rmse_vals.append(float(row["rmse"]))
        if not full:
            continue
        robust = all(v < a0 for v in rr_vals) and all(v > 0.0 for v in dtri_vals)
        if robust:
            robust_ks.append(k)
        rows_out.append(
            {
                "k": int(k),
                "window_count": len(windows),
                "rr_min": min(rr_vals),
                "rr_max": max(rr_vals),
                "delta_min": min(a0 - v for v in rr_vals),
                "delta_max": max(a0 - v for v in rr_vals),
                "dtri_min": min(dtri_vals),
                "rmse_max": max(rmse_vals),
                "rmse_min": min(rmse_vals),
                "robust": bool(robust),
            }
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "files": [str(p) for p in paths],
            "windows": windows,
            "a0": float(a0),
        },
        "rows": rows_out,
        "robust_k_values": robust_ks,
        "interpretation": {
            "note": (
                "Finite-window robustness only. "
                "Robust K values indicate stable candidate finite-mode splits."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14 Multimode K Robustness Summary ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Windows: `{', '.join(windows)}`")
    lines.append(f"- Alignment gate: `a0={a0}`")
    lines.append(
        "- Robust K values (`rr<a0` and `delta_tri>0` in all windows): "
        + (", ".join(str(k) for k in robust_ks) if robust_ks else "none")
    )
    lines.append("")
    lines.append("| K | rr_min | rr_max | delta_min | dtri_min | rmse_min | rmse_max | robust |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows_out:
        lines.append(
            f"| {r['k']} | {r['rr_min']:.6f} | {r['rr_max']:.6f} | {r['delta_min']:.6f} | "
            f"{r['dtri_min']:.6f} | {r['rmse_min']:.6e} | {r['rmse_max']:.6e} | {str(r['robust']).lower()} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
