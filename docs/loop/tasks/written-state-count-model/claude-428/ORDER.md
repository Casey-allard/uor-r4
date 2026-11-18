# ORDER — written-state-count-model (claude-428)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-428, per-order hash workers, as Casey chose on 18 Nov ("Per-order hash threads"). It
follows claude-427 (PR #4083).

## Why

claude-427 split `push` (150.4 s of the N = 10 count). The two feeds took 121.2 s, 80.6% of
push. Of that, 73.2 s was the caller in `send` on full hash channels and 48.0 s was encoding and
merging on the caller's thread. Each stream must be hashed in order. But a stream's next partition
arrives about 7 s after the last one (887 s over 128 shared ranges), while hashing one takes
about 140 ms even at order 10. So the hash threads have time to spare; the caller should not wait
for them. This unit keeps the sequential SHA-256 digest, so every replay identity is unchanged.

## What it does

1. **The sweep hands each partition over by value** (`ngram_count_device.rs`). The sink becomes
   `FnMut(u32, NgramPartition)`. The sweep already owns each adjusted partition and drops it
   right after the sink returns, so nothing is copied.
2. **`NgramAssembler::push_owned(partition)`** (`ngram_count.rs`).
   - **On the caller's thread:**
     - the per-record checks, `check_census` and the seam check, unchanged;
     - the cross-partition context order check. It moves here from the merge loop, so a
       refusal still happens before any byte is handed on, with the same message;
     - the counters (targets, grams, count-of-counts);
     - keeping triples in tests.
   - **Then the partition goes, in one `Arc`, to two worker threads per order:** one for
     triples and one for contexts.
     - Each worker encodes its records into a buffer of `chunk` bytes and hashes the buffer
       when it fills.
     - The context worker also merges a context that straddles a partition at order 0, counts
       the emitted contexts, and keeps them in tests.
     - Each worker's queue holds one partition behind the one in progress, so an order holds
       at most two partitions beyond the caller's.
   - **`push(&partition)`** keeps working for the other callers, through a copy into the `Arc`.
3. **Timing (claude-427) keeps its fields.**
   - The feeds become the counters plus the hand-off.
   - `send_wait` is the time the hand-off blocked on a full worker queue.
   - Hasher busy is now a worker's encoding plus hashing.
4. **`kn-reference`** sinks the partition by value: push and keep, the census, then
   `push_owned`. It writes `unit: claude-428`.
5. Nothing that is checked, counted, hashed or kept changes, and nothing changes in which
   partitions are refused.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, including claude-426's
    `ngram_count_threaded_hash_equals_inline_sha` and claude-427's timing test;
  - the instrument's tests pass;
  - a new test: `push_owned` and `push` give the same digest, contexts and kept records as an
    inline SHA-256. It covers orders 0–3, chunk sizes that cut records, an order-0 context
    that straddles partitions, and partitions handed faster than the workers drain;
  - every assembler refusal test still refuses with its message, and a context that goes
    backwards across partitions is refused by `push_owned` before anything is handed on.
- **G2, negatives,** each set aside on the laptop, then restored:
  - a context worker that drops the order-0 merge fails the new test;
  - a worker that never hashes its tail buffer fails it too.
- **G3, wick:** one N = 10 `--count-only --profile-phases` run on all of TRAIN against
  DEV-tune's keys, budget 2^24, as claude-427's G2.
  - Its `count-10.json` without `unit` equals claude-427's
    `run1/c427/count-10.json` without `unit`.
  - Push, its parts and peak RSS are recorded against claude-427's.
- **G4, the expected outcome:**
  - Push falls by at least half of the 73.2 s that claude-427 spent in channel waits.
  - Peak RSS rises by no more than 4 GiB (4,194,304 KiB).

  This gate is a bar, not a stop. A miss on either is recorded first in the REPORT, with its
  cause, and the unit still lands if G1–G3 pass, since nothing it changes moves a digest.

## Decisions recorded here

- **Queue depth one partition per worker.** This bounds the memory. The 4 GiB line is a guess
  from claude-427's partition sizes (about 740 MiB on average at order 10), and G4 measures it.
  [claude], reversible by Casey.
- **No pre-registration sheet,** as in claude-425–427: G3 counts and scores nothing. DEV-test is
  not read.

## Out of scope

- D3 (device residency) and any parallel digest; the census-on-sweep run.
