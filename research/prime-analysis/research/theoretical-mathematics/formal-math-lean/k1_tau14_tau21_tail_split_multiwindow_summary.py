#!/usr/bin/env python3
"""Aggregate tau14/tau21 tail-split probes by mode count across windows."""

from __future__ import annotations

import argparse
import json
from datetime import date
from pathlib import Path
from typing import Dict, List


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau14/tau21 tail-split multiwindow summary")
    ap.add_argument("--files", type=str, required=True, help="Comma-separated tail-split JSON paths")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_tau14_tau21_tail_split_multiwindow_summary_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    paths = [Path(p.strip()) for p in args.files.split(",") if p.strip()]
    if not paths:
        raise ValueError("files must be non-empty")

    by_m: Dict[int, List[Dict]] = {}
    windows: List[str] = []
    a1 = 0.98
    for p in paths:
        d = json.loads(p.read_text(encoding="utf-8"))
        xmax = int(d.get("meta", {}).get("xmax", 0))
        label = f"x{xmax}"
        windows.append(label)
        a1 = float(d.get("meta", {}).get("a1", a1))
        for r in d.get("rows", []):
            m = int(r["mode_count"])
            by_m.setdefault(m, []).append(
                {
                    "window": label,
                    "delta_split": float(r["delta_split"]),
                    "delta_split_neg": float(r.get("delta_split_neg", r["delta_split"])),
                    "delta_raw": float(r["delta_raw"]),
                    "delta_raw_neg": float(r.get("delta_raw_neg", r["delta_raw"])),
                    "q_split": float(r["q_split_cofinal"]),
                    "q_split_neg": float(r.get("q_split_neg_cofinal", r["q_split_cofinal"])),
                    "q_raw": float(r["q_raw_cofinal"]),
                    "q_raw_neg": float(r.get("q_raw_neg_cofinal", r["q_raw_cofinal"])),
                    "q_tail": float(r["q_tail_cofinal"]),
                    "q_tail_neg": float(r.get("q_tail_neg_cofinal", r["q_tail_cofinal"])),
                    "q_rem": float(r["q_rem_cofinal"]),
                    "q_rem_neg": float(r.get("q_rem_neg_cofinal", r["q_rem_cofinal"])),
                    "tri_viol": float(r["triangle_violation_max"]),
                }
            )

    windows = sorted(set(windows))
    rows: List[Dict] = []
    robust_m: List[int] = []
    robust_m_neg: List[int] = []
    for m in sorted(by_m.keys()):
        arr = by_m[m]
        if len(arr) != len(windows):
            continue
        delta_split_min = min(z["delta_split"] for z in arr)
        delta_split_neg_min = min(z["delta_split_neg"] for z in arr)
        delta_split_max = max(z["delta_split"] for z in arr)
        delta_raw_min = min(z["delta_raw"] for z in arr)
        delta_raw_neg_min = min(z["delta_raw_neg"] for z in arr)
        q_split_max = max(z["q_split"] for z in arr)
        q_split_neg_max = max(z["q_split_neg"] for z in arr)
        q_raw_max = max(z["q_raw"] for z in arr)
        q_raw_neg_max = max(z["q_raw_neg"] for z in arr)
        q_tail_max = max(z["q_tail"] for z in arr)
        q_tail_neg_max = max(z["q_tail_neg"] for z in arr)
        q_rem_max = max(z["q_rem"] for z in arr)
        q_rem_neg_max = max(z["q_rem_neg"] for z in arr)
        tri_viol_max = max(z["tri_viol"] for z in arr)
        robust = bool(delta_split_min > 0.0)
        robust_neg = bool(delta_split_neg_min > 0.0)
        if robust:
            robust_m.append(m)
        if robust_neg:
            robust_m_neg.append(m)
        rows.append(
            {
                "mode_count": int(m),
                "window_count": len(arr),
                "delta_split_min": float(delta_split_min),
                "delta_split_neg_min": float(delta_split_neg_min),
                "delta_split_max": float(delta_split_max),
                "delta_raw_min": float(delta_raw_min),
                "delta_raw_neg_min": float(delta_raw_neg_min),
                "q_split_max": float(q_split_max),
                "q_split_neg_max": float(q_split_neg_max),
                "q_raw_max": float(q_raw_max),
                "q_raw_neg_max": float(q_raw_neg_max),
                "q_tail_max": float(q_tail_max),
                "q_tail_neg_max": float(q_tail_neg_max),
                "q_rem_max": float(q_rem_max),
                "q_rem_neg_max": float(q_rem_neg_max),
                "triangle_violation_max": float(tri_viol_max),
                "robust_split": robust,
                "robust_split_neg": robust_neg,
                "details": arr,
            }
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "files": [str(p) for p in paths],
            "windows": windows,
            "a1": float(a1),
        },
        "rows": rows,
        "robust_mode_counts_split": robust_m,
        "robust_mode_counts_split_neg": robust_m_neg,
        "interpretation": {
            "note": (
                "Finite-window diagnostics only. "
                "robust_mode_counts_split marks M where split-bound margin stays positive in all windows."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14/Tau21 Tail-Split Multiwindow Summary ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Windows: `{', '.join(windows)}`")
    lines.append(f"- Base threshold: `a1={a1}`")
    lines.append(
        "- Robust mode counts (`delta_split_min>0` across all windows): "
        + (", ".join(str(m) for m in robust_m) if robust_m else "none")
    )
    lines.append(
        "- Robust mode counts (`delta_split_neg_min>0` across all windows): "
        + (", ".join(str(m) for m in robust_m_neg) if robust_m_neg else "none")
    )
    lines.append("")
    lines.append("| M | d_split_min | d_split_neg_min | d_raw_neg_min | q_split_max | q_split_neg_max | q_raw_neg_max | robust_abs | robust_neg |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r['mode_count']} | {r['delta_split_min']:.6f} | {r['delta_split_neg_min']:.6f} | "
            f"{r['delta_raw_neg_min']:.6f} | {r['q_split_max']:.6f} | {r['q_split_neg_max']:.6f} | "
            f"{r['q_raw_neg_max']:.6f} | {str(r['robust_split']).lower()} | {str(r['robust_split_neg']).lower()} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
