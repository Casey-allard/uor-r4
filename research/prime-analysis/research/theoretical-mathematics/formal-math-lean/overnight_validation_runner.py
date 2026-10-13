#!/usr/bin/env python3
"""Run overnight validation batch and emit gate report."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from datetime import datetime, timezone
from typing import Dict, List


def run(cmd: List[str]) -> None:
    subprocess.run(cmd, check=True)


def load_json(path: str) -> Dict[str, object]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def gate(flag: bool, reason: str) -> Dict[str, object]:
    return {"pass": bool(flag), "reason": reason}


def main() -> None:
    parser = argparse.ArgumentParser(description="Overnight validation orchestrator")
    parser.add_argument("--zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    parser.add_argument("--output", type=str, default="research/output/overnight_validation_report.json")
    args = parser.parse_args()

    out_dir = "research/output"
    os.makedirs(out_dir, exist_ok=True)

    pair2k = os.path.join(out_dir, "pair_correlation_probe_over_2k.json")
    pair8k = os.path.join(out_dir, "pair_correlation_probe_over_8k.json")
    exp64 = os.path.join(out_dir, "explicit_formula_probe_over_z64.json")
    exp256 = os.path.join(out_dir, "explicit_formula_probe_over_z256.json")
    ranked = os.path.join(out_dir, "formula_candidates_ranked_over.json")

    run([
        "python3",
        "research/pair_correlation_probe.py",
        "--zeros-file",
        args.zeros_file,
        "--limit",
        "2000",
        "--bootstrap-trials",
        "60",
        "--output",
        pair2k,
    ])
    run([
        "python3",
        "research/pair_correlation_probe.py",
        "--zeros-file",
        args.zeros_file,
        "--limit",
        "8000",
        "--bootstrap-trials",
        "60",
        "--output",
        pair8k,
    ])

    run([
        "python3",
        "research/explicit_formula_probe.py",
        "--zeros-file",
        args.zeros_file,
        "--zero-limit",
        "64",
        "--x-max",
        "120000",
        "--x-step",
        "500",
        "--output",
        exp64,
    ])
    run([
        "python3",
        "research/explicit_formula_probe.py",
        "--zeros-file",
        args.zeros_file,
        "--zero-limit",
        "256",
        "--x-max",
        "120000",
        "--x-step",
        "500",
        "--output",
        exp256,
    ])

    run([
        "python3",
        "research/formula_candidates_ranked.py",
        "--n-values",
        "20000,30000,40000,60000,80000,100000,120000",
        "--moduli-sets",
        "5,7,11,13,17;5,7,11,13,19;5,7,11,17,19",
        "--radius-power",
        "1.3",
        "--max-terms",
        "3",
        "--output",
        ranked,
    ])

    p2 = load_json(pair2k)
    p8 = load_json(pair8k)
    e64 = load_json(exp64)
    e256 = load_json(exp256)
    fr = load_json(ranked)

    pair2k_mse = p2["pair_correlation_fit"]["mse"]
    pair8k_mse = p8["pair_correlation_fit"]["mse"]
    w2 = p2["spacing_fit"]["wigner_mse"]
    w8 = p8["spacing_fit"]["wigner_mse"]

    rmse64 = e64["fit"]["rmse"]
    rmse256 = e256["fit"]["rmse"]
    std64 = e64["fit"]["scaled_residual_std"]
    std256 = e256["fit"]["scaled_residual_std"]

    top_model = fr["top_sparse_models"][0] if fr.get("top_sparse_models") else None
    top_inv = fr["top_invariants"][0] if fr.get("top_invariants") else None

    gates = {
        "pair_mse_improves_with_more_zeros": gate(
            pair8k_mse <= pair2k_mse,
            f"pair_mse 2k={pair2k_mse:.6f}, 8k={pair8k_mse:.6f}",
        ),
        "wigner_mse_improves_with_more_zeros": gate(
            w8 <= w2,
            f"wigner_mse 2k={w2:.6f}, 8k={w8:.6f}",
        ),
        "explicit_rmse_improves_with_more_zeros": gate(
            rmse256 <= rmse64,
            f"rmse z64={rmse64:.6f}, z256={rmse256:.6f}",
        ),
        "explicit_scaled_residual_std_improves": gate(
            std256 <= std64,
            f"scaled_std z64={std64:.6f}, z256={std256:.6f}",
        ),
        "top_sparse_model_transfer_positive": gate(
            bool(top_model) and float(top_model.get("transfer_r2_min", -1.0)) > 0.0,
            (
                f"transfer_r2_min={float(top_model.get('transfer_r2_min', -1.0)):.6f}, "
                f"features={top_model.get('features', [])}"
                if top_model
                else "missing top model"
            ),
        ),
        "top_invariant_stability_high": gate(
            bool(top_inv) and float(top_inv.get("stability_score", 0.0)) >= 0.95,
            (
                f"top_invariant={top_inv.get('name')} stability={float(top_inv.get('stability_score', 0.0)):.6f}"
                if top_inv
                else "missing top invariant"
            ),
        ),
    }

    pass_count = sum(1 for g in gates.values() if g["pass"])
    total = len(gates)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "zeros_file": args.zeros_file,
            "artifacts": {
                "pair_2k": pair2k,
                "pair_8k": pair8k,
                "explicit_z64": exp64,
                "explicit_z256": exp256,
                "formula_ranked": ranked,
            },
        },
        "summary": {
            "passed": pass_count,
            "total": total,
            "ratio": pass_count / total if total else 0.0,
        },
        "gates": gates,
        "recommendation": (
            "promote top formulas/invariants to long-run validation"
            if pass_count >= total - 1
            else "keep exploratory status; gather more overnight evidence"
        ),
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Overnight Validation Report\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"Passed gates: {pass_count}/{total}\n\n")
        for name, g in gates.items():
            mark = "PASS" if g["pass"] else "FAIL"
            f.write(f"- {mark} `{name}`: {g['reason']}\n")
        f.write(f"\nRecommendation: {report['recommendation']}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print(f"gates_passed: {pass_count}/{total}")


if __name__ == "__main__":
    main()
