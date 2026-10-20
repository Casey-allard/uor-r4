# #2247 — Optimize CellTree::route: drop the O(k) duplicate scan (References #2029)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-12T07:39:00Z
- **Updated:** 2026-10-13T04:16:37Z
- **Closed:** 2026-10-12T07:40:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2247
- **Merged:** 2026-10-12T07:40:02Z
- **Merge commit:** 48ec58b7726e65c567580a34efc9da0114b73c71
- **Base / Head:** main / UOR-Foundation:deepseek/invariant-router
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

CellTree::route walked the query path plus children, and for every candidate did an O(k) linear contains scan over the already-written prefix. The path and children have no duplicate cells, so each candidate matches at most one cell and the scan is dead work: it made the routed read host-side tree walk O(order x window x k) per row instead of O(order x window). Removed the scan. The 21 stack_address tests pass unchanged. This unblocks the nine fits, whose first arm ran about 12x slower than the pre-registered budget assumed.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

