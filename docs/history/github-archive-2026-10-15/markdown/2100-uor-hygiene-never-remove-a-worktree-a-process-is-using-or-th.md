# #2100 — uor-hygiene: never remove a worktree a process is using or that holds gitignored local data

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:20:24Z
- **Updated:** 2026-10-09T22:24:04Z
- **Closed:** 2026-10-09T22:21:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2100
- **Merged:** 2026-10-09T22:21:30Z
- **Merge commit:** 682397aafcbd43279fe2ef0647b7cbb4d7e19ac5
- **Base / Head:** main / UOR-Foundation:claude-hygiene-busy
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

**Bug:** a clean, merged worktree that held a live local training run, with its data under the gitignored `local/`, was removed by `uor-hygiene --apply` (run by a lab). Two gaps let it through:
- the busy check only recognised `cargo`/`rustc`;
- `git status` doesn't list gitignored files, and `local/` is the documented place for local-only data.

**Fix:**
- **Busy:** any process with an open file or working directory inside the path (`lsof -t +D`, excluding the script itself) keeps it.
- **Ignored data:** any gitignored content other than `target/` keeps the worktree, with the message "move it to iCloud first".

**Test:** two scratch worktrees at a main commit, one with a process running inside it, one with a file in `local/`. The dry run keeps both, with the right reasons. `bash -n` is clean.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6090208895 — 2026-10-09T22:20:25Z — Casey-allard

Exact-head self-review at `757e7cdb2163916a4fcca9acd3176d4288527b7b`: busy/ignored guards tested with two scratch worktrees (both kept); bash -n clean.


---
_Comment/review ids in this file: 6090208895_

