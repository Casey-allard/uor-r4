# #1581 — Learn capture and retention for variable-gap geometric binding

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-01T18:02:27Z
- **Updated:** 2026-10-09T13:17:18Z
- **Closed:** 2026-10-01T19:28:13Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1581
- **Merged:** 2026-10-01T19:28:13Z
- **Merge commit:** 115c74c958e8b848c9e2de7afa02e19a882fba9a
- **Base / Head:** main / UOR-Foundation:codex/geometric-attention-latch-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The native geometric read loses query-conditioned key binding across independently varied write/query gaps. This adds opt-in learned capture and retained identity alongside the current role/payload input, preserving the Lorentz read, values, NoRead and full causal candidate support. Matched Local controls have identical parameters and initialization. Capture and occurrence labels are auxiliary training targets; ordinary prediction uses learned gates and token inputs only.

Twelve 320-update Rust fits separate ordinary answer credit, direct occurrence credit, and added capture-event credit. The first eight remain query-insensitive. With capture credit, both Held seeds answer 128/128 original and 128/128 longer-gap rows and select the correct occurrence with head-zero majority throughout. Identity ablation drops original answers to 45/128 and 49/128; Local controls score 47/128 and54/128 and fail the answer-changing query pairs. Panels have32 correlated base groups; these are development diagnostics on an authored one-token-key grammar, not language or geometric-superiority qualification. Binding necessity with capture-only training is untested.

Default behavior/format stays unchanged. Versioned sidecars bind mode/config/weights; incompatible serving/export/grid/checkpoint paths refuse this floating-point research mode. The shared gate-logit accessor reuses stable weighted cross-entropy for capture credit. Successful soft artifacts and all negatives are retained; the next attention task is a hard/relaxed capture bridge, then scale-aware integer lowering, learned spans and bounded access.

Validation: six focused Rust tests passed (recurrence/gradients/causality, initialization/exclusivity, reload metadata, boundary refusal, stable capture logits); actual saved/reloaded models exercised every arm. Exact reload logits verified on the fixed first episode only. Touched-file rustfmt, diff and claim-wording checks passed. Existing dead-code warnings remain. Fit/evaluation/save/reload total2389.285s, two threads, maximum RSS52.1MB, within2700s cumulative allowance. No paid or SSD compute. PR/merge-group compatibility acknowledgements do not execute these checks.

Full report: docs/integration/geometric-attention-latch-2026-10-01.md. Compact source/executable/model evidence: docs/evidence/attention-latch-2026-10-01.json. Current state and canonical plan updated in their owning sections.

References #1512. Its broader language/serving acceptance remains open.


## Timeline (1 entries; chronological: conversation comments, reviews, review comments)

### comment 5938942705 — 2026-10-01T19:26:37Z — Casey-allard

Exact-head review and executed checks for `7df1d9808d37c072432f2b741e4c76f41b36138e` (base `a74f36860645b173224a5a96c33c9484d5b148e3`, tree `b6ddd97fca0d602d116474dc1f89fc95c0d6da50`).

Principal engineer authored/reviewed the latch and getter; principal mathematician independently reviewed mechanism/objectives and saved rows; independent evidence reviewer approved all counts/artifact bindings at07d6ee72. Root reviews the sole final documentation delta: it clarifies verified50/76 observed inputs versus60/100 grammar ceilings. No model code changed after the fitted sourcee2748351. Six focused tests completed again on this exact head:6passed,0failed, compiler/check exit0. Actual model source/executable hashes and12saved/reloaded fits are recorded in docs/evidence/attention-latch-2026-10-01.json; source blobs match the final code. Touched-file rustfmt, diff and claim-wording checks passed. Scope remains explicitly supervised finite-grammar retention/binding; no language, integer, geometry-superiority or binding-necessity promotion.

Under D19, the legacy delivery-evidence automation is advisory and is not a substitute for these actual executed checks. Protected delivery uses the required compatibility statuses and merge queue; no status is fabricated and no protection is bypassed.



---
_Comment/review ids in this file: 5938942705_

