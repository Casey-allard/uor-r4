# REPORT: written-state-count-model, claude-425 (one bucketing pass, one count per order)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The host work that reads every gram shrank by only 32.6 s (2.6%)** [claude]. In claude-423
  the six host steps that read every own-order gram took 1,277.5 s; here they take 1,244.8 s.
  Copies and record building fell by 117.3 s, because the separate extension count is gone. But
  `runs` (+42.3 s), `adjust` (+22.9 s), `push` (+11.7 s) and `keep` (+7.9 s) each got slower
  for the same work, by 84.7 s together. The cause is not measured. One candidate is that the sweep now interleaves
  11 orders' assemblers, filters and query lists per range, where claude-423 finished one order
  before starting the next. Those host steps are now 86% of the sweep's wall time (1,244.8 of
  1,447.0 s).
- **Peak resident memory rose 2.51 GiB (2,634,664 KiB),** from 8,864,644 KiB to 11,499,308 KiB. Likely cause, not
  measured: every order's query filter (its sorted contexts and pairs) now stays alive for the
  whole sweep. Before, each order's filter existed only while that order was counted.
- **3–5 minutes is still far off.** The count alone takes 1,447 s, down from 1,817 s. The device
  passes this unit targeted now take 163.2 s; the rest is host work this unit did not touch
  (fix 1's scout covers it).
- **Two of the three G3 negatives are in-suite tamper tests, not set-aside runs.** The planted
  count change and the dropped left type run as hooks inside
  `ngram_count_cuda_sweep_orders_read_only_their_extension_types`, and each asserts its digest
  effect. Only the bucket-offset miscount was run with its check set aside against the base.
- **The real-stream test reaches only orders 0–3.** It uses a 15.5 M-token stream, and orders
  4–10 are checked at full scale only by G4's record comparison.
- **The extension census on the sweep never ran.**
  - G4 ran without `--extension-census`, so the new routing (order k's partitions to census
    k−1) is checked only by reading.
  - `a_count_only_run_scores_nothing` checks only that `--count-only` and `--extension-census`
    parse into a count-only run. No test shows that the scorer is skipped.
  - G4's log does: it ends "nothing was scored" with no scoring line.

## What changed

- **Buckets and slice selection** (`ngram_count.cu`, `ngram_count_device.rs`).
  - `ngram_bucket_hist` and `ngram_bucket_scatter` do a stable counting sort of every position
    j ≥ 1 by t[j-1]: one warp per 2^20 tile, a scan over 4,096 bins, and a block-ordered
    scatter. No atomic decides the order.
  - The sort runs once per counter, is cached, and holds 8.36 GB of device memory beside the
    4.18 GB of tokens.
  - `select_slice` compacts only the bucket slice of a range's tokens, with `ngram_admitted`.
    The planned-count check runs before the scatter and again at its end.
  - Order 0 keeps the whole-stream `select_stream`.
  - No re-sort is needed. Equal grams share slot 1, so they share a bucket in ascending j, and
    the stable LSD sort gives the same array.
- **`NgramCounter::count_adjusted_sweep`** counts each order once.
  - One shared plan is cut from order 1's histogram.
  - For each range, for k = N down to 1, the sweep counts order k raw. It adjusts that count
    with order k+1's runs, hands it to order k's sink, and turns it into order k-1's runs.
    Order 1 feeds the unigram continuation.
  - Order 0 is counted last, on its own plan.
  - It returns the shared plan. `count_order_adjusted_with` stays as the reference.
- **`kn-reference` on the sweep.**
  - It keeps one assembler, filter and extension census per order, with per-order sink timings.
  - New `--count-only` writes the per-order records, discounts included, and stops before the
    scorer. `--extension-census` implies it.
- **New CUDA tests:**
  - `ngram_count_cuda_bucket_slices_equal_the_stream_select`
  - `_bucket_offset_miscount_is_refused`
  - `_sweep_matches_the_oracle`
  - `_sweep_counts_no_extension`
  - `_sweep_orders_read_only_their_extension_types`
  - `_real_stream_sweep_matches_the_adjusted_counter`
- **New instrument test:** `a_count_only_run_scores_nothing`.

## Measurements

The G4 run used wick's 4090 at `2c22da962`, budget 2^24, all of TRAIN (2,091,560,001 tokens)
against DEV-tune's keys at N = 10, with `--count-only --profile-phases`. It was job `c425-g4`;
the packet is in [`run1/c425/`](run1/c425/).
- 1,464 s wall; the count itself took 1,447.0 s over 128 shared ranges. The counter counted 1,408 partitions: 128 for each of orders 0–10. Order 0's 128 raw partitions are adjusted into one partition before its sink, so the orders were handed 1 + 10 × 128 = 1,281.
- `count-10.json` has sha256 `518a3e0729867477628ef020d84e8c4a9e90f16d272ac94af5c092c08ecabede`.
- Nothing was scored, and DEV-test was not read.
- Peak resident set: 11,499,308 KiB.
- Peak GPU use for the whole device: 21,095 MiB, with 3,946 MiB used by other work before the
  start. That leaves at most about 17.1 GB for this run.

Phase totals over orders 0–10, claude-423 ([`sweep-10.log`](../claude-423/run1/c423/sweep-10.log))
against this run, in seconds:

| phase | claude-423 | claude-425 | change | scout estimate |
|---|---|---|---|---|
| **count wall** | **1,816.6** | **1,447.0** | **−369.6 (−20.3%)** | |
| select | 260.9 | 29.3 | −231.7 | up to ~250 saved |
| extension count | 375.0 | 0 | −375.0 | ~350 saved |
| sort | 198.1 | 114.4 | −83.7 | |
| emit | 31.2 | 19.2 | −12.0 | |
| download | 90.6 | 47.8 | −42.9 | |
| records | 185.2 | 110.7 | −74.5 | |
| runs | 261.2 | 303.6 | +42.3 | |
| adjust | 120.3 | 143.2 | +22.9 | |
| push | 241.2 | 252.9 | +11.7 | |
| keep | 378.9 | 386.8 | +7.9 | |

- In claude-423 the extension count's 375.0 s was spent inside select, sort, emit, download and
  records. Its removal shows up in those rows; the two totals are not separate savings.
- Order 10's push (52.5 s) and keep (75.8 s) are each order's largest sink times
  ([`count-10.log`](run1/c425/count-10.log)).

## What the numbers say

- The device-side repetition is gone. Select fell 89% and the separate extension count is zero,
  each close to its scout estimate. The device passes (histogram, select, sort, emit) together
  fell from 490.7 s to 163.2 s.
- The count is now bound by host work over every gram: copying it, building records, the KN
  runs and adjust, hashing, and the query filter. Fix 1's scout costs the cuts:
  - a cursor instead of a binary search in `keep`;
  - hashing on parallel threads;
  - keeping partitions on the device, so only the ~45 M kept records are copied.
- What the sweep counts is unchanged: G4's records equal claude-423's in every compared field.

## Checks run at the head

- **G1, laptop, at `2c22da962`:** stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib
  --tests` and the instrument's `cargo check --features cuda` are clean. The only warnings are 4
  pre-existing dead-code warnings in `uor-r4-core`. The host `kn_` and `ngram_count` tests pass
  33 / 33, and the instrument's tests 7 / 7.
- **G2, wick:**
  - At `ef83434ee`, the discrete crate's code head ([`run1/c425b/`](run1/c425b/)):
    - CUDA tests 20 / 20, 4 ignored for missing env streams.
    - Slice selection equals the stream select at orders 1..10 and budgets 9,000 and 60,000,
      on a stream dense in BOS at k = 10.
    - The sweep equals `oracle_adjusted` at N in {1, 3} and budgets u64::MAX, 60,000 and 9,000.
    - A profiled sweep records an extension count of 0.
    - The real-stream tests pass 3 / 3: at orders 0–3 the sweep's digests equal
      `count_order_adjusted_with` byte for byte.
  - At `9446f32be` (item 3), `2c22da962` and this REPORT's head, only the instrument and run
    records changed, not the crate.
- **G3, negatives:**
  - With `select_slice`'s count checks set aside against the base, the bucket-offset miscount
    test fails ([`run1/c425a/negative.log`](run1/c425a/negative.log)).
  - The two tamper tests pass, and each asserts its digest effect:
    - adding 1 to a raw order-3 count moves order 3's digest and leaves orders 0–2 unchanged;
    - dropping a lone left type moves order 2's digest or is refused.
- **G4, wick:**
  - `jq -S '[.orders[] | del(.partitions)]'` of `count-10.json` is `cmp`-identical to
    claude-423's `sweep-10.json`: 11 fields × 11 orders, discounts included.
  - `partitions` is 1 for order 0 (own plan) and 128 for every other order, against 128
    everywhere before.
- **G5:** this REPORT.
- **Not run:** a final G2 with `UOR_KN_TRAIN` / `UOR_KN_QUERY` set for
  `kn_score_cuda_real_slice_equals_the_cpu_oracle`. The scorer did not change.

## Skills used

working-discipline.

## Still open

- **Why runs, adjust, push and keep got slower** in the interleaved sweep (+84.7 s), and the
  2.51 GiB of extra resident memory. A per-order timing of a sweep that drops each order's
  filter when it finishes, or one at N = 4, would separate the causes.
- **Fix 1: the host work that reads every gram (1,244.8 s).** The scout ranks the steps:
  - first, a cursor in `keep` and hashing on parallel threads, which keep the digest
    (est. ~640 s → ~50 s);
  - then partitions held on the device, which needs a parallel digest and so a recorded
    change of replay identity.
- **Orders 4–10 on a real stream** are compared only at full scale, by G4.
- **A census run on the sweep.** A small `--extension-census` run, with its totals compared to
  claude-424's `census-10.json`, would exercise the routing this unit only read.

## Plain summary

The count now reads TRAIN's positions once, sorted into 4,096 buckets, instead of scanning the
whole 2.09 B-token stream for every partition. It also counts each order once instead of twice.
It gives exactly the same counts, digests and discounts as before at N = 10. The GPU part of the
count fell from 491 s to 163 s, and the whole count from 1,817 s to 1,447 s. What remains is
almost all host work over every one of the 6.7 B grams: copying, hashing and filtering. That
work fell only 2.6% here and is the next target.
