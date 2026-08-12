#!/usr/bin/env python3
"""Prime modular geometry exploration loop.

No third-party dependencies. Produces quantitative signals for prime-vs-composite
structure in a 4D modular polar embedding, plus zeta-informed spectral alignment,
calibrated control tests, and lightweight SVG visualizations.
"""

from __future__ import annotations

import argparse
import cmath
import json
import math
import os
import random
import re
import statistics
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Dict, List, Optional, Sequence, Tuple

# First non-trivial zeta zero imaginary parts (hard-coded constants)
ZETA_ZERO_IMAG = [
    14.134725,
    21.022040,
    25.010858,
    30.424876,
    32.935062,
    37.586178,
    40.918719,
    43.327073,
    48.005151,
    49.773832,
]


@dataclass
class ExperimentConfig:
    n_max: int
    moduli: List[int]
    radius_power: float
    bins: int
    control_trials: int = 0
    control_seed: int = 1729
    zeta_zeros_imag: Optional[List[float]] = None
    max_zeta_zeros: int = 256


def parse_moduli(text: str) -> List[int]:
    values = []
    for token in text.split(","):
        token = token.strip()
        if not token:
            continue
        m = int(token)
        if m <= 1:
            raise ValueError(f"modulus must be >1, got {m}")
        values.append(m)
    if len(values) < 2:
        raise ValueError("provide at least two moduli")
    return values


def parse_floats(text: str) -> List[float]:
    values = []
    for token in text.split(","):
        token = token.strip()
        if token:
            values.append(float(token))
    return values


def load_zeta_zeros_file(path: str) -> List[float]:
    with open(path, "r", encoding="utf-8") as f:
        text = f.read().strip()

    if not text:
        return []

    # JSON list or JSON object with "zeros" / "positive_zeros".
    if text.startswith("{") or text.startswith("["):
        data = json.loads(text)
        if isinstance(data, list):
            return [float(x) for x in data]
        if isinstance(data, dict):
            if "zeros" in data and isinstance(data["zeros"], list):
                return [float(x) for x in data["zeros"]]
            if "positive_zeros" in data and isinstance(data["positive_zeros"], list):
                return [float(x) for x in data["positive_zeros"]]

    # Fallback: parse all floats from plain text.
    values = []
    for tok in re.findall(r"[+-]?(?:\d+\.\d+|\d+)", text):
        values.append(float(tok))
    return sorted(v for v in values if v > 0.0)


def sieve_primes(n_max: int) -> List[bool]:
    is_prime = [False, False] + [True] * (n_max - 1)
    limit = int(n_max**0.5)
    for p in range(2, limit + 1):
        if is_prime[p]:
            start = p * p
            step = p
            is_prime[start : n_max + 1 : step] = [False] * (((n_max - start) // step) + 1)
    return is_prime


def unit_complex(n: int, modulus: int) -> complex:
    angle = (2.0 * math.pi * (n % modulus)) / modulus
    return cmath.rect(1.0, angle)


def embed_4d(n: int, moduli: Sequence[int], radius_power: float) -> Tuple[float, float, float, float]:
    half = len(moduli) // 2
    left = moduli[:half]
    right = moduli[half:]

    c1 = sum(unit_complex(n, m) for m in left) / max(1, len(left))
    c2 = sum(unit_complex(n, m) for m in right) / max(1, len(right))

    r = n**radius_power
    return (r * c1.real, r * c1.imag, r * c2.real, r * c2.imag)


def l2(v: Sequence[float]) -> float:
    return math.sqrt(sum(x * x for x in v))


def mean_vec(rows: Sequence[Sequence[float]]) -> List[float]:
    dim = len(rows[0])
    return [sum(row[i] for row in rows) / len(rows) for i in range(dim)]


def avg_sq_distance(rows: Sequence[Sequence[float]], center: Sequence[float]) -> float:
    return sum(sum((row[i] - center[i]) ** 2 for i in range(len(center))) for row in rows) / len(rows)


def angle_entropy(vectors: Sequence[Sequence[float]], bins: int) -> float:
    counts = [0] * bins
    for x, y, *_ in vectors:
        angle = math.atan2(y, x)
        idx = int(((angle + math.pi) / (2 * math.pi)) * bins)
        idx = min(bins - 1, max(0, idx))
        counts[idx] += 1

    total = sum(counts)
    if total == 0:
        return 0.0

    entropy = 0.0
    for c in counts:
        if c:
            p = c / total
            entropy -= p * math.log(p)
    return entropy / math.log(bins)


def angle_counts(vectors: Sequence[Sequence[float]], bins: int) -> List[int]:
    counts = [0] * bins
    for x, y, *_ in vectors:
        angle = math.atan2(y, x)
        idx = int(((angle + math.pi) / (2 * math.pi)) * bins)
        idx = min(bins - 1, max(0, idx))
        counts[idx] += 1
    return counts


def dft_powers(series: Sequence[float], freqs: Sequence[float]) -> List[float]:
    n = len(series)
    if n == 0:
        return [0.0 for _ in freqs]

    mean = sum(series) / n
    centered = [x - mean for x in series]
    powers = []
    for f in freqs:
        w = -2.0 * math.pi * f / n
        total = 0j
        for i, x in enumerate(centered):
            total += x * cmath.exp(1j * w * i)
        powers.append((total.real * total.real + total.imag * total.imag) / n)
    return powers


def zeta_alignment(prime_norms: Sequence[float], zeros_imag: Optional[Sequence[float]] = None) -> Dict[str, object]:
    zeros = list(zeros_imag) if zeros_imag else ZETA_ZERO_IMAG
    if not zeros:
        return {"score": 0.0, "target_frequencies": [], "powers": []}

    gaps = [prime_norms[i + 1] - prime_norms[i] for i in range(len(prime_norms) - 1)]
    if len(gaps) < 32:
        return {"score": 0.0, "target_frequencies": [], "powers": []}

    max_zero = max(zeros)
    targets = [0.5 + 0.45 * (z / max_zero) * len(gaps) for z in zeros]

    target_powers = dft_powers(gaps, targets)
    baseline_freqs = [1 + i for i in range(min(64, len(gaps) // 2))]
    baseline_powers = dft_powers(gaps, baseline_freqs)
    baseline = statistics.fmean(baseline_powers) if baseline_powers else 1.0
    score = (statistics.fmean(target_powers) / baseline) if baseline else 0.0

    return {
        "score": score,
        "target_frequencies": targets,
        "powers": target_powers,
    }


def nearest_centroid_accuracy(prime_vectors: Sequence[Sequence[float]], comp_vectors: Sequence[Sequence[float]]) -> float:
    c_p = mean_vec(prime_vectors)
    c_c = mean_vec(comp_vectors)

    def predict(v: Sequence[float]) -> bool:
        d_p = sum((v[i] - c_p[i]) ** 2 for i in range(4))
        d_c = sum((v[i] - c_c[i]) ** 2 for i in range(4))
        return d_p < d_c

    correct = sum(1 for v in prime_vectors if predict(v))
    correct += sum(1 for v in comp_vectors if not predict(v))
    total = len(prime_vectors) + len(comp_vectors)
    return correct / total if total else 0.0


def core_metrics(prime_vectors: Sequence[Sequence[float]], comp_vectors: Sequence[Sequence[float]], bins: int) -> Dict[str, float]:
    p_center = mean_vec(prime_vectors)
    c_center = mean_vec(comp_vectors)
    center_distance = l2([p_center[i] - c_center[i] for i in range(4)])

    p_spread = math.sqrt(avg_sq_distance(prime_vectors, p_center))
    c_spread = math.sqrt(avg_sq_distance(comp_vectors, c_center))
    pooled_spread = max(1e-12, 0.5 * (p_spread + c_spread))

    separation = center_distance / pooled_spread
    entropy_prime = angle_entropy(prime_vectors, bins)
    entropy_comp = angle_entropy(comp_vectors, bins)

    return {
        "center_distance": center_distance,
        "pooled_spread": pooled_spread,
        "separation_ratio": separation,
        "nearest_centroid_accuracy": nearest_centroid_accuracy(prime_vectors, comp_vectors),
        "prime_angle_entropy": entropy_prime,
        "composite_angle_entropy": entropy_comp,
        "entropy_delta": entropy_prime - entropy_comp,
    }


def summarize_null_distribution(observed: float, null_values: List[float]) -> Dict[str, float]:
    if not null_values:
        return {
            "observed": observed,
            "null_mean": 0.0,
            "null_std": 0.0,
            "null_p95": 0.0,
            "p_value_ge": 1.0,
            "z_score": 0.0,
        }

    null_mean = statistics.fmean(null_values)
    null_std = statistics.pstdev(null_values)
    null_sorted = sorted(null_values)
    p95_idx = min(len(null_sorted) - 1, int(0.95 * (len(null_sorted) - 1)))
    p95 = null_sorted[p95_idx]
    ge_count = sum(1 for v in null_values if v >= observed)
    p_value_ge = (ge_count + 1) / (len(null_values) + 1)
    z_score = (observed - null_mean) / (null_std if null_std > 1e-12 else 1e-12)

    return {
        "observed": observed,
        "null_mean": null_mean,
        "null_std": null_std,
        "null_p95": p95,
        "p_value_ge": p_value_ge,
        "z_score": z_score,
    }


def permutation_controls(
    prime_vectors: Sequence[Sequence[float]],
    comp_vectors: Sequence[Sequence[float]],
    bins: int,
    trials: int,
    seed: int,
) -> Dict[str, object]:
    if trials <= 0:
        return {}

    rng = random.Random(seed)
    n = min(len(prime_vectors), len(comp_vectors))
    combined = list(prime_vectors[:n]) + list(comp_vectors[:n])
    total = 2 * n

    observed = core_metrics(prime_vectors[:n], comp_vectors[:n], bins)
    null_acc = []
    null_sep = []

    index_pool = list(range(total))
    for _ in range(trials):
        left = set(rng.sample(index_pool, n))
        a = [combined[i] for i in range(total) if i in left]
        b = [combined[i] for i in range(total) if i not in left]
        m = core_metrics(a, b, bins)
        null_acc.append(m["nearest_centroid_accuracy"])
        null_sep.append(m["separation_ratio"])

    return {
        "trials": trials,
        "seed": seed,
        "accuracy": summarize_null_distribution(observed["nearest_centroid_accuracy"], null_acc),
        "separation_ratio": summarize_null_distribution(observed["separation_ratio"], null_sep),
    }


def zeta_permutation_control(
    prime_norms: Sequence[float],
    trials: int,
    seed: int,
    zeros_imag: Optional[Sequence[float]] = None,
) -> Dict[str, float]:
    if trials <= 0:
        return {}

    rng = random.Random(seed)
    observed = zeta_alignment(prime_norms, zeros_imag=zeros_imag)["score"]
    null_scores = []
    norms = list(prime_norms)

    for _ in range(trials):
        rng.shuffle(norms)
        null_scores.append(zeta_alignment(norms, zeros_imag=zeros_imag)["score"])

    summary = summarize_null_distribution(observed, null_scores)
    summary["trials"] = trials
    summary["seed"] = seed
    return summary


def _project_points(points: Sequence[Tuple[float, float]], width: int, height: int, pad: int) -> List[Tuple[float, float]]:
    xs = [p[0] for p in points]
    ys = [p[1] for p in points]
    min_x, max_x = min(xs), max(xs)
    min_y, max_y = min(ys), max(ys)
    span_x = max(1e-12, max_x - min_x)
    span_y = max(1e-12, max_y - min_y)

    out = []
    for x, y in points:
        px = pad + (x - min_x) / span_x * (width - 2 * pad)
        py = height - pad - (y - min_y) / span_y * (height - 2 * pad)
        out.append((px, py))
    return out


def write_scatter_svg(
    prime_vectors: Sequence[Sequence[float]],
    comp_vectors: Sequence[Sequence[float]],
    out_path: str,
    seed: int,
    max_points: int = 3000,
) -> None:
    rng = random.Random(seed)
    p_idx = list(range(len(prime_vectors)))
    c_idx = list(range(len(comp_vectors)))
    rng.shuffle(p_idx)
    rng.shuffle(c_idx)
    p_sel = [prime_vectors[i] for i in p_idx[: min(max_points, len(prime_vectors))]]
    c_sel = [comp_vectors[i] for i in c_idx[: min(max_points, len(comp_vectors))]]

    points = [(v[0], v[1]) for v in p_sel + c_sel]
    projected = _project_points(points, width=900, height=650, pad=50)

    p_proj = projected[: len(p_sel)]
    c_proj = projected[len(p_sel) :]

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write('<svg xmlns="http://www.w3.org/2000/svg" width="900" height="650" viewBox="0 0 900 650">\n')
        f.write('<rect x="0" y="0" width="900" height="650" fill="#f7f7f5"/>\n')
        f.write('<text x="24" y="30" font-size="18" fill="#1f2937">Prime vs Composite Projection (x,y)</text>\n')
        f.write('<text x="24" y="50" font-size="12" fill="#4b5563">blue=prime, orange=composite</text>\n')
        for x, y in c_proj:
            f.write(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="1.8" fill="#e67e22" fill-opacity="0.28"/>\n')
        for x, y in p_proj:
            f.write(f'<circle cx="{x:.2f}" cy="{y:.2f}" r="1.8" fill="#1f77b4" fill-opacity="0.35"/>\n')
        f.write("</svg>\n")


def write_angle_hist_svg(
    prime_vectors: Sequence[Sequence[float]],
    comp_vectors: Sequence[Sequence[float]],
    bins: int,
    out_path: str,
) -> None:
    pc = angle_counts(prime_vectors, bins)
    cc = angle_counts(comp_vectors, bins)
    p_total = max(1, sum(pc))
    c_total = max(1, sum(cc))
    p_norm = [v / p_total for v in pc]
    c_norm = [v / c_total for v in cc]

    width = 900
    height = 420
    pad_x = 45
    pad_y = 35
    inner_w = width - 2 * pad_x
    inner_h = height - 2 * pad_y
    bar_w = inner_w / bins

    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">\n')
        f.write(f'<rect x="0" y="0" width="{width}" height="{height}" fill="#fcfcfb"/>\n')
        f.write('<text x="24" y="24" font-size="16" fill="#111827">Angular Density Histogram</text>\n')
        for i in range(bins):
            x = pad_x + i * bar_w
            ph = p_norm[i] * inner_h * 8.0
            ch = c_norm[i] * inner_h * 8.0
            y_p = height - pad_y - ph
            y_c = height - pad_y - ch
            f.write(f'<rect x="{x + 1:.2f}" y="{y_c:.2f}" width="{max(1.0, bar_w - 2):.2f}" height="{max(0.4, ch):.2f}" fill="#e67e22" fill-opacity="0.45"/>\n')
            f.write(f'<rect x="{x + 1:.2f}" y="{y_p:.2f}" width="{max(1.0, bar_w - 2):.2f}" height="{max(0.4, ph):.2f}" fill="#1f77b4" fill-opacity="0.45"/>\n')
        f.write("</svg>\n")


def run_experiment(cfg: ExperimentConfig) -> Dict[str, object]:
    is_prime = sieve_primes(cfg.n_max)
    primes = [n for n in range(2, cfg.n_max + 1) if is_prime[n]]
    composites = [n for n in range(4, cfg.n_max + 1) if not is_prime[n]]

    prime_vectors = [embed_4d(n, cfg.moduli, cfg.radius_power) for n in primes]
    comp_vectors = [embed_4d(n, cfg.moduli, cfg.radius_power) for n in composites]

    # Balance classes for a fair separation estimate.
    limit = min(len(prime_vectors), len(comp_vectors))
    pv = prime_vectors[:limit]
    cv = comp_vectors[:limit]

    prime_norms = [l2(v) for v in prime_vectors]
    zeta_zeros = cfg.zeta_zeros_imag if cfg.zeta_zeros_imag else ZETA_ZERO_IMAG
    if cfg.max_zeta_zeros > 0:
        zeta_zeros = zeta_zeros[: cfg.max_zeta_zeros]
    zeta = zeta_alignment(prime_norms, zeros_imag=zeta_zeros)
    core = core_metrics(pv, cv, cfg.bins)

    metrics = {
        "prime_count": len(primes),
        "composite_count": len(composites),
        "balanced_sample_size": limit,
        **core,
        "zeta_alignment_score": zeta["score"],
    }

    controls = {}
    if cfg.control_trials > 0:
        controls["label_permutation"] = permutation_controls(pv, cv, cfg.bins, cfg.control_trials, cfg.control_seed)
        controls["zeta_permutation"] = zeta_permutation_control(
            prime_norms,
            cfg.control_trials,
            cfg.control_seed + 97,
            zeros_imag=zeta_zeros,
        )

    recommendations = []
    if metrics["separation_ratio"] < 0.15:
        recommendations.append("increase modulus diversity or adjust radius_power to improve geometric separation")
    if metrics["nearest_centroid_accuracy"] < 0.56:
        recommendations.append("test alternate embeddings (weighted modulus groups or log-radius scaling)")

    if controls:
        lp = controls["label_permutation"]["accuracy"]
        zp = controls["zeta_permutation"]
        if lp["p_value_ge"] <= 0.05:
            recommendations.append("geometric separation survives permutation controls; prioritize stability sweeps")
        else:
            recommendations.append("geometric signal not significant under current controls; revise embedding")
        if zp["p_value_ge"] <= 0.05:
            recommendations.append("zeta score exceeds shuffled baseline; test across larger n_max bands")
        else:
            recommendations.append("zeta score is not significant vs shuffled controls; treat as exploratory")
    elif zeta["score"] > 1.15:
        recommendations.append("repeat run across larger n_max and neighboring modulus sets to test spectral stability")
    else:
        recommendations.append("zeta alignment is weak or neutral; prioritize geometric feature engineering first")

    return {
        "timestamp_utc": datetime.now(timezone.utc).isoformat(),
        "config": {
            "n_max": cfg.n_max,
            "moduli": cfg.moduli,
            "radius_power": cfg.radius_power,
            "angle_bins": cfg.bins,
            "control_trials": cfg.control_trials,
            "control_seed": cfg.control_seed,
            "zeta_zeros_count": len(zeta_zeros),
            "zeta_zeros_source_count": len(cfg.zeta_zeros_imag) if cfg.zeta_zeros_imag else len(ZETA_ZERO_IMAG),
            "max_zeta_zeros": cfg.max_zeta_zeros,
        },
        "metrics": metrics,
        "zeta_alignment": zeta,
        "controls": controls,
        "recommendations": recommendations,
        # small sample for debugging/inspection only
        "samples": {
            "prime_vectors_head": [list(v) for v in prime_vectors[:8]],
            "composite_vectors_head": [list(v) for v in comp_vectors[:8]],
        },
    }


def write_report(report: Dict[str, object], out_path: str) -> None:
    os.makedirs(os.path.dirname(out_path), exist_ok=True)
    with open(out_path, "w", encoding="utf-8") as f:
        json.dump(report, f, indent=2)


def build_visualizations(cfg: ExperimentConfig, out_dir: str) -> Dict[str, str]:
    is_prime = sieve_primes(cfg.n_max)
    primes = [n for n in range(2, cfg.n_max + 1) if is_prime[n]]
    composites = [n for n in range(4, cfg.n_max + 1) if not is_prime[n]]
    prime_vectors = [embed_4d(n, cfg.moduli, cfg.radius_power) for n in primes]
    comp_vectors = [embed_4d(n, cfg.moduli, cfg.radius_power) for n in composites]

    os.makedirs(out_dir, exist_ok=True)
    scatter_path = os.path.join(out_dir, "scatter_xy.svg")
    hist_path = os.path.join(out_dir, "angle_hist.svg")

    write_scatter_svg(prime_vectors, comp_vectors, scatter_path, seed=cfg.control_seed)
    write_angle_hist_svg(prime_vectors, comp_vectors, bins=cfg.bins, out_path=hist_path)

    return {
        "scatter_xy": scatter_path,
        "angle_hist": hist_path,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description="Prime modular geometry research loop")
    parser.add_argument("--n-max", type=int, default=20000)
    parser.add_argument("--moduli", type=str, default="5,7,11,13")
    parser.add_argument("--radius-power", type=float, default=1.7)
    parser.add_argument("--bins", type=int, default=36)
    parser.add_argument("--control-trials", type=int, default=0)
    parser.add_argument("--control-seed", type=int, default=1729)
    parser.add_argument("--output", type=str, default="")
    parser.add_argument("--zeta-zeros-file", type=str, default="")
    parser.add_argument("--max-zeta-zeros", type=int, default=256)
    parser.add_argument("--make-viz", action="store_true")
    parser.add_argument("--viz-dir", type=str, default="")
    args = parser.parse_args()

    moduli = parse_moduli(args.moduli)
    cfg = ExperimentConfig(
        n_max=args.n_max,
        moduli=moduli,
        radius_power=args.radius_power,
        bins=max(8, args.bins),
        control_trials=max(0, args.control_trials),
        control_seed=args.control_seed,
        zeta_zeros_imag=load_zeta_zeros_file(args.zeta_zeros_file) if args.zeta_zeros_file else None,
        max_zeta_zeros=max(0, args.max_zeta_zeros),
    )

    report = run_experiment(cfg)

    ts = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out_path = args.output or f"research/output/geometry_report_{ts}.json"
    write_report(report, out_path)
    write_report(report, "research/output/geometry_report.json")

    print(f"wrote: {out_path}")
    print("metrics:")
    for k, v in report["metrics"].items():
        if isinstance(v, float):
            print(f"  {k}: {v:.6f}")
        else:
            print(f"  {k}: {v}")

    if cfg.control_trials > 0:
        lp = report["controls"]["label_permutation"]["accuracy"]
        zp = report["controls"]["zeta_permutation"]
        print("controls:")
        print(f"  label_accuracy_p_value_ge: {lp['p_value_ge']:.6f}")
        print(f"  zeta_score_p_value_ge: {zp['p_value_ge']:.6f}")

    if args.make_viz:
        base = os.path.splitext(os.path.basename(out_path))[0]
        viz_dir = args.viz_dir or os.path.join("research/output/figures", base)
        artifacts = build_visualizations(cfg, viz_dir)
        print("visualizations:")
        for k, v in artifacts.items():
            print(f"  {k}: {v}")


if __name__ == "__main__":
    main()
