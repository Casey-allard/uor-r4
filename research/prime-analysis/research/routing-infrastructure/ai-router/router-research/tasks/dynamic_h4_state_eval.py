#!/usr/bin/env python3
import argparse
import json
import os
import sys
import time
from typing import Dict, Sequence, Tuple

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr


SECTOR_CHOICES = [
    "kmeans",
    "phase2",
    "phase4d",
    "phase4d_adaptive",
    "phase4d_hopf",
    "phase4d_hopf_iso",
    "phase4d_hopf_ball",
    "phase4d_hopf_chi",
    "phase4d_hopf_fib",
    "phase4d_hopf_fib_rung",
    "phase4d_hopf_fib_band",
    "phase4d_hopf_fib_band_iso",
    "phase4d_hopf_fib_band_bound",
    "phase4d_hopf_blend",
    "phase4d_complex_local",
    "complex2",
]


def _contiguous_subset(X: np.ndarray, Y: np.ndarray, max_n: int, seed: int):
    total = int(X.shape[0])
    if max_n <= 0 or total <= max_n:
        start = 0
        stop = total
    else:
        rs = np.random.RandomState(seed)
        start = int(rs.randint(0, total - max_n + 1))
        stop = start + int(max_n)
    idx = np.arange(start, stop, dtype=np.int64)
    return X[start:stop], Y[start:stop], idx, start, stop


def pairwise_poincare_distance(X: np.ndarray, Y: np.ndarray) -> np.ndarray:
    x2 = np.clip(np.sum(X * X, axis=1, keepdims=True), 0.0, 1.0 - 1e-12)
    y2 = np.clip(np.sum(Y * Y, axis=1, keepdims=True).T, 0.0, 1.0 - 1e-12)
    dot = X @ Y.T
    diff2 = np.maximum(x2 + y2 - 2.0 * dot, 0.0)
    denom = np.maximum((1.0 - x2) * (1.0 - y2), 1e-12)
    arg = 1.0 + (2.0 * diff2 / denom)
    return np.arccosh(np.maximum(arg, 1.0))


def pairwise_sqeuclidean(X: np.ndarray, Y: np.ndarray) -> np.ndarray:
    x2 = np.sum(X * X, axis=1, keepdims=True)
    y2 = np.sum(Y * Y, axis=1, keepdims=True).T
    dot = X @ Y.T
    return np.maximum(x2 + y2 - 2.0 * dot, 0.0)


def build_bucket_index(keys: Sequence[Tuple[int, ...]]) -> Dict[Tuple[int, ...], np.ndarray]:
    bucket_to_idx: Dict[Tuple[int, ...], list] = {}
    for idx, key in enumerate(keys):
        bucket_to_idx.setdefault(key, []).append(idx)
    return {key: np.asarray(idxs, dtype=np.int64) for key, idxs in bucket_to_idx.items()}


def complex_key_ids(field: np.ndarray, dim_i: int, dim_j: int, roots: int, radius_bins: int) -> np.ndarray:
    roots = max(1, int(roots))
    radius_bins = max(1, int(radius_bins))
    qi = field[:, dim_i]
    qj = field[:, dim_j]
    theta = np.mod(np.arctan2(qj, qi), 2.0 * np.pi)
    angle_ids = np.minimum((theta * (float(roots) / (2.0 * np.pi))).astype(np.int64), roots - 1)
    if radius_bins <= 1 or field.shape[0] == 0:
        radius_ids = np.zeros((field.shape[0],), dtype=np.int64)
    else:
        radius = np.sqrt(qi * qi + qj * qj)
        edges = np.quantile(radius, np.linspace(0.0, 1.0, radius_bins + 1)[1:-1])
        radius_ids = np.searchsorted(edges, radius, side="right").astype(np.int64)
    return angle_ids + roots * radius_ids


def augment_route_keys_with_complex(
    base_keys: Sequence[Tuple[int, int]],
    field: np.ndarray,
    dim_i: int,
    dim_j: int,
    roots: int,
    radius_bins: int,
) -> Tuple[list, int]:
    complex_ids = complex_key_ids(field, dim_i=dim_i, dim_j=dim_j, roots=roots, radius_bins=radius_bins)
    return (
        [(int(key[0]), int(key[1]), int(complex_ids[i])) for i, key in enumerate(base_keys)],
        int(len(np.unique(complex_ids))) if complex_ids.size else 0,
    )


def pointwise_state_distance(
    pos_a: np.ndarray,
    pos_b: np.ndarray,
    flow_a: np.ndarray,
    flow_b: np.ndarray,
    flow_ball_a: np.ndarray,
    flow_ball_b: np.ndarray,
    dynamic_state_mode: str,
    flow_weight: float,
) -> np.ndarray:
    d_pos = hr.poincare_distance(pos_a, pos_b)
    if dynamic_state_mode == "static_h4":
        return d_pos
    if dynamic_state_mode == "tangent_h4":
        d_flow_sq = np.sum((flow_a - flow_b) ** 2, axis=1)
        return np.sqrt(np.maximum((d_pos * d_pos) + float(flow_weight) * d_flow_sq, 0.0))
    if dynamic_state_mode == "product_h4x_h4":
        d_flow = hr.poincare_distance(flow_ball_a, flow_ball_b)
        return np.sqrt(np.maximum((d_pos * d_pos) + float(flow_weight) * (d_flow * d_flow), 0.0))
    raise ValueError(f"Unknown dynamic_state_mode: {dynamic_state_mode}")


def build_flow_state(z: np.ndarray, idx: np.ndarray, flow_step: int, flow_scale: float) -> Tuple[np.ndarray, np.ndarray, float]:
    flow = np.zeros_like(z)
    step = max(1, int(flow_step))
    for i in range(z.shape[0]):
        src = int(idx[i])
        prev = src - step
        if prev >= 0 and i - step >= 0 and int(idx[i - step]) == prev:
            flow[i] = z[i] - z[i - step]
        else:
            flow[i] = 0.0
    raw_norm = hr.safe_norm(flow, axis=1, keepdims=False)
    norm_q95 = float(np.quantile(raw_norm, 0.95)) if flow.shape[0] else 1.0
    norm_q95 = max(norm_q95, 1e-8)
    flow_scaled = (float(flow_scale) / norm_q95) * flow
    flow_ball = hr.exp_map0(flow_scaled)
    return flow_scaled, flow_ball, norm_q95


def topk_predict_from_distances(
    dist: np.ndarray,
    train_y: np.ndarray,
    topk: int,
) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    kk = min(max(1, int(topk)), train_y.shape[0])
    idx = np.argpartition(dist, kth=kk - 1, axis=1)[:, :kk]
    part = np.take_along_axis(dist, idx, axis=1)
    order = np.argsort(part, axis=1)
    idx = np.take_along_axis(idx, order, axis=1)
    yhat = np.mean(train_y[idx], axis=1)
    pred = np.argmax(yhat, axis=1).astype(np.int64)
    knn_dist = np.mean(np.take_along_axis(dist, idx, axis=1), axis=1)
    return yhat, pred, knn_dist


def state_distance_matrix(
    eval_pos: np.ndarray,
    train_pos: np.ndarray,
    eval_flow: np.ndarray,
    train_flow: np.ndarray,
    eval_flow_ball: np.ndarray,
    train_flow_ball: np.ndarray,
    dynamic_state_mode: str,
    flow_weight: float,
) -> np.ndarray:
    d_pos = pairwise_poincare_distance(eval_pos, train_pos)
    if dynamic_state_mode == "static_h4":
        return d_pos
    if dynamic_state_mode == "tangent_h4":
        eval_flow_sq = np.sum(eval_flow * eval_flow, axis=1, keepdims=True)
        train_flow_sq = np.sum(train_flow * train_flow, axis=1, keepdims=True).T
        d_flow_sq = np.maximum(eval_flow_sq + train_flow_sq - 2.0 * (eval_flow @ train_flow.T), 0.0)
        return np.sqrt(np.maximum((d_pos * d_pos) + float(flow_weight) * d_flow_sq, 0.0))
    if dynamic_state_mode == "product_h4x_h4":
        d_flow = pairwise_poincare_distance(eval_flow_ball, train_flow_ball)
        return np.sqrt(np.maximum((d_pos * d_pos) + float(flow_weight) * (d_flow * d_flow), 0.0))
    raise ValueError(f"Unknown dynamic_state_mode: {dynamic_state_mode}")


def knn_state_predict(
    train_pos: np.ndarray,
    train_flow: np.ndarray,
    train_flow_ball: np.ndarray,
    train_y: np.ndarray,
    eval_pos: np.ndarray,
    eval_flow: np.ndarray,
    eval_flow_ball: np.ndarray,
    dynamic_state_mode: str,
    flow_weight: float,
    topk: int,
    block_size: int,
) -> Tuple[np.ndarray, np.ndarray, float]:
    yhat = np.zeros((eval_pos.shape[0], train_y.shape[1]), dtype=np.float64)
    pred = np.zeros((eval_pos.shape[0],), dtype=np.int64)
    knn_dist = np.zeros((eval_pos.shape[0],), dtype=np.float64)

    for start in range(0, eval_pos.shape[0], block_size):
        stop = min(start + block_size, eval_pos.shape[0])
        dist = state_distance_matrix(
            eval_pos=eval_pos[start:stop],
            train_pos=train_pos,
            eval_flow=eval_flow[start:stop],
            train_flow=train_flow,
            eval_flow_ball=eval_flow_ball[start:stop],
            train_flow_ball=train_flow_ball,
            dynamic_state_mode=dynamic_state_mode,
            flow_weight=flow_weight,
        )
        y_block, pred_block, knn_block = topk_predict_from_distances(dist, train_y, topk=topk)
        yhat[start:stop] = y_block
        pred[start:stop] = pred_block
        knn_dist[start:stop] = knn_block
    return yhat, pred, float(np.mean(knn_dist))


def knn_state_predict_bucketed(
    train_pos: np.ndarray,
    train_flow: np.ndarray,
    train_flow_ball: np.ndarray,
    train_y: np.ndarray,
    train_keys: Sequence[Tuple[int, ...]],
    eval_pos: np.ndarray,
    eval_flow: np.ndarray,
    eval_flow_ball: np.ndarray,
    eval_keys: Sequence[Tuple[int, ...]],
    dynamic_state_mode: str,
    flow_weight: float,
    topk: int,
    block_size: int,
) -> Tuple[np.ndarray, np.ndarray, float, float, float, float, float]:
    yhat = np.zeros((eval_pos.shape[0], train_y.shape[1]), dtype=np.float64)
    pred = np.zeros((eval_pos.shape[0],), dtype=np.int64)
    knn_dist = np.zeros((eval_pos.shape[0],), dtype=np.float64)
    candidate_counts = np.zeros((eval_pos.shape[0],), dtype=np.float64)
    used_bucket = np.zeros((eval_pos.shape[0],), dtype=np.float64)
    fallback = np.zeros((eval_pos.shape[0],), dtype=np.float64)

    bucket_to_train = build_bucket_index(train_keys)
    grouped_eval = build_bucket_index(eval_keys)
    global_idx = np.arange(train_pos.shape[0], dtype=np.int64)

    for key, eval_group in grouped_eval.items():
        cand_idx = bucket_to_train.get(key)
        bucket_hit = cand_idx is not None and cand_idx.size > 0
        if not bucket_hit:
            cand_idx = global_idx
        candidate_counts[eval_group] = float(cand_idx.shape[0])
        used_bucket[eval_group] = 1.0 if bucket_hit else 0.0
        fallback[eval_group] = 0.0 if bucket_hit else 1.0
        for start in range(0, eval_group.shape[0], block_size):
            block_idx = eval_group[start:start + block_size]
            dist = state_distance_matrix(
                eval_pos=eval_pos[block_idx],
                train_pos=train_pos[cand_idx],
                eval_flow=eval_flow[block_idx],
                train_flow=train_flow[cand_idx],
                eval_flow_ball=eval_flow_ball[block_idx],
                train_flow_ball=train_flow_ball[cand_idx],
                dynamic_state_mode=dynamic_state_mode,
                flow_weight=flow_weight,
            )
            y_block, pred_block, knn_block = topk_predict_from_distances(dist, train_y[cand_idx], topk=topk)
            yhat[block_idx] = y_block
            pred[block_idx] = pred_block
            knn_dist[block_idx] = knn_block

    return (
        yhat,
        pred,
        float(np.mean(knn_dist)),
        float(np.mean(candidate_counts)),
        float(np.mean(candidate_counts) / max(1.0, float(train_pos.shape[0]))),
        float(np.mean(used_bucket)),
        float(np.mean(fallback)),
    )


def sample_random_pair_indices(n: int, max_pairs: int, seed: int) -> Tuple[np.ndarray, np.ndarray]:
    if n <= 1 or max_pairs <= 0:
        return np.zeros((0,), dtype=np.int64), np.zeros((0,), dtype=np.int64)
    rs = np.random.RandomState(seed)
    i = rs.randint(0, n, size=max_pairs)
    j = rs.randint(0, n - 1, size=max_pairs)
    j = np.where(j >= i, j + 1, j)
    return i.astype(np.int64), j.astype(np.int64)


def state_coherence_metrics(
    pos: np.ndarray,
    flow: np.ndarray,
    flow_ball: np.ndarray,
    dynamic_state_mode: str,
    flow_weight: float,
    max_pairs: int,
    seed: int,
) -> Dict[str, float]:
    if pos.shape[0] <= 1:
        return {
            "dynamic_step_dist_mean": 0.0,
            "dynamic_step_dist_std": 0.0,
            "dynamic_random_pair_dist_mean": 0.0,
            "dynamic_step_to_random_ratio": 0.0,
        }
    step_dist = pointwise_state_distance(
        pos[1:], pos[:-1], flow[1:], flow[:-1], flow_ball[1:], flow_ball[:-1], dynamic_state_mode, flow_weight
    )
    pair_i, pair_j = sample_random_pair_indices(pos.shape[0], max_pairs=max_pairs, seed=seed)
    if pair_i.size:
        pair_dist = pointwise_state_distance(
            pos[pair_i], pos[pair_j], flow[pair_i], flow[pair_j], flow_ball[pair_i], flow_ball[pair_j], dynamic_state_mode, flow_weight
        )
        pair_mean = float(np.mean(pair_dist))
    else:
        pair_mean = 0.0
    step_mean = float(np.mean(step_dist))
    return {
        "dynamic_step_dist_mean": step_mean,
        "dynamic_step_dist_std": float(np.std(step_dist)),
        "dynamic_random_pair_dist_mean": pair_mean,
        "dynamic_step_to_random_ratio": float(step_mean / max(pair_mean, 1e-12)),
    }


def global_mean_mse(train_y: np.ndarray, eval_y: np.ndarray) -> float:
    mean_vec = np.mean(train_y, axis=0, keepdims=True)
    err = eval_y - mean_vec
    return float(np.mean(err * err))


def chart_cache_payload(args: argparse.Namespace, n_train: int, d: int, dy: int) -> Dict:
    return {
        "cache_version": "dynamic_h4_chart_v1",
        "input": args.input,
        "eval_split": args.eval_split,
        "n_train": n_train,
        "d": d,
        "dy": dy,
        "args": {
            "seed": args.seed,
            "sector_mode": args.sector_mode,
            "K": args.K,
            "delta_r": args.delta_r,
            "kmeans_iters": args.kmeans_iters,
            "phase_dims": args.phase_dims,
            "phase4_dims": args.phase4_dims,
            "complex_dims": args.complex_dims,
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
            "route_scale_lambda": args.route_scale_lambda,
            "memory_coord_mode": args.memory_coord_mode,
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
            "shell_mode": args.shell_mode,
            "shell_phase_coupling": args.shell_phase_coupling,
            "hopf_chi_bins": args.hopf_chi_bins,
            "hopf_blend_lambda": args.hopf_blend_lambda,
            "hopf_blend_chi_weight": args.hopf_blend_chi_weight,
            "hopf_blend_shell_weight": args.hopf_blend_shell_weight,
            "fib_rung_gate_threshold": args.fib_rung_gate_threshold,
            "hybrid_local_k": args.hybrid_local_k,
            "hybrid_complex_roots": args.hybrid_complex_roots,
            "hybrid_local_min_k": args.hybrid_local_min_k,
            "hybrid_local_target": args.hybrid_local_target,
            "hybrid_local_hysteresis": args.hybrid_local_hysteresis,
            "hybrid_local_converge_lambda": args.hybrid_local_converge_lambda,
        },
    }


def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--eval_split", type=str, default="test", choices=["test", "val"])
    ap.add_argument("--max_train", type=int, default=6000)
    ap.add_argument("--max_eval", type=int, default=3000)
    ap.add_argument("--seed", type=int, default=0)

    ap.add_argument("--dynamic_state_mode", type=str, default="static_h4", choices=["static_h4", "tangent_h4", "product_h4x_h4"])
    ap.add_argument("--candidate_mode", type=str, default="global_knn", choices=["global_knn", "static_bucket_knn"])
    ap.add_argument("--route_key_mode", type=str, default="hopf_bucket", choices=["hopf_bucket", "hopf_plus_complex"])
    ap.add_argument("--flow_step", type=int, default=1)
    ap.add_argument("--flow_scale", type=float, default=1.0)
    ap.add_argument("--flow_weight", type=float, default=0.5)
    ap.add_argument("--state_topk", type=int, default=8)
    ap.add_argument("--state_block_size", type=int, default=256)
    ap.add_argument("--state_pair_samples", type=int, default=512)
    ap.add_argument("--complex_key_roots", type=int, default=8)
    ap.add_argument("--complex_key_radius_bins", type=int, default=1)

    ap.add_argument("--K", type=int, default=25)
    ap.add_argument("--delta_r", type=float, default=3.6)
    ap.add_argument("--kmeans_iters", type=int, default=12)
    ap.add_argument("--sector_mode", type=str, default="phase4d_hopf", choices=SECTOR_CHOICES)
    ap.add_argument("--phase_dims", type=str, default="0,1")
    ap.add_argument("--phase4_dims", type=str, default="0,2,4,6")
    ap.add_argument("--complex_dims", type=str, default="1,3")

    ap.add_argument("--adaptive_min_pair_bins", type=int, default=3)
    ap.add_argument("--adaptive_time_growth", type=float, default=1.4)
    ap.add_argument("--adaptive_balance", type=float, default=1.2)
    ap.add_argument("--adaptive_angle_growth", type=float, default=0.5)
    ap.add_argument("--adaptive_shell_growth", type=float, default=1.6)
    ap.add_argument("--adaptive_shell_balance", type=float, default=1.0)
    ap.add_argument("--adaptive_converge_lambda", type=float, default=0.65)
    ap.add_argument("--adaptive_converge_target", type=float, default=0.85)
    ap.add_argument("--adaptive_converge_hysteresis", type=float, default=0.05)
    ap.add_argument("--adaptive_converge_mode", type=str, default="phi_ladder", choices=["fixed", "phi_ratio", "phi_ladder"])
    ap.add_argument("--shell_mode", type=str, default="phi_log", choices=["linear", "phi_log", "phi_phase"])
    ap.add_argument("--shell_phase_coupling", type=float, default=0.0)
    ap.add_argument("--fib_rung_gate_threshold", type=float, default=0.0)
    ap.add_argument("--route_scale_lambda", type=float, default=1.0)
    ap.add_argument("--memory_coord_mode", type=str, default="route_chart", choices=["route_chart", "full_chart"])
    ap.add_argument("--hopf_chi_bins", type=int, default=2)
    ap.add_argument("--hopf_blend_lambda", type=float, default=0.8)
    ap.add_argument("--hopf_blend_chi_weight", type=float, default=1.0)
    ap.add_argument("--hopf_blend_shell_weight", type=float, default=0.5)
    ap.add_argument("--hybrid_local_k", type=int, default=4)
    ap.add_argument("--hybrid_complex_roots", type=int, default=4)
    ap.add_argument("--hybrid_local_min_k", type=int, default=1)
    ap.add_argument("--hybrid_local_target", type=float, default=0.6)
    ap.add_argument("--hybrid_local_hysteresis", type=float, default=0.05)
    ap.add_argument("--hybrid_local_converge_lambda", type=float, default=1.0)

    ap.add_argument("--learn_so8", type=int, default=0, choices=[0, 1])
    ap.add_argument("--learn_scale", type=int, default=1, choices=[0, 1])
    ap.add_argument("--scale_mode", type=str, default="radial", choices=["global", "radial"])
    ap.add_argument("--radial_bins", type=int, default=10)
    ap.add_argument("--radial_rmax", type=float, default=0.0)
    ap.add_argument("--radial_update_frac", type=float, default=0.25)
    ap.add_argument("--radial_l2", type=float, default=0.0)

    ap.add_argument("--chart_iters", type=int, default=40)
    ap.add_argument("--chart_alpha", type=float, default=0.01)
    ap.add_argument("--chart_beta", type=float, default=0.0)
    ap.add_argument("--so8_step", type=float, default=0.1)
    ap.add_argument("--so8_candidates", type=int, default=2)
    ap.add_argument("--scale_step", type=float, default=0.08)
    ap.add_argument("--scale_candidates", type=int, default=2)
    ap.add_argument("--scale_clip", type=float, default=2.0)

    ap.add_argument("--cache_dir", type=str, default="results/cache")
    ap.add_argument("--cache_chart", type=int, default=0, choices=[0, 1])
    ap.add_argument("--run_tag", type=str, default="")
    return ap.parse_args()


def main():
    args = parse_args()
    t_total_start = time.perf_counter()
    timings = {"dataset": 0.0, "chart_opt": 0.0, "routing_eval": 0.0, "total": 0.0}
    notes = []
    artifacts = {
        "input": args.input,
        "eval_split": args.eval_split,
        "run_tag": args.run_tag,
        "dynamic_state_mode": args.dynamic_state_mode,
    }

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

    v_tr, y_tr, idx_tr, start_tr, stop_tr = _contiguous_subset(x_train, y_train, args.max_train, args.seed + 1)
    v_ev, y_ev, idx_ev, start_ev, stop_ev = _contiguous_subset(x_eval, y_eval, args.max_eval, args.seed + 2)
    artifacts["train_window_start"] = int(start_tr)
    artifacts["train_window_stop"] = int(stop_tr)
    artifacts["eval_window_start"] = int(start_ev)
    artifacts["eval_window_stop"] = int(stop_ev)

    d = int(v_tr.shape[1])
    dy = int(y_tr.shape[1])
    phase_dim_i, phase_dim_j = hr.parse_pair_dims(args.phase_dims, "--phase_dims")
    phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l = hr.parse_quad_dims(args.phase4_dims, "--phase4_dims")
    complex_dim_i, complex_dim_j = hr.parse_pair_dims(args.complex_dims, "--complex_dims")
    hr.ensure_dims_in_range([phase_dim_i, phase_dim_j], d, "--phase_dims")
    hr.ensure_dims_in_range([phase4_dim_i, phase4_dim_j, phase4_dim_k, phase4_dim_l], d, "--phase4_dims")
    hr.ensure_dims_in_range([complex_dim_i, complex_dim_j], d, "--complex_dims")

    C0 = None
    if args.sector_mode == "kmeans":
        C0 = hr.spherical_kmeans(hr.normalize_rows(v_tr), K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

    chart = hr.Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)
    if learned_chart and int(args.cache_chart) == 1:
        hr.ensure_dir(args.cache_dir)
        key = hr.stable_hash(chart_cache_payload(args, n_train=int(v_tr.shape[0]), d=d, dy=dy))
        chart_cache_file = os.path.join(args.cache_dir, f"dynamic_chart_{key}.npz")
        artifacts["chart_cache_file"] = chart_cache_file
        if os.path.exists(chart_cache_file):
            zc = np.load(chart_cache_file)
            chart = hr.Chart(
                R=zc["R"].astype(np.float64),
                s_global=(zc["s_global"].astype(np.float64) if "s_global" in zc else None),
                S_radial=(zc["S_radial"].astype(np.float64) if "S_radial" in zc else None),
                scale_mode=str(zc["scale_mode"].tolist()),
                radial_rmax=float(zc["radial_rmax"][0]) if "radial_rmax" in zc else 0.0,
                radial_bins=int(zc["radial_bins"][0]) if "radial_bins" in zc else 0,
            )
            notes.append("chart_cache_hit")
        else:
            t1 = time.perf_counter()
            chart = hr.optimize_chart(
                v_train=v_tr,
                y_train=y_tr,
                delta_r=args.delta_r,
                C0=C0,
                learn_so8=args.learn_so8,
                learn_scale=args.learn_scale,
                scale_mode=args.scale_mode,
                radial_bins=args.radial_bins,
                radial_rmax=args.radial_rmax,
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
                seed=args.seed,
                early_stop_patience=0,
                early_stop_min_delta=0.0,
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
                hybrid_local_k=args.hybrid_local_k,
                hybrid_complex_roots=args.hybrid_complex_roots,
                hybrid_local_min_k=args.hybrid_local_min_k,
                hybrid_local_target=args.hybrid_local_target,
                hybrid_local_hysteresis=args.hybrid_local_hysteresis,
                hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
            ).chart
            timings["chart_opt"] += time.perf_counter() - t1
            np.savez_compressed(
                chart_cache_file,
                R=chart.R,
                s_global=chart.s_global if chart.s_global is not None else np.zeros((0,), dtype=np.float64),
                S_radial=chart.S_radial if chart.S_radial is not None else np.zeros((0, d), dtype=np.float64),
                scale_mode=np.array(chart.scale_mode),
                radial_rmax=np.array([chart.radial_rmax], dtype=np.float64),
                radial_bins=np.array([chart.radial_bins], dtype=np.int64),
            )
            notes.append("chart_cache_write")
    elif learned_chart:
        t1 = time.perf_counter()
        chart = hr.optimize_chart(
            v_train=v_tr,
            y_train=y_tr,
            delta_r=args.delta_r,
            C0=C0,
            learn_so8=args.learn_so8,
            learn_scale=args.learn_scale,
            scale_mode=args.scale_mode,
            radial_bins=args.radial_bins,
            radial_rmax=args.radial_rmax,
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
            seed=args.seed,
            early_stop_patience=0,
            early_stop_min_delta=0.0,
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
            hybrid_local_k=args.hybrid_local_k,
            hybrid_complex_roots=args.hybrid_complex_roots,
            hybrid_local_min_k=args.hybrid_local_min_k,
            hybrid_local_target=args.hybrid_local_target,
            hybrid_local_hysteresis=args.hybrid_local_hysteresis,
            hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
        ).chart
        timings["chart_opt"] += time.perf_counter() - t1

    t2 = time.perf_counter()
    z_tr = hr.route_coordinate(v_tr, chart, sector_mode=args.sector_mode, route_scale_lambda=args.route_scale_lambda)
    z_ev = hr.route_coordinate(v_ev, chart, sector_mode=args.sector_mode, route_scale_lambda=args.route_scale_lambda)

    pos_tr = hr.exp_map0(z_tr)
    pos_ev = hr.exp_map0(z_ev)
    flow_tr, flow_ball_tr, flow_norm_q95_tr = build_flow_state(z_tr, idx_tr, args.flow_step, args.flow_scale)
    flow_ev, flow_ball_ev, flow_norm_q95_ev = build_flow_state(z_ev, idx_ev, args.flow_step, args.flow_scale)

    shell_tr, sector_tr, _, _ = hr.route_addresses(
        v=v_tr,
        delta_r=args.delta_r,
        C=None,
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
        hybrid_local_k=args.hybrid_local_k,
        hybrid_complex_roots=args.hybrid_complex_roots,
        hybrid_local_min_k=args.hybrid_local_min_k,
        hybrid_local_target=args.hybrid_local_target,
        hybrid_local_hysteresis=args.hybrid_local_hysteresis,
        hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
    )
    keys_tr = [hr.make_bucket_key(int(shell_tr[i]), int(sector_tr[i])) for i in range(len(shell_tr))]

    shell_ev, sector_ev, _, _ = hr.route_addresses(
        v=v_ev,
        delta_r=args.delta_r,
        C=None,
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
        hybrid_local_k=args.hybrid_local_k,
        hybrid_complex_roots=args.hybrid_complex_roots,
        hybrid_local_min_k=args.hybrid_local_min_k,
        hybrid_local_target=args.hybrid_local_target,
        hybrid_local_hysteresis=args.hybrid_local_hysteresis,
        hybrid_local_converge_lambda=args.hybrid_local_converge_lambda,
    )
    keys_ev = [hr.make_bucket_key(int(shell_ev[i]), int(sector_ev[i])) for i in range(len(shell_ev))]
    secondary_key_count = 0
    if args.route_key_mode == "hopf_plus_complex":
        keys_tr, secondary_key_count_tr = augment_route_keys_with_complex(
            base_keys=keys_tr,
            field=flow_ball_tr,
            dim_i=complex_dim_i,
            dim_j=complex_dim_j,
            roots=args.complex_key_roots,
            radius_bins=args.complex_key_radius_bins,
        )
        keys_ev, secondary_key_count_ev = augment_route_keys_with_complex(
            base_keys=keys_ev,
            field=flow_ball_ev,
            dim_i=complex_dim_i,
            dim_j=complex_dim_j,
            roots=args.complex_key_roots,
            radius_bins=args.complex_key_radius_bins,
        )
        secondary_key_count = int(max(secondary_key_count_tr, secondary_key_count_ev))
    if args.candidate_mode == "static_bucket_knn":
        (
            yhat,
            pred_idx,
            knn_dist_mean,
            candidate_count_mean,
            candidate_fraction_mean,
            probe_bucket_mean,
            bucket_fallback_rate,
        ) = knn_state_predict_bucketed(
            train_pos=pos_tr,
            train_flow=flow_tr,
            train_flow_ball=flow_ball_tr,
            train_y=y_tr,
            train_keys=keys_tr,
            eval_pos=pos_ev,
            eval_flow=flow_ev,
            eval_flow_ball=flow_ball_ev,
            eval_keys=keys_ev,
            dynamic_state_mode=args.dynamic_state_mode,
            flow_weight=args.flow_weight,
            topk=args.state_topk,
            block_size=args.state_block_size,
        )
    else:
        yhat, pred_idx, knn_dist_mean = knn_state_predict(
            train_pos=pos_tr,
            train_flow=flow_tr,
            train_flow_ball=flow_ball_tr,
            train_y=y_tr,
            eval_pos=pos_ev,
            eval_flow=flow_ev,
            eval_flow_ball=flow_ball_ev,
            dynamic_state_mode=args.dynamic_state_mode,
            flow_weight=args.flow_weight,
            topk=args.state_topk,
            block_size=args.state_block_size,
        )
        candidate_count_mean = float(pos_tr.shape[0])
        candidate_fraction_mean = 1.0
        probe_bucket_mean = 0.0
        bucket_fallback_rate = 0.0
    timings["routing_eval"] = time.perf_counter() - t2

    true_idx = np.argmax(y_ev, axis=1).astype(np.int64)
    mse_before = global_mean_mse(y_tr, y_ev)
    err = yhat - y_ev
    mse_after = float(np.mean(err * err))
    top1_after = float(np.mean((pred_idx == true_idx).astype(np.float64)))

    coherence = state_coherence_metrics(
        pos=pos_ev,
        flow=flow_ev,
        flow_ball=flow_ball_ev,
        dynamic_state_mode=args.dynamic_state_mode,
        flow_weight=args.flow_weight,
        max_pairs=args.state_pair_samples,
        seed=args.seed + 101,
    )

    alignment = hr.poincare_alignment_diagnostics(v_ev, z_ev, max_pairs=args.state_pair_samples, seed=args.seed + 201)
    key_hash = shell_ev.astype(np.int64) * 1000003 + sector_ev.astype(np.int64)
    _, key_counts = np.unique(key_hash, return_counts=True)
    shell_counts = np.unique(shell_ev.astype(np.int64), return_counts=True)[1]
    sector_counts = np.unique(sector_ev.astype(np.int64), return_counts=True)[1]

    timings["total"] = time.perf_counter() - t_total_start

    metrics = {
        "test_mse_before": float(mse_before),
        "test_mse_after": float(mse_after),
        "test_top1_after": float(top1_after),
        "buckets": int(len(key_counts)),
        "eval_shells": int(len(shell_counts)),
        "eval_sectors": int(len(sector_counts)),
        "pmax_after": float(np.max(key_counts) / np.sum(key_counts)) if len(key_counts) else 0.0,
        "shell_pmax": float(np.max(shell_counts) / np.sum(shell_counts)) if len(shell_counts) else 0.0,
        "sector_pmax": float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0,
        "entropy_after": hr.entropy_from_counts(key_counts) if len(key_counts) else 0.0,
        "shell_entropy": hr.entropy_from_counts(shell_counts) if len(shell_counts) else 0.0,
        "sector_entropy": hr.entropy_from_counts(sector_counts) if len(sector_counts) else 0.0,
        "dynamic_knn_distance_mean": float(knn_dist_mean),
        "retrieval_candidate_count_mean": float(candidate_count_mean),
        "retrieval_candidate_fraction_mean": float(candidate_fraction_mean),
        "retrieval_probe_bucket_mean": float(probe_bucket_mean),
        "retrieval_bucket_fallback_rate": float(bucket_fallback_rate),
        "retrieval_secondary_key_count": int(secondary_key_count),
        "dynamic_flow_norm_mean": float(np.mean(hr.safe_norm(flow_ev, axis=1, keepdims=False))),
        "dynamic_flow_norm_q95": float(np.quantile(hr.safe_norm(flow_ev, axis=1, keepdims=False), 0.95)),
        "dynamic_flow_ball_radius_mean": float(np.mean(hr.poincare_radius(flow_ball_ev))),
        "dynamic_flow_ball_radius_q95": float(np.quantile(hr.poincare_radius(flow_ball_ev), 0.95)),
        "dynamic_train_flow_norm_q95": float(flow_norm_q95_tr),
        "dynamic_eval_flow_norm_q95": float(flow_norm_q95_ev),
        **{k: float(v) for k, v in coherence.items()},
        **{k: float(v) if not isinstance(v, int) else int(v) for k, v in alignment.items()},
    }

    summary = {
        "schema_version": "1.0",
        "task": "dynamic_h4_state_eval",
        "args": vars(args),
        "metrics": metrics,
        "timings_sec": {k: float(v) for k, v in timings.items()},
        "artifacts": artifacts,
        "git": hr.maybe_git_info(),
        "notes": notes,
    }
    print("__JSON_SUMMARY__ " + json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
