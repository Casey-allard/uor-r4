# Prep sheet: the count ceiling from the KN start, with a bound on the fit's distance (written-state-count-model, claude-438, 19 Nov 2026)

Lab: claude. Item: [written-state-count-model](../../roadmap/items/written-state-count-model.md) (#4063). Order: [claude-438](../../loop/tasks/written-state-count-model/claude-438/ORDER.md). Code: branch `claude/kn-start-438` at `d366c829f` (or a later commit of this branch that changes only documents, or repairs a step-1, step-2 or B1 failure under the Stop rule; after any repair every step re-runs from step 1, each into a new folder, and no number from the earlier build is reported). Release: Casey, 19 Nov, in Claude's chat, to "May I rent one 5090 (about $2) for the 100% KN-start rerun?": "Yes, rent it". The pod stays inside the caps (§3.6: at most 4 pods and $8/h).

## The question

This is P1 gate (c) again: how far the best bucketed linear mix of count experts can go below R\* on DEV-tune, fit on the targets it scores. The experts, buckets and modes are claude-433's. claude-433 could not read the gate: its 24-pass EM from the uniform start ended above KN-only at 100%.

This run changes two things:
- **The KN start.** Each bucket starts with 2^32 − (L − 1) on KN and 1 on each other present expert, so the fit starts at KN-only up to integer floors and EM only moves down from there. The passes rise to 96.
- **A bound on the fit's distance** (`gap_bound_q12`, see the ORDER). The bound G is an upper bound on how far the fitted weights' scored bits sit above the best real-valued bucketed weights. So, over DEV-tune:
  - **U** = U_count's scored micro-BPB. No bucketed fit needs more than this, because this fit exists.
  - **L** = `lower_bpb_micro` = (U_count's bits − G) in micro-BPB, floored. No bucketed fit scores below this.
  - The converged ceiling lies in [L, U].

The registered reading (DECISION §4):
- at or below 0.85 × R\*, counts carry most of bar Q, and P1's deployable model is the critical path;
- above 0.90 × R\*, P2 and P5 (geometry) are;
- in between, both stay open.

It is a branch, not a kill. With R\* = 1,432,318 micro-BPB, 0.85 × R\* = 1,217,470 and 0.90 × R\* = 1,289,086, each floored.

## Steps

Every step runs in one job on one rented RTX 5090 (Vast, through `uor-pod up --count 1`), built from a source tarball of the head above with `UOR_NVCC_ARCH=compute_120`. The streams are claude-420's split v1 (`train.tokens.u16`, `dev-tune.tokens.u16`, `id-bytes.u8`, `dev-tune.novel.u8`). They are copied from wick to the pod through the laptop, and their sha256 values are checked on the pod against claude-433's `streams.sha256` before any step. N = 10 and budget 2^24 throughout. Every run writes to a new folder under `/workspace/out/c438/`; no folder is reused (L-002). A completion watcher is armed before the job starts.

1. **Build and tests at the head:**
   - `cargo test --release --features cuda --lib -p uor-r4-discrete -- kn_lookup_cuda kn_score_cuda ngram_count_cuda kn_filter kn_ceiling`, all passing.
2. **The 1‰ slice with the host check:**
   - `kn-reference train dev-tune id-bytes dev-tune.novel 10 16777216 slice/out.json --per-mille 1 --check-host --ceiling`;
   - it passes when the device lookups equal the host's at every target and the host-checked KN masses pass as in claude-422.
3. **1% (`--per-mille 10`) and 10% (`--per-mille 100`)** with `--ceiling --profile-phases`.
   - Each size also runs plain `kn-reference` with the same flags and no `--ceiling`. Its `all` and `novel` tallies must equal the ceiling run's `kn_only`.
   - Ratios at a cut size are read against KN-only on the same cut. A ratio to R\*'s full DEV-tune number at a cut size is labelled "not comparable".
4. **100%** with `--ceiling --profile-phases`. The plain run is not repeated: claude-422's DEV-tune record is the comparison.
   - Peak resident memory is sampled once a second, and the GPU peak is read from `nvidia-smi`, at every size.
5. **Storage.** Each run's JSON, log and memory samples are copied to the laptop under `local/claude/c438/` before `uor-pod down`, and they are committed with the REPORT under `run1/`. No count table, stream or mask is copied off the pod.

## Bar

- **B1 (identity):** at 1% and 10%, `kn_only` equals the plain run's tallies. At 1‰, step 2's host check is B1: the host-checked KN masses and lookups equal the device's at every target. At 100%, `kn_only` equals claude-422's DEV-tune 1,432,318 micro-BPB overall and 1,747,150 novel.
- **B2 (the KN start):** at every size:
  - U_count's `fit_bits_q12[0]` is at most `kn_only`'s `all.bits_q12` plus the target count;
  - the last value of `fit_bits_q12` is at most `kn_only`'s `all.bits_q12` plus the target count (EM's integer floors make a false failure very unlikely, and any failure is read as I);
  - U_count's `scored.all.bits_q12` equals the last `fit_bits_q12`, which confirms that the fit set is the scored set, so L is computed over the targets the bound covers.
- **B3 (the bound's presence):** at every size, `gap_bound_q12` and `lower_bpb_micro` are present, with L ≤ U and the bound at least four times the target count. These hold by construction (the floor terms alone are four units per target, and the subtraction saturates), so B3 checks that the fields were written, not that the bound is right. The bound's soundness rests on the fixture test `the_gap_bound_covers_what_em_still_saves` and its negative (G1, G2 of the ORDER).
- **B4 (EM):** each fit's `fit_bits` is recorded per pass. A rise between passes is recorded with its size but does not stop the run.
- **Gate (c), read only at 100%,** overall, from U and L against 0.85 × R\* and 0.90 × R\*. The novel ratio U_count / R\*'s novel number is reported beside it, with no bound.

**Recorded, with no bar:** U_split and KN + U as ratios to KN-only and to R\*; KN + U's bound; each U_split fit half's bound, which bounds that half's own fit and not U_split's cross-scored score; the gap between U_count and U_split; per-bucket target counts and weights; the passes' fit bits; and wall time, phases and memory at each size.

**Reading map** (fixed before the run):
- **Counts carry:** U ≤ 0.85 × R\*. The roadmap runs the deployable P1 path next (claude-434 Pick, 435, 436).
- **Geometry critical:** L > 0.90 × R\*. P2 moves ahead of 435/436. P1(a), (b) and (d) are still built, but they no longer decide where bar Q's risk lies.
- **Between:** L > 0.85 × R\* and U ≤ 0.90 × R\*. Both stay open, and the order is unchanged.
- **Unsettled:** every outcome the three rows above do not match, that is, the closed band [L, U] holds 0.85 × R\* or 0.90 × R\*, so the fixed passes do not decide the branch. The REPORT records the band and which branches it allows. More passes need a new sheet; the 96 passes are not changed after a result is seen.
- **Large split gap:** U_split above U by more than half the distance from U to R\*. This sits beside the gate's reading and does not replace it.
- **I (instrument):** step 1 or 2 fails, B1, B2 or B3 fails, or the job's stream check fails. The repair is made under this sheet, and no number is reported from a failing build.

## Base artifact

- **Code:** `d366c829f` on `claude/kn-start-438`, which is `main` at `3ff8f4640` plus claude-438's commits. The job writes the tarball's commit to `head.txt`.
- **Streams:** claude-420's split v1, the files above, checked by sha256.
- **Machine:** one Vast RTX 5090 (32 GB). claude-433's 100% run peaked at 16,028 MiB of device memory and 10.8 GB resident on a 5090 host.

## Data split

- TRAIN supplies the counts.
- DEV-tune is the query and, for U_count, the fit set as well, which is the ceiling DECISION §4 defines.
- U_split fits on DEV-tune's even records and scores the odd ones, then the reverse, so no record is in both halves of a fold.
- **DEV-test is not read by any step.** It stays sealed for gate (b).

## Minimum test power

The quantities are exact integer sums over every target, and the bound is a deterministic inequality, not an estimate. There is no sampling, so no interval is needed.
- At 100%, DEV-tune has 10,807,417 targets and 21,928,475 bytes.
- One micro-BPB at 100% is about 21.9 bits in all, or 89,800 Q12 units. B3's floor terms alone (four Q12 units per target) are about 0.49 micro-BPB, so the bound's width is set by how far the fit is from converged, not by its rounding.

## Cost

claude-433's 100% run on a 5090 took 289 s for the raw count, 2 s for the lookups and 137 s for 24 passes of its four fits. At 96 passes, EM is about 550 s, plus one bound pass per fit. Each smaller size and the build add a few minutes. The job, including the stream copy, should take well under an hour. At about $0.40–0.90/h for one 5090, that is under $1, inside Casey's "about $2".

## Stop rule

- A failure in step 1 or 2 stops the job before step 3. It is repaired under this sheet.
- A crash at any size is repeated once, under a new name (`<name>-retry`) beside the crashed run's files, which are kept. A second crash at the same size needs a new sheet.
- A B1, B2 or B3 failure stops the reading. It is an instrument fault, and no ratio from that build is reported.
- Each size is run once. The first JSON written at a size is its record.
- No DEV-tune number feeds back into N, the bucket rule, the pass count, the start or the expert set: all are fixed in the ORDER.
- If the rented pod fails to start or stalls, it is destroyed and one replacement is tried. A second failure stops the unit, and the REPORT says so.

FIT history: round 1 (fresh-context non-author, at code `d366c829f`, sheet `16479bb3c`) was FIT with no blocker. It checked the bound's Jensen step, rounding direction, floor slack and overflow, and that L is computed over the fit set. Its should-fix items are folded in above: the Unsettled row now covers every unmatched outcome, including the band's endpoints; B3 is labelled a presence check; U_split's bounds are labelled per fit half; B1 at 1‰ is step 2's host check; B2 states that a false failure is unlikely and is read as I, and gains the identity of scored bits and last fit bits. The retry wording matches the job script.

TEST FITNESS: FIT - round 1 (fresh-context non-author, at `d366c829f`) found no blocker.
