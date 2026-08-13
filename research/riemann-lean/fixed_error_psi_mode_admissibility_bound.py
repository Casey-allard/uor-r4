#!/usr/bin/env python3
"""Analytic candidate bounds for aligned-sequence mode admissibility.

For fixed beta and tau candidates:
1) fit one-mode decomposition on y(x)=E*(x)/x^beta, E*=psi-x;
2) compute residual envelope model |R(x)| <= C*x^(-eta) (finite-window fit);
3) convert to ratio bound |R|/A <= D*x^(-eta), D=C/A;
4) derive threshold formulas X(q) from D*X^(-eta)=q.

This gives theorem-facing candidate constants for the mode-admissibility route.
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
class TauBoundRow:
    tau: float
    amplitude: float
    phase: float
    eta_hat: float
    c_hat: float
    d_ratio: float
    rho_sup_tail: float
    rr_cofinal_grid: float
    aligned_count_tail: int
    q_bound_at_tail_start: float
    q_bound_at_xmax: float
    x_for_q_0_98: float | None
    x_for_q_0_80: float | None
    x_for_q_0_60: float | None
    x_for_q_0_40: float | None


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


def _tail_cum_sup_abs(arr: np.ndarray) -> np.ndarray:
    out = np.empty_like(arr)
    best = -1.0
    for i in range(arr.size - 1, -1, -1):
        v = float(abs(arr[i]))
        if v > best:
            best = v
        out[i] = best
    return out


def _slope_loglog(x: np.ndarray, y: np.ndarray) -> float:
    lx = np.log(np.maximum(x, 1.0e-300))
    ly = np.log(np.maximum(y, 1.0e-300))
    mx = float(np.mean(lx))
    my = float(np.mean(ly))
    num = float(np.sum((lx - mx) * (ly - my)))
    den = float(np.sum((lx - mx) ** 2))
    if den <= 1.0e-30:
        return 0.0
    return num / den


def _eta_c_fit(x: np.ndarray, r: np.ndarray, tail_start: int) -> Dict[str, float]:
    env = _tail_cum_sup_abs(r)
    xt = x[tail_start:]
    yt = env[tail_start:]
    slope = _slope_loglog(xt, yt)
    eta = max(0.0, -slope)
    c_hat = float(np.max(np.abs(r) * np.power(x, eta)))
    return {"eta": float(eta), "c_hat": c_hat}


def _cofinal_rr(
    x: np.ndarray,
    t: np.ndarray,
    r: np.ndarray,
    amp: float,
    tau: float,
    phi: float,
    tail_start: int,
    a0: float,
    grid_count: int,
    xmax_frac: float,
) -> Dict[str, float]:
    if amp <= 1.0e-18:
        return {"rr_cofinal_grid": float("inf"), "aligned_count_tail": 0.0}
    phase = tau * t + phi
    cos_abs = np.abs(np.cos(phase))
    aligned_mask = np.zeros_like(cos_abs, dtype=bool)
    aligned_mask[tail_start:] = cos_abs[tail_start:] >= a0
    idx = np.nonzero(aligned_mask)[0]
    if idx.size == 0:
        return {"rr_cofinal_grid": float("inf"), "aligned_count_tail": 0.0}
    x_al = x[idx]
    rr_al = np.abs(r[idx]) / amp
    x_start = float(max(x_al[0], x[tail_start]))
    x_end = float(xmax_frac) * float(np.max(x_al))
    if x_end <= x_start:
        x_end = float(np.max(x_al))
    xg = np.exp(np.linspace(math.log(x_start), math.log(x_end), int(max(1, grid_count))))
    mins: List[float] = []
    for x0 in xg:
        m = x_al >= x0
        if np.any(m):
            mins.append(float(np.min(rr_al[m])))
    if not mins:
        return {"rr_cofinal_grid": float("inf"), "aligned_count_tail": float(idx.size)}
    return {
        "rr_cofinal_grid": float(max(mins)),
        "aligned_count_tail": float(idx.size),
    }


def _x_for_q(d_ratio: float, eta: float, q: float) -> float | None:
    if d_ratio <= 0.0 or eta <= 0.0 or q <= 0.0:
        return None
    if d_ratio <= q:
        return 1.0
    return float((d_ratio / q) ** (1.0 / eta))


def main() -> None:
    ap = argparse.ArgumentParser(description="Mode-admissibility bound candidates (fixed psi)")
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
    ap.add_argument("--cofinal-grid-count", type=int, default=40)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_mode_admissibility_bound_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")

    zeros = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file) if float(z) > 0.0]
    zeros.sort()
    if not zeros:
        raise ValueError("no positive zeta zeros loaded")
    taus = zeros[: max(1, int(args.tau_count))]

    xs = logspace_int(args.xmin, args.xmax, args.samples)
    x = xs.astype(np.float64)
    t = np.log(x)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))

    tail_start = int((1.0 - float(args.tail_frac)) * len(x))
    tail_start = max(1, min(tail_start, len(x) - 2))
    a0 = float(args.abs_cos_min)

    rows: List[TauBoundRow] = []
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
        rr_tail = np.abs(r[tail_start:]) / amp
        rho_sup_tail = float(np.max(rr_tail))

        ec = _eta_c_fit(x=x, r=r, tail_start=tail_start)
        eta = float(ec["eta"])
        c_hat = float(ec["c_hat"])
        d_ratio = c_hat / amp if amp > 0.0 else float("inf")
        q_tail = d_ratio * (float(x[tail_start]) ** (-eta)) if eta > 0.0 else float("inf")
        q_xmax = d_ratio * (float(x[-1]) ** (-eta)) if eta > 0.0 else float("inf")

        cof = _cofinal_rr(
            x=x,
            t=t,
            r=r,
            amp=amp,
            tau=float(tau),
            phi=phi,
            tail_start=tail_start,
            a0=a0,
            grid_count=int(args.cofinal_grid_count),
            xmax_frac=float(args.cofinal_xmax_frac),
        )

        rows.append(
            TauBoundRow(
                tau=float(tau),
                amplitude=amp,
                phase=phi,
                eta_hat=eta,
                c_hat=c_hat,
                d_ratio=float(d_ratio),
                rho_sup_tail=rho_sup_tail,
                rr_cofinal_grid=float(cof["rr_cofinal_grid"]),
                aligned_count_tail=int(cof["aligned_count_tail"]),
                q_bound_at_tail_start=float(q_tail),
                q_bound_at_xmax=float(q_xmax),
                x_for_q_0_98=_x_for_q(d_ratio, eta, 0.98),
                x_for_q_0_80=_x_for_q(d_ratio, eta, 0.80),
                x_for_q_0_60=_x_for_q(d_ratio, eta, 0.60),
                x_for_q_0_40=_x_for_q(d_ratio, eta, 0.40),
            )
        )

    rows.sort(key=lambda r: (r.x_for_q_0_98 is None, r.x_for_q_0_98 if r.x_for_q_0_98 is not None else float("inf")))

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
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window envelope model only. "
                "eta_hat and c_hat are empirical candidates for theorem-style remainder bounds."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Mode-Admissibility Bound Candidates ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Alignment gate target: `a0={a0}`")
    lines.append("")
    lines.append("## Tau Bound Table")
    lines.append("")
    lines.append(
        "| tau | A | eta_hat | D=C/A | rr_cofinal | q_bound(x_tail) | q_bound(xmax) | X(0.98) | X(0.80) | X(0.60) | X(0.40) |"
    )
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")

    def fmt_x(v: float | None) -> str:
        return "NA" if v is None else f"{v:.6e}"

    for r in rows:
        lines.append(
            f"| {r.tau:.6f} | {r.amplitude:.6e} | {r.eta_hat:.6f} | {r.d_ratio:.6e} | {r.rr_cofinal_grid:.6f} | "
            f"{r.q_bound_at_tail_start:.6f} | {r.q_bound_at_xmax:.6f} | "
            f"{fmt_x(r.x_for_q_0_98)} | {fmt_x(r.x_for_q_0_80)} | {fmt_x(r.x_for_q_0_60)} | {fmt_x(r.x_for_q_0_40)} |"
        )
    lines.append("")
    lines.append(payload["interpretation"]["note"])
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
