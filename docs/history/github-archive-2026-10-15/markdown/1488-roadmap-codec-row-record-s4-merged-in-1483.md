# #1488 — ROADMAP codec row: record S4 merged in #1483

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-29T02:38:31Z
- **Updated:** 2026-10-09T13:17:54Z
- **Closed:** 2026-09-29T03:24:36Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1488
- **Merged:** 2026-09-29T03:24:36Z
- **Merge commit:** 08517e0d96a3573f36885e975305d064c890eda4
- **Base / Head:** main / UOR-Foundation:codex/roadmap-s4-merged
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #973, per Lab 1 steward queue (comment 5882545339), task 1 follow-up. One-line documentation change: the codec row now states the trained-in S4 snap is merged (#1483) and its adoption is set by the pre-registered fit. No code, no claim change beyond what #1483's merge established. Claude reviews.

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5882909674 — 2026-09-29T03:12:11Z — Casey-allard

**Lab 1 review of `fd009a34`: two required fixes, then approve.**

1. **Conflict (`DIRTY`).** #1486 (merged `a10d71ec`) rewrote this same codec row to read "post-hoc 2I transport snap (+0.026 on `geometric_s1`, +0.022 on S2) is not adopted. The trained-in snap (S4) is in #1483, **not merged**". Merge `main` in (no rebase), keep #1486's wording, and change only "in #1483, **not merged**" to "**merged** in #1483 (`18525d07`)".
2. **The link `[pre-registered fit](#)` is a placeholder.** Point it at https://github.com/UOR-Foundation/uor-r4/issues/973#issuecomment-5882537988.

With both fixes in, this is approved at the new head, and Kimi may deliver it by the usual procedure: recheck the exact head, queue, verify the squash diff, notify.


---
_Comment/review ids in this file: 5882909674_

