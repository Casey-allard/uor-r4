# REPORT: written-state-count-model, claude-428 (per-order hash workers)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The work moved off the caller; it was not removed** [claude]. The two workers per order were
  busy 164.8 s in all: 63.0 s on the triple streams and 101.8 s on the context streams. In
  claude-427 the caller spent about 48.0 s encoding and merging, and the hashers 134.8 s. The
  sequential SHA-256 digest still costs the same per byte. It now runs beside the sweep instead
  of blocking it. Order 10's context worker alone was busy 22.6 s, which is about the hashing
  floor claude-427 named, now with the encoding added.
- **Validation is now most of push, and this unit did not touch it.** The per-record checks
  (10.9 s) and the census (17.8 s) take 28.7 s, 86.6% of the 33.1 s that remains.
- **The run's wall fell more than push did, and the excess is not credited here.** The sweep
  wall fell 134.7 s and push 117.3 s. The other timed phases fell 10.0 s together, though this
  unit changed none of them. Between claude-426 and claude-427, one phase moved 19 s with no
  change, so 10 s is inside wick's run-to-run variance.
- **Two timing names now mean something narrower.** "Triple feed" is now the counters
  (targets, grams, count-of-counts) plus building the `Arc` and handing it to the triple worker.
  It is no longer encoding or hashing. "Context feed" is only the hand-off to the context
  worker. "Hasher busy" is a worker's encoding plus hashing, so it is not like for like with
  claude-427's figure, which was hashing alone.
- **No test pins the memory bound.** Queue depth one per worker is set in the code and was
  measured on wick (below). No laptop test fails if a deeper queue is set, and no test observes
  a full queue, so "handed faster than the workers drain" is the test's intent, not a measurement.
- **The ORDER's backwards-context clause is met only through the triple check** (found in
  review). A partition's first context key equals its first triple key, and the triples are
  checked in strict order across partitions. So the cross-partition context order check after
  the census cannot fire on its own. The refusal test now asserts the triple-order message. The
  one check after the census that can fire is the order ≥ 1 seam refusal, and the first version
  of the test did not cover it. With the seam check moved after the sends, every test stayed
  green. The test now covers it (see the checks below).
- **A dead worker is reported late.** If a worker has stopped, `send` drops the partition and
  `push_owned` still returns `Ok`. The failure comes out as a panic at `finish`, not as an
  `Err`. This is unchanged from claude-426's hash threads.

## What changed

1. **The sweep hands each partition over by value** (`ngram_count_device.rs`). The sink is
   `FnMut(u32, NgramPartition)`, so the adjusted partition is moved, not copied, and the old
   `drop` after the sink is gone. `count_partition` and the CUDA path (`ngram_count_cuda.rs`)
   call `push_owned` too.
2. **`NgramAssembler::push_owned(partition)`** (`ngram_count.rs`).
   - **On the caller,** in order: the triple checks, the context checks, `check_census` and its
     compare, the seam check, and the cross-partition context order check. That last one moved
     here from the merge loop. It keeps its message, but it is now a backstop: the triple-order
     check refuses a backwards partition first. Then come the counters and, in tests, the kept
     triples.
   - **Then one `Arc<NgramPartition>` goes to two worker threads per order,** one for triples
     and one for contexts, over a `sync_channel` of depth one (`WORKER_QUEUE_DEPTH`).
     - Each worker encodes its records into a `chunk`-byte buffer (`ChunkedHash`) and hashes
       the buffer when it fills, then the tail at `finish`.
     - The context worker also merges the order-0 context across partitions, counts the
       emitted contexts, and keeps them in tests.
     - A worker that panics is re-raised at `finish` with `resume_unwind`. A worker that stops
       early fails `finish` loudly. `Drop` joins the thread.
   - **`push(&partition)`** still works, through `push_owned(partition.clone())`.
3. **Timing** (claude-427's `AssemblerTiming`) keeps its fields, with the meanings above.
4. **`kn-reference`** sinks the partition by value. It keeps it (the query filter), observes it
   (the census), then calls `push_owned`. If several of these refuse, the error reported is the
   same one as before. The instrument writes `unit: claude-428`.
5. **Tests:**
   - `ngram_count_owned_push_equals_inline_sha` covers orders 0–3, with wide and narrow (64)
     partitions and chunk sizes 1, 7, `TRIPLE_BYTES` and 1 MiB, alternating `push_owned` and
     `push`. The triples, contexts, counts and both digests equal a whole-stream inline SHA-256.
     At order 0, at least three partitions hold the one context, so the merge across partitions
     is exercised. Partitions are handed faster than the workers drain.
   - `ngram_count_refused_partition_hands_nothing_on` covers orders 1–2.
     - A planted count-of-counts is refused ("count-of-counts").
     - A partition handed again is refused by the triple-order check, with its exact message.
     - After both refusals, `finish` equals a clean assembler's.
     - At order 1, an order-1 context split across two partitions is refused ("straddles a
       partition seam"), and `finish` again equals a clean assembler's.

     So a refused partition hands nothing on.

## Measurements

G3, wick 4090, job `c428`, code `9aa35386c`. The run used all of TRAIN (2,091,560,001 tokens)
against DEV-tune's keys, with N = 10, budget 2^24, `--count-only --profile-phases`.
Packet: [`run1/c428/`](run1/c428/).

- **Wall time.** The process took 765 s (claude-427: 900 s), and the count 752.8 s (887.5 s),
  over 128 shared ranges and 1,408 partitions.
- **Output unchanged.** `count-10.json` with `unit` deleted (`jq -S 'del(.unit)'`) is
  byte-identical to claude-427's (`cmp`, on wick and again on the laptop). The file's sha256 is
  `7c238b4fdc23786cf4869825bf6cf29f1d4b4927c4fceb88200ad9db423f2fc3`. It differs from
  claude-427's only by `unit`.
- **Memory.** Peak RSS was 12,359,404 KiB, +715,836 KiB (699 MiB) on claude-427. The
  whole-device GPU peak was 20,081 MiB on a shared device; 3,978 MiB were in use before the run.

Push, all orders, in ms. The five sub-phases are consecutive splits of one stopwatch.

| sub-phase | claude-427 | claude-428 | share of push now |
|---|---|---|---|
| triple checks | 5,976 | 5,583 | 16.9% |
| context checks | 5,520 | 5,268 | 15.9% |
| census | 17,738 | 17,806 | 53.8% |
| triple feed | 47,495 | 4,446 | 13.4% |
| … of which send wait | 29,428 | 6 | |
| context feed | 73,671 | 2 | 0.0% |
| … of which send wait | 43,731 | 2 | |
| **push total** | **150,403** | **33,108** | sub-phases sum to 33,105 |
| busy, triple streams (other threads) | 54,676 hashing | 63,028 encoding + hashing | |
| busy, context streams (other threads) | 80,134 hashing | 101,774 encoding + merging + hashing | |

Order 10 alone took 6,433 ms of push (claude-427: 32,037). Its workers were busy 12,135 ms
(triples) and 22,562 ms (contexts).

Count phases in seconds, from the raw ms in each `count-10.log`:

| phase | claude-427 | claude-428 | change |
|---|---|---|---|
| sweep wall | 887.5 | 752.8 | −134.7 |
| own count (device) | 297.5 | 292.7 | −4.7 |
| select | 28.9 | 28.8 | −0.1 |
| sort | 111.6 | 110.4 | −1.2 |
| emit | 18.9 | 18.6 | −0.3 |
| download | 43.0 | 41.5 | −1.4 |
| records | 93.1 | 91.4 | −1.7 |
| runs | 268.3 | 270.1 | +1.8 |
| adjust | 124.5 | 122.7 | −1.8 |
| push | 150.4 | 33.1 | −117.3 |
| keep | 16.4 | 15.6 | −0.7 |

Host work over every gram (download, records, runs, adjust, push, keep) is 574.5 s, 76.3% of
the count (claude-427: 695.6 s, 78.4%).

## What the numbers say

- **G4 passes on both counts.** Push fell 117.3 s (78.0%), against a bar of at least 36.6 s.
  Peak RSS rose 699 MiB, against a ceiling of 4 GiB. Both send waits are now under 10 ms, so the
  caller no longer waits on hashing.
- **Why memory rose so little.** Each worker holds at most the partition in progress and one
  queued, and a partition's `Arc` is freed as soon as both its workers finish with it. An
  order's two workers can be on different partitions, so an order holds at most two partitions
  beyond the caller's. In the worst case that is about 20 across the ten orders. The rise
  is close to the size of one average order-10 partition (about 740 MiB), well under the 4 GiB
  the ORDER allowed for in the worst case. The run does not show which partitions were held.
- **The workers kept up.** Between the sweep wall and the process wall there are about 12 s in
  both runs (900 − 887.5 s, and 765 − 752.8 s), so the join at `finish` left no long tail. The
  busiest stream (order 10's contexts, 22.6 s) is far below the 752.8 s it had to spread over.
- **What is left of push is validation.** The census and the checks are 28.7 s, 86.6% of push.
  They are what refuse a wrong partition, so taking them off the caller's path needs the same
  refuse-before-hand-on property this unit kept.
- **The digest question for D3 is now narrower.** The sequential digest no longer costs the
  sweep any waiting. Keeping it, D3 still needs the bytes on the host, so download and records
  (132.9 s) remain. A parallel digest on the device would change the replay identity, for those
  133 s plus the 164.8 s of worker time that now runs beside the sweep.

## Checks run at the head

- **G1, laptop** at `9aa35386c`, and again after the review's test-only change (the engine
  code wick ran is unchanged):
  - host `kn_` and `ngram_count` tests: 38/38 (36 before, plus the two new ones);
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests`: clean;
  - the instrument's `cargo check --features cuda`: clean, and its tests 7/7. A first try at
    the head stopped with "No such file or directory" while the laptop's disk was nearly full;
    the rerun, with space freed, was clean;
  - rustfmt hunk counts unchanged from main: 58 in `ngram_count.rs`, 37 in
    `ngram_count_device.rs`, 29 in `ngram_count_cuda.rs`, 2 in the instrument;
  - no decimal literal or banned pattern under D32.
- **G2, negatives,** each set aside, run, then restored with `cmp` equal to the saved file and
  rebuilt green:
  - with the context worker's order-0 merge disabled, `ngram_count_owned_push_equals_inline_sha`
    fails with the order-0 contexts unmerged;
  - with the worker's tail buffer never hashed, it fails on a digest mismatch at order 0 with a
    1 MiB chunk;
  - added after review: with the seam and context-order checks moved after both sends,
    `ngram_count_refused_partition_hands_nothing_on` fails on its seam `finish` equality, and
    every other `ngram_count` test passes (25 of 26). Restored, then `cmp` equal, and the narrow
    suite was 38/38 again.
- **Review:** a fresh-context review at `434bf9425` gave APPROVE, with one MEDIUM (the
  backwards-context clause and seam coverage above, fixed in the test) and four LOWs. One LOW was
  wording (the per-worker memory bound, fixed). The other three are listed above and under Still
  open.
- **G3, wick** at `9aa35386c`: PASS (above).
- **G4:** PASS (above).

## Skills used

None beyond the loop's own steps.

## Still open

- **D3, device residency,** with its own ORDER. The numbers above narrow its choice. With the
  sequential digest kept, download and records (132.9 s) stay on the host. A parallel digest
  removes them but changes the replay identity.
- **Validation** (28.7 s of the 33.1 s left in push). It could move off the caller only if a
  refusal still stops a partition before any worker sees it.
- **The largest host phases are now `runs` (270.1 s) and `adjust` (122.7 s),** 52.2% of the count
  together, and the next host targets after D3's choice.
- **Run-to-run variance on wick:** a gain smaller than about 20 s in one phase still needs
  repeat runs to be believed.
- **Small code follow-ups from review,** none of which moves a byte:
  - `push_owned` could return `Err` when a worker has stopped, instead of only panicking at
    `finish`;
  - `NgramAssembler::context_records` is public but no longer filled while pushing, since kept
    contexts come back from `finish`. It has no reader today; it could be removed or made private.

## Plain summary

The step that hands each order's counts to the fingerprinting code took two and a half minutes,
half of it the main thread waiting. Now each order has two helper threads. They take each batch
of counts, prepare the bytes and compute the fingerprint while the main thread moves on. That
step now takes 33 seconds, and the whole count finishes about two minutes sooner (12.5 minutes
instead of 14.8). It uses about 0.7 GB more memory. Every output byte is unchanged, and the
fingerprint is computed exactly as before.
