# #2146 — Autonomous goal prompts for all three labs: continuous cycles, MAY / MAY NOT, a finish line (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T07:16:30Z
- **Updated:** 2026-10-10T07:17:46Z
- **Closed:** 2026-10-10T07:17:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2146
- **Merged:** 2026-10-10T07:17:36Z
- **Merge commit:** 8421e605f677175957ee3a268439bf8d62bfa311
- **Base / Head:** main / UOR-Foundation:claude/autonomous-goals
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: lab governance (owner request, not a milestone line) · count n/a · headline unchanged

## What and why

The owner asked for autonomous goal prompts so all three labs keep working under the same cadence and know exactly what they may and may not do.

The DeepSeek, Codex and Claude prompts in `docs/labs/session-goal.md` now each contain:
- **Owned milestones** and the **standing direction**, with today's headline and count.
- **The cycle:** claim and status card, pre-register, at most two preparation PRs, one result PR with KEEP/REJECT and a record, the count with a pivot card at 3/3, then the next cycle.
- **Every-PR rules:** the delivery cadence and the check-in cadence.
- **YOU MAY / YOU MAY NOT lists**, and when to ask the owner.
- **Audit duties.**
- **A FINISH LINE** met only by the milestone's acceptance on the saved model, an owner `STOP <LAB>` on #2028, or blocked cards on every owned milestone. A result PR or a pivot card ends a cycle, never the goal.

Contract items 2, 9 and the preparation limit now count per cycle instead of per session.

## Checklist
- [x] current-state entry with **Next:** (the owner pastes the prompts with `/goal`)
- [x] `check_claim_wording.py` passes
- [x] Docs only

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6095032463 — 2026-10-10T07:16:37Z — Casey-allard

Exact-head self-review at `f958748ad3930b8864d66451c7d2fa38511410ab`: docs only, based on fresh origin/main; the claim-wording gate passes.


---
_Comment/review ids in this file: 6095032463_

