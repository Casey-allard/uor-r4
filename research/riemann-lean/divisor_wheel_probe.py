#!/usr/bin/env python3
"""Probe divisor-neighborhood wheel effects and combine with modular geometry.

Compares prime detection quality from:
- modular geometry features
- divisor-neighborhood (tau around n) features
- combined feature set
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import ExperimentConfig, embed_4d, mean_vec, parse_moduli, sieve_primes


def divisor_count_sieve(n_max: int) -> List[int]:
    tau = [0] * (n_max + 1)
    for d in range(1, n_max + 1):
        for m in range(d, n_max + 1, d):
            tau[m] += 1
    return tau


def nearest_centroid_accuracy(features_a: Sequence[Sequence[float]], features_b: Sequence[Sequence[float]]) -> float:
    cp = mean_vec(features_a)
    cc = mean_vec(features_b)

    def pred(v: Sequence[float]) -> bool:
        dp = sum((v[i] - cp[i]) ** 2 for i in range(len(cp)))
        dc = sum((v[i] - cc[i]) ** 2 for i in range(len(cc)))
        return dp < dc

    correct = sum(1 for v in features_a if pred(v))
    correct += sum(1 for v in features_b if not pred(v))
    total = len(features_a) + len(features_b)
    return correct / total if total else 0.0


def linear_discriminant(features_a: Sequence[Sequence[float]], features_b: Sequence[Sequence[float]]) -> Dict[str, object]:
    cp = mean_vec(features_a)
    cc = mean_vec(features_b)
    w = [2.0 * (cp[i] - cc[i]) for i in range(len(cp))]
    b = sum(cc[i] * cc[i] - cp[i] * cp[i] for i in range(len(cp)))
    return {"w": w, "b": b, "rule": "prime-like if w·x + b > 0"}


def build_features(
    n_values: Sequence[int],
    tau: Sequence[int],
    moduli: Sequence[int],
    radius_power: float,
) -> Tuple[List[List[float]], List[List[float]], List[List[float]], List[List[float]], List[List[float]], List[List[float]]]:
    mod_prime = []
    mod_comp = []
    div_prime = []
    div_comp = []
    comb_prime = []
    comb_comp = []

    for n, is_p in n_values:
        # modular geometry: only stable direction components to reduce scale dominance
        x, y, z, t = embed_4d(n, moduli, radius_power)
        scale = max(1e-12, math.sqrt(x * x + y * y + z * z + t * t))
        mod = [x / scale, y / scale, z / scale, t / scale]

        # divisor-neighborhood features around n (wheel effect)
        d1m = math.log(tau[n - 1] + 1.0)
        d1p = math.log(tau[n + 1] + 1.0)
        d2m = math.log(tau[n - 2] + 1.0)
        d2p = math.log(tau[n + 2] + 1.0)
        d3m = math.log(tau[n - 3] + 1.0)
        d3p = math.log(tau[n + 3] + 1.0)
        div = [
            d1m,
            d1p,
            0.5 * (d1m + d1p),
            abs(d1m - d1p),
            0.5 * (d2m + d2p),
            0.5 * (d3m + d3p),
        ]

        comb = mod + div

        if is_p:
            mod_prime.append(mod)
            div_prime.append(div)
            comb_prime.append(comb)
        else:
            mod_comp.append(mod)
            div_comp.append(div)
            comb_comp.append(comb)

    k = min(len(mod_prime), len(mod_comp))
    return (
        mod_prime[:k],
        mod_comp[:k],
        div_prime[:k],
        div_comp[:k],
        comb_prime[:k],
        comb_comp[:k],
    )


def run_family(n_max: int, moduli: Sequence[int], radius_power: float) -> Dict[str, object]:
    is_prime = sieve_primes(n_max)
    tau = divisor_count_sieve(n_max + 4)

    nums = []
    for n in range(5, n_max - 4):
        nums.append((n, bool(is_prime[n])))

    mod_p, mod_c, div_p, div_c, cmb_p, cmb_c = build_features(nums, tau, moduli, radius_power)

    out = {
        "n_max": n_max,
        "moduli": list(moduli),
        "radius_power": radius_power,
        "balanced_size": min(len(mod_p), len(mod_c)),
        "accuracy_modular": nearest_centroid_accuracy(mod_p, mod_c),
        "accuracy_divisor_wheel": nearest_centroid_accuracy(div_p, div_c),
        "accuracy_combined": nearest_centroid_accuracy(cmb_p, cmb_c),
        "formula_modular": linear_discriminant(mod_p, mod_c),
        "formula_divisor_wheel": linear_discriminant(div_p, div_c),
        "formula_combined": linear_discriminant(cmb_p, cmb_c),
    }
    out["delta_combined_vs_modular"] = out["accuracy_combined"] - out["accuracy_modular"]
    out["delta_divisor_vs_modular"] = out["accuracy_divisor_wheel"] - out["accuracy_modular"]
    return out


def main() -> None:
    parser = argparse.ArgumentParser(description="Divisor-wheel + modular geometry probe")
    parser.add_argument("--n-max", type=int, default=200000)
    parser.add_argument(
        "--moduli-families",
        type=str,
        default="5,7,11,13,19;6,30,210,2310,30030",
        help="semicolon-separated families",
    )
    parser.add_argument("--radius-power", type=float, default=1.3)
    parser.add_argument("--output", type=str, default="research/output/divisor_wheel_probe.json")
    args = parser.parse_args()

    families = [parse_moduli(chunk) for chunk in args.moduli_families.split(";") if chunk.strip()]
    rows = []
    for fam in families:
        rows.append(run_family(args.n_max, fam, args.radius_power))

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "radius_power": args.radius_power,
            "families": families,
        },
        "results": rows,
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Divisor Wheel Probe\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        for r in rows:
            label = "-".join(str(x) for x in r["moduli"])
            f.write(f"## Family {label}\n\n")
            f.write(f"- balanced_size: {r['balanced_size']}\n")
            f.write(f"- modular accuracy: {r['accuracy_modular']:.6f}\n")
            f.write(f"- divisor-wheel accuracy: {r['accuracy_divisor_wheel']:.6f}\n")
            f.write(f"- combined accuracy: {r['accuracy_combined']:.6f}\n")
            f.write(f"- combined - modular: {r['delta_combined_vs_modular']:+.6f}\n")
            f.write(f"- divisor - modular: {r['delta_divisor_vs_modular']:+.6f}\n\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    for r in rows:
        print(
            "family",
            "-".join(str(x) for x in r["moduli"]),
            "mod",
            round(r["accuracy_modular"], 6),
            "div",
            round(r["accuracy_divisor_wheel"], 6),
            "comb",
            round(r["accuracy_combined"], 6),
        )


if __name__ == "__main__":
    main()
