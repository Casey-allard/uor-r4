# #1235 — Compare full-objective proposals and record exhausted block

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T01:41:18Z
- **Updated:** 2026-10-09T13:17:25Z
- **Closed:** 2026-09-13T01:42:54Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1235
- **Merged:** 2026-09-13T01:42:54Z
- **Merge commit:** 95e213758ce9bb66cc5bdf6f28fbc3fbc4169a71
- **Base / Head:** main / UOR-Foundation:codex/geometric-full-objective
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous two-parameter update improved a diagnostic subset but regressed complete construction. Add a Rust test-only comparison that selects fixed joint proposals and coordinate updates on all 672 construction targets, then checks an exhaustive block reference and row-level controls.

**FAIL_FULL_OBJECTIVE_JOINT_METHOD_SMOKE:** all eight arms retain the parent. None of the 14,400 pairs improves full-objective loss; the parent is the unique measured minimum for this block with all remaining parameters fixed. All 25,272 aligned comparisons and all generations match the parent. No model is promoted, and no production serving, training or artifact code changes.

Validation: final release build and focused witness/replay/row-accounting test; one sealed actual experiment (16,400 search and 14,400 oracle evaluations, plus behavior checks); independent source/result review; formatting and claim-wording checks. Preserve ten prior sealed roots / 70 files, original dirty worktree, retained artifact and all test-only witnesses. Charges include the initial private-method compile error: 120,471/360,000 ms; no paid compute.

Record the warranted next step: one existing exact emission-block recalibration on final ade9 state trajectories. Prior exact calibration preceded the later joint state/routing changes; do not repeat this exhausted sampler block or start an automatic campaign.

References #973 and #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

