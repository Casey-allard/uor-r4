# #2157 — M2: bottleneck objective raises saved complete replies from 145 to 175

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T12:41:41Z
- **Updated:** 2026-10-10T12:56:57Z
- **Closed:** 2026-10-10T12:47:29Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2157
- **Merged:** 2026-10-10T12:47:29Z
- **Merge commit:** 4f12574a4f442445909af5ec885aba84fb4dc1ae
- **Base / Head:** main / UOR-Foundation:codex/m2-bottleneck-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The saved native model completes **175/512 replies**, up from145:34 gains,4 losses,141 retained. This run replaces phase-balanced token training with an episode log-mean-exp objective on the same115,200 Q4 field, restoring the accepted145 fractional masters and freezing the upstream model/scorer. It is **KEEP** under the pre-registered complete>145 bar.

**Line: cross-state continuation · count 0/3 · headline 145/512 → 175/512.** Zero preparation PRs. References #2030; M2 remains below256 and fresh40% qualification is NOT_RUN.

The fixed256-update saved/reloaded endpoint improves every memory stratum, including length8 from2→8/128. Entry correctness454→340, teacher-prefix token correctness4110→3523/6664 and evaluator CE3.39448→3.48474 regress; these tradeoffs are explicit. No held-out, general-chat, geometry-advantage or energy claim.

[Pre-registration](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6097346580) and [pre-run metric-label correction](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6097395590). **If this came out the other way, would my next step differ? Yes:** a non-gain would preserve145/count2 and stop unchanged objective continuation; this gain retains175/count0 and supports a separately registered continuation from it after protected delivery/cleanup. The stopped phase-balanced recipe and constructor/attribution/solver line remain closed.

Validation at model producer351e93dcc: optimized Rust build and17 focused tests PASS; independent exact-head source and saved-evidence review PASS; exact512 baseline replay; unchanged full512 own-prefix endpoint; nine historical comparator checks. Model705.943s exit0, peak RSS802204KiB and sampled GPU837MiB. Artifact114309120B fully streamed to/from iCloud, MD5 and six-column index/readback PASS. Final delivery head a2cd2ee25 passed17focused tests, optimized build, format/diff/claim-wording/evidence checks and independent review; exact-head validation is posted before protected queueing.

Canonical record: docs/labs/m2-bottleneck-2026-10-10/README.md, with exact config/artifact/reader identities and measured costs. README, STATUS, ROADMAP and current-state updated. Delivery requires protected merge, fresh-main whole-tree verification, branch/worktree deletion, hygiene and complete-cycle accounting; this PR ends a cycle, not the standing goal.

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### comment 6097643612 — 2026-10-10T12:46:36Z — Casey-allard

Exact-head validation/self-review for **a2cd2ee25a518af978ee1663045b103482d2b9cb**:
- On owned pod vmsxf89l6ggbum with per-lab target: named rustfmt, diff, claim-wording, **17 focused Rust tests** and optimized fitter build PASS. Final source tree's crate files equal measured producer351e93dcc exactly. Model executable302e36db... remains preserved; final delivery build is distinct metadata, not a new model run.
- Evidence hashes/config/preservation receipts and KEEP175/count0 PASS. Independent final exact-head review PASS (receiptSHA92092fb5cf12949ef2b9ab39035d74256392bd7881e563df194dc3a9d77ffcc9). Self-review confirms registered objective/admission/streamingchain rule, unchanged native scorer/evaluator, scoped claim/regression reporting and delivery records.
- Model producer actual baseline512 exact, final175,34gains4losses141retained; savedreaderPASS all9comparators. Artifact iCloud114309120B MD58b10bf24ca7d5f042a7395eeb4164fb5 fullreadback/indexPASS.
- Review corrected one metric-label prose error before queueing; all intermediate source/build/review receipts retained. No model/result repair.
- GitHub's five status names are transport acknowledgements, not tests. Actual commands/logs at /workspace/uor-r4/codex/m2-bottleneck-20261010/final-checks-corrected; closeout will retain them.
Protected queue authorized by standinggoal. M2 remains unmet; no issue closure.

### comment 6097733827 — 2026-10-10T12:56:57Z — Casey-allard

**M2 headline:175/512 complete correct replies (target256) · line:cross-state continuation · count0/3 · next:pre-register parameter continuation from saved175 under the successful bottleneck objective.**

Cycle result **KEEP145→175**,34gains4losses141retained; six original8 retained. Length8 improves2→8/128, while entry454→340, teacher-token4110→3523/6664 and evaluator CE3.39448→3.48474 regress.337 complete failures remain; fresh40% NOT_RUN. No acceptance, scorer, panel or serving contract change.

**Delivery COMPLETE:** #2157 protected merge **4f12574a4f442445909af5ec885aba84fb4dc1ae** verified on fresh origin/main. Whole-tree **4c94561f9c53e914adcf87aa7e02024a7735d012** equals tested/reviewed head **a2cd2ee25a518af978ee1663045b103482d2b9cb**. Producer351e93dcc and finalhead each17focusedtests/buildPASS; independent exact-head/source/saved-reader/finalreviewPASS. All9historical comparators checked. GitHub/localdeliverybranch and bothownedsourceworktrees deleted;61ignoredlocalfiles SHAverified before removal. uor-hygiene --apply exit0,free42GiB,otherlabs active/unpreserved work retained.

**Preserved:** saved model report9632d32651d0bcce82b2cda2ba70dee7d7a7878ff99577f88002442e23acb572; manifest1a6216a307a739a804f81bf94e93fd89638a896fda1904986dd3aee0a280fccc; fieldc82c376df10f14d5c1c9af970fd3b1fe239fb3560e2ccd0020dd2805d8fb773e. iCloud codex-m2-bottleneck-20261010:114309120B,MD58b10bf24ca7d5f042a7395eeb4164fb5; closeout983040B,MD57ca97f068d2b7681becfc947499a689a. Both fullstream readback/indexPASS,no local archive staging. Canonical volume /workspace/uor-r4/codex/m2-bottleneck-20261010 retains model/runtime/receipts; sourceworktree removed. Closeout archive marks hygiene/cost thenpending; this card is their final receipt: hygiene nowPASS,final log also retainedonvolume.

**Complete cost:** cycle12:06:45→12:56:40Z **49m55s**,2995000ms charged including preparation/build/model/review/preservation/merge/cleanup. Shared cumulative1489462627→1492457627ms,limit1495800028ms (prior authorized+1800000 extension retained). Model705.943s,peakRSS802204KiB,sampledGPU837MiB. Podvmsxf89l6ggbum lease released/deleted12:56:23Z,2669s@$1.19/h≈**$0.882253**. No owned live pod/job/lease/sourceworktree remains. Setupref/cache assumptions and review-label correction retained; no modelrerun or evidence repair.

The phase-balanced dose/rate/seed,ordinary24/96,andconstructor/attribution/solver lines remain closed. No STOP CODEX or ownerblocker. This result closes one cycle, **not the standing goal**; the next cycle begins from175.


---
_Comment/review ids in this file: 6097643612, 6097733827_

