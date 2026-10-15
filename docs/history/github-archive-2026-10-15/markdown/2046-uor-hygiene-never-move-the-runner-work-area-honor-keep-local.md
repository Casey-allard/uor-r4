# #2046 — uor-hygiene: never move the runner work area; honor .keep-local

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T15:34:38Z
- **Updated:** 2026-10-09T15:37:28Z
- **Closed:** 2026-10-09T15:37:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2046
- **Merged:** 2026-10-09T15:37:17Z
- **Merge commit:** eb608879abf85629eefba183a2c5586a9bf3e4d9
- **Base / Head:** main / UOR-Foundation:claude/hygiene-keep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fix after the second `--apply`. That run moved `~/uor-r4-local/workspace` (the runner's shared work area, 6.6 GB) to the Trash once its newest file dropped below its iCloud upload time; it was restored immediately. The script now skips `workspace` by name and any folder holding a `.keep-local` marker. Dry run with the idle window set to 0 lists both workspace folders as kept.

References #2037

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6084079950 — 2026-10-09T15:34:46Z — Casey-allard

Exact-head self-review at `8e5f32058aefa1dce915c52827e3777ad3e35a5e`: two-line guard; dry run with idle window 0 no longer lists `workspace`; `bash -n` clean.


---
_Comment/review ids in this file: 6084079950_

