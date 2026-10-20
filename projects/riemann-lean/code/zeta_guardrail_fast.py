#!/usr/bin/env python3
"""Fast zeta-first search: rank fast, validate top-K only."""

from __future__ import annotations

import argparse
import json
import os
from datetime import datetime, timezone

from prime_geometry_loop import ExperimentConfig, load_zeta_zeros_file, parse_floats, run_experiment


def parse_families(text: str):
    fams = []
    for c in text.split(";"):
        c = c.strip()
        if not c:
            continue
        vals = [int(x.strip()) for x in c.split(",") if x.strip()]
        vals = [v for v in vals if v > 1]
        if vals:
            fams.append(vals)
    return fams


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--n", type=int, default=120000)
    ap.add_argument("--families", type=str, default="30;210;2310;30,210;5,7,11,13,19")
    ap.add_argument("--radii", type=str, default="1.1,1.3")
    ap.add_argument("--top-k", type=int, default=3)
    ap.add_argument("--control-trials", type=int, default=8)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--output", type=str, default="research/output/zeta_guardrail_fast.json")
    args = ap.parse_args()

    fams = parse_families(args.families)
    radii = parse_floats(args.radii)
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)

    # Stage 1: fast ranking (no controls)
    stage1 = []
    for fam in fams:
        for rp in radii:
            rep = run_experiment(
                ExperimentConfig(
                    n_max=args.n,
                    moduli=fam,
                    radius_power=rp,
                    bins=36,
                    control_trials=0,
                    zeta_zeros_imag=zeros,
                    max_zeta_zeros=args.max_zeta_zeros,
                )
            )
            stage1.append(
                {
                    "moduli": fam,
                    "moduli_label": "-".join(str(x) for x in fam),
                    "radius_power": rp,
                    "zeta_score": rep["metrics"]["zeta_alignment_score"],
                }
            )

    stage1.sort(key=lambda r: r["zeta_score"], reverse=True)
    top = stage1[: max(1, args.top_k)]

    # Stage 2: controls only on top-K
    stage2 = []
    for i, t in enumerate(top):
        rep = run_experiment(
            ExperimentConfig(
                n_max=args.n,
                moduli=t["moduli"],
                radius_power=t["radius_power"],
                bins=36,
                control_trials=max(0, args.control_trials),
                control_seed=20260216 + i,
                zeta_zeros_imag=zeros,
                max_zeta_zeros=args.max_zeta_zeros,
            )
        )
        zc = rep.get("controls", {}).get("zeta_permutation", {})
        stage2.append(
            {
                "moduli": t["moduli"],
                "moduli_label": t["moduli_label"],
                "radius_power": t["radius_power"],
                "zeta_score": rep["metrics"]["zeta_alignment_score"],
                "zeta_p": zc.get("p_value_ge", 1.0),
                "zeta_z": zc.get("z_score", 0.0),
            }
        )

    stage2.sort(key=lambda r: (r["zeta_p"], -r["zeta_z"], -r["zeta_score"]))

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n": args.n,
            "families": fams,
            "radii": radii,
            "top_k": args.top_k,
            "control_trials": args.control_trials,
        },
        "stage1": stage1,
        "stage2": stage2,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Zeta Guardrail Fast\n\n")
        for i, r in enumerate(stage2, 1):
            f.write(f"{i}. {r['moduli_label']} r={r['radius_power']} score={r['zeta_score']:.3f} p={r['zeta_p']:.6f} z={r['zeta_z']:.3f}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if stage2:
        b = stage2[0]
        print("best", b["moduli_label"], "r", b["radius_power"], "p", b["zeta_p"], "z", b["zeta_z"])


if __name__ == "__main__":
    main()
