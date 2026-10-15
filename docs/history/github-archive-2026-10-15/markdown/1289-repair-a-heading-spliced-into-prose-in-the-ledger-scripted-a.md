# #1289 — Repair a heading spliced into prose in the ledger (scripted-anchor defect)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-19T20:40:17Z
- **Updated:** 2026-09-19T20:42:56Z
- **Closed:** 2026-09-19T20:41:19Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1289
- **Merged:** 2026-09-19T20:41:19Z
- **Merge commit:** 52a485aa703aeb30db2f277c310a37c1d8ac05da
- **Base / Head:** main / UOR-Foundation:codex/fix-ledger-mangle
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

A script this session anchored a text edit on the literal string `## Not done`, which occurs **inside a sentence** in this file. The replacement spliced the new section heading into that paragraph, truncating it and removing the intended heading. Restored both, and recorded the lesson in place — this is the same class of error that damaged documents three times this session. Scanned the other five documents edited this session for the pattern (a heading marker away from a line start) and for duplicate headings: all clean. Documentation only; no model time spent.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

