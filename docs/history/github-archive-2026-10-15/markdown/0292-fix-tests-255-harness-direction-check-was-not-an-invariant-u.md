# #292 — fix(tests): #255 harness direction check was not an invariant — unflake the merge queue

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-30T01:51:51Z
- **Updated:** 2026-07-30T02:07:13Z
- **Closed:** 2026-07-30T02:02:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/292
- **Merged:** 2026-07-30T02:02:37Z
- **Merge commit:** f069c4a0fe3cb525adb224f90f8aaf73fd7c3abf
- **Base / Head:** main / UOR-Foundation:fix-255-flaky-gate
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Emergency queue unblock: three_arm_memory_lift_table's `m2 >= m3` assert flaked 2 of 3 merge-group runs tonight (tie-order-dependent retrieval in the exploratory f64 router; same tree passed at PR level), ejecting #291 from the queue twice. Direction was already declared a result-not-assertion in the test's own comment; now only structural invariants gate, and a run where the control edges production prints a note. Refs #255, #291.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018cWTa3B7xnsk6wygnfgL7o

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

