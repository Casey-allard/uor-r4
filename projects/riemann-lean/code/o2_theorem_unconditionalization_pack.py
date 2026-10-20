#!/usr/bin/env python3
"""Build theorem-side O2 unconditionalization pack for the A2 infinite-tail lemma."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def _read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def _fit_exp_envelope(rows: List[Dict[str, Any]]) -> Dict[str, float]:
    pts = sorted((float(r["M"]), float(r["tau_infinite_majorant"])) for r in rows if float(r["tau_infinite_majorant"]) > 0.0)
    if len(pts) < 2:
        raise ValueError("need at least two positive tau points")
    k_vals: List[float] = []
    for (m0, t0), (m1, t1) in zip(pts, pts[1:]):
        if m1 > m0 and t1 < t0:
            k_vals.append((math.log(t0) - math.log(t1)) / (m1 - m0))
    if not k_vals:
        raise ValueError("cannot infer positive exponential decay rate from rows")
    k = min(k_vals)
    c0 = max(t * math.exp(k * m) for m, t in pts)
    return {"k_tau": k, "c0_tau": c0}


def main() -> None:
    ap = argparse.ArgumentParser(description="Build O2 theorem unconditionalization pack")
    ap.add_argument("--canonical-manifest", default="research/output/proof_canonical_manifest.json")
    ap.add_argument("--o2-lemma", default="research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.json")
    ap.add_argument("--o2-checker", default="research/output/a2_sum_to_integral_domination_checker_2026-02-17.json")
    ap.add_argument("--output", default="research/output/o2_theorem_unconditionalization_pack_2026-02-17.json")
    args = ap.parse_args()

    _ = _read_json(args.canonical_manifest)
    o2 = _read_json(args.o2_lemma)["lemma_skeleton"]
    chk = _read_json(args.o2_checker)

    rows = o2["tau_majorant_table"]
    rate = _fit_exp_envelope(rows)
    c = o2["frozen_constants"]
    src = o2.get("theorem_assumption_source", {})

    blockers = [
        "Replace interface theorem assumptions with fully proved zero-count error proposition in-project.",
        "Promote sum-to-integral domination checker into a formal asymptotic lemma writeup.",
        "Promote tau exponential envelope into a formal monotone-vanishing theorem statement.",
    ]

    payload: Dict[str, Any] = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {
            "canonical_manifest": args.canonical_manifest,
            "o2_lemma": args.o2_lemma,
            "o2_checker": args.o2_checker,
        },
        "o2_theorem_unconditionalization_pack": {
            "status": "theorem_draft_strengthened",
            "target_lemma": "Lemma B* (A2 Infinite-Tail Truncation)",
            "frozen_constants": {
                "beta_fixed": float(c["beta_fixed"]),
                "C_delta_uplifted": float(c["C_delta_uplifted"]),
                "gamma_ref": float(c["gamma_ref"]),
                "m_ref": int(c["m_ref"]),
            },
            "citation_lock": {
                "label": src.get("label", ""),
                "url": src.get("url", ""),
                "constants": {
                    "nbound_c1": float(c["nbound_c1"]),
                    "nbound_c2": float(c["nbound_c2"]),
                    "nbound_c3": float(c["nbound_c3"]),
                    "nbound_h": float(c["nbound_h"]),
                },
            },
            "sum_integral_domination": {
                "checker_all_hold": bool(chk["checks"]["all_hold"]),
                "max_ratio_discrete_over_upper": float(chk["checks"]["max_ratio_discrete_over_upper"]),
                "worst_gap_discrete_minus_upper": float(chk["checks"]["worst_gap_discrete_minus_upper"]),
            },
            "tau_monotone_vanishing_rate": {
                "envelope_form": "tau_infty(M) <= c0_tau * exp(-k_tau*M)",
                "k_tau": float(rate["k_tau"]),
                "c0_tau": float(rate["c0_tau"]),
            },
            "proof_chain_statement": (
                "Assuming citation-locked explicit N(T) error control, weighted sum-to-integral domination, and "
                "tau exponential decay, derive Delta_M(x;W) <= C_delta*(log x)^beta*tau_infty(M) with tau_infty(M)->0."
            ),
            "remaining_blockers": blockers,
        },
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# O2 Theorem Unconditionalization Pack",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        f"- status: `{payload['o2_theorem_unconditionalization_pack']['status']}`",
        "",
        "## Citation Lock",
        "",
        f"- `{payload['o2_theorem_unconditionalization_pack']['citation_lock']['label']}`",
        f"- `{payload['o2_theorem_unconditionalization_pack']['citation_lock']['url']}`",
        "",
        "## Sum-Integral Domination Snapshot",
        "",
        f"- checker all hold: `{payload['o2_theorem_unconditionalization_pack']['sum_integral_domination']['checker_all_hold']}`",
        f"- max ratio discrete/upper: `{payload['o2_theorem_unconditionalization_pack']['sum_integral_domination']['max_ratio_discrete_over_upper']}`",
        "",
        "## Tau Rate",
        "",
        f"- `k_tau = {payload['o2_theorem_unconditionalization_pack']['tau_monotone_vanishing_rate']['k_tau']}`",
        f"- `c0_tau = {payload['o2_theorem_unconditionalization_pack']['tau_monotone_vanishing_rate']['c0_tau']}`",
        "",
        "## Remaining Blockers",
        "",
    ]
    for b in blockers:
        lines.append(f"- {b}")
    lines.append("")
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
