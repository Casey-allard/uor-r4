# §5.1 / §5.2 far-field analysis (issue #290)

Tracked deliberately. The original §5.1 analysis lived in an untracked
`scratch/fmm_rank/` and was destroyed by a working-tree cleanup, leaving a
published result with no reproducible pipeline behind it. Nothing here goes back
in `scratch/`.

## Contents

| file | role |
| --- | --- |
| `fmm_dump.rs` | extractor — re-induces the cover and dumps vectors / sigs / prototypes / members |
| `validate_51.py` | rebuilt analysis; reproduces §5.1's published figures as an acceptance gate |

## Running

`fmm_dump.rs` targets the **compile-era API at commit `9d95961`** and does not
build against current `main`. Run it from a worktree pinned there, as a
`uor-r4-graph-compiler` example:

```
# from a worktree at 9d95961, with the example placed in
# crates/uor-r4-graph-compiler/examples/
B=.uor-models/compiled/SmolLM2-135M-Instruct-7e27bd9f9532
cargo run --release --example fmm_dump -- \
    "$B/corpus.meta" "$B/corpus.records" "$B/tless_artifacts.bin" <out_dir>

python3 validate_51.py <out_dir> --eta 1.0
```

The dump self-verifies against three pinned kappas and the region census, and
**exits non-zero rather than emitting a usable dump** if reproduction fails.
`validate_51.py` refuses to analyze a dump whose `reproduction_verified` is
false. Do not weaken either check.

## Reproduction status (2026-08-02)

The dump reproduces bit-for-bit:

```
artifact_kappa blake3:c0916469…  MATCH
corpus_kappa   blake3:74491d1d…  MATCH
cover_kappa    blake3:67d957c3…  MATCH
regions 362, per_depth [16, 30, 58, 98, 160], n_train 159658
```

Analysis vs the figures published in the #290 §5.1 comment:

| figure | published | rebuilt | |
| --- | ---: | ---: | --- |
| admissible pairs, depths 1–5 (η=1.0) | 2 / 5 / 16 / 29 / 87 | 2 / 5 / 16 / 29 / 87 | exact |
| r(1e-2) at n=1024 | 20.5 | 20.0 | ±1 |
| r(1e-3) at n=1024 | 134.0 | 132.0 | ±2 |
| r(1e-4) at n=1024 | 274.5 | 273.0 | ±2 |
| growth exponents | 0.11 / 0.45 / 0.64 | 0.119 / 0.438 / 0.637 | ≤0.012 |
| null control at n=1024 | 57 / 198 / 287 | 58 / 198 / 287 | ±1 |
| pairs in exponent fit | **37** | **79** | unreconciled |

Every §5.1 conclusion reproduces: rank flat at ~20 for ε=1e-2 across a 32× block
range, sub-proportional ~n^0.44 growth at 1e-3, ambient saturation at 1e-4, and
admissible blocks sitting ~2.9× below the unstructured null at 1e-2.

### Two rebuild details worth keeping

**`diam` is taken over full membership.** Subsampling to ≤1024 rows applies to
block construction only. A first rebuild computed the q95 member angular radius
on the subsample, which shifted the admissibility threshold and produced 27 / 85
at depths 4 and 5 instead of 29 / 87 — the depths with the largest regions.
Getting this wrong is silent and only shows up as a small pair-count drift.

**The exponent-fit pair count does not match and was left alone.** Requiring the
full nested ladder (both regions ≥1024 members) yields 79 pairs; §5.1 reports 37.
The original filter is unrecoverable — `analyze.py` is gone. Since the exponents
agree to ≤0.012 and no §5.1 conclusion depends on the count, this is recorded as
an open discrepancy rather than tuned until 37 falls out. Fitting the filter to
reproduce a pair count would be fitting to a number, not reproducing a method.

### Spurious numpy warnings

`RuntimeWarning: overflow/invalid/divide by zero encountered in matmul` appears
on macOS with numpy 2.x over Accelerate. The data is clean — 0 non-finite entries
across 159658×288, no zero-norm rows, max |value| 0.71 — and unit-normalized
float64 cannot overflow. Verified, not suppressed blindly.

## §5.2

Criteria are pre-registered in the #290 thread before any numbers: the primary
measurement is the **Graph status slice** (n = 15,080, currently 1.23% top-1),
not the blended figure, which is ~half ExactContext rows that a far-field
operator cannot affect. Per §7 the next implementation step is baseline 4
(ℋ-matrix / HODLR), not full FMM.
