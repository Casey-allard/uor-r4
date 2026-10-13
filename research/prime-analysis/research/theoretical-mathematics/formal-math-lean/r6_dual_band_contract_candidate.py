#!/usr/bin/env python3
"""Build an R6 dual-band theorem-candidate pack from existing probe outputs.

This script does not claim a proof. It converts numerical probe outputs into a
single contract-shaped artifact that mirrors the new Lean frontier:
`R6DualBandPowerMajorantFittingTerm`.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _argmax_amplitude(rows: List[Dict[str, Any]]) -> int:
    best_i = 0
    best_v = float("-inf")
    for i, row in enumerate(rows):
        v = float(row.get("amplitude", 0.0))
        if v > best_v:
            best_v = v
            best_i = i
    return best_i


def build_candidate(multimode_path: str, dominant_scan_path: str) -> Dict[str, Any]:
    m = _read_json(multimode_path)
    d = _read_json(dominant_scan_path)

    best = m["best_by_score"]
    mode_rows = list(best.get("mode_rows", []))
    if not mode_rows:
        raise ValueError("multimode best_by_score.mode_rows is empty")

    dom_i = _argmax_amplitude(mode_rows)
    dom = mode_rows[dom_i]

    beta = float(m["config"]["beta"])
    tau_main = float(dom["tau"])
    a_main = float(dom["a"])
    b_main = float(dom["b"])
    amp_main = float(dom["amplitude"])
    amp_total = float(best["amplitude_total"])
    c_tail = float(best["remainder_majorant_C_tail"])
    eta_tail = float(best["remainder_majorant_eta"])
    x0_tail = float(best["remainder_majorant_tail_start_x"])
    ratio_tail_to_amp = float(best["tail_ratio_sup_to_amp_total"])

    power_contract_checks = {
        "beta_gt_half": beta > 0.5,
        "tau_pos": tau_main > 0.0,
        "main_nonzero": (abs(a_main) > 0.0) or (abs(b_main) > 0.0),
        "c_nonneg": c_tail >= 0.0,
        "eta_pos": eta_tail > 0.0,
        "mode_count_ge_6": int(best.get("mode_count", 0)) >= 6,
        "dominant_index_valid": 0 <= dom_i < int(best.get("mode_count", 0)),
    }
    power_contract_pass = all(power_contract_checks.values())

    near_strict_check = ratio_tail_to_amp < 1.0
    near_strict_margin = ratio_tail_to_amp - 1.0

    dominant_rows = d.get("rows", [])
    best_dominant_row = None
    if dominant_rows:
        best_dominant_row = min(
            dominant_rows, key=lambda r: float(r.get("tail_ratio_sup_to_amp", math.inf))
        )

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "source_files": {
            "multimode_probe": multimode_path,
            "dominant_scan_grid": dominant_scan_path,
        },
        "candidate_term": "R6DualBandPowerMajorantFittingTerm",
        "candidate_pack": {
            "beta": beta,
            "tau_main": tau_main,
            "a_main": a_main,
            "b_main": b_main,
            "amplitude_main": amp_main,
            "amplitude_total": amp_total,
            "dominant_index": dom_i,
            "mode_count": int(best.get("mode_count", 0)),
            "all_mode_rows": mode_rows,
            "power_majorant": {
                "C_tail": c_tail,
                "eta_tail": eta_tail,
                "x0_tail": x0_tail,
            },
            "tail_ratio_sup_to_amp_total": ratio_tail_to_amp,
        },
        "checks": {
            "power_majorant_contract_checks": power_contract_checks,
            "power_majorant_contract_pass": power_contract_pass,
            "near_strict_ratio_check": near_strict_check,
            "near_strict_ratio_margin": near_strict_margin,
        },
        "dominant_band_crosscheck": best_dominant_row,
        "interpretation": {
            "power_majorant_status": (
                "candidate-compatible-finite-range"
                if power_contract_pass
                else "candidate-incompatible-finite-range"
            ),
            "near_strict_status": (
                "satisfied-finite-range" if near_strict_check else "not-yet-satisfied-finite-range"
            ),
            "note": (
                "This is a finite-range theorem-candidate extraction from probe outputs. "
                "It is not an unconditional proof term."
            ),
        },
    }
    return result


def write_markdown(result: Dict[str, Any], out_md: str) -> None:
    p = result["candidate_pack"]
    c = result["checks"]
    with open(out_md, "w", encoding="utf-8") as f:
        f.write("# R6 Dual-Band Contract Candidate\n\n")
        f.write(f"Generated: {result['timestamp_utc']}\n\n")
        f.write("## Candidate pack\n")
        f.write(f"- beta: {p['beta']:.12g}\n")
        f.write(f"- tau_main: {p['tau_main']:.12g}\n")
        f.write(f"- a_main: {p['a_main']:.12g}\n")
        f.write(f"- b_main: {p['b_main']:.12g}\n")
        f.write(f"- amplitude_main: {p['amplitude_main']:.12g}\n")
        f.write(f"- amplitude_total: {p['amplitude_total']:.12g}\n")
        f.write(f"- dominant_index: {p['dominant_index']}\n")
        f.write(f"- mode_count: {p['mode_count']}\n")
        f.write(
            f"- power majorant: C_tail={p['power_majorant']['C_tail']:.12g}, "
            f"eta_tail={p['power_majorant']['eta_tail']:.12g}, "
            f"x0_tail={p['power_majorant']['x0_tail']:.12g}\n"
        )
        f.write(f"- tail_ratio_sup_to_amp_total: {p['tail_ratio_sup_to_amp_total']:.12g}\n\n")
        f.write("## Checks\n")
        for k, v in c["power_majorant_contract_checks"].items():
            f.write(f"- {k}: {v}\n")
        f.write(f"- power_majorant_contract_pass: {c['power_majorant_contract_pass']}\n")
        f.write(f"- near_strict_ratio_check: {c['near_strict_ratio_check']}\n")
        f.write(f"- near_strict_ratio_margin: {c['near_strict_ratio_margin']:.12g}\n\n")
        f.write("## Interpretation\n")
        f.write(f"- power_majorant_status: {result['interpretation']['power_majorant_status']}\n")
        f.write(f"- near_strict_status: {result['interpretation']['near_strict_status']}\n")
        f.write(f"- note: {result['interpretation']['note']}\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="Extract R6 dual-band theorem candidate from probe outputs")
    ap.add_argument(
        "--multimode",
        type=str,
        default="research/output/k1_multimode_phase_probe_2026-02-24_x1e20_m6_recheck.json",
    )
    ap.add_argument(
        "--dominant-scan",
        type=str,
        default="research/output/k1_w17_dominant_band_scan_grid_2026-02-24.json",
    )
    ap.add_argument(
        "--output",
        type=str,
        default="research/output/r6_dual_band_contract_candidate_2026-02-24.json",
    )
    args = ap.parse_args()

    result = build_candidate(args.multimode, args.dominant_scan)
    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    out_md = args.output.replace(".json", ".md")
    write_markdown(result, out_md)

    print(f"wrote: {args.output}")
    print(f"wrote: {out_md}")
    print("power_majorant_contract_pass:", result["checks"]["power_majorant_contract_pass"])
    print("near_strict_ratio_check:", result["checks"]["near_strict_ratio_check"])


if __name__ == "__main__":
    main()
