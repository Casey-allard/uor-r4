# #1248 — Measure addressed-attention gradient and context stability

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T15:46:02Z
- **Updated:** 2026-10-09T13:17:06Z
- **Closed:** 2026-09-13T15:47:02Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1248
- **Merged:** 2026-09-13T15:47:02Z
- **Merge commit:** 809922244ee5b874101763a2d9156960ee88b7c7
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-stability
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The 64-update pilot failed generated-answer acceptance. This adds a frozen-parameter diagnostic to distinguish gradient repeatability from stochastic/deterministic context differences without changing the learner, checkpoints or serving path.

Executed 32 four-particle batches on two preselected construction records at initial/final endpoints. EMIT-context overlap with compiled execution is 3/1440, 4/1440, 6/1792 and 7/1792. Gate pairwise gradient cosine is near zero; sampled endpoint response-loss changes are inconclusive. These are descriptive findings, not proof of zero gradient or stochastic success lost at export. The prior learning gate stays failed; retained15baec48 remains unchanged.

Validation:43 focused release tests pass, plus separate bounded diagnostic execution; fmt and claim wording pass. Exact batch-gradient/CE parity, causal target isolation and covariance fixtures are exercised. One compile error was corrected and its cost preserved. Total build/test/diagnostic charge121,732/180,000ms; no fit, update, paid compute or cleanup. Source-bound evidence records the sealed attempt and cumulative resources. Existing artifacts and original dirty worktrees are preserved.

Next: implement and compare exact causal cost-to-go with suffix-aligned independent LOO baselines before another fit. No model promotion or broader capability claim.

References #973. References #964. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

