#!/usr/bin/env python3
"""Compose W72 (tau12 source-locked C2) with W70 tail contract into one chain packet.

This script does three things:
1) verifies the tau12 C2 closure certificate from W72,
2) plugs it into the W70 q*A1 tail contract with explicit thresholds,
3) emits a single theorem-facing chain statement for q<a1 lower-envelope closure.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def finite_or_none(value: float) -> float | None:
    return value if math.isfinite(value) else None


def compose_rows(
    w70: dict[str, Any],
    *,
    x_round: float,
    a1_gate: float,
) -> list[dict[str, Any]]:
    log10_x_round = math.log10(x_round)
    rows: list[dict[str, Any]] = []
    for sc in w70.get("scenarios", []):
        if not isinstance(sc, dict):
            continue
        a1_amp = float(sc.get("A1_amp", 0.0))
        trows: list[dict[str, Any]] = []
        for th in sc.get("thresholds", []):
            if not isinstance(th, dict):
                continue
            q_target = float(th.get("q_target", 0.0))
            q_frac = float(th.get("q_fraction_of_a1", 0.0))
            log10_x_tail = float(th.get("log10_x_threshold", float("inf")))
            log10_x_joint = max(log10_x_round, log10_x_tail)
            x_joint = float("inf") if log10_x_joint > 308.0 else 10.0 ** log10_x_joint
            gap = a1_gate - q_target
            trows.append(
                {
                    "q_fraction_of_a1": q_frac,
                    "q_target": q_target,
                    "q_lt_a1": q_target < a1_gate,
                    "a1_minus_q": gap,
                    "x_tail_log10_threshold": log10_x_tail,
                    "x_joint_log10_threshold": log10_x_joint,
                    "x_joint_threshold": x_joint,
                    "lower_envelope_constant_c": a1_amp * gap,
                }
            )
        rows.append(
            {
                "label": str(sc.get("label", "")),
                "A1_amp": a1_amp,
                "x_round_threshold": x_round,
                "x_round_log10_threshold": log10_x_round,
                "q_rows": trows,
            }
        )
    return rows


def build_md(payload: dict[str, Any]) -> str:
    c2 = payload["c2_status"]
    w70 = payload["w70_tail"]
    contract = payload["composed_contract"]
    rows = payload["scenario_joint_thresholds"]

    lines: list[str] = []
    lines.append(f"# K1 W73 Tau12 Chain Composition ({payload['date']})")
    lines.append("")
    lines.append("## Objective")
    lines.append(
        "Compose source-locked tau12 C2 closure (W72) with rounding-preservation and W70 tail contract into one explicit q<a1 chain packet."
    )
    lines.append("")
    lines.append("## Component Status")
    lines.append(f"- C2 source-locked closure certified: `{str(c2['c2_half_closed_under_assumptions']).lower()}`")
    lines.append(
        f"- rho interval: `[{c2['rho_lo']:.18f}, {c2['rho_hi']:.18f}]`, excludes `3/2`: `{str(c2['rho_excludes_3_over_2']).lower()}`"
    )
    lines.append(f"- tau source rows: `{c2['tau_source_count']}`")
    lines.append(
        f"- rounding threshold (c0=1/2, a1={contract['a1_gate']}): `X_round <= {contract['x_round']:.12f}`"
    )
    lines.append(
        f"- W70 tail constants: `beta={w70['beta']}`, `eta={w70['eta']}`, `C_total={w70['c_total']:.12f}`, `x1={w70['x1']:.1e}`"
    )
    lines.append("")
    lines.append("## Composed Chain Statement")
    lines.append(
        "For any `A1>0`, `A2>=0`, and `0<q<a1`, under the W72 source-locked C2 assumptions and W70 tail envelope,"
    )
    lines.append("- define `X_tail = max(x1, (C_total/(q*A1))^(1/eta))`,")
    lines.append("- define `X_joint = max(X_round, X_tail)`,")
    lines.append("- then `c = A1*(a1-q) > 0` and for every `X` there exists `x>=X` with `Y(x) >= c`.")
    lines.append("")
    lines.append("This is exactly the theorem-shape consumed by")
    lines.append("`normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1`.")
    lines.append("")
    lines.append("## Scenario Thresholds")
    for sc in rows:
        lines.append(f"- {sc['label']} (A1={sc['A1_amp']:.12f}):")
        lines.append(
            f"  - X_round: `{sc['x_round_threshold']:.12f}` (log10 `{sc['x_round_log10_threshold']:.6f}`)"
        )
        for qrow in sc["q_rows"]:
            lines.append(
                "  - "
                f"q={qrow['q_target']:.12f} (fraction `{qrow['q_fraction_of_a1']}`), "
                f"a1-q={qrow['a1_minus_q']:.12f}, "
                f"log10(X_joint)={qrow['x_joint_log10_threshold']:.6f}, "
                f"c=A1*(a1-q)={qrow['lower_envelope_constant_c']:.12f}"
            )
    lines.append("")
    lines.append("## Remaining Blocker")
    lines.append(f"- {payload['remaining_blockers'][0]}")
    lines.append("")
    return "\n".join(lines)


def main() -> None:
    ap = argparse.ArgumentParser(description="Compose W72 C2 closure with W70 tail contract")
    ap.add_argument(
        "--w72-json",
        default="research/output/k1_w72_c2_source_locked_closure_2026-02-25.json",
    )
    ap.add_argument(
        "--w70-json",
        default="research/output/k1_w70_onesided_tail_contract_alignment_2026-02-25.json",
    )
    ap.add_argument(
        "--target-lock-json",
        default="research/output/proof_resume_checkpoint_2026-02-24_k1_target_lock.json",
    )
    ap.add_argument(
        "--out-prefix",
        default=(
            "research/output/"
            f"k1_w73_tau12_chain_composition_{datetime.now(timezone.utc).date().isoformat()}"
        ),
    )
    args = ap.parse_args()

    w72 = read_json(Path(args.w72_json))
    w70 = read_json(Path(args.w70_json))
    target_lock = read_json(Path(args.target_lock_json))

    c2 = w72.get("c2_dichotomy_certificate", {})
    rho = w72.get("rho_interval_certificate", {})
    tau = w72.get("tau_lock", {})
    rnd = w72.get("rounding_thresholds", {})
    tail = w70.get("tail_bound", {})

    c2_closed = bool(c2.get("c2_half_closed_under_assumptions", False))
    x_round = float(rnd.get("x_round_upper_for_c0_half", float("inf")))
    a1_gate = float(w70.get("inputs", {}).get("a1_gate", 0.98))

    rows = compose_rows(w70, x_round=x_round, a1_gate=a1_gate)

    all_q_lt_a1 = all(
        qrow.get("q_lt_a1", False)
        for sc in rows
        for qrow in sc.get("q_rows", [])
    )

    payload: dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "date": datetime.now(timezone.utc).date().isoformat(),
        "inputs": {
            "w72_json": str(Path(args.w72_json)),
            "w70_json": str(Path(args.w70_json)),
            "target_lock_json": str(Path(args.target_lock_json)),
        },
        "c2_status": {
            "c2_half_closed_under_assumptions": c2_closed,
            "c0_target": float(c2.get("c0_target", 0.5)),
            "rho_lo": float(rho.get("rho_lo", float("nan"))),
            "rho_hi": float(rho.get("rho_hi", float("nan"))),
            "rho_excludes_3_over_2": bool(rho.get("rho_excludes_3_over_2", False)),
            "tau_source_count": len(tau.get("sources", [])) if isinstance(tau, dict) else 0,
            "tau1_width": float(tau.get("tau1_width", float("nan"))),
            "tau2_width": float(tau.get("tau2_width", float("nan"))),
        },
        "w70_tail": {
            "beta": float(tail.get("beta", float("nan"))),
            "eta": float(tail.get("eta", float("nan"))),
            "c_total": float(tail.get("c_total", float("nan"))),
            "x1": float(tail.get("x1", float("nan"))),
        },
        "composed_contract": {
            "a1_gate": a1_gate,
            "x_round": x_round,
            "x_round_log10": math.log10(x_round),
            "x_tail_formula": "max(x1, (C_total/(q*A1))^(1/eta))",
            "x_joint_formula": "max(X_round, X_tail)",
            "lower_envelope_constant": "c = A1*(a1-q)",
            "symbolic_theorem_name": "normalized_lower_envelope_of_buffered_c2_rounding_tail_and_q_lt_a1",
        },
        "scenario_joint_thresholds": rows,
        "consistency_checks": {
            "all_sampled_q_lt_a1": all_q_lt_a1,
            "c2_certificate_true": c2_closed,
            "x_round_finite": math.isfinite(x_round),
        },
        "remaining_blockers": [
            str(target_lock.get("remaining_step", ""))
        ],
        "interpretation": {
            "status": (
                "C2+rounding+tail chain is quantitatively composed and checkpoint-ready."
                if c2_closed and all_q_lt_a1 and math.isfinite(x_round)
                else "Composition not fully certified with current artifacts."
            ),
            "next_step": "Attack the final non-circular K1 source-provider theorem term instantiation.",
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")

    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    out_md.write_text(build_md(payload), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
