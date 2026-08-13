#!/usr/bin/env python3
"""Probe canonical Chebyshev error E*(x) = psi(x) - x on sampled windows."""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import dataclass, asdict
from datetime import date
from pathlib import Path
from typing import List, Tuple

import numpy as np
import sympy as sp


@dataclass
class BetaStats:
    beta: float
    max_ratio_all: float
    max_ratio_tail: float
    median_ratio_tail: float
    p95_ratio_tail: float


def parse_beta_list(text: str) -> List[float]:
    out = []
    for raw in text.split(","):
        raw = raw.strip()
        if raw:
            out.append(float(raw))
    if not out:
        raise ValueError("empty beta list")
    return out


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
    ap = argparse.ArgumentParser(description="Fixed-error psi(x)-x oscillation probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=10_000_000)
    ap.add_argument("--samples", type=int, default=800)
    ap.add_argument("--betas", type=str, default="0.50,0.52,0.55,0.58,0.60,0.62")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    xmin = args.xmin
    xmax = args.xmax
    samples = args.samples
    betas = parse_beta_list(args.betas)
    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)

    xs = logspace_int(xmin, xmax, samples)
    if len(xs) < 10:
        raise RuntimeError("not enough sample points")

    events = psi_events(xmax)
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - xs.astype(np.float64)
    abs_e = np.abs(e_vals)

    tail_idx = int(0.75 * len(xs))
    xs_tail = xs[tail_idx:]
    abs_e_tail = abs_e[tail_idx:]

    beta_stats: List[BetaStats] = []
    for beta in betas:
        ratio = abs_e / np.power(xs.astype(np.float64), beta)
        ratio_tail = abs_e_tail / np.power(xs_tail.astype(np.float64), beta)
        beta_stats.append(
            BetaStats(
                beta=beta,
                max_ratio_all=float(np.max(ratio)),
                max_ratio_tail=float(np.max(ratio_tail)),
                median_ratio_tail=float(np.median(ratio_tail)),
                p95_ratio_tail=float(np.quantile(ratio_tail, 0.95)),
            )
        )

    sign_changes = int(np.sum(np.sign(e_vals[1:]) != np.sign(e_vals[:-1])))
    pos_count = int(np.sum(e_vals > 0))
    neg_count = int(np.sum(e_vals < 0))

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": xmin,
            "xmax": xmax,
            "samples": samples,
            "betas": betas,
            "event_count": len(events),
        },
        "series": {
            "count": int(len(xs)),
            "sign_changes": sign_changes,
            "positive_count": pos_count,
            "negative_count": neg_count,
            "min_E": float(np.min(e_vals)),
            "max_E": float(np.max(e_vals)),
        },
        "beta_stats": [asdict(x) for x in beta_stats],
    }

    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{xmin}, {xmax}]`")
    lines.append(f"- Samples: `{len(xs)}`")
    lines.append(f"- Prime-power events used for psi(x): `{len(events)}`")
    lines.append(f"- Sign changes in sampled sequence: `{sign_changes}`")
    lines.append(f"- Positive count: `{pos_count}`")
    lines.append(f"- Negative count: `{neg_count}`")
    lines.append("")
    lines.append("## Beta Ratios")
    lines.append("")
    lines.append("| beta | max all | max tail (last 25%) | median tail | p95 tail |")
    lines.append("|---:|---:|---:|---:|---:|")
    for b in beta_stats:
        lines.append(
            f"| {b.beta:.4f} | {b.max_ratio_all:.6e} | {b.max_ratio_tail:.6e} | "
            f"{b.median_ratio_tail:.6e} | {b.p95_ratio_tail:.6e} |"
        )
    lines.append("")
    lines.append("Interpretation: finite-window numeric probe only; not a theorem-grade Ω proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
