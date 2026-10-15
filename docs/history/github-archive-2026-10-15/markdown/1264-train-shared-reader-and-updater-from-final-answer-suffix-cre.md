# #1264 — Train shared reader and updater from final-answer suffix credit

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-14T03:05:25Z
- **Updated:** 2026-10-09T13:17:56Z
- **Closed:** 2026-09-14T03:06:33Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1264
- **Merged:** 2026-09-14T03:06:33Z
- **Merge commit:** 64b67de148193638530bad7402edfffccab4b3b3
- **Base / Head:** main / UOR-Foundation:codex/shared-language-output-credit
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Reader compatibility and query updating were frozen inside the shared language loop. This adds a bounded Rust learner that clears those rules, bootstraps direct answers, then trains both operators from final-answer suffix re-execution under the existing learned scheduler and writer. Whole-span reader probes and update probes must actually fire; changed selections recompute future routing, substitution and output. Oracle source paths and intermediates do not enter credit, and output-compatible shortcuts remain in the training set.

Actual gate: **PASS_SHARED_LANGUAGE_OUTPUT_CREDIT**. Four fixed blocks preserve training success (0 → 2048 → 4096 → 4096 → 4096), with 6144/6144 mixed reader sites covered. Development answers/paths1536/1536,96complete families, all prior controls including19968previous scheduling rows retained. The final reader/updater masks equal the prior artifact and bootstrap: no incremental mixed-credit accuracy advantage is demonstrated. Independent holdout NOT_RUN; no normal-model promotion or general-language claim.

Validation: optimized offline uor-r4-core build;1new+36retained focused tests; actual frozen-artifact fit/control/retention report with25sealed files; reload/state-continuity checks; cargo fmt --all --check; claim wording and git diff --check.47prior sealed roots/1196files, predecessor sources/binary and original dirty checkouts verified unchanged. Historical CI names are compatibility acknowledgements, not tests.

One build/fit,381962ms model charge under2100000ms local projection. Parent allowance extended900000ms before work under standing authorization; shared ceiling unchanged. No paid compute, cleanup or storage extension. Source/artifact/resource lineage and current next action are recorded in the result, evidence and project documents. Next: bounded third-read depth transfer under unchanged fitted operators before refitting.

References #973


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

