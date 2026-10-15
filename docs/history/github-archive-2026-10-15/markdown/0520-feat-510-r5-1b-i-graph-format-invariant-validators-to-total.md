# #520 — feat(#510): R5 1b(i) — graph-format invariant validators to total form

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T01:20:38Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-09T02:57:44Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/520
- **Merged:** 2026-08-09T02:57:44Z
- **Merge commit:** c5b4c905385c654a694030bd3ba51eca753a1b53
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t1b-invariants
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues the R5 march (validators/audits become total — no Result over an unsanctioned error). GraphInvariantOwnershipMatrix's two validators go total: `validate_graph_structure` → `Option<InvariantValidationError>` (None = every invariant holds; the enum is kept as report data), and `validate_score_fixed_point` → `Option<i16>`. The stage2 caller already folds a violation into `FormatError::InvariantViolation` → `NotAProduct`, so it just switches Err→Some. graph-format audit-limits **12→10**, total **333→331**. std+no_std build, tests pass. More of 1b (scoring_semantics, inference_contract, r4g1) follows. Refs #510.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

