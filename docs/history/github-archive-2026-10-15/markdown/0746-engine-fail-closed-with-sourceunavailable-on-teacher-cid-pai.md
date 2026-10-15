# #746 — engine: fail closed with SourceUnavailable on teacher_cid pairing mismatch (#743)

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-16T22:28:24Z
- **Updated:** 2026-10-09T14:17:19Z
- **Closed:** 2026-08-16T22:49:57Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/746
- **Merged:** 2026-08-16T22:49:57Z
- **Merge commit:** 6505d72f741833f79ebb666ea1f60dd750f00492
- **Base / Head:** main / UOR-Foundation:issue-743-engine-load-typed-decline
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`R4Engine::load` panicked instead of failing closed when a persisted
bundle's graph (`score.r4g1`) and teacher/signature artifact were no
longer paired -- e.g. a bundle directory reused across multiple compile
runs, so the two files no longer belong to the same generation.
`GraphScorer::from_artifact` is documented as total for self-produced,
same-process bytes and panics on the premise that any failure there is
a code defect (R5, #510); that premise doesn't hold once bytes are
reloaded from disk on the serving path.

**Root cause correction:** I originally hypothesized format/era drift
(#732/#733/#735). Empirical instrumentation showed that's not it --
`format_version` has stayed at 0.0 throughout and all three PRs were
verified non-breaking. The actual cause is a genuine data-pairing
mismatch: the graph's `HEAD.teacher_cid` doesn't match
`blake3(signature_artifact)` for the `smollm2-135m-instruct` bundle on
disk. This PR targets that empirically-confirmed cause, in the same
narrow-gate spirit already discussed on the issue (leave other panic
conditions alone; add an explicit gate for the specific detectable,
non-defect condition).

## Fix

In `R4Engine::load`, when the parsed graph carries an `EXCT` section
(the only case `from_artifact` consults `HEAD.teacher_cid`), check the
teacher_cid pairing explicitly before parsing `signature_artifact`, and
return a typed `SourceUnavailable` decline on mismatch. No new error
type: an earlier attempt added a `TeacherPairingError` enum in
`uor-r4-graph-certify`, but `cargo run -p xtask -- audit-limits` (R5)
correctly rejected it -- only `NotAProduct`, `ObservedBound`,
`KappaError`, and `SourceUnavailable` may appear in a `Result`
signature anywhere in a shipped crate. The check is inlined in
`engine.rs` instead, reusing `SourceUnavailable`.

Other panic conditions inside `from_artifact` are untouched and remain
intentional per the R5/#510 "self-produced defect" design -- this PR
closes the gap for the one condition that's provably not a code defect
(pairing drift between two independently-sourced files on disk), which
is also the exact condition #743 reported and reproduced.

## Verification

- `cargo fmt --check`
- `cargo run -p xtask -- audit-limits` -- no shipped crate returns an unsanctioned error (R5)
- `cargo clippy --all-targets -- -D warnings`
- `cargo test -p uor-r4-api --test api` -- 19 passed (2 new tests covering mismatch-declines and matched-pairing-clears-the-gate)
- `cargo test -p uor-r4-graph-certify -p uor-r4-api` -- full suite, all green
- `cargo run -q -p xtask -- check-model` -- CM-01 conformance holds
- `scripts/check_claim_wording.py` -- claim-wording gate passes
- Direct re-run of the original repro against the real bundle:
  ```
  R4_CERTIFY_C_ONLY=1 R4_CERTIFY_SERVING_BUNDLE=.../smollm2-135m-instruct ./target/debug/r4 certify
  ```
  now produces a clean typed decline (`teacher artifact mismatch: ...`)
  instead of a panic.

Closes #743

🤖 Generated with Claude (Cowork)

## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

