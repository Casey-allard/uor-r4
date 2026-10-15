# #1720 — mqar-bench decide: count only full-budget seeds and refuse mixed configs (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-05T05:33:50Z
- **Updated:** 2026-10-09T13:08:50Z
- **Closed:** 2026-10-05T05:35:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1720
- **Merged:** 2026-10-05T05:35:33Z
- **Merge commit:** 86e85291f1aa4d2a2946fae0cf9148a922fb5ee9
- **Base / Head:** main / UOR-Foundation:codex/step2-decide-full-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

This is the follow-up to #1716, which merged at `fd82c6f7` before its adversarial reviewer's blocking item was fixed ([review](https://github.com/UOR-Foundation/uor-r4/pull/1716#issuecomment-5988660033)).

**The defect:** `mode=decide` counted any root marked `complete` as a full seed. A run that stopped early at `MAX_SECONDS`, or one trained at a different step budget (such as the calibration root), could therefore set the frozen R1–R3/R6 verdicts.

**The fix:**
- A root is skipped when it reports `stopped_early_at_max_seconds`, or when `steps_completed` differs from the planned steps. The counts are recorded in `skipped`.
- Every counted seed must share one training-configuration fingerprint: the task settings and the fact layout, minus the seed and execution environment. A mixed grid is an error.

**Checks at this head:**
- `cargo test --release --example mqar-bench` passes 13/13, and decide.rs passes rustfmt.
- On the author's smoke grid, `rararr-none-s1` (282/400 steps) is now excluded, and the remaining roots share a config.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5988725590 — 2026-10-05T05:33:52Z — Casey-allard

Exact-head self-review at 6c26e0c51c13658a3f9771564ffd96ad889797b1: a one-file change to an example's decision tool; the tests and the smoke check above ran at this head. Queuing.


---
_Comment/review ids in this file: 5988725590_

