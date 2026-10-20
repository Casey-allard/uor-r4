# #2142 — Session goals: a FINISH LINE clause on GitHub for every lab prompt (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T04:47:42Z
- **Updated:** 2026-10-10T05:02:48Z
- **Closed:** 2026-10-10T04:48:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2142
- **Merged:** 2026-10-10T04:48:44Z
- **Merge commit:** f01e91230553ebfb3f17c1bb96a8526d98902c2d
- **Base / Head:** main / UOR-Foundation:claude/goal-finish-line
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: lab governance (owner correction, not a milestone line) · count n/a · headline unchanged

## What and why

The owner found that the Claude and Codex labs' `/goal` hooks cleared early. The prompts in `docs/labs/session-goal.md` described a way of working but named no end state, so a goal checker could judge them met at any turn end.

Contract item 9 and all three prompts now end with a `FINISH LINE` clause. A goal is met only by:
- a merged result PR on the lab's line, cited in a status card on the milestone issue; or
- a pivot or blocked card naming the exact blocker.

Status cards, plans, preparation PRs and summaries never meet it. Decision test: if the hooks held without this, the owner would not need it; they did not hold, twice.

## Checklist
- [x] Record: current-state entry with **Next:** (owner re-pastes the prompts with `/goal`)
- [x] `check_claim_wording.py` passes
- [x] Docs only

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093911700 — 2026-10-10T04:47:49Z — Casey-allard

Exact-head self-review at `6789c17e00e5d42e5932b82a7bc7aa8508167c04`: docs only, based on fresh origin/main; the claim-wording gate passes.


---
_Comment/review ids in this file: 6093911700_

