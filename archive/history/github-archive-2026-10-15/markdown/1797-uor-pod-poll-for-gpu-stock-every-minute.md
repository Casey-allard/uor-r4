# #1797 — uor-pod: poll for GPU stock every minute

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-06T13:37:32Z
- **Updated:** 2026-10-09T13:09:04Z
- **Closed:** 2026-10-06T13:39:11Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1797
- **Merged:** 2026-10-06T13:39:11Z
- **Merge commit:** c9bca09463a5d8fc416e6e36ecde38c0957cfb4a
- **Base / Head:** main / UOR-Foundation:claude/pod-wait-60s
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Labs kept failing to get a 5090 because short-lived \"Low\" stock was missed by the 5-minute --wait poll. Default \`UOR_POD_WAIT_INTERVAL\` 300 → 60 s; message and compute.md updated. The board \"waiting\" post is still made once per wait.

Checks at this head: \`bash -n scripts/pod/uor-pod\` passes; diff is 3 lines.

References #1750.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 6017489074 — 2026-10-06T13:37:45Z — Casey-allard

Self-review (owner rule: exact-head self-review allowed) at f9adae6ebd77bbe296b42deb78f79bd183bc8f12: APPROVE. bash -n passes; only the default poll interval and two messages change.


---
_Comment/review ids in this file: 6017489074_

