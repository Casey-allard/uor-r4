#!/usr/bin/env python3
"""Aligned-sequence tail-split bound candidates for mode admissibility.

For fixed beta and tau candidates:
1) fit one-mode model on y(x)=E*(x)/x^beta, E*=psi-x;
2) on aligned points (|cos|>=a0), build sequence rr=|R|/A;
3) define q0 from last block of rr and fit bound:
     rr(x) <= q0 + B*x^(-eta)
   with eta selected from a grid and B chosen to cover all sampled aligned points;
4) derive threshold X(a0) from q0 + B*X^(-eta) <= a0.

Finite-window evidence only.
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
class TauSplitRow:
    tau: float
    amplitude: float
    phase: float
    aligned_count_tail: int
    q0_tail_block: float
    q0_policy: str
    eta_star: float
    b_star: float
    x_a0_threshold: float | None
    max_rr_aligned: float
    rr_cofinal_last: float
    rr_cofinal_first: float
    max_rr_cofinal: float
    cover_all_aligned: bool
    cover_slack_min: float
    cover_slack_p05: float
    cover_slack_p50: float
    q0_below_a0: bool


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
    return {"amp": amp, "phi": phi, "r": r}


def _x_threshold_for_a0(q0: float, b: float, eta: float, a0: float) -> float | None:
    if eta <= 0.0:
        return None
    if q0 >= a0:
        return None
    rem = a0 - q0
    if rem <= 0.0:
        return None
    if b <= 0.0:
        return 1.0
    return float((b / rem) ** (1.0 / eta))


def _cofinal_curve(
    x_al: np.ndarray,
    rr_al: np.ndarray,
    grid_count: int,
    xmax_frac: float,
) -> Dict[str, np.ndarray]:
    if x_al.size == 0:
        return {"xg": np.array([], dtype=np.float64), "qg": np.array([], dtype=np.float64)}
    x_start = float(x_al[0])
    x_end = float(xmax_frac) * float(np.max(x_al))
    if x_end <= x_start:
        x_end = float(np.max(x_al))
    if x_end <= x_start:
        return {"xg": np.array([], dtype=np.float64), "qg": np.array([], dtype=np.float64)}
    xg = np.exp(np.linspace(math.log(x_start), math.log(x_end), int(max(1, grid_count))))
    qg: List[float] = []
    xk: List[float] = []
    for x0 in xg:
        m = x_al >= x0
        if np.any(m):
            xk.append(float(x0))
            qg.append(float(np.min(rr_al[m])))
    return {
        "xg": np.array(xk, dtype=np.float64),
        "qg": np.array(qg, dtype=np.float64),
    }


def _select_q0(q_tail: np.ndarray, policy: str) -> float:
    if q_tail.size == 0:
        return 0.0
    if policy == "tail_min":
        return float(np.min(q_tail))
    if policy == "tail_median":
        return float(np.median(q_tail))
    if policy == "tail_p25":
        return float(np.quantile(q_tail, 0.25))
    if policy == "tail_max":
        return float(np.max(q_tail))
    raise ValueError(f"unknown q0 policy: {policy}")


def _select_best_eta(
    x_curve: np.ndarray,
    rr_curve: np.ndarray,
    q0: float,
    a0: float,
    eta_grid: List[float],
) -> Dict[str, float]:
    best = {
        "eta": 0.0,
        "b": 0.0,
        "x_a0": float("inf"),
        "slack_min": float("-inf"),
        "slack_p05": float("-inf"),
        "slack_p50": float("-inf"),
    }
    for eta in eta_grid:
        if eta <= 0.0:
            continue
        residual = np.maximum(rr_curve - q0, 0.0)
        b = float(np.max(residual * np.power(x_curve, eta)))
        rr_hat = q0 + b * np.power(x_curve, -eta)
        slack = rr_hat - rr_curve
        x_a0 = _x_threshold_for_a0(q0=q0, b=b, eta=eta, a0=a0)
        x_obj = float("inf") if x_a0 is None else float(x_a0)
        if x_obj < best["x_a0"]:
            best = {
                "eta": float(eta),
                "b": b,
                "x_a0": x_obj,
                "slack_min": float(np.min(slack)),
                "slack_p05": float(np.quantile(slack, 0.05)),
                "slack_p50": float(np.quantile(slack, 0.50)),
            }
    return best


def main() -> None:
    ap = argparse.ArgumentParser(description="Aligned tail-split bound candidates")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=30_000_000)
    ap.add_argument("--samples", type=int, default=2600)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json",
    )
    ap.add_argument("--tau-count", type=int, default=12)
    ap.add_argument("--tail-frac", type=float, default=0.5)
    ap.add_argument("--abs-cos-min", type=float, default=0.98)
    ap.add_argument("--q0-tail-block", type=int, default=24)
    ap.add_argument("--q0-policy", type=str, default="tail_min", choices=["tail_min", "tail_median", "tail_p25", "tail_max"])
    ap.add_argument("--cofinal-grid-count", type=int, default=40)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--eta-grid", type=str, default="0.02,0.03,0.04,0.05,0.06,0.08,0.10,0.12,0.15,0.20,0.25,0.30")
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_aligned_tail_split_bound_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    eta_grid = parse_float_list(args.eta_grid)
    a0 = float(args.abs_cos_min)

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file) if float(z) > 0.0]
    zeros.sort()
    taus = zeros[: max(1, int(args.tau_count))]
    if not taus:
        raise ValueError("no tau candidates")

    xs = logspace_int(args.xmin, args.xmax, args.samples)
    x = xs.astype(np.float64)
    t = np.log(x)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))

    tail_start = int((1.0 - float(args.tail_frac)) * len(x))
    tail_start = max(1, min(tail_start, len(x) - 2))

    rows: List[TauSplitRow] = []
    diagnostics: List[Dict[str, object]] = []

    for tau in taus:
        fit = _fit_mode(
            x=x,
            t=t,
            y=y,
            beta=float(args.beta),
            tau=float(tau),
            include_decay_term=bool(args.include_decay_term),
        )
        amp = float(fit["amp"])
        if amp <= 1.0e-18:
            continue
        phi = float(fit["phi"])
        r = np.asarray(fit["r"], dtype=np.float64)

        phase = float(tau) * t + phi
        cos_abs = np.abs(np.cos(phase))
        aligned = np.zeros_like(cos_abs, dtype=bool)
        aligned[tail_start:] = cos_abs[tail_start:] >= a0
        idx = np.nonzero(aligned)[0]
        if idx.size < 10:
            continue

        x_al = x[idx]
        rr_al = np.abs(r[idx]) / amp
        cf = _cofinal_curve(
            x_al=x_al,
            rr_al=rr_al,
            grid_count=int(args.cofinal_grid_count),
            xmax_frac=float(args.cofinal_xmax_frac),
        )
        xg = cf["xg"]
        qg = cf["qg"]
        if xg.size < 8:
            continue

        block = int(max(4, args.q0_tail_block))
        block = min(block, qg.size)
        q_tail = qg[-block:]
        q0 = _select_q0(q_tail=q_tail, policy=str(args.q0_policy))

        best = _select_best_eta(
            x_curve=xg,
            rr_curve=qg,
            q0=q0,
            a0=a0,
            eta_grid=eta_grid,
        )
        eta_star = float(best["eta"])
        b_star = float(best["b"])
        x_a0 = None if not math.isfinite(best["x_a0"]) else float(best["x_a0"])

        rr_hat = q0 + b_star * np.power(xg, -eta_star) if eta_star > 0 else np.full_like(qg, q0)
        slack = rr_hat - qg
        cover = bool(np.min(slack) >= -1.0e-12)

        rows.append(
            TauSplitRow(
                tau=float(tau),
                amplitude=amp,
                phase=phi,
                aligned_count_tail=int(idx.size),
                q0_tail_block=q0,
                q0_policy=str(args.q0_policy),
                eta_star=eta_star,
                b_star=b_star,
                x_a0_threshold=x_a0,
                max_rr_aligned=float(np.max(rr_al)),
                rr_cofinal_last=float(qg[-1]),
                rr_cofinal_first=float(qg[0]),
                max_rr_cofinal=float(np.max(qg)),
                cover_all_aligned=cover,
                cover_slack_min=float(np.min(slack)),
                cover_slack_p05=float(np.quantile(slack, 0.05)),
                cover_slack_p50=float(np.quantile(slack, 0.50)),
                q0_below_a0=bool(q0 < a0),
            )
        )
        diagnostics.append(
            {
                "tau": float(tau),
                "x_aligned": [float(v) for v in x_al],
                "rr_aligned": [float(v) for v in rr_al],
                "x_cofinal_grid": [float(v) for v in xg],
                "rr_cofinal_curve": [float(v) for v in qg],
                "q0_tail_block": q0,
                "q0_policy": str(args.q0_policy),
                "eta_star": eta_star,
                "b_star": b_star,
            }
        )

    rows.sort(
        key=lambda r: (
            not (r.q0_below_a0 and r.cover_all_aligned and r.x_a0_threshold is not None),
            float("inf") if r.x_a0_threshold is None else r.x_a0_threshold,
        )
    )

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "samples": int(len(xs)),
            "beta": float(args.beta),
            "tau_count": int(args.tau_count),
            "tail_frac": float(args.tail_frac),
            "abs_cos_min": a0,
            "q0_tail_block": int(args.q0_tail_block),
            "q0_policy": str(args.q0_policy),
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "eta_grid": eta_grid,
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
        },
        "rows": [asdict(r) for r in rows],
        "diagnostics": diagnostics,
        "interpretation": {
            "note": (
                "Finite-window aligned-sequence bound only. "
                "Useful as a candidate theorem template: rr <= q0 + B*x^-eta with q0<a0."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Aligned Tail-Split Bound ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Alignment gate: `a0={a0}`")
    lines.append(f"- q0 tail block size: `{args.q0_tail_block}`")
    lines.append(f"- q0 policy: `{args.q0_policy}`")
    lines.append(f"- cofinal grid count: `{args.cofinal_grid_count}`")
    lines.append(f"- cofinal xmax frac: `{args.cofinal_xmax_frac}`")
    lines.append("")
    lines.append("## Tau Candidate Table")
    lines.append("")
    lines.append("| tau | aligned_n | q0 | eta* | B* | X(a0) | q0<a0 | cover_all | slack_min | rr_cf(last) | rr_cf(first) | rr_max(raw) |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")

    def fmt_x(v: float | None) -> str:
        return "NA" if v is None else f"{v:.6e}"

    for r in rows:
        lines.append(
                f"| {r.tau:.6f} | {r.aligned_count_tail} | {r.q0_tail_block:.6f} | {r.eta_star:.6f} | "
                f"{r.b_star:.6e} | {fmt_x(r.x_a0_threshold)} | {str(r.q0_below_a0).lower()} | "
                f"{str(r.cover_all_aligned).lower()} | {r.cover_slack_min:.6e} | {r.rr_cofinal_last:.6f} | "
                f"{r.rr_cofinal_first:.6f} | {r.max_rr_aligned:.6f} |"
        )
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
