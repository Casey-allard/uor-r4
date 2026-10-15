# #779 — docs(#655-E): scope engine profiles design (E0 research + decision record)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-18T02:45:50Z
- **Updated:** 2026-08-18T03:07:17Z
- **Closed:** 2026-08-18T03:06:37Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/779
- **Merged:** 2026-08-18T03:06:37Z
- **Merge commit:** 99667c8558f7a95d3739904a012998d6eb94314b
- **Base / Head:** main / UOR-Foundation:docs/655-e0-profile-design
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Scopes #655-E ("Production vs Experimental engine profiles; migrate
`.uor-models/last_engine.txt`") — the last unstarted serving sub-track
now that C0-C1d and D0-D3 are all merged. Design and decision record
only, mirroring #655-C1b/D0's own shape — no server/CLI code changes.

## A genuine ambiguity, resolved

A research pass (subagent, read-only) found E's own spec text —
"Production profile = only the audited table-native transformerless
tier" — collides with the literal name of cascade **Tier 2**
(`transformerless`), an older, separately-pathed, explicitly-unaudited
system. Internal evidence (`docs/matrix_operation_census.md`'s audit
scope, `docs/inference_contract.md:11` calling r4g1 "the production
R⁴/R4G1 transformerless graph inference engine") pointed the other way:
"transformerless" is being used there as this project's overall
adjective, applied to **Tier 1 (`r4g1`)**. **Confirmed with Casey:
Production = `r4g1` specifically.**

## What's in the doc

- Current mechanics: exactly how `.uor-models/last_engine.txt`,
  `resolve_pinned_tier`, and `run_serving_cascade`'s tier-list
  construction work today, with file:line citations.
- Design decision: a new, separate `.uor-models/engine_profile.txt`
  (not a `last_engine.txt` rewrite) — the two files answer orthogonal
  questions.
- Design decision: enforcement filters the tier list itself; reuses the
  existing `declined_by_all` honest terminal — no new terminal state.
- Design decision: an explicit non-r4g1 `engine` request under
  `production` should be a typed decline, not a silent substitution; a
  stale persisted preference for a disallowed tier stays silently inert
  (same treatment `"r4g1"` already gets today).
- Proposed sequence: **E1** (additive, dormant `EngineProfile` type +
  persisted-read helper, zero behavior change, safe under the standing
  cadence with no check-in) then **E2** (actual enforcement wiring).

## A deliberate caution flagged in the doc

E2 is the slice that changes default runtime behavior for any install
without an `engine_profile.txt` (defaulting to `production`, i.e.
restricting reachable tiers). Even though Casey's own A-F decomposition
scoped this under E rather than F, its practical effect sits close
enough to **#655-F**'s "flip default" territory ("Casey sign-off
required, never autonomous") that the doc flags E2 as needing an
explicit Casey confirmation before merge — not just a design-doc
mention. E1 carries no such risk and needs no check-in.

## Verification

`cargo run -p xtask -- check-model` (R1) and `audit-deferral` (R4) both
pass. Docs-only change, no source edits — fmt/clippy/tests/wasm32 gates
are unaffected.

## Next

E1: additive `EngineProfile` enum + `engine_profile_preference()`
helper, unit-tested against synthetic fixtures, not wired into
`run_serving_cascade` yet.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

