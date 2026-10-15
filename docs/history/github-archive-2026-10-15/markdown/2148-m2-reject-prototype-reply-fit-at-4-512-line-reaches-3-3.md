# #2148 — M2: reject prototype reply fit at 4/512; line reaches 3/3

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T08:13:29Z
- **Updated:** 2026-10-10T08:23:56Z
- **Closed:** 2026-10-10T08:16:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2148
- **Merged:** 2026-10-10T08:16:36Z
- **Merge commit:** 52022ae10c35ed68011124bb3a8994154425041c
- **Base / Head:** main / UOR-Foundation:codex/m2-prototype-reply-20261010
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The registered prototype intervention changes the saved native model but regresses complete replies from 8 to 4 of 512. Four accepted successes are lost and no new complete replies appear; the accepted 8/512 parent is retained. The fixed 24-row fit also falls 8→4/24, so the candidate is REJECTED.

**Line: ordinary reply-completion gradient learning · count 3/3 · headline 8/512 → 8/512 (candidate 8→4).**

This adds ordinary Generate prototype-choice training to the existing 96-update qualification recipe. Context remains frozen; the protected constructor/attribution/solver line stays closed. Independent saved-result review confirms all 1,024 endpoint rows, both earlier rejected comparisons, and 31,828/32,768 changed native prototype codes. All sixteen added training examples still fail at entry. No architectural-impossibility or convergence claim follows.

Before opening: **if this came out the other way, would my next step differ? Yes.** Retained 24/24 fit would support extending qualification; retained new complete replies would preserve progress. Regression stops this recipe and requires the mandatory D21 pivot after merge. This result ends a cycle, never the continuous goal.

Pre-registration: https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6095037606
Result record: docs/labs/m2-prototype-reply-2026-10-10/README.md.
No preparation PRs. Negative artifacts, executable, source and exact inputs preserved in iCloud with full archive MD5 round-trip and standard index. STATUS/ROADMAP/current-state updated.

Validation: producer optimized CUDA-feature build and eight focused reply tests passed; independent producer/source and saved-artifact review passed. Exact delivery-head commands and self-review will be posted before queue admission. Full model process 2,077.447 seconds, peak RSS 1,463,744 KiB, GPU memory peak 4,216 MiB. Whole-cycle and pod costs remain cumulative and will be closed after delivery.

References #2030.

## Timeline (4 entries; chronological: conversation comments, reviews, review comments)

### comment 6095577235 — 2026-10-10T08:14:27Z — Casey-allard

Exact-head self-review at `ac7ff42c6bc69d556d4258201567e5f2d1ea8f4d`: PASS. I inspected the diff from current main, confirmed prototype parameters join the selected gradient set before clipping and have a separate ordinary AdamW optimizer, frozen-family identities are checked, the fixed schedule and saved/reloaded baseline/endpoint are unchanged, and report scope distinguishes the prototype mode. The complete-reply KEEP predicate remains all-original-eight retention plus new complete gains. Canonical documents consistently reject this candidate and retain accepted 8/512; inherited count is 3/3. No serving-path activation or acceptance change. Independent exact-head review by reply_review also PASS, following independent row/hash/native-code verification. Its only note was that the root README still accurately describes the two earlier explicitly scoped interventions. Final-head actual Rust checks are pending and will be posted before queuing.

### comment 6095586878 — 2026-10-10T08:15:44Z — Casey-allard

Actual checks PASS at exact head ac7ff42c6bc69d556d4258201567e5f2d1ea8f4d on owned pod p9mrhlsn2jlsha, isolated source and CARGO_TARGET_DIR=/root/build/target-codex-m2-prototype:
- git diff --check.
- rustfmt --edition 2021 --config skip_children=true --check on the two changed Rust files.
- python3 scripts/check_claim_wording.py.
- cargo test --release --locked --offline -p uor-r4-training --features cuda --example geometric-frozen-map-fit reply_ -- --test-threads=2: 8 passed, 0 failed, 0 ignored, 164 filtered; command 100.125 seconds, peak child RSS 2,292,092 KiB.

Actual model execution is producer0462003, whose Rust source is identical to this delivery head. Baseline/final full512 saved rows and native code payloads independently verified. The required GitHub status names are transport acknowledgements, not these tests. Exact-head self-review and independent review PASS are recorded above.

### comment 6095633656 — 2026-10-10T08:22:06Z — Casey-allard

Delivery closeout: MERGED52022ae10c35ed68011124bb3a8994154425041c, verified ancestor on fresh origin/main and entire tree equal to ac7ff42c6 (ad594fd6a56f900479fbe99cc59f337e2e3ee42d). GitHub branch and owned remote branch/worktree deleted; no laptop branch/worktree allocated. Pod lease released and pod deleted08:19:16UTC; paid uptime3464s at$1.19/h ≈$1.145. Main iCloud archive166871040bytes MD5dd3af2e779b0ea55ad2eb9a80ba9b2f3; closeout51200bytes MD523b228138ae0d610374b9fbc3c5457ea; both whole downloads verified and six-column indices reread. Cumulative ledger charged4170000ms for complete cycle07:12–08:21:30, now1475947627ms of1481400028ms. Ongoing hygiene cloud-index scan and subsequent source review charge to next cycle, with no gap/reset. Hygiene preserved other labs active worktrees; tool still running. Pivot6095599251 and next cycle card6095631309 posted. Standing goal continues.

### comment 6095647175 — 2026-10-10T08:23:56Z — Casey-allard

Hygiene closeout supplement: scripts/storage/uor-hygiene --apply completed exit0, about0GB reclaimed, free24GiB. Other labs active/recent or unarchived material preserved. Its remaining elapsed time is charged in the next continuous cycle as declared. Current cycle delivery, source cleanup and preservation are complete.


---
_Comment/review ids in this file: 6095577235, 6095586878, 6095633656, 6095647175_

