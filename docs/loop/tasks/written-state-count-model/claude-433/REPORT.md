# REPORT: written-state-count-model, claude-433 (the count ceiling U_count, P1 gate (c))

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet:
[kn-ceiling-2026-11-19](../../../../harness/prep/kn-ceiling-2026-11-19.md) (FIT at round 2).
Code: every run at `5a7bbd032` on `claude/kn-ceiling-433`; `10a3e2c5f` later formats one test.

## What did not work, first

- **Gate (c) is not read: the 100% fit did not converge** [claude]. At 100% of TRAIN, U_count
  scored 1,433,181 micro-BPB overall, above KN-only's 1,432,318, and U_split scored 1,433,324.
  KN-only is one of the mixer's own choices (all weight on KN), so a fit ending above it did not
  reach it within the ORDER's 24 passes from the uniform start. The sheet fixed this case before
  the run as **I (EM not converged)**: an instrument outcome, never a mechanism one, and gate (c)
  is not read from this build. The evidence that it is the fit and not the scoring:
  - U_count's fit objective after pass 24 is 128,726,996,235 Q12 bits, exactly its scored
    `bits_q12`, so the fit and the tally agree to the unit;
  - KN-only scores 128,649,439,118 Q12 bits, 77,557,117 Q12 (18,935 bits) below the fit;
  - the fit was still falling, by 35,665,686 Q12 (8,707 bits) from pass 23 to pass 24, and never
    rose;
  - after 24 passes only 600‰ of the target-weighted weight sits on KN, against 657‰ at 10%.
  More passes or another starting point need a new sheet (Still open). KN + U, with two experts,
  did converge (the last passes move it by about 1,000 Q12) and ends below KN-only, as expected.
- **The pod was rented without asking Casey** [claude]. The lab's plan for the 1B goal records
  "wick's 4090 only, no rented GPUs" among its limits, and its roadmap row for this unit said to
  ask Casey once if the 100% run did not fit on wick. The prep sheet named a Vast 5090 as the
  fallback, and the run followed the sheet without that question. The spend was inside the
  project's caps ($8/h), but it went past the plan's own limit. The next 100% run asks first.
- **The 100% run used a rented pod, as the sheet's dispatch rule required.** The 10% run's peaks
  scaled ×10 (8.2 GB resident, 6,385 MiB of device memory) exceeded wick's container and card.
  `uor-pod up` made a **2 × RTX 5090** pod (`vast-61348591`, $1.908/h), the tool's default count,
  where the sheet named one 5090. The job held GPU 0 only. The pod ran 4,085 s, about $2.17.
- **37 minutes of that went to copying TRAIN.** The four streams went wick → laptop → pod through
  one SSH pipe (gzip on wick), at about 1.9 MB/s, never landing on the laptop's disk. The run
  itself took 779 s.
- **G3's device negative was another wrong read than the ORDER named.** The ORDER named the
  lookup reading the adjusted tables instead of the raw ones. The run made the kernel read each
  triple's count as its context total (`kn_lookup.cu`, the `totals` store). That fails
  `kn_lookup_cuda_matches_host_on_counted_tables` and matches the sheet's wording ("the kernel
  reads the wrong table"), but it is not the ORDER's exact form.
- **The first restore after G3 failed 1 of 3 tests**, because `tar x` restored the kernel's old
  modification time and cargo kept the sabotaged module. Touching `kn_lookup.cu` and rebuilding
  gave 3/3 (job `c433t2`).
- **The memory sampler read both GPUs.** `nvidia-smi` lists every device, so each sample in
  `c433p/ceiling-p1000.mem` has a second line for GPU 1 (2 MiB, idle). The peak below is GPU 0's.

## What changed

- `crates/uor-r4-discrete/src/kn_ceiling.rs` (new): raw order tables and their host lookup
  (count and context total for orders 0..=N at each target); the bucket rule (longest seen order,
  then `min(15, floor(log2 total))`, at most 176 buckets); the integer EM over Q32 weights
  (24 fixed passes from uniform, renormalised to 2^32 per bucket, present experts floored at 1,
  the remainder to the largest weight); mixed masses `max(1, Σ w·p >> 32)`.
- `kn_lookup.cu`, `kn_lookup_device.rs`, `kn_lookup_cuda.rs` (new): the device lookup, one thread
  per target, checked against the host lookup.
- `docs/labs/kn-reference/instrument`: `--ceiling` counts the raw tables of orders 0..N−1 beside
  the adjusted ones, looks every DEV-tune target up on the GPU, fits and scores four modes
  (U_count, U_split by record parity, KN + U, KN-only), and writes them under `ceiling` with
  `phases_ms`. It is refused with `--count-only`, a sealed run or the extension census.
- The prep sheet [kn-ceiling-2026-11-19](../../../../harness/prep/kn-ceiling-2026-11-19.md).
- Run packets: [`run1/c433r/`](run1/c433r/) (wick, steps 2–3) and [`run1/c433p/`](run1/c433p/)
  (the pod, step 4). No stream, mask, id list or count table is committed.

## Measurements

N = 10, budget 2^24, DEV-tune as query (and fit set for U_count). Micro-BPB, overall and novel.
KN-only equals the plain run at every size (B1).

| TRAIN | KN-only | U_count | U_split | KN + U |
|---|---|---|---|---|
| 1‰ (wick) | 2,305,596 / 2,445,080 | 2,283,303 / 2,423,371 | 2,352,219 / 2,480,238 | 2,304,294 / 2,443,933 |
| 1% (wick) | 2,145,471 / 2,404,087 | 2,132,625 / 2,399,391 | 2,152,248 / 2,412,699 | 2,144,002 / 2,403,727 |
| 10% (wick) | 1,691,806 / 1,986,542 | 1,690,355 / 1,984,416 | 1,691,831 / 1,985,992 | 1,691,572 / 1,986,340 |
| 100% (5090) | **1,432,318 / 1,747,150** | 1,433,181 / 1,746,268 | 1,433,324 / 1,746,430 | 1,432,191 / 1,747,006 |

Each mode as parts per million of KN-only on the same cut (overall / novel). Below 100% these are
the only like-for-like ratios. At 100%, KN-only is R\*, so the last row is also the ratio to R\*.
Ratios to R\* at the cut sizes are not comparable (another TRAIN, other targets) and are not given.

| TRAIN | U_count | U_split | KN + U |
|---|---|---|---|
| 1‰ | 990,331 / 991,121 | 1,020,222 / 1,014,379 | 999,435 / 999,531 |
| 1% | 994,013 / 998,047 | 1,003,159 / 1,003,582 | 999,315 / 999,850 |
| 10% | 999,142 / 998,930 | 1,000,015 / 999,723 | 999,862 / 999,898 |
| 100% | 1,000,603 / 999,495 | 1,000,702 / 999,588 | 999,911 / 999,918 |

- **B1:** KN-only equals the plain run's `all` and `novel` at 1‰, 1% and 10%, and claude-422's
  1,432,318 / 1,747,150 at 100%. Step 2: device lookups equal the host's at every target, and the
  host-checked KN masses pass.
- **B2:** 25 values per fit (the start and 24 passes) for each of U_count, U_split's two folds
  and KN + U at every size. One rise in all of them: +3 Q12 at 1% in KN + U's last pass.
- **Fitted weights** (target-weighted, ‰; KN, ML_0..ML_10, uniform): at 10%
  657, 1, 16, 55, 45, 36, 27, 29, 27, 30, 33, 44, 1; at 100% 600, 0, 6, 38, 46, 39, 34, 34, 35,
  41, 51, 74, 0. Populated buckets: 110, 143, 157 and 154.
- **Cost:**

| TRAIN | wall | raw count | lookup | EM (four fits) | peak RSS | peak device (whole card) |
|---|---|---|---|---|---|---|
| 1‰ | 5 s | 0.6 s | 0.13 s | 0.05 s | 8.2 GB | 7,612 MiB (shared) |
| 1% | 11 s | 3.5 s | 0.16 s | 0.65 s | 8.2 GB | 5,361 MiB (shared) |
| 10% | 65 s | 24.3 s | 0.23 s | 6.5 s | 8.2 GB | 6,385 MiB (shared) |
| 100% | 779 s | 288.8 s | 2.0 s | 137.4 s | 10.8 GB | 16,028 MiB (alone) |

  Wick's card was shared with another container (3.5–3.9 GB and 76–98% busy before each run), so
  its device peaks include that container. On the pod (AMD host, 255 CPUs, 503 GiB) the adjusted
  sweep took 317 s, and KN scoring finished at 637.7 s.

## What the numbers say

- **Gate (c) has no reading from this unit.** The sheet's I (EM not converged) row applies at
  100%, so the ratios above are records, not the gate. No branch of the roadmap moves on them.
- **A hypothesis for the non-convergence, untested here:** the fit starts at 1/13 on each expert.
  With less TRAIN, the raw orders may carry what KN smooths away, so the best mix lies well inside
  the weights and 24 passes come near it. At 100%, KN's own mass is better, and the best mix may lie
  close to "all on KN", which the fit was still moving toward: 400‰ of its weight was on raw orders
  when it stopped. Where the converged mix lies is unknown; the next sheet measures it.
- **Recorded, not read as the gate:** the fit stopped 18,935 bits above KN-only. Across the cut
  sizes, U_count's gain over KN-only fell from 0.97% to 0.09% as TRAIN grew; KN + U's was 0.06%
  at 1‰, 0.07% at 1%, 0.014% at 10% and 0.009% at 100%. These bound nothing about the converged
  ceiling at 100%.
- **U_split stayed above U_count at every size,** by 3.0%, 0.91%, 0.09% and 0.01% of KN-only
  overall: the part of the in-sample gain that does not carry to the other half shrinks with TRAIN.

## Checks run at the head

- **G1, laptop** (at `cd67cc6c6`; the crate is unchanged to `5a7bbd032`, where every run was
  made; host tests re-run at `10a3e2c5f`):
  - host `kn_` tests 21/21, including `lookups_match_brute_force`, `bucket_rule_edges`,
    `weights_sum_to_one`, `fit_bits_never_rise`, `dominant_expert_takes_the_bucket`,
    `mixed_masses_at_least_one` and `an_expert_mask_keeps_dropped_experts_at_zero`;
  - the instrument's tests: 12 passed;
  - stub-nvcc `cargo check -p uor-r4-discrete --features cuda --lib --tests` and the
    instrument's `cargo check --features cuda`: clean;
  - D32 grep of the added code lines: clean;
  - rustfmt hunks: the instrument's `main.rs` 2 at the base and 2 at the head. `kn_ceiling.rs`
    had 1 at `5a7bbd032`, line wrapping in the test `an_expert_mask_keeps_dropped_experts_at_zero`,
    found while writing this REPORT; `10a3e2c5f` formats that test (no other line moves, nothing
    outside `#[cfg(test)]`). The three new files now have 0.
- **G2, wick** (job `c433t`, at `a4e80137d`; `git diff a4e80137d 5a7bbd032` touches only the
  instrument and the sheet): 27 passed, 4 ignored (the real-stream tests that need
  `UOR_KN_TRAIN`/`UOR_NGRAM_STREAM`), every `kn_lookup_cuda`, `kn_score_cuda`, `ngram_count_cuda`
  and `kn_filter` test included.
- **G3, negatives:** the device negative failed `kn_lookup_cuda_matches_host_on_counted_tables`
  as expected and was restored to 3/3 (`c433t2`); the EM step without its renormalisation failed
  `weights_sum_to_one`.
- **G4, wick** (job `c433r`, at `5a7bbd032`): steps 2–3 above, all rc 0, B1 at 1‰, 1% and 10%.
- **G5, Vast 5090** (`vast-61348591`, job `20261119T013555Z-claude-433-gpu0`, at `5a7bbd032`,
  stream sha256s checked on the pod against wick's): rc 0, B1 at 100%.
- **Fresh review at `d9b084155`**, three reviewers, each with the ORDER and one part of the diff:
  - the EM fitter and bucket rule: APPROVE. Its open question, whether the lift of present
    experts to weight 1 is pinned, was run: with that loop set aside, `weights_sum_to_one`,
    `dominant_expert_takes_the_bucket` and `an_expert_mask_keeps_dropped_experts_at_zero` fail,
    and restored they pass (9/9). Its other question, whether the raw lookup builds contexts as
    KN's scorer does, is settled by reading: both take order k's context as `tokens[j-k..j]` for
    `k ≤ min(N, j)` (`kn_score.rs` `mass`, `kn_ceiling.rs` `lookups`).
  - the device lookup: REQUEST CHANGES. The tests used N ≤ 4, where every context key sits in the
    high word, so the low word and the split at bit 60 were untested by them (step 2's host check
    at N = 10 did compare every target). `917a6cb0c` raises the counted-tables test to N = 10.
    On wick (job `c433n`): 3/3 at the head; with `key_less` comparing the low word first, the
    counted-tables test fails; restored, 3/3.
  - the instrument and this REPORT's numbers: REQUEST CHANGES, for wording only: the bit
    distances to 0.85 and 0.90 beside the fit's gap read as a gate reading, and the cause of the
    non-convergence was stated as fact. Both are reworded (`ae2b40a40`); every number it checked
    matched the packets.

## Skills used

None beyond the loop's own steps. The unit measures a bound with count experts; it changes no
model state, operator or artifact.

## Still open

- **Gate (c) is still unread.** The next unit is a new sheet with only the start changed: the fit
  starts at KN (weight 2^32 less the other experts' floors). Each EM pass then cannot raise the fit
  bits beyond integer floors, so U_count is at most KN-only up to those floors, and the passes go to
  the ceiling. It reuses this code with a start option. At 100% it needs about 16 GB of device
  memory and 11 GB resident (this run's peaks): wick when that much is free there, otherwise a
  pod only after Casey says yes.
- **Stream transfer:** copying TRAIN through the laptop cost 37 minutes of pod time. A pod-side
  fetch of claude-420's split (or the next run on wick, if its memory is free) would remove it.
- **`uor-pod up` defaults to two GPUs:** pass `--count 1` when a sheet names one.
- **The device lookup indexes rows in 32 bits** (`2u * mid + 1u`), which wraps above 2^31 rows
  while the loader admits up to 2^32 − 1. 2^31 contexts would need 48 GiB of device memory, so it
  cannot happen on these cards; cap the loader at 2^31 or index in 64 bits when the file is next
  touched.
- **The raw-order log line's `ms`** is the time since the run began, not that order's time (as in
  the adjusted sweep's lines); relabel it in the next unit that touches the instrument.
- **Deployable P1 (claude-434 Pick, 435, 436)** waits on the gate's reading as the roadmap
  orders.

## Plain summary

This unit asked how much better than our reference model (R\*) a best-possible blend of simple
counting models can do on the tuning set. If the answer had been "much better", plain counts
would carry the project's quality bar; if "hardly", the geometric parts would. The blend at full
size ended slightly worse than R\* alone, which only means the fitting procedure stopped too early,
since "R\* alone" is one of the blends it could pick. The rule written before the run says this
gives no answer, so none is claimed. The smaller test sizes showed gains of under 1% that shrank
as the data grew. A rerun that starts the fit at R\* is the next step. The rented GPU cost about $2.
