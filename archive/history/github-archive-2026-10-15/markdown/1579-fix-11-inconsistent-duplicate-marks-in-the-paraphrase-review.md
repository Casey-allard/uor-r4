# #1579 — Fix 11 inconsistent duplicate marks in the paraphrase review decisions

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T16:32:26Z
- **Updated:** 2026-10-09T13:08:46Z
- **Closed:** 2026-10-01T17:21:42Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1579
- **Merged:** 2026-10-01T17:21:42Z
- **Merge commit:** e784a5436bb8bd9b9c4e7161aa1c5e4e245bf7e2
- **Base / Head:** main / UOR-Foundation:codex/paraphrase-review-fix
- **Reviews / inline review comments:** 1 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #1552. Data-only.

`paraphrase-review` refused #1573's `decisions.tsv`. Exact-text duplicates were recomputed against the marks:
- **8 dropped rows and `p1:45` (kept)** are exact repeats of earlier rows. They are now `duplicate`.
- **`p2:364` and `p2:388`** named a later copy as their first occurrence. They now name the first.
- **`p2:280`** ('The number that brings me good fortune is {v}.') is not a duplicate; it is a clean lucky-number statement and is now `keep`.

**New counts:** 372 keep, 22 relabel, 253 drop, 129 duplicate. Still 394 output rows.

**Check:** `paraphrase-review-27adc006` built the derivative from this exact file without error (card v21 on #1552).

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (2 entries; chronological: conversation comments, reviews, review comments)

### review 5382385565 — 2026-10-01T16:32:28Z — Casey-allard — COMMENTED

Exact-head self-review at `fed2a7b31342225341e5c3a07ebe82c3cbcadb42` (data-only): the full consistency check passes (every duplicate mark equals the first identical row), and `paraphrase-review` accepted the file and wrote 394 rows.

### comment 5938985197 — 2026-10-01T19:29:18Z — Casey-allard

Merged as `e784a543`. The squash tree equals `git merge-tree --write-tree` of the parent `eb999998` and the reviewed head `fed2a7b3`.


---
_Comment/review ids in this file: 5938985197, 5382385565_

