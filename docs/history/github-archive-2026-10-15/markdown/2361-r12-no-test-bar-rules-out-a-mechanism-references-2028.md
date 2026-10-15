# #2361 — R12: no test bar rules out a mechanism (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T19:57:38Z
- **Updated:** 2026-10-13T20:00:55Z
- **Closed:** 2026-10-13T19:59:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2361
- **Merged:** 2026-10-13T19:59:43Z
- **Merge commit:** 76bb988a1dc199c2950de8f3d760c5c71f214875
- **Base / Head:** main / UOR-Foundation:kimi/pm-rule-r12-test-authority-20261013
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction, 13 Oct 2026: arbitrary test values must never acquire veto power over the project.

Adds **R12** to docs/harness/pm-rules.md:
- No check, tolerance or acceptance bar rules out, closes or redirects a mechanism or work line — failure meaning is decided by full-context analysis, never by the bar.
- Before a check has authority, the sheet author and the TEST FITNESS reviewer must answer four questions on the sheet: what it measures; whether it is computable at the current mechanism stage; whether the instrument can beat the bar in principle (an unbeatable bar = invalid test, not failed mechanism); whether the test should run at this stage at all.
- Failure attribution order: instrument → fixture → bar → mechanism, each with evidence. Precision failures are instrument repairs (e.g. F64 oracle grading an F32 secant; production path untouched), never mechanism edits.
- Build-first: construction checks verify construction (exact laws, causality, refusals); performance bars apply only after a mechanism exists and is wired. No tuning gate blocks a construction gate.

Motivating case: murmuration control 11 (murm_backward_secants) stopped a 33-control qualification line on a 0.002 relative-error bar an F32 finite-difference secant at h=1e-3 may be unable to beat in principle.

Docs-only, one file, no harness/model surface. PM rules are PM-delivered per R11.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

