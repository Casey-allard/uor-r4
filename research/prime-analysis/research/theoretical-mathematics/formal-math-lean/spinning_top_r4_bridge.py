#!/usr/bin/env python3
"""Spinning-top bridge: zeta trajectory in 2D and lifted R^4 manifold tests.

Maps each integer n to:
  F(u_n)   = sum_gamma exp(i*gamma*u_n) / (1/2 + i*gamma)
  F'(u_n)  = sum_gamma i*gamma*exp(i*gamma*u_n) / (1/2 + i*gamma)

R^4 point is (Re F, Im F, Re F', Im F').
Then evaluates prime-vs-composite structure and top-like phase metrics.
"""

from __future__ import annotations

import argparse
import json
import math
import os
from datetime import datetime, timezone
from typing import Dict, List, Sequence, Tuple

from prime_geometry_loop import load_zeta_zeros_file, sieve_primes


def unwrap_delta(a: float, b: float) -> float:
    d = b - a
    while d > math.pi:
        d -= 2.0 * math.pi
    while d < -math.pi:
        d += 2.0 * math.pi
    return d


def mean_vec(rows: Sequence[Sequence[float]]) -> List[float]:
    if not rows:
        return [0.0, 0.0, 0.0, 0.0]
    d = len(rows[0])
    return [sum(r[i] for r in rows) / len(rows) for i in range(d)]


def l2(v: Sequence[float]) -> float:
    return math.sqrt(sum(x * x for x in v))


def avg_sq_distance(rows: Sequence[Sequence[float]], center: Sequence[float]) -> float:
    if not rows:
        return 0.0
    return sum(sum((r[i] - center[i]) ** 2 for i in range(len(center))) for r in rows) / len(rows)


def centroid_accuracy(primes: Sequence[Sequence[float]], comps: Sequence[Sequence[float]]) -> float:
    cp = mean_vec(primes)
    cc = mean_vec(comps)

    def predict(v: Sequence[float]) -> bool:
        dp = sum((v[i] - cp[i]) ** 2 for i in range(4))
        dc = sum((v[i] - cc[i]) ** 2 for i in range(4))
        return dp < dc

    good = sum(1 for v in primes if predict(v))
    good += sum(1 for v in comps if not predict(v))
    total = len(primes) + len(comps)
    return good / total if total else 0.0


def top_metrics(prime_path: Sequence[Sequence[float]]) -> Dict[str, float]:
    if len(prime_path) < 8:
        return {"precession_mean": 0.0, "nutation_mean": 0.0, "wobble_std": 0.0}
    phi1 = [math.atan2(v[1], v[0]) for v in prime_path]
    phi2 = [math.atan2(v[3], v[2]) for v in prime_path]
    pre = []
    nut = []
    for i in range(len(prime_path) - 1):
        d1 = unwrap_delta(phi1[i], phi1[i + 1])
        d2 = unwrap_delta(phi2[i], phi2[i + 1])
        pre.append(0.5 * (d1 + d2))
        nut.append(0.5 * (d1 - d2))
    wob = [nut[i + 1] - nut[i] for i in range(len(nut) - 1)]
    wob_mean = sum(wob) / max(1, len(wob))
    wob_var = sum((x - wob_mean) ** 2 for x in wob) / max(1, len(wob))
    return {
        "precession_mean": sum(pre) / max(1, len(pre)),
        "nutation_mean": sum(nut) / max(1, len(nut)),
        "wobble_std": math.sqrt(wob_var),
    }


def zeta_r4_points(n_max: int, zeros: Sequence[float], u_mode: str) -> List[Tuple[float, float, float, float]]:
    out = []
    for n in range(2, n_max + 1):
        u = math.log(n) if u_mode == "log" else float(n)
        fr = 0.0
        fi = 0.0
        gr = 0.0
        gi = 0.0
        for g in zeros:
            c = math.cos(g * u)
            s = math.sin(g * u)
            den = 0.25 + g * g
            # (c + i s) / (1/2 + i g)
            ar = (0.5 * c + g * s) / den
            ai = (0.5 * s - g * c) / den
            fr += ar
            fi += ai
            # derivative wrt u: i*g * term
            gr += -g * ai
            gi += g * ar
        out.append((fr, fi, gr, gi))
    return out


def write_svg(points: Sequence[Tuple[float, float]], out_path: str) -> None:
    if not points:
        return
    w, h, pad = 900, 700, 50
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    x0, x1 = min(xs), max(xs)
    y0, y1 = min(ys), max(ys)
    sx = max(1e-12, x1 - x0)
    sy = max(1e-12, y1 - y0)

    def px(x: float) -> float:
        return pad + (x - x0) / sx * (w - 2 * pad)

    def py(y: float) -> float:
        return h - pad - (y - y0) / sy * (h - 2 * pad)

    path = " ".join(f"{px(x):.2f},{py(y):.2f}" for x, y in points)
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">\n')
        f.write(f'<rect x="0" y="0" width="{w}" height="{h}" fill="#f7f7f7"/>\n')
        f.write('<text x="20" y="30" font-size="18" fill="#111827">Spinning-Top Trajectory: Re(F) vs Im(F)</text>\n')
        f.write(f'<polyline fill="none" stroke="#1f77b4" stroke-width="1.2" points="{path}"/>\n')
        f.write("</svg>\n")


def main() -> None:
    ap = argparse.ArgumentParser(description="R^4 spinning-top bridge from zeta trajectory")
    ap.add_argument("--n-max", type=int, default=200000)
    ap.add_argument("--zeta-zeros-file", type=str, default="research/data/zeta_zeros_odlyzko_100k.json")
    ap.add_argument("--max-zeta-zeros", type=int, default=128)
    ap.add_argument("--u-mode", type=str, default="log", choices=["log", "linear"])
    ap.add_argument("--output", type=str, default="research/output/spinning_top_r4_bridge.json")
    args = ap.parse_args()

    zeros = load_zeta_zeros_file(args.zeta_zeros_file)
    zeros = zeros[: args.max_zeta_zeros] if args.max_zeta_zeros > 0 else zeros

    points = zeta_r4_points(args.n_max, zeros, args.u_mode)
    is_prime = sieve_primes(args.n_max)
    prime_vec = [points[i - 2] for i in range(2, args.n_max + 1) if is_prime[i]]
    comp_vec = [points[i - 2] for i in range(4, args.n_max + 1) if not is_prime[i]]
    k = min(len(prime_vec), len(comp_vec))
    prime_vec = prime_vec[:k]
    comp_vec = comp_vec[:k]

    cp = mean_vec(prime_vec)
    cc = mean_vec(comp_vec)
    center_distance = l2([cp[i] - cc[i] for i in range(4)])
    p_spread = math.sqrt(avg_sq_distance(prime_vec, cp))
    c_spread = math.sqrt(avg_sq_distance(comp_vec, cc))
    pooled = max(1e-12, 0.5 * (p_spread + c_spread))
    separation = center_distance / pooled
    acc = centroid_accuracy(prime_vec, comp_vec)
    top = top_metrics(prime_vec)

    report = {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": args.n_max,
            "u_mode": args.u_mode,
            "zeta_zeros_file": args.zeta_zeros_file,
            "max_zeta_zeros": len(zeros),
        },
        "metrics": {
            "prime_count_used": len(prime_vec),
            "composite_count_used": len(comp_vec),
            "center_distance": center_distance,
            "pooled_spread": pooled,
            "separation_ratio": separation,
            "nearest_centroid_accuracy": acc,
            **top,
        },
    }

    os.makedirs(os.path.dirname(args.output), exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)

    svg = args.output.replace(".json", ".svg")
    write_svg([(p[0], p[1]) for p in points], svg)

    md = args.output.replace(".json", ".md")
    with open(md, "w", encoding="utf-8") as f:
        f.write("# Spinning-Top R4 Bridge\n\n")
        f.write(f"Generated: {report['timestamp_utc']}\n\n")
        f.write(f"- n_max: {args.n_max}\n")
        f.write(f"- zeros used: {len(zeros)}\n")
        f.write(f"- separation_ratio: {separation:.6f}\n")
        f.write(f"- nearest_centroid_accuracy: {acc:.6f}\n")
        f.write(f"- precession_mean: {top['precession_mean']:.6f}\n")
        f.write(f"- nutation_mean: {top['nutation_mean']:.6f}\n")
        f.write(f"- wobble_std: {top['wobble_std']:.6f}\n")
        f.write(f"- svg: {svg}\n")

    print(f"wrote: {args.output}")
    print(f"wrote: {md}")
    print("separation_ratio:", round(separation, 6))
    print("nearest_centroid_accuracy:", round(acc, 6))


if __name__ == "__main__":
    main()
