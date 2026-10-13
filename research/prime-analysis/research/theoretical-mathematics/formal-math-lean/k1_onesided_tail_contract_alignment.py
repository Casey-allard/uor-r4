#!/usr/bin/env python3
"""Align one-sided tail power constants with the active constructive-gate contract.

Input bound (from W69 scanner):
  |R(x)/x^beta| <= C_total * x^(-eta),   x >= x1.

Active gate contract uses:
  max(-(R2 x), 0) <= q * A1
with R2(x) = R(x)/x^beta.

So:
  max(-(R2 x), 0) <= |R2(x)| <= C_total*x^(-eta).
To enforce max(-(R2 x),0) <= q_target*A1, it suffices that
  C_total*x^(-eta) <= q_target*A1
  x >= (C_total/(q_target*A1))^(1/eta).

This script computes those contract-level thresholds for A1 scenarios.
"""

from __future__ import annotations

import argparse
import glob
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Dict, List


def parse_float_list(text: str) -> List[float]:
    vals: List[float] = []
    for tok in text.split(","):
        tok = tok.strip()
        if not tok:
            continue
        vals.append(float(tok))
    vals = sorted(set(vals))
    if not vals:
        raise ValueError("expected at least one float")
    return vals


def threshold_stats(
    c_total: float, eta: float, q_target: float, a1_amp: float, x1: float
) -> Dict[str, float]:
    if c_total <= 0.0:
        return {"x_threshold": x1, "log10_x_threshold": math.log10(x1)}
    if eta <= 0.0 or q_target <= 0.0 or a1_amp <= 0.0:
        return {"x_threshold": float("inf"), "log10_x_threshold": float("inf")}
    denom = q_target * a1_amp
    if c_total <= denom:
        return {"x_threshold": x1, "log10_x_threshold": math.log10(x1)}
    log_x = math.log(c_total / denom) / eta
    log10_x = log_x / math.log(10.0)
    if log_x >= 700.0:
        return {"x_threshold": float("inf"), "log10_x_threshold": float(log10_x)}
    x_th = max(x1, float(math.exp(log_x)))
    return {"x_threshold": x_th, "log10_x_threshold": float(max(log10_x, math.log10(x1)))}


def load_best_tail_row(path: Path) -> Dict[str, float]:
    d = json.loads(path.read_text(encoding="utf-8"))
    cfg = d.get("config", {})
    best = d.get("best_feasible_row")
    if not isinstance(best, dict):
        raise ValueError("best_feasible_row missing in tail constants file")
    return {
        "beta": float(cfg["beta"]),
        "eta": float(cfg["eta"]),
        "x1": float(cfg["x1"]),
        "a1": float(cfg.get("a1", 0.98)),
        "theta_best": float(best["theta"]),
        "c_total": float(best["c_total"]),
        "c_band": float(best["c_band"]),
        "c_rem": float(best["c_rem"]),
    }


def collect_a1_samples(glob_pat: str) -> List[float]:
    vals: List[float] = []
    for p in sorted(glob.glob(glob_pat)):
        try:
            d = json.loads(Path(p).read_text(encoding="utf-8"))
        except Exception:
            continue
        fit = d.get("fit", {})
        a1 = fit.get("A1")
        if a1 is None:
            continue
        try:
            aval = float(a1)
        except Exception:
            continue
        if aval > 0.0:
            vals.append(aval)
    return vals


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 one-sided tail contract alignment")
    ap.add_argument(
        "--tail-constants-json",
        type=str,
        default="research/output/k1_w69_onesided_tail_theorem_grade_beta062_eta001_CHJ_2026-02-25.json",
    )
    ap.add_argument(
        "--a1-glob",
        type=str,
        default="research/output/k1_w58*_tau14_tau21_constructive_gate_nonneg_2026-02-24_x*e7.json",
    )
    ap.add_argument(
        "--a1-manual",
        type=str,
        default="",
        help="Optional comma-separated A1 values to include as extra scenarios",
    )
    ap.add_argument("--a1", type=float, default=0.98)
    ap.add_argument("--q-fractions", type=str, default="0.9,0.7,0.5")
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_onesided_tail_contract_alignment_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    q_fracs = parse_float_list(args.q_fractions)
    if any(q <= 0.0 for q in q_fracs):
        raise ValueError("q-fractions must be > 0")

    tail = load_best_tail_row(Path(args.tail_constants_json))
    c_total = tail["c_total"]
    eta = tail["eta"]
    x1 = tail["x1"]

    a1_samples = collect_a1_samples(args.a1_glob)
    manual = parse_float_list(args.a1_manual) if args.a1_manual.strip() else []
    a1_samples.extend(manual)
    a1_samples = sorted(v for v in set(a1_samples) if v > 0.0)
    if not a1_samples:
        raise ValueError("no positive A1 samples found")

    scenarios: List[Dict[str, object]] = []
    scenario_points = {
        "A1_min_sample": min(a1_samples),
        "A1_median_sample": a1_samples[len(a1_samples) // 2],
        "A1_max_sample": max(a1_samples),
    }

    for label, a1_amp in scenario_points.items():
        q_rows: List[Dict[str, float]] = []
        for frac in q_fracs:
            q_target = frac * args.a1
            th = threshold_stats(
                c_total=c_total,
                eta=eta,
                q_target=q_target,
                a1_amp=a1_amp,
                x1=x1,
            )
            q_rows.append(
                {
                    "q_fraction_of_a1": float(frac),
                    "q_target": float(q_target),
                    "x_threshold": float(th["x_threshold"]),
                    "log10_x_threshold": float(th["log10_x_threshold"]),
                }
            )
        q_at_x1 = c_total / (a1_amp * (x1**eta))
        scenarios.append(
            {
                "label": label,
                "A1_amp": float(a1_amp),
                "normalized_constant_C_over_A1": float(c_total / a1_amp),
                "q_at_x1_from_bound": float(q_at_x1),
                "thresholds": q_rows,
            }
        )

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "tail_constants_json": str(args.tail_constants_json),
            "a1_glob": str(args.a1_glob),
            "a1_manual": manual,
            "a1_gate": float(args.a1),
            "q_fractions": q_fracs,
        },
        "tail_bound": {
            "beta": float(tail["beta"]),
            "eta": float(tail["eta"]),
            "x1": float(tail["x1"]),
            "theta_best": float(tail["theta_best"]),
            "c_total": float(tail["c_total"]),
            "c_band": float(tail["c_band"]),
            "c_rem": float(tail["c_rem"]),
            "form": "|R(x)/x^beta| <= C_total*x^(-eta)",
        },
        "a1_samples": a1_samples,
        "contract_translation": {
            "form": "max(-(R2 x),0) <= q*A1",
            "identity": "R2(x)=R(x)/x^beta",
            "sufficient_condition": "C_total*x^(-eta) <= q_target*A1",
            "threshold_formula": "x >= (C_total/(q_target*A1))^(1/eta)",
        },
        "scenarios": scenarios,
        "interpretation": {
            "note": (
                "This report aligns the one-sided power envelope to the exact q*A1 gate contract. "
                "Thresholds are purely analytic consequences of the derived C_total."
            )
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 One-Sided Tail Contract Alignment ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Tail Bound Input")
    lines.append(f"- beta: `{tail['beta']}`")
    lines.append(f"- eta: `{tail['eta']}`")
    lines.append(f"- x1: `{tail['x1']}`")
    lines.append(f"- theta_best: `{tail['theta_best']}`")
    lines.append(f"- C_total: `{tail['c_total']}`")
    lines.append("")
    lines.append("## Contract Translation")
    lines.append("- `R2(x)=R(x)/x^beta`")
    lines.append("- `max(-(R2 x),0) <= q*A1` is implied by `|R2(x)| <= q*A1`")
    lines.append("- sufficient: `C_total*x^{-eta} <= q_target*A1`")
    lines.append("")
    lines.append("## A1 Samples")
    lines.append(
        "- sample range: "
        f"`[{min(a1_samples):.12f}, {max(a1_samples):.12f}]` (n={len(a1_samples)})"
    )
    lines.append("")
    for sc in scenarios:
        lines.append(f"### {sc['label']}")
        lines.append(f"- A1: `{float(sc['A1_amp']):.12f}`")
        lines.append(
            f"- normalized constant C/A1: `{float(sc['normalized_constant_C_over_A1']):.12f}`"
        )
        lines.append(f"- q(x1) from bound: `{float(sc['q_at_x1_from_bound']):.12f}`")
        lines.append("")
        lines.append("| q_target | x_threshold | log10(x_threshold) |")
        lines.append("|---:|---:|---:|")
        for th in sc["thresholds"]:
            xth = float(th["x_threshold"])
            xth_s = f"{xth:.6e}" if math.isfinite(xth) else "inf"
            l10 = float(th["log10_x_threshold"])
            l10_s = f"{l10:.6f}" if math.isfinite(l10) else "inf"
            lines.append(f"| {float(th['q_target']):.6f} | {xth_s} | {l10_s} |")
        lines.append("")
    lines.append("Analytic contract-alignment report only.")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
