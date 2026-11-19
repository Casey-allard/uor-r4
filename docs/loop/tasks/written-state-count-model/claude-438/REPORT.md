# REPORT: written-state-count-model, claude-438 (P1 gate (c): the KN start and a bound on the fit's distance)

Lab: claude. Order: [ORDER.md](ORDER.md). Prep sheet:
[kn-ceiling-kn-start-2026-11-19](../../../../harness/prep/kn-ceiling-kn-start-2026-11-19.md) (FIT at round 1).
Code: every run at `6c66a76d5` on `claude/kn-start-438` (code as at `d366c829f`; later commits change
documents only). Release: Casey, 19 Nov, "Yes, rent it", for one RTX 5090. The branch later merged
`main` with codex-14 (PR 4102); the merged head is checked under Checks run at the head.

## What did not work, first

- **This unit duplicated codex-14** [claude]. Codex opened PR 4102 on the same gate at 03:20:10Z, and
  I opened PR 4103 at 03:27:54Z without reading item #4063's open PRs. Codex-14 ran on wick at no
  rental cost, merged at 04:03:48Z while this job ran, and its certificate settles gate (c) as
  **Geometry critical**. It encloses the best all-target score of the same family (order 10,
  176 buckets, 13 experts, Q32 weights, the same mass floors) in [0.98939548, 0.99983067] × KN. The
  work that this unit adds beyond codex-14 is in What the numbers say; most of the rest repeats it.
- **By its own bound, gate (c) is Unsettled** [claude]. At 100% of TRAIN this unit's band is
  [L, U] = [1,284,092, 1,431,925] micro-BPB, and it holds 0.90 × R\* = 1,289,086: L misses it by
  4,994 micro-BPB. This bound charges each bucket N_b · log2(max_e g_e), as if all its weight could
  move to the best expert, and it is far looser than codex-14's tangent-term certificate: L is
  0.8965 × KN, where codex-14's lower end is 0.9894 × KN.
- **The KN start barely left KN** [claude]. After 96 passes at 100%, U_count is 393 micro-BPB
  (8,609 bits) below KN-only and still falling 9.5 bits a pass. KN holds 997‰ of the
  target-weighted weight (claude-433's uniform start ended at 600‰ after 24 passes). Weighted by
  each bucket's targets, on average 7.1 of the 12 other experts sit at exactly weight 1 of 2^32,
  and about one other than KN rises above 2^20 (the plain means over the 154 buckets are 6.4 and
  0.84). At every cut size the KN start ended above claude-433's 24 passes from uniform
  (U_count 2,293,695 against 2,283,303 at 1‰, 2,138,432 against 2,132,625 at 1%, 1,691,107
  against 1,690,355 at 10%).
- **The ORDER's reason for the start was wrong for this integer EM** [claude]. It said every
  present expert starts above 0, so EM can still move weight onto it, and that the start cannot
  change EM's fixed point. Reading the M step (`kn_ceiling.rs`, `fit_em_from` and
  `update_bucket`): each target's responsibility `(w·p << 32) / m` is truncated to an integer, and
  the new weight is `R_e · 2^32 / ΣR`, floored. For a weight of 1, each target's term is about
  p/m, truncated, so the weight stays at 1 unless the expert's ratios are mostly at least 2, and
  a weight at 1 is lifted back to 1 if it falls to 0. An expert whose mean ratio g lies in (1, 2)
  is held at 1, while the bound still counts what it could save. **This is a hypothesis drawn from
  the code and the weight census above; no test isolates it yet** (Still open). Real-valued EM
  would have no such trap. Codex-14 saw the same thing in its CUDA fitter: 970 of 1,182 active
  non-KN weights stayed at one throughout its near-KN fit.
- **The first pod job stopped on a bug in my job script** [claude]. In `b1`, the loop
  `for c in all novel` overwrote the global `c` that held the ceiling run's name, so `bars` then
  read `novel.json` and the job exited 3 after B1 at 1%. Every check it reached held: step 1
  39/39, step 2 B2 and B3 true at 1‰, B1 at 1%. The loop variable was renamed (`part`) and the
  output folder moved to `run2`; nothing else changed. Under the stop rule every step re-ran from
  step 1 in the second job. The first job's 1‰ and 1% JSONs are those sizes' records, and the
  second job's are identical to them (whole JSON, timings aside).
- **The memory sampler's 1‰ device peak is not reliable.** It read 796 MiB in the first job and
  8,052 MiB in the second for the same 12 s run: one sample a second misses short allocations.

## What changed

- `crates/uor-r4-discrete/src/kn_ceiling.rs`: `EmStart::{Uniform, Kn}` and `fit_em_from` (the KN
  start: 2^32 − (L − 1) on KN and 1 on each other present expert, a mask without KN refused);
  `gap_bound_q12`, the bound on how many bits the best real-valued bucketed weights can still save
  over given weights on the fit set: per bucket N_b · log2(max_e g_e), rounded up one Q12 unit per
  target, plus the integer floors' slack of 2 + ⌈8192 / mixed mass⌉ Q12 units per target.
  `fit_em` and `fit_em_with` keep the uniform start. New host tests: the start's weights, the
  start's bits against KN-only's, and the bound against what EM saves from both starts.
- `docs/labs/kn-reference/instrument`: every ceiling mode fits from the KN start with 96 passes;
  the JSON gains `em_start`, each fit's `gap_bound_q12` (per fold for U_split) and, for U_count,
  `lower_bpb_micro`; `unit` reads `claude-438`.
- The prep sheet [kn-ceiling-kn-start-2026-11-19](../../../../harness/prep/kn-ceiling-kn-start-2026-11-19.md).
- Run packets: [`run1/c438-job1/`](run1/c438-job1/) (the stopped job) and
  [`run1/c438-job2/`](run1/c438-job2/) (every step). No stream, mask, id list or count table is
  committed.

## Measurements

N = 10, budget 2^24, DEV-tune as query and, for U_count, fit set. Micro-BPB, overall and novel. All
on one Vast RTX 5090 (`vast-61370413`, 96 CPUs, 503 GiB), one build, stream sha256 values checked
on the pod against claude-433's.

| TRAIN | KN-only | U_count (U) | L | U_split | KN + U |
|---|---|---|---|---|---|
| 1‰ | 2,305,596 / 2,445,080 | 2,293,695 / 2,433,675 | 2,116,747 | 2,321,475 / 2,458,528 | 2,304,617 / 2,444,479 |
| 1% | 2,145,471 / 2,404,087 | 2,138,432 / 2,401,227 | 1,878,820 | 2,147,941 / 2,406,914 | 2,144,110 / 2,403,792 |
| 10% | 1,691,806 / 1,986,542 | 1,691,107 / 1,985,858 | 1,488,912 | 1,691,566 / 1,986,250 | 1,691,589 / 1,986,365 |
| 100% | **1,432,318 / 1,747,150** | **1,431,925** / 1,746,667 | **1,284,092** | 1,432,030 / 1,746,774 | 1,432,196 / 1,747,012 |

Parts per million of KN-only on the same cut (overall / novel; L overall only). At 100% KN-only is
R\*, so that row is also the ratio to R\*. Ratios to R\* at the cut sizes are not comparable.

| TRAIN | U_count | L | U_split | KN + U |
|---|---|---|---|---|
| 1‰ | 994,838 / 995,335 | 918,091 | 1,006,887 / 1,005,500 | 999,575 / 999,754 |
| 1% | 996,719 / 998,810 | 875,714 | 1,001,151 / 1,001,175 | 999,365 / 999,877 |
| 10% | 999,586 / 999,655 | 880,072 | 999,858 / 999,853 | 999,871 / 999,910 |
| 100% | 999,725 / 999,723 | 896,513 | 999,798 / 999,784 | 999,914 / 999,921 |

- **Gate (c) inputs at 100%:** U = 1,431,925, L = 1,284,092; 0.85 × R\* = 1,217,470 and
  0.90 × R\* = 1,289,086. L / R\* = 0.896513 and U / R\* = 0.999726; novel U_count / R\*'s novel
  = 0.999724 (no bound on the novel ratio).
- **The 100% fit:** fit bits 128,649,429,056 Q12 at the start (KN-only 128,649,439,118),
  128,627,655,182 after pass 24 and 128,614,178,256 after pass 96 (equal to the scored
  `bits_q12`); pass 96 saved 39,065 Q12 (9.5 bits). The bound is 13,278,202,590 Q12
  (3,241,749 bits), 377 times what the 96 passes saved.
- **Bounds of the other fits at 100% (Q12, recorded, no bar):** U_split's even and odd fit halves
  7,916,061,244 and 7,339,621,169, each bounding its own fit, not the cross-scored score; KN + U
  3,352,888,438.
- **B1:** KN-only equals the plain run's `all` and `novel` at 1% and 10%, and claude-422's
  1,432,318 / 1,747,150 at 100%; at 1‰ the host check passed (device lookups and host-checked KN
  masses).
- **B2:** true at every size. Start and last fit bits are at most KN-only's bits plus the target
  count (at 100% the start is 10,062 Q12 below KN-only), and the scored bits equal the last fit
  bits.
- **B3:** true at every size (L ≤ U, bound at least four units per target).
- **B4 (rises between passes, Q12):** U_count 8 rises at 1‰ (largest 4), 2 at 1% (23), 1 at 10%
  (20), none at 100%. KN + U 5, 8, 17 and 23 rises (largest 15, 52, 51, 316). U_split's folds:
  15 and 10 at 1‰ (21, 4), 25 and 0 at 1% (23), 0 and 3 at 10% (29), none at 100%. At most
  316 Q12 (0.08 bits), the size of the integer floors.
- **Weights at 100% (target-weighted, ‰; KN, ML_0..ML_10, uniform):** 997, 0, 0, 1, 0, 0, 0, 0,
  0, 0, 0, 0, 0. 154 buckets populated.
- **Cost:**

| TRAIN | wall | raw count | lookup | EM (four fits) | peak RSS | peak device (whole card) |
|---|---|---|---|---|---|---|
| 1‰ (host check) | 12 s | — | — | — | 8.2 GB | see above |
| 1% | 29 s | 7.1 s | 0.26 s | 5.3 s | 8.2 GB | 7,908 MiB |
| 10% | 175 s | 50.7 s | 0.42 s | 52.2 s | 8.2 GB | 8,696 MiB |
| 100% | 1,354 s | 352.5 s | 2.2 s | 546.6 s | 10.3 GB | 16,028 MiB |

  The pod ran 2,923 s (03:44:43Z to 04:33:26Z) at $0.908/h, about $0.74, of which the stream copy
  was about 5 minutes and the stopped first job 9 minutes.

## What the numbers say

- **Gate (c) is read from codex-14: Geometry critical.** This unit's own band only excludes
  Counts carry (L = 1,284,092 is above 0.85 × R\* by 66,622 micro-BPB), and that agrees with
  codex-14's interval. Nothing here contradicts codex-14.
- **Two things here add to codex-14:**
  - **A cross-implementation replication.** This unit's 24th pass from the KN start,
    128,627,655,182 Q12, equals codex-14's near-KN upper witness at pass 24 to the unit. Codex-14
    fit on wick's 4090 with its own CUDA fitter, and this unit fit on a 5090 host's CPU with
    `kn_ceiling.rs`, from TRAIN streams checked by sha256 on each machine.
  - **A tighter upper witness.** After 96 passes the fit is 128,614,178,256 Q12, 13,476,926 Q12
    (3,290 bits) below codex-14's witness. Combined with codex-14's lower end, the best all-target
    score of this family lies in **[0.98939548, 0.99972592] × KN**: an optimal mix of these
    experts can save at most 1.06% over KN, and 96 passes from KN found 0.027%.
- **The bound is the loose side here, not the fit.** The optimum is at least 0.9894 × KN, so this
  unit's L lies about 9.3 points under it, while U is within 1.04% of it. Charging each bucket
  for its single best expert's ratio is far weaker than codex-14's per-expert tangent terms, so
  the bound added here should not be reused where codex-14's certificate is available.
- **Recorded, not read as the gate:** U_count's gain over KN-only at 100% is 0.027% overall and
  0.028% on novel targets; U_split stays above U_count at every size, by 0.0073% of KN-only at
  100%, far below the "large split gap" threshold (half of R\* − U).

## Checks run at the head

- **G1, laptop** (at `d366c829f`): `kn_ceiling` host tests 12/12, including
  `the_kn_start_puts_the_rest_on_kn`, `the_kn_start_scores_kn_up_to_floors` and
  `the_gap_bound_covers_what_em_still_saves` (both masks, both starts, passes 0, 1, 3, 10 and 40
  against 400); the instrument's tests 11/11, including
  `the_lower_bound_subtracts_the_gap_and_floors`; stub-nvcc `cargo check`s of the crate and the
  instrument clean; D32 grep of the added lines clean; `kn_ceiling.rs` formatter-clean, the
  instrument's `main.rs` 2 rustfmt hunks as on main.
- **G2, negatives** (set aside against the base, run, restored): the bound taking a w-weighted
  mean of g instead of its maximum fails the bound test (868,534 saved against a bound of 768);
  the KN start leaving the other present experts at 0 fails two start tests.
- **G3, one Vast RTX 5090** (`vast-61370413`, jobs `20261119T035338Z-claude-438-gpu0` and
  `20261119T040330Z-claude-438-gpu0`, at `6c66a76d5`): step 1 39 passed, 0 failed, 4 ignored
  (the real-stream tests), every `kn_lookup_cuda`, `kn_score_cuda`, `ngram_count_cuda`,
  `kn_filter` and `kn_ceiling` test included, in both jobs; steps 2 to 4 rc 0; B1, B2 and B3 true
  at every size; the second job's 1‰ and 1% JSONs equal the first's.
- **Prep sheet FIT** at round 1 (fresh-context non-author), no blocker; its should-fix items are
  folded into the sheet.
- **After merging `main` (codex-14's `kn_ceiling.rs` refactor into `ceiling_expert_row` and its
  `--fit-frame` export), at `d83b21e18`, laptop:** host `kn_` tests 24/24; the instrument's
  targets 13, 2 and 1 passed; stub-nvcc `cargo check`s of the crate and the instrument clean. No
  run was repeated at the merged head; every number above is from `6c66a76d5`.
- **Fresh review at `1fcb1f079`**, two reviewers, each with the spec and one part:
  - **the code** (`kn_ceiling.rs`, the instrument) against the ORDER: APPROVE. The bound
    rounds up at every step (`bits_q12` never falls below the exact value, `floor(2^80 / G)`
    pushes up, `+ 1` per target covers G's truncation, the floor slack covers the mass floors).
    The KN start is safe for L = 1 and refuses a mask without KN. L uses U_count's target set
    and tally. The merge with codex-14's refactor keeps the input checks, and no D32 or §3
    pattern appears. Three low findings are under Still open.
  - **this REPORT's numbers** against the packets and codex-14's REPORT: REQUEST CHANGES, on two
    points, both fixed. The summary stated the floor trap as a finding, and is now worded as
    untested. The census figure 7.1 is target-weighted, which it now says, with the plain means
    (6.4 and 0.84) beside it. Every other number it checked matched, including both tables, the
    fit bits at passes 0, 24 and 96, the bounds, the codex-14 comparisons and job 1 = job 2.

## Skills used

None beyond the loop's own steps. The unit measures a bound with count experts; it changes no
model state, operator or artifact.

## Still open

- **No further gate (c) unit is needed:** codex-14 settled it, and its DECISION stops
  optimizer-start tuning within these buckets and experts. It names what PR 4103 could still add,
  a tighter reference bound and a cross-check of the fixed-point trajectory's prefix, and this
  unit delivers both above.
- **The floor trap matters if P1(b) (claude-436) fits its lawful EM weights with this M step.**
  Before such a fit, a host test should isolate the trap (an expert at weight 1 with mean ratio
  in (1, 2) never grows under `update_bucket`), and the M step should keep responsibilities at
  higher resolution or start with real weight on every expert.
- **R2 (claude-434, Pick) is not held by gate (c).** Codex-14's DECISION keeps P1(a, b) as
  prerequisites for P2 and leaves P3 to P5 on their existing dependencies.
- **Check an item's open PRs before an ORDER** (the duplication above).
- **The code review's three low findings, left for the next unit that touches `kn_ceiling.rs`:**
  - the `+ 1` per target in `gap_bound_q12` is not pinned by a test: the floor slack alone meets
    `gap >= 3 * fitted`. Tighten that assertion, or assert the exact sum on a fixture with
    G ≤ 2^32;
  - `the_gap_bound_covers_what_em_still_saves` checks the bound against 400 more EM passes, not
    against the real-valued optimum. That is necessary, not sufficient; validity rests on the
    Jensen argument;
  - the doc comment's "Any weights' scored bits are at least that best" holds for real-valued
    scoring; for a target whose mix falls below one Q48 unit, the integer floor scores fewer bits.
- **The device peak sampler** should poll faster than once a second, or read the CUDA driver's
  own high-water mark, before device peaks are compared at small sizes.
- Carried from claude-433: the device lookup's 32-bit row index, the raw-order log line's `ms`
  label, and the two REPORT nits.

## Plain summary

This unit asked again how much better than our reference model (R\*) the best blend of simple
counting models can do on the tuning set. Codex's lab answered the same question in parallel and
finished first: the best blend is at most about 1% better than R\*, so the project's quality gain
has to come from the geometric parts. This unit's own proven floor was too loose to say that much
(it only ruled out the "counts carry the project" line), but its numbers match Codex's exactly
where they overlap, on a different machine and different code, and its longer fit narrows Codex's
range slightly from above. It also suggests a likely reason, not yet tested, why the blend hardly
moves from R\*: with whole-number weights, a model holding the smallest possible share may not
grow unless it is at least twice as useful as the blend. If that holds, it matters for the
deployable count model, should it fit the same kind of weights. The duplicated work cost about $0.74 of rented GPU and should have been caught by
checking the item's open pull requests first.
