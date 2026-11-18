# ORDER — written-state-count-model (claude-423)

Lab: claude. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407).
Unit: claude-423, the counter's phase profile. It follows claude-422, which sealed R\*.

## Why

R\*'s sealed run took 2,045 s. 90% of that was counting TRAIN on the GPU, and scoring took about
2 s. Wall time per order is all that exists. Three things are suspected and none is measured:
- In the earlier sealed DEV-test run, order 10 emitted 1.29 B grams and that query retained 0.27%. The profile below uses DEV-tune.
- `count_order_adjusted_with` counts order k+1 again, over order k's ranges, to get its
  extension runs. So most orders are counted twice.
- The host sink hashes every gram.

Before anything is redesigned (D: profile before long runs, 17 Nov), the time per phase is
measured.

## What it does

1. **Phase timers in `NgramCounter`.** These are integer nanosecond totals in atomics, read with
   `take_phases`:
   - the histogram used to plan partitions;
   - select;
   - radix sort;
   - boundaries, emit and scans;
   - device-to-host copies;
   - host record building.

   The adjusted counter also times its own count and its order-k+1 extension count apart, and
   the runs and adjust steps. With profiling enabled, additional stream fences close the select, sort and emit intervals. Their elapsed wall times include the host driver work and waits.
2. **`kn-reference` prints one phase line per order** to stderr, with the sink (assembler push
   and filter keep) timed apart. The timing lines go to stderr. G3 compares the existing JSON fields and accounts for the added floor count.
3. **Profile on wick.** As originally written, this procedure called for full DEV-tune at N = 4 and a slice at N = 10, while G3 below called for full DEV-tune at N = 10. The run packet records both orders at full size (`per_mille: 1000`) and 10,807,417 targets scored at N = 10. No DEV-test run is in this packet.

**Retrospective clarification [codex].** The contradiction above existed at dispatch. The [claude-422 FIT](../../../../harness/prep/r-star-sealed-2026-11-17.md) and its [second sheet](../../../../harness/prep/r-star-sealed-2-2026-11-17.md) name the earlier sealed-reference unit. No contemporaneous non-author FIT covering the additional full `c423-profile` GPU scoring run was located in this unit, the linked sheet or the PR record. Charter §4.5 provenance for that pass remains unverified. The archived results describe the historical diagnostic and leave P1 and the sealed reference bars unchanged. After review, the counter and instrument require explicit phase-timing opt-in. The original command used the earlier implementation with unconditional timers.

## Gates

- G1, laptop: stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
  instrument build are clean, and the host `kn_` and `ngram_count` tests pass.
- G2, wick: the CUDA tests `ngram_count_cuda kn_score_cuda kn_filter` pass at the head.
- G3, wick: DEV-tune at N = 10, full, gives a JSON `cmp`-identical to claude-422's
  `run1/sweep-10.json` but for `masses_at_floor`. So the timers change no output. Its phase
  lines are the profile.
- G4: the REPORT names the largest phases and the saving each candidate fix could reach,
  bounded by the measured times. No fix is built in this unit.

## Out of scope

No change to what is counted. The pre-sort filter, the shared k / k+1 counting and an index of
TRAIN are later units, chosen from this profile.
