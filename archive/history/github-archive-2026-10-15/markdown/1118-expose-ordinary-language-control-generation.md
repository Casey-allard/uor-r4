# #1118 — Expose ordinary language control generation

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-04T15:42:17Z
- **Updated:** 2026-09-04T15:44:50Z
- **Closed:** 2026-09-04T15:43:35Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1118
- **Merged:** 2026-09-04T15:43:35Z
- **Merge commit:** 3269021a9f4639c325bd1294edb9725a2c5e9251
- **Base / Head:** main / UOR-Foundation:codex/973-ordinary-generation
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The current retained-language iterations could not distinguish a broken retained read from a generally incapable compact language recipe. This adds a fixed `generate-ordinary-language-path` CLI path that loads the already-trained ordinary causal-softmax control, verifies its artifact and completed arm record, and recomputes the exact full causal prefix for each generated token. It reads no corpus, teacher, geometry, checkpoint, provider, or fit input.

Under the same seed-9738, top-k-40, temperature-0.8, 16-token comparison used by the retained variants:

- `A purple turtle found a clock in the garden` continued `, there was a time, there was a little girl named It found a big`.
- `Albert Einstein was born in` continued ` his friend, a time, there was a little girl named he put it with`.

The ordinary control is prompt-dependent and materially more grammatical than the broken retained outputs, while remaining generic and factually unreliable. That selects a concrete next implementation: stop group-retained attention micro-edits and expose the existing position-preserving chronological K/V implementation with real H4 content transport against this same artifact before attempting recurrent compression or a larger fit.

Validation:

- Both fixed CLI generations completed once from artifact `blake3:c1cd34b36c7df7c53915785a608ccd353a11de56eebb3ecc58e74092cb5d1933`, with 16 full-prefix forwards and zero forbidden reads.
- `python3 scripts/check_claim_wording.py`
- `git diff --check`
- Independent source/diff review: no actionable findings.

This is an offline architectural control. It does not establish factual recall, general language quality, transformerless serving, or retained/geometric equivalence.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

