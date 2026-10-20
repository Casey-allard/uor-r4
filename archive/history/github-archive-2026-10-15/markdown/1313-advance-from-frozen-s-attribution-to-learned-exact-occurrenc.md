# #1313 — Advance from frozen S attribution to learned exact-occurrence access

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T19:00:39Z
- **Updated:** 2026-10-09T13:17:10Z
- **Closed:** 2026-09-20T19:01:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1313
- **Merged:** 2026-09-20T19:01:52Z
- **Merge commit:** 1dae322ce7770673c0046597c10f4361564cd3a0
- **Base / Head:** main / UOR-Foundation:codex/attribution-next-review
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The frozen S attribution supports keeping the local query contribution and moving to query-selectable exact occurrences. It does not establish that length dependence explains the mean-history comparator, and its reported submillisecond timings used precomputed residuals and a large parent-logit cache rather than direct serving.

This review preserves the verified numerical result, documents the scoped estimator/API/provenance defects, and selects one learned BPE occurrence reader over frozen E plus the S-query-only residual. The comprehensive DeepSeek handoff allows reasoned implementation and evaluation choices, requires actual generated binding behavior and direct serving cost, and limits continuation work to the trainer actually used. Roadmap, current state, evidence navigation and first-party READMEs now reflect that decision; original records remain preserved.

The owner-authorized storage cleanup removed regenerable build/install/application caches and recovered 23.21 decimal GB, leaving 51.17 GB free at completion. All model/research/session data, worktrees and release binaries were preserved. No model training, forward evaluation or Rust build ran in this review; the shared model ledger is unchanged.

Validation: three independent source/evidence/math reviews; saved-vector statistics and sealed-set/artifact identities verified; preserved attribution executable hash checked after cleanup; claim-wording gate, relative-document links and git diff checks pass. Historical CI names are queue acknowledgements, not model tests.

References #973, #820, #963, #964.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

