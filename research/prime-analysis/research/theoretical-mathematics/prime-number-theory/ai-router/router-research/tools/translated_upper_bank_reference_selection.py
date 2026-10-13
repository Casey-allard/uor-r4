#!/usr/bin/env python3
import argparse
import datetime as dt
import json
import math
import os
import sys
from pathlib import Path
from typing import Any, Dict

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)


DEFAULT_TOP1_TOLERANCE = 0.0001
DEFAULT_AMORTIZED_TOLERANCE_SEC = 0.05
DEFAULT_CANDIDATE_FRACTION_TOLERANCE = 0.002


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


def _comparison_by_label(analysis: Dict[str, Any], label: str) -> Dict[str, Any]:
    for comparison in analysis.get("comparisons", []):
        if str(comparison.get("label")) == label:
            return comparison
    raise KeyError(f"comparison label not found: {label}")


def _route_by_id(analysis: Dict[str, Any], route_id: str) -> Dict[str, Any]:
    for row in analysis.get("route_stats", []):
        if str(row.get("route_id")) == route_id:
            return row
    raise KeyError(f"route_id not found: {route_id}")


def _route_metrics(route_row: Dict[str, Any]) -> Dict[str, float]:
    return {
        "top1": _safe_float(route_row.get("mean_test_top1_after")),
        "candidate_fraction": _safe_float(route_row.get("mean_retrieval_candidate_fraction")),
        "amortized_per_repeat_sec": _safe_float(route_row.get("mean_amortized_total_per_repeat_sec")),
    }


def _eligible(quality_comparison: Dict[str, Any], gap_comparison: Dict[str, Any]) -> bool:
    return (
        str(quality_comparison.get("classification")) == "quality-near systems promotion"
        and str(gap_comparison.get("next_branch_bias")) == "operationally_negligible"
    )


def _within_tolerance(candidate: Dict[str, float], other: Dict[str, float], top1_tol: float, amortized_tol: float, cand_frac_tol: float) -> bool:
    return (
        candidate["top1"] + top1_tol >= other["top1"]
        and candidate["amortized_per_repeat_sec"] <= other["amortized_per_repeat_sec"] + amortized_tol
        and candidate["candidate_fraction"] <= other["candidate_fraction"] + cand_frac_tol
    )


def _tie_break_key(metrics: Dict[str, float]):
    return (
        metrics["top1"],
        -metrics["amortized_per_repeat_sec"],
        -metrics["candidate_fraction"],
    )


def build_payload(
    quality_analysis: Dict[str, Any],
    gap_analysis: Dict[str, Any],
    confirm_analysis: Dict[str, Any],
    *,
    soft_label: str = "upper_dense_vs_soft_sparse",
    backfill_label: str = "upper_dense_vs_backfill",
    soft_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000",
    backfill_route_id: str = "CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000",
    top1_tolerance: float = DEFAULT_TOP1_TOLERANCE,
    amortized_tolerance_sec: float = DEFAULT_AMORTIZED_TOLERANCE_SEC,
    candidate_fraction_tolerance: float = DEFAULT_CANDIDATE_FRACTION_TOLERANCE,
) -> Dict[str, Any]:
    soft_quality = _comparison_by_label(quality_analysis, soft_label)
    backfill_quality = _comparison_by_label(quality_analysis, backfill_label)
    soft_gap = _comparison_by_label(gap_analysis, soft_label)
    backfill_gap = _comparison_by_label(gap_analysis, backfill_label)
    soft_metrics = _route_metrics(_route_by_id(confirm_analysis, soft_route_id))
    backfill_metrics = _route_metrics(_route_by_id(confirm_analysis, backfill_route_id))

    soft_ok = _eligible(soft_quality, soft_gap)
    backfill_ok = _eligible(backfill_quality, backfill_gap)
    soft_within = soft_ok and _within_tolerance(
        soft_metrics,
        backfill_metrics,
        top1_tolerance,
        amortized_tolerance_sec,
        candidate_fraction_tolerance,
    )
    backfill_within = backfill_ok and _within_tolerance(
        backfill_metrics,
        soft_metrics,
        top1_tolerance,
        amortized_tolerance_sec,
        candidate_fraction_tolerance,
    )

    deltas = {
        "soft_minus_backfill_top1": soft_metrics["top1"] - backfill_metrics["top1"],
        "soft_minus_backfill_candidate_fraction": soft_metrics["candidate_fraction"] - backfill_metrics["candidate_fraction"],
        "soft_minus_backfill_amortized_per_repeat_sec": soft_metrics["amortized_per_repeat_sec"] - backfill_metrics["amortized_per_repeat_sec"],
    }

    selection_mode = "keep_pair"
    promoted_route_id = ""
    promoted_label = ""
    supporting_route_id = ""
    supporting_label = ""
    if soft_within and not backfill_within:
        selection_mode = "soft_within_tolerance_dominance"
        promoted_route_id = soft_route_id
        promoted_label = soft_label
        supporting_route_id = backfill_route_id
        supporting_label = backfill_label
    elif backfill_within and not soft_within:
        selection_mode = "backfill_within_tolerance_dominance"
        promoted_route_id = backfill_route_id
        promoted_label = backfill_label
        supporting_route_id = soft_route_id
        supporting_label = soft_label
    elif soft_within and backfill_within:
        selection_mode = "tie_break_within_tolerance"
        if _tie_break_key(soft_metrics) >= _tie_break_key(backfill_metrics):
            promoted_route_id = soft_route_id
            promoted_label = soft_label
            supporting_route_id = backfill_route_id
            supporting_label = backfill_label
        else:
            promoted_route_id = backfill_route_id
            promoted_label = backfill_label
            supporting_route_id = soft_route_id
            supporting_label = soft_label

    if selection_mode == "keep_pair":
        carry_forward_contract = "explicit_pair"
        interpretation = (
            "Both upper-bank sparse translated points remain valid dense-near systems promotions, "
            "but the pair does not collapse cleanly to one promoted reference under the configured tolerances."
        )
    else:
        carry_forward_contract = "single_promoted_reference"
        interpretation = (
            f"{promoted_route_id} is promoted as the single upper-bank dense-near routed reference. "
            f"The remaining delta versus {supporting_route_id} stays inside the configured top-1, amortized, "
            "and candidate-fraction tolerance band, so the supporting route can drop to comparator status."
        )

    return {
        "generated_at": dt.datetime.now().astimezone().isoformat(),
        "sources": {
            "quality_analysis": quality_analysis.get("source_robust_analysis", []),
            "gap_analysis": gap_analysis.get("source_experiments", []),
            "confirm_analysis": confirm_analysis.get("experiment_id", ""),
        },
        "tolerances": {
            "top1": top1_tolerance,
            "amortized_per_repeat_sec": amortized_tolerance_sec,
            "candidate_fraction": candidate_fraction_tolerance,
        },
        "soft_sparse": {
            "label": soft_label,
            "route_id": soft_route_id,
            "quality_classification": soft_quality.get("classification", ""),
            "gap_bias": soft_gap.get("next_branch_bias", ""),
            "eligible": soft_ok,
            "metrics": soft_metrics,
        },
        "bounded_backfill": {
            "label": backfill_label,
            "route_id": backfill_route_id,
            "quality_classification": backfill_quality.get("classification", ""),
            "gap_bias": backfill_gap.get("next_branch_bias", ""),
            "eligible": backfill_ok,
            "metrics": backfill_metrics,
        },
        "pair_deltas": deltas,
        "selection_mode": selection_mode,
        "carry_forward_contract": carry_forward_contract,
        "promoted_route_id": promoted_route_id,
        "promoted_label": promoted_label,
        "supporting_route_id": supporting_route_id,
        "supporting_label": supporting_label,
        "interpretation": interpretation,
    }


def write_report(payload: Dict[str, Any], output_path: Path, title: str) -> None:
    soft = payload["soft_sparse"]
    backfill = payload["bounded_backfill"]
    deltas = payload["pair_deltas"]
    lines = [
        f"# {title}",
        "",
        "## Summary",
        f"- Carry-forward contract: `{payload['carry_forward_contract']}`",
        f"- Selection mode: `{payload['selection_mode']}`",
        f"- Promoted route: `{payload['promoted_route_id'] or 'none'}`",
        f"- Supporting route: `{payload['supporting_route_id'] or 'none'}`",
        f"- Read: {payload['interpretation']}",
        "",
        "## Tolerances",
        f"- Top-1 tolerance: `{_fmt(payload['tolerances']['top1'])}`",
        f"- Amortized tolerance: `{_fmt(payload['tolerances']['amortized_per_repeat_sec'])}s`",
        f"- Candidate-fraction tolerance: `{_fmt(payload['tolerances']['candidate_fraction'])}`",
        "",
        "## Upper-Bank Points",
        f"- Soft sparse `{soft['route_id']}`: "
        f"`top1={_fmt(soft['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(soft['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(soft['metrics']['amortized_per_repeat_sec'])}s`, "
        f"`classification={soft['quality_classification']}`, "
        f"`gap_bias={soft['gap_bias']}`",
        f"- Bounded backfill `{backfill['route_id']}`: "
        f"`top1={_fmt(backfill['metrics']['top1'])}`, "
        f"`cand_frac={_fmt(backfill['metrics']['candidate_fraction'])}`, "
        f"`amortized={_fmt(backfill['metrics']['amortized_per_repeat_sec'])}s`, "
        f"`classification={backfill['quality_classification']}`, "
        f"`gap_bias={backfill['gap_bias']}`",
        "",
        "## Pair Deltas",
        f"- Soft minus backfill top-1: `{_fmt(deltas['soft_minus_backfill_top1'])}`",
        f"- Soft minus backfill candidate fraction: `{_fmt(deltas['soft_minus_backfill_candidate_fraction'])}`",
        f"- Soft minus backfill amortized: `{_fmt(deltas['soft_minus_backfill_amortized_per_repeat_sec'])}s`",
    ]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser()
    ap.add_argument("--quality-analysis", required=True)
    ap.add_argument("--gap-analysis", required=True)
    ap.add_argument("--confirm-analysis", required=True)
    ap.add_argument("--soft-label", default="upper_dense_vs_soft_sparse")
    ap.add_argument("--backfill-label", default="upper_dense_vs_backfill")
    ap.add_argument("--soft-route-id", default="CHART_H4XH4_FIELD_A150_EVT_T070_CPX8_Q01_T40000")
    ap.add_argument("--backfill-route-id", default="CHART_H4XH4_FIELD_A150_EVT_T070_BF2_SB1_CPX8_Q01_T40000")
    ap.add_argument("--top1-tolerance", type=float, default=DEFAULT_TOP1_TOLERANCE)
    ap.add_argument("--amortized-tolerance-sec", type=float, default=DEFAULT_AMORTIZED_TOLERANCE_SEC)
    ap.add_argument("--candidate-fraction-tolerance", type=float, default=DEFAULT_CANDIDATE_FRACTION_TOLERANCE)
    ap.add_argument("--output-json", required=True)
    ap.add_argument("--output-report", required=True)
    ap.add_argument("--report-title", default="Upper-Bank Dense Reference Selection")
    return ap.parse_args()


def main() -> None:
    args = parse_args()
    payload = build_payload(
        _load_json(args.quality_analysis),
        _load_json(args.gap_analysis),
        _load_json(args.confirm_analysis),
        soft_label=args.soft_label,
        backfill_label=args.backfill_label,
        soft_route_id=args.soft_route_id,
        backfill_route_id=args.backfill_route_id,
        top1_tolerance=float(args.top1_tolerance),
        amortized_tolerance_sec=float(args.amortized_tolerance_sec),
        candidate_fraction_tolerance=float(args.candidate_fraction_tolerance),
    )
    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_report(payload, Path(args.output_report), title=args.report_title)


if __name__ == "__main__":
    main()
