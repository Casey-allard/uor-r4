# #1324 — Retain contextual utility and guide matched geometric relation learning

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-21T03:50:50Z
- **Updated:** 2026-10-09T13:17:12Z
- **Closed:** 2026-09-21T03:52:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1324
- **Merged:** 2026-09-21T03:52:15Z
- **Merge commit:** 472767dc7cfacba3a3b0622da5fb2f4221d3ddd5
- **Base / Head:** main / UOR-Foundation:codex/contextual-utility-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

PR #1323 delivers a useful contextual controller, but its residual-loss diagnostic measures the global arm, conflates NoRead with ranking and undercounts duplicate correct payloads. This review preserves its independently reproduced primary gain and exposes the final-query improvement (8→35 correct emitted tokens on 119 present queries), while correcting text overlap, control scope, artifact binding and cost/accounting claims.

The next task is matched learning of the existing signed H4 relation, with scorer/descriptor/refit alternation, a categorical+contextual comparator and real reader-level document holdout. An antipodal expressivity witness shows the current fixture does not yet justify a larger pair table. The five-stage roadmap retains structural persistence, dependent composition, broader language/executed Rust and qualified efficiency, with Hopf/S7/E8/harmonic mechanisms tied to demonstrated needs.

Validation: independently reconstructed the exact primary sequence-cluster interval; all 23 retained manifest files match size/BLAKE3 with zero unlisted files; independent source and mathematical review; claim-wording gate, JSON parse, 833 local documentation links and git diff --check pass. Documentation and saved-data analysis only: no Rust build, model fit or inference was run for this PR. Historical results and sealed artifacts are preserved. Storage inventory found 43.06 GB free; no deletion or model-time debit.

References #820, #973, #1139, #962, #963, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

