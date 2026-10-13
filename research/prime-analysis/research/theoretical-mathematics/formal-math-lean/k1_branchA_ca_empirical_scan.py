#!/usr/bin/env python3
"""Empirical scan of Branch-A remainder constant C_A from 100k-zero surrogate.

For omitted-tail residual in normalized form Y(x)=R(x)/x^beta, estimate:
  C_A_req(theta) = sup_x |Y_tail(x;theta)| * x^(theta-(1-beta)) / (log x)^2
for schedule T(x)=x^theta.

This is numerical evidence (finite-zero surrogate), not a theorem.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List

import numpy as np

from prime_geometry_loop import load_zeta_zeros_file


def parse_theta_list(text: str) -> List[float]:
    vals: List[float] = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        vals.append(float(token))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("theta-list is empty")
    return vals


def parse_theta_range(start: float, end: float, step: float) -> List[float]:
    vals: List[float] = []
    t = start
    while t <= end + 1.0e-12:
        vals.append(round(t, 10))
        t += step
    return vals


def prepare_x_grid(x1: float, xmax: float, grid_size: int) -> tuple[np.ndarray, np.ndarray]:
    t = np.linspace(math.log(x1), math.log(xmax), grid_size, dtype=np.float64)
    x = np.exp(t)
    return x, t


def build_cutoff_maps(
    zeros: np.ndarray,
    n_head: int,
    x: np.ndarray,
    thetas: List[float],
) -> Dict[float, np.ndarray]:
    out: Dict[float, np.ndarray] = {}
    m_omit = max(0, zeros.size - n_head)
    for th in thetas:
        tcut = np.power(x, th)
        k = np.searchsorted(zeros, tcut, side="right")
        k_omit = np.clip(k - n_head, 0, m_omit).astype(np.int64)
        out[th] = k_omit
    return out


def compute_omitted_prefix_snapshots(
    zeros: np.ndarray,
    n_head: int,
    beta: float,
    x: np.ndarray,
    t: np.ndarray,
    need_indices: List[int],
    zero_chunk: int,
) -> Dict[int, np.ndarray]:
    g = zeros[n_head:].astype(np.float64)
    m = g.size
    den = 0.25 + g * g
    c_coef = 1.0 / den
    s_coef = (2.0 * g) / den
    scale = np.power(x, 0.5 - beta)

    need = sorted(set(i for i in need_indices if 0 <= i <= m))
    snapshots: Dict[int, np.ndarray] = {}
    running = np.zeros_like(x)
    pos = 0
    need_ptr = 0

    # snapshot at 0
    if need_ptr < len(need) and need[need_ptr] == 0:
        snapshots[0] = running.copy()
        need_ptr += 1

    while pos < m:
        end = min(m, pos + zero_chunk)
        gj = g[pos:end]
        cj = c_coef[pos:end]
        sj = s_coef[pos:end]

        phase = np.outer(t, gj)
        mixed = -scale[:, None] * (np.cos(phase) * cj + np.sin(phase) * sj)

        # If there are needed prefix cuts in this chunk, use cumsum once.
        local_need = []
        while need_ptr < len(need) and need[need_ptr] <= end:
            if need[need_ptr] > pos:
                local_need.append(need[need_ptr] - pos)
            else:
                snapshots[need[need_ptr]] = running.copy()
            need_ptr += 1

        if local_need:
            pref = np.cumsum(mixed, axis=1)
            for take in local_need:
                idx = pos + take
                snapshots[idx] = running + pref[:, take - 1]

        running += np.sum(mixed, axis=1)
        pos = end

    # any trailing needs at m
    while need_ptr < len(need):
        snapshots[need[need_ptr]] = running.copy()
        need_ptr += 1

    return snapshots


def main() -> None:
    ap = argparse.ArgumentParser(description="Empirical Branch-A C_A scan")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko_100k.json",
    )
    ap.add_argument("--max-zeta-zeros", type=int, default=100000)
    ap.add_argument("--n-head", type=int, default=256)
    ap.add_argument("--beta", type=float, default=0.55)
    ap.add_argument("--eta-target", type=float, default=0.01)
    ap.add_argument("--x1", type=float, default=1.0e21)
    ap.add_argument("--xmax", type=float, default=1.0e30)
    ap.add_argument("--grid-size", type=int, default=1024)
    ap.add_argument("--zero-chunk", type=int, default=512)
    ap.add_argument("--theta-list", type=str, default="")
    ap.add_argument("--theta-start", type=float, default=0.13)
    ap.add_argument("--theta-end", type=float, default=0.70)
    ap.add_argument("--theta-step", type=float, default=0.03)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w34_branchA_ca_empirical_scan_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    if args.theta_list.strip():
        thetas = parse_theta_list(args.theta_list)
    else:
        thetas = parse_theta_range(args.theta_start, args.theta_end, args.theta_step)

    zeros_raw = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    zeros = np.array([z for z in zeros_raw if z > 0.0], dtype=np.float64)
    zeros.sort()
    if args.max_zeta_zeros > 0:
        zeros = zeros[: args.max_zeta_zeros]
    if zeros.size <= args.n_head:
        raise ValueError("need max_zeta_zeros > n_head")

    x, t = prepare_x_grid(args.x1, args.xmax, args.grid_size)
    k_map = build_cutoff_maps(zeros=zeros, n_head=args.n_head, x=x, thetas=thetas)
    all_need = sorted({0, zeros.size - args.n_head} | {int(v) for arr in k_map.values() for v in arr})

    snaps = compute_omitted_prefix_snapshots(
        zeros=zeros,
        n_head=args.n_head,
        beta=args.beta,
        x=x,
        t=t,
        need_indices=all_need,
        zero_chunk=args.zero_chunk,
    )

    total_omit = snaps[zeros.size - args.n_head]
    log2 = np.maximum(np.log(x), 1.0e-300) ** 2
    x_eta = np.power(x, args.eta_target)

    rows = []
    m_omit = zeros.size - args.n_head
    for th in thetas:
        k_omit = k_map[th]
        tail = np.empty_like(x)
        for kv in np.unique(k_omit):
            mask = k_omit == kv
            tail[mask] = total_omit[mask] - snaps[int(kv)][mask]

        abs_tail = np.abs(tail)
        clipped_frac = float(np.mean(k_omit >= m_omit))
        # Required CA for Branch-A form:
        # |tail| <= CA * x^(1-beta) * log^2 / x^theta
        # => CA >= |tail| * x^(theta-(1-beta)) / log^2
        ca_req = abs_tail * np.power(x, th - (1.0 - args.beta)) / log2
        ca_sup = float(np.max(ca_req))
        ca_q95 = float(np.quantile(ca_req, 0.95))
        ca_q99 = float(np.quantile(ca_req, 0.99))
        # Fit eta by log-log slope on absolute tail if non-trivial.
        if float(np.max(abs_tail)) > 0.0:
            lx = np.log(x)
            ly = np.log(np.maximum(abs_tail, 1.0e-300))
            mx = float(np.mean(lx))
            my = float(np.mean(ly))
            den = float(np.sum((lx - mx) ** 2))
            slope = float(np.sum((lx - mx) * (ly - my)) / den) if den > 1.0e-30 else 0.0
            eta_fit = max(0.0, -slope)
        else:
            eta_fit = 0.0
        c_eta = float(np.max(abs_tail * x_eta))

        eta_raw = th + args.beta - 1.0
        feasible_asymptotic = bool(eta_raw > args.eta_target)

        rows.append(
            {
                "theta": float(th),
                "eta_raw": float(eta_raw),
                "feasible_asymptotic_for_eta_target": feasible_asymptotic,
                "cutoff_clipped_fraction": clipped_frac,
                "ca_required_sup": ca_sup,
                "ca_required_q99": ca_q99,
                "ca_required_q95": ca_q95,
                "tail_eta_fit": float(eta_fit),
                "tail_c_eta_target": c_eta,
                "tail_sup_abs": float(np.max(abs_tail)),
            }
        )

    # Rank by CA required where scan is not clipped by finite M data.
    valid = [r for r in rows if r["cutoff_clipped_fraction"] < 1.0]
    if valid:
        best_ca = min(valid, key=lambda r: r["ca_required_sup"])
    else:
        best_ca = min(rows, key=lambda r: r["ca_required_sup"])

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": int(zeros.size),
            "n_head": int(args.n_head),
            "beta": float(args.beta),
            "eta_target": float(args.eta_target),
            "x1": float(args.x1),
            "xmax": float(args.xmax),
            "grid_size": int(args.grid_size),
            "zero_chunk": int(args.zero_chunk),
            "thetas": thetas,
        },
        "rows": rows,
        "best_theta_by_empirical_ca": best_ca,
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines = []
    lines.append(f"# K1 Branch-A Empirical C_A Scan ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        if k == "thetas":
            lines.append(f"- {k}: {len(v)} values from {v[0]} to {v[-1]}")
        else:
            lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append(
        "| theta | eta_raw | asymptotic_feasible | clipped_frac | CA_sup | CA_q99 | eta_fit | C_eta_target |"
    )
    lines.append("|---:|---:|:---:|---:|---:|---:|---:|---:|")
    for r in rows:
        lines.append(
            f"| {r['theta']:.2f} | {r['eta_raw']:.4f} | "
            f"{'yes' if r['feasible_asymptotic_for_eta_target'] else 'no'} | "
            f"{r['cutoff_clipped_fraction']:.3f} | "
            f"{r['ca_required_sup']:.6g} | {r['ca_required_q99']:.6g} | "
            f"{r['tail_eta_fit']:.4f} | {r['tail_c_eta_target']:.6g} |"
        )
    lines.append("")
    lines.append("## Best by Empirical C_A")
    lines.append(f"- theta: {best_ca['theta']}")
    lines.append(f"- CA_sup: {best_ca['ca_required_sup']}")
    lines.append(f"- eta_raw: {best_ca['eta_raw']}")
    lines.append("")
    lines.append("## Interpretation")
    lines.append(
        "- `CA_sup` estimates the finite-range constant needed for Branch-A remainder inequality using 100k-zero surrogate tails."
    )
    lines.append(
        "- Asymptotic feasibility still requires `eta_raw > eta_target`; rows marked `no` fail that condition for x->infinity."
    )
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
