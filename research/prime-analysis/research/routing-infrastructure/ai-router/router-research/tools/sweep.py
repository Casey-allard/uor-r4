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


def arg_items(common_args: Dict[str, Any], route_args: Dict[str, Any], seed: int, run_tag: str) -> List[str]:
    merged = dict(common_args)
    merged.update(route_args)
    merged["seed"] = seed
    merged["run_tag"] = run_tag

    out = []
    for k, v in merged.items():
        out.append(f"--{k}")
        out.append(str(v))
    return out


def run_one(cmd: List[str], log_path: str) -> int:
    with open(log_path, "w", encoding="utf-8") as lf:
        proc = subprocess.run(cmd, stdout=lf, stderr=subprocess.STDOUT)
    return int(proc.returncode)


def stage_mean_mse(summaries: List[Dict[str, Any]]) -> float:
    vals = []
    for s in summaries:
        m = s.get("metrics", {}) if isinstance(s, dict) else {}
        if "test_mse_after" in m:
            vals.append(float(m["test_mse_after"]))
    if not vals:
        return float("inf")
    return float(sum(vals) / len(vals))


def write_gate_note(path: str, config_path: str, decisions: List[str], recommendation: str):
    ts = datetime.datetime.now().isoformat(timespec="seconds")
    lines = [
        "# Gate Note",
        "",
        f"- Timestamp: {ts}",
        f"- Config: `{config_path}`",
        "",
        "## Decisions",
    ]
    for d in decisions:
        lines.append(f"- {d}")
    lines.extend([
        "",
        "## Recommendation",
        f"- {recommendation}",
        "",
        "## Required User Input",
        "- None unless recommendation marks branch escalation.",
    ])
    with open(path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--config", type=str, default="configs/route_sweep.yaml")
    ap.add_argument("--log_dir", type=str, default="results/raw")
    ap.add_argument("--gate_dir", type=str, default="docs/governance/gates")
    ap.add_argument("--python_bin", type=str, default=sys.executable)
    args = ap.parse_args()

    os.makedirs(args.log_dir, exist_ok=True)
    os.makedirs(args.gate_dir, exist_ok=True)

    cfg = load_config(args.config)
    stages: Dict[str, List[int]] = cfg.get("stages", {})
    common_args: Dict[str, Any] = cfg.get("common_args", {})
    routes: List[Dict[str, Any]] = cfg.get("routes", [])

    all_results: Dict[str, Dict[str, List[Dict[str, Any]]]] = {}
    total_runs = 0

    for route in routes:
        route_id = route["route_id"]
        route_args = route.get("args", {})
        all_results[route_id] = {}
        for stage_name in ["screen", "confirm", "finalize"]:
            seeds = stages.get(stage_name, [])
            stage_summaries: List[Dict[str, Any]] = []
            for seed in seeds:
                ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
                run_tag = f"{route_id}_{stage_name}_seed{seed}_{ts}"
                log_name = f"sweep_{run_tag}.log"
                log_path = os.path.join(args.log_dir, log_name)

                cmd = [args.python_bin, "hyperbolic_router_so8.py"] + arg_items(common_args, route_args, seed, run_tag)
                rc = run_one(cmd, log_path)
                summary = parse_summary_from_log(log_path)
                if summary is None:
                    summary = {
                        "parsed": False,
                        "log_file": log_name,
                        "metrics": {},
                        "return_code": rc,
                    }
                summary["log_file"] = log_name
                summary["return_code"] = rc
                stage_summaries.append(summary)
                total_runs += 1

            all_results[route_id][stage_name] = stage_summaries

    decisions: List[str] = []
    recommendation = "Proceed with current best route and continue staged sweeps."

    baseline_final = stage_mean_mse(all_results.get("R0", {}).get("finalize", []))
    best_route = "R0"
    best_score = baseline_final

    for route_id, per_stage in all_results.items():
        m = stage_mean_mse(per_stage.get("finalize", []))
        decisions.append(f"{route_id} finalize mean test_mse_after={m:.6f}")
        if m < best_score:
            best_score = m
            best_route = route_id

    if best_route != "R0":
        gain = (baseline_final - best_score) / max(abs(baseline_final), 1e-9)
        if gain >= 0.08:
            recommendation = (
                f"Escalate: {best_route} improves finalize mean MSE by {100.0 * gain:.2f}% vs R0; "
                "request user decision before promoting branch."
            )
        else:
            recommendation = f"Track {best_route} as candidate; continue confirm/finalize before branch promotion."

    gate_ts = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
    gate_path = os.path.join(args.gate_dir, f"gate_{gate_ts}.md")
    write_gate_note(gate_path, args.config, decisions, recommendation)

    print(f"Sweep complete. runs={total_runs} gate_note={gate_path}")


if __name__ == "__main__":
    main()
