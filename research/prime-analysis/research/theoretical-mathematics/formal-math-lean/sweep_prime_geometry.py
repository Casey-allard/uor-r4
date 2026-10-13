#!/usr/bin/env python3
"""Systematic sweep for prime modular geometry experiments."""

from __future__ import annotations

import argparse
import itertools
import json
import os
from datetime import datetime, timezone
from typing import Dict, List

from prime_geometry_loop import ExperimentConfig, parse_floats, parse_moduli, run_experiment


def write_heatmap_svg(rows: List[Dict[str, object]], out_path: str) -> None:
    if not rows:
        return

    moduli_labels = sorted({r["moduli_label"] for r in rows})
    radius_values = sorted({r["radius_power"] for r in rows})

    width = 1180
    height = 70 + len(moduli_labels) * 28 + 60
    left = 250
    top = 50
    cell_w = max(55, (width - left - 30) // max(1, len(radius_values)))
    cell_h = 22

    data = {(r["moduli_label"], r["radius_power"]): r["nearest_centroid_accuracy"] for r in rows}

    def color_for(v: float) -> str:
        # map approx [0.5, 0.8] -> red..green
        t = max(0.0, min(1.0, (v - 0.5) / 0.3))
        r = int((1.0 - t) * 214 + t * 39)
        g = int((1.0 - t) * 89 + t * 174)
        b = int((1.0 - t) * 89 + t * 96)
        return f"#{r:02x}{g:02x}{b:02x}"

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">\n')
        f.write(f'<rect x="0" y="0" width="{width}" height="{height}" fill="#fbfaf8"/>\n')
        f.write('<text x="24" y="30" font-size="18" fill="#111827">Sweep Heatmap: Nearest-Centroid Accuracy</text>\n')

        for j, rp in enumerate(radius_values):
            x = left + j * cell_w
            f.write(f'<text x="{x + 6}" y="44" font-size="11" fill="#374151">r={rp:g}</text>\n')

        for i, ml in enumerate(moduli_labels):
            y = top + i * cell_h
            f.write(f'<text x="16" y="{y + 15}" font-size="10" fill="#374151">{ml}</text>\n')
            for j, rp in enumerate(radius_values):
                x = left + j * cell_w
                v = data.get((ml, rp), 0.5)
                color = color_for(v)
                f.write(f'<rect x="{x}" y="{y}" width="{cell_w - 2}" height="{cell_h - 2}" fill="{color}"/>\n')
                f.write(f'<text x="{x + 4}" y="{y + 14}" font-size="10" fill="#ffffff">{v:.3f}</text>\n')

        f.write("</svg>\n")


def main() -> None:
    parser = argparse.ArgumentParser(description="Sweep modulus/radius configs for prime geometry")
    parser.add_argument("--n-max", type=int, default=80000)
    parser.add_argument("--candidate-moduli", type=str, default="5,7,11,13,17,19")
    parser.add_argument("--choose", type=int, default=5)
    parser.add_argument("--radius-powers", type=str, default="1.3,1.5,1.7,1.9,2.1")
    parser.add_argument("--control-trials", type=int, default=40)
    parser.add_argument("--seed", type=int, default=1729)
    parser.add_argument("--output", type=str, default="")
    args = parser.parse_args()

    candidates = parse_moduli(args.candidate_moduli)
    radii = parse_floats(args.radius_powers)

    rows: List[Dict[str, object]] = []
    for combo_idx, combo in enumerate(itertools.combinations(candidates, args.choose)):
        for r_idx, rp in enumerate(radii):
            cfg = ExperimentConfig(
                n_max=args.n_max,
                moduli=list(combo),
                radius_power=rp,
                bins=36,
                control_trials=max(0, args.control_trials),
                control_seed=args.seed + combo_idx * 100 + r_idx,
            )
            report = run_experiment(cfg)
            metrics = report["metrics"]
            lp = report["controls"].get("label_permutation", {}).get("accuracy", {}) if report.get("controls") else {}
            zp = report["controls"].get("zeta_permutation", {}) if report.get("controls") else {}

            entry = {
                "moduli": list(combo),
                "moduli_label": "-".join(str(x) for x in combo),
                "radius_power": rp,
                "separation_ratio": metrics["separation_ratio"],
                "nearest_centroid_accuracy": metrics["nearest_centroid_accuracy"],
                "entropy_delta": metrics["entropy_delta"],
                "zeta_alignment_score": metrics["zeta_alignment_score"],
                "accuracy_p_value_ge": lp.get("p_value_ge", 1.0),
                "zeta_p_value_ge": zp.get("p_value_ge", 1.0),
            }
            # Higher is better. Penalize weak significance.
            entry["rank_score"] = (
                entry["nearest_centroid_accuracy"]
                * max(0.0, entry["separation_ratio"])
                * (1.0 - min(1.0, entry["accuracy_p_value_ge"]))
            )
            rows.append(entry)

    rows.sort(key=lambda r: r["rank_score"], reverse=True)

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_path = args.output or f"research/output/sweep_report_{ts}.json"
    os.makedirs(os.path.dirname(out_path), exist_ok=True)

    summary = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "candidate_moduli": candidates,
            "choose": args.choose,
            "radius_powers": radii,
            "control_trials": args.control_trials,
            "seed": args.seed,
        },
        "top": rows[:12],
        "all": rows,
    }

    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2)

    tsv_path = out_path.replace(".json", ".tsv")
    with open(tsv_path, "w", encoding="utf-8") as f:
        f.write("moduli\tradius_power\taccuracy\tseparation\tacc_p\tzeta_p\trank_score\n")
        for r in rows:
            f.write(
                f"{r['moduli_label']}\t{r['radius_power']}\t{r['nearest_centroid_accuracy']:.6f}\t"
                f"{r['separation_ratio']:.6f}\t{r['accuracy_p_value_ge']:.6f}\t{r['zeta_p_value_ge']:.6f}\t{r['rank_score']:.6f}\n"
            )

    heatmap_path = out_path.replace(".json", "_heatmap.svg")
    write_heatmap_svg(rows, heatmap_path)

    print(f"wrote: {out_path}")
    print(f"wrote: {tsv_path}")
    print(f"wrote: {heatmap_path}")
    print("top_results:")
    for r in rows[:8]:
        print(
            f"  moduli={r['moduli_label']} radius={r['radius_power']:.2f} "
            f"acc={r['nearest_centroid_accuracy']:.6f} sep={r['separation_ratio']:.6f} "
            f"acc_p={r['accuracy_p_value_ge']:.4f} zeta_p={r['zeta_p_value_ge']:.4f}"
        )


if __name__ == "__main__":
    main()
