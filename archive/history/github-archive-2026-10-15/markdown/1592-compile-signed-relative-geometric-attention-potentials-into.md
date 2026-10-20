# #1592 — Compile signed-relative geometric attention potentials into integer tables

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-10-02T01:24:36Z
- **Updated:** 2026-10-09T13:17:28Z
- **Closed:** 2026-10-02T01:26:08Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1592
- **Merged:** 2026-10-02T01:26:08Z
- **Merge commit:** 86188622a90fb6fea0a1d89867c545688f880069
- **Base / Head:** main / UOR-Foundation:codex/geometric-potential-native-20261001
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The direct geometric reader still evaluated its seven learned potential families with floating-point arithmetic. This change compiles the actual saved coefficients into signed Q24 tables addressed by directed signed-H4 relatives, radial bins and presence states; the scoring kernel uses integer table reads and additions. The existing predicted span controller, code producer and surrounding reader remain explicitly floating point.

The source-bound compiler independently rebuilds table bytes on reload and refuses resealed tampering or stale live weights. The saved-model replay retains all 512 answer decisions, both heads’ source-majority decisions and top-ranked occurrence/NoRead choices, with no score-bound violations or training updates. Prior errors remain preserved. Rounding changes floating-point logits; bit equality is not the acceptance criterion.

Validation: 11 focused integer/compiler/stack/causality/evaluator checks, formatting, diff and claim-wording checks passed. Release AArch64 emitted scoring body has no multiply/divide/float arithmetic; defensive panic callees and the full model are not qualified. Execution source `c69e496502641098588056b1a27dfcfb0600cf99`, replay198.612s, RSS157,728,768B. Independent nonauthor source and raw-evidence review is recorded with the report. Offline donor-weight recompilation remains supported; donor reasoning transfer is not established. Next obligation is learned native geometric event control, with bounded access and values/output still open.

References #1512


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

