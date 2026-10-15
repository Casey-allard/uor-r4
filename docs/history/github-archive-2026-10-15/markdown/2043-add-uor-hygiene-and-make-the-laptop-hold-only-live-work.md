# #2043 — Add uor-hygiene and make the laptop hold only live work

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:16:34Z
- **Updated:** 2026-10-09T15:18:30Z
- **Closed:** 2026-10-09T15:18:16Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2043
- **Merged:** 2026-10-09T15:18:15Z
- **Merge commit:** b6bd7336c6c9a95a6bfa0c087b71d96b9e01be64
- **Base / Head:** main / UOR-Foundation:claude/storage-hygiene
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Owner direction (9 October 2026): `origin/main` holds the source and iCloud holds the results; the laptop keeps only live work.

- **`scripts/storage/uor-hygiene`** is a dry run by default; `--apply` acts. It removes:
  - worktrees whose HEAD is already in `origin/main` and that have no dirty or untracked files, together with their local branch;
  - Rust build folders (`~/.cache/uor-*`, worktree `target/`) idle for 24 hours and not in use by a running cargo/rustc;
  - `~/uor-r4-local/<dir>` already in the `cloud-store` index with an upload newer than its newest file (moved to the Trash);
  - then it runs `git gc --auto` and `uv cache prune`.

  It never touches the main checkout, dirty worktrees, recent writes, `~/.local/share/uor-r4`, or folders not yet in iCloud, and it never empties the Trash.
- **AGENTS.md:** the storage bullet now says to move results to iCloud as soon as they're used, keep local-only data under gitignored paths, and run uor-hygiene after every merge or below 30 GB free.
- **.gitignore:** adds `/local/`, `/.worktrees/`, `/.claude/worktrees/` and `.token-optimizer/` (none of them have tracked files).

**Checks:** `bash -n` clean. A dry run on the lab laptop correctly kept DeepSeek's active worktree, the live runner `workspace`, folders written today and a folder not yet in iCloud, and listed only iCloud-backed folders for the Trash. `check_claim_wording.py` passes.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6083774376 — 2026-10-09T15:16:41Z — Casey-allard

Exact-head self-review at `c81abb85fcf6e047780b7d055c75ae565a19cab8`: read the script's guards (dirty, ancestor-of-main, busy, idle via `find -mmin`, iCloud index time vs newest file); `bash -n` clean; dry run on the lab laptop behaved as described; claim wording passes.


---
_Comment/review ids in this file: 6083774376_

