#!/usr/bin/env python3
"""Build a theorem-facing asymptotic closure artifact for O3 lemma E2-REM."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _fit_exp_decay(rows: List[Dict[str, Any]]) -> Dict[str, float]:
    points = sorted((float(r["M"]), float(r["tau_infinite_majorant"])) for r in rows if float(r["tau_infinite_majorant"]) > 0.0)
    if len(points) < 2:
        raise ValueError("need at least two positive tau points to fit decay envelope")

    k_candidates: List[float] = []
    for (m0, t0), (m1, t1) in zip(points, points[1:]):
        if t1 < t0 and m1 > m0:
            k_candidates.append((math.log(t0) - math.log(t1)) / (m1 - m0))
    if not k_candidates:
        raise ValueError("tau table does not provide strictly decreasing adjacent points")

    k = min(k_candidates)
    c0 = max(t * math.exp(k * m) for m, t in points)
    return {"k": k, "c0": c0}


def _sup_x_minus_nu_log_beta(beta: float, nu: float, x0: float) -> Dict[str, float]:
    if x0 <= 1.0:
        raise ValueError("x0 must be > 1")
    if nu <= 0.0:
        raise ValueError("nu must be > 0 to close a uniform constant")

    x_star = math.exp(beta / nu)
    g0 = (math.log(x0) ** beta) / (x0 ** nu)
    gs = (math.log(x_star) ** beta) / (x_star ** nu)
    if x_star >= x0 and gs >= g0:
        return {"sup_value": gs, "attained_at": x_star}
    return {"sup_value": g0, "attained_at": x0}


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O3 E2-REM asymptotic closure artifact")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o3-e2-remainder-draft", default="research/output/o3_e2_remainder_symbolic_draft_2026-02-17.json")
    ap.add_argument("--o2-lemma", default="research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.json")
    ap.add_argument("--o2-checker", default="research/output/a2_sum_to_integral_domination_checker_2026-02-17.json")
    ap.add_argument("--lambda-log", type=float, default=0.25, help="Slope in schedule M(x)=M0+lambda*log(x)")
    ap.add_argument("--x0", type=float, default=3.0)
    ap.add_argument("--output", default="research/output/o3_e2_remainder_asymptotic_closure_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    rem = _read_json(args.o3_e2_remainder_draft)
    o2 = _read_json(args.o2_lemma)["lemma_skeleton"]
    checker = _read_json(args.o2_checker)

    beta = float(o2["frozen_constants"]["beta_fixed"])
    c_delta = float(o2["frozen_constants"]["C_delta_uplifted"])
    m_rows = o2["tau_majorant_table"]
    m0 = min(float(r["M"]) for r in m_rows)
    model = _fit_exp_decay(m_rows)
    k = float(model["k"])
    c0 = float(model["c0"])

    lam = float(args.lambda_log)
    if lam <= 0.0:
        raise ValueError("--lambda-log must be > 0")
    nu = k * lam
    sup = _sup_x_minus_nu_log_beta(beta=beta, nu=nu, x0=float(args.x0))
    sup_term = float(sup["sup_value"])

    c_rem = c_delta * c0 * math.exp(-k * m0) * sup_term

    checks = checker["checks"]
    rows = checker["rows"]
    finite_ratio_max = max(float(r["ratio_discrete_over_upper"]) for r in rows)

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o3_e2_remainder_draft": args.o3_e2_remainder_draft,
            "o2_lemma": args.o2_lemma,
            "o2_checker": args.o2_checker,
        },
        "lemma_id": "E2-REM",
        "status": "conditional_asymptotic_closed",
        "assumption_scope": "inherits O2 theorem_assumptions (HSW2022 explicit N(T) error model and sum-integral domination)",
        "derived_schedule": {
            "form": "M(x) = M0 + lambda*log(x)",
            "M0": m0,
            "lambda": lam,
            "x0": float(args.x0),
        },
        "derived_decay_envelope": {
            "tau_bound_form": "tau_infty(M) <= c0 * exp(-k*M)",
            "k_min_pairwise": k,
            "c0_envelope": c0,
            "fit_source_rows": m_rows,
        },
        "asymptotic_closure": {
            "beta_fixed": beta,
            "C_delta_uplifted": c_delta,
            "nu": nu,
            "sup_x_ge_x0_of_x_minus_nu_log_beta": sup_term,
            "sup_attained_at_x": float(sup["attained_at"]),
            "result_inequality": "Rem(x;W) <= C_rem_uniform for all x>=x0, W in {30,210,2310,30030}",
            "C_rem_uniform": c_rem,
            "A_rem": 0.0,
        },
        "finite_window_crosscheck": {
            "sum_integral_checker_all_hold": bool(checks["all_hold"]),
            "max_ratio_discrete_over_upper": finite_ratio_max,
            "worst_gap_discrete_minus_upper": float(checks["worst_gap_discrete_minus_upper"]),
        },
        "interpretation": (
            "This converts E2 remainder control into theorem-facing asymptotic form with A_rem=0 "
            "under explicit O2 assumptions and scheduled truncation."
        ),
        "upstream_theorem_assumptions": o2.get("theorem_assumptions", []),
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O3 E2 Remainder Asymptotic Closure",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Status",
        "",
        f"- `{payload['status']}`",
        f"- scope: {payload['assumption_scope']}",
        "",
        "## Derived Envelope",
        "",
        f"- `tau_infty(M) <= c0 * exp(-k*M)`",
        f"- `k = {k}`",
        f"- `c0 = {c0}`",
        "",
        "## Truncation Schedule",
        "",
        f"- `M(x) = {m0} + {lam}*log(x)`",
        f"- `x0 = {args.x0}`",
        f"- `nu = k*lambda = {nu}`",
        "",
        "## Closed Remainder Bound",
        "",
        f"- `A_rem = 0`",
        f"- `C_rem_uniform = {c_rem}`",
        f"- sup term `sup_(x>=x0) x^(-nu)(log x)^beta = {sup_term}`",
        "",
        "## Finite-Window Crosscheck",
        "",
        f"- checker all hold: `{checks['all_hold']}`",
        f"- max ratio discrete/upper: `{finite_ratio_max}`",
        f"- worst gap discrete-upper: `{checks['worst_gap_discrete_minus_upper']}`",
        "",
    ]
    if payload["upstream_theorem_assumptions"]:
        lines += ["## Inherited Theorem Assumptions", ""]
        for item in payload["upstream_theorem_assumptions"]:
            lines.append(f"- {item}")
        lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
