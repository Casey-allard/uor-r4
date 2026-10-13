#!/usr/bin/env python3
import argparse
import datetime
import json
import math
import os
import subprocess
import sys
from typing import Any, Dict, Iterable, List


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def load_config(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def route_arg_map(proxy_config: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    mapping: Dict[str, Dict[str, Any]] = {}
    common_args = dict(proxy_config.get("common_args", {}))
    for route in proxy_config.get("routes", []):
        route_id = str(route["route_id"])
        merged = dict(common_args)
        merged.update(route.get("args", {}))
        mapping[route_id] = merged
    return mapping


def route_ids_from_config(cfg: Dict[str, Any], proxy_config: Dict[str, Any]) -> List[str]:
    available = [str(route["route_id"]) for route in proxy_config.get("routes", [])]
    raw = cfg.get("route_ids", [])
    if not raw:
        return available
    return [str(route_id) for route_id in raw]


def build_seed_command(
    python_bin: str,
    proxy_config: str,
    seed: int,
    route_ids: Iterable[str],
    max_points: int,
    knn_k: int,
    lowfreq_modes: int,
    output_path: str,
) -> List[str]:
    return [
        python_bin,
        os.path.join(ROOT, "tools", "spectral_route_audit.py"),
        "--config",
        proxy_config,
        "--seed",
        str(seed),
        "--routes",
        ",".join(route_ids),
        "--max-points",
        str(max_points),
        "--knn-k",
        str(knn_k),
        "--lowfreq-modes",
        str(lowfreq_modes),
        "--output",
        output_path,
    ]


def run_seed(cmd: List[str], log_path: str) -> None:
    with open(log_path, "w", encoding="utf-8") as log_file:
        subprocess.run(cmd, stdout=log_file, stderr=subprocess.STDOUT, check=True)


def _numeric_metric_names(seed_payloads: List[Dict[str, Any]]) -> List[str]:
    names = set()
    for payload in seed_payloads:
        for row in payload.get("results", []):
            metrics = row.get("metrics", {})
            for name, value in metrics.items():
                if isinstance(value, (int, float)) and not isinstance(value, bool):
                    names.add(str(name))
    return sorted(names)


def _mean(values: List[float]) -> float:
    return float(sum(values) / len(values)) if values else float("nan")


def _std(values: List[float], mean: float) -> float:
    if not values:
        return float("nan")
    return float(math.sqrt(sum((value - mean) ** 2 for value in values) / len(values)))


def aggregate_route_stats(
    route_id: str,
    route_args: Dict[str, Any],
    seed_payloads: List[Dict[str, Any]],
) -> Dict[str, Any]:
    metric_names = _numeric_metric_names(seed_payloads)
    per_seed: List[Dict[str, Any]] = []
    for payload in seed_payloads:
        seed = int(payload["seed"])
        result = next(row for row in payload.get("results", []) if row["route_id"] == route_id)
        metrics = result.get("metrics", {})
        per_seed.append({"seed": seed, "metrics": metrics})

    stats: Dict[str, Any] = {
        "route_id": route_id,
        "n_seeds": len(per_seed),
        "args": {
            "sector_mode": route_args.get("sector_mode", ""),
            "phase4_dims": route_args.get("phase4_dims", ""),
            "field4_dims": route_args.get("field4_dims", ""),
            "phase_transport_lambda": float(route_args.get("phase_transport_lambda", 0.0)),
            "phase_field_lambda": float(route_args.get("phase_field_lambda", 0.0)),
        },
        "per_seed": per_seed,
    }
    for metric_name in metric_names:
        values = [
            float(seed_entry["metrics"][metric_name])
            for seed_entry in per_seed
            if metric_name in seed_entry["metrics"]
        ]
        mean = _mean(values)
        std = _std(values, mean)
        stats[f"mean_{metric_name}"] = mean
        stats[f"std_{metric_name}"] = std
        stats[f"min_{metric_name}"] = float(min(values)) if values else float("nan")
        stats[f"max_{metric_name}"] = float(max(values)) if values else float("nan")
    return stats


def classify_product_routes(route_stats: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    out = []
    for row in route_stats:
        if row.get("args", {}).get("sector_mode") == "phase4d_hopf_product_phase":
            out.append(row)
    return out


def classify_control_routes(route_stats: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    out = []
    for row in route_stats:
        if row.get("args", {}).get("sector_mode") != "phase4d_hopf_product_phase":
            out.append(row)
    return out


def summarize_group(route_stats: List[Dict[str, Any]]) -> Dict[str, Any]:
    if not route_stats:
        return {
            "route_ids": [],
            "mean_spectral_lambda2": float("nan"),
            "mean_spectral_mode_participation_mean": float("nan"),
            "mean_sector_lowfreq_energy": float("nan"),
            "mean_shell_lowfreq_energy": float("nan"),
        }
    return {
        "route_ids": [row["route_id"] for row in route_stats],
        "mean_spectral_lambda2": _mean([float(row["mean_spectral_lambda2"]) for row in route_stats]),
        "mean_spectral_mode_participation_mean": _mean(
            [float(row["mean_spectral_mode_participation_mean"]) for row in route_stats]
        ),
        "mean_sector_lowfreq_energy": _mean([float(row["mean_sector_lowfreq_energy"]) for row in route_stats]),
        "mean_shell_lowfreq_energy": _mean([float(row["mean_shell_lowfreq_energy"]) for row in route_stats]),
    }


def evaluate_acceptance(route_stats: List[Dict[str, Any]]) -> Dict[str, Any]:
    product_routes = classify_product_routes(route_stats)
    control_routes = classify_control_routes(route_stats)
    all_connected = all(int(row["mean_spectral_zero_eigs"]) == 1 for row in route_stats)
    product_participation = summarize_group(product_routes)["mean_spectral_mode_participation_mean"]
    control_participation = summarize_group(control_routes)["mean_spectral_mode_participation_mean"]
    product_sector = summarize_group(product_routes)["mean_sector_lowfreq_energy"]
    control_sector = summarize_group(control_routes)["mean_sector_lowfreq_energy"]
    participation_gap = float(product_participation - control_participation)
    sector_gap = float(control_sector - product_sector)
    distinct_signature = participation_gap > 0.005 and sector_gap > 0.01
    return {
        "all_graphs_connected": all_connected,
        "participation_gap_product_minus_control": participation_gap,
        "sector_lowfreq_gap_control_minus_product": sector_gap,
        "distinct_signature": distinct_signature,
    }


def choose_recommendation(route_stats: List[Dict[str, Any]], acceptance: Dict[str, Any], stage: str) -> str:
    if not acceptance["all_graphs_connected"]:
        return "Hold the spectral branch: at least one audited route graph is disconnected under the current operator."
    if acceptance["distinct_signature"]:
        return (
            f"Close the INC-0066 {stage} positive and continue spectral measurement: "
            "the confirmed product routes show a distinct low-mode signature "
            "relative to the control set while staying connected."
        )
    best_sector = max(route_stats, key=lambda row: float(row["mean_sector_lowfreq_energy"]))
    best_participation = max(route_stats, key=lambda row: float(row["mean_spectral_mode_participation_mean"]))
    return (
        "Treat the INC-0066 screen as connected but inconclusive: "
        f"{best_participation['route_id']} leads low-mode delocalization while "
        f"{best_sector['route_id']} leads sector low-frequency concentration."
    )


def write_gate_note(path: str, config_path: str, route_stats: List[Dict[str, Any]], acceptance: Dict[str, Any], recommendation: str) -> None:
    ts = datetime.datetime.now().isoformat(timespec="seconds")
    lines = [
        "# Gate Note",
        "",
        f"- Timestamp: {ts}",
        f"- Config: `{config_path}`",
        "",
        "## Spectral Route Stats",
    ]
    for row in route_stats:
        lines.append(
            "- "
            f"{row['route_id']}: "
            f"lambda2={row['mean_spectral_lambda2']:.6f}, "
            f"lambda3={row['mean_spectral_lambda3']:.6f}, "
            f"lambda4={row['mean_spectral_lambda4']:.6f}, "
            f"part_mean={row['mean_spectral_mode_participation_mean']:.6f}, "
            f"part_std={row['std_spectral_mode_participation_mean']:.6f}, "
            f"lowfreq_mass={row['mean_spectral_lowfreq_mass_ratio']:.6f}, "
            f"shell_lowfreq={row['mean_shell_lowfreq_energy']:.6f}, "
            f"sector_lowfreq={row['mean_sector_lowfreq_energy']:.6f}, "
            f"zero_eigs={row['mean_spectral_zero_eigs']:.1f}, "
            f"edges={row['mean_graph_edges']:.1f}, "
            f"sigma={row['mean_graph_sigma']:.6f}"
        )
    lines.extend(
        [
            "",
            "## Acceptance Read",
            f"- all_graphs_connected={acceptance['all_graphs_connected']}",
            f"- participation_gap_product_minus_control={acceptance['participation_gap_product_minus_control']:.6f}",
            f"- sector_lowfreq_gap_control_minus_product={acceptance['sector_lowfreq_gap_control_minus_product']:.6f}",
            f"- distinct_signature={acceptance['distinct_signature']}",
            "",
            "## Recommendation",
            f"- {recommendation}",
            "",
            "## Required User Input",
            "- None unless the recommendation requests branch redirection.",
        ]
    )
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", required=True)
    ap.add_argument("--analysis_dir", default="results/analysis")
    ap.add_argument("--log_dir", default="results/raw")
    ap.add_argument("--gate_dir", default="docs/governance/gates")
    ap.add_argument("--python_bin", default=sys.executable)
    args = ap.parse_args()

    cfg = load_config(args.config)
    proxy_config_path = str(cfg["source_proxy_config"])
    proxy_config = load_config(proxy_config_path)
    experiment_id = str(cfg["experiment_id"])
    seeds = [int(seed) for seed in cfg["seeds"]]
    stage = str(cfg.get("stage", "screen"))
    max_points = int(cfg.get("max_points", 384))
    knn_k = int(cfg.get("knn_k", 12))
    lowfreq_modes = int(cfg.get("lowfreq_modes", 8))
    route_ids = route_ids_from_config(cfg, proxy_config)
    route_args = route_arg_map(proxy_config)

    os.makedirs(args.analysis_dir, exist_ok=True)
    os.makedirs(args.log_dir, exist_ok=True)
    os.makedirs(args.gate_dir, exist_ok=True)

    seed_payloads: List[Dict[str, Any]] = []
    seed_artifacts: List[Dict[str, Any]] = []
    for seed in seeds:
        output_path = os.path.join(args.analysis_dir, f"{experiment_id}_seed{seed}.json")
        log_path = os.path.join(args.log_dir, f"{experiment_id}_seed{seed}.log")
        cmd = build_seed_command(
            python_bin=args.python_bin,
            proxy_config=proxy_config_path,
            seed=seed,
            route_ids=route_ids,
            max_points=max_points,
            knn_k=knn_k,
            lowfreq_modes=lowfreq_modes,
            output_path=output_path,
        )
        run_seed(cmd, log_path)
        payload = load_config(output_path)
        seed_payloads.append(payload)
        seed_artifacts.append({"seed": seed, "analysis_path": output_path, "log_path": log_path})

    route_stats = []
    for route_id in route_ids:
        route_stats.append(aggregate_route_stats(route_id, route_args.get(route_id, {}), seed_payloads))
    route_stats.sort(key=lambda row: float(row["mean_spectral_mode_participation_mean"]), reverse=True)

    product_summary = summarize_group(classify_product_routes(route_stats))
    control_summary = summarize_group(classify_control_routes(route_stats))
    acceptance = evaluate_acceptance(route_stats)
    recommendation = choose_recommendation(route_stats, acceptance, stage=stage)

    batch = {
        "experiment_id": experiment_id,
        "config": args.config,
        "source_proxy_config": proxy_config_path,
        "generated_at": datetime.datetime.now().isoformat(timespec="seconds"),
        "stage": stage,
        "seeds": seeds,
        "route_ids": route_ids,
        "max_points": max_points,
        "knn_k": knn_k,
        "lowfreq_modes": lowfreq_modes,
        "seed_artifacts": seed_artifacts,
        "route_stats": route_stats,
        "product_summary": product_summary,
        "control_summary": control_summary,
        "acceptance": acceptance,
        "recommendation": recommendation,
    }
    analysis_path = os.path.join(args.analysis_dir, f"{experiment_id}.json")
    with open(analysis_path, "w", encoding="utf-8") as f:
        json.dump(batch, f, indent=2, sort_keys=True)

    gate_ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
    gate_path = os.path.join(args.gate_dir, f"gate_{gate_ts}.md")
    write_gate_note(gate_path, args.config, route_stats, acceptance, recommendation)
    print(
        json.dumps(
            {
                "analysis": analysis_path,
                "gate_note": gate_path,
                "recommendation": recommendation,
            },
            indent=2,
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    main()
