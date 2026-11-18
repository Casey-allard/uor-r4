# ORDER — written-state-count-model (claude-431)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-431, the records phase, as Casey asked on 18 Nov ("Order the records unit").
It follows claude-430 (PR #4091).

## Why

After claude-430, the N = 10 count takes 288.0 s, and `records` is the largest phase at 83.6 s
(29.0%). It is host work in `count` (`ngram_count_device.rs`). After the download, one thread
turns the downloaded word arrays into a `Vec<NgramTriple>` and a `Vec<NgramContext>`, one
element at a time.
- At N = 10 that is about 6.73 billion triples and 5.44 billion contexts. At 32 and 48 bytes
  each, that is about 476 GB written into newly allocated memory, about 5.7 GB/s on one core.
- Every element depends only on the downloaded arrays at its own index (a context also reads
  the key of its first triple). The digest is computed later from the finished partition, so
  the order in which elements are written changes no byte and no hash.

claude-429 framed records as removable only through a parallel digest, which would change the
replay identity. This unit takes the identity-preserving step first: the same structs, built on
several threads.

## What it does

1. **A parallel builder** (`ngram_count.rs`, host code). `build_indexed(n, threads, f)` returns
   the `Vec` whose element i is `f(i)`. It splits 0..n into at most `threads` contiguous
   chunks and fills them on scoped threads. Below a fixed size (2^16 elements), or with one
   thread, it fills them on the caller. The output does not depend on `threads`.
2. **`count`** (`ngram_count_device.rs`) builds `partition.triples` and then
   `partition.contexts` with it. Each closure is the same expression as today's loop body. The
   thread count is `available_parallelism()` capped at 8, read once when the counter is built.
3. **Timing.** `records` still times both builds. No phase is added or renamed.
4. **`kn-reference`** writes `unit: claude-431` and is otherwise unchanged.

No new memory: each `Vec` is allocated once at its final size, as today.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, and so do the instrument's tests;
  - a new host test checks that `build_indexed` equals the sequential map for n in {0, 1, 7,
    2^16 − 1, 2^16, 2^16 + 1, 1,000,003} and threads in {1, 2, 3, 8, 64};
  - the D32 grep of the added lines is clean, and no touched file has more rustfmt hunks than
    on main.
- **G2, wick (CUDA tests at the head):** every `ngram_count_cuda`, `kn_score_cuda` and
  `kn_filter` test passes, unchanged.
- **G3, negatives,** each set aside against the base, run, expected to fail, then restored:
  - laptop: a builder whose chunks start one element late fails the new host test;
  - wick: the same change fails G2's oracle tests.
- **G4, wick, the same GPU for both:** main's binary (`96c8841d2`) and the head's each run one
  N = 10 `--count-only --profile-phases` count on all of TRAIN against DEV-tune's keys, budget
  2^24, one after the other, as in claude-430. The container's CPU count and `nvidia-smi` are
  read before each.
  - Both `count-10.json` files without `unit` equal claude-430's `run1/c430/count-10-head.json`
    without `unit`.
  - Phases, peak RSS and the whole-device GPU peak are recorded for both.
- **G5, the expected outcome:** the head's `records` is at most half of main's in the same
  pair, and its count wall is lower than main's.

  This gate is a bar, not a stop. A miss is recorded first in the REPORT, with its cause, and
  the unit still lands if G1–G4 pass.

## Decisions recorded here

- **No identity change.** The digest, the partition structs and their order stay as they are;
  only the build of each partition's vectors runs on several threads. [claude], reversible by
  Casey.
- **At most 8 builder threads.** claude-428's hash workers, two per order, run beside the sweep
  on the same 12-CPU container. [claude], reversible by Casey.
- **No pre-registration sheet,** as in claude-425–430: G4 counts and scores nothing. DEV-test is
  not read.

## Out of scope

- `download` (pinned host buffers would be a later unit), a device-side or parallel digest,
  validation, and the sort.
