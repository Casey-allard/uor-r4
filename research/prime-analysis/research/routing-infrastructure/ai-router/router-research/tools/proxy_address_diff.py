#!/usr/bin/env python3
import argparse
import json
import os
import sys
from typing import Any, Dict, Tuple

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr
from tasks import router_proxy_eval as task


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def build_args(common_args: Dict[str, Any], route_args: Dict[str, Any], seed: int) -> argparse.Namespace:
    old_argv = sys.argv[:]
    try:
        sys.argv = ["router_proxy_eval.py"]
        args = task.parse_args()
    finally:
        sys.argv = old_argv
    merged = dict(common_args)
    merged.update(route_args)
    merged["seed"] = seed
    merged["run_tag"] = ""
    for key, value in merged.items():
        setattr(args, key, value)
    task.apply_proxy_fast_dev(args)
    return args


def load_proxy_subset(args: argparse.Namespace) -> Tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    data = np.load(args.input)
    x_train = data["x_train"].astype(np.float64)
    y_train = data["y_train"].astype(np.float64)
    if args.eval_split == "val":
        x_eval = data["x_val"].astype(np.float64)
        y_eval = data["y_val"].astype(np.float64)
    else:
        x_eval = data["x_test"].astype(np.float64)
        y_eval = data["y_test"].astype(np.float64)
    v_tr, y_tr = task._subset(x_train, y_train, args.max_train, args.seed + 1)
    v_ev, y_ev = task._subset(x_eval, y_eval, args.max_eval, args.seed + 2)
    return v_tr, y_tr, v_ev, y_ev


def route_eval_addresses(args: argparse.Namespace) -> Tuple[np.ndarray, np.ndarray]:
    v_tr, y_tr, v_ev, _y_ev = load_proxy_subset(args)
    d = int(v_tr.shape[1])
    phase_dim_i, phase_dim_j = hr.parse_pair_dims(args.phase_dims, "--phase_dims")
    phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l = hr.parse_quad_dims(args.phase4_dims, "--phase4_dims")
    field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l = hr.parse_quad_dims(args.field4_dims, "--field4_dims")
    complex_dim_i, complex_dim_j = hr.parse_pair_dims(args.complex_dims, "--complex_dims")

    hr.ensure_dims_in_range([phase_dim_i, phase_dim_j], d, "--phase_dims")
    hr.ensure_dims_in_range([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], d, "--phase4_dims")
    hr.ensure_dims_in_range([field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l], d, "--field4_dims")
    hr.ensure_dims_in_range([complex_dim_i, complex_dim_j], d, "--complex_dims")
    hr.ensure_distinct_dims([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], "--phase4_dims")
    hr.ensure_distinct_dims([field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l], "--field4_dims")
    if args.sector_mode == "phase4d_hopf_product_phase":
        hr.ensure_disjoint_dims(
            [phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l],
            "--phase4_dims",
            [field4_dim_i, field4_dim_j, field4_dim_k, field4_dim_l],
            "--field4_dims",
        )

    c0 = None
    if args.sector_mode == "kmeans":
        u0 = hr.normalize_rows(v_tr)
        c0 = hr.spherical_kmeans(u0, K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

    chart = hr.Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)
    radial_rmax = float(args.radial_rmax)
    if learned_chart and args.learn_scale == 1 and args.scale_mode == "radial" and radial_rmax <= 0:
        radial_rmax = float(np.quantile(hr.safe_norm(v_tr, axis=1, keepdims=False), 0.995))
        radial_rmax = max(radial_rmax, 1e-6)

    if learned_chart:
        opt_res = hr.optimize_chart(
            v_train=v_tr,
            y_train=y_tr,
            delta_r=args.delta_r,
            C0=c0,
            learn_so8=args.learn_so8,
            learn_scale=args.learn_scale,
            scale_mode=args.scale_mode,
            radial_bins=args.radial_bins,
            radial_rmax=radial_rmax,
            radial_update_frac=args.radial_update_frac,
            radial_l2=args.radial_l2,
            iters=args.chart_iters,
            so8_step=args.so8_step,
            so8_candidates=args.so8_candidates,
            scale_step=args.scale_step,
            scale_candidates=args.scale_candidates,
            scale_clip=args.scale_clip,
            alpha_overload=args.chart_alpha,
            beta_bucketcount=args.chart_beta,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i,
            phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i,
            phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k,
            phase4_dim_l=phase4_dim_l,
            field4_dim_i=field4_dim_i,
            field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k,
            field4_dim_l=field4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            K=args.K,
            seed=args.seed + 23,
            early_stop_patience=args.early_stop_patience,
            early_stop_min_delta=args.early_stop_min_delta,
            adaptive_min_pair_bins=args.adaptive_min_pair_bins,
            adaptive_time_growth=args.adaptive_time_growth,
            adaptive_balance=args.adaptive_balance,
            adaptive_angle_growth=args.adaptive_angle_growth,
            adaptive_shell_growth=args.adaptive_shell_growth,
            adaptive_shell_balance=args.adaptive_shell_balance,
            adaptive_converge_lambda=args.adaptive_converge_lambda,
            adaptive_converge_target=args.adaptive_converge_target,
            adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
            adaptive_converge_mode=args.adaptive_converge_mode,
            fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            route_scale_lambda=args.route_scale_lambda,
            memory_coord_mode=args.memory_coord_mode,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            hopf_chi_bins=args.hopf_chi_bins,
            hopf_blend_lambda=args.hopf_blend_lambda,
            hopf_blend_chi_weight=args.hopf_blend_chi_weight,
            hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        chart = opt_res.chart

    c_used = c0
    if args.sector_mode == "kmeans" and learned_chart and args.recluster_after_chart == 1:
        z_tr0 = hr.apply_chart(v_tr, chart)
        u_tr_chart = hr.normalize_rows(z_tr0)
        c_used = hr.spherical_kmeans(u_tr_chart, K=args.K, iters=args.kmeans_iters, seed=args.seed + 111)

    shell_ev, sector_ev, _u, _z = hr.route_addresses(
        v_ev,
        delta_r=args.delta_r,
        C=c_used,
        chart=chart,
        sector_mode=args.sector_mode,
        phase_dim_i=phase_dim_i,
        phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i,
        phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k,
        phase4_dim_l=phase4_dim_l,
        field4_dim_i=field4_dim_i,
        field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k,
        field4_dim_l=field4_dim_l,
        complex_dim_i=complex_dim_i,
        complex_dim_j=complex_dim_j,
        K=args.K,
        time_pressure_lambda=0.0,
        tau=1.0,
        adaptive_min_pair_bins=args.adaptive_min_pair_bins,
        adaptive_time_growth=args.adaptive_time_growth,
        adaptive_balance=args.adaptive_balance,
        adaptive_angle_growth=args.adaptive_angle_growth,
        adaptive_shell_growth=args.adaptive_shell_growth,
        adaptive_shell_balance=args.adaptive_shell_balance,
        adaptive_converge_lambda=args.adaptive_converge_lambda,
        adaptive_converge_target=args.adaptive_converge_target,
        adaptive_converge_hysteresis=args.adaptive_converge_hysteresis,
        adaptive_converge_mode=args.adaptive_converge_mode,
        fib_rung_gate_threshold=args.fib_rung_gate_threshold,
        route_scale_lambda=args.route_scale_lambda,
        memory_coord_mode=args.memory_coord_mode,
        shell_mode=args.shell_mode,
        shell_phase_coupling=args.shell_phase_coupling,
        hopf_chi_bins=args.hopf_chi_bins,
        hopf_blend_lambda=args.hopf_blend_lambda,
        hopf_blend_chi_weight=args.hopf_blend_chi_weight,
        hopf_blend_shell_weight=args.hopf_blend_shell_weight,
        phase_transport_lambda=args.phase_transport_lambda,
        phase_field_lambda=args.phase_field_lambda,
        hybrid_local_k=args.hybrid_local_k,
        hybrid_complex_roots=args.hybrid_complex_roots,
        hybrid_local_min_k=args.hybrid_local_min_k,
        hybrid_local_target=args.hybrid_local_target,
        hybrid_local_hysteresis=args.hybrid_local_hysteresis,
        hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
    )
    return shell_ev.astype(np.int64), sector_ev.astype(np.int64)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--baseline-route", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    cfg = load_config(args.config)
    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    routes_by_id = {route["route_id"]: route.get("args", {}) for route in routes}
    if args.baseline_route not in routes_by_id:
        raise SystemExit(f"baseline route {args.baseline_route!r} not found in config")

    baseline_args = build_args(common_args, routes_by_id[args.baseline_route], args.seed)
    base_shell, base_sector = route_eval_addresses(baseline_args)
    base_keys = np.stack([base_shell, base_sector], axis=1)

    comparisons = []
    for route in routes:
        route_id = route["route_id"]
        route_args = build_args(common_args, route.get("args", {}), args.seed)
        shell_ev, sector_ev = route_eval_addresses(route_args)
        keys = np.stack([shell_ev, sector_ev], axis=1)
        shell_diff = int(np.count_nonzero(shell_ev != base_shell))
        sector_diff = int(np.count_nonzero(sector_ev != base_sector))
        key_diff = int(np.count_nonzero(np.any(keys != base_keys, axis=1)))
        comparisons.append(
            {
                "route_id": route_id,
                "shell_diff_count": shell_diff,
                "shell_diff_rate": float(shell_diff / max(len(shell_ev), 1)),
                "sector_diff_count": sector_diff,
                "sector_diff_rate": float(sector_diff / max(len(sector_ev), 1)),
                "key_diff_count": key_diff,
                "key_diff_rate": float(key_diff / max(len(keys), 1)),
            }
        )

    payload = {
        "config": args.config,
        "seed": int(args.seed),
        "baseline_route": args.baseline_route,
        "n_eval": int(base_shell.shape[0]),
        "comparisons": comparisons,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, sort_keys=True)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
