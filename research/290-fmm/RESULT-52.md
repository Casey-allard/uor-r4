## §5.2 gate — negative. The far-field operator cannot reach usable precision, and there is no asymptotic win to buy.

Per §10 ("a recorded answer either way"), recording a negative. This is the
offline precision check that **gates** §5.2's accuracy measurement, not §5.2
itself: held-out top-1 / bits-token on the Graph slice was never run, because the
precondition it rests on — that a compressed far field reproduces the
interaction operator at a stated tolerance — fails first. Nothing was built into
the deployed path, the artifact format, or the kernel; total cost was one SVD per
admissible pair.

### Method

Measurement-only, on the same bit-for-bit cover as §5.1 (`cover_kappa
blake3:67d957c3…`, 362 regions, 159,658 train observations; all three pinned
kappas re-verified). Admissible pairs recomputed and matching §5.1 exactly:
**2 / 5 / 16 / 29 / 87** at depths 1–5, η = 1.0.

Error is measured against the **truncated SVD**, which by Eckart–Young is the
optimal rank-k approximation. That makes every number here an *upper bound on
achievable quality*: any practical scheme — ACA, cross approximation,
interpolatory bases — does worse. If optimal rank-k misses, nothing cheaper hits.

Reported quantity is operator error, `||Kx − K̃x|| / ||Kx||`, median over 79
admissible pairs at n = 1024 with 32 random right-hand sides. An ℋ-matrix is
*applied*; Frobenius error of an isolated block would flatter the result.

### Numbers

| rank | cosine: rel Frobenius | σ_k/σ₁ | **rel matvec** | sign-bit: rel Frobenius | σ_k/σ₁ | **rel matvec** |
|---:|---:|---:|---:|---:|---:|---:|
| 1 (monopole) | 10.81% | 5.32% | **15.02%** | 6.88% | 3.07% | **10.29%** |
| 8 | 6.28% | 1.84% | **8.97%** | 4.72% | 0.95% | **6.97%** |
| 16 | 4.55% | 1.12% | **6.64%** | 4.13% | 0.67% | **6.05%** |
| 20 | 4.04% | 0.92% | **5.97%** | 3.93% | 0.61% | **5.82%** |
| 32 | 2.98% | 0.63% | **4.53%** | 3.44% | 0.50% | **4.98%** |
| 64 | 1.61% | 0.30% | **2.45%** | 2.53% | 0.34% | **3.68%** |

Rank 1 reproduces §5.1's monopole figure (σ₂/σ₁ = 5.32%, against the recorded
5–7%), and σ₂₁/σ₁ = 0.92% reproduces `r(1e-2) ≈ 20` exactly. The pipeline agrees
with §5.1 wherever the two overlap.

### Finding 1 — `r(ε) ≈ 20` is not a 1% error budget

`r(ε) = #{σᵢ ≥ ε·σ₁}` counts spectrum above a threshold. It was never an error
bound, and the two differ by ~6× here: at rank 20 the *next* singular value is
0.92% of the first, while discarding the tail costs 4.0% Frobenius and **~6%
operator error**.

My own §5.2 pre-registration earlier in this thread inherited that ambiguity
when it wrote "a ~1%-precise, rank-~20 far field." Those are two different
quantities. Correcting it here: there is no rank at which this far field is 1%
precise without approaching the ambient dimension.

Against the bar that matters, rank 20 beats the Barnes-Hut monopole by only
**2.5× (cosine) / 1.8× (sign-bit)** — for the whole M2L apparatus, admissibility
bookkeeping, and compile-time translation tables.

### Finding 2 — the kernel is already globally low-rank, so there is no O(n²) to remove

This is the structural objection and it does not depend on any tolerance.

The interaction block, as §5.1 operationally defines it, is

```
K_AB = V_A V_Bᵀ
```

an inner product of L2-normalized threshold-centered context bundles in
D = 288 dimensions. It is therefore **exactly rank ≤ 288 by construction** —
§5.1 already noted this ceiling in passing, observing r(1e-4) ≈ 275 approaching
it. The exact operator application factors by associativity:

```
K x = V_A (V_Bᵀ x)        cost O((n_A + n_B)·D), exact, no approximation
```

FMM exists to accelerate kernels such as 1/r that are *not* globally low rank;
its win comes from replacing a dense O(n²) interaction that has no exact
factorization. Here the interaction is an inner-product kernel that already has
one. Any far-field operator must therefore beat rank 288 to earn anything at all,
and at rank 64 the error is still 2.5–3.7% — roughly 4.5× compression at a
precision no one pre-registered as acceptable. Driving error toward 1% pushes the
rank back toward the ambient dimension, where the compression vanishes.

The premise in §2 — "three of the six FMM stages are already built" — holds. What
does not hold is the assumption that the missing stage buys an asymptotic
improvement. Against this kernel it buys a constant factor bounded by D, and
§5.4 already flagged constant factors as "the usual assassin."

### Incidental correction to §5.1

§5.1 recorded the deployed sign-bit / masked-Hamming geometry as ~2× worse than
cosine (median rank 266–279 vs 134 at 1e-3) and inferred that binarization
destroys low-rank structure. In **reconstruction-error** terms that does not
hold: 5.82% vs 5.97% at rank 20, essentially identical, and sign-bit is *better*
at rank 1 (10.29% vs 15.02%). The ~2× gap is real in rank counts and absent in
error. The #230-style concern is not supported by this measurement.

### What this does and does not kill

**Killed:** FMM / ℋ-matrix / HODLR far-field aggregation over this interaction
object. Per §7 the sequencing was ℋ-matrix before FMM precisely so this could be
answered cheaply; it has been, and full FMM does not need evaluating.

**Not killed:** the §3a motivation. Long-range context remains a real gap — the
graph-path slice sits at 1.23% top-1 (#234's closure names it "the number to
improve"). This result says a far-field operator over the semantic Gram is not
the mechanism, not that the goal is wrong.

### Caveat on Finding 2

It rests on the interaction object being the Gram block of 288-dim bundles, which
is how §5.1 operationalized it and how a far field would be constructed. If the
intended object is something else — a kernel that is not an inner product of
bounded-dimension vectors — the associativity argument does not transfer and this
should be re-examined. Flagging rather than asserting it as settled.

### Reproduction

Branch `issue-290-hmatrix-prototype`, `research/290-fmm/` — `fmm_dump.rs`
(extractor, self-verifying against the pinned kappas), `validate_51.py` (§5.1
reproduction gate), `hmatrix_proto.py` (this measurement). Tracked deliberately:
the original §5.1 analysis lived in an untracked scratch directory and was
destroyed by a working-tree cleanup, which is why the reproduction status and
both known discrepancies are written into the README rather than left implicit.
