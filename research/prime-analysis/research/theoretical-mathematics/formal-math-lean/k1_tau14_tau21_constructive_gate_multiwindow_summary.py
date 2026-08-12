#!/usr/bin/env python3
"""Aggregate multi-window constructive gate probe outputs by search window."""

from __future__ import annotations

import argparse
import json
from datetime import date
from pathlib import Path
from typing import Dict, List


def main() -> None:
    ap = argparse.ArgumentParser(description="Constructive gate multi-window summary")
    ap.add_argument("--files", type=str, required=True, help="Comma-separated constructive-gate JSON files")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_tau14_tau21_constructive_gate_multiwindow_summary_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    paths = [Path(p.strip()) for p in args.files.split(",") if p.strip()]
    if not paths:
        raise ValueError("files must be non-empty")

    by_w: Dict[int, List[Dict]] = {}
    windows: List[str] = []
    a1 = 0.98
    gate_mode = None
    for p in paths:
        d = json.loads(p.read_text(encoding="utf-8"))
        xmax = int(d.get("meta", {}).get("xmax", 0))
        label = f"x{xmax}"
        windows.append(label)
        a1 = float(d.get("meta", {}).get("a1", a1))
        gate_mode = str(d.get("meta", {}).get("gate_mode", "unknown"))
        for r in d.get("rows", []):
            w = int(r["search_window"])
            by_w.setdefault(w, []).append(
                {
                    "window": label,
                    "delta": float(r["delta_eff"]),
                    "q": float(r["q_cofinal_grid"]),
                    "rr": float(r["rr_cofinal_grid"]),
                    "selected_n": int(r["selected_count"]),
                    "b2_cofinal": float(r["b2_cofinal_grid"]),
                }
            )

    windows = sorted(set(windows))
    rows: List[Dict] = []
    robust_w: List[int] = []
    for w in sorted(by_w.keys()):
        arr = by_w[w]
        if len(arr) != len(windows):
            continue
        delta_min = min(z["delta"] for z in arr)
        delta_max = max(z["delta"] for z in arr)
        q_max = max(z["q"] for z in arr)
        rr_max = max(z["rr"] for z in arr)
        robust = bool(delta_min > 0.0)
        if robust:
            robust_w.append(w)
        rows.append(
            {
                "search_window": int(w),
                "window_count": len(arr),
                "delta_min": float(delta_min),
                "delta_max": float(delta_max),
                "q_max": float(q_max),
                "rr_max": float(rr_max),
                "robust": robust,
                "details": arr,
            }
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "files": [str(p) for p in paths],
            "windows": windows,
            "a1": float(a1),
            "gate_mode": gate_mode,
        },
        "rows": rows,
        "robust_search_windows": robust_w,
        "interpretation": {
            "note": (
                "Finite-window robustness only. "
                "Robust windows indicate constructive-gate parameter ranges with positive cofinal margin."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14/Tau21 Constructive Gate Multiwindow Summary ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Windows: `{', '.join(windows)}`")
    lines.append(f"- Gate mode: `{gate_mode}`")
    lines.append(f"- Base threshold: `a1={a1}`")
    lines.append(
        "- Robust search windows (`delta_min>0` across all windows): "
        + (", ".join(str(w) for w in robust_w) if robust_w else "none")
    )
    lines.append("")
    lines.append("| search_W | delta_min | delta_max | q_max | rr_max | robust |")
    lines.append("|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r['search_window']} | {r['delta_min']:.6f} | {r['delta_max']:.6f} | "
            f"{r['q_max']:.6f} | {r['rr_max']:.6f} | {str(r['robust']).lower()} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
