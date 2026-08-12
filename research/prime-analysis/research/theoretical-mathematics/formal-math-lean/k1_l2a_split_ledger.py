#!/usr/bin/env python3
"""Build L2A split-ledger for omitted-tail decomposition at fixed N.

Numerical research artifact (not a formal proof):
E_N(x) = E_{N,<=T(x)}(x) + E_{>T(x)}(x), with T(x) policy-based cutoff.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, Tuple

import numpy as np

from k1_source_shape_probe import majorant_profile
from prime_geometry_loop import load_zeta_zeros_file


def omega_vk(x: np.ndarray, a0: float) -> np.ndarray:
    lx = np.log(np.maximum(x, 1.0 + 1.0e-300))
    llx = np.log(np.maximum(lx, 1.0 + 1.0e-300))
    c = ((5.0**6 * a0**3) / (2.0**2 * 3.0**4)) ** (1.0 / 5.0)
    return c * np.power(lx, 3.0 / 5.0) / np.power(llx, 1.0 / 5.0)


def fks_bound(x: np.ndarray) -> np.ndarray:
    lx = np.log(np.maximum(x, 1.0 + 1.0e-300))
    return 9.22022 * np.power(lx, 1.5) * np.exp(-0.8476836 * np.sqrt(lx))


def bellotti_2025_bound(x: np.ndarray, a0: float) -> np.ndarray:
    om = omega_vk(x, a0)
    return math.exp(55.0 * a0) * np.exp(-om)


def cutoff_from_policy(
    x: np.ndarray,
    gamma_n: float,
    policy: str,
    a0: float,
    x_pow: float,
) -> Tuple[np.ndarray, np.ndarray]:
    if policy == "omega":
        om = omega_vk(x, a0)
        t_raw = np.exp(2.0 * om)
    elif policy == "xpow":
        om = omega_vk(x, a0)
        t_raw = np.power(x, x_pow)
    else:
        raise ValueError(f"unsupported cutoff policy: {policy}")
    t_cut = np.maximum(t_raw, gamma_n)
    return t_raw, t_cut


def split_signals(
    zeros: np.ndarray,
    n_head: int,
    beta: float,
    x: np.ndarray,
    t: np.ndarray,
    k_cut: np.ndarray,
    zero_chunk: int,
) -> Dict[str, np.ndarray]:
    # Omitted list starts at n_head (0-based index).
    g_all = zeros[n_head:]
    if g_all.size == 0:
        return {
            "e_total": np.zeros_like(x),
            "e_band": np.zeros_like(x),
            "e_high": np.zeros_like(x),
        }

    k_omit = np.maximum(0, k_cut - n_head)
    k_omit = np.minimum(k_omit, g_all.size)

    scale = np.power(x, 0.5 - beta)
    den = 0.25 + g_all * g_all
    c_coef = 1.0 / den
    s_coef = (2.0 * g_all) / den

    e_total = np.zeros_like(x)
    e_band = np.zeros_like(x)

    for start in range(0, g_all.size, zero_chunk):
        end = min(g_all.size, start + zero_chunk)
        gj = g_all[start:end]
        cj = c_coef[start:end]
        sj = s_coef[start:end]

        phase = np.outer(t, gj)
        mixed = -scale[:, None] * (np.cos(phase) * cj + np.sin(phase) * sj)
        e_total += np.sum(mixed, axis=1)

        take = np.clip(k_omit - start, 0, end - start).astype(np.int64)
        idx = np.nonzero(take > 0)[0]
        if idx.size > 0:
            pref = np.cumsum(mixed, axis=1)
            e_band[idx] += pref[idx, take[idx] - 1]

    return {
        "e_total": e_total,
        "e_band": e_band,
        "e_high": e_total - e_band,
    }


def majorant_row(x: np.ndarray, y: np.ndarray, eta_target: float) -> Dict[str, float]:
    maj = majorant_profile(x=x, r=y, tail_frac=1.0)
    c_target = float(np.max(np.abs(y) * np.power(x, eta_target)))
    return {
        "sup_abs": float(np.max(np.abs(y))),
        "rmse": float(np.sqrt(np.mean(np.square(y)))),
        "eta_fit": float(maj["eta"]),
        "c_fit": float(maj["C_all"]),
        "c_at_eta_target": c_target,
    }


def solve_x_for_omega_threshold(target_omega: float, a0: float) -> float:
    lo = 1.0e6
    hi = 1.0e200
    for _ in range(200):
        mid = math.sqrt(lo * hi)
        om = float(omega_vk(np.array([mid], dtype=np.float64), a0)[0])
        if om < target_omega:
            lo = mid
        else:
            hi = mid
    return hi


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 L2A split ledger")
    ap.add_argument(
        "--zeta-zeros-file",
        type=str,
        default="research/data/zeta_zeros_odlyzko_100k.json",
    )
    ap.add_argument("--max-zeta-zeros", type=int, default=100000)
    ap.add_argument("--n-head", type=int, default=256)
    ap.add_argument("--beta", type=float, default=0.55)
    ap.add_argument("--x-min", type=float, default=1.0e21)
    ap.add_argument("--x-max", type=float, default=1.0e30)
    ap.add_argument("--grid-size", type=int, default=1024)
    ap.add_argument("--zero-chunk", type=int, default=512)
    ap.add_argument("--eta-target", type=float, default=0.01)
    ap.add_argument("--a0", type=float, default=(1.0 / 48.0718))
    ap.add_argument(
        "--cutoff-policy",
        type=str,
        choices=["omega", "xpow"],
        default="omega",
    )
    ap.add_argument("--xpow", type=float, default=0.5)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w31_l2a_split_ledger_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    zeros_raw = [float(z) for z in load_zeta_zeros_file(args.zeta_zeros_file)]
    zeros = np.array([z for z in zeros_raw if z > 0.0], dtype=np.float64)
    zeros.sort()
    if args.max_zeta_zeros > 0:
        zeros = zeros[: args.max_zeta_zeros]
    if zeros.size <= args.n_head:
        raise ValueError("need max_zeta_zeros > n_head")

    x = np.exp(
        np.linspace(math.log(args.x_min), math.log(args.x_max), args.grid_size, dtype=np.float64)
    )
    t = np.log(x)

    gamma_n = float(zeros[args.n_head - 1])
    t_raw, t_cut = cutoff_from_policy(
        x=x,
        gamma_n=gamma_n,
        policy=args.cutoff_policy,
        a0=args.a0,
        x_pow=args.xpow,
    )
    k_cut = np.searchsorted(zeros, t_cut, side="right")
    band_count = np.maximum(0, k_cut - args.n_head)

    sig = split_signals(
        zeros=zeros,
        n_head=args.n_head,
        beta=args.beta,
        x=x,
        t=t,
        k_cut=k_cut,
        zero_chunk=args.zero_chunk,
    )

    e_total = sig["e_total"]
    e_band = sig["e_band"]
    e_high = sig["e_high"]

    row_total = majorant_row(x=x, y=e_total, eta_target=args.eta_target)
    row_band = majorant_row(x=x, y=e_band, eta_target=args.eta_target)
    row_high = majorant_row(x=x, y=e_high, eta_target=args.eta_target)

    fks = fks_bound(x)
    bel = bellotti_2025_bound(x, args.a0)
    c_fks = float(np.max(fks * np.power(x, args.eta_target)))
    c_bel = float(np.max(bel * np.power(x, args.eta_target)))

    omega_threshold = 0.5 * math.log(gamma_n)
    x_cross = solve_x_for_omega_threshold(omega_threshold, args.a0)

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": int(zeros.size),
            "n_head": int(args.n_head),
            "beta": float(args.beta),
            "x_min": float(args.x_min),
            "x_max": float(args.x_max),
            "grid_size": int(args.grid_size),
            "zero_chunk": int(args.zero_chunk),
            "eta_target": float(args.eta_target),
            "a0": float(args.a0),
            "cutoff_policy": args.cutoff_policy,
            "xpow": float(args.xpow),
        },
        "locked_values": {
            "gamma_n": gamma_n,
            "omega_threshold_for_tcut_eq_gamma_n": float(omega_threshold),
            "x_where_exp2omega_hits_gamma_n": float(x_cross),
        },
        "cutoff_summary": {
            "t_raw_min": float(np.min(t_raw)),
            "t_raw_max": float(np.max(t_raw)),
            "t_cut_min": float(np.min(t_cut)),
            "t_cut_max": float(np.max(t_cut)),
            "k_cut_min": int(np.min(k_cut)),
            "k_cut_median": int(np.median(k_cut)),
            "k_cut_max": int(np.max(k_cut)),
            "band_count_min": int(np.min(band_count)),
            "band_count_median": int(np.median(band_count)),
            "band_count_max": int(np.max(band_count)),
            "band_count_nonzero_fraction": float(np.mean(band_count > 0)),
        },
        "component_majorants": {
            "e_total": row_total,
            "e_band": row_band,
            "e_high": row_high,
            "eta_target_pass_empirical": bool(
                row_total["eta_fit"] >= args.eta_target and row_high["eta_fit"] >= args.eta_target
            ),
        },
        "external_envelopes_eta_target": {
            "c_fks": c_fks,
            "c_bellotti_2025_model": c_bel,
        },
        "interpretation": {
            "note": (
                "Split-ledger uses finite 100k-zero surrogate plus external explicit envelopes; "
                "it diagnoses where cutoff choice blocks or enables L2A constant propagation."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")

    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines = []
    lines.append(f"# K1 L2A Split Ledger ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Config")
    for k, v in result["config"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Locked Values")
    for k, v in result["locked_values"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Cutoff Summary")
    for k, v in result["cutoff_summary"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Component Majorants")
    for key in ["e_total", "e_band", "e_high"]:
        row = result["component_majorants"][key]
        lines.append(
            f"- {key}: sup={row['sup_abs']:.6e}, rmse={row['rmse']:.6e}, "
            f"eta_fit={row['eta_fit']:.6f}, C_fit={row['c_fit']:.6e}, "
            f"C_eta{args.eta_target:g}={row['c_at_eta_target']:.6e}"
        )
    lines.append(
        f"- eta_target_pass_empirical: {result['component_majorants']['eta_target_pass_empirical']}"
    )
    lines.append("")
    lines.append("## External Envelopes At Eta Target")
    env = result["external_envelopes_eta_target"]
    lines.append(f"- c_fks: {env['c_fks']:.6e}")
    lines.append(f"- c_bellotti_2025_model: {env['c_bellotti_2025_model']:.6e}")
    lines.append("")
    lines.append("## Interpretation")
    lines.append(f"- {result['interpretation']['note']}")
    lines.append("")

    out_md.write_text("\n".join(lines), encoding="utf-8")
    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()

