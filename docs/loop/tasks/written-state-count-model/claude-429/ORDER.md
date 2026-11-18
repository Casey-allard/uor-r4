# ORDER — written-state-count-model (claude-429)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-429, D3: adjusted counts on the device, as Casey chose on 18 Nov ("Order D3 next").
It follows claude-428 (PR #4085).

## Why

After claude-428, the N = 10 count takes 752.8 s. Two host phases are 392.8 s of it (52.2%):
- **`runs`, 270.1 s.** For each order k+1 partition, the host takes every triple's suffix
  (its key without the oldest slot) and `next`, sorts the pairs, and counts each run. That
  gives N1+(• g), the number of distinct left tokens of each order-k gram g.
- **`adjust`, 122.7 s.** The host merges order k's triples with those runs, sets each
  adjusted count, and rebuilds the contexts and count-of-counts.

claude-426 framed D3 as moving these phases, the digest and the filter to the device, with a
parallel digest that would change the replay identity. claude-428 has since taken the digest
off the caller's path: the workers hash beside the sweep and the caller no longer waits. So
this unit moves only the two host phases and keeps every byte and digest unchanged.

The device can get N1+(• g) from order k's own sort, with no order-(k+1) partition at all.
- The suffix of the order-(k+1) gram at position j is the order-k gram at j. So the left
  tokens of g are the tokens t[j-k-1] at the positions j where g occurs **and** order k+1
  admits j: j ≥ k+1, and t[j-k] is not BOS (the slot that becomes interior).
- Give each sorted position a left digit L(j): t[j-k-1] + 1 where k+1 admits j, else 0. Add L
  as the least significant key of order k's LSD radix sort. N1+(• g) is then the number of
  distinct nonzero L values in g's run, which is one flag and one scan.

## What it does

1. **Device sort** (`ngram_count_device.rs`, `ngram_count.cu`). When order < N, `sort` first
   sorts on L, before its first pass on `next`. L ranges over 0..=4,096, so it takes 13 bits:
   three 6-bit radix passes. The existing passes and their order are unchanged.
2. **Device adjust.**
   - A new kernel flags each sorted position whose L is nonzero and differs from the previous
     position's L in the same gram. A scan of the flags gives each gram its N1+.
   - `ngram_emit_grams` takes an adjust mode. A gram keeps its raw count when k = N or when
     its oldest slot is BOS. Otherwise its count is its N1+.
   - Contexts and count-of-counts are then built from the emitted counts, by the existing
     kernels.
   - A gram with N1+ = 0 that is not a document start (rule 4), or a document-start gram with
     N1+ > 0, is counted on the device. The host refuses the partition if either count is
     nonzero, with the host path's messages.
3. **Sweep.** `count_adjusted_sweep` counts each order k ≥ 1 of each shared range already
   adjusted, and downloads only that partition. No runs are carried between orders. The
   order-1 adjusted partition feeds the unigram continuation before it goes to the sink. The
   continuation reads only keys and `next`, which are the same as in the raw partition.
   Order 0 is unchanged.
4. **The host-runs sweep is kept** as `count_adjusted_sweep_host`, with its tamper hook, as the
   reference path for tests. `count_order_adjusted_with` and `adjust_partition` are unchanged.
5. **Timing.** The extra sort pass is in `sort`, and the flags and scan are in `emit`. In the
   device sweep, `runs` and `adjust` are 0.
6. **`kn-reference`** writes `unit: claude-429` and is otherwise unchanged.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, and so do the instrument's tests;
  - a new host test models the device rule: positions sorted by (key, next, L), counted by
    distinct nonzero L. For orders 1–4 under N in {1, 3, 4}, on a stream with BOS every 61
    tokens and on one dense in BOS, the model's adjusted partitions equal `adjust_partition`'s
    over `oracle_count` partitions.
- **G2, wick (CUDA tests at the head):**
  - the device sweep's records and digests equal `oracle_adjusted` for every order, at N in
    {1, 3, 10} and budgets u64::MAX, 60,000 and 9,000;
  - the device sweep equals `count_adjusted_sweep_host` byte for byte at the same budgets;
  - every existing `ngram_count_cuda`, `kn_score_cuda` and `kn_filter` test passes. The tamper
    tests run on the host-runs sweep.
- **G3, negatives,** each set aside against the base, run, expected to fail, then restored:
  - without the L pass, G2's oracle test fails;
  - with L not requiring t[j-k] ≠ BOS, G1's model test fails, and so does G2's oracle test on
    the BOS-dense stream;
  - a test hook that plants a rule-4 gram is refused with the host path's message.
- **G4, wick:** one N = 10 `--count-only --profile-phases` run on all of TRAIN against
  DEV-tune's keys, budget 2^24, as in claude-428's G3.
  - Its `count-10.json` without `unit` equals claude-428's `run1/c428/count-10.json` without
    `unit`.
  - Phases, peak RSS and the whole-device GPU peak are recorded against claude-428's.
- **G5, the expected outcome:**
  - The count's wall time falls by at least half of `runs` plus `adjust`, 196.4 s.
  - Peak RSS rises by no more than 1 GiB.

  This gate is a bar, not a stop. A miss is recorded first in the REPORT, with its cause, and
  the unit still lands if G1–G4 pass.

## Decisions recorded here

- **D3 keeps the sequential digest.** There is no parallel digest and no replay-identity
  change. The digest, `download`, `records`, the census and the filter stay on the host, and
  each is a later unit if the numbers call for it. [claude], reversible by Casey.
- **The host-runs sweep stays** as the tests' reference and the tamper tests' path. No
  production caller uses it. [claude], reversible by Casey.
- **No pre-registration sheet,** as in claude-425–428: G4 counts and scores nothing. DEV-test is
  not read.

## Out of scope

- A parallel or device-side digest, device-side validation, `download`/`records` changes, and
  the census-on-sweep run.
