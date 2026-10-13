#!/usr/bin/env python3
"""Lower-envelope ledger for fixed canonical error E*(x)=psi(x)-x.

For each beta:
1) Fit y(x)=E*(x)/x^beta by one log-phase mode at tau candidates.
2) Compute tail remainder ratios vs fitted amplitude A.
3) Evaluate phase-aligned peak subsequences (near cos=+/-1) and extract
   empirical lower constants c_beta.
4) Compare against endpoint envelope C*sqrt(x)*log(x)^2 by crossover scale.

This is finite-window empirical evidence, designed to map theorem obligations.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List, Sequence

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from prime_geometry_loop import load_zeta_zeros_file


@dataclass
class BetaLedgerRow:
    beta: float
    tau_best: float
    amplitude: float
    phase: float
    tail_rho_sup: float
    tail_rho_p95: float
    aligned_count_tail: int
    aligned_mean_abs_cos: float
    c_obs_q10_aligned_tail: float
    c_obs_q25_aligned_tail: float
    c_obs_median_aligned_tail: float
    c_triangle_q10_aligned_tail: float
    c_triangle_q25_aligned_tail: float
    c_triangle_median_aligned_tail: float
    triangle_positive_fraction: float
    witness_c_obs_grid: float
    witness_c_triangle_grid: float
    witness_grid_count: int
    witness_xmax_frac: float
    witness_triangle_all_positive: bool
    crossover_x_obs_q10_vs_endpoint: float | None
    crossover_x_triangle_q10_vs_endpoint: float | None
    crossover_x_witness_obs_vs_endpoint: float | None
    crossover_x_witness_triangle_vs_endpoint: float | None


def parse_float_list(text: str) -> List[float]:
    out: List[float] = []
    for raw in text.split(","):
        tok = raw.strip()
        if tok:
            out.append(float(tok))
    if not out:
        raise ValueError("empty float list")
    return out


def _fit_one_mode(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau: float,
    include_decay_term: bool,
) -> Dict[str, np.ndarray | float]:
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
        "yhat": yhat,
        "r": r,
        "decay_coeff": float(coef[2]) if include_decay_term else None,
    }


def _tau_candidates(path: str, count: int) -> List[float]:
    zeros = [float(z) for z in load_zeta_zeros_file(path) if float(z) > 0.0]
    zeros.sort()
    if count <= 0:
        count = 1
    out = zeros[:count]
    if not out:
        raise ValueError("no tau candidates from zeros file")
    return out


def _aligned_tail_indices(
    t: np.ndarray,
    tau: float,
    phi: float,
    tail_start: int,
    abs_cos_min: float,
) -> np.ndarray:
    tt = t[tail_start:]
    phase = tau * tt + phi
    mask = np.abs(np.cos(phase)) >= abs_cos_min
    idx_local = np.nonzero(mask)[0]
    if idx_local.size == 0:
        return np.array([], dtype=np.int64)
    return idx_local + tail_start


def _quantiles(vals: np.ndarray) -> List[float]:
    if vals.size == 0:
        return [0.0, 0.0, 0.0]
    return [
        float(np.quantile(vals, 0.10)),
        float(np.quantile(vals, 0.25)),
        float(np.quantile(vals, 0.50)),
    ]


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


def _grid_witness_constant(
    x_aligned: np.ndarray,
    v_aligned: np.ndarray,
    x_start: float,
    x_end: float,
    grid_count: int,
) -> float:
    if x_aligned.size == 0 or v_aligned.size == 0:
        return 0.0
    if grid_count <= 0 or x_end <= x_start:
        return 0.0
    x_grid = np.exp(np.linspace(math.log(x_start), math.log(x_end), int(grid_count)))
    vals: List[float] = []
    for x0 in x_grid:
        mask = x_aligned >= x0
        if np.any(mask):
            vals.append(float(np.max(v_aligned[mask])))
        else:
            vals.append(0.0)
    return float(min(vals)) if vals else 0.0


def main() -> None:
    ap = argparse.ArgumentParser(description="Lower-envelope ledger for fixed psi(x)-x")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=30_000_000)
    ap.add_argument("--samples", type=int, default=2600)
    ap.add_argument("--betas", type=str, default="0.58,0.60,0.62")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--tau-count", type=int, default=12)
    ap.add_argument("--tail-frac", type=float, default=0.5)
    ap.add_argument("--abs-cos-min", type=float, default=0.98)
    ap.add_argument("--witness-grid-count", type=int, default=40)
    ap.add_argument("--witness-xmax-frac", type=float, default=0.95)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_lower_envelope_ledger_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    betas = parse_float_list(args.betas)
    taus = _tau_candidates(args.zeta_zeros_file, args.tau_count)

    xs = logspace_int(args.xmin, args.xmax, args.samples)
    x = xs.astype(np.float64)
    t = np.log(x)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    abs_e = np.abs(e_vals)

    endpoint_ratio = abs_e / (np.sqrt(x) * np.power(np.log(x), 2))
    c_endpoint = float(np.max(endpoint_ratio))
    c_endpoint_p95 = float(np.quantile(endpoint_ratio, 0.95))

    tail_start = int((1.0 - float(args.tail_frac)) * len(x))
    tail_start = max(1, min(tail_start, len(x) - 2))

    rows: List[BetaLedgerRow] = []
    diagnostics: List[Dict[str, object]] = []

    for beta in betas:
        if beta <= 0.5:
            continue
        y = e_vals / np.power(x, beta)

        best = None
        tau_rows: List[Dict[str, float]] = []
        for tau in taus:
            fit = _fit_one_mode(
                x=x,
                t=t,
                y=y,
                beta=beta,
                tau=tau,
                include_decay_term=bool(args.include_decay_term),
            )
            amp = float(fit["amp"])
            r = np.asarray(fit["r"], dtype=np.float64)
            if amp <= 1.0e-18:
                continue

            rr = np.abs(r[tail_start:]) / amp
            rho_sup = float(np.max(rr))
            rho_p95 = float(np.quantile(rr, 0.95))

            idx = _aligned_tail_indices(
                t=t,
                tau=float(tau),
                phi=float(fit["phi"]),
                tail_start=tail_start,
                abs_cos_min=float(args.abs_cos_min),
            )
            if idx.size > 0:
                cos_abs = np.abs(np.cos(float(tau) * t[idx] + float(fit["phi"])))
                y_abs = np.abs(y[idx])
                tri = amp * cos_abs - np.abs(r[idx])
                tri = np.maximum(tri, 0.0)
                q_obs_10, q_obs_25, q_obs_50 = _quantiles(y_abs)
                q_tri_10, q_tri_25, q_tri_50 = _quantiles(tri)
                tri_pos_frac = float(np.mean(tri > 0.0))
                mean_cos = float(np.mean(cos_abs))
                x_aligned = x[idx]
                witness_x_start = float(max(x_aligned[0], x[tail_start]))
                witness_x_end = float(args.witness_xmax_frac) * float(np.max(x_aligned))
                w_obs = _grid_witness_constant(
                    x_aligned=x_aligned,
                    v_aligned=y_abs,
                    x_start=witness_x_start,
                    x_end=witness_x_end,
                    grid_count=int(args.witness_grid_count),
                )
                w_tri = _grid_witness_constant(
                    x_aligned=x_aligned,
                    v_aligned=tri,
                    x_start=witness_x_start,
                    x_end=witness_x_end,
                    grid_count=int(args.witness_grid_count),
                )
                w_tri_all_positive = bool(w_tri > 0.0)
            else:
                q_obs_10 = q_obs_25 = q_obs_50 = 0.0
                q_tri_10 = q_tri_25 = q_tri_50 = 0.0
                tri_pos_frac = 0.0
                mean_cos = 0.0
                w_obs = 0.0
                w_tri = 0.0
                w_tri_all_positive = False

            # score prefers small rho_sup and larger conservative triangle q10.
            score = rho_sup - 0.25 * q_tri_10
            row = {
                "tau": float(tau),
                "amp": amp,
                "phi": float(fit["phi"]),
                "rho_sup": rho_sup,
                "rho_p95": rho_p95,
                "aligned_count": int(idx.size),
                "aligned_mean_abs_cos": mean_cos,
                "q_obs_10": q_obs_10,
                "q_obs_25": q_obs_25,
                "q_obs_50": q_obs_50,
                "q_tri_10": q_tri_10,
                "q_tri_25": q_tri_25,
                "q_tri_50": q_tri_50,
                "tri_pos_frac": tri_pos_frac,
                "witness_c_obs_grid": float(w_obs),
                "witness_c_triangle_grid": float(w_tri),
                "witness_triangle_all_positive": bool(w_tri_all_positive),
                "score": float(score),
            }
            tau_rows.append(row)
            if best is None or row["score"] < best["score"]:
                best = row

        if not best:
            continue

        c_obs_10 = float(best["q_obs_10"])
        c_tri_10 = float(best["q_tri_10"])
        rows.append(
            BetaLedgerRow(
                beta=float(beta),
                tau_best=float(best["tau"]),
                amplitude=float(best["amp"]),
                phase=float(best["phi"]),
                tail_rho_sup=float(best["rho_sup"]),
                tail_rho_p95=float(best["rho_p95"]),
                aligned_count_tail=int(best["aligned_count"]),
                aligned_mean_abs_cos=float(best["aligned_mean_abs_cos"]),
                c_obs_q10_aligned_tail=c_obs_10,
                c_obs_q25_aligned_tail=float(best["q_obs_25"]),
                c_obs_median_aligned_tail=float(best["q_obs_50"]),
                c_triangle_q10_aligned_tail=c_tri_10,
                c_triangle_q25_aligned_tail=float(best["q_tri_25"]),
                c_triangle_median_aligned_tail=float(best["q_tri_50"]),
                triangle_positive_fraction=float(best["tri_pos_frac"]),
                witness_c_obs_grid=float(best["witness_c_obs_grid"]),
                witness_c_triangle_grid=float(best["witness_c_triangle_grid"]),
                witness_grid_count=int(args.witness_grid_count),
                witness_xmax_frac=float(args.witness_xmax_frac),
                witness_triangle_all_positive=bool(best["witness_triangle_all_positive"]),
                crossover_x_obs_q10_vs_endpoint=_crossover_x(
                    c_beta=c_obs_10, beta=float(beta), c_endpoint=c_endpoint, x_lo=float(args.xmin)
                ),
                crossover_x_triangle_q10_vs_endpoint=_crossover_x(
                    c_beta=c_tri_10, beta=float(beta), c_endpoint=c_endpoint, x_lo=float(args.xmin)
                ),
                crossover_x_witness_obs_vs_endpoint=_crossover_x(
                    c_beta=float(best["witness_c_obs_grid"]),
                    beta=float(beta),
                    c_endpoint=c_endpoint,
                    x_lo=float(args.xmin),
                ),
                crossover_x_witness_triangle_vs_endpoint=_crossover_x(
                    c_beta=float(best["witness_c_triangle_grid"]),
                    beta=float(beta),
                    c_endpoint=c_endpoint,
                    x_lo=float(args.xmin),
                ),
            )
        )
        diagnostics.append(
            {
                "beta": float(beta),
                "best": best,
                "tau_rows": tau_rows,
            }
        )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "samples": int(len(xs)),
            "betas": betas,
            "tau_count": int(args.tau_count),
            "tail_frac": float(args.tail_frac),
            "abs_cos_min": float(args.abs_cos_min),
            "witness_grid_count": int(args.witness_grid_count),
            "witness_xmax_frac": float(args.witness_xmax_frac),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
            "cache_dir": str(args.cache_dir),
        },
        "endpoint_upper": {
            "form": "|E*(x)| <= C_endpoint * x^(1/2) * (log x)^2",
            "c_endpoint_sup_window": c_endpoint,
            "c_endpoint_p95_window": c_endpoint_p95,
        },
        "rows": [asdict(r) for r in rows],
        "diagnostics": diagnostics,
        "interpretation": {
            "note": (
                "Finite-window lower-envelope ledger only. "
                "Theorem closure still requires asymptotic quantifiers and theorem-grade "
                "main-term/remainder constants."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Lower-Envelope Ledger ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Samples: `{len(xs)}`")
    lines.append(f"- Prime-power events used: `{len(events)}`")
    lines.append(f"- Tail fraction: `{args.tail_frac}`")
    lines.append(f"- Alignment threshold: `|cos| >= {args.abs_cos_min}`")
    lines.append("")
    lines.append("## Endpoint Upper Envelope (Empirical Window)")
    lines.append("")
    lines.append(f"- `C_endpoint_sup_window = {c_endpoint:.9e}`")
    lines.append(f"- `C_endpoint_p95_window = {c_endpoint_p95:.9e}`")
    lines.append("")
    lines.append("## Beta Ledger")
    lines.append("")
    lines.append(
        "| beta | tau_best | A | rho_sup | aligned_n | c_obs_q10 | c_tri_q10 | w_obs_grid | w_tri_grid | tri_pos_frac | x_cross(obs_q10) | x_cross(tri_q10) | x_cross(w_obs) | x_cross(w_tri) |"
    )
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")

    def fmt_x(v: float | None) -> str:
        return "NA" if v is None else f"{v:.6e}"

    for r in rows:
        lines.append(
                f"| {r.beta:.4f} | {r.tau_best:.6f} | {r.amplitude:.6e} | "
                f"{r.tail_rho_sup:.6f} | {r.aligned_count_tail} | "
                f"{r.c_obs_q10_aligned_tail:.6e} | {r.c_triangle_q10_aligned_tail:.6e} | "
                f"{r.witness_c_obs_grid:.6e} | {r.witness_c_triangle_grid:.6e} | "
                f"{r.triangle_positive_fraction:.4f} | "
                f"{fmt_x(r.crossover_x_obs_q10_vs_endpoint)} | "
                f"{fmt_x(r.crossover_x_triangle_q10_vs_endpoint)} | "
                f"{fmt_x(r.crossover_x_witness_obs_vs_endpoint)} | "
                f"{fmt_x(r.crossover_x_witness_triangle_vs_endpoint)} |"
        )

    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
