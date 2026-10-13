#!/usr/bin/env python3
"""Probe constructive anchor existence for C1/C2-style lemmas.

Given fitted (tau1,tau2,phi1,phi2), analyze tau1 anchor centers and check:
  - center sign density: cos2(center) >= 0
  - neighborhood success: exists integer in [center-W, center+W] with
      |cos1| >= a1 and cos2 >= 0
This is finite-window evidence for constructive subsequence existence.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import List

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from fixed_error_psi_tau14_tau21_constructive_gate_probe import _anchor_center, _phase_anchors
from fixed_error_psi_tau14_tau21_phase_gate_probe import _fit_two_mode


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    if not vals:
        raise ValueError("empty int list")
    return vals


@dataclass
class WindowRow:
    search_window: int
    anchors_total: int
    center_nonnegative_count: int
    center_nonnegative_fraction: float
    success_count: int
    success_fraction: float
    fail_count: int
    min_cos2_on_success: float
    max_cos2_on_success: float
    min_abs_cos1_on_success: float


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau1/tau2 anchor existence probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--search-window-list", type=str, default="0,20,50,100,200,500,1000")
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau12_anchor_existence_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.a1 <= 0.0 or args.a1 > 1.0:
        raise ValueError("a1 must be in (0,1]")

    w_list = sorted(set(w for w in parse_int_list(args.search_window_list) if w >= 0))
    if not w_list:
        raise ValueError("search-window list empty")

    xs = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x = xs.astype(np.float64)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    y = (psi_vals - x) / np.power(x, float(args.beta))

    fit2 = _fit_two_mode(
        x=x,
        y=y,
        tau1=float(args.tau1),
        tau2=float(args.tau2),
        beta=float(args.beta),
        include_decay_term=bool(args.include_decay_term),
    )
    phi1 = float(fit2["phi1"])
    phi2 = float(fit2["phi2"])

    anchors = _phase_anchors(float(args.tau1), phi1, int(args.xmin), int(args.xmax))
    if not anchors:
        raise ValueError("no anchors in range")
    centers = [_anchor_center(k, float(args.tau1), phi1) for k in anchors]

    rows: List[WindowRow] = []
    for w in w_list:
        center_nonneg = 0
        success = 0
        fail = 0
        cos2_success: List[float] = []
        abs_cos1_success: List[float] = []
        for c in centers:
            c0 = max(int(args.xmin), min(int(args.xmax), int(c)))
            t0 = math.log(float(max(2, c0)))
            cos2_center = math.cos(float(args.tau2) * t0 + phi2)
            if cos2_center >= 0.0:
                center_nonneg += 1

            lo = max(int(args.xmin), c0 - int(w))
            hi = min(int(args.xmax), c0 + int(w))
            found = False
            best_cos2 = -1.0
            best_abs_cos1 = 0.0
            for n in range(lo, hi + 1):
                t = math.log(float(max(2, n)))
                abs_cos1 = abs(math.cos(float(args.tau1) * t + phi1))
                if abs_cos1 < float(args.a1):
                    continue
                cos2 = math.cos(float(args.tau2) * t + phi2)
                if cos2 < 0.0:
                    continue
                found = True
                if best_cos2 < 0.0 or cos2 < best_cos2:
                    best_cos2 = cos2
                    best_abs_cos1 = abs_cos1
            if found:
                success += 1
                cos2_success.append(best_cos2)
                abs_cos1_success.append(best_abs_cos1)
            else:
                fail += 1

        n_total = len(centers)
        rows.append(
            WindowRow(
                search_window=int(w),
                anchors_total=n_total,
                center_nonnegative_count=int(center_nonneg),
                center_nonnegative_fraction=float(center_nonneg / n_total),
                success_count=int(success),
                success_fraction=float(success / n_total),
                fail_count=int(fail),
                min_cos2_on_success=float(min(cos2_success) if cos2_success else 0.0),
                max_cos2_on_success=float(max(cos2_success) if cos2_success else 0.0),
                min_abs_cos1_on_success=float(min(abs_cos1_success) if abs_cos1_success else 0.0),
            )
        )

    rows.sort(key=lambda r: r.search_window)
    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "fit_samples": int(len(xs)),
            "beta": float(args.beta),
            "tau1": float(args.tau1),
            "tau2": float(args.tau2),
            "a1": float(args.a1),
            "search_window_list": w_list,
            "include_decay_term": bool(args.include_decay_term),
            "anchor_count": len(centers),
        },
        "fit": {
            "A1": float(fit2["A1"]),
            "A2": float(fit2["A2"]),
            "phi1": phi1,
            "phi2": phi2,
            "rmse_global_fit": float(fit2["rmse"]),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window anchor-existence evidence only. "
                "Supports constructive-gate viability diagnostics for C1/C2."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau1/Tau2 Anchor Existence Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- `a1={args.a1}`")
    lines.append(f"- Anchor count: `{len(centers)}`")
    lines.append("")
    lines.append("| W | center_nonneg_frac | success_frac | success_n | fail_n | min_cos2_success | min_abs_cos1_success |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.search_window} | {r.center_nonnegative_fraction:.6f} | {r.success_fraction:.6f} | "
            f"{r.success_count} | {r.fail_count} | {r.min_cos2_on_success:.6f} | {r.min_abs_cos1_on_success:.6f} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
