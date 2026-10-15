# #2138 — Scheduled lab audit: every lab, Claude included, against the session-goal contract (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-10T03:25:21Z
- **Updated:** 2026-10-10T03:29:06Z
- **Closed:** 2026-10-10T03:26:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2138
- **Merged:** 2026-10-10T03:26:39Z
- **Merge commit:** ff7836e144cea44eecd783213d1c45cf03f99ea9
- **Base / Head:** main / UOR-Foundation:claude-lab-audit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner request: put every lab's track, including the Claude lab's, through a recurring peer review.

`docs/labs/audit.md` defines the audit:
- **When and who:** every 3 h, a fresh-context run checks DeepSeek, Codex and Claude against the session-goal contract and D21. The checks are status cards, the line/count field, the three-negatives limit, the preparation-PR limit, the decision test, the owner direction, pre-registration and delivery.
- **Output:** one card on #2028, and one comment per breach on the lab's milestone issue. The auditor changes nothing.
- **Independence:** Claude-lab PRs get a non-Claude second opinion (DeepSeek through OpenCode), or are marked self-audited.

The audit runs as a scheduled task on the owner's machine.

References #2028

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6093280939 — 2026-10-10T03:25:25Z — Casey-allard

Exact-head self-review at `f5341a502a7e662f75719752e9a195fc2870c39e`: docs only; origin/main merged in; claim-wording gate passes.


---
_Comment/review ids in this file: 6093280939_

