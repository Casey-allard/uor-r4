# #521 — feat(#510): R5 1b(ii+iii) — scoring_semantics + inference_contract to the sanctioned surface

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-09T01:48:16Z
- **Updated:** 2026-10-09T13:08:52Z
- **Closed:** 2026-08-09T02:07:41Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/521
- **Merged:** 2026-08-09T02:07:41Z
- **Merge commit:** 3beb2fd3aa02f0a1b4a2696beef03beb01b6a8da
- **Base / Head:** main / UOR-Foundation:feat/510-r5-t1b-scoring
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

Continues the R5 march. ScoreAccumulator::accumulate → Result<bool, ObservedBound> (the evidence-capacity bound is a measured bound). audit_scoring_compliance → total Option<&str> (observes invariants, never errors; None = all held). ScoringError removed (dead after conversion). proof-model caller switched to if-let-Some. graph-format audit-limits −2; std+no_std build, clippy clean, tests pass. Independent of 1b(i) #520. Refs #510.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

