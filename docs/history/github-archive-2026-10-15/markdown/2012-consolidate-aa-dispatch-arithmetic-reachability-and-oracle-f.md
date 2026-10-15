# #2012 — Consolidate aa-dispatch: arithmetic reachability and oracle findings (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:05:36Z
- **Updated:** 2026-10-09T14:17:01Z
- **Closed:** 2026-10-09T13:07:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2012
- **Merged:** 2026-10-09T13:07:22Z
- **Merge commit:** 44963d6124c2cce7ab8dcbd0e2db9e7da00d7aff
- **Base / Head:** main / UOR-Foundation:deepseek/aa-dispatch
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction, 9 October 2026: origin/main is the shared source of truth and each deliverable must be
finished and verified through a protected merge before its successor starts. This branch carried
validated work that was never merged into main because its delivery stopped at the branch; it is
consolidated here with the status recorded rather than silently dropped.

Branch: deepseek/aa-dispatch
Commits whose patches are not present in main (patch-id comparison against origin/main):
  1b13d1d92 2026-10-05 addressed_attention: arithmetic reachability probe — the zero was the legal mask, not the objective
  c83c25e57 2026-10-05 addressed_attention: arithmetic oracle — the operator computes exactly; the emission head cannot say it
Scope: the branch's own changes since its merge base.

Method note: this was identified by comparing PATCH IDS (git cherry), not ancestry. Squash merges make a
branch tip a non-ancestor of main even when its content is fully merged, so an ancestry check reports
140 of the 146 deepseek branches as unmerged when only 6 genuinely are. Content diffs are misleading for
the same reason, since branches are cut from an older main. The patch-id comparison is what separates
work that would be lost from work that is already on main.

References #820


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

