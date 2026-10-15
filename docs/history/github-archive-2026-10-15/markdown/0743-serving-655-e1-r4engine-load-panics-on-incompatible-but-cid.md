# #743 — serving/#655-E1: R4Engine::load panics on incompatible-but-CID-valid graph artifacts instead of failing closed

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-16T21:23:04Z
- **Updated:** 2026-08-16T22:51:22Z
- **Closed:** 2026-08-16T22:49:58Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/743
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

# Context

Found while running a coherence-measurement pass across the locally
compiled bundles under `.uor-models/compiled/` (see the comment on #655
dated this session: https://github.com/UOR-Foundation/uor-r4/issues/655#issuecomment-5309698590).
`r4 certify`'s C-serving row (`R4_CERTIFY_C_ONLY=1 R4_CERTIFY_SERVING_BUNDLE=<bundle> r4 certify`)
hard-panics instead of producing a measurement or a typed skip.

# Current evidence

- Bundle: `.uor-models/compiled/smollm2-135m-instruct`, compiled 2026-07-28
  (per file mtimes and `compile_report.json`). Its `corpus.meta`/`corpus.records`
  load fine (unlike the 360M/1.7B bundles, which correctly SKIP with
  "incomplete corpus" because their `corpus.meta` byte-24 completion flag is
  unset -- that path is working as intended and is not part of this issue).
- Exact panic, with `RUST_BACKTRACE=1`:
  ```
  thread 'main' panicked at crates/uor-r4-api/src/engine.rs:1095:10:
  engine load: scorer rebuild from the parsed and CID-verified graph bytes
  stack backtrace:
     ...
     4: <core::option::Option<uor_r4_graph_certify::score_runtime::GraphScorer>>::expect
     5: <uor_r4_api::engine::R4Engine>::load
               at ./crates/uor-r4-api/src/engine.rs:1095:10
     6: uor_r4_api::serving_eval::evaluate_serving_bundle
               at ./crates/uor-r4-api/src/serving_eval.rs:263:22
     7: r4::certify_serving_row
               at ./src/main.rs:801:15
  ```
- The offending line (`crates/uor-r4-api/src/engine.rs:1095`, current `main`
  as of `0b8654c0`):
  ```rust
  let scorer = GraphScorer::from_artifact(
      parts.graph,
      Some(parts.signature_artifact),
      root_top_b,
      exct_top_x,
  )
  .expect("engine load: scorer rebuild from the parsed and CID-verified graph bytes");
  ```
- `GraphScorer::from_artifact` (`crates/uor-r4-graph-certify/src/score_runtime.rs:884`)
  is a `-> Option<Self>` function built almost entirely out of `?`-early-return
  steps (`GraphView::parse(...).ok()?`, `view.verify_cids().ok()?`,
  `head()?`, `regions_from_view(&view)?`, `view.section(SectionId::EMIT)?`,
  several slice `.get(...)?` calls, `view.ngram_table().ok()?`, etc). Any one
  of these returning `None` after the container itself parses and its CIDs
  verify will trip the `.expect()` above. Which specific step fails for this
  bundle has **not** been isolated in this pass.
- Timing context: this bundle predates three merged R4G1-format-affecting
  PRs: `#732` (freeze PROV/1 provenance-roots section), `#733` (opt-in PROV/1
  producer propagation), and `#735` (packed R4G1 lowering for
  `MsaStructuredSelectorV1`, #643). Working theory: the on-disk graph's
  section layout no longer matches what `GraphScorer::from_artifact`
  currently assumes, even though the container-level parse/CID-verify still
  succeeds (CID verification checks byte integrity, not section-shape
  compatibility). **This theory is not confirmed** -- it wasn't tested
  against a bundle freshly recompiled with current `main`.
- `R4Engine::load` is not certify-only tooling: per
  `docs/SERVING_MODEL_DISCOVERY.md`, it backs the `#248` cascade's `r4g1`
  tier in the live HTTP server as well. A bundle that trips this panic in
  certify would very likely trip the identical panic loading in `r4 serve`.

# Problem

A serving-path function that's supposed to report typed outcomes for every
input state (this repo's D4 philosophy, formalized in #248 and #654: "never
fabricate prose or silently cross into a traditional/experimental engine")
instead calls `.expect()` and crashes the process when it receives a
graph artifact that is well-formed and CID-valid at the container level but
structurally incompatible with the scorer-rebuild logic. A crash is a worse
outcome than either a served answer or a typed decline: it takes the whole
process down (in `r4 serve`, this would kill the server on whatever request
or startup path touches this bundle) instead of failing one request/bundle
closed.

This is also actively blocking work: it prevented a clean C-serving
measurement of this bundle during the coherence investigation that spawned
this issue, and would block similar future measurement/certify work for
any stale-format bundle.

# Goal

Any bundle whose graph bytes parse and CID-verify but fail scorer
reconstruction for any reason must produce a typed decline
(`SourceUnavailable` or equivalent), never a panic -- consistently across
every caller of `R4Engine::load` (certify's C-row, the `#248` HTTP cascade,
and any other call site).

# Scope

- Change `GraphScorer::from_artifact`'s call site in
  `crates/uor-r4-api/src/engine.rs:1095` from `.expect(...)` to propagate a
  typed error (`SourceUnavailable` or the existing error type used
  elsewhere in `R4Engine::load`/`EngineParts` handling in that same file).
- Audit `crates/uor-r4-api/src/engine.rs` and
  `crates/uor-r4-graph-certify/src/score_runtime.rs` for any other
  `.expect()`/`.unwrap()`/`panic!` calls reachable from `R4Engine::load` on
  attacker-or-drift-controlled artifact bytes (not just this one site) and
  bring them under the same typed-decline discipline. Slice-parsing panics
  inside tightly-bounds-checked byte reads (the `try_into().expect("4
  bytes")` calls, for example, where the length is already asserted by a
  preceding bounds check) are lower priority / may be legitimate
  invariants -- use judgment and note which ones you left as invariants and
  why.
- If feasible without excessive scope, isolate exactly which sub-step
  inside `GraphScorer::from_artifact` returns `None` for the specific
  `smollm2-135m-instruct` bundle producing this panic today, and record
  that finding (this materially helps issue #(C) below and anyone hitting
  a similar stale-bundle failure in the future). If it turns out to be
  section-layout drift from #732/#733/#735, say so explicitly.
- Determine, empirically, whether a bundle freshly recompiled against
  current `main` loads without panicking (confirms the stale-artifact
  theory) or still fails (would mean this is a live regression, which
  changes the priority and blast radius of this issue significantly --
  escalate back to the maintainer if so).
- Add a regression test: a deliberately constructed graph artifact that
  parses and CID-verifies but has an incompatible/corrupted internal
  section (matching whatever failure mode was isolated above, or a
  synthetic one if isolation wasn't feasible) must produce a typed decline
  from `R4Engine::load`, not a panic. This is the same "planted falsifier"
  pattern already used elsewhere in this repo's verification ladder.
- Confirm the server's `#248` cascade path and any other caller of
  `R4Engine::load` handle the new typed error the same way (fail closed to
  the next cascade tier or a typed unavailable response, never panic,
  never silently serve something else without recording it).

# Non-goals

- Do not fix the underlying generation-quality problem (tracked separately,
  see the sibling issue on degenerate generation).
- Do not fix or relax the `instruction_eval_passed` quality gate (tracked
  separately, see the sibling issue on manifest/eval disagreement).
- Do not change artifact/era/κ semantics, silently recompile stale bundles,
  or auto-upgrade an old-format bundle to the new format as a side effect
  of this fix.
- Do not weaken CID verification or any other integrity check to make this
  bundle load "successfully" -- the correct outcome for a genuinely
  incompatible bundle is a typed decline, not a forced load.

# Dependencies and blockers

- None known. This is a narrowly scoped, self-contained robustness fix in
  `uor-r4-api`/`uor-r4-graph-certify`.

# Proposed implementation shape

Convert the `.expect()` to propagate `SourceUnavailable` (or whatever typed
error `R4Engine::load`'s signature already returns for other failure
modes in the same function -- follow the existing pattern rather than
inventing a new error type). Add the regression-test artifact and a unit
test asserting the typed-decline outcome. Re-run `r4 certify`'s C-row
against the `smollm2-135m-instruct` bundle and confirm it now either
produces a measurement or a clean typed SKIP instead of a panic.

# Acceptance criteria

- [ ] `R4Engine::load` never panics on any input that parses and CID-verifies
      at the container level, regardless of internal section-structure
      compatibility; it always returns a typed decline for such inputs.
- [ ] A regression test exercises this exact failure mode (or the closest
      reproducible equivalent) and asserts a typed decline, not a panic.
- [ ] `R4_CERTIFY_C_ONLY=1 R4_CERTIFY_SERVING_BUNDLE=.uor-models/compiled/smollm2-135m-instruct r4 certify`
      no longer panics.
- [ ] The root cause (stale-format drift vs. live regression) is determined
      and stated explicitly in the PR/issue closeout.
- [ ] Workspace fmt/clippy/tests pass; no unrelated behavior changes.

# Verification

- Reproduce the panic on current `main` before the fix (already done, see
  evidence above).
- After the fix, re-run the same command and confirm a typed outcome.
- Run the new regression test.
- Confirm the `#248` HTTP cascade path (or its existing test suite) also
  handles the same typed error without panicking.


## Comments (3)

### comment 5309741387 — 2026-08-16T21:23:18Z — Casey-allard

Sibling issues from the same investigation: #744 (quality-gate honesty), #745 (root-cause degenerate generation).

### comment 5309742828 — 2026-08-16T21:23:35Z — Casey-allard

Starting work on this one first: it's the most bounded fix of the three E1/E2/E3 issues, and unblocks clean `r4 certify` measurement of stale/incompatible bundles (needed to properly ground #745's root-cause work). #744 and #745 need more investigation before implementation and are left for a follow-up decision on sequencing.

### comment 5310077281 — 2026-08-16T22:51:21Z — Casey-allard

Fixed and merged in #746 (main@6505d72f).

**Root cause was not what I originally hypothesized.** I initially suspected R4G1 format/era drift (#732/#733/#735), and the fix direction we agreed on ("add an explicit format/era gate") was scoped around that theory. Empirical instrumentation of `GraphScorer::from_artifact` showed that's not it: `format_version` has stayed at 0.0 the whole time, and all three of those PRs were verified additive/non-breaking. The actual cause is a genuine **data-pairing mismatch**: for the `smollm2-135m-instruct` bundle on disk, the graph's `HEAD.teacher_cid` doesn't match `blake3(signature_artifact)` -- the two files are independently sourced and had drifted out of pairing (consistent with a bundle directory reused across multiple compile runs).

**Fix**, same narrow-gate spirit we agreed on, retargeted at the confirmed cause: in `R4Engine::load`, when the graph carries an `EXCT` section (the only case `from_artifact` consults `HEAD.teacher_cid`), check the pairing explicitly before parsing `signature_artifact`, and return a typed `SourceUnavailable` decline on mismatch instead of falling through into `from_artifact`'s self-produced-defect panic. Other panic conditions inside `from_artifact` are untouched -- they remain correct under R5/#510 for bytes that really are self-produced, same-process, and therefore a code defect if they fail.

One implementation note worth recording: my first attempt added a new `TeacherPairingError` enum in `uor-r4-graph-certify`. `cargo run -p xtask -- audit-limits` (the R5 gate) correctly rejected it -- only `NotAProduct`/`ObservedBound`/`KappaError`/`SourceUnavailable` are sanctioned in a shipped-crate `Result` signature. Reworked to an inline check in `engine.rs` reusing `SourceUnavailable`; no new error type anywhere.

Verified: fmt, clippy -D warnings, `audit-limits` (R5), full `uor-r4-api` + `uor-r4-graph-certify` test suites, `xtask check-model` (CM-01), `check_claim_wording.py`, and a direct re-run of the original repro against the real bundle -- now a clean typed decline instead of a panic.

Closing this one out; moving on to the rest of the coherence-sweep follow-up work (#744, #745) and the broader audit/documentation/roadmap request.


---
_Comment/review ids in this file: 5309741387, 5309742828, 5310077281_

