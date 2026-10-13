#!/usr/bin/env python3
"""Greedy multi-mode phase probe for K1 source contract research.

This extends the single-mode probe by fitting
  E(x)/x^beta ~= sum_j (a_j cos(tau_j log x) + b_j sin(tau_j log x)) + optional decay term
with tau_j selected greedily from candidate zeta-zero frequencies.

Numerical evidence only; not a proof term.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

import numpy as np

from prime_geometry_loop import load_zeta_zeros_file
from k1_source_shape_probe import (
    compute_scaled_signal,
    key_hash,
    majorant_profile,
    parse_beta_grid,
    tail_cumulative_sup_abs,
    zeros_signature,
)


def fit_with_taus(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    taus: Sequence[float],
    tail_frac: float,
    include_decay_term: bool,
) -> Dict[str, object]:
    cols: List[np.ndarray] = []
    for tau in taus:
        cols.append(np.cos(float(tau) * t))
        cols.append(np.sin(float(tau) * t))
    if include_decay_term:
        cols.append(x ** (-beta))

    xmat = np.column_stack(cols)
    coef, _, _, _ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    r = y - yhat

    k = max(32, int(round(tail_frac * y.size)))
    k = min(k, y.size)
    tail = r[-k:]

    amps = []
    for i in range(len(taus)):
        a = float(coef[2 * i])
        b = float(coef[2 * i + 1])
        amps.append(math.hypot(a, b))

    amp_total = float(np.sqrt(np.sum(np.square(amps)))) if amps else 0.0
    global_rmse = float(np.sqrt(np.mean(r * r)))
    tail_rmse = float(np.sqrt(np.mean(tail * tail)))
    tail_sup = float(np.max(np.abs(tail)))
    tail_ratio = tail_sup / max(amp_total, 1.0e-12)

    tail_env = tail_cumulative_sup_abs(r)
    x_tail = x[-k:]
    env_tail = tail_env[-k:]
    lx = np.log(np.maximum(x_tail, 1.0e-300))
    ly = np.log(np.maximum(env_tail, 1.0e-300))
    mx = float(np.mean(lx))
    my = float(np.mean(ly))
    den = float(np.sum((lx - mx) ** 2))
    slope = 0.0 if den <= 1.0e-30 else float(np.sum((lx - mx) * (ly - my)) / den)
    decay_exp = -slope

    maj = majorant_profile(x=x, r=r, tail_frac=tail_frac)

    score = tail_ratio + global_rmse / max(amp_total, 1.0e-12)
    mode_rows = []
    for i, tau in enumerate(taus):
        mode_rows.append(
            {
                "tau": float(tau),
                "a": float(coef[2 * i]),
                "b": float(coef[2 * i + 1]),
                "amplitude": float(amps[i]),
            }
        )

    out: Dict[str, object] = {
        "taus": [float(v) for v in taus],
        "mode_rows": mode_rows,
        "mode_count": int(len(taus)),
        "amplitude_total": amp_total,
        "global_rmse": global_rmse,
        "tail_rmse": tail_rmse,
        "tail_sup_abs": tail_sup,
        "tail_ratio_sup_to_amp_total": tail_ratio,
        "tail_loglog_slope_sup": float(slope),
        "tail_decay_exponent_sup": float(decay_exp),
        "remainder_majorant_eta": float(maj["eta"]),
        "remainder_majorant_C_all": float(maj["C_all"]),
        "remainder_majorant_C_tail": float(maj["C_tail"]),
        "remainder_majorant_tail_start_x": float(maj["tail_start_x"]),
        "remainder_majorant_tail_end_x": float(maj["tail_end_x"]),
        "remainder_majorant_tail_max_ratio_to_bound": float(maj["tail_max_ratio_to_bound"]),
        "remainder_majorant_tail_mean_ratio_to_bound": float(maj["tail_mean_ratio_to_bound"]),
        "remainder_majorant_all_max_ratio_to_bound": float(maj["all_max_ratio_to_bound"]),
        "remainder_majorant_implied_tendsto_zero_shape": bool(maj["implied_tendsto_zero_shape"]),
        "score": float(score),
    }
    if include_decay_term:
        out["decay_term_coeff"] = float(coef[-1])
    return out


def pick_tag(row: Dict[str, object]) -> str:
    ratio = float(row["tail_ratio_sup_to_amp_total"])
    eta = float(row.get("remainder_majorant_eta", 0.0))
    if ratio <= 0.25 and eta > 0.0:
        return "strong_multimode_plus_decaying_tail_finite_range"
    if ratio <= 0.60 and eta > 0.0:
        return "moderate_multimode_plus_decaying_tail_finite_range"
    if ratio <= 1.00 and eta > 0.0:
        return "near_strict_multimode_tail_dominance_finite_range"
    if ratio <= 1.10 and eta > 0.0:
        return "borderline_multimode_tail_dominance_finite_range"
    return "weak_multimode_dominance_finite_range"


def greedy_modes(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau_candidates: Sequence[float],
    max_modes: int,
    tail_frac: float,
    include_decay_term: bool,
) -> List[Dict[str, object]]:
    selected: List[float] = []
    chosen_set = set()
    packs: List[Dict[str, object]] = []

    for step in range(1, max_modes + 1):
        best_tau = None
        best_pack = None

        for tau in tau_candidates:
            tau_f = float(tau)
            if tau_f in chosen_set:
                continue
            try_pack = fit_with_taus(
                x=x,
                t=t,
                y=y,
                beta=beta,
                taus=selected + [tau_f],
                tail_frac=tail_frac,
                include_decay_term=include_decay_term,
            )
            if best_pack is None or float(try_pack["score"]) < float(best_pack["score"]):
                best_pack = try_pack
                best_tau = tau_f

        if best_tau is None or best_pack is None:
            break

        selected.append(best_tau)
        chosen_set.add(best_tau)
        best_pack["selected_at_step"] = step
        best_pack["interpretation"] = pick_tag(best_pack)
        packs.append(best_pack)

    return packs


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 multi-mode phase probe")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=20000)
    ap.add_argument("--tau-candidate-count", type=int, default=64)
    ap.add_argument("--beta", type=float, default=0.6)
    ap.add_argument("--x-min", type=float, default=1.0e7)
    ap.add_argument("--x-max", type=float, default=1.0e20)
    ap.add_argument("--grid-size", type=int, default=4096)
    ap.add_argument("--zero-chunk", type=int, default=512)
    ap.add_argument("--max-modes", type=int, default=4)
    ap.add_argument("--tail-frac", type=float, default=0.20)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--cache-dir", type=str, default="research/cache/k1_source_shape_probe")
    ap.add_argument("--output", type=str, default="research/output/k1_multimode_phase_probe.json")
    args = ap.parse_args()

    beta_grid = parse_beta_grid(str(args.beta))
    beta = float(beta_grid[0])

    zeros_raw = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    zeros = [z for z in zeros_raw if z > 0.0]
    zeros.sort()
    if args.max_zeta_zeros > 0:
        zeros = zeros[: args.max_zeta_zeros]
    if not zeros:
        raise ValueError("no positive zeta zeros loaded")

    tau_candidates = zeros[: max(1, min(args.tau_candidate_count, len(zeros)))]

    zsig = zeros_signature(zeros)
    os.makedirs(args.cache_dir, exist_ok=True)
    cfg = {
        "zeros_sig": zsig,
        "zeros_used": len(zeros),
        "beta": beta,
        "x_min": float(args.x_min),
        "x_max": float(args.x_max),
        "grid_size": int(args.grid_size),
        "zero_chunk": int(args.zero_chunk),
    }
    ckey = key_hash(cfg)
    cpath = os.path.join(args.cache_dir, ckey + ".npz")

    if os.path.exists(cpath):
        loaded = np.load(cpath)
        x = loaded["x"]
        t = loaded["t"]
        y = loaded["y"]
        cache_status = "warm"
    else:
        computed = compute_scaled_signal(
            zeros=zeros,
            beta=beta,
            x_min=args.x_min,
            x_max=args.x_max,
            grid_size=args.grid_size,
            zero_chunk=args.zero_chunk,
        )
        x = computed["x"]
        t = computed["t"]
        y = computed["y"]
        np.savez_compressed(cpath, x=x, t=t, y=y)
        cache_status = "cold"

    packs = greedy_modes(
        x=x,
        t=t,
        y=y,
        beta=beta,
        tau_candidates=tau_candidates,
        max_modes=max(1, int(args.max_modes)),
        tail_frac=float(args.tail_frac),
        include_decay_term=bool(args.include_decay_term),
    )

    if not packs:
        raise RuntimeError("no mode packs produced")

    best_pack = min(packs, key=lambda r: float(r["score"]))

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "zeros_used": len(zeros),
            "tau_candidate_count": len(tau_candidates),
            "beta": beta,
            "x_min": args.x_min,
            "x_max": args.x_max,
            "grid_size": args.grid_size,
            "zero_chunk": args.zero_chunk,
            "max_modes": args.max_modes,
            "tail_frac": args.tail_frac,
            "include_decay_term": bool(args.include_decay_term),
            "cache_dir": args.cache_dir,
            "cache_path": cpath,
            "cache_status": cache_status,
        },
        "mode_steps": packs,
        "best_by_score": best_pack,
        "interpretation": {
            "best": str(best_pack["interpretation"]),
            "note": "Finite-range greedy multi-mode fit only; not a theorem term.",
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    md_path = args.output.replace(".json", ".md")
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# K1 Multi-Mode Phase Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        f.write(f"- cache status: {cache_status}\n")
        f.write(f"- beta: {beta}\n")
        f.write(f"- zeros used: {len(zeros)}\n")
        f.write(f"- tau candidates: {len(tau_candidates)}\n")
        f.write(f"- max modes: {args.max_modes}\n")
        f.write(f"- x-range: [{args.x_min:g}, {args.x_max:g}]\n")
        f.write(f"- overall best interpretation: {result['interpretation']['best']}\n\n")
        f.write("## Greedy steps\n\n")
        for row in packs:
            taus = ", ".join(f"{float(v):.12g}" for v in row["taus"])
            f.write(
                f"- modes={row['mode_count']} | taus=[{taus}] | score={float(row['score']):.6f} | "
                f"tail_sup/amp_total={float(row['tail_ratio_sup_to_amp_total']):.6f} | "
                f"tail_decay_exp={float(row['tail_decay_exponent_sup']):.6f} | "
                f"tag={row['interpretation']}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md_path}")
    print("cache_status:", cache_status)
    print("best_mode_count:", int(best_pack["mode_count"]))
    print("best_score:", f"{float(best_pack['score']):.6f}")
    print("best_tail_ratio_sup_to_amp_total:", f"{float(best_pack['tail_ratio_sup_to_amp_total']):.6e}")
    print("best_interpretation:", best_pack["interpretation"])


if __name__ == "__main__":
    main()
