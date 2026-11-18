# ORDER — written-state-count-model (claude-426)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-426, fix 1 parts D1 and D2 from the device-push scout, as Casey decided on 18 Nov
("Start fix 1"):
- D1: the query filter keeps records with a forward cursor instead of a binary search;
- D2: each order's two SHA-256 streams are hashed on their own threads.

It follows claude-425 (PR #4081), whose N = 10 sweep spent 1,244.8 of its 1,447.0 s in host work
over every gram.

## Why

claude-425's G4 phase lines (`claude-425/run1/c425/count-10.log`), summed over orders 0..10:
- **`keep`: 386.8 s.** `KnFilter::keep` runs a `binary_search` of every triple into the order's
  sorted query pairs, and of every context into its query contexts: about 23 probes per record
  over 6.73 B triples and 5.44 B contexts. Both sides are sorted, and the filter already refuses
  records that fall, so a cursor that only moves forward does the same job in about one compare
  per record.
- **`push`: 252.9 s.** `NgramAssembler::push` validates each partition and feeds two SHA-256
  streams. The bytes hashed (6.73 B × 22 B + 5.44 B × 40 B, about 366 GB) at about 1.5 GB/s
  match one core's SHA-NI rate. Wick's Ryzen 9 9950X has SHA-NI and 32 hardware threads. The
  sweep keeps 11 assemblers, so 22 streams live at once, and each can run on its own thread.

The scout's estimates, not measurements: keep 379 s -> about 15-30 s; push 241 s -> about 35 s
(the slowest single stream, order 10's contexts).

## What it does

1. **D1, `KnFilter::keep` by cursor** (`kn_filter.rs`).
   - The filter holds one index into the order's sorted `pairs` and one into its `contexts`.
   - For each record, the cursor moves forward past smaller keys, then the record is kept only
     if the key at the cursor equals its own.
   - The cursors carry across partitions, as `last_triple` and `last_context` already do.
   - The ascent checks, the refusals and the seam rule for order-0 contexts are unchanged.
   - `KnExtensionCensus::observe` keeps its binary search: the suffixes it looks up do not
     ascend.
2. **D2, hashing on threads** (`ngram_count.rs`).
   - Each of an assembler's two streams gets a hasher thread, fed byte chunks over a bounded
     channel (std only, no new dependency).
   - The assembler encodes records with `triple_bytes` and `context_bytes` as now, in the same
     order, into a buffer it sends when full. `finish` sends the rest, joins the thread and
     reads the hash.
   - SHA-256 of a byte stream does not depend on how the stream is cut into chunks, so every
     digest is unchanged. All validation stays on the caller's thread, before any byte is sent.
   - A hasher thread that panics fails `finish`. Dropping an assembler without `finish` ends its
     threads.
3. **`kn-reference`**: the output `unit` becomes `claude-426`. Nothing else changes.

## Gates

- **G1, laptop:**
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda` are clean;
  - the host `kn_` and `ngram_count` tests pass, and the instrument's tests pass;
  - a new test: over random partitions and queries (including keys absent from the records,
    queries before the first record and after the last, and an order-0 context straddling a
    seam), the cursor filter keeps exactly the records a binary search keeps;
  - a new test: with a chunk size small enough to force many sends, the threaded assembler's
    digests equal SHA-256 computed inline over the same encoded records, for raw and adjusted
    domains.
- **G2, wick:** the `ngram_count_cuda`, `kn_score_cuda` and `kn_filter` tests pass at the head.
- **G3, negative gates** (each set aside against the base, run, expected to fail, restored):
  - a cursor that never moves past a non-matching key (so it keeps nothing after the first
    miss) fails the cursor test;
  - a hasher that drops the last partial chunk fails the digest test.
- **G4, wick:** one N = 10 `--count-only --profile-phases` run on all of TRAIN against DEV-tune's
  keys, budget 2^24, as claude-425's G4.
  - Its `count-10.json`, with `unit` deleted (`jq -S 'del(.unit)'`, then `cmp`), equals
    claude-425's `run1/c425/count-10.json` with `unit` deleted, in every field, `partitions`
    included: the plan is unchanged.
  - Its phase lines give `push` and `keep` against claude-425's, and the new wall time.
  - Peak resident memory is recorded beside claude-425's 11.0 GiB.
- **G5:** the REPORT states the measured saving per phase beside the scout's estimates,
  negatives first.

## Decisions recorded here

- **D1 and D2 in one unit.** Both are host-only, both keep every digest, and they edit different
  files. One G4 run checks both. [claude], reversible by Casey.
- **No pre-registration sheet.** As in claude-425: G4 counts and scores nothing, so it is a
  check of equal output, not an evaluation (§4.5). DEV-test is not read.

## Out of scope

- D3, keeping partitions on the device with a new parallel digest, is the next unit. Its
  identity change gets its own decision there.
- `records`, `download`, `runs` and `adjust` are not changed here.
- No change to what is counted or how it is hashed.
