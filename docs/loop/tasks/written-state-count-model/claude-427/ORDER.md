# ORDER — written-state-count-model (claude-427)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-427, timing what is left of `push`, as Casey decided on 18 Nov ("Time push first").
It follows claude-426 (PR #4082).

## Why

claude-426 moved the SHA-256 streams onto their own threads, but `push` fell only from 252.9 s to
148.2 s. The scout estimated about 35 s. What remains runs on the caller's thread, and claude-426
did not split it. D3 (device residency) has to decide whether the checks, the census and the
digest move to the device with adjust. That choice needs to know which part of the 148.2 s is
which.

## What it does

1. **Sub-phase timing in `NgramAssembler::push`** (`ngram_count.rs`), off by default and switched
   on by `enable_timing`. Totals in nanoseconds, summed over partitions:
   - `triple_checks`: the per-triple loop (order, vocabulary, ascent, nonzero count);
   - `context_checks`: the per-context loop (order, followers = n1 + n2 + n3+);
   - `census`: `check_census`;
   - `triple_feed`: encoding triples into the triple stream, including any wait on its channel;
   - `context_feed`: the context merge and encoding, including any wait;
   - `send_wait`: the time `send` blocked on a full channel, for each stream (a part of the two
     feeds above, reported apart).
   - Each hasher thread also reports its busy time, the time spent in `Sha256::update`.
2. **`kn-reference`** prints those totals per order under `--profile-phases`, next to the push
   and keep lines it already prints. The output JSON is unchanged except for `unit`
   (`claude-427`).
3. No change to what is checked, counted, hashed or kept.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, and the instrument's tests pass;
  - a new test: with timing on, the digests equal those with timing off, and every sub-phase
    and hasher-busy total of a non-trivial push is above zero; with timing off they stay zero.
    A channel wait may rightly be zero (the hasher kept up), so the test checks only that each
    `send_wait` is no more than the feed it is part of.
- **G2, wick:** one N = 10 `--count-only --profile-phases` run on all of TRAIN against
  DEV-tune's keys, budget 2^24, as claude-426's G4.
  - Its `count-10.json` without `unit` equals claude-426's `run1/c426/count-10.json` without
    `unit` (`jq -S 'del(.unit)'`, then `cmp`).
  - Its per-order sub-phase lines are recorded, and the sum of a push's sub-phases is compared
    with its push total.
- **G3:** the REPORT names which sub-phase holds most of `push`, what D3 would have to take with
  it, and how much of the hashers' capacity was used, negatives first.

## Decisions recorded here

- **No negative gate.** This unit adds diagnostics only, and the G1 test is what pins them. A
  timer that is never written would fail its above-zero check. [claude], reversible by Casey.
- **No pre-registration sheet,** as in claude-425 and claude-426: G2 counts and scores nothing.
  DEV-test is not read.

## Out of scope

- Any speed-up of `push`. D3 comes after this, with its own ORDER.
