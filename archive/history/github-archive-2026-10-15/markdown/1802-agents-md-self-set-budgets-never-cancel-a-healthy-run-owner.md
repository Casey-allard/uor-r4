# #1802 — AGENTS.md: self-set budgets never cancel a healthy run (owner direction)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T17:58:23Z
- **Updated:** 2026-10-09T13:09:03Z
- **Closed:** 2026-10-06T18:00:00Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1802
- **Merged:** 2026-10-06T18:00:00Z
- **Merge commit:** 240af5d95c9d31ea602198dbeca44575ada666a1
- **Base / Head:** main / UOR-Foundation:claude/no-midrun-cancel
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction (Casey, 10-06): labs were cancelling runs mid-way because they hit a budget they had set themselves.

- Agent policy bullet: never cancel a healthy run because a self-set budget ran out (`max_seconds`, a step or wall-time estimate, lease `--hours`, planned tokens). Extend, renew and let it finish, and record the extension in one line. The only hard stops: owner caps (≤ 4 pods unless the owner approves more, ≤ $8/h, no new spending class), the disk floor, a failed or diverged run.
- Resources paragraph: self-set budgets are estimates, not stop rules.
- GPU pods: set `--hours` to the whole job plus margin and renew. An expiring lease is never a reason to stop a live job.

Docs only. References #820.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6022283345 — 2026-10-06T17:58:31Z — Casey-allard

Exact-head self-review at `5e3434129a59d0628dcfaa39857dd2d2c9fef524`: docs-only diff to AGENTS.md read in full (3 hunks: agent-policy bullet, resources paragraph, GPU-pod lease line). `python3 scripts/check_claim_wording.py` rc 0. Implements the owner's 10-06 direction verbatim in scope; owner caps unchanged. APPROVE.


---
_Comment/review ids in this file: 6022283345_

