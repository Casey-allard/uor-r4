# #2143 — M2 stratified reply fit: reject 2/24 and 2/512; retain accepted8

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T05:33:59Z
- **Updated:** 2026-10-10T05:44:40Z
- **Closed:** 2026-10-10T05:36:24Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2143
- **Merged:** 2026-10-10T05:36:24Z
- **Merge commit:** d62811d02a814f3124383744fabfef4950f8aecf
- **Base / Head:** main / UOR-Foundation:codex/m2-stratified-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: ordinary reply-completion gradient learning · count 2/3 · M2 headline 8/512 → 8/512 (candidate 8/512 → 2/512).

## What and why

The preceding full-panel pass (#2141) regressed from8 to2 complete replies. This pre-registered model-changing qualification tests whether concentrating ordinary Potential/Generate training on24 fixed exposed examples—including alloriginal8—can establish complete-answer learnability before curriculum expansion. The fixed schedule is96 AdamW updates,32 exposures perexample,9,280 answer/EOSpositions. Context, prototypes, bridge, Cue and Prefix stay fixed; no constructor, attribution or solver work reopens. Legacy64 mode is preserved.

Pre-registration: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6093892615

## Result

**REJECT.** The saved/reloaded native endpoint produces2/512 complete correct replies and2/24 on its training subset. Six original successes are lost, none gained; the same two complete rows survive as in #2141. All16 new selected examples still miss the opening token. Only2/24 have every token correct with canonical teacher prefixes, although subset token correctness improves96→118/290 and full-panel teacher-prefix CE6.199264→5.591269. The accepted8/512 parent is retained. This does not prove24 or512 is unrealizable by the native model family.

The full unchanged512 evaluation always runs; all1,024 saved endpoint rows authenticate. Source/evaluator/data identity, full gain/loss IDs, perstratum counts, resources and restoration instructions are in [the canonical result record](docs/labs/m2-stratified-reply-2026-10-10/README.md). Full sealed evidence/checkpoints/runtime/parent/panel/sourcebundle are in verified iCloud package `codex-m2-stratified-reply-20261010`:164157440bytes, MD5`81884843fe3bc9e917d0da16910a66d9`. It was streamed with full download MD5 verification and standardindex to avoid localarchive staging below thediskfloor.

Model process33.11min; producer optimized CUDA build232.170s and sixfocusedtests25.234s; bootstrap200s separately. Peak modelRSS1437044KiB, sampledGPU4118MiB (22MiB above4GiB projection, disclosed). OneleasedRTX5090 at$1.19/h; no newspendclass. Entire preparation/review/preservation/delivery/hygiene elapsed is charged from04:40UTC, with finalcloseout posted aftercleanup.

**Counterfactual PR question:** if this result came out the other way, would my next step differ? Yes: retained24/24 would justify a larger fixed qualification; retained partialgain would preserve that candidate. This regression stops the recipe and does not justify expansion or an unchanged dose/rate/seed sweep. No successor compute starts in thisdelivery. Zero preparationPRs.

## Checklist

- [x] Checks and independent review at exact head5ad36052e4cc69d6e30f0bc35f5f93b57c2e1532 posted beforequeue; six focused CUDA-feature tests pass, zeroignored
- [x] Canonical dated result record and newest current-state entry
- [x] STATUS/ROADMAP/README updated; #2028 table updated afterverifiedmerge
- [x] Frozen acceptance/panel/grader preserved; candidateREJECT and parentretained
- [x] Full source/negative evidence durably preserved, hash-verified
- [x] Claim-wording gate, touched-source rustfmt and diffcheck pass
- [x] Protectedmerge verified on freshmain, branches/worktreeremoved, hygiene and final#2030card posted

References #2030. M3 remains unchanged; no milestone is closed.


## Timeline (3 entries; chronological: conversation comments, reviews, review comments)

### comment 6094262033 — 2026-10-10T05:35:29Z — Casey-allard

Exact-head independent review PASS at5ad36052e4cc69d6e30f0bc35f5f93b57c2e1532 (reply_review agent), no blocking findings. Native example source is byte-identical to executed producer30c7d018825dcd4b3f2d2e7791af66ab4c192635.

Reviewer read raw report+manifest on pod through uor-pod SSH: reportSHA16d03a31fc1eb27ee270cec96a7d388c108522c79b2a87856a0ca94b232aab68 andmanifest987bc137e5876821aef3b80777986c40bf3808f90203a46060fd135bb1e53720 matched. Committedsummary byte-identical to rawreader SHA4e769c9287e659ba75cac2bb750764dc81e4c5bdb893e995aba19a661e539fe1. Independently reconciled96updates/768draws/9280positions, all48subsetendpoint measurements, fullcompletion sets, CE, gain/loss, source-swap pairs4→0, andall16newentryfailures. REJECT/count2/3/accepted8retained and no-incapacity-proof claims are supported. Review doesnotrerunmodel/backwards/tokenizer orrepeatcloudtransfer. Resourceoverrun4118MiB vs4096MiBestimate and localfloor/streamingfallback are explicit.

Executed exact-head checks (not CI acknowledgement): touched-source rustfmt --edition2021 --check PASS; git diff --check PASS; python3 scripts/check_claim_wording.py PASS; sourcecomparison withproducer PASS. CUDA focused-test command and timing follow in the same deliveryreceipt. GitHubrequired-transport names are compatibility acknowledgements and are not these executed tests.

Actual final-head command: `CARGO_TARGET_DIR=/root/build/target-codex-m2-stratified CUDA_COMPUTE_CAP=120 CARGO_BUILD_JOBS=4 UOR_BUILD_SOURCE_COMMIT=5ad36052e4cc69d6e30f0bc35f5f93b57c2e1532 cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit reply_ -- --test-threads=2`:6passed,0failed,0ignored,164filtered; command97.1889994919984s, peakRSS2279872KiB, exit0. This recompiles/exercises the changed Rust path; no second model run.


### comment 6094280541 — 2026-10-10T05:38:11Z — Casey-allard

Verified protected merge of #2143: d62811d02a814f3124383744fabfef4950f8aecf on fresh origin/main. All19 delivered file blobs equal reviewed head5ad36052e4cc69d6e30f0bc35f5f93b57c2e1532. #2028 M2 table updated; accepted8/512 retained, negative ordinary-learning count2/3. Remote deliverybranch deleted. Exact-head6CUDAfocusedtests pass(0ignored), independentreviewPASS.

Evidencepackage codex-m2-stratified-reply-20261010:164157440bytes, MD581884843fe3bc9e917d0da16910a66d9; closeout package codex-m2-stratified-reply-closeout-20261010:81920bytes, MD511146efedc7ac42b5bdf33e696eadee5. Both iCloud fulltar download MD5roundtrip verified andstandardindex reread. Closeout contains finalheadtest logs, review/delivery receipts andmergeverification. Streaming avoids large localarchive staging belowthefloor. Podrelease/shutdown andownedworktree/branchcleanup inprogress; finalhygiene andcompleteledgercharge follow. No successor modelrun.


### comment 6094325609 — 2026-10-10T05:44:37Z — Casey-allard

M2 headline: 8/512 complete correct replies (target 256) · line: ordinary reply-completion gradient learning · count 2/3 · result session complete.

Landed #2143 through protected merge d62811d02a814f3124383744fabfef4950f8aecf. Fresh origin/main verified; all 19 delivered file blobs match reviewed head 5ad36052e4cc69d6e30f0bc35f5f93b57c2e1532. Zero preparation PRs; one model-result PR. STATUS/ROADMAP/README/current-state and #2028 M2 table are updated. No milestone is closed.

REJECT: the pre-registered 96-update stratified fit scores 2/24 trained replies and 2/512 on the unchanged full panel, losing six original successes and gaining none. All 16 new training rows still miss entry; all-token teacher-prefix correctness is 2/24. Accepted 8/512 is retained. This does not prove 24 or 512 impossible for the native family. The fixed Potential/Generate-only recipe stops: do not expand its curriculum or repeat an unchanged dose/rate/seed sweep. Closed constructor/attribution/solver work remains closed. Next M2 compute requires a distinct model-changing intervention and pre-registration; no successor started here. M3 is unchanged.

Validation: producer optimized CUDA build and focused tests passed; final-head CUDA tests passed 6/6, zero ignored (97.189 s command), formatting/claim-wording/source identity checks passed, and independent exact-head source/result review passed. All 1,024 saved endpoint rows authenticate; raw report and manifest hashes match the committed record.

Durable evidence: iCloud package codex-m2-stratified-reply-20261010 (164157440 bytes, MD5 81884843fe3bc9e917d0da16910a66d9), plus codex-m2-stratified-reply-closeout-20261010 (81920 bytes, MD5 11146efedc7ac42b5bdf33e696eadee5). Both whole-archive MD5 download round-trips and standard indexes verified. Direct streaming avoided local tar staging while the local disk was below the floor; cloud-store fetch is compatible.

Model process: 33.11 min. Pod 5ty2c8xjj1azsm: 3150 seconds at $1.19/h, estimated $1.04125, lease released and pod deleted. Sampled GPU peak 4118 MiB exceeded the 4 GiB estimate by 22 MiB; disclosed in the result. Full session charge through final hygiene: 3850362 ms (64.17 min); cumulative 1471610605 / limit 1477800028 ms, no reset. Standing same-class 60-minute extension is recorded.

Remote/local delivery branches and owned worktree are removed; owner checkout is clean on main. Required post-merge hygiene completed exit0, reclaimed about0 GB, and preserved other labs' active builds/material. Free space remains about22 GiB (23631110144 bytes), below the local admission floor; no further local artifact/build allocation is admitted in this session. No Codex model job or GPU lease remains. Work claim released; finish line met by the merged result and this card.



---
_Comment/review ids in this file: 6094262033, 6094280541, 6094325609_

