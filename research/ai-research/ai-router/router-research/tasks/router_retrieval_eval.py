#!/usr/bin/env python3
import argparse
from dataclasses import dataclass
import json
import math
import os
import sys
import time
from typing import Callable, Dict, List, Optional, Sequence, Tuple

import numpy as np

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

import hyperbolic_router_so8 as hr
from tasks.router_proxy_eval import event_gate_stats, event_gate_value_from_error
from tasks.wikitext2_proxy import contexts_targets, hashed_targets


def _subset_with_index(X: np.ndarray, Y: np.ndarray, T: np.ndarray, max_n: int, seed: int):
    if max_n <= 0 or X.shape[0] <= max_n:
        idx = np.arange(X.shape[0], dtype=np.int64)
        return X, Y, T, idx
    rs = np.random.RandomState(seed)
    idx = rs.permutation(X.shape[0])[:max_n]
    return X[idx], Y[idx], T[idx], idx


def _normalize_rows(X: np.ndarray) -> np.ndarray:
    n = np.linalg.norm(X, axis=1, keepdims=True)
    n = np.maximum(n, 1e-8)
    return (X / n).astype(np.float64)


def apply_retrieval_fast_dev(args: argparse.Namespace):
    if int(args.fast_dev) != 1:
        return
    args.max_train = min(int(args.max_train), 3000)
    args.max_eval = min(int(args.max_eval), 1500)
    args.chart_iters = min(int(args.chart_iters), 40)
    args.kmeans_iters = min(int(args.kmeans_iters), 12)
    args.so8_candidates = min(int(args.so8_candidates), 2)
    args.scale_candidates = min(int(args.scale_candidates), 2)
    args.probe_buckets = min(int(args.probe_buckets), 2)


def compute_amortized_retrieval_metrics(offline_total: float, online_total: float, query_repeats: int) -> Tuple[float, float]:
    reps = max(1, int(query_repeats))
    return float(online_total / reps), float((offline_total + online_total) / reps)


def event_gate_changes_retrieval_surface(args: argparse.Namespace) -> bool:
    return bool(
        getattr(args, "retrieval_backend", "") == "routed_probe"
        and getattr(args, "event_gate_mode", "off") != "off"
        and getattr(args, "event_gate_translation_coupling", "off") != "off"
    )


def sparse_event_training_audit(
    train_keys_static: Sequence[Tuple[int, ...]],
    z_tr_static: np.ndarray,
    v_tr: np.ndarray,
    y_tr: np.ndarray,
    d: int,
    dy: int,
    eta_p: float,
    eta_m: float,
    epochs: int,
    seed: int,
    event_gate_mode: str,
    event_gate_threshold: float,
    event_gate_tau: float,
    route_one_fn: Optional[Callable[[np.ndarray, float], Tuple[Tuple[int, ...], np.ndarray]]] = None,
) -> Dict[str, float]:
    metrics = {
        "event_gate_error_mean": 0.0,
        "event_gate_mean": 1.0,
        "event_gate_active_frac": 1.0,
        "event_gate_cost_proxy": 1.0,
        "training_total_sec": 0.0,
        "training_route_sec": 0.0,
        "training_update_sec": 0.0,
        "sample_gate_mean": np.ones((v_tr.shape[0],), dtype=np.float64),
    }
    total_steps = max(0, int(epochs)) * int(v_tr.shape[0])
    if total_steps <= 0 or len(train_keys_static) == 0:
        return metrics

    buckets = hr.init_buckets(train_keys_static, dy=dy, d=d, seed=int(seed) + 101)
    rs = np.random.RandomState(int(seed) + 404)
    event_gate_error_sum = 0.0
    event_gate_sum = 0.0
    event_gate_active_sum = 0.0
    sample_gate_sum = np.zeros((v_tr.shape[0],), dtype=np.float64)
    sample_gate_count = np.zeros((v_tr.shape[0],), dtype=np.float64)
    route_sec = 0.0
    update_sec = 0.0
    step_ctr = 0
    t_train = time.perf_counter()
    for _epoch in range(max(0, int(epochs))):
        order = rs.permutation(v_tr.shape[0])
        for jj in order:
            tau_sched = 1.0 if total_steps <= 1 else float(step_ctr) / float(total_steps - 1)
            t_route = time.perf_counter()
            if route_one_fn is None:
                key = train_keys_static[jj]
                z1 = z_tr_static[jj]
            else:
                key, z1 = route_one_fn(v_tr[jj], tau_sched)
            route_sec += time.perf_counter() - t_route

            t_update = time.perf_counter()
            yhat, sj, _ = hr.predict_from_bucket(buckets, key, z1, d=d, dy=dy)
            error_mag, event_gate, event_active = event_gate_stats(
                yhat=yhat,
                y=y_tr[jj],
                mode=event_gate_mode,
                threshold=event_gate_threshold,
                tau=event_gate_tau,
            )
            event_gate_error_sum += error_mag
            event_gate_sum += event_gate
            event_gate_active_sum += event_active
            sample_gate_sum[jj] += float(event_gate)
            sample_gate_count[jj] += 1.0
            hr.ema_update(
                hr.get_bucket(buckets, key, d=d, dy=dy).slots[sj],
                z=z1,
                y=y_tr[jj],
                eta_p=float(eta_p) * float(event_gate),
                eta_m=float(eta_m) * float(event_gate),
            )
            update_sec += time.perf_counter() - t_update
            step_ctr += 1

    metrics["training_total_sec"] = float(time.perf_counter() - t_train)
    metrics["training_route_sec"] = float(route_sec)
    metrics["training_update_sec"] = float(update_sec)
    metrics["event_gate_error_mean"] = float(event_gate_error_sum / float(total_steps))
    metrics["event_gate_mean"] = float(event_gate_sum / float(total_steps))
    metrics["event_gate_active_frac"] = float(event_gate_active_sum / float(total_steps))
    metrics["event_gate_cost_proxy"] = float(metrics["event_gate_mean"])
    counts = np.maximum(sample_gate_count, 1.0)
    metrics["sample_gate_mean"] = (sample_gate_sum / counts).astype(np.float64)
    return metrics


def build_event_gate_train_mask(
    train_keys: Sequence[Tuple[int, ...]],
    sample_gate_mean: np.ndarray,
    threshold: float,
    min_keep_per_bucket: int,
) -> np.ndarray:
    gate = np.asarray(sample_gate_mean, dtype=np.float64)
    if gate.shape[0] != len(train_keys):
        raise ValueError("sample_gate_mean must align with train_keys")
    keep = gate >= float(threshold)
    bucket_to_idx = build_bucket_index(train_keys)
    min_keep = max(1, int(min_keep_per_bucket))
    for idx in bucket_to_idx.values():
        local_keep = keep[idx]
        if int(np.sum(local_keep)) >= min_keep:
            continue
        order = idx[np.argsort(-gate[idx], kind="stable")]
        keep[order[:min(min_keep, order.shape[0])]] = True
    if not np.any(keep) and gate.size:
        keep[int(np.argmax(gate))] = True
    return keep.astype(bool)


def build_event_gate_train_score_bias(sample_gate_mean: np.ndarray) -> np.ndarray:
    gate = np.asarray(sample_gate_mean, dtype=np.float64)
    if gate.ndim != 1:
        raise ValueError("sample_gate_mean must be one-dimensional")
    if gate.size <= 1:
        return np.zeros_like(gate, dtype=np.float64)
    center = float(np.mean(gate))
    scale = max(float(np.std(gate)), 1e-6)
    bias = (center - gate) / (3.0 * scale)
    return np.clip(bias, -1.0, 1.0).astype(np.float64)


def parse_args():
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", type=str, default="data/wikitext2_proxy/wikitext2_proxy.npz")
    ap.add_argument("--tokens_input", type=str, default="data/wikitext2_proxy/wikitext2_tokens.npz")
    ap.add_argument("--proxy_meta", type=str, default="data/wikitext2_proxy/wikitext2_proxy_meta.json")
    ap.add_argument("--eval_split", type=str, default="test", choices=["test", "val"])
    ap.add_argument("--max_train", type=int, default=12000)
    ap.add_argument("--max_eval", type=int, default=6000)

    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--retrieval_backend", type=str, default="routed_probe", choices=["dense_exact", "routed_probe"])
    ap.add_argument("--topk", type=int, default=8)
    ap.add_argument("--probe_buckets", type=int, default=1)
    ap.add_argument("--query_repeats", type=int, default=1)
    ap.add_argument("--route_key_mode", type=str, default="hopf_bucket", choices=["hopf_bucket", "hopf_plus_complex"])
    ap.add_argument("--complex_key_roots", type=int, default=8)
    ap.add_argument("--complex_key_radius_bins", type=int, default=1)
    ap.add_argument("--complex_backfill_items", type=int, default=0)
    ap.add_argument("--complex_backfill_mode", type=str, default="always", choices=["always", "small_bucket", "low_margin"])
    ap.add_argument("--complex_backfill_max_exact", type=int, default=0)
    ap.add_argument("--complex_backfill_margin_threshold", type=float, default=0.0)
    ap.add_argument("--complex_rerank_mode", type=str, default="none", choices=["none", "complex_plane", "complex_plane_low_margin"])
    ap.add_argument("--complex_rerank_lambda", type=float, default=0.0)
    ap.add_argument("--complex_rerank_margin_threshold", type=float, default=0.0)

    ap.add_argument("--K", type=int, default=8)
    ap.add_argument("--delta_r", type=float, default=3.0)
    ap.add_argument("--kmeans_iters", type=int, default=25)

    ap.add_argument("--epochs", type=int, default=1)
    ap.add_argument("--eta_p", type=float, default=0.04)
    ap.add_argument("--eta_m", type=float, default=0.08)
    ap.add_argument("--extra_budget", type=int, default=32)
    ap.add_argument("--max_slots_per_bucket", type=int, default=4)
    ap.add_argument("--split_rounds", type=int, default=40)
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
    ap.add_argument("--shell_mode", type=str, default="linear", choices=["linear", "phi_log", "phi_phase"])
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
    ap.add_argument("--phase_transport_lambda", type=float, default=1.0)
    ap.add_argument("--phase_field_lambda", type=float, default=0.0)
    ap.add_argument("--time_pressure_lambda", type=float, default=0.0)
    ap.add_argument("--train_route_mode", type=str, default="final_static", choices=["dynamic", "final_static"])
    ap.add_argument("--event_gate_mode", type=str, default="off", choices=["off", "soft_error"])
    ap.add_argument("--event_gate_threshold", type=float, default=0.0)
    ap.add_argument("--event_gate_tau", type=float, default=0.01)
    ap.add_argument("--event_gate_translation_coupling", type=str, default="off", choices=["off", "train_gate_prune", "train_gate_score_bias"])
    ap.add_argument("--event_gate_translation_prune_threshold", type=float, default=0.0)
    ap.add_argument("--event_gate_translation_min_keep_per_bucket", type=int, default=1)
    ap.add_argument("--event_gate_translation_score_bias_lambda", type=float, default=0.0)

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
    ap.add_argument("--so8_step", type=float, default=0.10)
    ap.add_argument("--so8_candidates", type=int, default=2)
    ap.add_argument("--scale_step", type=float, default=0.08)
    ap.add_argument("--scale_candidates", type=int, default=2)
    ap.add_argument("--scale_clip", type=float, default=2.0)
    ap.add_argument("--recluster_after_chart", type=int, default=1, choices=[0, 1])

    ap.add_argument("--fast_dev", type=int, default=0, choices=[0, 1])
    ap.add_argument("--early_stop_patience", type=int, default=0)
    ap.add_argument("--early_stop_min_delta", type=float, default=0.0)
    ap.add_argument("--cache_dir", type=str, default="results/cache")
    ap.add_argument("--cache_chart", type=int, default=0, choices=[0, 1])
    ap.add_argument("--cache_routes", type=int, default=0, choices=[0, 1])
    ap.add_argument("--retrieval_audit_dir", type=str, default="")
    ap.add_argument("--run_tag", type=str, default="")
    return ap.parse_args()


def retrieval_chart_cache_payload(args: argparse.Namespace, d: int, dy: int, n_train: int) -> Dict[str, object]:
    return {
        "cache_version": "retrieval_chart_v1",
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


def retrieval_route_cache_payload(args: argparse.Namespace, chart_fp: str, n_train: int) -> Dict[str, object]:
    return {
        "cache_version": "retrieval_train_routes_v1",
        "input": args.input,
        "eval_split": args.eval_split,
        "n_train": n_train,
        "chart_fp": chart_fp,
        "args": {
            "seed": args.seed,
            "K": args.K,
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
            "time_pressure_lambda": args.time_pressure_lambda,
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
            "hybrid_local_k": args.hybrid_local_k,
            "hybrid_complex_roots": args.hybrid_complex_roots,
            "hybrid_local_min_k": args.hybrid_local_min_k,
            "hybrid_local_target": args.hybrid_local_target,
            "hybrid_local_hysteresis": args.hybrid_local_hysteresis,
            "hybrid_local_converge_lambda": args.hybrid_local_converge_lambda,
        },
    }


def _load_chart_cache(path: str) -> hr.Chart:
    payload = np.load(path, allow_pickle=True)
    s_global = payload["s_global"] if bool(payload["has_s_global"]) else None
    s_radial = payload["S_radial"] if bool(payload["has_S_radial"]) else None
    return hr.Chart(
        R=payload["R"],
        s_global=s_global,
        S_radial=s_radial,
        scale_mode=str(payload["scale_mode"]),
        radial_rmax=float(payload["radial_rmax"]),
        radial_bins=int(payload["radial_bins"]),
    )


def _save_chart_cache(path: str, chart: hr.Chart) -> None:
    np.savez_compressed(
        path,
        R=chart.R,
        has_s_global=(chart.s_global is not None),
        s_global=(chart.s_global if chart.s_global is not None else np.zeros((0,), dtype=np.float64)),
        has_S_radial=(chart.S_radial is not None),
        S_radial=(chart.S_radial if chart.S_radial is not None else np.zeros((0, 0), dtype=np.float64)),
        scale_mode=np.array(chart.scale_mode),
        radial_rmax=np.array(chart.radial_rmax),
        radial_bins=np.array(chart.radial_bins),
    )


def _load_route_cache(path: str) -> Tuple[np.ndarray, np.ndarray, np.ndarray]:
    payload = np.load(path)
    return payload["shell_tr"], payload["sector_tr"], payload["z_tr"]


def _save_route_cache(path: str, shell_tr: np.ndarray, sector_tr: np.ndarray, z_tr: np.ndarray) -> None:
    np.savez_compressed(path, shell_tr=shell_tr, sector_tr=sector_tr, z_tr=z_tr)


def load_token_targets(tokens_input: str, proxy_meta_path: str, eval_split: str) -> Tuple[np.ndarray, np.ndarray]:
    with open(proxy_meta_path, "r", encoding="utf-8") as f:
        meta = json.load(f)
    context_len = int(meta["context_len"])
    n_train = int(meta["n_train"])
    n_eval = int(meta["n_val"] if eval_split == "val" else meta["n_test"])
    z = np.load(tokens_input)
    _, y_train_tok = contexts_targets(z["train_ids"], context_len, n_train)
    split_key = "valid_ids" if eval_split == "val" else "test_ids"
    _, y_eval_tok = contexts_targets(z[split_key], context_len, n_eval)
    return y_train_tok.astype(np.int32), y_eval_tok.astype(np.int32)


def key_stats(keys: Sequence[Tuple[int, ...]]) -> Tuple[float, float, float, int, int, int]:
    if not keys:
        return 0.0, 0.0, 0.0, 0, 0, 0
    sh = np.array([k[0] for k in keys], dtype=np.int64)
    se = np.array([k[1] for k in keys], dtype=np.int64)
    key_hash = sh * 1000003 + se
    _, counts = np.unique(key_hash, return_counts=True)
    pmax = float(np.max(counts) / np.sum(counts)) if len(counts) else 0.0
    entropy = hr.entropy_from_counts(counts) if len(counts) else 0.0
    shell_vals = np.unique(sh)
    sector_vals = np.unique(se)
    shell_counts = np.unique(sh, return_counts=True)[1]
    sector_counts = np.unique(se, return_counts=True)[1]
    shell_pmax = float(np.max(shell_counts) / np.sum(shell_counts)) if len(shell_counts) else 0.0
    sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
    return pmax, entropy, shell_pmax, int(len(counts)), int(len(shell_vals)), int(len(sector_vals))


def global_mean_mse(y_train: np.ndarray, y_eval: np.ndarray) -> float:
    mean_vec = np.mean(y_train, axis=0, keepdims=True)
    dif = y_eval - mean_vec
    return float(np.mean(dif * dif))


def vote_top1(token_ids: np.ndarray) -> int:
    vals, counts = np.unique(token_ids, return_counts=True)
    return int(vals[np.argmax(counts)])


def topk_reduce(
    sim: np.ndarray,
    cand_y: np.ndarray,
    cand_tok: np.ndarray,
    topk: int,
) -> Tuple[np.ndarray, np.ndarray]:
    kk = min(topk, cand_y.shape[0])
    idx = np.argpartition(-sim, kth=kk - 1, axis=1)[:, :kk]
    part = np.take_along_axis(sim, idx, axis=1)
    order = np.argsort(-part, axis=1)
    idx = np.take_along_axis(idx, order, axis=1)
    yhat = np.mean(cand_y[idx], axis=1)
    pred_tok = np.zeros((idx.shape[0],), dtype=np.int32)
    for j in range(idx.shape[0]):
        pred_tok[j] = vote_top1(cand_tok[idx[j]])
    return yhat, pred_tok


def _new_query_audit(n_rows: int) -> Dict[str, np.ndarray]:
    return {
        "pred_tok": np.zeros((n_rows,), dtype=np.int32),
        "correct": np.zeros((n_rows,), dtype=np.int8),
        "candidate_count": np.zeros((n_rows,), dtype=np.int32),
        "candidate_fraction": np.zeros((n_rows,), dtype=np.float64),
        "target_present": np.zeros((n_rows,), dtype=np.int8),
        "best_target_rank": np.zeros((n_rows,), dtype=np.int32),
        "topk_target_present": np.zeros((n_rows,), dtype=np.int8),
    }


def _query_target_rank_stats(local_sim: np.ndarray, cand_tok: np.ndarray, target_tok: int, topk: int) -> Dict[str, int]:
    cand_tok = np.asarray(cand_tok, dtype=np.int32)
    target_mask = cand_tok == int(target_tok)
    target_present = int(np.any(target_mask))
    best_target_rank = 0
    if target_present:
        best_target_score = float(np.max(local_sim[target_mask]))
        best_target_rank = int(1 + np.sum(local_sim > best_target_score))
    kk = min(max(1, int(topk)), int(cand_tok.shape[0]))
    if cand_tok.shape[0] <= kk:
        best = np.argsort(-local_sim)
    else:
        best = np.argpartition(-local_sim, kth=kk - 1)[:kk]
        best = best[np.argsort(-local_sim[best])]
    top_tok = cand_tok[best]
    pred_tok = int(vote_top1(top_tok))
    return {
        "pred_tok": pred_tok,
        "correct": int(pred_tok == int(target_tok)),
        "target_present": target_present,
        "best_target_rank": best_target_rank,
        "topk_target_present": int(np.any(top_tok == int(target_tok))),
    }


def _record_query_audit_rows(
    query_audit: Optional[Dict[str, np.ndarray]],
    row_indices: np.ndarray,
    sim: np.ndarray,
    cand_tok: np.ndarray,
    eval_target_tok: np.ndarray,
    topk: int,
    n_train: int,
) -> None:
    if query_audit is None:
        return
    sim2d = np.atleast_2d(sim)
    row_idx = np.asarray(row_indices, dtype=np.int64)
    cand_tok = np.asarray(cand_tok, dtype=np.int32)
    cand_count = int(cand_tok.shape[0])
    cand_frac = float(cand_count / max(1.0, float(n_train)))
    for local_row, global_row in enumerate(row_idx):
        stats = _query_target_rank_stats(sim2d[local_row], cand_tok, int(eval_target_tok[global_row]), topk=topk)
        query_audit["pred_tok"][global_row] = stats["pred_tok"]
        query_audit["correct"][global_row] = stats["correct"]
        query_audit["candidate_count"][global_row] = cand_count
        query_audit["candidate_fraction"][global_row] = cand_frac
        query_audit["target_present"][global_row] = stats["target_present"]
        query_audit["best_target_rank"][global_row] = stats["best_target_rank"]
        query_audit["topk_target_present"][global_row] = stats["topk_target_present"]


def save_retrieval_query_audit(
    path: str,
    eval_source_idx: np.ndarray,
    target_tok: np.ndarray,
    query_audit: Dict[str, np.ndarray],
) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    np.savez_compressed(
        path,
        eval_local_idx=np.arange(target_tok.shape[0], dtype=np.int64),
        eval_source_idx=np.asarray(eval_source_idx, dtype=np.int64),
        target_tok=np.asarray(target_tok, dtype=np.int32),
        pred_tok=np.asarray(query_audit["pred_tok"], dtype=np.int32),
        correct=np.asarray(query_audit["correct"], dtype=np.int8),
        candidate_count=np.asarray(query_audit["candidate_count"], dtype=np.int32),
        candidate_fraction=np.asarray(query_audit["candidate_fraction"], dtype=np.float64),
        target_present=np.asarray(query_audit["target_present"], dtype=np.int8),
        best_target_rank=np.asarray(query_audit["best_target_rank"], dtype=np.int32),
        topk_target_present=np.asarray(query_audit["topk_target_present"], dtype=np.int8),
    )


def maybe_blend_complex_scores(
    sim: np.ndarray,
    eval_z: np.ndarray,
    train_z: np.ndarray,
    dim_i: int,
    dim_j: int,
    blend_lambda: float,
    mode: str,
    margin_threshold: float = 0.0,
) -> np.ndarray:
    if mode == "none" or abs(float(blend_lambda)) <= 1e-12:
        return sim
    eval_complex = _normalize_rows(eval_z[:, [dim_i, dim_j]])
    train_complex = _normalize_rows(train_z[:, [dim_i, dim_j]])
    complex_sim = eval_complex @ train_complex.T
    if mode == "complex_plane":
        return sim + float(blend_lambda) * complex_sim
    if mode == "complex_plane_low_margin":
        if sim.shape[1] <= 1:
            return sim
        top2 = min(2, sim.shape[1])
        best2 = np.argpartition(-sim, kth=top2 - 1, axis=1)[:, :top2]
        top2_vals = np.take_along_axis(sim, best2, axis=1)
        top2_vals.sort(axis=1)
        margins = top2_vals[:, -1] - top2_vals[:, -2]
        trigger = margins <= float(margin_threshold)
        if not np.any(trigger):
            return sim
        blended = np.array(sim, copy=True)
        blended[trigger] = blended[trigger] + float(blend_lambda) * complex_sim[trigger]
        return blended
    return sim


def maybe_blend_event_gate_scores(
    sim: np.ndarray,
    train_score_bias: Optional[np.ndarray],
    candidate_idx: np.ndarray,
    bias_lambda: float,
) -> np.ndarray:
    if train_score_bias is None or abs(float(bias_lambda)) <= 1e-12:
        return sim
    cand_idx = np.asarray(candidate_idx, dtype=np.int64)
    bias = np.asarray(train_score_bias[cand_idx], dtype=np.float64)
    if sim.ndim == 1:
        return sim + float(bias_lambda) * bias
    return sim + float(bias_lambda) * bias[None, :]


def dense_retrieval(
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    block_size: int = 256,
    candidate_fraction_denominator: Optional[int] = None,
    train_score_bias: Optional[np.ndarray] = None,
    score_bias_lambda: float = 0.0,
    return_query_audit: bool = False,
    eval_target_tok: Optional[np.ndarray] = None,
):
    if return_query_audit and eval_target_tok is None:
        raise ValueError("eval_target_tok is required when return_query_audit=True")
    denom = int(candidate_fraction_denominator or train_z.shape[0])
    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    query_audit = _new_query_audit(ev_u.shape[0]) if return_query_audit else None
    for start in range(0, ev_u.shape[0], block_size):
        stop = min(start + block_size, ev_u.shape[0])
        sim = ev_u[start:stop] @ tr_u.T
        sim = maybe_blend_event_gate_scores(
            sim,
            train_score_bias=train_score_bias,
            candidate_idx=np.arange(train_z.shape[0], dtype=np.int64),
            bias_lambda=score_bias_lambda,
        )
        y_block, tok_block = topk_reduce(sim, train_y, train_tok, topk=topk)
        yhat[start:stop] = y_block
        pred_tok[start:stop] = tok_block
        _record_query_audit_rows(
            query_audit,
            np.arange(start, stop, dtype=np.int64),
            sim,
            train_tok,
            eval_target_tok,
            topk=topk,
            n_train=denom,
        )
    candidate_count = float(train_z.shape[0])
    if query_audit is not None:
        return yhat, pred_tok, candidate_count, candidate_count / max(1.0, float(denom)), query_audit
    return yhat, pred_tok, candidate_count, candidate_count / max(1.0, float(denom))


def build_bucket_index(keys: Sequence[Tuple[int, ...]]) -> Dict[Tuple[int, ...], np.ndarray]:
    bucket_to_idx: Dict[Tuple[int, ...], List[int]] = {}
    for i, key in enumerate(keys):
        bucket_to_idx.setdefault(key, []).append(i)
    return {k: np.array(v, dtype=np.int64) for k, v in bucket_to_idx.items()}


@dataclass
class RetrievalBlock:
    idx: np.ndarray
    u: np.ndarray
    z: np.ndarray
    y: np.ndarray
    tok: np.ndarray


@dataclass
class GroupedSameBucketPlan:
    bucket_to_train_idx: Dict[Tuple[int, ...], np.ndarray]
    bucket_to_eval_idx: Dict[Tuple[int, ...], np.ndarray]
    tr_u: np.ndarray
    ev_u: np.ndarray
    full_idx: np.ndarray
    full_block: RetrievalBlock
    exact_blocks: Dict[Tuple[int, ...], RetrievalBlock]
    use_backfill: bool
    primary_to_train_idx: Dict[Tuple[int, int], np.ndarray]
    extra_pool_by_key: Dict[Tuple[int, ...], np.ndarray]


def prepare_grouped_same_bucket_plan(
    train_keys: Sequence[Tuple[int, ...]],
    eval_keys: Sequence[Tuple[int, ...]],
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    complex_backfill_items: int = 0,
) -> GroupedSameBucketPlan:
    bucket_to_train_idx = build_bucket_index(train_keys)
    bucket_to_eval_idx = build_bucket_index(eval_keys)
    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    full_idx = np.arange(train_z.shape[0], dtype=np.int64)
    full_block = RetrievalBlock(
        idx=full_idx,
        u=tr_u,
        z=train_z,
        y=train_y,
        tok=train_tok,
    )
    exact_blocks = {
        key: RetrievalBlock(
            idx=idx,
            u=tr_u[idx],
            z=train_z[idx],
            y=train_y[idx],
            tok=train_tok[idx],
        )
        for key, idx in bucket_to_train_idx.items()
    }
    use_backfill = int(complex_backfill_items) > 0 and bool(train_keys) and len(train_keys[0]) > 2
    primary_to_train_idx = build_bucket_index([primary_route_key(k) for k in train_keys]) if use_backfill else {}
    extra_pool_by_key: Dict[Tuple[int, ...], np.ndarray] = {}
    if use_backfill:
        for key, exact_idx in bucket_to_train_idx.items():
            if len(key) <= 2:
                continue
            base_idx = primary_to_train_idx.get(primary_route_key(key))
            if base_idx is None or base_idx.size == 0:
                extra_pool_by_key[key] = np.zeros((0,), dtype=np.int64)
            else:
                extra_pool_by_key[key] = np.setdiff1d(base_idx, exact_idx, assume_unique=False)
    return GroupedSameBucketPlan(
        bucket_to_train_idx=bucket_to_train_idx,
        bucket_to_eval_idx=bucket_to_eval_idx,
        tr_u=tr_u,
        ev_u=ev_u,
        full_idx=full_idx,
        full_block=full_block,
        exact_blocks=exact_blocks,
        use_backfill=use_backfill,
        primary_to_train_idx=primary_to_train_idx,
        extra_pool_by_key=extra_pool_by_key,
    )


def complex_key_ids(field: np.ndarray, dim_i: int, dim_j: int, roots: int, radius_bins: int) -> np.ndarray:
    roots = max(1, int(roots))
    radius_bins = max(1, int(radius_bins))
    qi = field[:, dim_i]
    qj = field[:, dim_j]
    theta = np.mod(np.arctan2(qj, qi), 2.0 * math.pi)
    angle_ids = np.minimum((theta * (float(roots) / (2.0 * math.pi))).astype(np.int64), roots - 1)
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
) -> Tuple[List[Tuple[int, int, int]], int]:
    complex_ids = complex_key_ids(field, dim_i=dim_i, dim_j=dim_j, roots=roots, radius_bins=radius_bins)
    keys = [(int(key[0]), int(key[1]), int(complex_ids[i])) for i, key in enumerate(base_keys)]
    return keys, int(len(np.unique(complex_ids))) if complex_ids.size else 0


def primary_route_key(key: Tuple[int, ...]) -> Tuple[int, int]:
    return int(key[0]), int(key[1])


def routed_retrieval_grouped_same_bucket(
    train_keys: Sequence[Tuple[int, ...]],
    eval_keys: Sequence[Tuple[int, ...]],
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    candidate_fraction_denominator: Optional[int] = None,
    complex_backfill_items: int = 0,
    complex_backfill_mode: str = "always",
    complex_backfill_max_exact: int = 0,
    complex_backfill_margin_threshold: float = 0.0,
    complex_rerank_mode: str = "none",
    complex_rerank_lambda: float = 0.0,
    complex_rerank_margin_threshold: float = 0.0,
    complex_dim_i: int = 0,
    complex_dim_j: int = 1,
    train_score_bias: Optional[np.ndarray] = None,
    score_bias_lambda: float = 0.0,
    prepared_plan: Optional[GroupedSameBucketPlan] = None,
    return_query_audit: bool = False,
    eval_target_tok: Optional[np.ndarray] = None,
) -> Tuple[np.ndarray, np.ndarray, float, float, float, float, float, float]:
    if return_query_audit and eval_target_tok is None:
        raise ValueError("eval_target_tok is required when return_query_audit=True")
    denom = int(candidate_fraction_denominator or train_z.shape[0])
    plan = prepared_plan or prepare_grouped_same_bucket_plan(
        train_keys=train_keys,
        eval_keys=eval_keys,
        train_z=train_z,
        train_y=train_y,
        train_tok=train_tok,
        eval_z=eval_z,
        complex_backfill_items=complex_backfill_items,
    )
    bucket_to_train_idx = plan.bucket_to_train_idx
    bucket_to_eval_idx = plan.bucket_to_eval_idx
    tr_u = plan.tr_u
    ev_u = plan.ev_u
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    candidate_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    fallback = np.zeros((ev_u.shape[0],), dtype=np.float64)
    backfill_trigger = np.zeros((ev_u.shape[0],), dtype=np.float64)
    backfill_added = np.zeros((ev_u.shape[0],), dtype=np.float64)
    query_audit = _new_query_audit(ev_u.shape[0]) if return_query_audit else None

    full_idx = plan.full_idx
    use_backfill = plan.use_backfill
    primary_to_train_idx = plan.primary_to_train_idx
    extra_pool_by_key = plan.extra_pool_by_key
    for key, ev_idx in bucket_to_eval_idx.items():
        if not use_backfill:
            block = plan.exact_blocks.get(key)
            if block is None:
                block = plan.full_block
                fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ block.u.T
            sim = maybe_blend_complex_scores(
                sim,
                eval_z[ev_idx],
                block.z,
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                blend_lambda=complex_rerank_lambda,
                mode=complex_rerank_mode,
                margin_threshold=complex_rerank_margin_threshold,
            )
            sim = maybe_blend_event_gate_scores(
                sim,
                train_score_bias=train_score_bias,
                candidate_idx=block.idx,
                bias_lambda=score_bias_lambda,
            )
            y_block, tok_block = topk_reduce(sim, block.y, block.tok, topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(block.idx.shape[0])
            _record_query_audit_rows(
                query_audit,
                ev_idx,
                sim,
                block.tok,
                eval_target_tok,
                topk=topk,
                n_train=denom,
            )
            continue

        cand_idx = bucket_to_train_idx.get(key)
        base_idx = primary_to_train_idx.get(primary_route_key(key))
        if base_idx is None or base_idx.size == 0:
            cand_idx_row = full_idx
            fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ tr_u[cand_idx_row].T
            sim = maybe_blend_complex_scores(
                sim,
                eval_z[ev_idx],
                train_z[cand_idx_row],
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                blend_lambda=complex_rerank_lambda,
                mode=complex_rerank_mode,
                margin_threshold=complex_rerank_margin_threshold,
            )
            sim = maybe_blend_event_gate_scores(
                sim,
                train_score_bias=train_score_bias,
                candidate_idx=cand_idx_row,
                bias_lambda=score_bias_lambda,
            )
            y_block, tok_block = topk_reduce(sim, train_y[cand_idx_row], train_tok[cand_idx_row], topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(cand_idx_row.shape[0])
            _record_query_audit_rows(
                query_audit,
                ev_idx,
                sim,
                train_tok[cand_idx_row],
                eval_target_tok,
                topk=topk,
                n_train=denom,
            )
            continue
        if cand_idx is None or cand_idx.size == 0:
            fallback[ev_idx] = 1.0
            sim = ev_u[ev_idx] @ tr_u[base_idx].T
            sim = maybe_blend_complex_scores(
                sim,
                eval_z[ev_idx],
                train_z[base_idx],
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                blend_lambda=complex_rerank_lambda,
                mode=complex_rerank_mode,
                margin_threshold=complex_rerank_margin_threshold,
            )
            sim = maybe_blend_event_gate_scores(
                sim,
                train_score_bias=train_score_bias,
                candidate_idx=base_idx,
                bias_lambda=score_bias_lambda,
            )
            y_block, tok_block = topk_reduce(sim, train_y[base_idx], train_tok[base_idx], topk=topk)
            yhat[ev_idx] = y_block
            pred_tok[ev_idx] = tok_block
            candidate_counts[ev_idx] = float(base_idx.shape[0])
            _record_query_audit_rows(
                query_audit,
                ev_idx,
                sim,
                train_tok[base_idx],
                eval_target_tok,
                topk=topk,
                n_train=denom,
            )
            continue
        extra_pool = extra_pool_by_key.get(key)
        sim_exact = ev_u[ev_idx] @ tr_u[cand_idx].T
        sim_exact = maybe_blend_complex_scores(
            sim_exact,
            eval_z[ev_idx],
            train_z[cand_idx],
            dim_i=complex_dim_i,
            dim_j=complex_dim_j,
            blend_lambda=complex_rerank_lambda,
            mode=complex_rerank_mode,
            margin_threshold=complex_rerank_margin_threshold,
        )
        sim_exact = maybe_blend_event_gate_scores(
            sim_exact,
            train_score_bias=train_score_bias,
            candidate_idx=cand_idx,
            bias_lambda=score_bias_lambda,
        )
        y_block, tok_block = topk_reduce(sim_exact, train_y[cand_idx], train_tok[cand_idx], topk=topk)
        yhat[ev_idx] = y_block
        pred_tok[ev_idx] = tok_block
        candidate_counts[ev_idx] = float(cand_idx.shape[0])
        _record_query_audit_rows(
            query_audit,
            ev_idx,
            sim_exact,
            train_tok[cand_idx],
            eval_target_tok,
            topk=topk,
            n_train=denom,
        )
        if extra_pool is None or extra_pool.size == 0:
            continue
        if complex_backfill_mode == "small_bucket":
            max_exact = max(1, int(complex_backfill_max_exact))
            trigger_mask = np.full((ev_idx.shape[0],), cand_idx.shape[0] <= max_exact, dtype=bool)
        elif complex_backfill_mode == "low_margin":
            if cand_idx.shape[0] <= 1:
                trigger_mask = np.ones((ev_idx.shape[0],), dtype=bool)
            else:
                top2 = min(2, cand_idx.shape[0])
                best2 = np.argpartition(-sim_exact, kth=top2 - 1, axis=1)[:, :top2]
                top2_vals = np.take_along_axis(sim_exact, best2, axis=1)
                top2_vals.sort(axis=1)
                margins = top2_vals[:, -1] - top2_vals[:, -2]
                trigger_mask = margins <= float(complex_backfill_margin_threshold)
        else:
            trigger_mask = np.ones((ev_idx.shape[0],), dtype=bool)
        if not np.any(trigger_mask):
            continue
        backfill_n = min(int(complex_backfill_items), int(extra_pool.size))
        chosen_eval_idx = ev_idx[trigger_mask]
        extra_sim = ev_u[chosen_eval_idx] @ tr_u[extra_pool].T
        extra_sim = maybe_blend_complex_scores(
            extra_sim,
            eval_z[chosen_eval_idx],
            train_z[extra_pool],
            dim_i=complex_dim_i,
            dim_j=complex_dim_j,
            blend_lambda=complex_rerank_lambda,
            mode=complex_rerank_mode,
            margin_threshold=complex_rerank_margin_threshold,
        )
        extra_idx = np.argpartition(-extra_sim, kth=backfill_n - 1, axis=1)[:, :backfill_n]
        part = np.take_along_axis(extra_sim, extra_idx, axis=1)
        order = np.argsort(-part, axis=1)
        extra_idx = np.take_along_axis(extra_idx, order, axis=1)
        for local_row, row_idx in enumerate(chosen_eval_idx):
            cand_idx_row = np.unique(np.concatenate([cand_idx, extra_pool[extra_idx[local_row]]]))
            sim = ev_u[row_idx:row_idx + 1] @ tr_u[cand_idx_row].T
            sim = maybe_blend_complex_scores(
                sim,
                eval_z[row_idx:row_idx + 1],
                train_z[cand_idx_row],
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                blend_lambda=complex_rerank_lambda,
                mode=complex_rerank_mode,
                margin_threshold=complex_rerank_margin_threshold,
            )
            sim = maybe_blend_event_gate_scores(
                sim,
                train_score_bias=train_score_bias,
                candidate_idx=cand_idx_row,
                bias_lambda=score_bias_lambda,
            )
            y_block, tok_block = topk_reduce(sim, train_y[cand_idx_row], train_tok[cand_idx_row], topk=topk)
            yhat[row_idx:row_idx + 1] = y_block
            pred_tok[row_idx] = tok_block[0]
            candidate_counts[row_idx] = float(cand_idx_row.shape[0])
            backfill_trigger[row_idx] = 1.0
            backfill_added[row_idx] = float(cand_idx_row.shape[0] - cand_idx.shape[0])
            _record_query_audit_rows(
                query_audit,
                np.array([row_idx], dtype=np.int64),
                sim,
                train_tok[cand_idx_row],
                eval_target_tok,
                topk=topk,
                n_train=denom,
            )

    result = (
        yhat,
        pred_tok,
        float(np.mean(candidate_counts)),
        float(np.mean(candidate_counts) / max(1.0, float(denom))),
        1.0,
        float(np.mean(fallback)),
        float(np.mean(backfill_trigger)),
        float(np.mean(backfill_added)),
    )
    if query_audit is not None:
        return result + (query_audit,)
    return result


def routed_retrieval(
    train_keys: Sequence[Tuple[int, ...]],
    eval_keys: Sequence[Tuple[int, ...]],
    train_z: np.ndarray,
    train_y: np.ndarray,
    train_tok: np.ndarray,
    eval_z: np.ndarray,
    topk: int,
    probe_buckets: int,
    candidate_fraction_denominator: Optional[int] = None,
    complex_backfill_items: int = 0,
    complex_backfill_mode: str = "always",
    complex_backfill_max_exact: int = 0,
    complex_backfill_margin_threshold: float = 0.0,
    complex_rerank_mode: str = "none",
    complex_rerank_lambda: float = 0.0,
    complex_rerank_margin_threshold: float = 0.0,
    complex_dim_i: int = 0,
    complex_dim_j: int = 1,
    train_score_bias: Optional[np.ndarray] = None,
    score_bias_lambda: float = 0.0,
    prepared_same_bucket_plan: Optional[GroupedSameBucketPlan] = None,
    return_query_audit: bool = False,
    eval_target_tok: Optional[np.ndarray] = None,
):
    if return_query_audit and eval_target_tok is None:
        raise ValueError("eval_target_tok is required when return_query_audit=True")
    denom = int(candidate_fraction_denominator or train_z.shape[0])
    bucket_to_idx = build_bucket_index(train_keys)
    bucket_keys = list(bucket_to_idx.keys())
    if not bucket_keys:
        dense_result = dense_retrieval(
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=topk,
            candidate_fraction_denominator=denom,
            train_score_bias=train_score_bias,
            score_bias_lambda=score_bias_lambda,
            return_query_audit=return_query_audit,
            eval_target_tok=eval_target_tok,
        )
        if return_query_audit:
            yhat, pred_tok, cand_mean, cand_frac, query_audit = dense_result
            return yhat, pred_tok, cand_mean, cand_frac, 0.0, 1.0, 0.0, 0.0, query_audit
        yhat, pred_tok, cand_mean, cand_frac = dense_result
        return yhat, pred_tok, cand_mean, cand_frac, 0.0, 1.0, 0.0, 0.0
    if probe_buckets <= 1:
        return routed_retrieval_grouped_same_bucket(
            train_keys,
            eval_keys,
            train_z,
            train_y,
            train_tok,
            eval_z,
            topk=topk,
            candidate_fraction_denominator=denom,
            complex_backfill_items=complex_backfill_items,
            complex_backfill_mode=complex_backfill_mode,
            complex_backfill_max_exact=complex_backfill_max_exact,
            complex_backfill_margin_threshold=complex_backfill_margin_threshold,
            complex_rerank_mode=complex_rerank_mode,
            complex_rerank_lambda=complex_rerank_lambda,
            complex_rerank_margin_threshold=complex_rerank_margin_threshold,
            complex_dim_i=complex_dim_i,
            complex_dim_j=complex_dim_j,
            train_score_bias=train_score_bias,
            score_bias_lambda=score_bias_lambda,
            prepared_plan=prepared_same_bucket_plan,
            return_query_audit=return_query_audit,
            eval_target_tok=eval_target_tok,
        )

    tr_u = _normalize_rows(train_z)
    ev_u = _normalize_rows(eval_z)
    centroids = np.stack([np.mean(tr_u[idx], axis=0) for idx in bucket_to_idx.values()], axis=0)
    centroids_u = _normalize_rows(centroids)
    yhat = np.zeros((ev_u.shape[0], train_y.shape[1]), dtype=np.float64)
    pred_tok = np.zeros((ev_u.shape[0],), dtype=np.int32)
    candidate_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    probe_counts = np.zeros((ev_u.shape[0],), dtype=np.float64)
    fallback = np.zeros((ev_u.shape[0],), dtype=np.float64)
    query_audit = _new_query_audit(ev_u.shape[0]) if return_query_audit else None

    for i in range(ev_u.shape[0]):
        q = ev_u[i]
        if probe_buckets <= 1:
            selected = [eval_keys[i]] if eval_keys[i] in bucket_to_idx else []
        else:
            sim = centroids_u @ q
            kk = min(probe_buckets, len(bucket_keys))
            probe_idx = np.argpartition(-sim, kth=kk - 1)[:kk]
            probe_idx = probe_idx[np.argsort(-sim[probe_idx])]
            selected = [bucket_keys[j] for j in probe_idx]
            if eval_keys[i] in bucket_to_idx and eval_keys[i] not in selected:
                selected = [eval_keys[i]] + selected[:-1]
        cand_parts = [bucket_to_idx[k] for k in selected if k in bucket_to_idx]
        if not cand_parts:
            cand_idx = np.arange(train_z.shape[0], dtype=np.int64)
            fallback[i] = 1.0
        else:
            cand_idx = np.unique(np.concatenate(cand_parts))
        local_sim = tr_u[cand_idx] @ q
        local_sim = maybe_blend_complex_scores(
            local_sim[None, :],
            eval_z[i:i + 1],
            train_z[cand_idx],
            dim_i=complex_dim_i,
            dim_j=complex_dim_j,
            blend_lambda=complex_rerank_lambda,
            mode=complex_rerank_mode,
            margin_threshold=complex_rerank_margin_threshold,
        )[0]
        local_sim = maybe_blend_event_gate_scores(
            local_sim,
            train_score_bias=train_score_bias,
            candidate_idx=cand_idx,
            bias_lambda=score_bias_lambda,
        )
        kk = min(topk, cand_idx.shape[0])
        best = np.argpartition(-local_sim, kth=kk - 1)[:kk]
        best = best[np.argsort(-local_sim[best])]
        top_idx = cand_idx[best]
        yhat[i] = np.mean(train_y[top_idx], axis=0)
        pred_tok[i] = vote_top1(train_tok[top_idx])
        candidate_counts[i] = float(cand_idx.shape[0])
        probe_counts[i] = float(len(selected)) if selected else 1.0
        _record_query_audit_rows(
            query_audit,
            np.array([i], dtype=np.int64),
            local_sim[None, :],
            train_tok[cand_idx],
            eval_target_tok,
            topk=topk,
            n_train=denom,
        )

    result = (
        yhat,
        pred_tok,
        float(np.mean(candidate_counts)),
        float(np.mean(candidate_counts) / max(1.0, float(train_z.shape[0]))),
        float(np.mean(probe_counts)),
        float(np.mean(fallback)),
        0.0,
        0.0,
    )
    if query_audit is not None:
        return result + (query_audit,)
    return result


def main():
    args = parse_args()
    t_total_start = time.perf_counter()
    timings = {
        "dataset": 0.0,
        "chart_opt": 0.0,
        "routing_eval": 0.0,
        "route_index_build": 0.0,
        "query_route": 0.0,
        "retrieval_search": 0.0,
        "offline_total": 0.0,
        "online_total": 0.0,
        "training_route": 0.0,
        "training_update": 0.0,
        "training_ema": 0.0,
        "growth": 0.0,
        "total": 0.0,
    }
    notes: List[str] = []
    artifacts = {
        "input": args.input,
        "tokens_input": args.tokens_input,
        "eval_split": args.eval_split,
        "chart_cache_file": "",
        "route_cache_file": "",
        "retrieval_query_audit_file": "",
        "run_tag": args.run_tag,
    }

    apply_retrieval_fast_dev(args)
    args.query_repeats = max(1, int(args.query_repeats))

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
    y_train_tok_all, y_eval_tok_all = load_token_targets(args.tokens_input, args.proxy_meta, args.eval_split)
    timings["dataset"] = time.perf_counter() - t0

    v_tr, y_tr, y_tr_tok, tr_source_idx = _subset_with_index(x_train, y_train, y_train_tok_all, args.max_train, args.seed + 1)
    v_ev, y_ev, y_ev_tok, ev_source_idx = _subset_with_index(x_eval, y_eval, y_eval_tok_all, args.max_eval, args.seed + 2)

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

    train_label_sse_per = global_mean_mse(y_tr, y_tr)
    eval_label_sse_per = global_mean_mse(y_tr, y_ev)
    mse_before = eval_label_sse_per

    chart = hr.Chart(R=np.eye(d, dtype=np.float64), s_global=None, S_radial=None, scale_mode="global")
    C_used = None
    z_tr = v_tr
    v_tr_effective = v_tr
    z_ev = v_ev
    keys_tr: List[Tuple[int, ...]] = []
    keys_ev: List[Tuple[int, ...]] = []

    yhat = np.zeros((v_ev.shape[0], dy), dtype=np.float64)
    pred_tok = np.zeros((v_ev.shape[0],), dtype=np.int32)
    candidate_count_mean = 0.0
    candidate_fraction_mean = 1.0
    probe_bucket_mean = 0.0
    bucket_fallback_rate = 0.0
    backfill_trigger_rate = 0.0
    backfill_extra_candidates_mean = 0.0
    secondary_key_count = 0
    chart_cache_hit = False
    route_cache_hit = False
    event_gate_error_mean = 0.0
    event_gate_mean = 1.0
    event_gate_active_frac = 1.0
    event_gate_cost_proxy = 1.0
    prepared_same_bucket_plan = None
    event_gate_retrieval_surface_active = int(event_gate_changes_retrieval_surface(args))
    event_gate_translation_keep_frac = 1.0
    event_gate_translation_pruned_frac = 0.0
    retrieval_train_items_effective = int(v_tr.shape[0])
    event_gate_translation_score_bias_abs_mean = 0.0
    train_score_bias = None

    if args.retrieval_backend == "routed_probe":
        if int(args.cache_chart) == 1 or int(args.cache_routes) == 1:
            hr.ensure_dir(args.cache_dir)
        if args.sector_mode == "kmeans":
            U0 = hr.normalize_rows(v_tr)
            C_used = hr.spherical_kmeans(U0, K=args.K, iters=args.kmeans_iters, seed=args.seed + 11)

        learned_chart = (args.learn_so8 == 1 or args.learn_scale == 1)
        chart_cache_file = ""
        if learned_chart:
            if int(args.cache_chart) == 1:
                chart_key = hr.stable_hash(retrieval_chart_cache_payload(args, d=d, dy=dy, n_train=v_tr.shape[0]))
                chart_cache_file = os.path.join(args.cache_dir, f"retrieval_chart_{chart_key}.npz")
                artifacts["chart_cache_file"] = chart_cache_file
                if os.path.exists(chart_cache_file):
                    t_chart = time.perf_counter()
                    chart = _load_chart_cache(chart_cache_file)
                    timings["chart_opt"] = time.perf_counter() - t_chart
                    chart_cache_hit = True
                    notes.append("retrieval chart cache hit")
            if not chart_cache_hit:
                t_chart = time.perf_counter()
                opt_res = hr.optimize_chart(
                    v_train=v_tr,
                    y_train=y_tr,
                    delta_r=args.delta_r,
                    C0=C_used,
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
                    field4_dim_i=field4_dim_i,
                    field4_dim_j=field4_dim_j,
                    field4_dim_k=field4_dim_k,
                    field4_dim_l=field4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    K=args.K,
                    seed=args.seed,
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
                chart = opt_res.chart
                timings["chart_opt"] = time.perf_counter() - t_chart
                notes.append(f"retrieval chart iters={len(opt_res.loss_hist) - 1}")
                if int(args.cache_chart) == 1:
                    if not chart_cache_file:
                        chart_key = hr.stable_hash(retrieval_chart_cache_payload(args, d=d, dy=dy, n_train=v_tr.shape[0]))
                        chart_cache_file = os.path.join(args.cache_dir, f"retrieval_chart_{chart_key}.npz")
                        artifacts["chart_cache_file"] = chart_cache_file
                    _save_chart_cache(chart_cache_file, chart)
        route_cache_file = ""
        if int(args.cache_routes) == 1:
            chart_fp = hr.chart_fingerprint(chart)
            route_key = hr.stable_hash(retrieval_route_cache_payload(args, chart_fp=chart_fp, n_train=v_tr.shape[0]))
            route_cache_file = os.path.join(args.cache_dir, f"retrieval_train_routes_{route_key}.npz")
            artifacts["route_cache_file"] = route_cache_file
            if os.path.exists(route_cache_file):
                t_route_tr = time.perf_counter()
                shell_tr, sector_tr, z_tr = _load_route_cache(route_cache_file)
                timings["route_index_build"] = time.perf_counter() - t_route_tr
                route_cache_hit = True
                notes.append("retrieval route cache hit")
        if not route_cache_hit:
            t_route_tr = time.perf_counter()
            shell_tr, sector_tr, _, z_tr = hr.route_addresses(
                v_tr, delta_r=args.delta_r, C=C_used, chart=chart,
                sector_mode=args.sector_mode,
                phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
                phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
                phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
                field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
                field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
                complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
                K=args.K,
                time_pressure_lambda=args.time_pressure_lambda, tau=1.0,
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
            timings["route_index_build"] = time.perf_counter() - t_route_tr
            if int(args.cache_routes) == 1:
                if not route_cache_file:
                    chart_fp = hr.chart_fingerprint(chart)
                    route_key = hr.stable_hash(retrieval_route_cache_payload(args, chart_fp=chart_fp, n_train=v_tr.shape[0]))
                    route_cache_file = os.path.join(args.cache_dir, f"retrieval_train_routes_{route_key}.npz")
                    artifacts["route_cache_file"] = route_cache_file
                _save_route_cache(route_cache_file, shell_tr=shell_tr, sector_tr=sector_tr, z_tr=z_tr)
        base_keys_tr = [hr.make_bucket_key(int(shell_tr[i]), int(sector_tr[i])) for i in range(shell_tr.shape[0])]
        event_audit_route_one = None
        if args.train_route_mode != "final_static":
            def event_audit_route_one(v_row: np.ndarray, tau_sched: float):
                return hr.route_one(
                    v_row,
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
                    field4_dim_i=field4_dim_i,
                    field4_dim_j=field4_dim_j,
                    field4_dim_k=field4_dim_k,
                    field4_dim_l=field4_dim_l,
                    complex_dim_i=complex_dim_i,
                    complex_dim_j=complex_dim_j,
                    K=args.K,
                    time_pressure_lambda=float(args.time_pressure_lambda),
                    tau=tau_sched,
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

        event_audit = sparse_event_training_audit(
            train_keys_static=base_keys_tr,
            z_tr_static=z_tr,
            v_tr=v_tr,
            y_tr=y_tr,
            d=d,
            dy=dy,
            eta_p=args.eta_p,
            eta_m=args.eta_m,
            epochs=args.epochs,
            seed=args.seed,
            event_gate_mode=args.event_gate_mode,
            event_gate_threshold=args.event_gate_threshold,
            event_gate_tau=args.event_gate_tau,
            route_one_fn=event_audit_route_one,
        )
        event_gate_error_mean = float(event_audit["event_gate_error_mean"])
        event_gate_mean = float(event_audit["event_gate_mean"])
        event_gate_active_frac = float(event_audit["event_gate_active_frac"])
        event_gate_cost_proxy = float(event_audit["event_gate_cost_proxy"])
        timings["training_ema"] = float(event_audit["training_total_sec"])
        timings["training_route"] = float(event_audit["training_route_sec"])
        timings["training_update"] = float(event_audit["training_update_sec"])
        notes.append(
            "event_gate="
            f"mode={args.event_gate_mode} "
            f"threshold={args.event_gate_threshold:.6f} "
            f"tau={args.event_gate_tau:.6f} "
            f"error_mean={event_gate_error_mean:.6f} "
            f"gate_mean={event_gate_mean:.6f} "
            f"active_frac={event_gate_active_frac:.6f} "
            f"cost_proxy={event_gate_cost_proxy:.6f}"
        )
        if args.event_gate_mode != "off" and not event_gate_retrieval_surface_active:
            notes.append(
                "event_gate_retrieval_surface=inactive "
                "(sparse-event knobs affect training audit metrics only; "
                "translated retrieval still uses fixed routed embeddings)"
            )
        if event_gate_retrieval_surface_active and args.event_gate_translation_coupling == "train_gate_prune":
            train_mask = build_event_gate_train_mask(
                train_keys=base_keys_tr,
                sample_gate_mean=event_audit["sample_gate_mean"],
                threshold=args.event_gate_translation_prune_threshold,
                min_keep_per_bucket=args.event_gate_translation_min_keep_per_bucket,
            )
            keep_idx = np.flatnonzero(train_mask)
            base_keys_tr = [base_keys_tr[i] for i in keep_idx]
            v_tr_effective = v_tr[keep_idx]
            z_tr = z_tr[keep_idx]
            y_tr = y_tr[keep_idx]
            y_tr_tok = y_tr_tok[keep_idx]
            event_gate_translation_keep_frac = float(np.mean(train_mask))
            event_gate_translation_pruned_frac = float(1.0 - event_gate_translation_keep_frac)
            retrieval_train_items_effective = int(z_tr.shape[0])
            notes.append(
                "event_gate_retrieval_surface=active "
                f"mode={args.event_gate_translation_coupling} "
                f"threshold={args.event_gate_translation_prune_threshold:.6f} "
                f"min_keep_per_bucket={args.event_gate_translation_min_keep_per_bucket} "
                f"keep_frac={event_gate_translation_keep_frac:.6f}"
            )
        elif event_gate_retrieval_surface_active and args.event_gate_translation_coupling == "train_gate_score_bias":
            train_score_bias = build_event_gate_train_score_bias(event_audit["sample_gate_mean"])
            if train_score_bias.size:
                event_gate_translation_score_bias_abs_mean = float(np.mean(np.abs(train_score_bias)))
            notes.append(
                "event_gate_retrieval_surface=active "
                f"mode={args.event_gate_translation_coupling} "
                f"score_bias_lambda={args.event_gate_translation_score_bias_lambda:.6f} "
                f"bias_abs_mean={event_gate_translation_score_bias_abs_mean:.6f}"
            )
        keys_tr = list(base_keys_tr)
        if args.route_key_mode == "hopf_plus_complex":
            keys_tr, secondary_key_count_tr = augment_route_keys_with_complex(
                base_keys=keys_tr,
                field=z_tr,
                dim_i=complex_dim_i,
                dim_j=complex_dim_j,
                roots=args.complex_key_roots,
                radius_bins=args.complex_key_radius_bins,
            )
            secondary_key_count = max(secondary_key_count, int(secondary_key_count_tr))
        train_key_set = set(keys_tr)

        def run_retrieval_only(
            z_ev_local: np.ndarray,
            keys_ev_local: Sequence[Tuple[int, ...]],
            secondary_key_count_local: int,
            prepared_same_bucket_plan: Optional[GroupedSameBucketPlan] = None,
        ):
            t_retr = time.perf_counter()
            (
                yhat_local,
                pred_tok_local,
                candidate_count_local,
                candidate_fraction_local,
                probe_bucket_local,
                fallback_local,
                backfill_trigger_local,
                backfill_added_local,
            ) = routed_retrieval(
                keys_tr, keys_ev_local, z_tr, y_tr, y_tr_tok, z_ev_local,
                topk=args.topk,
                probe_buckets=args.probe_buckets,
                candidate_fraction_denominator=v_tr.shape[0],
                complex_backfill_items=args.complex_backfill_items,
                complex_backfill_mode=args.complex_backfill_mode,
                complex_backfill_max_exact=args.complex_backfill_max_exact,
                complex_backfill_margin_threshold=args.complex_backfill_margin_threshold,
                complex_rerank_mode=args.complex_rerank_mode,
                complex_rerank_lambda=args.complex_rerank_lambda,
                complex_rerank_margin_threshold=args.complex_rerank_margin_threshold,
                complex_dim_i=complex_dim_i,
                complex_dim_j=complex_dim_j,
                train_score_bias=train_score_bias,
                score_bias_lambda=args.event_gate_translation_score_bias_lambda,
                prepared_same_bucket_plan=prepared_same_bucket_plan,
            )
            retrieval_search_sec = time.perf_counter() - t_retr
            return {
                "z_ev": z_ev_local,
                "keys_ev": keys_ev_local,
                "query_route_sec": 0.0,
                "retrieval_search_sec": retrieval_search_sec,
                "yhat": yhat_local,
                "pred_tok": pred_tok_local,
                "candidate_count_mean": candidate_count_local,
                "candidate_fraction_mean": candidate_fraction_local,
                "probe_bucket_mean": probe_bucket_local,
                "bucket_fallback_rate": fallback_local,
                "backfill_trigger_rate": backfill_trigger_local,
                "backfill_extra_candidates_mean": backfill_added_local,
                "secondary_key_count": secondary_key_count_local,
                "prepared_same_bucket_plan": prepared_same_bucket_plan,
            }

        def run_routed_query_pass():
            t_route_ev = time.perf_counter()
            shell_ev, sector_ev, _, z_ev_local = hr.route_addresses(
                v_ev, delta_r=args.delta_r, C=C_used, chart=chart,
                sector_mode=args.sector_mode,
                phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
                phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
                phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
                field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
                field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
                complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
                K=args.K,
                time_pressure_lambda=args.time_pressure_lambda, tau=1.0,
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
            query_route_sec = time.perf_counter() - t_route_ev
            keys_ev_local = [hr.make_bucket_key(int(shell_ev[i]), int(sector_ev[i])) for i in range(shell_ev.shape[0])]
            secondary_key_count_local = 0
            if args.route_key_mode == "hopf_plus_complex":
                keys_ev_local, secondary_key_count_local = augment_route_keys_with_complex(
                    base_keys=keys_ev_local,
                    field=z_ev_local,
                    dim_i=complex_dim_i,
                    dim_j=complex_dim_j,
                    roots=args.complex_key_roots,
                    radius_bins=args.complex_key_radius_bins,
                )
            prepared_same_bucket_plan = None
            if args.probe_buckets <= 1:
                prepared_same_bucket_plan = prepare_grouped_same_bucket_plan(
                    train_keys=keys_tr,
                    eval_keys=keys_ev_local,
                    train_z=z_tr,
                    train_y=y_tr,
                    train_tok=y_tr_tok,
                    eval_z=z_ev_local,
                    complex_backfill_items=args.complex_backfill_items,
                )
            result = run_retrieval_only(
                z_ev_local=z_ev_local,
                keys_ev_local=keys_ev_local,
                secondary_key_count_local=secondary_key_count_local,
                prepared_same_bucket_plan=prepared_same_bucket_plan,
            )
            result["shell_ev"] = shell_ev
            result["sector_ev"] = sector_ev
            result["query_route_sec"] = query_route_sec
            return result

        first_pass = run_routed_query_pass()
        z_ev = first_pass["z_ev"]
        keys_ev = first_pass["keys_ev"]
        yhat = first_pass["yhat"]
        pred_tok = first_pass["pred_tok"]
        candidate_count_mean = float(first_pass["candidate_count_mean"])
        candidate_fraction_mean = float(first_pass["candidate_fraction_mean"])
        probe_bucket_mean = float(first_pass["probe_bucket_mean"])
        bucket_fallback_rate = float(first_pass["bucket_fallback_rate"])
        backfill_trigger_rate = float(first_pass["backfill_trigger_rate"])
        backfill_extra_candidates_mean = float(first_pass["backfill_extra_candidates_mean"])
        secondary_key_count = max(int(secondary_key_count), int(first_pass["secondary_key_count"]))
        prepared_same_bucket_plan = first_pass.get("prepared_same_bucket_plan")
        timings["query_route"] += float(first_pass["query_route_sec"])
        timings["retrieval_search"] += float(first_pass["retrieval_search_sec"])

        unseen_rate = hr.unseen_key_rate(keys_ev, train_key_set)
        pmax_after, entropy_after, shell_pmax, buckets, eval_shells, eval_sectors = key_stats(keys_ev)
        sector_pmax = 0.0
        if keys_ev:
            sectors = np.array([k[1] for k in keys_ev], dtype=np.int64)
            _, sector_counts = np.unique(sectors, return_counts=True)
            sector_pmax = float(np.max(sector_counts) / np.sum(sector_counts)) if len(sector_counts) else 0.0
        train_label_sse = hr.label_coherence_sse(
            v_tr_effective, y_tr, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
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
        eval_label_sse = hr.label_coherence_sse(
            v_ev, y_ev, delta_r=args.delta_r, C=C_used, chart=chart,
            sector_mode=args.sector_mode,
            phase_dim_i=phase_dim_i, phase_dim_j=phase_dim_j,
            phase4_dim_i=phase4_dim_i, phase4_dim_j=phase4_dim_j,
            phase4_dim_k=phase4_dim_k, phase4_dim_l=phase4_dim_l,
            field4_dim_i=field4_dim_i, field4_dim_j=field4_dim_j,
            field4_dim_k=field4_dim_k, field4_dim_l=field4_dim_l,
            complex_dim_i=complex_dim_i, complex_dim_j=complex_dim_j,
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
        train_label_sse_per = float(train_label_sse / max(1, v_tr_effective.shape[0]))
        eval_label_sse_per = float(eval_label_sse / max(1, v_ev.shape[0]))
        alignment = hr.poincare_alignment_diagnostics(v_ev, hr.route_coordinate(v_ev, chart, args.sector_mode, args.route_scale_lambda))

        if args.query_repeats > 1:
            notes.append("reuse_eval_routes_across_repeats=1")
        for _ in range(1, args.query_repeats):
            repeat_pass = run_retrieval_only(
                z_ev_local=z_ev,
                keys_ev_local=keys_ev,
                secondary_key_count_local=secondary_key_count,
                prepared_same_bucket_plan=prepared_same_bucket_plan,
            )
            timings["retrieval_search"] += float(repeat_pass["retrieval_search_sec"])
            bucket_fallback_rate = float(max(bucket_fallback_rate, float(repeat_pass["bucket_fallback_rate"])))
            backfill_trigger_rate = float(max(backfill_trigger_rate, float(repeat_pass["backfill_trigger_rate"])))
            backfill_extra_candidates_mean = float(max(backfill_extra_candidates_mean, float(repeat_pass["backfill_extra_candidates_mean"])))
            secondary_key_count = max(int(secondary_key_count), int(repeat_pass["secondary_key_count"]))
    else:
        unseen_rate = 0.0
        pmax_after = 1.0
        entropy_after = 0.0
        shell_pmax = 1.0
        sector_pmax = 1.0
        buckets = 1
        eval_shells = 1
        eval_sectors = 1
        alignment = hr.poincare_alignment_diagnostics(v_ev, v_ev)
        notes.append("dense exact retrieval baseline")
        backfill_trigger_rate = 0.0
        backfill_extra_candidates_mean = 0.0
        for _ in range(args.query_repeats):
            t_retr = time.perf_counter()
            yhat, pred_tok, candidate_count_mean, candidate_fraction_mean = dense_retrieval(
                z_tr, y_tr, y_tr_tok, z_ev, topk=args.topk
            )
            timings["retrieval_search"] += time.perf_counter() - t_retr

    if args.retrieval_audit_dir:
        audit_run_tag = args.run_tag or f"retrieval_seed{args.seed}_{args.retrieval_backend}"
        audit_path = os.path.join(args.retrieval_audit_dir, f"{audit_run_tag}.npz")
        if args.retrieval_backend == "dense_exact":
            _, _, _, _, query_audit = dense_retrieval(
                z_tr,
                y_tr,
                y_tr_tok,
                z_ev,
                topk=args.topk,
                return_query_audit=True,
                eval_target_tok=y_ev_tok,
            )
        else:
            _, _, _, _, _, _, _, _, query_audit = routed_retrieval(
                keys_tr,
                keys_ev,
                z_tr,
                y_tr,
                y_tr_tok,
                z_ev,
                topk=args.topk,
                probe_buckets=args.probe_buckets,
                complex_backfill_items=args.complex_backfill_items,
                complex_backfill_mode=args.complex_backfill_mode,
                complex_backfill_max_exact=args.complex_backfill_max_exact,
                complex_backfill_margin_threshold=args.complex_backfill_margin_threshold,
                complex_rerank_mode=args.complex_rerank_mode,
                complex_rerank_lambda=args.complex_rerank_lambda,
                complex_rerank_margin_threshold=args.complex_rerank_margin_threshold,
                complex_dim_i=complex_dim_i,
                complex_dim_j=complex_dim_j,
                prepared_same_bucket_plan=prepared_same_bucket_plan,
                return_query_audit=True,
                eval_target_tok=y_ev_tok,
            )
        save_retrieval_query_audit(
            audit_path,
            eval_source_idx=ev_source_idx,
            target_tok=y_ev_tok,
            query_audit=query_audit,
        )
        artifacts["retrieval_query_audit_file"] = audit_path
        notes.append("retrieval_query_audit=1")

    timings["routing_eval"] = timings["route_index_build"] + timings["query_route"] + timings["retrieval_search"]
    timings["offline_total"] = timings["chart_opt"] + timings["route_index_build"]
    timings["online_total"] = timings["query_route"] + timings["retrieval_search"]
    online_per_repeat_sec, amortized_per_repeat_sec = compute_amortized_retrieval_metrics(
        timings["offline_total"], timings["online_total"], args.query_repeats
    )

    mse_after = float(np.mean((yhat - y_ev) ** 2))
    top1_after = float(np.mean(pred_tok == y_ev_tok)) if y_ev_tok.size else 0.0
    timings["total"] = time.perf_counter() - t_total_start
    notes.append(f"query_repeats={args.query_repeats}")

    summary = {
        "schema_version": "1.0",
        "parsed": True,
        "args": {k: v for k, v in vars(args).items()},
        "metrics": {
            "test_mse_before": float(mse_before),
            "test_mse_after": float(mse_after),
            "train_label_sse_per": float(train_label_sse_per),
            "test_label_sse_per": float(eval_label_sse_per),
            "buckets": int(buckets),
            "slots_used": int(buckets),
            "test_unseen_rate": float(unseen_rate),
            "pmax_before": 1.0,
            "pmax_after": float(pmax_after),
            "entropy_before": 0.0,
            "entropy_after": float(entropy_after),
            "eval_shells": int(eval_shells),
            "eval_sectors": int(eval_sectors),
            "shell_pmax": float(shell_pmax),
            "sector_pmax": float(sector_pmax),
            "shell_entropy": 0.0,
            "sector_entropy": 0.0,
            "poincare_alignment_pairs_used": int(alignment["poincare_alignment_pairs_used"]),
            "poincare_alignment_radial_mae": float(alignment["poincare_alignment_radial_mae"]),
            "poincare_alignment_radial_rel_mean": float(alignment["poincare_alignment_radial_rel_mean"]),
            "poincare_alignment_radial_corr": float(alignment["poincare_alignment_radial_corr"]),
            "poincare_alignment_pair_mae": float(alignment["poincare_alignment_pair_mae"]),
            "poincare_alignment_pair_rel_mean": float(alignment["poincare_alignment_pair_rel_mean"]),
            "poincare_alignment_pair_corr": float(alignment["poincare_alignment_pair_corr"]),
            "new_slots": 0,
            "accepted_splits": 0,
            "n_buckets_total": int(buckets),
            "test_top1_after": float(top1_after),
            "retrieval_candidate_count_mean": float(candidate_count_mean),
            "retrieval_candidate_fraction_mean": float(candidate_fraction_mean),
            "retrieval_probe_bucket_mean": float(probe_bucket_mean),
            "retrieval_bucket_fallback_rate": float(bucket_fallback_rate),
            "retrieval_backfill_trigger_rate": float(backfill_trigger_rate),
            "retrieval_backfill_extra_candidates_mean": float(backfill_extra_candidates_mean),
            "retrieval_secondary_key_count": int(secondary_key_count),
            "retrieval_chart_cache_hit": int(chart_cache_hit),
            "retrieval_route_cache_hit": int(route_cache_hit),
            "retrieval_train_items": int(v_tr.shape[0]),
            "retrieval_train_items_effective": int(retrieval_train_items_effective),
            "retrieval_eval_items": int(v_ev.shape[0]),
            "retrieval_training_total_sec": float(timings["training_ema"]),
            "retrieval_training_route_sec": float(timings["training_route"]),
            "retrieval_training_update_sec": float(timings["training_update"]),
            "retrieval_query_repeats": int(args.query_repeats),
            "retrieval_offline_total_sec": float(timings["offline_total"]),
            "retrieval_online_total_sec": float(timings["online_total"]),
            "retrieval_online_total_per_repeat_sec": float(online_per_repeat_sec),
            "retrieval_total_amortized_per_repeat_sec": float(amortized_per_repeat_sec),
            "event_gate_error_mean": float(event_gate_error_mean),
            "event_gate_mean": float(event_gate_mean),
            "event_gate_active_frac": float(event_gate_active_frac),
            "event_gate_cost_proxy": float(event_gate_cost_proxy),
            "event_gate_retrieval_surface_active": int(event_gate_retrieval_surface_active),
            "event_gate_translation_keep_frac": float(event_gate_translation_keep_frac),
            "event_gate_translation_pruned_frac": float(event_gate_translation_pruned_frac),
            "event_gate_translation_score_bias_abs_mean": float(event_gate_translation_score_bias_abs_mean),
        },
        "timings_sec": {k: float(v) for k, v in timings.items()},
        "artifacts": {
            **artifacts,
            "chart_cache_hit": int(chart_cache_hit),
            "route_cache_hit": int(route_cache_hit),
        },
        "git": hr.maybe_git_info(),
        "notes": notes,
    }
    print("__JSON_SUMMARY__ " + json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
