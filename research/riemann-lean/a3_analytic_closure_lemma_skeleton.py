#!/usr/bin/env python3
"""Build a theorem-ready O3 (A3) analytic-closure skeleton from existing artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 analytic-closure lemma skeleton builder")
    ap.add_argument(
        "--a3-primary",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json",
    )
    ap.add_argument(
        "--a3-stress",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json",
    )
    ap.add_argument(
        "--eta-probe",
        default="research/output/a3_eta_exponent_probe_2026-02-17.json",
    )
    ap.add_argument(
        "--eta4-just",
        default="research/output/a3_eta4_justification_probe_2026-02-17_sf3.json",
    )
    ap.add_argument(
        "--output",
        default="research/output/a3_analytic_closure_lemma_skeleton_2026-02-17.json",
    )
    args = ap.parse_args()

    primary = read_json(args.a3_primary)
    stress = read_json(args.a3_stress)
    eta_probe = read_json(args.eta_probe)
    eta4_just = read_json(args.eta4_just)

    cfg = primary["config"]
    p_eta = primary["eta_envelope"]
    p_h = primary["h_transfer_envelope"]
    p_checks = primary["checks"]
    s_h = stress["h_transfer_envelope"]
    s_checks = stress["checks"]

    lemma = {
        "name": "Lemma C* (A3 Bridge-Growth Analytic Closure)",
        "proof_primary_branch": "offdiag_dynamic_eta_fixed_Aeta4",
        "statement_template": (
            "For wheel family W and all x >= x0: |H_W(x)| <= C_H (log x)^A_H, "
            "obtained via eta_+(x;W) <= C_eta (log x)^A_eta and "
            "|H| <= sqrt((1+eta_+) E2/x)."
        ),
        "frozen_constants": {
            "A_eta": float(p_eta["A_eta"]),
            "C_eta_uplifted": float(p_eta["C_eta_uplifted"]),
            "A_H_primary": float(p_h["A_H"]),
            "C_H_primary": float(p_h["C_H_from_density_transfer"]),
            "A_H_stress": float(s_h["A_H"]),
            "C_H_stress": float(s_h["C_H_from_density_transfer"]),
            "eta_safety": float(cfg["eta_safety"]),
            "m_zero": int(cfg["m_zero"]),
            "zero_kernel": str(cfg["zero_kernel"]),
            "kernel_scale": float(cfg["kernel_scale"]),
            "bases": list(cfg["bases"]),
        },
        "empirical_validation": {
            "primary_train_holds": bool(p_checks["train"]["holds"]),
            "primary_valid_holds": bool(p_checks["valid"]["holds"]),
            "primary_valid_ratio_max": float(p_checks["valid"]["ratio_max"]),
            "primary_valid_max_gap_h_minus_rhs": float(p_checks["valid"]["max_gap_h_minus_rhs"]),
            "stress_valid_holds": bool(s_checks["valid"]["holds"]),
            "stress_valid_ratio_max": float(s_checks["valid"]["ratio_max"]),
            "stress_valid_max_gap_h_minus_rhs": float(s_checks["valid"]["max_gap_h_minus_rhs"]),
            "eta4_just_valid_ratio_over_c_eta": float(eta4_just["fit_summary"]["valid_ratio_over_C_eta"]),
        },
        "supporting_checks": {
            "eta_probe_safety_needed_for_valid": float(
                eta_probe["recommended_under_safety_3p0"]["safety_needed_for_valid"]
            ),
            "eta_probe_recommended_A_eta": float(
                eta_probe["recommended_under_safety_3p0"]["A_eta"]
            ),
        },
        "analytic_obligations": [
            "Replace empirical eta safety uplift by explicit sign-sensitive offdiag inequality constants.",
            "Prove base-uniform eta_+(x;W) <= C_eta(log x)^A_eta for all x>=x0 over W in {30,210,2310,30030}.",
            "Bind E2(x;W)/x analytically from Lemma A/B ingredients, independent of sampled grids.",
            "Derive asymptotic x0 and final C_H,A_H from theorem constants (not fitted envelopes).",
        ],
        "dependency_links": {
            "depends_on": ["Lemma A", "Lemma B", "O2 tail closure"],
            "upstream_artifacts": [args.a3_primary, args.a3_stress, args.eta_probe, args.eta4_just],
            "downstream_use": ["A4 base-uniform asymptotic constants", "RH endpoint implication O5"],
        },
    }

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "a3_primary": args.a3_primary,
            "a3_stress": args.a3_stress,
            "eta_probe": args.eta_probe,
            "eta4_just": args.eta4_just,
        },
        "lemma_skeleton": lemma,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    fc = lemma["frozen_constants"]
    ev = lemma["empirical_validation"]
    lines = [
        "# A3 Analytic-Closure Lemma Skeleton",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Candidate Statement",
        "",
        lemma["statement_template"],
        "",
        "## Proof-Primary Branch",
        "",
        f"- `{lemma['proof_primary_branch']}`",
        "",
        "## Frozen Constants",
        "",
        f"- `A_eta`: `{fc['A_eta']}`",
        f"- `C_eta_uplifted`: `{fc['C_eta_uplifted']}`",
        f"- `A_H_primary`: `{fc['A_H_primary']}`",
        f"- `C_H_primary`: `{fc['C_H_primary']}`",
        f"- `A_H_stress`: `{fc['A_H_stress']}`",
        f"- `C_H_stress`: `{fc['C_H_stress']}`",
        f"- `eta_safety`: `{fc['eta_safety']}`",
        f"- `m_zero`: `{fc['m_zero']}`",
        f"- `zero_kernel`: `{fc['zero_kernel']}`",
        f"- `kernel_scale`: `{fc['kernel_scale']}`",
        f"- `bases`: `{fc['bases']}`",
        "",
        "## Validation Snapshot",
        "",
        f"- `primary_train_holds`: `{ev['primary_train_holds']}`",
        f"- `primary_valid_holds`: `{ev['primary_valid_holds']}`",
        f"- `primary_valid_ratio_max`: `{ev['primary_valid_ratio_max']:.12g}`",
        f"- `primary_valid_max_gap_h_minus_rhs`: `{ev['primary_valid_max_gap_h_minus_rhs']:.12g}`",
        f"- `stress_valid_holds`: `{ev['stress_valid_holds']}`",
        f"- `stress_valid_ratio_max`: `{ev['stress_valid_ratio_max']:.12g}`",
        f"- `stress_valid_max_gap_h_minus_rhs`: `{ev['stress_valid_max_gap_h_minus_rhs']:.12g}`",
        f"- `eta4_just_valid_ratio_over_c_eta`: `{ev['eta4_just_valid_ratio_over_c_eta']:.12g}`",
        "",
        "## Analytic Obligations (O3)",
        "",
    ]
    for item in lemma["analytic_obligations"]:
        lines.append(f"- {item}")

    lines += ["", "## Sources", ""]
    for src in lemma["dependency_links"]["upstream_artifacts"]:
        lines.append(f"- `{src}`")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
