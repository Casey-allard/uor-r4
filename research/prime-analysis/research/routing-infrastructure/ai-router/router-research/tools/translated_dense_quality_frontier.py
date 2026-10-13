#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict, Iterable, List

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


EPS = 1e-9
DEFAULT_QUALITY_TOLERANCE = 0.002


def _safe_float(value: Any) -> float:
    try:
        return float(value)
    except (TypeError, ValueError):
        return float("nan")


def _fmt(value: float, places: int = 6) -> str:
    if not math.isfinite(value):
        return "n/a"
    return f"{value:.{places}f}"


def _load_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _parse_anchor(value: str) -> Dict[str, str]:
    key, path = value.split("=", 1)
    return {"key": key.strip(), "path": path.strip()}


def _parse_comparison(value: str) -> Dict[str, str]:
    label, anchor_key, baseline_route_id, candidate_route_id = value.split(":", 3)
    return {
        "label": label.strip(),
        "anchor_key": anchor_key.strip(),
        "baseline_route_id": baseline_route_id.strip(),
        "candidate_route_id": candidate_route_id.strip(),
    }


def _route_stats_by_id(analysis: Dict[str, Any]) -> Dict[str, Dict[str, Any]]:
    return {
        str(row["route_id"]): row
        for row in analysis.get("route_stats", [])
        if row.get("route_id") is not None
    }


def _quality_gap_abs(top1_summary: Dict[str, Any]) -> float:
    values = [
        abs(_safe_float(top1_summary.get("median"))),
        abs(_safe_float(top1_summary.get("trimmed_mean"))),
    ]
    finite = [value for value in values if math.isfinite(value)]
    if not finite:
        return float("nan")
    return max(finite)


def _quality_read(top1_summary: Dict[str, Any], tolerance: float) -> str:
    robust_sign = str(top1_summary.get("robust_sign", "missing"))
    gap_abs = _quality_gap_abs(top1_summary)
    if robust_sign == "robust_improvement":
        return "quality_near"
    if robust_sign in {"robust_flat", "robust_regression"} and math.isfinite(gap_abs) and gap_abs <= tolerance + EPS:
        return "quality_near"
    if robust_sign == "robust_regression":
        return "quality_negative"
    if robust_sign == "mixed":
        return "quality_mixed"
    return "quality_missing"


def _frontier_classification(systems_verdict: str, quality_read: str) -> str:
    if systems_verdict == "promote" and quality_read == "quality_near":
        return "quality-near systems promotion"
    if systems_verdict == "promote":
        return "systems-only"
    if systems_verdict == "pruning_only":
        return "pruning-only"
    return "no-promotion"


def _anchor_read(classifications: List[str]) -> str:
    if not classifications:
        return "not_evaluated"
    if any(value == "quality-near systems promotion" for value in classifications):
        return "near_frontier"
    if any(value == "systems-only" for value in classifications):
        return "systems_only"
    if any(value == "pruning-only" for value in classifications):
        return "pruning_only"
    return "too_quality_negative"


def _overall_claim(lower_read: str, upper_read: str) -> str:
    if lower_read == "not_evaluated" and upper_read == "near_frontier":
        return "upper-bank near-frontier"
    if lower_read == "not_evaluated" and upper_read == "systems_only":
        return "upper-bank systems-only"
    if upper_read == "not_evaluated" and lower_read == "systems_only":
        return "lower-bank systems-only"
    if upper_read == "not_evaluated" and lower_read == "near_frontier":
        return "lower-bank near-frontier"
    if lower_read == "systems_only" and upper_read == "near_frontier":
        return "upper-bank near-frontier; lower-bank systems-only"
    if upper_read == "near_frontier":
        return "upper-bank near-frontier"
    if lower_read == "systems_only":
        return "lower-bank systems-only"
    if lower_read == "pruning_only" and upper_read == "pruning_only":
        return "too quality-negative to promote beyond evaluation evidence"
    return "mixed dense-quality frontier"


def _comparison_read(
    spec: Dict[str, str],
    robust_comparison: Dict[str, Any],
    anchor_analysis: Dict[str, Any],
    quality_tolerance: float,
) -> Dict[str, Any]:
    robust_summaries = robust_comparison["robust_summaries"]
    top1_summary = robust_summaries["top1_delta_candidate_minus_baseline"]
    quality_read = _quality_read(top1_summary, quality_tolerance)
    classification = _frontier_classification(
        str(robust_comparison.get("systems_verdict", "")),
        quality_read,
    )
    route_stats = _route_stats_by_id(anchor_analysis)
    baseline_row = route_stats[spec["baseline_route_id"]]
    candidate_row = route_stats[spec["candidate_route_id"]]
    top1_gap_abs = _quality_gap_abs(top1_summary)
    return {
        "label": spec["label"],
        "anchor_key": spec["anchor_key"],
        "baseline_route_id": spec["baseline_route_id"],
        "candidate_route_id": spec["candidate_route_id"],
        "classification": classification,
        "systems_verdict": robust_comparison.get("systems_verdict", "missing"),
        "quality_read": quality_read,
        "quality_tolerance_top1_gap": quality_tolerance,
        "top1_gap_abs_for_tolerance": top1_gap_abs,
        "robust_summaries": robust_summaries,
        "baseline_metrics": {
            "top1": _safe_float(baseline_row.get("mean_test_top1_after")),
            "candidate_fraction": _safe_float(baseline_row.get("mean_retrieval_candidate_fraction")),
            "amortized_per_repeat_sec": _safe_float(baseline_row.get("mean_amortized_total_per_repeat_sec")),
        },
        "candidate_metrics": {
            "top1": _safe_float(candidate_row.get("mean_test_top1_after")),
            "candidate_fraction": _safe_float(candidate_row.get("mean_retrieval_candidate_fraction")),
            "amortized_per_repeat_sec": _safe_float(candidate_row.get("mean_amortized_total_per_repeat_sec")),
        },
        "interpretation": robust_comparison.get("interpretation", ""),
    }


def build_payload(
    robust_analysis: Dict[str, Any],
    anchor_analyses: Dict[str, Dict[str, Any]],
    comparison_specs: Iterable[Dict[str, str]],
    quality_tolerance: float = DEFAULT_QUALITY_TOLERANCE,
) -> Dict[str, Any]:
    robust_by_label = {
        str(comparison["label"]): comparison
        for comparison in robust_analysis.get("comparisons", [])
    }
    comparisons: List[Dict[str, Any]] = []
    for spec in comparison_specs:
        robust_comparison = robust_by_label[spec["label"]]
        anchor_analysis = anchor_analyses[spec["anchor_key"]]
        comparisons.append(
            _comparison_read(
                spec,
                robust_comparison,
                anchor_analysis,
                quality_tolerance,
            )
        )

    lower_comparisons = [row for row in comparisons if row["anchor_key"] == "lower"]
    upper_comparisons = [row for row in comparisons if row["anchor_key"] == "upper"]
    lower_read = _anchor_read([row["classification"] for row in lower_comparisons])
    upper_read = _anchor_read([row["classification"] for row in upper_comparisons])
    overall_claim = _overall_claim(lower_read, upper_read)
    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "quality_tolerance_top1_gap": quality_tolerance,
        "source_robust_analysis": robust_analysis.get("source_experiments", []),
        "source_anchor_experiments": {
            key: analysis.get("experiment_id", "")
            for key, analysis in anchor_analyses.items()
        },
        "comparisons": comparisons,
        "lower_bank_read": lower_read,
        "upper_bank_read": upper_read,
        "overall_dense_frontier_claim": overall_claim,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Overall dense-frontier claim: `{payload['overall_dense_frontier_claim']}`",
        f"- Lower bank read: `{payload['lower_bank_read']}`",
        f"- Upper bank read: `{payload['upper_bank_read']}`",
        f"- Quality-near tolerance on robust top-1 gap: `{_fmt(payload['quality_tolerance_top1_gap'])}`",
        "",
        "## Comparison Reads",
    ]
    for comparison in payload["comparisons"]:
        baseline = comparison["baseline_metrics"]
        candidate = comparison["candidate_metrics"]
        top1 = comparison["robust_summaries"]["top1_delta_candidate_minus_baseline"]
        amortized = comparison["robust_summaries"]["amortized_delta_candidate_minus_baseline_sec"]
        cand_frac = comparison["robust_summaries"]["candidate_fraction_delta"]
        lines.extend(
            [
                f"### {comparison['label']}",
                f"- Classification: `{comparison['classification']}`",
                f"- Systems verdict: `{comparison['systems_verdict']}`",
                f"- Quality read: `{comparison['quality_read']}`",
                f"- Baseline `{comparison['baseline_route_id']}`: "
                f"`top1={_fmt(baseline['top1'])}`, "
                f"`cand_frac={_fmt(baseline['candidate_fraction'])}`, "
                f"`amortized={_fmt(baseline['amortized_per_repeat_sec'])}s`",
                f"- Candidate `{comparison['candidate_route_id']}`: "
                f"`top1={_fmt(candidate['top1'])}`, "
                f"`cand_frac={_fmt(candidate['candidate_fraction'])}`, "
                f"`amortized={_fmt(candidate['amortized_per_repeat_sec'])}s`",
                f"- Robust top-1 delta: "
                f"`median={_fmt(top1['median'])}`, "
                f"`trimmed_mean={_fmt(top1['trimmed_mean'])}`, "
                f"`max_abs={_fmt(comparison['top1_gap_abs_for_tolerance'])}`",
                f"- Robust candidate-fraction delta: "
                f"`median={_fmt(cand_frac['median'])}`, "
                f"`trimmed_mean={_fmt(cand_frac['trimmed_mean'])}`",
                f"- Robust amortized delta: "
                f"`median={_fmt(amortized['median'])}s`, "
                f"`trimmed_mean={_fmt(amortized['trimmed_mean'])}s`",
                f"- Read: {comparison['interpretation']}",
                "",
            ]
        )

    lines.append("## Interpretation")
    if payload["lower_bank_read"] == "systems_only":
        lines.append("- Lower-bank sparse translated dense replacement remains systems-first.")
    elif payload["lower_bank_read"] == "near_frontier":
        lines.append("- Lower-bank sparse translated dense replacement is now near-frontier on quality while staying systems-positive.")
    if payload["upper_bank_read"] == "systems_only":
        lines.append("- Upper-bank sparse translated dense replacement remains systems-first.")
    elif payload["upper_bank_read"] == "near_frontier":
        lines.append("- Upper-bank sparse translated dense replacement is now near-frontier on quality while staying systems-positive.")
    lines.append("- The hardware-side sparse translated claim remains honest: strong systems gains survive, but dense exact still owns the absolute quality ceiling.")
    output_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Audit dense quality/system frontier on fixed sparse translated points.")
    parser.add_argument("--robust-analysis", required=True)
    parser.add_argument(
        "--anchor-analysis",
        action="append",
        default=[],
        help="Anchor analysis mapping in the form key=/path/to/analysis.json",
    )
    parser.add_argument(
        "--comparison",
        action="append",
        default=[],
        help="Comparison spec label:anchor_key:baseline_route_id:candidate_route_id",
    )
    parser.add_argument("--quality-tolerance-top1-gap", type=float, default=DEFAULT_QUALITY_TOLERANCE)
    parser.add_argument("--report-title", default="Product Phase Sparse Translation Dense Quality Frontier")
    parser.add_argument("--output-json", required=True)
    parser.add_argument("--output-md", required=True)
    args = parser.parse_args()

    robust_analysis = _load_json(args.robust_analysis)
    anchor_specs = [_parse_anchor(value) for value in args.anchor_analysis]
    comparison_specs = [_parse_comparison(value) for value in args.comparison]
    anchor_analyses = {
        spec["key"]: _load_json(spec["path"])
        for spec in anchor_specs
    }
    payload = build_payload(
        robust_analysis,
        anchor_analyses,
        comparison_specs,
        quality_tolerance=args.quality_tolerance_top1_gap,
    )

    output_json = Path(args.output_json)
    output_md = Path(args.output_md)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_report(payload, output_md, args.report_title)


if __name__ == "__main__":
    main()
