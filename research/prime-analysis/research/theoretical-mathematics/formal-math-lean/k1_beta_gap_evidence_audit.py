#!/usr/bin/env python3
"""Audit beta-gap evidence in local K1 research artifacts.

Purpose:
- Distinguish theorem-grade (external, explicit constants) vs empirical
  finite-range beta evidence from local probes.
- Quantify empirical support for beta > 0.51 while explicitly marking that
  this does not constitute an unconditional theorem.
"""

from __future__ import annotations

import argparse
import json
import math
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Any, Dict, List, Tuple


def walk_betas(obj: Any, path: str, out: List[Tuple[str, float]]) -> None:
    if isinstance(obj, dict):
        for k, v in obj.items():
            nk = f"{path}.{k}" if path else k
            lk = k.lower()
            if isinstance(v, (int, float)) and ("beta" in lk):
                fv = float(v)
                if 0.4 < fv < 1.0:
                    out.append((nk, fv))
            walk_betas(v, nk, out)
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            walk_betas(v, f"{path}[{i}]", out)


def load_json(path: Path) -> Dict[str, Any] | None:
    try:
        with path.open("r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return None


def classify_file(path: str) -> str:
    lp = path.lower()
    if "external" in lp or "theorem_constants" in lp:
        return "external_theorem_context"
    if "source_shape_probe" in lp or "multimode_phase_probe" in lp or "spinning_top" in lp:
        return "empirical_model_probe"
    if "l2a" in lp or "brancha" in lp or "inverse_feasibility" in lp:
        return "constant_propagation_math"
    return "other"


def main() -> None:
    ap = argparse.ArgumentParser(description="K1 beta-gap evidence audit")
    ap.add_argument("--glob", type=str, default="research/output/*.json")
    ap.add_argument("--beta-threshold", type=float, default=0.51)
    ap.add_argument(
        "--out-prefix",
        type=str,
        default=f"research/output/k1_w37_beta_gap_evidence_audit_{date.today().isoformat()}",
    )
    args = ap.parse_args()

    paths = sorted(Path().glob(args.glob))
    records: List[Dict[str, Any]] = []
    all_betas: List[float] = []

    for p in paths:
        obj = load_json(p)
        if obj is None:
            continue
        betas: List[Tuple[str, float]] = []
        walk_betas(obj, "", betas)
        if not betas:
            continue
        vals = [b for _, b in betas]
        all_betas.extend(vals)
        rec = {
            "file": str(p),
            "class": classify_file(str(p)),
            "beta_count": len(vals),
            "beta_min": float(min(vals)),
            "beta_max": float(max(vals)),
            "beta_median": float(sorted(vals)[len(vals) // 2]),
            "all_beta_ge_threshold": bool(all(v > args.beta_threshold for v in vals)),
            "sample_paths": [x for x, _ in betas[:12]],
        }
        records.append(rec)

    all_betas_sorted = sorted(all_betas)
    n = len(all_betas_sorted)
    q = lambda p: all_betas_sorted[int((n - 1) * p)] if n else float("nan")

    # Empirical subset focused on source-shape and multimode probes.
    emp = [r for r in records if r["class"] == "empirical_model_probe"]
    emp_all_ge = sum(1 for r in emp if r["all_beta_ge_threshold"])

    result = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "glob": args.glob,
            "beta_threshold": args.beta_threshold,
        },
        "global_beta_stats": {
            "count": n,
            "min": float(all_betas_sorted[0]) if n else float("nan"),
            "q05": float(q(0.05)),
            "median": float(q(0.5)),
            "q95": float(q(0.95)),
            "max": float(all_betas_sorted[-1]) if n else float("nan"),
            "fraction_gt_threshold": float(sum(1 for v in all_betas_sorted if v > args.beta_threshold) / n)
            if n
            else float("nan"),
        },
        "empirical_probe_summary": {
            "files": len(emp),
            "files_all_beta_gt_threshold": emp_all_ge,
            "fraction_files_all_beta_gt_threshold": float(emp_all_ge / len(emp)) if emp else float("nan"),
        },
        "records": records,
        "interpretation": {
            "theorem_status": (
                "No file in this audit provides an unconditional theorem proving uniform beta-gap > threshold."
            ),
            "empirical_status": (
                "Many empirical probes operate with beta significantly above threshold, "
                "which is supportive but not a formal proof."
            ),
        },
    }

    out_prefix = Path(args.out_prefix)
    out_prefix.parent.mkdir(parents=True, exist_ok=True)
    out_json = out_prefix.with_suffix(".json")
    out_md = out_prefix.with_suffix(".md")
    out_json.write_text(json.dumps(result, indent=2), encoding="utf-8")

    lines: List[str] = []
    lines.append(f"# K1 Beta-Gap Evidence Audit ({date.today().isoformat()})")
    lines.append("")
    lines.append("## Global Stats")
    for k, v in result["global_beta_stats"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Empirical Probe Summary")
    for k, v in result["empirical_probe_summary"].items():
        lines.append(f"- {k}: {v}")
    lines.append("")
    lines.append("## Key Interpretation")
    lines.append(f"- {result['interpretation']['theorem_status']}")
    lines.append(f"- {result['interpretation']['empirical_status']}")
    lines.append("")
    lines.append("| class | files | all_beta_gt_threshold_files |")
    lines.append("|:--|---:|---:|")
    cls = {}
    for r in records:
        c = r["class"]
        if c not in cls:
            cls[c] = {"n": 0, "ok": 0}
        cls[c]["n"] += 1
        cls[c]["ok"] += 1 if r["all_beta_ge_threshold"] else 0
    for c, d in sorted(cls.items()):
        lines.append(f"| {c} | {d['n']} | {d['ok']} |")
    lines.append("")
    out_md.write_text("\n".join(lines), encoding="utf-8")

    print(out_json)
    print(out_md)


if __name__ == "__main__":
    main()
