# ORDER: written-state-count-model, claude-438 (P1 gate (c), read: the KN start and a bound on the fit's distance)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `3ff8f4640` (main, after claude-433's PR 4097 merged as `dd7a4b6b0`).

**Bar moved:** P1 gate (c) of the written-state approved sequence, the U_count ceiling against
0.85 × and 0.90 × R\*. claude-433 measured it and could not read it: at 100% of TRAIN, the
24-pass EM from the uniform start ended above KN-only (U_count 1,433,181 micro-BPB against
1,432,318). Its sheet's row "I (EM not converged)" applied, and that row asks for a new sheet with
a different starting point. This unit is that sheet's run.

## What changes, and why it makes gate (c) readable

1. **The KN start.** Each bucket's weights start at 2^32 − (L − 1) on KN and 1 on each of the
   other L − 1 present experts, where L is the number of present experts in the bucket.
   - The start's mix is KN's mass to within (L − 1)/2^32 of it, plus a little from the others, so
     its bits are KN-only's up to integer floors.
   - EM never lowers the likelihood, so the fit ends at or below KN-only up to those floors.
   - Every present expert starts above 0, so EM can still move weight onto it. EM multiplies a
     weight by its expert's mean responsibility ratio, and a weight at 0 stays at 0.
2. **A bound on the fit's distance from the best weights.** For a bucket b with N_b fit
   targets, let g_e = (1/N_b) Σ_t p_te / m_t, where m_t is the mix at the current weights. Then
   Σ_e w_e g_e = 1, and Jensen's inequality gives, for any weights w\* on the same simplex:
   Σ_t log2(m\*_t / m_t) ≤ N_b · log2(Σ_e w\*_e g_e) ≤ N_b · log2(max_e g_e).
   - Summed over buckets, that bounds how many bits the best bucketed weights can still save over
     the fitted ones, on the fit set.
   - It is computed in integers: g_e in Q32, its log2 through `bits_q12` of the Q48 mass
     floor(2^80 / G), rounded up by one Q12 unit per target.
   - Then U_count − gap is a lower bound on the converged ceiling, and U_count itself is an upper
     bound. Gate (c) is read from whichever side settles it.
3. **96 EM passes,** up from 24, in every mode. At 100% that is about 10 min of host EM,
   measured from claude-433's 137 s for 24 passes of its four fits.

Nothing else changes: the 13 experts, the bucket rule, the modes (U_count, U_split, KN + U,
KN-only), DEV-tune as the query, N = 10 and budget 2^24.

## What it builds

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: `EmStart::{Uniform, Kn}`, `fit_em_from`, and
  `gap_bound_q12` (the bound above). `fit_em` and `fit_em_with` keep the uniform start.
- `docs/labs/kn-reference/instrument`: every ceiling mode fits from the KN start, with 96 passes,
  and the JSON records each fit's bound (Q12 bits and, for U_count, micro-BPB, rounded up) and
  the start's bits. The JSON's `unit` reads `claude-438` for a ceiling run.

## Gates

- **G1, laptop:** host tests:
  - the KN start's weights sum to 2^32 in every bucket, with KN at 2^32 − (L − 1), each other
    present expert at 1, and dropped experts at 0; a mask without KN is refused;
  - the KN start's pass-0 bits are at most KN-only's bits plus one Q12 unit per target;
  - the bound holds on fixtures: the bits EM saves from a start to its last pass are at most that
    start's bound, from both starts;
  - after many passes the bound is small and never negative;
  - every claude-433 test still passes, the instrument's tests pass, stub-nvcc `cargo check`s are
    clean, and the D32 grep of the added lines is clean.
- **G2, negatives,** each set aside against the base, run, expected to fail, then restored:
  - the bound taking a w-weighted mean of g instead of its maximum fails the bound test;
  - the KN start leaving the other present experts at 0 fails the start test.
- **G3, one rented RTX 5090, after a new sheet and a non-author FIT.** Casey released it on
  19 Nov, answering "May I rent one 5090 (about $2) for the 100% KN-start rerun?" with:
  "Yes, rent it". It runs through `uor-pod` with `--count 1`, inside the caps, with a completion
  watcher armed and results copied off before `down`. The same job runs every size, so the
  build, the inputs and the GPU are one:
  - the CUDA lookup tests at the head;
  - 1‰ with `--check-host`, then 1% and 10%, each beside a plain run whose tallies must equal
    `kn_only`;
  - 100%, where `kn_only` must equal claude-422's 1,432,318 / 1,747,150, and gate (c) is read.

DEV-test is not read: it stays sealed for gate (b).

## Decisions recorded here

- **Every size on the rented pod, not wick.** The code change is host-only, and one job on one
  machine keeps every size on the same build and inputs. The small sizes add minutes to a rental
  Casey released. [claude]
- **The bound is the reading's instrument, not a stopping rule.** The pass count is fixed at 96
  before any result. The bound only decides whether the fixed-pass result settles the gate.
  [claude]
- **The KN start is a starting point, not a prior.** EM's fixed point does not depend on where
  it starts (the objective is concave over each bucket's simplex). The start only removes the
  slow first passes away from uniform. [claude]

## Out of scope

- Any change to the experts, buckets or modes;
- the S5 Pick expert (claude-434) and every later R-unit;
- the two claude-433 REPORT nits (the header's last code commit, "bit 60") and the 32-bit index
  and log-label nits, which stay in the tracker.
