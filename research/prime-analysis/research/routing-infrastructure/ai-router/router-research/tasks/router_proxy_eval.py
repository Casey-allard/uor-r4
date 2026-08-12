#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr


def _subset(X: np.ndarray, Y: np.ndarray, max_n: int, seed: int):
    if max_n <= 0 or X.shape[0] <= max_n:
        return X, Y
    rs = np.random.RandomState(seed)
    idx = rs.permutation(X.shape[0])[:max_n]
    return X[idx], Y[idx]


def apply_proxy_fast_dev(args: argparse.Namespace):
    if int(args.fast_dev) != 1:
        return
    args.max_train = min(int(args.max_train), 2500)
    args.max_eval = min(int(args.max_eval), 1200)
    args.epochs = min(int(args.epochs), 1)
    args.chart_iters = min(int(args.chart_iters), 120)
    args.kmeans_iters = min(int(args.kmeans_iters), 12)
    args.so8_candidates = min(int(args.so8_candidates), 8)
    args.scale_candidates = min(int(args.scale_candidates), 8)
    args.split_rounds = min(int(args.split_rounds), 40)
    args.extra_budget = min(int(args.extra_budget), 32)


def event_gate_value_from_error(
    error_mag: float,
    mode: str,
    threshold: float,
    tau: float,
) -> float:
    if mode == "off":
        return 1.0
    if mode == "hard_error":
        return 1.0 if float(error_mag) >= float(threshold) else 0.0
    if mode != "soft_error":
        raise ValueError(f"unsupported event gate mode: {mode}")
    gate_tau = max(float(tau), 1e-9)
    logit = np.clip((float(error_mag) - float(threshold)) / gate_tau, -60.0, 60.0)
    return float(1.0 / (1.0 + np.exp(-logit)))


def event_gate_stats(
    yhat: np.ndarray,
    y: np.ndarray,
    mode: str,
    threshold: float,
    tau: float,
):
    err = np.asarray(yhat, dtype=np.float64) - np.asarray(y, dtype=np.float64)
    error_mag = float(np.sqrt(np.mean(err * err)))
    gate = event_gate_value_from_error(
        error_mag=error_mag,
        mode=mode,
        threshold=threshold,
        tau=tau,
    )
    active = 1.0 if gate >= 0.5 else 0.0
    return error_mag, gate, active


def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--input_transform", type=str, default="none", choices=["none", "col_perm", "gaussian"])
    ap.add_argument("--eval_split", type=str, default="test", choices=["test", "val"])
    ap.add_argument("--max_train", type=int, default=12000)
    ap.add_argument("--max_eval", type=int, default=6000)

    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--K", type=int, default=8)
    ap.add_argument("--delta_r", type=float, default=3.0)
    ap.add_argument("--kmeans_iters", type=int, default=25)

    ap.add_argument("--epochs", type=int, default=2)
    ap.add_argument("--eta_p", type=float, default=0.04)
    ap.add_argument("--eta_m", type=float, default=0.08)
    ap.add_argument("--extra_budget", type=int, default=64)
    ap.add_argument("--max_slots_per_bucket", type=int, default=4)
    ap.add_argument("--split_rounds", type=int, default=128)
    ap.add_argument("--min_split_gain", type=float, default=1e-4)

    ap.add_argument("--sector_mode", type=str, default="kmeans", choices=["kmeans", "phase2", "phase4d", "phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local", "complex2"])
    ap.add_argument("--phase_dims", type=str, default="0,1")
    ap.add_argument("--phase4_dims", type=str, default="0,2,4,6")
    ap.add_argument("--field4_dims", type=str, default="1,3,5,7")
    ap.add_argument("--complex_dims", type=str, default="0,1")
    ap.add_argument("--hybrid_local_k", type=int, default=4)
    ap.add_argument("--hybrid_complex_roots", type=int, default=4)
    ap.add_argument("--hybrid_local_min_k", type=int, default=1)
    ap.add_argument("--hybrid_local_target", type=float, default=0.60)
    ap.add_argument("--hybrid_local_hysteresis", type=float, default=0.05)
    ap.add_argument("--hybrid_local_converge_lambda", type=float, default=1.0)
    ap.add_argument("--adaptive_min_pair_bins", type=int, default=2)
    ap.add_argument("--adaptive_time_growth", type=float, default=1.0)
    ap.add_argument("--adaptive_balance", type=float, default=1.0)
    ap.add_argument("--adaptive_angle_growth", type=float, default=0.35)
    ap.add_argument("--adaptive_shell_growth", type=float, default=0.0)
    ap.add_argument("--adaptive_shell_balance", type=float, default=0.0)
    ap.add_argument("--adaptive_converge_lambda", type=float, default=0.0)
    ap.add_argument("--adaptive_converge_target", type=float, default=1.0)
    ap.add_argument("--adaptive_converge_hysteresis", type=float, default=0.1)
    ap.add_argument("--adaptive_converge_mode", type=str, default="fixed", choices=["fixed", "phi_ratio", "phi_ladder"])
    ap.add_argument("--shell_mode", type=str, default="linear", choices=["linear", "phi_log", "phi_phase", "h4_mass", "h4_mass_phi"])
    ap.add_argument("--shell_phase_coupling", type=float, default=0.0)
    ap.add_argument("--product_shell_control_mode", type=str, default="continuous", choices=["continuous", "gated", "banded"])
    ap.add_argument("--product_shell_gate_threshold", type=float, default=0.0)
    ap.add_argument("--fib_rung_gate_threshold", type=float, default=0.0)
    ap.add_argument("--route_scale_lambda", type=float, default=1.0)
    ap.add_argument("--memory_coord_mode", type=str, default="route_chart", choices=["route_chart", "full_chart"])
    ap.add_argument("--hopf_chi_bins", type=int, default=2)
    ap.add_argument("--hopf_blend_lambda", type=float, default=0.8)
    ap.add_argument("--hopf_blend_chi_weight", type=float, default=1.0)
    ap.add_argument("--hopf_blend_shell_weight", type=float, default=0.5)
    ap.add_argument("--shell_pressure_w", type=float, default=0.0,
                    help="INC-0137: blend weight in [0,1] between chart radius and geodesic radius "
                         "for shell bucket assignment (only active for sector_mode=phase4d_hopf_base). "
                         "0=pure chart (default), 1=pure geodesic (= failed INC-0136 approach).")
    ap.add_argument("--phase_transport_lambda", type=float, default=1.0)
    ap.add_argument("--phase_field_lambda", type=float, default=0.0)
    ap.add_argument("--time_pressure_lambda", type=float, default=0.0)
    ap.add_argument("--train_route_mode", type=str, default="dynamic", choices=["dynamic", "final_static"])
    ap.add_argument("--event_gate_mode", type=str, default="off", choices=["off", "soft_error", "hard_error"])
    ap.add_argument("--event_gate_threshold", type=float, default=0.0)
    ap.add_argument("--event_gate_tau", type=float, default=0.01)

    ap.add_argument("--learn_so8", type=int, default=0, choices=[0, 1])
    ap.add_argument("--learn_scale", type=int, default=1, choices=[0, 1])
    ap.add_argument("--scale_mode", type=str, default="radial", choices=["global", "radial"])
    ap.add_argument("--radial_bins", type=int, default=10)
    ap.add_argument("--radial_rmax", type=float, default=0.0)
    ap.add_argument("--radial_update_frac", type=float, default=0.25)
    ap.add_argument("--radial_l2", type=float, default=0.0)

    ap.add_argument("--chart_iters", type=int, default=300)
    ap.add_argument("--chart_alpha", type=float, default=0.01)
    ap.add_argument("--chart_beta", type=float, default=50.0)
    ap.add_argument("--so8_step", type=float, default=0.10)
    ap.add_argument("--so8_candidates", type=int, default=20)
    ap.add_argument("--scale_step", type=float, default=0.08)
    ap.add_argument("--scale_candidates", type=int, default=16)
    ap.add_argument("--scale_clip", type=float, default=2.0)
    ap.add_argument("--recluster_after_chart", type=int, default=1, choices=[0, 1])

    ap.add_argument("--fast_dev", type=int, default=0, choices=[0, 1])
    ap.add_argument("--early_stop_patience", type=int, default=0)
    ap.add_argument("--early_stop_min_delta", type=float, default=0.0)
    ap.add_argument("--cache_dir", type=str, default="results/cache")
    ap.add_argument("--cache_chart", type=int, default=0, choices=[0, 1])
    ap.add_argument("--cache_routes", type=int, default=0, choices=[0, 1])
    ap.add_argument("--run_tag", type=str, default="")

    return ap.parse_args()


def proxy_chart_cache_payload(args: argparse.Namespace, d: int, dy: int, n_train: int):
    return {
        "cache_version": "proxy_chart_v12",
        "input": args.input,
        "eval_split": args.eval_split,
        "d": d,
        "dy": dy,
        "n_train": n_train,
        "args": {
            "seed": args.seed,
            "K": args.K,
            "delta_r": args.delta_r,
            "kmeans_iters": args.kmeans_iters,
            "sector_mode": args.sector_mode,
            "phase_dims": args.phase_dims,
            "phase4_dims": args.phase4_dims,
            "field4_dims": args.field4_dims,
            "complex_dims": args.complex_dims,
            "route_scale_lambda": args.route_scale_lambda,
            "memory_coord_mode": args.memory_coord_mode,
            "shell_mode": args.shell_mode,
            "shell_phase_coupling": args.shell_phase_coupling,
            "product_shell_control_mode": args.product_shell_control_mode,
            "product_shell_gate_threshold": args.product_shell_gate_threshold,
            "fib_rung_gate_threshold": args.fib_rung_gate_threshold,
            "hopf_chi_bins": args.hopf_chi_bins,
            "hopf_blend_lambda": args.hopf_blend_lambda,
            "hopf_blend_chi_weight": args.hopf_blend_chi_weight,
            "hopf_blend_shell_weight": args.hopf_blend_shell_weight,
            "phase_transport_lambda": args.phase_transport_lambda,
            "phase_field_lambda": args.phase_field_lambda,
            "event_gate_mode": args.event_gate_mode,
            "event_gate_threshold": args.event_gate_threshold,
            "event_gate_tau": args.event_gate_tau,
            "hybrid_local_k": args.hybrid_local_k,
            "hybrid_complex_roots": args.hybrid_complex_roots,
            "hybrid_local_min_k": args.hybrid_local_min_k,
            "hybrid_local_target": args.hybrid_local_target,
            "hybrid_local_hysteresis": args.hybrid_local_hysteresis,
            "hybrid_local_converge_lambda": args.hybrid_local_converge_lambda,
            "adaptive_min_pair_bins": args.adaptive_min_pair_bins,
            "adaptive_time_growth": args.adaptive_time_growth,
            "adaptive_balance": args.adaptive_balance,
            "adaptive_angle_growth": args.adaptive_angle_growth,
            "adaptive_shell_growth": args.adaptive_shell_growth,
            "adaptive_shell_balance": args.adaptive_shell_balance,
            "adaptive_converge_lambda": args.adaptive_converge_lambda,
            "adaptive_converge_target": args.adaptive_converge_target,
            "adaptive_converge_hysteresis": args.adaptive_converge_hysteresis,
            "adaptive_converge_mode": args.adaptive_converge_mode,
            "learn_so8": args.learn_so8,
            "learn_scale": args.learn_scale,
            "scale_mode": args.scale_mode,
            "radial_bins": args.radial_bins,
            "radial_rmax": args.radial_rmax,
            "radial_update_frac": args.radial_update_frac,
            "radial_l2": args.radial_l2,
            "chart_iters": args.chart_iters,
            "chart_alpha": args.chart_alpha,
            "chart_beta": args.chart_beta,
            "so8_step": args.so8_step,
            "so8_candidates": args.so8_candidates,
            "scale_step": args.scale_step,
            "scale_candidates": args.scale_candidates,
            "scale_clip": args.scale_clip,
            "recluster_after_chart": args.recluster_after_chart,
            "fast_dev": args.fast_dev,
        },
    }


def proxy_route_cache_payload(args: argparse.Namespace, chart_fp: str, c_fp: str, n_train: int, n_eval: int):
    return {
        "cache_version": "proxy_routes_v12",
        "input": args.input,
        "eval_split": args.eval_split,
        "n_train": n_train,
        "n_eval": n_eval,
        "chart_fp": chart_fp,
        "c_fp": c_fp,
        "args": {
            "seed": args.seed,
            "delta_r": args.delta_r,
            "sector_mode": args.sector_mode,
            "phase_dims": args.phase_dims,
            "phase4_dims": args.phase4_dims,
            "field4_dims": args.field4_dims,
            "complex_dims": args.complex_dims,
            "route_scale_lambda": args.route_scale_lambda,
            "memory_coord_mode": args.memory_coord_mode,
            "shell_mode": args.shell_mode,
            "shell_phase_coupling": args.shell_phase_coupling,
            "product_shell_control_mode": args.product_shell_control_mode,
            "product_shell_gate_threshold": args.product_shell_gate_threshold,
            "fib_rung_gate_threshold": args.fib_rung_gate_threshold,
            "hopf_chi_bins": args.hopf_chi_bins,
            "hopf_blend_lambda": args.hopf_blend_lambda,
            "hopf_blend_chi_weight": args.hopf_blend_chi_weight,
            "hopf_blend_shell_weight": args.hopf_blend_shell_weight,
            "phase_transport_lambda": args.phase_transport_lambda,
            "phase_field_lambda": args.phase_field_lambda,
            "event_gate_mode": args.event_gate_mode,
            "event_gate_threshold": args.event_gate_threshold,
            "event_gate_tau": args.event_gate_tau,
            "hybrid_local_k": args.hybrid_local_k,
            "hybrid_complex_roots": args.hybrid_complex_roots,
            "hybrid_local_min_k": args.hybrid_local_min_k,
            "hybrid_local_target": args.hybrid_local_target,
            "hybrid_local_hysteresis": args.hybrid_local_hysteresis,
            "hybrid_local_converge_lambda": args.hybrid_local_converge_lambda,
            "K": args.K,
            "adaptive_min_pair_bins": args.adaptive_min_pair_bins,
            "adaptive_time_growth": args.adaptive_time_growth,
            "adaptive_balance": args.adaptive_balance,
            "adaptive_angle_growth": args.adaptive_angle_growth,
            "adaptive_shell_growth": args.adaptive_shell_growth,
            "adaptive_shell_balance": args.adaptive_shell_balance,
            "adaptive_converge_lambda": args.adaptive_converge_lambda,
            "adaptive_converge_target": args.adaptive_converge_target,
            "adaptive_converge_hysteresis": args.adaptive_converge_hysteresis,
            "adaptive_converge_mode": args.adaptive_converge_mode,
        },
    }


def main():
    args = parse_args()
    t_total_start = time.perf_counter()
    timings = {
        "dataset": 0.0,
        "chart_opt": 0.0,
        "routing_eval": 0.0,
        "training_route": 0.0,
        "training_update": 0.0,
        "training_ema": 0.0,
        "growth": 0.0,
        "total": 0.0,
    }
    notes = []
    artifacts = {"input": args.input, "eval_split": args.eval_split}

    apply_proxy_fast_dev(args)

    t0 = time.perf_counter()
    data = np.load(args.input)
    x_train = data["x_train"].astype(np.float64)
    y_train = data["y_train"].astype(np.float64)
    if args.eval_split == "val":
        x_eval = data["x_val"].astype(np.float64)
        y_eval = data["y_val"].astype(np.float64)
    else:
        x_eval = data["x_test"].astype(np.float64)
        y_eval = data["y_test"].astype(np.float64)
    timings["dataset"] = time.perf_counter() - t0

    v_tr, y_tr = _subset(x_train, y_train, args.max_train, args.seed + 1)
    v_ev, y_ev = _subset(x_eval, y_eval, args.max_eval, args.seed + 2)

    if args.input_transform != "none":
        rng = np.random.RandomState(args.seed + 77)
        if args.input_transform == "col_perm":
            for j in range(v_tr.shape[1]):
                v_tr[:, j] = rng.permutation(v_tr[:, j])
            for j in range(v_ev.shape[1]):
                v_ev[:, j] = rng.permutation(v_ev[:, j])
        elif args.input_transform == "gaussian":
            mu = v_tr.mean(axis=0)
            sd = v_tr.std(axis=0) + 1e-12
            v_tr = rng.randn(*v_tr.shape) * sd + mu
            v_ev = rng.randn(*v_ev.shape) * sd + mu

    d = int(v_tr.shape[1])
    dy = int(y_tr.shape[1])

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

    if int(args.cache_chart) == 1 or int(args.cache_routes) == 1:
        hr.ensure_dir(args.cache_dir)

    C0 = None
    if args.sector_mode == "kmeans":
        U0 = hr.normalize_rows(v_tr)
        C0 = hr.spherical_kmeans(U0, K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

    chart = hr.Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)

    radial_rmax = float(args.radial_rmax)
    if learned_chart and args.learn_scale == 1 and args.scale_mode == "radial" and radial_rmax <= 0:
        radial_rmax = float(np.quantile(hr.safe_norm(v_tr, axis=1, keepdims=False), 0.995))
        radial_rmax = max(radial_rmax, 1e-6)

    chart_cache_hit = False
    chart_cache_file = ""
    opt_res = None
    if learned_chart and int(args.cache_chart) == 1:
        key = hr.stable_hash(proxy_chart_cache_payload(args, d=d, dy=dy, n_train=v_tr.shape[0]))
        chart_cache_file = os.path.join(args.cache_dir, f"proxy_chart_{key}.npz")
        artifacts["chart_cache_file"] = chart_cache_file
        if os.path.exists(chart_cache_file):
            cc = np.load(chart_cache_file, allow_pickle=True)
            s_global = cc["s_global"] if bool(cc["has_s_global"]) else None
            S_radial = cc["S_radial"] if bool(cc["has_S_radial"]) else None
            chart = hr.Chart(
                R=cc["R"],
                s_global=s_global,
                S_radial=S_radial,
                scale_mode=str(cc["scale_mode"]),
                radial_rmax=float(cc["radial_rmax"]),
                radial_bins=int(cc["radial_bins"]),
            )
            chart_cache_hit = True
            notes.append("proxy chart cache hit")

    if learned_chart and not chart_cache_hit:
        t_chart = time.perf_counter()
        opt_res = hr.optimize_chart(
            v_train=v_tr,
            y_train=y_tr,
            delta_r=args.delta_r,
            C0=C0,
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
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            field4_dim_i=field4_dim_i,
            field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k,
            field4_dim_l=field4_dim_l,
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
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
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
        timings["chart_opt"] = time.perf_counter() - t_chart
        chart = opt_res.chart
        if int(args.cache_chart) == 1:
            if chart_cache_file == "":
                key = hr.stable_hash(proxy_chart_cache_payload(args, d=d, dy=dy, n_train=v_tr.shape[0]))
                chart_cache_file = os.path.join(args.cache_dir, f"proxy_chart_{key}.npz")
                artifacts["chart_cache_file"] = chart_cache_file
            np.savez_compressed(
                chart_cache_file,
                R=chart.R,
                has_s_global=(chart.s_global is not None),
                s_global=(chart.s_global if chart.s_global is not None else np.zeros((0,), dtype=np.float64)),
                has_S_radial=(chart.S_radial is not None),
                S_radial=(chart.S_radial if chart.S_radial is not None else np.zeros((0, 0), dtype=np.float64)),
                scale_mode=np.array(chart.scale_mode),
                radial_rmax=np.array(chart.radial_rmax),
                radial_bins=np.array(chart.radial_bins),
            )

    reclustered = False
    C_used = C0
    if args.sector_mode == "kmeans" and learned_chart and args.recluster_after_chart == 1:
        z_tr0 = hr.apply_chart(v_tr, chart)
        U_tr_chart = hr.normalize_rows(z_tr0)
        C_used = hr.spherical_kmeans(U_tr_chart, K=args.K, iters=args.kmeans_iters, seed=args.seed + 111)
        reclustered = True

    t_route = time.perf_counter()
    route_cache_hit = False
    route_cache_file = ""
    if int(args.cache_routes) == 1:
        chart_fp = hr.chart_fingerprint(chart)
        c_fp = "none" if C_used is None else hr.stable_hash({"C": C_used.tolist()})
        r_key = hr.stable_hash(proxy_route_cache_payload(args, chart_fp=chart_fp, c_fp=c_fp, n_train=v_tr.shape[0], n_eval=v_ev.shape[0]))
        route_cache_file = os.path.join(args.cache_dir, f"proxy_routes_{r_key}.npz")
        artifacts["route_cache_file"] = route_cache_file
        if os.path.exists(route_cache_file):
            rr = np.load(route_cache_file)
            shell_tr_eval = rr["shell_tr_eval"]
            sector_tr_eval = rr["sector_tr_eval"]
            z_tr_eval = rr["z_tr_eval"]
            shell_ev = rr["shell_ev"]
            sector_ev = rr["sector_ev"]
            z_ev = rr["z_ev"]
            route_cache_hit = True
            notes.append("proxy route cache hit")
    if not route_cache_hit:
        shell_tr_eval, sector_tr_eval, _, z_tr_eval = hr.route_addresses(
            v_tr, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            K=args.K,
            time_pressure_lambda=0.0, tau=1.0,
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
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
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
            shell_pressure_w=args.shell_pressure_w,
        )
        shell_ev, sector_ev, _, z_ev = hr.route_addresses(
            v_ev, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
            K=args.K,
            time_pressure_lambda=0.0, tau=1.0,
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
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
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
            shell_pressure_w=args.shell_pressure_w,
        )
        if int(args.cache_routes) == 1:
            if route_cache_file == "":
                chart_fp = hr.chart_fingerprint(chart)
                c_fp = "none" if C_used is None else hr.stable_hash({"C": C_used.tolist()})
                r_key = hr.stable_hash(proxy_route_cache_payload(args, chart_fp=chart_fp, c_fp=c_fp, n_train=v_tr.shape[0], n_eval=v_ev.shape[0]))
                route_cache_file = os.path.join(args.cache_dir, f"proxy_routes_{r_key}.npz")
                artifacts["route_cache_file"] = route_cache_file
            np.savez_compressed(
                route_cache_file,
                shell_tr_eval=shell_tr_eval,
                sector_tr_eval=sector_tr_eval,
                z_tr_eval=z_tr_eval,
                shell_ev=shell_ev,
                sector_ev=sector_ev,
                z_ev=z_ev,
            )
    timings["routing_eval"] = time.perf_counter() - t_route

    keys_tr_eval = [hr.make_bucket_key(shell_tr_eval[i], sector_tr_eval[i]) for i in range(v_tr.shape[0])]
    keys_ev = [hr.make_bucket_key(shell_ev[i], sector_ev[i]) for i in range(v_ev.shape[0])]

    train_key_set = set(keys_tr_eval)
    unseen_rate = hr.unseen_key_rate(keys_ev, train_key_set)

    shell_vals, shell_counts = np.unique(shell_ev, return_counts=True)
    sector_vals, sector_counts = np.unique(sector_ev, return_counts=True)
    shell_pmax = float(np.max(shell_counts) / np.sum(shell_counts)) if len(shell_counts) else 0.0
    sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
    shell_entropy = hr.entropy_from_counts(shell_counts) if len(shell_counts) else 0.0
    sector_entropy = hr.entropy_from_counts(sector_counts) if len(sector_counts) else 0.0
    route_z_ev = hr.route_coordinate(v_ev, chart, sector_mode=args.sector_mode, route_scale_lambda=args.route_scale_lambda)
    alignment = hr.poincare_alignment_diagnostics(
        v_ev,
        route_z_ev,
        max_pairs=512,
        seed=args.seed + 811,
    )
    poincare_alignment_pairs_used = int(alignment["poincare_alignment_pairs_used"])
    poincare_alignment_radial_mae = float(alignment["poincare_alignment_radial_mae"])
    poincare_alignment_radial_rel_mean = float(alignment["poincare_alignment_radial_rel_mean"])
    poincare_alignment_radial_corr = float(alignment["poincare_alignment_radial_corr"])
    poincare_alignment_pair_mae = float(alignment["poincare_alignment_pair_mae"])
    poincare_alignment_pair_rel_mean = float(alignment["poincare_alignment_pair_rel_mean"])
    poincare_alignment_pair_corr = float(alignment["poincare_alignment_pair_corr"])
    shell_measure = hr.shell_measure_diagnostics(shell_ev, delta_r=args.delta_r, shell_mode=args.shell_mode)
    shell_mass_error_l1 = float(shell_measure["shell_mass_error_l1"])
    shell_mass_error_max = float(shell_measure["shell_mass_error_max"])
    shell_mass_kl = float(shell_measure["shell_mass_kl"])
    shell_mass_corr = float(shell_measure["shell_mass_corr"])
    shell_mass_shells_used = int(shell_measure["shell_mass_shells_used"])
    route_entropy = hr.route_entropy_radius_diagnostics(shell_ev, sector_ev)
    route_entropy_radius_corr = float(route_entropy["route_entropy_radius_corr"])
    route_entropy_radius_slope = float(route_entropy["route_entropy_radius_slope"])
    route_entropy_shells_used = int(route_entropy["route_entropy_shells_used"])
    hopf_measure = hr.hopf_angular_measure_diagnostics(
        route_z_ev,
        dim_i=phase4_dim_i,
        dim_j=phase4_dim_j,
        dim_k=phase4_dim_k,
        dim_l=phase4_dim_l,
        chi_bins=max(2, int(args.hopf_chi_bins)),
        theta_bins=12,
    )
    hopf_angular_mass_error = float(hopf_measure["hopf_angular_mass_error"])
    hopf_chi_mass_error = float(hopf_measure["hopf_chi_mass_error"])
    hopf_theta1_mass_error = float(hopf_measure["hopf_theta1_mass_error"])
    hopf_theta2_mass_error = float(hopf_measure["hopf_theta2_mass_error"])
    hopf_theta1_entropy = float(hopf_measure["hopf_theta1_entropy"])
    hopf_theta2_entropy = float(hopf_measure["hopf_theta2_entropy"])
    hopf_base_measure = hr.hopf_base_measure_diagnostics(
        route_z_ev,
        dim_i=phase4_dim_i,
        dim_j=phase4_dim_j,
        dim_k=phase4_dim_k,
        dim_l=phase4_dim_l,
        chi_bins=max(2, int(args.hopf_chi_bins)),
        delta_bins=12,
        alpha_bins=12,
    )
    hopf_base_mass_error = float(hopf_base_measure["hopf_base_mass_error"])
    hopf_delta_mass_error = float(hopf_base_measure["hopf_delta_mass_error"])
    hopf_alpha_entropy = float(hopf_base_measure["hopf_alpha_entropy"])
    if args.sector_mode == "phase4d_hopf_transport_complex":
        phase_transport = hr.hopf_phase_transport_complex_diagnostics(
            route_z_ev,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
        )
    elif args.sector_mode == "phase4d_hopf_product_phase":
        phase_transport = hr.hopf_product_phase_diagnostics(
            route_z_ev,
            route_dim_i=phase4_dim_i,
            route_dim_j=phase4_dim_j,
            route_dim_k=phase4_dim_k,
            route_dim_l=phase4_dim_l,
            field_dim_i=field4_dim_i,
            field_dim_j=field4_dim_j,
            field_dim_k=field4_dim_k,
            field_dim_l=field4_dim_l,
            phase_transport_lambda=args.phase_transport_lambda,
            phase_field_lambda=args.phase_field_lambda,
        )
    else:
        phase_transport = hr.hopf_phase_transport_diagnostics(
            route_z_ev,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            phase_transport_lambda=args.phase_transport_lambda,
        )
    phase_transport_coherence = float(phase_transport["phase_transport_coherence"])
    phase_transport_shift_abs_mean = float(phase_transport["phase_transport_shift_abs_mean"])
    phase_transport_shift_abs_max = float(phase_transport["phase_transport_shift_abs_max"])
    phase_transport_connection_abs_mean = float(phase_transport["phase_transport_connection_abs_mean"])
    phase_transport_field_shift_abs_mean = float(phase_transport.get("phase_transport_field_shift_abs_mean", 0.0))
    phase_transport_field_weight_abs_mean = float(phase_transport.get("phase_transport_field_weight_abs_mean", 0.0))
    product_shell_gate_score_mean = 0.0
    product_shell_gate_score_max = 0.0
    product_shell_active_frac = 0.0
    product_shell_states_used = 0.0
    product_shell_multiplier_mean = 1.0
    product_shell_multiplier_max = 1.0
    event_gate_error_mean = 0.0
    event_gate_mean = 1.0
    event_gate_active_frac = 1.0
    event_gate_cost_proxy = 1.0
    geodesic_neighbors = hr.geodesic_neighborhood_diagnostics(
        v_ev,
        route_z_ev,
        max_points=256,
        k=8,
        seed=args.seed + 977,
    )
    geodesic_knn_overlap_k = float(geodesic_neighbors["geodesic_knn_overlap_k"])
    geodesic_knn_overlap_mean = float(geodesic_neighbors["geodesic_knn_overlap_mean"])
    geodesic_knn_jaccard_mean = float(geodesic_neighbors["geodesic_knn_jaccard_mean"])
    geodesic_knn_points_used = int(geodesic_neighbors["geodesic_knn_points_used"])
    adaptive_k1_mean = 0.0
    adaptive_k2_mean = 0.0
    adaptive_k1_max = 0
    adaptive_k2_max = 0
    adaptive_shell_drive_mean = 0.0
    adaptive_shell_drive_max = 0.0
    adaptive_shell_expand_mean = 0.0
    adaptive_shell_expand_max = 0.0
    adaptive_shell_target_band_mean = 0.0
    adaptive_shell_target_band_max = 0.0
    adaptive_shell_overflow_mean = 0.0
    adaptive_shell_overflow_max = 0.0
    adaptive_shell_ratio_mean = 0.0
    adaptive_shell_ratio_max = 0.0
    adaptive_shell_ratio_pressure_mean = 0.0
    adaptive_shell_ratio_pressure_max = 0.0
    adaptive_shell_ladder_steps_mean = 0.0
    adaptive_shell_ladder_steps_max = 0
    adaptive_shell_converge_mean = 0.0
    adaptive_shell_converge_max = 0.0
    adaptive_shell_mult_mean = 1.0
    adaptive_shell_mult_max = 1.0
    adaptive_phase_gap_abs_mean = 0.0
    adaptive_phase_gap_abs_max = 0.0
    adaptive_shell_phase_bias_mean = 0.0
    adaptive_shell_phase_bias_max = 0.0
    adaptive_chi_mean = 0.0
    adaptive_chi_entropy = 0.0
    adaptive_r_alpha_mean = 0.0
    adaptive_r_alpha_max = 0.0
    adaptive_hopf_shell_capacity_mean = 0.0
    adaptive_hopf_shell_capacity_max = 0.0
    adaptive_hopf_k1_mean = 0.0
    adaptive_hopf_k2_mean = 0.0
    adaptive_hopf_k1_max = 0
    adaptive_hopf_k2_max = 0
    adaptive_hopf_k1_gap_mean = 0.0
    adaptive_hopf_k2_gap_mean = 0.0
    adaptive_chi_bins_used = 0
    adaptive_chi_bin_pmax = 0.0
    adaptive_chi_bin_entropy = 0.0
    adaptive_fib_total_mean = 0.0
    adaptive_fib_total_max = 0
    adaptive_fib_kchi_mean = 0.0
    adaptive_fib_kchi_max = 0
    adaptive_fib_forced_total_mean = 0.0
    adaptive_fib_forced_total_max = 0
    adaptive_fib_band_mean = 0.0
    adaptive_fib_band_max = 0
    adaptive_fib_band_entropy = 0.0
    adaptive_fib_band_states_used = 0
    adaptive_blend_total_mean = 0.0
    adaptive_blend_total_max = 0
    adaptive_blend_score_mean = 0.0
    adaptive_blend_score_max = 0.0
    adaptive_blend_chi_pressure_mean = 0.0
    adaptive_blend_chi_pressure_max = 0.0
    adaptive_blend_shell_pressure_mean = 0.0
    adaptive_blend_shell_pressure_max = 0.0
    hopf_sector_groups_used = 0
    hopf_sector_chi_std_mean = 0.0
    hopf_sector_delta_cvar_mean = 0.0
    hopf_sector_alpha_entropy_mean = 0.0
    hopf_sector_alpha_entropy_gap = 0.0
    phase_transport_alpha_bins = 0.0
    hybrid_coarse_eval_sectors = 0
    hybrid_local_eval_sectors = 0
    hybrid_local_pmax = 0.0
    hybrid_local_entropy = 0.0
    hybrid_local_drive_mean = 0.0
    hybrid_local_drive_max = 0.0
    hybrid_local_target_band_mean = 0.0
    hybrid_local_target_band_max = 0.0
    hybrid_local_ratio_mean = 0.0
    hybrid_local_ratio_max = 0.0
    hybrid_local_ratio_pressure_mean = 0.0
    hybrid_local_ratio_pressure_max = 0.0
    hybrid_local_converge_mean = 0.0
    hybrid_local_converge_max = 0.0
    hybrid_local_activation_mean = 0.0
    hybrid_local_activation_max = 0.0
    hybrid_local_k_eff_mean = 0.0
    hybrid_local_k_eff_max = 0
    diag_z_ev = route_z_ev
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        comp = hr.phase4d_adaptive_components(
            z=diag_z_ev,
            K=args.K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            delta_r=args.delta_r,
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
            shell_phase_coupling=args.shell_phase_coupling,
        )
        k1_dbg = comp["k1"]
        k2_dbg = comp["k2"]
        chi_ids = np.minimum((comp["chi_u"] * max(1, int(args.hopf_chi_bins))).astype(np.int64), max(max(1, int(args.hopf_chi_bins)) - 1, 0))
        if args.sector_mode in ("phase4d_hopf_base", "phase4d_hopf_base_ball"):
            _, kchi_dbg, kdelta_dbg = hr.assign_sectors_phase4d_hopf_base(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
            )
            k1_dbg = kchi_dbg
            k2_dbg = kdelta_dbg
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode in ("phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase"):
            if args.sector_mode == "phase4d_hopf_transport_complex":
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = hr.assign_sectors_phase4d_hopf_transport_complex(
                    z=diag_z_ev,
                    K=args.K,
                    dim_i=phase4_dim_i,
                    dim_j=phase4_dim_j,
                    dim_k=phase4_dim_k,
                    dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    phase_transport_lambda=args.phase_transport_lambda,
                    phase_field_lambda=args.phase_field_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            elif args.sector_mode == "phase4d_hopf_product_phase":
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = hr.assign_sectors_phase4d_hopf_product_phase(
                    z=diag_z_ev,
                    K=args.K,
                    route_dim_i=phase4_dim_i,
                    route_dim_j=phase4_dim_j,
                    route_dim_k=phase4_dim_k,
                    route_dim_l=phase4_dim_l,
                    field_dim_i=field4_dim_i,
                    field_dim_j=field4_dim_j,
                    field_dim_k=field4_dim_k,
                    field_dim_l=field4_dim_l,
                    phase_transport_lambda=args.phase_transport_lambda,
                    phase_field_lambda=args.phase_field_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            else:
                _, kchi_dbg, kdelta_dbg, kalpha_dbg = hr.assign_sectors_phase4d_hopf_transport(
                    z=diag_z_ev,
                    K=args.K,
                    dim_i=phase4_dim_i,
                    dim_j=phase4_dim_j,
                    dim_k=phase4_dim_k,
                    dim_l=phase4_dim_l,
                    phase_transport_lambda=args.phase_transport_lambda,
                    hopf_chi_bins=args.hopf_chi_bins,
                )
            k1_dbg = kchi_dbg
            k2_dbg = kdelta_dbg
            phase_transport_alpha_bins = float(np.mean(kalpha_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_fib":
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg = hr.assign_sectors_phase4d_hopf_fib(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(kchi_dbg * k1_dbg * k2_dbg))
            adaptive_fib_forced_total_max = int(np.max(kchi_dbg * k1_dbg * k2_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_fib_rung":
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg, fib_forced_total_dbg = hr.assign_sectors_phase4d_hopf_fib_rung(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
                fib_rung_gate_threshold=args.fib_rung_gate_threshold,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(fib_forced_total_dbg))
            adaptive_fib_forced_total_max = int(np.max(fib_forced_total_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode in ("phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound"):
            _, kchi_dbg, k1_dbg, k2_dbg, fib_total_dbg, fib_forced_total_dbg, fib_band_dbg = hr.assign_sectors_phase4d_hopf_fib_band(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
            )
            adaptive_fib_total_mean = float(np.mean(fib_total_dbg))
            adaptive_fib_total_max = int(np.max(fib_total_dbg))
            adaptive_fib_kchi_mean = float(np.mean(kchi_dbg))
            adaptive_fib_kchi_max = int(np.max(kchi_dbg))
            adaptive_fib_forced_total_mean = float(np.mean(fib_forced_total_dbg))
            adaptive_fib_forced_total_max = int(np.max(fib_forced_total_dbg))
            adaptive_fib_band_mean = float(np.mean(fib_band_dbg))
            adaptive_fib_band_max = int(np.max(fib_band_dbg))
            band_counts = np.unique(fib_band_dbg, return_counts=True)[1]
            adaptive_fib_band_entropy = hr.entropy_from_counts(band_counts) if len(band_counts) else 0.0
            adaptive_fib_band_states_used = int(len(band_counts))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_blend":
            _, kchi_dbg, k1_dbg, k2_dbg, blend_total_dbg, blend_score_dbg, chi_pressure_dbg, shell_pressure_dbg = hr.assign_sectors_phase4d_hopf_blend(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
                tau=1.0,
                adaptive_min_pair_bins=args.adaptive_min_pair_bins,
                adaptive_time_growth=args.adaptive_time_growth,
                adaptive_balance=args.adaptive_balance,
                adaptive_angle_growth=args.adaptive_angle_growth,
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            product_shell_control_mode=args.product_shell_control_mode,
            product_shell_gate_threshold=args.product_shell_gate_threshold,
            hopf_chi_bins=args.hopf_chi_bins,
                hopf_blend_lambda=args.hopf_blend_lambda,
                hopf_blend_chi_weight=args.hopf_blend_chi_weight,
                hopf_blend_shell_weight=args.hopf_blend_shell_weight,
            )
            adaptive_blend_total_mean = float(np.mean(blend_total_dbg))
            adaptive_blend_total_max = int(np.max(blend_total_dbg))
            adaptive_blend_score_mean = float(np.mean(blend_score_dbg))
            adaptive_blend_score_max = float(np.max(blend_score_dbg))
            adaptive_blend_chi_pressure_mean = float(np.mean(chi_pressure_dbg))
            adaptive_blend_chi_pressure_max = float(np.max(chi_pressure_dbg))
            adaptive_blend_shell_pressure_mean = float(np.mean(shell_pressure_dbg))
            adaptive_blend_shell_pressure_max = float(np.max(shell_pressure_dbg))
            chi_ids = np.minimum((comp["chi_u"] * kchi_dbg.astype(np.float64)).astype(np.int64), np.maximum(kchi_dbg - 1, 0))
        if args.sector_mode == "phase4d_hopf_product_phase":
            product_shell_diag = hr.product_phase_shell_diagnostics(
                z=diag_z_ev,
                K=args.K,
                dim_i=phase4_dim_i,
                dim_j=phase4_dim_j,
                dim_k=phase4_dim_k,
                dim_l=phase4_dim_l,
                delta_r=args.delta_r,
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
                product_shell_control_mode=args.product_shell_control_mode,
                product_shell_gate_threshold=args.product_shell_gate_threshold,
            )
            product_shell_gate_score_mean = float(product_shell_diag["product_shell_gate_score_mean"])
            product_shell_gate_score_max = float(product_shell_diag["product_shell_gate_score_max"])
            product_shell_active_frac = float(product_shell_diag["product_shell_active_frac"])
            product_shell_states_used = float(product_shell_diag["product_shell_states_used"])
            product_shell_multiplier_mean = float(product_shell_diag["product_shell_multiplier_mean"])
            product_shell_multiplier_max = float(product_shell_diag["product_shell_multiplier_max"])
        adaptive_k1_mean = float(np.mean(k1_dbg))
        adaptive_k2_mean = float(np.mean(k2_dbg))
        adaptive_k1_max = int(np.max(k1_dbg))
        adaptive_k2_max = int(np.max(k2_dbg))
        adaptive_shell_drive_mean = float(np.mean(comp["shell_drive"]))
        adaptive_shell_drive_max = float(np.max(comp["shell_drive"]))
        adaptive_shell_expand_mean = float(np.mean(comp["shell_expand"]))
        adaptive_shell_expand_max = float(np.max(comp["shell_expand"]))
        adaptive_shell_target_band_mean = float(np.mean(comp["shell_target_band"]))
        adaptive_shell_target_band_max = float(np.max(comp["shell_target_band"]))
        adaptive_shell_overflow_mean = float(np.mean(comp["shell_overflow"]))
        adaptive_shell_overflow_max = float(np.max(comp["shell_overflow"]))
        adaptive_shell_ratio_mean = float(np.mean(comp["shell_ratio"]))
        adaptive_shell_ratio_max = float(np.max(comp["shell_ratio"]))
        adaptive_shell_ratio_pressure_mean = float(np.mean(comp["shell_ratio_pressure"]))
        adaptive_shell_ratio_pressure_max = float(np.max(comp["shell_ratio_pressure"]))
        adaptive_shell_ladder_steps_mean = float(np.mean(comp["shell_ladder_steps"]))
        adaptive_shell_ladder_steps_max = int(np.max(comp["shell_ladder_steps"]))
        adaptive_shell_converge_mean = float(np.mean(comp["shell_converge"]))
        adaptive_shell_converge_max = float(np.max(comp["shell_converge"]))
        adaptive_shell_mult_mean = float(np.mean(comp["shell_multiplier"]))
        adaptive_shell_mult_max = float(np.max(comp["shell_multiplier"]))
        adaptive_phase_gap_abs_mean = float(np.mean(np.abs(comp["phase_gap"])))
        adaptive_phase_gap_abs_max = float(np.max(np.abs(comp["phase_gap"])))
        adaptive_shell_phase_bias_mean = float(np.mean(comp["shell_phase_bias"]))
        adaptive_shell_phase_bias_max = float(np.max(np.abs(comp["shell_phase_bias"])))
        adaptive_chi_mean = float(np.mean(comp["chi"]))
        adaptive_chi_entropy = float(comp["chi_entropy"][0]) if len(comp["chi_entropy"]) else 0.0
        adaptive_r_alpha_mean = float(np.mean(comp["r_alpha"]))
        adaptive_r_alpha_max = float(np.max(comp["r_alpha"]))
        adaptive_hopf_shell_capacity_mean = float(np.mean(comp["hopf_shell_capacity"]))
        adaptive_hopf_shell_capacity_max = float(np.max(comp["hopf_shell_capacity"]))
        adaptive_hopf_k1_mean = float(np.mean(comp["hopf_k1"]))
        adaptive_hopf_k2_mean = float(np.mean(comp["hopf_k2"]))
        adaptive_hopf_k1_max = int(np.max(comp["hopf_k1"]))
        adaptive_hopf_k2_max = int(np.max(comp["hopf_k2"]))
        adaptive_hopf_k1_gap_mean = float(np.mean(comp["hopf_k1_gap"]))
        adaptive_hopf_k2_gap_mean = float(np.mean(comp["hopf_k2_gap"]))
        chi_counts = np.unique(chi_ids, return_counts=True)[1]
        adaptive_chi_bins_used = int(len(chi_counts))
        adaptive_chi_bin_pmax = float(np.max(chi_counts) / np.sum(chi_counts)) if len(chi_counts) else 0.0
        adaptive_chi_bin_entropy = hr.entropy_from_counts(chi_counts) if len(chi_counts) else 0.0
    if args.sector_mode == "phase4d_complex_local":
        _, _, local_sector_ev, hybrid_dbg = hr.assign_sectors_phase4d_complex_local(
            z=diag_z_ev,
            K=args.K,
            dim_i=phase4_dim_i,
            dim_j=phase4_dim_j,
            dim_k=phase4_dim_k,
            dim_l=phase4_dim_l,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            delta_r=args.delta_r,
            tau=1.0,
            time_pressure_lambda=0.0,
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
            shell_mode=args.shell_mode,
            shell_phase_coupling=args.shell_phase_coupling,
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        )
        coarse_sector_ev = hybrid_dbg["coarse_sector"]
        coarse_counts = np.unique(coarse_sector_ev, return_counts=True)[1]
        local_counts = np.unique(local_sector_ev, return_counts=True)[1]
        hybrid_coarse_eval_sectors = int(len(coarse_counts))
        hybrid_local_eval_sectors = int(len(local_counts))
        hybrid_local_pmax = float(np.max(local_counts) / np.sum(local_counts)) if len(local_counts) else 0.0
        hybrid_local_entropy = hr.entropy_from_counts(local_counts) if len(local_counts) else 0.0
        hybrid_local_drive_mean = float(np.mean(hybrid_dbg["local_drive"]))
        hybrid_local_drive_max = float(np.max(hybrid_dbg["local_drive"]))
        hybrid_local_target_band_mean = float(np.mean(hybrid_dbg["local_target_band"]))
        hybrid_local_target_band_max = float(np.max(hybrid_dbg["local_target_band"]))
        hybrid_local_ratio_mean = float(np.mean(hybrid_dbg["local_ratio"]))
        hybrid_local_ratio_max = float(np.max(hybrid_dbg["local_ratio"]))
        hybrid_local_ratio_pressure_mean = float(np.mean(hybrid_dbg["local_ratio_pressure"]))
        hybrid_local_ratio_pressure_max = float(np.max(hybrid_dbg["local_ratio_pressure"]))
        hybrid_local_converge_mean = float(np.mean(hybrid_dbg["local_converge"]))
        hybrid_local_converge_max = float(np.max(hybrid_dbg["local_converge"]))
        hybrid_local_activation_mean = float(np.mean(hybrid_dbg["local_activation"]))
        hybrid_local_activation_max = float(np.max(hybrid_dbg["local_activation"]))
        hybrid_local_k_eff_mean = float(np.mean(hybrid_dbg["local_k_eff"]))
        hybrid_local_k_eff_max = int(np.max(hybrid_dbg["local_k_eff"]))

    hopf_sector = hr.hopf_sector_routing_diagnostics(
        route_z_ev,
        sector_ev,
        dim_i=phase4_dim_i,
        dim_j=phase4_dim_j,
        dim_k=phase4_dim_k,
        dim_l=phase4_dim_l,
        alpha_bins=12,
    )
    hopf_sector_groups_used = int(hopf_sector["hopf_sector_groups_used"])
    hopf_sector_chi_std_mean = float(hopf_sector["hopf_sector_chi_std_mean"])
    hopf_sector_delta_cvar_mean = float(hopf_sector["hopf_sector_delta_cvar_mean"])
    hopf_sector_alpha_entropy_mean = float(hopf_sector["hopf_sector_alpha_entropy_mean"])
    hopf_sector_alpha_entropy_gap = float(hopf_sector["hopf_sector_alpha_entropy_gap"])

    train_label_sse = hr.label_coherence_sse(
        v_tr, y_tr, delta_r=args.delta_r, C=C_used, chart=chart,
        sector_mode=args.sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=args.K,
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
    eval_label_sse = hr.label_coherence_sse(
        v_ev, y_ev, delta_r=args.delta_r, C=C_used, chart=chart,
        sector_mode=args.sector_mode,
        phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
        phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
        phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
        complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
        field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
        field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
        K=args.K,
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
    train_label_sse_per = train_label_sse / max(1, v_tr.shape[0])
    eval_label_sse_per = eval_label_sse / max(1, v_ev.shape[0])

    buckets = hr.init_buckets(keys_tr_eval, dy=dy, d=d, seed=args.seed + 101)

    t_train = time.perf_counter()
    total_steps = args.epochs * v_tr.shape[0]
    step_ctr = 0
    event_gate_error_sum = 0.0
    event_gate_sum = 0.0
    event_gate_active_sum = 0.0
    for _epoch in range(args.epochs):
        order = np.random.permutation(v_tr.shape[0])
        for jj in order:
            tau = 1.0 if total_steps <= 1 else float(step_ctr) / float(total_steps - 1)
            t_train_route = time.perf_counter()
            if args.train_route_mode == "final_static":
                key = keys_tr_eval[jj]
                z1 = z_tr_eval[jj]
            else:
                key, z1 = hr.route_one(
                    v_tr[jj],
                    delta_r=args.delta_r,
                    C=C_used,
                    chart=chart,
                    sector_mode=args.sector_mode,
                    phase_dim_i=phase_dim_i,
                    phase_dim_j=phase_dim_j,
                    phase4_dim_i=phase4_dim_i,
                    phase4_dim_j=phase4_dim_j,
                    phase4_dim_k=phase4_dim_k,
                    phase4_dim_l=phase4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    field4_dim_i=field4_dim_i,
                    field4_dim_j=field4_dim_j,
                    field4_dim_k=field4_dim_k,
                    field4_dim_l=field4_dim_l,
                    K=args.K,
                    time_pressure_lambda=float(args.time_pressure_lambda),
                    tau=tau,
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
                    product_shell_control_mode=args.product_shell_control_mode,
                    product_shell_gate_threshold=args.product_shell_gate_threshold,
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
            timings["training_route"] += time.perf_counter() - t_train_route
            t_train_update = time.perf_counter()
            yhat, sj, _ = hr.predict_from_bucket(buckets, key, z1, d=d, dy=dy)
            error_mag, event_gate, event_active = event_gate_stats(
                yhat=yhat,
                y=y_tr[jj],
                mode=args.event_gate_mode,
                threshold=args.event_gate_threshold,
                tau=args.event_gate_tau,
            )
            event_gate_error_sum += error_mag
            event_gate_sum += event_gate
            event_gate_active_sum += event_active
            hr.ema_update(
                hr.get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                z=z1,
                y=y_tr[jj],
                eta_p=float(args.eta_p) * float(event_gate),
                eta_m=float(args.eta_m) * float(event_gate),
            )
            timings["training_update"] += time.perf_counter() - t_train_update
            step_ctr += 1
    timings["training_ema"] = time.perf_counter() - t_train
    if total_steps > 0:
        event_gate_error_mean = float(event_gate_error_sum / float(total_steps))
        event_gate_mean = float(event_gate_sum / float(total_steps))
        event_gate_active_frac = float(event_gate_active_sum / float(total_steps))
        event_gate_cost_proxy = float(event_gate_mean)

    mse0, pmax0, H0, nb0 = hr.eval_metrics(buckets, keys_ev, z_ev, y_ev)

    t_growth = time.perf_counter()
    created, accepted = hr.loss_based_splitting(
        buckets=buckets,
        keys=keys_tr_eval,
        z=z_tr_eval,
        y=y_tr,
        max_slots_per_bucket=args.max_slots_per_bucket,
        extra_budget=args.extra_budget,
        eta_p=args.eta_p,
        eta_m=args.eta_m,
        n_split_rounds=args.split_rounds,
        min_gain=args.min_split_gain,
        seed=args.seed + 303,
        d=d,
        dy=dy
    )
    timings["growth"] = time.perf_counter() - t_growth

    mse1, pmax1, H1, nb1 = hr.eval_metrics(buckets, keys_ev, z_ev, y_ev)
    slots_used = int(sum(len(b.slots) for b in buckets.values()))
    timings["total"] = time.perf_counter() - t_total_start

    print("=== PROXY RUN SUMMARY ===")
    print(f"input={args.input} eval_split={args.eval_split} seed={args.seed}")
    print(
        f"sector_mode={args.sector_mode} phase_dims={args.phase_dims} "
        f"phase4_dims={args.phase4_dims} field4_dims={args.field4_dims} "
        f"complex_dims={args.complex_dims}"
    )
    print(
        f"time_pressure_lambda={args.time_pressure_lambda} "
        f"train_route_mode={args.train_route_mode} fast_dev={args.fast_dev}"
    )
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        print(
            "adaptive_phase4d="
            f"min_pair_bins={args.adaptive_min_pair_bins} "
            f"time_growth={args.adaptive_time_growth} "
            f"balance={args.adaptive_balance} "
            f"angle_growth={args.adaptive_angle_growth} "
            f"shell_growth={args.adaptive_shell_growth} "
            f"shell_balance={args.adaptive_shell_balance} "
            f"converge_lambda={args.adaptive_converge_lambda} "
            f"converge_target={args.adaptive_converge_target} "
            f"converge_hysteresis={args.adaptive_converge_hysteresis} converge_mode={args.adaptive_converge_mode} "
            f"memory_coord_mode={args.memory_coord_mode} "
            f"shell_mode={args.shell_mode} shell_phase_coupling={args.shell_phase_coupling}"
        )
    if args.sector_mode == "phase4d_hopf_blend":
        print(
            "adaptive_hopf_blend="
            f"lambda={args.hopf_blend_lambda} "
            f"chi_weight={args.hopf_blend_chi_weight} "
            f"shell_weight={args.hopf_blend_shell_weight} "
            f"chi_bins={args.hopf_chi_bins}"
        )
    if args.sector_mode in ("phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase"):
        print(
            "phase_transport="
            f"lambda={args.phase_transport_lambda:.6f} "
            f"field_lambda={args.phase_field_lambda:.6f} "
            f"coherence={phase_transport_coherence:.6f} "
            f"shift_abs_mean={phase_transport_shift_abs_mean:.6f} "
            f"shift_abs_max={phase_transport_shift_abs_max:.6f} "
            f"conn_abs_mean={phase_transport_connection_abs_mean:.6f} "
            f"field_shift_abs_mean={phase_transport_field_shift_abs_mean:.6f} "
            f"field_weight_abs_mean={phase_transport_field_weight_abs_mean:.6f} "
            f"alpha_bins={phase_transport_alpha_bins:.6f}"
        )
    print(
        "event_gate="
        f"mode={args.event_gate_mode} "
        f"threshold={args.event_gate_threshold:.6f} "
        f"tau={args.event_gate_tau:.6f} "
        f"error_mean={event_gate_error_mean:.6f} "
        f"gate_mean={event_gate_mean:.6f} "
        f"active_frac={event_gate_active_frac:.6f} "
        f"cost_proxy={event_gate_cost_proxy:.6f}"
    )
    if args.sector_mode == "phase4d_complex_local":
        print(
            "hybrid_complex_local="
            f"local_k={args.hybrid_local_k} "
            f"local_min_k={args.hybrid_local_min_k} "
            f"local_target={args.hybrid_local_target} "
            f"local_hysteresis={args.hybrid_local_hysteresis} "
            f"local_converge_lambda={args.hybrid_local_converge_lambda} "
            f"roots={args.hybrid_complex_roots}"
        )
    print(f"n_train={v_tr.shape[0]} n_eval={v_ev.shape[0]} d={d} dy={dy}")
    print(f"recluster_after_chart={args.recluster_after_chart} reclustered={reclustered}")
    print(f"timings_sec={timings}")
    print(f"eval_mse_before={mse0:.6f} eval_mse_after={mse1:.6f} unseen_rate={unseen_rate:.6f}")
    print(
        "poincare_alignment="
        f"pairs={poincare_alignment_pairs_used} "
        f"radial_mae={poincare_alignment_radial_mae:.6f} "
        f"radial_rel={poincare_alignment_radial_rel_mean:.6f} "
        f"radial_corr={poincare_alignment_radial_corr:.6f} "
        f"pair_mae={poincare_alignment_pair_mae:.6f} "
        f"pair_rel={poincare_alignment_pair_rel_mean:.6f} "
        f"pair_corr={poincare_alignment_pair_corr:.6f}"
    )
    print(
        "measure_consistency="
        f"shell_l1={shell_mass_error_l1:.6f} "
        f"shell_max={shell_mass_error_max:.6f} "
        f"shell_kl={shell_mass_kl:.6f} "
        f"shell_corr={shell_mass_corr:.6f} "
        f"hopf_mass={hopf_angular_mass_error:.6f} "
        f"hopf_base_mass={hopf_base_mass_error:.6f} "
        f"chi_mass={hopf_chi_mass_error:.6f} "
        f"delta_mass={hopf_delta_mass_error:.6f} "
        f"theta1_mass={hopf_theta1_mass_error:.6f} "
        f"theta2_mass={hopf_theta2_mass_error:.6f} "
        f"alpha_H={hopf_alpha_entropy:.6f} "
        f"theta1_H={hopf_theta1_entropy:.6f} "
        f"theta2_H={hopf_theta2_entropy:.6f} "
        f"route_H_r_corr={route_entropy_radius_corr:.6f} "
        f"route_H_r_slope={route_entropy_radius_slope:.6f} "
        f"knn_k={geodesic_knn_overlap_k:.0f} "
        f"knn_overlap={geodesic_knn_overlap_mean:.6f} "
        f"knn_jaccard={geodesic_knn_jaccard_mean:.6f}"
    )
    if args.sector_mode in ("phase4d_adaptive", "phase4d_hopf", "phase4d_hopf_base", "phase4d_hopf_transport", "phase4d_hopf_transport_complex", "phase4d_hopf_product_phase", "phase4d_hopf_iso", "phase4d_hopf_ball", "phase4d_hopf_base_ball", "phase4d_hopf_chi", "phase4d_hopf_fib", "phase4d_hopf_fib_rung", "phase4d_hopf_fib_band", "phase4d_hopf_fib_band_iso", "phase4d_hopf_fib_band_bound", "phase4d_hopf_blend", "phase4d_complex_local"):
        print(
            "adaptive_shell="
            f"drive_mean={adaptive_shell_drive_mean:.6f} "
            f"drive_max={adaptive_shell_drive_max:.6f} "
            f"expand_mean={adaptive_shell_expand_mean:.6f} "
            f"expand_max={adaptive_shell_expand_max:.6f} "
            f"target_band_mean={adaptive_shell_target_band_mean:.6f} "
            f"target_band_max={adaptive_shell_target_band_max:.6f} "
            f"overflow_mean={adaptive_shell_overflow_mean:.6f} "
            f"overflow_max={adaptive_shell_overflow_max:.6f} "
            f"ratio_mean={adaptive_shell_ratio_mean:.6f} "
            f"ratio_max={adaptive_shell_ratio_max:.6f} "
            f"ratio_pressure_mean={adaptive_shell_ratio_pressure_mean:.6f} "
            f"ratio_pressure_max={adaptive_shell_ratio_pressure_max:.6f} "
            f"ladder_steps_mean={adaptive_shell_ladder_steps_mean:.6f} "
            f"ladder_steps_max={adaptive_shell_ladder_steps_max} "
            f"converge_mean={adaptive_shell_converge_mean:.6f} "
            f"converge_max={adaptive_shell_converge_max:.6f} "
            f"phase_gap_abs_mean={adaptive_phase_gap_abs_mean:.6f} "
            f"phase_gap_abs_max={adaptive_phase_gap_abs_max:.6f} "
            f"phase_bias_mean={adaptive_shell_phase_bias_mean:.6f} "
            f"phase_bias_max={adaptive_shell_phase_bias_max:.6f} "
            f"mult_mean={adaptive_shell_mult_mean:.6f} "
            f"mult_max={adaptive_shell_mult_max:.6f}"
        )
        print(
            "adaptive_hopf="
            f"chi_mean={adaptive_chi_mean:.6f} "
            f"chi_entropy={adaptive_chi_entropy:.6f} "
            f"r_alpha_mean={adaptive_r_alpha_mean:.6f} "
            f"r_alpha_max={adaptive_r_alpha_max:.6f} "
            f"shell_capacity_mean={adaptive_hopf_shell_capacity_mean:.6f} "
            f"shell_capacity_max={adaptive_hopf_shell_capacity_max:.6f} "
            f"hopf_k1_mean={adaptive_hopf_k1_mean:.6f} "
            f"hopf_k2_mean={adaptive_hopf_k2_mean:.6f} "
            f"hopf_k1_max={adaptive_hopf_k1_max} "
            f"hopf_k2_max={adaptive_hopf_k2_max} "
            f"hopf_k1_gap_mean={adaptive_hopf_k1_gap_mean:.6f} "
            f"hopf_k2_gap_mean={adaptive_hopf_k2_gap_mean:.6f} "
            f"chi_bins_used={adaptive_chi_bins_used} "
            f"chi_bin_pmax={adaptive_chi_bin_pmax:.6f} "
            f"chi_bin_entropy={adaptive_chi_bin_entropy:.6f} "
            f"fib_total_mean={adaptive_fib_total_mean:.6f} "
            f"fib_total_max={adaptive_fib_total_max} "
            f"fib_kchi_mean={adaptive_fib_kchi_mean:.6f} "
            f"fib_kchi_max={adaptive_fib_kchi_max} "
            f"fib_forced_total_mean={adaptive_fib_forced_total_mean:.6f} "
            f"fib_forced_total_max={adaptive_fib_forced_total_max} "
            f"fib_band_mean={adaptive_fib_band_mean:.6f} "
            f"fib_band_max={adaptive_fib_band_max} "
            f"fib_band_entropy={adaptive_fib_band_entropy:.6f} "
            f"fib_band_states_used={adaptive_fib_band_states_used} "
            f"blend_total_mean={adaptive_blend_total_mean:.6f} "
            f"blend_total_max={adaptive_blend_total_max} "
            f"blend_score_mean={adaptive_blend_score_mean:.6f} "
            f"blend_score_max={adaptive_blend_score_max:.6f} "
            f"blend_chi_pressure_mean={adaptive_blend_chi_pressure_mean:.6f} "
            f"blend_chi_pressure_max={adaptive_blend_chi_pressure_max:.6f} "
            f"blend_shell_pressure_mean={adaptive_blend_shell_pressure_mean:.6f} "
            f"blend_shell_pressure_max={adaptive_blend_shell_pressure_max:.6f}"
        )
        print(
            "hopf_sector="
            f"groups={hopf_sector_groups_used} "
            f"chi_std_mean={hopf_sector_chi_std_mean:.6f} "
            f"delta_cvar_mean={hopf_sector_delta_cvar_mean:.6f} "
            f"alpha_entropy_mean={hopf_sector_alpha_entropy_mean:.6f} "
            f"alpha_entropy_gap={hopf_sector_alpha_entropy_gap:.6f}"
        )
        if args.sector_mode == "phase4d_hopf_product_phase":
            print(
                "product_shell="
                f"mode={args.product_shell_control_mode} "
                f"gate_threshold={args.product_shell_gate_threshold:.6f} "
                f"gate_score_mean={product_shell_gate_score_mean:.6f} "
                f"gate_score_max={product_shell_gate_score_max:.6f} "
                f"active_frac={product_shell_active_frac:.6f} "
                f"states_used={int(product_shell_states_used)} "
                f"mult_mean={product_shell_multiplier_mean:.6f} "
                f"mult_max={product_shell_multiplier_max:.6f}"
            )
    if args.sector_mode == "phase4d_complex_local":
        print(
            "hybrid_local="
            f"coarse_eval_sectors={hybrid_coarse_eval_sectors} "
            f"local_eval_sectors={hybrid_local_eval_sectors} "
            f"local_pmax={hybrid_local_pmax:.6f} "
            f"local_entropy={hybrid_local_entropy:.6f} "
            f"drive_mean={hybrid_local_drive_mean:.6f} "
            f"drive_max={hybrid_local_drive_max:.6f} "
            f"target_band_mean={hybrid_local_target_band_mean:.6f} "
            f"target_band_max={hybrid_local_target_band_max:.6f} "
            f"ratio_mean={hybrid_local_ratio_mean:.6f} "
            f"ratio_max={hybrid_local_ratio_max:.6f} "
            f"ratio_pressure_mean={hybrid_local_ratio_pressure_mean:.6f} "
            f"ratio_pressure_max={hybrid_local_ratio_pressure_max:.6f} "
            f"converge_mean={hybrid_local_converge_mean:.6f} "
            f"converge_max={hybrid_local_converge_max:.6f} "
            f"activation_mean={hybrid_local_activation_mean:.6f} "
            f"activation_max={hybrid_local_activation_max:.6f} "
            f"k_eff_mean={hybrid_local_k_eff_mean:.6f} "
            f"k_eff_max={hybrid_local_k_eff_max}"
        )

    summary = {
        "schema_version": "1.0",
        "parsed": True,
        "args": {k: v for k, v in vars(args).items()},
        "metrics": {
            "test_mse_before": float(mse0),
            "test_mse_after": float(mse1),
            "train_label_sse_per": float(train_label_sse_per),
            "test_label_sse_per": float(eval_label_sse_per),
            "buckets": int(nb1),
            "slots_used": int(slots_used),
            "test_unseen_rate": float(unseen_rate),
            "pmax_before": float(pmax0),
            "pmax_after": float(pmax1),
            "entropy_before": float(H0),
            "entropy_after": float(H1),
            "eval_shells": int(len(shell_vals)),
            "eval_sectors": int(len(sector_vals)),
            "shell_pmax": float(shell_pmax),
            "sector_pmax": float(sector_pmax),
            "shell_entropy": float(shell_entropy),
            "sector_entropy": float(sector_entropy),
            "poincare_alignment_pairs_used": int(poincare_alignment_pairs_used),
            "poincare_alignment_radial_mae": float(poincare_alignment_radial_mae),
            "poincare_alignment_radial_rel_mean": float(poincare_alignment_radial_rel_mean),
            "poincare_alignment_radial_corr": float(poincare_alignment_radial_corr),
            "poincare_alignment_pair_mae": float(poincare_alignment_pair_mae),
            "poincare_alignment_pair_rel_mean": float(poincare_alignment_pair_rel_mean),
            "poincare_alignment_pair_corr": float(poincare_alignment_pair_corr),
            "shell_mass_error_l1": float(shell_mass_error_l1),
            "shell_mass_error_max": float(shell_mass_error_max),
            "shell_mass_kl": float(shell_mass_kl),
            "shell_mass_corr": float(shell_mass_corr),
            "shell_mass_shells_used": int(shell_mass_shells_used),
            "hopf_angular_mass_error": float(hopf_angular_mass_error),
            "hopf_base_mass_error": float(hopf_base_mass_error),
            "hopf_chi_mass_error": float(hopf_chi_mass_error),
            "hopf_delta_mass_error": float(hopf_delta_mass_error),
            "hopf_theta1_mass_error": float(hopf_theta1_mass_error),
            "hopf_theta2_mass_error": float(hopf_theta2_mass_error),
            "hopf_alpha_entropy": float(hopf_alpha_entropy),
            "phase_transport_coherence": float(phase_transport_coherence),
            "phase_transport_shift_abs_mean": float(phase_transport_shift_abs_mean),
            "phase_transport_shift_abs_max": float(phase_transport_shift_abs_max),
            "phase_transport_connection_abs_mean": float(phase_transport_connection_abs_mean),
            "phase_transport_field_shift_abs_mean": float(phase_transport_field_shift_abs_mean),
            "phase_transport_field_weight_abs_mean": float(phase_transport_field_weight_abs_mean),
            "phase_transport_alpha_bins": float(phase_transport_alpha_bins),
            "event_gate_error_mean": float(event_gate_error_mean),
            "event_gate_mean": float(event_gate_mean),
            "event_gate_active_frac": float(event_gate_active_frac),
            "event_gate_cost_proxy": float(event_gate_cost_proxy),
            "product_shell_gate_score_mean": float(product_shell_gate_score_mean),
            "product_shell_gate_score_max": float(product_shell_gate_score_max),
            "product_shell_active_frac": float(product_shell_active_frac),
            "product_shell_states_used": float(product_shell_states_used),
            "product_shell_multiplier_mean": float(product_shell_multiplier_mean),
            "product_shell_multiplier_max": float(product_shell_multiplier_max),
            "hopf_sector_groups_used": int(hopf_sector_groups_used),
            "hopf_sector_chi_std_mean": float(hopf_sector_chi_std_mean),
            "hopf_sector_delta_cvar_mean": float(hopf_sector_delta_cvar_mean),
            "hopf_sector_alpha_entropy_mean": float(hopf_sector_alpha_entropy_mean),
            "hopf_sector_alpha_entropy_gap": float(hopf_sector_alpha_entropy_gap),
            "hopf_theta1_entropy": float(hopf_theta1_entropy),
            "hopf_theta2_entropy": float(hopf_theta2_entropy),
            "route_entropy_radius_corr": float(route_entropy_radius_corr),
            "route_entropy_radius_slope": float(route_entropy_radius_slope),
            "route_entropy_shells_used": int(route_entropy_shells_used),
            "geodesic_knn_overlap_k": float(geodesic_knn_overlap_k),
            "geodesic_knn_overlap_mean": float(geodesic_knn_overlap_mean),
            "geodesic_knn_jaccard_mean": float(geodesic_knn_jaccard_mean),
            "geodesic_knn_points_used": int(geodesic_knn_points_used),
            "adaptive_k1_mean": float(adaptive_k1_mean),
            "adaptive_k2_mean": float(adaptive_k2_mean),
            "adaptive_k1_max": int(adaptive_k1_max),
            "adaptive_k2_max": int(adaptive_k2_max),
            "adaptive_shell_drive_mean": float(adaptive_shell_drive_mean),
            "adaptive_shell_drive_max": float(adaptive_shell_drive_max),
            "adaptive_shell_expand_mean": float(adaptive_shell_expand_mean),
            "adaptive_shell_expand_max": float(adaptive_shell_expand_max),
            "adaptive_shell_target_band_mean": float(adaptive_shell_target_band_mean),
            "adaptive_shell_target_band_max": float(adaptive_shell_target_band_max),
            "adaptive_shell_overflow_mean": float(adaptive_shell_overflow_mean),
            "adaptive_shell_overflow_max": float(adaptive_shell_overflow_max),
            "adaptive_shell_ratio_mean": float(adaptive_shell_ratio_mean),
            "adaptive_shell_ratio_max": float(adaptive_shell_ratio_max),
            "adaptive_shell_ratio_pressure_mean": float(adaptive_shell_ratio_pressure_mean),
            "adaptive_shell_ratio_pressure_max": float(adaptive_shell_ratio_pressure_max),
            "adaptive_shell_ladder_steps_mean": float(adaptive_shell_ladder_steps_mean),
            "adaptive_shell_ladder_steps_max": int(adaptive_shell_ladder_steps_max),
            "adaptive_shell_converge_mean": float(adaptive_shell_converge_mean),
            "adaptive_shell_converge_max": float(adaptive_shell_converge_max),
            "adaptive_phase_gap_abs_mean": float(adaptive_phase_gap_abs_mean),
            "adaptive_phase_gap_abs_max": float(adaptive_phase_gap_abs_max),
            "adaptive_shell_phase_bias_mean": float(adaptive_shell_phase_bias_mean),
            "adaptive_shell_phase_bias_max": float(adaptive_shell_phase_bias_max),
            "adaptive_shell_mult_mean": float(adaptive_shell_mult_mean),
            "adaptive_shell_mult_max": float(adaptive_shell_mult_max),
            "adaptive_chi_mean": float(adaptive_chi_mean),
            "adaptive_chi_entropy": float(adaptive_chi_entropy),
            "adaptive_r_alpha_mean": float(adaptive_r_alpha_mean),
            "adaptive_r_alpha_max": float(adaptive_r_alpha_max),
            "adaptive_hopf_shell_capacity_mean": float(adaptive_hopf_shell_capacity_mean),
            "adaptive_hopf_shell_capacity_max": float(adaptive_hopf_shell_capacity_max),
            "adaptive_hopf_k1_mean": float(adaptive_hopf_k1_mean),
            "adaptive_hopf_k2_mean": float(adaptive_hopf_k2_mean),
            "adaptive_hopf_k1_max": int(adaptive_hopf_k1_max),
            "adaptive_hopf_k2_max": int(adaptive_hopf_k2_max),
            "adaptive_hopf_k1_gap_mean": float(adaptive_hopf_k1_gap_mean),
            "adaptive_hopf_k2_gap_mean": float(adaptive_hopf_k2_gap_mean),
            "adaptive_chi_bins_used": int(adaptive_chi_bins_used),
            "adaptive_chi_bin_pmax": float(adaptive_chi_bin_pmax),
            "adaptive_chi_bin_entropy": float(adaptive_chi_bin_entropy),
            "adaptive_fib_total_mean": float(adaptive_fib_total_mean),
            "adaptive_fib_total_max": int(adaptive_fib_total_max),
            "adaptive_fib_kchi_mean": float(adaptive_fib_kchi_mean),
            "adaptive_fib_kchi_max": int(adaptive_fib_kchi_max),
            "adaptive_fib_forced_total_mean": float(adaptive_fib_forced_total_mean),
            "adaptive_fib_forced_total_max": int(adaptive_fib_forced_total_max),
            "adaptive_fib_band_mean": float(adaptive_fib_band_mean),
            "adaptive_fib_band_max": int(adaptive_fib_band_max),
            "adaptive_fib_band_entropy": float(adaptive_fib_band_entropy),
            "adaptive_fib_band_states_used": int(adaptive_fib_band_states_used),
            "adaptive_blend_total_mean": float(adaptive_blend_total_mean),
            "adaptive_blend_total_max": int(adaptive_blend_total_max),
            "adaptive_blend_score_mean": float(adaptive_blend_score_mean),
            "adaptive_blend_score_max": float(adaptive_blend_score_max),
            "adaptive_blend_chi_pressure_mean": float(adaptive_blend_chi_pressure_mean),
            "adaptive_blend_chi_pressure_max": float(adaptive_blend_chi_pressure_max),
            "adaptive_blend_shell_pressure_mean": float(adaptive_blend_shell_pressure_mean),
            "adaptive_blend_shell_pressure_max": float(adaptive_blend_shell_pressure_max),
            "hybrid_coarse_eval_sectors": int(hybrid_coarse_eval_sectors),
            "hybrid_local_eval_sectors": int(hybrid_local_eval_sectors),
            "hybrid_local_pmax": float(hybrid_local_pmax),
            "hybrid_local_entropy": float(hybrid_local_entropy),
            "hybrid_local_drive_mean": float(hybrid_local_drive_mean),
            "hybrid_local_drive_max": float(hybrid_local_drive_max),
            "hybrid_local_target_band_mean": float(hybrid_local_target_band_mean),
            "hybrid_local_target_band_max": float(hybrid_local_target_band_max),
            "hybrid_local_ratio_mean": float(hybrid_local_ratio_mean),
            "hybrid_local_ratio_max": float(hybrid_local_ratio_max),
            "hybrid_local_ratio_pressure_mean": float(hybrid_local_ratio_pressure_mean),
            "hybrid_local_ratio_pressure_max": float(hybrid_local_ratio_pressure_max),
            "hybrid_local_converge_mean": float(hybrid_local_converge_mean),
            "hybrid_local_converge_max": float(hybrid_local_converge_max),
            "hybrid_local_activation_mean": float(hybrid_local_activation_mean),
            "hybrid_local_activation_max": float(hybrid_local_activation_max),
            "hybrid_local_k_eff_mean": float(hybrid_local_k_eff_mean),
            "hybrid_local_k_eff_max": int(hybrid_local_k_eff_max),
            "new_slots": int(created),
            "accepted_splits": int(accepted),
            "n_buckets_total": int(len(buckets)),
        },
        "timings_sec": {k: float(v) for k, v in timings.items()},
        "artifacts": {
            "input": args.input,
            "eval_split": args.eval_split,
            "cache_dir": args.cache_dir,
            "chart_cache_file": chart_cache_file,
            "route_cache_file": route_cache_file,
            "run_tag": args.run_tag,
        },
        "git": hr.maybe_git_info(),
        "notes": notes,
    }
    print("__JSON_SUMMARY__ " + json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
