# #753 — docs(research): record #745 investigation progress

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T01:55:54Z
- **Updated:** 2026-08-17T08:42:09Z
- **Closed:** 2026-08-17T02:17:30Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/753
- **Merged:** 2026-08-17T02:17:30Z
- **Merge commit:** ee249b91c94445f571dec77b27211c10e9e4146b
- **Base / Head:** main / UOR-Foundation:issue-745-degenerate-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

Documents current state of the #745 degenerate-generation investigation in `docs/RESEARCH.md`. No code changes — this is a docs-only PR recording what was ruled out and what's next, per the repo's own research-log discipline.

- A live prompt-encoding bug was found and fixed experimentally, then empirically confirmed **not** to be #745's cause (re-running with the fix applied produced byte-identical word-salad). Filed as its own issue, #751, since it's a real bug worth fixing on its own merits.
- A suspected cross-vocabulary mismatch (the recurring token ids resembled well-known Llama/SentencePiece fixture values) was directly checked and ruled out: the compiled artifact's own declared vocab size (read from its `TLA5` header: 49152) matches its paired tokenizer's vocab size exactly, and decoding the exact predicted token ids through that tokenizer reproduces the observed garbage precisely — these are native SmolLM2 subword pieces, not a foreign vocabulary bleeding through.
- Resolution-depth witness tracing on the live prediction path shows generation resolves at real context depths (1-4) with high evidence counts, not context-independent unigram fallback (depth 0) — ruling the root cause into the observe/corpus/training side rather than live serving. Filed as a scoped continuation, #752.
- Also updates #744's status row (PR #749 landed) and adds #750/#751/#752 rows.

References #745, #744, #750, #751, #752

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

