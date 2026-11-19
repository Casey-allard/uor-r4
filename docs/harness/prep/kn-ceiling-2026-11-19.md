# Prep sheet: the count ceiling U_count, P1 gate (c) (written-state-count-model, claude-433, 19 Nov 2026)

Lab: claude. Item: [written-state-count-model](../../roadmap/items/written-state-count-model.md) (#4063). Order: [claude-433](../../loop/tasks/written-state-count-model/claude-433/ORDER.md). Code: branch `claude/kn-ceiling-433` at `cd67cc6c6` (or a later commit of this branch that changes only documents, or repairs a step-1, step-2 or B1 failure under the Stop rule; after any repair every step re-runs from step 1, each into a new folder, and no ratio from the earlier build is reported). Release cited at dispatch: D86 (§3.10), wick is the owner's GPU. A 100% run on a rented pod stays inside the caps (§3.6: at most 4 pods and $8/h).

## The question

P1 gate (c) asks how far the best bucketed linear mix of count experts can go below R\* on DEV-tune. The mix is fit on the very targets it scores.
- The experts are R\*'s own KN mass, the raw maximum-likelihood mass of every order 0..=10, and uniform.
- The mixer is DECISION §3.4's integer EM, with weights per bucket. A bucket is the longest seen order and the size class of its context's total.

The registered reading (DECISION §4):
- at or below 0.85 × R\*, counts carry most of bar Q, and P1's deployable model is the critical path;
- above 0.90 × R\*, P2 and P5 (geometry) are;
- in between, both stay open.

It is a branch, not a kill.

## Steps

All runs use wick's RTX 4090, in Claude's container, from a source tarball of the head above. The streams are claude-420's `/workspace/out/c420/run1/` (read only): `train.tokens.u16`, `dev-tune.tokens.u16`, `id-bytes.u8`, `dev-tune.novel.u8`. N = 10 and budget 2^24 throughout. Every run writes to a new folder under `/workspace/out/c433/`; no folder is reused (L-002).

1. **Build and CUDA tests at the head:**
   - `cargo test --release --features cuda --lib -- kn_lookup_cuda kn_score_cuda ngram_count_cuda kn_filter`, all passing;
   - then the negative: the kernel reads the wrong table, `kn_lookup_cuda_matches_host_on_counted_tables` must fail, and the kernel is restored and rebuilt.
2. **The 1‰ slice with the host check:**
   - `kn-reference train dev-tune id-bytes dev-tune.novel 10 16777216 slice/out.json --per-mille 1 --check-host --ceiling`;
   - it passes when the device lookups equal the host's at every target and the host-checked KN masses pass as in claude-422.
3. **1% (`--per-mille 10`) and 10% (`--per-mille 100`)** with `--ceiling --profile-phases`.
   - `--per-mille` cuts TRAIN and the query alike, at record boundaries. So each size's ratios are read against KN-only on the same cut, which is the only like-for-like comparison below 100%. A ratio to R\*'s full DEV-tune number at a cut size is recorded and labelled "not comparable": it has other targets and another TRAIN.
   - Each size also runs plain `kn-reference` with the same flags and no `--ceiling`, and its `all` and `novel` tallies must equal the ceiling run's `kn_only`.
   - Peak resident memory is sampled once a second, and the GPU peak is read from `nvidia-smi`.
4. **100%.**
   - It runs on wick if, at dispatch, the free device memory and the container's memory exceed the 10% run's peaks scaled to 100% with a 20% margin. Otherwise it runs on one Vast RTX 5090 through `uor-pod`, inside the caps, with a completion watcher armed and the results copied off before `down`.
   - The plain run is not repeated: claude-422's DEV-tune record is the comparison.
5. **Storage.** Each run's JSON, log and memory samples are copied to the laptop under `local/claude/c433/` and summarised in the REPORT. No count table, stream or mask is committed.

## Bar

- **B1 (identity):** at every size, `kn_only` equals the plain run's tallies; at 100% it equals claude-422's DEV-tune 1,432,318 micro-BPB overall and 1,747,150 novel. Step 2 passes.
- **B2 (EM):** each fitted mode's `fit_bits` is recorded per iteration. A rise between iterations is recorded with its size, but does not stop the run: integer floors can move the likelihood by a few units.
- **Gate (c), read only at 100%:** U_count / R\* on DEV-tune, overall, against 0.85 and 0.90. The novel ratio is reported beside it.

**Recorded, with no bar:** U_split and KN + U, each as a ratio to KN-only and to R\*; the gap between U_count and U_split; per-bucket target counts and weights; and the wall time, phases and memory at each size.

**Reading map** (fixed before the run):
- **Counts carry:** U_count ≤ 0.85 × R\*. The roadmap runs the deployable P1 path next (claude-434 Pick, 435, 436).
- **Geometry critical:** U_count > 0.90 × R\*. P2 moves ahead of 435/436 in the roadmap. P1(a), (b) and (d) are still built, but they no longer decide where bar Q's risk lies.
- **Between:** both stay open, and the order is unchanged.
- **Large split gap:** U_split above U_count by more than half the distance from U_count to R\*. This sits beside the gate's reading and does not replace it. The REPORT adds that much of the ceiling's gain is fit to the evaluation set, and that claude-436's held-out fit will carry the deployable number.
- **I (instrument):** step 1 or 2 fails, or B1 fails. The repair is made under this sheet, and no ratio is reported from a failing build.
- **I (EM not converged):** U_count or KN + U scores more bits than `kn_only`, overall or novel, at any size. KN-only is itself one of the mixer's choices, so this is a fit that did not reach it within its fixed passes from its uniform start: an instrument outcome, never a mechanism one. Gate (c) is not read from that build. More passes, or a different starting point, need a new sheet; the 24 passes and the uniform start are fixed in the ORDER and are not changed after a result is seen.
- **Still falling at pass 24:** at 100%, if the drop in U_count's `fit_bits` from pass 23 to pass 24, times 4, would move U_count / R\* across 0.85 or 0.90, the REPORT calls U_count an upper bound on the ceiling and reads the branch with that caveat. The passes are not changed.

## Base artifact

- **Code:** `cd67cc6c6` on `claude/kn-ceiling-433`, which is `main` at `146db0e31` plus claude-433's commits. The job writes the tarball's commit to `head.txt`.
- **Streams:** claude-420's split v1, the files above.
- **Machine:** wick, NVIDIA GeForce RTX 4090, shared with other jobs; a Vast RTX 5090 for step 4 if wick lacks memory.

## Data split

- TRAIN supplies the counts.
- DEV-tune is the query and, for U_count, the fit set as well, which is the ceiling DECISION §4 defines.
- U_split fits on DEV-tune's even records and scores the odd ones, then the reverse, so no record is in both halves of a fold.
- **DEV-test is not read by any step.** It stays sealed for gate (b).

## Minimum test power

The quantities are exact integer sums over every target. There is no sampling, so no interval is needed for B1 or for gate (c).
- At 100%, DEV-tune has 10,807,417 targets.
- At 1‰, 1% and 10%, the cut query holds about 10.8 k, 108 k and 1.08 M targets.

A mixer that moved a 0.05 × R\* margin would change the total by about 1.6 M bits at 100% (0.05 × 1,432,318 micro-BPB × 21,928,475 bytes). That is far above any integer rounding in the EM, whose per-target error is below 2^-32 of a mass.

## Cost

Wick is the owner's GPU, so there is no rental (D86). The estimate from claude-430/431's timings is:
- about 3 s of counting at 1%, 30 s at 10% and 290 s at 100%;
- about as much again for the raw sweep;
- seconds for the lookups;
- EM on one host thread: about 13 u128 divisions per target per pass, about 3.4 G per mode at 100%, so several minutes per mode and four fits in all. The 100% estimate is taken from the 10% run's measured `em` phase, scaled by ten, before step 4 is dispatched.

A Vast 5090 for step 4, if needed, is under one hour at about $0.40–0.70/h.

## Stop rule

- A failure in step 1 or 2 stops the job before step 3. It is repaired under this sheet.
- A crash at any size is repeated once, into a new folder. A second crash at the same size needs a new sheet.
- A B1 mismatch stops the job. It is an instrument fault, and no ratio from that build is reported.
- Each size is run once. The first JSON written at a size is its record.
- No DEV-tune number feeds back into N, the bucket rule, the iteration count or the expert set: all are fixed in the ORDER.

FIT history: round 1 (fresh-context non-author, at `cd67cc6c6`) was NOT FIT, with two blockers: no reading-map row for an EM fit ending above KN-only, and a code-head clause narrower than the repairs the sheet allows. Both were fixed, with three should-fix items (ratios at cut sizes labelled not comparable, the EM cost restated, the split-gap row placed beside the gate). Round 2 (another fresh-context non-author) was FIT, and its one should-fix item, the "still falling at pass 24" row, is folded in above.

TEST FITNESS: FIT - round 2 (fresh-context non-author, at `cd67cc6c6`) found no blocker.
