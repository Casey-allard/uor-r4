#!/usr/bin/env python3
"""Build a structured A3 eta-lemma scaffold from current artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path


def read_json(path: str):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="Generate A3 eta-lemma scaffold memo")
    ap.add_argument(
        "--a3",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3.json",
    )
    ap.add_argument(
        "--stress",
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
        default="research/output/a3_eta_lemma_scaffold_2026-02-17.md",
    )
    args = ap.parse_args()

    a3 = read_json(args.a3)
    stress = read_json(args.stress)
    eta_probe = read_json(args.eta_probe)
    eta4j = read_json(args.eta4_just)

    a_eta = float(a3["eta_envelope"]["A_eta"])
    c_eta = float(a3["eta_envelope"]["C_eta_uplifted"])
    a_h = float(a3["h_transfer_envelope"]["A_H"])
    c_h = float(a3["h_transfer_envelope"]["C_H_from_density_transfer"])
    stress_ah = float(stress["h_transfer_envelope"]["A_H"])
    stress_ch = float(stress["h_transfer_envelope"]["C_H_from_density_transfer"])
    safety_needed = float(eta_probe["recommended_under_safety_3p0"]["safety_needed_for_valid"])
    valid_ratio = float(eta4j["fit_summary"]["valid_ratio_over_C_eta"])

    lines = [
        "# A3 Eta Lemma Scaffold",
        "",
        f"Generated: {datetime.now(timezone.utc).isoformat()}",
        "",
        "## Candidate Lemma Statement",
        "",
        "There exist constants `C_eta, x0` such that for all `x >= x0` and all tested bases `W`:",
        "",
        "`eta_+(x;W) <= C_eta * (log x)^4`.",
        "",
        "With this bound and deterministic identity",
        "",
        "`|H(x;W)| <= sqrt((1 + eta_+(x;W)) * E2(x;W) / x)`,",
        "",
        "derive",
        "",
        "`|H(x;W)| <= C_H * (log x)^{A_H}`.",
        "",
        "## Current Calibrated Constants",
        "",
        f"- Base branch (`{args.a3}`): `A_eta={a_eta:.6g}`, `C_eta={c_eta:.12g}`, `A_H={a_h:.6g}`, `C_H={c_h:.12g}`",
        f"- Stress branch (`{args.stress}`): `A_H={stress_ah:.6g}`, `C_H={stress_ch:.12g}`",
        f"- Fixed-`A_eta` probe suggests minimum safety near `{safety_needed:.6g}` for held-out validity (`{args.eta_probe}`).",
        f"- `A_eta=4` justification run has held-out ratio `{valid_ratio:.6g}` under safety 3.0 (`{args.eta4_just}`).",
        "",
        "## Proof Obligations Remaining",
        "",
        "1. Replace empirical safety factor by symbolic constants from an explicit offdiag bound.",
        "2. Prove base-uniform control over positive offdiag ratio for all `x >= x0`.",
        "3. Bound `E2(x)/x` analytically in the same regime to avoid calibration dependence.",
        "4. Convert finite-grid verification into asymptotic implication with explicit `x0` and constants.",
        "",
        "## Practical Next Action",
        "",
        "Draft an analytic inequality chain for offdiag terms using weighted bilinear decomposition and compare resulting symbolic constants against the calibrated `C_eta` budget above.",
        "",
    ]

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines), encoding="utf-8")
    print(json.dumps({"md": str(out)}, indent=2))


if __name__ == "__main__":
    main()
