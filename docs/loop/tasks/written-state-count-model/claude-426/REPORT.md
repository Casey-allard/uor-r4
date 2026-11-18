# REPORT: written-state-count-model, claude-426 (keep by cursor, hash on threads)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **`push` fell only to 148.2 s, not the scout's ~35 s** [claude]. Hashing on 22 threads took
  104.7 s off claude-425's 252.9 s. What remains runs on the caller's thread: the per-partition
  checks, `check_census`, encoding records into the send buffers, and any wait on a full
  channel. Which of those dominates is not measured. Order 10 alone spends 31.6 s in push, and
  the per-order push times rise with the order (0.4 s at k = 2, 12.3 s at k = 5, 31.6 s at
  k = 10), as the gram counts do.
- **Host work over every gram is still 78.8% of the count** (700.5 of 889.3 s). `runs`
  (287.3 s), `adjust` (119.6 s), `push` (148.2 s), `records` (89.1 s) and `download` (40.6 s)
  remain. 3–5 minutes needs D3, keeping partitions on the device.
- **Peak resident memory rose 137,844 KiB (about 135 MiB),** from 11,499,308 to 11,637,152 KiB.
  Each of the 22 hash streams holds up to six 4 MiB buffers (one filling, four queued, one being
  hashed), so the bound is about 528 MiB; the measured rise is under that.
- **Other host phases also moved, and the cause is not measured.** `records` (−21.6 s),
  `adjust` (−23.6 s), `runs` (−16.2 s) and `download` (−7.2 s) fell, though this unit did not
  change them. Candidates: the binary searches no longer evict the caches the other host steps
  use, or a different load on wick (load average 5.4 at the start; the 4090 had 3.9 GiB in use
  by others, as in claude-425).
- **G2 and G4 ran at `66e812bb9`.** The review fixes in `564e0b56d` came after: a hasher thread
  that cannot start is now an `Err` from `NgramAssembler::new` rather than a panic, and a test
  doc comment moved back to its test. Neither touches a digest or a kept record. They were
  checked on the laptop only (host tests 35/35, stub-nvcc check clean).

## What changed

1. **D1, `KnFilter::keep` by cursor** (`crates/uor-r4-discrete/src/kn_filter.rs`). The filter
   holds a forward index into the order's sorted query `pairs` and one into its `contexts`.
   For each record the index moves past smaller keys, and the record is kept only if the key
   there equals its own. The indexes carry across partitions. The ascent checks, refusals and
   the order-0 seam merge are unchanged. `KnExtensionCensus::observe` keeps its binary search.
2. **D2, hashing on threads** (`crates/uor-r4-discrete/src/ngram_count.rs`). A `StreamHasher`
   per stream owns a thread fed 4 MiB chunks over a channel of depth 4. The records are encoded
   with `triple_bytes`/`context_bytes` in the same order as before. `finish` sends the tail,
   joins and reads the hash; a panic on the thread is re-raised there. Dropping an assembler
   ends its threads.
3. **`kn-reference`** writes `unit: claude-426`.
4. **Tests:** `kn_filter_cursor_keeps_as_a_binary_search` and
   `ngram_count_threaded_hash_equals_inline_sha`.

## Measurements

G4, wick 4090, job `c426b`, code `66e812bb9`: all TRAIN (2,091,560,001 tokens) against
DEV-tune's keys, N = 10, budget 2^24, `--count-only --profile-phases`. Packet:
[`run1/c426/`](run1/c426/).

- Process wall 904 s (claude-425: 1,464 s). The count took **889.3 s** (1,447.0 s), over the same
  128 shared ranges and 1,408 counted partitions.
- `count-10.json` with `unit` deleted (`jq -S 'del(.unit)'`) is byte-identical to claude-425's
  (`cmp`, on wick and again on the laptop): every order's grams, contexts, count-of-counts,
  triple and context digests, discounts, partitions and kept records. Kept triples total
  23,189,023. The file's sha256 is
  `1d90389b1217e952a0c6127cb112ae1e2261b856ee37546c137c94d1365d5226`.
- Peak RSS 11,637,152 KiB; whole-device GPU peak 20,017 MiB (shared device).

Phase totals over orders 0–10, in seconds, from the raw ms in each `count-10.log`:

| phase | claude-425 | claude-426 | change | scout estimate |
|---|---|---|---|---|
| sweep wall | 1,447.0 | 889.3 | −557.7 (−38.5%) | |
| own count (device) | 323.5 | 290.6 | −33.0 | |
| select | 29.3 | 29.2 | −0.1 | |
| sort | 114.4 | 111.1 | −3.4 | |
| emit | 19.2 | 18.8 | −0.4 | |
| download | 47.8 | 40.6 | −7.2 | |
| records | 110.7 | 89.1 | −21.6 | |
| runs | 303.6 | 287.3 | −16.2 | |
| adjust | 143.2 | 119.6 | −23.6 | |
| **push** | 252.9 | **148.2** | −104.7 | ~35 |
| **keep** | 386.8 | **15.8** | −371.0 | ~15–30 |

## What the numbers say

- The cursor did what the scout said: `keep` fell 96%, to 15.8 s, inside the 15–30 s estimate.
- Threaded hashing cut `push` by 41%: 104.7 s, about half the scout's estimated cut of 206 s
  (241 s to about 35 s, from claude-423's push). Hashing was not all of
  push; the rest of it is on the caller's thread and is the next thing to time.
- The count is 38.5% faster with every digest unchanged. 557.7 s came off, against the scout's
  combined estimate of about 555–570 s for D1 + D2; the miss on push was offset by the unexplained
  falls in other host phases.

## Checks run at the head

- **G1, laptop** at `66e812bb9`, and again at `564e0b56d`: host `kn_` and `ngram_count` tests
  35/35 (33 before, plus the two new ones); stub-nvcc `cargo check -p uor-r4-discrete --features
  cuda --lib --tests` clean; at `66e812bb9`, the instrument's `cargo check --features cuda` clean and its
  tests 7/7.
- **G2, wick** at `66e812bb9`: `ngram_count_cuda`, `kn_score_cuda` and `kn_filter` 21 passed,
  0 failed, 4 ignored (the real-stream tests, which need UORT streams); the cursor test ran
  there ([`cuda-tests.log`](run1/c426/cuda-tests.log)).
- **G3, negatives** at `66e812bb9`: both fixes were sabotaged in place on that commit (not set
  aside against the base, as the ORDER worded it), in one build, then restored with
  `git checkout` and rebuilt green:
  - a cursor that never moves (`while false && …`) fails the cursor test: order 0 from
    partition 0 kept no triples;
  - a `finish` that drops the tail chunk fails the hash test, at order 0 with chunk 40.
- **G4:** PASS, above.
- **Review:** a fresh-context reviewer read the ORDER and the diff at `66e812bb9`: APPROVE, no
  HIGH or MEDIUM. Of its three LOWs, two are fixed in `564e0b56d` (the spawn error, the moved
  doc comment); the third, the memory bound, is measured above.

## Skills used

None beyond the loop's own steps.

## Still open

- **Time the rest of `push`.** Split the caller's thread into checks, `check_census`, encoding
  and channel wait, before deciding whether D3 should take push with it.
- **D3, device residency:** adjust, runs, digest and filter on the device, downloading only the
  ~45 M kept records. It needs a parallel digest, which changes the replay identity; that
  decision belongs to its own ORDER.
- **Why the other host phases fell** is not measured; a second run would show whether it holds.

## Plain summary

The count that builds the reference scorer's tables now takes about 15 minutes instead of 24,
with every output byte unchanged. The filter now walks its sorted lists in step instead of
searching them for every record, and that step went from 6.4 minutes to 16 seconds. Hashing on
parallel threads helped less than hoped: 4.2 minutes became 2.5, because part of that step was
never hashing. Most of the remaining time is still host-side work on every gram, which the next
unit moves onto the GPU.
