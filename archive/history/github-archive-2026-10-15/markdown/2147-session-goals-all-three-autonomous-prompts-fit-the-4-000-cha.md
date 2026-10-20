# #2147 — Session goals: all three autonomous prompts fit the 4,000-character /goal limit (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T07:31:01Z
- **Updated:** 2026-10-10T07:32:18Z
- **Closed:** 2026-10-10T07:32:03Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2147
- **Merged:** 2026-10-10T07:32:03Z
- **Merge commit:** 53c8804a64399bbf757abb2f008c2806d9d48c1c
- **Base / Head:** main / UOR-Foundation:claude/goal-compact
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline (D21, docs/labs/session-goal.md)

Line: lab governance (owner request) · count n/a · headline unchanged

## What and why

`/goal` refuses a condition over 4,000 characters, and the #2146 prompts were about 4,500. All three are compacted with the same rules, cycle, MAY / MAY NOT lists and finish line: DeepSeek 3,309, Codex 3,149, Claude 3,351 characters. The Claude version is the one the owner has active. The intro notes the limit.

## Checklist
- [x] `check_claim_wording.py` passes
- [x] Docs only; no current-state change (same content, shorter)

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6095140645 — 2026-10-10T07:31:04Z — Casey-allard

Exact-head self-review at `05853d6aad905f88886dff1e951c9ce19a23ae56`: docs only, based on fresh origin/main; the claim-wording gate passes.


---
_Comment/review ids in this file: 6095140645_

