#!/usr/bin/env python3
"""Reproduce the §5.1 published numbers from a fresh `fmm_dump` (issue #290).

§5.1's original analysis lived in an untracked scratch directory and was lost to
a working-tree cleanup. The extractor (`fmm_dump.rs`) survived; this rebuilds the
analysis side and checks it against the figures already published in the issue,
so §5.2 is not built on an unvalidated pipeline.

Targets from the #290 §5.1 comment (cosine kernel, eta = 1.0):

  admissible same-depth pairs   2 / 5 / 16 / 29 / 87   at depths 1..5
  within-pair nested medians, r(eps) at n = 1024:
      eps=1e-2 -> ~20.5      eps=1e-3 -> ~134.0     eps=1e-4 -> ~274.5
  median growth exponents      0.11 / 0.45 / 0.64
  null control at n = 1024     57 / 198 / 287        (seed 290501)

Operational definitions restated from the issue:

  interaction block  submatrix of the semantic Gram matrix K(x,y) = <v(x),v(y)>
                     over the cover's top-1 partition, on L2-normalized
                     threshold-centered context bundles
  admissibility      arccos(<p_A,p_B>) > eta * max(diam), diam = 2 * q95 of
                     member angular radius
  numerical rank     r(eps) = #{sigma_i >= eps * sigma_1}

Usage: validate_51.py <dump_dir> [--eta 1.0]
"""

import argparse
import json
import pathlib
import sys

import numpy as np

EPSILONS = (1e-2, 1e-3, 1e-4)
NESTED_SIZES = (32, 64, 128, 256, 512, 1024)
NULL_SEED = 290501
MAX_BLOCK_ROWS = 1024


def load(dump_dir: pathlib.Path):
    meta = json.loads((dump_dir / "meta.json").read_text())
    if not meta.get("reproduction_verified"):
        sys.exit("dump reports reproduction_verified = false; refusing to analyze")
    d = meta["d"]
    regions = json.loads((dump_dir / "regions.json").read_text())
    vectors = np.fromfile(dump_dir / "vectors.bin", dtype="<f4").reshape(-1, d)
    prototypes = np.fromfile(dump_dir / "prototypes.bin", dtype="<f4").reshape(-1, d)
    if vectors.shape[0] != meta["n_train"]:
        sys.exit(f"vectors.bin has {vectors.shape[0]} rows, meta says {meta['n_train']}")
    if prototypes.shape[0] != meta["n_regions"]:
        sys.exit("prototypes.bin row count disagrees with meta")
    return meta, regions, vectors, prototypes


def l2_normalize(mat: np.ndarray) -> np.ndarray:
    norms = np.linalg.norm(mat, axis=1, keepdims=True)
    norms[norms == 0.0] = 1.0
    return mat / norms


def numerical_rank(block: np.ndarray) -> dict:
    # Gram blocks are symmetric PSD only when A == B; use plain SVD for the
    # general off-diagonal case.
    sv = np.linalg.svd(block, compute_uv=False)
    if sv.size == 0 or sv[0] == 0.0:
        return {eps: 0 for eps in EPSILONS}
    return {eps: int(np.count_nonzero(sv >= eps * sv[0])) for eps in EPSILONS}


def member_angular_radius(unit_vectors: np.ndarray, prototype: np.ndarray) -> float:
    """diam = 2 * q95 of member angular distance to the region prototype."""
    p = prototype / (np.linalg.norm(prototype) or 1.0)
    cos = np.clip(unit_vectors @ p, -1.0, 1.0)
    return 2.0 * float(np.quantile(np.arccos(cos), 0.95))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("dump_dir", type=pathlib.Path)
    ap.add_argument("--eta", type=float, default=1.0)
    args = ap.parse_args()

    meta, regions, vectors, prototypes = load(args.dump_dir)
    print(f"dump: n_train={meta['n_train']} regions={meta['n_regions']} "
          f"per_depth={meta['per_depth']}")
    print(f"cover_kappa={meta['cover_kappa']}")

    units = l2_normalize(vectors.astype(np.float64))
    protos = l2_normalize(prototypes.astype(np.float64))

    by_depth: dict[int, list[int]] = {}
    for r in regions:
        by_depth.setdefault(r["depth"], []).append(r["id"])

    # Subsampling applies to BLOCK CONSTRUCTION only. The q95 member angular
    # radius that sets admissibility is taken over full membership: computing it
    # on the subsample shifts the threshold and changes which pairs qualify,
    # which is where a first rebuild diverged from §5.1 at depths 4 and 5 (the
    # depths with the largest regions).
    full_members = {r["id"]: np.asarray(r["members"], dtype=np.int64) for r in regions}
    members = {}
    for rid, m in full_members.items():
        if m.size > MAX_BLOCK_ROWS:
            stride = m.size / MAX_BLOCK_ROWS
            idx = (np.arange(MAX_BLOCK_ROWS) * stride).astype(np.int64)
            m = m[idx]
        members[rid] = m

    diam = {}
    for r in regions:
        m = full_members[r["id"]]
        diam[r["id"]] = (
            member_angular_radius(units[m], protos[r["id"]]) if m.size else 0.0
        )

    print(f"\n=== admissible same-depth pairs (eta = {args.eta}) ===")
    admissible: list[tuple[int, int, int]] = []
    for depth in sorted(by_depth):
        ids = by_depth[depth]
        count = 0
        for i, a in enumerate(ids):
            for b in ids[i + 1:]:
                sep = np.arccos(np.clip(float(protos[a] @ protos[b]), -1.0, 1.0))
                if sep > args.eta * max(diam[a], diam[b]):
                    admissible.append((depth, a, b))
                    count += 1
        total = len(ids) * (len(ids) - 1) // 2
        pct = 100.0 * count / total if total else 0.0
        print(f"  depth {depth}: {count} / {total} pairs ({pct:.2f}%)")

    if not admissible:
        print("\nno admissible pairs; nothing further to measure")
        return

    print(f"\n=== within-pair nested scaling ({len(admissible)} pairs) ===")
    rows: dict[int, dict[float, list[int]]] = {
        n: {eps: [] for eps in EPSILONS} for n in NESTED_SIZES
    }
    slopes: dict[float, list[float]] = {eps: [] for eps in EPSILONS}

    for depth, a, b in admissible:
        ma, mb = members[a], members[b]
        # §5.1 fits growth over the full nested ladder; a pair qualifies only
        # if both regions reach the largest block size. Accepting short ladders
        # inflates the pair count (103 vs the published 37) and biases the
        # median exponent toward small-n behavior.
        usable = [n for n in NESTED_SIZES if ma.size >= n and mb.size >= n]
        if len(usable) != len(NESTED_SIZES):
            continue
        per_eps: dict[float, list[tuple[int, int]]] = {eps: [] for eps in EPSILONS}
        for n in usable:
            block = units[ma[:n]] @ units[mb[:n]].T
            ranks = numerical_rank(block)
            for eps in EPSILONS:
                rows[n][eps].append(ranks[eps])
                per_eps[eps].append((n, ranks[eps]))
        for eps in EPSILONS:
            pts = [(n, r) for n, r in per_eps[eps] if r > 0]
            if len(pts) >= 3:
                xs = np.log(np.array([p[0] for p in pts], dtype=float))
                ys = np.log(np.array([p[1] for p in pts], dtype=float))
                slopes[eps].append(float(np.polyfit(xs, ys, 1)[0]))

    print(f"{'n':>6} | " + " | ".join(f"r({e:g})".rjust(12) for e in EPSILONS))
    for n in NESTED_SIZES:
        cells = []
        for eps in EPSILONS:
            vals = rows[n][eps]
            if vals:
                med = float(np.median(vals))
                cells.append(f"{med:6.1f} ({med / n:4.2f})")
            else:
                cells.append(" " * 12)
        print(f"{n:>6} | " + " | ".join(cells))

    print("\n=== median growth exponents (log-log slope of r vs n) ===")
    for eps in EPSILONS:
        if slopes[eps]:
            print(f"  eps={eps:g}: n^{float(np.median(slopes[eps])):.3f} "
                  f"({len(slopes[eps])} pairs)")

    print(f"\n=== null control (random subsets, n = {MAX_BLOCK_ROWS}, "
          f"seed {NULL_SEED}) ===")
    rng = np.random.default_rng(NULL_SEED)
    total = units.shape[0]
    null: dict[float, list[int]] = {eps: [] for eps in EPSILONS}
    for _ in range(16):
        ia = rng.choice(total, size=MAX_BLOCK_ROWS, replace=False)
        ib = rng.choice(total, size=MAX_BLOCK_ROWS, replace=False)
        ranks = numerical_rank(units[ia] @ units[ib].T)
        for eps in EPSILONS:
            null[eps].append(ranks[eps])
    for eps in EPSILONS:
        print(f"  eps={eps:g}: median r = {float(np.median(null[eps])):.1f}")

    print("\nCompare against the §5.1 figures in the docstring before using this "
          "pipeline for §5.2.")


if __name__ == "__main__":
    main()
