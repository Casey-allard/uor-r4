# #2015 — Unify branch work into main: archive 60 Claude branches and make main the single source of truth

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T13:06:52Z
- **Updated:** 2026-10-09T13:09:19Z
- **Closed:** 2026-10-09T13:07:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2015
- **Merged:** 2026-10-09T13:07:39Z
- **Merge commit:** aa5129137afc86057dea68628b6e19451975e81e
- **Base / Head:** main / UOR-Foundation:claude/unify-main
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction (9 October 2026): `origin/main` is the single source of truth. Every branch's work must be in main; branches are deleted once their PR merges.

**What this does**
- Adds `docs/history/branch-archive/` holding one patch per Claude-authored branch (60 of them) whose changes were not in main, each `git diff <merge-base> <tip>` of that branch, plus [INDEX.md](docs/history/branch-archive/INDEX.md) listing tip, merge-base, date and how many added lines are absent from main. Vendored Python libraries (`*/pylib/*`) are excluded from the patches.
- Rewrites the delivery rule in `AGENTS.md` and `docs/labs/protocol.md`: a branch is only the transport for one PR and is deleted after merge; negative or superseded work lands in main as an archive patch; never start new work while completed work is unmerged.

**How "not in main" was decided** (2,724 refs inventoried): a branch counts as landed if its tip is the head of a merged PR, it is an ancestor of main, its changed files are identical to main, or its patch reverse-applies on main. Claude owns 412 branches; 352 are landed and will be deleted without archiving; the 60 others are archived here and deleted after this merges.

**Checks:** documentation and patch files only; no Rust changes.

References #820

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6081496250 — 2026-10-09T13:07:04Z — Casey-allard

Exact-head self-review at `50e58c9d5d3291a15924b9874861bb84147a9194` (owner rule of 30 September): docs and archive patches only. Verified: the 60 patch files match `git diff <merge-base> <tip>` for the branches listed in INDEX.md; AGENTS.md and protocol.md edits read as intended; no Rust touched, so no compile/test applies.


---
_Comment/review ids in this file: 6081496250_

