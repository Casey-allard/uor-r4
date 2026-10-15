# #906 — runtime(S1): inject skip-mix/psi-bag-known candidates into the decided list instead of only re-ranking (follow-up to #897/#904)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-21T21:10:20Z
- **Updated:** 2026-08-21T21:58:43Z
- **Closed:** 2026-08-21T21:58:43Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/906
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Context

#897 shipped a 1-token skip-mix re-ranking lane (SKMX joint table + PSIB
unconditioned fallback) that stays **dormant** (no `model/ids.toml` serving
row). #904 diagnosed its recorded LOWERING-FIDELITY GAP (deployed-lane
follow 41/87, below the predeclared 53/87 = 60% bar,
`docs/skipmix_lane_897_result.json`) and found it **breadth-bound**: 35/87
favorable pairs have at least one side's teacher target absent from the
engine's own served candidate list (`StepCandidates::ranked()`,
capped at `STEP_TOP_CANDIDATES = 8`) before any skip-mix scoring runs.

## Follow-up investigation (this issue's basis)

Before choosing a fix, every known width/selection lever was tested
empirically, off-serving, using `skipmix_gap_diagnostic_904.rs`'s
in-process compile step (no full pipeline recompile needed per experiment):

1. **`STEP_TOP_CANDIDATES` alone** (8 → 16/32/64): coverage ceiling
   plateaus at 52-53/87; deployed follow does not improve.
2. **All four compile/load-time candidate-breadth caps together**
   (`emission_entries`/`context_entries`/`exct_top_x`/`root_top_b`:
   64 → 4096, a 64x increase, plus `STEP_TOP_CANDIDATES` → 300): verified
   via direct debug instrumentation that the overrides genuinely reached
   the engine (not a wiring bug), and verified via the compiler's own
   `[emission-selection]` diagnostic that this was a real, substantial
   widening (held-out probability mass captured jumped from 1.57% to
   37.32%). Outcome: **unchanged** (52-53/87 ceiling throughout).
3. **`EmissionSelection::Probability`** (an existing, already-implemented
   but never-selected-in-production alternative to the shipped `Ratio`
   selection rule -- `score.rs:255-264`): tested directly at the default
   width. Verified as a huge, real quality improvement (probability mass
   captured jumped from 1.57% to 58.77% at the same E=64, full 100%
   overlap with a top-count selection vs 0.78% before). Outcome:
   **unchanged** (52/87 ceiling, 41/87 deployed follow -- identical to the
   shipped `Ratio` numbers).
4. **Skip-mix/psi-bag table rescue check**: of the 45 pair-sides currently
   missing their teacher target from the candidate list, **45/45 (100%)**
   already have that exact token present as a recorded co-occurrence
   partner in the SKMX/PSIB tables (`crates/uor-r4-graph-format/src/skipmix.rs`)
   -- verified directly against the real parsed tables, not simulated.

**Root cause**: `skipmix_adjusted_token`/`segment_argmax`
(`crates/uor-r4-api/src/engine.rs`) only **re-rank** the base engine's
already-decided `state.touched`/`ranked` candidate list -- they never
consult SKMX/PSIB as a candidate *source*. Since the base engine's own
candidate generation (region clustering + bigram/trigram context rows) is
structurally incapable of surfacing these 35 pairs' targets at any tested
width or selection rule, and the skip-mix lane can only choose among what
the base engine already proposed, the lane is architecturally unable to
close this portion of the gap no matter how its own tables are built or
tuned.

## Prototype-measured fix (off-serving, same harness)

Simulating candidate injection -- adding every SKMX/PSIB-known token as a
new candidate, not just re-ranking `ranked` -- combined with a
non-additive "unit-safe" combine rule (candidates with positive skip-mix
support always outrank candidates with none; among supported candidates,
rank by contribution magnitude; among unsupported candidates, fall back to
the real base `ScoreQ`; never sum the two incompatible scales, per #904
arm 3's already-validated design) projects:

| metric | value |
|---|---|
| coverage (target present in the candidate list) | 87/87 (100%) |
| deployed follow, realistic combine (this issue's target) | 58/87 (66.7%) |
| deployed follow, idealized upper bound (perfect scorer, same list) | 85/87 (97.7%) |
| deployed follow, today (shipped, #897/#904 record) | 41/87 (47.1%) |

58/87 clears the predeclared 60% bar (53/87) with room to spare. The
85/87 idealized ceiling shows further headroom if the combine rule itself
is improved later (out of scope here).

## Scope of this issue

Implement candidate injection for real in the skip-mix serving path only:

- `crates/uor-r4-api/src/engine.rs`: add an injected-candidate variant of
  the skip-mix decision combinator (discovers every token present in a
  relevant SKMX row `(t, last_token)` or PSIB row `t`, for the window's own
  unique tokens, in addition to the base `ranked` list; ranks by the
  unit-safe rule above), and wire it into
  `predict_decision_candidates_with_skipmix` and
  `predict_decision_candidates_with_skipmix_witness`.
- Preserve absent-section identity: with neither SKMX nor PSIB present,
  behavior must stay byte-identical to today (no candidates are ever
  discovered, so the combine reduces to the base engine's own argmax).
- Allocation-free (P-4 discipline, matching every other combinator in this
  module): no set/Vec is materialized for the expanded candidate list; a
  candidate discovered via more than one row is simply reconsidered
  (harmless -- deterministic, bounded work), not deduplicated.
- Unit tests mirroring the existing `skipmix_adjusted_token` test block,
  plus a confirmation test (analogous to #904's diagnostic, but exercising
  the real `predict_decision_candidates_with_skipmix` call path rather than
  a simulated harness) that reproduces the 58/87 (or better) projected
  number against the real engine.
- Out of scope: the shared base engine's own candidate generation
  (region clustering, context rows, `STEP_TOP_CANDIDATES`) and the #836
  segment lane -- neither is touched. No compile-time/wire-format change;
  no recompile of any released bundle is required. The lane remains
  dormant (no `model/ids.toml` serving row) -- this changes what the lane
  *would* do if turned on, not live production behavior today.


## Comments (0)

_No comments._

---
_Comment/review ids in this file: _

