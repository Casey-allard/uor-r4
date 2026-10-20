# #2024 — Archive 64 stale branches from other labs into the branch archive

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T14:14:20Z
- **Updated:** 2026-10-09T14:17:39Z
- **Closed:** 2026-10-09T14:16:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2024
- **Merged:** 2026-10-09T14:16:15Z
- **Merge commit:** d7a872813f61e347f9eb0c9a09280e32204edce8
- **Base / Head:** main / UOR-Foundation:claude/archive-other-labs
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner-approved (9 October) cleanup pass over the other labs' branches, same method as #2015/#2017.

- **Skipped:** branches with an open PR, a commit in the last 48 hours, or a checked-out worktree; `codex/lab-state`; `gh-pages` (the live Pages source).
- **Archived here:** 64 stale branches whose changes are not in main, as `git diff <merge-base> <tip>` patches under `docs/history/branch-archive/other-labs-20261009/`, with a new INDEX section (tip, merge-base, author, date).
- **Deleted after merge (not in this diff):** 514 branch refs whose work is already in main (merged-PR head, ancestor, identical files, or patch already applied).

Docs/patch files only; no Rust changes.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6082689402 — 2026-10-09T14:14:26Z — Casey-allard

Exact-head self-review at `0fc86df5f0a3451ef19cefdb07d04a6c6c8bd97f`: 64 patch files plus an INDEX section; each equals `git diff <merge-base> <tip>` for the listed ref. gh-pages and codex/lab-state excluded. No Rust touched.


---
_Comment/review ids in this file: 6082689402_

