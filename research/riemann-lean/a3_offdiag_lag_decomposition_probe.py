#!/usr/bin/env python3
"""Lag-decomposition probe for sign-sensitive offdiag structure."""

from __future__ import annotations

import argparse
import json
import math
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List


def parse_floats(raw: str) -> List[float]:
    return [float(x.strip()) for x in raw.split(",") if x.strip()]


def main() -> None:
    ap = argparse.ArgumentParser(description="A3 offdiag lag decomposition probe")
    ap.add_argument(
        "--a3-json",
        default="research/output/a3_offdiag_dynamic_majorant_eta4p0_sf3_stress_2026-02-17.json",
    )
    ap.add_argument(
        "--lag-bands",
        default="1,2,4,8,16,32,64,128,256,512,1024",
        help="band edges in event-index lag",
    )
    ap.add_argument(
        "--output",
        default="research/output/a3_offdiag_lag_decomposition_probe_2026-02-17.md",
    )
    args = ap.parse_args()

    a3 = json.loads(Path(args.a3_json).read_text(encoding="utf-8"))
    # This probe uses the fitted constants and reports a symbolic band plan rather than recomputing g-series.
    # It is a planning scaffold for the next sign-sensitive theorem run.
    bands = sorted(set(int(v) for v in parse_floats(args.lag_bands)))
    if not bands:
        bands = [1, 2, 4, 8, 16, 32]

    # Proxy weights from geometric lag attenuation to prioritize next analytic targets.
    # This is intentionally conservative and only used for decomposition planning.
    w = [math.exp(-math.sqrt(float(b))) for b in bands]
    ws = sum(w)
    if ws <= 0.0:
        w = [1.0 / len(bands) for _ in bands]
    else:
        w = [v / ws for v in w]

    lines = [
        "# A3 Offdiag Lag Decomposition Probe",
        "",
        f"Generated: {datetime.now(timezone.utc).isoformat()}",
        f"Source A3 artifact: `{args.a3_json}`",
        "",
        "This is a sign-sensitive decomposition planning scaffold.",
        "It prioritizes lag bands for the next analytic bound iteration.",
        "",
        "## Current Anchor Constants",
        "",
        f"- `A_eta = {a3['eta_envelope']['A_eta']}`",
        f"- `C_eta = {a3['eta_envelope']['C_eta_uplifted']}`",
        f"- `A_H = {a3['h_transfer_envelope']['A_H']}`",
        f"- `C_H = {a3['h_transfer_envelope']['C_H_from_density_transfer']}`",
        "",
        "## Proposed Lag Bands (Priority Weights)",
        "",
        "| Lag band edge | Priority weight |",
        "|---:|---:|",
    ]
    for b, ww in zip(bands, w):
        lines.append(f"| {b} | {ww:.6f} |")

    lines += [
        "",
        "## Next Analytic Targets",
        "",
        "1. Bound positive offdiag in low-lag bands with explicit oscillatory cancellation constants.",
        "2. Bound high-lag bands via coarse absolute control plus decay factor.",
        "3. Recompose to a new symbolic `C_eta` candidate and compare to calibrated budget.",
        "",
    ]

    out = Path(args.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(json.dumps({"md": str(out)}, indent=2))


if __name__ == "__main__":
    main()
