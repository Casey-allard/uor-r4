# REPORT: written-state-count-model, claude-430 (each sort key gathered once per slot)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The sort's time moves by tens of seconds between runs of the same code on wick** [claude].
  Main's binary sorted in 138.5 s here and in 169.7 s in claude-429's run, 31.2 s apart, with no
  change to the code. The 4090 was shared both times with another container's `train-in-space`,
  at 91–98% use. So the comparison below is main's binary and the head's run back to back in one
  job, never the head against claude-429's figure. Against that figure the head's sort would look
  110.2 s faster, not the 79.0 s measured.
- **The ORDER's second negative could not give wrong counts; it was replaced** [claude]. The
  ORDER named a gather that reads `tokens[pos[i] - back - 1]`. At the first target of the oldest
  slot, j = back, so that read is t[-1]. It wraps to an address outside the stream and would end
  the test process with a CUDA fault instead of a wrong count. The negative run instead skips the
  gather on every slot but the left digit and `next` (`if back == 0 || back & LEFT_DIGIT != 0`), so
  those slots sort on stale keys. It tests the same claim, that each slot needs its own gather.
- **No idle-GPU timing.** The GPU was shared during both timed runs. Just before each, with ours
  not yet started, it read 91% (main) and 98% (head) in use, and during them 98.1% and 97.6% on
  average, sampled each second. The container cannot list the other container's process;
  `wick.sh gpu` on the host showed `train-in-space` before the job. How much faster either binary
  is on an idle GPU was not measured.
- **The split of the remaining sort time is not measured.** Random token reads per position fell
  to about a quarter (What changed, below), but sort time fell to 43%. The rest is the sequential
  passes, the scans and the scatter's writes; no timer separates them.

## What changed

1. **One gather per slot** (`ngram_count.cu`). A new kernel, `ngram_radix_gather`, writes each
   position's whole key word for one slot into a `u16` array: `tokens[pos[i] - back]`, or the
   13-bit left digit L when `back & LEFT_DIGIT`. `radix_key` is the old `radix_digit` without its
   shift and mask, so the reads and their bounds are the same.
2. **Digits from the key array.** `ngram_radix_hist(keys, n, shift, tiles, hist)` and
   `ngram_radix_scatter(keys_in, keys_out, pos_in, pos_out, n, shift, tiles, offsets)` take the
   digit as `(keys[i] >> shift) & 63`. The scatter writes each key beside its position, with the
   same stable rank, so the keys stay aligned with the positions for the slot's next pass.
3. **`sort`** (`ngram_count_device.rs`) runs one gather per slot, then that slot's passes: three
   for L (shifts 0, 6, 12), two for each token slot (0, 6). The slots, the shifts and their order
   are unchanged, so the output is the same permutation. Two `u16` key buffers of the partition's
   size are added, at most 64 MiB at budget 2^24.
4. **Random token reads per position,** for order k, fall from 4(k + 1) to k + 1, plus 12 to 2
   for L when k < N. For order 5 with L, that is 36 to 8.
5. **`build.rs` and `NGRAM_COUNT_ENTRIES`** list 21 kernels; `kn-reference` writes
   `unit: claude-430`. No test is changed.

## Measurements

wick 4090 (shared, above), all of TRAIN against DEV-tune's keys, N = 10, `--count-only
--profile-phases`, budget 2^24, job `c430`. Main's binary (`34eff518f`, built from the head's
tarball with `code.patch` reversed) ran first, then the head's (`2f48351f3`). Packet:
[`run1/c430/`](run1/c430/).

- **Output unchanged.** Both `count-10.json` files without `unit` (`jq -S 'del(.unit)'`) are
  byte-identical to claude-429's (`cmp`, on wick and again on the laptop). Main's file has claude-429's sha256,
  `a04e402e2f40555cb19a3e142af031691a01f1752938875267bff02bdf3b273e`; the head's
  (`d0540bab7c24797853200e90956880ffb6083a4a8385c50156623f3827dd7148`) differs only in its
  `unit` line.
- **Wall time.** The process took 374 s (main) and 300 s (head); the count 363.2 s and 288.0 s.
- **Memory.** Peak RSS 8,207,624 KiB (main) and 8,207,744 KiB (head). Whole-device GPU peak
  19,985 MiB and 20,017 MiB, with 3,402 MiB and 3,978 MiB in use before each run. On a shared
  device these cannot be split between the two processes.

Count phases in seconds, from the raw ms in each log. Own count holds histogram, select, sort,
emit, download and records.

| phase | main | head | change |
|---|---|---|---|
| sweep wall | 363.2 | 288.0 | −75.2 |
| own count | 314.3 | 239.4 | −74.9 |
| select | 29.1 | 29.0 | −0.0 |
| sort | 138.5 | 59.5 | −79.0 |
| emit | 23.4 | 24.1 | +0.7 |
| download | 40.8 | 41.9 | +1.0 |
| records | 81.1 | 83.6 | +2.4 |
| push | 33.0 | 32.9 | −0.1 |
| keep | 15.7 | 15.5 | −0.2 |

Histogram is 0.2 s in both; runs and adjust are 0 in both.

## What the numbers say

- **G5 passes on both counts.** The head's sort is 43.0% of main's in the same pair (59.5 s
  against 138.5 s), under the 70% bar, and its count wall is 75.2 s lower.
- **The random reads were most of the sort.** Taking them out removed 57% of its time; the
  sequential passes that remain cost 59.5 s.
- **Every other phase moved by 2.4 s or less,** inside wick's run-to-run variance; none is
  credited to this unit.
- **Records is now the largest phase,** 83.6 s, 29.0% of the head's count. The host phases
  (download, records, push, keep) are 173.9 s, 60.4%; the device phases (histogram, select, sort,
  emit) are 112.9 s, 39.2%, and sort alone 20.7%.

## Checks run at the head

- **G1, laptop** at `2f48351f3`:
  - host `kn_` and `ngram_count` tests: 39/39;
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests`: clean, no new
    warning (the four warnings are old ones in `uor-r4-core`);
  - the instrument's `cargo check --features cuda`: clean; its tests 7/7 and 2/2;
  - rustfmt hunks: 35 in `ngram_count_device.rs` against main's 37 (the new block is in rustfmt's
    layout, and it replaced two of main's), 0 in `build.rs`, 2 in the instrument, as on main;
  - no decimal literal or other D32 pattern in the added lines, and no softmax, sigmoid, tanh,
    exponential or matrix multiply.
- **G2, wick** at `2f48351f3`, job `c430`: `ngram_count_cuda`, `kn_score_cuda` and `kn_filter`,
  24 passed, 0 failed, 4 ignored (the four that need real token streams, ignored as before), in
  88.1 s. They include the raw count against `oracle_count` and the adjusted device sweep against
  `oracle_adjusted` and the host-runs sweep, at N in {1, 3, 10} and three budgets.
- **G3, negatives,** wick job `c430n` at `2f48351f3`, each on a fresh copy of the head with one
  sed checked to match its line once and leave none. Each runs
  `ngram_count_cuda_matches_the_oracle` and `ngram_count_cuda_device_sweep_matches_the_oracle`.
  Packet: [`run1/c430n/`](run1/c430n/).
  - **The scatter drops the keys** (`keys_out[slot] = key;` deleted): both tests fail. The raw
    count is refused with "n-gram triples are not in strictly ascending order", and the sweep
    with "unigram continuation: order-1 triples are not strictly ascending across partitions".
  - **Gather on the L and `next` slots only** (the replacement for the ORDER's negative, first
    section): both tests fail with the same two refusals.
  - Both fail at the partition assembler's order check, before any count is compared. A sort on
    stale keys puts grams out of order, and the assembler refuses that first. The unchanged head
    passes both tests (G2).
- **G4, wick,** job `c430`: both runs finished (rc 0), and both `count-10.json` files without
  `unit` equal claude-429's. PASS.
- **G5:** PASS on both counts (above).
- **Review:** a fresh-context review of `0ad9a1ce1..2f48351f3`, given the ORDER and the diff only,
  gave APPROVE with no HIGH or MEDIUM finding and two LOWs:
  - the third left pass (shift 12) sees a nonzero digit only where L = 4096, so only where token
    4095 precedes a gram. The test streams draw one id in four uniformly from 0..4096, so a stream
    of a million tokens is expected to hold about 61 of them, and the N = 10 streams of 200,000
    about 12. That is an expectation, not a count. G4's byte-identical file over all of TRAIN
    covers the pass on real data as well;
  - the new launch chains are one call per line, unlike the file's long chains. That is rustfmt's
    layout, and it lowers the file's hunk count (G1).

## Skills used

None beyond the loop's own steps. The unit changes how the counter sorts, not a model's state,
its operators or the artifact; its check is the byte-identical `count-10.json`.

## Still open

- **Records (83.6 s) and download (41.9 s)** stay on the host while the digest stays sequential.
  A parallel digest on the device would remove most of them and change the replay identity; that
  is a decision for a later ORDER.
- **The sort's remaining 59.5 s:** wider digits would halve the passes and the scans, at a larger
  per-tile histogram. Measure the passes alone first, on an idle GPU if wick has one.
- **Validation** (part of push) and the three small claude-428 follow-ups (an `Err` when a hash
  worker has stopped, the unused public `context_records`, the queue-full case not measured) are
  unchanged.

## Plain summary

The counter sorts every position in the training text by the words before it, six bits at a
time. Each step used to look up the word for every position in a 2 GB array, twice, in random
order, which is slow on a graphics card. Now each word is looked up once and carried along with
its position through the steps that use it. On the same shared card, one run after the other,
the sort took 59.5 seconds instead of 138.5, and the whole count 288 seconds instead of 363. The
counts are byte for byte the same.
