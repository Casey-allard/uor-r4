# #904 — #897 follow-up: diagnose the LOWERING-FIDELITY GAP (41/87) before deciding a fix

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T19:01:38Z
- **Updated:** 2026-08-21T19:51:26Z
- **Closed:** 2026-08-21T19:51:26Z
- **Labels:** graph-compiler, conformance
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/904
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

Follow-up to #897 (merged in #903, `1b84770e`). The deployed-fidelity spot-check
(`skipmix_lane_spotcheck_897.rs`, `docs/skipmix_lane_897_spotcheck.md`) recorded
**LOWERING-FIDELITY GAP**: the deployed 1-token skip-mix lane followed 41/87
(47.1%) of the phase-0-confirmed reference favorable pairs, below the
predeclared 53/87 (60%) bar. Servability was 100% (174/174) — this is not the
#886 emission-path confound; the lane is fully wired via the real release
emitter and simply doesn't track its own reference ceiling yet.

Per an explicit sign-off request, three fix-or-abandon options were presented;
**"diagnose the gap first"** was chosen: isolate the cause(s) with a cheap,
off-serving measurement before committing to any fix, before leaving the lane
dormant permanently, and before removing the code.

## Correction to the #897 record

`docs/skipmix_lane_897_spotcheck.md` attributed the gap partly to "the per-key
top-64 cap." Re-reading the harness: the reference `Tables` construction and
the deployed `skipmix_fit::fit_skipmix_tables` both cap at the identical
`CAP = DEFAULT_TOP_K = 64`. **The cap is not a differentiator** and should not
be treated as a suspect; this issue supersedes that attribution.

## Two real candidate causes (identified by code reading, not yet measured)

1. **Candidate-list breadth.** `predict_decision_candidates_with_skipmix`
   (`uor-r4-api/src/engine.rs`) re-ranks strictly within `candidates.ranked()`,
   which is capped at `StepCandidates::STEP_TOP_CANDIDATES = 8`
   (`uor-r4-graph-certify/src/score_runtime.rs:1898`). The reference arm's
   `skipmix_scores` is evaluated over `d1_cands`, a suffix ∪ content ∪
   joint-widened union up to ~97 tokens
   (`CAND_SUFFIX=CAND_CONTENT=CAND_JOINT=32`). If the reference-favored
   teacher target isn't among the engine's own top-8 base candidates, no
   re-rank can ever recover it — a structural ceiling independent of scoring.
2. **Units/scale mismatch.** The skip-mix contribution stored in SKMX/PSIB
   rows is `segment_fit::quantize_rate(count, total)` — a **linear rate**
   scaled by `2^20` (`RATE_SCALE_SHIFT = 20`). It is `saturating_add`-ed
   directly onto `ScoreQ.raw()`, a **log-probability** scaled by `2^16`
   (`ScoreQ`'s Q16.16 format) — `segment_argmax`'s
   `score.raw().saturating_add(contribution(token))`
   (`uor-r4-api/src/engine.rs:597`). `skipmix_scale_by_lambda_and_support` is
   currently the identity function — no calibration step reconciles the two
   scales. A high-rate contribution (up to ~1,048,576) can swamp typical base
   score gaps, or a low-rate one can vanish against them, depending on
   magnitude — plausibly the "served-but-noisy" pattern (46/87 pairs served,
   not following).

## Diagnostic plan (off-serving, no production code changes)

Reusing the already-fitted/emitted/loaded real bundle from the #897 spot-check
(same corpus, same fit, same real emitter, same engine), replay the 87
favorable pairs with added instrumentation, decomposed into three arms:

1. **Candidate coverage ceiling** — for each pair, is the teacher target
   present at all in the engine's real `candidates.ranked()` (top-8) list on
   each side? The max any re-ranking policy bound to this candidate list
   could ever achieve.
2. **Reference math restricted to the narrow (real, top-8) candidate list** —
   the validated, unquantized, wide-candidate reference formula, evaluated
   only over each position's actual top-8 tokens. Isolates the pure
   candidate-breadth effect, independent of quantization.
3. **Real base score + a unit-safe (non-additive) combine rule over the same
   narrow candidate list** — rank candidates by (has skip-mix support,
   contribution magnitude, base `ScoreQ`, token id) instead of summing raw
   `quantize_rate` output onto `ScoreQ.raw()`. Isolates whether the additive
   scale-mismatch, specifically, is costing follow-throughs among reachable
   candidates.

No fit changes, no engine changes, no serving-default changes — purely
measurement, gated behind `--ignored`, mirroring the #897/#886 precedent.
Result to be written to `docs/` and reported back before any fix is chosen.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

