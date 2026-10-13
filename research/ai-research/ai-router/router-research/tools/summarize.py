#!/usr/bin/env python3
import csv
import datetime
import json
import os
import sys
from typing import Dict, List

FIELDS = [
    "timestamp",
    "parsed",
    "parse_error",
    "schema_version",
    "log_file",
    "seed",
    "mode",
    "retrieval_backend",
    "query_repeats",
    "dynamic_state_mode",
    "candidate_mode",
    "route_key_mode",
    "flow_step",
    "flow_scale",
    "flow_weight",
    "state_topk",
    "complex_key_roots",
    "complex_key_radius_bins",
    "complex_rerank_mode",
    "complex_rerank_lambda",
    "sector_mode",
    "phase_dims",
    "phase4_dims",
    "field4_dims",
    "complex_dims",
    "time_pressure_lambda",
    "scale_mode",
    "radial_bins",
    "learn_so8",
    "learn_scale",
    "fast_dev",
    "extra_budget",
    "max_slots_per_bucket",
    "chart_beta",
    "chart_iters",
    "test_mse_before",
    "test_mse_after",
    "test_top1_after",
    "train_label_sse_per",
    "test_label_sse_per",
    "test_unseen_rate",
    "buckets",
    "slots_used",
    "new_slots",
    "accepted_splits",
    "retrieval_candidate_count_mean",
    "retrieval_candidate_fraction_mean",
    "retrieval_probe_bucket_mean",
    "retrieval_bucket_fallback_rate",
    "retrieval_backfill_trigger_rate",
    "retrieval_backfill_extra_candidates_mean",
    "retrieval_secondary_key_count",
    "retrieval_query_repeats",
    "retrieval_offline_total_sec",
    "retrieval_online_total_sec",
    "retrieval_online_total_per_repeat_sec",
    "retrieval_total_amortized_per_repeat_sec",
    "poincare_alignment_pairs_used",
    "poincare_alignment_radial_mae",
    "poincare_alignment_radial_rel_mean",
    "poincare_alignment_radial_corr",
    "poincare_alignment_pair_mae",
    "poincare_alignment_pair_rel_mean",
    "poincare_alignment_pair_corr",
    "shell_mass_error_l1",
    "shell_mass_error_max",
    "shell_mass_kl",
    "shell_mass_corr",
    "shell_mass_shells_used",
    "hopf_angular_mass_error",
    "hopf_base_mass_error",
    "hopf_chi_mass_error",
    "hopf_delta_mass_error",
    "hopf_theta1_mass_error",
    "hopf_theta2_mass_error",
    "hopf_alpha_entropy",
    "phase_transport_coherence",
    "phase_transport_shift_abs_mean",
    "phase_transport_shift_abs_max",
    "phase_transport_connection_abs_mean",
    "phase_transport_field_shift_abs_mean",
    "phase_transport_field_weight_abs_mean",
    "phase_transport_alpha_bins",
    "hopf_theta1_entropy",
    "hopf_theta2_entropy",
    "route_entropy_radius_corr",
    "route_entropy_radius_slope",
    "route_entropy_shells_used",
    "geodesic_knn_overlap_k",
    "geodesic_knn_overlap_mean",
    "geodesic_knn_jaccard_mean",
    "geodesic_knn_points_used",
    "dynamic_knn_distance_mean",
    "dynamic_flow_norm_mean",
    "dynamic_flow_norm_q95",
    "dynamic_flow_ball_radius_mean",
    "dynamic_flow_ball_radius_q95",
    "dynamic_train_flow_norm_q95",
    "dynamic_eval_flow_norm_q95",
    "dynamic_step_dist_mean",
    "dynamic_step_dist_std",
    "dynamic_random_pair_dist_mean",
    "dynamic_step_to_random_ratio",
    "dataset_sec",
    "chart_opt_sec",
    "routing_eval_sec",
    "route_index_build_sec",
    "query_route_sec",
    "retrieval_search_sec",
    "offline_total_sec",
    "online_total_sec",
    "training_route_sec",
    "training_update_sec",
    "training_ema_sec",
    "growth_sec",
    "total_sec",
    "chart_cache_file",
    "route_cache_file",
    "run_tag",
    "git_branch",
    "git_commit",
    "notes",
]


def build_row(j: Dict, default_log_file: str) -> Dict:
    row = {k: "" for k in FIELDS}
    row["timestamp"] = datetime.datetime.now().isoformat(timespec="seconds")
    row["parsed"] = bool(j.get("parsed", False))
    row["parse_error"] = j.get("parse_error", "")
    row["schema_version"] = j.get("schema_version", "")
    row["log_file"] = j.get("log_file", default_log_file)

    args = j.get("args", {}) if isinstance(j.get("args", {}), dict) else {}
    metrics = j.get("metrics", {}) if isinstance(j.get("metrics", {}), dict) else {}
    timings = j.get("timings_sec", {}) if isinstance(j.get("timings_sec", {}), dict) else {}
    artifacts = j.get("artifacts", {}) if isinstance(j.get("artifacts", {}), dict) else {}
    git = j.get("git", {}) if isinstance(j.get("git", {}), dict) else {}

    for k in [
        "seed", "mode", "retrieval_backend", "query_repeats", "dynamic_state_mode", "candidate_mode", "route_key_mode", "flow_step", "flow_scale", "flow_weight",
        "state_topk", "complex_key_roots", "complex_key_radius_bins", "complex_rerank_mode", "complex_rerank_lambda",
        "sector_mode", "phase_dims", "phase4_dims", "field4_dims", "complex_dims",
        "time_pressure_lambda", "scale_mode", "radial_bins", "learn_so8", "learn_scale",
        "fast_dev", "extra_budget", "max_slots_per_bucket", "chart_beta", "chart_iters",
    ]:
        row[k] = args.get(k, "")

    for k in [
        "test_mse_before", "test_mse_after", "test_top1_after", "train_label_sse_per", "test_label_sse_per",
        "test_unseen_rate", "buckets", "slots_used", "new_slots", "accepted_splits",
        "retrieval_candidate_count_mean", "retrieval_candidate_fraction_mean", "retrieval_probe_bucket_mean",
        "retrieval_bucket_fallback_rate", "retrieval_backfill_trigger_rate",
        "retrieval_backfill_extra_candidates_mean", "retrieval_secondary_key_count",
        "retrieval_query_repeats", "retrieval_offline_total_sec", "retrieval_online_total_sec",
        "retrieval_online_total_per_repeat_sec", "retrieval_total_amortized_per_repeat_sec",
        "poincare_alignment_pairs_used",
        "poincare_alignment_radial_mae",
        "poincare_alignment_radial_rel_mean",
        "poincare_alignment_radial_corr",
        "poincare_alignment_pair_mae",
        "poincare_alignment_pair_rel_mean",
        "poincare_alignment_pair_corr",
        "shell_mass_error_l1",
        "shell_mass_error_max",
        "shell_mass_kl",
        "shell_mass_corr",
        "shell_mass_shells_used",
        "hopf_angular_mass_error",
        "hopf_base_mass_error",
        "hopf_chi_mass_error",
        "hopf_delta_mass_error",
        "hopf_theta1_mass_error",
        "hopf_theta2_mass_error",
        "hopf_alpha_entropy",
        "phase_transport_coherence",
        "phase_transport_shift_abs_mean",
        "phase_transport_shift_abs_max",
        "phase_transport_connection_abs_mean",
        "phase_transport_field_shift_abs_mean",
        "phase_transport_field_weight_abs_mean",
        "phase_transport_alpha_bins",
        "hopf_theta1_entropy",
        "hopf_theta2_entropy",
        "route_entropy_radius_corr",
        "route_entropy_radius_slope",
        "route_entropy_shells_used",
        "geodesic_knn_overlap_k",
        "geodesic_knn_overlap_mean",
        "geodesic_knn_jaccard_mean",
        "geodesic_knn_points_used",
        "dynamic_knn_distance_mean",
        "dynamic_flow_norm_mean",
        "dynamic_flow_norm_q95",
        "dynamic_flow_ball_radius_mean",
        "dynamic_flow_ball_radius_q95",
        "dynamic_train_flow_norm_q95",
        "dynamic_eval_flow_norm_q95",
        "dynamic_step_dist_mean",
        "dynamic_step_dist_std",
        "dynamic_random_pair_dist_mean",
        "dynamic_step_to_random_ratio",
    ]:
        row[k] = metrics.get(k, "")

    row["dataset_sec"] = timings.get("dataset", "")
    row["chart_opt_sec"] = timings.get("chart_opt", "")
    row["routing_eval_sec"] = timings.get("routing_eval", "")
    row["route_index_build_sec"] = timings.get("route_index_build", "")
    row["query_route_sec"] = timings.get("query_route", "")
    row["retrieval_search_sec"] = timings.get("retrieval_search", "")
    row["offline_total_sec"] = timings.get("offline_total", "")
    row["online_total_sec"] = timings.get("online_total", "")
    row["training_route_sec"] = timings.get("training_route", "")
    row["training_update_sec"] = timings.get("training_update", "")
    row["training_ema_sec"] = timings.get("training_ema", "")
    row["growth_sec"] = timings.get("growth", "")
    row["total_sec"] = timings.get("total", "")

    row["chart_cache_file"] = artifacts.get("chart_cache_file", "")
    row["route_cache_file"] = artifacts.get("route_cache_file", "")
    row["run_tag"] = artifacts.get("run_tag", "")

    row["git_branch"] = git.get("branch", "")
    row["git_commit"] = git.get("commit", "")

    notes = j.get("notes", [])
    if isinstance(notes, list):
        row["notes"] = " | ".join(str(x) for x in notes)
    else:
        row["notes"] = str(notes)

    return row


def main():
    if len(sys.argv) != 3:
        print("Usage: summarize.py <parsed_dir> <summary_csv>")
        sys.exit(2)

    parsed_dir, out_csv = sys.argv[1], sys.argv[2]
    rows: List[Dict] = []

    for fn in sorted(os.listdir(parsed_dir)):
        if not fn.endswith(".json"):
            continue
        with open(os.path.join(parsed_dir, fn), "r", encoding="utf-8") as f:
            j = json.load(f)

        default_log = fn.replace(".json", ".log")
        row = build_row(j, default_log)
        rows.append(row)

    os.makedirs(os.path.dirname(out_csv) or ".", exist_ok=True)
    with open(out_csv, "w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS)
        w.writeheader()
        for r in rows:
            w.writerow(r)

    print(f"Wrote {len(rows)} row(s) to {out_csv}")


if __name__ == "__main__":
    main()
