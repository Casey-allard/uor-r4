#!/usr/bin/env python3
"""Compute cofinal q-thresholds from explicit C*x^{-eta} bounds.

If q(x) <= C * x^{-eta} with eta>0, then for any q_target>0:
  x >= (C / q_target)^(1/eta)  =>  q(x) <= q_target.

This script turns explicit-ledger constants into concrete cofinal thresholds
for the one-sided split target q < a1.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date
from pathlib import Path
from typing import Dict, List


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(float(tok))
    if not vals:
        raise ValueError("empty float list")
    return vals


def threshold_x(C: float, eta: float, q_target: float) -> float:
    if C <= 0.0:
        return 1.0
    if eta <= 0.0:
        return float("inf")
    if q_target <= 0.0:
        return float("inf")
    if C <= q_target:
        return 1.0
    return float((C / q_target) ** (1.0 / eta))


def load_ledger(path: Path) -> Dict[str, float]:
    d = json.loads(path.read_text(encoding="utf-8"))
    cfg = d["config"]
    cc = d["combined_candidates"]
    return {
        "beta": float(cfg["beta"]),
        "eta": float(cfg["eta"]),
        "theta": float(cfg["theta"]),
        "x1": float(cfg["x1"]),
        "c_total_A": float(cc["c_total_A"]),
        "c_total_B": float(cc["c_total_B"]),
        "c_band_eta": float(d["explicit_band_bound"]["c_band_eta"]),
        "c_high_selected": float(d["external_high_envelopes_eta"]["c_selected_min"]),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Cofinal q-thresholds from explicit constants")
    ap.add_argument(
        "--ledger-files",
        type=str,
        required=True,
        help="Comma-separated explicit-ledger JSON files",
    )
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument(
        "--q-fractions",
        type=str,
        default="0.9,0.7,0.5",
        help="Use q_target = fraction * a1",
    )
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_q_cofinal_threshold_from_explicit_constants_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.a1 <= 0.0:
        raise ValueError("a1 must be positive")

    ledger_files = [Path(s.strip()) for s in args.ledger_files.split(",") if s.strip()]
    if not ledger_files:
        raise ValueError("no ledger files")
    q_fracs = sorted(set(parse_float_list(args.q_fractions)), reverse=True)
    if any(f <= 0.0 for f in q_fracs):
        raise ValueError("q-fractions must be > 0")

    rows: List[Dict[str, object]] = []
    for lf in ledger_files:
        info = load_ledger(lf)
        eta = info["eta"]
        x1 = info["x1"]
        out_row: Dict[str, object] = {
            "ledger_file": str(lf),
            "beta": info["beta"],
            "eta": eta,
            "theta": info["theta"],
            "x1": x1,
            "constants": {
                "c_total_A": info["c_total_A"],
                "c_total_B": info["c_total_B"],
                "c_band_eta": info["c_band_eta"],
                "c_high_selected": info["c_high_selected"],
            },
            "thresholds": [],
        }
        for frac in q_fracs:
            q_target = frac * float(args.a1)
            xA = max(x1, threshold_x(info["c_total_A"], eta, q_target))
            xB = max(x1, threshold_x(info["c_total_B"], eta, q_target))
            out_row["thresholds"].append(
                {
                    "q_fraction_of_a1": frac,
                    "q_target": q_target,
                    "x_threshold_model_A": xA,
                    "x_threshold_model_B": xB,
                }
            )
        rows.append(out_row)

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "a1": float(args.a1),
            "q_fractions": q_fracs,
            "ledger_files": [str(p) for p in ledger_files],
        },
        "rows": rows,
        "interpretation": {
            "note": (
                "These thresholds convert finite explicit constants into cofinal q<a1 statements. "
                "Mathematical closure still requires theorem-grade validity of the underlying "
                "C*x^{-eta} bounds."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Cofinal q-Thresholds ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- `a1={args.a1}`")
    lines.append(f"- `q_fractions={q_fracs}`")
    lines.append("")
    for row in rows:
        lines.append(f"## Ledger: `{row['ledger_file']}`")
        lines.append(
            f"- beta={float(row['beta']):.4f}, eta={float(row['eta']):.4f}, theta={float(row['theta']):.4f}, x1={float(row['x1']):.3e}"
        )
        c = row["constants"]
        lines.append(
            f"- constants: c_total_A={float(c['c_total_A']):.6g}, c_total_B={float(c['c_total_B']):.6g}, "
            f"c_band={float(c['c_band_eta']):.6g}, c_high={float(c['c_high_selected']):.6g}"
        )
        lines.append("")
        lines.append("| q_target | x_threshold_model_A | x_threshold_model_B |")
        lines.append("|---:|---:|---:|")
        for th in row["thresholds"]:
            lines.append(
                f"| {float(th['q_target']):.6f} | {float(th['x_threshold_model_A']):.6e} | {float(th['x_threshold_model_B']):.6e} |"
            )
        lines.append("")
    lines.append("Threshold report only; not a theorem proof.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

