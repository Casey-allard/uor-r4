# #1247 — Add addressed-attention SGD and preserve failed 64-update learning pilot

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-09-13T07:27:52Z
- **Updated:** 2026-10-09T13:17:05Z
- **Closed:** 2026-09-13T07:29:15Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/1247
- **Merged:** 2026-09-13T07:29:15Z
- **Merge commit:** f366a95602e2ac11e3660c8b4e32705b989bb691
- **Base / Head:** main / UOR-Foundation:codex/addressed-attention-learning-pilot
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

The complete addressed-attention path lacked an optimizer/checkpoint caller and a tested learning result. Add offline four-particle full-trajectory SGD, bitwise source/data/config-bound parameter checkpoints, frozen Rust memory/coding fixtures and deterministic generated-response evaluation with three scoped controls. Existing initialized artifact/serving implementations are preserved; trained state is explicitly a checkpoint plus compiled witness, with no retained-model promotion.

**Actual model gate: FAIL_ADDRESSED_ATTENTION_LEARNING_PILOT.** One fixed64-update run completes in17.126809s. Development response CE5.5530255→5.5516114; correctness stays1/154 with one formerly correct symbol lost.0/8development and0/16construction responses are exact; no generated coding output qualifies for execution. Parameters and876compiled bytes change, and checkpoint/export interpreter parity passes. Preserve witness96a0faff and both sealed attempts. No second fit or changed acceptance.

Local validation:37focused release tests pass; the separately invoked fit and evaluation drivers pass execution checks. Formatting, claim wording, source/evidence/link checks pass. Original checkouts, retained15baec48, prior initialized implementation and22prior seals/740files verify unchanged. Build/fit/evaluation charge132806/240000ms. Necessary parent allowance+160000ms was recorded before use; shared ceiling unchanged. No paid compute or cleanup. Queue statuses acknowledge compatibility and execute no tests.

Result/evidence and current-state/canonical plan now distinguish the implemented learner from failed behavioral acceptance. Next is one zero-update saved-endpoint gradient/context-stability diagnostic before another training dose or architectural change.

References #973. References #964. References #820.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

