#!/usr/bin/env python3
"""Probe L1 tail representation for the R6 omitted-mode frontier.

Given a multimode probe artifact, reconstruct the normalized residual
`R(x)/x^beta`, then fit a finite decaying-mode representation on a tail window:

  R(x)/x^beta ~= sum_j kappa_j * x^(-eta) * sin(omega_j * log(x) + theta_j)

Numerical evidence only; not a formal proof term.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Any, Dict, List, Sequence

import numpy as np

from prime_geometry_loop import load_zeta_zeros_file


def _read_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _reconstruct_multimode_fit(
    x: np.ndarray,
    t: np.ndarray,
    beta: float,
    mode_rows: Sequence[Dict[str, Any]],
    decay_term_coeff: float | None,
) -> np.ndarray:
    yhat = np.zeros_like(x)
    for row in mode_rows:
        tau = float(row["tau"])
        a = float(row["a"])
        b = float(row["b"])
        yhat += a * np.cos(tau * t) + b * np.sin(tau * t)
    if decay_term_coeff is not None:
        yhat += float(decay_term_coeff) * np.power(x, -beta)
    return yhat


def _select_omegas(
    multimode_cfg: Dict[str, Any],
    selected_taus: Sequence[float],
    policy: str,
    omega_count: int,
) -> List[float]:
    selected = [float(v) for v in selected_taus]
    if policy == "selected":
        return selected[:omega_count]

    zeros_file = str(multimode_cfg.get("zeta_zeros_file", ""))
    if not zeros_file:
        if policy == "selected_plus_top":
            return selected[:omega_count]
        raise ValueError("zeta_zeros_file missing in multimode config")

    zeros = [float(z) for z in load_zeta_zeros_file(zeros_file) if float(z) > 0.0]
    zeros.sort()
    top = zeros[: max(omega_count, len(selected))]

    if policy == "top":
        return top[:omega_count]

    if policy == "selected_plus_top":
        merged: List[float] = []
        seen = set()
        for v in selected + top:
            if v in seen:
                continue
            seen.add(v)
            merged.append(v)
            if len(merged) >= omega_count:
                break
        return merged

    raise ValueError(f"unknown omega policy: {policy}")


def _fit_tail_representation(
    x_tail: np.ndarray,
    t_tail: np.ndarray,
    r_tail: np.ndarray,
    eta: float,
    omegas: Sequence[float],
    include_decay_column: bool,
    ridge_alpha: float,
) -> Dict[str, Any]:
    xpow = np.power(x_tail, -eta)
    cols: List[np.ndarray] = []
    for w in omegas:
        wf = float(w)
        cols.append(xpow * np.sin(wf * t_tail))
        cols.append(xpow * np.cos(wf * t_tail))
    if include_decay_column:
        cols.append(xpow)

    xmat = np.column_stack(cols)
    if ridge_alpha > 0.0:
        gram = xmat.T @ xmat
        rhs = xmat.T @ r_tail
        coef = np.linalg.solve(gram + ridge_alpha * np.eye(gram.shape[0]), rhs)
    else:
        coef, _, _, _ = np.linalg.lstsq(xmat, r_tail, rcond=None)
    r_hat = xmat @ coef
    eps = r_tail - r_hat

    mode_rows: List[Dict[str, Any]] = []
    for i, w in enumerate(omegas):
        p = float(coef[2 * i])
        q = float(coef[2 * i + 1])
        kappa = float(math.hypot(p, q))
        theta = float(math.atan2(q, p))
        mode_rows.append(
            {
                "omega": float(w),
                "p_sin_coeff": p,
                "q_cos_coeff": q,
                "kappa": kappa,
                "theta": theta,
            }
        )

    out: Dict[str, Any] = {
        "mode_rows": mode_rows,
        "tail_rmse_after": float(np.sqrt(np.mean(eps * eps))),
        "tail_sup_abs_after": float(np.max(np.abs(eps))),
        "tail_mean_abs_after": float(np.mean(np.abs(eps))),
    }
    if include_decay_column:
        out["decay_column_coeff"] = float(coef[-1])
    return out


def run_probe(
    multimode_path: str,
    eta: float | None,
    x1: float | None,
    omega_policy: str,
    omega_count: int,
    include_decay_column: bool,
    ridge_alpha: float,
) -> Dict[str, Any]:
    m = _read_json(multimode_path)
    cfg = m["config"]
    best = m["best_by_score"]

    cache_path = str(cfg["cache_path"])
    loaded = np.load(cache_path)
    x = loaded["x"].astype(np.float64)
    t = loaded["t"].astype(np.float64)
    y = loaded["y"].astype(np.float64)

    beta = float(cfg["beta"])
    mode_rows = list(best["mode_rows"])
    decay_term_coeff = best.get("decay_term_coeff")
    decay_term_coeff_val = float(decay_term_coeff) if decay_term_coeff is not None else None

    eta_val = float(eta) if eta is not None else float(best.get("remainder_majorant_eta", 0.0))
    if eta_val <= 0.0:
        raise ValueError("eta must be positive; pass --eta explicitly")

    x1_val = (
        float(x1)
        if x1 is not None
        else float(best.get("remainder_majorant_tail_start_x", np.min(x)))
    )
    mask_tail = x >= x1_val
    if not np.any(mask_tail):
        raise ValueError(f"no points in tail window for x1={x1_val}")

    yhat = _reconstruct_multimode_fit(
        x=x,
        t=t,
        beta=beta,
        mode_rows=mode_rows,
        decay_term_coeff=decay_term_coeff_val,
    )
    r = y - yhat

    x_tail = x[mask_tail]
    t_tail = t[mask_tail]
    r_tail = r[mask_tail]

    selected_taus = [float(row["tau"]) for row in mode_rows]
    omegas = _select_omegas(
        multimode_cfg=cfg,
        selected_taus=selected_taus,
        policy=omega_policy,
        omega_count=max(1, int(omega_count)),
    )

    fit = _fit_tail_representation(
        x_tail=x_tail,
        t_tail=t_tail,
        r_tail=r_tail,
        eta=eta_val,
        omegas=omegas,
        include_decay_column=include_decay_column,
        ridge_alpha=float(ridge_alpha),
    )

    # Evaluate fitted omitted-mode representation on full grid and tail grid.
    # Build x-matrices directly from fitted coefficients encoded in mode rows.
    xpow_all = np.power(x, -eta_val)
    cols_all: List[np.ndarray] = []
    cols_tail: List[np.ndarray] = []
    for row in fit["mode_rows"]:
        omega = float(row["omega"])
        cols_all.append(xpow_all * np.sin(omega * t))
        cols_all.append(xpow_all * np.cos(omega * t))
        cols_tail.append(np.power(x_tail, -eta_val) * np.sin(omega * t_tail))
        cols_tail.append(np.power(x_tail, -eta_val) * np.cos(omega * t_tail))
    if include_decay_column:
        cols_all.append(xpow_all)
        cols_tail.append(np.power(x_tail, -eta_val))

    xmat_all = np.column_stack(cols_all)
    xmat_tail = np.column_stack(cols_tail)
    coef_list: List[float] = []
    for row in fit["mode_rows"]:
        coef_list.extend([float(row["p_sin_coeff"]), float(row["q_cos_coeff"])])
    if include_decay_column:
        coef_list.append(float(fit.get("decay_column_coeff", 0.0)))
    coef = np.array(coef_list, dtype=np.float64)

    r_hat_all = xmat_all @ coef
    r_hat_tail = xmat_tail @ coef
    eps_all = r - r_hat_all
    eps_tail = r_tail - r_hat_tail

    sup_tail_before = float(np.max(np.abs(r_tail)))
    sup_tail_after = float(np.max(np.abs(eps_tail)))
    rmse_tail_before = float(np.sqrt(np.mean(r_tail * r_tail)))
    rmse_tail_after = float(np.sqrt(np.mean(eps_tail * eps_tail)))
    sup_all_before = float(np.max(np.abs(r)))
    sup_all_after = float(np.max(np.abs(eps_all)))

    # Power-majorant size of leftover after representation fit.
    c_eps_tail = float(np.max(np.abs(eps_tail) * np.power(x_tail, eta_val)))
    c_eps_all = float(np.max(np.abs(eps_all) * np.power(x, eta_val)))

    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source_files": {
            "multimode_probe": multimode_path,
            "cache_npz": cache_path,
        },
        "config": {
            "beta": beta,
            "eta": eta_val,
            "x1": x1_val,
            "tail_points": int(x_tail.size),
            "omega_policy": omega_policy,
            "omega_count": int(len(omegas)),
            "include_decay_column": bool(include_decay_column),
            "ridge_alpha": float(ridge_alpha),
        },
        "selected_omegas": [float(v) for v in omegas],
        "omitted_mode_fit": fit,
        "metrics": {
            "tail_sup_abs_before": sup_tail_before,
            "tail_sup_abs_after": sup_tail_after,
            "tail_sup_ratio_after_over_before": float(
                sup_tail_after / max(sup_tail_before, 1.0e-300)
            ),
            "tail_rmse_before": rmse_tail_before,
            "tail_rmse_after": rmse_tail_after,
            "tail_rmse_ratio_after_over_before": float(
                rmse_tail_after / max(rmse_tail_before, 1.0e-300)
            ),
            "all_sup_abs_before": sup_all_before,
            "all_sup_abs_after": sup_all_after,
            "all_sup_ratio_after_over_before": float(
                sup_all_after / max(sup_all_before, 1.0e-300)
            ),
            "eps_majorant_C_tail_with_eta": c_eps_tail,
            "eps_majorant_C_all_with_eta": c_eps_all,
        },
        "interpretation": {
            "note": (
                "If tail ratios are very small, finite omitted-mode representation is "
                "numerically plausible on the tested window; this remains evidence, not proof."
            )
        },
    }


def write_markdown(result: Dict[str, Any], out_md: str) -> None:
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# R6 Tail Representation Probe\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")

        cfg = result["config"]
        f.write("## Config\n")
        f.write(f"- beta: {cfg['beta']}\n")
        f.write(f"- eta: {cfg['eta']}\n")
        f.write(f"- x1: {cfg['x1']}\n")
        f.write(f"- tail_points: {cfg['tail_points']}\n")
        f.write(f"- omega_policy: {cfg['omega_policy']}\n")
        f.write(f"- omega_count: {cfg['omega_count']}\n")
        f.write(f"- include_decay_column: {cfg['include_decay_column']}\n\n")
        f.write(f"- ridge_alpha: {cfg['ridge_alpha']}\n\n")

        m = result["metrics"]
        f.write("## Fit Metrics\n")
        for k in [
            "tail_sup_abs_before",
            "tail_sup_abs_after",
            "tail_sup_ratio_after_over_before",
            "tail_rmse_before",
            "tail_rmse_after",
            "tail_rmse_ratio_after_over_before",
            "all_sup_abs_before",
            "all_sup_abs_after",
            "all_sup_ratio_after_over_before",
            "eps_majorant_C_tail_with_eta",
            "eps_majorant_C_all_with_eta",
        ]:
            f.write(f"- {k}: {m[k]}\n")
        f.write("\n")

        f.write("## Omitted Modes (Fitted)\n")
        for row in result["omitted_mode_fit"]["mode_rows"]:
            f.write(
                f"- omega={row['omega']:.12g}, kappa={row['kappa']:.6g}, "
                f"theta={row['theta']:.6g}\n"
            )
        if "decay_column_coeff" in result["omitted_mode_fit"]:
            f.write(
                f"- decay_column_coeff: {result['omitted_mode_fit']['decay_column_coeff']}\n"
            )
        f.write("\n")
        f.write("## Interpretation\n")
        f.write(f"- {result['interpretation']['note']}\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="R6 L1 tail representation probe")
    ap.add_argument(
        "--multimode",
        type=str,
        default="research/output/k1_multimode_phase_probe_2026-02-24_x1e22_m12_beta055_growth.json",
    )
    ap.add_argument("--eta", type=float, default=None)
    ap.add_argument("--x1", type=float, default=None)
    ap.add_argument(
        "--omega-policy",
        type=str,
        default="selected_plus_top",
        choices=["selected", "top", "selected_plus_top"],
    )
    ap.add_argument("--omega-count", type=int, default=24)
    ap.add_argument("--ridge-alpha", type=float, default=0.0)
    ap.add_argument(
        "--include-decay-column",
        dest="include_decay_column",
        action="store_true",
    )
    ap.add_argument(
        "--no-decay-column",
        dest="include_decay_column",
        action="store_false",
    )
    ap.set_defaults(include_decay_column=True)
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/r6_tail_representation_probe_2026-02-24_x1e22_m12_beta055.json",
    )
    args = ap.parse_args()

    result = run_probe(
        multimode_path=str(args.multimode),
        eta=args.eta,
        x1=args.x1,
        omega_policy=str(args.omega_policy),
        omega_count=int(args.omega_count),
        include_decay_column=bool(args.include_decay_column),
        ridge_alpha=float(args.ridge_alpha),
    )

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    out_md = args.output.replace(".json", ".md")
    write_markdown(result, out_md)
    print(f"wrote: {args.output}")
    print(f"wrote: {out_md}")
    print("metrics:", json.dumps(result["metrics"], indent=2))


if __name__ == "__main__":
    main()
