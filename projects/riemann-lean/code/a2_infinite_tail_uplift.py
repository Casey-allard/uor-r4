#!/usr/bin/env python3
"""A2 uplift: replace finite-M_ref tail with analytic infinite-tail majorant."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


def zeros_sig(zeros: Sequence[float]) -> str:
    txt = ",".join(f"{z:.8f}" for z in zeros)
    return hashlib.sha1(txt.encode("utf-8")).hexdigest()[:16]


def cache_key(payload: Dict[str, object]) -> str:
    blob = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha1(blob.encode("utf-8")).hexdigest()


def load_json(path: str):
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def save_json(path: str, payload) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2)


def tau_tail_fp_finite(zeros_ref: Sequence[float], m: int, kernel: str, scale: float) -> float:
    s = 0.0
    for g in zeros_ref[m:]:
        w = s4.zero_weight(g, kernel, scale)
        den = math.sqrt(0.25 + g * g)
        s += w * (g / den)
    return s


def nprime_upper_affine(t: float, a_n: float, b_n: float) -> float:
    # Legacy smooth majorant (empirical-style).
    return max(0.0, a_n * math.log(max(3.0, t)) + b_n)


def nprime_upper_rvm_explicit(t: float, c0: float, c1: float) -> float:
    # Explicit-formula-inspired derivative envelope:
    # N'(t) ~= (1/(2*pi))*log(t/(2*pi)) + O(1/t).
    tt = max(3.0, t)
    lead = (1.0 / (2.0 * math.pi)) * math.log(max(1.0, tt / (2.0 * math.pi)))
    corr = c0 + (c1 / tt)
    return max(0.0, lead + corr)


def n_main_rvm(t: float) -> float:
    tt = max(3.0, t)
    return (tt / (2.0 * math.pi)) * math.log(max(1.0, tt / (2.0 * math.pi * math.e)))


def n_err_explicit(t: float, c1: float, c2: float, c3: float) -> float:
    tt = max(math.e, t)
    return (c1 * math.log(tt)) + (c2 * math.log(max(1.0, math.log(tt)))) + c3


def nprime_upper_from_n_bound_diff(
    t: float,
    h: float,
    c1: float,
    c2: float,
    c3: float,
) -> float:
    # If |N(T)-M(T)|<=E(T), then
    # N(T+h)-N(T) <= (M(T+h)-M(T)) + E(T+h)+E(T).
    hh = max(1e-6, h)
    t0 = max(math.e, t)
    t1 = t0 + hh
    m_inc = n_main_rvm(t1) - n_main_rvm(t0)
    e_up = n_err_explicit(t1, c1, c2, c3) + n_err_explicit(t0, c1, c2, c3)
    return max(0.0, (m_inc + e_up) / hh)


def kernel_weight(t: float, kernel: str, scale: float) -> float:
    if kernel == "none" or scale <= 0:
        return 1.0
    x = t / scale
    if kernel == "gaussian":
        return math.exp(-(x * x))
    if kernel == "lorentz":
        return 1.0 / (1.0 + x * x)
    return 1.0


def tail_integral_majorant(
    t0: float,
    kernel: str,
    scale: float,
    density_model: str,
    a_n: float,
    b_n: float,
    rvm_c0: float,
    rvm_c1: float,
    nbound_c1: float,
    nbound_c2: float,
    nbound_c3: float,
    nbound_h: float,
    steps: int = 4000,
) -> float:
    if kernel == "none" or scale <= 0:
        return float("inf")
    if kernel == "gaussian":
        t1 = t0 + max(50.0 * scale, 2000.0)
    else:
        # Lorentz decays slower; integrate farther.
        t1 = t0 + max(500.0 * scale, 50000.0)
    if t1 <= t0:
        return 0.0

    h = (t1 - t0) / max(1, steps)
    acc = 0.0
    for i in range(steps + 1):
        t = t0 + i * h
        if density_model == "rvm_explicit":
            nup = nprime_upper_rvm_explicit(t, c0=rvm_c0, c1=rvm_c1)
        elif density_model == "n_diff_explicit":
            nup = nprime_upper_from_n_bound_diff(
                t=t,
                h=nbound_h,
                c1=nbound_c1,
                c2=nbound_c2,
                c3=nbound_c3,
            )
        else:
            nup = nprime_upper_affine(t, a_n=a_n, b_n=b_n)
        # t/sqrt(1/4+t^2) <= 1, but keep exact factor for tighter majorant.
        fp = t / math.sqrt(0.25 + t * t)
        f = nup * fp * kernel_weight(t, kernel, scale)
        c = 1.0
        if i == 0 or i == steps:
            c = 0.5
        acc += c * f
    return acc * h


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 infinite-tail analytic uplift")
    ap.add_argument(
        "--a2-pack-json",
        type=str,
        default="research/output/a2_theorem_constant_pack_refresh_2026-02-17_sf3p5.json",
    )
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--m-ref", type=int, default=512)
    ap.add_argument("--safety-factor", type=float, default=1.2)
    ap.add_argument("--density-model", type=str, default="affine", choices=["affine", "rvm_explicit", "n_diff_explicit"])
    ap.add_argument("--density-a", type=float, default=(1.0 / (2.0 * math.pi)))
    ap.add_argument("--density-b", type=float, default=0.5)
    ap.add_argument("--rvm-c0", type=float, default=0.5)
    ap.add_argument("--rvm-c1", type=float, default=1.0)
    ap.add_argument("--nbound-c1", type=float, default=0.1038, help="N(T) explicit error C1 (log T)")
    ap.add_argument("--nbound-c2", type=float, default=0.2573, help="N(T) explicit error C2 (log log T)")
    ap.add_argument("--nbound-c3", type=float, default=9.3675, help="N(T) explicit error C3")
    ap.add_argument("--nbound-h", type=float, default=1.0, help="finite-difference step h for N'(t) upper model")
    ap.add_argument("--integral-steps", type=int, default=4000)
    ap.add_argument("--cache-dir", type=str, default="research/cache/a2_infinite_tail_uplift")
    ap.add_argument("--no-cache", action="store_true")
    ap.add_argument("--output", type=str, default="research/output/a2_infinite_tail_uplift.json")
    args = ap.parse_args()

    pack = load_json(args.a2_pack_json)
    cfg = pack.get("config", {})
    kernel = cfg.get("zero_kernel", "gaussian")
    scale = float(cfg.get("kernel_scale", 100.0))
    beta = float(cfg.get("beta_fixed", 2.6))
    m_grid = [int(m) for m in cfg.get("m_grid", [64, 128, 192, 256])]
    train_n = set(int(v) for v in cfg.get("train_n_values", []))

    zeros_all = load_zeta_zeros_file(args.zeta_zeros_file)
    if args.m_ref > len(zeros_all):
        raise ValueError("m-ref exceeds available zeros")
    zeros_ref = zeros_all[: args.m_ref]
    gamma_ref = zeros_ref[-1]

    cache_payload = {
        "version": 1,
        "kernel": kernel,
        "scale": scale,
        "m_ref": args.m_ref,
        "m_grid": m_grid,
        "density_model": args.density_model,
        "density_a": args.density_a,
        "density_b": args.density_b,
        "rvm_c0": args.rvm_c0,
        "rvm_c1": args.rvm_c1,
        "nbound_c1": args.nbound_c1,
        "nbound_c2": args.nbound_c2,
        "nbound_c3": args.nbound_c3,
        "nbound_h": args.nbound_h,
        "integral_steps": args.integral_steps,
        "zsig": zeros_sig(zeros_ref),
    }
    ckey = cache_key(cache_payload)
    cpath = os.path.join(args.cache_dir, f"{ckey}.json")
    use_cache = not args.no_cache

    tail_info = None
    if use_cache and os.path.exists(cpath):
        tail_info = load_json(cpath)
    else:
        by_m = {}
        for m in m_grid:
            tau_fin = tau_tail_fp_finite(zeros_ref, m, kernel, scale)
            tau_inf_extra = tail_integral_majorant(
                t0=gamma_ref,
                kernel=kernel,
                scale=scale,
                density_model=args.density_model,
                a_n=args.density_a,
                b_n=args.density_b,
                rvm_c0=args.rvm_c0,
                rvm_c1=args.rvm_c1,
                nbound_c1=args.nbound_c1,
                nbound_c2=args.nbound_c2,
                nbound_c3=args.nbound_c3,
                nbound_h=args.nbound_h,
                steps=max(200, args.integral_steps),
            )
            tau_inf = tau_fin + tau_inf_extra
            by_m[str(m)] = {
                "tau_finite_ref": tau_fin,
                "tau_infinite_extra_majorant": tau_inf_extra,
                "tau_infinite_majorant": tau_inf,
            }
        tail_info = {
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
            "config": cache_payload,
            "gamma_ref": gamma_ref,
            "tau_by_M": by_m,
        }
        if use_cache:
            save_json(cpath, tail_info)

    rows = pack.get("rows", [])
    if not rows:
        raise ValueError("A2 pack has no rows; cannot uplift")

    # Train-sup constant using tau_infinite_majorant.
    c_train = 0.0
    for r in rows:
        m = str(int(r["M"]))
        n = int(r["n_max"])
        tau_inf = float(tail_info["tau_by_M"][m]["tau_infinite_majorant"])
        rhs_unit = (math.log(max(3, n)) ** beta) * max(1e-30, tau_inf)
        ratio = float(r["delta"]) / max(1e-30, rhs_unit)
        if n in train_n:
            c_train = max(c_train, ratio)
    c_uplift = max(1e-30, c_train) * max(1.0, args.safety_factor)

    train_viol = 0
    valid_viol = 0
    train_max_gap = -1e18
    valid_max_gap = -1e18
    train_rat = []
    valid_rat = []
    for r in rows:
        m = str(int(r["M"]))
        n = int(r["n_max"])
        tau_inf = float(tail_info["tau_by_M"][m]["tau_infinite_majorant"])
        rhs = c_uplift * (math.log(max(3, n)) ** beta) * max(1e-30, tau_inf)
        gap = float(r["delta"]) - rhs
        rr = float(r["delta"]) / max(1e-30, rhs)
        if n in train_n:
            train_max_gap = max(train_max_gap, gap)
            train_rat.append(rr)
            if gap > 1e-12:
                train_viol += 1
        else:
            valid_max_gap = max(valid_max_gap, gap)
            valid_rat.append(rr)
            if gap > 1e-12:
                valid_viol += 1

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "input_pack": args.a2_pack_json,
        "cache": {
            "enabled": use_cache,
            "path": cpath,
            "hit": bool(use_cache and os.path.exists(cpath)),
        },
        "config": {
            "kernel": kernel,
            "kernel_scale": scale,
            "beta_fixed": beta,
            "m_ref": args.m_ref,
            "m_grid": m_grid,
            "density_a": args.density_a,
            "density_b": args.density_b,
            "density_model": args.density_model,
            "rvm_c0": args.rvm_c0,
            "rvm_c1": args.rvm_c1,
            "nbound_c1": args.nbound_c1,
            "nbound_c2": args.nbound_c2,
            "nbound_c3": args.nbound_c3,
            "nbound_h": args.nbound_h,
            "integral_steps": args.integral_steps,
            "safety_factor": max(1.0, args.safety_factor),
        },
        "tail_majorant": tail_info,
        "uplift_constant": {
            "C_delta_train_sup_ratio_raw": c_train,
            "C_delta_uplifted": c_uplift,
        },
        "checks": {
            "train": {
                "holds": train_viol == 0 and train_max_gap <= 1e-12,
                "violations": int(train_viol),
                "max_gap_delta_minus_rhs": float(train_max_gap),
                "ratio_min": min(train_rat) if train_rat else 0.0,
                "ratio_max": max(train_rat) if train_rat else 0.0,
            },
            "valid": {
                "holds": valid_viol == 0 and valid_max_gap <= 1e-12,
                "violations": int(valid_viol),
                "max_gap_delta_minus_rhs": float(valid_max_gap),
                "ratio_min": min(valid_rat) if valid_rat else 0.0,
                "ratio_max": max(valid_rat) if valid_rat else 0.0,
            },
        },
    }

    save_json(args.output, report)
    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        u = report["uplift_constant"]
        ch = report["checks"]
        f.write("# A2 Infinite-Tail Uplift\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- kernel: {kernel}, scale={scale}\n")
        f.write(f"- beta (fixed): {beta}\n")
        f.write(f"- C_delta_uplifted: {u['C_delta_uplifted']:.12e}\n")
        f.write(f"- train holds: {ch['train']['holds']} (viol={ch['train']['violations']})\n")
        f.write(f"- valid holds: {ch['valid']['holds']} (viol={ch['valid']['violations']})\n")
        f.write(f"- valid max gap (delta-rhs): {ch['valid']['max_gap_delta_minus_rhs']:.6e}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()
