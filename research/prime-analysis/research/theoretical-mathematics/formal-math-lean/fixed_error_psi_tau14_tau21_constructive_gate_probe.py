#!/usr/bin/env python3
"""Constructive dual-phase subsequence probe for tau14/tau21.

For each tau1 phase anchor k:
  x_k* = exp((2*pi*k - phi1)/tau1)
Pick an integer n_k in [round(x_k*)-W, round(x_k*)+W] such that
  |cos1(n_k)| >= a1
and selecting by a gate mode:
  - suppress: minimize |cos2(n_k)|
  - nonnegative: require cos2(n_k)>=0 and minimize |cos2(n_k)|

This builds an explicit one-point-per-anchor subsequence and evaluates
the effective margin:
  delta_eff = a1 - q_cofinal,
where alpha=A2/A1, b2=max |cos2| on selected points, rr=|R2|/A1.
"""

from __future__ import annotations

import argparse
import json
import math
from dataclasses import asdict, dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List, Tuple

import numpy as np

from fixed_error_psi_contradiction_probe import logspace_int, psi_at_samples, psi_events
from fixed_error_psi_tau14_tau21_phase_gate_probe import _crossover_x, _fit_two_mode, _upper_gap


def parse_int_list(text: str) -> List[int]:
    vals: List[int] = []
    for raw in text.split(","):
        tok = raw.strip()
        if not tok:
            continue
        vals.append(int(tok))
    if not vals:
        raise ValueError("empty int list")
    return vals


def _phase_anchors(tau1: float, phi1: float, xmin: int, xmax: int) -> List[int]:
    two_pi = 2.0 * math.pi
    k_lo = int(math.ceil((tau1 * math.log(float(xmin)) + phi1) / two_pi))
    k_hi = int(math.floor((tau1 * math.log(float(xmax)) + phi1) / two_pi))
    if k_hi < k_lo:
        return []
    return list(range(k_lo, k_hi + 1))


def _anchor_center(k: int, tau1: float, phi1: float) -> int:
    two_pi = 2.0 * math.pi
    x0 = math.exp((two_pi * float(k) - phi1) / tau1)
    return int(round(x0))


@dataclass
class WindowRow:
    search_window: int
    anchor_count: int
    selected_count: int
    b2_max_selected: float
    b2_cofinal_grid: float
    rr_cofinal_grid: float
    alpha_b2_cofinal: float
    q_cofinal_grid: float
    q_eff_cofinal: float
    delta_eff: float
    c_beta_eff: float
    c_beta_obs_cofinal: float
    crossover_x_eff: float | None


def main() -> None:
    ap = argparse.ArgumentParser(description="Constructive tau14/tau21 gate probe")
    ap.add_argument("--xmin", type=int, default=10_000)
    ap.add_argument("--xmax", type=int, default=50_000_000)
    ap.add_argument("--fit-samples", type=int, default=10_000)
    ap.add_argument("--beta", type=float, default=0.62)
    ap.add_argument("--tau1", type=float, default=14.134725142)
    ap.add_argument("--tau2", type=float, default=21.022039639)
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--search-window-list", type=str, default="20,50,100,200,500,1000,2000,5000")
    ap.add_argument(
        "--gate-mode",
        type=str,
        choices=["suppress", "nonnegative"],
        default="suppress",
        help="suppress: minimize |cos2|; nonnegative: require cos2>=0 and minimize |cos2|",
    )
    ap.add_argument("--cofinal-grid-count", type=int, default=60)
    ap.add_argument("--cofinal-xmax-frac", type=float, default=0.95)
    ap.add_argument("--min-selected-points", type=int, default=8)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/fixed_error_psi")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/fixed_error_psi_tau14_tau21_constructive_gate_probe_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.beta <= 0.5:
        raise ValueError("beta must be > 0.5")
    if args.a1 <= 0.0 or args.a1 > 1.0:
        raise ValueError("a1 must be in (0,1]")

    w_list = sorted(set(w for w in parse_int_list(args.search_window_list) if w >= 0))
    if not w_list:
        raise ValueError("search-window list empty")

    xs = logspace_int(args.xmin, args.xmax, args.fit_samples)
    x = xs.astype(np.float64)
    events = psi_events(args.xmax, Path(args.cache_dir))
    psi_vals = psi_at_samples(xs, events)
    e_vals = psi_vals - x
    y = e_vals / np.power(x, float(args.beta))
    abs_e = np.abs(e_vals)
    c_endpoint = float(np.max(abs_e / (np.sqrt(x) * np.power(np.log(x), 2))))

    fit = _fit_two_mode(
        x=x,
        y=y,
        tau1=float(args.tau1),
        tau2=float(args.tau2),
        beta=float(args.beta),
        include_decay_term=bool(args.include_decay_term),
    )
    a1_amp = float(fit["A1"])
    a2_amp = float(fit["A2"])
    if a1_amp <= 1.0e-18:
        raise ValueError("A1 too small")
    alpha = float(a2_amp / a1_amp)
    phi1 = float(fit["phi1"])
    phi2 = float(fit["phi2"])
    coef = np.asarray(fit["coef"], dtype=np.float64)

    anchors = _phase_anchors(float(args.tau1), phi1, int(args.xmin), int(args.xmax))
    if not anchors:
        raise ValueError("no anchors in range")
    centers = [_anchor_center(k, float(args.tau1), phi1) for k in anchors]
    w_max = max(w_list)

    # Precompute psi(n) for all integers that might be visited across all windows.
    candidate_set: set[int] = set()
    for c in centers:
        lo = max(int(args.xmin), c - int(w_max))
        hi = min(int(args.xmax), c + int(w_max))
        if hi < lo:
            continue
        candidate_set.update(range(lo, hi + 1))
    if not candidate_set:
        raise ValueError("no candidate integers for constructive search")
    cand_sorted = np.array(sorted(candidate_set), dtype=np.int64)
    psi_cand = psi_at_samples(cand_sorted, events)
    psi_lookup: Dict[int, float] = {int(n): float(v) for n, v in zip(cand_sorted.tolist(), psi_cand.tolist())}

    rows: List[WindowRow] = []
    for w in w_list:
        selected: List[int] = []
        b2_vals: List[float] = []
        rr_vals: List[float] = []
        yabs_vals: List[float] = []

        for c in centers:
            lo = max(int(args.xmin), c - int(w))
            hi = min(int(args.xmax), c + int(w))
            if hi < lo:
                continue

            best: Tuple[float, float, int] | None = None  # (|cos2|, rr, n)
            for n in range(lo, hi + 1):
                nf = float(max(2, n))
                t = math.log(nf)
                cos1 = abs(math.cos(float(args.tau1) * t + phi1))
                if cos1 < float(args.a1):
                    continue
                cos2_signed = math.cos(float(args.tau2) * t + phi2)
                cos2 = abs(cos2_signed)
                if args.gate_mode == "nonnegative" and cos2_signed < 0.0:
                    continue
                c1 = math.cos(float(args.tau1) * t)
                s1 = math.sin(float(args.tau1) * t)
                c2 = math.cos(float(args.tau2) * t)
                s2 = math.sin(float(args.tau2) * t)
                vec = [c1, s1, c2, s2]
                if bool(args.include_decay_term):
                    vec.append(nf ** (-float(args.beta)))
                yhat = float(np.dot(np.array(vec, dtype=np.float64), coef))
                psi_n = psi_lookup[int(n)]
                y_n = float((psi_n - nf) / (nf ** float(args.beta)))
                rr = abs(y_n - yhat) / a1_amp
                if args.gate_mode == "suppress":
                    cand = (cos2, rr, n)
                else:
                    # Phase-only selection (no residual-based cherry-picking).
                    cand = (cos2, rr, n)
                if best is None or cand[0] < best[0] or (cand[0] == best[0] and cand[1] < best[1]):
                    best = cand

            if best is None:
                continue
            cos2_min, rr_min, n_sel = best
            selected.append(int(n_sel))
            b2_vals.append(float(cos2_min))
            rr_vals.append(float(rr_min))
            nf = float(n_sel)
            psi_n = psi_lookup[int(n_sel)]
            yabs_vals.append(abs((psi_n - nf) / (nf ** float(args.beta))))

        if len(selected) < int(args.min_selected_points):
            continue

        x_sel = np.array(sorted(selected), dtype=np.float64)
        rr_arr = np.array([rr for _, rr in sorted(zip(selected, rr_vals))], dtype=np.float64)
        b2_arr = np.array([b2 for _, b2 in sorted(zip(selected, b2_vals))], dtype=np.float64)
        yabs_arr = np.array([ya for _, ya in sorted(zip(selected, yabs_vals))], dtype=np.float64)
        if args.gate_mode == "suppress":
            q_arr = alpha * b2_arr + rr_arr
        else:
            q_arr = rr_arr

        x_end = float(args.cofinal_xmax_frac) * float(np.max(x_sel))
        if x_end <= x_sel[0]:
            x_end = float(np.max(x_sel))
        grid = np.exp(np.linspace(math.log(float(x_sel[0])), math.log(float(x_end)), int(args.cofinal_grid_count)))

        min_rr_future: List[float] = []
        min_b2_future: List[float] = []
        min_q_future: List[float] = []
        max_obs_future: List[float] = []
        for x0 in grid:
            m = x_sel >= x0
            if not np.any(m):
                continue
            min_rr_future.append(float(np.min(rr_arr[m])))
            min_b2_future.append(float(np.min(b2_arr[m])))
            min_q_future.append(float(np.min(q_arr[m])))
            max_obs_future.append(float(np.max(yabs_arr[m])))
        if not min_rr_future or not min_b2_future or not min_q_future:
            continue

        rr_cofinal = float(max(min_rr_future))
        b2_cofinal = float(max(min_b2_future))
        q_cofinal = float(max(min_q_future))
        b2_max = float(np.max(b2_vals))
        alpha_b2 = float(alpha * b2_cofinal) if args.gate_mode == "suppress" else 0.0
        q_eff = float(q_cofinal)
        delta = float(float(args.a1) - q_eff)
        c_eff = float(max(0.0, delta) * a1_amp)
        c_obs = float(min(max_obs_future))

        rows.append(
            WindowRow(
                search_window=int(w),
                anchor_count=len(anchors),
                selected_count=len(selected),
                b2_max_selected=b2_max,
                b2_cofinal_grid=b2_cofinal,
                rr_cofinal_grid=rr_cofinal,
                alpha_b2_cofinal=alpha_b2,
                q_cofinal_grid=q_cofinal,
                q_eff_cofinal=q_eff,
                delta_eff=delta,
                c_beta_eff=c_eff,
                c_beta_obs_cofinal=c_obs,
                crossover_x_eff=_crossover_x(c_eff, float(args.beta), c_endpoint, float(args.xmin)),
            )
        )

    rows.sort(key=lambda r: (-r.delta_eff, r.search_window))

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
            "search_window_list": w_list,
            "gate_mode": str(args.gate_mode),
            "cofinal_grid_count": int(args.cofinal_grid_count),
            "cofinal_xmax_frac": float(args.cofinal_xmax_frac),
            "min_selected_points": int(args.min_selected_points),
            "include_decay_term": bool(args.include_decay_term),
            "event_count": int(len(events)),
            "endpoint_c_sup_window": c_endpoint,
            "anchor_count_total": len(anchors),
        },
        "fit": {
            "A1": a1_amp,
            "A2": a2_amp,
            "alpha_A2_over_A1": alpha,
            "phi1": phi1,
            "phi2": phi2,
            "rmse_global_fit": float(fit["rmse"]),
        },
        "rows": [asdict(r) for r in rows],
        "interpretation": {
            "note": (
                "Finite-window constructive subsequence evidence only. "
                "Positive delta_eff indicates a viable explicit gated witness candidate."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    json_path = out_prefix.with_suffix(".json")
    md_path = out_prefix.with_suffix(".md")
    json_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# Tau14/Tau21 Constructive Gate Probe ({date.today().isoformat()})")
    lines.append("")
    lines.append(f"- Window: `[xmin, xmax] = [{args.xmin}, {args.xmax}]`")
    lines.append(f"- Beta: `{args.beta}`")
    lines.append(f"- `A1={a1_amp:.9e}`, `A2={a2_amp:.9e}`, `alpha={alpha:.6f}`")
    lines.append(f"- Gate mode: `{args.gate_mode}`")
    lines.append(f"- Anchor count: `{len(anchors)}`")
    lines.append("")
    lines.append("| search_W | selected_n | b2_max | b2_cofinal | rr_cofinal | q_cofinal | delta_eff | c_eff |")
    lines.append("|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r.search_window} | {r.selected_count} | {r.b2_max_selected:.6f} | {r.b2_cofinal_grid:.6f} | "
            f"{r.rr_cofinal_grid:.6f} | {r.q_cofinal_grid:.6f} | {r.delta_eff:.6f} | {r.c_beta_eff:.6e} |"
        )
    lines.append("")
    lines.append("Finite-window report only; not a theorem proof.")
    md_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(json_path)
    print(md_path)


if __name__ == "__main__":
    main()
