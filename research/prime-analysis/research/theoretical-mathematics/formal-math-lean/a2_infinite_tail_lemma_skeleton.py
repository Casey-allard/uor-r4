#!/usr/bin/env python3
"""Build a theorem-ready O2 (A2) infinite-tail lemma skeleton from existing artifacts."""

from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict, List


def read_json(path: str) -> Dict[str, Any]:
    return json.loads(Path(path).read_text(encoding="utf-8"))


def main() -> None:
    ap = argparse.ArgumentParser(description="A2 infinite-tail lemma skeleton builder")
    ap.add_argument(
        "--a2-pack",
        default="research/output/a2_theorem_constant_pack_refresh_2026-02-17_sf3p5.json",
    )
    ap.add_argument(
        "--a2-infinite",
        default="research/output/a2_infinite_tail_uplift_2026-02-17_n_diff_explicit_hsw2022_sf3p5.json",
    )
    ap.add_argument(
        "--citation-label",
        default="HSW2021",
        help="Short label for explicit zero-count bound source.",
    )
    ap.add_argument(
        "--citation-url",
        default="https://arxiv.org/abs/2107.06506",
        help="Source URL for explicit zero-count bound assumptions.",
    )
    ap.add_argument(
        "--output",
        default="research/output/a2_infinite_tail_lemma_skeleton_2026-02-17_explicit_density.json",
    )
    args = ap.parse_args()

    pack = read_json(args.a2_pack)
    inf = read_json(args.a2_infinite)

    cfg = inf["config"]
    checks = inf["checks"]
    uplift = inf["uplift_constant"]
    tail = inf["tail_majorant"]

    m_grid = [int(m) for m in cfg["m_grid"]]
    tau_rows: List[Dict[str, Any]] = []
    for m in m_grid:
        row = tail["tau_by_M"][str(m)]
        tau_rows.append(
            {
                "M": m,
                "tau_finite_ref": float(row["tau_finite_ref"]),
                "tau_infinite_extra_majorant": float(row["tau_infinite_extra_majorant"]),
                "tau_infinite_majorant": float(row["tau_infinite_majorant"]),
            }
        )

    density_model = str(cfg.get("density_model", "affine"))
    theorem_assumptions: List[str] = []
    if density_model == "n_diff_explicit":
        c1 = float(cfg.get("nbound_c1", 0.0))
        c2 = float(cfg.get("nbound_c2", 0.0))
        c3 = float(cfg.get("nbound_c3", 0.0))
        h = float(cfg.get("nbound_h", 1.0))
        theorem_assumptions = [
            (
                f"[{args.citation_label}] Assume explicit zero-count error bound "
                f"|N(T)-M(T)| <= {c1} log(T) + {c2} log log(T) + {c3} for T>=e, "
                "where M(T)=T/(2pi)*log(T/(2pi e))."
            ),
            (
                "Define derivative majorant by finite difference: "
                "N'(t)_up := (M(t+h)-M(t) + E(t+h)+E(t))/h with h>0."
            ),
            (
                "Use N'(t)_up in the weighted tail integral for tau_infty(M), "
                "with kernel-weighted integrability and monotone decay in M."
            ),
        ]

    lemma = {
        "name": "Lemma B* (A2 Infinite-Tail Truncation)",
        "statement_template": (
            "For wheel family W and all x >= x0, M >= M0: "
            "Delta_M(x;W) <= C_delta * (log x)^beta * tau_infty(M), with tau_infty(M)->0."
        ),
        "frozen_constants": {
            "beta_fixed": float(cfg["beta_fixed"]),
            "C_delta_uplifted": float(uplift["C_delta_uplifted"]),
            "density_model": density_model,
            "density_a": float(cfg["density_a"]),
            "density_b": float(cfg["density_b"]),
            "rvm_c0": float(cfg.get("rvm_c0", 0.0)),
            "rvm_c1": float(cfg.get("rvm_c1", 0.0)),
            "nbound_c1": float(cfg.get("nbound_c1", 0.0)),
            "nbound_c2": float(cfg.get("nbound_c2", 0.0)),
            "nbound_c3": float(cfg.get("nbound_c3", 0.0)),
            "nbound_h": float(cfg.get("nbound_h", 1.0)),
            "kernel": cfg["kernel"],
            "kernel_scale": float(cfg["kernel_scale"]),
            "m_ref": int(cfg["m_ref"]),
            "m_grid": m_grid,
            "gamma_ref": float(tail["gamma_ref"]),
        },
        "theorem_assumptions": theorem_assumptions,
        "theorem_assumption_source": {
            "label": args.citation_label,
            "url": args.citation_url,
        },
        "empirical_validation": {
            "train_holds": bool(checks["train"]["holds"]),
            "valid_holds": bool(checks["valid"]["holds"]),
            "valid_ratio_max": float(checks["valid"]["ratio_max"]),
            "valid_max_gap_delta_minus_rhs": float(checks["valid"]["max_gap_delta_minus_rhs"]),
        },
        "tau_majorant_table": tau_rows,
        "analytic_obligations": [
            "State and cite theorem-side assumptions for explicit zero-count bound constants used in N'(t)_up.",
            "Prove sum-to-integral tail domination for weighted term t/sqrt(1/4+t^2) * K(t).",
            "Show tau_infty(M) is monotone nonincreasing in M and tends to 0 with explicit rate.",
            "Derive final C_delta, x0, M0 independent of finite sampled n-grid and m-grid.",
        ],
        "dependency_links": {
            "upstream_artifacts": [args.a2_pack, args.a2_infinite],
            "downstream_use": [
                "A1-A4 unified theorem pack",
                "RH assumption theorem draft (A2 obligation)",
                "Proof closure tracker O2",
            ],
        },
    }

    payload = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "inputs": {"a2_pack": args.a2_pack, "a2_infinite": args.a2_infinite},
        "lemma_skeleton": lemma,
    }

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    md = out.with_suffix(".md")
    lines = [
        "# A2 Infinite-Tail Lemma Skeleton",
        "",
        f"Generated: {payload['timestamp_utc']}",
        "",
        "## Candidate Statement",
        "",
        lemma["statement_template"],
        "",
        "## Frozen Constants",
        "",
    ]
    fc = lemma["frozen_constants"]
    for k in [
        "beta_fixed",
        "C_delta_uplifted",
        "density_model",
        "density_a",
        "density_b",
        "rvm_c0",
        "rvm_c1",
        "nbound_c1",
        "nbound_c2",
        "nbound_c3",
        "nbound_h",
        "kernel",
        "kernel_scale",
        "m_ref",
        "gamma_ref",
    ]:
        lines.append(f"- `{k}`: `{fc[k]}`")

    if lemma["theorem_assumptions"]:
        lines += [
            "",
            "## Theorem Assumptions",
            "",
            f"- source: `{lemma['theorem_assumption_source']['label']}` ({lemma['theorem_assumption_source']['url']})",
        ]
        for item in lemma["theorem_assumptions"]:
            lines.append(f"- {item}")

    lines += ["", "## Tau Majorant Table", "", "| M | tau_finite_ref | tau_infinite_extra_majorant | tau_infinite_majorant |", "|---:|---:|---:|---:|"]
    for r in lemma["tau_majorant_table"]:
        lines.append(
            f"| {r['M']} | {r['tau_finite_ref']:.12g} | {r['tau_infinite_extra_majorant']:.12g} | {r['tau_infinite_majorant']:.12g} |"
        )

    ev = lemma["empirical_validation"]
    lines += [
        "",
        "## Validation Snapshot",
        "",
        f"- `train_holds`: `{ev['train_holds']}`",
        f"- `valid_holds`: `{ev['valid_holds']}`",
        f"- `valid_ratio_max`: `{ev['valid_ratio_max']:.12g}`",
        f"- `valid_max_gap_delta_minus_rhs`: `{ev['valid_max_gap_delta_minus_rhs']:.12g}`",
        "",
        "## Analytic Obligations (O2)",
        "",
    ]
    for item in lemma["analytic_obligations"]:
        lines.append(f"- {item}")

    lines += ["", "## Sources", "", f"- `{args.a2_pack}`", f"- `{args.a2_infinite}`", ""]
    md.write_text("\n".join(lines), encoding="utf-8")

    print(json.dumps({"json": str(out), "md": str(md)}, indent=2))


if __name__ == "__main__":
    main()
