# #219 — fix(router): Cascade to Transformerless Fallback on R4G1 Pathological Text Rejection

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-26T05:29:33Z
- **Updated:** 2026-10-09T14:17:02Z
- **Closed:** 2026-07-26T05:39:03Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/219
- **Merged:** 2026-07-26T05:39:03Z
- **Merge commit:** c03961a6541726ea60dac48c92bb61a62784fd8f
- **Base / Head:** main / UOR-Foundation:fix/r4g1-pathological-fallback-routing
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Fixes issue where R4G1 pathological/non-readable generation rejection failed to trigger transformerless secondary fallback and surfaced HTTP 503 errors. Updates handle_chat_request in src/server.rs to execute FallbackRouter secondary generation whenever primary r4g1 text is empty and not a declared D4 policy abstention.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

