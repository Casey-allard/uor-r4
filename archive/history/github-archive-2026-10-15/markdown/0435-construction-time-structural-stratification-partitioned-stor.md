# #435 — Construction-time structural stratification: partitioned store construction vs flat (from #393; candidate mechanism for #433)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-05T16:37:14Z
- **Updated:** 2026-08-07T05:08:50Z
- **Closed:** 2026-08-07T05:08:50Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/435
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The last unresolved #393 program row, confirmed unstarted (the 'private track' never ran — Casey, 2026-08-05). **Claim to test:** building the store/cover in structural strata (partitioned construction) beats one flat construction on held-out, graded against #391 null discipline. **Why it matters more now:** the #399 falsifier-2 record shows the substrate scores 36.5% when construction is effectively stratified (few deeply-observed topics) and ~26% when one flat construction spans 10,000 topics at ANY measured capacity — the corpus-composition finding is plausibly a stratification effect. If per-stratum construction + stratum routing recovers the redundancy-regime numbers on the broad corpus, this is THE mechanism for #433's ceiling. First measurement: partition wiki10k construction by story/topic clusters (e.g., per-story or k-cluster strata), build per-stratum stores, route held-out queries to strata (oracle-stratum arm = ceiling; routed-stratum arm = the claim; flat arm = the measured 26% baseline), grade with pre-declared exit rule. The #432 env knobs and existing harnesses make this cheap to stand up.

## Comments (7)

### comment 5194869959 — 2026-08-05T17:07:28Z — Casey-allard

Home-regime measurement (harness PR #438): **NEGATIVE under the pre-declared rule** — content-K8 routed-backoff 34.6% vs flat 35.2% (−0.6pp; rule needed +2pp); K32 34.2%; oracle ceilings only 36.0–36.3% (barely above flat; flat evidence-hit already 90.1%). On the coherent home corpus, stratifying the evidence store adds nothing — the flat store already captures the evidence. **The stratification claim's real test is the broad corpus, which is now teacher-gated (#433's finding → #320)**: this issue stays open, scoped to re-measure once a suitable teacher generates a broad corpus. Recorded per discipline.

### comment 5194886408 — 2026-08-05T17:09:05Z — Casey-allard

Scope note (Casey): NOT ruling stratification out — the oracle-stratum ceiling (36.3%) sits ABOVE flat (35.2%) even on the home regime, so stratum-level signal exists; the routed arm's miss means the cheap unigram-signature ROUTING is what failed, not the stratification idea. Open follow-ups: better stratum routing (the store's own evidence as router), and the broad-corpus re-measure once #320 lands a suitable teacher. Issue stays open and assigned.

### comment 5196230921 — 2026-08-05T19:14:57Z — Casey-allard

Routing v2 record (PR above): evidence-mass routing NEGATIVE (best 34.0 vs flat 35.2) — but mechanistically informative: hard selection discards cross-strata counts flat aggregates; oracle wins are distribution flips on shared keys. v3 = soft stratum-distribution mixtures (per-key confidence weighting). Stratification still not ruled out; the oracle edge (36.3) remains the recoverable target.

### comment 5196854362 — 2026-08-05T20:15:39Z — Casey-allard

v3 record (PR #443): NEGATIVE with an algebraic close — mass-linear mixing provably ≡ flat; affinity tilt recovers <9% of oracle flips. Home-regime routing EXHAUSTED (3 designs + identity). Issue dormant pending #320's broad-corpus teacher, where the stratification claim actually lives. Oracle edge (36.3 vs 35.2) stands as the recorded unrecovered signal.

### comment 5199339094 — 2026-08-06T01:26:29Z — Casey-allard

Growth law + criterion sweep (PR #445, harness in #444). The class count does NOT saturate: distinct left keys ~n^0.44, distribution-signature classes ~n^0.46 (left) / ~n^0.80 (two-sided) across a 17× data range, with mean support per class rising 7.3→35.6 (genuine structure, not noise-splitting). Criterion sweep at 2.11M records: fixed-θ relative FAILS (H(parent) grows 6.52→9.65 bits so θ·H becomes a stricter bar, cover collapses to 8 regions), MDL FAILS, and only the capacity law k0_eff = k0·(n/n_ref)^0.45 is monotone (14→23→25 regions) while beating baseline held-out top-1 (0.0470 vs 0.0438). Conclusion: scale capacity, don't tune thresholds.

### comment 5201046809 — 2026-08-06T06:12:29Z — Casey-allard

Capacity-law follow-on result (run at 02:30): enabling scaled k0 (α=0.45) on the 2.11M corpus took the cover from 14 → 25 regions exactly as designed, but Rule 1+2 came out at **26.1%, identical to baseline**, bits 22.0948 vs 22.1317. Reason: 393,851 of 402,802 held-out positions resolve as ExactContext and only 8,635 (2.1%) ever touch the graph path — fixing the cover cannot move a headline the cover barely participates in. **The cover collapse was real but is not the cause of the scale regression.** The binding constraint is the fixed 4×256 graded code space diluting as records grow, which #446 M1 then confirmed from the other side (17.1× cell subdivision by right context gains +4.6pp on exactly the diluted population). Recommend narrowing this issue to the capacity law itself (shipped, default-off, PR #445) and tracking code-space capacity separately.

### comment 5212669662 — 2026-08-07T05:08:49Z — Casey-allard

Closing the stratification claim with its record; the one live leg attached to this issue moves to #460 so it has a single home.

**The claim in the title is exhausted.** Three routing designs plus an identity argument, all against pre-declared +2pp rules recorded in `strata_construction.rs`'s module docs: v1 content-K8 routed-backoff 34.6% vs flat 35.2%; v2 evidence-mass routing 33.7-34.0 vs flat 35.2; v3 mass-linear mixing **reduces algebraically to flat** and is retained only as the mixture-machinery control. The oracle-stratum edge (36.3 vs 35.2) stands as recorded unrecovered signal, but no routing design reached it and the algebra says mass-linear mixing never will.

**What was left dark here:** PR #445 shipped a second, unrelated leg under this issue number — `SplitCriterion::{RelativeGain,Mdl}` plus `scale_k0`/`scale_regions_budget` in `induction.rs`, all default-off, with an ignored harness `tests/cover_scaling.rs` carrying its own 3-condition exit rule. That code is shipped-but-unmeasured. It is a cover-criteria question, not a stratification question, and it is the same measurement #460's audit ranks as its highest-priority lever — so it now lives on #460 rather than dangling behind a dead claim.


---
_Comment/review ids in this file: 5194869959, 5194886408, 5196230921, 5196854362, 5199339094, 5201046809, 5212669662_

