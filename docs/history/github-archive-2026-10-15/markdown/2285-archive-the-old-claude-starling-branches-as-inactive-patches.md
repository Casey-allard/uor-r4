# #2285 — Archive the old Claude starling branches as inactive patches (References #2028)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-13T04:18:21Z
- **Updated:** 2026-10-13T04:19:42Z
- **Closed:** 2026-10-13T04:19:23Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2285
- **Merged:** 2026-10-13T04:19:23Z
- **Merge commit:** 749005527e281fbfb95444ee75309dd14ccf2a85
- **Base / Head:** main / UOR-Foundation:claude/archive-starling-branches-2026-10-13
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Line and headline

Line: housekeeping (branch hygiene) · no count · headline unchanged (M1 v5 memory 20/40, reply 37/232). Owner direction 13 Oct: no branch sprawl — merge finished work, delete its branch.

## What

`claude/starling-language` (2 commits) and `claude/starling-confirm` (3 commits) were never merged. They carry starling training aids built on a softmax-over-kept gradient surrogate, selector KL from a dense teacher and a dense-bootstrap knob — prohibited by D23 — so they are **archived, not merged**: two patches in `docs/history/branch-archive/` (sha256 in the INDEX row) plus the INDEX entry. Both remote branches are deleted once this lands. No source or test file changes.

Also done (no PR needed): `deepseek/invariant-router` and `deepseek/handoff-doc` were deleted from GitHub after checking each tip equals its merged PR head (#2247, #2237); the local `claude/starling-gpu` branch has no commits outside main and its worktree's untracked research data goes to iCloud (`cloud-store`) before removal.

## Checks at head `f1bb29a0137489017646eac166c9ca27c370f7b3`

`check_no_softmax` 1649/1649 (patch files are not executable source) · `check_no_matmul` 93/93 · claim wording pass · `harness check` rc 0. Patches reproduce with `git diff 503a6d0b5 7bdc5393e` and `git diff 503a6d0b5 b6ab5af2d`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

