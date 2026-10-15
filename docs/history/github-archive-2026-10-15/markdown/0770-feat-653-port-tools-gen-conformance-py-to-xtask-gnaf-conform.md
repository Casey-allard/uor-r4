# #770 — feat(#653): port Tools/gen_conformance.py to xtask gnaf-conformance

- **Type:** Pull request
- **State:** closed
- **Author:** @Casey-allard
- **Created:** 2026-08-17T19:11:05Z
- **Updated:** 2026-08-17T19:30:46Z
- **Closed:** 2026-08-17T19:27:52Z
- **URL:** https://github.com/UOR-Foundation/uor-r4/pull/770
- **Merged:** 2026-08-17T19:27:52Z
- **Merge commit:** eb75cd5083ea6908f6fdb341f9ee24f8a775c50d
- **Base / Head:** main / UOR-Foundation:i653-gen-conformance
- **Reviews / inline review comments:** 0 / 0
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

## Summary

`docs/gnaf_integration_653.md` (#653 phase-2 follow-up) recommends
porting each `Tools/*.py` check to an r4-native equivalent rather than
vendoring Python into the Rust workspace. This is the sixth of the 10
`Tools/*.py` scripts, and the last of the 6 that don't invoke
`lean`/`lake` (directly or transitively) -- the previous 5
(`gnaf-firewall`, `gnaf-root`, `gnaf-scan`, `gnaf-release-path`,
`gnaf-manifest`) landed in #767. The remaining 4 (`axioms.py`,
`required.py`, `gate.py`, `mutation.py`) all invoke the Lean toolchain
and stay out of scope.

### `gnaf-conformance`

Ports `Tools/gen_conformance.py` (SPEC 17.3): renders `CONFORMANCE.md`
deterministically from `model/claims.json` -- a live Lean
module/line/theorem-line inventory under `WasmGemmGnaf/`, the claims
table, each `formalProof` claim's axiom closure, refuted framings, and
outstanding obligations.

Byte-for-byte parity with the Python source, including its `f"{n:,}"`
thousands-separated counts (hand-rolled digit grouping) and its two
distinct truncation rules: the claims table escapes `|` then cuts to
107 chars + `"..."` past a 110-char threshold, while outstanding
obligations take a bare first-100-char cut with no ellipsis. Both
operate on Unicode scalar values, matched with `.chars()` rather than
byte indexing. The theorem count reproduces the Python source's own
crude heuristic as-is (a `startswith("theorem ")` line count due to
Python operator precedence collapsing away the `"instance "` half of
the original condition) -- not a real theorem count from Lean
compilation, kept for parity rather than "fixed."

**Unlike** `gnaf-release-path`/`gnaf-manifest` (both currently red
against the real tree and deliberately excluded from `validate`), the
real committed `CONFORMANCE.md` **is** currently current against
`model/claims.json`: verified via cross-tool round trip against a
scratch copy of the real vendored tree -- Python-written
`CONFORMANCE.md` diffed byte-identical against the Rust writer's
output, and `cargo xtask gnaf-conformance` (no `--write`) reports the
real checked-in file as already current. So this port **is** wired
into `validate`, joining `gnaf-firewall`/`gnaf-root`/`gnaf-scan` as a
green gate.

## Remaining #653 scope (not this PR)

Per `docs/gnaf_integration_653.md`'s integration matrix: the 4
Lean-toolchain `Tools/*.py` scripts (blocked on CI policy), Atlas
incremental-rebuild discipline in xtask, and the CI policy decision on
Lean workflows. This PR does not close #653.

## Test plan

- [x] `cargo test -p xtask gnaf --offline` -- 28/28 pass (5 new
      `gnaf_conformance` tests)
- [x] `cargo xtask validate` -- passes with `gnaf-conformance` wired in
- [x] Cross-tool round trip on a scratch copy of the real vendored
      tree: Python-written `CONFORMANCE.md` byte-identical to the Rust
      writer's output (throwaway verification, not checked in)
- [x] `cargo fmt --check` -- clean
- [x] `cargo clippy --workspace --all-targets --all-features --offline -- -D warnings` -- clean
- [x] `cargo test --workspace --offline` -- 1890 passed, 0 failed


## Timeline (0 entries; chronological: conversation comments, reviews, review comments)

_No comments or reviews._

---
_Comment/review ids in this file: _

