# #2141 — M2 reply-completion gradient result: reject 8→2/512 candidate; retain accepted8

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T04:28:35Z
- **Updated:** 2026-10-10T04:39:08Z
- **Closed:** 2026-10-10T04:29:53Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2141
- **Merged:** 2026-10-10T04:29:53Z
- **Merge commit:** 0d1314053175e6b60f5ed491802d4fa94e4b7010
- **Base / Head:** main / UOR-Foundation:codex/m2-reply-gradient-20261009
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: ordinary full-panel reply-completion gradient learning · count 1/3 · M2 headline 8/512 → 8/512 (candidate 8/512 → 2/512).

## What and why

Following D21, this runs ordinary AdamW on the accepted Source48/Generate64 learner's Potential and Generate coefficients over all 512 complete canonical answers/EOS. Context, prototypes, bridge, Cue and Prefix stay fixed; no U, protected constructor, attribution or solver step. The earlier protected/discrete line remains closed, with unactivated source indexed in the branch archive. Zero preparation PRs; this is the session's one integrated result PR.

Preregistered before compute: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093395330. Before opening this PR: “If this result came out the other way, would my next step differ?” Yes: retention plus new complete replies would justify wider qualification; this regression rejects the candidate and stops this fixed recipe without an unchanged dose/rate/seed rerun.

## Result

**REJECT candidate; retain accepted 8/512 parent.** Fixed seed1001,64 updates,batch8,6,664 answer/EOS positions. Saved native checkpoint64 completes2/512 versus8/512 baseline: retained2, lost6, gained0. Complete source-swap pairs4/256→0/256, correct entry8→7, EOS11→3. Teacher-prefix CE6.1992635→5.4545226 does not qualify reply improvement. Both endpoints use the unchanged frozen panel and own-prefix grader. This exposed training/development result is not held-out, general conversation, durable memory or energy evidence.

Producer `9a1394f4efb480fbabe95c2c8c80b3c4ad24dce0`, executable SHA256 `dbcf785773136115ececb066a828b6e3102edf5bb1b12d19accd1194ea253b8e`; report SHA256 `678f4ff4d8b97559260076607f6168b7d5f4317cc1472bd9fe5e253ca76bdbb7`. Full28.01min process exits0, sealed/verified report; saved-row reader authenticates1024/1024 row hashes/IDs and reconstructs EOS/membership/own-prefix feedback and gain/loss sets. Native inference/backward replication is outside that reader's scope.

Full raw evidence, exact inputs/parent, checkpoints/runtime and source history are MD5-round-trip verified in iCloud: `codex-m2-reply-gradient-20261010`,129723392bytes,MD5 `2771a7f1f7ca8007ecba1f7c0e5d87e3`. Failed setup/compile attempts are retained separately from model quality. Canonical record: `docs/labs/m2-reply-gradient-2026-10-10/README.md`.

## Validation and delivery

- Producer optimized CUDA build and focused tests:3 pass,0 fail,0 ignored; matching CPU tests:3/3. Bootstrap parity separately37 pass,3 ignored.
- Exact-head checks and independent review are posted below before queueing; CI transport names do not execute tests.
- New current-state entry; README/STATUS/ROADMAP and geometry/closed-source index updated. #2028 table/#2030 outcome updated at delivery.
- Claim-wording, scoped rustfmt and full diff whitespace checks pass.
- Complete ledger/pod cost, final verified merge, branch/worktree deletion and post-merge hygiene are recorded in closeout comments.

References #2030


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6093772424 — 2026-10-10T04:29:01Z — Casey-allard

Exact-head independent review: PASS, no blocking findings.
Reviewer: reply_review independent agent.
Head: a5799895cf6bf90ac9656d74146998b5f200c2cf.

Reconciled committed result-summary against producer report and saved-row review: candidate8→2, gains0/losses6/retained2, changedoutputs510, complete swap pairs4→0, keepfalse. Independently recomputed report/manifest SHA256 and verified manifest byte/BLAKE3 for report, bothmetrics and rowcomparison. Source unchanged from afebac942 review; both reply source hashes match producer9a1394f4. Fixed64-step schedule/frozen families/unchanged own-prefix evaluator and all-answer/EOS loss reviewed. Claims retain accepted8 and distinguish CE from completion; counterfactual changes nextstep; count1/3 on ordinary fullpanel intervention, constructorline remainsclosed. Process exit/timing/RSS/sampledGPU and cloudreceipt match records. Scope: source and saved evidence; model/native/backward not independently regenerated, cloud roundtrip not independently repeated.

Executed checks at head a5799895cf6bf90ac9656d74146998b5f200c2cf: `cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit reply_completion -- --test-threads=2` on admitted pod:3pass/0fail/0ignored,97.093s compile+test, peak2,300,148KiB. CPU retained binary3pass/0fail/0ignored and checked reply-source hashes identical. Scoped rustfmt, claim-wording and full `git diff --check origin/main` PASS. No model rerun. Five transport statuses are compatibility acknowledgements only.


### comment 6093787422 — 2026-10-10T04:31:09Z — Casey-allard

Protected merge verified: 0d1314053175e6b60f5ed491802d4fa94e4b7010 is on fresh origin/main; all20 delivered paths match reviewed head a5799895cf6bf90ac9656d74146998b5f200c2cf blob-for-blob. Final integrated CUDA-feature test compiled and passed3/3 (97.093s; no ignored tests). Full experiment evidence is iCloud-verified. GPU lease released and pod fhaffpigxdge4a deleted. Branch/worktree and final ledger/hygiene closeout follow; owner has separately requested an exact panel/trainability assessment, which is being performed read-only without new model compute.

### comment 6093847518 — 2026-10-10T04:39:07Z — Casey-allard

M2 headline:8/512 complete correct replies (target256) · line:ordinary full-panel reply-completion gradient learning · count1/3 · result session complete.

Landed #2141 via protected merge0d1314053175e6b60f5ed491802d4fa94e4b7010, verified on fresh origin/main with20 delivered file blobs identical to reviewed heada5799895cf6bf90ac9656d74146998b5f200c2cf. Fixed64-update candidate8→2/512 is REJECTED (retained2/lost6/gained0); accepted8 preserved. Zero preparationPRs, one model-resultPR. Closed protected/discrete/solver line remains archived; compile-failed source also indexed. No milestone closed.

Producer process28.01min; GPU pod2963seconds at$1.19/h, estimated$0.97944. Full raw129723392byte experiment package MD52771a7f1f7ca8007ecba1f7c0e5d87e3 and45262336byte closeout/assessment package MD5a32c7e30a01c6dcca93d8da94ccab24a verified in iCloud. Names:codex-m2-reply-gradient-20261010 andcodex-m2-reply-gradient-closeout-20261010. Lease released, podfhaffpigxdge4a deleted; no Codex model job running. Remote/local delivery branches and owned worktree removed; owner checkout clean. Required hygiene result and complete cumulative charge follow below.

Owner follow-up assessment: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093792471. Supplied facts and runtime limits permit all canonical answers, but current learned-family512/512 reachability remains unproven.512 distinct entry states; target opening token unavailable toCopy on all512, baseline entry correct only8. The historical512 answerability control is explicit-reference evidence with native_predictionsNOT_RUN, not a native reply result. Next: one pre-registered stratified complete-reply learning qualification, with exact native save/reload, teacher-prefix versus own-prefix distinction, and unchanged full512 endpoint; no unchanged dose/rate/seed repeat and no constructor/solver reactivation. That successor is NOT_RUN; M3 unchanged.

Hygiene completed exit0, about1GB reclaimed and32GiB free; active builds preserved. Complete session charge through final cleanup:3755000ms (62.58min), cumulative1467760243ms / limit1474200028ms. Standing120min extension remains recorded; no new spending class.



---
_Comment/review ids in this file: 6093772424, 6093787422, 6093847518_

