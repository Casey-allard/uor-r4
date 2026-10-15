# #662 — perf(cover-sweep): cache cover-independent scoring preparation across points (#610)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-13T12:30:15Z
- **Updated:** 2026-10-09T14:17:05Z
- **Closed:** 2026-08-13T12:50:50Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/662
- **Merged:** 2026-08-13T12:50:49Z
- **Merge commit:** c1ee27f7cb3465a55eb453848f1fc8cd7fbf2dad
- **Base / Head:** main / UOR-Foundation:i610-cache-invariant-prep
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary
The nine-point sweep rebuilt the vocabulary and the whole-corpus **context n-gram rows** and **forward-anchor rows** for every cover point, though all three depend only on the corpus, teacher artifact, and fixed scorer — never on the induced cover. This builds them **once per run** and hands shared references to every point.

## What's included
- **`PreparedScoring`** — built once in `run_sweep`: `vocab` + `compile_context_rows` + `compile_forward_anchor_rows`, plus a fingerprint of the inputs (both κs, train count) and scorer config.
- **`run_point`** now takes `&PreparedScoring`, **validates the fingerprint** (a reuse across incompatible inputs aborts rather than scoring a point against the wrong tables), and binds the shared references instead of recomputing.
- **Cover-dependent work stays per point**, unchanged: induction, regions, transitions, emissions, artifact emission, and Gate C — whose scorers are rebuilt from *each point's own emitted artifact bytes*, so they are legitimately not hoistable.
- **Timing (schema 4 → 5):** the `timing` block gains `preparation_ms` (the one-time build); each point's `context_and_forward_rows_ms` now measures only the per-point reference binding (≈0). The moved cost stays **visible**, not a false measured-zero.

## Acceptance criteria
- **Shared preparation runs once per sweep, not once per point.** ✔ built before the loop in `run_sweep`.
- **Results and emitted artifact bytes remain identical.** ✔ measured — see verification.
- **κ/config validation prevents reuse across incompatible inputs.** ✔ `PreparedScoring::validate` panics on fingerprint mismatch.
- **Memory ownership explicit and bounded; no per-point mutation of shared state.** ✔ points receive `&` shared references; the context is immutable.
- **A regression test proves report metric parity on the fixture.** ✔ new parity test.

## Verification (measured)
- **Parity, real fixtures** — new `#[ignore]` test runs all nine grid points twice: once with the shared context, once with a context prepared *fresh per point* (the pre-#610 behavior), asserting emitted artifact bytes **and** full report rows are identical. **All 9 points byte-identical** (e.g. 15,440,996 B baseline; 15,500,516 B at `k0=16/gain=0.1`). `test result: ok. 1 passed` (1096 s).
- **Fingerprint guard** unit test (runs in CI, no fixtures): the fingerprint moves on any change to artifact κ, corpus κ, train count, or a scorer knob.
- **Local CI parity, all green:** `fmt --check`, `clippy --workspace --all-targets --all-features -D warnings`, `wasm32 --lib`, `cargo test --workspace`, `check-model` (R1), `audit-deferral` (R4 — nothing deferred), `repo-conformance` (R2/R3), `cargo deny` (R6).

## Breaking changes
None functional. Report schema bumps 4 → 5 (additive `preparation_ms`); all existing row fields and metrics are byte-identical.

## Follow-up
Deeper Gate C corpus-side table reuse would need graph-certify changes and must preserve the pre-registered Gate C instrument; **not attempted here** (the per-point Gate C scorers legitimately depend on each point's emitted artifact). Noted in the `PreparedScoring` doc comment.

Closes #610

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

