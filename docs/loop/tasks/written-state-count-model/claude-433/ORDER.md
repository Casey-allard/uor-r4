# ORDER: written-state-count-model, claude-433 (P1 gate (c): the count ceiling)

Lab: claude. Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md)
(#4063). Base: `146db0e31` (Codex #4096).
This is the first P1 unit of the roadmap Casey asked for on 18 Nov: "please plan the roadmap based
on your above operations and then begin the work with ordering P1 as you have suggested".

**Bar moved:** P1 gate (c) of the written-state approved sequence
([PLAN.md](../../../../../PLAN.md#approved-sequences), [DECISION
§4](../../../../research/reviews/pi-review-2026-11-16/DECISION.md)), the U_count ceiling against
0.85 × and 0.90 × R\*. That gate is a branch, not a kill:
- at or below 0.85 × R\*, counts carry most of the quality, and P1's deployable model (gates (a),
  (b), (d)) is the critical path;
- above 0.90 × R\*, P2 and P5 (geometry) are the critical path;
- in between, both stay open.

## Why this unit first

P0 is done: R\* is sealed (claude-422), and the count engine is fast and identical
(claude-425–431). Every P1 unit since then has made counting faster; none has moved a P1 gate.
Gate (c) is the cheapest P1 gate to measure. It also decides where the rest of P1 and P2 should
put their effort, so it comes before the larger build of gates (a), (b) and (d).

Codex's closure plan ([codex-9](../codex-9/CLOSURE-PLAN.md), [TRAINING](../codex-9/TRAINING.md))
proposes the same P1 build and asks for "the test-fitted mixer ceiling" to be reported
separately. This unit is that report. It claims nothing else of Codex's plan.

## What it measures

For every DEV-tune target, there are N + 3 experts, each a Q48 mass (M = 2^48), with N = 10:
- **KN:** R\*'s own mass, from the existing `KnScorer`.
- **ML_k, for k = 0..=N:** floor(a · M / total) from the RAW counts at order k, where a is the
  count of (context, target) and total is the context's total. The expert is absent when the
  order-k context was never seen in TRAIN.
- **U:** the uniform mass M / 4096.

Each target falls into a bucket (h, c):
- h is the longest order whose context was seen;
- c = min(15, floor(log2 total_h)).

That makes at most 11 × 16 = 176 buckets. Experts above h are absent from that bucket.

**The mixer** is DECISION §3.4's bucketed linear mix, fit by integer EM:
- weights are Q32 per bucket, summing to 2^32, and start uniform over the present experts;
- per target, m = Σ w_e · p_e (u128), and r_e = floor(w_e · p_e · 2^32 / m);
- r_e is summed per bucket; each bucket is renormalised to 2^32, with the remainder going to the
  largest weight, ties to the lowest expert index;
- the mixed mass is max(1, floor(m / 2^32));
- each pass is scored through `bits_q12` and bytes as R\*'s tally is;
- the iteration count is fixed at 24, and the bits are recorded after every iteration.

Logistic, Hedge and maximum-entropy mixing stay banned. Nothing squashes.

**Modes, all on the same targets:**
1. **U_count:** fit on all of DEV-tune and score the same. This is the ceiling, fit on the
   evaluation set as DECISION §4 defines it.
2. **U_split:** fit on the even records and score the odd ones, and the reverse. This is the gap
   between the ceiling and a held-out fit.
3. **KN only.** It must equal claude-422's own DEV-tune number at the same TRAIN size: at 100%,
   1,432,318 micro-BPB overall and 1,747,150 novel.
4. **KN + U only,** fit by the same EM. This is what bucketing alone adds.

Each mode is reported overall and on the novel class. The JSON carries every bucket's target
count and final weights.

## What it builds

1. **Host** (`crates/uor-r4-discrete/src/kn_ceiling.rs`):
   - raw tables at every order 0..=N;
   - the host lookup oracle, (a, total) per order per target;
   - the bucket rule;
   - the integer EM fitter and its scorer.
2. **Device:** a CUDA kernel `kn_lookup_targets` in its own `.cu` module.
   - It is built by nvcc through `build.rs` like `kn_score.cu`, never NVRTC, and leaves the
     compiled `kn_score` image untouched.
   - It returns (a, total) for every order of every target, by the same binary searches as
     `kn_score_targets`.
3. **Instrument** (`docs/labs/kn-reference/instrument`): a `--ceiling` mode.
   - It counts RAW tables at orders 0..N − 1 beside the existing sweep, filtered to the query's
     keys by `KnQuery`. Order N's raw table is R\*'s own.
   - It runs the lookups and `KnScorer` on the GPU, then EM and the tally on the host.
   - It writes `ceiling.json` with `unit: claude-433`.

## Gates

- **G1, laptop:**
  - host tests:
    - the EM fitter on fixtures, where bits never rise between iterations and a one-expert bucket
      keeps weight 2^32;
    - the host lookup equals a brute-force count on a small stream;
    - the bucket rule at its edges;
    - mixed masses are at least 1;
  - the instrument's tests pass;
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the instrument's
    `cargo check --features cuda` are clean;
  - the D32 grep of the added lines is clean;
  - no touched file has more rustfmt hunks than on main.
- **G2, wick (CUDA tests at the head):**
  - the device lookup equals the host oracle on the KN test fixtures and on a 1‰ TRAIN slice
    scored against 1‰ of DEV-tune;
  - every existing `kn_score_cuda`, `ngram_count_cuda` and `kn_filter` test passes, unchanged.
- **G3, negatives,** each set aside against the base, run, expected to fail, then restored:
  - the lookup reading the ADJUSTED tables instead of the raw ones fails G2's oracle test;
  - the EM step without its renormalisation fails the fitter's fixture test.
- **G4, wick, after the pre-registration sheet and a non-author FIT:** `--ceiling` runs at N = 10
  at 1% and 10% of TRAIN, with DEV-tune as the query, budget 2^24.
  - At each size, KN-only equals a plain `kn-reference` run at the same size, overall and novel.
  - U_count, U_split and KN + U are recorded, with the ratio of each to KN-only at the same size
    and to R\* (100%).
  - Wall time, phases and the GPU peak are recorded.
- **G5, 100% of TRAIN:**
  - It runs on wick if `nvidia-smi` shows enough free GPU memory for the 10% run's measured peak
    scaled to 100%. Otherwise it runs on one Vast 5090 inside the caps, as in claude-431.
  - KN-only must equal claude-422's 1,432,318 / 1,747,150.
  - Gate (c) is read from this size: U_count / R\* against 0.85 and 0.90.

DEV-test is not read: it stays sealed for gate (b).

## Decisions recorded here

- **DEV-tune, not DEV-test.** This is the ceiling's evaluation set. R\* has a sealed DEV-tune
  number at N = 10, so the ratio compares the same targets. DEV-test stays single-use for P1
  gate (b). [claude], reversible by Casey.
- **R\*'s KN mass is one of the experts.** U ≤ R\* up to EM's iteration error, so U measures what
  bucketed mixing adds to counts above KN itself. The registered reading of 0.85 / 0.90 is
  unchanged. [claude]
- **13 experts,** above the deployable ≤ 12. This is a ceiling, not the deployable mixer;
  R4/claude-436 fits the deployable one on held-out TRAIN. [claude]
- **EM and the tally run on the host, from GPU lookups.** They are integer sums over the GPU's
  per-target output, like R\*'s own tally in claude-422; the counting and the lookups run on the
  GPU (§3.5). [claude], reversible by Casey.
- **Two-fold split by record parity** for U_split, so no record's targets are split between
  folds. [claude]

## Out of scope

- The S5 Pick expert (claude-434);
- S2 in `TrunkTables`, the frozen form and loader identity (claude-435);
- pruning, the deployable mixer and ft/dev (claude-436);
- any geometric expert (P2).
