#!/usr/bin/env python3
import argparse
import datetime
import json
import os
import subprocess
import sys
from typing import Any, Dict, List, Optional


SUMMARY_PREFIX = "__JSON_SUMMARY__"


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def parse_summary_from_log(log_path: str) -> Optional[Dict[str, Any]]:
    payload = None
    with open(log_path, "r", encoding="utf-8", errors="ignore") as f:
        for line in f:
            line = line.strip()
            if line.startswith(SUMMARY_PREFIX):
                payload = line[len(SUMMARY_PREFIX):].strip()
    if payload is None:
        return None
    try:
        return json.loads(payload)
    except Exception:
        return None


def build_arg_items(common_args: Dict[str, Any], route_args: Dict[str, Any], seed: int, run_tag: str) -> List[str]:
    merged = dict(common_args)
    merged.update(route_args)
    merged["seed"] = seed
    merged["run_tag"] = run_tag
    out: List[str] = []
    for key, value in merged.items():
        out.append(f"--{key}")
        out.append(str(value))
    return out


def run_one(cmd: List[str], log_path: str) -> int:
    with open(log_path, "w", encoding="utf-8") as lf:
        proc = subprocess.run(cmd, stdout=lf, stderr=subprocess.STDOUT)
    return int(proc.returncode)


def build_run_plan(routes: List[Dict[str, Any]], seeds: List[int], run_order: str) -> List[tuple]:
    plan: List[tuple] = []
    if run_order == "seed_major":
        for seed in seeds:
            for route in routes:
                plan.append((route, seed))
        return plan
    for route in routes:
        for seed in seeds:
            plan.append((route, seed))
    return plan


def metric_mean(summaries: List[Dict[str, Any]], metric_name: str, scope: str = "metrics") -> float:
    vals: List[float] = []
    for summary in summaries:
        block = summary.get(scope, {})
        if isinstance(block, dict) and metric_name in block:
            vals.append(float(block[metric_name]))
    if not vals:
        return float("nan")
    return float(sum(vals) / len(vals))


def arg_mean(summaries: List[Dict[str, Any]], arg_name: str) -> float:
    vals: List[float] = []
    for summary in summaries:
        args = summary.get("args", {})
        if isinstance(args, dict) and arg_name in args:
            vals.append(float(args[arg_name]))
    if not vals:
        return float("nan")
    return float(sum(vals) / len(vals))


def summarize_route(route_id: str, summaries: List[Dict[str, Any]]) -> Dict[str, Any]:
    return {
        "route_id": route_id,
        "n_runs": len(summaries),
        "mean_test_mse_after": metric_mean(summaries, "test_mse_after"),
        "mean_test_top1_after": metric_mean(summaries, "test_top1_after"),
        "mean_query_repeats": arg_mean(summaries, "query_repeats"),
        "mean_total_sec": metric_mean(summaries, "total", scope="timings_sec"),
        "mean_offline_total_sec": metric_mean(summaries, "offline_total", scope="timings_sec"),
        "mean_online_total_sec": metric_mean(summaries, "online_total", scope="timings_sec"),
        "mean_online_total_per_repeat_sec": metric_mean(summaries, "retrieval_online_total_per_repeat_sec"),
        "mean_amortized_total_per_repeat_sec": metric_mean(summaries, "retrieval_total_amortized_per_repeat_sec"),
        "mean_route_index_build_sec": metric_mean(summaries, "route_index_build", scope="timings_sec"),
        "mean_query_route_sec": metric_mean(summaries, "query_route", scope="timings_sec"),
        "mean_retrieval_search_sec": metric_mean(summaries, "retrieval_search", scope="timings_sec"),
        "mean_buckets": metric_mean(summaries, "buckets"),
        "mean_eval_shells": metric_mean(summaries, "eval_shells"),
        "mean_eval_sectors": metric_mean(summaries, "eval_sectors"),
        "mean_pmax_after": metric_mean(summaries, "pmax_after"),
        "mean_shell_pmax": metric_mean(summaries, "shell_pmax"),
        "mean_sector_pmax": metric_mean(summaries, "sector_pmax"),
        "mean_entropy_after": metric_mean(summaries, "entropy_after"),
        "mean_shell_entropy": metric_mean(summaries, "shell_entropy"),
        "mean_sector_entropy": metric_mean(summaries, "sector_entropy"),
        "mean_unseen_rate": metric_mean(summaries, "test_unseen_rate"),
        "mean_adaptive_chi": metric_mean(summaries, "adaptive_chi_mean"),
        "mean_adaptive_chi_entropy": metric_mean(summaries, "adaptive_chi_entropy"),
        "mean_adaptive_r_alpha": metric_mean(summaries, "adaptive_r_alpha_mean"),
        "mean_adaptive_hopf_shell_capacity": metric_mean(summaries, "adaptive_hopf_shell_capacity_mean"),
        "mean_adaptive_hopf_k1": metric_mean(summaries, "adaptive_hopf_k1_mean"),
        "mean_adaptive_hopf_k2": metric_mean(summaries, "adaptive_hopf_k2_mean"),
        "mean_adaptive_hopf_k1_gap": metric_mean(summaries, "adaptive_hopf_k1_gap_mean"),
        "mean_adaptive_hopf_k2_gap": metric_mean(summaries, "adaptive_hopf_k2_gap_mean"),
        "mean_adaptive_chi_bins_used": metric_mean(summaries, "adaptive_chi_bins_used"),
        "mean_adaptive_chi_bin_pmax": metric_mean(summaries, "adaptive_chi_bin_pmax"),
        "mean_adaptive_chi_bin_entropy": metric_mean(summaries, "adaptive_chi_bin_entropy"),
        "mean_poincare_alignment_pairs_used": metric_mean(summaries, "poincare_alignment_pairs_used"),
        "mean_poincare_alignment_radial_mae": metric_mean(summaries, "poincare_alignment_radial_mae"),
        "mean_poincare_alignment_radial_rel_mean": metric_mean(summaries, "poincare_alignment_radial_rel_mean"),
        "mean_poincare_alignment_radial_corr": metric_mean(summaries, "poincare_alignment_radial_corr"),
        "mean_poincare_alignment_pair_mae": metric_mean(summaries, "poincare_alignment_pair_mae"),
        "mean_poincare_alignment_pair_rel_mean": metric_mean(summaries, "poincare_alignment_pair_rel_mean"),
        "mean_poincare_alignment_pair_corr": metric_mean(summaries, "poincare_alignment_pair_corr"),
        "mean_shell_mass_error_l1": metric_mean(summaries, "shell_mass_error_l1"),
        "mean_shell_mass_error_max": metric_mean(summaries, "shell_mass_error_max"),
        "mean_shell_mass_kl": metric_mean(summaries, "shell_mass_kl"),
        "mean_shell_mass_corr": metric_mean(summaries, "shell_mass_corr"),
        "mean_hopf_angular_mass_error": metric_mean(summaries, "hopf_angular_mass_error"),
        "mean_hopf_base_mass_error": metric_mean(summaries, "hopf_base_mass_error"),
        "mean_hopf_chi_mass_error": metric_mean(summaries, "hopf_chi_mass_error"),
        "mean_hopf_delta_mass_error": metric_mean(summaries, "hopf_delta_mass_error"),
        "mean_hopf_theta1_mass_error": metric_mean(summaries, "hopf_theta1_mass_error"),
        "mean_hopf_theta2_mass_error": metric_mean(summaries, "hopf_theta2_mass_error"),
        "mean_hopf_alpha_entropy": metric_mean(summaries, "hopf_alpha_entropy"),
        "mean_hopf_sector_groups_used": metric_mean(summaries, "hopf_sector_groups_used"),
        "mean_hopf_sector_chi_std_mean": metric_mean(summaries, "hopf_sector_chi_std_mean"),
        "mean_hopf_sector_delta_cvar_mean": metric_mean(summaries, "hopf_sector_delta_cvar_mean"),
        "mean_hopf_sector_alpha_entropy_mean": metric_mean(summaries, "hopf_sector_alpha_entropy_mean"),
        "mean_hopf_sector_alpha_entropy_gap": metric_mean(summaries, "hopf_sector_alpha_entropy_gap"),
        "mean_phase_transport_coherence": metric_mean(summaries, "phase_transport_coherence"),
        "mean_phase_transport_shift_abs_mean": metric_mean(summaries, "phase_transport_shift_abs_mean"),
        "mean_phase_transport_shift_abs_max": metric_mean(summaries, "phase_transport_shift_abs_max"),
        "mean_phase_transport_connection_abs_mean": metric_mean(summaries, "phase_transport_connection_abs_mean"),
        "mean_phase_transport_field_shift_abs_mean": metric_mean(summaries, "phase_transport_field_shift_abs_mean"),
        "mean_phase_transport_field_weight_abs_mean": metric_mean(summaries, "phase_transport_field_weight_abs_mean"),
        "mean_phase_transport_alpha_bins": metric_mean(summaries, "phase_transport_alpha_bins"),
        "mean_event_gate_error_mean": metric_mean(summaries, "event_gate_error_mean"),
        "mean_event_gate_mean": metric_mean(summaries, "event_gate_mean"),
        "mean_event_gate_active_frac": metric_mean(summaries, "event_gate_active_frac"),
        "mean_event_gate_cost_proxy": metric_mean(summaries, "event_gate_cost_proxy"),
        "mean_route_entropy_radius_corr": metric_mean(summaries, "route_entropy_radius_corr"),
        "mean_route_entropy_radius_slope": metric_mean(summaries, "route_entropy_radius_slope"),
        "mean_geodesic_knn_overlap": metric_mean(summaries, "geodesic_knn_overlap_mean"),
        "mean_geodesic_knn_jaccard": metric_mean(summaries, "geodesic_knn_jaccard_mean"),
        "mean_dynamic_knn_distance": metric_mean(summaries, "dynamic_knn_distance_mean"),
        "mean_dynamic_flow_norm": metric_mean(summaries, "dynamic_flow_norm_mean"),
        "mean_dynamic_flow_ball_radius": metric_mean(summaries, "dynamic_flow_ball_radius_mean"),
        "mean_dynamic_step_dist": metric_mean(summaries, "dynamic_step_dist_mean"),
        "mean_dynamic_random_pair_dist": metric_mean(summaries, "dynamic_random_pair_dist_mean"),
        "mean_dynamic_step_to_random_ratio": metric_mean(summaries, "dynamic_step_to_random_ratio"),
        "mean_retrieval_candidate_count": metric_mean(summaries, "retrieval_candidate_count_mean"),
        "mean_retrieval_candidate_fraction": metric_mean(summaries, "retrieval_candidate_fraction_mean"),
        "mean_retrieval_probe_bucket": metric_mean(summaries, "retrieval_probe_bucket_mean"),
        "mean_retrieval_bucket_fallback_rate": metric_mean(summaries, "retrieval_bucket_fallback_rate"),
        "mean_retrieval_backfill_trigger_rate": metric_mean(summaries, "retrieval_backfill_trigger_rate"),
        "mean_retrieval_backfill_extra_candidates": metric_mean(summaries, "retrieval_backfill_extra_candidates_mean"),
        "mean_retrieval_secondary_key_count": metric_mean(summaries, "retrieval_secondary_key_count"),
        "mean_retrieval_chart_cache_hit": metric_mean(summaries, "retrieval_chart_cache_hit"),
        "mean_retrieval_route_cache_hit": metric_mean(summaries, "retrieval_route_cache_hit"),
    }


def _metric_value(summary: Dict[str, Any], metric_name: str, scope: str = "metrics") -> float:
    block = summary.get(scope, {})
    if not isinstance(block, dict):
        return float("nan")
    value = block.get(metric_name)
    if value is None:
        return float("nan")
    return float(value)


def _run_seed(summary: Dict[str, Any]) -> str:
    args = summary.get("args", {})
    if isinstance(args, dict) and "seed" in args:
        return str(args["seed"])
    return "?"


def _seed_health_reasons(summary: Dict[str, Any], health_gate: Dict[str, Any]) -> List[str]:
    reasons: List[str] = []
    min_buckets = health_gate.get("min_buckets")
    min_eval_shells = health_gate.get("min_eval_shells")
    max_pmax_after = health_gate.get("max_pmax_after")
    max_shell_pmax = health_gate.get("max_shell_pmax")
    max_unseen_rate = health_gate.get("max_unseen_rate")
    seed = _run_seed(summary)

    mean_checks = [
        ("buckets", min_buckets, "<", "metrics"),
        ("eval_shells", min_eval_shells, "<", "metrics"),
        ("pmax_after", max_pmax_after, ">", "metrics"),
        ("shell_pmax", max_shell_pmax, ">", "metrics"),
        ("test_unseen_rate", max_unseen_rate, ">", "metrics"),
    ]
    for metric_name, threshold, op, scope in mean_checks:
        if threshold is None:
            continue
        value = _metric_value(summary, metric_name, scope=scope)
        if op == "<" and value < float(threshold):
            reasons.append(f"seed{seed}_{metric_name}<{float(threshold):.3f}")
        if op == ">" and value > float(threshold):
            reasons.append(f"seed{seed}_{metric_name}>{float(threshold):.3f}")
    return reasons


def apply_health_gate(
    route_stats: List[Dict[str, Any]],
    results: Dict[str, List[Dict[str, Any]]],
    health_gate: Dict[str, Any],
    baseline_route_id: str = "R0",
) -> None:
    by_route = {row["route_id"]: row for row in route_stats}
    baseline = by_route.get(baseline_route_id)
    min_buckets = health_gate.get("min_buckets")
    min_eval_shells = health_gate.get("min_eval_shells")
    max_pmax_after = health_gate.get("max_pmax_after")
    max_shell_pmax = health_gate.get("max_shell_pmax")
    max_unseen_rate = health_gate.get("max_unseen_rate")
    max_mse_ratio_vs_r0 = health_gate.get("max_mse_ratio_vs_r0")
    max_runtime_ratio_vs_r0 = health_gate.get("max_runtime_ratio_vs_r0")
    enforce_seed_health = bool(health_gate.get("enforce_seed_health", False))

    for stats in route_stats:
        reasons: List[str] = []
        passes = True

        if min_buckets is not None and float(stats["mean_buckets"]) < float(min_buckets):
            passes = False
            reasons.append(f"mean_buckets<{float(min_buckets):.3f}")
        if min_eval_shells is not None and float(stats["mean_eval_shells"]) < float(min_eval_shells):
            passes = False
            reasons.append(f"mean_eval_shells<{float(min_eval_shells):.3f}")
        if max_pmax_after is not None and float(stats["mean_pmax_after"]) > float(max_pmax_after):
            passes = False
            reasons.append(f"mean_pmax_after>{float(max_pmax_after):.3f}")
        if max_shell_pmax is not None and float(stats["mean_shell_pmax"]) > float(max_shell_pmax):
            passes = False
            reasons.append(f"mean_shell_pmax>{float(max_shell_pmax):.3f}")
        if max_unseen_rate is not None and float(stats["mean_unseen_rate"]) > float(max_unseen_rate):
            passes = False
            reasons.append(f"mean_unseen_rate>{float(max_unseen_rate):.3f}")
        if baseline is not None and stats["route_id"] != baseline_route_id:
            if max_mse_ratio_vs_r0 is not None:
                ratio = float(stats["mean_test_mse_after"]) / max(float(baseline["mean_test_mse_after"]), 1e-12)
                stats["mse_ratio_vs_r0"] = ratio
                if ratio > float(max_mse_ratio_vs_r0):
                    passes = False
                    reasons.append(f"mse_ratio_vs_{baseline_route_id}>{float(max_mse_ratio_vs_r0):.3f}")
            if max_runtime_ratio_vs_r0 is not None:
                ratio = float(stats["mean_total_sec"]) / max(float(baseline["mean_total_sec"]), 1e-12)
                stats["runtime_ratio_vs_r0"] = ratio
                if ratio > float(max_runtime_ratio_vs_r0):
                    passes = False
                    reasons.append(f"runtime_ratio_vs_{baseline_route_id}>{float(max_runtime_ratio_vs_r0):.3f}")

        seed_reasons: List[str] = []
        if enforce_seed_health:
            for summary in results.get(stats["route_id"], []):
                seed_reasons.extend(_seed_health_reasons(summary, health_gate))
            if seed_reasons:
                passes = False
                reasons.extend(seed_reasons)

        stats["passes_health_gate"] = passes
        stats["health_gate_reasons"] = reasons
        if seed_reasons:
            stats["seed_health_reasons"] = seed_reasons


def write_gate_note(path: str, config_path: str, route_stats: List[Dict[str, Any]], recommendation: str):
    ts = datetime.datetime.now().isoformat(timespec="seconds")
    lines = [
        "# Gate Note",
        "",
        f"- Timestamp: {ts}",
        f"- Config: `{config_path}`",
        "",
        "## Route Stats",
    ]
    for stats in route_stats:
        gate_suffix = ""
        if "passes_health_gate" in stats:
            gate_suffix = f", health_pass={stats['passes_health_gate']}"
            reasons = stats.get("health_gate_reasons", [])
            if reasons:
                gate_suffix += f", health_notes={'; '.join(reasons)}"
        lines.append(
            "- "
            f"{stats['route_id']}: "
            f"mse={stats['mean_test_mse_after']:.6f}, "
            f"top1={stats['mean_test_top1_after']:.6f}, "
            f"total_sec={stats['mean_total_sec']:.3f}, "
            f"offline_sec={stats['mean_offline_total_sec']:.3f}, "
            f"online_sec={stats['mean_online_total_sec']:.3f}, "
            f"q_repeats={stats['mean_query_repeats']:.1f}, "
            f"online_per_repeat_sec={stats['mean_online_total_per_repeat_sec']:.3f}, "
            f"amortized_per_repeat_sec={stats['mean_amortized_total_per_repeat_sec']:.3f}, "
            f"buckets={stats['mean_buckets']:.3f}, "
            f"shells={stats['mean_eval_shells']:.3f}, "
            f"sectors={stats['mean_eval_sectors']:.3f}, "
            f"pmax={stats['mean_pmax_after']:.3f}, "
            f"shell_pmax={stats['mean_shell_pmax']:.3f}, "
            f"sector_pmax={stats['mean_sector_pmax']:.3f}, "
            f"entropy={stats['mean_entropy_after']:.3f}, "
            f"unseen={stats['mean_unseen_rate']:.6f}, "
            f"cand_mean={stats['mean_retrieval_candidate_count']:.3f}, "
            f"cand_frac={stats['mean_retrieval_candidate_fraction']:.6f}, "
            f"probe_mean={stats['mean_retrieval_probe_bucket']:.3f}, "
            f"fallback={stats['mean_retrieval_bucket_fallback_rate']:.6f}, "
            f"secondary_keys={stats['mean_retrieval_secondary_key_count']:.3f}, "
            f"align_radial_mae={stats['mean_poincare_alignment_radial_mae']:.6f}, "
            f"align_pair_mae={stats['mean_poincare_alignment_pair_mae']:.6f}, "
            f"align_pair_corr={stats['mean_poincare_alignment_pair_corr']:.6f}, "
            f"shell_mass_l1={stats['mean_shell_mass_error_l1']:.6f}, "
            f"shell_mass_corr={stats['mean_shell_mass_corr']:.6f}, "
            f"hopf_mass={stats['mean_hopf_angular_mass_error']:.6f}, "
            f"hopf_base_mass={stats['mean_hopf_base_mass_error']:.6f}, "
            f"delta_mass={stats['mean_hopf_delta_mass_error']:.6f}, "
            f"alpha_H={stats['mean_hopf_alpha_entropy']:.6f}, "
            f"hopf_sector_chi_std={stats['mean_hopf_sector_chi_std_mean']:.6f}, "
            f"hopf_sector_alpha_gap={stats['mean_hopf_sector_alpha_entropy_gap']:.6f}, "
            f"phase_coh={stats['mean_phase_transport_coherence']:.6f}, "
            f"phase_shift={stats['mean_phase_transport_shift_abs_mean']:.6f}, "
            f"phase_conn={stats['mean_phase_transport_connection_abs_mean']:.6f}, "
            f"phase_field_shift={stats['mean_phase_transport_field_shift_abs_mean']:.6f}, "
            f"phase_field_weight={stats['mean_phase_transport_field_weight_abs_mean']:.6f}, "
            f"route_H_r_corr={stats['mean_route_entropy_radius_corr']:.6f}, "
            f"knn_overlap={stats['mean_geodesic_knn_overlap']:.6f}, "
            f"dyn_knn={stats['mean_dynamic_knn_distance']:.6f}, "
            f"dyn_flow={stats['mean_dynamic_flow_norm']:.6f}, "
            f"dyn_step={stats['mean_dynamic_step_dist']:.6f}, "
            f"dyn_step_ratio={stats['mean_dynamic_step_to_random_ratio']:.6f}"
            f"{gate_suffix}"
        )
    lines.extend([
        "",
        "## Recommendation",
        f"- {recommendation}",
        "",
        "## Required User Input",
        "- None unless recommendation requests branch redirection.",
    ])
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def choose_recommendation(
    route_stats: List[Dict[str, Any]],
    health_gate: Optional[Dict[str, Any]] = None,
    baseline_route_id: str = "R0",
) -> str:
    def is_retrieval_batch(row: Dict[str, Any]) -> bool:
        return all(
            key in row
            for key in (
                "mean_test_top1_after",
                "mean_retrieval_candidate_fraction",
                "mean_online_total_per_repeat_sec",
                "mean_amortized_total_per_repeat_sec",
            )
        )

    by_route = {row["route_id"]: row for row in route_stats}
    baseline = by_route.get(baseline_route_id)
    best = min(route_stats, key=lambda row: row["mean_test_mse_after"])
    passing: List[Dict[str, Any]] = []
    if baseline is None:
        return f"Track {best['route_id']} as provisional best; baseline {baseline_route_id} missing from batch."

    if health_gate:
        passing = [row for row in route_stats if row["route_id"] != baseline_route_id and row.get("passes_health_gate", False)]
    if is_retrieval_batch(baseline):
        retrieval_pool = passing if passing else [
            row for row in route_stats if row["route_id"] != baseline_route_id
        ]
        if retrieval_pool:
            baseline_cand_frac = float(baseline["mean_retrieval_candidate_fraction"])
            baseline_online = float(baseline["mean_online_total_per_repeat_sec"])
            baseline_amortized = float(baseline["mean_amortized_total_per_repeat_sec"])
            retrieval_systems = [
                row for row in retrieval_pool
                if float(row.get("mean_retrieval_candidate_fraction", float("inf"))) < baseline_cand_frac
                and float(row.get("mean_online_total_per_repeat_sec", float("inf"))) < baseline_online
                and float(row.get("mean_amortized_total_per_repeat_sec", float("inf"))) < baseline_amortized
            ]
            if retrieval_systems:
                best_retrieval = min(
                    retrieval_systems,
                    key=lambda row: (
                        float(row["mean_amortized_total_per_repeat_sec"]),
                        float(row["mean_online_total_per_repeat_sec"]),
                        float(row["mean_retrieval_candidate_fraction"]),
                        -float(row["mean_test_top1_after"]),
                    ),
                )
                if float(best_retrieval["mean_test_top1_after"]) >= float(baseline["mean_test_top1_after"]):
                    return (
                        f"Promote {best_retrieval['route_id']} as translated systems lead; "
                        f"it cuts candidate fraction plus online/amortized runtime vs {baseline_route_id} "
                        "while also improving top-1."
                    )
                return (
                    f"Track {best_retrieval['route_id']} as translated systems lead; "
                    f"it cuts candidate fraction plus online/amortized runtime vs {baseline_route_id}, "
                    "but top-1 regressed and remains the next retrieval-quality recovery target."
                )

            best_retrieval_quality = max(
                retrieval_pool,
                key=lambda row: (
                    float(row.get("mean_test_top1_after", float("-inf"))),
                    -float(row.get("mean_retrieval_candidate_fraction", float("inf"))),
                    -float(row.get("mean_amortized_total_per_repeat_sec", float("inf"))),
                ),
            )
            if passing:
                return (
                    f"Track {best_retrieval_quality['route_id']} as the healthiest translated quality branch; "
                    f"it passes the configured route-health gate, but none of the healthy routes beat {baseline_route_id} "
                    "on candidate pruning plus online/amortized runtime."
                )
            return (
                f"Track {best_retrieval_quality['route_id']} as the translated quality branch; "
                f"none of the alternates beat {baseline_route_id} on candidate pruning plus online/amortized runtime."
            )

    if health_gate and passing:
        faster_passing = [
            row for row in passing
            if float(row.get("runtime_ratio_vs_r0", float("inf"))) < 1.0
        ]
        if faster_passing:
            best_hardware = min(
                faster_passing,
                key=lambda row: (row["mean_total_sec"], row["mean_test_mse_after"]),
            )
            if float(best_hardware["mean_test_mse_after"]) <= float(baseline["mean_test_mse_after"]):
                return (
                    f"Promote {best_hardware['route_id']} as stabilized transfer candidate; "
                    f"it passes the configured route-health gate and beats {baseline_route_id} on both quality and runtime."
                )
            return (
                f"Promote {best_hardware['route_id']} as hardware-efficiency transfer lead; "
                "it passes the configured route-health gate, reduces runtime materially, "
                f"and stays within the allowed quality tolerance vs {baseline_route_id}."
            )

        best_passing = min(passing, key=lambda row: row["mean_test_mse_after"])
        return (
            f"Track {best_passing['route_id']} as the healthiest quality branch; "
            f"it passes the configured route-health gate, but none of the healthy routes reduced runtime vs {baseline_route_id}."
        )

    if best["route_id"] == baseline_route_id:
        return f"Keep {baseline_route_id} as transfer baseline; no alternate route beat it on mean proxy MSE."

    better_runtime = best["mean_total_sec"] <= baseline["mean_total_sec"]
    unhealthy_concentration = (
        best["mean_pmax_after"] >= 0.70
        or best["mean_buckets"] <= max(2.0, baseline["mean_buckets"] * 0.50)
        or best["mean_eval_shells"] <= 1.0
        or best["mean_shell_pmax"] >= 0.95
    )

    if better_runtime and unhealthy_concentration:
        return (
            f"Hold promotion of {best['route_id']}: it beats {baseline_route_id} on mean proxy MSE and runtime, "
            "but route health remains poor due to concentration/collapse. Prioritize stabilization research."
        )
    if better_runtime:
        return f"Promote {best['route_id']} as proxy-transfer lead candidate; it beats {baseline_route_id} on mean MSE and runtime without a collapse flag."
    return f"Track {best['route_id']} as a quality candidate, but keep {baseline_route_id} operationally preferred because runtime regressed."


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", type=str, required=True)
    ap.add_argument("--log_dir", type=str, default="results/raw")
    ap.add_argument("--analysis_dir", type=str, default="results/analysis")
    ap.add_argument("--gate_dir", type=str, default="docs/governance/gates")
    ap.add_argument("--parsed_dir", type=str, default="results/parsed")
    ap.add_argument("--summary_csv", type=str, default="results/summary.csv")
    ap.add_argument("--python_bin", type=str, default=sys.executable)
    args = ap.parse_args()

    os.makedirs(args.log_dir, exist_ok=True)
    os.makedirs(args.analysis_dir, exist_ok=True)
    os.makedirs(args.gate_dir, exist_ok=True)

    cfg = load_config(args.config)
    experiment_id = cfg.get("experiment_id", "proxy_experiment")
    common_args = cfg.get("common_args", {})
    routes = cfg.get("routes", [])
    seeds = cfg.get("seeds", [])
    run_order = str(cfg.get("run_order", "route_major"))
    health_gate = cfg.get("health_gate", {})
    baseline_route_id = str(cfg.get("baseline_route_id", "R0"))
    task_script = str(cfg.get("task_script", "tasks/router_proxy_eval.py"))
    if not routes or not seeds:
        raise SystemExit("config must contain non-empty routes and seeds")
    if run_order not in {"route_major", "seed_major"}:
        raise SystemExit("run_order must be 'route_major' or 'seed_major'")

    results: Dict[str, List[Dict[str, Any]]] = {}
    total_runs = 0
    for route in routes:
        results[route["route_id"]] = []

    for route, seed in build_run_plan(routes, seeds, run_order):
        route_id = route["route_id"]
        route_args = route.get("args", {})
        ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        run_tag = f"{experiment_id}_{route_id}_seed{seed}_{ts}"
        log_name = f"{run_tag}.log"
        log_path = os.path.join(args.log_dir, log_name)
        cmd = [args.python_bin, task_script] + build_arg_items(common_args, route_args, seed, run_tag)
        rc = run_one(cmd, log_path)
        summary = parse_summary_from_log(log_path)
        if summary is None:
            summary = {
                "parsed": False,
                "log_file": log_name,
                "metrics": {},
                "timings_sec": {},
                "return_code": rc,
                "route_id": route_id,
            }
        summary["log_file"] = log_name
        summary["return_code"] = rc
        summary["route_id"] = route_id
        results[route_id].append(summary)
        total_runs += 1

    route_stats = [summarize_route(route_id, summaries) for route_id, summaries in results.items()]
    route_stats.sort(key=lambda row: row["mean_test_mse_after"])
    if health_gate:
        apply_health_gate(route_stats, results, health_gate, baseline_route_id=baseline_route_id)
    recommendation = choose_recommendation(route_stats, health_gate=health_gate, baseline_route_id=baseline_route_id)

    batch = {
        "experiment_id": experiment_id,
        "config": args.config,
        "generated_at": datetime.datetime.now().isoformat(timespec="seconds"),
        "health_gate": health_gate,
        "baseline_route_id": baseline_route_id,
        "task_script": task_script,
        "run_order": run_order,
        "route_stats": route_stats,
        "recommendation": recommendation,
        "total_runs": total_runs,
        "results": results,
    }

    analysis_path = os.path.join(args.analysis_dir, f"{experiment_id}.json")
    with open(analysis_path, "w", encoding="utf-8") as f:
        json.dump(batch, f, indent=2, sort_keys=True)

    gate_ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
    gate_path = os.path.join(args.gate_dir, f"gate_{gate_ts}.md")
    write_gate_note(gate_path, args.config, route_stats, recommendation)

    subprocess.run([args.python_bin, "tools/parse_logs.py", args.log_dir, args.parsed_dir], check=True)
    subprocess.run([args.python_bin, "tools/summarize.py", args.parsed_dir, args.summary_csv], check=True)

    print(
        "Proxy sweep complete. "
        f"runs={total_runs} analysis={analysis_path} gate_note={gate_path}"
    )


if __name__ == "__main__":
    main()
