# #1929 — Emit the response mask: 71.1% of the chat corpus is response region (References #820)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-08T16:03:10Z
- **Updated:** 2026-10-09T13:21:07Z
- **Closed:** 2026-10-08T16:04:39Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1929
- **Merged:** 2026-10-08T16:04:39Z
- **Merge commit:** eff935c279f2b458798bee55d0519d9fab969298
- **Base / Head:** main / UOR-Foundation:deepseek/mask-emitted
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

mask-scan now writes the mask: 82,248,461 bytes on chat-v0-p2/train, one per token, 58,492,458 response tokens (71.1 percent), marking the span after each Assistant: marker until the next role marker. Higher than the median-gap estimate and the better number, because these corpora pair short prompts with long replies. The verification chain is complete - markers found, rule implemented and run, mask exists. Remaining: transfer beside the pod corpus, check dialogue-train's train_manifest requirement, run it, score the two frozen instruments. References #820

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

