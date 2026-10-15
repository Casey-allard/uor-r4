# #1619 — Add mix-chat-corpus and heldout-memory-panel

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T14:28:14Z
- **Updated:** 2026-10-02T14:39:18Z
- **Closed:** 2026-10-02T14:30:32Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1619
- **Merged:** 2026-10-02T14:30:32Z
- **Merge commit:** 5311e6632f6aa11209f39f1f89265b5b7454cc24
- **Base / Head:** main / UOR-Foundation:codex/mix-chat-corpus-20261002
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What and why

Also adds `heldout-memory-panel`: an evaluation panel whose relation ids, value vocabularies, question forms and statement forms are **all disjoint** from the synthetic generator's training tables, so it can separate a learned binding from memorised templates.

Used on the emitter fit it showed the difference matters — parent **0/40**, fitted **1/40** on disjoint relations, against 10/10 on panels sharing the training templates. That result is reported on #1512 and #820 as a scoped negative; this PR carries the instrument, not the finding.

Full description in the first comment below.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

