# #2491 — PM hygiene: clear stale tracked branch claims (References #2028)

- **Type:** Pull request
- **State:** open
- **Author:** @Casey-allard
- **Created:** 2026-10-15T00:24:35Z
- **Updated:** 2026-10-15T00:24:35Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2491
- **Merged:** not merged
- **Merge commit:** e4dfcbcde748bd3ad7923941c8e5fa98742527f0
- **Base / Head:** main / UOR-Foundation:kimi/claims-cleanup-20261015
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Tracked `docs/harness/claims.json` still listed `claude/completion-pins` and `claude/spiralcore-attention-review-20261014` — both delivered and merged (seed-1 pins family; #2482 review at `618e422ce`) with their remote branches deleted, so the claims pointed at nothing live. Clears both entries; live-tier claims unaffected; Claude claims fresh branches on its next deliveries. Docs-only (claims.json + current-state entry), no compute.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

