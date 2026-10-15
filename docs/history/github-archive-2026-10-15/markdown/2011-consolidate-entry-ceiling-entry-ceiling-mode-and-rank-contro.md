# #2011 — Consolidate entry-ceiling: entry-ceiling mode and rank controls (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:05:33Z
- **Updated:** 2026-10-09T14:18:19Z
- **Closed:** 2026-10-09T13:23:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2011
- **Merged:** not merged
- **Base / Head:** main / UOR-Foundation:deepseek/entry-ceiling
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction, 9 October 2026: origin/main is the shared source of truth and each deliverable must be
finished and verified through a protected merge before its successor starts. This branch carried
validated work that was never merged into main because its delivery stopped at the branch; it is
consolidated here with the status recorded rather than silently dropped.

Branch: deepseek/entry-ceiling
Commits whose patches are not present in main (patch-id comparison against origin/main):
  ebf97e779 2026-10-07 Add a zero-update entry-ceiling mode that removes the Copy channel
  09b0c0a9e 2026-10-07 Measure the prediction-control artifact on the panel it was selected from
  469a0075d 2026-10-07 Report the gold rank in the ordered emission at the entry position
  e56c1fc74 2026-10-07 Add the one-token-prefix rank control to the entry ceiling
Scope: the branch's own changes since its merge base.

Method note: this was identified by comparing PATCH IDS (git cherry), not ancestry. Squash merges make a
branch tip a non-ancestor of main even when its content is fully merged, so an ancestry check reports
140 of the 146 deepseek branches as unmerged when only 6 genuinely are. Content diffs are misleading for
the same reason, since branches are cut from an older main. The patch-id comparison is what separates
work that would be lost from work that is already on main.

References #820


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081775560 — 2026-10-09T13:23:31Z — Casey-allard

Superseded by #2019, which archives this branch (and the other five holding unmerged work) as an indexed patch bundle under docs/archive/deepseek-branches-20261009/ with a full commit index. This PR became DIRTY against main after the other consolidation merges landed (#2012, #2015-#2018), and its content is preserved verbatim in the archive, so it is closed rather than rebased: nothing is lost, and the single serialized required-transport queue is not spent re-landing work that is already captured. Owner direction, 9 October 2026: retained work lands as indexed source patches on main; branches are temporary transport, not alternate authorities. References #820


---
_Comment/review ids in this file: 6081775560_

