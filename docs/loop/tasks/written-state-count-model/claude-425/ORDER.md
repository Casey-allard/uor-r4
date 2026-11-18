# ORDER — written-state-count-model (claude-425)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-425, fixes 2 and 3 from claude-423's profile in one unit, as Casey decided on 18 Nov:
- bucket TRAIN once, instead of a stream pass per partition;
- count each order once.

It follows claude-423 (PR #4075, the profile) and claude-424 (PR #4080, the extension census for
fix 1). Code work starts after #4080 merges, because both edit `kn-reference`.

## Why

claude-423's N = 10 count on DEV-tune took 1,817 s of host wall time. Two parts are repeated
work. The figures are elapsed wall times around CUDA-driving code at `61d011ee5`; see #4075 for
what each one includes.
- **`select`: 261 s.** Each partition flags, scans and scatters the whole 2.09 B-token stream,
  in about 32 windows of 2^26, each ending in a blocking read. That is up to 256 passes per
  order.
- **The extension count: 375 s.** For every order k < N, `count_order_adjusted_with` counts
  order k+1 over order k's ranges, only to read its (key, next). The next order then counts
  order k+1 again as its own order.

The design comes from two read-only scouts, kept in the lab's notes. Scout estimates, not
measurements:
- bucketing could save up to about 250 s of `select`;
- counting each order once could save about 350 s.

## What it does

1. **Buckets and slice selection.**
   - A stable counting sort of every position j >= 1 by t[j-1]: a histogram, a scan over 4,096
     bins, and a block-ordered scatter in the pattern of `ngram_radix_hist`/`ngram_radix_scatter`.
     No atomics decide the order.
   - It runs once per counter, as u32 positions. The stream bound is 2^31.
   - For k >= 1, `select` compacts only the bucket slice of its key range, with the same
     admission rule (`ngram_admitted`). It must yield the same ascending positions as today, so
     the sort input is unchanged.
   - The exact-count check against the plan stays.
   - Order 0, which keys on t[j], keeps its current select.
2. **One count per order: `NgramCounter::count_adjusted_sweep`.**
   - **One shared plan, cut from order 1's histogram.** Order 1 dominates every higher order in
     every bin, so the plan bounds every k >= 1 under the budget.
   - **For each range, for k = N down to 1:** count order k raw; adjust it with the runs taken
     from order k+1; hand it to order k's sink; turn its raw (key, next) into order k-1's runs.
     Order 1 feeds the unigram continuation.
   - **Order 0** is counted last, on its own plan.
   - `count_order_adjusted_with` stays, as the reference the tests compare the sweep to.
3. **`kn-reference` on the sweep.**
   - It holds one assembler, filter and extension census per order. Each sink receives its
     partitions in ascending key order.
   - A new `--count-only` flag writes the per-order records and stops before scoring.
     `--extension-census` implies it.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass;
  - the instrument's tests pass, including `--count-only` parsing.
- **G2, wick (CUDA tests at the head):**
  - slice selection gives the same positions as the stream `select` for every partition, at
    orders 1..N and budgets 9,000 and 60,000, including a stream dense in BOS at k = 10;
  - the sweep's records and digests equal `oracle_adjusted` for every order, at N in {1, 3} and
    budgets u64::MAX, 60,000 and 9,000;
  - on the real-stream test, the sweep equals `count_order_adjusted_with` byte for byte;
  - the phase totals of a sweep show an extension count of 0;
  - the existing `ngram_count_cuda`, `kn_score_cuda` and `kn_filter` tests stay green.
- **G3, negative gates** (each set aside against the base, run, expected to fail, restored):
  - a planted miscount in a bucket offset fails the exact-count check;
  - a test hook that adds 1 to one raw order-k+1 count changes order k+1's digest and leaves
    order k's unchanged;
  - dropping that triple changes order k's digest or is refused.
- **G4, wick:** one N = 10 `--count-only --profile-phases` run on all of TRAIN against
  DEV-tune's keys, budget 2^24.
  - Its per-order records equal claude-423's `run1/c423/sweep-10.json` (`jq -S`, then `cmp`)
    in every field but `partitions`: grams, contexts, count-of-counts, triple and context
    digests, kept triples and contexts.
  - Its phase lines give the new wall time, with `select` and the extension count each
    against claude-423's.
  - Peak resident memory and the nvidia-smi peak are recorded. The 4090 is shared, and about
    2.7 GB was in use on 18 Nov.
- **G5:** the REPORT states the measured saving per phase, beside the scouts' estimates, with
  negatives first.

## Decisions recorded here

- **`partitions` is not compared.** With one shared plan, orders >= 2 are counted in more
  partitions than before, so that per-order field changes. It describes the plan, not what is
  counted. Every digest, count and kept-record total must still match exactly. [claude],
  reversible by Casey.
- **No pre-registration sheet.** G4 counts and scores nothing: `--count-only` stops before the
  scorer, and no BPB, bar or milestone claim rests on it. Under §4.5 that makes it a check of
  equal output, not an evaluation. This closes the gap Codex found in claude-423, where the
  scored full pass had no named FIT. A scored run, if one is wanted, gets its own sheet and a
  non-author FIT first. DEV-test is not read.

## Out of scope

- Fix 1 (filtering before the copy) waits on claude-424's census.
- Dropping the discarded host context build on the adjusted path, and fix 4 (the shared N
  sweep), come later.
- No change to what is counted.
