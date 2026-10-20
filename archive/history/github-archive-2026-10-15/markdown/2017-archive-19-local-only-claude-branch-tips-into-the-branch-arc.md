# #2017 — Archive 19 local-only Claude branch tips into the branch archive

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:10:28Z
- **Updated:** 2026-10-09T13:12:55Z
- **Closed:** 2026-10-09T13:12:09Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2017
- **Merged:** 2026-10-09T13:12:09Z
- **Merge commit:** 961e33b0b7eb4f66c172e51db6bad53455d29a43
- **Base / Head:** main / UOR-Foundation:claude/unify-main-2
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #2015. These 19 local branch copies had commits that differed from their (now deleted, archived) GitHub branches. Each is added as `<name>.local.patch` (`git diff <merge-base> <local tip>`) with a new INDEX section, so the local branches can be deleted without losing anything.

Docs/patch files only; no Rust changes.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081552903 — 2026-10-09T13:10:35Z — Casey-allard

Exact-head self-review at `36f58999f4e30dcf3456dd3e65133c388b188e10`: 19 patch files plus an INDEX section; each patch equals `git diff <merge-base> <local tip>` for the listed branch. No Rust touched.


---
_Comment/review ids in this file: 6081552903_

