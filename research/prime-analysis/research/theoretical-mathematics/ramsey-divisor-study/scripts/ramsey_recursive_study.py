#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import math
import random
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Tuple

try:
    import numpy as np
except Exception:  # pragma: no cover
    np = None


PHI = (1.0 + math.sqrt(5.0)) / 2.0


@dataclass
class InstanceResult:
    family: str
    n: int
    seed: int
    omega_red: int
    omega_blue: int
    alpha_red: int
    alpha_blue: int
    max_mono_clique_present: int
    max_mono_independent_present: int
    first_absent_mono_clique_size: int
    first_absent_mono_independent_size: int
    delay_score: float
    force_index: float
    red_density: float
    spectral_radius: float
    spectral_gap: float
    adjacency_energy: float
    signature_name: str
    signature_value: float
    signature_phi_distance: float


def set_edge(mat: List[List[int]], i: int, j: int, red: bool) -> None:
    v = 1 if red else 0
    mat[i][j] = v
    mat[j][i] = v


def combine_blocks(
    left: List[List[int]],
    right: List[List[int]],
    cross_rule,
) -> List[List[int]]:
    a = len(left)
    b = len(right)
    n = a + b
    out = [[0] * n for _ in range(n)]
    for i in range(a):
        for j in range(i + 1, a):
            out[i][j] = left[i][j]
            out[j][i] = left[j][i]
    for i in range(b):
        for j in range(i + 1, b):
            out[a + i][a + j] = right[i][j]
            out[a + j][a + i] = right[j][i]
    for i in range(a):
        for j in range(b):
            set_edge(out, i, a + j, cross_rule(i, j, a, b))
    return out


def fibonacci_family(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    rng = random.Random(seed)
    offset = rng.random()
    parity_flip = seed % 2

    g1 = [[0]]
    g2 = [[0, 1], [1, 0]]
    sizes = [1, 2]
    graphs = [g1, g2]
    while sizes[-1] < n:
        k = len(sizes) + 1
        left = graphs[-1]
        right = graphs[-2]

        def cross(i: int, j: int, a: int, b: int) -> bool:
            x = (((i + 1) * PHI + (j + 1) * (PHI - 1.0) + offset) % 1.0) < 0.5
            return x if (k + parity_flip) % 2 == 0 else (not x)

        new_graph = combine_blocks(left, right, cross)
        graphs.append(new_graph)
        sizes.append(sizes[-1] + sizes[-2])
    g = graphs[-1]
    g = [row[:n] for row in g[:n]]
    ratio = sizes[-2] / sizes[-3] if len(sizes) >= 3 else 1.0
    return g, {
        "signature_name": "fib_block_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def lucas_family(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    rng = random.Random(seed + 101)
    offset = rng.random()
    parity_flip = (seed + 1) % 2

    g1 = [[0, 1], [1, 0]]
    g2 = [[0]]
    sizes = [2, 1]
    graphs = [g1, g2]
    while sizes[-1] < n:
        k = len(sizes) + 1
        left = graphs[-1]
        right = graphs[-2]

        def cross(i: int, j: int, a: int, b: int) -> bool:
            x = (((i + 1) * (PHI - 1.0) + (j + 1) * PHI + offset) % 1.0) < 0.5
            if (k + parity_flip) % 3 == 0:
                x = not x
            return x

        new_graph = combine_blocks(left, right, cross)
        graphs.append(new_graph)
        sizes.append(sizes[-1] + sizes[-2])
    g = graphs[-1]
    g = [row[:n] for row in g[:n]]
    ratio = sizes[-2] / sizes[-3] if len(sizes) >= 3 else 1.0
    return g, {
        "signature_name": "lucas_block_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def balanced_binary_family(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    mat = [[0] * n for _ in range(n)]
    root_color = seed % 2

    def fill(lo: int, hi: int, depth: int) -> None:
        if hi - lo <= 1:
            return
        mid = (lo + hi) // 2
        red = ((depth + root_color) % 2 == 0)
        for i in range(lo, mid):
            for j in range(mid, hi):
                set_edge(mat, i, j, red)
        fill(lo, mid, depth + 1)
        fill(mid, hi, depth + 1)

    fill(0, n, 0)
    left = n // 2
    right = n - left
    ratio = max(left, right) / max(1, min(left, right))
    return mat, {
        "signature_name": "binary_partition_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def primes_upto(n: int) -> List[int]:
    if n < 2:
        return []
    sieve = [True] * (n + 1)
    sieve[0] = False
    sieve[1] = False
    p = 2
    while p * p <= n:
        if sieve[p]:
            for x in range(p * p, n + 1, p):
                sieve[x] = False
        p += 1
    return [i for i in range(2, n + 1) if sieve[i]]


def nearest_prime_cluster(v: int, primes: List[int]) -> int:
    best_idx = 0
    best_dist = abs(v - primes[0])
    for idx, p in enumerate(primes[1:], start=1):
        d = abs(v - p)
        if d < best_dist or (d == best_dist and p < primes[best_idx]):
            best_dist = d
            best_idx = idx
    return best_idx


def prime_anchor_family(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    mat = [[0] * n for _ in range(n)]
    primes = primes_upto(max(3, n))
    clusters = [nearest_prime_cluster(v + 1, primes) for v in range(n)]
    for i in range(n):
        ci = clusters[i]
        for j in range(i + 1, n):
            cj = clusters[j]
            if ci == cj:
                red = ((ci + seed) % 2 == 0)
            else:
                red = ((ci * cj + abs(ci - cj) + seed) % 3 == 0)
            set_edge(mat, i, j, red)
    gaps = [primes[i + 1] - primes[i] for i in range(len(primes) - 1)]
    mean_gap = (sum(gaps) / len(gaps)) if gaps else 1.0
    return mat, {
        "signature_name": "prime_mean_gap",
        "signature_value": mean_gap,
        "phi_distance": abs(mean_gap - PHI),
    }


def random_baseline_family(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    rng = random.Random(seed + 17)
    mat = [[0] * n for _ in range(n)]
    for i in range(n):
        for j in range(i + 1, n):
            set_edge(mat, i, j, rng.random() < 0.5)
    return mat, {
        "signature_name": "random_none",
        "signature_value": float("nan"),
        "phi_distance": float("nan"),
    }


def matrix_to_masks(mat: List[List[int]]) -> List[int]:
    n = len(mat)
    masks = [0] * n
    for i in range(n):
        m = 0
        row = mat[i]
        for j in range(n):
            if i != j and row[j]:
                m |= 1 << j
        masks[i] = m
    return masks


def complement_masks(masks: List[int], n: int) -> List[int]:
    all_bits = (1 << n) - 1
    out = [0] * n
    for i in range(n):
        out[i] = (~masks[i]) & (all_bits ^ (1 << i))
    return out


def max_clique_size(masks: List[int]) -> int:
    n = len(masks)
    best = 0

    def dfs(candidates: int, size: int) -> None:
        nonlocal best
        if not candidates:
            if size > best:
                best = size
            return
        if size + candidates.bit_count() <= best:
            return
        while candidates:
            if size + candidates.bit_count() <= best:
                return
            v_bit = candidates & -candidates
            v = v_bit.bit_length() - 1
            dfs(candidates & masks[v], size + 1)
            candidates ^= v_bit

    dfs((1 << n) - 1, 0)
    return best


def spectral_stats(mat: List[List[int]]) -> Tuple[float, float, float]:
    n = len(mat)
    if np is not None:
        arr = np.array(mat, dtype=float)
        eigs = np.linalg.eigvalsh(arr)
        spectral_radius = float(np.max(np.abs(eigs)))
        spectral_gap = float(eigs[-1] - eigs[-2]) if n >= 2 else 0.0
        energy = float(np.sum(np.abs(eigs)))
        return spectral_radius, spectral_gap, energy
    # Fallback: cheap approximations if numpy is unavailable.
    row_sums = [sum(r) for r in mat]
    spectral_radius = float(max(row_sums))
    spectral_gap = 0.0
    energy = float(sum(abs(x) for x in row_sums))
    return spectral_radius, spectral_gap, energy


FAMILY_BUILDERS = {
    "fibonacci_block_growth": fibonacci_family,
    "lucas_like_growth": lucas_family,
    "balanced_binary_partitions": balanced_binary_family,
    "prime_anchor_partitioning": prime_anchor_family,
    "random_baseline": random_baseline_family,
}


def evaluate_instance(family: str, n: int, seed: int) -> InstanceResult:
    mat, signature = FAMILY_BUILDERS[family](n, seed)
    masks_red = matrix_to_masks(mat)
    masks_blue = complement_masks(masks_red, n)
    omega_red = max_clique_size(masks_red)
    omega_blue = max_clique_size(masks_blue)
    alpha_red = omega_blue
    alpha_blue = omega_red
    max_mono_clique_present = max(omega_red, omega_blue)
    max_mono_independent_present = max(alpha_red, alpha_blue)
    first_absent_mono_clique_size = max_mono_clique_present + 1
    first_absent_mono_independent_size = max_mono_independent_present + 1
    delay_score = n / max_mono_clique_present
    force_index = max_mono_clique_present / n
    red_edges = sum(mat[i][j] for i in range(n) for j in range(i + 1, n))
    total_edges = n * (n - 1) / 2
    red_density = red_edges / total_edges if total_edges else 0.0
    spectral_radius, spectral_gap, adjacency_energy = spectral_stats(mat)
    return InstanceResult(
        family=family,
        n=n,
        seed=seed,
        omega_red=omega_red,
        omega_blue=omega_blue,
        alpha_red=alpha_red,
        alpha_blue=alpha_blue,
        max_mono_clique_present=max_mono_clique_present,
        max_mono_independent_present=max_mono_independent_present,
        first_absent_mono_clique_size=first_absent_mono_clique_size,
        first_absent_mono_independent_size=first_absent_mono_independent_size,
        delay_score=delay_score,
        force_index=force_index,
        red_density=red_density,
        spectral_radius=spectral_radius,
        spectral_gap=spectral_gap,
        adjacency_energy=adjacency_energy,
        signature_name=signature["signature_name"],
        signature_value=signature["signature_value"],
        signature_phi_distance=signature["phi_distance"],
    )


def write_instance_csv(path: Path, rows: List[InstanceResult]) -> None:
    fieldnames = list(InstanceResult.__dataclass_fields__.keys())
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        for r in rows:
            writer.writerow(r.__dict__)


def aggregate(rows: List[InstanceResult]) -> List[Dict[str, float]]:
    by_family: Dict[str, List[InstanceResult]] = defaultdict(list)
    for r in rows:
        by_family[r.family].append(r)
    out = []
    for family, items in by_family.items():
        items = sorted(items, key=lambda x: (x.n, x.seed))
        mean = lambda vals: sum(vals) / len(vals)
        summary = {
            "family": family,
            "instances": len(items),
            "mean_max_mono_clique_present": mean([i.max_mono_clique_present for i in items]),
            "mean_delay_score": mean([i.delay_score for i in items]),
            "mean_force_index": mean([i.force_index for i in items]),
            "mean_red_density": mean([i.red_density for i in items]),
            "mean_spectral_radius": mean([i.spectral_radius for i in items]),
            "mean_spectral_gap": mean([i.spectral_gap for i in items]),
            "mean_adjacency_energy": mean([i.adjacency_energy for i in items]),
            "best_delay_score": max(i.delay_score for i in items),
            "best_n": max(items, key=lambda i: i.delay_score).n,
            "mean_signature_value": mean(
                [i.signature_value for i in items if not math.isnan(i.signature_value)]
            )
            if any(not math.isnan(i.signature_value) for i in items)
            else float("nan"),
            "mean_signature_phi_distance": mean(
                [i.signature_phi_distance for i in items if not math.isnan(i.signature_phi_distance)]
            )
            if any(not math.isnan(i.signature_phi_distance) for i in items)
            else float("nan"),
        }
        out.append(summary)
    out.sort(key=lambda x: (-x["mean_delay_score"], x["mean_force_index"]))
    return out


def write_summary_csv(path: Path, summary: List[Dict[str, float]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not summary:
        return
    fieldnames = list(summary[0].keys())
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(summary)


def detect_signatures(rows: List[InstanceResult]) -> List[str]:
    notes: List[str] = []
    by_family: Dict[str, List[InstanceResult]] = defaultdict(list)
    for r in rows:
        by_family[r.family].append(r)
    for family, items in by_family.items():
        densities = [x.red_density for x in items]
        dmin, dmax = min(densities), max(densities)
        if dmax - dmin < 0.05:
            notes.append(f"{family}: red-edge density is stable ({dmin:.3f}..{dmax:.3f}).")
        else:
            notes.append(f"{family}: red-edge density varies ({dmin:.3f}..{dmax:.3f}).")
        phi_d = [x.signature_phi_distance for x in items if not math.isnan(x.signature_phi_distance)]
        if phi_d:
            notes.append(
                f"{family}: mean signature distance to phi is {sum(phi_d)/len(phi_d):.4f}."
            )
    return notes


def write_report(
    path: Path,
    summary: List[Dict[str, float]],
    rows: List[InstanceResult],
    sizes: List[int],
    seeds: List[int],
) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    by_family = {s["family"]: s for s in summary}
    baseline = by_family.get("random_baseline")
    recommendation = "REFINE"
    rationale = "Signal is mixed."
    if baseline is not None:
        winners = [
            fam
            for fam, s in by_family.items()
            if fam != "random_baseline"
            and s["mean_delay_score"] > baseline["mean_delay_score"] + 0.05
            and s["mean_force_index"] < baseline["mean_force_index"] - 0.02
        ]
        if winners:
            recommendation = "KEEP"
            rationale = (
                "At least one structured family outperforms random baseline on both "
                "delay score and force index."
            )
        else:
            losers = [
                fam
                for fam, s in by_family.items()
                if fam != "random_baseline"
                and s["mean_delay_score"] < baseline["mean_delay_score"] - 0.05
            ]
            if len(losers) >= 3:
                recommendation = "KILL"
                rationale = "Most structured families underperform random baseline."
            else:
                recommendation = "REFINE"
                rationale = "No robust separation from random baseline at this scale."

    signature_notes = detect_signatures(rows)
    top = summary[0]["family"] if summary else "n/a"
    report = []
    report.append("# Ramsey Recursive Exploratory Study")
    report.append("")
    report.append("## Scope")
    report.append(
        f"- Families: {', '.join(FAMILY_BUILDERS.keys())}"
    )
    report.append(f"- Sizes: {sizes}")
    report.append(f"- Seeds: {seeds}")
    report.append("")
    report.append("## Metric definitions")
    report.append(
        "- `max_mono_clique_present`: max of largest red clique and largest blue clique."
    )
    report.append(
        "- `max_mono_independent_present`: max monochromatic independent set size (dual of clique in 2-color complete graphs)."
    )
    report.append("- `delay_score = n / max_mono_clique_present` (higher is better).")
    report.append("- `force_index = max_mono_clique_present / n` (lower is better).")
    report.append(
        "- Spectral metrics are from red adjacency matrix eigenvalues (cheap, exact eigensolve)."
    )
    report.append("")
    report.append("## Family ranking (by mean delay score)")
    for s in summary:
        report.append(
            f"- {s['family']}: mean_delay={s['mean_delay_score']:.3f}, "
            f"mean_force={s['mean_force_index']:.3f}, "
            f"mean_max_clique={s['mean_max_mono_clique_present']:.3f}"
        )
    report.append("")
    report.append("## Signature notes")
    for note in signature_notes:
        report.append(f"- {note}")
    report.append("")
    report.append("## Recommendation")
    report.append(f"- Decision: **{recommendation}**")
    report.append(f"- Rationale: {rationale}")
    report.append(
        f"- Best observed family in this screen run: **{top}** (exploratory only; not a theorem)."
    )
    report.append("")
    report.append("## Honesty / limits")
    report.append(
        "- This is a small finite-n exploratory screen and does not establish asymptotic Ramsey bounds."
    )
    report.append(
        "- Family definitions here are heuristic constructions; different recursion rules could change outcomes."
    )
    report.append(
        "- Independent-set and clique metrics are tightly coupled in 2-color complete graphs."
    )
    report_text = "\n".join(report) + "\n"
    path.write_text(report_text, encoding="utf-8")
    return recommendation


def write_run_metadata(path: Path, rows: List[InstanceResult], recommendation: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "instances": len(rows),
        "families": sorted(set(r.family for r in rows)),
        "n_values": sorted(set(r.n for r in rows)),
        "seeds": sorted(set(r.seed for r in rows)),
        "recommendation": recommendation,
    }
    path.write_text(json.dumps(payload, indent=2), encoding="utf-8")


def parse_sizes(s: str) -> List[int]:
    vals = []
    for x in s.split(","):
        x = x.strip()
        if x:
            vals.append(int(x))
    return vals


def parse_seeds(s: str) -> List[int]:
    vals = []
    for x in s.split(","):
        x = x.strip()
        if x:
            vals.append(int(x))
    return vals


def main() -> None:
    parser = argparse.ArgumentParser(description="Ramsey recursive exploratory study")
    parser.add_argument(
        "--sizes",
        default="8,10,12,14,16,18,20,22,24",
        help="Comma-separated n values",
    )
    parser.add_argument(
        "--seeds",
        default="0,1,2,3",
        help="Comma-separated seeds",
    )
    parser.add_argument(
        "--outdir",
        default="results",
        help="Output directory",
    )
    args = parser.parse_args()

    sizes = parse_sizes(args.sizes)
    seeds = parse_seeds(args.seeds)
    outdir = Path(args.outdir)
    rows: List[InstanceResult] = []

    for family in FAMILY_BUILDERS:
        for n in sizes:
            for seed in seeds:
                rows.append(evaluate_instance(family, n, seed))

    instance_csv = outdir / "tables" / "ramsey_recursive_instances.csv"
    summary_csv = outdir / "tables" / "ramsey_recursive_summary.csv"
    report_md = outdir / "reports" / "ramsey_recursive_writeup.md"
    run_json = outdir / "raw" / "ramsey_recursive_run.json"

    write_instance_csv(instance_csv, rows)
    summary = aggregate(rows)
    write_summary_csv(summary_csv, summary)
    recommendation = write_report(report_md, summary, rows, sizes, seeds)
    write_run_metadata(run_json, rows, recommendation)

    print(f"Wrote: {instance_csv}")
    print(f"Wrote: {summary_csv}")
    print(f"Wrote: {report_md}")
    print(f"Wrote: {run_json}")
    print(f"Recommendation: {recommendation}")


if __name__ == "__main__":
    main()
