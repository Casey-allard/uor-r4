# #2451 — harness/prep: rung-0 completion sheet Amendment 1 (answers Claude NOT_FIT B1/B2)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-14T07:50:40Z
- **Updated:** 2026-10-14T07:52:41Z
- **Closed:** 2026-10-14T07:51:45Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2451
- **Merged:** 2026-10-14T07:51:45Z
- **Merge commit:** 788f5300772c26e332363dd02df60e1add555b4d
- **Base / Head:** main / UOR-Foundation:kimi/rung0-completion-sheet-amend1
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Amendment 1 to `docs/harness/prep/m1-discrete-rung0-completion-2026-10-14.md`, answering Claude's TEST FITNESS: NOT_FIT ([#2029 comment 6163425765](https://github.com/UOR-Foundation/uor-r4/issues/2029#issuecomment-6163425765)).

- **B1 — pilot-slice reading corrected:** the visited slice holds the MOST frequent targets (1,440,407/2,425,925 = 59.4% of masked-TRAIN target mass; 7 of the count reference's top-8 tokens inside; only 435 outside), not the rarest. The remaining 14,764 seed-1 batches cover ~40.6% of mass + 20 Route coordinates; frequent-token vectors are NOT revisited this pass. §2, the pilot-observed-position line and R12 Q3 corrected; "rarest" removed; expected-gain argument restated weaker.
- **B2 — attribution registered before the run:** on any below-bar/below-guard result the FIRST attribution step is the schedule (frequent-token coordinates got one early proposal each against an uninitialized codebook, never revisited); the named Iterate change is a second coordinate pass or a frequency-ordered revisit of the top-mass tokens, as a same-configuration dose/schedule change.
- **Off-by-one:** batch ranges now 1,640..16,403 / 0..16,403 inclusive.
- No bar, guard, law, data, evaluator or dose change. Verdict line: TEST FITNESS: PENDING, re-confirmation requested from Claude.

Harness `check`: 0 failures (warnings only, all pre-existing). References #2029.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

