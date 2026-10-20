# #285 — feat(server): thread the HF BPE tokenizer through serving-side teacher prompting (#254)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-07-29T17:55:27Z
- **Updated:** 2026-10-09T13:09:12Z
- **Closed:** 2026-07-29T18:06:46Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/285
- **Merged:** 2026-07-29T18:06:46Z
- **Merge commit:** 06bc93b6d062c6b7badcac306e4a0fb983c2babc
- **Base / Head:** main / UOR-Foundation:issue-254-serving-bpe
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Per the plan on #254 (comment 5120731140): serving-side seed encode + generation decode move into the teacher's own id space via HfBpeTokenizer loaded from the oracle source dir, with the legacy tless pair as the explicit local-llama fallback and the tokenizer address logged for attestation lineage. #253 lineage (observation-path fix measured agreement 2.6%→23.1%). Rebased on post-#283 main. Gates: tests 71 suites ok, clippy -D warnings clean, fmt clean.

Closes #254.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_017tu2Y4AGZvcgM4vmoSz5x6

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

