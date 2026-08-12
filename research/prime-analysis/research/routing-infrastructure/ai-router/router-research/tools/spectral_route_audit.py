#!/usr/bin/env python3
import argparse
import json
import os
import sys
from typing import Any, Dict, Iterable, List, Tuple

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

    input_transform = getattr(args, "input_transform", "none")
    if input_transform != "none":
        rng = np.random.RandomState(args.seed + 77)
        if input_transform == "col_perm":
            for j in range(v_tr.shape[1]):
                v_tr[:, j] = rng.permutation(v_tr[:, j])
            for j in range(v_ev.shape[1]):
                v_ev[:, j] = rng.permutation(v_ev[:, j])
        elif input_transform == "gaussian":
            mu = v_tr.mean(axis=0)
            sd = v_tr.std(axis=0) + 1e-12
            v_tr = rng.randn(*v_tr.shape) * sd + mu
            v_ev = rng.randn(*v_ev.shape) * sd + mu

    return v_tr, y_tr, v_ev, y_ev


def _validated_dims(args: argparse.Namespace, d: int) -> Tuple[int, ...]:
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
    return (
        phase_dim_i,
        phase_dim_j,
        phase4_dim_i,
        phase4_dim_j,
        phase4_dim_k,
        phase4_dim_l,
        field4_dim_i,
        field4_dim_j,
        field4_dim_k,
        field4_dim_l,
        complex_dim_i,
        complex_dim_j,
    )


def _routing_kwargs(
    args: argparse.Namespace,
    c_used: np.ndarray,
    chart: hr.Chart,
    dims: Tuple[int, ...],
    time_pressure_lambda: float,
    tau: float,
) -> Dict[str, Any]:
    (
        phase_dim_i,
        phase_dim_j,
        phase4_dim_i,
        phase4_dim_j,
        phase4_dim_k,
        phase4_dim_l,
        field4_dim_i,
        field4_dim_j,
        field4_dim_k,
        field4_dim_l,
        complex_dim_i,
        complex_dim_j,
    ) = dims
    return {
        "delta_r": args.delta_r,
        "C": c_used,
        "chart": chart,
        "sector_mode": args.sector_mode,
        "phase_dim_i": phase_dim_i,
        "phase_dim_j": phase_dim_j,
        "phase4_dim_i": phase4_dim_i,
        "phase4_dim_j": phase4_dim_j,
        "phase4_dim_k": phase4_dim_k,
        "phase4_dim_l": phase4_dim_l,
        "field4_dim_i": field4_dim_i,
        "field4_dim_j": field4_dim_j,
        "field4_dim_k": field4_dim_k,
        "field4_dim_l": field4_dim_l,
        "complex_dim_i": complex_dim_i,
        "complex_dim_j": complex_dim_j,
        "K": args.K,
        "time_pressure_lambda": time_pressure_lambda,
        "tau": tau,
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
        "fib_rung_gate_threshold": args.fib_rung_gate_threshold,
        "route_scale_lambda": args.route_scale_lambda,
        "memory_coord_mode": args.memory_coord_mode,
        "shell_mode": args.shell_mode,
        "shell_phase_coupling": args.shell_phase_coupling,
        "hopf_chi_bins": args.hopf_chi_bins,
        "hopf_blend_lambda": args.hopf_blend_lambda,
        "hopf_blend_chi_weight": args.hopf_blend_chi_weight,
        "hopf_blend_shell_weight": args.hopf_blend_shell_weight,
        "phase_transport_lambda": args.phase_transport_lambda,
        "phase_field_lambda": args.phase_field_lambda,
        "hybrid_local_k": args.hybrid_local_k,
        "hybrid_complex_roots": args.hybrid_complex_roots,
        "hybrid_local_min_k": args.hybrid_local_min_k,
        "hybrid_local_target": args.hybrid_local_target,
        "hybrid_local_hysteresis": args.hybrid_local_hysteresis,
        "hybrid_local_converge_lambda": args.hybrid_local_converge_lambda,
    }


def _optimized_chart_and_centers(
    args: argparse.Namespace,
    v_tr: np.ndarray,
    y_tr: np.ndarray,
    d: int,
    dims: Tuple[int, ...],
) -> Tuple[hr.Chart, np.ndarray]:
    (
        phase_dim_i,
        phase_dim_j,
        phase4_dim_i,
        phase4_dim_j,
        phase4_dim_k,
        phase4_dim_l,
        field4_dim_i,
        field4_dim_j,
        field4_dim_k,
        field4_dim_l,
        complex_dim_i,
        complex_dim_j,
    ) = dims

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
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            K=args.K,
            seed=args.seed + 23,
            early_stop_patience=args.early_stop_patience,
            early_stop_min_delta=args.early_stop_min_delta,
            field4_dim_i=field4_dim_i,
            field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k,
            field4_dim_l=field4_dim_l,
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
    return chart, c_used


def _prepared_route_state(args: argparse.Namespace) -> Dict[str, Any]:
    v_tr, y_tr, v_ev, y_ev = load_proxy_subset(args)
    d = int(v_tr.shape[1])
    dy = int(y_tr.shape[1])
    dims = _validated_dims(args, d)
    chart, c_used = _optimized_chart_and_centers(args, v_tr, y_tr, d, dims)
    route_kwargs = _routing_kwargs(args, c_used, chart, dims, time_pressure_lambda=0.0, tau=1.0)
    shell_tr_eval, sector_tr_eval, _, z_tr_eval = hr.route_addresses(v_tr, **route_kwargs)
    shell_ev, sector_ev, _, z_ev = hr.route_addresses(v_ev, **route_kwargs)
    route_z_ev = hr.route_coordinate(v_ev, chart, sector_mode=args.sector_mode, route_scale_lambda=args.route_scale_lambda)
    return {
        "v_tr": v_tr,
        "y_tr": y_tr,
        "v_ev": v_ev,
        "y_ev": y_ev,
        "d": d,
        "dy": dy,
        "dims": dims,
        "chart": chart,
        "c_used": c_used,
        "shell_tr_eval": shell_tr_eval,
        "sector_tr_eval": sector_tr_eval,
        "z_tr_eval": z_tr_eval,
        "shell_ev": shell_ev,
        "sector_ev": sector_ev,
        "z_ev": z_ev,
        "route_z_ev": route_z_ev,
    }


def _sample_indices(n_points: int, max_points: int, seed: int) -> np.ndarray:
    if max_points > 0 and n_points > max_points:
        rs = np.random.RandomState(seed + 991)
        return np.sort(rs.permutation(n_points)[:max_points])
    return np.arange(n_points, dtype=np.int64)


def route_eval_snapshot(args: argparse.Namespace, max_points: int) -> Dict[str, np.ndarray]:
    state = _prepared_route_state(args)
    idx = _sample_indices(state["route_z_ev"].shape[0], max_points=max_points, seed=args.seed)

    return {
        "route_z": state["route_z_ev"][idx].astype(np.float64),
        "shell": state["shell_ev"][idx].astype(np.int64),
        "sector": state["sector_ev"][idx].astype(np.int64),
        "y_eval": state["y_ev"][idx].astype(np.float64),
        "v_ev": state["v_ev"][idx].astype(np.float64),
        "dims": state["dims"],
    }


def task_eval_snapshot(args: argparse.Namespace, max_points: int) -> Dict[str, np.ndarray]:
    state = _prepared_route_state(args)
    v_tr = state["v_tr"]
    y_tr = state["y_tr"]
    y_ev = state["y_ev"]
    d = int(state["d"])
    dy = int(state["dy"])
    dims = state["dims"]
    chart = state["chart"]
    c_used = state["c_used"]
    z_tr_eval = state["z_tr_eval"]
    z_ev = state["z_ev"]
    keys_tr_eval = [
        hr.make_bucket_key(int(state["shell_tr_eval"][i]), int(state["sector_tr_eval"][i]))
        for i in range(v_tr.shape[0])
    ]
    keys_ev = [
        hr.make_bucket_key(int(state["shell_ev"][i]), int(state["sector_ev"][i]))
        for i in range(state["v_ev"].shape[0])
    ]

    buckets = hr.init_buckets(keys_tr_eval, dy=dy, d=d, seed=args.seed + 101)
    total_steps = int(args.epochs) * int(v_tr.shape[0])
    step_ctr = 0
    for _epoch in range(int(args.epochs)):
        order = np.random.permutation(v_tr.shape[0])
        for jj in order:
            tau = 1.0 if total_steps <= 1 else float(step_ctr) / float(total_steps - 1)
            if args.train_route_mode == "final_static":
                key = keys_tr_eval[jj]
                z1 = z_tr_eval[jj]
            else:
                key, z1 = hr.route_one(
                    v_tr[jj],
                    **_routing_kwargs(
                        args,
                        c_used,
                        chart,
                        dims,
                        time_pressure_lambda=float(args.time_pressure_lambda),
                        tau=tau,
                    ),
                )
            _, sj, _ = hr.predict_from_bucket(buckets, key, z1, d=d, dy=dy)
            hr.ema_update(
                hr.get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                z=z1,
                y=y_tr[jj],
                eta_p=args.eta_p,
                eta_m=args.eta_m,
            )
            step_ctr += 1

    hr.loss_based_splitting(
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
        dy=dy,
    )

    yhat_ev = np.zeros_like(y_ev, dtype=np.float64)
    slot_ev = np.zeros(y_ev.shape[0], dtype=np.int64)
    proto_d2_ev = np.zeros(y_ev.shape[0], dtype=np.float64)
    for idx_eval, key in enumerate(keys_ev):
        yhat, slot_idx, proto_d2 = hr.predict_from_bucket(buckets, key, z_ev[idx_eval], d=d, dy=dy)
        yhat_ev[idx_eval] = yhat
        slot_ev[idx_eval] = int(slot_idx)
        proto_d2_ev[idx_eval] = float(proto_d2)

    idx = _sample_indices(state["route_z_ev"].shape[0], max_points=max_points, seed=args.seed)
    return {
        "route_z": state["route_z_ev"][idx].astype(np.float64),
        "shell": state["shell_ev"][idx].astype(np.int64),
        "sector": state["sector_ev"][idx].astype(np.int64),
        "y_eval": y_ev[idx].astype(np.float64),
        "yhat_eval": yhat_ev[idx].astype(np.float64),
        "slot_eval": slot_ev[idx].astype(np.int64),
        "proto_d2_eval": proto_d2_ev[idx].astype(np.float64),
        "v_ev": state["v_ev"][idx].astype(np.float64),
        "dims": state["dims"],
    }


def pairwise_distances(x: np.ndarray) -> np.ndarray:
    x2 = np.sum(x * x, axis=1, keepdims=True)
    d2 = np.maximum(x2 + x2.T - 2.0 * (x @ x.T), 0.0)
    return np.sqrt(d2).astype(np.float64)


def hopf_coords_from_raw(
    v: np.ndarray,
    dims: Tuple[int, ...],
) -> np.ndarray:
    """Extract Hopf coordinates as a 5D embedding respecting angular wrapping.

    Returns (chi_u, cos(delta), sin(delta), cos(alpha), sin(alpha)) per point.
    chi_u ∈ [0,1] is natural; delta, alpha are embedded on S^1 to handle wrap.
    """
    phase4_dim_i = dims[2]
    phase4_dim_j = dims[3]
    phase4_dim_k = dims[4]
    phase4_dim_l = dims[5]
    comp = hr.hopf_coordinate_components(
        v, dim_i=phase4_dim_i, dim_j=phase4_dim_j,
        dim_k=phase4_dim_k, dim_l=phase4_dim_l,
    )
    chi_u = comp["chi_u"][:, None]
    cos_d = np.cos(comp["delta"])[:, None]
    sin_d = np.sin(comp["delta"])[:, None]
    cos_a = np.cos(comp["alpha"])[:, None]
    sin_a = np.sin(comp["alpha"])[:, None]
    return np.hstack([chi_u, cos_d, sin_d, cos_a, sin_a]).astype(np.float64)


def poincare_pairwise_distances(
    v: np.ndarray,
    dims: Tuple[int, ...],
) -> np.ndarray:
    """Compute pairwise Poincaré ball distance on the 4 Hopf routing dims."""
    phase4_dim_i = dims[2]
    phase4_dim_j = dims[3]
    phase4_dim_k = dims[4]
    phase4_dim_l = dims[5]
    sub = v[:, [phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l]].astype(np.float64)
    n = sub.shape[0]
    d = np.zeros((n, n), dtype=np.float64)
    for i in range(n):
        d[i, :] = hr.poincare_distance(sub[i:i+1, :], sub)
    return d


def build_knn_graph_from_distances(d: np.ndarray, knn_k: int) -> Tuple[np.ndarray, float]:
    """Build KNN graph from a precomputed distance matrix."""
    n = int(d.shape[0])
    if n < 2:
        return np.zeros((n, n), dtype=np.float64), 0.0
    d_work = d.copy()
    np.fill_diagonal(d_work, np.inf)
    k = max(1, min(int(knn_k), n - 1))
    nbr_idx = np.argpartition(d_work, kth=k - 1, axis=1)[:, :k]
    nbr_dist = np.take_along_axis(d_work, nbr_idx, axis=1)
    sigma = float(np.median(nbr_dist[np.isfinite(nbr_dist)]))
    sigma = max(sigma, 1e-8)
    a = np.zeros((n, n), dtype=np.float64)
    w = np.exp(-(nbr_dist * nbr_dist) / (2.0 * sigma * sigma))
    rows = np.repeat(np.arange(n), k)
    cols = nbr_idx.reshape(-1)
    vals = w.reshape(-1)
    a[rows, cols] = vals
    a = np.maximum(a, a.T)
    np.fill_diagonal(a, 0.0)
    return a, sigma


def build_knn_graph(x: np.ndarray, knn_k: int) -> Tuple[np.ndarray, float]:
    n = int(x.shape[0])
    if n < 2:
        return np.zeros((n, n), dtype=np.float64), 0.0
    d = pairwise_distances(x)
    np.fill_diagonal(d, np.inf)
    k = max(1, min(int(knn_k), n - 1))
    nbr_idx = np.argpartition(d, kth=k - 1, axis=1)[:, :k]
    nbr_dist = np.take_along_axis(d, nbr_idx, axis=1)
    sigma = float(np.median(nbr_dist[np.isfinite(nbr_dist)]))
    sigma = max(sigma, 1e-8)
    a = np.zeros((n, n), dtype=np.float64)
    w = np.exp(-(nbr_dist * nbr_dist) / (2.0 * sigma * sigma))
    rows = np.repeat(np.arange(n), k)
    cols = nbr_idx.reshape(-1)
    vals = w.reshape(-1)
    a[rows, cols] = vals
    a = np.maximum(a, a.T)
    np.fill_diagonal(a, 0.0)
    return a, sigma


def normalized_laplacian(a: np.ndarray) -> np.ndarray:
    deg = np.sum(a, axis=1)
    inv_sqrt = np.zeros_like(deg)
    mask = deg > 1e-12
    inv_sqrt[mask] = 1.0 / np.sqrt(deg[mask])
    s = inv_sqrt[:, None] * a * inv_sqrt[None, :]
    l = np.eye(a.shape[0], dtype=np.float64) - s
    l = 0.5 * (l + l.T)
    return l


def spectral_decomposition(
    route_z: np.ndarray,
    knn_k: int,
    graph_mode: str = "ambient_euclidean",
    v_ev: np.ndarray = None,
    dims: Tuple[int, ...] = None,
) -> Dict[str, np.ndarray]:
    if graph_mode == "hopf_coords":
        coords = hopf_coords_from_raw(v_ev, dims)
        a, sigma = build_knn_graph(coords, knn_k=knn_k)
    elif graph_mode == "poincare_4d":
        d = poincare_pairwise_distances(v_ev, dims)
        a, sigma = build_knn_graph_from_distances(d, knn_k=knn_k)
    else:
        a, sigma = build_knn_graph(route_z, knn_k=knn_k)
    l = normalized_laplacian(a)
    evals, evecs = np.linalg.eigh(l)
    evals = np.clip(evals.astype(np.float64), 0.0, None)
    return {
        "adjacency": a,
        "laplacian": l,
        "sigma": np.array(float(sigma), dtype=np.float64),
        "evals": evals,
        "evecs": evecs.astype(np.float64),
    }


def participation_ratio(vec: np.ndarray) -> float:
    norm = float(np.sum(vec * vec))
    if norm <= 1e-12:
        return 0.0
    unit = vec / np.sqrt(norm)
    return float(1.0 / (len(vec) * np.sum(np.power(unit, 4))))


def low_frequency_energy(evecs: np.ndarray, signal: np.ndarray, modes: int) -> float:
    if evecs.shape[0] == 0 or evecs.shape[1] <= 1:
        return 0.0
    f = signal.astype(np.float64)
    f = f - float(np.mean(f))
    norm = float(np.dot(f, f))
    if norm <= 1e-12:
        return 0.0
    use = min(max(int(modes), 0), max(evecs.shape[1] - 1, 0))
    if use == 0:
        return 0.0
    coeff = evecs[:, 1 : 1 + use].T @ f
    return float(np.dot(coeff, coeff) / norm)


def center_columns(signal: np.ndarray) -> np.ndarray:
    arr = np.asarray(signal, dtype=np.float64)
    if arr.ndim == 1:
        arr = arr[:, None]
    return arr - np.mean(arr, axis=0, keepdims=True)


def low_frequency_matrix_energy(evecs: np.ndarray, signal: np.ndarray, modes: int) -> float:
    if evecs.shape[0] == 0 or evecs.shape[1] <= 1:
        return 0.0
    centered = center_columns(signal)
    norm = float(np.sum(centered * centered))
    if norm <= 1e-12:
        return 0.0
    use = min(max(int(modes), 0), max(evecs.shape[1] - 1, 0))
    if use == 0:
        return 0.0
    basis = evecs[:, 1 : 1 + use]
    coeff = basis.T @ centered
    return float(np.sum(coeff * coeff) / norm)


def normalized_dirichlet_energy(laplacian: np.ndarray, signal: np.ndarray) -> float:
    centered = center_columns(signal)
    norm = float(np.sum(centered * centered))
    if norm <= 1e-12:
        return 0.0
    energy = float(np.sum(centered * (laplacian @ centered)))
    return float(energy / norm)


def spectral_metrics_from_decomposition(
    decomp: Dict[str, np.ndarray],
    route_z: np.ndarray,
    shell: np.ndarray,
    sector: np.ndarray,
    lowfreq_modes: int,
) -> Dict[str, Any]:
    a = decomp["adjacency"]
    l = decomp["laplacian"]
    sigma = float(decomp["sigma"])
    evals = decomp["evals"]
    evecs = decomp["evecs"]
    zero_eigs = int(np.count_nonzero(evals <= 1e-8))
    lambda2 = float(evals[1]) if len(evals) > 1 else 0.0
    lambda3 = float(evals[2]) if len(evals) > 2 else lambda2
    lambda4 = float(evals[3]) if len(evals) > 3 else lambda3
    positive = evals[evals > 1e-8]
    lowfreq_sum = float(np.sum(positive[: min(lowfreq_modes, len(positive))])) if len(positive) else 0.0
    total_sum = float(np.sum(positive)) if len(positive) else 0.0

    pr_vals: List[float] = []
    for kk in range(1, min(lowfreq_modes + 1, evecs.shape[1])):
        pr_vals.append(participation_ratio(evecs[:, kk]))

    return {
        "n_points": int(route_z.shape[0]),
        "graph_edges": int(np.count_nonzero(np.triu(a > 0.0, 1))),
        "graph_sigma": sigma,
        "spectral_zero_eigs": zero_eigs,
        "spectral_lambda2": lambda2,
        "spectral_lambda3": lambda3,
        "spectral_lambda4": lambda4,
        "spectral_gap_23": float(max(lambda3 - lambda2, 0.0)),
        "spectral_gap_34": float(max(lambda4 - lambda3, 0.0)),
        "spectral_lowfreq_mass_ratio": float(lowfreq_sum / total_sum) if total_sum > 1e-12 else 0.0,
        "spectral_mode_participation_mean": float(np.mean(pr_vals)) if pr_vals else 0.0,
        "spectral_mode_participation_min": float(np.min(pr_vals)) if pr_vals else 0.0,
        "shell_lowfreq_energy": low_frequency_energy(evecs, shell.astype(np.float64), modes=lowfreq_modes),
        "sector_lowfreq_energy": low_frequency_energy(evecs, sector.astype(np.float64), modes=lowfreq_modes),
        "spectral_eigenvalues_head": [float(x) for x in evals[: min(12, len(evals))]],
    }


def spectral_metrics(
    route_z: np.ndarray,
    shell: np.ndarray,
    sector: np.ndarray,
    knn_k: int,
    lowfreq_modes: int,
    graph_mode: str = "ambient_euclidean",
    v_ev: np.ndarray = None,
    dims: Tuple[int, ...] = None,
) -> Dict[str, Any]:
    decomp = spectral_decomposition(
        route_z, knn_k=knn_k, graph_mode=graph_mode, v_ev=v_ev, dims=dims,
    )
    return spectral_metrics_from_decomposition(
        decomp,
        route_z=route_z,
        shell=shell,
        sector=sector,
        lowfreq_modes=lowfreq_modes,
    )


def parse_route_ids(raw: str, available: Iterable[str]) -> List[str]:
    if raw.strip() == "":
        return list(available)
    want = [part.strip() for part in raw.split(",") if part.strip()]
    return want


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--routes", type=str, default="")
    ap.add_argument("--max-points", type=int, default=384)
    ap.add_argument("--knn-k", type=int, default=12)
    ap.add_argument("--lowfreq-modes", type=int, default=8)
    ap.add_argument("--graph-mode", type=str, default="ambient_euclidean",
                     choices=["ambient_euclidean", "hopf_coords", "poincare_4d"])
    ap.add_argument("--output", required=True)
    args = ap.parse_args()

    cfg = load_config(args.config)
    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    route_map = {route["route_id"]: route.get("args", {}) for route in routes}
    route_ids = parse_route_ids(args.routes, route_map.keys())

    results = []
    for route_id in route_ids:
        if route_id not in route_map:
            raise SystemExit(f"route_id {route_id!r} not found in config")
        route_args = build_args(common_args, route_map[route_id], args.seed)
        snap = route_eval_snapshot(route_args, max_points=args.max_points)
        metrics = spectral_metrics(
            snap["route_z"],
            snap["shell"],
            snap["sector"],
            knn_k=args.knn_k,
            lowfreq_modes=args.lowfreq_modes,
            graph_mode=args.graph_mode,
            v_ev=snap["v_ev"],
            dims=snap["dims"],
        )
        results.append(
            {
                "route_id": route_id,
                "args": {
                    "sector_mode": route_args.sector_mode,
                    "phase4_dims": route_args.phase4_dims,
                    "field4_dims": route_args.field4_dims,
                    "phase_field_lambda": float(route_args.phase_field_lambda),
                    "phase_transport_lambda": float(route_args.phase_transport_lambda),
                },
                "metrics": metrics,
            }
        )

    payload = {
        "config": args.config,
        "seed": int(args.seed),
        "max_points": int(args.max_points),
        "knn_k": int(args.knn_k),
        "lowfreq_modes": int(args.lowfreq_modes),
        "graph_mode": args.graph_mode,
        "results": results,
    }
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(payload, f, indent=2, sort_keys=True)
    print(json.dumps(payload, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
