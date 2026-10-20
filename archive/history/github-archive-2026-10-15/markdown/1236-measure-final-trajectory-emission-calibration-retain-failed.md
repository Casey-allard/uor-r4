# #1236 — Measure final-trajectory emission calibration; retain failed candidate

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T02:06:58Z
- **Updated:** 2026-10-09T13:17:24Z
- **Closed:** 2026-09-13T02:08:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1236
- **Merged:** 2026-09-13T02:08:03Z
- **Merge commit:** d4de76659ffd9627c1516cca27a4c48b52b973ea
- **Base / Head:** main / UOR-Foundation:codex/geometric-final-emission
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The previous exact emission calibration preceded changes to state/routing. This adds a bounded Rust test driver that recalibrates the final ade9a1cf trajectories once, saves and reloads the actual artifact, and compares every row across construction/opened data and Full/ContextDisabled/StateDisabled controls.

**Actual result: FAIL_FINAL_EMISSION_DEVELOPMENT_GATE.** Construction improves 79→81/672 and opened development 28→33/381, but 35 previously correct rows become wrong. All four Full continuations remain incoherent with no EOS within 96 bytes. Preserve rejected 0f82b728; retain 15baec48. Production source is unchanged. The current-state record now points to a bounded learned angular decision-tree hypothesis, with limits and gates to freeze before implementation.

Validation: release build and focused accounting test, the single actual sealed calibration/behavior run, actual-candidate 512-step zero-allocation census, formatting, claim wording, local links and exact source/evidence preservation. Eleven prior sealed roots / 93 files and 22 parked dirty paths verify unchanged. The allocation binary is reused from the source-identical production checkpoint. These are execution checks; they do not turn the failed model gate into PASS.

Resource charge: 108,848/300,000 ms model/build/test; cumulative ledger 121,612,556/132,950,000 ms. Added storage 26,083,328 bytes within 128 MiB. No allowance extension, paid compute, cleanup, fresh draw or promotion.

References #973 and #964. Programme #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

