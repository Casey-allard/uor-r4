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
from statistics import mean, stdev
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


def fibonacci_block_growth(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
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

        graphs.append(combine_blocks(left, right, cross))
        sizes.append(sizes[-1] + sizes[-2])
    g = [row[:n] for row in graphs[-1][:n]]
    ratio = sizes[-2] / sizes[-3] if len(sizes) >= 3 else 1.0
    return g, {
        "signature_name": "fib_block_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def fibonacci_mirrored_growth(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    rng = random.Random(seed + 313)
    offset = rng.random()
    parity_flip = (seed + 1) % 2
    g1 = [[0]]
    g2 = [[0, 1], [1, 0]]
    sizes = [1, 2]
    graphs = [g1, g2]
    while sizes[-1] < n:
        k = len(sizes) + 1
        # mirrored: swap block order from base construction.
        left = graphs[-2]
        right = graphs[-1]

        def cross(i: int, j: int, a: int, b: int) -> bool:
            x = (((i + 1) * (PHI - 1.0) + (j + 1) * PHI + offset) % 1.0) < 0.5
            return x if (k + parity_flip) % 2 == 0 else (not x)

        graphs.append(combine_blocks(left, right, cross))
        sizes.append(sizes[-1] + sizes[-2])
    g = [row[:n] for row in graphs[-1][:n]]
    ratio = sizes[-2] / sizes[-3] if len(sizes) >= 3 else 1.0
    return g, {
        "signature_name": "fib_mirrored_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def fibonacci_phi_partition(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    mat = [[0] * n for _ in range(n)]
    rng = random.Random(seed + 911)
    offset = rng.random()

    def fill(lo: int, hi: int, depth: int) -> None:
        size = hi - lo
        if size <= 1:
            return
        left_size = max(1, min(size - 1, round(size / PHI)))
        mid = lo + left_size
        red = ((depth + seed) % 2 == 0)
        for i in range(lo, mid):
            for j in range(mid, hi):
                hash_bit = (((i + 1) * PHI + (j + 1) * (PHI - 1.0) + offset) % 1.0) < 0.5
                set_edge(mat, i, j, red if hash_bit else (not red))
        fill(lo, mid, depth + 1)
        fill(mid, hi, depth + 1)

    fill(0, n, 0)
    # mean realized left/right ratio across levels (single split proxy at root for simplicity)
    root_left = max(1, min(n - 1, round(n / PHI)))
    root_right = n - root_left
    ratio = max(root_left, root_right) / max(1, min(root_left, root_right))
    return mat, {
        "signature_name": "phi_partition_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def fibonacci_offbyone_growth(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
    rng = random.Random(seed + 1229)
    offset = rng.random()
    g_prev2 = [[0]]
    g_prev1 = [[0, 1], [1, 0]]
    sizes = [1, 2]
    graphs = [g_prev2, g_prev1]

    while sizes[-1] < n:
        k = len(sizes) + 1
        left = graphs[-1]
        right = graphs[-2]

        def cross(i: int, j: int, a: int, b: int) -> bool:
            x = (((i + 1) * PHI + (j + 1) * PHI + offset) % 1.0) < 0.5
            return x if (k + seed) % 2 == 0 else (not x)

        combined = combine_blocks(left, right, cross)

        # off-by-one perturbation: add one extra vertex each growth step.
        m = len(combined)
        ext = [[0] * (m + 1) for _ in range(m + 1)]
        for i in range(m):
            for j in range(m):
                ext[i][j] = combined[i][j]
        extra = m
        for i in range(m):
            red = (((i + 1) * (PHI - 1.0) + offset) % 1.0) < 0.5
            set_edge(ext, i, extra, red)

        graphs.append(ext)
        sizes.append(sizes[-1] + sizes[-2] + 1)

    g = [row[:n] for row in graphs[-1][:n]]
    ratio = sizes[-2] / sizes[-3] if len(sizes) >= 3 else 1.0
    return g, {
        "signature_name": "fib_offbyone_ratio",
        "signature_value": ratio,
        "phi_distance": abs(ratio - PHI),
    }


def balanced_binary_partitions(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
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


def random_baseline(n: int, seed: int) -> Tuple[List[List[int]], Dict[str, float]]:
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


FAMILIES = {
    "fibonacci_block_growth": fibonacci_block_growth,
    "fibonacci_mirrored_growth": fibonacci_mirrored_growth,
    "fibonacci_phi_partition": fibonacci_phi_partition,
    "fibonacci_offbyone_growth": fibonacci_offbyone_growth,
    "balanced_binary_partitions": balanced_binary_partitions,
    "random_baseline": random_baseline,
}


def matrix_to_masks(mat: List[List[int]]) -> List[int]:
    n = len(mat)
    masks = [0] * n
    for i in range(n):
        m = 0
        for j in range(n):
            if i != j and mat[i][j]:
                m |= 1 << j
        masks[i] = m
    return masks


def complement_masks(masks: List[int], n: int) -> List[int]:
    all_bits = (1 << n) - 1
    return [(~masks[i]) & (all_bits ^ (1 << i)) for i in range(n)]


def max_clique_size(masks: List[int]) -> int:
    n = len(masks)
    best = 0

    def dfs(candidates: int, size: int) -> None:
        nonlocal best
        if not candidates:
            best = max(best, size)
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
        return (
            float(np.max(np.abs(eigs))),
            float(eigs[-1] - eigs[-2]) if n >= 2 else 0.0,
            float(np.sum(np.abs(eigs))),
        )
    row_sums = [sum(r) for r in mat]
    return float(max(row_sums)), 0.0, float(sum(abs(x) for x in row_sums))


def evaluate(family: str, n: int, seed: int) -> InstanceResult:
    mat, signature = FAMILIES[family](n, seed)
    masks_red = matrix_to_masks(mat)
    masks_blue = complement_masks(masks_red, n)
    omega_red = max_clique_size(masks_red)
    omega_blue = max_clique_size(masks_blue)
    max_mono_clique_present = max(omega_red, omega_blue)
    max_mono_independent_present = max_mono_clique_present
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


def parse_int_list(s: str) -> List[int]:
    return [int(x.strip()) for x in s.split(",") if x.strip()]


def write_csv(path: Path, rows: List[dict]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not rows:
        return
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)


def summarize(rows: List[InstanceResult]) -> List[dict]:
    by_family: Dict[str, List[InstanceResult]] = defaultdict(list)
    for r in rows:
        by_family[r.family].append(r)
    out = []
    for family, items in by_family.items():
        out.append(
            {
                "family": family,
                "instances": len(items),
                "mean_max_mono_clique_present": mean([x.max_mono_clique_present for x in items]),
                "mean_max_mono_independent_present": mean(
                    [x.max_mono_independent_present for x in items]
                ),
                "mean_delay_score": mean([x.delay_score for x in items]),
                "mean_force_index": mean([x.force_index for x in items]),
                "mean_red_density": mean([x.red_density for x in items]),
                "mean_spectral_radius": mean([x.spectral_radius for x in items]),
                "mean_spectral_gap": mean([x.spectral_gap for x in items]),
                "mean_adjacency_energy": mean([x.adjacency_energy for x in items]),
                "mean_signature_phi_distance": mean(
                    [x.signature_phi_distance for x in items if not math.isnan(x.signature_phi_distance)]
                )
                if any(not math.isnan(x.signature_phi_distance) for x in items)
                else float("nan"),
            }
        )
    out.sort(key=lambda d: (-d["mean_delay_score"], d["mean_force_index"]))
    return out


def paired_diffs(base: List[InstanceResult], comp: List[InstanceResult], metric: str) -> List[float]:
    bmap = {(x.n, x.seed): x for x in base}
    cmap = {(x.n, x.seed): x for x in comp}
    diffs = []
    for k in sorted(set(bmap.keys()) & set(cmap.keys())):
        diffs.append(getattr(bmap[k], metric) - getattr(cmap[k], metric))
    return diffs


def mean_ci(vals: List[float]) -> Tuple[float, float, float]:
    m = mean(vals)
    if len(vals) <= 1:
        return m, m, m
    s = stdev(vals)
    half = 1.96 * s / math.sqrt(len(vals))
    return m, m - half, m + half


def robustness(rows: List[InstanceResult]) -> List[dict]:
    by_family: Dict[str, List[InstanceResult]] = defaultdict(list)
    for r in rows:
        by_family[r.family].append(r)
    fib = by_family["fibonacci_block_growth"]
    comparators = [
        "random_baseline",
        "balanced_binary_partitions",
        "fibonacci_mirrored_growth",
        "fibonacci_phi_partition",
        "fibonacci_offbyone_growth",
    ]
    out = []
    for comp_name in comparators:
        comp = by_family[comp_name]
        d_delay = paired_diffs(fib, comp, "delay_score")
        d_force = paired_diffs(fib, comp, "force_index")
        m_delay, lo_delay, hi_delay = mean_ci(d_delay)
        m_force, lo_force, hi_force = mean_ci(d_force)
        wins_delay = sum(1 for x in d_delay if x > 0) / len(d_delay)
        wins_force = sum(1 for x in d_force if x < 0) / len(d_force)
        out.append(
            {
                "comparison": f"fibonacci_block_growth - {comp_name}",
                "paired_instances": len(d_delay),
                "mean_delay_diff": m_delay,
                "delay_diff_ci_low": lo_delay,
                "delay_diff_ci_high": hi_delay,
                "mean_force_diff": m_force,
                "force_diff_ci_low": lo_force,
                "force_diff_ci_high": hi_force,
                "delay_win_rate": wins_delay,
                "force_win_rate": wins_force,
                "robust_better_delay": lo_delay > 0.0,
                "robust_better_force": hi_force < 0.0,
            }
        )
    return out


def verdict(robust_rows: List[dict]) -> Tuple[str, str]:
    row = {r["comparison"]: r for r in robust_rows}
    vs_random = row["fibonacci_block_growth - random_baseline"]
    vs_binary = row["fibonacci_block_growth - balanced_binary_partitions"]
    perturbs = [
        row["fibonacci_block_growth - fibonacci_mirrored_growth"],
        row["fibonacci_block_growth - fibonacci_phi_partition"],
        row["fibonacci_block_growth - fibonacci_offbyone_growth"],
    ]
    fib_robust = (
        vs_random["robust_better_delay"]
        and vs_random["robust_better_force"]
        and vs_binary["robust_better_delay"]
        and vs_binary["robust_better_force"]
    )
    perturb_close = sum(
        1
        for p in perturbs
        if abs(p["mean_delay_diff"]) < 0.08 and abs(p["mean_force_diff"]) < 0.015
    )
    perturb_reverse = sum(1 for p in perturbs if p["mean_delay_diff"] < -0.05)
    if fib_robust and perturb_close == 0:
        return "KEEP", "Fibonacci base stays ahead under replication and perturbations do not match it."
    if fib_robust and perturb_close >= 1:
        return (
            "REFINE",
            "Signal persists, but nearby perturbations achieve similar performance; effect may be broader recursive-ratio structure.",
        )
    if perturb_reverse >= 2:
        return "KILL", "Edge is not stable under nearby perturbations."
    return "REFINE", "Evidence is mixed or only partially robust under confirm controls."


def write_report(path: Path, summary_rows: List[dict], robust_rows: List[dict], v: Tuple[str, str], sizes: List[int], seeds: List[int]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    lines.append("# Confirm Pass: Fibonacci Ramsey Signal (v1)")
    lines.append("")
    lines.append("## Setup")
    lines.append("- task_id: `ramsey_fibonacci_confirm_v1`")
    lines.append("- families: fibonacci base, random baseline, balanced-binary control, and 3 nearby Fibonacci perturbations")
    lines.append(f"- sizes: {sizes}")
    lines.append(f"- seeds: {seeds} (count={len(seeds)})")
    lines.append("")
    lines.append("## Family means")
    for row in summary_rows:
        lines.append(
            f"- {row['family']}: delay={row['mean_delay_score']:.4f}, force={row['mean_force_index']:.4f}, max_clique={row['mean_max_mono_clique_present']:.3f}"
        )
    lines.append("")
    lines.append("## Robustness checks (paired by n,seed vs Fibonacci base)")
    for r in robust_rows:
        lines.append(
            f"- {r['comparison']}: mean_delay_diff={r['mean_delay_diff']:.4f} "
            f"[{r['delay_diff_ci_low']:.4f}, {r['delay_diff_ci_high']:.4f}], "
            f"mean_force_diff={r['mean_force_diff']:.4f} "
            f"[{r['force_diff_ci_low']:.4f}, {r['force_diff_ci_high']:.4f}], "
            f"delay_win_rate={r['delay_win_rate']:.3f}, force_win_rate={r['force_win_rate']:.3f}"
        )
    lines.append("")
    lines.append("## Direct answers")
    rmap = {r["comparison"]: r for r in robust_rows}
    a = rmap["fibonacci_block_growth - random_baseline"]
    b = rmap["fibonacci_block_growth - balanced_binary_partitions"]
    lines.append(
        f"- Does Fibonacci still beat random? {'Yes' if a['mean_delay_diff'] > 0 and a['mean_force_diff'] < 0 else 'No/unclear'}."
    )
    lines.append(
        f"- Does it beat strongest non-Fibonacci structured control? {'Yes' if b['mean_delay_diff'] > 0 and b['mean_force_diff'] < 0 else 'No/unclear'}."
    )
    lines.append(
        "- Do nearby perturbations collapse the edge or preserve it? Mixed; see paired differences above."
    )
    lines.append("")
    lines.append("## Verdict")
    lines.append(f"- Decision: **{v[0]}**")
    lines.append(f"- Rationale: {v[1]}")
    lines.append("")
    lines.append("## Limits")
    lines.append("- Finite small-n confirm pass only; this does not establish asymptotic Ramsey behavior.")
    lines.append("- Construction choices are heuristic and could be sensitive to exact recursive assignment rules.")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Confirm pass for Fibonacci Ramsey signal")
    parser.add_argument("--sizes", default="8,10,12,14,16,18,20,22,24,26,28")
    parser.add_argument("--seeds", default="0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15")
    parser.add_argument("--outdir", default="results/confirm/ramsey_fibonacci_confirm_v1")
    args = parser.parse_args()

    sizes = parse_int_list(args.sizes)
    seeds = parse_int_list(args.seeds)
    outdir = Path(args.outdir)

    rows: List[InstanceResult] = []
    for family in FAMILIES:
        for n in sizes:
            for seed in seeds:
                rows.append(evaluate(family, n, seed))

    instance_rows = [r.__dict__ for r in rows]
    summary_rows = summarize(rows)
    robust_rows = robustness(rows)
    v = verdict(robust_rows)

    write_csv(outdir / "tables" / "instances.csv", instance_rows)
    write_csv(outdir / "tables" / "family_summary.csv", summary_rows)
    write_csv(outdir / "tables" / "robustness_summary.csv", robust_rows)
    write_report(outdir / "report.md", summary_rows, robust_rows, v, sizes, seeds)
    (outdir / "raw").mkdir(parents=True, exist_ok=True)
    (outdir / "raw" / "run.json").write_text(
        json.dumps(
            {
                "task_id": "ramsey_fibonacci_confirm_v1",
                "instances": len(rows),
                "families": list(FAMILIES.keys()),
                "sizes": sizes,
                "seeds": seeds,
                "verdict": v[0],
                "verdict_rationale": v[1],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    print(f"Wrote: {outdir / 'tables' / 'instances.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'family_summary.csv'}")
    print(f"Wrote: {outdir / 'tables' / 'robustness_summary.csv'}")
    print(f"Wrote: {outdir / 'report.md'}")
    print(f"Wrote: {outdir / 'raw' / 'run.json'}")
    print(f"Verdict: {v[0]}")


if __name__ == "__main__":
    main()
