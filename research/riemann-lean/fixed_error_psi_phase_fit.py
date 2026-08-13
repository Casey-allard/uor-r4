#!/usr/bin/env python3
"""Fit single-mode log-phase model to E*(x)=psi(x)-x."""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import List, Tuple

import numpy as np
import sympy as sp


@dataclass
class FitRow:
    beta: float
    tau: float
    a: float
    b: float
    amplitude: float
    phase: float
    rms_ratio_all: float
    rms_ratio_tail: float
    p95_ratio_tail: float
    max_ratio_tail: float


def logspace_int(xmin: int, xmax: int, n: int) -> np.ndarray:
    vals = np.unique(np.floor(np.logspace(math.log10(xmin), math.log10(xmax), n)).astype(np.int64))
    vals = vals[(vals >= xmin) & (vals <= xmax)]
    return vals


def psi_events(xmax: int) -> List[Tuple[int, float]]:
    events: List[Tuple[int, float]] = []
    for p in sp.primerange(2, xmax + 1):
        lp = math.log(float(p))
        pk = int(p)
        while pk <= xmax:
            events.append((pk, lp))
            if pk > xmax // p:
                break
            pk *= p
    events.sort(key=lambda t: t[0])
    return events


def psi_at_samples(xs: np.ndarray, events: List[Tuple[int, float]]) -> np.ndarray:
    out = np.zeros(len(xs), dtype=np.float64)
    acc = 0.0
    j = 0
    m = len(events)
    for i, x in enumerate(xs):
        xi = int(x)
        while j < m and events[j][0] <= xi:
            acc += events[j][1]
            j += 1
        out[i] = acc
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description="Single-mode phase fit for psi(x)-x")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=10_000_000)
    ap.add_argument("--samples", type=int, default=800)
    ap.add_argument("--beta-min", type=float, default=0.50)
    ap.add_argument("--beta-max", type=float, default=0.64)
    ap.add_argument("--beta-step", type=float, default=0.01)
    ap.add_argument("--tau-min", type=float, default=4.0)
    ap.add_argument("--tau-max", type=float, default=80.0)
    ap.add_argument("--tau-count", type=int, default=180)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_phase_fit_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)

    xs = logspace_int(args.xmin, args.xmax, args.samples).astype(np.float64)
    logs = np.log(xs)

    events = psi_events(args.xmax)
    psi_vals = psi_at_samples(xs.astype(np.int64), events)
    e_vals = psi_vals - xs

    tail_start = int(0.75 * len(xs))
    tail_slice = slice(tail_start, len(xs))

    betas = np.arange(args.beta_min, args.beta_max + 1e-12, args.beta_step)
    taus = np.linspace(args.tau_min, args.tau_max, args.tau_count)

    rows: List[FitRow] = []
    for beta in betas:
        xpow = np.power(xs, beta)
        for tau in taus:
            c = xpow * np.cos(tau * logs)
            s = xpow * np.sin(tau * logs)
            A = np.column_stack([c, s])
            coeff, *_ = np.linalg.lstsq(A, e_vals, rcond=None)
            a = float(coeff[0])
            b = float(coeff[1])
            fit = A @ coeff
            resid = e_vals - fit
            ratio = np.abs(resid) / xpow
            ratio_tail = ratio[tail_slice]
            rows.append(
                FitRow(
                    beta=float(beta),
                    tau=float(tau),
                    a=a,
                    b=b,
                    amplitude=float(math.hypot(a, b)),
                    phase=float(math.atan2(-b, a)),
                    rms_ratio_all=float(np.sqrt(np.mean(ratio * ratio))),
                    rms_ratio_tail=float(np.sqrt(np.mean(ratio_tail * ratio_tail))),
                    p95_ratio_tail=float(np.quantile(ratio_tail, 0.95)),
                    max_ratio_tail=float(np.max(ratio_tail)),
                )
            )

    rows.sort(key=lambda r: (r.rms_ratio_tail, r.p95_ratio_tail, r.max_ratio_tail))
    top = rows[:12]

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": args.xmin,
            "xmax": args.xmax,
            "samples": int(len(xs)),
            "beta_min": args.beta_min,
            "beta_max": args.beta_max,
            "beta_step": args.beta_step,
            "tau_min": args.tau_min,
            "tau_max": args.tau_max,
            "tau_count": args.tau_count,
            "event_count": len(events),
        },
        "top_fits": [asdict(r) for r in top],
    }

    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Single-Mode Phase Fit ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Samples: `{len(xs)}`")
    lines.append(f"- Prime-power events: `{len(events)}`")
    lines.append(f"- Grid: beta in `[{args.beta_min}, {args.beta_max}]` step `{args.beta_step}`, tau count `{args.tau_count}`")
    lines.append("")
    lines.append("## Top Fits (sorted by tail RMS residual ratio)")
    lines.append("")
    lines.append("| rank | beta | tau | amplitude | rms tail | p95 tail | max tail |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|")
    for i, r in enumerate(top, start=1):
        lines.append(
            f"| {i} | {r.beta:.4f} | {r.tau:.4f} | {r.amplitude:.6f} | "
            f"{r.rms_ratio_tail:.6e} | {r.p95_ratio_tail:.6e} | {r.max_ratio_tail:.6e} |"
        )
    lines.append("")
    lines.append("Interpretation: finite-window empirical fit only; not a formal explicit-formula proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
