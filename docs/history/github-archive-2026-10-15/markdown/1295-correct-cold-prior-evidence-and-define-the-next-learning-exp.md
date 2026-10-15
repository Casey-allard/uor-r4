# #1295 — Correct cold-prior evidence and define the next learning experiment

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-20T01:12:41Z
- **Updated:** 2026-10-09T13:17:49Z
- **Closed:** 2026-09-20T01:13:48Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1295
- **Merged:** 2026-09-20T01:13:48Z
- **Merge commit:** 9852b8aa93cbc338524d913865e330d90dd92c23
- **Base / Head:** main / UOR-Foundation:codex/prior-learning-recovery
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The #1294 cold-prior pilot reports clipped natural-log losses as bits, trains a floating bias that differs from its exported integer forward, and samples a document-ordered prefix against full-fit references. Those findings preserve the negative point-estimate directions but do not establish undertraining, healthy logit calibration, or modulo aliasing as the cause.

This review defines the next DeepSeek implementation: repair numerical/data contracts, pass a balanced exported contextual-learning fixture, then obtain one prior-only learning curve above a frozen quantized fit-unigram bias. A fixed context-permutation control distinguishes useful context from residual marginal correction; memory remains diagnostic-only. The handoff requires a durable manifest and real resumable checkpoint. README, current state, roadmap, evidence, project map and continuation pointers now agree. Historical receipts and artifacts are preserved.

Validation: independent source/numerical and experimental-design reviews; claim-wording gate; git diff --check; 645 local Markdown links checked with no missing paths (one pre-existing historical file URI preserved). Source unchanged; no Rust build or model execution in this documentation change. Recorded resource balance remains 149038565 / 154400000 ms; the proposed 3600000 ms tranche is not a charge. Queue compatibility statuses are not tests.

References #973, #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

