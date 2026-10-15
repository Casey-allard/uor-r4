# #2137 — Session-goal contract + paste-in prompts for the labs; PR template gains Line/count/headline (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T03:19:08Z
- **Updated:** 2026-10-10T03:21:27Z
- **Closed:** 2026-10-10T03:21:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2137
- **Merged:** 2026-10-10T03:21:05Z
- **Merge commit:** e74865637baf2f1f56c696ad392644710b208a26
- **Base / Head:** main / UOR-Foundation:claude-session-goal
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The owner asked for a goal prompt to paste into each lab at session start, so that both labs stay under the project's rules. On 9 October both worked alone for about seven hours, and neither milestone moved (D21).

**This PR adds:**
- **`docs/labs/session-goal.md`:** the contract plus two ready prompts, one for DeepSeek (M1/M4) and one for Codex (M2/M3). Each session:
  - ends in a result, a pivot card or a blocked card;
  - allows at most 2 preparation PRs, each naming its run;
  - posts a status card every 90 min or every 3 PRs.

  It also adds the "would my next step differ?" test before every PR, and lists the loopholes seen on 9 October.
- **PR template:** a `Line · count n/3 · headline before → after` field.
- **AGENTS.md:** a one-line pointer, plus a current-state entry.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093237304 — 2026-10-10T03:19:38Z — Casey-allard

Exact-head self-review at `52971dcd18aa29aae2679f17f399c983ea0cd337`: docs and PR template only; origin/main merged in (no conflict); claim-wording gate passes. Adds the Claude-lab prompt and the wind-down prompt the owner asked for.


---
_Comment/review ids in this file: 6093237304_

