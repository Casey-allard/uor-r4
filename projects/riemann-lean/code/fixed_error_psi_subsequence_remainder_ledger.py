#!/usr/bin/env python3
"""Cofinal aligned-subsequence remainder ledger for E*(x)=psi(x)-x.

For fixed beta and tau, fit:
  E*(x)/x^beta ~= a*cos(tau log x) + b*sin(tau log x) [+ d*x^-beta]
with amplitude A and phase phi.

Then on aligned points (|cos(tau log x + phi)| >= a0), compute finite-grid
cofinal witnesses for:
  - remainder ratio condition: |R|/A <= a0 - delta
  - triangle lower condition:  A|cos|-|R| >= delta*A

This is finite-window empirical evidence, not a theorem.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from prime_geometry_loop import load_zeta_zeros_file


@dataclass
class BetaSubseqRow:
    beta: float
    tau: float
    amplitude: float
    phase: float
    aligned_count_tail: int
    rho_sup_tail: float
    rratio_cofinal_grid: float
    delta_rr_cofinal_grid: float
    delta_tri_cofinal_grid: float
    c_beta_rr_cofinal_grid: float
    c_beta_tri_cofinal_grid: float
    c_beta_obs_cofinal_grid: float
    endpoint_c_sup_window: float
    crossover_x_rr_cofinal: float | None
    crossover_x_tri_cofinal: float | None
    crossover_x_obs_cofinal: float | None


def parse_float_list(text: str) -> List[float]:
    out: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if tok:
            out.append(float(tok))
    if not out:
        raise ValueError("empty float list")
    return out


def _fit_mode(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau: float,
    include_decay_term: bool,
) -> Dict[str, float | np.ndarray]:
    c = np.cos(tau * t)
    s = np.sin(tau * t)
    cols = [c, s]
    if include_decay_term:
        cols.append(np.power(x, -beta))
    xmat = np.column_stack(cols)
    coef, *_ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    r = y - yhat
    a = float(coef[0])
    b = float(coef[1])
    amp = float(math.hypot(a, b))
    phi = float(math.atan2(-b, a))
    return {
        "a": a,
        "b": b,
        "amp": amp,
        "phi": phi,
        "r": r,
        "yhat": yhat,
        "decay_coeff": float(coef[2]) if include_decay_term else None,
    }


def _choose_tau(zeros_file: str, tau: float | None) -> float:
    if tau is not None and tau > 0:
        return float(tau)
    zeros = [float(z) for z in load_zeta_zeros_file(zeros_file) if float(z) > 0.0]
    zeros.sort()
    if not zeros:
        raise ValueError("no positive zeta zeros available")
    return float(zeros[0])


def _upper_gap(x: float, c_beta: float, beta: float, c_endpoint: float) -> float:
    return c_beta * (x ** beta) - c_endpoint * (x ** 0.5) * (math.log(x) ** 2)


def _crossover_x(c_beta: float, beta: float, c_endpoint: float, x_lo: float) -> float | None:
    if c_beta <= 0.0 or beta <= 0.5 or c_endpoint <= 0.0:
        return None
    lo = max(2.0, x_lo)
    if _upper_gap(lo, c_beta, beta, c_endpoint) >= 0.0:
        return float(lo)
    hi = max(lo * 2.0, 1.0e6)
    while hi < 1.0e300:
        if _upper_gap(hi, c_beta, beta, c_endpoint) >= 0.0:
            break
        hi *= 2.0
    else:
        return None
    for _ in range(160):
        mid = math.sqrt(lo * hi)
        if _upper_gap(mid, c_beta, beta, c_endpoint) >= 0.0:
            hi = mid
        else:
            lo = mid
    return float(hi)


def _cofinal_threshold_grid(x_tail_max: float, x_start: float, frac: float, count: int) -> np.ndarray:
    x_end = float(frac) * float(x_tail_max)
    if x_end <= x_start:
        x_end = float(x_tail_max)
    if x_end <= x_start or count <= 0:
        return np.array([], dtype=np.float64)
    return np.exp(np.linspace(math.log(x_start), math.log(x_end), int(count)))


def main() -> None:
    ap = argparse.ArgumentParser(description="Cofinal aligned-subsequence remainder ledger (fixed psi)")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=30_000_000)
    ap.add_argument("--samples", type=int, default=2600)
    ap.add_argument("--betas", type=str, default="0.58,0.60,0.62")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--tau", type=float, default=0.0, help="if <=0 use first zeta zero")
    ap.add_argument("--tail-frac", type=float, default=0.5)
    ap.add_argument("--abs-cos-min", type=float, default=0.98, help="alignment gate a0")
    ap.add_argument("--cofinal-grid-count", type=int, default=40)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_subsequence_remainder_ledger_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    betas = parse_float_list(args.betas)
    tau = _choose_tau(args.zeta_zeros_file, args.tau if args.tau > 0 else None)

    xs = logspace_int(args.xmin, args.xmax, args.samples)
    x = xs.astype(np.float64)
    t = np.log(x)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    abs_e = np.abs(e_vals)

    c_endpoint = float(np.max(abs_e / (np.sqrt(x) * np.power(np.log(x), 2)))
    )

    tail_start = int((1.0 - float(args.tail_frac)) * len(x))
    tail_start = max(1, min(tail_start, len(x) - 2))

    rows: List[BetaSubseqRow] = []
    diagnostics: List[Dict[str, object]] = []
    a0 = float(args.abs_cos_min)

    for beta in betas:
        if beta <= 0.5:
            continue
        y = e_vals / np.power(x, beta)
        fit = _fit_mode(
            x=x,
            t=t,
            y=y,
            beta=float(beta),
            tau=float(tau),
            include_decay_term=bool(args.include_decay_term),
        )
        amp = float(fit["amp"])
        phi = float(fit["phi"])
        r = np.asarray(fit["r"], dtype=np.float64)
        if amp <= 1.0e-18:
            continue

        rr_tail = np.abs(r[tail_start:]) / amp
        rho_sup_tail = float(np.max(rr_tail))

        phase = float(tau) * t + phi
        cos_abs = np.abs(np.cos(phase))
        aligned_mask = np.zeros_like(cos_abs, dtype=bool)
        aligned_mask[tail_start:] = cos_abs[tail_start:] >= a0
        idx = np.nonzero(aligned_mask)[0]
        if idx.size == 0:
            continue

        x_al = x[idx]
        y_abs_al = np.abs(y[idx])
        rr_al = np.abs(r[idx]) / amp
        tri_ratio_al = np.maximum(cos_abs[idx] - rr_al, 0.0)  # triangle/A

        x_grid = _cofinal_threshold_grid(
            x_tail_max=float(np.max(x_al)),
            x_start=float(max(x_al[0], x[tail_start])),
            frac=float(args.cofinal_xmax_frac),
            count=int(args.cofinal_grid_count),
        )
        if x_grid.size == 0:
            continue

        min_rr_future: List[float] = []
        max_tri_future: List[float] = []
        max_obs_future: List[float] = []
        for x0 in x_grid:
            mask_future = x_al >= x0
            if not np.any(mask_future):
                continue
            min_rr_future.append(float(np.min(rr_al[mask_future])))
            max_tri_future.append(float(np.max(tri_ratio_al[mask_future])))
            max_obs_future.append(float(np.max(y_abs_al[mask_future])))

        if not min_rr_future or not max_tri_future or not max_obs_future:
            continue

        rratio_cofinal = float(max(min_rr_future))
        delta_rr = float(a0 - rratio_cofinal)
        delta_tri = float(min(max_tri_future))

        c_beta_rr = float(max(0.0, delta_rr) * amp)
        c_beta_tri = float(max(0.0, delta_tri) * amp)
        c_beta_obs = float(min(max_obs_future))

        row = BetaSubseqRow(
            beta=float(beta),
            tau=float(tau),
            amplitude=float(amp),
            phase=float(phi),
            aligned_count_tail=int(idx.size),
            rho_sup_tail=rho_sup_tail,
            rratio_cofinal_grid=rratio_cofinal,
            delta_rr_cofinal_grid=delta_rr,
            delta_tri_cofinal_grid=delta_tri,
            c_beta_rr_cofinal_grid=c_beta_rr,
            c_beta_tri_cofinal_grid=c_beta_tri,
            c_beta_obs_cofinal_grid=c_beta_obs,
            endpoint_c_sup_window=c_endpoint,
            crossover_x_rr_cofinal=_crossover_x(c_beta_rr, float(beta), c_endpoint, float(args.xmin)),
            crossover_x_tri_cofinal=_crossover_x(c_beta_tri, float(beta), c_endpoint, float(args.xmin)),
            crossover_x_obs_cofinal=_crossover_x(c_beta_obs, float(beta), c_endpoint, float(args.xmin)),
        )
        rows.append(row)

        diagnostics.append(
            {
                "beta": float(beta),
                "a0": a0,
                "amp": amp,
                "phi": phi,
                "aligned_count_tail": int(idx.size),
                "min_rr_future": min_rr_future,
                "max_tri_future": max_tri_future,
                "max_obs_future": max_obs_future,
            }
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "samples": int(len(xs)),
            "betas": betas,
            "tau": float(tau),
            "tail_frac": float(args.tail_frac),
            "abs_cos_min": a0,
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
        },
        "rows": [asdict(r) for r in rows],
        "diagnostics": diagnostics,
        "interpretation": {
            "note": (
                "Finite-grid cofinal witness only. "
                "Theorem closure needs an asymptotic/cofinal proof for these quantities."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Subsequence Remainder Ledger ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Samples: `{len(xs)}`")
    lines.append(f"- Tau fixed: `{tau:.9f}`")
    lines.append(f"- Alignment gate: `a0 = {a0}`")
    lines.append(f"- Cofinal threshold grid count: `{args.cofinal_grid_count}`")
    lines.append("")
    lines.append("## Cofinal Witness Table")
    lines.append("")
    lines.append(
        "| beta | A | rho_sup_tail | rr_cofinal | delta_rr | delta_tri | c_rr | c_tri | c_obs | x_cross(c_rr) | x_cross(c_tri) | x_cross(c_obs) |"
    )
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")

    def fmt_x(v: float | None) -> str:
        return "NA" if v is None else f"{v:.6e}"

    for r in rows:
        lines.append(
            f"| {r.beta:.4f} | {r.amplitude:.6e} | {r.rho_sup_tail:.6f} | "
            f"{r.rratio_cofinal_grid:.6f} | {r.delta_rr_cofinal_grid:.6f} | {r.delta_tri_cofinal_grid:.6f} | "
            f"{r.c_beta_rr_cofinal_grid:.6e} | {r.c_beta_tri_cofinal_grid:.6e} | {r.c_beta_obs_cofinal_grid:.6e} | "
            f"{fmt_x(r.crossover_x_rr_cofinal)} | {fmt_x(r.crossover_x_tri_cofinal)} | {fmt_x(r.crossover_x_obs_cofinal)} |"
        )
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
