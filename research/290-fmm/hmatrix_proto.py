#!/usr/bin/env python3
"""§5.2 offline prototype: can a rank-k far field reproduce the scores? (#290)

Measurement only. Nothing here touches the deployed path or the artifact format
-- §5.1 showed compressibility is a ~1%-tolerance phenomenon that vanishes by
1e-4, so the cheapest place to kill this proposal is offline, before any M2L
machinery or compile-time tables exist.

What is measured, and why this ordering:

1. **Optimal rank-k block error** via truncated SVD. By Eckart-Young this is the
   best any rank-k approximation can do, so it is an upper bound on every
   practical scheme (ACA, cross approximation, interpolatory). If optimal rank-20
   cannot reach ~1%, no cheaper method will, and the proposal dies here for the
   cost of one SVD per admissible pair.
2. **Monopole (Barnes-Hut) baseline** at rank 1 -- the bar §5.1 recorded at
   sigma_2/sigma_1 ~ 5-7%. Any FMM/H-matrix machinery must beat this by enough to
   justify itself.
3. **Operator error, not just block error.** An H-matrix is applied, so the
   quantity that matters downstream is ||Kx - K~x|| / ||Kx|| for the far-field
   aggregation, not the Frobenius error of an isolated block.
4. **Both kernels.** Cosine is the geometry §5.1 measured; the DEPLOYED kernel is
   sign-bit / masked-Hamming, which §5.1 found ~2x worse at 1e-3 (median rank
   266-279 vs 134). A far field that only works in cosine is not deployable, and
   the #230 precedent says mul-free surrogates have already cost us structure
   once.

Usage: hmatrix_proto.py <dump_dir> [--eta 1.0] [--ranks 1,8,16,20,32,64]
"""

import argparse
import json
import pathlib
import sys

import numpy as np

NESTED_N = 1024
MATVEC_SEED = 290502  # distinct from §5.1's null-control seed (290501)
MATVEC_TRIALS = 32


def load(dump_dir: pathlib.Path):
    meta = json.loads((dump_dir / "meta.json").read_text())
    if not meta.get("reproduction_verified"):
        sys.exit("dump reports reproduction_verified = false; refusing to analyze")
    d = meta["d"]
    regions = json.loads((dump_dir / "regions.json").read_text())
    vectors = np.fromfile(dump_dir / "vectors.bin", dtype="<f4").reshape(-1, d)
    prototypes = np.fromfile(dump_dir / "prototypes.bin", dtype="<f4").reshape(-1, d)
    sigs = np.fromfile(dump_dir / "sigs.bin", dtype=np.uint8).reshape(
        -1, meta["sig_bytes"]
    )
    return meta, regions, vectors, prototypes, sigs


def l2_normalize(mat: np.ndarray) -> np.ndarray:
    norms = np.linalg.norm(mat, axis=1, keepdims=True)
    norms[norms == 0.0] = 1.0
    return mat / norms


def member_angular_radius(units: np.ndarray, prototype: np.ndarray) -> float:
    p = prototype / (np.linalg.norm(prototype) or 1.0)
    return 2.0 * float(np.quantile(np.arccos(np.clip(units @ p, -1.0, 1.0)), 0.95))


def admissible_pairs(regions, protos, units, full_members, eta):
    """arccos(<p_A,p_B>) > eta * max(diam); diam over FULL membership.

    The full-membership scope is load-bearing: computing the q95 radius on the
    subsample shifts the threshold and silently changes which pairs qualify.
    See README.md.
    """
    diam = {
        r["id"]: (
            member_angular_radius(units[full_members[r["id"]]], protos[r["id"]])
            if full_members[r["id"]].size
            else 0.0
        )
        for r in regions
    }
    by_depth: dict[int, list[int]] = {}
    for r in regions:
        by_depth.setdefault(r["depth"], []).append(r["id"])
    out = []
    for depth, ids in sorted(by_depth.items()):
        for i, a in enumerate(ids):
            for b in ids[i + 1:]:
                sep = np.arccos(np.clip(float(protos[a] @ protos[b]), -1.0, 1.0))
                if sep > eta * max(diam[a], diam[b]):
                    out.append((depth, a, b))
    return out


def sign_bit_kernel(sig_a: np.ndarray, sig_b: np.ndarray) -> np.ndarray:
    """Deployed geometry: masked Hamming similarity over packed sign bits.

    Returned as a similarity in [0,1] (1 - normalized Hamming distance) so its
    spectrum is comparable to the cosine Gram block.
    """
    bits_a = np.unpackbits(sig_a, axis=1).astype(np.float64)
    bits_b = np.unpackbits(sig_b, axis=1).astype(np.float64)
    nbits = bits_a.shape[1]
    # Hamming distance via inner products on +-1 encoding.
    pm_a = 2.0 * bits_a - 1.0
    pm_b = 2.0 * bits_b - 1.0
    agree = (pm_a @ pm_b.T + nbits) / 2.0
    return agree / nbits


def block_errors(block: np.ndarray, ranks, rng) -> dict:
    u, sv, vt = np.linalg.svd(block, full_matrices=False)
    fro = float(np.linalg.norm(sv))
    if fro == 0.0:
        return {}
    x = rng.standard_normal((block.shape[1], MATVEC_TRIALS))
    exact = block @ x
    exact_norm = np.linalg.norm(exact, axis=0)
    exact_norm[exact_norm == 0.0] = 1.0

    out = {}
    for k in ranks:
        kk = min(k, sv.size)
        approx = (u[:, :kk] * sv[:kk]) @ vt[:kk]
        tail = float(np.linalg.norm(sv[kk:])) if kk < sv.size else 0.0
        rel_fro = tail / fro
        rel_spec = float(sv[kk] / sv[0]) if kk < sv.size else 0.0
        rel_mv = float(
            np.median(np.linalg.norm(exact - approx @ x, axis=0) / exact_norm)
        )
        out[k] = (rel_fro, rel_spec, rel_mv)
    return out


def summarize(label, per_pair, ranks):
    print(f"\n=== {label} ===")
    print(f"{'rank':>5} | {'rel Frobenius':>14} | {'sigma_k/sigma_1':>15} | "
          f"{'rel matvec':>12}")
    print("-" * 56)
    for k in ranks:
        fr = [p[k][0] for p in per_pair if k in p]
        sp = [p[k][1] for p in per_pair if k in p]
        mv = [p[k][2] for p in per_pair if k in p]
        if not fr:
            continue
        print(f"{k:>5} | {np.median(fr):13.4%} | {np.median(sp):14.4%} | "
              f"{np.median(mv):11.4%}")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("dump_dir", type=pathlib.Path)
    ap.add_argument("--eta", type=float, default=1.0)
    ap.add_argument("--ranks", default="1,8,16,20,32,64")
    args = ap.parse_args()
    ranks = [int(x) for x in args.ranks.split(",")]

    meta, regions, vectors, protos_raw, sigs = load(args.dump_dir)
    print(f"dump n_train={meta['n_train']} regions={meta['n_regions']}")
    print(f"cover_kappa={meta['cover_kappa']}")

    units = l2_normalize(vectors.astype(np.float64))
    protos = l2_normalize(protos_raw.astype(np.float64))
    full_members = {r["id"]: np.asarray(r["members"], dtype=np.int64) for r in regions}

    pairs = admissible_pairs(regions, protos, units, full_members, args.eta)
    print(f"admissible pairs (eta={args.eta}): {len(pairs)}")

    usable = [
        (d, a, b)
        for d, a, b in pairs
        if full_members[a].size >= NESTED_N and full_members[b].size >= NESTED_N
    ]
    print(f"pairs with >= {NESTED_N} members on both sides: {len(usable)}")

    rng = np.random.default_rng(MATVEC_SEED)
    cos_pairs, sig_pairs = [], []
    for _, a, b in usable:
        ma = full_members[a][:NESTED_N]
        mb = full_members[b][:NESTED_N]
        cos_block = units[ma] @ units[mb].T
        e = block_errors(cos_block, ranks, rng)
        if e:
            cos_pairs.append(e)
        e2 = block_errors(sign_bit_kernel(sigs[ma], sigs[mb]), ranks, rng)
        if e2:
            sig_pairs.append(e2)

    summarize(f"cosine kernel (n={NESTED_N}, {len(cos_pairs)} pairs)", cos_pairs, ranks)
    summarize(
        f"DEPLOYED sign-bit/masked-Hamming kernel (n={NESTED_N}, {len(sig_pairs)} pairs)",
        sig_pairs,
        ranks,
    )

    print("\nReading guide:")
    print("  rank 1 == monopole / Barnes-Hut baseline (§5.1 recorded 5-7%).")
    print("  'rel matvec' is the operator error a far-field aggregation incurs;")
    print("  it is the number §5.2 accuracy claims must ultimately rest on.")
    print("  A far field usable only in cosine is not deployable: the shipped")
    print("  kernel is the sign-bit one (§5.1: ~2x worse at 1e-3; cf. #230).")


if __name__ == "__main__":
    main()
