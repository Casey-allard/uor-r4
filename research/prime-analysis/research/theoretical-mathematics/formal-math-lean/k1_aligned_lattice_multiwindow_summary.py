#!/usr/bin/env python3
"""Aggregate multi-window results from fixed_error_psi_aligned_lattice_probe."""

from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List


def _tau_key(tau: float) -> str:
    return f"{tau:.9f}"


@dataclass
class LatticeSummaryRow:
    tau: float
    window_count: int
    rr_cofinal_min: float
    rr_cofinal_max: float
    delta_tri_min: float
    delta_tri_max: float
    cos_min_used: float
    robust_rr_and_delta: bool
    windows: Dict[str, Dict[str, float]]


def main() -> None:
    ap = argparse.ArgumentParser(description="Lattice probe multi-window summary")
    ap.add_argument("--files", type=str, required=True, help="Comma-separated lattice JSON paths")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_aligned_lattice_multiwindow_summary_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    paths = [Path(p.strip()) for p in args.files.split(",") if p.strip()]
    if not paths:
        raise ValueError("files must be non-empty")

    by_window: Dict[str, Dict[str, Dict[str, float]]] = {}
    a0 = 0.98
    for p in paths:
        d = json.loads(p.read_text(encoding="utf-8"))
        xmax = int(d.get("meta", {}).get("xmax", 0))
        label = f"x{xmax}"
        a0 = float(d.get("meta", {}).get("abs_cos_min", a0))
        rows: Dict[str, Dict[str, float]] = {}
        for r in d.get("rows", []):
            tau = float(r["tau"])
            rows[_tau_key(tau)] = {
                "tau": tau,
                "rr": float(r["rr_cofinal_grid"]),
                "delta_tri": float(r["delta_tri_cofinal_grid"]),
                "cos_min_used": float(r["cos_abs_min_used"]),
                "amp": float(r["amplitude"]),
                "used_n": float(r["lattice_points_used"]),
            }
        by_window[label] = rows

    window_labels = sorted(by_window.keys())
    tau_keys = sorted({tk for rows in by_window.values() for tk in rows.keys()})

    out_rows: List[LatticeSummaryRow] = []
    for tk in tau_keys:
        rows_by_w: Dict[str, Dict[str, float]] = {}
        rr_vals: List[float] = []
        delta_vals: List[float] = []
        cosmins: List[float] = []
        tau_val = None
        for w in window_labels:
            row = by_window[w].get(tk)
            if row is None:
                continue
            tau_val = float(row["tau"])
            rows_by_w[w] = row
            rr_vals.append(float(row["rr"]))
            delta_vals.append(float(row["delta_tri"]))
            cosmins.append(float(row["cos_min_used"]))
        if tau_val is None:
            continue
        robust = (
            len(rows_by_w) == len(window_labels)
            and bool(rr_vals)
            and all(v < a0 for v in rr_vals)
            and all(v > 0.0 for v in delta_vals)
        )
        out_rows.append(
            LatticeSummaryRow(
                tau=tau_val,
                window_count=len(rows_by_w),
                rr_cofinal_min=min(rr_vals) if rr_vals else 0.0,
                rr_cofinal_max=max(rr_vals) if rr_vals else 0.0,
                delta_tri_min=min(delta_vals) if delta_vals else 0.0,
                delta_tri_max=max(delta_vals) if delta_vals else 0.0,
                cos_min_used=min(cosmins) if cosmins else 0.0,
                robust_rr_and_delta=bool(robust),
                windows=rows_by_w,
            )
        )

    robust_taus = [r.tau for r in out_rows if r.robust_rr_and_delta]

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "files": [str(p) for p in paths],
            "windows": window_labels,
            "a0": float(a0),
        },
        "rows": [asdict(r) for r in sorted(out_rows, key=lambda z: z.tau)],
        "robust_taus": robust_taus,
        "interpretation": {
            "note": (
                "Finite-window structured-lattice robustness only. "
                "Use robust_taus as candidates for theorem-facing admissibility analysis."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Aligned-Lattice Multiwindow Summary ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Windows: `{', '.join(window_labels)}`")
    lines.append(f"- Alignment gate: `a0={a0}`")
    lines.append(
        "- Robust taus (`rr<a0` and `delta_tri>0` in all windows): "
        + (", ".join(f"{t:.9f}" for t in robust_taus) if robust_taus else "none")
    )
    lines.append("")
    lines.append("| tau | windows | rr_min | rr_max | delta_tri_min | cos_min_used | robust |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for r in sorted(out_rows, key=lambda z: z.tau):
        lines.append(
            f"| {r.tau:.9f} | {r.window_count} | {r.rr_cofinal_min:.6f} | {r.rr_cofinal_max:.6f} | "
            f"{r.delta_tri_min:.6f} | {r.cos_min_used:.6f} | {str(r.robust_rr_and_delta).lower()} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
