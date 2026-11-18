# REPORT: written-state-count-model, claude-429 (adjusted counts on the device)

Lab: claude. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **The device paid 70.6 s for what the host did in 392.8 s, more than the pass count predicts**
  [claude]. Sort rose 59.2 s (+53.7%) and emit 11.4 s. The left digit adds 3 radix passes to each
  order below N: 27 passes on top of claude-428's 130 over orders 1–10, which is +20.8%. Each left
  pass reads two tokens per position and branches, where a token pass reads one. The GPU was also
  shared (below). This run cannot say how much of the extra comes from each.
- **About 18.5 s of the old wall was never timed, and it is gone; this unit does not show why.**
  claude-428's timed phases cover 734.3 s of its 752.8 s sweep wall, and claude-429's cover 408.2 s
  of 408.5 s. The gap was most likely the building and freeing of runs between orders, which no
  longer happens, but no timer measured it.
- **The ORDER's second negative, as written, could not fail the model test** [claude]. The ORDER
  said that with L not requiring t[j-k] ≠ BOS, G1's model test would fail. In the first version
  of the test it passed. t[j-k] is the oldest slot of the order-k gram at j, so the condition is
  exactly "this gram is not a document start". A document-start gram keeps its raw count, so its
  N1+ never reaches the output. The condition shows only through the check that refuses a
  document-start gram with N1+ > 0. The model test now makes that check, as the device does, and
  the negative fails as a refusal (G3 below).
- **That document-start check cannot fire while L is right** (fresh review, LOW). With L's BOS
  condition in place, a document-start gram always has N1+ = 0. The check guards the condition;
  only the G3 set-aside reaches it, on the laptop and on wick.
- **The builder's code was not formatted to the project's counts.** Its new lines were longer than
  rustfmt allows: 41 hunks against main's 37 in `ngram_count_device.rs` and 36 against 29 in
  `ngram_count_cuda.rs`. They were hand-formatted to rustfmt's layout (commit `4373d5ab4`), and only
  the lines this unit added or changed. No behaviour changed; every check below ran after it.
- **The GPU was shared during the timed run.** Another process (`train-in-space`) used the 4090 at
  98% with 3.8 GiB when the job started. The device phases may be slower than on an idle GPU, and
  this run cannot say by how much.
- **N = 10 runs on 200,000 tokens in the two new CUDA tests,** not on a million, because the host
  oracle and the host-runs sweep are slow at N = 10. N = 1 and N = 3 run on a million.

## What changed

1. **The sort takes a left digit** (`ngram_count.cu`, `ngram_count_device.rs`). For order k < N,
   `sort(.., left)` first runs three 6-bit radix passes (shifts 0, 6, 12) on
   L(j) = t[j-k-1] + 1 when j > k and t[j-k] is not BOS, else 0. The existing passes and their order
   are unchanged, so L is the least significant key. No new kernel: the radix kernels' `back`
   argument carries a flag bit (`LEFT_DIGIT = 0x100`) with the order in the low byte, because
   build.rs checks the compiled kernel list by name.
2. **Left flags and the adjusted emit.**
   - `ngram_boundaries` takes `left` and writes `left_flag[r] = 1` where L is nonzero and either a
     new gram starts or L differs from r-1's. An exclusive scan of the flags gives each gram its
     N1+(• g) as `left_scan[r1] - left_scan[r0]`.
   - `ngram_emit_grams` takes `left_scan`, `adjust` and `violations`. In adjust mode a gram whose
     oldest slot is BOS keeps its raw count; any other gram gets N1+. Rule 4 (a gram that is not a
     document start with N1+ = 0) and a document start with N1+ > 0 are counted with integer
     `atomicAdd`, and the least gram index of each kind is kept with `atomicMin`.
   - Context totals come from an exclusive scan of the adjusted counts, passed to the unchanged
     `ngram_emit_contexts` in place of the gram starts. Count-of-counts come from adjusted counts.
   - With the mode off, the new arguments are one-word buffers that are never touched, and the raw
     path is as before.
3. **`count_partition_adjusted(order, highest, range)`** returns the adjusted partition, for
   1 ≤ order ≤ highest; at order = highest it is the raw path. A nonzero violation count is refused
   with the host path's messages; rule 4 names the gram's key and next, read from the device.
4. **The sweep** (`count_adjusted_sweep`) counts each order ≥ 1 of each shared range already
   adjusted. No runs are built and nothing is carried between orders. Order 1's adjusted partition
   feeds the unigram continuation before it goes to the sink. Order 0 is unchanged. The old sweep is
   kept as `count_adjusted_sweep_host`, and `count_adjusted_sweep_tampered` still drives it.
5. **Timing.** The left passes count under `sort` and the flags and scan under `emit`. The device
   sweep records no `runs` and no `adjust` time.
6. **A test-only plant hook,** `plant_rule4_violation`, makes the next adjusted partition skip its
   left flags. It is `#[cfg(test)]`; outside tests the check is a constant `false`.
7. **`kn-reference`** writes `unit: claude-429` and is otherwise unchanged.
8. **Tests.**
   - Host, `ngram_count_device_adjust_rule_equals_adjust_partition` (`ngram_count_oracle.rs`): the
     device rule modelled on the host (sort by (key, next, L), count distinct nonzero L, refuse a
     document start with N1+ > 0) equals `adjust_partition` for orders 1–4 under N in {1, 3, 4},
     with BOS every 61 and every 7, over the whole range and three cuts of it.
   - CUDA, `ngram_count_cuda_device_sweep_matches_the_oracle`: every order's digest and records
     equal `oracle_adjusted` at N in {1, 3, 10}, budgets u64::MAX, 60,000 and 9,000, with BOS every
     61 and every 7.
   - CUDA, `ngram_count_cuda_device_sweep_equals_the_host_sweep`: the same shared plan and, per
     order, the same digest and records as `count_adjusted_sweep_host`, at N in {3, 10} and the
     same budgets and streams.
   - CUDA, `ngram_count_cuda_planted_rule4_is_refused`: the plant is refused with the rule-4
     message naming the first gram that is not a document start, is spent by that refusal, and is
     refused by the sweep too.
   - `ngram_count_cuda_sweep_counts_no_extension` now asserts `runs` and `adjust` are 0 and own
     count above 0 on the device sweep, and above 0 on the host-runs sweep with equal records. The
     tamper test runs on the host-runs sweep.

## Measurements

G4, wick 4090, job `c429`, code `4373d5ab4`. The run used all of TRAIN (2,091,560,001 tokens)
against DEV-tune's keys, with N = 10, budget 2^24, `--count-only --profile-phases`, as in
claude-428. Packet: [`run1/c429/`](run1/c429/).

- **Wall time.** The process took 421 s (claude-428: 765 s), and the count 408.5 s (752.8 s),
  over 128 shared ranges and 1,408 partitions.
- **Output unchanged.** `count-10.json` with `unit` deleted (`jq -S 'del(.unit)'`) is
  byte-identical to claude-428's (`cmp`, on wick and again on the laptop); the two files differ
  only in the `unit` line. The file's sha256 is
  `a04e402e2f40555cb19a3e142af031691a01f1752938875267bff02bdf3b273e`.
- **Memory.** Peak RSS was 8,207,772 KiB, 4,151,632 KiB (3.96 GiB) below claude-428's
  12,359,404 KiB. The whole-device GPU peak was 20,506 MiB on a shared device (claude-428:
  20,081 MiB); 3,594 MiB were in use before the run (claude-428: 3,978 MiB).

Count phases in seconds, from the raw ms in each `count-10.log`. Own count holds select, sort,
emit, download and records.

| phase | claude-428 | claude-429 | change |
|---|---|---|---|
| sweep wall | 752.8 | 408.5 | −344.3 |
| own count | 292.7 | 359.2 | +66.5 |
| select | 28.8 | 30.8 | +2.0 |
| sort | 110.4 | 169.7 | +59.2 |
| emit | 18.6 | 30.0 | +11.4 |
| download | 41.5 | 43.0 | +1.5 |
| records | 91.4 | 84.0 | −7.3 |
| runs | 270.1 | 0 | −270.1 |
| adjust | 122.7 | 0 | −122.7 |
| push | 33.1 | 33.4 | +0.2 |
| keep | 15.6 | 15.7 | +0.0 |

The timed phases cover 734.3 s of claude-428's sweep wall and 408.2 s of claude-429's. Push's
sub-phases are as in claude-428: checks 11.0 s, census 18.0 s, feeds 4.3 s, send waits 7 ms.

Host work over every gram (download, records, push, keep) is now 176.0 s, 43.1% of the count
(claude-428, with runs and adjust: 574.5 s, 76.3%). The device phases (histogram, select, sort,
emit) are 230.7 s, 56.5%; sort alone is 169.7 s, 41.5%.

## What the numbers say

- **G5 passes on both counts.** The count's wall fell 344.3 s (45.7%), against a bar of at least
  196.4 s. Peak RSS fell 3.96 GiB, against a ceiling of a 1 GiB rise.
- **Why memory fell.** The host no longer holds order k+1's runs while it adjusts order k, nor the
  raw partition beside the adjusted one; each partition comes off the device already adjusted. The
  run shows the peak, not which buffers made it.
- **The GPU cost is small and was measured on a shared device.** Its whole-device peak rose
  425 MiB, with 384 MiB less in use before the run, so this run's own share rose by about 809 MiB.
  The new buffers are a flag and a scan per sorted position and a scanned copy of the counts.
- **Records fell 7.3 s with no change to them.** That is inside wick's run-to-run variance (up to
  about 20 s per phase, claude-427 to claude-428), so it is not credited here.
- **Sort is now the largest phase,** 169.7 s, 41.5% of the count. The host phases left are
  records (84.0 s), download (43.0 s), push (33.4 s, of which validation is 29.0 s) and keep
  (15.7 s).

## Checks run at the head

- **G1, laptop** at `4373d5ab4`:
  - host `kn_` and `ngram_count` tests: 39/39 (38 before, plus the model test);
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests`: clean, no new
    warning (the four warnings are old ones in `uor-r4-core`);
  - the instrument's `cargo check --features cuda`: clean; its tests 7/7 and 2/2;
  - rustfmt hunk counts equal to main's: 37 in `ngram_count_device.rs`, 29 in
    `ngram_count_cuda.rs`, 4 in `ngram_count_oracle.rs`, 2 in the instrument;
  - no decimal literal or other D32 pattern in the added lines, and no softmax, sigmoid, tanh,
    exponential or matrix multiply.
- **G2, wick** at `4373d5ab4`, job `c429`: `ngram_count_cuda`, `kn_score_cuda` and `kn_filter`,
  24 passed, 0 failed, 4 ignored (the four that need real token streams, ignored as before),
  in 100.7 s. The tamper test passed on the host-runs sweep.
- **G3, negatives,** each set aside on a fresh copy, run, expected to fail:
  - laptop, the model test: with `left_digit` not requiring t[j-k] ≠ BOS (one line set aside,
    then restored with `cmp` equal and rerun green),
    `ngram_count_device_adjust_rule_equals_adjust_partition` fails as a refusal at its
    `expect("device rule")`. Its first version, without the document-start check, passed (first
    section above);
  - wick, job `c429n`, at `4373d5ab4` on a fresh copy each. Packet: [`run1/c429n/`](run1/c429n/):
    - **no L pass** (`if left {` made `if false && left {`):
      `ngram_count_cuda_device_sweep_matches_the_oracle` fails at BOS every 61, N 3, order 1, one
      partition, on the digest: 941,584 targets against the oracle's 565,753, with different
      count-of-counts;
    - **L not requiring t[j-k] ≠ BOS** (`.cu`, `(j > k && tokens[j - k] != 0u)` made `(j > k)`):
      the same test fails as a refusal, "adjusted counts: an order-3 triple extends an order-2
      gram whose oldest slot is BOS". It fails first on the stream with BOS every 61, N = 3, so it
      never reaches the stream with BOS every 7 that the ORDER names; the change touches every
      stream that has a document start below N;
    - each sed was checked to match its line once and leave none, and the unchanged head is G2's
      green run;
  - the planted rule-4 gram: `ngram_count_cuda_planted_rule4_is_refused` passed in G2, with the
    host path's message naming the gram's key and next.
- **Review:** a fresh-context review of `c601ea04a..4373d5ab4` gave APPROVE on the kernels and the
  device code, with three LOWs. The reviewer stopped at its context budget before the tests and the
  conventions grep; those were checked here (above, and the tests are as listed under What
  changed). The LOWs:
  - the document-start check cannot fire while L is right (first section above);
  - the rule-4 message names the least gram index, which equals the first gram in (key, next)
    order because the assembler refuses triples out of that order, and the planted test asserts
    that gram's key and next;
  - a plant spent on a partition of document starts only would refuse nothing; the planted test
    fails in that case, since it asserts both refusals.
- **G4, wick** at `4373d5ab4`, job `c429`: the N = 10 run on all of TRAIN finished (rc 0), and
  its `count-10.json` without `unit` equals claude-428's (`cmp`, on wick and on the laptop). PASS.
- **G5:** PASS on both counts (Measurements, above).

## Skills used

None beyond the loop's own steps. The unit changes how counts are computed, not a model's state,
its operators or the artifact; its check is the byte-identical `count-10.json`.

## Still open

- **Sort, 169.7 s,** is now the largest phase. Two ways to cut it, each its own unit: a cheaper
  left pass (one 13-bit digit read once per position, or fewer passes by packing L with `next`),
  and measuring the passes on an idle GPU first, since this run's GPU was shared.
- **Records (84.0 s) and download (43.0 s)** stay on the host while the digest stays sequential
  (D3's choice, kept here). A parallel digest on the device would remove them and change the replay
  identity; that is a decision for a later ORDER.
- **Validation** (29.0 s of push) and the three small claude-428 follow-ups (an `Err` when a hash
  worker has stopped, the unused public `context_records`, the queue-full case not measured) are
  unchanged.
- **The host-runs sweep** is now only the tests' reference and the tamper tests' path. No
  production caller uses it.

## Plain summary

The n-gram counter used to work out, on the CPU, how many different words come before each
phrase. That took 393 of the 753 seconds it needs to count the 2.1 billion training tokens up to
ten words long. That work now happens on the graphics card, inside the sort the card already
does: one extra key in the sort puts each phrase's preceding words side by side, and the card
counts how many are different. The counts are byte for byte the same as before. The whole count
now takes 408 seconds instead of 753, and the program uses about 4 GB less memory. The sort itself
is now the slowest step.
