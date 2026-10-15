# #279 — fix(certify): wall-clock budget for the R4G1 C-stage — silent multi-hour stall becomes a recorded skip (#278)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T14:21:47Z
- **Updated:** 2026-10-09T13:09:08Z
- **Closed:** 2026-07-29T14:59:05Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/279
- **Merged:** 2026-07-29T14:59:04Z
- **Merge commit:** 4fcb85752405fa507378f75f2692bc0d02d4fb77
- **Base / Head:** main / UOR-Foundation:issue-278-certify-c-budget
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #278 (stays open for the perf fix that makes the C row measurable again).

**Problem.** The C-stage (R4G1 graph eval) routes every position through `predict_distribution`, whose per-position cost on current artifact sizes makes the stage computationally infeasible: an observed run spent **15h05m at 100% CPU with zero output** after the "B bit-prefix" row — indistinguishable from a deadlock from the outside. Stack sample (8182 samples): ~94% of time in `check_node_emits` → `GraphView::edge` → per-access `decode_entry`/`decode_edge`. Two working sessions were lost to environment theories before sampling. Full diagnosis in #278.

**Change (certifier-side only, no runtime-kernel change).**
- `r4g1_eval_readiness_within`: budgeted readiness probe (default 120s, `R4_CERTIFY_R4G1_BUDGET_SECS`), new explicit `BudgetExceeded` outcome printed in the same absent-measurement style as the #232 skips.
- Full evaluation: progress line every 256 positions + its own budget (default 600s, `R4_CERTIFY_R4G1_EVAL_BUDGET_SECS`); on budget, partial counts are discarded and the row records the skip. Budget checks run between positions; one in-flight prediction bounds the overshoot.
- Tests: zero-budget classifies as `BudgetExceeded` before any position; generous-budget classification matches the unbudgeted wrapper.

**Verified end-to-end** on the D3 artifacts (2026-07-29, 13 min wall-clock vs. never):
```
C R4G1 readiness probe: 64 positions, wall-clock budget 120s (R4_CERTIFY_R4G1_BUDGET_SECS overrides)
progress: C R4G1 eval 256/30036 (0%, 402s)
C R4G1 (graph path): SKIPPED — evaluation exceeded its 600s wall-clock budget after 340/30036 positions (600s elapsed; R4_CERTIFY_R4G1_EVAL_BUDGET_SECS overrides). Partial counts discarded; no measurement recorded. Perf: issue #278.
CERTIFY_EXIT:0
```
A/B rows, op census, equality witness, and compression witnesses all produced normally. Gates: workspace tests, clippy `-D warnings`, fmt, claim-wording — all green.

**Unblocking note.** certify no longer gates the queue on an infeasible row: measurement-driven decisions (#244 et al.) can proceed on the A/B rows while #278's index/caching fix restores the C measurement.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_011ZjgQf41T8t2HETiw3T1dK


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

