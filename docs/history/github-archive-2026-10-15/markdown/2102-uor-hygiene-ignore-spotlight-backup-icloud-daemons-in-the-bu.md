# #2102 — uor-hygiene: ignore Spotlight/backup/iCloud daemons in the busy check

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T22:24:24Z
- **Updated:** 2026-10-09T22:40:13Z
- **Closed:** 2026-10-09T22:25:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2102
- **Merged:** 2026-10-09T22:25:30Z
- **Merge commit:** 64c596636a22e06c64bd2fe65396f88735716d16
- **Base / Head:** main / UOR-Foundation:claude-hygiene-indexers
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Follow-up to #2100. `lsof +D` also sees macOS indexers (`mdworker`, `mds`), backup and iCloud daemons briefly reading files. Counting them as busy would keep worktrees indefinitely, so these system processes are now skipped. Real work (shells, training, copies) still counts. `bash -n` is clean.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6090255876 — 2026-10-09T22:24:26Z — Casey-allard

Exact-head self-review at `11d54f7c1ad2460dd9ae442b39afa1d02fc3787b`: 6-line exclusion; bash -n clean.


---
_Comment/review ids in this file: 6090255876_

