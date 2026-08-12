#!/usr/bin/env python3
import argparse
import json
import math
from pathlib import Path
from typing import Any, Dict, Iterable, List, Optional


TIMING_KEYS = ["dataset", "chart_opt", "routing_eval", "training_route", "training_update", "training_ema", "growth", "total"]


def load_analysis(path: Path) -> Dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def route_rows(payload: Dict[str, Any], route_id: str) -> List[Dict[str, Any]]:
    results = payload.get("results", {})
    if isinstance(results, dict):
        rows = results.get(route_id, [])
        if isinstance(rows, list):
            return rows
    return []


def mean(values: Iterable[float]) -> float:
    vals = [float(v) for v in values]
    if not vals:
        return float("nan")
    return sum(vals) / float(len(vals))


def route_stats_row(payload: Dict[str, Any], route_id: str) -> Optional[Dict[str, Any]]:
    stats = payload.get("route_stats", [])
    if not isinstance(stats, list):
        return None
    for row in stats:
        if isinstance(row, dict) and row.get("route_id") == route_id:
            return row
    return None


def route_timing_means(rows: List[Dict[str, Any]]) -> Dict[str, float]:
    out: Dict[str, float] = {}
    for key in TIMING_KEYS:
        out[key] = mean(
            float(r.get("timings_sec", {}).get(key, float("nan")))
            for r in rows
            if isinstance(r.get("timings_sec", {}), dict) and key in r.get("timings_sec", {})
        )
    return out


def fmt(value: float, digits: int = 3) -> str:
    if value is None or math.isnan(value):
        return "n/a"
    return f"{value:.{digits}f}"


def build_markdown(payload: Dict[str, Any], route_order: List[str], baseline: str, title: str) -> str:
    lines: List[str] = [f"# {title}", ""]
    config = payload.get("config", "")
    exp_id = payload.get("experiment_id", "")
    generated = payload.get("generated_at", "")
    recommendation = payload.get("recommendation", "")
    lines.extend(
        [
            f"- Experiment: `{exp_id}`" if exp_id else "- Experiment: n/a",
            f"- Config: `{config}`" if config else "- Config: n/a",
            f"- Generated: `{generated}`" if generated else "- Generated: n/a",
        ]
    )
    if recommendation:
        lines.append(f"- Recommendation: {recommendation}")
    lines.append("")

    baseline_stats = route_stats_row(payload, baseline)
    baseline_total = None if baseline_stats is None else float(baseline_stats.get("mean_total_sec", float("nan")))
    baseline_mse = None if baseline_stats is None else float(baseline_stats.get("mean_test_mse_after", float("nan")))

    lines.extend(
        [
            "## Route Summary",
            "| Route | Health | MSE | Total (s) | Runtime vs baseline | MSE vs baseline | Shells | Sectors | Buckets |",
            "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for route_id in route_order:
        stats = route_stats_row(payload, route_id)
        if stats is None:
            continue
        total = float(stats.get("mean_total_sec", float("nan")))
        mse = float(stats.get("mean_test_mse_after", float("nan")))
        runtime_ratio = total / max(baseline_total, 1e-12) if baseline_total and not math.isnan(total) else float("nan")
        mse_ratio = mse / max(baseline_mse, 1e-12) if baseline_mse and not math.isnan(mse) else float("nan")
        lines.append(
            "| "
            f"`{route_id}` | "
            f"{'pass' if stats.get('passes_health_gate') else 'fail'} | "
            f"{fmt(mse, 6)} | "
            f"{fmt(total, 3)} | "
            f"{fmt(runtime_ratio, 3)} | "
            f"{fmt(mse_ratio, 3)} | "
            f"{fmt(float(stats.get('mean_eval_shells', float('nan'))), 2)} | "
            f"{fmt(float(stats.get('mean_eval_sectors', float('nan'))), 2)} | "
            f"{fmt(float(stats.get('mean_buckets', float('nan'))), 2)} |"
        )
    lines.append("")

    lines.extend(
        [
            "## Timing Breakdown",
            "| Route | Dataset | Chart Opt | Routing Eval | Train Route | Train Update | Training EMA | Growth | Total |",
            "|---|---:|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for route_id in route_order:
        rows = route_rows(payload, route_id)
        if not rows:
            continue
        means = route_timing_means(rows)
        lines.append(
            "| "
            f"`{route_id}` | "
            f"{fmt(means['dataset'], 3)} | "
            f"{fmt(means['chart_opt'], 3)} | "
            f"{fmt(means['routing_eval'], 3)} | "
            f"{fmt(means['training_route'], 3)} | "
            f"{fmt(means['training_update'], 3)} | "
            f"{fmt(means['training_ema'], 3)} | "
            f"{fmt(means['growth'], 3)} | "
            f"{fmt(means['total'], 3)} |"
        )
    lines.append("")

    lines.extend([
        "## Timing Read",
        "- `chart_opt` dominates when it is near the route total.",
        "- `training_route` isolates per-step rerouting cost inside the EMA phase.",
        "- `training_update` isolates bucket prediction and EMA writes inside the EMA phase.",
        "",
    ])
    return "\n".join(lines)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--input", required=True, help="analysis JSON path")
    ap.add_argument("--output", default="", help="optional markdown output path")
    ap.add_argument("--baseline", default="R0")
    ap.add_argument("--title", default="Hopf Cost Decomposition")
    ap.add_argument("--routes", nargs="*", default=[])
    args = ap.parse_args()

    payload = load_analysis(Path(args.input))
    route_order = args.routes
    if not route_order:
        route_order = [row.get("route_id", "") for row in payload.get("route_stats", []) if isinstance(row, dict)]
    md = build_markdown(payload, route_order, args.baseline, args.title)
    if args.output:
        out_path = Path(args.output)
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(md + "\n", encoding="utf-8")
    else:
        print(md)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
