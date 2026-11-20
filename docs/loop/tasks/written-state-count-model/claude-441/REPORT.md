# REPORT: written-state-count-model, claude-441 (split the S5 Pick buckets by pair distance)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet:
[kn-ceiling-pick-distance-split-2026-11-20](../../../../harness/prep/kn-ceiling-pick-distance-split-2026-11-20.md)
(FIT at round 1, `aa36f9414756796a53db80d31aee693ab8f9c902`). Every number below is from
`run1/pick-p1000-retry` on wick's RTX 4090, built from a source tarball of that head (`run1/head.txt`).
No rental.

**Reading: `split_gain_geometric`, and tiny.** The split beats claude-439's three-class A1 by 16 to 32
micro-BPB on all DEV-tune targets, about 2 parts in 100,000 of A1's bits. A permuted code map gains
nearly as much from the same split (10 to 27). The D11 codes' edge over the permuted map is unchanged:
[−241, −28] after the split against [−232, −24] before it.

## What did not work, first

- **The split gain is tiny, and most of it is not geometry** [claude]. A1s − A1 is [−32, −16]
  micro-BPB (novel [−44, −23]). C3s − C3, the same split over a permuted token → code map, is
  [−27, −10] (novel [−38, −15]). The sheet's bar is met: A1s beats A1 and beats C3s. But the gain from
  splitting is about the same with scrambled codes, so it comes mainly from separating classes of
  reads, not from the codes being near.
- **Distance 1 never occurs** [claude]. Class 2 (pair distance 1) holds no DEV-tune target and no R
  target, in A1s and in C3s. The split is in effect three fired classes: distance 0, distance 2, and 3 or
  more. A permutation keeps the code set, so both maps see the same distance set. That the D11 code set
  has no two pairs at distance 1 is my reading of these counts; I did not check the code set itself.
- **Distance 2 is small** [claude]. Of 10,802,077 fired DEV-tune targets, 4,111,759 are at distance 0,
  36,362 at distance 2 and 6,653,956 at 3 or more. The whole gain sits on the 36,362: their copy hits
  about 17% of the time, against about 8% at 3 or more, and EM gives Pick 37‰ there against 10‰. On R
  the distance-2 class has 53,736 fit targets.
- **The first 100% run crashed** with CUDA out of memory after 480 s, at order 7 of the count build,
  before any Pick fit. Another lab's GPU job grew from 2,186 MiB at the start to 15,392 MiB, and the
  card peaked at 24,081 MiB. Its log and memory samples are kept in `run1/` (`pick-p1000.*`); it wrote
  no JSON. The Stop rule's one retry ran from the same binary, with the start bar raised from 16,384 to
  21,504 MiB free (the sheet's run note). No run parameter changed.
- **The numbers belong to the build at `aa36f9414`, not to this PR's merged head** [claude]. After the
  run, main's #4126 and #4128 (codex) were merged into this branch to clear a conflict. They change code
  the run went through (`kn_score.cu`, `ngram_count.cu`, `kn_pick.rs`, and the move of the Pick
  mixture code into `kn_ceiling/runtime/mixture.rs`). This unit's own diff is unchanged apart from where
  `ceiling_pick_row_classes` lives. The merged head is tested on the laptop only (below); the CUDA tests
  have not run at it. The sheet's rule for later commits ("only documents, or repair a step-1, B1, B2,
  B3 or replication failure") does not name a merge from main. This merge (`6faf5d535`) is outside that
  list, and it is handled by scoping every number here to the binary built at `aa36f9414`.
- **The sheet's power estimate was far off** [claude]. It expected an A1s − A1 band about as wide as
  A1 − C3 (208 micro-BPB). It was 16 wide: A1s and A1 differ only in the 36,362 distance-2 targets'
  buckets, so their paired differences are near zero on almost every record.
- **The container cap was raised before the memory was free** [claude]. The sheet raises the cap to
  26 GiB when wick's MemAvailable is at least 28 GB. The cap can only be changed from the host, so I
  raised it at launch, while MemAvailable was about 27 GB, and the job waited for 28 GB before the 100%
  run. That wait passed at once on the retry (33,183,644 kB). The cap went back to 16 GiB after.

## What changed

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: `PICK_SPLIT_CLASSES` (5) and `pick_split_class`
  (class 1 + min(distance, 3)); the ceiling table carries its class count and builds tables of 2 to 5
  classes. At 3 classes the rows and buckets are byte-identical to claude-439's.
- `crates/uor-r4-discrete/src/kn_ceiling/runtime/mixture.rs`: `ceiling_pick_row` is a wrapper over
  `ceiling_pick_row_classes`, so the serving-only count runtime keeps its three-class behaviour.
- `docs/labs/kn-reference/instrument/src/pick_fit.rs`: the split arms A1s and C3s, their four bands, the
  `split_reading` map, B3 over the split arms, and per-class targets and hits in `pick_stats`.
- `docs/labs/kn-reference/instrument/src/pick_run.rs`: split targets from each row's distance, with a
  refusal when a row's split class, cut to 2, differs from its three-class class. It runs before any fit.
- The prep sheet and its run note.
- The run packet [`run1/`](run1/): the retry's JSON, log and memory samples, the crashed run's log and
  memory samples, `head.txt`, and step 1's logs. No stream, mask, id list or count table is committed.

## Measurements

N = 10, budget 2^24, 100% of C. Weights fitted on R (4,060 TRAIN records, 32,538,487 bytes), scored
frozen on DEV-tune (2,670 records, 10,807,417 targets).

**Scores** (micro-BPB):

| Arm | DEV-tune all | DEV-tune novel | Fit on R | Bound (as micro-BPB of R) |
|---|---|---|---|---|
| A0 pure count | 1,432,620 | 1,747,432 | 1,421,840 | 135,088 |
| A1 Pick, D11 codes, 3 classes | 1,360,806 | 1,651,188 | 1,348,645 | 119,196 |
| **A1s Pick, D11 codes, split** | **1,360,781** | **1,651,154** | **1,348,603** | **118,665** |
| C1 read disabled | 1,432,607 | 1,747,418 | 1,421,780 | 143,715 |
| C2 exact pair | 1,365,580 | 1,657,701 | 1,353,588 | 129,139 |
| C3 permuted codes, 3 classes | 1,360,932 | 1,651,364 | 1,348,818 | 122,873 |
| **C3s permuted codes, split** | **1,360,913** | **1,651,338** | **1,348,786** | **123,120** |

**Bands** (paired bootstrap over 2,670 records, 1,000 resamples, seed 4390; micro-BPB):

| Band | All | Beats | Novel |
|---|---|---|---|
| A1s − A1 | [−32, −16] | yes | [−44, −23] |
| A1s − C3s | [−241, −28] | yes | [−329, −46] |
| C3s − C3 | [−27, −10] | yes | [−38, −15] |
| A1s − C2 | [−5,259, −4,343] | yes | [−7,177, −5,922] |
| A1 − C3 (claude-439's) | [−232, −24] | yes | [−319, −37] |

**Pick by class** (DEV-tune targets and hits; fit-target-weighted Pick weight over R's buckets):

| Class | Pair distance | A1s targets | A1s hits | A1s Pick weight | C3s targets | C3s Pick weight |
|---|---|---|---|---|---|---|
| 0 | not fired | 5,340 | 0 | absent | 5,340 | absent |
| 1 | 0 | 4,111,759 | 2,253,627 | 190‰ | 4,108,843 | 190‰ |
| 2 | 1 | 0 | 0 | no target | 0 | no target |
| 3 | 2 | 36,362 | 6,160 | 37‰ | 24,779 | 39‰ |
| 4 | 3 or more | 6,653,956 | 505,294 | 10‰ | 6,668,455 | 9‰ |

A1's three-class row, from the same run: class 2 (distance > 0) holds 6,690,318 targets with 511,454
hits and 10‰ of weight, the sum of A1s's classes 3 and 4.

**Replication** against claude-439's `run2/pick-p1000.json`: each of A0, A1, C1, C2 and C3's `b3` and
`scored`, the five bands, `b2` and `reading` (`geometric_gain`) are equal. `replication: PASS`.

**Speed and memory** (wick's 4090, shared): wall 4,798 s; A1 main read 72,361 rows/s, A1 plus read
71,029, C3 read 70,961; the seven main fits 962 s, the P+ fits 541 s; `--check-host`'s host lookups
241 s (main) and 334 s (plus). Peak RSS 20,042,956 KiB (self-reported); peak device 24,071 MiB, whole
card, sampled.

## What the numbers say

- **The split is a real but tiny gain, and it costs nothing at run time.** The Pick read is unchanged;
  only the bucket rule grows, from 528 to 880 buckets (704 of them can hold a target, since the 176 of
  the distance-1 class are empty). By the sheet's map, the REPORT recommends the split for S5; a decision card under §5.1
  makes the change.
- **The codes' edge does not move under the split.** A1s − C3s is the same band as A1 − C3 within a few
  micro-BPB at both ends. The split refines what Pick trusts by distance, and a permuted map's distances
  are refined almost as usefully.
- **Near reads are worth more than far ones, but there are very few near reads.** A copy at distance 2
  hits about twice as often as one at 3 or more, and EM trusts it about four times as much. Only 36,362
  of 6,690,318 fired near reads are at distance 2. A split of the 3-or-more class would need the
  distance itself, which the read does not report past the cap at 3 here.
- **B2 and the replication are unchanged**, as they must be: the split leaves the five original arms
  bit-identical.

## Checks run at the head

- **G1, laptop, at the code head `57bfb7035`** (the commit to the FIT head `aa36f9414` changes only the sheet): `kn_ceiling` tests 25 passed (46 under the `kn_`
  filter), among them `the_split_class_rule` and `the_split_bucket_rule`; the instrument's tests 44
  passed and 1 ignored, among them `split_arms_leave_the_five_arms_unchanged`,
  `pick_stats_count_each_class`, `the_split_reading_map` and `split_targets_follow_the_distance`;
  stub-nvcc `cargo check --features cuda --bins` clean apart from the unused-import warning already at
  `pick_device.rs:133`.
- **G2, negatives** (set aside, run red, restored, run green): distance 3 mapped to class 3 failed
  `the_split_class_rule`; a three-class build failed `the_split_bucket_rule`; with the split refusal switched off (at the
  review, 20 Nov), `split_targets_follow_the_distance` failed.
- **The job's step 3, dry-run on the laptop** against claude-439's committed record: the record against
  itself gave `replication: PASS`; one split arm's B3 flag set false stopped with exit 2 as a B3 failure;
  one changed `scored` gave `replication: FAIL at main.c2.scored`, exit 6.
- **G3, wick's RTX 4090, at `aa36f9414`:**
  - step 1: 65 discrete CUDA tests passed (14 ignored); the instrument suites passed (45 with 2 ignored,
    5, 4 with 2 ignored, 2, 1, 1); the trunk oracle `host_pick_equals_the_trunk_oracle` passed;
  - step 2 (the retry, `--check-host`): B1 true (A1 main 26,755,001 rows, A1 plus 53,503,272, C2 and C3
    26,755,001 each, 10,807,417 oracle targets); B2 true; every arm's B3 true with no rises;
    `band_records` 2,670;
  - step 3: `replication: PASS`;
  - the inputs and claude-439's split checked by sha256 before each job.
- **The merged head `6faf5d535`, laptop:** `kn_` tests 48 passed; the count runtime builds with no
  warning and its test passes; the instrument's tests pass; the stub-nvcc check is clean.
- **Prep sheet FIT** at round 1, by a fresh-context non-author reviewer at `57bfb7035`: FIT with three
  minor changes, all made in `aa36f9414`.
- DEV-test was not read by any step.

## Skills used

None beyond the loop's own steps. The unit measures a bucket rule in an integer count mixture; it
changes no model state, operator or artifact.

## Still open

- **A decision card for the split in S5**, as the sheet's map asks. The gain is 16 to 32 micro-BPB and
  the cost is the bucket rule only.
- **The CUDA tests at the merged head** have not run; main's #4126 and #4128 changed `.cu` files after
  this run's build. Their own PRs carry their tests.
- **Whether the D11 code set has minimum pair distance 2**, as the empty distance-1 class suggests. A
  read of the code set would settle it.
- **Wick runs move to bare metal** from the next unit (Casey, 20 Nov). This unit finished in the
  container it started in.
- After merge: delete my wick build folders (`/root/work/c441`, `/root/work/c441-target-instrument`)
  and the laptop's `local/c441-target` and `local/c441-instr-target`.

## Plain summary

The copy helper in the counting model trusts a copy more when the earlier spot matches exactly, and
much less when it only nearly matches. We tried telling it how near the match was: exact, one step
off, two steps off, or further. It never sees "one step off" at all, and "two steps off" is rare. Where
it does happen, that copy is right about twice as often as a further one. Using that makes predictions
better by about 2 parts in 100,000. A scrambled version of the token codes gains almost the same
amount, so the project's geometric codes are not what makes the split work. The split is free to use,
so the report recommends keeping it, but it changes almost nothing.
