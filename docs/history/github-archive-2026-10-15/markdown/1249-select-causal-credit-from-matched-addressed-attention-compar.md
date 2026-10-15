# #1249 — Select causal credit from matched addressed-attention comparison

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T16:23:15Z
- **Updated:** 2026-10-09T13:17:04Z
- **Closed:** 2026-09-13T16:24:17Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1249
- **Merged:** 2026-09-13T16:24:17Z
- **Merge commit:** 500bcef4bfdf29b48b1113b1baea03b4ada10f22
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-causal-credit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The failed addressed-attention pilot used whole-sequence loss to credit every sampled event. This adds a separately source-bound causal suffix-LOO comparator: each event receives only downstream loss, centered against the other particles at the same loss boundary. The existing learner, checkpoints, sampling policy and serving implementation are preserved.

One frozen comparison executes32 four-particle batches from the saved initial/final checkpoints. Both estimators use the same paths. Historical traces/counts/losses and old-estimator statistics match. Gate-gradient covariance falls47.4%; all four gate cells and every other family improve. The predeclared engineering gate selects causal credit for one subsequent matched learning pilot. The previous generated-behavior gate remains failed; no parameters change and no model is promoted.

Validation:47 focused release tests pass, plus the separate frozen comparison. Tests cover exact old/new gradients/direct/path parity, exhaustive causal expectation and real publication/cancellation future credit. Formatting, claim wording and source/evidence/preservation checks pass. Build/test/comparison charge121,760/180,000ms; comparison8.534075s internally, tape32,001,856bytes. No second runtime replay, new fit, paid compute or cleanup.

References #973. References #964. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

