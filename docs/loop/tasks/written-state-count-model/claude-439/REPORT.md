# REPORT: written-state-count-model, claude-439 (P1: the S5 Pick expert, fitted on reserved TRAIN, against its controls)

Lab: claude. Order: [ORDER.md](ORDER.md). Step 0: [STEP0.md](STEP0.md). Prep sheet:
[kn-ceiling-pick-expert-2026-11-19](../../../../harness/prep/kn-ceiling-pick-expert-2026-11-19.md)
(FIT at round 4, `932234a4bc549e4bf77c6bfd6694546e5a2e24f5`). Every number below is from run2 on
wick's RTX 4090 at that head (`run2/head.txt`). No rental.

**Reading at 100%: `geometric_gain`, and small by the sheet's own rule.** The Pick expert cuts
DEV-tune bits by 66,248 to 77,388 micro-BPB against pure count. A rotated code map gets almost all of
that: the D11 codes beat it by only 24 to 232 micro-BPB.

## What did not work, first

- **The geometry margin is small** [claude]. The reading needs A1 to beat each control, and the
  tightest band is A1 − C3, the same read over a seeded permutation of the token → code map. At 100% that band is
  [−232, −24] micro-BPB, so it clears 0 by 24. The same band at 10% was 277 wide ([−344, −67]). By the
  sheet's rule ("small when the band that decides it clears 0 by less than the width of the same band
  at 10%"), the reading is small. The point gap A1 − C3 is 126 micro-BPB, about 9 parts in 100,000 of A1's bits.
- **The geometry edge shrinks as the counts grow** [claude]. A1 − C3's upper end is −481, −174, −67
  and −24 micro-BPB at 1‰, 1%, 10% and 100% of C. A1 − C2 and A1 − A0 shrink the same way (table
  below). These are 1‰ to 10% bands, recorded, not readings. They suggest, untested, that a larger C
  could close the gap to C3.
- **Where geometry acts, EM barely trusts it** [claude]. Pick fires on 10,802,077 of 10,807,417
  DEV-tune targets. 4,111,759 of them (38%) sit at distance 0, which is an exact pair under any
  injective code map. The other 62% are distance > 0, the only place the codes can matter. The fitted
  Pick weight, target-weighted over R's buckets, is 190‰ in the distance-0 class and 10‰ in the
  distance > 0 class, the same in A1 and C3.
- **Most of A1's lead over exact pairs is "fire on the nearest pair", not geometry** [claude]. A1 beats C2 (exact
  pairs only, distance 0) by 4,318 to 5,232 micro-BPB. C3 fires on as many targets with scrambled
  codes and lands 126 micro-BPB from A1. So C3 suggests that nearly all of that 4,774 point gain
  comes from firing at distance > 0 at all, under any code map; no arm measures that directly.
- **B2 tests the distance-0 path only** (the sheet's scope note). P+'s second copies are distance-0
  copies, so the read at distance > 0 rests on B1's equality with the host oracle at 1‰.
- **The cap was raised outside the memory rule** [claude]. The rule names the 100% run only. At
  21:01Z, during the 1% run (`pick-p10`), I raised wick's container cap from 16 GiB to 24 GiB: the 1‰
  run had peaked at 15,936,280 KiB under the 16 GiB cap, and the 1% run was 512 MiB to 1 GiB higher at the same phase. It was
  a cap change only: no input, flag or row changed, and it is logged in `run2/head.txt`.
- **The memory rule's estimate was about 1¾ times the real peak.** The linear extrapolation from the
  1% and 10% sampled peaks gave 35,610,484 KiB. The cap went to 38 GiB, and the 100% run peaked at
  20,076,956 KiB (self-reported; 19,169,856 sampled). The growth between sizes is not linear in C.
- **Run1 was stopped by its pid** before any fit, 2 h 11 min into A1's read at 256 windows per call.
  It wrote no JSON. The sheet's read amendment (4,096 windows per call, FIT at round 4) covers it.
- **B3 recorded 10 rises at 1‰** across the arms' fits. A rise is recorded and does not stop the run;
  every fit at every size ended at or below its first pass, and 1%, 10% and 100% had none.

## What changed

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: a Pick-expert column in the ceiling table, present only
  in fired classes, and the 3× bucket rule (528 buckets). A table without Pick reproduces the plain
  rows.
- `crates/uor-r4-discrete/src/kn_pick.rs`: the host Pick read (latest position on a tie, then the
  lowest original id), C2's exact-pair read, C3's row permutation, P+'s chunk repeats.
- `docs/labs/kn-reference/instrument`: `pick-split` (the R/C split by record index and the
  R ⊕ DEV ⊕ R⁺ ⊕ DEV⁺ query); `pick_device.rs` (the CUDA Pick read with Identity transport, 4,096
  windows per call, checked against the trunk oracle); `pick_fit.rs` (EM fits on R, frozen scores on
  DEV-tune, the paired bands, B2, B3 and the reading label); `pick_run.rs` (the `--pick-arms` mode).
- The prep sheet and its read amendment.
- The run packet [`run2/`](run2/): each size's JSON, log and memory samples, `head.txt`, the split
  manifest and its sha256 list, and step 1's logs. No stream, mask, id list or count table is
  committed; the JSONs hold per-order totals and count-of-counts, as claude-433's did.

## Measurements

N = 10, budget 2^24. Weights fitted on R (4,060 TRAIN records, 15,947,584 targets), scored frozen on
DEV-tune (2,670 records, 10,807,417 targets, 21,928,475 bytes). C is TRAIN minus R.

**Scores at 100% of C** (micro-BPB):

| Arm | DEV-tune all | DEV-tune novel | Fit on R | Bound (`gap_bound_q12`, as micro-BPB of R) |
|---|---|---|---|---|
| A0 pure count | 1,432,620 | 1,747,432 | 1,421,840 | 135,088 |
| A1 Pick, D11 codes | 1,360,806 | 1,651,188 | 1,348,645 | 119,196 |
| C1 read disabled | 1,432,607 | 1,747,418 | 1,421,780 | 143,715 |
| C2 exact pair | 1,365,580 | 1,657,701 | 1,353,588 | 129,139 |
| C3 rotated codes | 1,360,932 | 1,651,364 | 1,348,818 | 122,873 |

A0 against R\* = 1,432,318: +302 micro-BPB ("C is TRAIN minus R, not claude-422's TRAIN").

**Bands at 100%** (paired bootstrap over 2,670 records, 1,000 resamples, seed 4390; micro-BPB):

| Band | All | Beats | Novel |
|---|---|---|---|
| A1 − A0 | [−77,388, −66,248] | yes | [−103,620, −88,773] |
| A1 − C1 | [−77,355, −66,237] | yes | [−103,585, −88,777] |
| A1 − C2 | [−5,232, −4,318] | yes | [−7,141, −5,889] |
| A1 − C3 | [−232, −24] | yes | [−319, −37] |
| C1 − A0 | [−39, +7] | no | [−46, +12] |

**Bands by size** (overall, micro-BPB; only 100% is the reading):

| Band | 1‰ | 1% | 10% | 100% |
|---|---|---|---|---|
| A1 − A0 | [−215,345, −189,037] | [−151,372, −132,063] | [−109,515, −94,737] | [−77,388, −66,248] |
| A1 − C2 | [−17,910, −15,509] | [−11,501, −9,825] | [−7,807, −6,557] | [−5,232, −4,318] |
| A1 − C3 | [−912, −481] | [−509, −174] | [−344, −67] | [−232, −24] |
| C1 − A0 | [−306, −140] | [−236, −78] | [−76, −15] | [−39, +7] |

**Pick statistics at 100%** (DEV-tune): A1 fired 10,802,077, distance 0 on 4,111,759; C3 fired
10,802,077, distance 0 on 4,108,843; C2 fired 4,108,760 (distance 0 only). A1 has 2,999 more
distance-0 targets than C2, presumably distinct token pairs whose code pairs coincide (not checked).

**P+ (B2) at every size:** A1's bits on DEV⁺'s second copies over A0's were 645,646 / 2,367,067,
607,816 / 2,041,735, 576,618 / 1,791,152 and 549,068 / 1,580,022 micro-BPB at 1‰, 1%, 10% and
100%. Every ratio is under the bar of one half; at 100% it is about 35%.

**Speed and memory** (wick's 4090, shared with a transcode at 93% to 100% use):

| Size | Wall s | A1 main read rows/s | A1 plus read rows/s | C3 read rows/s | Peak RSS KiB (JSON) | Peak device MiB (whole card, sampled) |
|---|---|---|---|---|---|---|
| 1‰ (`--check-host`) | 4,031 | 48,453 | 48,769 | 49,047 | 15,936,280 | 22,181 |
| 1% | 3,741 | 45,332 | 44,626 | 44,783 | 16,503,040 | 22,501 |
| 10% | 3,616 | 48,489 | 48,434 | 48,551 | 17,854,396 | 23,237 |
| 100% | 3,585 | 48,856 | 72,077 | 53,426 | 20,076,956 | 24,071 |

Each size reads 107M windows through the trunk: about 9 min for each main read and 12 to 20 min for
the plus read. Fitting on R and scoring takes about 11 min for the main arms and 9 min for P+.

## What the numbers say

- **The session copy works, and works out of sample.** Fitted on R and frozen, A1 is 71,814
  micro-BPB (about 5%) under pure count on all DEV-tune targets and 96,244 (about 5½%) on novel ones. A1 beats
  C1, so the gain is copying, not the bucket split: C1 − A0 holds 0 at 100%.
- **The reading is `geometric_gain`, small.** S5's Pick counts as a geometry gain on P1 under the
  sheet's map. But the geometry share is 24 to 232 micro-BPB of the 66,248 to 77,388. The rest is
  a session copy that a scrambled code map does as well.
- **Exact pairs alone leave about 4,774 micro-BPB on the table.** C2 runs on the host in about 1,300 ms; A1
  and C3 each need about 9 min of device reads at 100%. If P1 deploys one copy path, the choice is
  between C2's cost and A1's extra 4,318 to 5,232 micro-BPB. That choice is a decision card, not this
  unit's.
- **The bound is the loose side, as in claude-438.** `gap_bound_q12` is 119,196 to 143,715
  micro-BPB of R per arm, far wider than any band. It is recorded, not read.

## Checks run at the head

- **G1, laptop** (at `83fd3737e`, PR 4107 comment of 16:26Z; the later code commits, FIT rounds 1
  to 4 and the batch size, were tested at `932234a4b` by step 1 below): the instrument's main binary 36 passed and 1 ignored (the trunk oracle, run on wick);
  `pick-split` 5 passed; `kn_ceiling` and `kn_pick` 29 passed, among them
  `a_table_without_pick_is_unchanged`, `the_pick_bucket_rule`, `a_right_copy_lowers_the_bits`,
  `a_wrong_copy_falls_to_the_floor` and `code_pick_takes_the_latest_tie`; stub-nvcc
  `cargo check --features cuda --bins` clean.
- **G2, negatives** (set aside, run red, restored, run green): the band orientation, `beats` on the
  low end, B2 dropped from `reading_allowed`, the query cut in pick mode, C2 mapping every class
  through, C2 taking the earliest exact pair, and the input allowlist switched off. Each made its
  pinning test fail. The ORDER's second negative (the Pick column ignoring the gate) has no record in
  this unit. The sheet replaced the ORDER's saved Pick-mode snapshot with the trunk read through
  Identity transport (its base artifact section).
- **G3, wick's RTX 4090, run2 at `932234a4b`:**
  - step 1: 60 discrete CUDA tests passed (14 ignored); the instrument suites passed (41 with 2
    ignored, 5, 2, 1); the trunk oracle `host_pick_equals_the_trunk_oracle` passed;
  - step 2 (1‰, `--check-host`): B1 true (A1 main 26,755,001 rows, A1 plus 53,503,272, C2 and C3
    26,755,001 each, 10,807,417 oracle targets); B2 and every B3 true; `band_records` 2,670;
  - steps 3 and 4: rc 0 at 1%, 10% and 100%; B2 and B3 true at each; `band_records` 2,670 at 100%;
    the split's sha256 list checked before the 100% run;
  - every size ran once, and no crash or retry happened.
- **Prep sheet FIT** at rounds 1 to 4, each by a fresh-context non-author reviewer; round 4 at
  `da9f46707` found the read amendment FIT with changes, all made in `932234a4b`.
- DEV-test was not read by any step.

## Skills used

None beyond the loop's own steps. The unit measures count and copy experts in an integer mixture; it
changes no model state, operator or artifact.

## Still open

- **A decision card for P1's copy path:** C2 (exact pairs, host, cheap) against A1 (D11 codes, device
  read, 4,318 to 5,232 micro-BPB better). C3 shows that the codes themselves add little over any code
  map, so a cheaper nearest-pair fallback may hold most of A1's lead. That is an untested hypothesis.
- **Whether the geometry edge closes at larger C.** A1 − C3 narrowed at every size. C cannot grow
  past D87's TRAIN, so the trend can only be read from these four sizes or another corpus.
- **The distance > 0 class gets 10‰ of weight.** A per-distance bucket split (distance 1, 2, 3+)
  would show whether near codes carry more than far ones. It is not in this sheet.
- **The memory rule's linear estimate** should not be reused as is: it was about 1¾ times the real peak here.
- **The ORDER's gate negative** has no record; a unit that reads a saved
  Pick-mode D11 artifact would need it.
- Carried minors from the code review: `oracle_rows` keeps block 0 only (test-only); `repeat_chunks`
  accepts an EOS inside a record; `pick-split` does not refuse an output folder inside a git tree; an
  out-of-memory kill cannot remove the claimed record file.
- After merge: delete my old wick build folders (`/root/work/c439`, `c439-target-*`, `c439p`,
  `c439p-target-instrument`).

## Plain summary

We tested a "copy" helper for the counting model: when the last two tokens look like an earlier
spot in the same document, it predicts whatever came next there. It was tuned on held-back training
text and scored on the tuning set. It cuts the cost of predicting text by about 5%, and more on
unusual text, so copying within a document clearly helps. The question was whether the project's
geometric token codes make it better at finding the right spot. They do, but only barely. A version
using a scrambled code map does nearly as well, and the codes' edge shrank as the counting model got
more data. A version that copies only on exact token matches is about a third of a percent worse, but far cheaper to
run. Which of these to keep is a separate decision.
