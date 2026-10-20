# #1927 — Verify the response mask is extractable: literal role markers found (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T15:59:18Z
- **Updated:** 2026-10-09T13:21:16Z
- **Closed:** 2026-10-08T16:00:59Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1927
- **Merged:** 2026-10-08T16:00:59Z
- **Merge commit:** 3a5330a39495cef7483bd2aa5dfe78369368975a
- **Base / Head:** main / UOR-Foundation:deepseek/mask-verified
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Wrote and ran mask-scan locally on the chat corpus. Assistant: [35,560,652,714,28] occurs 129,472 times and User: [55,2728,28] 129,728 times in the 82,248,461-token training stream, with median gaps of 548 and 526 tokens; heldout shows 10,251 and 10,264. The markers are present and literal, the alternation is the expected single-turn pattern to 0.2 percent, and roughly half the corpus is response region - so the mask is a scan away rather than a rebuild, and a response-region loss has something substantial to weight. The document records what remains undone: no mask emitted, dialogue-train not run with one, its manifest requirement unchecked, and no objective-changed model to score against the two frozen instruments. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

