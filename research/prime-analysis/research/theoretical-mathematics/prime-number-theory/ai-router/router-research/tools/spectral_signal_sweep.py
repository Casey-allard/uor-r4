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
    graph_mode: str = "ambient_euclidean",
) -> List[str]:
    return [
        python_bin,
        os.path.join(ROOT, "tools", "spectral_signal_probe.py"),
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
        "--graph-mode",
        graph_mode,
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
    return [row for row in route_stats if row.get("args", {}).get("sector_mode") == "phase4d_hopf_product_phase"]


def classify_full_controls(route_stats: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    return [row for row in route_stats if row.get("args", {}).get("sector_mode") != "phase4d_hopf_product_phase"]


def classify_hopf_controls(route_stats: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    out = []
    for row in route_stats:
        mode = row.get("args", {}).get("sector_mode")
        if mode in {"phase4d_hopf_base", "phase4d_hopf", "phase4d_hopf_fib_band"}:
            out.append(row)
    return out


def summarize_group(route_stats: List[Dict[str, Any]]) -> Dict[str, Any]:
    if not route_stats:
        return {
            "route_ids": [],
            "mean_label_onehot_lowfreq_energy": float("nan"),
            "mean_label_onehot_dirichlet_energy": float("nan"),
            "mean_label_indicator_lowfreq_mean": float("nan"),
            "mean_shell_lowfreq_energy": float("nan"),
            "mean_sector_lowfreq_energy": float("nan"),
        }
    return {
        "route_ids": [row["route_id"] for row in route_stats],
        "mean_label_onehot_lowfreq_energy": _mean([float(row["mean_label_onehot_lowfreq_energy"]) for row in route_stats]),
        "mean_label_onehot_dirichlet_energy": _mean([float(row["mean_label_onehot_dirichlet_energy"]) for row in route_stats]),
        "mean_label_indicator_lowfreq_mean": _mean([float(row["mean_label_indicator_lowfreq_mean"]) for row in route_stats]),
        "mean_shell_lowfreq_energy": _mean([float(row["mean_shell_lowfreq_energy"]) for row in route_stats]),
        "mean_sector_lowfreq_energy": _mean([float(row["mean_sector_lowfreq_energy"]) for row in route_stats]),
    }


def evaluate_acceptance(route_stats: List[Dict[str, Any]]) -> Dict[str, Any]:
    product = summarize_group(classify_product_routes(route_stats))
    hopf = summarize_group(classify_hopf_controls(route_stats))
    full = summarize_group(classify_full_controls(route_stats))
    all_connected = all(int(row["mean_spectral_zero_eigs"]) == 1 for row in route_stats)
    hopf_label_gap = float(product["mean_label_onehot_lowfreq_energy"] - hopf["mean_label_onehot_lowfreq_energy"])
    hopf_dirichlet_gap = float(hopf["mean_label_onehot_dirichlet_energy"] - product["mean_label_onehot_dirichlet_energy"])
    full_label_gap = float(product["mean_label_onehot_lowfreq_energy"] - full["mean_label_onehot_lowfreq_energy"])
    full_dirichlet_gap = float(full["mean_label_onehot_dirichlet_energy"] - product["mean_label_onehot_dirichlet_energy"])
    distinct_task_signal = hopf_label_gap > 0.0 and hopf_dirichlet_gap > 0.0
    return {
        "all_graphs_connected": all_connected,
        "hopf_label_lowfreq_gap": hopf_label_gap,
        "hopf_label_dirichlet_gap": hopf_dirichlet_gap,
        "full_label_lowfreq_gap": full_label_gap,
        "full_label_dirichlet_gap": full_dirichlet_gap,
        "distinct_task_signal": distinct_task_signal,
    }


def choose_recommendation(acceptance: Dict[str, Any], stage: str) -> str:
    if not acceptance["all_graphs_connected"]:
        return "Hold the signal-probe branch: at least one audited route graph is disconnected under the fixed operator."
    if acceptance["distinct_task_signal"]:
        return (
            f"Close the INC-0067 {stage} positive: the product routes carry stronger low-mode label signal "
            "than the Hopf controls on the fixed confirmed operator set."
        )
    return (
        f"Treat the INC-0067 {stage} as connected but inconclusive: "
        "the operator remains distinct, but the task-label projection gap is not yet positive against the Hopf controls."
    )


def write_gate_note(path: str, config_path: str, route_stats: List[Dict[str, Any]], acceptance: Dict[str, Any], recommendation: str) -> None:
    ts = datetime.datetime.now().isoformat(timespec="seconds")
    lines = [
        "# Gate Note",
        "",
        f"- Timestamp: {ts}",
        f"- Config: `{config_path}`",
        "",
        "## Signal Probe Stats",
    ]
    for row in route_stats:
        lines.append(
            "- "
            f"{row['route_id']}: "
            f"label_lowfreq={row['mean_label_onehot_lowfreq_energy']:.6f}, "
            f"label_dirichlet={row['mean_label_onehot_dirichlet_energy']:.6f}, "
            f"label_indicator_lowfreq={row['mean_label_indicator_lowfreq_mean']:.6f}, "
            f"shell_lowfreq={row['mean_shell_lowfreq_energy']:.6f}, "
            f"sector_lowfreq={row['mean_sector_lowfreq_energy']:.6f}, "
            f"part_mean={row['mean_spectral_mode_participation_mean']:.6f}, "
            f"zero_eigs={row['mean_spectral_zero_eigs']:.1f}"
        )
    lines.extend(
        [
            "",
            "## Acceptance Read",
            f"- all_graphs_connected={acceptance['all_graphs_connected']}",
            f"- hopf_label_lowfreq_gap={acceptance['hopf_label_lowfreq_gap']:.6f}",
            f"- hopf_label_dirichlet_gap={acceptance['hopf_label_dirichlet_gap']:.6f}",
            f"- full_label_lowfreq_gap={acceptance['full_label_lowfreq_gap']:.6f}",
            f"- full_label_dirichlet_gap={acceptance['full_label_dirichlet_gap']:.6f}",
            f"- distinct_task_signal={acceptance['distinct_task_signal']}",
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
    graph_mode = str(cfg.get("graph_mode", "ambient_euclidean"))
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
            graph_mode=graph_mode,
        )
        run_seed(cmd, log_path)
        payload = load_config(output_path)
        seed_payloads.append(payload)
        seed_artifacts.append({"seed": seed, "analysis_path": output_path, "log_path": log_path})

    route_stats = []
    for route_id in route_ids:
        route_stats.append(aggregate_route_stats(route_id, route_args.get(route_id, {}), seed_payloads))
    route_stats.sort(key=lambda row: float(row["mean_label_onehot_lowfreq_energy"]), reverse=True)

    product_summary = summarize_group(classify_product_routes(route_stats))
    full_control_summary = summarize_group(classify_full_controls(route_stats))
    hopf_control_summary = summarize_group(classify_hopf_controls(route_stats))
    acceptance = evaluate_acceptance(route_stats)
    recommendation = choose_recommendation(acceptance, stage=stage)

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
        "full_control_summary": full_control_summary,
        "hopf_control_summary": hopf_control_summary,
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
