# #382 — feat(runtime): #247 — pinned multi-turn fixture, ROUT calibration, session lane admitted as secondary fallback probe

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-03T16:48:48Z
- **Updated:** 2026-10-09T13:09:11Z
- **Closed:** 2026-08-03T17:03:06Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/382
- **Merged:** 2026-08-03T17:03:06Z
- **Merge commit:** f7f786a57ac4e15f23618288a113ddf0b42cff29
- **Base / Head:** main / UOR-Foundation:i247-fixture
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## #247: pinned multi-turn fixture, calibration measurement, and the ROUT admission decision — measured, then flipped conservatively

Closes the remaining DoD item ("evaluation extended so the lane is measured") and takes the decision it was for.

**Fixture:** six pinned multi-turn transcripts evolved through the **shipped** path (`UorR4Router::new` → `index_corpus` → `evolve_state` → `session_signature_from_state`, pinned γ=0.5) — the #344 bench had only ever exercised the `from_tokens` client fallback, never the deployed manifold quantizer. Determinism pinned by double-construction equality, not golden bytes (which would silently invalidate on any deliberate representation change).

**Measurement** (`--rout-calibration` mode on the session bench; declared criterion posted to #247 before the run; fixture bundle, 515 held-out positions):

| | result |
|---|---|
| session sigs within ROUT radius | **24/24** (median dist/radius 0.928; context reference 515/515 at 0.765) |
| bias-lane token changes | 0/515 (teacher agreement identical, 227 = 227; 1 score change) |
| verdict per declared criterion | **ADMIT-CANDIDATE** |

Reading: the fallback prototypes **are** calibrated for session-space inputs, and the bias lane is inert wherever context routing already works — so the lane's entire value lives on fallback traffic, which is exactly where admission acts.

**Decision (conservative):** context signature keeps primacy; the session signature becomes the **secondary** fallback probe, consulted only when the context probe admits nothing within any calibrated radius — positions that previously routed via the nearest out-of-radius prototype or fell to Novel. `rout_probe` extracted (byte-identical scan) so both lanes share one implementation. The bias-only contract test is replaced by `session_signature_enters_rout_fallback_as_secondary_probe` (context primacy + deterministic session-only fallback), per the #315 guard's own "until calibrated" wording.

Runtime kernel contract untouched (XOR/AND/popcount/compare). Gates from cleaned targets: fmt, clippy `--all-targets --all-features`, full workspace tests.

Closes #247. Refs #315, #344, #350, #374.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

https://claude.ai/code/session_018UJdEQFdckSK2et9G44jsm


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

