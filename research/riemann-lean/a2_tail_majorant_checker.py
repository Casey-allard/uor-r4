#!/usr/bin/env python3
"""Check candidate analytic tail majorants against cached finite A2 tail table."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List

from a2_infinite_tail_uplift import tail_integral_majorant


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def parse_ms(raw: str) -> List[int]:
    return [int(x.strip()) for x in raw.split(",") if x.strip()]


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 tail majorant checker")
    ap.add_argument(
        "--a2-infinite-json",
        default="research/output/a2_infinite_tail_uplift_refresh_2026-02-17_sf3p5.json",
    )
    ap.add_argument("--m-grid", default="", help="optional override comma-list")
    ap.add_argument("--density-a", type=float, default=1.0 / (2.0 * math.pi))
    ap.add_argument("--density-b", type=float, default=0.5)
    ap.add_argument("--density-model", type=str, default="auto", choices=["auto", "affine", "rvm_explicit"])
    ap.add_argument("--rvm-c0", type=float, default=0.5)
    ap.add_argument("--rvm-c1", type=float, default=1.0)
    ap.add_argument("--nbound-c1", type=float, default=0.1038)
    ap.add_argument("--nbound-c2", type=float, default=0.2573)
    ap.add_argument("--nbound-c3", type=float, default=9.3675)
    ap.add_argument("--nbound-h", type=float, default=1.0)
    ap.add_argument("--integral-steps", type=int, default=4000)
    ap.add_argument(
        "--output",
        default="research/output/a2_tail_majorant_checker_2026-02-17.json",
    )
    args = ap.parse_args()

    src = read_json(args.a2_infinite_json)
    cfg = src["config"]
    tcfg = src["tail_majorant"]["config"]
    gamma_ref = float(src["tail_majorant"]["gamma_ref"])
    kernel = str(cfg["kernel"])
    scale = float(cfg["kernel_scale"])
    density_model = str(args.density_model)
    if density_model == "auto":
        density_model = str(cfg.get("density_model", "affine"))
    use_cfg = args.density_model == "auto"
    density_a = float(args.density_a if not use_cfg else cfg.get("density_a", args.density_a))
    density_b = float(args.density_b if not use_cfg else cfg.get("density_b", args.density_b))
    rvm_c0 = float(args.rvm_c0 if not use_cfg else cfg.get("rvm_c0", args.rvm_c0))
    rvm_c1 = float(args.rvm_c1 if not use_cfg else cfg.get("rvm_c1", args.rvm_c1))
    nbound_c1 = float(args.nbound_c1 if not use_cfg else cfg.get("nbound_c1", args.nbound_c1))
    nbound_c2 = float(args.nbound_c2 if not use_cfg else cfg.get("nbound_c2", args.nbound_c2))
    nbound_c3 = float(args.nbound_c3 if not use_cfg else cfg.get("nbound_c3", args.nbound_c3))
    nbound_h = float(args.nbound_h if not use_cfg else cfg.get("nbound_h", args.nbound_h))
    m_grid = parse_ms(args.m_grid) if args.m_grid.strip() else [int(m) for m in cfg["m_grid"]]

    rows = []
    all_hold = True
    worst_gap = -1e99
    for m in m_grid:
        key = str(m)
        if key not in src["tail_majorant"]["tau_by_M"]:
            continue
        tau_finite_ref = float(src["tail_majorant"]["tau_by_M"][key]["tau_finite_ref"])
        tau_extra_old = float(src["tail_majorant"]["tau_by_M"][key]["tau_infinite_extra_majorant"])
        tau_extra_new = float(
            tail_integral_majorant(
                t0=gamma_ref,
                kernel=kernel,
                scale=scale,
                density_model=density_model,
                a_n=density_a,
                b_n=density_b,
                rvm_c0=rvm_c0,
                rvm_c1=rvm_c1,
                nbound_c1=nbound_c1,
                nbound_c2=nbound_c2,
                nbound_c3=nbound_c3,
                nbound_h=nbound_h,
                steps=max(200, int(args.integral_steps)),
            )
        )
        tau_inf_new = tau_finite_ref + tau_extra_new
        gap = tau_inf_new - tau_finite_ref
        holds = gap >= -1e-30
        all_hold = all_hold and holds
        worst_gap = max(worst_gap, gap)
        rows.append(
            {
                "M": m,
                "tau_finite_ref": tau_finite_ref,
                "tau_extra_old": tau_extra_old,
                "tau_extra_new": tau_extra_new,
                "tau_infinite_new": tau_inf_new,
                "gap_new_minus_finite": gap,
                "holds_upper_bound": holds,
            }
        )

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "input": args.a2_infinite_json,
        "candidate_density": {
            "model": density_model,
            "a": density_a,
            "b": density_b,
            "rvm_c0": rvm_c0,
            "rvm_c1": rvm_c1,
            "nbound_c1": nbound_c1,
            "nbound_c2": nbound_c2,
            "nbound_c3": nbound_c3,
            "nbound_h": nbound_h,
        },
        "kernel": {"name": kernel, "scale": scale},
        "m_grid": m_grid,
        "checks": {
            "all_hold": bool(all_hold),
            "worst_gap_new_minus_finite": float(worst_gap),
        },
        "rows": rows,
        "cache_reuse_note": "Reuses cached finite-tail table from a2_infinite_tail_uplift artifact.",
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# A2 Tail Majorant Checker",
        "",
        f"Generated: {payload['timestamp_utc']}",
        f"- Input: `{args.a2_infinite_json}`",
        f"- Candidate density model: `{density_model}`",
        f"- Candidate density: `a={density_a}`, `b={density_b}`, `rvm_c0={rvm_c0}`, `rvm_c1={rvm_c1}`, "
        f"`nbound_c1={nbound_c1}`, `nbound_c2={nbound_c2}`, `nbound_c3={nbound_c3}`, `nbound_h={nbound_h}`",
        f"- all_hold: `{payload['checks']['all_hold']}`",
        f"- worst_gap_new_minus_finite: `{payload['checks']['worst_gap_new_minus_finite']:.12g}`",
        "",
        "| M | tau_finite_ref | tau_extra_new | tau_infinite_new | gap_new_minus_finite | hold |",
        "|---:|---:|---:|---:|---:|---:|",
    ]
    for r in rows:
        lines.append(
            f"| {r['M']} | {r['tau_finite_ref']:.12g} | {r['tau_extra_new']:.12g} | "
            f"{r['tau_infinite_new']:.12g} | {r['gap_new_minus_finite']:.12g} | {r['holds_upper_bound']} |"
        )
    md.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
