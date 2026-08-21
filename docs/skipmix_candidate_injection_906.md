# Skip-mix candidate-injection fix (#906, follow-up to #897/#904)

Empirical record. Implements and confirms the fix for the #904 BREADTH-BOUND
finding (`docs/skipmix_gap_diagnostic_904.md`): the deployed skip-mix lane
(`predict_decision_candidates_with_skipmix`) only re-ranked the base
engine's already-decided candidate list and never consulted its own
SKMX/PSIB tables as a candidate *source*, capping deployed follow at
41/87 against a 52/87 (59.8%) breadth ceiling that itself falls below the
predeclared 53/87 (60%) fidelity bar.

Harness: `crates/uor-r4-api/tests/skipmix_candidate_injection_906.rs`
(`--ignored`; exercises the real, patched production code path, not a
simulation). Machine result:
`docs/skipmix_candidate_injection_906_result.json`.

## Investigation preceding the fix (off-serving, no production change)

Before implementing anything, every width/selection lever available to the
*base* engine's own candidate generation was tested empirically, per
Casey's "do this properly, do not prematurely dismiss" standing guidance:

1. **`STEP_TOP_CANDIDATES` alone** (8 -> 16/32/64): coverage ceiling
   plateaus at 52-53/87; deployed follow does not improve.
2. **All four compile/load-time candidate-breadth caps together**
   (`emission_entries`/`context_entries`/`exct_top_x`/`root_top_b`: 64 ->
   4096, a 64x widening, plus `STEP_TOP_CANDIDATES` -> 300): confirmed via
   direct debug instrumentation that the overrides genuinely reached the
   engine, and via the compiler's own `[emission-selection]` diagnostic
   that this was a real, substantial widening (held-out probability mass
   captured jumped from 1.57% to 37.32%). Outcome: **unchanged** (52-53/87
   ceiling throughout).
3. **`EmissionSelection::Probability`** (an existing, already-implemented,
   never-selected-in-production alternative ranking rule to the shipped
   `Ratio` selection -- `crates/uor-r4-graph-certify/src/score.rs:251-257`):
   tested directly at the default width. Verified as a huge, real quality
   improvement (probability mass captured jumped from 1.57% to 58.77% at
   the same E=64). Outcome: **unchanged** (52/87 ceiling, 41/87 deployed
   follow -- identical to the shipped `Ratio` numbers).
4. **Skip-mix/psi-bag table rescue check**: of the 45 pair-sides missing
   their teacher target from the candidate list, **45/45 (100%)** already
   have that exact token present as a recorded co-occurrence partner in the
   SKMX/PSIB tables (`crates/uor-r4-graph-format/src/skipmix.rs`) --
   verified directly against the real parsed tables, not simulated.

**Root cause**: the base engine's own candidate generation (region
clustering + bigram/trigram context rows) is structurally incapable of
surfacing these pairs' targets at any tested width or selection rule, and
the skip-mix lane -- being a pure re-ranker of `state.touched`/`ranked` --
could never close this portion of the gap regardless of how its own tables
were tuned, since it never looked at them as a candidate source.

## Prototype (off-serving) before implementing

Simulating candidate injection -- adding every SKMX/PSIB-known token as a
new candidate, not just re-ranking `ranked` -- combined with a
non-additive "unit-safe" combine rule (already validated in #904 arm 3:
candidates with positive skip-mix support always outrank candidates with
none; among supported candidates, rank by contribution magnitude; among
unsupported candidates, fall back to the real base `ScoreQ`; never sum the
two incompatible scales) projected 58/87 (66.7%) deployed follow, clearing
the 60% bar, with an 85/87 (97.7%) idealized ceiling if the combine rule
itself were later improved. Per Casey's sign-off ("Implement the real fix
now"), the real production change was then made.

## The fix

`crates/uor-r4-api/src/engine.rs`:

- `skipmix_adjusted_token`/`skipmix_lane_attribution` (re-rank-only) were
  removed and replaced by `skipmix_injected_argmax`/
  `skipmix_injected_lane_attribution`, which extend the candidate space to
  every token discoverable via a relevant SKMX row `(t, last_token)` or
  PSIB row `t`, for the window's own unique tokens, in addition to the base
  `ranked` list -- ranked by the same unit-safe rule #904 arm 3 validated.
- `predict_decision_candidates_with_skipmix` and
  `predict_decision_candidates_with_skipmix_witness` were rewired onto the
  new functions.
- Allocation-free (P-4 discipline, matching every other combinator in this
  module): no set/Vec is materialized for the expanded candidate space; a
  single-pass running-best accumulator considers each candidate as
  discovered, and a candidate reachable via more than one row is simply
  reconsidered (harmless, deterministic) rather than deduplicated.
- Absent-section identity preserved: with neither SKMX nor PSIB present, no
  candidates are ever discovered beyond `ranked`, so behavior is
  byte-identical to today.
- Unit tests updated/added (7 tests in `engine.rs`'s `skipmix` module,
  including a new `skipmix_injects_candidate_absent_from_ranked` that
  demonstrates the core new capability: a candidate not present in
  `ranked` at all is discovered and wins via table support alone).

## Result (confirmed against the real, patched production code path)

| quantity | value |
|---|---|
| reference favorable pairs (reproduced) | 87 / 87 expected |
| positions (served / abstained / unservable) | 174 / 0 / 0 |
| **deployed follow, NEW (candidate-injection combine)** | **58 / 87 (66.7%)** |
| deployed follow, OLD (re-rank-only, #897/#904 record) | 41 / 87 (47.1%) |
| predeclared 60% fidelity bar | 53 / 87 |
| clears fidelity bar | **yes** |

The real production measurement (58/87) matched the off-serving prototype
projection exactly on the first clean run -- no adjustment to the
pre-registered expectation was needed.

## Read

**FIXED, bar cleared.** Candidate injection closes the architectural gap
identified in #904: the skip-mix lane can now recover targets the base
engine's own candidate generation structurally cannot surface, using
tables the lane already ships and fits today. Deployed follow rises from
41/87 (47.1%, below bar) to 58/87 (66.7%, above the 53/87 bar), a 17-pair
absolute improvement, without widening any shared base-engine constant
(`STEP_TOP_CANDIDATES` and friends are untouched -- out of scope, per
#906's issue body) and without any wire-format or compile-time change to
already-released bundles.

The idealized reference-math ceiling on the same expanded candidate list
(85/87, 97.7%, measured off-serving in the prototype phase) shows headroom
remains if the combine rule itself is refined further -- out of scope for
this issue.

## Scope / conformance

Production code changed: `crates/uor-r4-api/src/engine.rs` only. The base
engine's own candidate generation (region clustering, context rows,
`STEP_TOP_CANDIDATES`) and the #836 segment lane are untouched. No
compile-time/wire-format change; no recompile of any released bundle is
required. The skip-mix lane remains **dormant** (no `model/ids.toml`
serving row) -- this changes what the lane *would* do if turned on, not
live production behavior today; `CONFORMANCE.md` is unaffected.

This record supersedes `docs/skipmix_gap_diagnostic_904.md`'s
"Implications" section's open question ("whether to attempt the wider
candidate-list change... is left to the maintainer") -- the narrower,
skip-mix-local candidate-injection fix (not a `STEP_TOP_CANDIDATES` widen)
was chosen and closes the gap.
