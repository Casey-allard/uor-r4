# #774 — docs(#655-D): scope release-bundle packaging (D0 research + decision record)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T21:46:16Z
- **Updated:** 2026-08-17T22:09:06Z
- **Closed:** 2026-08-17T22:06:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/774
- **Merged:** 2026-08-17T22:06:52Z
- **Merge commit:** e95780d0e3169d605259f38b112c807ed46bf34f
- **Base / Head:** main / UOR-Foundation:docs/655-d0-packaging-design
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## What

Scopes #655-D ("produce + package one minimal evaluated serving bundle")
before any packaging code lands — docs only, no server/CLI changes.
Mirrors the research-then-decide convention already used for C1
(`docs/SERVING_MODEL_DISCOVERY.md` → `docs/serving_shared_loader_655_c1.md`
→ PR #773's additive sidecar verifier).

## Why

C1c (PR #773, merged) added an additive loader that verifies an optional
`release-bundle.json` sidecar against a resolved R4G1 bundle — but
nothing produces that sidecar yet. #655-D's own scope in the parent issue
is large (packaging, distribution, quality gating, release-artifact
integration); before writing any packaging code, this maps how a real
bundle is actually produced on disk today (the real `compile` →
`transformerless cover` → `transformerless score` CLI pipeline, confirmed
against a real local compiled bundle) against every field
`ReleaseBundleManifest` requires, and resolves the one ambiguous mapping
(`components.compile_report`) with a documented rationale rather than a
guess.

## What this decides

- `components.compile_report` maps to `graph-cover/cover_report.json`
  (the real pipeline's cover-stage report), not a literal
  `compile_report.json` rename — those are two separate cover
  implementations in this codebase, not the same file under two names.
- `uor-r4-api::compile` (confirmed still uncalled outside its own crate's
  tests) is not a shortcut for packaging: its stage-2 cover call and
  `work_dir` layout diverge from the real CLI pipeline, so routing D
  through it would mean unifying two divergent implementations as a
  packaging prerequisite — bigger and riskier than D needs. Packaging
  reads already-produced files off disk; it doesn't need to own how
  they're produced.
- Proposes D1 (pure digest-and-build-manifest helper, unit + `#[ignore]`d
  real-bundle tests) → D2 (thin CLI subcommand writing the sidecar) → D3
  (golden test confirming C1c's verifier returns `Some` end-to-end,
  handing off to the already-blocked C1d).
- Explicitly defers (not decided here): whether packaging should *gate*
  on `score_report.json`'s quality signal or just record it verbatim
  (D1-D3 record verbatim — no new gate).

## Non-goals

Does not change the default engine (#655-F, needs Casey's explicit
sign-off). Does not touch `ModelStore`/`import` (C1e). Does not change
what C1c verifies or how `resolve_loadable_compiled_bundle_with_authority`
resolves a bundle. Does not distribute a packaged bundle as part of any
release artifact yet.

## Verification

Docs-only change (one new file under `docs/`, zero code touched) — no
build/test/clippy signal to gain from running the full ladder, so it was
not run for this PR. Confirmed via `git diff --stat` that only
`docs/serving_release_packaging_655_d.md` changed.

## Next

Follow-up PR: D1 (the additive digest-and-build-manifest helper), per
this document's proposed slice sequence.


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

