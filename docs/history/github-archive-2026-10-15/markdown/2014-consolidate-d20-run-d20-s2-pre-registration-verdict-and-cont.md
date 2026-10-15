# #2014 — Consolidate d20-run: D20 s2 pre-registration, verdict and controls (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:05:40Z
- **Updated:** 2026-10-09T14:18:17Z
- **Closed:** 2026-10-09T13:23:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2014
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:deepseek/d20-run
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction, 9 October 2026: origin/main is the shared source of truth and each deliverable must be
finished and verified through a protected merge before its successor starts. This branch carried
validated work that was never merged into main because its delivery stopped at the branch; it is
consolidated here with the status recorded rather than silently dropped.

Branch: deepseek/d20-run
Commits whose patches are not present in main (patch-id comparison against origin/main):
  5a37b9814 2026-10-06 mqar-bench: ordinary parameter-matched control arm (D20 s2 conditions 1 and 3)
  a4c37aa3b 2026-10-06 D20 s2: pre-registration, run report, and the two holes in the battery
  ece47fc7a 2026-10-06 D20 s2: complete the pre-registered 3x3 -- verdict INCONCLUSIVE
  d5172e7f6 2026-10-07 Preserve the D20 matched-control run products
Scope: the branch's own changes since its merge base.

Method note: this was identified by comparing PATCH IDS (git cherry), not ancestry. Squash merges make a
branch tip a non-ancestor of main even when its content is fully merged, so an ancestry check reports
140 of the 146 deepseek branches as unmerged when only 6 genuinely are. Content diffs are misleading for
the same reason, since branches are cut from an older main. The patch-id comparison is what separates
work that would be lost from work that is already on main.

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081777136 — 2026-10-09T13:23:36Z — Casey-allard

Superseded by #2019, which archives this branch (and the other five holding unmerged work) as an indexed patch bundle under docs/archive/deepseek-branches-20261009/ with a full commit index. This PR became DIRTY against main after the other consolidation merges landed (#2012, #2015-#2018), and its content is preserved verbatim in the archive, so it is closed rather than rebased: nothing is lost, and the single serialized required-transport queue is not spent re-landing work that is already captured. Owner direction, 9 October 2026: retained work lands as indexed source patches on main; branches are temporary transport, not alternate authorities. References #820


---
_Comment/review ids in this file: 6081777136_

