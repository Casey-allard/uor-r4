#!/usr/bin/env python3
"""O2 checker: partition-based sum-to-integral domination for A2 tail term."""

from __future__ import annotations

import argparse
import bisect
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple

import spinning_top_r4_candidate_a as s4
from a2_infinite_tail_uplift import (
    n_err_explicit,
    n_main_rvm,
    nprime_upper_affine,
    nprime_upper_rvm_explicit,
    nprime_upper_from_n_bound_diff,
)
from prime_geometry_loop import load_zeta_zeros_file


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def weight_fp(t: float, kernel: str, scale: float) -> float:
    w = s4.zero_weight(t, kernel, scale)
    return w * (t / math.sqrt(0.25 + t * t))


def nprime_up_at(t: float, cfg: Dict[str, Any]) -> float:
    model = str(cfg.get("density_model", "affine"))
    if model == "n_diff_explicit":
        return nprime_upper_from_n_bound_diff(
            t=t,
            h=float(cfg.get("nbound_h", 1.0)),
            c1=float(cfg.get("nbound_c1", 0.1038)),
            c2=float(cfg.get("nbound_c2", 0.2573)),
            c3=float(cfg.get("nbound_c3", 9.3675)),
        )
    if model == "rvm_explicit":
        return nprime_upper_rvm_explicit(
            t=t,
            c0=float(cfg.get("rvm_c0", 0.5)),
            c1=float(cfg.get("rvm_c1", 1.0)),
        )
    return nprime_upper_affine(
        t=t,
        a_n=float(cfg.get("density_a", 1.0 / (2.0 * math.pi))),
        b_n=float(cfg.get("density_b", 0.5)),
    )


def n_inc_up(a: float, b: float, cfg: Dict[str, Any]) -> float:
    model = str(cfg.get("density_model", "affine"))
    if model == "n_diff_explicit":
        e = lambda t: n_err_explicit(
            t=t,
            c1=float(cfg.get("nbound_c1", 0.1038)),
            c2=float(cfg.get("nbound_c2", 0.2573)),
            c3=float(cfg.get("nbound_c3", 9.3675)),
        )
        m = lambda t: n_main_rvm(t)
        return max(0.0, (m(b) - m(a)) + e(b) + e(a))
    # fallback numerical integral of nprime_up
    n0 = nprime_up_at(a, cfg)
    n1 = nprime_up_at(b, cfg)
    return max(0.0, 0.5 * (n0 + n1) * (b - a))


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 sum-to-integral domination checker")
    ap.add_argument(
        "--a2-infinite-json",
        default="research/output/a2_infinite_tail_uplift_2026-02-17_n_diff_explicit_hsw2022_sf3p5.json",
    )
    ap.add_argument("--zeta-zeros-file", default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--bin-width", type=float, default=1.0)
    ap.add_argument(
        "--output",
        default="research/output/a2_sum_to_integral_domination_checker_2026-02-17.json",
    )
    args = ap.parse_args()

    src = read_json(args.a2_infinite_json)
    cfg = dict(src["config"])
    tail = src["tail_majorant"]
    kernel = str(cfg["kernel"])
    scale = float(cfg["kernel_scale"])
    m_ref = int(cfg["m_ref"])
    m_grid = [int(m) for m in cfg["m_grid"]]
    gamma_ref = float(tail["gamma_ref"])

    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    if len(zeros) < m_ref:
        raise ValueError("insufficient zeros for m_ref")
    zref = zeros[:m_ref]
    zarr = [float(z) for z in zref]

    rows: List[Dict[str, Any]] = []
    all_hold = True
    worst_gap = -1e99
    worst_count_gap = -1e99
    max_ratio = 0.0

    h = max(1e-6, float(args.bin_width))
    for m in m_grid:
        if m <= 0 or m >= m_ref:
            continue
        t0 = float(zarr[m - 1])

        # Exact discrete tail over known zero list.
        discrete = sum(weight_fp(g, kernel, scale) for g in zarr[m:])

        # Partition-based upper bound using N-increment upper control.
        t = t0
        upper = 0.0
        count_gap_max_m = -1e99
        while t < gamma_ref:
            b = min(gamma_ref, t + h)
            # count zeros in [t,b]
            li = bisect.bisect_left(zarr, t)
            ri = bisect.bisect_right(zarr, b)
            cnt = max(0, ri - li)

            cnt_up = n_inc_up(t, b, cfg)
            count_gap = float(cnt) - float(cnt_up)
            count_gap_max_m = max(count_gap_max_m, count_gap)

            wmax = max(weight_fp(t, kernel, scale), weight_fp(b, kernel, scale))
            upper += max(0.0, cnt_up) * max(0.0, wmax)
            t = b

        gap = float(discrete - upper)
        ratio = float(discrete / max(1e-30, upper))
        hold = gap <= 1e-12 and count_gap_max_m <= 1e-12
        all_hold = all_hold and hold
        worst_gap = max(worst_gap, gap)
        worst_count_gap = max(worst_count_gap, count_gap_max_m)
        max_ratio = max(max_ratio, ratio)
        rows.append(
            {
                "M": m,
                "t0": t0,
                "discrete_tail_sum": discrete,
                "partition_upper_bound": upper,
                "gap_discrete_minus_upper": gap,
                "ratio_discrete_over_upper": ratio,
                "max_count_gap_actual_minus_upper": count_gap_max_m,
                "holds": hold,
            }
        )

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "input": args.a2_infinite_json,
        "config": {
            "density_model": cfg.get("density_model", "affine"),
            "kernel": kernel,
            "kernel_scale": scale,
            "m_ref": m_ref,
            "m_grid": m_grid,
            "gamma_ref": gamma_ref,
            "bin_width": h,
            "nbound_c1": cfg.get("nbound_c1", None),
            "nbound_c2": cfg.get("nbound_c2", None),
            "nbound_c3": cfg.get("nbound_c3", None),
            "nbound_h": cfg.get("nbound_h", None),
        },
        "checks": {
            "all_hold": bool(all_hold),
            "worst_gap_discrete_minus_upper": float(worst_gap),
            "worst_count_gap_actual_minus_upper": float(worst_count_gap),
            "max_ratio_discrete_over_upper": float(max_ratio),
        },
        "rows": rows,
        "note": (
            "This is a finite-reference checker for the sum-to-integral domination shape "
            "used in O2; theorem closure still requires asymptotic argument."
        ),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# A2 Sum-To-Integral Domination Checker",
        "",
        f"Generated: {payload['timestamp_utc']}",
        f"- Input: `{args.a2_infinite_json}`",
        f"- density_model: `{payload['config']['density_model']}`",
        f"- kernel: `{kernel}`, scale=`{scale}`",
        f"- bin_width: `{h}`",
        f"- all_hold: `{payload['checks']['all_hold']}`",
        f"- worst_gap_discrete_minus_upper: `{payload['checks']['worst_gap_discrete_minus_upper']:.12g}`",
        f"- worst_count_gap_actual_minus_upper: `{payload['checks']['worst_count_gap_actual_minus_upper']:.12g}`",
        "",
        "| M | discrete_tail_sum | partition_upper_bound | gap_discrete_minus_upper | ratio_discrete_over_upper | count_gap_max | hold |",
        "|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for r in rows:
        lines.append(
            f"| {r['M']} | {r['discrete_tail_sum']:.12g} | {r['partition_upper_bound']:.12g} | "
            f"{r['gap_discrete_minus_upper']:.12g} | {r['ratio_discrete_over_upper']:.12g} | "
            f"{r['max_count_gap_actual_minus_upper']:.12g} | {r['holds']} |"
        )
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
