# #2068 — Record fix for #2067: name the copy source's response-token count and both test counts (docs only)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-09T18:19:45Z
- **Updated:** 2026-10-09T18:23:31Z
- **Closed:** 2026-10-09T18:22:22Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/2068
- **Merged:** 2026-10-09T18:22:22Z
- **Merge commit:** 388e30731a74eb60523d210b590a49d3516f7c58
- **Base / Head:** main / UOR-Foundation:answer-span-record-fix
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Two docs-only corrections to the merged answer-span record (#2067, `483e69037`). No code, no numbers change.

1. **Factual.** The record said `dcopy` is "57,229 response tokens". 57,229 is the source's *token* count; its response (mask=1) tokens are **10,757** (as `mixed-cp`'s own file entry states: `tokens: 57229, response_tokens: 10757`). Now reads "57,229 tokens, of which 10,757 response tokens".
2. **Clarity.** The debug-underflow item quoted "811 passed, 3 failed" — that is the instrument tree, which carried this round's two extra tests. The delivered branch, which does not carry them, gives **809 passed / 3 failed**. The record now names both counts.

Only `docs/labs/answer-span-supervision-2026-10-09/README.md` changes; the archived patch, the INDEX row, the current-state entry and the live `dialogue_episodes.rs` guard are untouched. References #2029.

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

