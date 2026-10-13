#!/usr/bin/env python3
"""Structured aligned-lattice probe for mode admissibility on E*(x)=psi(x)-x.

Key difference vs logspace-alignment scans:
  - Uses phase-locked candidate points x_k ~ exp((2*pi*k - phi)/tau), so the
    alignment set is generated analytically from (tau, phi) rather than
    discovered from sparse sample-grid coincidences.
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
class LatticeRow:
    tau: float
    amplitude: float
    phase: float
    fit_samples: int
    lattice_points_total: int
    lattice_points_used: int
    cos_abs_min_used: float
    cos_abs_median_used: float
    rr_max_used: float
    rr_cofinal_grid: float
    delta_rr_cofinal_grid: float
    delta_tri_cofinal_grid: float
    c_beta_tri_cofinal_grid: float
    c_beta_obs_cofinal_grid: float


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


def _cofinal_grid(x_start: float, x_end: float, count: int) -> np.ndarray:
    if x_end <= x_start or count <= 0:
        return np.array([], dtype=np.float64)
    return np.exp(np.linspace(math.log(x_start), math.log(x_end), int(count)))


def _build_lattice_points(
    tau: float,
    phi: float,
    xmin: int,
    xmax: int,
    local_window: int,
) -> np.ndarray:
    # Solve tau*log(x) + phi ~= 2*pi*k  =>  x ~ exp((2*pi*k - phi)/tau).
    two_pi = 2.0 * math.pi
    k_lo = int(math.ceil((tau * math.log(float(xmin)) + phi) / two_pi))
    k_hi = int(math.floor((tau * math.log(float(xmax)) + phi) / two_pi))
    if k_hi < k_lo:
        return np.array([], dtype=np.int64)

    xs: List[int] = []
    for k in range(k_lo, k_hi + 1):
        x0 = math.exp((two_pi * float(k) - phi) / tau)
        n0 = int(round(x0))
        if n0 < xmin or n0 > xmax:
            continue
        best_n = n0
        best_c = abs(math.cos(tau * math.log(float(max(2, n0))) + phi))
        if local_window > 0:
            lo = max(xmin, n0 - local_window)
            hi = min(xmax, n0 + local_window)
            for n in range(lo, hi + 1):
                c = abs(math.cos(tau * math.log(float(max(2, n))) + phi))
                if c > best_c:
                    best_c = c
                    best_n = n
        xs.append(best_n)
    if not xs:
        return np.array([], dtype=np.int64)
    arr = np.array(sorted(set(xs)), dtype=np.int64)
    return arr[(arr >= xmin) & (arr <= xmax)]


def main() -> None:
    ap = argparse.ArgumentParser(description="Structured aligned-lattice mode probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--taus", type=str, required=True, help="Comma-separated tau values")
    ap.add_argument("--abs-cos-min", type=float, default=0.98)
    ap.add_argument("--local-window", type=int, default=4)
    ap.add_argument("--cofinal-grid-count", type=int, default=80)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_aligned_lattice_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.xmin < 2 or args.xmax <= args.xmin:
        raise ValueError("invalid [xmin, xmax]")

    taus = parse_float_list(args.taus)
    xs_fit = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x_fit = xs_fit.astype(np.float64)
    t_fit = np.log(x_fit)

    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_fit = psi_at_samples(xs_fit, events)
    e_fit = psi_fit - x_fit
    y_fit = e_fit / np.power(x_fit, float(args.beta))

    a0 = float(args.abs_cos_min)
    rows: List[LatticeRow] = []
    for tau in taus:
        fit = _fit_mode(
            x=x_fit,
            t=t_fit,
            y=y_fit,
            beta=float(args.beta),
            tau=float(tau),
            include_decay_term=bool(args.include_decay_term),
        )
        amp = float(fit["amp"])
        phi = float(fit["phi"])
        if amp <= 1.0e-18:
            continue

        x_lattice = _build_lattice_points(
            tau=float(tau),
            phi=phi,
            xmin=int(args.xmin),
            xmax=int(args.xmax),
            local_window=int(args.local_window),
        )
        if x_lattice.size < 8:
            continue

        psi_lat = psi_at_samples(x_lattice, events)
        x_lat = x_lattice.astype(np.float64)
        y_lat = (psi_lat - x_lat) / np.power(x_lat, float(args.beta))
        phase_lat = float(tau) * np.log(x_lat) + phi
        cos_lat = np.cos(phase_lat)
        cos_abs_lat = np.abs(cos_lat)
        r_lat = y_lat - amp * cos_lat
        rr_lat = np.abs(r_lat) / amp
        tri_lat = np.maximum(cos_abs_lat - rr_lat, 0.0)

        use = cos_abs_lat >= a0
        if np.count_nonzero(use) < 8:
            continue
        x_use = x_lat[use]
        rr_use = rr_lat[use]
        tri_use = tri_lat[use]
        yabs_use = np.abs(y_lat[use])
        cosabs_use = cos_abs_lat[use]

        x_end = float(args.cofinal_xmax_frac) * float(np.max(x_use))
        if x_end <= x_use[0]:
            x_end = float(np.max(x_use))
        xg = _cofinal_grid(float(x_use[0]), x_end, int(args.cofinal_grid_count))
        if xg.size == 0:
            continue

        min_rr_future: List[float] = []
        max_tri_future: List[float] = []
        max_obs_future: List[float] = []
        for x0 in xg:
            m = x_use >= x0
            if not np.any(m):
                continue
            min_rr_future.append(float(np.min(rr_use[m])))
            max_tri_future.append(float(np.max(tri_use[m])))
            max_obs_future.append(float(np.max(yabs_use[m])))
        if not min_rr_future:
            continue

        rr_cofinal = float(max(min_rr_future))
        delta_rr = float(a0 - rr_cofinal)
        delta_tri = float(min(max_tri_future))
        c_tri = float(max(0.0, delta_tri) * amp)
        c_obs = float(min(max_obs_future))

        rows.append(
            LatticeRow(
                tau=float(tau),
                amplitude=amp,
                phase=phi,
                fit_samples=int(len(xs_fit)),
                lattice_points_total=int(len(x_lattice)),
                lattice_points_used=int(np.count_nonzero(use)),
                cos_abs_min_used=float(np.min(cosabs_use)),
                cos_abs_median_used=float(np.median(cosabs_use)),
                rr_max_used=float(np.max(rr_use)),
                rr_cofinal_grid=rr_cofinal,
                delta_rr_cofinal_grid=delta_rr,
                delta_tri_cofinal_grid=delta_tri,
                c_beta_tri_cofinal_grid=c_tri,
                c_beta_obs_cofinal_grid=c_obs,
            )
        )

    rows.sort(key=lambda r: (-r.delta_tri_cofinal_grid, r.rr_cofinal_grid))

    payload = {
        "meta": {
            "date": date.today().isoformat(),
            "xmin": int(args.xmin),
            "xmax": int(args.xmax),
            "fit_samples": int(len(xs_fit)),
            "beta": float(args.beta),
            "taus": taus,
            "abs_cos_min": a0,
            "local_window": int(args.local_window),
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window structured-lattice evidence only. "
                "Reduces alignment aliasing relative to sparse logspace gating."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Fixed Error psi(x)-x Aligned Lattice Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Fit samples: `{len(xs_fit)}`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- Alignment gate: `a0={a0}`")
    lines.append(f"- Local integer search window: `±{args.local_window}`")
    lines.append("")
    lines.append("| tau | A | lattice_n | used_n | cos_min | rr_max_used | rr_cofinal | delta_tri | c_tri |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.tau:.9f} | {r.amplitude:.6e} | {r.lattice_points_total} | {r.lattice_points_used} | "
            f"{r.cos_abs_min_used:.6f} | {r.rr_max_used:.6f} | {r.rr_cofinal_grid:.6f} | "
            f"{r.delta_tri_cofinal_grid:.6f} | {r.c_beta_tri_cofinal_grid:.6e} |"
        )
    lines.append("")
    lines.append("Finite-window structured-lattice report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
