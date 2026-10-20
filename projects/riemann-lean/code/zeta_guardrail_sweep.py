#!/usr/bin/env python3
"""Zeta-first sweep: rank embeddings by zeta-guardrail stability."""

from __future__ import annotations

import argparse
import itertools
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List

from prime_geometry_loop import (
    ExperimentConfig,
    load_zeta_zeros_file,
    parse_floats,
    run_experiment,
)


def parse_family_sets(text: str) -> List[List[int]]:
    out = []
    for chunk in text.split(";"):
        chunk = chunk.strip()
        if not chunk:
            continue
        vals = [int(x.strip()) for x in chunk.split(",") if x.strip()]
        vals = [v for v in vals if v > 1]
        if vals:
            out.append(vals)
    return out


def main() -> None:
    p = argparse.ArgumentParser(description="Sweep zeta-guardrail quality across embeddings")
    p.add_argument("--n-values", type=str, default="80000,120000,200000")
    p.add_argument("--moduli-families", type=str, default="30;210;2310;30,210;5,7,11,13,19")
    p.add_argument("--radius-powers", type=str, default="1.1,1.3,1.5,1.7")
    p.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    p.add_argument("--max-zeta-zeros", type=int, default=128)
    p.add_argument("--control-trials", type=int, default=40)
    p.add_argument("--seed", type=int, default=20260216)
    p.add_argument("--output", type=str, default="research/output/zeta_guardrail_sweep.json")
    args = p.parse_args()

    n_values = [int(x) for x in args.n_values.split(",") if x.strip()]
    radii = parse_floats(args.radius_powers)
    families = parse_family_sets(args.moduli_families)
    zeros = load_zeta_zeros_file(args.zeta_zeros_file) if args.zeta_zeros_file else []

    rows = []
    run_id = 0
    for fam in families:
        for rp in radii:
            scores = []
            pvals = []
            zscores = []
            for n in n_values:
                cfg = ExperimentConfig(
                    n_max=n,
                    moduli=fam,
                    radius_power=rp,
                    bins=36,
                    control_trials=max(0, args.control_trials),
                    control_seed=args.seed + run_id,
                    zeta_zeros_imag=zeros,
                    max_zeta_zeros=max(0, args.max_zeta_zeros),
                )
                rep = run_experiment(cfg)
                zc = rep.get("controls", {}).get("zeta_permutation", {})
                scores.append(float(rep["metrics"]["zeta_alignment_score"]))
                pvals.append(float(zc.get("p_value_ge", 1.0)))
                zscores.append(float(zc.get("z_score", 0.0)))
                run_id += 1

            mean_score = sum(scores) / len(scores)
            mean_p = sum(pvals) / len(pvals)
            mean_z = sum(zscores) / len(zscores)
            var_p = sum((x - mean_p) ** 2 for x in pvals) / len(pvals)
            var_z = sum((x - mean_z) ** 2 for x in zscores) / len(zscores)

            # Higher better: low p, high z, stable across N.
            guardrail_rank = (1.0 - mean_p) * max(0.0, mean_z) / (1.0 + math.sqrt(var_p + var_z))

            rows.append(
                {
                    "moduli": fam,
                    "moduli_label": "-".join(str(x) for x in fam),
                    "radius_power": rp,
                    "n_values": n_values,
                    "zeta_score_mean": mean_score,
                    "zeta_p_mean": mean_p,
                    "zeta_z_mean": mean_z,
                    "zeta_p_var": var_p,
                    "zeta_z_var": var_z,
                    "guardrail_rank": guardrail_rank,
                    "per_n": [
                        {"n_max": n, "score": s, "p": pv, "z": zv}
                        for n, s, pv, zv in zip(n_values, scores, pvals, zscores)
                    ],
                }
            )

    rows.sort(key=lambda r: r["guardrail_rank"], reverse=True)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_values": n_values,
            "families": families,
            "radius_powers": radii,
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": args.max_zeta_zeros,
            "control_trials": args.control_trials,
        },
        "top": rows[:12],
        "all": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Zeta Guardrail Sweep\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for i, r in enumerate(report["top"][:8], start=1):
            f.write(
                f"{i}. moduli={r['moduli_label']} r={r['radius_power']} rank={r['guardrail_rank']:.4f} "
                f"p_mean={r['zeta_p_mean']:.4f} z_mean={r['zeta_z_mean']:.4f}\n"
            )

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    if rows:
        b = rows[0]
        print("best", b["moduli_label"], "r", b["radius_power"], "p_mean", round(b["zeta_p_mean"], 6), "z_mean", round(b["zeta_z_mean"], 6))


if __name__ == "__main__":
    main()
