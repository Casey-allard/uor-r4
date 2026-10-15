# #2044 — uor-hygiene: treat a worktree as merged when its HEAD is a merged PR's head

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:23:34Z
- **Updated:** 2026-10-09T15:25:24Z
- **Closed:** 2026-10-09T15:25:14Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2044
- **Merged:** 2026-10-09T15:25:14Z
- **Merge commit:** bede6da6d9414c4f554511c954f42fb55e188bfb
- **Base / Head:** main / UOR-Foundation:claude/hygiene-squash
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix for #2043. Main uses squash merges, so a merged branch's HEAD is never an ancestor of `origin/main`, and the first `--apply` skipped exactly the worktrees it was written to remove. A worktree now counts as merged on ancestry **or** an exact match with a merged PR's `headRefOid` (via `gh`). It is still kept when dirty or busy.

Test: a scratch worktree on the merged `claude/storage-hygiene` branch (head = #2043's merged head) is now listed as "would remove"; a dirty worktree is still kept. `bash -n` clean.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6083895882 — 2026-10-09T15:23:46Z — Casey-allard

Exact-head self-review at `b85b7d028b107e241572d9424e07395e31ca16e8`: 5-line guard change; tested on a real merged-branch worktree (listed for removal) and a dirty one (kept); `bash -n` clean.


---
_Comment/review ids in this file: 6083895882_

