#!/usr/bin/env python3
"""Lemma A checker: channel identities from (F_M, F'_M) coordinates."""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence

import spinning_top_r4_candidate_a as s4
from prime_geometry_loop import load_zeta_zeros_file


def clamp(x: float, lo: float, hi: float) -> float:
    return lo if x < lo else (hi if x > hi else x)


def max_abs_diff(a: Sequence[float], b: Sequence[float]) -> float:
    if not a or len(a) != len(b):
        return 0.0
    return max(abs(x - y) for x, y in zip(a, b))


def derived_channels(points, seq_idx: Sequence[int]) -> Dict[str, List[float]]:
    theta = []
    frs = []
    fis = []
    grs = []
    gis = []
    for n in seq_idx:
        fr, fi, gr, gi = points[n - 2]
        theta.append(math.atan2(fi, fr))
        frs.append(fr)
        fis.append(fi)
        grs.append(gr)
        gis.append(gi)
    if len(theta) < 3:
        return {"Y11s": [], "Y20": [], "su2_trace": [], "phase_vel": []}

    m = len(theta) - 2
    c1 = math.sqrt(3.0 / (4.0 * math.pi))
    c2 = math.sqrt(5.0 / (16.0 * math.pi))
    y11s = []
    y20 = []
    phase_vel = []
    su2_trace = []
    eps = 1e-18
    for j in range(m):
        r1 = max(eps, math.hypot(frs[j], fis[j]))
        r2 = max(eps, math.hypot(grs[j], gis[j]))
        cos_phi2 = clamp(grs[j] / r2, -1.0, 1.0)

        # chi_j = 0.5*(atan2(gi,gr)+pi)
        # sin(chi_j)=sqrt((1+cos(phi2_j))/2), cos^2(chi_j)=(1-cos(phi2_j))/2
        sin_chi = math.sqrt(max(0.0, 0.5 * (1.0 + cos_phi2)))
        cos2_chi = 0.5 * (1.0 - cos_phi2)
        sin_theta = fis[j] / r1

        y11s.append(c1 * sin_chi * sin_theta)
        y20.append(c2 * (3.0 * cos2_chi - 1.0))

        dtheta = s4.unwrap_delta(theta[j], theta[j + 1])
        phase_vel.append(dtheta)
        # Tr(U_j) real part under su2_step parametrization equals 2*cos(dtheta/2)
        su2_trace.append(2.0 * math.cos(0.5 * dtheta))

    return {
        "Y11s": y11s,
        "Y20": y20,
        "su2_trace": su2_trace,
        "phase_vel": phase_vel,
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Lemma A channel derivation checker")
    ap.add_argument("--bases", type=str, default="30,210,2310,30030")
    ap.add_argument("--n-max", type=int, default=1000000)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=64)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--zero-kernel", type=str, default="none", choices=["none", "gaussian", "lorentz"])
    ap.add_argument("--kernel-scale", type=float, default=0.0)
    ap.add_argument("--output", type=str, default="research/output/lemma_a_channel_derivation_check.json")
    args = ap.parse_args()

    bases = [int(x.strip()) for x in args.bases.split(",") if x.strip()]
    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    points = s4.manifold_points(args.n_max, zeros, args.u_mode, args.zero_kernel, args.kernel_scale)
    per_base = []
    worst = {
        "Y11s": 0.0,
        "Y20": 0.0,
        "su2_trace": 0.0,
        "phase_vel": 0.0,
    }
    for b in bases:
        idx = [n for n in range(2, args.n_max + 1) if s4.gcd(n, b) == 1]
        ref = s4.candidate_series_for_base(points, idx)
        der = derived_channels(points, idx)
        m = min(len(ref["Y11s"]), len(der["Y11s"]))
        diffs = {
            "Y11s": max_abs_diff(ref["Y11s"][:m], der["Y11s"][:m]),
            "Y20": max_abs_diff(ref["Y20"][:m], der["Y20"][:m]),
            "su2_trace": max_abs_diff(ref["su2_trace"][:m], der["su2_trace"][:m]),
            "phase_vel": max_abs_diff(ref["phase_vel"][:m], der["phase_vel"][:m]),
        }
        for k in worst:
            worst[k] = max(worst[k], diffs[k])
        per_base.append({"base": b, "sample_count": m, "max_abs_diff": diffs})

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "bases": bases,
            "n_max": args.n_max,
            "max_zeta_zeros": len(zeros),
            "u_mode": args.u_mode,
            "zero_kernel": args.zero_kernel,
            "kernel_scale": args.kernel_scale,
        },
        "identities": {
            "Y11s": "sqrt(3/(4*pi))*sqrt((1+gr/sqrt(gr^2+gi^2))/2)*(fi/sqrt(fr^2+fi^2))",
            "Y20": "sqrt(5/(16*pi))*(3*((1-gr/sqrt(gr^2+gi^2))/2)-1)",
            "phase_vel": "unwrap(arg(F_{n+1})-arg(F_n))",
            "su2_trace": "2*cos(phase_vel/2)",
        },
        "per_base": per_base,
        "worst_case_max_abs_diff": worst,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Lemma A Channel Derivation Check\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write("## Identities Checked\n\n")
        for k, v in report["identities"].items():
            f.write(f"- `{k}`: `{v}`\n")
        f.write("\n## Worst-Case Max Abs Differences\n\n")
        for k, v in worst.items():
            f.write(f"- `{k}`: {v:.3e}\n")
        f.write("\n## Per Base\n\n")
        for row in per_base:
            f.write(f"- base={row['base']} sample_count={row['sample_count']}")
            for k in ["Y11s", "Y20", "su2_trace", "phase_vel"]:
                f.write(f" {k}={row['max_abs_diff'][k]:.3e}")
            f.write("\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")


if __name__ == "__main__":
    main()

