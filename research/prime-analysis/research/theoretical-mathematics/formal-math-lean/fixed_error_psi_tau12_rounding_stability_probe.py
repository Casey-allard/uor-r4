#!/usr/bin/env python3
"""Rounding-stability probe for tau1-anchor constructive subsequences.

Anchors:
  x*_k = exp((2*pi*k - phi1)/tau1), so tau1*log(x*_k)+phi1 = 2*pi*k.
Rounded integers:
  n_k = round(x*_k).

We probe whether rounding preserves:
  - high tau1 alignment |cos1(n_k)| >= a1
  - nonnegative tau2 phase cos2(n_k) >= 0
especially on anchor subsets with cos2(x*_k) >= c0 > 0.
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
from fixed_error_psi_tau14_tau21_constructive_gate_probe import _phase_anchors
from fixed_error_psi_tau14_tau21_phase_gate_probe import _fit_two_mode


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


@dataclass
class C0Row:
    c0: float
    anchors_c0_count: int
    anchors_c0_fraction: float
    sign_preserved_count: int
    sign_preserved_fraction: float
    cos1_alignment_count: int
    cos1_alignment_fraction: float
    both_count: int
    both_fraction: float
    min_cos2_anchor_in_set: float
    min_cos2_integer_in_set: float
    min_abs_cos1_integer_in_set: float


def main() -> None:
    ap = argparse.ArgumentParser(description="Tau1 anchor rounding stability probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--c0-list", type=str, default="0.0,0.1,0.2,0.3,0.4,0.5")
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau12_rounding_stability_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.a1 <= 0.0 or args.a1 > 1.0:
        raise ValueError("a1 must be in (0,1]")
    c0_list = sorted(set(parse_float_list(args.c0_list)))

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

    rho = float(args.tau2 / args.tau1)
    beta0 = float(phi2 - rho * phi1)
    two_pi = 2.0 * math.pi

    x_star_vals: List[float] = []
    n_vals: List[int] = []
    cos2_anchor_vals: List[float] = []
    cos2_int_vals: List[float] = []
    abs_cos1_int_vals: List[float] = []
    dlog_vals: List[float] = []
    d1_vals: List[float] = []
    d2_vals: List[float] = []
    d2_bound_vals: List[float] = []

    for k in anchors:
        x_star = math.exp((two_pi * float(k) - phi1) / float(args.tau1))
        n = int(round(x_star))
        n = max(int(args.xmin), min(int(args.xmax), n))
        nf = float(max(2, n))
        t_int = math.log(nf)

        dlog = float(t_int - math.log(x_star))
        d1 = float(float(args.tau1) * dlog)
        d2 = float(float(args.tau2) * dlog)
        denom = max(1.0, x_star - 0.5)
        d2_bound = float(float(args.tau2) * 0.5 / denom)

        theta_anchor = two_pi * rho * float(k) + beta0
        cos2_anchor = float(math.cos(theta_anchor))
        cos2_int = float(math.cos(float(args.tau2) * t_int + phi2))
        abs_cos1_int = float(abs(math.cos(float(args.tau1) * t_int + phi1)))

        x_star_vals.append(float(x_star))
        n_vals.append(int(n))
        cos2_anchor_vals.append(cos2_anchor)
        cos2_int_vals.append(cos2_int)
        abs_cos1_int_vals.append(abs_cos1_int)
        dlog_vals.append(dlog)
        d1_vals.append(d1)
        d2_vals.append(d2)
        d2_bound_vals.append(d2_bound)

    cos2_anchor_arr = np.array(cos2_anchor_vals, dtype=np.float64)
    cos2_int_arr = np.array(cos2_int_vals, dtype=np.float64)
    abs_cos1_int_arr = np.array(abs_cos1_int_vals, dtype=np.float64)

    rows: List[C0Row] = []
    n_anchor = len(anchors)
    for c0 in c0_list:
        m = cos2_anchor_arr >= float(c0)
        n_c0 = int(np.count_nonzero(m))
        if n_c0 == 0:
            rows.append(
                C0Row(
                    c0=float(c0),
                    anchors_c0_count=0,
                    anchors_c0_fraction=0.0,
                    sign_preserved_count=0,
                    sign_preserved_fraction=0.0,
                    cos1_alignment_count=0,
                    cos1_alignment_fraction=0.0,
                    both_count=0,
                    both_fraction=0.0,
                    min_cos2_anchor_in_set=0.0,
                    min_cos2_integer_in_set=0.0,
                    min_abs_cos1_integer_in_set=0.0,
                )
            )
            continue
        sign_ok = cos2_int_arr[m] >= 0.0
        cos1_ok = abs_cos1_int_arr[m] >= float(args.a1)
        both = sign_ok & cos1_ok
        rows.append(
            C0Row(
                c0=float(c0),
                anchors_c0_count=n_c0,
                anchors_c0_fraction=float(n_c0 / n_anchor),
                sign_preserved_count=int(np.count_nonzero(sign_ok)),
                sign_preserved_fraction=float(np.mean(sign_ok)),
                cos1_alignment_count=int(np.count_nonzero(cos1_ok)),
                cos1_alignment_fraction=float(np.mean(cos1_ok)),
                both_count=int(np.count_nonzero(both)),
                both_fraction=float(np.mean(both)),
                min_cos2_anchor_in_set=float(np.min(cos2_anchor_arr[m])),
                min_cos2_integer_in_set=float(np.min(cos2_int_arr[m])),
                min_abs_cos1_integer_in_set=float(np.min(abs_cos1_int_arr[m])),
            )
        )

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
            "c0_list": c0_list,
            "anchor_count": n_anchor,
            "include_decay_term": bool(args.include_decay_term),
        },
        "fit": {
            "A1": float(fit2["A1"]),
            "A2": float(fit2["A2"]),
            "phi1": phi1,
            "phi2": phi2,
            "rho": rho,
            "beta0": beta0,
            "rmse_global_fit": float(fit2["rmse"]),
        },
        "rounding_error": {
            "max_abs_dlog": float(np.max(np.abs(np.array(dlog_vals, dtype=np.float64)))),
            "max_abs_d1": float(np.max(np.abs(np.array(d1_vals, dtype=np.float64)))),
            "max_abs_d2": float(np.max(np.abs(np.array(d2_vals, dtype=np.float64)))),
            "max_d2_bound": float(np.max(np.array(d2_bound_vals, dtype=np.float64))),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window rounding-stability evidence only. "
                "Supports constructive C2 planning by quantifying phase-error/sign robustness."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau12 Rounding Stability Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- `a1 = {args.a1}`")
    lines.append(f"- Anchor count: `{n_anchor}`")
    lines.append(f"- max `|d2|`: `{payload['rounding_error']['max_abs_d2']:.6e}`")
    lines.append("")
    lines.append("| c0 | frac(anchor cos2>=c0) | sign_preserved_frac | cos1_align_frac | both_frac | min_cos2_int | min_|cos1|_int |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.c0:.3f} | {r.anchors_c0_fraction:.6f} | {r.sign_preserved_fraction:.6f} | "
            f"{r.cos1_alignment_fraction:.6f} | {r.both_fraction:.6f} | {r.min_cos2_integer_in_set:.6f} | "
            f"{r.min_abs_cos1_integer_in_set:.6f} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
