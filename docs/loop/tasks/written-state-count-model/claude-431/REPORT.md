# REPORT: written-state-count-model, claude-431 (each partition's records built on several threads)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **G4 could not run on wick; it ran on a rented 5090 instead** [claude]. The ORDER named wick's
  4090 for both binaries. Three wick jobs ran out of GPU memory (`CUDA_ERROR_OUT_OF_MEMORY`):
  - `c431`, budget 2^24: main's count finished (rc 0), and the head's stopped after 55 s. The
    other container's processes held 6,383 MiB before main's run and 5,263 MiB before the
    head's, and the whole device peaked at 23,896 MiB in the head's.
  - `c431b`, budget 2^24, each count waiting for 18 GiB free and retried on an out-of-memory
    error: main failed all three attempts, 141–143 s in, with 4,687–5,615 MiB in use before
    each and whole-device peaks of 23,286–23,990 MiB.
  - `c431c`, budget 12 × 2^20 targets: main failed three more times, 140–142 s in, with
    6,639–7,503 MiB in use before each.
  The budget caps the targets per partition (about 16.7 million), not device memory, so it did
  not move the peak: the count needs about 16 GB on its own (16,028 MiB on the idle 5090), and the
  other container's training processes, which grow during a run, left too little. This is the
  instrument, not the change: main failed the same way. G4 and G5 then ran on one Vast RTX 5090
  (`vast-61321505`, about 52 min at $1.042/h by the launch log), main and head back to back, at the ORDER's budget
  and with the ORDER's comparison. Packets: [`run1/c431/`](run1/c431/),
  [`run1/c431b/`](run1/c431b/), [`run1/c431c/`](run1/c431c/), and the pair
  [`run1/c431p/`](run1/c431p/).
- **The timings are from a different machine than claude-430's,** an AMD EPYC 9334 host with
  128 visible CPUs. Main's records took 234.1 s there, against 86.5 s on wick in `c431`. So the
  pair below compares main and head on the same machine, never either against wick's figures.
- **Peak RSS rose by 1.76 GiB** (8,502,616 KiB main, 10,343,184 KiB head). The ORDER said no new
  memory, and the builder allocates each vector once at its final size, as before. The cause was
  not measured. A likely one: records now finish sooner, so more finished partitions wait at once
  in claude-428's hash-worker queues.
- **The device negative ran on the 5090, not on wick** (G3 named wick), in the same pod job after
  the pair.
- **The device negative fails by a panic, not a wrong count.** With each chunk starting one element
  late, the last chunk reads one element past the downloaded arrays (`keys[2 * g]`,
  `ngram_count_device.rs:741`, "index out of bounds: the len is 195568 but the index is
  195568"), and `thread::scope` re-raises it as "a scoped thread panicked". Both oracle tests
  fail, before any count is compared.
- **A builder thread that cannot be spawned panics** (review LOW-3). `thread::scope` has no
  fallible spawn. The sequential path it replaced could not fail this way. Accepted: a host that
  cannot start 8 threads cannot run the sweep's hash workers either.

## What changed

1. **`build_indexed(n, threads, f)`** (`ngram_count.rs`, host code, `pub` like the file's other
   device-only helpers) returns the `Vec` whose element i is `f(i)`. It splits 0..n into at most
   `threads` contiguous chunks of `n.div_ceil(threads)` and fills each on a scoped thread through
   `spare_capacity_mut`, then sets the length. Below 2^16 elements, or with one thread, it maps on
   the caller. Its output does not depend on `threads`.
2. **`count`** (`ngram_count_device.rs`) builds `partition.triples` and then `partition.contexts`
   with it. Each closure is the old loop body. The thread count is `available_parallelism()`
   capped at 8 (`MAX_BUILD_THREADS`), read once in `new`.
3. **`records`** still times both builds. No phase is added or renamed.
4. **`kn-reference`** writes `unit: claude-431`. No test is changed; one host test is added.

## Measurements

Vast RTX 5090 (32,607 MiB, driver 580.95.05; AMD EPYC 9334, 128 CPUs visible, 792 GB), alone on
the device (2 MiB in use before each run). All of TRAIN against DEV-tune's keys, N = 10,
`--count-only --profile-phases`, budget 2^24. Main's binary (`96c8841d2`, the head's tarball with
`code.patch` reversed) ran first, then the head's (`ceb1da88d`). The four input streams were
copied from wick; both count files record their sha256 (below).

- **Output unchanged.** Both `count-10.json` files without `unit` are byte-identical to
  claude-430's `run1/c430/count-10-head.json` without `unit` (`cmp`). Main's whole file has that
  file's sha256, `d0540bab7c24797853200e90956880ffb6083a4a8385c50156623f3827dd7148`, inputs
  included (`train_sha256` `c6cba29e…`, `query_sha256` `75c6d1ac…`). The head's,
  `8fdbb98d8d1b764b0514002dfeff30921e4652e79e3304e881cec03d04c7ebe7`, differs only in its
  `unit` line.
- **Wall time.** The process took 465 s (main) and 283 s (head).
- **Memory.** Peak RSS 8,502,616 KiB and 10,343,184 KiB; GPU peak 16,028 MiB for both.

Count phases in seconds, from the raw ms in each log. Own count holds histogram, select, sort,
emit, download and records.

| phase | main | head | change |
|---|---|---|---|
| sweep wall | 444.8 | 262.7 | −182.1 |
| own count | 367.9 | 184.4 | −183.5 |
| select | 10.4 | 10.5 | +0.0 |
| sort | 16.9 | 17.0 | +0.1 |
| emit | 7.5 | 7.7 | +0.2 |
| download | 94.4 | 99.3 | +5.0 |
| records | 234.1 | 42.8 | −191.3 |
| push | 49.7 | 50.3 | +0.6 |
| keep | 27.0 | 27.8 | +0.8 |

Histogram is 0.2 s in both; runs and adjust are 0 in both.

## What the numbers say

- **G5 passes on both counts.** The head's records is 18.3% of main's in the same pair (42.8 s
  against 234.1 s), under the 50% bar, and its count wall is 182.1 s lower.
- **Eight threads gave 5.5 times,** not 8. The vectors are written into new memory at about 11 GB/s
  in all (476 GB in 42.8 s), so the build is now likely bound by memory bandwidth, not by cores.
  That was not measured.
- **Every other phase moved by 5.0 s or less.** Download moved most (+5.0 s, 5%); this unit does
  not touch it, and no change in it is credited to the unit.
- **The host now holds the count.** On this machine download (99.3 s) is the largest phase, 37.8%
  of the head's sweep wall. Of that wall, the host phases (download, records, push, keep) are
  220.2 s, 83.8%; the device phases (histogram, select, sort, emit) are 35.3 s, 13.5%.

## Checks run at the head

- **G1, laptop** at `ceb1da88d` (and at `0caa3d097` before the visibility fix):
  - host `kn_` and `ngram_count` tests: 40/40, with the new
    `ngram_count_build_indexed_equals_the_sequential_map` (n in {0, 1, 7, 2^16 − 1, 2^16,
    2^16 + 1, 1,000,003}, threads in {1, 2, 3, 8, 64});
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests`: clean, no new
    warning (the four warnings are old ones in `uor-r4-core`);
  - the instrument's `cargo check --features cuda`: clean; its tests 2/2 and 7/7;
  - rustfmt hunks equal main's in every touched file: 58 in `ngram_count.rs`, 35 in
    `ngram_count_device.rs`, 2 in the instrument;
  - no decimal literal or other D32 pattern in the added lines, and no softmax, sigmoid, tanh,
    exponential or matrix multiply.
- **G2,** `ngram_count_cuda`, `kn_score_cuda` and `kn_filter`: 24 passed, 0 failed, 4 ignored
  (the four that need real token streams, as before). On wick at `0caa3d097` (job `c431`) and at
  `ceb1da88d` (job `c431b`, 180.1 s), and on the 5090 at `ceb1da88d` (42.3 s).
- **G3, negatives,** chunks starting one element late (`let base = index * chunk + 1;`), one sed
  checked to match its line once and leave none:
  - laptop: the new host test fails; restored, it passes;
  - 5090: `ngram_count_cuda_matches_the_oracle` and
    `ngram_count_cuda_device_sweep_matches_the_oracle` both fail, by the panic described in the
    first section. The unchanged head passes both (G2).
- **G4,** on the 5090 (first section): both runs finished (rc 0), and both `count-10.json` files
  without `unit` equal claude-430's. PASS.
- **G5:** PASS on both counts (above).
- **Review:** a fresh-context review of `96c8841d2..0caa3d097`, given the ORDER and the diff
  only, gave APPROVE with no HIGH or MEDIUM finding and three LOWs:
  - LOW-1: `build_indexed` and its constant were `pub(crate)` and warned as unused without the
    `cuda` feature. Fixed in `ceb1da88d` (`pub`); G1 passed again there;
  - LOW-2: the device file's rustfmt hunk count; checked, 35 against main's 35;
  - LOW-3: a thread that cannot be spawned panics (first section).

## Skills used

None beyond the loop's own steps. The unit changes how the counter builds host vectors, not a
model's state, its operators or the artifact; its check is the byte-identical `count-10.json`.

## Still open

- **Download** (99.3 s on the 5090, 41.9 s on wick in claude-430) is now the largest phase.
  Pinned host buffers are the next unit.
- **The peak RSS rise** (1.76 GiB): measure the hash-worker queue depth before the next host unit.
- **`uor-pod down` on a Vast pod** failed its own archive step ("uor-r4/pods: Is a directory",
  then an SSH check). The pod's two folders were archived by hand to the tool's own place,
  `vast-archive/vast-61321505/`, and the pod deleted with `--confirm-archived`. The tool is not
  fixed here.
- **wick's 4090 no longer fits a whole-TRAIN N = 10 count** beside the other container's
  training. Units that need one run on a rented GPU until that changes.
- **Validation** (part of push), the sort's 17.0 s, and the three small claude-428 follow-ups are
  unchanged.

## Plain summary

After counting, the counter turns the raw arrays it copies back from the graphics card into the
records it keeps, about 12 billion of them at the top order. One core did all of that. Now up to
eight cores each build their own stretch of the same list, and the list comes out byte for byte
the same. On a rented card, one run after the other, that step took 42.8 seconds instead of 234,
and the whole count 263 seconds instead of 445. Casey's own card could not run the comparison,
because another lab's training left too little room on it.
