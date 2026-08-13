#!/usr/bin/env python3
"""Probe K1-source waveform shape from truncated zeta explicit formula.

Goal:
- test whether E(x)/x^beta is well-approximated (finite range) by
  a single mode a*cos(tau*log x) + b*sin(tau*log x) plus a decaying tail.
- rank candidate tau values from the zeta-zero list.

This is a numerical research probe (not a proof term).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import numpy as np

from prime_geometry_loop import load_zeta_zeros_file


def key_hash(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def zeros_signature(zeros: Sequence[float]) -> str:
    if not zeros:
        return "empty"
    m = min(2048, len(zeros))
    text = ",".join(f"{z:.10f}" for z in zeros[:m])
    return hashlib.sha1(text.encode("utf-8")).hexdigest()[:16]


def parse_beta_grid(text: str) -> List[float]:
    vals = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        vals.append(float(token))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("beta grid is empty")
    return vals


def tail_cumulative_sup_abs(arr: np.ndarray) -> np.ndarray:
    out = np.empty_like(arr)
    best = -1.0
    for i in range(arr.size - 1, -1, -1):
        v = float(abs(arr[i]))
        if v > best:
            best = v
        out[i] = best
    return out


def slope_log_log(x: np.ndarray, y: np.ndarray) -> float:
    if x.size != y.size or x.size < 2:
        return 0.0
    lx = np.log(np.maximum(x, 1.0e-300))
    ly = np.log(np.maximum(y, 1.0e-300))
    mx = float(np.mean(lx))
    my = float(np.mean(ly))
    num = float(np.sum((lx - mx) * (ly - my)))
    den = float(np.sum((lx - mx) ** 2))
    if den <= 1.0e-30:
        return 0.0
    return num / den


def majorant_profile(
    x: np.ndarray,
    r: np.ndarray,
    tail_frac: float,
) -> Dict[str, float]:
    if x.size != r.size or x.size < 32:
        return {
            "eta": 0.0,
            "C_all": float(np.max(np.abs(r))) if r.size else 0.0,
            "C_tail": float(np.max(np.abs(r))) if r.size else 0.0,
            "tail_start_x": float(x[0]) if x.size else 0.0,
            "tail_end_x": float(x[-1]) if x.size else 0.0,
            "tail_max_ratio_to_bound": 1.0,
            "tail_mean_ratio_to_bound": 1.0,
            "all_max_ratio_to_bound": 1.0,
            "implied_tendsto_zero_shape": False,
        }

    k = max(32, int(round(tail_frac * x.size)))
    k = min(k, x.size)
    x_tail = x[-k:]
    r_tail = r[-k:]
    env_tail = tail_cumulative_sup_abs(r)[-k:]

    slope = slope_log_log(x_tail, env_tail)
    eta = max(0.0, -slope)

    x_eta_all = np.power(x, eta)
    x_eta_tail = np.power(x_tail, eta)
    abs_r_all = np.abs(r)
    abs_r_tail = np.abs(r_tail)

    c_all = float(np.max(abs_r_all * x_eta_all))
    c_tail = float(np.max(abs_r_tail * x_eta_tail))
    c_all = max(c_all, 1.0e-300)
    c_tail = max(c_tail, 1.0e-300)

    bound_all = c_all * np.power(x, -eta)
    bound_tail = c_tail * np.power(x_tail, -eta)
    ratio_all = abs_r_all / np.maximum(bound_all, 1.0e-300)
    ratio_tail = abs_r_tail / np.maximum(bound_tail, 1.0e-300)

    return {
        "eta": float(eta),
        "C_all": float(c_all),
        "C_tail": float(c_tail),
        "tail_start_x": float(x_tail[0]),
        "tail_end_x": float(x_tail[-1]),
        "tail_max_ratio_to_bound": float(np.max(ratio_tail)),
        "tail_mean_ratio_to_bound": float(np.mean(ratio_tail)),
        "all_max_ratio_to_bound": float(np.max(ratio_all)),
        "implied_tendsto_zero_shape": bool(eta > 0.0),
    }


def compute_scaled_signal(
    zeros: Sequence[float],
    beta: float,
    x_min: float,
    x_max: float,
    grid_size: int,
    zero_chunk: int,
) -> Dict[str, np.ndarray]:
    if x_min <= 1.0:
        raise ValueError("x_min must be > 1")
    if x_max <= x_min:
        raise ValueError("x_max must be > x_min")
    if grid_size < 32:
        raise ValueError("grid_size must be >= 32")

    t = np.linspace(math.log(x_min), math.log(x_max), grid_size, dtype=np.float64)
    x = np.exp(t)
    y = np.zeros_like(x)

    scale = x ** (0.5 - beta)
    decay_term = math.log(2.0 * math.pi) * (x ** (-beta))

    g = np.asarray(zeros, dtype=np.float64)
    den = 0.25 + g * g
    c_coef = 1.0 / den
    s_coef = (2.0 * g) / den

    for start in range(0, g.size, zero_chunk):
        end = min(g.size, start + zero_chunk)
        gj = g[start:end]
        cj = c_coef[start:end]
        sj = s_coef[start:end]
        phase = np.outer(t, gj)
        # y = E(x)/x^beta with explicit-formula truncation at selected zeros.
        # E(x) ~= -2*sqrt(x)*sum((0.5 cos + g sin)/(0.25+g^2)) - log(2*pi)
        mixed = np.cos(phase) * cj + np.sin(phase) * sj
        y -= scale * np.sum(mixed, axis=1)

    y -= decay_term
    return {"x": x, "t": t, "y": y}


def fit_single_mode(
    x: np.ndarray,
    t: np.ndarray,
    y: np.ndarray,
    beta: float,
    tau: float,
    tail_frac: float,
    include_decay_term: bool,
) -> Dict[str, float]:
    c = np.cos(tau * t)
    s = np.sin(tau * t)
    cols = [c, s]
    if include_decay_term:
        cols.append(x ** (-beta))

    xmat = np.column_stack(cols)
    coef, _, _, _ = np.linalg.lstsq(xmat, y, rcond=None)
    yhat = xmat @ coef
    r = y - yhat

    k = max(32, int(round(tail_frac * y.size)))
    k = min(k, y.size)
    tail = r[-k:]

    amp = float(math.hypot(float(coef[0]), float(coef[1])))
    global_rmse = float(np.sqrt(np.mean(r * r)))
    tail_rmse = float(np.sqrt(np.mean(tail * tail)))
    tail_sup = float(np.max(np.abs(tail)))
    tail_ratio_sup_to_amp = tail_sup / max(amp, 1.0e-12)

    tail_env = tail_cumulative_sup_abs(r)
    env_tail = tail_env[-k:]
    x_tail = x[-k:]
    slope = slope_log_log(x_tail, env_tail)
    decay_exponent = -slope
    maj = majorant_profile(x=x, r=r, tail_frac=tail_frac)

    score = tail_ratio_sup_to_amp + (global_rmse / max(amp, 1.0e-12))
    row = {
        "tau": float(tau),
        "a": float(coef[0]),
        "b": float(coef[1]),
        "amplitude": amp,
        "global_rmse": global_rmse,
        "tail_rmse": tail_rmse,
        "tail_sup_abs": tail_sup,
        "tail_ratio_sup_to_amp": tail_ratio_sup_to_amp,
        "tail_loglog_slope_sup": slope,
        "tail_decay_exponent_sup": decay_exponent,
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
        row["decay_term_coeff"] = float(coef[2])
    return row


def pick_interpretation(row: Dict[str, float]) -> str:
    ratio = float(row["tail_ratio_sup_to_amp"])
    dexp = float(row["tail_decay_exponent_sup"])
    eta = float(row.get("remainder_majorant_eta", 0.0))
    if ratio <= 0.25 and dexp >= 0.05 and eta > 0.0:
        return "strong_single_mode_plus_decaying_tail_finite_range"
    if ratio <= 0.60 and dexp > 0.0 and eta > 0.0:
        return "moderate_single_mode_plus_decaying_tail_finite_range"
    if ratio <= 1.00 and dexp > 0.0 and eta > 0.0:
        return "near_strict_single_mode_tail_dominance_finite_range"
    if ratio <= 1.10 and eta > 0.0:
        return "borderline_single_mode_tail_dominance_finite_range"
    return "weak_single_mode_dominance_finite_range"


def main() -> None:
    ap = argparse.ArgumentParser(description="K1-source shape probe")
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko100k_2026-02-18.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=2048)
    ap.add_argument("--tau-candidate-count", type=int, default=64)
    ap.add_argument("--beta-grid", type=str, default="0.50,0.52,0.55,0.58,0.60")
    ap.add_argument("--x-min", type=float, default=1.0e4)
    ap.add_argument("--x-max", type=float, default=1.0e8)
    ap.add_argument("--grid-size", type=int, default=12000)
    ap.add_argument("--tail-frac", type=float, default=0.20)
    ap.add_argument("--include-decay-term", dest="include_decay_term", action="store_true")
    ap.add_argument("--no-decay-term", dest="include_decay_term", action="store_false")
    ap.set_defaults(include_decay_term=True)
    ap.add_argument("--zero-chunk", type=int, default=128)
    ap.add_argument("--cache-dir", type=str, default="research/cache/k1_source_shape_probe")
    ap.add_argument("--output", type=str, default="research/output/k1_source_shape_probe.json")
    args = ap.parse_args()

    beta_grid = parse_beta_grid(args.beta_grid)
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

    per_beta = []
    overall_best = None

    for beta in beta_grid:
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

        rows = []
        for tau in tau_candidates:
            rows.append(
                fit_single_mode(
                    x=x,
                    t=t,
                    y=y,
                    beta=beta,
                    tau=float(tau),
                    tail_frac=args.tail_frac,
                    include_decay_term=bool(args.include_decay_term),
                )
            )
        rows.sort(key=lambda r: (r["score"], r["tail_ratio_sup_to_amp"]))
        best = rows[0]
        best["interpretation"] = pick_interpretation(best)

        beta_pack = {
            "beta": beta,
            "cache_status": cache_status,
            "cache_path": cpath,
            "best": best,
            "top5": rows[:5],
        }
        per_beta.append(beta_pack)
        if overall_best is None or float(best["score"]) < float(overall_best["best"]["score"]):
            overall_best = beta_pack

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "zeros_used": len(zeros),
            "tau_candidate_count": len(tau_candidates),
            "beta_grid": beta_grid,
            "x_min": args.x_min,
            "x_max": args.x_max,
            "grid_size": args.grid_size,
            "tail_frac": args.tail_frac,
            "include_decay_term": bool(args.include_decay_term),
            "cache_dir": args.cache_dir,
        },
        "overall_best": overall_best,
        "per_beta": per_beta,
        "interpretation": {
            "overall": overall_best["best"]["interpretation"] if overall_best else "unavailable",
            "note": "Finite-range waveform-fit evidence only; not a formal theorem term.",
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    md_path = args.output.replace(".json", ".md")
    with open(md_path, "w", encoding="utf-8") as f:
        f.write("# K1 Source Shape Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        f.write(f"- zeros used: {result['config']['zeros_used']}\n")
        f.write(f"- tau candidates: {result['config']['tau_candidate_count']}\n")
        f.write(f"- beta grid: {result['config']['beta_grid']}\n")
        f.write(f"- x-range: [{args.x_min:g}, {args.x_max:g}]\n")
        f.write(f"- grid size: {args.grid_size}\n")
        f.write(f"- overall interpretation: {result['interpretation']['overall']}\n\n")
        if overall_best is not None:
            b = overall_best["best"]
            f.write("## Overall Best Fit\n\n")
            f.write(f"- beta: {overall_best['beta']:.6f}\n")
            f.write(f"- tau: {b['tau']:.12g}\n")
            f.write(f"- amplitude: {b['amplitude']:.6e}\n")
            f.write(f"- tail ratio (sup/amp): {b['tail_ratio_sup_to_amp']:.6e}\n")
            f.write(f"- tail decay exponent (sup envelope): {b['tail_decay_exponent_sup']:.6f}\n")
            f.write(f"- remainder majorant eta: {b['remainder_majorant_eta']:.6f}\n")
            f.write(f"- remainder majorant C_all: {b['remainder_majorant_C_all']:.6e}\n")
            f.write(f"- remainder majorant C_tail: {b['remainder_majorant_C_tail']:.6e}\n")
            f.write(
                f"- remainder majorant tail window: [{b['remainder_majorant_tail_start_x']:.6g}, "
                f"{b['remainder_majorant_tail_end_x']:.6g}]\n"
            )
            f.write(
                f"- remainder majorant max ratio (all grid): "
                f"{b['remainder_majorant_all_max_ratio_to_bound']:.6f}\n"
            )
            f.write(
                f"- remainder majorant max ratio (tail): "
                f"{b['remainder_majorant_tail_max_ratio_to_bound']:.6f}\n"
            )
            f.write(f"- score: {b['score']:.6e}\n")
            f.write(f"- interpretation: {b['interpretation']}\n")
            f.write("\n")
            f.write("## Candidate Finite-Range Witness\n\n")
            f.write(
                "- Normalized model: "
                "`E(x)/x^beta ≈ a*cos(tau*log x) + b*sin(tau*log x) + remainder(x)`\n"
            )
            f.write(
                "- Candidate remainder majorant: "
                "`|remainder(x)| <= C_all * x^{-eta}` "
                f"with `eta={b['remainder_majorant_eta']:.6f}` and "
                f"`C_all={b['remainder_majorant_C_all']:.6e}` (finite range)\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md_path}")
    if overall_best is not None:
        b = overall_best["best"]
        print("overall_best_beta:", f"{overall_best['beta']:.6f}")
        print("overall_best_tau:", f"{b['tau']:.12g}")
        print("overall_best_tail_ratio_sup_to_amp:", f"{b['tail_ratio_sup_to_amp']:.6e}")
        print("overall_best_tail_decay_exponent_sup:", f"{b['tail_decay_exponent_sup']:.6f}")
        print("overall_interpretation:", b["interpretation"])


if __name__ == "__main__":
    main()
